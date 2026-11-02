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
    ClosuresFnCandidate, ClosuresFunctionFrontend, NonFastReason, UnitCommitFacts, UnitKind,
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
    /// The owner route re-resolved the owner plus exactly `changed` lambdas;
    /// `reused` lambdas were pushed from the retained entries without a walk.
    FastOwner {
        owner: String,
        changed: Vec<String>,
        reused: Vec<String>,
    },
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
            RouteReport::FastOwner { .. } => "fast-owner",
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
    /// The units this edit was allowed to produce (`d_<name>`): none for a
    /// zero-hit, one for a lambda edit, owner + re-resolved lambdas for an
    /// owner edit.
    pub changed_units: Vec<String>,
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
    /// PLAN-912 V-06 forced-red control: when set, every lowered unit is
    /// resubmitted even if its wire is unchanged — the "same hash, new
    /// object" degradation the session's structural gate must refuse.
    degrade_resubmit_unchanged: bool,
}

impl ClosuresFunctionSession {
    pub fn new(store_root: &Path, source_path: &str) -> Self {
        Self {
            update: ClosuresUpdateSession::new(Some(store_root), "d_main"),
            front: None,
            current_wires: BTreeMap::new(),
            source_path: source_path.to_string(),
            degrade_resubmit_unchanged: false,
        }
    }

    /// PLAN-912 V-06: the "same hash, new object" degradation control for
    /// tests. It is NOT a feature — it exists so a test can prove the
    /// structural gate refuses a revision that re-mints unchanged units.
    #[doc(hidden)]
    pub fn set_degrade_resubmit_unchanged(&mut self, on: bool) {
        self.degrade_resubmit_unchanged = on;
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
        if matches!(
            run.route,
            RouteReport::FastUnit { .. } | RouteReport::FastOwner { .. } | RouteReport::ZeroHit
        ) {
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

    /// The warm route (lambda-only or owner batch): verify every re-resolved
    /// unit against the registry (overlaid with the owner's re-derived
    /// creation sites when the owner re-resolved) → lower each from VERIFIED
    /// facts → spanless identity wire → submit only units whose wire changed
    /// → commit the frontend (and the overlay) only after the session
    /// installed the revision.
    fn apply_fast(
        &mut self,
        candidate: ClosuresFnCandidate,
    ) -> Result<ClosuresFunctionRun, CompileAndRunError> {
        let front = self.front.as_ref().expect("warm implies a retained frontend");
        let registry_after = if candidate.owner.is_some() {
            Some(
                front
                    .registry()
                    .overlay_from_batch(&candidate.parts.bundle, &candidate.real_def_ids())
                    .map_err(CompileAndRunError::Diagnostics)?,
            )
        } else {
            None
        };
        let ctx = registry_after.as_ref().unwrap_or_else(|| front.registry());
        let source_snapshot = fnv1a64(self.source_path.as_bytes());
        let mut lowered: Vec<(String, MirModule, Vec<u8>)> = Vec::with_capacity(candidate.units.len());
        let mut facts: Vec<UnitCommitFacts> = Vec::with_capacity(candidate.units.len());
        for unit in &candidate.units {
            let wire_name = candidate.def_id_text_of(unit);
            // The REAL credential: verifier-minted against the (overlaid)
            // retained registry.
            let credential = verify_closures_function(
                &candidate.parts.bundle,
                candidate.def_index_of(unit),
                ctx,
            )
            .map_err(CompileAndRunError::Diagnostics)?;
            if credential.def_id_text() != wire_name {
                return Err(CompileAndRunError::Pipeline(format!(
                    "credential `{}` does not name the edited unit `{wire_name}`",
                    credential.def_id_text()
                )));
            }
            facts.push(UnitCommitFacts {
                fn_index: unit.fn_index,
                body_digest: credential.body_digest().to_string(),
                context_revision: credential.context_revision().to_string(),
            });
            body_work::record_attempt(Stage::MirLower, &wire_name);
            let mir = lower_function_closures_unit_checked(
                &candidate.parts.bundle,
                &credential,
                &[],
                source_snapshot,
            )
            .map_err(|error| {
                CompileAndRunError::Pipeline(format!("lower {wire_name}: {error:?}"))
            })?;
            let mut identity = mir.clone();
            for function in &mut identity.funcs {
                function.spans.clear();
            }
            body_work::record_attempt(Stage::AbcEncode, &wire_name);
            let bytes = encode_abc(&identity)
                .map_err(|error| {
                    CompileAndRunError::Pipeline(format!("encode {wire_name}: {}", error.code()))
                })?
                .to_flat();
            lowered.push((wire_name, mir, bytes));
        }
        let mut submitted: Vec<(&str, MirModule)> = lowered
            .iter()
            .filter(|(name, _, bytes)| self.current_wires.get(name) != Some(bytes))
            .map(|(name, mir, _)| (name.as_str(), mir.clone()))
            .collect();
        // V-06 forced-red control: re-mint every INSTALLED unit this edit did
        // not touch from its resident MIR (identical wire, fresh object) —
        // the degradation the session's structural gate must refuse.
        let degraded_resubmits: Vec<(String, MirModule)> = if self.degrade_resubmit_unchanged {
            self.current_wires
                .keys()
                .filter(|name| !lowered.iter().any(|(n, _, _)| n == *name))
                .filter_map(|name| {
                    self.update
                        .unit(name)
                        .map(|code| (name.clone(), code.unit().module().clone()))
                })
                .collect()
        } else {
            Vec::new()
        };
        for (name, mir) in &degraded_resubmits {
            submitted.push((name.as_str(), mir.clone()));
        }
        let submitted_names: Vec<String> = submitted.iter().map(|(n, _)| n.to_string()).collect();
        let outcome = self
            .update
            .apply_revision(&submitted)
            .map_err(CompileAndRunError::Pipeline)?;
        if !outcome.gate.pass() {
            // The session leaves the installed revision intact on refusal.
            return Err(CompileAndRunError::Pipeline(format!(
                "warm revision failed the closures structural gate: {:?}",
                outcome.gate
            )));
        }
        let route = match &candidate.owner {
            None => RouteReport::FastUnit {
                unit: candidate.def_id_text(),
            },
            Some(owner) => {
                let owner_name = candidate.def_id_text();
                let changed: Vec<String> = candidate
                    .units
                    .iter()
                    .skip(1)
                    .map(|u| candidate.def_id_text_of(u))
                    .collect();
                let reused: Vec<String> = owner
                    .sites
                    .iter()
                    .filter(|(_, site)| owner.reused_ordinals.contains(&site.ordinal))
                    .map(|(_, site)| format!("d_{}", site.name))
                    .collect();
                RouteReport::FastOwner {
                    owner: owner_name,
                    changed,
                    reused,
                }
            }
        };
        self.front
            .as_mut()
            .expect("frontend present")
            .commit_candidate(candidate, registry_after, &facts)
            .map_err(CompileAndRunError::Pipeline)?;
        for (name, _, bytes) in lowered {
            self.current_wires.insert(name, bytes);
        }
        self.run_face(outcome, submitted_names.len(), submitted_names, route)
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
        // A rebuild over an existing frontend CARRIES lambda logical ids by
        // unique (owner, text) match (§5.1); the truth path still runs whole.
        let (front, _parts, checked) = match self.front.as_ref() {
            Some(prev) => ClosuresFunctionFrontend::cold_carrying(
                source,
                &self.source_path,
                &[],
                prev,
            ),
            None => ClosuresFunctionFrontend::cold(source, &self.source_path, &[]),
        }
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

/// PLAN-912 §5.3 / Design43 §16.2: the READ-ONLY accepted source state the
/// closures function session exports after a successful production+install
/// commit — the A→B contract PLAN-913 consumes. It carries identities and
/// references only: no guest value, no PASS, no live env/cell, no host
/// address. It is a data VIEW, not a credential: a consumer that persists it
/// must re-verify after landing (913's deep re-verification).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedSourceState {
    pub schema: &'static str,
    pub source_sha256: String,
    /// The logical owner (module path) and entry def.
    pub logical_owner: String,
    pub entry: String,
    pub profile: String,
    pub capabilities: Vec<String>,
    /// Producer identity of the accepted pipeline.
    pub producer_mir_lowering_version: String,
    pub producer_closures_schema: String,
    pub producer_unit_codec: &'static str,
    /// The frontend's accepted generation and the session's commit point.
    pub accepted_generation: u64,
    pub commit_point: u64,
    pub context_revision: String,
    pub units: Vec<AcceptedUnit>,
}

/// One unit of the accepted state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptedUnit {
    /// Wire identity (`d_<name>`).
    pub unit: String,
    pub logical: AcceptedUnitLogical,
    /// Span-free canonical digest of the unit's verified HIR body.
    pub body_digest: String,
    /// Identity of the closure facts this unit READS: for a lambda its
    /// registered environment declaration; for a creator its ordered
    /// creation sites (callee + modes + callee envs).
    pub dependency_digest: String,
    /// For a lambda body: its own env layout identity.
    pub env_layout_identity: Option<String>,
    /// Spanless MIR wire fingerprint (the session's submission identity).
    pub mir_identity: String,
    /// Store artifact reference of the current wire (persisted sessions).
    pub store_ref: Option<String>,
    /// (mint, current) source spans of the unit.
    pub mint_span: (usize, usize),
    pub current_span: (usize, usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcceptedUnitLogical {
    Owner,
    Lambda { logical_id: u64, owner: String },
}

impl ClosuresFunctionSession {
    /// PLAN-912 §5.3: export the accepted state. `None` before the first
    /// successful apply. Formed ONLY from committed state (the frontend's
    /// accepted catalog + the session's installed wires/revisions).
    pub fn accepted_state(&self) -> Option<AcceptedSourceState> {
        let front = self.front.as_ref()?;
        let mut units = Vec::with_capacity(front.units().len());
        for u in front.units() {
            let wire = format!("d_{}", u.name);
            let (logical, dependency_digest, env_layout_identity) = match &u.kind {
                UnitKind::Owner { .. } => {
                    let sites = front.registry().sites_of(&wire);
                    let mut h = 0xcbf2_9ce4_8422_2325u64;
                    for callee in sites {
                        if let Some(decl) = front.registry().lambda(callee) {
                            mix_bytes(&mut h, callee);
                            for f in &decl.env {
                                mix_bytes(&mut h, &format!("{:?}|{}|{}", f.ty, f.mutable, f.cell));
                            }
                        }
                    }
                    (
                        AcceptedUnitLogical::Owner,
                        format!("{h:016x}"),
                        None,
                    )
                }
                UnitKind::Lambda { logical_id, owner, .. } => {
                    let owner_name = &front.units()[*owner].name;
                    let mut h = 0xcbf2_9ce4_8422_2325u64;
                    let mut env_layout = String::new();
                    if let Some(decl) = front.registry().lambda(&wire) {
                        mix_bytes(&mut h, &decl.creator);
                        for f in &decl.env {
                            mix_bytes(&mut h, &format!("{:?}|{}|{}", f.ty, f.mutable, f.cell));
                        }
                    }
                    env_layout = format!("{h:016x}");
                    (
                        AcceptedUnitLogical::Lambda {
                            logical_id: *logical_id,
                            owner: owner_name.clone(),
                        },
                        format!("{h:016x}"),
                        Some(env_layout),
                    )
                }
            };
            units.push(AcceptedUnit {
                unit: wire.clone(),
                logical,
                body_digest: u.body_digest.clone(),
                dependency_digest,
                env_layout_identity,
                mir_identity: format!(
                    "{:016x}",
                    fnv1a64(self.current_wires.get(&wire).map(|w| w.as_slice()).unwrap_or(&[]))
                ),
                store_ref: self.update.artifact_of(&wire).map(str::to_string),
                mint_span: (u.mint_span.start, u.mint_span.end),
                current_span: (u.span.start, u.span.end),
            });
        }
        Some(AcceptedSourceState {
            schema: "ace.closures.accepted-source/1",
            source_sha256: front.source_sha().to_string(),
            logical_owner: front.source_path().to_string(),
            entry: "d_main".to_string(),
            profile: ace_core::source::CLOSURES_SOURCE_PROFILE.to_string(),
            capabilities: front.capabilities().to_vec(),
            producer_mir_lowering_version: ace_lowering::MIR_LOWERING_VERSION.to_string(),
            producer_closures_schema: ace_av_core::contract::MIR_CLOSURES_SCHEMA.to_string(),
            producer_unit_codec: "av-closures-unit/v1",
            accepted_generation: front.generation(),
            commit_point: self.update.revision(),
            context_revision: front.context_revision().to_string(),
            units,
        })
    }
    /// PLAN-913 §5.3 (T-02): publish the ACCEPTED source state as this
    /// project's CURRENT source-origin recovery root. Requires a store-backed
    /// session — the per-unit ABC artifacts the session persisted are the
    /// ones the manifest references. The caller keeps the returned ticket's
    /// artifact id + digest OUT OF BAND and forwards it to a DIFFERENT
    /// process; the digest is never re-derived from the store.
    pub fn publish_source_recovery(
        &self,
        project: &str,
    ) -> Result<crate::closures_source_recovery::RecoveryTicket, String> {
        use crate::closures_source_recovery::{
            publish_source_recovery, PublicationRequest, RecoveryUnitInput,
        };
        let store_root = self
            .update
            .store_root()
            .ok_or("source recovery requires a store-backed session")?;
        let state = self.accepted_state().ok_or("no accepted state to publish")?;
        let front = self.front.as_ref().ok_or("no retained frontend")?;
        // Every unit's wire was lowered against ONE MIR owner module; the
        // unit envelopes bind to it, so the manifest records it and the
        // publication refuses a mixed-owner catalog.
        let mut owner_module: Option<String> = None;
        let mut owner_schema: Option<String> = None;
        for u in &state.units {
            let code = self
                .update
                .unit(&u.unit)
                .ok_or_else(|| format!("unit {} is not installed", u.unit))?;
            let module = code.unit().module().owner.module.clone();
            let schema = code.unit().module().owner.schema.clone();
            match &owner_schema {
                None => owner_schema = Some(schema),
                Some(existing) if *existing != schema => {
                    return Err(format!(
                        "unit {} carries owner schema `{schema}`, catalog is `{existing}`",
                        u.unit
                    ))
                }
                Some(_) => {}
            }
            match &owner_module {
                None => owner_module = Some(module),
                Some(existing) if *existing != module => {
                    return Err(format!(
                        "unit {} carries owner module `{module}`, catalog is `{existing}`",
                        u.unit
                    ))
                }
                Some(_) => {}
            }
        }
        let owner_module = owner_module.ok_or("no units to publish")?;
        let owner_schema = owner_schema.ok_or("no units to publish")?;
        let mut units = Vec::with_capacity(state.units.len());
        for u in &state.units {
            let artifact = u
                .store_ref
                .clone()
                .ok_or_else(|| format!("unit {} has no persisted artifact", u.unit))?;
            let wire = self
                .update
                .current_wire_bytes(&u.unit)
                .ok_or_else(|| format!("unit {} has no installed wire", u.unit))?;
            if wire.is_empty() {
                return Err(format!("unit {} has an empty wire", u.unit));
            }
            let logical = match &u.logical {
                AcceptedUnitLogical::Owner => "owner".to_string(),
                AcceptedUnitLogical::Lambda { logical_id, owner } => {
                    format!("lambda:{logical_id}:{owner}")
                }
            };
            units.push(RecoveryUnitInput {
                unit: u.unit.clone(),
                logical,
                body_digest: u.body_digest.clone(),
                dependency_digest: u.dependency_digest.clone(),
                env_layout_identity: u.env_layout_identity.clone(),
                mir_identity: u.mir_identity.clone(),
                body_fp: format!("{:016x}", fnv1a64(wire)),
                wire_bytes: wire.len(),
                artifact,
                mint_span: u.mint_span,
                current_span: u.current_span,
            });
        }
        publish_source_recovery(
            store_root,
            PublicationRequest {
                project,
                source_sha256: &state.source_sha256,
                source_len: front.source().len(),
                logical_owner: &state.logical_owner,
                owner_module: &owner_module,
                owner_schema: &owner_schema,
                entry: &state.entry,
                profile: &state.profile,
                capabilities: &state.capabilities,
                accepted_generation: state.accepted_generation,
                commit_point: state.commit_point,
                context_revision: &state.context_revision,
                accepted_state_schema: state.schema,
                units,
            },
        )
        .map_err(|e| e.to_string())
    }
}

fn mix_bytes(h: &mut u64, s: &str) {
    for b in s.as_bytes() {
        *h ^= *b as u64;
        *h = h.wrapping_mul(0x0000_0100_0000_001b_3);
    }
}

fn mix_u64(h: &mut u64, v: u64) {
    for b in v.to_le_bytes() {
        *h ^= b as u64;
        *h = h.wrapping_mul(0x0000_0100_0000_001b_3);
    }
}

fn non_fast_text(reason: &NonFastReason) -> String {
    match reason {
        NonFastReason::StaleBase => "stale-base".to_string(),
        NonFastReason::CrossesUnits => "crosses-units".to_string(),
        NonFastReason::OwnerSignatureChanged => "owner-signature-changed".to_string(),
        NonFastReason::OwnerTopologyChanged { before, after } => {
            format!("owner-topology-changed:{before}->{after}")
        }
        NonFastReason::LambdaSiteChanged(why) => format!("lambda-site-changed:{why:?}"),
        NonFastReason::LambdaNested => "lambda-nested".to_string(),
        NonFastReason::RestoredContext => "restored-context".to_string(),
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
    warm_gate_report(&run.route, &run.body, &run.stages, &run.submitted_unit_names)
}

/// PLAN-912 §5.4: the warm structural gate over an OBSERVATION (route +
/// per-unit body work + stage attempts + submitted units). Public so the
/// forced-red controls can feed it synthetic observations (a missing
/// counter, a hidden sibling walk, a whole-axis attempt) and prove each is
/// a violation — the gate never passes on a zero nobody recorded.
pub fn warm_gate_report(
    route: &RouteReport,
    body: &BTreeMap<String, BodyWork>,
    stages: &StageAttempts,
    submitted_unit_names: &[String],
) -> WarmGateReport {
    let changed_units: Vec<String> = match route {
        RouteReport::FastUnit { unit } => vec![unit.clone()],
        RouteReport::FastOwner { owner, changed, .. } => {
            let mut v = vec![owner.clone()];
            v.extend(changed.iter().cloned());
            v
        }
        _ => Vec::new(),
    };
    let mut report = WarmGateReport {
        changed_units: changed_units.clone(),
        violations: Vec::new(),
    };
    if stages.whole_total() > 0 {
        report
            .violations
            .push(format!("whole-axis attempts non-zero: {}", stages.whole_total()));
    }
    let is_changed = |unit: &str| changed_units.iter().any(|c| c == unit);
    for ((unit, stage), n) in &stages.by_unit {
        if !stage.is_production() || unit == "*" {
            continue;
        }
        let cap: u64 = if is_changed(unit) {
            match stage {
                // The identity wire (session-front) plus the canonical wire
                // (produce_unit) are two REAL encodes of a changed unit.
                Stage::AbcEncode => 2,
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
    for (unit, work) in body {
        let wire = format!("d_{unit}");
        let expected: u64 = if is_changed(&wire) { 1 } else { 0 };
        for (name, value) in [
            ("resolve", work.resolve),
            ("typecheck", work.typecheck),
            ("plan", work.plan),
            ("bind", work.bind),
        ] {
            if value > expected {
                report.violations.push(format!(
                    "unit {unit}: {value} {name} walks (cap {expected})"
                ));
            }
        }
    }
    // Every changed unit must ALSO show its work — a zero nobody recorded is
    // the "Default 0" control's red. A same-wire unit (e.g. whitespace-only
    // body edit) stops after encode: the session produce stages are then
    // legitimately absent for it.
    for changed in &changed_units {
        let plain = changed.strip_prefix("d_").unwrap_or(changed).to_string();
        let work = body.get(&plain).cloned().unwrap_or_default();
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
        // The owner's parse_project attempt is recorded once for the owner
        // batch (its lambdas' text is part of the owner item): re-resolved
        // lambdas inside an owner batch have no separate parse attempt.
        let produced = submitted_unit_names.iter().any(|u| u == changed);
        let owner_batch_lambda = matches!(route, RouteReport::FastOwner { owner, .. } if owner != changed);
        let mut required = vec![Stage::HirVerify, Stage::MirLower];
        if !owner_batch_lambda {
            required.push(Stage::ParseProject);
        }
        if produced {
            required.push(Stage::MirVerify);
        }
        for stage in required {
            if stages.of(changed, stage) < 1 {
                report
                    .violations
                    .push(format!("unit {changed}: missing {} attempt", stage.as_str()));
            }
        }
    }
    report
}
