//! The CTE driver: one query step end-to-end (admission → query
//! resolution → cache lookup or REAL AV execution → freeze/publish →
//! consumer substitution/infer → HIR backfill/re-verify → native
//! observable). Counters are the trust anchor: `ct_execute` counts real
//! AV calls only; `meta_query` counts Meta reads; hits never move
//! `ct_execute`.

use crate::av_driver::{self, AvExecResult};
use crate::observe::Obs;
use crate::store::{self, CteStore};
use ac_sema_contract::identity::domain_hash;
use ac_sema_core::adapter::{parse_into_input_typed, LanguageDiagnostic};
use ac_sema_core::SemaInput;
use auto_ac_prototype::native;
use auto_ac_prototype::source::SourceProfile;
use auto_ac_prototype::verify::CheckedModule;
use auto_av_core::CancelFlag;
use auto_av_probe::{mempeek, native_reference};
use auto_cte_contract::{
    CteAdmissionError, CteCode, CteDepsManifestV1, CteExecutedV1, CteExpectType,
    CteRequestIdentityV1, CteRequestV1, CteValue, CTE_EXECUTED_SCHEMA, CTE_PROBE_VERSION,
    CTE_PRODUCER, CTE_QUERY_ALGO_ID, CTE_RECIPE, CTE_SEMA_PACKAGE,
};
use auto_cte_core::consumer::{
    backfill_and_verify, resolve_bindings, substitute_and_infer, type_matches,
};
use auto_cte_core::dependencies::{build_manifest, manifest_fp};
use auto_cte_core::schedule::resolve_queries;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

/// Step outcome status.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StepStatus {
    Ok,
    Diag,
}

/// The observable result of one query step.
#[derive(Clone, Debug)]
pub struct CteStepOutcome {
    pub status: StepStatus,
    /// `cte.*` code on a diag outcome.
    pub code: Option<String>,
    pub message: Option<String>,
    pub language_class: bool,
    /// hit/miss/stale of the frozen layer (coarse value the runner
    /// judges). None when the step diaged before cache lookup.
    pub cache: Option<&'static str>,
    pub ct_execute: u64,
    pub meta_query: u64,
    pub query_nodes: usize,
    /// ct_ref -> frozen/executed CT value.
    pub values: BTreeMap<String, CteValue>,
    /// ct_ref -> inferred consumer binding type name.
    pub consumer_types: BTreeMap<String, String>,
    /// ct_ref -> backfilled HIR constant rendering (V-01 proof).
    pub hir_proof: BTreeMap<String, String>,
    /// Native observable exit of the BACKFILLED consumer (None when the
    /// deck disables native, no workdir was given, or the step diaged).
    pub native_exit: Option<i32>,
    pub wall_ms: f64,
    pub peak_private_bytes: Option<u64>,
    /// Managed-ledger peak of the live total (hot+staging+av slots).
    pub managed_peak_bytes: u64,
    /// The managed hard gate (§7): false on ANY over-cap reservation or
    /// an over-total peak.
    pub managed_target_met: bool,
    pub counters: BTreeMap<String, u64>,
    pub spans: BTreeMap<String, f64>,
}

impl CteStepOutcome {
    fn diag(e: &CteAdmissionError, cache: Option<&'static str>, obs: &Obs) -> Self {
        CteStepOutcome {
            status: StepStatus::Diag,
            code: Some(e.code.as_str().to_string()),
            message: Some(e.to_string()),
            language_class: e.is_language_class(),
            cache,
            ct_execute: obs.counters().get("ct_execute").copied().unwrap_or(0),
            meta_query: obs.counters().get("meta_query").copied().unwrap_or(0),
            query_nodes: obs.counters().get("query_nodes").copied().unwrap_or(0) as usize,
            values: BTreeMap::new(),
            consumer_types: BTreeMap::new(),
            hir_proof: BTreeMap::new(),
            native_exit: None,
            wall_ms: obs.request_wall_ms(),
            peak_private_bytes: None,
            managed_peak_bytes: 0,
            managed_target_met: true,
            counters: obs.counters().clone(),
            spans: obs.spans().clone(),
        }
    }
}

fn expect_name(e: CteExpectType) -> &'static str {
    match e {
        CteExpectType::I32 => "i32",
        CteExpectType::Bool => "bool",
    }
}

/// The driver-resident snapshot lane (source-synced, F-817-R1-02).
pub struct ResidentLane {
    pub lane: av_driver::SnapshotLane,
    /// The session source the lane was last synced to.
    pub src: String,
}

/// Contract §7 (F-817-R6-01): one CTE request's CUMULATIVE wall clock —
/// scheduling, cache, compile/execution, consumer and the necessary
/// waits — is bounded at 10 seconds. Per-AV-call caps stay untouched;
/// this is the request-total deadline.
pub const REQUEST_WALL_LIMIT: std::time::Duration = std::time::Duration::from_millis(10_000);

/// The per-request deadline state: ONE clock started at request
/// preparation; every phase checks it; the AV receives the remaining
/// wall as its per-call budget; a watchdog fires the cancel flag at the
/// limit so a running AV call aborts at its 1024-step poll.
pub(crate) struct RequestDeadline {
    at: std::time::Instant,
    limit_ms: u64,
    hit: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl RequestDeadline {
    pub(crate) fn exceeded(&self) -> bool {
        self.hit.load(std::sync::atomic::Ordering::Relaxed) || std::time::Instant::now() >= self.at
    }

    fn diag(&self, cache: Option<&'static str>, obs: &mut Obs) -> CteStepOutcome {
        let e = CteAdmissionError::new(
            CteCode::Resource("request-deadline".into()),
            format!(
                "request wall clock {:.0}ms exceeded the {}ms cumulative limit (resource state; nothing published)",
                obs.request_wall_ms(),
                self.limit_ms
            ),
        );
        CteStepOutcome::diag(&e, cache, obs)
    }

    /// The AV call's wall budget: never above the D-798 per-call cap,
    /// never past the request deadline.
    fn av_wall_budget(&self) -> std::time::Duration {
        let remaining = self.at.saturating_duration_since(std::time::Instant::now());
        remaining.min(av_driver::MAX_CALL_WALL)
    }
}

/// The deadline watchdog: sleeps the limit, then (unless the request
/// finished) marks the hit and fires the cancel flag.
pub(crate) struct DeadlineWatch {
    hit: std::sync::Arc<std::sync::atomic::AtomicBool>,
    done: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl DeadlineWatch {
    fn start(
        limit: std::time::Duration,
        cancel: std::sync::Arc<CancelFlag>,
    ) -> (RequestDeadline, DeadlineWatch) {
        let hit = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let hit2 = hit.clone();
        let done2 = done.clone();
        let handle = std::thread::Builder::new()
            .name("cte-request-deadline".into())
            .spawn(move || {
                // Tick-sleep (never a full-limit sleep): `stop()` joins
                // this thread, so it must observe `done` within one
                // tick — a request that finishes early never waits for
                // its own deadline.
                let at = std::time::Instant::now() + limit;
                while !done2.load(std::sync::atomic::Ordering::Relaxed) {
                    let now = std::time::Instant::now();
                    if now >= at {
                        if !done2.load(std::sync::atomic::Ordering::Relaxed) {
                            hit2.store(true, std::sync::atomic::Ordering::Relaxed);
                            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        break;
                    }
                    let remaining = at - now;
                    std::thread::sleep(remaining.min(std::time::Duration::from_millis(20)));
                }
            })
            .expect("deadline watchdog thread");
        let limit_ms = limit.as_millis() as u64;
        (
            RequestDeadline {
                at: std::time::Instant::now() + limit,
                limit_ms,
                hit: hit.clone(),
            },
            DeadlineWatch {
                hit,
                done,
                handle: Some(handle),
            },
        )
    }

    /// Disarm (the request ended) and report whether the deadline fired.
    fn stop(mut self) -> bool {
        self.done.store(true, std::sync::atomic::Ordering::Relaxed);
        let fired = self.hit.load(std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        fired
    }
}

/// The driver over one optional store root.
pub struct CteDriver {
    store: Option<CteStore>,
    forbid_av: bool,
    /// Per-REQUEST cancellation (V-03): set before or during a query
    /// step; the step surfaces the isolated `Cancelled` state (never a
    /// language verdict), publishes nothing, and the flag auto-clears
    /// for the next request.
    cancel: std::sync::Arc<CancelFlag>,
    /// Managed-budget caps. TEST COUNTER-CONTROL SEAM: production keeps
    /// the frozen defaults; the resources tier shrinks partitions to
    /// make over-budget/spike shapes deterministic.
    caps: crate::observe::BudgetCaps,
    /// Per-request wall limit. `None` = the frozen 10s contract default
    /// (F-817-R6-01); the resources tier shortens it for fast
    /// classification controls — never in production.
    request_wall: Option<std::time::Duration>,
    /// The last request's ordered budget events (reservation/allocation
    /// order evidence, F-817-R6-02).
    last_budget_events: Vec<crate::observe::BudgetEvent>,
    /// Artifact id of the LAST successful frozen publish on this driver
    /// (lifecycle tests lease/pin real published entries).
    last_publish: Option<String>,
    /// Byte length of the last serialized envelope (spike sizing).
    last_envelope_bytes: Option<u64>,
    /// The RESIDENT snapshot lane (F-817-R1-02): the PLAN-828 session
    /// survives ACROSS requests — unchanged functions keep their code
    /// objects and generations; source changes sync via single-hunk
    /// incremental edits, and only a rejected edit re-warms the whole
    /// module (counted, never silent).
    resident: Option<ResidentLane>,
    /// Whole-module re-warms of the resident lane (the forced-rebuild
    /// counter; 0 across pure incremental traffic).
    snapshot_rewarms: u64,
}

impl CteDriver {
    pub fn ephemeral() -> Self {
        CteDriver {
            store: None,
            forbid_av: false,
            cancel: std::sync::Arc::new(CancelFlag::new(false)),
            caps: crate::observe::BudgetCaps::default(),
            request_wall: None,
            last_budget_events: Vec::new(),
            last_publish: None,
            last_envelope_bytes: None,
            resident: None,
            snapshot_rewarms: 0,
        }
    }

    pub fn with_store(root: &Path, project: &str) -> Result<Self, String> {
        Ok(CteDriver {
            store: Some(CteStore::open(root, project)?),
            forbid_av: false,
            cancel: std::sync::Arc::new(CancelFlag::new(false)),
            caps: crate::observe::BudgetCaps::default(),
            request_wall: None,
            last_budget_events: Vec::new(),
            last_publish: None,
            last_envelope_bytes: None,
            resident: None,
            snapshot_rewarms: 0,
        })
    }

    /// Test counter-control (V-05): with AV forbidden, a cache HIT still
    /// completes and a MISS becomes a controlled infra error — proving
    /// hits never execute the AV.
    pub fn set_forbid_av(&mut self, v: bool) {
        self.forbid_av = v;
    }

    /// The resident snapshot lane (observations for the V-12 oracle).
    pub fn snapshot_lane(&self) -> Option<&ResidentLane> {
        self.resident.as_ref()
    }

    /// Whole-module re-warms of the resident lane (forced-rebuild count).
    pub fn snapshot_rewarms(&self) -> u64 {
        self.snapshot_rewarms
    }

    /// The artifact id of the last successful frozen publish (None when
    /// the last step published nothing — a hit, a diag, or no store).
    pub fn last_published_artifact(&self) -> Option<&str> {
        self.last_publish.as_deref()
    }

    /// Byte length of the last serialized envelope (None when the last
    /// step published nothing).
    pub fn last_envelope_bytes(&self) -> Option<u64> {
        self.last_envelope_bytes
    }

    /// Attach (or keep) the store on this driver — the stream worker
    /// builds a driver once and attaches per request root.
    pub fn attach_store(&mut self, root: &Path, project: &str) -> Result<(), String> {
        if self.store.is_none() {
            self.store = Some(CteStore::open(root, project)?);
        }
        Ok(())
    }

    /// Test-only: shrink the managed partitions (counter-control seam).
    pub fn set_managed_caps(&mut self, caps: crate::observe::BudgetCaps) {
        self.caps = caps;
    }

    /// Test-only: override the per-request cumulative wall limit
    /// (F-817-R6-01 counter-control seam). Production always runs the
    /// frozen 10s contract default. Tests shorten it for fast
    /// classification controls; the debug scale tier (multi-call
    /// requests at unoptimized speed) lengthens it so the reference
    /// RELEASE bound is not misattributed to debug timing — the frozen
    /// default itself is proven by the real-10s replay.
    pub fn set_request_wall_limit(&mut self, limit: std::time::Duration) {
        self.request_wall = Some(limit);
    }

    /// The last request's ordered budget events — reservation before
    /// allocation is observable here, not just claimed (F-817-R6-02).
    pub fn last_budget_events(&self) -> &[crate::observe::BudgetEvent] {
        &self.last_budget_events
    }

    /// Fire the per-request cancellation flag.
    pub fn cancel(&self) {
        self.cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// The shared flag (a test thread can hold it mid-flight).
    pub fn cancel_flag(&self) -> std::sync::Arc<CancelFlag> {
        self.cancel.clone()
    }

    pub fn store(&self) -> Option<&CteStore> {
        self.store.as_ref()
    }

    /// Run one query step over the CURRENT deck text.
    pub fn query_step(&mut self, deck_text: &str, native_workdir: Option<&Path>) -> CteStepOutcome {
        // A pre-set flag cancels THIS request outright (consume it);
        // the request itself starts with a clear flag.
        let pre_cancelled = self
            .cancel
            .swap(false, std::sync::atomic::Ordering::Relaxed);
        if pre_cancelled {
            let obs = Obs::new();
            let e = CteAdmissionError::new(CteCode::Cancelled, "request cancelled before start");
            return CteStepOutcome::diag(&e, None, &obs);
        }
        let mut obs = Obs::new();
        let mut budget = crate::observe::ManagedBudget::new(self.caps);
        let sampler = mempeek::PeakSampler::start(1);
        // F-817-R6-01: ONE cumulative request deadline — from request
        // preparation through scheduling, cache, execution, consumer
        // and publish. A watchdog interrupts a running AV call via the
        // cancel flag; phase checks reject past the limit.
        let limit = self.request_wall.unwrap_or(REQUEST_WALL_LIMIT);
        let (dl, watch) = DeadlineWatch::start(limit, self.cancel.clone());
        let mut out = self.query_step_inner(deck_text, native_workdir, &mut obs, &mut budget, &dl);
        let deadline_fired = watch.stop();
        self.last_budget_events = budget.events().to_vec();
        let peak = sampler.stop();
        let cancelled = self
            .cancel
            .swap(false, std::sync::atomic::Ordering::Relaxed);
        if deadline_fired && out.status == StepStatus::Ok {
            // Slipped past the deadline at the boundary — the resource
            // state wins over success (in-loop checks gate publishes;
            // this completion is never reported green).
            let e = CteAdmissionError::new(
                CteCode::Resource("request-deadline".into()),
                format!(
                    "request wall clock {:.0}ms exceeded the {}ms cumulative limit (resource state; late completion not admitted)",
                    obs.request_wall_ms(),
                    limit.as_millis()
                ),
            );
            out = CteStepOutcome::diag(&e, out.cache, &mut obs);
        } else if cancelled && out.status == StepStatus::Ok {
            // The flag fired after the last poll but before completion —
            // surface the isolated state; nothing was published late.
            out = CteStepOutcome::diag(
                &CteAdmissionError::new(CteCode::Cancelled, "request cancelled before completion"),
                out.cache,
                &mut obs,
            );
        }
        CteStepOutcome {
            peak_private_bytes: Some(peak),
            managed_peak_bytes: budget.peak_live_total(),
            managed_target_met: budget.target_met(),
            ..out
        }
    }

    fn query_step_inner(
        &mut self,
        deck_text: &str,
        native_workdir: Option<&Path>,
        obs: &mut Obs,
        budget: &mut crate::observe::ManagedBudget,
        dl: &RequestDeadline,
    ) -> CteStepOutcome {
        // The request text itself is the first staging reservation —
        // BEFORE any parse/clone.
        if let Err(m) = budget.reserve(
            crate::observe::Partition::Staging,
            deck_text.len() as u64,
            "request deck",
        ) {
            let e = CteAdmissionError::new(CteCode::Resource("managed-staging".into()), m);
            return CteStepOutcome::diag(&e, None, obs);
        }
        if dl.exceeded() {
            return dl.diag(None, obs);
        }
        let req = match obs.span("deck_parse", |_| crate::deck::parse_deck(deck_text)) {
            Ok(r) => r,
            Err(e) => return CteStepOutcome::diag(&e, None, obs),
        };
        // The embedded source dominates the projection scratch — reserve
        // a small multiple before projecting.
        if let Err(m) = budget.reserve(
            crate::observe::Partition::Staging,
            (req.source.len() as u64).saturating_mul(4),
            "source projection",
        ) {
            let e = CteAdmissionError::new(CteCode::Resource("managed-staging".into()), m);
            return CteStepOutcome::diag(&e, None, obs);
        }
        obs.set_counter("query_nodes", req.queries.len() as u64);
        if self.cancel.load(std::sync::atomic::Ordering::Relaxed) {
            let e = CteAdmissionError::new(CteCode::Cancelled, "request cancelled");
            return CteStepOutcome::diag(&e, None, obs);
        }

        // Projection of the ORIGINAL source (consumer + callee decls),
        // under the deck's declared profile.
        let ac_profile = match req.profile {
            auto_cte_contract::CteSourceProfile::Core => SourceProfile::Core,
            auto_cte_contract::CteSourceProfile::Record => SourceProfile::Record,
        };
        let mut input = match obs.span("project", |_| {
            parse_into_input_typed(&req.source, CTE_SEMA_PACKAGE, &req.module, ac_profile)
        }) {
            Ok(i) => i,
            Err(e) => {
                let msg = match &e {
                    LanguageDiagnostic::Parse { code, message } => {
                        format!("parse: {message} ({code})")
                    }
                    LanguageDiagnostic::Projection(m) => format!("projection: {m}"),
                };
                let err = CteAdmissionError::new(CteCode::Infra("frontend".into()), msg);
                return CteStepOutcome::diag(&err, None, obs);
            }
        };
        // The meta block (F-817-R1-05): record declarations project under
        // the RECORD profile and merge into the core input's record
        // domain — FieldCount/HasField read them and their results feed
        // as REAL AV arguments, with freshness riding the manifest.
        if let Some(meta_src) = &req.meta_source {
            let meta_input = match obs.span("project_meta", |_| {
                parse_into_input_typed(
                    meta_src,
                    CTE_SEMA_PACKAGE,
                    &req.module,
                    SourceProfile::Record,
                )
            }) {
                Ok(i) => i,
                Err(e) => {
                    let msg = match &e {
                        LanguageDiagnostic::Parse { code, message } => {
                            format!("meta parse: {message} ({code})")
                        }
                        LanguageDiagnostic::Projection(m) => format!("meta projection: {m}"),
                    };
                    let err = CteAdmissionError::new(CteCode::ConsumerFrontend, msg);
                    return CteStepOutcome::diag(&err, None, obs);
                }
            };
            input.records.extend(meta_input.records);
        }

        // Binding resolution against the projection (placeholder drift,
        // unknown owners, ambiguous anchors all reject here).
        let resolved = match obs.span("bind_resolve", |_| resolve_bindings(&input, &req.bindings)) {
            Ok(r) => r,
            Err(e) => return CteStepOutcome::diag(&e, None, obs),
        };

        // Query resolution (literals/meta now; AV calls planned).
        let plan = match obs.span("query_plan", |_| resolve_queries(&req, &input)) {
            Ok(p) => p,
            Err(e) => return CteStepOutcome::diag(&e, None, obs),
        };
        obs.set_counter("meta_query", plan.meta.len() as u64);
        if dl.exceeded() {
            return dl.diag(None, obs);
        }

        // ---- AV calls: cache lookup or real execution -------------------
        // The engine identity carries the EXECUTION LANE (V-12): closure
        // and snapshot requests never share cache entries.
        let engine = av_driver::current_engine_for(req.exec.axis());
        let mut cache_verdict: Option<&'static str> = None;
        let mut values_by_query: BTreeMap<String, CteValue> = plan.values.clone();

        // The frozen envelope carrier is one REASSEMBLED LayoutData
        // artifact (the store chunks pages and get_artifact joins
        // them). Legit envelopes are fingerprint manifests (~87B per
        // closure member, ≈3.2× the source — structurally under the
        // 4× projection reserve), so 1MiB covers every reachable
        // envelope; a larger payload is foreign/corrupt — a controlled
        // infra rejection (F-817-R6-02: reserve BEFORE get_artifact
        // allocates).
        const FROZEN_PAYLOAD_BOUND: u64 = 1024 * 1024;

        for call in &plan.av_calls {
            if dl.exceeded() {
                return dl.diag(cache_verdict, obs);
            }
            // RAW args concretize NOW — every earlier call's value is
            // present by execution order (legal DAG deps).
            let mut concrete: Vec<CteValue> = Vec::with_capacity(call.args.len());
            for a in &call.args {
                match auto_cte_core::schedule::concretize(a, &values_by_query, &call.callee) {
                    Ok(v) => concrete.push(v),
                    Err(e) => return CteStepOutcome::diag(&e, cache_verdict, obs),
                }
            }
            let manifest = match build_manifest(
                &input,
                &call.callee,
                &req.caps,
                &plan.meta_records,
                &plan.meta_probes,
            ) {
                Ok(m) => m,
                Err(m) => {
                    let e = CteAdmissionError::new(
                        CteCode::UnknownQueryRef,
                        format!("callee manifest: {m}"),
                    );
                    return CteStepOutcome::diag(&e, cache_verdict, obs);
                }
            };
            let mfp = manifest_fp(&manifest);
            let identity = CteRequestIdentityV1 {
                module: req.module.clone(),
                package: CTE_SEMA_PACKAGE.into(),
                callee: call.callee.clone(),
                args_canonical: store::args_canonical(&concrete),
                query_algo: CTE_QUERY_ALGO_ID.into(),
                caps: req.caps.clone(),
                engine: engine.clone(),
                manifest_fp: mfp,
            };
            // Cache lookup first — a hit must never touch the AV. The
            // load and decode are budget-gated IN ORDER (F-817-R6-02):
            // the codec-page bound is reserved BEFORE get_artifact
            // allocates the payload; the payload length (a sound upper
            // bound for the decoded structure) is reserved Hot BEFORE
            // from_slice allocates the envelope.
            let (cached, held_hot): (Option<CteExecutedV1>, u64) = match self.store.as_ref() {
                Some(st) => {
                    if let Err(m) = budget.reserve(
                        crate::observe::Partition::Staging,
                        FROZEN_PAYLOAD_BOUND,
                        "frozen payload load",
                    ) {
                        let e =
                            CteAdmissionError::new(CteCode::Resource("managed-staging".into()), m);
                        return CteStepOutcome::diag(&e, cache_verdict, obs);
                    }
                    match obs.span("cache_lookup", |_| {
                        store::load_frozen_payload(st, &identity)
                    }) {
                        Ok(Some(payload)) => {
                            if (payload.len() as u64) > FROZEN_PAYLOAD_BOUND {
                                let e = CteAdmissionError::new(
                                    CteCode::Infra("store".into()),
                                    format!(
                                        "frozen artifact payload {}B exceeds the {}B codec page bound",
                                        payload.len(),
                                        FROZEN_PAYLOAD_BOUND
                                    ),
                                );
                                return CteStepOutcome::diag(&e, cache_verdict, obs);
                            }
                            budget.record_alloc("frozen-payload", payload.len() as u64);
                            if let Err(m) = budget.reserve(
                                crate::observe::Partition::Hot,
                                payload.len() as u64,
                                "restored envelope decode",
                            ) {
                                let e = CteAdmissionError::new(
                                    CteCode::Resource("managed-hot".into()),
                                    m,
                                );
                                return CteStepOutcome::diag(&e, cache_verdict, obs);
                            }
                            match obs.span("cache_restore_decode", |_| {
                                store::decode_envelope(&payload, &identity)
                            }) {
                                Ok(env) => {
                                    budget.record_alloc("restore-decode", payload.len() as u64);
                                    budget.release(
                                        crate::observe::Partition::Staging,
                                        FROZEN_PAYLOAD_BOUND,
                                    );
                                    (Some(env), payload.len() as u64)
                                }
                                Err(e) => return CteStepOutcome::diag(&e, cache_verdict, obs),
                            }
                        }
                        Ok(None) => {
                            budget
                                .release(crate::observe::Partition::Staging, FROZEN_PAYLOAD_BOUND);
                            (None, 0)
                        }
                        Err(e) => return CteStepOutcome::diag(&e, cache_verdict, obs),
                    }
                }
                None => (None, 0),
            };
            let restored_held = held_hot;
            let value = if let Some(env) = cached {
                let fresh = obs.span("freshness", |_| {
                    store::frozen_is_fresh(&env, &input, &req.caps)
                });
                if fresh {
                    obs.counter("cache_hits", 1);
                    if cache_verdict.is_none() {
                        cache_verdict = Some("hit");
                    }
                    env.value
                } else {
                    // Invalidated entry: recompute. The new manifest fp
                    // re-keys the next publish; the old row survives
                    // untouched (no in-place rebuild).
                    if cache_verdict.is_none() {
                        cache_verdict = Some("stale");
                    }
                    let wall = dl.av_wall_budget();
                    let exec = match self.execute_call(
                        obs,
                        &req,
                        &input,
                        call,
                        &concrete,
                        cache_verdict,
                        wall,
                        dl,
                    ) {
                        Ok(x) => x,
                        Err(out) => return out,
                    };
                    budget.settle(crate::observe::Partition::AvSlots, exec.peak_slot_bytes);
                    if dl.exceeded() {
                        return dl.diag(cache_verdict, obs);
                    }
                    let env = self.mint_envelope(&req, &identity, &exec, manifest);
                    if let Err(e) = self.publish_env(&env, budget, dl) {
                        return CteStepOutcome::diag(&e, cache_verdict, obs);
                    }
                    exec.value
                }
            } else {
                if cache_verdict.is_none() {
                    cache_verdict = Some("miss");
                }
                let wall = dl.av_wall_budget();
                if wall.is_zero() {
                    return dl.diag(cache_verdict, obs);
                }
                let exec = match self.perform_execution(
                    obs,
                    &req,
                    &input,
                    call,
                    &concrete,
                    cache_verdict,
                    wall,
                    dl,
                ) {
                    Ok(x) => x,
                    Err(out) => return out,
                };
                budget.settle(crate::observe::Partition::AvSlots, exec.peak_slot_bytes);
                if dl.exceeded() {
                    return dl.diag(cache_verdict, obs);
                }
                let env = self.mint_envelope(&req, &identity, &exec, manifest);
                if let Err(e) = self.publish_env(&env, budget, dl) {
                    return CteStepOutcome::diag(&e, cache_verdict, obs);
                }
                exec.value
            };
            // The restored envelope is consumed — return its Hot bytes.
            if restored_held > 0 {
                budget.release(crate::observe::Partition::Hot, restored_held);
            }
            for qid in &call.query_ids {
                values_by_query.insert(qid.clone(), value);
            }
        }

        // ---- binding values + expected-type checks ----------------------
        if dl.exceeded() {
            return dl.diag(cache_verdict, obs);
        }
        let mut ct_values: BTreeMap<String, CteValue> = BTreeMap::new();
        for b in &req.bindings {
            let Some(v) = values_by_query.get(&b.source_query) else {
                let e = CteAdmissionError::new(
                    CteCode::UnknownQueryRef,
                    format!(
                        "binding `{}` references unknown query `{}`",
                        b.ct_ref, b.source_query
                    ),
                );
                return CteStepOutcome::diag(&e, cache_verdict, obs);
            };
            if !b.expect.matches_value(v) {
                let e = CteAdmissionError::new(
                    CteCode::BindingTypeMismatch,
                    format!(
                        "binding `{}` expects {} but query `{}` produced {}",
                        b.ct_ref,
                        expect_name(b.expect),
                        b.source_query,
                        v.type_name()
                    ),
                );
                return CteStepOutcome::diag(&e, cache_verdict, obs);
            }
            ct_values.insert(b.ct_ref.clone(), *v);
        }

        // ---- consumer: substitution + infer -----------------------------
        let mut consumer_input = input.clone();
        let derived = match obs.span("consumer_infer", |_| {
            substitute_and_infer(&mut consumer_input, &resolved, &ct_values)
        }) {
            Ok(d) => d,
            Err(e) => return CteStepOutcome::diag(&e, cache_verdict, obs),
        };
        let mut consumer_types = BTreeMap::new();
        for (r, (ct_ref, ty)) in resolved.iter().zip(derived.iter()) {
            if !type_matches(r.binding.expect, ty) {
                let e = CteAdmissionError::new(
                    CteCode::BindingTypeMismatch,
                    format!(
                        "binding `{}` expected {} but consumer inference derived {ty}",
                        r.binding.ct_ref,
                        expect_name(r.binding.expect)
                    ),
                );
                return CteStepOutcome::diag(&e, cache_verdict, obs);
            }
            consumer_types.insert(ct_ref.clone(), ty.clone());
        }

        // ---- consumer: HIR backfill + re-verify -------------------------
        if dl.exceeded() {
            return dl.diag(cache_verdict, obs);
        }
        let (checked, hir_proof) = match obs.span("hir_backfill", |_| {
            backfill_and_verify(
                &req.source,
                &req.module,
                &resolved,
                &ct_values,
                req.profile,
                &req.entry,
            )
        }) {
            Ok(x) => x,
            Err(e) => return CteStepOutcome::diag(&e, cache_verdict, obs),
        };

        // ---- native observable (backfilled HIR, never a re-parse) ------
        let mut native_exit = None;
        if req.native {
            if let Some(workdir) = native_workdir {
                match run_native(&checked, &req.entry, workdir, obs) {
                    Ok(code) => native_exit = Some(code),
                    Err(e) => return CteStepOutcome::diag(&e, cache_verdict, obs),
                }
            }
        }

        CteStepOutcome {
            status: StepStatus::Ok,
            code: None,
            message: None,
            language_class: false,
            cache: cache_verdict,
            ct_execute: obs.counters().get("ct_execute").copied().unwrap_or(0),
            meta_query: obs.counters().get("meta_query").copied().unwrap_or(0),
            query_nodes: req.queries.len(),
            values: ct_values,
            consumer_types,
            hir_proof,
            native_exit,
            wall_ms: obs.request_wall_ms(),
            peak_private_bytes: None,
            managed_peak_bytes: budget.peak_live_total(),
            managed_target_met: budget.target_met(),
            counters: obs.counters().clone(),
            spans: obs.spans().clone(),
        }
    }

    /// Execute one call through the request's lane. The closure lane
    /// compiles per call; the snapshot lane warms once and retargets.
    /// `wall_budget` is the deadline-derived per-call AV wall budget;
    /// `dl` reclassifies deadline-driven aborts precisely.
    fn perform_execution(
        &mut self,
        obs: &mut Obs,
        req: &CteRequestV1,
        input: &SemaInput,
        call: &auto_cte_core::schedule::AvCall,
        concrete: &[CteValue],
        cache_verdict: Option<&'static str>,
        wall_budget: std::time::Duration,
        dl: &RequestDeadline,
    ) -> Result<AvExecResult, CteStepOutcome> {
        if self.forbid_av {
            let e = CteAdmissionError::new(
                CteCode::Infra("av-forbidden".into()),
                "AV execution is forbidden by the counter-control flag (a cache miss cannot complete)",
            );
            return Err(CteStepOutcome::diag(&e, cache_verdict, obs));
        }
        let exec_started = std::time::Instant::now();
        let out_res: Result<AvExecResult, CteAdmissionError> = if req.exec
            == auto_cte_contract::CteExecMode::Snapshot
        {
            let is_bool = av_driver::callee_returns_bool(input, &call.callee);
            // F-817-R1-02: sync the RESIDENT lane to this call's
            // wrapper source (single-hunk incremental edit; a
            // rejected edit re-warms the whole module, counted).
            let desired = av_driver::wrapper_source(&req.source, &call.callee, concrete, is_bool);
            let mut warm_err: Option<CteAdmissionError> = None;
            match self.resident.as_mut() {
                Some(r) if r.src == desired => {}
                Some(r) => match sync_lane_source(&mut r.lane, &r.src, &desired) {
                    Ok(()) => {
                        r.src = desired.clone();
                        obs.counter("snapshot_incremental_syncs", 1);
                    }
                    Err(_) => {
                        self.snapshot_rewarms += 1;
                        self.resident = None;
                        obs.counter("snapshot_rewarms", 1);
                    }
                },
                None => {}
            }
            if self.resident.is_none() {
                match av_driver::SnapshotLane::warm(&req.source, &call.callee, concrete, is_bool) {
                    Ok(l) => {
                        self.resident = Some(ResidentLane {
                            lane: l,
                            src: desired,
                        });
                    }
                    Err(e) => warm_err = Some(e),
                }
            }
            if let Some(e) = warm_err {
                Err(e)
            } else {
                // The lane was either freshly warmed on THIS call or
                // synced to it — its wrapper already carries the
                // callee/args/shape, so run DIRECTLY (a redundant
                // whole-body retarget after a sync is an identity
                // edit the incremental engine rightly refuses).
                let r = self.resident.as_ref().expect("resident lane");
                r.lane.run(is_bool, &self.cancel, wall_budget)
            }
        } else {
            let cancel = self.cancel.clone();
            av_driver::execute_call(
                &req.source,
                &req.module,
                input,
                &call.callee,
                concrete,
                self.forbid_av,
                &cancel,
                wall_budget,
            )
        };
        obs.record_span("av_execute", exec_started.elapsed().as_secs_f64() * 1000.0);
        let out = out_res;
        match out {
            Ok(x) => {
                obs.counter("ct_execute", 1);
                Ok(x)
            }
            Err(e) => {
                if matches!(
                    e.code,
                    CteCode::Trap(_) | CteCode::Resource(_) | CteCode::Cancelled
                ) {
                    obs.counter("ct_execute", 1);
                }
                // A deadline-driven abort (the watchdog fired the cancel
                // flag / the wall budget closed) reports as the request
                // deadline resource state, precisely.
                if dl.exceeded() {
                    return Err(dl.diag(cache_verdict, obs));
                }
                Err(CteStepOutcome::diag(&e, cache_verdict, obs))
            }
        }
    }

    fn execute_call(
        &mut self,
        obs: &mut Obs,
        req: &CteRequestV1,
        input: &SemaInput,
        call: &auto_cte_core::schedule::AvCall,
        concrete: &[CteValue],
        cache_verdict: Option<&'static str>,
        wall_budget: std::time::Duration,
        dl: &RequestDeadline,
    ) -> Result<AvExecResult, CteStepOutcome> {
        let forbid = self.forbid_av;
        let cancel = self.cancel.clone();
        let out = obs.span("av_execute", |_| {
            av_driver::execute_call(
                &req.source,
                &req.module,
                input,
                &call.callee,
                concrete,
                forbid,
                &cancel,
                wall_budget,
            )
        });
        match out {
            Ok(x) => {
                obs.counter("ct_execute", 1);
                Ok(x)
            }
            Err(e) => {
                // A trap/resource/cancelled outcome still REACHED the AV:
                // the execution fact counts (admission/pipeline faults
                // before the executor do not).
                if matches!(
                    e.code,
                    CteCode::Trap(_) | CteCode::Resource(_) | CteCode::Cancelled
                ) {
                    obs.counter("ct_execute", 1);
                }
                if dl.exceeded() {
                    return Err(dl.diag(cache_verdict, obs));
                }
                Err(CteStepOutcome::diag(&e, cache_verdict, obs))
            }
        }
    }

    fn mint_envelope(
        &self,
        req: &CteRequestV1,
        identity: &CteRequestIdentityV1,
        exec: &AvExecResult,
        manifest: CteDepsManifestV1,
    ) -> CteExecutedV1 {
        let source_fp = domain_hash("ac817/source-fp/v1", &[req.source.as_bytes()]);
        CteExecutedV1 {
            schema: CTE_EXECUTED_SCHEMA.to_string(),
            recipe: CTE_RECIPE.to_string(),
            request_identity: identity.clone(),
            value: exec.value,
            declared_type: match exec.value {
                CteValue::I32(_) => CteExpectType::I32,
                CteValue::Bool(_) => CteExpectType::Bool,
            },
            producer: CTE_PRODUCER.into(),
            engine: identity.engine.clone(),
            exec_steps: exec.steps,
            exec_wall_ms: exec.wall_ms,
            deps: manifest,
            source_fp,
        }
    }

    fn publish_env(
        &mut self,
        env: &CteExecutedV1,
        budget: &mut crate::observe::ManagedBudget,
        dl: &RequestDeadline,
    ) -> Result<(), CteAdmissionError> {
        let Some(st) = self.store.as_ref() else {
            return Ok(());
        };
        // F-817-R6-02: reserve BEFORE the encode allocation — count the
        // bytes WITHOUT allocating, reserve the exact count, then
        // serialize; the payload MOVES into the store (no second
        // unreserved copy). A late completion past the deadline never
        // publishes.
        if dl.exceeded() {
            return Err(CteAdmissionError::new(
                CteCode::Resource("request-deadline".into()),
                "request deadline reached before publish (nothing published)".to_string(),
            ));
        }
        let payload = store::serialize_envelope_reserved(env, budget)?;
        self.last_envelope_bytes = Some(payload.len() as u64);
        let n = payload.len() as u64;
        let out = store::publish_frozen_payload(st, env, payload).map(|id| {
            self.last_publish = Some(id);
        });
        budget.release(crate::observe::Partition::Staging, n);
        out
    }
}

/// Native observable through the merged 798 reference path — the
/// BACKFILLED checked module goes straight to `build_stages` (no source
/// re-parse ever sees the CT value).
fn run_native(
    checked: &CheckedModule,
    entry: &str,
    workdir: &Path,
    obs: &mut Obs,
) -> Result<i32, CteAdmissionError> {
    // HIR def ids are `d_`-prefixed (`d_main`); the deck names the
    // source-level function.
    let entry_id = format!("d_{entry}");
    // Entry admission through the real backend check (0-param i32).
    if let Err(ds) = native::find_entry(checked, &entry_id) {
        let d = ds.first().cloned();
        return Err(CteAdmissionError::new(
            CteCode::EntryMissing,
            format!(
                "native entry: {}",
                d.map(|x| format!("{} {}", x.code, x.message))
                    .unwrap_or_else(|| "entry not found".into())
            ),
        ));
    }
    let (_obj, exe, _receipt) = obs.span("native_build", |_| {
        native_reference::build_stages(checked, &entry_id, workdir, None)
            .map_err(|e| CteAdmissionError::new(CteCode::Infra("native-build".into()), e))
    })?;
    let (code, _so, _se) = obs.span("native_run", |_| {
        native_reference::run_stage(&exe, Duration::from_millis(5000), 1024 * 1024, 1024 * 1024)
            .map_err(|e| CteAdmissionError::new(CteCode::Infra("native-run".into()), e))
    })?;
    let code = code.ok_or_else(|| {
        CteAdmissionError::new(
            CteCode::Infra("native-run".into()),
            "native process terminated abnormally (no exit code)",
        )
    })?;
    Ok(code)
}

/// Probe version surfaced by the worker handshake.
pub fn probe_version() -> &'static str {
    CTE_PROBE_VERSION
}

/// Sync a resident lane's session source to `desired` via ONE single-
/// hunk incremental edit (common prefix/suffix trim). A rejected edit is
/// an Err — the caller re-warms and counts it (never a silent rebuild).
fn sync_lane_source(
    lane: &mut av_driver::SnapshotLane,
    old_src: &str,
    desired: &str,
) -> Result<(), String> {
    let old = old_src.as_bytes();
    let new = desired.as_bytes();
    let mut p = 0usize;
    while p < old.len() && p < new.len() && old[p] == new[p] {
        p += 1;
    }
    let mut suf = 0usize;
    while suf < old.len() - p
        && suf < new.len() - p
        && old[old.len() - 1 - suf] == new[new.len() - 1 - suf]
    {
        suf += 1;
    }
    let start = p as u64;
    let end = (old.len() - suf) as u64;
    let insert = new[p..new.len() - suf].to_vec();
    if start == end && insert.is_empty() {
        return Ok(());
    }
    lane.edit(start, end, insert)
}
