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
    abc_unit_key_with_codec, assemble_closure, caps_fingerprint, decode_sidecar, encode_sidecar,
    manifest_key, merge_spans, open_store, payload_fingerprint, put_enveloped, sidecar_key,
    split_units, Counters, EnvelopeIdentity, ModuleManifest, SemaRefSer, UnitLink,
};
use crate::observations::{Obs, SpanRec};
use auto_ac_prototype::source::compile_scalar_source;
use auto_ac_prototype::verify::CheckedModule;
use auto_av_core::contract::{
    fnv1a64, ExecOutcome, ResourceKind, SpanInfo, Value, MAX_FRONTEND_NESTING, MAX_INPUT_BYTES,
};
use auto_av_core::mir::{MirModule, OwnerScope, TraceEvent};
use auto_av_core::{
    decode_abc, encode_abc, execute, verify_mir, CancelFlag, ExecLimits, LoadedModule,
};
use auto_mir_lower::{lower_checked, AdapterError};
use serde::{Deserialize, Serialize};

/// One diagnosed pipeline failure: stage + code + message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StageDiag {
    pub stage: String,
    pub code: String,
    pub message: String,
}

/// F798-R12-03: is this cache error a same-key-different-content store
/// CONFLICT (the store never overwrites; the old row survives)? Tolerated
/// only for re-persisting a unit this request rebuilt over a stale
/// payload — never for a first publish or an unrelated writer.
fn is_store_conflict(e: &crate::cache::CacheError) -> bool {
    matches!(
        e,
        crate::cache::CacheError::Store(ac_artifact_probe::store::StoreError::Conflict { .. })
    )
}

/// F798-R12-03 (K14-06): O(1) format-generation gate for a cached payload.
/// The kind-ambiguity that needs isolating is exactly the WIRE GENERATION:
/// a kind-less /1 artifact under a /2 key must never be read in place.
/// A current-generation payload cannot be kind-ambiguous (the /2 flag
/// field has definite semantics either way) — its full decode and deep
/// admission still happen at the closure stage, where a genuinely corrupt
/// or tampered payload is a controlled rejection.
fn payload_is_current_generation(payload: &[u8]) -> bool {
    payload.len() >= 6
        && payload[..4] == *auto_av_core::abc::ABC_MAGIC
        && u16::from_le_bytes([payload[4], payload[5]]) == auto_av_core::abc::ABC_VERSION
}

/// Terminal state of one request.
#[derive(Clone, Debug)]
pub struct RequestOutcome {
    /// Diagnosed rejection before execution (admission/unsupported/infra).
    pub diag: Option<StageDiag>,
    /// AV execution outcome when the pipeline reached execution.
    pub av: Option<ExecOutcome>,
    pub trace: Vec<TraceEvent>,
    pub execution: auto_av_core::av::ExecTrace,
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

    /// PLAN-862: the UNIT result channel. `Some(())` only for a Unit entry
    /// that succeeded — an `i32 0` result is a DIFFERENT payload and never
    /// satisfies this.
    pub fn value_unit(&self) -> Option<()> {
        match &self.av {
            Some(ExecOutcome::Success {
                value: Value::Unit, ..
            }) => Some(()),
            _ => None,
        }
    }

    /// PLAN-862: the observed result channel, named (diagnostics must print
    /// the channel, never just a number — that would fold Unit into 0).
    pub fn channel(&self) -> &'static str {
        match &self.av {
            Some(ExecOutcome::Success { value, .. }) => match value {
                Value::Unit => "unit",
                Value::I32(_) => "value",
                _ => "value-other",
            },
            Some(ExecOutcome::Trap { .. }) => "trap",
            Some(ExecOutcome::Resource { .. }) => "resource",
            Some(ExecOutcome::Cancelled) => "cancelled",
            Some(ExecOutcome::Infra(_)) => "infra",
            None => "none",
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
    /// PLAN-826: the A09/closure wire generation this driver encodes,
    /// caches and admits under. Defaults to the delivered legacy AVAB core
    /// wire; the compact AVAC policy is opt-in per driver so every legacy
    /// consumer path is byte-identical until a caller switches.
    wire: crate::wire::WireCodec,
    /// PLAN-862: which SOURCE profile this driver serves. `Scalar` keeps the
    /// historical path byte-for-byte; `EntryUnit` compiles through the
    /// entry-unit front door and rides the `AVAU` wire generation.
    profile: RequestProfile,
}

/// PLAN-862: the request profile selector. One driver serves one profile at
/// a time (the wire generation and the cache namespace follow it).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RequestProfile {
    /// `auto.source.scalar-i32.draft` → `av.core-i32/1` (the delivered path).
    Scalar,
    /// `auto.source.entry-unit.draft` → `av.entry-unit/1` (PLAN-862).
    EntryUnit,
}

impl Driver {
    /// A driver WITHOUT a store: pure pipeline (no cache), for admission /
    /// semantics tiers and cross-checks.
    pub fn ephemeral() -> Driver {
        Driver {
            store: None,
            project: String::new(),
            owner_override: None,
            wire: crate::wire::WireCodec::LegacyAbc,
            profile: RequestProfile::Scalar,
        }
    }

    /// A driver over a real managed store root (created when absent).
    pub fn with_store(root: &std::path::Path, project: &str) -> Result<Driver, String> {
        let store = open_store(root).map_err(|e| e.to_string())?;
        Ok(Driver {
            store: Some(store),
            project: project.to_string(),
            owner_override: None,
            wire: crate::wire::WireCodec::LegacyAbc,
            profile: RequestProfile::Scalar,
        })
    }

    pub fn set_owner(&mut self, owner: Option<String>) {
        self.owner_override = owner;
    }

    /// PLAN-826: switch this driver's A09/closure wire to the compact
    /// AVAC generation (codec-dimensioned cache namespace + compact
    /// admission). Legacy rows under the same store are a deterministic
    /// miss for this policy — never reinterpreted.
    pub fn set_wire_compact(&mut self) {
        self.wire = crate::wire::WireCodec::CompactAbc;
    }

    /// PLAN-862: serve the entry-unit profile. The wire generation follows
    /// the profile (the AVAU family): cached AVAB/AVAD rows under the same
    /// store are a deterministic stale miss, never reinterpreted.
    pub fn set_profile(&mut self, profile: RequestProfile) {
        self.profile = profile;
        self.wire = match profile {
            RequestProfile::Scalar => match self.wire {
                crate::wire::WireCodec::CompactAbc => crate::wire::WireCodec::CompactAbc,
                _ => crate::wire::WireCodec::LegacyAbc,
            },
            RequestProfile::EntryUnit => crate::wire::WireCodec::EntryUnitAbc,
        };
    }

    /// The profile this driver currently serves.
    pub fn profile(&self) -> RequestProfile {
        self.profile
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
            execution: auto_av_core::av::ExecTrace {
                events: trace.clone(),
                loaded_index_builds: *obs.counters().get("loaded_index_builds").unwrap_or(&0),
                loaded_index_reuses: *obs.counters().get("loaded_index_reuses").unwrap_or(&0),
                loaded_index_bytes: *obs.counters().get("loaded_index_bytes").unwrap_or(&0),
                loaded_pc_lookups: *obs.counters().get("loaded_pc_lookups").unwrap_or(&0),
                reference_pc_lookups: *obs.counters().get("reference_pc_lookups").unwrap_or(&0),
                loaded_param_map_queries: *obs
                    .counters()
                    .get("loaded_param_map_queries")
                    .unwrap_or(&0),
                reference_param_map_copies: *obs
                    .counters()
                    .get("reference_param_map_copies")
                    .unwrap_or(&0),
                ..Default::default()
            },
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
        // PLAN-862: the profile selects the front door. The scalar path is
        // untouched; the entry-unit path compiles the same way plus the
        // entry matrix / Unit result, into its own HIR domain.
        let compiled = obs.span("frontend", Some("request"), |o| {
            #[cfg(feature = "test-case-batch")]
            o.counter("frontend_compile_calls", 1);
            let result = match self.profile {
                RequestProfile::Scalar => compile_scalar_source(src, "probe.at", caps),
                RequestProfile::EntryUnit => {
                    auto_ac_prototype::source::compile_entry_unit_source(src, "probe.at", caps)
                }
            };
            result.map_err(|ds| {
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
        // F798-R12-03: units whose cached payload failed to DECODE under
        // the current wire generation (e.g. a kind-less /1 artifact under
        // a /2 key). They rebuild this request; re-persisting over the
        // stale row may Conflict — tolerated, never a request failure.
        let mut stale_rebuilds: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        // F798-R15-04: consumption leases — every A09 unit artifact this
        // request loads or emits is held live for the request window
        // (dropped at request end); the manifest's required-data edges
        // extend the same protection to future restores.
        let mut held_leases: Vec<ac_artifact_probe::store::Lease> = Vec::new();
        // F798-R15-04 (warm path): sidecar artifact ids captured from the
        // reads the rebind cascade performs anyway — manifest edges reuse
        // them instead of one extra store read per unit.
        let mut sidecar_art_ids: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        if let Some(store) = self.store.as_ref() {
            for info in infos.iter_mut() {
                // F798-R6-01: the hit expects the artifact's envelope to
                // declare THIS unit's body fingerprint — a good payload
                // stored under another body's key fails identity admission
                // (controlled rejection), never a wrong-body hit.
                let expect = EnvelopeIdentity::abc_bound(&owner, &info.name, &info.fp);
                let key = abc_unit_key_with_codec(
                    &owner,
                    &info.name,
                    &info.fp,
                    &caps_fp,
                    self.wire.schema(),
                );
                match crate::cache::get_enveloped_with_id(store, &key, "A09", &expect) {
                    Ok(Some((artifact_id, payload))) => {
                        // F798-R12-03 (K14-06): a payload of a DIFFERENT
                        // wire generation (e.g. a kind-less /1 artifact
                        // under a /2 key) is a deterministic STALE MISS —
                        // never read in place as new-semantics bytes and
                        // never a hard error: this unit rebuilds from the
                        // current source and is re-verified through the
                        // real pipeline. Current-generation payloads deep-
                        // decode at closure assembly (tamper = rejection).
                        // PLAN-826: the generation gate is policy-aware —
                        // under the compact policy an AVAB payload is the
                        // other generation (and vice versa).
                        if !self.wire.is_current_generation(&payload) {
                            obs.counter("cache_stale_abc_miss", 1);
                            stale_rebuilds.insert(info.name.clone());
                            continue;
                        }
                        hits += 1;
                        staging_now += payload.len();
                        staging_peak = staging_peak.max(staging_now);
                        info.payload_fp = Some(payload_fingerprint(&payload));
                        info.abc_hit = Some(payload);
                        // Pin every cached artifact consumed by this request. A
                        // first-unit lease does not retain its siblings: required
                        // edges point from the manifest to units, never backwards.
                        {
                            match store.acquire_lease(
                                "plan798-request-consume",
                                300,
                                Some(&artifact_id),
                            ) {
                                Ok(l) => held_leases.push(l),
                                Err(e) => {
                                    return (
                                        Some(StageDiag {
                                            stage: "cache".to_string(),
                                            code: "cache.lease".to_string(),
                                            message: format!("unit `{}`: {}", info.name, e),
                                        }),
                                        None,
                                        vec![],
                                        None,
                                    )
                                }
                            }
                        }
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
            let mut remaining: Vec<usize> = Vec::new();
            for idx in abc_misses.iter() {
                let idx = *idx;
                let info = &infos[idx];
                // F798-R6-01: unit/body-fp bound envelope on the A08 layer.
                let expect = EnvelopeIdentity::mir_bound(&owner, &info.name, &info.fp);
                let key = crate::cache::mir_unit_key(&owner, &info.name, &info.fp, &caps_fp);
                match crate::cache::get_enveloped_with_id(store, &key, "A08", &expect) {
                    Ok(Some((artifact_id, payload))) => {
                        match store.acquire_lease("plan798-request-mir", 300, Some(&artifact_id)) {
                            Ok(l) => held_leases.push(l),
                            Err(e) => {
                                return (
                                    Some(StageDiag {
                                        stage: "cache".into(),
                                        code: "cache.lease".into(),
                                        message: e.to_string(),
                                    }),
                                    None,
                                    vec![],
                                    None,
                                )
                            }
                        }
                        staging_now += payload.len();
                        staging_peak = staging_peak.max(staging_now);
                        if !payload_is_current_generation(&payload) {
                            // F798-R12-03 (K14-06): an MIR unit payload of
                            // a different wire generation is a deterministic
                            // stale miss — this unit re-lowers and
                            // re-verifies from the current source (never
                            // read in place).
                            obs.counter("cache_stale_mir_miss", 1);
                            stale_rebuilds.insert(info.name.clone());
                            remaining.push(idx);
                            continue;
                        }
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
                        // F798-R15-02: the A08 envelope declares a UNIT —
                        // a complete-module payload under a unit envelope
                        // is a producer/payload category conflict.
                        if !decoded.module.is_unit {
                            return (
                                Some(StageDiag {
                                    stage: "cache".to_string(),
                                    code: "cache.identity".to_string(),
                                    message: format!(
                                        "unit `{}` envelope carries a complete-module payload — category conflict",
                                        info.name
                                    ),
                                }),
                                None,
                                vec![],
                                None,
                            );
                        }
                        // F798-R15-01: capability re-admission against the
                        // CURRENT request grants (the key's caps axis is
                        // the request's, never the payload's needs).
                        for c in decoded.module.capabilities.iter() {
                            if !caps.iter().any(|x| x == c) {
                                return (
                                    Some(StageDiag {
                                        stage: "cache".to_string(),
                                        code: "cache.grant".to_string(),
                                        message: format!(
                                            "unit `{}` requires capability `{}` not granted by the current request",
                                            info.name, c
                                        ),
                                    }),
                                    None,
                                    vec![],
                                    None,
                                );
                            }
                        }
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
                    let mexpect = EnvelopeIdentity::mir_bound(&owner, &info.name, &info.fp);
                    if let Err(e) = put_enveloped(
                        store,
                        &mkey,
                        "A08",
                        &mexpect,
                        &mbytes.to_flat(),
                        vec![],
                        Some(&self.project),
                    ) {
                        // F798-R12-03: re-persisting a rebuilt unit over its
                        // STALE row conflicts (the store never overwrites).
                        // The rebuild itself is the invalidation — the old
                        // row survives untouched and the request proceeds
                        // on the freshly verified in-memory unit.
                        if stale_rebuilds.contains(info.name.as_str()) && is_store_conflict(&e) {
                            obs.counter("cache_stale_rebuild_conflict", 1);
                        } else {
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
                let payload = match self.wire.encode(&spanless) {
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
                staging_now += spanless.heap_estimate() + payload.len();
                staging_peak = staging_peak.max(staging_now);
                let payload_fp = payload_fingerprint(&payload);
                abc_emitted += 1;
                if self.wire == crate::wire::WireCodec::CompactAbc {
                    obs.counter("abc_wire_compact_units", 1);
                }
                if let Some(store) = self.store.as_ref() {
                    let key = abc_unit_key_with_codec(
                        &owner,
                        &name,
                        &info.fp,
                        &caps_fp,
                        self.wire.schema(),
                    );
                    let expect = EnvelopeIdentity::abc_bound(&owner, &name, &info.fp);
                    if let Err(e) = put_enveloped(
                        store,
                        &key,
                        "A09",
                        &expect,
                        &payload,
                        vec![],
                        Some(&self.project),
                    ) {
                        // F798-R12-03: same tolerance as the MIR re-persist
                        // above — the stale row survives, the rebuilt unit
                        // still serves THIS request (re-verified in full).
                        if stale_rebuilds.contains(name.as_str()) && is_store_conflict(&e) {
                            obs.counter("cache_stale_rebuild_conflict", 1);
                        } else {
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
            // PLAN-862: admission re-stamps the MIR module owner, so the
            // schema here is the MIR IDENTITY of the profile (the entry-unit
            // family's MIR schema), not the ABC wire schema — `verify_mir`
            // checks that pair. Every earlier profile keeps the wire schema
            // (their verifiers do not key on it).
            schema: match self.profile {
                RequestProfile::EntryUnit => {
                    auto_av_core::contract::MIR_ENTRY_UNIT_SCHEMA.to_string()
                }
                RequestProfile::Scalar => self.wire.owner_schema().to_string(),
            },
            module: owner.clone(),
        };
        for info in infos.iter().filter(|i| i.abc_hit.is_some()) {
            let payload = info.abc_hit.as_ref().expect("filtered");
            let wire = self.wire;
            let mut decoded = match obs.span("abc_load", Some("request"), |_| wire.decode(payload))
            {
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
            // F798-R15-02: the A09 envelope declares a UNIT — a
            // complete-module payload under a unit envelope is a
            // producer/payload category conflict.
            if !decoded.module.is_unit {
                return (
                    Some(StageDiag {
                        stage: "cache".to_string(),
                        code: "cache.identity".to_string(),
                        message: format!(
                            "unit `{}` envelope carries a complete-module payload — category conflict",
                            info.name
                        ),
                    }),
                    None,
                    vec![],
                    None,
                );
            }
            // PLAN-826: the compact wire carries no function names — the
            // envelope-bound unit name re-binds onto the decoded payload
            // (same envelope-responsibility rule as the owner re-stamp).
            let wire = self.wire;
            wire.rebind_unit_name(&mut decoded.module, &info.name);
            // F798-R15-01: capability re-admission against the CURRENT
            // request grants.
            for c in decoded.module.capabilities.iter() {
                if !caps.iter().any(|x| x == c) {
                    return (
                        Some(StageDiag {
                            stage: "cache".to_string(),
                            code: "cache.grant".to_string(),
                            message: format!(
                                "unit `{}` requires capability `{}` not granted by the current request",
                                info.name, c
                            ),
                        }),
                        None,
                        vec![],
                        None,
                    );
                }
            }
            match obs.span("abc_verify", Some("request"), |_| {
                wire.admit_decoded(
                    crate::wire::DecodedWire {
                        version: decoded.version,
                        profile: decoded.profile,
                        module: decoded.module.clone(),
                    },
                    &owner_scope,
                )
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
                        match crate::cache::get_enveloped_with_id(store, &skey, "A08", &sexpect) {
                            Ok(Some((sp_artifact, sp))) => {
                                // F798-R15-04 (warm-path): the sidecar's
                                // artifact id comes from THIS read — the
                                // manifest edge loop never re-reads it.
                                sidecar_art_ids.insert(name.clone(), sp_artifact);
                                if let Ok(spans) = decode_sidecar(&sp) {
                                    merge_spans(unit, &spans);
                                    // F798-R15-03: decodable is not
                                    // APPLICABLE — empty/non-covering maps
                                    // fall through to the re-derive (which
                                    // rewrites the bad sidecar) instead of
                                    // publishing source-less traps.
                                    if crate::cache::sidecar_applicable(&unit.funcs[0]).is_err() {
                                        unit.funcs[0].spans.clear();
                                    } else {
                                        bound = true;
                                    }
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
        // F798-R15-01 (central): the assembled closure's required
        // capabilities must be a subset of the CURRENT request grants —
        // dispatch refuses before encoding when a cached/restored payload
        // needs a capability this request never granted (the per-unit
        // checks above attribute it; this is the invariant backstop).
        for c in merged.capabilities.iter() {
            if !caps.iter().any(|x| x == c) {
                return (
                    Some(StageDiag {
                        stage: "closure".to_string(),
                        code: "closure.grant".to_string(),
                        message: format!(
                            "closure requires capability `{}` not granted by the current request",
                            c
                        ),
                    }),
                    None,
                    vec![],
                    None,
                );
            }
        }
        // Whole-closure admission through the REAL codec + verifier (the
        // executor accepts only this credential). PLAN-826: the closure
        // roundtrips through THIS driver's wire policy — the compact
        // policy encodes, decodes and admits the whole closure as AVAC.
        let mut closure_bytes_total: usize = 0;
        let wire = self.wire;
        let loaded: LoadedModule = match obs.span("verify_load", Some("request"), |_| {
            wire.encode(&merged)
                .map_err(|e| StageDiag {
                    stage: "abc-emit".to_string(),
                    code: e.code(),
                    message: "closure re-encode failed".to_string(),
                })
                .and_then(|payload| {
                    closure_bytes_total = payload.len();
                    wire.decode(&payload).map_err(|e| StageDiag {
                        stage: "abc-load".to_string(),
                        code: e.code(),
                        message: "closure roundtrip decode failed".to_string(),
                    })
                })
                .and_then(|decoded| {
                    wire.admit_decoded(decoded, &owner_scope)
                        .map_err(|e| StageDiag {
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
        if wire == crate::wire::WireCodec::CompactAbc {
            obs.counter("abc_wire_compact_closure", 1);
        }
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
                let key = abc_unit_key_with_codec(
                    &owner,
                    &info.name,
                    &info.fp,
                    &caps_fp,
                    self.wire.schema(),
                );
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
                        body_fp: info.fp.clone(),
                        payload_fp: payload_fp.clone(),
                        sema: Some(SemaRefSer::for_payload(&payload_fp)),
                    });
                }
            }
            let manifest = ModuleManifest {
                format: 2,
                diagnostics_snapshot: snapshot,
                owner: owner.clone(),
                entry: entry.to_string(),
                profile: auto_av_core::contract::AV_PROFILE.to_string(),
                av_sem: 1,
                abc_codec: self.wire.schema().to_string(),
                units: links,
            };
            let key = manifest_key(&owner, entry, &manifest.fingerprint());
            let expect = EnvelopeIdentity::manifest(&owner, 0);
            let body = serde_json::to_vec(&manifest).unwrap_or_default();
            // F798-R15-04: the manifest carries REQUIRED-DATA edges to
            // every unit artifact it names — reachability from a
            // rooted/leased manifest now keeps the closure alive; a
            // failed publish is OBSERVABLE, never swallowed.
            let mut edges: Vec<(String, ac_artifact_probe::identity::RefKind)> = manifest
                .units
                .iter()
                .map(|l| {
                    (
                        l.artifact.clone(),
                        ac_artifact_probe::identity::RefKind::RequiredData,
                    )
                })
                .collect();
            // F798-R15-04: the closure's diagnostics provenance is REQUIRED
            // too — edge the current snapshot's sidecar artifacts so a
            // rooted/leased manifest keeps the whole closure restorable.
            for u in infos.iter() {
                let sid = match sidecar_art_ids.get(&u.name) {
                    // Warm path: captured from the cascade's own read.
                    Some(id) => Some(id.clone()),
                    // Edit/fallback path (this unit lowered or re-derived
                    // without a read): resolve now — rare, not per-warm-hit.
                    None => store
                        .get_artifact("A08", &sidecar_key(&owner, &u.name, snapshot))
                        .ok()
                        .flatten()
                        .map(|a| a.artifact_id),
                };
                if let Some(id) = sid {
                    edges.push((id, ac_artifact_probe::identity::RefKind::RequiredData));
                }
            }
            if let Err(e) = put_enveloped(
                store,
                &key,
                "A09",
                &expect,
                &body,
                edges,
                Some(&self.project),
            ) {
                // The manifest key IS its fingerprint: a same-key CONFLICT
                // means another writer already published this logical
                // manifest — the store keeps the old row (never
                // overwritten) and it serves future restores identically.
                // Tolerated and counted; every other failure propagates.
                if !is_store_conflict(&e) {
                    return (
                        Some(StageDiag {
                            stage: "cache".to_string(),
                            code: "cache.put".to_string(),
                            message: format!("manifest publish: {}", e),
                        }),
                        None,
                        vec![],
                        None,
                    );
                }
            }
            manifest_out = Some(manifest);
        }

        // ---- execute ------------------------------------------------------
        let cancel = CancelFlag::new(false);
        let limits = ExecLimits::default();
        let (outcome, trace) = obs.span("av_execute", Some("request"), |_| {
            execute(&loaded, &limits, &cancel)
        });
        obs.set_counter("av_execute", 1);
        obs.set_counter(
            "av_peak_slot_bytes",
            trace
                .peak_slot_bytes
                .saturating_add(trace.loaded_index_bytes),
        );
        obs.set_counter("loaded_index_builds", trace.loaded_index_builds);
        obs.set_counter("loaded_index_reuses", trace.loaded_index_reuses);
        obs.set_counter("loaded_index_bytes", trace.loaded_index_bytes);
        obs.set_counter("loaded_pc_lookups", trace.loaded_pc_lookups);
        obs.set_counter("reference_pc_lookups", trace.reference_pc_lookups);
        obs.set_counter("loaded_param_map_queries", trace.loaded_param_map_queries);
        obs.set_counter(
            "reference_param_map_copies",
            trace.reference_param_map_copies,
        );
        if let ExecOutcome::Success { steps, .. } = &outcome {
            obs.set_counter("av_steps", *steps);
        }
        (None, Some(outcome), trace.events.clone(), manifest_out)
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
            if let Err(e) = put_enveloped(
                store,
                &skey,
                "A08",
                &sexpect,
                &spayload,
                vec![],
                Some(project),
            ) {
                // F798-R15-03: rewriting a BAD sidecar row can conflict
                // (the store never overwrites — e.g. an inapplicable map
                // we just rejected). The in-memory rederived spans still
                // serve THIS request: tolerated, row survives untouched.
                // Every other write failure propagates (F798-R3-03).
                if !is_store_conflict(&e) {
                    return Err(StageDiag {
                        stage: "cache".to_string(),
                        code: "cache.put".to_string(),
                        message: format!("sidecar re-derive `{}`: {}", name, e),
                    });
                }
            }
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
/// [`front_door_admit_atom`]) and operator CHAINS: a flat `a + b + c + …`
/// run builds one construction level per operator even at bracket depth 1,
/// so the front door measures the expression CONSTRUCTION depth too
/// (F798-R3-04 — a 600-term flat chain must be a resource rejection HERE,
/// not a `mir.lower.depth` unsupported from inside the recursive parser at
/// its own 512 limit).
/// Breaches are RESOURCE rejections, never syntax errors.
///
/// The construction metric (F798-R12-01) is a miniature
/// operator-precedence depth evaluator ([`LevelChain`]): it reproduces the
/// tree depth of the REAL parser's association (parser.rs
/// `scalar_prec_of`, left-associative), so the door measures the LONGEST
/// CONSTRUCTION PATH — branch-sibling depths never add, genuinely nested
/// levels still compose, and every call level counts exactly once.
#[derive(Default)]
struct LevelChain {
    /// Completed operand path depths at this level.
    vals: Vec<u32>,
    /// Precedences of pending binary operators (mirrors `scalar_prec_of`).
    ops: Vec<u8>,
    /// Pending prefix unary operators (Neg/Not nest one level each).
    unary: u32,
    /// The last significant token completed an operand (binary vs prefix
    /// `-` disambiguation).
    after_operand: bool,
    /// The top of `vals` is a bare identifier — `(`/`:` may retract it as
    /// a callee/parameter NAME (names are not construction levels).
    ident_top: bool,
    /// Prefix unary levels folded into the `ident_top` value
    /// (F798-R13-01): the `(`-retract of a callee name RESTORES them so
    /// they bind to the full CALL expression (`Unary(Call(..))`), never
    /// vanish with the discarded name.
    ident_unary: u32,
    /// Folded max path of COMPLETED siblings (argument-list commas).
    sib_done: u32,
    /// This level is a sibling list (argument commas / named colons).
    had_comma: bool,
}

impl LevelChain {
    /// Left-associative reduction of pending operators `>= min_prec`
    /// (same `>=` pop rule as the parser's precedence climbing). Reducing
    /// `a OP b` yields `1 + max(a, b)` — the real tree's depth.
    fn reduce_ge(&mut self, min_prec: u8) {
        while self.ops.last().is_some_and(|&top| top >= min_prec) {
            self.ops.pop();
            let b = self.vals.pop().unwrap_or(1);
            let a = self.vals.pop().unwrap_or(1);
            self.vals.push(1 + a.max(b));
            self.ident_top = false;
        }
    }

    fn reduce_all(&mut self) -> u32 {
        self.reduce_ge(0);
        // Defensive max over adjacency-merged leftovers (malformed input
        // is the parser's to reject; the door just stays robust).
        self.vals.iter().copied().max().unwrap_or(0)
    }

    /// A completed operand — literal, `true`/`false`, or a closed paren
    /// group of path `depth` — joins the current chain; pending prefix
    /// unary operators nest around it.
    fn operand(&mut self, depth: u32) {
        let depth = depth + std::mem::take(&mut self.unary);
        if self.after_operand {
            // Adjacent operands with no operator (`a.b`-shapes): one node.
            if let Some(top) = self.vals.last_mut() {
                *top = (*top).max(depth);
            } else {
                self.vals.push(depth);
            }
            self.ident_top = false;
            self.ident_unary = 0;
        } else {
            self.vals.push(depth);
        }
        self.after_operand = true;
    }

    /// A bare IDENTIFIER operand. When it is a FRESH value it becomes the
    /// callee/name candidate (`ident_top`), and the prefix unary levels
    /// folded into it are recorded (`ident_unary`) — the `(`-retract then
    /// RESTORES them onto the full CALL expression instead of letting them
    /// vanish with the discarded name (F798-R13-01: `-f(..)`/`!f(..)` are
    /// `Unary(Call(..))`, one level per prefix, on the construction path).
    fn ident_operand(&mut self) {
        let prefix = std::mem::take(&mut self.unary);
        if self.after_operand {
            // Adjacent identifiers with no operator: one node, not a name.
            if let Some(top) = self.vals.last_mut() {
                *top = (*top).max(1 + prefix);
            } else {
                self.vals.push(1 + prefix);
            }
            self.ident_top = false;
            self.ident_unary = 0;
        } else {
            self.vals.push(1 + prefix);
            self.ident_top = true;
            self.ident_unary = prefix;
        }
        self.after_operand = true;
    }

    fn binary(&mut self, prec: u8) {
        self.reduce_ge(prec);
        self.ops.push(prec);
        self.after_operand = false;
        self.ident_top = false;
        self.ident_unary = 0;
    }

    fn prefix_unary(&mut self) {
        self.unary += 1;
        self.after_operand = false;
        self.ident_top = false;
        self.ident_unary = 0;
    }

    /// A comma ends the current sibling: its COMPOSED path folds by MAX
    /// into the completed-siblings pool (F798-R10-01 — different arguments'
    /// maxima never concatenate); the next sibling starts fresh.
    fn end_sibling(&mut self) -> u32 {
        let v = self.reduce_all();
        self.sib_done = self.sib_done.max(v);
        self.vals.clear();
        self.ops.clear();
        self.unary = 0;
        self.after_operand = false;
        self.ident_top = false;
        self.ident_unary = 0;
        self.had_comma = true;
        v
    }

    /// Close the level: the call/group level itself adds exactly ONE level
    /// over the deepest sibling path — a named single-argument call is not
    /// a list level PLUS an expression level (F798-R12-01: each call layer
    /// counts once).
    fn close(&mut self) -> u32 {
        let v = self.reduce_all();
        1 + self.sib_done.max(v)
    }

    /// Statement/clause boundary (keywords, `;`, `=`, `{`, `}`): the
    /// current chain folds into the global peak (a MAX, never a sum across
    /// statements — F798-R9-01) and the level resets.
    fn clause_boundary(&mut self, peak: &mut u32) {
        let v = self.reduce_all();
        *peak = (*peak).max(v);
        self.vals.clear();
        self.ops.clear();
        self.unary = 0;
        self.after_operand = false;
        self.ident_top = false;
        self.ident_unary = 0;
        self.sib_done = 0;
        self.had_comma = false;
    }
}

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
    // Scalar binary-operator precedence (mirrors the real parser's
    // `scalar_prec_of`: higher binds tighter; left-associative). This is
    // what makes the depth evaluator follow the REAL association instead
    // of summing branch-sibling depths (F798-R12-01).
    let prec_of = |p: &P| -> Option<u8> {
        match p {
            P::PipePipe => Some(1),
            P::AmpAmp => Some(2),
            P::EqEq | P::NotEq => Some(3),
            P::Lt | P::Gt | P::LeEq | P::GeEq => Some(4),
            P::Plus | P::Minus => Some(5),
            P::Star | P::Slash | P::Percent => Some(6),
            _ => None,
        }
    };
    // The CURRENT bracket level's chain; `[`/`(` push a fresh one.
    let mut cur = LevelChain::default();
    let mut level_stack: Vec<LevelChain> = Vec::new();
    let mut peak = 0u32;
    for t in &tokens {
        match &t.tok {
            Tok::Punct(p) if matches!(p, P::LParen | P::LBracket) => {
                depth += 1;
                max_depth = max_depth.max(depth);
                // A callee identifier directly before `(` is a NAME, not
                // an operand of the enclosing chain. The prefix unary
                // levels folded into it are RESTORED here — they bind to
                // the full CALL expression (`-f(..)`/`!f(..)` are
                // `Unary(Call(..))`), never vanish with the name
                // (F798-R13-01).
                if cur.ident_top {
                    cur.vals.pop();
                    cur.unary += cur.ident_unary;
                    cur.ident_unary = 0;
                    cur.after_operand = false;
                    cur.ident_top = false;
                }
                level_stack.push(std::mem::take(&mut cur));
            }
            Tok::Punct(p) if matches!(p, P::RParen | P::RBracket) => {
                depth = depth.saturating_sub(1);
                // F798-R12-01: closing a level yields ITS OWN longest path
                // (call level counted once); the closed group then joins
                // the enclosing chain as ONE operand — composing on the
                // real association path, never a sibling-sum.
                let path = cur.close();
                peak = peak.max(path);
                match level_stack.pop() {
                    Some(mut outer) => {
                        std::mem::swap(&mut cur, &mut outer);
                        cur.operand(path);
                    }
                    None => cur = LevelChain::default(),
                }
            }
            Tok::Punct(P::LBrace) => {
                depth += 1;
                max_depth = max_depth.max(depth);
                // Block structure is a statement boundary, not an
                // expression level (bracket depth covers it).
                cur.clause_boundary(&mut peak);
            }
            Tok::Punct(P::RBrace) => {
                depth = depth.saturating_sub(1);
                cur.clause_boundary(&mut peak);
            }
            // Clause boundaries end the current statement's expression.
            Tok::Punct(P::Semi) | Tok::Punct(P::Assign) => {
                cur.clause_boundary(&mut peak);
            }
            Tok::Punct(P::Comma) => {
                // An argument comma ends the current sibling; its composed
                // path folds by MAX into the siblings pool.
                let v = cur.end_sibling();
                peak = peak.max(v);
            }
            Tok::Punct(P::Colon) => {
                // Named-argument `name:` — the parameter NAME is not a
                // construction level (retract it); the VALUE expression is
                // this level's sibling. The value's own internal
                // composition stays intact (F798-R11-01).
                if cur.ident_top {
                    cur.vals.pop();
                }
                let v = cur.end_sibling();
                peak = peak.max(v);
            }
            // Prefix unary always nests one level; `-` after an operand is
            // the left-associative binary subtraction instead.
            Tok::Punct(P::Bang) => cur.prefix_unary(),
            Tok::Punct(P::Minus) => {
                if cur.after_operand {
                    cur.binary(5);
                } else {
                    cur.prefix_unary();
                }
            }
            Tok::Punct(p) => {
                if let Some(prec) = prec_of(p) {
                    cur.binary(prec);
                }
                // Other puncts (`.`) stay inside the current operand.
            }
            // `true`/`false` are expression OPERANDS, not clause
            // boundaries (F798-R7-02); every other keyword starts a fresh
            // clause in this newline-terminated grammar.
            Tok::Kw(auto_ac_prototype::source::lexer::Kw::True)
            | Tok::Kw(auto_ac_prototype::source::lexer::Kw::False) => cur.operand(1),
            Tok::Kw(_) | Tok::Reserved(_) => cur.clause_boundary(&mut peak),
            Tok::Ident(_) => cur.ident_operand(),
            Tok::Int(_) | Tok::Raw { .. } => cur.operand(1),
            _ => {}
        }
    }
    // The final statement's path closes the metric.
    cur.clause_boundary(&mut peak);
    // The accumulated per-level max construction PATH is the expression
    // construction depth.
    let expr_depth = peak;
    let over = max_depth > MAX_FRONTEND_NESTING || expr_depth > MAX_FRONTEND_NESTING;
    if over {
        return Err(StageDiag {
            stage: "admission".into(),
            code: "av.resource.frontend-nesting".into(),
            message: format!(
                "token nesting {} / expression construction path {} exceeds the frozen front-door budget {}",
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
