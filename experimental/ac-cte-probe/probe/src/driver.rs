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

/// The driver over one optional store root.
pub struct CteDriver {
    store: Option<CteStore>,
    forbid_av: bool,
}

impl CteDriver {
    pub fn ephemeral() -> Self {
        CteDriver {
            store: None,
            forbid_av: false,
        }
    }

    pub fn with_store(root: &Path, project: &str) -> Result<Self, String> {
        Ok(CteDriver {
            store: Some(CteStore::open(root, project)?),
            forbid_av: false,
        })
    }

    /// Test counter-control (V-05): with AV forbidden, a cache HIT still
    /// completes and a MISS becomes a controlled infra error — proving
    /// hits never execute the AV.
    pub fn set_forbid_av(&mut self, v: bool) {
        self.forbid_av = v;
    }

    pub fn store(&self) -> Option<&CteStore> {
        self.store.as_ref()
    }

    /// Run one query step over the CURRENT deck text.
    pub fn query_step(&mut self, deck_text: &str, native_workdir: Option<&Path>) -> CteStepOutcome {
        let mut obs = Obs::new();
        let sampler = mempeek::PeakSampler::start(20);
        let out = self.query_step_inner(deck_text, native_workdir, &mut obs);
        let peak = sampler.stop();
        CteStepOutcome {
            peak_private_bytes: Some(peak),
            ..out
        }
    }

    fn query_step_inner(
        &mut self,
        deck_text: &str,
        native_workdir: Option<&Path>,
        obs: &mut Obs,
    ) -> CteStepOutcome {
        let req = match obs.span("deck_parse", |_| crate::deck::parse_deck(deck_text)) {
            Ok(r) => r,
            Err(e) => return CteStepOutcome::diag(&e, None, obs),
        };
        obs.set_counter("query_nodes", req.queries.len() as u64);

        // Projection of the ORIGINAL source (consumer + callee decls).
        let input = match obs.span("project", |_| {
            parse_into_input_typed(
                &req.source,
                CTE_SEMA_PACKAGE,
                &req.module,
                SourceProfile::Core,
            )
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

        // ---- AV calls: cache lookup or real execution -------------------
        let engine = av_driver::current_engine();
        let mut cache_verdict: Option<&'static str> = None;
        let mut values_by_query: BTreeMap<String, CteValue> = plan.values.clone();
        for call in &plan.av_calls {
            let manifest = match build_manifest(&input, &call.callee, &req.caps) {
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
                args_canonical: store::args_canonical(&call.args),
                query_algo: CTE_QUERY_ALGO_ID.into(),
                caps: req.caps.clone(),
                engine: engine.clone(),
                manifest_fp: mfp,
            };
            // Cache lookup first — a hit must never touch the AV.
            let cached = match self.store.as_ref() {
                Some(st) => {
                    match obs.span("cache_lookup", |_| store::lookup_frozen(st, &identity)) {
                        Ok(opt) => opt,
                        Err(e) => return CteStepOutcome::diag(&e, cache_verdict, obs),
                    }
                }
                None => None,
            };
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
                    let exec = match self.execute_call(obs, &req, &input, call, cache_verdict) {
                        Ok(x) => x,
                        Err(out) => return out,
                    };
                    let env = self.mint_envelope(&req, &identity, &exec, manifest);
                    if let Err(e) = obs.span("freeze_publish", |_| self.publish_env(&env)) {
                        return CteStepOutcome::diag(&e, cache_verdict, obs);
                    }
                    exec.value
                }
            } else {
                if cache_verdict.is_none() {
                    cache_verdict = Some("miss");
                }
                let exec = match self.execute_call(obs, &req, &input, call, cache_verdict) {
                    Ok(x) => x,
                    Err(out) => return out,
                };
                let env = self.mint_envelope(&req, &identity, &exec, manifest);
                if let Err(e) = obs.span("freeze_publish", |_| self.publish_env(&env)) {
                    return CteStepOutcome::diag(&e, cache_verdict, obs);
                }
                exec.value
            };
            for qid in &call.query_ids {
                values_by_query.insert(qid.clone(), value);
            }
        }

        // ---- binding values + expected-type checks ----------------------
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
        let (checked, hir_proof) = match obs.span("hir_backfill", |_| {
            backfill_and_verify(&req.source, &req.module, &resolved, &ct_values)
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
            counters: obs.counters().clone(),
            spans: obs.spans().clone(),
        }
    }

    fn execute_call(
        &mut self,
        obs: &mut Obs,
        req: &CteRequestV1,
        input: &SemaInput,
        call: &auto_cte_core::schedule::AvCall,
        cache_verdict: Option<&'static str>,
    ) -> Result<AvExecResult, CteStepOutcome> {
        let forbid = self.forbid_av;
        let out = obs.span("av_execute", |_| {
            av_driver::execute_call(
                &req.source,
                &req.module,
                input,
                &call.callee,
                &call.args,
                forbid,
            )
        });
        match out {
            Ok(x) => {
                obs.counter("ct_execute", 1);
                Ok(x)
            }
            Err(e) => Err(CteStepOutcome::diag(&e, cache_verdict, obs)),
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

    fn publish_env(&self, env: &CteExecutedV1) -> Result<(), CteAdmissionError> {
        let Some(st) = self.store.as_ref() else {
            return Ok(());
        };
        store::publish_frozen(st, env).map(|_| ())
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
