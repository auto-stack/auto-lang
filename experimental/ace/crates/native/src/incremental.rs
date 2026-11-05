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
use crate::source_diagnostics as sd;
use crate::{emit, MnError};
use ace_artifacts::store::ManagedStore;
use ace_av::session::{self, FunctionIncrementalSession};
use ace_av_core::mir::OwnerScope;
use ace_core::body_work::BodyWork;
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
    /// PLAN-911: the typed refusal (kind, producer phase, and for language
    /// diagnostics the operation-owned snapshot). `refused_reason` is its
    /// display projection.
    pub refusal: Option<sd::EditRefusal>,
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
    /// PLAN-911 V-17: computed on EVERY exit, refusals included.
    pub unaffected_stage_delta: u64,
    pub whole_module_stage_delta: u64,
    /// PLAN-911 V-17: the core producer's own per-function body work for
    /// this edit (resolve/typecheck/plan/bind walks), captured around the
    /// real prepare/commit calls. Unchanged functions must be all-zero on
    /// every path; a reached-but-failed body shows its attempt here.
    pub body_work: BTreeMap<String, BodyWork>,
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

/// PLAN-911 V-17: what an edit exit needs to report honestly — the counter
/// baseline at entry, the core producer's body-work window(s), the
/// attribution of the changed body (when known) and the window clock.
struct ExitCtx {
    window_started: std::time::Instant,
    before: BTreeMap<String, u64>,
    changed_name: Option<String>,
    body_work: BTreeMap<String, BodyWork>,
}

/// Sum a second capture window into the first (prepare + commit windows).
fn merge_body_work(dst: &mut BTreeMap<String, BodyWork>, more: BTreeMap<String, BodyWork>) {
    for (name, w) in more {
        let e = dst.entry(name).or_default();
        e.resolve += w.resolve;
        e.typecheck += w.typecheck;
        e.plan += w.plan;
        e.bind += w.bind;
    }
}

impl MnIncrementalSession {
    /// The session's link support libraries. PLAN-870 boundary: the
    /// incremental session remains scalar-profile (its 828 session and
    /// frontend are scalar); owned-source incremental lands with the T-05
    /// integration once D862/H release — not silently half-wired here.
    fn support_libs(&self) -> Vec<PathBuf> {
        self.tracelib.iter().cloned().collect()
    }
}

fn whole_module_units(
    src: &str,
    entry: &str,
    caps: &[String],
    operation: sd::Operation,
) -> Result<Vec<ace_av_core::mir::MirModule>, MnError> {
    // PLAN-934 T-08: the ONE scalar cold compile goes through the SQ-A
    // fact entry (query facts are the type authority); a frontend failure
    // is still the operation's typed snapshot.
    let compiled = ace_av::query_entry::compile_scalar_source_via_facts(src, "mn.at", caps)
        .map_err(|ds| {
            sd::mn_error_from_compile_failure(
                src,
                "mn.at",
                sd::CompileFailure {
                    profile: sd::SourceProfile::Scalar,
                    attempted: vec![sd::SourceProfile::Scalar],
                    surface_rejections: Vec::new(),
                    diagnostics: ds,
                },
                operation,
            )
        })?;
    let mir = ace_lowering::lower_checked(
        &compiled.checked,
        entry,
        ace_av_core::contract::fnv1a64(src.as_bytes()),
    )
    .map_err(|e| MnError::Unsupported("adapter", format!("{e:?}")))?;
    let checked = ace_av_core::mir_verify::verify_mir(&mir)
        .map_err(|e| MnError::Unsupported("mir", format!("{e:?}")))?;
    ace_av::cache::split_units(checked.module())
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
        // PLAN-911: the mn unit compile goes FIRST so a real frontend
        // rejection of the warm-up source is this operation's typed
        // snapshot (the 828 session compiles the same source through the
        // same shared frontend and would only re-render it as infra text).
        let units = whole_module_units(src, entry, caps, sd::Operation::cold(src))?;
        let av = session::warmup(src, entry, caps)
            .map_err(|e| MnError::Infra(format!("828 warmup: {e}")))?;
        let store = ManagedStore::open(store_root)
            .map_err(|e| MnError::Infra(format!("store open: {e}")))?;
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
            &self.support_libs(),
            &format!("incremental round {}\n", self.round),
        )?;
        Ok(EditObservation {
            ok: true,
            exit_code: run.exit_code,
            reused_history_unit: false,
            changed_fn: changed.map(|s| s.to_string()),
            refused_reason: None,
            refusal: None,
            infra: None,
            codegen_units,
            cache_hits: hits,
            fallback_whole_lower: fallback_whole,
            phase_walls_us: phase_walls,
            unaffected_stage_delta: unaffected_delta,
            whole_module_stage_delta: whole_module_delta,
            body_work: BTreeMap::new(),
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
    ///
    /// PLAN-911: every exit — refusal, infra, success — closes through
    /// [`Self::close_exit`], so the structural deltas and the core
    /// producer's body-work window are measured on failures too (a zero is
    /// an observed zero, never a `Default` left behind by an early return),
    /// and a refusal carries its typed kind/snapshot from the session's own
    /// `EditOutcome`.
    pub fn edit(&mut self, start: u64, end: u64, insert: Vec<u8>) -> EditObservation {
        let mut ctx = ExitCtx {
            window_started: std::time::Instant::now(),
            before: self.av.counters().snapshot(),
            changed_name: None,
            body_work: BTreeMap::new(),
        };
        let base_src = self.av.src().to_string();
        let base_revision = self.av.revision();
        let insert_len = insert.len() as u64;
        // The post-edit source, rebuilt locally (prepare does not publish;
        // the session's own `src()` stays pre-edit until commit).
        let new_src = {
            let cur = &base_src;
            let mut n = String::with_capacity(cur.len());
            n.push_str(&cur[..(start as usize).min(cur.len())]);
            match std::str::from_utf8(&insert) {
                Ok(text) => n.push_str(text),
                Err(_) => {
                    let reason = "insert is not UTF-8".to_string();
                    return self.close_exit(
                        &ctx,
                        EditObservation {
                            refused_reason: Some(reason.clone()),
                            refusal: Some(sd::EditRefusal {
                                kind: sd::RefusalKind::InvalidEdit,
                                phase: None,
                                source_error: None,
                                reason,
                            }),
                            ..Default::default()
                        },
                    );
                }
            }
            n.push_str(&cur[(end as usize).min(cur.len())..]);
            n
        };
        // The producer's own body-work window wraps the REAL prepare call
        // (W-S2): reached-but-failed bodies show their attempt here.
        let (prepared, work) =
            ace_core::body_work::capture_body_work(|| self.av.prepare_edit(start, end, insert));
        ctx.body_work = work;
        let prepared = match prepared {
            Err(e) => {
                return self.close_exit(
                    &ctx,
                    EditObservation {
                        infra: Some(format!("prepare infra: {e}")),
                        ..Default::default()
                    },
                );
            }
            Ok(Err(outcome)) => {
                ctx.changed_name = outcome
                    .rejection
                    .as_ref()
                    .and_then(|r| r.changed_fn.clone());
                let refusal = Self::typed_refusal(
                    &outcome,
                    &base_src,
                    &new_src,
                    base_revision,
                    (start, end, insert_len),
                );
                return self.close_exit(
                    &ctx,
                    EditObservation {
                        refused_reason: Some(refusal.reason.clone()),
                        refusal: Some(refusal),
                        ..Default::default()
                    },
                );
            }
            Ok(Ok(p)) => p,
        };
        // The session speaks SOURCE names; the unit layer's keys are the
        // pipeline's def ids (`d_<name>`).
        let changed_name = prepared.changed_name().to_string();
        ctx.changed_name = Some(changed_name.clone());
        let changed_key = format!("d_{changed_name}");
        let mut codegen_units = 0u64;
        let mut fallback_whole = 0u64;
        let mut reused_history = false;

        match prepared.unit_mir() {
            Some(unit) => {
                // MINT: mn door re-verifies the exact unit (cheap, single
                // unit); a changed fingerprint re-emits exactly this unit.
                if let Err(e) = ace_av_core::mir_verify::verify_mir(unit) {
                    return self.close_exit(
                        &ctx,
                        EditObservation {
                            infra: Some(format!("mn re-verify: {e:?}")),
                            ..Default::default()
                        },
                    );
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
                            return self.close_exit(
                                &ctx,
                                EditObservation {
                                    infra: Some(format!("emit: {e}")),
                                    ..Default::default()
                                },
                            );
                        }
                    };
                    let meta = self.meta_of(&changed_key, &fp);
                    if let Err(e) = cache::put_unit_coff(&self.store, None, &meta, &obj.bytes) {
                        return self.close_exit(
                            &ctx,
                            EditObservation {
                                infra: Some(format!("put: {e}")),
                                ..Default::default()
                            },
                        );
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
                let (outcome, commit_work) =
                    ace_core::body_work::capture_body_work(|| self.av.commit(prepared));
                merge_body_work(&mut ctx.body_work, commit_work);
                if !outcome.ok {
                    let refusal = Self::typed_refusal(
                        &outcome,
                        &base_src,
                        &new_src,
                        base_revision,
                        (start, end, insert_len),
                    );
                    return self.close_exit(
                        &ctx,
                        EditObservation {
                            refused_reason: Some(refusal.reason.clone()),
                            refusal: Some(refusal),
                            ..Default::default()
                        },
                    );
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
                let (outcome, commit_work) =
                    ace_core::body_work::capture_body_work(|| self.av.commit(prepared));
                merge_body_work(&mut ctx.body_work, commit_work);
                if !outcome.ok {
                    let refusal = Self::typed_refusal(
                        &outcome,
                        &base_src,
                        &new_src,
                        base_revision,
                        (start, end, insert_len),
                    );
                    return self.close_exit(
                        &ctx,
                        EditObservation {
                            refused_reason: Some(refusal.reason.clone()),
                            refusal: Some(refusal),
                            ..Default::default()
                        },
                    );
                }
                let cv = match av_resident_cv(&self.av, &changed_name) {
                    Some(cv) => cv,
                    None => {
                        return self.close_exit(
                            &ctx,
                            EditObservation {
                                infra: Some(
                                    "content hit without a resident content version".into(),
                                ),
                                ..Default::default()
                            },
                        );
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
                                return self.close_exit(
                                    &ctx,
                                    EditObservation {
                                        infra: Some(format!(
                                            "historical unit restore failed: {other:?}"
                                        )),
                                        ..Default::default()
                                    },
                                );
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
                        if let Err(e) = self.whole_lower_fallback(&base_src, &new_src, &changed_key)
                        {
                            return self.close_exit(
                                &ctx,
                                EditObservation {
                                    infra: Some(e),
                                    fallback_whole_lower: fallback_whole,
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            }
        }
        // Structural evidence from the session counters (deltas).
        let after = self.av.counters().snapshot();
        let (unaffected, whole) = structural_deltas(&ctx.before, &after, &changed_name);
        let phase_walls = self.av.phase_walls().clone();
        let elapsed_ms = ctx.window_started.elapsed().as_secs_f64() * 1000.0;
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
        self.close_exit(&ctx, obs)
    }

    /// PLAN-911: finish ANY edit exit with measured facts — structural
    /// deltas against the entry baseline, the body-work window, the
    /// session's phase walls, and the window clock. No exit leaves zeros
    /// it did not observe.
    fn close_exit(&self, ctx: &ExitCtx, mut obs: EditObservation) -> EditObservation {
        let after = self.av.counters().snapshot();
        let (unaffected, whole) = structural_deltas(
            &ctx.before,
            &after,
            ctx.changed_name.as_deref().unwrap_or(""),
        );
        obs.unaffected_stage_delta = unaffected;
        obs.whole_module_stage_delta = whole;
        obs.body_work = ctx.body_work.clone();
        if obs.phase_walls_us.is_empty() {
            obs.phase_walls_us = self.av.phase_walls().clone();
        }
        obs.wall_ms = ctx.window_started.elapsed().as_secs_f64() * 1000.0;
        obs
    }

    /// PLAN-911: the typed refusal of one edit straight from the session's
    /// own `EditOutcome` — kind/phase from the enum, the snapshot built from
    /// the exact typed diagnostics (never re-parsed from the rendered text,
    /// never re-compiled). The candidate identity is the post-edit source
    /// the compiler actually saw; the base is the session's accepted prefix.
    fn typed_refusal(
        outcome: &session::EditOutcome,
        base_src: &str,
        new_src: &str,
        base_revision: u64,
        region: (u64, u64, u64),
    ) -> sd::EditRefusal {
        let reason = outcome
            .non_fast_path_reason
            .clone()
            .or_else(|| outcome.diagnostics.first().cloned())
            .unwrap_or_default();
        let Some(rej) = &outcome.rejection else {
            // The scalar session types every refusal; an untyped one is
            // reported as such (a consumer in strict mode treats it as an
            // error, never as a language family).
            return sd::EditRefusal {
                kind: sd::RefusalKind::Untyped,
                phase: None,
                source_error: None,
                reason,
            };
        };
        let phase = rej.phase.map(sd::RejectPhase::from);
        if rej.kind != session::RefusalKind::LanguageDiagnostics {
            return sd::EditRefusal {
                kind: rej.kind.into(),
                phase,
                source_error: None,
                reason,
            };
        }
        let failure = sd::CompileFailure {
            profile: sd::SourceProfile::Scalar,
            attempted: vec![sd::SourceProfile::Scalar],
            surface_rejections: Vec::new(),
            diagnostics: rej.typed_diagnostics.clone(),
        };
        let (start, end, insert_len) = region;
        let operation = sd::Operation::warm_edit(
            base_src,
            new_src,
            base_revision,
            sd::EditRegion {
                start,
                end,
                insert_len,
            },
        );
        // The 828 session renders against `probe.at`: that IS the path the
        // producer received for this candidate.
        match sd::SourceDiagnosticSnapshot::capture(new_src, "probe.at", failure, operation) {
            Ok(snap) => sd::EditRefusal {
                kind: sd::RefusalKind::LanguageDiagnostics,
                phase,
                source_error: Some(snap),
                reason,
            },
            Err(e) => sd::EditRefusal {
                kind: sd::RefusalKind::Resource,
                phase,
                source_error: None,
                reason: format!("{reason} [{e}]"),
            },
        }
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
    fn whole_lower_fallback(
        &mut self,
        base_src: &str,
        new_src: &str,
        changed_key: &str,
    ) -> Result<(), String> {
        let units = whole_module_units(
            new_src,
            &self.entry,
            &self.caps,
            sd::Operation::fallback_cold(base_src, new_src),
        )
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
