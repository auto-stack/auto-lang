//! PLAN-820 T-09 (F820-R1-01/R2-01 repair): the data profile's
//! function-incremental consumer over the 828 PUBLIC compile/load/snapshot
//! interfaces — a REAL full-chain increment with transactional installs.
//!
//! Fast path (a pure body edit — signature/type/declaration set/caps
//! unchanged): every production stage is bounded to the CHANGED function.
//! 1. a byte-level segment diff (common prefix/suffix) names the ONE
//!    function declaration the edit touches; anything else (zero, several,
//!    a type item, a declaration-set change) is an explicit non-fast
//!    rejection the caller routes to [`apply_source`];
//! 2. a masked single-def parse document — every OTHER function's source
//!    segment replaced by an equal-length line-comment block, byte offsets
//!    preserved — is parsed: the parse stage touches exactly ONE function
//!    (the fn-item count in the evidence is the MEASURED parse result);
//! 3. `prepare_candidate_document` runs the 828 single-body transaction
//!    against the RETAINED checked environment: recheck (resolve +
//!    typecheck of ONE body; signature drift is a controlled refusal) and
//!    single-body emission with body-less scaffolds. The PRODUCER attests
//!    the scope (`BodyEmitReceipt`: 1 real body, n−1 scaffolds) and the
//!    `body_work` window records per-function resolve/typecheck/plan/bind
//!    counts — unchanged functions contribute ZERO body work;
//! 4. the changed def alone lowers/verifies/admits as its av.data/1 unit.
//! Every stage/IO axis is metered AT the call site that performs the work
//! ([`ProductionEvents`]) — no constant "zero evidence"; the
//! counter-control seams (`set_control_disk_read`,
//! `set_control_reverify_def`) write through the SAME recording entries
//! mid-edit and the structural gate reddens, proving the meter is live.
//!
//! F820-R2-01 (transactional installs): a candidate publishes ONLY after
//! admission, link, the candidate RUN, and the structural gate ALL succeed
//! ("失败/取消/迟到不安装"). A runtime trap (e.g. division by zero), a link
//! rejection, a gate violation or a commit failure returns `Err` with the
//! session's source/environment/HIR/code/snapshot/revision/caps exactly at
//! their pre-failure values — the next legal edit proceeds normally. The
//! same build-aside/run-before-publish shape holds for the sibling
//! entries ([`apply_source`], [`apply_caps`]).
//!
//! Unchanged functions keep their resident HIR body Arc, their PRODUCING
//! bundle Arc (the frontend's own output object — not a consumer-side
//! clone) and their FunctionCode Arc BY IDENTITY (pointer + mint
//! generation), zero production events, zero IO.
//! [`DataIncrementalSession::increment_gate`] enforces exactly these
//! invariants INSIDE the production path — a regression to whole-frontend
//! edits reddens the edit itself (and with it the formal script tier).
//! Subcase-4 counter-controls ([`forced_whole_frontend_edit`],
//! [`forced_hash_only_replacement`]) drive the SAME gate over deliberately
//! degraded edits and expose the violations.
//!
//! Non-fast path ([`apply_source`]): a type-table / signature /
//! declaration-set change is a bounded re-warmup whose REBUILD SET is
//! computed from actual dependency facts extracted from the checked HIR
//! (record mentions through locals/places/field paths/constructors, closed
//! over record nesting) plus cross-bundle SIGNATURE shape equality — a
//! def whose signature CONTENT changed invalidates every unit (each unit
//! imports every def with its signature; the snapshot link's interning
//! gate owns that dependency), while a def whose BODY did not change
//! keeps its resident HIR Arc. A pure helper-BODY change never re-lowers
//! its callers. [`apply_source_partial`] keeps the explicit partial-rebuild
//! counter-control (the cross-revision link hazard) and [`apply_caps`] the
//! authorization axis (capabilities embed in every unit, so any caps delta
//! re-lowers all units while HIR identity is untouched).
//!
//! Verification ownership on the fast path: the changed body's REAL
//! resolve/typecheck is the record domain's semantic gate (inside the
//! recheck transaction) and the data profile's own MIR/ABC verifiers gate
//! execution (`verify_mir` + `verify_abc_bytes`); the whole-module
//! `verify_record` credential runs at cold warmup and at every
//! `apply_source` revision (the record domain owns whole-module
//! credentials — per-function HIR verification exists for scalar/core
//! only, which is why unchanged bodies are never re-verified here).

use auto_ac_prototype::body_work::{self, BodyWork};
use auto_ac_prototype::hir;
use auto_ac_prototype::source::ast::{FnItem, Item};
use auto_ac_prototype::source::{
    parse_program_with, prepare_candidate_document, resolve_env, BodyEmitReceipt, CheckedEnv,
    SourceProfile,
};
use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute_program_units, CancelFlag, ExecLimits};
use auto_av_core::contract::{fnv1a64, ExecOutcome, Value};
use auto_av_core::function_code::FunctionCode;
use auto_av_core::mir_verify::verify_mir;
use auto_av_core::program_snapshot::ProgramSnapshot;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// One function's resident code identity evidence (V-13 observability).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodeIdentity {
    pub ptr: u64,
    pub generation: u64,
}

/// One function's resident HIR identity evidence. `body_ptr` is the
/// resident per-def body Arc the session materialized; `bundle_ptr` is
/// the PRODUCING bundle's Arc pointer — the frontend's own output object
/// (the warm-up whole bundle, or the single-def candidate bundle of the
/// edit that last changed this function), NOT a consumer-side clone.
/// Content equality passes neither axis: a "reuse" that re-materializes
/// an equal-content object is a degradation the gate reddens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HirIdentity {
    pub body_ptr: u64,
    pub bundle_ptr: u64,
}

/// Real production-event meter for ONE edit window (the F820-R1-01 close):
/// every counter is incremented AT the call site performing the work —
/// constants never stand in for raw evidence. The disk axes exist so any
/// future IO added to the edit path has to flow through the gate; the
/// counter-control seams prove they are live.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProductionEvents {
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    /// per-def ABC decode executions (`verify_abc_bytes` decodes then
    /// verifies — one call records one decode AND one verify event).
    pub decode: BTreeMap<String, u64>,
    /// per-def MIR verification executions.
    pub verify_mir: BTreeMap<String, u64>,
    /// per-def ABC verification executions.
    pub verify_abc: BTreeMap<String, u64>,
    /// per-def FunctionCode admission executions.
    pub admit: BTreeMap<String, u64>,
    /// MEASURED fn-item count of the edit's parse document (from the
    /// parse result itself, never a literal).
    pub parse_fn_items: usize,
}

impl ProductionEvents {
    fn bump(map: &mut BTreeMap<String, u64>, def: &str) {
        *map.entry(def.to_string()).or_insert(0) += 1;
    }

    /// The recording entry for a disk read (F820-R1-01: no constant
    /// zeros — nothing in-tree calls this on the edit path today, and the
    /// gate reddens the moment anything does).
    pub fn record_disk_read(&mut self, bytes: u64) {
        self.disk_read_bytes += bytes;
    }

    /// The recording entry for a disk write.
    pub fn record_disk_write(&mut self, bytes: u64) {
        self.disk_write_bytes += bytes;
    }
}

/// The producer-observed facts of one edit window (the V-13 evidence the
/// structural gate judges). `body_work` keys are SOURCE function names
/// (the producer records `item.name.name`); the event maps key def ids.
#[derive(Clone, Debug)]
pub struct EditFacts {
    pub changed_name: String,
    pub body_work: BTreeMap<String, BodyWork>,
    /// The emission receipt: 1 real body, n−1 scaffolds on the fast path.
    pub receipt: BodyEmitReceipt,
    /// Metered production events (recorded at the real call sites).
    pub events: ProductionEvents,
}

/// The result of one function-level body edit.
#[derive(Clone, Debug)]
pub struct EditReport {
    pub changed_def: String,
    pub facts: EditFacts,
    /// Lower counters of every function AFTER the edit (cumulative).
    pub lowered: BTreeMap<String, u64>,
    pub identity_after: BTreeMap<String, CodeIdentity>,
    pub hir_identity_after: BTreeMap<String, HirIdentity>,
}

struct OutlineEntry {
    is_fn: bool,
    name: String,
    start: usize,
    end: usize,
}

/// The data-profile incremental session.
pub struct DataIncrementalSession {
    src: String,
    entry: String,
    caps: Vec<String>,
    /// The RETAINED checked environment (828 seam): the hot path's
    /// single-body recheck/emission transaction runs against it and only
    /// `FnCandidate::commit` publishes into it.
    env: CheckedEnv,
    /// Source outline (item kind/name/byte span) of `src`.
    outline: Vec<OutlineEntry>,
    /// Function def ids in source order == environment slot order.
    fn_order: Vec<String>,
    /// The current revision's whole-module bundle (cold credential input
    /// and the lower source for every def without a per-def override).
    bundle: Arc<hir::Bundle>,
    /// Per-def PRODUCING bundle overrides (Arc identity axis): set for a
    /// def whose latest HIR came from a fast-path edit.
    def_bundle: BTreeMap<String, Arc<hir::Bundle>>,
    /// Resident per-def HIR bodies, one Arc each.
    hir: BTreeMap<String, Arc<hir::Body>>,
    code: BTreeMap<String, Arc<FunctionCode>>,
    snapshot: Arc<ProgramSnapshot>,
    revision: u64,
    /// Per-def lower counters ("lower:<def>"; the structural axis).
    counters: BTreeMap<String, u64>,
    /// The warmup's own producer body work (cold evidence; the gate only
    /// ever judges edit windows).
    cold_body_work: BTreeMap<String, BodyWork>,
    /// F820-R1-01 counter-control seam A: when set, the NEXT edit records
    /// a simulated disk read of N bytes through the real metering entry
    /// mid-edit — the gate must redden. Consumed once.
    control_disk_read: Option<u64>,
    /// F820-R1-01 counter-control seam B: when set to a def id, the NEXT
    /// edit records a simulated re-verify (verify_mir) event for that
    /// (unchanged) def through the real metering entry — the gate must
    /// redden. Consumed once.
    control_reverify_def: Option<String>,
}

impl DataIncrementalSession {
    /// Warm up over a record-domain source: front-door admission, the full
    /// cold credential (`compile_record_source` = parse + resolve +
    /// typecheck + emit + bind + whole-module verify), the retained
    /// environment, per-def resident HIR Arcs, per-def lower/admit, link
    /// and one cold run.
    pub fn warmup(src: &str, entry: &str, caps: &[String]) -> Result<Self, String> {
        crate::driver::front_door_admit(src).map_err(|d| d.message)?;
        let (compiled, cold_work) = body_work::capture_body_work(|| {
            auto_ac_prototype::source::compile_record_source(src, "sess.at", caps)
        });
        let compiled = compiled.map_err(|ds| {
            ds.first()
                .map(|d| format!("frontend: {} {}", d.code, d.message))
                .unwrap_or_else(|| "frontend failed".to_string())
        })?;
        let bundle = Arc::new(compiled.checked.bundle().clone());
        let program = parse_program_with(src, SourceProfile::Record)
            .map_err(|d| format!("outline parse: {}", d.message))?;
        let outline = outline_of(&program)?;
        let env = resolve_env(&program, caps, SourceProfile::Record).map_err(|ds| {
            ds.first()
                .map(|d| format!("resolve env: {} {}", d.code, d.message))
                .unwrap_or_else(|| "resolve env failed".to_string())
        })?;
        let fn_order: Vec<String> = outline
            .iter()
            .filter(|o| o.is_fn)
            .map(|o| format!("d_{}", o.name))
            .collect();
        if !fn_order.iter().any(|n| n == entry) {
            return Err(format!("entry def `{entry}` not found"));
        }
        // Bind the consumer's slot mapping once: environment slots are the
        // source fn order.
        let env_names: Vec<String> = auto_ac_prototype::source::env_fn_names(&env)
            .iter()
            .map(|s| s.to_string())
            .collect();
        let order_names: Vec<String> = outline
            .iter()
            .filter(|o| o.is_fn)
            .map(|o| o.name.clone())
            .collect();
        if env_names != order_names {
            return Err(format!(
                "environment slots {env_names:?} != source fn order {order_names:?}"
            ));
        }
        let mut sess = DataIncrementalSession {
            src: src.to_string(),
            entry: entry.to_string(),
            caps: caps.to_vec(),
            env,
            outline,
            fn_order,
            bundle,
            def_bundle: BTreeMap::new(),
            hir: BTreeMap::new(),
            code: BTreeMap::new(),
            snapshot: Arc::new(ProgramSnapshot::vacant(entry, 0)),
            revision: 0,
            counters: BTreeMap::new(),
            cold_body_work: cold_work,
            control_disk_read: None,
            control_reverify_def: None,
        };
        for def in sess.fn_order.clone() {
            sess.install_cold_unit(&def)?;
        }
        sess.relink()?;
        sess.run().map(|_| ())?;
        Ok(sess)
    }

    /// Cold lower/admit of one def from the whole bundle (warmup and
    /// `apply_source` rebuild path).
    fn install_cold_unit(&mut self, def: &str) -> Result<(), String> {
        let bundle = self.bundle.clone();
        let mut ev = ProductionEvents::default();
        let (body_arc, code) = self.lower_and_admit(&bundle, def, 1, &self.src.clone(), &mut ev)?;
        self.hir.insert(def.to_string(), body_arc);
        self.code.insert(def.to_string(), code);
        *self.counters.entry(format!("lower:{def}")).or_insert(0) += 1;
        Ok(())
    }

    /// Lower + MIR-verify + ABC-encode/verify + admit ONE def out of a
    /// bundle that contains it, metering every stage event INTO `ev` at
    /// the call site that performs it. Returns the def's new HIR Arc
    /// (extracted from the same bundle) and its new FunctionCode.
    /// `src_for_hash` is the CURRENT source text (the unit's source
    /// snapshot identity).
    fn lower_and_admit(
        &self,
        bundle: &hir::Bundle,
        def: &str,
        mint: u64,
        src_for_hash: &str,
        ev: &mut ProductionEvents,
    ) -> Result<(Arc<hir::Body>, Arc<FunctionCode>), String> {
        let m = bundle.modules.first().ok_or("bundle has no module")?;
        let di = m
            .defs
            .iter()
            .position(|d| d.id_text == def)
            .ok_or_else(|| format!("def `{def}` missing in bundle"))?;
        let body_id = match &m.defs[di].kind {
            hir::DefKind::Function { body, .. } => *body,
            _ => return Err(format!("def `{def}` is not a plain function")),
        };
        let body_arc = Arc::new(
            m.bodies
                .get(body_id.0 as usize)
                .ok_or_else(|| format!("body of `{def}` out of range"))?
                .clone(),
        );
        let mir = crate::hir_adapter::lower_function_data_unit(
            bundle,
            def,
            &self.caps,
            fnv1a64(src_for_hash.as_bytes()),
        )
        .map_err(|e| format!("lower {def}: {}", e.code()))?;
        verify_mir(&mir).map_err(|e| format!("verify {def}: {:?}", e))?;
        ProductionEvents::bump(&mut ev.verify_mir, def);
        let wire = encode_abc(&mir).map_err(|e| format!("encode {def}: {}", e.code()))?;
        let loaded = verify_abc_bytes(&wire.to_flat(), &mir.owner)
            .map_err(|e| format!("admit {def}: {e}"))?;
        // verify_abc_bytes decodes the wire and verifies the decoded
        // module — one call is one decode AND one verify event.
        ProductionEvents::bump(&mut ev.decode, def);
        ProductionEvents::bump(&mut ev.verify_abc, def);
        let code =
            FunctionCode::admit_unit(loaded, mint).map_err(|e| format!("mint {def}: {e}"))?;
        ProductionEvents::bump(&mut ev.admit, def);
        Ok((body_arc, Arc::new(code)))
    }

    /// The bundle holding `def`'s CURRENT body (per-def override first,
    /// the whole bundle otherwise).
    fn bundle_for(&self, def: &str) -> Arc<hir::Bundle> {
        self.def_bundle
            .get(def)
            .cloned()
            .unwrap_or_else(|| self.bundle.clone())
    }

    // ------------------------------------------------------------------
    // Fast path: the function-level body edit (transactional)
    // ------------------------------------------------------------------

    /// Apply a whole-source replacement that only changes ONE function's
    /// body. Every production stage is bounded to that function and the
    /// structural gate enforces it in-path. PUBLICATION IS TRANSACTIONAL
    /// (F820-R2-01): admission, link, the candidate RUN and the gate must
    /// ALL succeed before anything installs; any failure (non-fast
    /// rejection, candidate diagnostics, link rejection, runtime trap,
    /// gate violation, commit refusal) returns `Err` with the session
    /// exactly at its pre-edit values.
    pub fn edit_body(&mut self, new_src: &str, changed_def: &str) -> Result<EditReport, String> {
        let before_code = self.identity_all();
        let before_hir = self.hir_identity_all();

        // 1. The single-def parse plan (segment diff + masked document);
        //    the fn-item count comes back MEASURED from the parse.
        let (item, changed_span, parse_fn_items) =
            self.plan_single_def_item(new_src, changed_def)?;
        let changed_name = changed_def
            .strip_prefix("d_")
            .unwrap_or(changed_def)
            .to_string();
        let idx = self
            .fn_order
            .iter()
            .position(|d| d == changed_def)
            .ok_or_else(|| format!("`{changed_def}` is not a resident function"))?;

        // 2. Input front door on the REAL new source (frozen budget; the
        //    same whole-text admission every 828 edit runs).
        crate::driver::front_door_admit(new_src).map_err(|d| d.message)?;

        // 3. The 828 single-body transaction under the producer window.
        let (cand, work) = body_work::capture_body_work(|| {
            prepare_candidate_document(&mut self.env, &item, idx, new_src, "sess.at")
        });
        let candidate = cand.map_err(|ds| {
            format!(
                "candidate rejected: {}",
                ds.first()
                    .map(|d| format!("{} {}", d.code, d.message))
                    .unwrap_or_else(|| "no diagnostics".to_string())
            )
        })?;
        let receipt = candidate.receipt();
        let cand_bundle = Arc::new(candidate.parts().bundle.clone());

        // 4. Lower/admit the changed def alone, metering into the window's
        //    event recorder (starts from the MEASURED parse count).
        let mut ev = ProductionEvents {
            parse_fn_items,
            ..Default::default()
        };
        let mint = self
            .code
            .get(changed_def)
            .map(|c| c.generation() + 1)
            .unwrap_or(1);
        let (new_body_arc, new_code) =
            self.lower_and_admit(&cand_bundle, changed_def, mint, new_src, &mut ev)?;

        // Counter-control seams (F820-R1-01 close): an injected event
        // flows through the REAL recording entry mid-edit; the gate below
        // must redden. Consumed once either way.
        if let Some(bytes) = self.control_disk_read.take() {
            ev.record_disk_read(bytes);
        }
        if let Some(def) = self.control_reverify_def.take() {
            ProductionEvents::bump(&mut ev.verify_mir, &def);
        }

        // 5. Build the candidate state ASIDE — nothing below installs
        //    until every check has passed.
        let mut new_hir = self.hir.clone();
        new_hir.insert(changed_def.to_string(), new_body_arc);
        let mut new_code_map = self.code.clone();
        new_code_map.insert(changed_def.to_string(), new_code);
        let mut new_def_bundle = self.def_bundle.clone();
        new_def_bundle.insert(changed_def.to_string(), cand_bundle);
        let code_vec: Vec<Arc<FunctionCode>> = new_code_map.values().cloned().collect();
        let new_snap = Arc::new(ProgramSnapshot::link(
            code_vec,
            &self.entry,
            self.revision + 1,
        )?);

        // 6. RUN the candidate snapshot BEFORE publishing (F820-R2-01):
        //    a runtime trap rejects the edit, it never installs the bad
        //    program as the session's current state.
        self.run_snapshot(&new_snap)?;

        // 7. The structural gate over the aside state and the meter.
        let after_code = code_identity_of(&new_code_map);
        let after_hir = hir_identity_of(&new_hir, &new_def_bundle, &self.bundle);
        let facts = EditFacts {
            changed_name: changed_name.clone(),
            body_work: work,
            receipt,
            events: ev,
        };
        let violations = Self::increment_gate(
            changed_def,
            &self.fn_order,
            &facts,
            &before_hir,
            &before_code,
            &after_hir,
            &after_code,
        );
        if !violations.is_empty() {
            return Err(format!(
                "increment gate RED — the edit degraded to non-incremental production: {}",
                violations.join("; ")
            ));
        }

        // 8. Publish: the environment entry first (a commit refusal
        //    installs nothing), then the resident swap.
        candidate.commit(&mut self.env).map_err(|ds| {
            ds.first()
                .map(|d| format!("commit: {} {}", d.code, d.message))
                .unwrap_or_else(|| "commit failed".to_string())
        })?;
        let delta = new_src.len() as i64 - self.src.len() as i64;
        self.shift_outline_after(changed_def, delta);
        if let Some(e) = self
            .outline
            .iter_mut()
            .find(|o| o.is_fn && format!("d_{}", o.name) == changed_def)
        {
            e.start = changed_span.0;
            e.end = changed_span.1;
        }
        self.src = new_src.to_string();
        self.hir = new_hir;
        self.code = new_code_map;
        self.def_bundle = new_def_bundle;
        self.revision += 1;
        self.snapshot = new_snap;
        *self
            .counters
            .entry(format!("lower:{changed_def}"))
            .or_insert(0) += 1;

        Ok(EditReport {
            changed_def: changed_def.to_string(),
            facts,
            lowered: self.counters.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            identity_after: after_code,
            hir_identity_after: after_hir,
        })
    }

    /// The V-13 structural gate: the invariants a function-level edit must
    /// hold, judged over producer facts, metered events and identity maps.
    /// Shared by the production path, the non-fast precise-invalidation
    /// gate and the subcase-4 degradation counter-controls.
    pub fn increment_gate(
        changed_def: &str,
        fn_order: &[String],
        facts: &EditFacts,
        hir_before: &BTreeMap<String, HirIdentity>,
        code_before: &BTreeMap<String, CodeIdentity>,
        hir_after: &BTreeMap<String, HirIdentity>,
        code_after: &BTreeMap<String, CodeIdentity>,
    ) -> Vec<String> {
        let mut v = Vec::new();
        if facts.events.parse_fn_items != 1 {
            v.push(format!(
                "parse scope: the edit's parse document held {} fn items (expected 1)",
                facts.events.parse_fn_items
            ));
        }
        for (name, w) in &facts.body_work {
            if *name != facts.changed_name {
                v.push(format!(
                    "frontend body work touched unchanged function `{name}` (resolve={} typecheck={} plan={} bind={})",
                    w.resolve, w.typecheck, w.plan, w.bind
                ));
            }
        }
        if facts.receipt.real_body_plans != 1 {
            v.push(format!(
                "emission rendered {} real bodies (expected 1)",
                facts.receipt.real_body_plans
            ));
        }
        let expect_scaffold = fn_order.len().saturating_sub(1);
        if facts.receipt.scaffold_fns != expect_scaffold {
            v.push(format!(
                "emission scaffold count {} != {} (n−1)",
                facts.receipt.scaffold_fns, expect_scaffold
            ));
        }
        // Metered stage events: every axis must name the changed def
        // EXACTLY once — a re-verify/decode/admit of any other def is a
        // production regression.
        for (axis, map) in [
            ("decode", &facts.events.decode),
            ("verify_mir", &facts.events.verify_mir),
            ("verify_abc", &facts.events.verify_abc),
            ("admit", &facts.events.admit),
        ] {
            for (def, count) in map {
                if def != changed_def {
                    v.push(format!(
                        "stage event {axis}×{count} recorded for unchanged `{def}`"
                    ));
                }
            }
            if map.get(changed_def) != Some(&1) {
                v.push(format!(
                    "stage event {axis} for `{changed_def}` is {:?} (expected exactly 1)",
                    map.get(changed_def)
                ));
            }
        }
        if facts.events.disk_read_bytes != 0 || facts.events.disk_write_bytes != 0 {
            v.push(format!(
                "edit-path IO read={}B write={}B (must be 0)",
                facts.events.disk_read_bytes, facts.events.disk_write_bytes
            ));
        }
        for def in fn_order {
            if def == changed_def {
                continue;
            }
            if hir_after.get(def) != hir_before.get(def) {
                v.push(format!("HIR identity moved for unchanged `{def}`"));
            }
            if code_after.get(def) != code_before.get(def) {
                v.push(format!("code identity moved for unchanged `{def}`"));
            }
        }
        if hir_after.get(changed_def) == hir_before.get(changed_def) {
            v.push(format!("`{changed_def}`'s HIR object was not replaced"));
        }
        let want_gen = code_before
            .get(changed_def)
            .map(|c| c.generation + 1)
            .unwrap_or(1);
        match code_after.get(changed_def) {
            Some(c) if c.generation == want_gen => {}
            other => v.push(format!(
                "`{changed_def}`'s code generation is {other:?} (expected {want_gen})"
            )),
        }
        v
    }

    /// Segment diff + masked single-def parse document. Returns the
    /// changed function's parsed item (absolute spans), its new byte span
    /// and the MEASURED fn-item count of the parse document. `Err` = an
    /// explicit non-fast rejection (never a silent whole compile).
    fn plan_single_def_item(
        &self,
        new_src: &str,
        changed_def: &str,
    ) -> Result<(FnItem, (usize, usize), usize), String> {
        let old = self.src.as_bytes();
        let new = new_src.as_bytes();
        let mut p = 0usize;
        while p < old.len().min(new.len()) && old[p] == new[p] {
            p += 1;
        }
        let mut s = 0usize;
        while s < old.len().min(new.len()) - p && old[old.len() - 1 - s] == new[new.len() - 1 - s] {
            s += 1;
        }
        let old_lo = p;
        let old_hi = old.len() - s;
        let new_len = new.len();
        let delta = new_len as i64 - old.len() as i64;

        // Exactly ONE fn item may overlap the changed region, and it must
        // be the named def; no type item may.
        let mut overlapped: Vec<&OutlineEntry> = self
            .outline
            .iter()
            .filter(|o| o.start < old_hi && old_lo < o.end)
            .collect();
        if overlapped.len() != 1 {
            return Err(format!(
                "non-fast-path: the edit meets {} declarations (a single body edit is the fast path)",
                overlapped.len()
            ));
        }
        let hit = overlapped.remove(0);
        if !hit.is_fn || format!("d_{}", hit.name) != changed_def {
            return Err(format!(
                "non-fast-path: the edit changes `{}` (a `{changed_def}` body edit was requested)",
                hit.name
            ));
        }
        let new_span = (
            hit.start,
            (new_len as i64 - (old.len() as i64 - hit.end as i64)) as usize,
        );

        // Mask every OTHER fn to an equal-length comment block; type items
        // and the changed fn keep their bytes (offsets preserved).
        let mut mask_ranges: Vec<(usize, usize)> = Vec::new();
        for o in &self.outline {
            if !o.is_fn || format!("d_{}", o.name) == changed_def {
                continue;
            }
            let (a, b) = if o.end <= old_lo {
                (o.start, o.end)
            } else {
                (
                    (o.start as i64 + delta) as usize,
                    (o.end as i64 + delta) as usize,
                )
            };
            mask_ranges.push((a, b));
        }
        let masked = masked_document(new_src, &mask_ranges);
        let program = parse_program_with(&masked, SourceProfile::Record).map_err(|d| {
            format!(
                "non-fast-path: masked single-def parse failed: {}",
                d.message
            )
        })?;
        let fns: Vec<&FnItem> = program
            .items
            .iter()
            .filter_map(|i| match i {
                Item::Fn(f) => Some(f),
                _ => None,
            })
            .collect();
        let measured = fns.len();
        if measured != 1 || fns[0].name.name != hit.name {
            return Err(format!(
                "non-fast-path: the masked parse document held {measured} fn items (expected exactly `{}`)",
                hit.name
            ));
        }
        Ok((fns[0].clone(), new_span, measured))
    }

    /// Shift every outline entry AFTER `changed_def`'s segment by `delta`.
    fn shift_outline_after(&mut self, changed_def: &str, delta: i64) {
        let changed_end = self
            .outline
            .iter()
            .find(|o| o.is_fn && format!("d_{}", o.name) == changed_def)
            .map(|o| o.end)
            .unwrap_or(0);
        for e in self.outline.iter_mut() {
            if e.start >= changed_end {
                e.start = (e.start as i64 + delta) as usize;
                e.end = (e.end as i64 + delta) as usize;
            }
        }
    }

    // ------------------------------------------------------------------
    // Non-fast path: dependency-computed invalidation (transactional)
    // ------------------------------------------------------------------

    /// Apply a source edit that changes the type table, a signature, or
    /// the declaration set. The REBUILD SET is computed from actual
    /// dependency facts; unaffected functions keep their resident
    /// HIR/code identity and are not re-lowered. TRANSACTIONAL: link, the
    /// candidate run and the precise-invalidation gate must all succeed
    /// BEFORE anything installs. Returns the rebuilt defs.
    pub fn apply_source(&mut self, new_src: &str) -> Result<Vec<String>, String> {
        self.apply_source_impl(new_src, None)
    }

    /// The explicit PARTIAL-rebuild counter-control (cross-revision link
    /// hazard): rebuild exactly `defs` from `new_src` and leave the other
    /// residents in place — a signature-incompatible mix must be a
    /// controlled LINK rejection that leaves the previous snapshot
    /// executable.
    pub fn apply_source_partial(
        &mut self,
        new_src: &str,
        defs: &[&str],
    ) -> Result<Vec<String>, String> {
        self.apply_source_impl(new_src, Some(defs))
    }

    fn apply_source_impl(
        &mut self,
        new_src: &str,
        forced: Option<&[&str]>,
    ) -> Result<Vec<String>, String> {
        let before_code = self.identity_all();
        let before_hir = self.hir_identity_all();
        crate::driver::front_door_admit(new_src).map_err(|d| d.message)?;
        // Cold-class whole frontend for the new revision (justified by the
        // non-body change); its body work is recorded, not gated as an edit.
        let (compiled, _cold_work) = body_work::capture_body_work(|| {
            auto_ac_prototype::source::compile_record_source(new_src, "sess.at", &self.caps)
        });
        let compiled = compiled.map_err(|ds| {
            ds.first()
                .map(|d| format!("frontend: {} {}", d.code, d.message))
                .unwrap_or_else(|| "frontend failed".to_string())
        })?;
        let bundle = Arc::new(compiled.checked.bundle().clone());
        let program = parse_program_with(new_src, SourceProfile::Record)
            .map_err(|d| format!("outline parse: {}", d.message))?;
        let new_outline = outline_of(&program)?;
        let new_m = bundle.modules.first().ok_or("bundle has no module")?;

        let new_fn_defs: Vec<String> = new_outline
            .iter()
            .filter(|o| o.is_fn)
            .map(|o| format!("d_{}", o.name))
            .collect();
        if !new_fn_defs.contains(&self.entry) {
            return Err(format!(
                "non-fast-path rejected: entry `{}` missing in the new source",
                self.entry
            ));
        }

        // ---- dependency facts from the checked HIR of BOTH revisions ----
        let changed_types: BTreeSet<String> = {
            let old_segs = segment_map(&self.outline, &self.src, false);
            let new_segs = segment_map(&new_outline, new_src, false);
            let mut set = BTreeSet::new();
            for (name, seg) in &new_segs {
                if old_segs.get(name) != Some(seg) {
                    set.insert(name.clone());
                }
            }
            for name in old_segs.keys() {
                if !new_segs.contains_key(name) {
                    set.insert(name.clone());
                }
            }
            set
        };
        let changed_fns: BTreeSet<String> = {
            let old_segs = segment_map(&self.outline, &self.src, true);
            let new_segs = segment_map(&new_outline, new_src, true);
            let mut set = BTreeSet::new();
            for (name, seg) in &new_segs {
                if old_segs.get(name) != Some(seg) {
                    set.insert(format!("d_{name}"));
                }
            }
            for name in old_segs.keys() {
                if !new_segs.contains_key(name) {
                    set.insert(format!("d_{name}"));
                }
            }
            set
        };
        // Transitive record nesting of the NEW module: a fn mentioning R is
        // affected when any type inside R's (transitive) shape changed.
        let affected_types = close_over_nesting(new_m, &changed_types);
        let mentions = per_def_record_mentions(new_m);

        // SIGNATURE-content changes: the snapshot link's units each import
        // EVERY other def with its signature (the 828 unit convention), so
        // a def whose signature CONTENT changed invalidates every unit —
        // precisely what the link's interning gate checks. Signature
        // equality is NAME+SHAPE structural (two bundles never share type
        // table indices).
        let old_m = self
            .bundle
            .modules
            .first()
            .ok_or("old bundle has no module")?;
        let mut sig_changed: BTreeSet<String> = BTreeSet::new();
        for def in new_fn_defs.iter().chain(self.fn_order.iter()) {
            let new_sig = new_fn_defs
                .contains(def)
                .then(|| sig_shape_of(new_m, def))
                .flatten();
            let old_sig = self
                .fn_order
                .contains(def)
                .then(|| sig_shape_of(old_m, def))
                .flatten();
            if new_sig != old_sig {
                sig_changed.insert(def.clone());
            }
        }

        let mut affected: BTreeSet<String> = changed_fns.clone();
        for def in &new_fn_defs {
            if let Some(ms) = mentions.get(def) {
                if !ms.is_disjoint(&affected_types) {
                    affected.insert(def.clone());
                }
            }
        }
        // A pure helper-BODY change never re-lowers its callers: their
        // units are content-unchanged and the snapshot binds the call to
        // the callee's slot at link time (the "helper body" precision
        // axis). Removed defs must still drop out of the resident set.
        for def in self.fn_order.clone() {
            if !new_fn_defs.contains(&def) {
                affected.insert(def.clone());
            }
        }
        if !sig_changed.is_empty() {
            // At least one imported signature changed content: every unit
            // embeds every def's import signature, so every unit re-lowers
            // (the link gate owns exactly this dependency).
            affected.extend(new_fn_defs.iter().cloned());
        }
        let affected: Vec<String> = match forced {
            Some(defs) => {
                for d in defs {
                    if !new_fn_defs.iter().any(|n| n == d) {
                        return Err(format!(
                            "forced rebuild def `{d}` missing in the new source"
                        ));
                    }
                }
                defs.iter().map(|s| s.to_string()).collect()
            }
            None => affected.into_iter().collect(),
        };

        // ---- build the new state ASIDE (metered); install only after
        //      link + run + gate all pass (F820-R2-01) ----
        let mut ev = ProductionEvents::default();
        let mut new_hir = self.hir.clone();
        let mut new_code = self.code.clone();
        let mut new_def_bundle = self.def_bundle.clone();
        // Pin the producing bundle of every def whose BODY did not change
        // to the object that actually produced its resident HIR — the new
        // revision's whole bundle did NOT re-produce it (identity axis
        // honesty: the bundle pointer names the real producer).
        for def in &new_fn_defs {
            if !changed_fns.contains(def)
                && self.hir.contains_key(def)
                && !new_def_bundle.contains_key(def)
            {
                new_def_bundle.insert(def.clone(), self.bundle.clone());
            }
        }
        for def in &affected {
            if !new_fn_defs.contains(def) {
                new_hir.remove(def);
                new_code.remove(def);
                new_def_bundle.remove(def);
                continue;
            }
            let mint = new_code.get(def).map(|c| c.generation() + 1).unwrap_or(1);
            let (body_arc, code) = self.lower_and_admit(&bundle, def, mint, new_src, &mut ev)?;
            // Layer precision: a def whose BODY did not change keeps its
            // resident HIR Arc (and producing bundle) even when its unit
            // re-lowers (its import table embeds a changed sibling
            // signature).
            if changed_fns.contains(def) || !new_hir.contains_key(def) {
                new_hir.insert(def.clone(), body_arc);
                new_def_bundle.insert(def.clone(), bundle.clone());
            }
            new_code.insert(def.clone(), code);
        }
        let code_vec: Vec<Arc<FunctionCode>> = new_code.values().cloned().collect();
        let new_snap = Arc::new(
            ProgramSnapshot::link(code_vec, &self.entry, self.revision + 1)
                .map_err(|e| format!("link rejected: {e}"))?,
        );

        // Precise-invalidation gate (non-fast variant): every def OUTSIDE
        // the affected set keeps BOTH identities, and the meter names only
        // affected defs.
        let after_code = code_identity_of(&new_code);
        let after_hir = hir_identity_of(&new_hir, &new_def_bundle, &self.bundle);
        let mut violations = Vec::new();
        for def in &new_fn_defs {
            if affected.contains(def) {
                continue;
            }
            if after_hir.get(def) != before_hir.get(def) {
                violations.push(format!("HIR identity moved for unaffected `{def}`"));
            }
            if after_code.get(def) != before_code.get(def) {
                violations.push(format!("code identity moved for unaffected `{def}`"));
            }
        }
        for (axis, map) in [
            ("decode", &ev.decode),
            ("verify_mir", &ev.verify_mir),
            ("verify_abc", &ev.verify_abc),
            ("admit", &ev.admit),
        ] {
            for def in map.keys() {
                if !affected.contains(def) {
                    violations.push(format!(
                        "stage event {axis} recorded for unaffected `{def}`"
                    ));
                }
            }
        }
        if !violations.is_empty() {
            return Err(format!(
                "precise-invalidation gate RED: {}",
                violations.join("; ")
            ));
        }

        // RUN before publishing (F820-R2-01): a runtime trap in the new
        // revision rejects the apply, the session stays on its previous
        // state.
        self.run_snapshot(&new_snap)?;

        // ---- publish ----
        let mut env = resolve_env(&program, &self.caps, SourceProfile::Record).map_err(|ds| {
            ds.first()
                .map(|d| format!("resolve env: {} {}", d.code, d.message))
                .unwrap_or_else(|| "resolve env failed".to_string())
        })?;
        auto_ac_prototype::source::env_note_cold_walks(&mut env);
        self.env = env;
        self.outline = new_outline;
        self.fn_order = new_fn_defs;
        self.src = new_src.to_string();
        self.bundle = bundle;
        self.def_bundle = new_def_bundle;
        self.hir = new_hir;
        self.code = new_code;
        self.revision += 1;
        self.snapshot = new_snap;
        for def in &affected {
            if self.code.contains_key(def) {
                *self.counters.entry(format!("lower:{def}")).or_insert(0) += 1;
            }
        }
        Ok(affected)
    }

    /// The authorization axis: capabilities embed in every av.data/1 unit,
    /// so any caps delta re-lowers and re-admits EVERY unit while HIR
    /// identity stays put; an unchanged caps list produces ZERO work.
    /// TRANSACTIONAL (F820-R2-01): the re-admitted set links and runs
    /// BEFORE the caps/code swap installs.
    pub fn apply_caps(&mut self, new_caps: &[String]) -> Result<Vec<String>, String> {
        if new_caps == self.caps.as_slice() {
            return Ok(Vec::new());
        }
        // Lower with the CANDIDATE caps; restore the old set on any
        // failure — nothing installs before link + run pass.
        let old_caps = std::mem::replace(&mut self.caps, new_caps.to_vec());
        let restore = |s: &mut Self| {
            s.caps = old_caps.clone();
        };
        let mut ev = ProductionEvents::default();
        let mut new_code = self.code.clone();
        for def in self.fn_order.clone() {
            let mint = new_code.get(&def).map(|c| c.generation() + 1).unwrap_or(1);
            let bundle = self.bundle_for(&def);
            match self.lower_and_admit(&bundle, &def, mint, &self.src.clone(), &mut ev) {
                Ok((_body_arc, code)) => {
                    new_code.insert(def.clone(), code);
                }
                Err(e) => {
                    restore(self);
                    return Err(e);
                }
            }
        }
        let code_vec: Vec<Arc<FunctionCode>> = new_code.values().cloned().collect();
        let new_snap = match ProgramSnapshot::link(code_vec, &self.entry, self.revision + 1) {
            Ok(s) => Arc::new(s),
            Err(e) => {
                restore(self);
                return Err(format!("link rejected: {e}"));
            }
        };
        if let Err(e) = self.run_snapshot(&new_snap) {
            restore(self);
            return Err(e);
        }
        // publish (self.caps already holds the candidate set)
        self.code = new_code;
        self.revision += 1;
        self.snapshot = new_snap;
        for def in self.fn_order.clone() {
            *self.counters.entry(format!("lower:{def}")).or_insert(0) += 1;
        }
        Ok(self.fn_order.clone())
    }

    // ------------------------------------------------------------------
    // Subcase-4 degradation counter-controls (non-mutating)
    // ------------------------------------------------------------------

    /// Counter-control A — the forced WHOLE-frontend degradation: re-run
    /// the old edit shape (`compile_record_source` per edit: every body
    /// re-walked, fresh HIR objects for all) over `new_src` and judge it
    /// with the SAME structural gate. Returns the violations; empty
    /// violations would mean the gate is blind. The session is NOT
    /// mutated.
    pub fn forced_whole_frontend_edit(
        &mut self,
        new_src: &str,
        changed_def: &str,
    ) -> Result<Vec<String>, String> {
        let before_hir = self.hir_identity_all();
        let before_code = self.identity_all();
        crate::driver::front_door_admit(new_src).map_err(|d| d.message)?;
        let program = parse_program_with(new_src, SourceProfile::Record)
            .map_err(|d| format!("parse: {}", d.message))?;
        let fn_count = program
            .items
            .iter()
            .filter(|i| matches!(i, Item::Fn(_)))
            .count();
        // The real degraded walk, under the real producer window.
        let (compiled, work) = body_work::capture_body_work(|| {
            auto_ac_prototype::source::compile_record_source(new_src, "sess.at", &self.caps)
        });
        let compiled = compiled.map_err(|ds| {
            ds.first()
                .map(|d| format!("frontend: {} {}", d.code, d.message))
                .unwrap_or_else(|| "frontend failed".to_string())
        })?;
        let fresh_bundle = Arc::new(compiled.checked.bundle().clone());
        let m = fresh_bundle.modules.first().ok_or("bundle has no module")?;
        // Fresh HIR objects (a FRESH producing bundle Arc) for EVERY def
        // and a generation bump for every unit — the identities the
        // degradation would install.
        let mut after_hir = before_hir.clone();
        let mut after_code = before_code.clone();
        for d in m.defs.iter() {
            if let hir::DefKind::Function { body, .. } = &d.kind {
                if let Some(b) = m.bodies.get(body.0 as usize) {
                    after_hir.insert(
                        d.id_text.clone(),
                        HirIdentity {
                            body_ptr: Arc::as_ptr(&Arc::new(b.clone())) as u64,
                            bundle_ptr: Arc::as_ptr(&fresh_bundle) as u64,
                        },
                    );
                }
            }
            if let Some(c) = after_code.get_mut(&d.id_text) {
                c.generation += 1;
                c.ptr = c.ptr.wrapping_add(1);
            }
        }
        let facts = EditFacts {
            changed_name: changed_def
                .strip_prefix("d_")
                .unwrap_or(changed_def)
                .to_string(),
            body_work: work,
            receipt: BodyEmitReceipt {
                real_body_plans: fn_count,
                scaffold_fns: 0,
            },
            events: ProductionEvents {
                parse_fn_items: fn_count,
                ..Default::default()
            },
        };
        Ok(Self::increment_gate(
            changed_def,
            &self.fn_order,
            &facts,
            &before_hir,
            &before_code,
            &after_hir,
            &after_code,
        ))
    }

    /// Counter-control B — "only the hash kept, objects replaced": a
    /// content-addressed "reuse" that re-materializes an EQUAL-content
    /// producing bundle for the UNCHANGED functions. Every stage fact
    /// stays clean (no body work, single-body receipt) — only the
    /// object-identity axes can catch it, and they must.
    pub fn forced_hash_only_replacement(
        &mut self,
        changed_def: &str,
    ) -> Result<Vec<String>, String> {
        let before_hir = self.hir_identity_all();
        let before_code = self.identity_all();
        // A FRESH equal-content bundle Arc (the "reuse" would produce new
        // objects even though every content hash matches).
        let fresh_bundle = Arc::new((*self.bundle).clone());
        let m = fresh_bundle.modules.first().ok_or("bundle has no module")?;
        let mut after_hir = before_hir.clone();
        for def in &self.fn_order {
            if def == changed_def {
                continue;
            }
            let di = m
                .defs
                .iter()
                .position(|d| d.id_text == *def)
                .ok_or_else(|| format!("def `{def}` missing in bundle"))?;
            if let hir::DefKind::Function { body, .. } = &m.defs[di].kind {
                let b = m
                    .bodies
                    .get(body.0 as usize)
                    .ok_or_else(|| format!("body of `{def}` out of range"))?;
                after_hir.insert(
                    def.clone(),
                    HirIdentity {
                        body_ptr: Arc::as_ptr(&Arc::new(b.clone())) as u64,
                        bundle_ptr: Arc::as_ptr(&fresh_bundle) as u64,
                    },
                );
            }
        }
        let facts = EditFacts {
            changed_name: changed_def
                .strip_prefix("d_")
                .unwrap_or(changed_def)
                .to_string(),
            body_work: BTreeMap::new(),
            receipt: BodyEmitReceipt {
                real_body_plans: 1,
                scaffold_fns: self.fn_order.len().saturating_sub(1),
            },
            events: ProductionEvents {
                parse_fn_items: 1,
                ..Default::default()
            },
        };
        Ok(Self::increment_gate(
            changed_def,
            &self.fn_order,
            &facts,
            &before_hir,
            &before_code,
            &after_hir,
            &before_code,
        ))
    }

    // ------------------------------------------------------------------
    // Counter-control seams (F820-R1-01 close)
    // ------------------------------------------------------------------

    /// Counter-control seam A: the NEXT `edit_body` records a simulated
    /// disk read of `bytes` THROUGH the real metering entry, mid-edit —
    /// the structural gate must redden the edit (proving the zero-IO
    /// rule judges live events, not a constant). Consumed once.
    pub fn set_control_disk_read(&mut self, bytes: u64) {
        self.control_disk_read = Some(bytes);
    }

    /// Counter-control seam B: the NEXT `edit_body` records a simulated
    /// re-verify (`verify_mir`) event for `def` THROUGH the real metering
    /// entry — the gate must redden (proving stage events are metered per
    /// def, not asserted). Consumed once.
    pub fn set_control_reverify_def(&mut self, def: &str) {
        self.control_reverify_def = Some(def.to_string());
    }

    // ------------------------------------------------------------------
    // Execution and evidence accessors
    // ------------------------------------------------------------------

    /// Run the current snapshot's entry (unit view).
    pub fn run(&self) -> Result<Value, String> {
        self.run_snapshot(&self.snapshot.clone())
    }

    /// Run ONE snapshot's entry (old-snapshot comparison evidence: a
    /// superseded snapshot stays executable; a CANDIDATE snapshot can be
    /// verified BEFORE publication — F820-R2-01).
    pub fn run_snapshot(&self, snap: &Arc<ProgramSnapshot>) -> Result<Value, String> {
        let (out, _trace) =
            execute_program_units(snap, &ExecLimits::default(), &CancelFlag::new(false));
        match out {
            ExecOutcome::Success { value, .. } => Ok(value),
            other => Err(format!("run: {other:?}")),
        }
    }

    /// The current snapshot.
    pub fn snapshot(&self) -> &Arc<ProgramSnapshot> {
        &self.snapshot
    }

    /// Link the resident code set into a NEW snapshot (a failed link
    /// leaves the previous snapshot and revision untouched).
    fn relink(&mut self) -> Result<(), String> {
        let code: Vec<Arc<FunctionCode>> = self.code.values().cloned().collect();
        let snap = ProgramSnapshot::link(code, &self.entry, self.revision + 1)?;
        self.revision += 1;
        self.snapshot = Arc::new(snap);
        Ok(())
    }

    pub fn identity_of(&self, def_id: &str) -> Option<CodeIdentity> {
        self.code.get(def_id).map(|c| CodeIdentity {
            ptr: Arc::as_ptr(c) as u64,
            generation: c.generation(),
        })
    }

    pub fn identity_all(&self) -> BTreeMap<String, CodeIdentity> {
        code_identity_of(&self.code)
    }

    pub fn hir_identity_of(&self, def_id: &str) -> Option<HirIdentity> {
        hir_identity_of(&self.hir, &self.def_bundle, &self.bundle)
            .into_iter()
            .find(|(k, _)| k == def_id)
            .map(|(_, v)| v)
    }

    pub fn hir_identity_all(&self) -> BTreeMap<String, HirIdentity> {
        hir_identity_of(&self.hir, &self.def_bundle, &self.bundle)
    }

    /// Per-def lower counters ("lower:<def>"; the structural axis).
    pub fn counters(&self) -> &BTreeMap<String, u64> {
        &self.counters
    }

    /// The warmup's own producer body work (cold evidence).
    pub fn cold_body_work(&self) -> &BTreeMap<String, BodyWork> {
        &self.cold_body_work
    }

    /// Resident function def ids in source order.
    pub fn fn_order(&self) -> &[String] {
        &self.fn_order
    }
}

// ----------------------------------------------------------------------
// Identity-map helpers (shared by the production path and the controls)
// ----------------------------------------------------------------------

fn code_identity_of(code: &BTreeMap<String, Arc<FunctionCode>>) -> BTreeMap<String, CodeIdentity> {
    code.iter()
        .map(|(k, c)| {
            (
                k.clone(),
                CodeIdentity {
                    ptr: Arc::as_ptr(c) as u64,
                    generation: c.generation(),
                },
            )
        })
        .collect()
}

fn hir_identity_of(
    hir_map: &BTreeMap<String, Arc<hir::Body>>,
    def_bundle: &BTreeMap<String, Arc<hir::Bundle>>,
    whole: &Arc<hir::Bundle>,
) -> BTreeMap<String, HirIdentity> {
    hir_map
        .iter()
        .map(|(k, b)| {
            let bundle_ptr = def_bundle
                .get(k)
                .map(|bb| Arc::as_ptr(bb) as u64)
                .unwrap_or_else(|| Arc::as_ptr(whole) as u64);
            (
                k.clone(),
                HirIdentity {
                    body_ptr: Arc::as_ptr(b) as u64,
                    bundle_ptr,
                },
            )
        })
        .collect()
}

// ----------------------------------------------------------------------
// Source outline helpers
// ----------------------------------------------------------------------

fn outline_of(
    program: &auto_ac_prototype::source::ast::Program,
) -> Result<Vec<OutlineEntry>, String> {
    let mut out = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(f) => out.push(OutlineEntry {
                is_fn: true,
                name: f.name.name.clone(),
                start: f.span.start,
                end: f.span.end,
            }),
            Item::Type(t) => out.push(OutlineEntry {
                is_fn: false,
                name: t.name.name.clone(),
                start: t.span.start,
                end: t.span.end,
            }),
        }
    }
    Ok(out)
}

/// name -> source segment text of all items (`fns_only` selects the axis).
fn segment_map(outline: &[OutlineEntry], src: &str, fns_only: bool) -> BTreeMap<String, String> {
    outline
        .iter()
        .filter(|o| o.is_fn == fns_only)
        .map(|o| (o.name.clone(), src[o.start..o.end].to_string()))
        .collect()
}

/// Replace byte ranges with equal-length line-comment filler (newlines
/// preserved): the document keeps every byte offset, so the un-masked
/// function parses at its ABSOLUTE source position.
fn masked_document(src: &str, ranges: &[(usize, usize)]) -> String {
    let bytes = src.as_bytes();
    let mut sorted: Vec<(usize, usize)> = ranges.to_vec();
    sorted.sort();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    for (a, b) in sorted {
        if a < i || b > bytes.len() || a > b {
            continue;
        }
        out.extend_from_slice(&bytes[i..a]);
        out.extend(mask_fill(&bytes[a..b]));
        i = b;
    }
    out.extend_from_slice(&bytes[i..]);
    String::from_utf8(out).expect("newline-preserving ASCII mask keeps UTF-8 valid")
}

fn mask_fill(seg: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(seg.len());
    for (i, line) in seg.split(|&b| b == b'\n').enumerate() {
        if i > 0 {
            out.push(b'\n');
        }
        if line.len() >= 2 {
            out.extend_from_slice(b"//");
            out.extend(std::iter::repeat(b'x').take(line.len() - 2));
        } else if line.len() == 1 {
            out.push(b' ');
        }
    }
    out
}

// ----------------------------------------------------------------------
// Dependency facts from the checked HIR
// ----------------------------------------------------------------------

fn type_name(m: &hir::Module, tu: hir::TypeUse) -> Option<&str> {
    match tu {
        hir::TypeUse::Ref(t) => m.types.get(t.0 as usize).map(|td| td.id_text.as_str()),
        hir::TypeUse::Builtin(_) => None,
    }
}

fn is_record_type(m: &hir::Module, name: &str) -> bool {
    m.types
        .iter()
        .any(|t| t.id_text == name && matches!(t.kind, hir::TypeKind::Record(_)))
}

/// Record names a body MENTIONS: local/param/result types, place roots and
/// walked field paths, constructor records, expression result types.
fn record_mentions_of(m: &hir::Module, body: &hir::Body) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    let note = |tu: hir::TypeUse, set: &mut BTreeSet<String>| {
        if let Some(name) = type_name(m, tu) {
            if is_record_type(m, name) {
                set.insert(name.to_string());
            }
        }
    };
    let walk_path = |root: hir::TypeUse, path: &[hir::FieldId], set: &mut BTreeSet<String>| {
        let mut cur = root;
        for fi in path {
            let Some(name) = type_name(m, cur) else {
                return;
            };
            let Some(td) = m.types.iter().find(|t| t.id_text == name) else {
                return;
            };
            let hir::TypeKind::Record(shape) = &td.kind else {
                return;
            };
            set.insert(name.to_string());
            let Some(field) = shape.fields.get(fi.0 as usize) else {
                return;
            };
            note(field.ty, set);
            cur = field.ty;
        }
    };
    for l in &body.locals {
        note(l.ty, &mut set);
    }
    for p in &body.places {
        let root_ty = body
            .locals
            .get(p.local.0 as usize)
            .map(|l| l.ty)
            .unwrap_or(hir::TypeUse::Builtin(hir::BuiltinType::I32));
        walk_path(root_ty, p_path(&p.kind), &mut set);
        note(p.ty, &mut set);
    }
    for e in &body.exprs {
        note(e.ty, &mut set);
        match &e.kind {
            hir::ExprKind::RecordCtor { record, .. } => {
                if let Some(td) = m.types.get(record.0 as usize) {
                    set.insert(td.id_text.clone());
                }
            }
            hir::ExprKind::FieldGet { base, path } => {
                let root_ty = body
                    .exprs
                    .get(base.0 as usize)
                    .map(|b| b.ty)
                    .unwrap_or(hir::TypeUse::Builtin(hir::BuiltinType::I32));
                walk_path(root_ty, path, &mut set);
            }
            _ => {}
        }
    }
    set
}

fn p_path(kind: &hir::PlaceKind) -> &[hir::FieldId] {
    match kind {
        hir::PlaceKind::FieldPath(p) => p,
        hir::PlaceKind::Local => &[],
    }
}

/// def id -> record names mentioned.
fn per_def_record_mentions(m: &hir::Module) -> BTreeMap<String, BTreeSet<String>> {
    let mut out = BTreeMap::new();
    for d in m.defs.iter() {
        if let hir::DefKind::Function { body, .. } = &d.kind {
            if let Some(b) = m.bodies.get(body.0 as usize) {
                out.insert(d.id_text.clone(), record_mentions_of(m, b));
            }
        }
    }
    out
}

/// Changed types widened over record nesting: a changed type invalidates
/// itself plus every record whose (transitive) shape contains it.
fn close_over_nesting(m: &hir::Module, changed: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = changed.clone();
    loop {
        let mut grew = false;
        for td in m.types.iter() {
            if out.contains(&td.id_text) {
                continue;
            }
            let hir::TypeKind::Record(shape) = &td.kind else {
                continue;
            };
            if shape
                .fields
                .iter()
                .any(|f| type_name(m, f.ty).map(|n| out.contains(n)).unwrap_or(false))
            {
                grew = out.insert(td.id_text.clone()) || grew;
            }
        }
        if !grew {
            return out;
        }
    }
}

// ----------------------------------------------------------------------
// Signature shape equality (across bundles — never by type table index)
// ----------------------------------------------------------------------

/// A type use reduced to a name+shape tree two bundles can compare.
#[derive(Clone, Debug, PartialEq, Eq)]
enum TypeShape {
    Builtin(&'static str),
    Named(String, Option<Box<Vec<(String, TypeShape)>>>),
    Fn(Box<Vec<TypeShape>>, Box<TypeShape>),
    Other(String),
}

fn type_shape(m: &hir::Module, tu: hir::TypeUse) -> TypeShape {
    match tu {
        hir::TypeUse::Builtin(b) => TypeShape::Builtin(builtin_tag(b)),
        hir::TypeUse::Ref(t) => match m.types.get(t.0 as usize) {
            Some(td) => match &td.kind {
                hir::TypeKind::Record(shape) => TypeShape::Named(
                    td.id_text.clone(),
                    Some(Box::new(
                        shape
                            .fields
                            .iter()
                            .map(|f| (f.name.clone(), type_shape(m, f.ty)))
                            .collect(),
                    )),
                ),
                hir::TypeKind::Function(sig) => TypeShape::Fn(
                    Box::new(sig.params.iter().map(|p| type_shape(m, *p)).collect()),
                    Box::new(type_shape(m, sig.result)),
                ),
                _ => TypeShape::Other(td.id_text.clone()),
            },
            None => TypeShape::Other("<missing>".to_string()),
        },
    }
}

fn builtin_tag(b: hir::BuiltinType) -> &'static str {
    match b {
        hir::BuiltinType::I32 => "i32",
        hir::BuiltinType::Bool => "bool",
        hir::BuiltinType::StrSlice => "str",
        hir::BuiltinType::StrOwned => "Str",
        hir::BuiltinType::Char => "char",
    }
}

/// A def's signature as a name+shape tree (`fn(O) -> O` reduces O to its
/// full field shape, so an O-layout change IS a signature change).
fn sig_shape_of(m: &hir::Module, def: &str) -> Option<(Vec<TypeShape>, TypeShape)> {
    let d = m.defs.iter().find(|d| d.id_text == def)?;
    let ty = match &d.kind {
        hir::DefKind::Function { ty, .. } => *ty,
        _ => return None,
    };
    match m.types.get(ty.0 as usize).map(|t| &t.kind) {
        Some(hir::TypeKind::Function(sig)) => Some((
            sig.params.iter().map(|p| type_shape(m, *p)).collect(),
            type_shape(m, sig.result),
        )),
        _ => None,
    }
}
