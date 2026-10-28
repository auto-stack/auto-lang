//! PLAN-818 T-11: the mn CONSUMER of the D-828 function-incremental
//! session. The 828 session owns the frontend/snapshot increment (one
//! whole-module warm-up; each body edit re-checks exactly ONE body) and
//! hands the consumer the changed function's lowered + verified MIR unit
//! (the `PreparedEdit::unit_mir` seam). This layer turns that into the
//! native pipeline's increment: re-emit exactly the changed unit's COFF
//! (bumping only ITS fingerprint/generation), link and run for real, and
//! publish the structural evidence — unchanged units' fingerprints and
//! generations never move, no whole-frontend/whole-lower stage runs, and
//! every degenerate fallback is counted, never silent.

use crate::cache::{self, CoffMeta};
use crate::config::*;
use crate::link_driver::{self, ObjectInput};
use crate::{emit, MnError};
use ac_artifact_probe::store::ManagedStore;
use auto_av_core::mir::OwnerScope;
use auto_av_probe::session::{self, FunctionIncrementalSession};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One unit's identity in the current revision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitEntry {
    pub fp: String,
    pub generation: u64,
}

/// The observable outcome of one incremental edit.
#[derive(Clone, Debug, Default)]
pub struct EditObservation {
    pub ok: bool,
    pub exit_code: Option<i32>,
    /// The single changed function on the fast path.
    pub changed_fn: Option<String>,
    /// Refusal reason (non-fast-path / diagnostics) — nothing changed.
    pub refused_reason: Option<String>,
    /// Infra failure text (the mn pipeline failed around a committed edit).
    pub infra: Option<String>,
    /// mn-side counters for this edit.
    pub codegen_units: u64,
    pub cache_hits: u64,
    /// Degenerate-fallback counter: the edit could not ride the unit seam
    /// and fell back to a WHOLE-module lower (recorded, never silent).
    pub fallback_whole_lower: u64,
    /// F-818-R1-02: a content hit REUSED the historical unit (zero
    /// whole-module work; the generation did not bump).
    pub reused_history_unit: bool,
    /// Session (828) per-edit phase walls (µs) — the structure evidence.
    pub phase_walls_us: BTreeMap<String, u64>,
    /// Session stage-counter deltas of functions OTHER than the changed
    /// one (must be 0 on the fast path) and any whole-module stage delta.
    pub unaffected_stage_delta: u64,
    pub whole_module_stage_delta: u64,
    /// Edit entry → real PE result observed by the parent (ms).
    pub wall_ms: f64,
    /// The round's raw PE stdout/stderr (byte-exact — the trace evidence
    /// F-818-R2-01: authorized trace bytes must survive the incremental
    /// path, never be hardcoded away).
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub struct MnIncrementalSession {
    av: FunctionIncrementalSession,
    entry: String,
    store: ManagedStore,
    /// Declaration order of the units (stable link identity).
    unit_order: Vec<String>,
    units: BTreeMap<String, UnitEntry>,
    owner: OwnerScope,
    workdir_root: PathBuf,
    round: u64,
    /// RESIDENT unit payloads (the D828 residency model): the working set
    /// the link consumes; the store is the persistence layer (cold restore
    /// reads it, warm edits never do).
    resident: BTreeMap<String, Vec<u8>>,
    /// Content-version -> fingerprint per unit (F-818-R1-02): a session
    /// content HIT means the changed function's body content equals a
    /// HISTORICAL revision — the historical UNIT is reused from the A09'
    /// store by this map (zero whole-module work). Seeded at warm-up from
    /// the session's resident content versions; extended at every mint.
    cv_map: BTreeMap<(String, String), String>,
    /// The warm-up capability grants (fallback lowers keep them).
    caps: Vec<String>,
    /// The trace support staticlib for authorized-trace sessions
    /// (F-818-R2-01): every link/run of this session links against it —
    /// `hir.test.mark_a` and friends resolve or the link fails loudly.
    tracelib: Option<PathBuf>,
}

/// Session counter deltas for one edit: the stage work of functions OTHER
/// than the changed one, and any whole-module stage work.
fn structural_deltas(
    before: &BTreeMap<String, u64>,
    after: &BTreeMap<String, u64>,
    changed_name: &str,
) -> (u64, u64) {
    let mut unaffected = 0u64;
    let mut whole = 0u64;
    for (k, v) in after {
        let delta = v.saturating_sub(before.get(k).copied().unwrap_or(0));
        if delta == 0 {
            continue;
        }
        if k.starts_with("whole_module") {
            whole += delta;
        } else if let Some((_, func)) = k.rsplit_once(':') {
            if func != changed_name {
                unaffected += delta;
            }
        }
    }
    (unaffected, whole)
}

/// The 828 session's resident content version of one function (by its
/// SOURCE name — the session's naming convention).
fn av_resident_cv(av: &FunctionIncrementalSession, source_name: &str) -> Option<String> {
    av.resident_content_version(source_name)
}

fn whole_module_units(
    src: &str,
    entry: &str,
    caps: &[String],
) -> Result<Vec<auto_av_core::mir::MirModule>, MnError> {
    let compiled =
        auto_ac_prototype::source::compile_scalar_source(src, "mn.at", caps).map_err(|ds| {
            MnError::Unsupported(
                "frontend",
                ds.first().map(|d| d.message.clone()).unwrap_or_default(),
            )
        })?;
    let mir = auto_mir_lower::lower_checked(
        &compiled.checked,
        entry,
        auto_av_core::contract::fnv1a64(src.as_bytes()),
    )
    .map_err(|e| MnError::Unsupported("adapter", format!("{e:?}")))?;
    let checked = auto_av_core::mir_verify::verify_mir(&mir)
        .map_err(|e| MnError::Unsupported("mir", format!("{e:?}")))?;
    auto_av_probe::cache::split_units(checked.module())
        .map_err(|e| MnError::Backend(format!("unit split: {e}")))
}

impl MnIncrementalSession {
    /// Cold warm-up: the ONE whole-module compile of the session's life —
    /// the 828 session's frontend + the mn whole-module unit layer (every
    /// unit emitted + persisted once; generations start at 0). Returns
    /// the session and the cold run's observation (real link/run).
    pub fn warmup(
        src: &str,
        entry: &str,
        caps: &[String],
        store_root: &Path,
        tracelib: Option<&Path>,
    ) -> Result<(MnIncrementalSession, EditObservation), MnError> {
        let av = session::warmup(src, entry, caps)
            .map_err(|e| MnError::Infra(format!("828 warmup: {e}")))?;
        let store = ManagedStore::open(store_root)
            .map_err(|e| MnError::Infra(format!("store open: {e}")))?;
        let units = whole_module_units(src, entry, caps)?;
        let owner = units.first().expect("units").owner.clone();
        let workdir_root = store_root
            .parent()
            .unwrap_or(store_root)
            .join("mn-inc-work");
        std::fs::create_dir_all(&workdir_root)
            .map_err(|e| MnError::Infra(format!("workdir: {e}")))?;
        let mut s = MnIncrementalSession {
            av,
            entry: entry.to_string(),
            store,
            unit_order: Vec::new(),
            units: BTreeMap::new(),
            owner,
            workdir_root,
            round: 0,
            resident: BTreeMap::new(),
            cv_map: BTreeMap::new(),
            caps: caps.to_vec(),
            tracelib: tracelib.map(|p| p.to_path_buf()),
        };
        for unit in &units {
            let name = unit.funcs[0].name.clone();
            let obj = emit::emit_unit_coff(unit)?;
            let fp = cache::unit_fp(unit);
            let meta = CoffMeta::for_unit(unit, fp.clone())?;
            cache::put_unit_coff(&s.store, None, &meta, &obj.bytes)?;
            // Seed the content-version map: the session's resident
            // content version of this function (by SOURCE name — the unit
            // key is the `d_`-prefixed def id) maps to its warm-up fp.
            let source_name = name.strip_prefix("d_").unwrap_or(&name);
            if let Some(cv) = av_resident_cv(&s.av, source_name) {
                s.cv_map.insert((name.clone(), cv), fp.clone());
            }
            s.unit_order.push(name.clone());
            s.units
                .insert(name.clone(), UnitEntry { fp, generation: 0 });
            s.resident.insert(name, obj.bytes);
        }
        let started = std::time::Instant::now();
        let obs = s.run_current(None, 0, 0, BTreeMap::new(), 0, 0, 0.0)?;
        let _ = started; // warm-up is initialization, single-listed (not a sample)
        Ok((s, obs))
    }

    fn meta_of(&self, name: &str, fp: &str) -> CoffMeta {
        CoffMeta {
            owner: self.owner.clone(),
            unit: name.to_string(),
            symbol: crate::symbols::function_symbol(&self.owner, name),
            fp: fp.to_string(),
            abi: MN_ABI.to_string(),
            target: MN_TARGET.to_string(),
            flags: MN_FLAGS.to_string(),
            backend: MN_BACKEND_TAG.to_string(),
            runtime: MN_RUNTIME_TAG.to_string(),
            mir_schema: MIR_SCHEMA_AXIS.to_string(),
            mir_semantics: MIR_SEMANTICS_AXIS.to_string(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn run_current(
        &mut self,
        changed: Option<&str>,
        codegen_units: u64,
        fallback_whole: u64,
        phase_walls: BTreeMap<String, u64>,
        unaffected_delta: u64,
        whole_module_delta: u64,
        // F-818-R1-01: the observation window is the CONTINUOUS
        // edit-entry -> correct-PE interval; the caller owns the clock.
        wall_ms: f64,
    ) -> Result<EditObservation, MnError> {
        let mut objects = Vec::with_capacity(self.unit_order.len());
        let mut hits = 0u64;
        for name in &self.unit_order {
            match self.resident.get(name) {
                Some(bytes) => {
                    if Some(name.as_str()) != changed {
                        hits += 1;
                    }
                    objects.push(ObjectInput {
                        name: name.clone(),
                        bytes: bytes.clone(),
                    });
                }
                None => {
                    return Err(MnError::Admit(
                        "address",
                        format!("unit `{name}` is not resident (cold restore not in this path)"),
                    ));
                }
            }
        }
        let entry_fp = self.units.get(&self.entry).expect("entry unit").fp.clone();
        let entry_meta = self.meta_of(&self.entry, &entry_fp);
        let wrapper = emit::emit_wrapper_coff(&entry_meta.symbol)?;
        self.round += 1;
        let workdir = self.workdir_root.join(format!("r{}", self.round));
        let run = link_driver::link_and_run(
            &objects,
            &wrapper,
            START_SYMBOL,
            &workdir,
            self.tracelib.as_deref(),
            &format!("incremental round {}\n", self.round),
        )?;
        Ok(EditObservation {
            ok: true,
            exit_code: run.exit_code,
            reused_history_unit: false,
            changed_fn: changed.map(|s| s.to_string()),
            refused_reason: None,
            infra: None,
            codegen_units,
            cache_hits: hits,
            fallback_whole_lower: fallback_whole,
            phase_walls_us: phase_walls,
            unaffected_stage_delta: unaffected_delta,
            whole_module_stage_delta: whole_module_delta,
            wall_ms,
            stdout: run.stdout,
            stderr: run.stderr,
        })
    }

    /// The incremental edit: the 828 session's single-body transaction,
    /// the changed unit's COFF re-emission (or HISTORY REUSE on a content
    /// hit), and a real link/run. A non-fast-path edit (interface change
    /// etc.) is REFUSED with the mn layer untouched. The observation
    /// window (F-818-R1-01) is the CONTINUOUS edit-entry -> real PE result
    /// interval: preparation, the single-body transaction, re-verify,
    /// emission/persist (or historical restore), commit, link and run are
    /// all inside it.
    pub fn edit(&mut self, start: u64, end: u64, insert: Vec<u8>) -> EditObservation {
        let window_started = std::time::Instant::now();
        let wall = |obs: &mut EditObservation| {
            obs.wall_ms = window_started.elapsed().as_secs_f64() * 1000.0;
        };
        let before = self.av.counters().snapshot();
        // The post-edit source, rebuilt locally (prepare does not publish;
        // the session's own `src()` stays pre-edit until commit).
        let new_src = {
            let cur = self.av.src();
            let mut n = String::with_capacity(cur.len());
            n.push_str(&cur[..(start as usize).min(cur.len())]);
            match std::str::from_utf8(&insert) {
                Ok(text) => n.push_str(text),
                Err(_) => {
                    let mut o = EditObservation {
                        refused_reason: Some("insert is not UTF-8".into()),
                        ..Default::default()
                    };
                    wall(&mut o);
                    return o;
                }
            }
            n.push_str(&cur[(end as usize).min(cur.len())..]);
            n
        };
        let prepared = match self.av.prepare_edit(start, end, insert) {
            Err(e) => {
                let mut o = EditObservation {
                    infra: Some(format!("prepare infra: {e}")),
                    ..Default::default()
                };
                wall(&mut o);
                return o;
            }
            Ok(Err(outcome)) => {
                let mut o = EditObservation {
                    refused_reason: outcome
                        .non_fast_path_reason
                        .or_else(|| outcome.diagnostics.first().cloned()),
                    ..Default::default()
                };
                wall(&mut o);
                return o;
            }
            Ok(Ok(p)) => p,
        };
        // The session speaks SOURCE names; the unit layer's keys are the
        // pipeline's def ids (`d_<name>`).
        let changed_name = prepared.changed_name().to_string();
        let changed_key = format!("d_{changed_name}");
        let mut codegen_units = 0u64;
        let mut fallback_whole = 0u64;
        let mut reused_history = false;

        match prepared.unit_mir() {
            Some(unit) => {
                // MINT: mn door re-verifies the exact unit (cheap, single
                // unit); a changed fingerprint re-emits exactly this unit.
                if let Err(e) = auto_av_core::mir_verify::verify_mir(unit) {
                    let mut o = EditObservation {
                        infra: Some(format!("mn re-verify: {e:?}")),
                        ..Default::default()
                    };
                    wall(&mut o);
                    return o;
                }
                let fp = cache::unit_fp(unit);
                let prev = self
                    .units
                    .get(&changed_key)
                    .expect("changed unit known")
                    .clone();
                if fp != prev.fp {
                    let obj = match emit::emit_unit_coff(unit) {
                        Ok(o) => o,
                        Err(e) => {
                            let mut o = EditObservation {
                                infra: Some(format!("emit: {e}")),
                                ..Default::default()
                            };
                            wall(&mut o);
                            return o;
                        }
                    };
                    let meta = self.meta_of(&changed_key, &fp);
                    if let Err(e) = cache::put_unit_coff(&self.store, None, &meta, &obj.bytes) {
                        let mut o = EditObservation {
                            infra: Some(format!("put: {e}")),
                            ..Default::default()
                        };
                        wall(&mut o);
                        return o;
                    }
                    codegen_units = 1;
                    self.units.insert(
                        changed_key.clone(),
                        UnitEntry {
                            fp: fp.clone(),
                            generation: prev.generation + 1,
                        },
                    );
                    self.resident.insert(changed_key.clone(), obj.bytes);
                }
                // COMMIT after the mn layer accepted the unit.
                let outcome = self.av.commit(prepared);
                if !outcome.ok {
                    let mut o = EditObservation {
                        refused_reason: outcome
                            .non_fast_path_reason
                            .or_else(|| outcome.diagnostics.first().cloned()),
                        ..Default::default()
                    };
                    wall(&mut o);
                    return o;
                }
                // The minted content version now maps to the unit's fp (the
                // current identity, whether it just moved or not).
                if let Some(cv) = av_resident_cv(&self.av, &changed_name) {
                    self.cv_map.insert((changed_key.clone(), cv), fp);
                }
            }
            None => {
                // Content HIT (F-818-R1-02): the changed function's body
                // content equals a HISTORICAL revision — the historical
                // UNIT is REUSED from the A09' store via the content-version
                // map. Zero whole-module work; the identity moves back to
                // the historical fp and the generation does NOT bump (no
                // new object was minted).
                let outcome = self.av.commit(prepared);
                if !outcome.ok {
                    let mut o = EditObservation {
                        refused_reason: outcome
                            .non_fast_path_reason
                            .or_else(|| outcome.diagnostics.first().cloned()),
                        ..Default::default()
                    };
                    wall(&mut o);
                    return o;
                }
                let cv = match av_resident_cv(&self.av, &changed_name) {
                    Some(cv) => cv,
                    None => {
                        let mut o = EditObservation {
                            infra: Some("content hit without a resident content version".into()),
                            ..Default::default()
                        };
                        wall(&mut o);
                        return o;
                    }
                };
                match self.cv_map.get(&(changed_key.clone(), cv.clone())).cloned() {
                    Some(fp) => {
                        let meta = self.meta_of(&changed_key, &fp);
                        match cache::get_unit_coff(&self.store, &meta) {
                            Ok(Some(bytes)) => {
                                let generation = self
                                    .units
                                    .get(&changed_key)
                                    .map(|e| e.generation)
                                    .unwrap_or(0);
                                self.resident.insert(changed_key.clone(), bytes);
                                self.units
                                    .insert(changed_key.clone(), UnitEntry { fp, generation });
                                reused_history = true;
                            }
                            other => {
                                let mut o = EditObservation {
                                    infra: Some(format!(
                                        "historical unit restore failed: {other:?}"
                                    )),
                                    ..Default::default()
                                };
                                wall(&mut o);
                                return o;
                            }
                        }
                    }
                    None => {
                        // Deliberate fault-injection seam (V-12.4) or an
                        // anomaly: the content-version map cannot resolve
                        // the hit — a RECORDED whole-module fallback runs
                        // (with the warm-up capability grants), never
                        // silently. The structural gate fires on it.
                        fallback_whole = 1;
                        if let Err(e) = self.whole_lower_fallback(&new_src, &changed_key) {
                            let mut o = EditObservation {
                                infra: Some(e),
                                fallback_whole_lower: fallback_whole,
                                ..Default::default()
                            };
                            wall(&mut o);
                            return o;
                        }
                    }
                }
            }
        }
        // Structural evidence from the session counters (deltas).
        let after = self.av.counters().snapshot();
        let (unaffected, whole) = structural_deltas(&before, &after, &changed_name);
        let phase_walls = self.av.phase_walls().clone();
        let elapsed_ms = window_started.elapsed().as_secs_f64() * 1000.0;
        let mut obs = match self.run_current(
            Some(&changed_key),
            codegen_units,
            fallback_whole,
            phase_walls,
            unaffected,
            whole,
            elapsed_ms,
        ) {
            Ok(mut o) => {
                o.changed_fn = Some(changed_name);
                o
            }
            Err(e) => EditObservation {
                infra: Some(format!("link/run: {e}")),
                changed_fn: Some(changed_name),
                codegen_units,
                fallback_whole_lower: fallback_whole,
                ..Default::default()
            },
        };
        obs.reused_history_unit = reused_history;
        wall(&mut obs);
        obs
    }

    /// A judged RUN of the current revision (no edit): a fresh real
    /// link/run of the resident working set — the formal recipe's judged
    /// trailing step.
    pub fn run_only(&mut self) -> EditObservation {
        let window_started = std::time::Instant::now();
        let mut obs = match self.run_current(
            None,
            0,
            0,
            BTreeMap::new(),
            0,
            0,
            window_started.elapsed().as_secs_f64() * 1000.0,
        ) {
            Ok(o) => o,
            Err(e) => EditObservation {
                infra: Some(format!("link/run: {e}")),
                ..Default::default()
            },
        };
        obs.wall_ms = window_started.elapsed().as_secs_f64() * 1000.0;
        obs
    }

    /// The RECORDED degenerate fallback (deliberate injection or anomaly
    /// only — normal sequences never reach it): re-lower the whole
    /// post-edit module WITH the warm-up capability grants and re-emit
    /// every unit whose fingerprint moved.
    fn whole_lower_fallback(&mut self, new_src: &str, changed_key: &str) -> Result<(), String> {
        let units = whole_module_units(new_src, &self.entry, &self.caps)
            .map_err(|e| format!("fallback whole lower: {e}"))?;
        for unit in &units {
            let name = unit.funcs[0].name.clone();
            let fp = cache::unit_fp(unit);
            let prev = self.units.get(&name).cloned();
            if prev.as_ref().map(|p| p.fp == fp).unwrap_or(false) {
                continue;
            }
            let obj = emit::emit_unit_coff(unit).map_err(|e| format!("fallback emit: {e}"))?;
            let meta = self.meta_of(&name, &fp);
            cache::put_unit_coff(&self.store, None, &meta, &obj.bytes)
                .map_err(|e| format!("fallback put: {e}"))?;
            let generation = prev.map(|p| p.generation + 1).unwrap_or(0);
            self.resident.insert(name.clone(), obj.bytes);
            self.units.insert(name, UnitEntry { fp, generation });
        }
        let _ = changed_key;
        Ok(())
    }

    /// Diagnostics-only views for the tier tests.
    pub fn debug_cv_map(&self) -> &BTreeMap<(String, String), String> {
        &self.cv_map
    }

    pub fn debug_resident_cv(&self, source_name: &str) -> Option<String> {
        av_resident_cv(&self.av, source_name)
    }

    /// Fault-injection seam (V-12.4): forget a unit's content-version
    /// history so a content hit cannot resolve — the NEXT such edit must
    /// take the recorded whole-module fallback (the structural gate
    /// fires). Test-only; never used by the production path.
    pub fn drop_unit_history(&mut self, unit_key: &str) {
        self.cv_map.retain(|(k, _), _| k != unit_key);
    }

    /// The workload's owner scope (identity reconstruction for tests).
    pub fn owner(&self) -> &OwnerScope {
        &self.owner
    }

    /// Current unit identities (fp, generation) — the structure oracle.
    pub fn units(&self) -> &BTreeMap<String, UnitEntry> {
        &self.units
    }

    pub fn unit_order(&self) -> &[String] {
        &self.unit_order
    }

    /// The session's current source (worker edit locator seam).
    pub fn source(&self) -> &str {
        self.av.src()
    }

    pub fn session_revision(&self) -> u64 {
        self.av.revision()
    }

    pub fn close(&mut self) {
        self.av.close();
    }
}
