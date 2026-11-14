//! PLAN-967 T-03: the F0 Rust-provider certification recipe on the ABI
//! worker transport — the production TestCase lane's consumer. The deck
//! carries the ordinary `use.rs` Auto source AND the annotated provider
//! source; the manifest is DERIVED at execution (the F0 contract: no
//! hand-written manifest anywhere). Legs: `av` (in-process AV execution
//! through a real loaded session) and `native` (caller COFF + import-lib
//! link + real machine-code run).
use crate::worker::{DiagOut, MeterOut, OkOut, StageSpan, StepOut, WorkerRequest, WORKER_VERSION};
use base64::Engine as _;
use serde::Deserialize;

pub const F0_RECIPE: &str = "abi/rust-f0-certify-v1";
pub const F0_DECK_SCHEMA: &str = "abi.rust-f0-deck/1";

/// The F0 deck grammar (strict): the Auto source, the annotated Rust
/// provider source it imports, and the module id.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct F0Deck {
    pub schema: String,
    pub module: String,
    pub source: String,
    pub provider_source: String,
}

fn diag(code: &str, message: impl Into<String>) -> DiagOut {
    DiagOut {
        code: code.into(),
        message: message.into(),
    }
}

/// Serve one F0 certification request: build the provider unit from the
/// deck's annotated source, admit, and run the deck's `use.rs` source on
/// the requested leg. Observed data is returned; the parent owns the
/// oracle.
pub fn execute(req: &WorkerRequest) -> Result<OkOut, DiagOut> {
    if req.recipe != F0_RECIPE {
        return Err(diag("recipe", format!("worker implements {F0_RECIPE}")));
    }
    let deck_bytes = base64::engine::general_purpose::STANDARD
        .decode(&req.deck_b64)
        .map_err(|e| diag("deck-b64", format!("{e}")))?;
    let deck: F0Deck = serde_json::from_slice(&deck_bytes)
        .map_err(|e| diag("deck", format!("F0 deck decode: {e}")))?;
    if deck.schema != F0_DECK_SCHEMA {
        return Err(diag(
            "deck.schema",
            format!("expected {F0_DECK_SCHEMA}, got {}", deck.schema),
        ));
    }
    if !req.steps.iter().all(|s| s.op == "query") || req.steps.is_empty() {
        return Err(diag("op", "the F0 recipe accepts query steps only"));
    }

    let scratch = std::path::Path::new(&req.store_root).join("f0-scratch");
    std::fs::create_dir_all(&scratch).map_err(|e| diag("infra", format!("scratch: {e}")))?;
    let started = std::time::Instant::now();
    let build = crate::rust_provider::build_source(&scratch, &deck.provider_source)
        .map_err(|e| diag("f0.build", e))?;
    let scratch_bytes = std::fs::metadata(&build.cdylib)
        .map(|m| m.len())
        .unwrap_or(0);

    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let exit: i32 = match req.leg.as_str() {
        "av" => {
            let authority = crate::rust_provider::f0_authority(&build, 1);
            let admitted = build
                .manual
                .clone()
                .admit(&authority)
                .map_err(|e| diag("f0.admit", e))?;
            let mut provider =
                crate::rust_provider::load(&build, 1).map_err(|e| diag("f0.load", e))?;
            let mut session = ace_av_core::foreign_exec::ForeignSession::new();
            session.adopt(std::mem::replace(
                &mut provider.table,
                ace_av_core::foreign_exec::ForeignStubTable::new(1),
            ));
            let mut query = ace_source_query::SourceQuerySession::new("p967f0", "m967f0");
            query.set_foreign(Some(admitted.clone()));
            let product = ace_source_query::compile_text_record_product(
                &deck.source,
                &deck.module,
                &caps,
                "p967f0",
                "m967f0",
                &mut query,
            )
            .map_err(|ds| {
                diag(
                    "f0.source",
                    format!(
                        "source diagnostics: {:?}",
                        ds.first().map(|d| (d.stage.as_str(), d.code.clone()))
                    ),
                )
            })?;
            let mir = ace_lowering::hir_adapter::lower_checked_owned(
                &product.compiled.checked,
                "d_main",
                ace_av_core::contract::fnv1a64(deck.source.as_bytes()),
            )
            .map_err(|e| diag("f0.lower", format!("{e:?}")))?;
            ace_av_core::mir_verify::verify_mir(&mir)
                .map_err(|e| diag("f0.mir", format!("{e:?}")))?;
            let encoded = ace_av_core::abc::encode_abc(&mir)
                .map_err(|e| diag("f0.encode", format!("{e:?}")))?
                .to_flat();
            let decoded = ace_av_core::abc::decode_abc(&encoded)
                .map_err(|e| diag("f0.decode", format!("{e:?}")))?;
            let loaded = ace_av_core::abc_verify::verify_abc_foreign(
                decoded,
                &mir.owner,
                session.table_shared(),
            )
            .map_err(|e| diag("f0.verify", e.to_string()))?;
            let (outcome, _trace) = session
                .execute(
                    &loaded,
                    &ace_av_core::av::ExecLimits::default(),
                    &ace_av_core::av::CancelFlag::new(false),
                )
                .map_err(|e| diag("f0.authorize", format!("{e}")))?;
            match outcome {
                ace_av_core::contract::ExecOutcome::Success {
                    value: ace_av_core::contract::Value::I32(n),
                    ..
                } => n,
                other => {
                    return Err(diag(
                        "f0.outcome",
                        format!("expected an i32 success, got {other:?}"),
                    ))
                }
            }
        }
        "native" => {
            let authority = crate::rust_provider::f0_authority(&build, 1);
            let admitted = build
                .manual
                .clone()
                .admit(&authority)
                .map_err(|e| diag("f0.admit", e))?;
            let mut query = ace_source_query::SourceQuerySession::new("p967f0", "m967f0");
            let (_product, units) = ace_native::foreign_lower::compile_source_units(
                &deck.source,
                &deck.module,
                &caps,
                &admitted,
                &mut query,
            )
            .map_err(|e| diag("f0.source", e))?;
            let unit = units
                .first()
                .ok_or_else(|| diag("f0.units", "no unit produced"))?;
            let object = ace_native::emit::emit_foreign_unit_coff(unit, &admitted)
                .map_err(|e| diag("f0.emit", e.message().to_string()))?;
            let wrapper = ace_native::emit::emit_wrapper_coff(&object.symbol)
                .map_err(|e| diag("f0.emit", e.message().to_string()))?;
            let run_dir = scratch.join("native-run");
            std::fs::create_dir_all(&run_dir)
                .map_err(|e| diag("infra", format!("run dir: {e}")))?;
            std::fs::copy(&build.cdylib, run_dir.join("f0_text_provider.dll"))
                .map_err(|e| diag("infra", format!("stage dll: {e}")))?;
            let import_lib = build.cdylib.with_file_name("f0_text_provider.dll.lib");
            let linked = ace_native::link_driver::link_only(
                &[ace_native::link_driver::ObjectInput {
                    name: "caller".into(),
                    bytes: object.bytes.clone(),
                }],
                &wrapper,
                "ac_start",
                &run_dir,
                &[import_lib],
                "plan967 f0 worker",
            )
            .map_err(|e| diag("f0.link", e.message().to_string()))?;
            let run = ace_native::link_driver::run_pe(
                &linked.exe_path,
                std::time::Duration::from_secs(20),
                ace_core::link::CaptureCaps {
                    stdout_max: 1 << 16,
                    stderr_max: 1 << 16,
                },
            )
            .map_err(|e| diag("f0.run", e.message().to_string()))?;
            run.exit_code.unwrap_or(-1)
        }
        other => {
            return Err(diag(
                "leg",
                format!("the F0 recipe accepts legs av|native, got {other:?}"),
            ))
        }
    };

    let steps = req
        .steps
        .iter()
        .map(|s| StepOut {
            op: if s.op == "query" { "query" } else { "query" },
            outcome: "ok",
            cache: "miss".into(),
            compute_calls: 1,
            diagnostics: vec![],
            stages: vec![StageSpan {
                name: format!("f0-{}", req.leg),
                wall_ns: started.elapsed().as_nanos(),
            }],
            meter: MeterOut {
                scratch_peak_disk_bytes: scratch_bytes,
                hot_bytes: 0,
                rejections: 0,
                gc_runs: 0,
                gc_deleted_chunks: 0,
                gc_freed_bytes: 0,
            },
            pe_exit: Some(exit),
        })
        .collect();
    Ok(OkOut {
        recipe: F0_RECIPE,
        steps,
        restart_pending: false,
        worker_version: WORKER_VERSION.to_string(),
        source_import: None,
    })
}
