//! Explicit source-import CERTIFICATION recipe on the existing bounded ABI
//! worker transport. Observed data is returned; only the parent owns oracles.
use crate::{
    source_foreign::{CurrentAuthority, Edit, PrepareError, Session, Stored, Target},
    source_provider as provider,
    worker::{DiagOut, MeterOut, OkOut, StageSpan, StepOut, WorkerRequest},
};
use ace_artifacts::store::ManagedStore;
use ace_foreign_contract::manual::ManualManifest;
use ace_source_query::frontend::Frontend;
use ace_source_query::resources::{Charge, Reservation, Resources};
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
    binding_generation: u64,
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
        cache: if outcome == "resource" {
            "-".into()
        } else if outcome == "diags" {
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
            rejections: if matches!(outcome, "diags" | "resource") {
                1
            } else {
                0
            },
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
fn decode(req: &WorkerRequest, resources: &Resources) -> Result<(Deck, Reservation), DiagOut> {
    if req.deck_b64.len() > crate::worker::MAX_LINE {
        return Err(err("deck", "source deck transport budget"));
    }
    let decoded_bound = req
        .deck_b64
        .len()
        .saturating_div(4)
        .saturating_mul(3)
        .saturating_add(3);
    let _input = resources
        .reserve(Charge::io(decoded_bound))
        .map_err(|e| err("foreign.resource", e))?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&req.deck_b64)
        .map_err(|e| err("deck", e))?;
    if bytes.len() > crate::worker::MAX_LINE {
        return Err(err("deck", "source deck budget"));
    }
    let (deck, model): (Deck, _) =
        resources
            .decode_foreign_request(&bytes)
            .map_err(|e| match e {
                ace_source_query::resources::ResourceError::Budget(_) => err("foreign.resource", e),
                ace_source_query::resources::ResourceError::Decode(_) => err("deck", e),
            })?;
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
    Ok((deck, model))
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
        contract_digest: provider::ADD_CONTRACT_DIGEST.into(),
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
    let resources = Resources::default();
    let (deck, _request_model) = match decode(req, &resources) {
        Ok(d) => d,
        Err(d) => {
            report.diagnostic_stage = "parse".into();
            let outcome = if d.code == "foreign.resource" {
                "resource"
            } else {
                "diags"
            };
            let s = step(outcome, vec![d], 0, &store, started, None);
            return Ok(output(s, report));
        }
    };
    if let Err(e) = target.admit_source_bytes(deck.source.len()) {
        report.diagnostic_stage = "parse".into();
        return Ok(output(
            step(
                "resource",
                vec![err("foreign.resource", e)],
                0,
                &store,
                started,
                None,
            ),
            report,
        ));
    }
    report.source_sha256 = ace_a2x_core::sha256::sha256_hex(deck.source.as_bytes());
    // Reject unauthorized ABI/declaration candidates before compiling/linking
    // or calling any C provider. No candidate supplies this policy digest.
    if let Err(e) = provider::qualify_candidate(&deck.manifest, &deck.capabilities) {
        report.diagnostic_stage = "resolve".into();
        let code = e.split(':').next().unwrap_or("foreign.authority");
        return Ok(output(
            step("diags", vec![err(code, &e)], 0, &store, started, None),
            report,
        ));
    }
    let (bytes, hit) = prepare_provider(&store, Path::new(&req.store_root), deck.provider_bias)?;
    let mut provider_productions = (!hit) as u64;
    report.c_toolchain = crate::ctool::toolchain_fingerprint().map_err(|e| err("infra", e))?;
    report.provider_digest = provider::hash(&bytes);
    let current = Path::new(&req.store_root).join(format!("source938-current-{}.json", req.leg));
    let (previous, expected_pointer) =
        match crate::source_foreign::read_pointer_bytes(&current).map_err(|e| err("store", e))? {
            Some(bytes) => {
                let digest = ace_a2x_core::sha256::sha256_hex(&bytes);
                let previous =
                    serde_json::from_slice::<Publication>(&bytes).map_err(|e| err("restore", e))?;
                (Some(previous), Some(digest))
            }
            None => (None, None),
        };
    let binding_generation = match &previous {
        Some(p) if p.provider_bias != deck.provider_bias => p
            .binding_generation
            .checked_add(1)
            .ok_or_else(|| err("foreign.binding", "generation overflow"))?,
        Some(p) => p.binding_generation,
        None => 1,
    };
    let (mut session, edit) = if let Some(previous) = &previous {
        if previous.module != deck.module {
            return Err(err(
                "restore",
                "changed module requires a separate source session",
            ));
        }
        // R admits the accepted old binding first. A different requested body
        // then uses the explicit provider transaction, never a cold fallback.
        let restore_digest = if previous.provider_bias != deck.provider_bias {
            let (old_bytes, old_hit) =
                prepare_provider(&store, Path::new(&req.store_root), previous.provider_bias)?;
            provider_productions += (!old_hit) as u64;
            provider::hash(&old_bytes)
        } else {
            report.provider_digest.clone()
        };
        let restore_generation = previous.binding_generation;
        let mut restore_current = || {
            Ok(CurrentAuthority {
                foreign: provider::authority(
                    &restore_digest,
                    restore_generation,
                    &deck.capabilities,
                )?,
                runtime_digest: crate::source_foreign::certification_runtime_policy(target),
                emitter: target.identity(),
            })
        };
        let mut restored = Session::restore_stored_current_with_resources(
            &store,
            &previous.stored,
            target,
            &deck.capabilities,
            &mut restore_current,
            resources.clone(),
        )
        .map_err(|e| err("restore", e))?;
        let changed_source = restored.frontend().source() != deck.source;
        let changed_provider = previous.provider_bias != deck.provider_bias;
        let result = if changed_provider {
            let authority = provider::authority(
                &report.provider_digest,
                binding_generation,
                &deck.capabilities,
            )
            .map_err(|e| err("foreign.authority", e))?;
            let (environment, interface_work) = restored
                .frontend()
                .work()
                .observe_foreign(|| restored.frontend().foreign().rebind(&authority));
            let environment = environment.map_err(|e| err("foreign.authority", e))?;
            restored
                .prepare_foreign_source_checked(
                    environment,
                    Arc::new(bytes.clone()),
                    changed_source.then_some(deck.source.as_str()),
                )
                .map(|candidate| Edit::Foreign(candidate.include_interface_work(interface_work)))
        } else if changed_source {
            restored
                .prepare_source_checked(&deck.source)
                .map(Edit::Body)
        } else {
            Ok(Edit::Current)
        };
        let edit = match result {
            Ok(candidate) => candidate,
            Err(PrepareError::Source(ace_source_query::frontend::Error::Diagnostics(ds))) => {
                let fm = restored.frontend().meter();
                return Ok(source_diagnostics(
                    report,
                    ds,
                    fm.whole_parse_attempts + fm.body_parse_attempts,
                    restored.frontend().query_counters().solver_runs,
                    provider_productions,
                    &store,
                    started,
                ));
            }
            Err(e) => return Err(err("source", e)),
        };
        (restored, edit)
    } else {
        let mut observation = ace_source_query::frontend::ColdObservation::default();
        let (environment, interface_work) = observation.work.observe_foreign(|| {
            provider::admit(
                deck.manifest,
                &report.provider_digest,
                binding_generation,
                &deck.capabilities,
            )
        });
        let environment = environment.map_err(|e| err("foreign.authority", e))?;
        let (front, checked) = match Frontend::cold_observed_with_resources(
            &deck.source,
            &deck.module,
            "source938",
            &deck.capabilities,
            environment,
            &mut observation,
            resources.clone(),
        ) {
            Ok(c) => c,
            Err(ace_source_query::frontend::Error::Diagnostics(ds)) => {
                return Ok(source_diagnostics(
                    report,
                    ds,
                    observation.whole_parse_attempts,
                    observation.queries.solver_runs,
                    provider_productions,
                    &store,
                    started,
                ));
            }
            Err(e) => return Err(err("source", e)),
        };
        let session =
            Session::from_frontend_stored(front, checked, Arc::new(bytes.clone()), target, &store)
                .map_err(|e| err("source", e))?;
        for work in interface_work {
            work.accept();
        }
        (session, Edit::Current)
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
    let digest = report.provider_digest.clone();
    let runtime_policy = crate::source_foreign::certification_runtime_policy(target);
    let mut current_authority = || {
        Ok(CurrentAuthority {
            foreign: provider::authority(&digest, binding_generation, &deck.capabilities)?,
            runtime_digest: runtime_policy.clone(),
            emitter: target.identity(),
        })
    };
    let prepared = session
        .prepare_publication(edit, &run_dir, &mut current_authority, None)
        .map_err(|e| err("publication", e))?;
    let actual = prepared
        .run(&mut current_authority, None)
        .map_err(|e| err("infra", e))?;
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
    session
        .publish_prepared(
            prepared,
            &store,
            &current,
            expected_pointer.as_deref(),
            previous.as_ref().map(|p| &p.stored),
            |stored| {
                serde_json::to_vec(&Publication {
                    module: deck.module.clone(),
                    provider_bias: deck.provider_bias,
                    binding_generation,
                    stored: stored.clone(),
                })
                .map_err(|e| e.to_string())
            },
            &mut current_authority,
            None,
        )
        .map_err(|e| err("store", e))?;
    if let Some(error) = &session.publication_maintenance().cleanup_error {
        return Err(err("store.cleanup-pending", error));
    }
    let compute = report.solver
        + report.hir
        + report.mir
        + report.native
        + report.c_fragment
        + provider_productions;
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

fn prepare_provider(
    store: &ManagedStore,
    root: &Path,
    bias: u8,
) -> Result<(Vec<u8>, bool), DiagOut> {
    let inputs = root.join(format!("source938-provider-{bias}"));
    std::fs::create_dir_all(&inputs).map_err(|e| err("store", e))?;
    let src = inputs.join("provider.c");
    let object = inputs.join("provider.obj");
    write(&inputs.join("source_foreign938.h"), provider::HEADER).map_err(|e| err("store", e))?;
    write(
        &src,
        format!(
            "#define SOURCE938_EVENTS\n#define BODY_BIAS {}\n{}",
            bias,
            include_str!("../cside/source_foreign938.c")
        ),
    )
    .map_err(|e| err("store", e))?;
    let result = crate::cache::produce_c_object(store, &src, &inputs, &[object.clone()], || {
        let deps =
            crate::ctool::compile_c_with_deps(&src, &inputs, &object).map_err(|e| e.to_string())?;
        Ok((std::fs::read(&object).map_err(|e| e.to_string())?, deps))
    })
    .map_err(|e| err("infra", e))?;
    Ok(result)
}

fn source_diagnostics(
    mut report: SourceReport,
    ds: Vec<ace_core::atom_text::Diagnostic>,
    source_parse: u64,
    solver: u64,
    provider_productions: u64,
    store: &ManagedStore,
    started: Instant,
) -> OkOut {
    report.source_parse = source_parse;
    report.solver = solver;
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
    output(
        step(
            "diags",
            ds.into_iter().map(|d| err(&d.code, d.message)).collect(),
            (provider_productions + solver) as u32,
            store,
            started,
            None,
        ),
        report,
    )
}

#[cfg(test)]
mod request_budget_tests {
    use super::*;
    use ace_source_query::resources::Limits;

    fn request(value: &impl Serialize) -> WorkerRequest {
        WorkerRequest {
            recipe: RECIPE.into(),
            nonce: "budget".into(),
            request_hash: "a".repeat(64),
            deck_b64: base64::engine::general_purpose::STANDARD
                .encode(serde_json::to_vec(value).unwrap()),
            leg: "native".into(),
            store_root: String::new(),
            steps: vec![],
        }
    }
    fn deck() -> Deck {
        Deck {
            schema: DECK_SCHEMA.into(),
            source: "fn main() int { return foreign_add(20,22) }".into(),
            module: "source938".into(),
            manifest: provider::reviewed_add_manifest(),
            capabilities: vec![ace_foreign_contract::manual::SOURCE_CAP.into()],
            provider_bias: 0,
            reference: "add".into(),
        }
    }
    #[test]
    fn source_request_reserves_base64_model_and_structured_manifest_before_decode() {
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let input = request(&deck());
        let (decoded, model) = decode(&input, &resources).unwrap();
        assert_eq!(decoded.manifest, provider::reviewed_add_manifest());
        assert!(resources.stats().current.index > baseline.index);
        assert!(resources.stats().current.staging >= decoded.source.capacity());
        assert_eq!(resources.stats().current.io, 0);
        drop(decoded);
        drop(model);
        assert_eq!(resources.stats().current, baseline);

        let tight = Resources::limited(Limits {
            index: 1024,
            ..Limits::default()
        })
        .unwrap();
        assert_eq!(
            decode(&input, &tight).err().unwrap().code,
            "foreign.resource"
        );
        assert_eq!(tight.stats().current.io, 0);
        let io = Resources::limited(Limits {
            io: 128,
            ..Limits::default()
        })
        .unwrap();
        assert_eq!(decode(&input, &io).err().unwrap().code, "foreign.resource");
        assert_eq!(io.stats().current.io, 0);
        let mut malformed = input;
        malformed.deck_b64 = "%%%".into();
        assert_eq!(decode(&malformed, &resources).err().unwrap().code, "deck");
        assert_eq!(resources.stats().current, baseline);
        let mut unknown = serde_json::to_value(deck()).unwrap();
        unknown
            .as_object_mut()
            .unwrap()
            .insert("safe".into(), true.into());
        assert_eq!(
            decode(&request(&unknown), &resources).err().unwrap().code,
            "deck"
        );
        assert_eq!(resources.stats().current, baseline);
    }

    #[test]
    fn source_target_limits_keep_original_native_and_neutral_frontend_caps() {
        let native = ace_native::config::MAX_INPUT_BYTES;
        let neutral = ace_source_query::frontend::MAX_SOURCE_BYTES;
        assert_eq!(native, 4 << 20);
        assert_eq!(neutral, 8 << 20);
        assert!(Target::Native.admit_source_bytes(native).is_ok());
        assert!(Target::Native
            .admit_source_bytes(native + 1)
            .unwrap_err()
            .contains("foreign.resource.source"));
        assert!(Target::C.admit_source_bytes(native + 1).is_ok());
        assert!(Target::C.admit_source_bytes(neutral).is_ok());
        assert!(Target::C.admit_source_bytes(neutral + 1).is_err());
    }
}
