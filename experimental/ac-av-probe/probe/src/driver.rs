//! Request driver: the probe's request → pipeline → AV outcome loop
//! (PLAN-798 §5.5/§5.6).
//!
//! One `request()` spans: front-door admission (size + nesting over the
//! REAL token streams — never just brace counting), the landed scalar
//! frontend, per-function cache lookup keyed on canonical HIR body
//! fingerprints (hits skip that unit's MIR lowering AND ABC emission),
//! MIR lowering + verification, ABC emission, unit/manifest persistence,
//! closure assembly + whole-closure verification, AV execution. A cache
//! hit STILL executes AV — nothing here caches PASS or run results.

use crate::cache::{
    abc_unit_key, assemble_closure, caps_fingerprint, decode_sidecar, encode_sidecar,
    get_enveloped_opt, manifest_key, merge_spans, open_store, payload_fingerprint, put_enveloped,
    sidecar_key, split_units, Counters, EnvelopeIdentity, ModuleManifest, SemaRefSer, UnitLink,
};
use crate::hir_adapter::{lower_checked, AdapterError};
use crate::observations::{Obs, SpanRec};
use auto_ac_prototype::source::compile_scalar_source;
use auto_ac_prototype::verify::CheckedModule;
use auto_av_core::contract::{
    fnv1a64, ExecOutcome, ResourceKind, SpanInfo, Value, MAX_FRONTEND_NESTING, MAX_INPUT_BYTES,
};
use auto_av_core::mir::{MirModule, OwnerScope, TraceEvent};
use auto_av_core::{
    abc_verify, decode_abc, encode_abc, execute, verify_mir, CancelFlag, ExecLimits, LoadedModule,
};
use serde::{Deserialize, Serialize};

/// One diagnosed pipeline failure: stage + code + message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StageDiag {
    pub stage: String,
    pub code: String,
    pub message: String,
}

/// Terminal state of one request.
#[derive(Clone, Debug)]
pub struct RequestOutcome {
    /// Diagnosed rejection before execution (admission/unsupported/infra).
    pub diag: Option<StageDiag>,
    /// AV execution outcome when the pipeline reached execution.
    pub av: Option<ExecOutcome>,
    pub trace: Vec<TraceEvent>,
    pub spans: Vec<SpanRec>,
    pub counters: Counters,
    pub source_hash: String,
    pub snapshot: u64,
    pub manifest: Option<ModuleManifest>,
    pub wall_ms: f64,
}

impl RequestOutcome {
    /// The i32 result when the request executed successfully.
    pub fn value_i32(&self) -> Option<i32> {
        match &self.av {
            Some(ExecOutcome::Success {
                value: Value::I32(v),
                ..
            }) => Some(*v),
            _ => None,
        }
    }

    pub fn ok(&self) -> bool {
        self.diag.is_none() && matches!(self.av, Some(ExecOutcome::Success { .. }))
    }
}

pub struct Driver {
    store: Option<ac_artifact_probe::store::ManagedStore>,
    project: String,
    owner_override: Option<String>,
}

impl Driver {
    /// A driver WITHOUT a store: pure pipeline (no cache), for admission /
    /// semantics tiers and cross-checks.
    pub fn ephemeral() -> Driver {
        Driver {
            store: None,
            project: String::new(),
            owner_override: None,
        }
    }

    /// A driver over a real managed store root (created when absent).
    pub fn with_store(root: &std::path::Path, project: &str) -> Result<Driver, String> {
        let store = open_store(root).map_err(|e| e.to_string())?;
        Ok(Driver {
            store: Some(store),
            project: project.to_string(),
            owner_override: None,
        })
    }

    pub fn set_owner(&mut self, owner: Option<String>) {
        self.owner_override = owner;
    }

    /// One full request. `entry` selects the zero-param i32 entry function.
    pub fn request(&mut self, src: &str, entry: &str, caps: &[String]) -> RequestOutcome {
        let mut obs = Obs::new();
        let snapshot = fnv1a64(src.as_bytes());
        let source_hash = format!("{:016x}", snapshot);
        let (diag, av, trace, manifest) = self.request_inner(src, entry, caps, snapshot, &mut obs);
        RequestOutcome {
            diag,
            av,
            trace,
            spans: obs.spans().to_vec(),
            counters: obs.counters().clone(),
            source_hash,
            snapshot,
            manifest,
            wall_ms: obs.request_wall().as_secs_f64() * 1000.0,
        }
    }

    fn request_inner(
        &mut self,
        src: &str,
        entry: &str,
        caps: &[String],
        snapshot: u64,
        obs: &mut Obs,
    ) -> (
        Option<StageDiag>,
        Option<ExecOutcome>,
        Vec<TraceEvent>,
        Option<ModuleManifest>,
    ) {
        obs.set_counter("requests", 1);
        // ---- front-door admission --------------------------------------
        if let Err(d) = front_door_admit(src) {
            obs.counter("admission_reject", 1);
            return (Some(d), None, vec![], None);
        }
        // ---- frontend ---------------------------------------------------
        let compiled = obs.span("frontend", Some("request"), |o| {
            compile_scalar_source(src, "probe.at", caps).map_err(|ds| {
                let d = ds.first().cloned();
                o.counter("frontend_reject", 1);
                StageDiag {
                    stage: "frontend".to_string(),
                    code: d
                        .as_ref()
                        .map(|x| x.code.clone())
                        .unwrap_or_else(|| "source.unknown".into()),
                    message: d
                        .map(|x| x.message.clone())
                        .unwrap_or_else(|| "frontend failed without a diagnostic".into()),
                }
            })
        });
        let compiled = match compiled {
            Ok(c) => c,
            Err(d) => return (Some(d), None, vec![], None),
        };
        let checked: &CheckedModule = &compiled.checked;
        let module = &checked.bundle().modules[0];
        let owner = self
            .owner_override
            .clone()
            .unwrap_or_else(|| module.id_text.clone());
        let caps_fp = caps_fingerprint(caps);

        // Function bodies in def order, with canonical fingerprints.
        struct FnInfo {
            name: String,
            sig: String,
            fp: String,
            /// Shared-domain logical id (796 identity; V-13 axis).
            logical: String,
            /// Canonical HIR dump — retained so the manifest round can mint
            /// the 796 DeclRef and run the REAL association admission over
            /// the payload this request actually persisted/loaded
            /// (F798-R3-02).
            dump: Vec<u8>,
            abc_hit: Option<Vec<u8>>,
            /// Content fingerprint of the payload bytes this request binds
            /// to the unit (computed where the bytes are already in hand:
            /// the ABC hit lookup or the emission), never via a re-read.
            payload_fp: Option<String>,
        }
        let mut infos: Vec<FnInfo> = Vec::new();
        for d in module.defs.iter() {
            if let auto_ac_prototype::hir::DefKind::Function { ty, body, .. } = &d.kind {
                let dump = crate::cache::canonical_hir_function(module, *body);
                infos.push(FnInfo {
                    name: d.id_text.clone(),
                    sig: sig_string(module, *ty),
                    // T-06: the content axis rides the landed 796
                    // ContentVersion domain (order-independent identity);
                    // the local FNV fingerprint snapshot is retired.
                    fp: crate::semrefs::content_version_of(&dump),
                    logical: crate::semrefs::unit_logical_id(
                        &crate::semrefs::av_package(),
                        &owner,
                        d.id_text.as_str(),
                    ),
                    dump,
                    abc_hit: None,
                    payload_fp: None,
                });
            }
        }
        obs.set_counter("units", infos.len() as u64);

        // ---- per-unit cache lookup (A09 ABC layer first) ------------------
        let mut hits = 0u64;
        let mut staging_now: usize = 0;
        let mut staging_peak: usize = 0;
        let mut verify_analysis_bytes: usize = 0;
        if let Some(store) = self.store.as_ref() {
            let expect = EnvelopeIdentity::abc(&owner);
            for info in infos.iter_mut() {
                let key = abc_unit_key(&owner, &info.name, &info.fp, &caps_fp);
                match get_enveloped_opt(store, &key, "A09", &expect) {
                    Ok(Some(payload)) => {
                        hits += 1;
                        staging_now += payload.len();
                        staging_peak = staging_peak.max(staging_now);
                        info.payload_fp = Some(payload_fingerprint(&payload));
                        info.abc_hit = Some(payload);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        // A live-but-wrong-identity artifact is a controlled
                        // rejection, never a silent rebuild-over.
                        return (
                            Some(StageDiag {
                                stage: "cache".to_string(),
                                code: "cache.identity".to_string(),
                                message: format!("unit `{}`: {}", info.name, e),
                            }),
                            None,
                            vec![],
                            None,
                        );
                    }
                }
            }
        }
        let abc_misses: Vec<usize> = infos
            .iter()
            .enumerate()
            .filter(|(_, i)| i.abc_hit.is_none())
            .map(|(n, _)| n)
            .collect();
        obs.set_counter("cache_hit_abc", hits);
        obs.set_counter("cache_miss_abc", abc_misses.len() as u64);

        // ---- A08 MIR layer: restore-verify-reuse before any lowering -------
        // A MIR hit re-enters verify_mir on restore (never trusted) and
        // feeds ABC emission without re-lowering (F798-R1-06 wiring).
        let mir_owner = OwnerScope {
            schema: auto_av_core::contract::MIR_SCHEMA.to_string(),
            module: owner.clone(),
        };
        let mut mir_restored = 0u64;
        let mut mir_units: Vec<(usize, MirModule)> = Vec::new();
        let mut still_missing: Vec<usize> = abc_misses.clone();
        if let Some(store) = self.store.as_ref() {
            let expect = EnvelopeIdentity::mir(&owner, 0);
            let mut remaining: Vec<usize> = Vec::new();
            for idx in abc_misses.iter() {
                let idx = *idx;
                let info = &infos[idx];
                let key = crate::cache::mir_unit_key(&owner, &info.name, &info.fp, &caps_fp);
                match get_enveloped_opt(store, &key, "A08", &expect) {
                    Ok(Some(payload)) => {
                        staging_now += payload.len();
                        staging_peak = staging_peak.max(staging_now);
                        let decoded = match decode_abc(&payload) {
                            Ok(d) => d,
                            Err(e) => {
                                return (
                                    Some(StageDiag {
                                        stage: "mir-load".to_string(),
                                        code: e.code(),
                                        message: format!("unit `{}`: {:?}", info.name, e),
                                    }),
                                    None,
                                    vec![],
                                    None,
                                )
                            }
                        };
                        let mut unit = decoded.module;
                        unit.owner = mir_owner.clone();
                        for f in unit.funcs.iter_mut() {
                            f.owner = mir_owner.clone();
                        }
                        // F798-R3-08: the restored MIR unit is a real live
                        // allocation — the staging ledger carries its heap
                        // estimate, not just the wire length.
                        staging_now += unit.heap_estimate();
                        staging_peak = staging_peak.max(staging_now);
                        // MIR restore MUST re-enter the MIR verifier.
                        let verified =
                            obs.span("mir_restore_verify", Some("request"), |_| verify_mir(&unit));
                        match verified {
                            Err(e) => {
                                return (
                                    Some(StageDiag {
                                        stage: "mir-verify".to_string(),
                                        code: e.code(),
                                        message: format!(
                                            "unit `{}` MIR restore re-verification failed",
                                            info.name
                                        ),
                                    }),
                                    None,
                                    vec![],
                                    None,
                                )
                            }
                            Ok(checked) => {
                                verify_analysis_bytes += checked.stats().analysis_bytes as usize;
                            }
                        }
                        mir_restored += 1;
                        mir_units.push((idx, unit));
                    }
                    Ok(None) => remaining.push(idx),
                    Err(e) => {
                        return (
                            Some(StageDiag {
                                stage: "cache".to_string(),
                                code: "cache.identity".to_string(),
                                message: format!("MIR unit `{}`: {}", info.name, e),
                            }),
                            None,
                            vec![],
                            None,
                        );
                    }
                }
            }
            still_missing = remaining;
        }
        obs.set_counter("mir_units_restored", mir_restored);

        // ---- remaining misses: ONE whole-module lower, then per-unit persist --
        let mut fresh_units: Vec<MirModule> = Vec::new();
        let mut mir_persisted = 0u64;
        let mut abc_emitted = 0u64;
        // F798-R3-03: spans of EVERY function of the current revision, from
        // any lowering this request performed — hit units re-bind their
        // diagnostics track from here instead of silently going spanless.
        let mut lowered_spans: std::collections::BTreeMap<String, Vec<(u32, SpanInfo)>> =
            std::collections::BTreeMap::new();
        if !still_missing.is_empty() {
            // Real whole-module lowering — counted as exactly ONE call per
            // request, never per-unit (review F798-R1-06).
            let m = match obs.span("mir_lower", Some("request"), |o| {
                lower_checked(checked, entry, snapshot).map_err(|e: AdapterError| {
                    o.counter("lower_reject", 1);
                    StageDiag {
                        stage: "mir-lower".to_string(),
                        code: e.code(),
                        message: format!("{:?}", e),
                    }
                })
            }) {
                Ok(m) => m,
                Err(d) => return (Some(d), None, vec![], None),
            };
            obs.counter("mir_lower_modules", 1);
            match obs.span("mir_verify", Some("request"), |o| {
                verify_mir(&m).map_err(|e| {
                    o.counter("mir_verify_reject", 1);
                    StageDiag {
                        stage: "mir-verify".to_string(),
                        code: e.code(),
                        message: format!("{:?}", e),
                    }
                })
            }) {
                Ok(checked) => {
                    verify_analysis_bytes += checked.stats().analysis_bytes as usize;
                }
                Err(d) => return (Some(d), None, vec![], None),
            }
            let units = match obs.span("mir_split", Some("request"), |_| {
                split_units(&m).map_err(|e| StageDiag {
                    stage: "mir-split".to_string(),
                    code: "mir.split".to_string(),
                    message: e,
                })
            }) {
                Ok(u) => u,
                Err(d) => return (Some(d), None, vec![], None),
            };
            // The lowered MIR is a live composite allocation (F798-R3-08).
            staging_now += m.heap_estimate();
            staging_peak = staging_peak.max(staging_now);
            // Sidecars under the CURRENT snapshot for the units THIS
            // request persists (no ABC hit): position drift on a later hit
            // re-binds instead of losing the source mapping (F798-R3-03).
            // Units that merely HIT keep binding in-memory from
            // `lowered_spans` below — no store write per hit (writing
            // every unit's sidecar here regressed the warm-edit window by
            // >1s of store puts). Write failures propagate — diagnostics
            // provenance is not best-effort.
            {
                let missing_names: std::collections::BTreeSet<&str> = still_missing
                    .iter()
                    .map(|idx| infos[*idx].name.as_str())
                    .collect();
                for u in units.iter() {
                    let name = u.funcs[0].name.clone();
                    if let Some(store) = self.store.as_ref() {
                        if missing_names.contains(name.as_str()) {
                            let skey = sidecar_key(&owner, &name, snapshot);
                            let sexpect = EnvelopeIdentity::sidecar(&owner, snapshot);
                            let spayload = encode_sidecar(&u.funcs[0].spans);
                            if let Err(e) = put_enveloped(
                                store,
                                &skey,
                                "A08",
                                &sexpect,
                                &spayload,
                                vec![],
                                Some(&self.project),
                            ) {
                                return (
                                    Some(StageDiag {
                                        stage: "cache".to_string(),
                                        code: "cache.put".to_string(),
                                        message: format!("sidecar `{}`: {}", name, e),
                                    }),
                                    None,
                                    vec![],
                                    None,
                                );
                            }
                        }
                    }
                    lowered_spans.insert(name, u.funcs[0].spans.clone());
                }
            }
            let by_fp: std::collections::BTreeMap<&str, &MirModule> = units
                .iter()
                .map(|u| (u.funcs[0].name.as_str(), u))
                .collect();
            for idx in still_missing.iter() {
                let idx = *idx;
                let info = &infos[idx];
                let unit = match by_fp.get(info.name.as_str()) {
                    Some(u) => *u,
                    None => {
                        return (
                            Some(StageDiag {
                                stage: "mir-split".to_string(),
                                code: "mir.split.missing".to_string(),
                                message: format!("unit `{}` absent from split", info.name),
                            }),
                            None,
                            vec![],
                            None,
                        )
                    }
                };
                // Persist the MIR unit (code content, spans stripped) under
                // the A08 MirUnit envelope. (Sidecars for ALL units under
                // the current snapshot were written above — F798-R3-03.)
                let mut spanless = (*unit).clone();
                std::mem::take(&mut spanless.funcs[0].spans);
                if let Some(store) = self.store.as_ref() {
                    let mkey = crate::cache::mir_unit_key(&owner, &info.name, &info.fp, &caps_fp);
                    let mbytes = match encode_abc(&spanless) {
                        Ok(b) => b,
                        Err(e) => {
                            return (
                                Some(StageDiag {
                                    stage: "mir-emit".to_string(),
                                    code: e.code(),
                                    message: format!("unit `{}`: {:?}", info.name, e),
                                }),
                                None,
                                vec![],
                                None,
                            )
                        }
                    };
                    staging_now += spanless.heap_estimate() + mbytes.total_len;
                    staging_peak = staging_peak.max(staging_now);
                    let mexpect = EnvelopeIdentity::mir(&owner, 0);
                    if let Err(e) = put_enveloped(
                        store,
                        &mkey,
                        "A08",
                        &mexpect,
                        &mbytes.to_flat(),
                        vec![],
                        Some(&self.project),
                    ) {
                        return (
                            Some(StageDiag {
                                stage: "cache".to_string(),
                                code: "cache.put".to_string(),
                                message: format!("MIR unit `{}`: {}", info.name, e),
                            }),
                            None,
                            vec![],
                            None,
                        );
                    }
                    mir_persisted += 1;
                }
                fresh_units.push((*unit).clone());
            }
        }

        // ---- ABC emission for every unit without an ABC hit ----------------
        // (from restored MIR units first, then the freshly split ones).
        {
            let mut emit_sources: Vec<MirModule> =
                mir_units.iter().map(|(_, u)| u.clone()).collect();
            emit_sources.extend(fresh_units.iter().cloned());
            let by_name: std::collections::BTreeMap<&str, &MirModule> = emit_sources
                .iter()
                .map(|u| (u.funcs[0].name.as_str(), u))
                .collect();
            for idx in abc_misses.iter() {
                let idx = *idx;
                let name = infos[idx].name.clone();
                let info = &infos[idx];
                let unit = match by_name.get(name.as_str()) {
                    Some(u) => *u,
                    None => {
                        return (
                            Some(StageDiag {
                                stage: "abc-emit".to_string(),
                                code: "abc.emit.missing-unit".to_string(),
                                message: format!("no MIR source for unit `{}`", name),
                            }),
                            None,
                            vec![],
                            None,
                        )
                    }
                };
                let mut spanless = (*unit).clone();
                std::mem::take(&mut spanless.funcs[0].spans);
                let bytes = match encode_abc(&spanless) {
                    Ok(b) => b,
                    Err(e) => {
                        return (
                            Some(StageDiag {
                                stage: "abc-emit".to_string(),
                                code: e.code(),
                                message: format!("{:?}", e),
                            }),
                            None,
                            vec![],
                            None,
                        )
                    }
                };
                staging_now += spanless.heap_estimate() + bytes.total_len;
                staging_peak = staging_peak.max(staging_now);
                let payload_fp = payload_fingerprint(&bytes.to_flat());
                abc_emitted += 1;
                if let Some(store) = self.store.as_ref() {
                    let key = abc_unit_key(&owner, &name, &info.fp, &caps_fp);
                    let expect = EnvelopeIdentity::abc(&owner);
                    if let Err(e) = put_enveloped(
                        store,
                        &key,
                        "A09",
                        &expect,
                        &bytes.to_flat(),
                        vec![],
                        Some(&self.project),
                    ) {
                        return (
                            Some(StageDiag {
                                stage: "cache".to_string(),
                                code: "cache.put".to_string(),
                                message: format!("unit `{}`: {}", name, e),
                            }),
                            None,
                            vec![],
                            None,
                        );
                    }
                }
                // (the immutable borrow of `infos[idx]` ended above)
                infos[idx].payload_fp = Some(payload_fp);
            }
        }
        obs.set_counter("mir_units_persisted", mir_persisted);
        obs.set_counter("abc_units_emitted", abc_emitted);
        obs.set_counter("peak_staging_bytes", staging_peak as u64);
        // The restored-MIR units participate in the closure as well.
        let mut fresh_units = fresh_units;
        fresh_units.extend(mir_units.iter().map(|(_, u)| u.clone()));

        // ---- closure assembly -------------------------------------------
        // Hits: decode + verify each restored unit (never trusted).
        let mut decoded_units: Vec<MirModule> = Vec::new();
        let owner_scope = OwnerScope {
            schema: auto_av_core::contract::ABC_SCHEMA.to_string(),
            module: owner.clone(),
        };
        for info in infos.iter().filter(|i| i.abc_hit.is_some()) {
            let payload = info.abc_hit.as_ref().expect("filtered");
            let decoded = match obs.span("abc_load", Some("request"), |_| decode_abc(payload)) {
                Ok(d) => d,
                Err(e) => {
                    return (
                        Some(StageDiag {
                            stage: "abc-load".to_string(),
                            code: e.code(),
                            message: format!("unit `{}`: {:?}", info.name, e),
                        }),
                        None,
                        vec![],
                        None,
                    )
                }
            };
            match obs.span("abc_verify", Some("request"), |_| {
                abc_verify::verify_abc(decoded_clone(&decoded), &owner_scope)
            }) {
                Ok(loaded) => {
                    verify_analysis_bytes += loaded.stats().analysis_bytes as usize;
                }
                Err(e) => {
                    return (
                        Some(StageDiag {
                            stage: "abc-verify".to_string(),
                            code: format!("{}", e),
                            message: format!("unit `{}` re-verification failed", info.name),
                        }),
                        None,
                        vec![],
                        None,
                    )
                }
            }
            let mut modu = decoded.module;
            modu.owner = owner_scope.clone();
            for f in modu.funcs.iter_mut() {
                f.owner = owner_scope.clone();
            }
            staging_now += modu.heap_estimate();
            staging_peak = staging_peak.max(staging_now);
            decoded_units.push(modu);
        }
        let mut all_units = decoded_units;
        all_units.extend(fresh_units);
        // ---- F798-R3-03: diagnostics re-binding to the CURRENT revision --
        // Persisted MIR/ABC units are spanless by design; a unit entering
        // the closure without spans binds them (sidecar under this
        // snapshot → this request's lowered spans → ONE bounded
        // diagnostics re-derive lower, counted separately and honestly as
        // `sidecar_rebind_lower_modules`). A unit with no derivable source
        // mapping is a controlled identity rejection — never a silently
        // spanless executable whose traps would lose their source.
        {
            let names: Vec<String> = all_units
                .iter()
                .filter(|u| u.funcs[0].spans.is_empty())
                .map(|u| u.funcs[0].name.clone())
                .collect();
            let mut rederived: Option<std::collections::BTreeMap<String, Vec<(u32, SpanInfo)>>> =
                None;
            for name in names {
                let unit = all_units
                    .iter_mut()
                    .find(|u| u.funcs[0].name == name && u.funcs[0].spans.is_empty())
                    .expect("just collected");
                let mut bound = false;
                // This request's own lowered spans first: zero IO and by
                // construction the exact current revision. The sidecar
                // under the current snapshot serves requests that hit
                // without lowering; the bounded re-derive covers the rest.
                if let Some(spans) = lowered_spans.get(&name) {
                    merge_spans(unit, spans);
                    bound = true;
                }
                if !bound {
                    if let Some(store) = self.store.as_ref() {
                        let skey = sidecar_key(&owner, &name, snapshot);
                        let sexpect = EnvelopeIdentity::sidecar(&owner, snapshot);
                        match get_enveloped_opt(store, &skey, "A08", &sexpect) {
                            Ok(Some(sp)) => {
                                if let Ok(spans) = decode_sidecar(&sp) {
                                    merge_spans(unit, &spans);
                                    bound = true;
                                }
                                // Present but undecodable: fall through to
                                // the re-derive, which rewrites the bad
                                // sidecar.
                            }
                            Ok(None) => {}
                            Err(e) => {
                                return (
                                    Some(StageDiag {
                                        stage: "cache".to_string(),
                                        code: "cache.identity".to_string(),
                                        message: format!("sidecar `{}`: {}", name, e),
                                    }),
                                    None,
                                    vec![],
                                    None,
                                );
                            }
                        }
                    }
                }
                if !bound {
                    if rederived.is_none() {
                        match obs.span("sidecar_rebind_lower", Some("request"), |o| {
                            rederive_spans(
                                checked,
                                &owner,
                                entry,
                                snapshot,
                                self.store.as_ref(),
                                &self.project,
                                o,
                            )
                        }) {
                            Ok(map) => {
                                rederived = Some(map);
                                obs.counter("sidecar_rebind_lower_modules", 1);
                            }
                            Err(d) => return (Some(d), None, vec![], None),
                        }
                    }
                    let spans = rederived
                        .as_ref()
                        .unwrap()
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| {
                            StageDiag {
                                stage: "cache".to_string(),
                                code: "cache.identity".to_string(),
                                message: format!(
                                    "unit `{}` has no sidecar under the current snapshot and is absent from the current revision",
                                    name
                                ),
                            }
                        });
                    match spans {
                        Ok(s) => {
                            merge_spans(unit, &s);
                        }
                        Err(d) => return (Some(d), None, vec![], None),
                    }
                }
            }
        }
        let merged = match obs.span("closure_assemble", Some("request"), |o| {
            assemble_closure(&all_units, &owner, entry).map_err(|e| {
                o.counter("closure_reject", 1);
                StageDiag {
                    stage: "closure".to_string(),
                    code: "closure.inconsistent".to_string(),
                    message: e,
                }
            })
        }) {
            Ok(m) => m,
            Err(d) => return (Some(d), None, vec![], None),
        };
        // Whole-closure admission through the REAL codec + verifier (the
        // executor accepts only this credential).
        let mut closure_bytes_total: usize = 0;
        let loaded: LoadedModule = match obs.span("verify_load", Some("request"), |_| {
            encode_abc(&merged)
                .map_err(|e| StageDiag {
                    stage: "abc-emit".to_string(),
                    code: e.code(),
                    message: "closure re-encode failed".to_string(),
                })
                .and_then(|bytes| {
                    closure_bytes_total = bytes.total_len;
                    decode_abc(&bytes.to_flat()).map_err(|e| StageDiag {
                        stage: "abc-load".to_string(),
                        code: e.code(),
                        message: "closure roundtrip decode failed".to_string(),
                    })
                })
                .and_then(|decoded| {
                    abc_verify::verify_abc(decoded, &owner_scope).map_err(|e| StageDiag {
                        stage: "abc-verify".to_string(),
                        code: format!("{}", e),
                        message: "closure verification failed".to_string(),
                    })
                })
        }) {
            Ok(l) => l,
            Err(d) => return (Some(d), None, vec![], None),
        };
        verify_analysis_bytes += loaded.stats().analysis_bytes as usize;
        obs.set_counter("verify_load", 1);
        obs.counter("closure_encodes", 1);
        // F798-R3-08: partition accounting over REAL working-set estimates.
        // Hot = the loaded closure retained through execution (wire + the
        // decoded module structure + the credential's own clone); staging
        // additionally covers the codec roundtrip buffers and every
        // verification's composite analysis bound (recorded on the
        // credential), finalized AFTER the closure codec — decode/index/
        // assembly staging lives until here (F798-R2-02).
        let hot_bytes =
            closure_bytes_total + merged.heap_estimate() + loaded.module().heap_estimate();
        staging_now += closure_bytes_total + merged.heap_estimate() + verify_analysis_bytes;
        staging_peak = staging_peak.max(staging_now);
        obs.set_counter("peak_hot_bytes", hot_bytes as u64);
        obs.set_counter("peak_staging_bytes", staging_peak as u64);

        // ---- manifest ----------------------------------------------------
        let mut manifest_out: Option<ModuleManifest> = None;
        if let Some(store) = self.store.as_ref() {
            let mut links: Vec<UnitLink> = Vec::new();
            for info in infos.iter() {
                let key = abc_unit_key(&owner, &info.name, &info.fp, &caps_fp);
                // F798-R3-02: bind the link to the EXACT payload bytes this
                // request loaded/emitted (fingerprinted where the bytes
                // were in hand — no per-unit store re-read on the warm
                // path), and run the REAL shared-domain association
                // admission (DeclRef re-derivation + envelope + payload
                // binding) at the point of consumption.
                let Some(payload_fp) = info.payload_fp.clone() else {
                    continue;
                };
                let decl = crate::semrefs::unit_decl_ref(
                    &crate::semrefs::av_package(),
                    &owner,
                    &info.name,
                    &info.dump,
                );
                let envelope = crate::semrefs::unit_sema_ref(&payload_fp);
                if let Err(e) = crate::semrefs::admit_unit_ref(&envelope, &decl, &payload_fp) {
                    return (
                        Some(StageDiag {
                            stage: "sema".to_string(),
                            code: "sema.ref-admission".to_string(),
                            message: format!("unit `{}`: {:?}", info.name, e),
                        }),
                        None,
                        vec![],
                        None,
                    );
                }
                if let Ok(Some(art)) = store.get_artifact("A09", &key) {
                    links.push(UnitLink {
                        name: info.name.clone(),
                        artifact: art.artifact_id.clone(),
                        sig: info.sig.clone(),
                        logical: info.logical.clone(),
                        payload_fp: payload_fp.clone(),
                        sema: Some(SemaRefSer::for_payload(&payload_fp)),
                    });
                }
            }
            let manifest = ModuleManifest {
                owner: owner.clone(),
                entry: entry.to_string(),
                profile: auto_av_core::contract::AV_PROFILE.to_string(),
                av_sem: 1,
                units: links,
            };
            let key = manifest_key(&owner, entry, &manifest.fingerprint());
            let expect = EnvelopeIdentity::manifest(&owner, 0);
            let body = serde_json::to_vec(&manifest).unwrap_or_default();
            let _ = put_enveloped(
                store,
                &key,
                "A09",
                &expect,
                &body,
                vec![],
                Some(&self.project),
            );
            manifest_out = Some(manifest);
        }

        // ---- execute ------------------------------------------------------
        let cancel = CancelFlag::new(false);
        let limits = ExecLimits::default();
        let (outcome, trace) = obs.span("av_execute", Some("request"), |_| {
            execute(&loaded, &limits, &cancel)
        });
        obs.set_counter("av_execute", 1);
        obs.set_counter("av_peak_slot_bytes", trace.peak_slot_bytes);
        if let ExecOutcome::Success { steps, .. } = &outcome {
            obs.set_counter("av_steps", *steps);
        }
        (None, Some(outcome), trace.events.clone(), manifest_out)
    }
}

fn decoded_clone(d: &auto_av_core::DecodedAbc) -> auto_av_core::DecodedAbc {
    auto_av_core::DecodedAbc {
        version: d.version,
        profile: d.profile,
        module: d.module.clone(),
    }
}

/// Diagnostics-only span re-derive (F798-R3-03): lower the CURRENT checked
/// module once and return every function's span map, persisting sidecars
/// under the current snapshot when a store is attached. Counted separately
/// from `mir_lower_modules` — this is diagnostics provenance regeneration,
/// never a code-artifact rebuild (no MIR unit or ABC payload is persisted).
fn rederive_spans(
    checked: &CheckedModule,
    owner: &str,
    entry: &str,
    snapshot: u64,
    store: Option<&ac_artifact_probe::store::ManagedStore>,
    project: &str,
    _obs: &mut Obs,
) -> Result<std::collections::BTreeMap<String, Vec<(u32, SpanInfo)>>, StageDiag> {
    let m = lower_checked(checked, entry, snapshot).map_err(|e: AdapterError| StageDiag {
        stage: "mir-lower".to_string(),
        code: e.code(),
        message: format!("span re-derive lowering failed: {:?}", e),
    })?;
    let units = split_units(&m).map_err(|e| StageDiag {
        stage: "mir-split".to_string(),
        code: "mir.split".to_string(),
        message: format!("span re-derive split failed: {}", e),
    })?;
    let mut map = std::collections::BTreeMap::new();
    for u in units.iter() {
        let name = u.funcs[0].name.clone();
        if let Some(store) = store {
            let skey = sidecar_key(owner, &name, snapshot);
            let sexpect = EnvelopeIdentity::sidecar(owner, snapshot);
            let spayload = encode_sidecar(&u.funcs[0].spans);
            put_enveloped(
                store,
                &skey,
                "A08",
                &sexpect,
                &spayload,
                vec![],
                Some(project),
            )
            .map_err(|e| StageDiag {
                stage: "cache".to_string(),
                code: "cache.put".to_string(),
                message: format!("sidecar re-derive `{}`: {}", name, e),
            })?;
        }
        map.insert(name, u.funcs[0].spans.clone());
    }
    Ok(map)
}

fn sig_string(
    module: &auto_ac_prototype::hir::Module,
    ty: auto_ac_prototype::hir::TypeId,
) -> String {
    use auto_ac_prototype::hir::{TypeKind, TypeUse};
    let shape = |u: &TypeUse| -> String {
        match u {
            TypeUse::Builtin(b) => b.as_str().to_string(),
            TypeUse::Ref(tid) => match module.types.get(tid.0 as usize).map(|t| &t.kind) {
                Some(TypeKind::I32) => "i32".to_string(),
                Some(TypeKind::Bool) => "bool".to_string(),
                _ => "?".to_string(),
            },
        }
    };
    match module.types.get(ty.0 as usize).map(|t| &t.kind) {
        Some(TypeKind::Function(sig)) => {
            let mut s = String::from("(");
            for p in &sig.params {
                s.push_str(&shape(p));
                s.push(',');
            }
            s.push_str(")->");
            s.push_str(&shape(&sig.result));
            s
        }
        _ => "?".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Front-door admission
// ---------------------------------------------------------------------------

/// Frozen front-door policy: input size + token-stream nesting ≤128 over
/// ALL bracket kinds — measured on the REAL token streams of both landed
/// frontends (`source::lexer::tokenize`, `atom_text::tokenize`). The source
/// lexer/parser/resolve/typecheck walks are ITERATIVE (landed R5/R6), so
/// the genuine host-recursion paths in the old frontends are the Atom
/// document parser's nested-list recursion (the `[` depth scan in
/// [`front_door_admit_atom`]) and the recursive `eval_expr` over
/// left/right-associated operator CHAINS: a flat `a + b + c + …` run
/// builds one parse-recursion level per operand even at bracket depth 1,
/// so the front door counts consecutive operator tokens as expression
/// CONSTRUCTION depth too (F798-R3-04 — a 600-term flat chain must be a
/// resource rejection HERE, not a `mir.lower.depth` unsupported from
/// inside the recursive parser at its own 512 limit).
/// Breaches are RESOURCE rejections, never syntax errors.
pub fn front_door_admit(src: &str) -> Result<(), StageDiag> {
    use auto_ac_prototype::source::lexer::{tokenize, Tok, P};
    if src.len() > MAX_INPUT_BYTES {
        return Err(StageDiag {
            stage: "admission".into(),
            code: ResourceKind::InputTooLarge.code().into(),
            message: format!(
                "source input {} exceeds the frozen budget {}",
                src.len(),
                MAX_INPUT_BYTES
            ),
        });
    }
    let tokens = tokenize(src).map_err(|d| StageDiag {
        stage: "admission".into(),
        code: d.code.clone(),
        message: d.message.clone(),
    })?;
    let (mut depth, mut max_depth) = (0u32, 0u32);
    // Operator-run depth: consecutive binary/unary operator puncts at one
    // bracket level, reset by ANY other punct (delimiters terminate an
    // expression) but not by operands (Idents/keywords/literals continue
    // the same expression construction).
    let is_chain_op = |p: &P| {
        matches!(
            p,
            P::Plus
                | P::Star
                | P::Slash
                | P::Percent
                | P::Minus
                | P::Lt
                | P::Gt
                | P::LeEq
                | P::GeEq
                | P::EqEq
                | P::NotEq
                | P::AmpAmp
                | P::PipePipe
        )
    };
    let (mut chain_ops, mut max_chain) = (0u32, 0u32);
    for t in &tokens {
        match &t.tok {
            Tok::Punct(p) if matches!(p, P::LParen | P::LBracket | P::LBrace) => {
                depth += 1;
                max_depth = max_depth.max(depth);
                chain_ops = 0;
            }
            Tok::Punct(p) if matches!(p, P::RParen | P::RBracket | P::RBrace) => {
                depth = depth.saturating_sub(1);
                chain_ops = 0;
            }
            Tok::Punct(p) if is_chain_op(p) => {
                chain_ops += 1;
                max_chain = max_chain.max(chain_ops);
            }
            Tok::Punct(_) => {
                // Any other punct (`,` `;` `:` `=` `.` `!`) terminates the
                // current expression construction.
                chain_ops = 0;
            }
            _ => {}
        }
    }
    // A run of K operators forms a K+1-operand construction — one parse
    // recursion level per operand.
    let expr_depth = max_chain + 1;
    let over = max_depth > MAX_FRONTEND_NESTING || expr_depth > MAX_FRONTEND_NESTING;
    if over {
        return Err(StageDiag {
            stage: "admission".into(),
            code: "av.resource.frontend-nesting".into(),
            message: format!(
                "token nesting {} / expression construction depth {} exceeds the frozen front-door budget {}",
                max_depth, expr_depth, MAX_FRONTEND_NESTING
            ),
        });
    }
    Ok(())
}

/// Atom-text front door (the recursion-critical path): nesting over the
/// Atom token stream before the recursive document parser runs.
pub fn front_door_admit_atom(src: &str) -> Result<(), StageDiag> {
    if src.len() > MAX_INPUT_BYTES {
        return Err(StageDiag {
            stage: "admission".into(),
            code: ResourceKind::InputTooLarge.code().into(),
            message: format!(
                "atom input {} exceeds the frozen budget {}",
                src.len(),
                MAX_INPUT_BYTES
            ),
        });
    }
    let tokens = auto_ac_prototype::atom_text::tokenize(src).map_err(|d| StageDiag {
        stage: "admission".into(),
        code: d.code.clone(),
        message: d.message.clone(),
    })?;
    let (mut depth, mut max_depth) = (0u32, 0u32);
    for t in &tokens {
        if let auto_ac_prototype::atom_text::Tok::Punct(c) = t.tok {
            match c {
                '[' => {
                    depth += 1;
                    max_depth = max_depth.max(depth);
                }
                ']' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    if max_depth > MAX_FRONTEND_NESTING {
        return Err(StageDiag {
            stage: "admission".into(),
            code: "av.resource.frontend-nesting".into(),
            message: format!(
                "atom nesting {} exceeds the frozen front-door budget {} (rejected before the recursive document parser)",
                max_depth, MAX_FRONTEND_NESTING
            ),
        });
    }
    Ok(())
}
