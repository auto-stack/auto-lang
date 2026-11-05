//! Explicit source-import CERTIFICATION recipe on the existing bounded ABI
//! worker transport. Observed data is returned; only the parent owns oracles.
use crate::{
    source_foreign::{Session, Stored, Target},
    source_provider as provider,
    worker::{DiagOut, MeterOut, OkOut, StageSpan, StepOut, WorkerRequest},
};
use ace_artifacts::store::ManagedStore;
use ace_foreign_contract::manual::ManualManifest;
use ace_source_query::frontend::Frontend;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc, time::Instant};
pub const RECIPE: &str = "abi/source-foreign-certify-v1";
pub const DECK_SCHEMA: &str = "abi.source-import-deck/1";
pub const REPORT_SCHEMA: &str = "abi.source-import-report/1";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDiagnostic {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub span: Option<(usize, usize)>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deck {
    pub schema: String,
    pub source: String,
    pub module: String,
    pub manifest: ManualManifest,
    pub capabilities: Vec<String>,
    pub provider_bias: u8,
    pub reference: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReport {
    pub diagnostics: Vec<SourceDiagnostic>,
    pub schema: String,
    pub direction: String,
    pub input_profile: String,
    pub abi_domain: String,
    pub abi_version: String,
    pub abi_profile: String,
    pub callconv: String,
    pub target: String,
    pub consumer: String,
    pub source_sha256: String,
    pub compiler: String,
    pub c_toolchain: String,
    pub contract_digest: String,
    pub provider_digest: String,
    pub header_digest: String,
    pub physical_table_digest: String,
    pub review_id: String,
    pub actual_exit: Option<i32>,
    pub reference_exit: Option<i32>,
    pub actual_events: Vec<String>,
    pub reference_events: Vec<String>,
    pub actual_stderr: String,
    pub reference_stderr: String,
    pub source_parse: u64,
    pub solver: u64,
    pub hir: u64,
    pub mir: u64,
    pub native: u64,
    pub c_fragment: u64,
    pub diagnostic_stage: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Publication {
    module: String,
    provider_bias: u8,
    stored: Stored,
}
fn err(code: &str, message: impl ToString) -> DiagOut {
    DiagOut {
        code: code.into(),
        message: message.to_string(),
    }
}
fn step(
    outcome: &'static str,
    diagnostics: Vec<DiagOut>,
    compute: u32,
    store: &ManagedStore,
    started: Instant,
    exit: Option<i32>,
) -> StepOut {
    StepOut {
        op: "query",
        outcome,
        cache: if outcome == "diags" {
            "diags".into()
        } else if compute == 0 {
            "hit".into()
        } else {
            "published".into()
        },
        compute_calls: compute,
        diagnostics,
        stages: vec![StageSpan {
            name: "source-import".into(),
            wall_ns: started.elapsed().as_nanos(),
        }],
        meter: MeterOut {
            scratch_peak_disk_bytes: 0,
            hot_bytes: store.space().total,
            rejections: if outcome == "diags" { 1 } else { 0 },
            gc_runs: 0,
            gc_deleted_chunks: 0,
            gc_freed_bytes: 0,
        },
        pe_exit: exit,
    }
}
fn events(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter_map(|s| s.strip_prefix("source938.call=").map(String::from))
        .collect()
}
fn write(path: &Path, bytes: impl AsRef<[u8]>) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}
fn decode(req: &WorkerRequest) -> Result<Deck, DiagOut> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&req.deck_b64)
        .map_err(|e| err("deck", e))?;
    if bytes.len() > crate::worker::MAX_LINE {
        return Err(err("deck", "source deck budget"));
    }
    let deck: Deck = serde_json::from_slice(&bytes).map_err(|e| err("deck", e))?;
    if deck.schema != DECK_SCHEMA
        || deck.source.is_empty()
        || deck.module.is_empty()
        || deck.provider_bias > 1
        || deck.reference != "add"
    {
        return Err(err(
            "deck",
            "unsupported source deck schema/module/provider/reference",
        ));
    }
    Ok(deck)
}
pub fn execute(req: &WorkerRequest) -> Result<OkOut, DiagOut> {
    if req.recipe != RECIPE || !matches!(req.leg.as_str(), "native" | "c") || req.steps.len() != 1 {
        return Err(err("recipe","source certification requires its recipe, explicit native/c consumer and one operation"));
    }
    if req.steps[0].from.is_some() || req.steps[0].to.is_some() {
        return Err(err("op", "query/restart cannot carry ignored edit fields"));
    }
    if req.steps[0].op == "restart" {
        return Ok(OkOut {
            recipe: RECIPE,
            steps: vec![],
            restart_pending: true,
            worker_version: crate::worker::WORKER_VERSION.into(),
            source_import: None,
        });
    }
    if req.steps[0].op != "query" {
        return Err(err(
            "op",
            "source certification operation must be query/restart",
        ));
    }
    let started = Instant::now();
    let store = ManagedStore::open(Path::new(&req.store_root)).map_err(|e| err("store", e))?;
    let target = if req.leg == "native" {
        Target::Native
    } else {
        Target::C
    };
    let policy = provider::reviewed_add_manifest();
    let mut report = SourceReport {
        diagnostics: vec![],
        schema: REPORT_SCHEMA.into(),
        direction: "source-import".into(),
        input_profile: ace_core::source::SCALAR_SOURCE_PROFILE.into(),
        abi_domain: ace_foreign_contract::plan::DOMAIN.into(),
        abi_version: ace_foreign_contract::plan::DOMAIN_VERSION.into(),
        callconv: "windows-x64".into(),
        abi_profile: ace_foreign_contract::plan::PROFILE.into(),
        target: ace_foreign_contract::plan::TARGET.into(),
        consumer: req.leg.clone(),
        compiler: target.identity(),
        source_sha256: String::new(),
        c_toolchain: String::new(),
        contract_digest: policy.contract_digest().map_err(|e| err("policy", e))?,
        provider_digest: String::new(),
        header_digest: provider::hash(provider::HEADER),
        physical_table_digest: provider::hash(provider::TABLE),
        review_id: provider::REVIEW.into(),
        actual_exit: None,
        reference_exit: None,
        actual_events: vec![],
        reference_events: vec![],
        actual_stderr: String::new(),
        reference_stderr: String::new(),
        source_parse: 0,
        solver: 0,
        hir: 0,
        mir: 0,
        native: 0,
        c_fragment: 0,
        diagnostic_stage: String::new(),
    };
    let deck = match decode(req) {
        Ok(d) => d,
        Err(d) => {
            report.diagnostic_stage = "parse".into();
            let s = step("diags", vec![d], 0, &store, started, None);
            return Ok(output(s, report));
        }
    };
    report.source_sha256 = ace_a2x_core::sha256::sha256_hex(deck.source.as_bytes());
    // Reject unauthorized ABI/declaration candidates before compiling/linking
    // or calling any C provider. No candidate supplies this policy digest.
    if let Err(e) = provider::admit(
        deck.manifest.clone(),
        &provider::hash(b"preflight"),
        1,
        &deck.capabilities,
    ) {
        report.diagnostic_stage = "resolve".into();
        let code = e.split(':').next().unwrap_or("foreign.authority");
        return Ok(output(
            step("diags", vec![err(code, &e)], 0, &store, started, None),
            report,
        ));
    }
    let inputs =
        Path::new(&req.store_root).join(format!("source938-provider-{}", deck.provider_bias));
    std::fs::create_dir_all(&inputs).map_err(|e| err("store", e))?;
    let src = inputs.join("provider.c");
    let object = inputs.join("provider.obj");
    write(&inputs.join("source_foreign938.h"), provider::HEADER).map_err(|e| err("store", e))?;
    write(
        &src,
        format!(
            "#define SOURCE938_EVENTS\n#define BODY_BIAS {}\n{}",
            deck.provider_bias,
            include_str!("../cside/source_foreign938.c")
        ),
    )
    .map_err(|e| err("store", e))?;
    let (bytes, hit) =
        crate::cache::produce_c_object(&store, &src, &inputs, &[object.clone()], || {
            let deps = crate::ctool::compile_c_with_deps(&src, &inputs, &object)
                .map_err(|e| e.to_string())?;
            Ok((std::fs::read(&object).map_err(|e| e.to_string())?, deps))
        })
        .map_err(|e| err("infra", e))?;
    report.c_toolchain = crate::ctool::toolchain_fingerprint().map_err(|e| err("infra", e))?;
    report.provider_digest = provider::hash(&bytes);
    let environment = provider::admit(
        deck.manifest,
        &report.provider_digest,
        deck.provider_bias as u64 + 1,
        &deck.capabilities,
    )
    .map_err(|e| err("foreign.authority", e))?;
    let current = Path::new(&req.store_root).join(format!("source938-current-{}.json", req.leg));
    let previous = if current.is_file() {
        Some(
            serde_json::from_slice::<Publication>(
                &std::fs::read(&current).map_err(|e| err("store", e))?,
            )
            .map_err(|e| err("restore", e))?,
        )
    } else {
        None
    };
    let mut session = if let Some(previous) = &previous {
        if previous.module != deck.module || previous.provider_bias != deck.provider_bias {
            return Err(err(
                "restore",
                "changed module/provider requires an explicit binding transaction",
            ));
        }
        let mut restored = Session::restore_stored(
            &store,
            &previous.stored,
            target,
            &deck.capabilities,
            environment,
        )
        .map_err(|e| err("restore", e))?;
        if restored.frontend().source() != deck.source {
            let candidate = restored
                .prepare_source(&deck.source)
                .map_err(|e| err("source", e))?;
            restored.commit(candidate).map_err(|e| err("source", e))?;
        }
        restored
    } else {
        let mut observation = ace_source_query::frontend::ColdObservation::default();
        let (front, checked) = match Frontend::cold_observed(
            &deck.source,
            &deck.module,
            "source938",
            &deck.capabilities,
            environment,
            &mut observation,
        ) {
            Ok(c) => c,
            Err(ace_source_query::frontend::Error::Diagnostics(ds)) => {
                report.source_parse = observation.whole_parse_attempts;
                report.solver = observation.queries.solver_runs;
                report.diagnostic_stage = match ds.first().map(|d| d.stage) {
                    Some(ace_core::atom_text::Stage::Text) => "parse",
                    Some(ace_core::atom_text::Stage::Verify) => "verify",
                    _ => "resolve",
                }
                .into();
                report.diagnostics = ds
                    .iter()
                    .map(|d| SourceDiagnostic {
                        stage: report.diagnostic_stage.clone(),
                        code: d.code.clone(),
                        message: d.message.clone(),
                        span: Some((d.span.start, d.span.end)),
                    })
                    .collect();
                return Ok(output(
                    step(
                        "diags",
                        ds.into_iter().map(|d| err(&d.code, d.message)).collect(),
                        (!hit) as u32 + report.solver as u32,
                        &store,
                        started,
                        None,
                    ),
                    report,
                ));
            }
            Err(e) => return Err(err("source", e)),
        };
        Session::from_frontend(front, checked, Arc::new(bytes.clone()), target)
            .map_err(|e| err("source", e))?
    };
    let fm = session.frontend().meter();
    let tm = session.meter();
    report.source_parse = fm.whole_parse_attempts + fm.body_parse_attempts;
    report.solver = session.frontend().query_counters().solver_runs;
    report.hir = fm.hir_produced;
    report.mir = tm.mir_produced;
    report.native = tm.native_produced;
    report.c_fragment = tm.c_produced;
    let run_dir = Path::new(&req.store_root).join(format!(
        "source938-run-{}-{}",
        req.leg,
        std::process::id()
    ));
    let actual = session.run(&run_dir).map_err(|e| err("infra", e))?;
    let reference_dir = run_dir.join("reference");
    std::fs::create_dir_all(&reference_dir).map_err(|e| err("infra", e))?;
    write(&reference_dir.join("source_foreign938.h"), provider::HEADER)
        .map_err(|e| err("infra", e))?;
    write(
        &reference_dir.join("reference.c"),
        include_str!("../cside/source_foreign938_reference.c"),
    )
    .map_err(|e| err("infra", e))?;
    write(&reference_dir.join("events.c"), provider::EVENT_RUNTIME).map_err(|e| err("infra", e))?;
    write(&reference_dir.join("provider.obj"), &bytes).map_err(|e| err("infra", e))?;
    let exe = reference_dir.join("reference.exe");
    crate::bridge::compile_bridge_crt_with_objects(
        &reference_dir,
        &["reference.c", "events.c"],
        &[reference_dir.join("provider.obj")],
        &exe,
    )
    .map_err(|e| err("infra", e))?;
    let reference = ace_native::link_driver::run_pe(
        &exe,
        crate::bridge::BRIDGE_RUN_DEADLINE,
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 20,
            stderr_max: 1 << 20,
        },
    )
    .map_err(|e| err("infra", e))?;
    report.actual_exit = actual.exit_code;
    report.reference_exit = reference.exit_code;
    report.actual_events = events(&actual.stderr);
    report.reference_events = events(&reference.stderr);
    report.actual_stderr = String::from_utf8_lossy(&actual.stderr).into();
    report.reference_stderr = String::from_utf8_lossy(&reference.stderr).into();
    let stored = session
        .snapshot()
        .map_err(|e| err("store", e))?
        .publish(
            &store,
            "source938-certification",
            previous.as_ref().map(|p| &p.stored),
        )
        .map_err(|e| err("store", e))?;
    let publication = Publication {
        module: deck.module,
        provider_bias: deck.provider_bias,
        stored,
    };
    let temp = current.with_extension(format!("tmp-{}", std::process::id()));
    write(
        &temp,
        serde_json::to_vec(&publication).map_err(|e| err("store", e))?,
    )
    .map_err(|e| err("store", e))?;
    // Current-pointer fault/atomic replacement handling is part of T-08.
    std::fs::rename(&temp, &current).map_err(|e| err("store", e))?;
    let compute =
        report.solver + report.hir + report.mir + report.native + report.c_fragment + (!hit) as u64;
    Ok(output(
        step(
            "ok",
            vec![],
            compute as u32,
            &store,
            started,
            actual.exit_code,
        ),
        report,
    ))
}
fn output(step: StepOut, mut report: SourceReport) -> OkOut {
    if report.diagnostics.is_empty() {
        report.diagnostics = step
            .diagnostics
            .iter()
            .map(|d| SourceDiagnostic {
                stage: report.diagnostic_stage.clone(),
                code: d.code.clone(),
                message: d.message.clone(),
                span: None,
            })
            .collect();
    }
    OkOut {
        recipe: RECIPE,
        steps: vec![step],
        restart_pending: false,
        worker_version: crate::worker::WORKER_VERSION.into(),
        source_import: Some(report),
    }
}
