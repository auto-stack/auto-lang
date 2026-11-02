//! PLAN-912: the closures FUNCTION session — warm SINGLE-UNIT production on
//! top of [`ClosuresFunctionFrontend`] (core) and [`ClosuresUpdateSession`]
//! (install/link/execute). The warm path re-parses only the edited unit's
//! fragment, re-resolves only that unit, mints a real closures credential
//! against the retained registry, lowers it from VERIFIED facts and submits
//! at most that one unit; every unchanged unit keeps its installed `Arc`
//! object, wire and generation with ZERO production on every stage.
//!
//! `apply_source` is a diff wrapper over the explicit `edit_body` entry: the
//! diff, fragment parse and bookkeeping are inside the metered window
//! (§5.1) — no hidden whole-source parse. Edits the single-unit path cannot
//! prove (signature change, capture-set change, nested lambdas, owner body,
//! a range crossing units) take the NAMED non-fast route: the whole-module
//! truth path, honestly counted on the whole axis.

use std::collections::BTreeMap;
use std::path::Path;

use ace_av_core::abc::encode_abc;
use ace_av_core::contract::{fnv1a64, MAX_INPUT_BYTES};
use ace_av_core::mir::MirModule;
use ace_core::body_work::{self, BodyWork, Stage, StageAttempts};
use ace_core::hir::DefKind;
use ace_core::source::closures_fn::{
    ClosuresFnCandidate, ClosuresFunctionFrontend, NonFastReason,
};
use ace_core::verify::verify_closures_function;
use ace_lowering::{lower_function_closures_unit, lower_function_closures_unit_checked};

use crate::closures_865_support::CompileAndRunError;
use crate::closures_session::{ApplyOutcome, ClosuresUpdateSession, ProductionCounters};

/// The route THIS apply took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteReport {
    /// First application: the whole-module truth path (cold init).
    Cold { units: usize },
    /// The source is byte-identical to the last accepted apply: a
    /// zero-production hit (still links + executes the installed revision).
    ZeroHit,
    /// The single-unit fast path produced exactly this unit (`d_<name>`).
    FastUnit { unit: String },
    /// A NAMED non-fast reason routed the apply through the whole-module
    /// path (correct invalidation, honestly counted on the whole axis).
    NonFast { reason: String },
}

impl RouteReport {
    pub fn as_str(&self) -> &'static str {
        match self {
            RouteReport::Cold { .. } => "cold",
            RouteReport::ZeroHit => "zero-hit",
            RouteReport::FastUnit { .. } => "fast-unit",
            RouteReport::NonFast { .. } => "non-fast",
        }
    }
}

/// The warm single-unit structural gate (§5.4): the fast route's own
/// observation must show (a) ZERO whole-axis attempts, (b) exactly one
/// attempt of each frontend stage for the changed unit, (c) zero attempts
/// and zero body-work for every other unit. A synthesized zero (a counter
/// nobody increments) fails the gate exactly like a hidden whole walk.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WarmGateReport {
    pub changed_unit: Option<String>,
    pub violations: Vec<String>,
}
impl WarmGateReport {
    pub fn pass(&self) -> bool {
        self.violations.is_empty()
    }
}

/// The per-apply face of the function session.
pub struct ClosuresFunctionRun {
    pub value: i32,
    pub live_after_release: u64,
    pub allocs: usize,
    pub frees: usize,
    pub submitted_units: usize,
    pub submitted_unit_names: Vec<String>,
    pub production: ProductionCounters,
    pub current_hot_bytes: usize,
    pub history_hot_bytes: usize,
    /// PLAN-912: per-unit body work (resolve/typecheck/plan/bind; plain
    /// fn-name keys, `d_`-less).
    pub body: BTreeMap<String, BodyWork>,
    /// PLAN-912: the P/M/R stage-attempt ledger (`d_<name>` unit keys).
    pub stages: StageAttempts,
    pub route: RouteReport,
    /// The fast/zero-hit route's structural gate (None on cold/non-fast).
    pub warm_gate: Option<WarmGateReport>,
    /// The retained frontend's byte estimate after this apply (§5.5 pool).
    pub retained_bytes: usize,
}

pub struct ClosuresFunctionSession {
    update: ClosuresUpdateSession,
    front: Option<ClosuresFunctionFrontend>,
    current_wires: BTreeMap<String, Vec<u8>>,
    source_path: String,
}

impl ClosuresFunctionSession {
    pub fn new(store_root: &Path, source_path: &str) -> Self {
        Self {
            update: ClosuresUpdateSession::new(Some(store_root), "d_main"),
            front: None,
            current_wires: BTreeMap::new(),
            source_path: source_path.to_string(),
        }
    }

    pub fn unit(&self, name: &str) -> Option<&std::sync::Arc<ace_av_core::function_code::FunctionCode>> {
        self.update.unit(name)
    }

    pub fn frontend(&self) -> Option<&ClosuresFunctionFrontend> {
        self.front.as_ref()
    }

    pub fn close(&mut self) -> Result<(), String> {
        self.update.close()
    }

    pub fn apply_source(
        &mut self,
        source: &str,
    ) -> Result<ClosuresFunctionRun, CompileAndRunError> {
        // The observation window covers EVERYTHING: the diff, the fragment
        // parse, the verify/lower/encode, the session's produce gates and
        // the commit. Nothing may run outside it (§5.1).
        let ((result, body), stages) = body_work::capture_stage_attempts(|| {
            body_work::capture_body_work(|| self.apply_source_inner(source))
        });
        let mut run = result?;
        run.body = body;
        run.stages = stages;
        if matches!(run.route, RouteReport::FastUnit { .. } | RouteReport::ZeroHit) {
            run.warm_gate = Some(warm_gate(&run));
        }
        Ok(run)
    }

    fn apply_source_inner(
        &mut self,
        source: &str,
    ) -> Result<ClosuresFunctionRun, CompileAndRunError> {
        // §5.5: the 4 MiB source cap at the source front door (the 865 path
        // only met it at ABC decode).
        if source.len() > MAX_INPUT_BYTES {
            return Err(CompileAndRunError::Pipeline(format!(
                "source exceeds the {} MiB source cap",
                MAX_INPUT_BYTES / (1024 * 1024)
            )));
        }
        if self.front.is_none() {
            return self.apply_cold(source);
        }
        let Some(front) = self.front.as_ref() else {
            unreachable!("checked above");
        };
        let old = front.source().to_string();
        if old == source {
            // Zero-production hit: identical source. Link + execute the
            // installed revision (the session carries every unit).
            let outcome = self
                .update
                .apply_revision(&[])
                .map_err(CompileAndRunError::Pipeline)?;
            if !outcome.gate.pass() {
                return Err(CompileAndRunError::Pipeline(format!(
                    "zero-hit revision failed the closures structural gate: {:?}",
                    outcome.gate
                )));
            }
            return self.run_face(outcome, 0, Vec::new(), RouteReport::ZeroHit);
        }
        // M axis: the diff itself is metered work (never a hidden parse).
        body_work::record_whole(Stage::FragmentDiff);
        let (range, replacement) = diff_single_region(&old, source);
        let base_sha = front.source_sha().to_string();
        match self
            .front
            .as_mut()
            .expect("frontend present")
            .edit_body(&base_sha, range, &replacement)
        {
            Ok(Ok(candidate)) => self.apply_fast(candidate),
            Ok(Err(reason)) => self.apply_nonfast(source, reason),
            Err(ds) => Err(CompileAndRunError::Diagnostics(ds)),
        }
    }

    /// The single-unit fast path: verify → lower from verified facts →
    /// spanless identity wire → submit at most this unit → commit.
    fn apply_fast(
        &mut self,
        candidate: ClosuresFnCandidate,
    ) -> Result<ClosuresFunctionRun, CompileAndRunError> {
        let front = self.front.as_ref().expect("fast implies a retained frontend");
        let wire_name = candidate.def_id_text();
        // The REAL credential: verifier-minted against the retained registry.
        let credential = verify_closures_function(
            &candidate.parts.bundle,
            candidate.def_index(),
            front.registry(),
        )
        .map_err(CompileAndRunError::Diagnostics)?;
        if credential.def_id_text() != wire_name {
            return Err(CompileAndRunError::Pipeline(format!(
                "credential `{}` does not name the edited unit `{wire_name}`",
                credential.def_id_text()
            )));
        }
        let source_snapshot = fnv1a64(self.source_path.as_bytes());
        body_work::record_attempt(Stage::MirLower, &wire_name);
        let unit = lower_function_closures_unit_checked(
            &candidate.parts.bundle,
            &credential,
            &[],
            source_snapshot,
        )
        .map_err(|error| {
            CompileAndRunError::Pipeline(format!("lower {wire_name}: {error:?}"))
        })?;
        let mut identity = unit.clone();
        for function in &mut identity.funcs {
            function.spans.clear();
        }
        body_work::record_attempt(Stage::AbcEncode, &wire_name);
        let bytes = encode_abc(&identity)
            .map_err(|error| {
                CompileAndRunError::Pipeline(format!("encode {wire_name}: {}", error.code()))
            })?
            .to_flat();
        let unchanged = self.current_wires.get(&wire_name) == Some(&bytes);
        let submitted: Vec<(&str, MirModule)> = if unchanged {
            Vec::new()
        } else {
            vec![(wire_name.as_str(), unit)]
        };
        let outcome = self
            .update
            .apply_revision(&submitted)
            .map_err(CompileAndRunError::Pipeline)?;
        if !outcome.gate.pass() {
            // The session leaves the installed revision intact on refusal.
            return Err(CompileAndRunError::Pipeline(format!(
                "fast revision failed the closures structural gate: {:?}",
                outcome.gate
            )));
        }
        let submitted_names: Vec<String> = if unchanged { Vec::new() } else { vec![wire_name.clone()] };
        self.front
            .as_mut()
            .expect("frontend present")
            .commit_candidate(candidate)
            .map_err(CompileAndRunError::Pipeline)?;
        if !unchanged {
            self.current_wires.insert(wire_name.clone(), bytes);
        }
        self.run_face(
            outcome,
            submitted_names.len(),
            submitted_names,
            RouteReport::FastUnit { unit: wire_name },
        )
    }

    /// The named non-fast route: the whole-module truth path (cold rebuild
    /// of the frontend), honestly counted. Object identity of unchanged
    /// units is preserved by the wire compare + session carry, not by the
    /// frontend.
    fn apply_nonfast(
        &mut self,
        source: &str,
        reason: NonFastReason,
    ) -> Result<ClosuresFunctionRun, CompileAndRunError> {
        let reason_text = non_fast_text(&reason);
        self.apply_cold_routed(source, Some(RouteReport::NonFast { reason: reason_text }))
    }

    fn apply_cold(&mut self, source: &str) -> Result<ClosuresFunctionRun, CompileAndRunError> {
        self.apply_cold_routed(source, None)
    }

    fn apply_cold_routed(
        &mut self,
        source: &str,
        route: Option<RouteReport>,
    ) -> Result<ClosuresFunctionRun, CompileAndRunError> {
        let (front, _parts, checked) =
            ClosuresFunctionFrontend::cold(source, &self.source_path, &[])
                .map_err(CompileAndRunError::Diagnostics)?;
        let bundle = checked.bundle();
        let source_snapshot = fnv1a64(self.source_path.as_bytes());
        // Cold lowers every unit once (whole-module frontend work).
        body_work::record_whole(Stage::WholeLower);
        let module = &bundle.modules[0];
        let mut all_units = BTreeMap::<String, (MirModule, Vec<u8>)>::new();
        for def in module
            .defs
            .iter()
            .filter(|def| matches!(def.kind, DefKind::Function { .. }))
        {
            body_work::record_attempt(Stage::MirLower, &def.id_text);
            let unit =
                lower_function_closures_unit(bundle, &def.id_text, &[], source_snapshot).map_err(
                    |error| {
                        CompileAndRunError::Pipeline(format!("lower {}: {error:?}", def.id_text))
                    },
                )?;
            let name = unit.funcs[0].name.clone();
            let mut identity = unit.clone();
            for function in &mut identity.funcs {
                function.spans.clear();
            }
            body_work::record_attempt(Stage::AbcEncode, &def.id_text);
            let bytes = encode_abc(&identity)
                .map_err(|error| {
                    CompileAndRunError::Pipeline(format!("encode {name}: {}", error.code()))
                })?
                .to_flat();
            all_units.insert(name, (unit, bytes));
        }
        if all_units.is_empty() {
            return Err(CompileAndRunError::Pipeline(
                "closures source produced no function units".into(),
            ));
        }
        let submitted = all_units
            .iter()
            .filter(|(name, (_, bytes))| self.current_wires.get(*name) != Some(bytes))
            .map(|(name, (unit, _))| (name.clone(), unit.clone()))
            .collect::<Vec<_>>();
        let submitted_names: Vec<String> = submitted.iter().map(|(n, _)| n.clone()).collect();
        let submitted_refs = submitted
            .iter()
            .map(|(name, unit)| (name.as_str(), unit.clone()))
            .collect::<Vec<_>>();
        let mut next_wires = self.current_wires.clone();
        for (name, (_, bytes)) in &all_units {
            next_wires.insert(name.clone(), bytes.clone());
        }
        let outcome = self
            .update
            .apply_revision(&submitted_refs)
            .map_err(CompileAndRunError::Pipeline)?;
        if !outcome.gate.pass() {
            return Err(CompileAndRunError::Pipeline(format!(
                "source revision failed the closures structural gate: {:?}",
                outcome.gate
            )));
        }
        self.current_wires = next_wires;
        let unit_count = all_units.len();
        self.front = Some(front);
        let route = route.unwrap_or(RouteReport::Cold { units: unit_count });
        self.run_face(outcome, submitted_names.len(), submitted_names, route)
    }

    fn run_face(
        &self,
        outcome: ApplyOutcome,
        submitted_units: usize,
        submitted_unit_names: Vec<String>,
        route: RouteReport,
    ) -> Result<ClosuresFunctionRun, CompileAndRunError> {
        let value = outcome.value.ok_or_else(|| {
            CompileAndRunError::Pipeline("source entry did not produce an i32 value".into())
        })?;
        Ok(ClosuresFunctionRun {
            value,
            live_after_release: outcome.live_after_release,
            allocs: outcome.allocs,
            frees: outcome.frees,
            submitted_units,
            submitted_unit_names,
            production: outcome.production,
            current_hot_bytes: outcome.current_hot_bytes,
            history_hot_bytes: outcome.history_hot_bytes,
            body: BTreeMap::new(),
            stages: StageAttempts::default(),
            route,
            warm_gate: None,
            retained_bytes: self
                .front
                .as_ref()
                .map(|f| f.retained_bytes_estimate())
                .unwrap_or(0),
        })
    }
}

fn non_fast_text(reason: &NonFastReason) -> String {
    match reason {
        NonFastReason::StaleBase => "stale-base".to_string(),
        NonFastReason::CrossesUnits => "crosses-units".to_string(),
        NonFastReason::OwnerBodyEdit => "owner-body-edit".to_string(),
        NonFastReason::LambdaSiteChanged(why) => format!("lambda-site-changed:{why:?}"),
        NonFastReason::LambdaNested => "lambda-nested".to_string(),
        NonFastReason::MultiRegion => "multi-region".to_string(),
    }
}

/// Prefix/suffix single-region diff: the maximal common prefix and suffix;
/// the middle is one replacement range. Any edit expressible as a single
/// contiguous replacement is found exactly; anything else spans multiple
/// units and is classified out by the edit classifier.
fn diff_single_region(old: &str, new: &str) -> (std::ops::Range<usize>, String) {
    let ob = old.as_bytes();
    let nb = new.as_bytes();
    let max_prefix = ob.iter().zip(nb.iter()).take_while(|(a, b)| a == b).count();
    let mut prefix = max_prefix;
    // keep the cut on char boundaries
    while prefix > 0
        && (!old.is_char_boundary(prefix) || !new.is_char_boundary(prefix))
    {
        prefix -= 1;
    }
    let max_suffix = ob[prefix..]
        .iter()
        .rev()
        .zip(nb[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let mut suffix = max_suffix;
    while suffix > 0
        && (!old.is_char_boundary(old.len() - suffix) || !new.is_char_boundary(new.len() - suffix))
    {
        suffix -= 1;
    }
    let range = prefix..old.len() - suffix;
    let replacement = new[prefix..new.len() - suffix].to_string();
    (range, replacement)
}

/// Compute the fast/zero-hit route's structural gate from the captured
/// observation (§5.4): the ledger must prove BOTH zero work for unchanged
/// units AND exactly-one work for the changed unit — a missing counter is a
/// violation, not a pass.
fn warm_gate(run: &ClosuresFunctionRun) -> WarmGateReport {
    let mut report = WarmGateReport {
        changed_unit: match &run.route {
            RouteReport::FastUnit { unit } => Some(unit.clone()),
            _ => None,
        },
        violations: Vec::new(),
    };
    if run.stages.whole_total() > 0 {
        report
            .violations
            .push(format!("whole-axis attempts non-zero: {}", run.stages.whole_total()));
    }
    let produced = run
        .submitted_unit_names
        .iter()
        .any(|u| Some(u) == report.changed_unit.as_ref());
    for ((unit, stage), n) in &run.stages.by_unit {
        if !stage.is_production() || unit == "*" {
            continue;
        }
        let cap: u64 = if Some(unit) == report.changed_unit.as_ref() {
            match stage {
                // The identity wire (session-front) plus the canonical wire
                // (produce_unit) are two REAL encodes of the changed unit.
                Stage::AbcEncode => 2,
                Stage::DecodeAdmit | Stage::ArtifactWrite => 1,
                _ => 1,
            }
        } else {
            0
        };
        if *n > cap {
            report.violations.push(format!(
                "unit {unit}: {n} {} attempts (cap {cap})",
                stage.as_str()
            ));
        }
    }
    for (unit, work) in &run.body {
        let wire = format!("d_{unit}");
        let changed = report.changed_unit.as_deref() == Some(wire.as_str());
        let expected: u64 = if changed { 1 } else { 0 };
        for (name, value) in [("resolve", work.resolve), ("typecheck", work.typecheck), ("plan", work.plan), ("bind", work.bind)] {
            if value > expected {
                report.violations.push(format!(
                    "unit {unit}: {value} {name} walks (cap {expected})"
                ));
            }
        }
    }
    // The changed unit must ALSO show its work — a zero nobody recorded is
    // the "Default 0" control's red. A same-wire edit (comment-only) stops
    // after encode: the session produce stages are then legitimately absent.
    if let Some(changed) = report.changed_unit.clone() {
        let plain = changed.strip_prefix("d_").unwrap_or(&changed).to_string();
        let work = run.body.get(&plain).cloned().unwrap_or_default();
        for (name, value) in [
            ("resolve", work.resolve),
            ("typecheck", work.typecheck),
            ("plan", work.plan),
            ("bind", work.bind),
        ] {
            if value < 1 {
                report
                    .violations
                    .push(format!("unit {plain}: missing {name} walk ({value} < 1)"));
            }
        }
        let mut required = vec![Stage::ParseProject, Stage::HirVerify, Stage::MirLower];
        if produced {
            required.push(Stage::MirVerify);
        }
        for stage in required {
            if run.stages.of(&changed, stage) < 1 {
                report
                    .violations
                    .push(format!("unit {changed}: missing {} attempt", stage.as_str()));
            }
        }
    }
    report
}
