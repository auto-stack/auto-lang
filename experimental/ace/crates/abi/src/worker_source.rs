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
    #[serde(default = "provider::legacy_add_contract")]
    contract_digest: String,
    provider_bias: u8,
    binding_generation: u64,
    stored: Stored,
}
#[derive(Serialize)]
struct PublicationRef<'a> {
    module: &'a str,
    #[serde(skip_serializing_if = "is_add_contract")]
    contract_digest: &'a str,
    provider_bias: u8,
    binding_generation: u64,
    stored: &'a Stored,
}
fn is_add_contract(digest: &&str) -> bool {
    *digest == provider::ADD_CONTRACT_DIGEST
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
) -> Result<StepOut, DiagOut> {
    let gc = crate::source_store::StorePolicy::worker()
        .collect(store)
        .map_err(|e| {
            err(
                "store.cleanup-pending",
                format!("source operation completed; {e}"),
            )
        })?;
    if let Some(backoff) = &gc.backoff {
        // This is maintenance after the selected operation, which may have
        // already published. Never disguise it as a rejected source candidate
        // whose current root supposedly stayed unchanged.
        return Err(err(
            "store.cleanup-pending",
            format!("source operation completed; store backoff: {backoff}"),
        ));
    }
    Ok(StepOut {
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
            gc_runs: u32::from(gc.ran),
            gc_deleted_chunks: gc.deleted_chunks.try_into().unwrap_or(u32::MAX),
            gc_freed_bytes: gc.freed_bytes,
        },
        pe_exit: exit,
    })
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
fn prepare_reference(
    run_dir: &Path,
    reference: &str,
    provider_bytes: &[u8],
    crt: &crate::source_crt::Toolchain,
) -> Result<std::path::PathBuf, DiagOut> {
    let reference_dir = run_dir.join("reference");
    std::fs::create_dir_all(&reference_dir).map_err(|e| err("infra", e))?;
    write(&reference_dir.join("source_foreign938.h"), provider::HEADER)
        .map_err(|e| err("infra", e))?;
    write(
        &reference_dir.join("reference.c"),
        format!(
            "#define SOURCE938_SCENARIO {}\n{}",
            provider::reference_case(reference)
                .ok_or_else(|| err("deck", "unreviewed reference"))?,
            include_str!("../cside/source_foreign938_reference.c"),
        ),
    )
    .map_err(|e| err("infra", e))?;
    write(&reference_dir.join("events.c"), provider::EVENT_RUNTIME).map_err(|e| err("infra", e))?;
    write(&reference_dir.join("provider.obj"), provider_bytes).map_err(|e| err("infra", e))?;
    let exe = reference_dir.join("reference.exe");
    crt.compile(
        &reference_dir,
        &["reference.c", "events.c"],
        &[reference_dir.join("provider.obj")],
        &exe,
    )
    .map_err(|e| {
        err(
            if e.0.starts_with("foreign.resource:") {
                "foreign.resource"
            } else {
                "infra"
            },
            e,
        )
    })?;
    Ok(exe)
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
        || provider::Review::for_reference(&deck.reference).is_none()
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
    // Reap dead tool/provider scopes even when this request ends with an early
    // source/declaration diagnostic. Active scopes are protected by OS locks.
    crate::source_tool_scratch::ToolScratch::sweep(&store, &resources).map_err(|e| {
        err(
            "store.cleanup-pending",
            format!("before source operation: {e}"),
        )
    })?;
    let (deck, _request_model) = match decode(req, &resources) {
        Ok(d) => d,
        Err(d) => {
            report.diagnostic_stage = "parse".into();
            let outcome = if d.code == "foreign.resource" {
                "resource"
            } else {
                "diags"
            };
            let s = step(outcome, vec![d], 0, &store, started, None)?;
            return Ok(output(s, report));
        }
    };
    let review = provider::Review::for_reference(&deck.reference)
        .ok_or_else(|| err("deck", "unreviewed reference"))?;
    report.contract_digest = review.digest().into();
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
            )?,
            report,
        ));
    }
    report.source_sha256 = ace_a2x_core::sha256::sha256_hex(deck.source.as_bytes());
    // Reject unauthorized ABI/declaration candidates before compiling/linking
    // or calling any C provider. No candidate supplies this policy digest.
    if let Err(e) = review.qualify(&deck.manifest, &deck.capabilities) {
        report.diagnostic_stage = "resolve".into();
        let code = e.split(':').next().unwrap_or("foreign.authority");
        return Ok(output(
            step("diags", vec![err(code, &e)], 0, &store, started, None)?,
            report,
        ));
    }
    let (bytes, hit) = prepare_provider(&store, &resources, deck.provider_bias)?;
    let mut provider_productions = (!hit) as u64;
    report.c_toolchain = crate::ctool::toolchain_fingerprint().map_err(|e| err("infra", e))?;
    report.provider_digest = provider::hash(&bytes);
    let current = Path::new(&req.store_root).join(format!("source938-current-{}.json", req.leg));
    let (previous_record, expected_pointer) =
        match crate::source_foreign::read_pointer_bytes_reserved(&current, &resources).map_err(
            |e| {
                let code = if e.starts_with("foreign.resource:") {
                    "foreign.resource"
                } else {
                    "store"
                };
                err(code, e)
            },
        )? {
            Some(bytes) => {
                let digest = ace_a2x_core::sha256::sha256_hex(bytes.bytes());
                let (previous, model) = resources
                    .decode_metadata_json::<Publication>(bytes.bytes())
                    .map_err(|e| match e {
                        ace_source_query::resources::ResourceError::Budget(_) => {
                            err("foreign.resource", e)
                        }
                        ace_source_query::resources::ResourceError::Decode(_) => err("restore", e),
                    })?;
                // Tuple field order releases the typed model before its
                // reservation, including every early-return/error path.
                (Some((previous, model)), Some(digest))
            }
            None => (None, None),
        };
    let previous = previous_record.as_ref().map(|(record, _ticket)| record);
    let changed_contract = previous.is_some_and(|p| p.contract_digest != review.digest());
    let binding_generation = match &previous {
        Some(p) if p.provider_bias != deck.provider_bias || changed_contract => p
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
                prepare_provider(&store, &resources, previous.provider_bias)?;
            provider_productions += (!old_hit) as u64;
            provider::hash(&old_bytes)
        } else {
            report.provider_digest.clone()
        };
        let restore_generation = previous.binding_generation;
        let previous_review = provider::Review::for_digest(&previous.contract_digest)
            .ok_or_else(|| err("restore", "unreviewed accepted contract"))?;
        let mut restore_current = || {
            Ok(CurrentAuthority {
                foreign: previous_review.authority(
                    &restore_digest,
                    restore_generation,
                    &deck.capabilities,
                ),
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
        let result = if changed_provider || changed_contract {
            let authority = review.authority(
                &report.provider_digest,
                binding_generation,
                &deck.capabilities,
            );
            let (environment, interface_work) = restored.frontend().work().observe_foreign(|| {
                if changed_contract {
                    deck.manifest.admit(&authority)
                } else {
                    resources.rebind_foreign(restored.frontend().foreign(), &authority)
                }
            });
            let environment = environment.map_err(|e| err("foreign.authority", e))?;
            restored
                .prepare_foreign_source_checked(
                    environment,
                    bytes.clone(),
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
                return source_diagnostics(
                    report,
                    ds,
                    fm.whole_parse_attempts + fm.body_parse_attempts,
                    restored.frontend().query_counters().solver_runs,
                    provider_productions,
                    &store,
                    started,
                );
            }
            Err(e) => return Err(err("source", e)),
        };
        (restored, edit)
    } else {
        let mut observation = ace_source_query::frontend::ColdObservation::default();
        let (environment, interface_work) = observation.work.observe_foreign(|| {
            deck.manifest.admit(&review.authority(
                &report.provider_digest,
                binding_generation,
                &deck.capabilities,
            ))
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
                return source_diagnostics(
                    report,
                    ds,
                    observation.whole_parse_attempts,
                    observation.queries.solver_runs,
                    provider_productions,
                    &store,
                    started,
                );
            }
            Err(e) => return Err(err("source", e)),
        };
        let session = Session::from_frontend_stored(front, checked, bytes.clone(), target, &store)
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
    let scratch = crate::source_tool_scratch::ToolScratch::create(&store, &resources)
        .map_err(|e| err("store.cleanup-pending", e))?;
    let mut publication_committed = false;
    // Keep compiler-context metadata charged until the request hands its
    // typed result to the bounded worker transport, including cleanup/GC.
    let mut crt_owner = None;
    let operation: Result<(), DiagOut> = (|| {
        let run_dir = scratch.path();
        let digest = report.provider_digest.clone();
        let runtime_policy = crate::source_foreign::certification_runtime_policy(target);
        let mut current_authority = || {
            Ok(CurrentAuthority {
                foreign: review.authority(&digest, binding_generation, &deck.capabilities),
                runtime_digest: runtime_policy.clone(),
                emitter: target.identity(),
            })
        };
        let crt = crt_owner.insert(crate::source_crt::Toolchain::resolve(&resources).map_err(
            |e| {
                err(
                    if e.0.starts_with("foreign.resource:") {
                        "foreign.resource"
                    } else {
                        "infra"
                    },
                    e,
                )
            },
        )?);
        report.c_toolchain = crt.fingerprint();
        let prepared = session
            .prepare_publication_with_crt(edit, run_dir, &mut current_authority, None, Some(crt))
            .map_err(|e| {
                err(
                    if e.starts_with("bridge: foreign.resource:") {
                        "foreign.resource"
                    } else {
                        "publication"
                    },
                    e,
                )
            })?;
        let exe = prepare_reference(run_dir, &deck.reference, bytes.as_slice(), crt)?;
        let actual = prepared
            .run(&mut current_authority, None)
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
                |stored, encoder| {
                    encoder.json(&PublicationRef {
                        module: &deck.module,
                        contract_digest: review.digest(),
                        provider_bias: deck.provider_bias,
                        binding_generation,
                        stored,
                    })
                },
                &mut current_authority,
                None,
            )
            .map_err(|e| err("store", e))?;
        publication_committed = true;
        if let Some(error) = &session.publication_maintenance().cleanup_error {
            return Err(err("store.cleanup-pending", error));
        }
        Ok(())
    })();
    if let Err(cleanup) = scratch.finish() {
        let original = operation
            .as_ref()
            .err()
            .map(|d| format!("{}: {}", d.code, d.message))
            .unwrap_or_else(|| "none".into());
        return Err(err("store.cleanup-pending", format!(
            "tool cleanup; publication committed={publication_committed}; original={original}; {cleanup}"
        )));
    }
    operation?;
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
            report.actual_exit,
        )?,
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
    resources: &Resources,
    bias: u8,
) -> Result<(Arc<Vec<u8>>, bool), DiagOut> {
    let scratch = crate::source_tool_scratch::ToolScratch::provider(store, resources, bias)
        .map_err(|e| err("store.cleanup-pending", e))?;
    let operation = (|| {
        let inputs = scratch.path();
        let src = inputs.join("provider.c");
        let object = inputs.join("provider.obj");
        write(&inputs.join("source_foreign938.h"), provider::HEADER)
            .map_err(|e| err("store", e))?;
        write(
            &src,
            format!(
                "#define SOURCE938_EVENTS\n#define BODY_BIAS {}\n{}",
                bias,
                include_str!("../cside/source_foreign938.c")
            ),
        )
        .map_err(|e| err("store", e))?;
        // This ticket lives outside the producer closure: the dependency
        // vector remains owned by the cache through hashing/generation.
        let mut dependencies_allocation = resources
            .reserve(ace_source_query::resources::Charge::index(0))
            .map_err(|e| err("foreign.resource", e))?;
        let result = crate::cache::produce_c_object_with_resources(
            store,
            &src,
            &inputs,
            &[object.clone()],
            resources,
            || {
                let deps = crate::ctool::compile_c_with_deps_admitted(
                    &src,
                    inputs,
                    &object,
                    Some(resources),
                    |bytes| {
                        dependencies_allocation
                            .resize(ace_source_query::resources::Charge::index(bytes))
                            .map_err(|e| e.to_string())
                    },
                )
                .map_err(|e| match e {
                    crate::ctool::CToolFailure::Tool(error) => {
                        ace_artifacts::store::StoreError::Internal(format!("producer: {error}"))
                    }
                    crate::ctool::CToolFailure::Resource(message) => {
                        ace_artifacts::store::StoreError::ResourceExhausted(format!(
                            "C dependency producer: {message}"
                        ))
                    }
                })?;
                let bytes = crate::cache::read_c_object_file(&object, resources)?;
                Ok((bytes, deps))
            },
        )
        .map_err(|e| {
            let code = if matches!(&e, ace_artifacts::store::StoreError::ResourceExhausted(_)) {
                "foreign.resource"
            } else {
                "infra"
            };
            err(code, e)
        })?;
        Ok(result)
    })();
    if let Err(cleanup) = scratch.finish() {
        let original = operation
            .as_ref()
            .err()
            .map(|d: &DiagOut| d.message.as_str())
            .unwrap_or("none");
        return Err(err(
            "store.cleanup-pending",
            format!(
                "provider preparation; publication committed=false; original={original}; {cleanup}"
            ),
        ));
    }
    operation
}

fn source_diagnostics(
    mut report: SourceReport,
    ds: Vec<ace_core::atom_text::Diagnostic>,
    source_parse: u64,
    solver: u64,
    provider_productions: u64,
    store: &ManagedStore,
    started: Instant,
) -> Result<OkOut, DiagOut> {
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
    Ok(output(
        step(
            "diags",
            ds.into_iter().map(|d| err(&d.code, d.message)).collect(),
            (provider_productions + solver) as u32,
            store,
            started,
            None,
        )?,
        report,
    ))
}

#[cfg(test)]
mod request_budget_tests {
    use super::*;
    use ace_source_query::resources::Limits;

    #[test]
    fn source_step_runs_original_hot_collector_and_reports_its_actual_receipt() {
        use ace_artifacts::{codec::ChunkKind, identity::QueryKey, store::PutRequest};
        let dir =
            std::path::PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests").join(format!(
                "source-worker-gc-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        let store = ManagedStore::open(&dir).unwrap();
        let layer = crate::cache::LAYER_OBJECT_V01;
        let mut kept = String::new();
        for revision in 0..5 {
            let id = store
                .put_artifact(PutRequest {
                    layer: layer.id,
                    key: QueryKey::new(&layer, vec![("revision".into(), revision.to_string())]),
                    kind: ChunkKind::Object,
                    payload: if revision == 0 {
                        vec![0; 1024]
                    } else {
                        vec![revision; 3 << 20]
                    },
                    edges: vec![],
                    project: None,
                })
                .unwrap();
            if revision == 0 {
                store
                    .register_root("source-gc-control", &id, "current")
                    .unwrap();
                kept = id;
            }
        }
        assert!(store.space().total > crate::source_store::HOT_HIGH);
        let result = step("ok", vec![], 0, &store, Instant::now(), Some(42)).unwrap();
        assert_eq!(result.outcome, "ok");
        assert_eq!(result.meter.gc_runs, 1);
        assert!(result.meter.gc_deleted_chunks > 0 && result.meter.gc_freed_bytes > 0);
        assert!(result.meter.hot_bytes <= crate::source_store::HOT_LOW);
        assert!(store.get_artifact_by_id(&kept).unwrap().is_some());
        // A collector error is a visible maintenance failure, not a source
        // rejection promising that a completed publication was rolled back.
        for revision in 5u8..8 {
            store
                .put_artifact(PutRequest {
                    layer: layer.id,
                    key: QueryKey::new(&layer, vec![("revision".into(), revision.to_string())]),
                    kind: ChunkKind::Object,
                    payload: vec![revision; 3 << 20],
                    edges: vec![],
                    project: None,
                })
                .unwrap();
        }
        assert!(store.space().total > crate::source_store::HOT_HIGH);
        store.db().execute_batch("PRAGMA query_only=ON").unwrap();
        let error = step("ok", vec![], 0, &store, Instant::now(), Some(42))
            .err()
            .unwrap();
        assert_eq!(error.code, "store.cleanup-pending");
        assert!(error.message.contains("source operation completed"));
        store.db().execute_batch("PRAGMA query_only=OFF").unwrap();
        assert!(step("ok", vec![], 0, &store, Instant::now(), Some(42)).is_ok());
        for revision in 8u8..11 {
            let id = store
                .put_artifact(PutRequest {
                    layer: layer.id,
                    key: QueryKey::new(&layer, vec![("revision".into(), revision.to_string())]),
                    kind: ChunkKind::Object,
                    payload: vec![revision; 3 << 20],
                    edges: vec![],
                    project: None,
                })
                .unwrap();
            store
                .register_root("source-live-floor", &id, "live")
                .unwrap();
        }
        let backoff = step("ok", vec![], 0, &store, Instant::now(), Some(42))
            .err()
            .unwrap();
        assert_eq!(backoff.code, "store.cleanup-pending");
        assert!(backoff.message.contains("backoff"));
        assert!(store.get_artifact_by_id(&kept).unwrap().is_some());
        assert_eq!(
            store
                .root_artifacts("source-live-floor", "live")
                .unwrap()
                .len(),
            3
        );
    }

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
