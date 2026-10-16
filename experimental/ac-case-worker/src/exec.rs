//! Request execution (PLAN-788 T-03): the worker-side consumer of the
//! ac-core public API. One implementation per phase, no parallel pipeline:
//!
//! - check  — real frontend (observed boundaries) + bind/verify; canonical
//!            HIR bytes come back as an artifact;
//! - build  — check + entry selection + Cranelift lowering + real staged
//!            link + publish (obj/exe manifests);
//! - run    — build + contained raw-bytes run under the request budget and
//!            stream caps.
//!
//! Cache participation (783 store) lands with T-05; this module's contract
//! with the protocol stays stable.

use crate::protocol::{
    ArtifactOut, DiagOut, InputSpec, OkOut, ReqBudget, RunOut, SpanOut, ToolchainOut, WorkerError,
    WorkerRequest, MAX_RUN_MS, MAX_SINGLE_ARTIFACT_BYTES, MAX_STDERR_BYTES, MAX_STDOUT_BYTES,
};
use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::source::adapter::{
    compile_parts_with_observer, SourceObserver, SourceStage,
};
use auto_ac_prototype::source::SourceProfile;
use auto_ac_prototype::{link, native, verify};
use base64::Engine;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub fn execute(req: &WorkerRequest) -> Result<OkOut, WorkerError> {
    req.validate()?;
    if !cfg!(windows) {
        return Err(WorkerError::admission(
            "request.platform",
            "native build/run requires Windows x86_64-pc-windows-msvc; this platform only serves protocol admission",
        ));
    }
    let started = Instant::now();
    let mut trace: Vec<SpanOut> = Vec::new();
    let profile = match req.profile.as_str() {
        "scalar" => SourceProfile::Scalar,
        _ => SourceProfile::Core,
    };

    // ---- frontend + verify (check/build/run all need a Checked module) --
    let (checked, hir_bytes, failure_diags) = frontend(req, profile, &mut trace)?;
    let checked = match checked {
        Some(c) => c,
        None => {
            // Language diagnostics ARE the observation — a well-formed
            // negative-case result, not a worker error.
            return Ok(OkOut {
                diagnostics: failure_diags,
                hir_b64: None,
                artifacts: vec![],
                run: None,
                trace,
                toolchain: toolchain_identity(),
            });
        }
    };

    // ---- canonical HIR artifact (always present once checked) ----------
    let scratch = Path::new(&req.scratch);
    let hir_path = scratch.join("hir.atom");
    std::fs::write(&hir_path, &hir_bytes)
        .map_err(|e| infra("request.scratch", format!("write hir.atom: {}", e)))?;
    let hir_artifact = artifact("hir", "hir.atom", &hir_path)?;

    if req.kind == "check" {
        trace.push(SpanOut {
            stage: "done".into(),
            wall_ms: ms(started),
        });
        return Ok(OkOut {
            diagnostics: vec![],
            hir_b64: Some(b64(&hir_bytes)),
            artifacts: vec![hir_artifact],
            run: None,
            trace,
            toolchain: toolchain_identity(),
        });
    }
    check_total(&started, req)?;

    // ---- lowering + real link (build/run) ------------------------------
    let sel_start = Instant::now();
    let sel = match native::find_entry(&checked, &format!("d_{}", req.entry)) {
        Ok(s) => s,
        Err(ds) => {
            trace.push(SpanOut {
                stage: "backend".into(),
                wall_ms: ms(sel_start),
            });
            return Ok(OkOut {
                diagnostics: diags_mapped(&ds, "backend"),
                hir_b64: Some(b64(&hir_bytes)),
                artifacts: vec![hir_artifact],
                run: None,
                trace,
                toolchain: toolchain_identity(),
            });
        }
    };
    let obj_bytes = match native::lower_object(&checked, &sel, &native::RuntimeInjection::default())
    {
        Ok(o) => o,
        Err(ds) => {
            trace.push(SpanOut {
                stage: "backend".into(),
                wall_ms: ms(sel_start),
            });
            return Ok(OkOut {
                diagnostics: diags_mapped(&ds, "backend"),
                hir_b64: Some(b64(&hir_bytes)),
                artifacts: vec![hir_artifact],
                run: None,
                trace,
                toolchain: toolchain_identity(),
            });
        }
    };
    trace.push(SpanOut {
        stage: "backend".into(),
        wall_ms: ms(sel_start),
    });
    check_total(&started, req)?;

    if obj_bytes.len() > MAX_SINGLE_ARTIFACT_BYTES {
        return Err(infra(
            "artifact.cap",
            format!(
                "object {} bytes exceeds single-artifact cap",
                obj_bytes.len()
            ),
        ));
    }

    let link_start = Instant::now();
    let toolchain = toolchain_identity();
    let lld = link::find_rust_lld().map_err(|d| infra("tool.missing", d.message))?;
    let sdk = link::find_sdk_um_dir().map_err(|d| infra("tool.missing", d.message))?;
    let obj_path = scratch.join("app.obj");
    let tmp_obj = scratch.join("app.obj.tmp-788");
    std::fs::write(&tmp_obj, &obj_bytes)
        .map_err(|e| infra("request.scratch", format!("stage obj: {}", e)))?;
    let exe_path = scratch.join("app.exe");
    let staged =
        link::link_object_staged(&lld, &sdk, &tmp_obj, &exe_path, native::START_SYMBOL, &[])
            .map_err(|d| {
                let _ = std::fs::remove_file(&tmp_obj);
                infra("link.failed", d.message)
            })?;
    let receipt = link::publish_artifacts(&staged, &tmp_obj, &obj_path)
        .map_err(|d| infra("link.failed", d.message))?;
    trace.push(SpanOut {
        stage: "link".into(),
        wall_ms: ms(link_start),
    });
    check_total(&started, req)?;

    let obj_artifact = artifact("obj", "app.obj", &obj_path)?;
    let exe_artifact = artifact("exe", "app.exe", &receipt.exe)?;

    if req.kind == "build" {
        trace.push(SpanOut {
            stage: "done".into(),
            wall_ms: ms(started),
        });
        return Ok(OkOut {
            diagnostics: vec![],
            hir_b64: Some(b64(&hir_bytes)),
            artifacts: vec![hir_artifact, obj_artifact, exe_artifact],
            run: None,
            trace,
            toolchain,
        });
    }

    // ---- contained raw-bytes run under the budget and caps --------------
    let run_start = Instant::now();
    let run_budget = Duration::from_millis(req.budget.run_ms.min(MAX_RUN_MS));
    let mut cmd = std::process::Command::new(&receipt.exe);
    let captured = link::run_deadlined_captured(
        &mut cmd,
        run_budget,
        link::CaptureCaps {
            stdout_max: MAX_STDOUT_BYTES,
            stderr_max: MAX_STDERR_BYTES,
        },
    )
    .map_err(|e| match e {
        link::SubprocError::Deadline | link::SubprocError::DeadlineCollect => infra(
            "budget.run",
            format!(
                "run exceeded its {}ms budget; the tree was terminated",
                run_budget.as_millis()
            ),
        ),
        link::SubprocError::OutputCapExceeded {
            stream,
            observed,
            cap,
        } => infra(
            "output.cap",
            format!(
                "{} exceeded its capture cap ({} > {} bytes); the tree was terminated",
                stream, observed, cap
            ),
        ),
        other => infra("run.failed", format!("{:?}", other)),
    })?;
    trace.push(SpanOut {
        stage: "run".into(),
        wall_ms: ms(run_start),
    });
    trace.push(SpanOut {
        stage: "done".into(),
        wall_ms: ms(started),
    });

    Ok(OkOut {
        diagnostics: vec![],
        hir_b64: Some(b64(&hir_bytes)),
        artifacts: vec![hir_artifact, obj_artifact, exe_artifact],
        run: Some(RunOut {
            exit_code: captured.exit_code,
            stdout_b64: b64(&captured.stdout),
            stderr_b64: b64(&captured.stderr),
        }),
        trace,
        toolchain,
    })
}

/// Frontend for both input forms. Returns (Checked, canonical HIR bytes,
/// failure diagnostics already mapped to v2 stages). `None` Checked +
/// non-empty diagnostics = the language rejected the input (a VALID
/// observation for negative cases).
fn frontend(
    req: &WorkerRequest,
    profile: SourceProfile,
    trace: &mut Vec<SpanOut>,
) -> Result<(Option<verify::CheckedModule>, Vec<u8>, Vec<DiagOut>), WorkerError> {
    match &req.input {
        InputSpec::Source { source_b64 } => {
            let src = String::from_utf8(
                b64d(source_b64).map_err(|e| infra("request.input", format!("base64: {}", e)))?,
            )
            .map_err(|_| infra("request.input", "source must be UTF-8 text"))?;
            let mut obs = TracingObserver::default();
            let parts = match compile_parts_with_observer(&src, "main", &[], profile, &mut obs) {
                Ok(p) => p,
                Err(ds) => {
                    // The observer's failed boundary is the ONLY valid
                    // parse/resolve/typecheck attribution (ac-core reports
                    // all of them as Stage::Text).
                    let stage = obs.failed_stage.clone().unwrap_or_else(|| "parse".into());
                    return Ok((None, Vec::new(), diags_mapped(&ds, &stage)));
                }
            };
            trace.extend(obs.spans);
            let t = Instant::now();
            let checked = match verify_of(profile, parts.bundle) {
                Ok(c) => c,
                Err(ds) => {
                    trace.push(SpanOut {
                        stage: "verify".into(),
                        wall_ms: ms(t),
                    });
                    return Ok((None, Vec::new(), diags_mapped(&ds, "verify")));
                }
            };
            trace.push(SpanOut {
                stage: "verify".into(),
                wall_ms: ms(t),
            });
            Ok((Some(checked), parts.atom_text.into_bytes(), Vec::new()))
        }
        InputSpec::Hir { hir_b64 } => {
            let bytes =
                b64d(hir_b64).map_err(|e| infra("request.input", format!("base64: {}", e)))?;
            let src = String::from_utf8(bytes.clone())
                .map_err(|_| infra("request.input", "hir payload must be UTF-8 atom text"))?;
            let t = Instant::now();
            let bundle = match auto_ac_prototype::bind_document(&src) {
                Ok(b) => b,
                Err(ds) => {
                    trace.push(SpanOut {
                        stage: "bind".into(),
                        wall_ms: ms(t),
                    });
                    return Ok((None, bytes, diags_mapped(&ds, "bind")));
                }
            };
            trace.push(SpanOut {
                stage: "bind".into(),
                wall_ms: ms(t),
            });
            let t = Instant::now();
            let checked = match verify_of(profile, bundle) {
                Ok(c) => c,
                Err(ds) => {
                    trace.push(SpanOut {
                        stage: "verify".into(),
                        wall_ms: ms(t),
                    });
                    return Ok((None, bytes, diags_mapped(&ds, "verify")));
                }
            };
            trace.push(SpanOut {
                stage: "verify".into(),
                wall_ms: ms(t),
            });
            // Canonical HIR for raw-HIR input is the input itself — it IS
            // the atom text the binder consumed (no re-render pipeline
            // exists, and inventing one is out of contract).
            Ok((Some(checked), bytes, Vec::new()))
        }
    }
}

fn verify_of(
    profile: SourceProfile,
    bundle: auto_ac_prototype::hir::Bundle,
) -> Result<verify::CheckedModule, Vec<Diagnostic>> {
    match profile {
        SourceProfile::Scalar => verify::verify_scalar(bundle),
        _ => verify::verify(bundle),
    }
}

/// Stage-boundary recorder: wall time per completed stage plus attribution
/// of the boundary a failure came from (the ONLY source of parse/resolve/
/// typecheck attribution — ac-core reports all three as Stage::Text).
#[derive(Default)]
struct TracingObserver {
    spans: Vec<SpanOut>,
    last: Option<Instant>,
    failed_stage: Option<String>,
}

impl SourceObserver for TracingObserver {
    fn stage(&mut self, stage: SourceStage, diags: &[Diagnostic]) {
        let now = Instant::now();
        let wall = self
            .last
            .map(|l| l.elapsed().as_millis() as u64)
            .unwrap_or(0);
        self.last = Some(now);
        if !diags.is_empty() {
            self.failed_stage = Some(stage.as_str().to_string());
        }
        self.spans.push(SpanOut {
            stage: stage.as_str().to_string(),
            wall_ms: wall,
        });
    }
}

fn diags_mapped(ds: &[Diagnostic], stage: &str) -> Vec<DiagOut> {
    ds.iter().map(|d| diag_out(d, stage.to_string())).collect()
}

fn diag_out(d: &Diagnostic, stage: String) -> DiagOut {
    DiagOut {
        stage,
        code: d.code.clone(),
        message: d.message.clone(),
        span_start: d.span.start,
        span_end: d.span.end,
    }
}

fn toolchain_identity() -> ToolchainOut {
    ToolchainOut {
        lld_path: String::new(),
        lld_sha256: String::new(),
        sdk_um: String::new(),
        ac_core_version: auto_ac_prototype::VERSION.to_string(),
        worker_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

fn artifact(kind: &str, logical: &str, path: &PathBuf) -> Result<ArtifactOut, WorkerError> {
    let bytes = std::fs::read(path)
        .map_err(|e| infra("request.scratch", format!("read {}: {}", logical, e)))?;
    Ok(ArtifactOut {
        kind: kind.to_string(),
        logical: logical.to_string(),
        path: path.display().to_string(),
        bytes: bytes.len(),
        sha256: sha256_hex(&bytes),
    })
}

fn check_total(started: &Instant, req: &WorkerRequest) -> Result<(), WorkerError> {
    if started.elapsed().as_millis() as u64 > req.budget.total_ms {
        return Err(infra(
            "budget.total",
            format!(
                "request exceeded its {}ms total budget",
                req.budget.total_ms
            ),
        ));
    }
    Ok(())
}

fn infra(code: &str, message: impl Into<String>) -> WorkerError {
    WorkerError {
        code: code.to_string(),
        message: message.into(),
    }
}

fn ms(t: Instant) -> u64 {
    t.elapsed().as_millis() as u64
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    out.iter().map(|b| format!("{:02x}", b)).collect()
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn b64d(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::engine::general_purpose::STANDARD.decode(s)
}

/// Budget ceiling helper for parents building requests.
pub fn clamp_budget(b: &ReqBudget) -> ReqBudget {
    ReqBudget {
        build_ms: b.build_ms.min(crate::protocol::MAX_BUILD_MS).max(1),
        run_ms: b.run_ms.min(MAX_RUN_MS).max(1),
        total_ms: b.total_ms.min(crate::protocol::MAX_TOTAL_MS).max(1),
    }
}
