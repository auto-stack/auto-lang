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
    ArtifactOut, CacheOut, DiagOut, InputSpec, OkOut, ReqBudget, RunOut, SpanOut, ToolchainOut,
    WorkerError, WorkerRequest, MAX_RUN_MS, MAX_SINGLE_ARTIFACT_BYTES, MAX_STDERR_BYTES,
    MAX_STDOUT_BYTES,
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

    // PLAN-871 T-04: the driver-layer backend switch (freeze F1). Resolved
    // per request from AC_NATIVE_BACKEND (never memoized), so a long-lived
    // worker honors a profile-level rollback on the very next request. A
    // malformed value is an explicit admission refusal — never a silent
    // fallback to the other backend.
    let switch_profile = match profile {
        SourceProfile::Scalar => ac_mn::backend::Profile::Scalar,
        _ => ac_mn::backend::Profile::Core,
    };
    let backend = ac_mn::backend::resolve(switch_profile)
        .map_err(|e| WorkerError::admission("request.backend", e.0))?;
    let backend_tag = backend.tag();

    // Cache participation (T-05): auto-source build/run requests route
    // through the 783 driver (single pipeline, real hits/misses); the run
    // leg stays HERE so per-request budgets, stream caps and raw bytes hold
    // on every path. Raw-HIR inputs and `check` (whose observation IS the
    // frontend) keep the direct path — recorded as cache mode "none".
    // PLAN-788 R1 (F01): a product consumer's RUN also lives in this
    // contained, budgeted executor — never a bare Command in the parent.
    if let InputSpec::Product { exe_path } = &req.input {
        return run_product(req, exe_path);
    }

    // T-06: syntax parse/edit — the 787 runtime is single-threaded (Rc/
    // RefCell), which is exactly this executor's shape.
    if matches!(req.kind.as_str(), "parse" | "edit") {
        return crate::syntax::execute(req);
    }

    let cache_mode = req.cache.as_deref().unwrap_or("reuse");
    // R1 (F08 mirror): behavioral cache modes on non-participating
    // combinations are rejected here too (the contract gate is primary;
    // the worker never silently no-ops a requested mode).
    if matches!(cache_mode, "fresh" | "read-only")
        && (req.kind == "check" || !matches!(req.input, InputSpec::Source { .. }))
    {
        return Err(WorkerError::admission(
            "request.cache",
            "cache fresh/read-only requires auto-source ac/build|run store participation",
        ));
    }
    // PLAN-871 T-04: build/run on the shared-MIR backend never touches the
    // retiring lowering nor the 783 legacy vertical — the mn driver owns
    // its own A09'/A10' cache (backend identity is structural in its keys),
    // and this worker keeps the run leg (budgets/caps/raw bytes).
    if backend == ac_mn::backend::Backend::Mn && matches!(req.kind.as_str(), "build" | "run") {
        return exec_mn(req, profile, cache_mode, &backend_tag);
    }

    let via_driver = req.store_root.is_some()
        && matches!(req.kind.as_str(), "build" | "run")
        && matches!(req.input, InputSpec::Source { .. });
    if via_driver {
        return exec_via_driver(req, profile, cache_mode, &backend_tag);
    }

    // ---- frontend + verify (check/build/run all need a Checked module) --
    let (checked, hir_bytes, failure_diags) = frontend(req, profile, &mut trace)?;
    let checked = match checked {
        Some(c) => c,
        None => {
            // Language diagnostics ARE the observation — a well-formed
            // negative-case result, not a worker error. The total budget
            // still applies to every exit (R1 F05).
            check_total(&started, req)?;
            return Ok(OkOut {
                diagnostics: failure_diags,
                hir_b64: None,
                artifacts: vec![],
                run: None,
                trace,
                toolchain: toolchain_identity(&backend_tag),
                cache: None,
                metric_gaps: metric_gaps(),
            });
        }
    };

    // R2 (F01): the build-side capability backstop — a checked module
    // requiring undeclared capabilities is a typed capability failure.
    if let Err(ds) = verify::capabilities_check(&checked, &req.capabilities) {
        check_total(&started, req)?;
        return Ok(OkOut {
            diagnostics: diags_mapped(&ds, "capability"),
            hir_b64: None,
            artifacts: vec![],
            run: None,
            trace,
            toolchain: toolchain_identity(&backend_tag),
            cache: None,
            metric_gaps: metric_gaps(),
        });
    }

    // ---- canonical HIR artifact (always present once checked) ----------
    let scratch = Path::new(&req.scratch);
    let hir_path = scratch.join("hir.atom");
    std::fs::write(&hir_path, &hir_bytes)
        .map_err(|e| infra("request.scratch", format!("write hir.atom: {}", e)))?;
    let hir_artifact = artifact("hir", "hir.atom", &hir_path)?;

    if req.kind == "check" {
        check_total(&started, req)?;
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
            toolchain: toolchain_identity(&backend_tag),
            cache: None,
            metric_gaps: metric_gaps(),
        });
    }
    check_total(&started, req)?;

    // ---- lowering + real link (build/run) ------------------------------
    let sel_start = Instant::now();
    let sel = match select_entry(&checked, &req.entry) {
        Ok(s) => s,
        Err(ds) => {
            trace.push(SpanOut {
                stage: "backend".into(),
                wall_ms: ms(sel_start),
            });
            check_total(&started, req)?;
            return Ok(OkOut {
                diagnostics: diags_mapped(&ds, "backend"),
                hir_b64: Some(b64(&hir_bytes)),
                artifacts: vec![hir_artifact],
                run: None,
                trace,
                toolchain: toolchain_identity(&backend_tag),
                cache: None,
                metric_gaps: metric_gaps(),
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
            check_total(&started, req)?;
            return Ok(OkOut {
                diagnostics: diags_mapped(&ds, "backend"),
                hir_b64: Some(b64(&hir_bytes)),
                artifacts: vec![hir_artifact],
                run: None,
                trace,
                toolchain: toolchain_identity(&backend_tag),
                cache: None,
                metric_gaps: metric_gaps(),
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
    let lld = link::find_rust_lld().map_err(|d| infra("tool.missing", d.message))?;
    let sdk = link::find_sdk_um_dir().map_err(|d| infra("tool.missing", d.message))?;
    // R3 (F05): a linking row reports the REAL external tools it consumed
    // (linker content hash + SDK library content hash) — the pre-link
    // placeholder identity is replaced once the link actually ran.
    let lld_bytes =
        std::fs::read(&lld).map_err(|e| infra("tool.missing", format!("read lld: {}", e)))?;
    let toolchain = ToolchainOut {
        lld_path: lld.display().to_string(),
        lld_sha256: sha256_hex(&lld_bytes),
        sdk_um: sdk.display().to_string(),
        sdk_um_sha256: sdk_um_sha256_of(&sdk),
        ac_core_version: auto_ac_prototype::VERSION.to_string(),
        worker_version: env!("CARGO_PKG_VERSION").to_string(),
        worker_exe_sha256: worker_exe_sha256(),
        backend: backend_tag.to_string(),
    };
    let obj_path = scratch.join("app.obj");
    let tmp_obj = scratch.join("app.obj.tmp-788");
    std::fs::write(&tmp_obj, &obj_bytes)
        .map_err(|e| infra("request.scratch", format!("stage obj: {}", e)))?;
    let exe_path = scratch.join("app.exe");
    let support_libs: Vec<PathBuf> = req
        .support_lib
        .as_ref()
        .map(|p| vec![PathBuf::from(p)])
        .unwrap_or_default();
    let staged = link::link_object_staged(
        &lld,
        &sdk,
        &tmp_obj,
        &exe_path,
        native::START_SYMBOL,
        &support_libs,
    )
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
    // The build segment (frontend → lower → link) carries its own budget;
    // the total covers everything (R1 F05).
    check_build(&started, req)?;
    check_total(&started, req)?;

    let obj_artifact = artifact("obj", "app.obj", &obj_path)?;
    let exe_artifact = artifact("exe", "app.exe", &receipt.exe)?;

    if req.kind == "build" {
        check_total(&started, req)?;
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
            cache: None,
            metric_gaps: metric_gaps(),
        });
    }

    // ---- contained raw-bytes run under the budget and caps --------------
    let run_start = Instant::now();
    let run_budget = remaining_run_budget(req, started);
    let mut cmd = std::process::Command::new(&receipt.exe);
    let captured = link::run_deadlined_captured(
        &mut cmd,
        run_budget,
        link::CaptureCaps {
            stdout_max: MAX_STDOUT_BYTES,
            stderr_max: MAX_STDERR_BYTES,
        },
    )
    .map_err(|e| map_run_err(e, run_budget))?;
    trace.push(SpanOut {
        stage: "run".into(),
        wall_ms: ms(run_start),
    });
    trace.push(SpanOut {
        stage: "done".into(),
        wall_ms: ms(started),
    });
    check_total(&started, req)?;

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
        cache: None,
        metric_gaps: metric_gaps(),
    })
}

/// PLAN-788 R1 (F05): the run budget is additionally capped by whatever
/// REMAINS of the total — a stage may use less than its slice, never more.
fn remaining_run_budget(req: &WorkerRequest, started: Instant) -> Duration {
    let run = req.budget.run_ms.min(MAX_RUN_MS);
    let total_left = req
        .budget
        .total_ms
        .saturating_sub(started.elapsed().as_millis() as u64);
    Duration::from_millis(run.min(total_left.max(1)))
}

fn check_build(started: &Instant, req: &WorkerRequest) -> Result<(), WorkerError> {
    if started.elapsed().as_millis() as u64 > req.budget.build_ms {
        return Err(infra(
            "budget.build",
            format!(
                "build segment exceeded its {}ms budget (took {}ms)",
                req.budget.build_ms,
                started.elapsed().as_millis()
            ),
        ));
    }
    Ok(())
}

fn map_run_err(e: link::SubprocError, run_budget: Duration) -> WorkerError {
    match e {
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
    }
}

/// Contained, budgeted, capped raw-bytes run of a product consumer's exe
/// (R1 F01): the parent never spawns product executables itself.
fn run_product(req: &WorkerRequest, exe_path: &str) -> Result<OkOut, WorkerError> {
    let started = Instant::now();
    if req.kind != "run" {
        return Err(WorkerError::admission(
            "request.kind",
            "product input is an ac/run observation only",
        ));
    }
    if req.store_root.is_some() {
        return Err(WorkerError::admission(
            "request.cache",
            "product runs do not participate in the unified store",
        ));
    }
    let path = Path::new(exe_path);
    if !path.is_absolute() {
        return Err(WorkerError::admission(
            "request.input",
            "product exe path must be absolute",
        ));
    }
    let meta = std::fs::symlink_metadata(path).map_err(|e| {
        infra(
            "request.input",
            format!("product exe {:?}: {}", exe_path, e),
        )
    })?;
    if meta.file_type().is_symlink() {
        return Err(WorkerError::admission(
            "request.input",
            "product exe must not be a reparse point",
        ));
    }
    if !meta.is_file() {
        return Err(WorkerError::admission(
            "request.input",
            "product exe must be a regular file",
        ));
    }
    if meta.len() as usize > MAX_SINGLE_ARTIFACT_BYTES {
        return Err(infra(
            "artifact.cap",
            format!(
                "product exe {} bytes exceeds the single-artifact cap",
                meta.len()
            ),
        ));
    }
    let run_budget = remaining_run_budget(req, started);
    let mut cmd = std::process::Command::new(path);
    let captured = link::run_deadlined_captured(
        &mut cmd,
        run_budget,
        link::CaptureCaps {
            stdout_max: MAX_STDOUT_BYTES,
            stderr_max: MAX_STDERR_BYTES,
        },
    )
    .map_err(|e| map_run_err(e, run_budget))?;
    check_total(&started, req)?;
    Ok(OkOut {
        diagnostics: vec![],
        hir_b64: None,
        artifacts: vec![],
        run: Some(RunOut {
            exit_code: captured.exit_code,
            stdout_b64: b64(&captured.stdout),
            stderr_b64: b64(&captured.stderr),
        }),
        trace: vec![
            SpanOut {
                stage: "run".into(),
                wall_ms: ms(started),
            },
            SpanOut {
                stage: "done".into(),
                wall_ms: ms(started),
            },
        ],
        // A product run consumes an ALREADY-BUILT exe: this worker did not
        // run a backend, so the axis stays empty (never claim mn/legacy).
        toolchain: toolchain_identity(""),
        cache: None,
        metric_gaps: metric_gaps(),
    })
}

/// Metrics this worker build cannot capture — reported as gaps, never 0
/// (R1 F10).
fn metric_gaps() -> Vec<String> {
    vec!["cpu_ms".into(), "io_bytes".into(), "peak_rss".into()]
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
    let capabilities: Vec<String> = req.capabilities.clone();
    match &req.input {
        InputSpec::Source { source_b64 } => {
            let src = String::from_utf8(
                b64d(source_b64).map_err(|e| infra("request.input", format!("base64: {}", e)))?,
            )
            .map_err(|_| infra("request.input", "source must be UTF-8 text"))?;
            // The observer starts its clock AT CONSTRUCTION so the first
            // stage's span is real, never 0 (R1 F10).
            let mut obs = TracingObserver::new();
            let parts =
                match compile_parts_with_observer(&src, "main", &capabilities, profile, &mut obs) {
                    Ok(p) => p,
                    Err(ds) => {
                        // The observer's failed boundary is the ONLY valid
                        // parse/resolve/typecheck attribution (ac-core reports
                        // all of them as Stage::Text). Its spans belong in the
                        // trace even on failure (R1 F10: negative-trace).
                        let stage = obs.failed_stage.clone().unwrap_or_else(|| "parse".into());
                        trace.extend(obs.spans);
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
        InputSpec::Product { .. } => unreachable!("product inputs never reach the frontend"),
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

/// Entry selection with the raw-HIR name fallback: source documents
/// follow the adapter's `d_<name>` def-id convention, but a hand-authored
/// HIR document carries author-chosen ids — resolve by function NAME then,
/// still through the same signature-checked single-match finder.
fn select_entry(
    checked: &verify::CheckedModule,
    entry: &str,
) -> Result<native::EntrySelection, Vec<Diagnostic>> {
    let direct = format!("d_{}", entry);
    match native::find_entry(checked, &direct) {
        Ok(s) => Ok(s),
        Err(ds) => {
            if !ds.iter().any(|d| d.code == "entry.not-found") {
                return Err(ds);
            }
            let bundle = checked.bundle();
            let by_name: Vec<&str> = bundle
                .modules
                .iter()
                .flat_map(|m| m.defs.iter())
                .filter_map(|d| match &d.kind {
                    auto_ac_prototype::hir::DefKind::Function { name, .. } if name == entry => {
                        Some(d.id_text.as_str())
                    }
                    _ => None,
                })
                .collect();
            match by_name.as_slice() {
                [only] => native::find_entry(checked, only),
                _ => Err(ds),
            }
        }
    }
}

/// Driver-path execution (T-05): the 783 unified store supplies real
/// hit/miss/resume semantics for auto-source build/run; canonical-HIR
/// observation and the contained raw-bytes run stay worker-owned.
fn exec_via_driver(
    req: &WorkerRequest,
    profile: SourceProfile,
    cache_mode: &str,
    backend_tag: &str,
) -> Result<OkOut, WorkerError> {
    use ac_artifact_probe::driver::{vertical_observed, DriverCacheMode, VerticalCfg};
    let started = Instant::now();
    let mode = match cache_mode {
        "fresh" => DriverCacheMode::Fresh,
        "read-only" => DriverCacheMode::ReadOnly,
        _ => DriverCacheMode::Reuse,
    };
    let store_root = req.store_root.as_deref().unwrap();
    let store = if mode == DriverCacheMode::ReadOnly {
        ac_artifact_probe::store::ManagedStore::open_existing(std::path::Path::new(store_root))
            .map_err(|e| {
                infra(
                    "store.missing-root",
                    format!("read-only store {:?}: {}", store_root, e),
                )
            })?
    } else {
        ac_artifact_probe::store::ManagedStore::open(std::path::Path::new(store_root))
            .map_err(|e| infra("store.open", format!("{}: {}", store_root, e)))?
    };

    let scratch = std::path::Path::new(&req.scratch);
    let input_path = scratch.join("input.at");
    let input_bytes = match &req.input {
        InputSpec::Source { source_b64 } => {
            b64d(source_b64).map_err(|e| infra("request.input", format!("base64: {}", e)))?
        }
        InputSpec::Hir { .. } | InputSpec::Product { .. } => {
            unreachable!("caller gates the driver path to source inputs")
        }
    };
    std::fs::write(&input_path, &input_bytes)
        .map_err(|e| infra("request.scratch", format!("stage input: {}", e)))?;

    // REAL toolchain fingerprint: the actual linker file hash plus the
    // compiler version — never a caller-supplied constant (§5.4).
    let lld = link::find_rust_lld().map_err(|d| infra("tool.missing", d.message))?;
    let sdk = link::find_sdk_um_dir().map_err(|d| infra("tool.missing", d.message))?;
    let lld_bytes =
        std::fs::read(&lld).map_err(|e| infra("tool.missing", format!("read lld: {}", e)))?;
    let toolchain_fp = format!(
        "lld-sha256:{}+ac-core:{}+worker:{}",
        sha256_hex(&lld_bytes),
        auto_ac_prototype::VERSION,
        env!("CARGO_PKG_VERSION")
    );

    // F08: the cache key carries the ACTUAL compiler/worker recipe — this
    // executable statically embeds ac-core, so its content hash covers
    // compiler + worker code and their resolved dependency versions.
    let own_exe = std::env::current_exe()
        .map_err(|e| infra("worker.identity", format!("current_exe: {}", e)))?;
    let own_bytes = std::fs::read(&own_exe)
        .map_err(|e| infra("worker.identity", format!("read self: {}", e)))?;
    let compiler_fp = format!("worker-sha256:{}", sha256_hex(&own_bytes));
    let cfg = VerticalCfg {
        project: req.project.clone().unwrap_or_else(|| "ac-case".into()),
        source: input_path.clone(),
        snapshot_override: None,
        logical_name: "main".into(),
        profile,
        target: "x86_64-pc-windows-msvc".to_string(),
        entry: req.entry.clone(),
        toolchain_fp,
        compiler_fp: Some(compiler_fp.clone()),
        capabilities: req.capabilities.clone(),
        support_libs: req
            .support_lib
            .as_ref()
            .map(|p| vec![std::path::PathBuf::from(p)])
            .unwrap_or_default(),
        work_dir: scratch.to_path_buf(),
    };
    // Build+link through the driver under the cache mode; the run leg (for
    // kind=run) stays HERE so budgets, caps and raw bytes hold everywhere.
    let observed = match vertical_observed(&store, &cfg, mode) {
        Ok(o) => o,
        Err(
            e @ (ac_artifact_probe::driver::DriverError::Frontend(_)
            | ac_artifact_probe::driver::DriverError::Verify(_)
            | ac_artifact_probe::driver::DriverError::Backend(_)),
        ) => {
            // Language failure: replay through the TYPED pipeline — the
            // driver's rendered string is display-only; failures are never
            // cached anywhere, so the replay IS the observation. Backend
            // replays cover entry/lowering diagnostics (F09) — tool/IO/
            // linker failures never reach this arm.
            return typed_replay(
                profile,
                &input_bytes,
                req.entry.as_str(),
                &lld,
                &lld_bytes,
                &sdk,
                &e,
            );
        }
        Err(other) => return Err(infra("driver.failed", format!("{:?}", other))),
    };
    let out = observed.outcome;
    check_build(&started, req)?;
    check_total(&started, req)?;

    let mut trace: Vec<SpanOut> = observed
        .stage_ms
        .iter()
        .map(|(s, ms)| SpanOut {
            stage: s.to_string(),
            wall_ms: *ms,
        })
        .collect();

    // Counters for worker-owned legs (hir-golden, run) — merged by key
    // over the driver's counters, never duplicating an axis.
    let mut counters_extra: Vec<(String, u32)> = Vec::new();

    // Canonical HIR on request (parent declared a hir.atom golden): the
    // driver's binary bundle is NOT the canonical atom text — run the real
    // frontend for it (observation-only; nothing published).
    let hir_bytes = if req.include_hir.unwrap_or(false) {
        let t = Instant::now();
        let src_text = String::from_utf8(input_bytes.clone())
            .map_err(|_| infra("request.input", "source must be UTF-8 text"))?;
        let parts = compile_parts_with_observer(
            &src_text,
            "main",
            &[],
            profile,
            &mut TracingObserver::new(),
        )
        .map_err(|ds| {
            infra(
                "driver.inconsistent",
                format!(
                    "driver accepted a program the typed frontend rejects: {:?}",
                    ds.first().map(|d| d.code.clone())
                ),
            )
        })?;
        trace.push(SpanOut {
            stage: "hir-golden".into(),
            wall_ms: ms(t),
        });
        // F10: the golden's frontend call is REAL work and must be counted
        // as its own axis — never folded into (nor hidden from) the
        // driver's hit counters.
        counters_extra.push(("hir-golden".into(), 1));
        parts.atom_text.into_bytes()
    } else {
        Vec::new()
    };

    // Artifacts from the driver's REAL files (parent re-verifies hashes).
    let mut artifacts = Vec::new();
    if !hir_bytes.is_empty() {
        let p = scratch.join("hir.atom");
        std::fs::write(&p, &hir_bytes)
            .map_err(|e| infra("request.scratch", format!("write hir: {}", e)))?;
        artifacts.push(artifact("hir", "hir.atom", &p)?);
    }
    // The driver stages its object as work_dir/link.obj; the manifest face
    // is the fixed logical set, so the real file is copied to the logical
    // name (bytes identical — hash-verified parent-side either way).
    let obj_path = scratch.join("app.obj");
    let driver_obj = scratch.join("link.obj");
    if driver_obj.is_file() && !obj_path.is_file() {
        std::fs::copy(&driver_obj, &obj_path)
            .map_err(|e| infra("request.scratch", format!("stage obj copy: {}", e)))?;
    }
    if obj_path.is_file() {
        artifacts.push(artifact("obj", "app.obj", &obj_path)?);
    }
    let exe_path = std::path::PathBuf::from(&out.exe);
    if exe_path.is_file() {
        artifacts.push(artifact("exe", "app.exe", &exe_path)?);
    }

    // Run leg (worker-owned): exact budget, caps, raw bytes.
    let mut run = None;
    if req.kind == "run" {
        let t = Instant::now();
        let run_budget = remaining_run_budget(req, started);
        let mut cmd = std::process::Command::new(&exe_path);
        let captured = link::run_deadlined_captured(
            &mut cmd,
            run_budget,
            link::CaptureCaps {
                stdout_max: MAX_STDOUT_BYTES,
                stderr_max: MAX_STDERR_BYTES,
            },
        )
        .map_err(|e| map_run_err(e, run_budget))?;
        trace.push(SpanOut {
            stage: "run".into(),
            wall_ms: ms(t),
        });
        counters_extra.push(("run".into(), 1));
        run = Some(RunOut {
            exit_code: captured.exit_code,
            stdout_b64: b64(&captured.stdout),
            stderr_b64: b64(&captured.stderr),
        });
    }
    trace.push(SpanOut {
        stage: "done".into(),
        wall_ms: ms(started),
    });
    check_total(&started, req)?;

    let mut cache = cache_out(cache_mode, &out);
    // Worker-owned legs override the driver's counter (the driver never
    // ran it): merge by key, never duplicate.
    for (k, v) in counters_extra {
        if let Some(slot) = cache.counters.iter_mut().find(|(n, _)| *n == k) {
            slot.1 = v;
        } else {
            cache.counters.push((k, v));
        }
    }

    Ok(OkOut {
        diagnostics: vec![],
        hir_b64: (!hir_bytes.is_empty()).then(|| b64(&hir_bytes)),
        artifacts,
        run,
        trace,
        toolchain: driver_toolchain(&lld, &lld_bytes, &sdk, backend_tag),
        cache: Some(cache),
        metric_gaps: metric_gaps(),
    })
}

/// PLAN-871 T-04: build/run on the SHARED-MIR native backend (`ac-mn`).
///
/// The worker keeps everything that is its contract — frontend observation,
/// capability backstop, budgets, contained run, artifact manifests — and
/// delegates only the lowering/emission/link closure to the mn driver. A
/// store root selects the mn A09'/A10' cache (its keys carry the backend
/// identity structurally); `read-only` forbids codegen so a miss is a
/// controlled refusal instead of a silent rebuild.
fn exec_mn(
    req: &WorkerRequest,
    profile: SourceProfile,
    cache_mode: &str,
    backend_tag: &str,
) -> Result<OkOut, WorkerError> {
    let started = Instant::now();
    let mut trace: Vec<SpanOut> = Vec::new();
    let (checked, hir_bytes, failure_diags) = frontend(req, profile, &mut trace)?;
    let checked = match checked {
        Some(c) => c,
        None => {
            check_total(&started, req)?;
            return Ok(OkOut {
                diagnostics: failure_diags,
                hir_b64: None,
                artifacts: vec![],
                run: None,
                trace,
                toolchain: toolchain_identity(backend_tag),
                cache: None,
                metric_gaps: metric_gaps(),
            });
        }
    };

    // The verified capability backstop is frontend/verify-level and thus
    // backend-independent — both backends enforce the same set.
    if let Err(ds) = verify::capabilities_check(&checked, &req.capabilities) {
        check_total(&started, req)?;
        return Ok(OkOut {
            diagnostics: diags_mapped(&ds, "capability"),
            hir_b64: None,
            artifacts: vec![],
            run: None,
            trace,
            toolchain: toolchain_identity(backend_tag),
            cache: None,
            metric_gaps: metric_gaps(),
        });
    }

    let scratch = Path::new(&req.scratch);
    let hir_path = scratch.join("hir.atom");
    std::fs::write(&hir_path, &hir_bytes)
        .map_err(|e| infra("request.scratch", format!("write hir.atom: {}", e)))?;
    let hir_artifact = artifact("hir", "hir.atom", &hir_path)?;

    // Entry selection owns the same typed refusals the legacy path has
    // (`entry.not-found` / `entry.ambiguous` at the backend stage).
    let (entry_id, _entry_symbol) = match ac_mn::entry::resolve(&checked, &req.entry) {
        Ok(e) => e,
        Err(ds) => {
            check_total(&started, req)?;
            return Ok(OkOut {
                diagnostics: diags_mapped(&ds, "backend"),
                hir_b64: Some(b64(&hir_bytes)),
                artifacts: vec![hir_artifact],
                run: None,
                trace,
                toolchain: toolchain_identity(backend_tag),
                cache: None,
                metric_gaps: metric_gaps(),
            });
        }
    };

    // ---- backend: the mn driver (store-backed when the request asks) ----
    let mut driver = match req.store_root.as_deref() {
        Some(root) => {
            let project = req.project.clone().unwrap_or_else(|| "ac-case".into());
            if cache_mode == "read-only" {
                // read-only NEVER creates a store: a missing root is a typed
                // refusal (the legacy contract's `store.missing-root`).
                let store = ac_artifact_probe::store::ManagedStore::open_existing(Path::new(root))
                    .map_err(|e| {
                        infra(
                            "store.missing-root",
                            format!("read-only store {root:?}: {e}"),
                        )
                    })?;
                ac_mn::driver::MnDriver::with_managed_store(store, &project)
            } else {
                ac_mn::driver::MnDriver::with_store(Path::new(root), &project)
                    .map_err(|e| infra("store.open", format!("{}", e)))?
            }
        }
        None => ac_mn::driver::MnDriver::ephemeral(),
    };
    // T-05 cache modes are the mn store's own read/publish discipline:
    // `fresh` never reads old units, `read-only` reads hits and publishes
    // nothing (a miss recomputes for real, unreached by later reuse).
    driver.cache_mode = match cache_mode {
        "fresh" => ac_mn::driver::CacheMode::Fresh,
        "read-only" => ac_mn::driver::CacheMode::ReadOnly,
        _ => ac_mn::driver::CacheMode::Reuse,
    };
    let workdir = scratch.join("mn");
    let snapshot = ac_mn::driver::content_snapshot(&hir_bytes);
    let backend_start = Instant::now();
    let built = match driver.build_checked(
        &checked,
        &entry_id,
        &workdir,
        req.support_lib.as_ref().map(Path::new),
        None,
        snapshot,
    ) {
        Ok(b) => b,
        Err(e) => {
            return match e {
                // Infra/tool failure: never a language observation.
                ac_mn::MnError::Infra(m) => Err(infra("backend.mn", m)),
                ref err @ (ac_mn::MnError::Unsupported(_, _)
                | ac_mn::MnError::Admit(_, _)
                | ac_mn::MnError::Backend(_)
                | ac_mn::MnError::Resource(_, _)) => {
                    check_total(&started, req)?;
                    Ok(OkOut {
                        diagnostics: vec![DiagOut {
                            stage: "backend".into(),
                            code: err.code(),
                            message: err.message(),
                            span_start: 0,
                            span_end: 0,
                        }],
                        hir_b64: Some(b64(&hir_bytes)),
                        artifacts: vec![hir_artifact],
                        run: None,
                        trace,
                        toolchain: toolchain_identity(backend_tag),
                        cache: None,
                        metric_gaps: metric_gaps(),
                    })
                }
            };
        }
    };
    trace.push(SpanOut {
        stage: "lower".into(),
        wall_ms: backend_start.elapsed().as_millis() as u64,
    });
    trace.push(SpanOut {
        stage: "link".into(),
        wall_ms: built.obs.spans.get("link").copied().unwrap_or(0) as u64,
    });
    check_build(&started, req)?;

    // ---- publish the exe at its LOGICAL manifest name -------------------
    // The parent admits artifacts by logical identity and requires the file
    // name to match it (`app.exe`), so the mn link product is moved to that
    // name inside the request scratch before the manifest is built.
    let exe_path = scratch.join("app.exe");
    if built.exe_path != exe_path {
        if let Err(e) = std::fs::rename(&built.exe_path, &exe_path) {
            // Cross-volume fallback (scratch and temp may live apart).
            std::fs::copy(&built.exe_path, &exe_path)
                .map_err(|e2| infra("request.scratch", format!("publish exe: {e} / {e2}")))?;
            let _ = std::fs::remove_file(&built.exe_path);
        }
    }
    let exe_artifact = artifact("exe", "app.exe", &exe_path)?;

    // ---- run: the worker's contained, budgeted executor ------------------
    let mut run = None;
    if req.kind == "run" {
        let run_start = Instant::now();
        let run_budget = remaining_run_budget(req, started);
        let mut cmd = std::process::Command::new(&exe_path);
        let captured = link::run_deadlined_captured(
            &mut cmd,
            run_budget,
            link::CaptureCaps {
                stdout_max: MAX_STDOUT_BYTES,
                stderr_max: MAX_STDERR_BYTES,
            },
        )
        .map_err(|e| map_run_err(e, run_budget))?;
        trace.push(SpanOut {
            stage: "run".into(),
            wall_ms: ms(run_start),
        });
        run = Some(RunOut {
            exit_code: captured.exit_code,
            stdout_b64: b64(&captured.stdout),
            stderr_b64: b64(&captured.stderr),
        });
    }
    trace.push(SpanOut {
        stage: "total".into(),
        wall_ms: ms(started),
    });
    trace.push(SpanOut {
        stage: "done".into(),
        wall_ms: ms(started),
    });
    check_total(&started, req)?;

    // ---- toolchain identity: the REAL linker/SDK the mn link consumed ---
    let lld = link::find_rust_lld().map_err(|d| infra("tool.missing", d.message))?;
    let sdk = link::find_sdk_um_dir().map_err(|d| infra("tool.missing", d.message))?;
    let lld_bytes =
        std::fs::read(&lld).map_err(|e| infra("tool.missing", format!("read lld: {}", e)))?;

    let cache = req.store_root.as_ref().map(|_| {
        let is_hir = matches!(req.input, InputSpec::Hir { .. });
        mn_cache_out(&req.kind, cache_mode, is_hir, &built)
    });

    Ok(OkOut {
        diagnostics: vec![],
        hir_b64: Some(b64(&hir_bytes)),
        artifacts: vec![hir_artifact, exe_artifact],
        run,
        trace,
        toolchain: driver_toolchain(&lld, &lld_bytes, &sdk, backend_tag),
        cache,
        metric_gaps: metric_gaps(),
    })
}

/// The cache record of one mn request (T-05). It reports the mn store's OWN
/// observation (unit COFF hits vs codegen, real publish count) under the
/// report vocabulary the ac contract already binds (`parse`/`verify`/
/// `lower`/`link`/`run`/`store_puts`, `hits.{summary,hir,object}`).
///
/// One deliberate semantic difference from the retiring 783 driver: the mn
/// backend caches EMITTED UNITS, not the frontend, and this worker owns the
/// frontend — so `parse`/`verify` are observed on EVERY request (a full
/// object hit still shows `parse=1, lower=0`). The invariant the cache
/// record must prove is "no recompilation on a hit", which `lower=0`
/// carries exactly.
fn mn_cache_out(
    kind: &str,
    cache_mode: &str,
    is_hir: bool,
    built: &ac_mn::driver::MnBuild,
) -> CacheOut {
    use std::collections::BTreeMap;
    let mut c: BTreeMap<String, u32> = built
        .obs
        .counters
        .iter()
        .map(|(k, v)| (k.clone(), u32::try_from(*v).unwrap_or(u32::MAX)))
        .collect();
    let lowered = *c.get("codegen_emit_units").unwrap_or(&0);
    let unit_hits = *c.get("cache_hit_coff").unwrap_or(&0);
    let link = *c.get("link_invocations").unwrap_or(&0);
    let store_puts = *c.get("store_puts").unwrap_or(&0);
    let object = if lowered == 0 && unit_hits > 0 {
        "hit"
    } else {
        "lowered"
    };
    c.insert("parse".into(), u32::from(!is_hir));
    c.insert("resolve_typecheck".into(), u32::from(!is_hir));
    c.insert("bind".into(), u32::from(is_hir));
    c.insert("verify".into(), 1);
    c.insert("lower".into(), lowered);
    c.insert("link".into(), link);
    c.insert("run".into(), u32::from(kind == "run"));
    c.insert("store_puts".into(), store_puts);
    CacheOut {
        mode: cache_mode.to_string(),
        hits: vec![
            ("summary".into(), object.to_string()),
            ("hir".into(), object.to_string()),
            ("object".into(), object.to_string()),
        ],
        counters: c.into_iter().collect(),
    }
}

/// Typed replay of a driver-reported language failure: the real frontend,
/// verifier and (for Backend reports) entry-selection/lowering produce the
/// stage/code/span observation the parent's error oracle matches against —
/// never the driver's rendered string, never a string-shaped classifier.
fn typed_replay(
    profile: SourceProfile,
    input_bytes: &[u8],
    entry: &str,
    lld: &std::path::Path,
    lld_bytes: &[u8],
    sdk: &std::path::Path,
    driver_error: &ac_artifact_probe::driver::DriverError,
) -> Result<OkOut, WorkerError> {
    let replay_backend = matches!(
        driver_error,
        ac_artifact_probe::driver::DriverError::Backend(_)
    );
    let src_text = String::from_utf8(input_bytes.to_vec())
        .map_err(|_| infra("request.input", "source must be UTF-8 text"))?;
    let mut trace: Vec<SpanOut> = Vec::new();
    let mut obs = TracingObserver::new();
    let parts = match compile_parts_with_observer(&src_text, "main", &[], profile, &mut obs) {
        Ok(p) => {
            trace.extend(obs.spans);
            p
        }
        Err(ds) => {
            // F10: observer spans travel with the failure.
            let stage = obs.failed_stage.clone().unwrap_or_else(|| "parse".into());
            trace.extend(obs.spans);
            return Ok(OkOut {
                diagnostics: diags_mapped(&ds, &stage),
                hir_b64: None,
                artifacts: vec![],
                run: None,
                trace,
                toolchain: driver_toolchain(
                    lld,
                    lld_bytes,
                    sdk,
                    &ac_mn::backend::Backend::Legacy.tag(),
                ),
                cache: None,
                metric_gaps: metric_gaps(),
            });
        }
    };
    let t = Instant::now();
    let checked = match verify_of(profile, parts.bundle) {
        Ok(c) => {
            trace.push(SpanOut {
                stage: "verify".into(),
                wall_ms: ms(t),
            });
            c
        }
        Err(ds) => {
            trace.push(SpanOut {
                stage: "verify".into(),
                wall_ms: ms(t),
            });
            return Ok(OkOut {
                diagnostics: diags_mapped(&ds, "verify"),
                hir_b64: None,
                artifacts: vec![],
                run: None,
                trace,
                toolchain: driver_toolchain(
                    lld,
                    lld_bytes,
                    sdk,
                    &ac_mn::backend::Backend::Legacy.tag(),
                ),
                cache: None,
                metric_gaps: metric_gaps(),
            });
        }
    };
    if replay_backend {
        // The driver's Backend failure maps to the real entry/lowering
        // boundary: replay entry selection (and lowering when the entry
        // resolves) for the typed diagnostic; a clean replay contradicts
        // the driver report and is an infrastructure inconsistency.
        let t = Instant::now();
        let sel = match select_entry(&checked, entry) {
            Err(ds) => {
                trace.push(SpanOut {
                    stage: "backend".into(),
                    wall_ms: ms(t),
                });
                return Ok(OkOut {
                    diagnostics: diags_mapped(&ds, "backend"),
                    hir_b64: None,
                    artifacts: vec![],
                    run: None,
                    trace,
                    toolchain: driver_toolchain(
                        lld,
                        lld_bytes,
                        sdk,
                        &ac_mn::backend::Backend::Legacy.tag(),
                    ),
                    cache: None,
                    metric_gaps: metric_gaps(),
                });
            }
            Ok(sel) => sel,
        };
        match native::lower_object(&checked, &sel, &native::RuntimeInjection::default()) {
            Err(ds) => {
                trace.push(SpanOut {
                    stage: "backend".into(),
                    wall_ms: ms(t),
                });
                return Ok(OkOut {
                    diagnostics: diags_mapped(&ds, "backend"),
                    hir_b64: None,
                    artifacts: vec![],
                    run: None,
                    trace,
                    toolchain: driver_toolchain(
                        lld,
                        lld_bytes,
                        sdk,
                        &ac_mn::backend::Backend::Legacy.tag(),
                    ),
                    cache: None,
                    metric_gaps: metric_gaps(),
                });
            }
            Ok(_) => {}
        }
    }
    Err(infra(
        "driver.inconsistent",
        format!(
            "driver rejected a program the typed pipeline accepts: {}",
            driver_error
        ),
    ))
}

fn cache_out(mode: &str, out: &ac_artifact_probe::driver::VerticalOutcome) -> CacheOut {
    let c = &out.counters;
    CacheOut {
        mode: mode.to_string(),
        hits: vec![
            ("summary".into(), out.hits.summary.to_string()),
            ("hir".into(), out.hits.hir.to_string()),
            ("object".into(), out.hits.object.to_string()),
        ],
        counters: vec![
            ("parse".into(), c.parse),
            ("resolve_typecheck".into(), c.resolve_typecheck),
            ("bind".into(), c.bind),
            ("verify".into(), c.verify),
            ("hir_encode".into(), c.hir_encode),
            ("hir_restore".into(), c.hir_restore),
            ("lower".into(), c.lower),
            ("link".into(), c.link),
            ("run".into(), c.run),
            ("store_puts".into(), c.store_puts),
        ],
    }
}

fn driver_toolchain(
    lld: &std::path::Path,
    lld_bytes: &[u8],
    sdk: &std::path::Path,
    backend: &str,
) -> ToolchainOut {
    ToolchainOut {
        lld_path: lld.display().to_string(),
        lld_sha256: sha256_hex(lld_bytes),
        sdk_um: sdk.display().to_string(),
        sdk_um_sha256: sdk_um_sha256_of(sdk),
        ac_core_version: auto_ac_prototype::VERSION.to_string(),
        worker_version: env!("CARGO_PKG_VERSION").to_string(),
        worker_exe_sha256: worker_exe_sha256(),
        backend: backend.to_string(),
    }
}

/// R3 (F05): the SDK axis is CONTENT-based — sha256 of the kernel32.lib
/// the link consumes (the display path stays out of identity). Missing
/// file → empty (a linking row then fails the parent's axis admission).
fn sdk_um_sha256_of(sdk: &std::path::Path) -> String {
    std::fs::read(sdk.join("kernel32.lib"))
        .map(|b| sha256_hex(&b))
        .unwrap_or_default()
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
struct TracingObserver {
    spans: Vec<SpanOut>,
    last: Instant,
    failed_stage: Option<String>,
}

impl TracingObserver {
    fn new() -> Self {
        TracingObserver {
            spans: Vec::new(),
            last: Instant::now(),
            failed_stage: None,
        }
    }
}

impl SourceObserver for TracingObserver {
    fn stage(&mut self, stage: SourceStage, diags: &[Diagnostic]) {
        let now = Instant::now();
        let wall = self.last.elapsed().as_millis() as u64;
        self.last = now;
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

fn toolchain_identity(backend: &str) -> ToolchainOut {
    // Display fields are filled opportunistically (discovery is only run
    // on paths that link); the IDENTITY axis (worker exe hash) is always
    // present so every native row carries the same verifiable set.
    ToolchainOut {
        lld_path: String::new(),
        lld_sha256: String::new(),
        sdk_um: String::new(),
        sdk_um_sha256: String::new(),
        ac_core_version: auto_ac_prototype::VERSION.to_string(),
        worker_version: env!("CARGO_PKG_VERSION").to_string(),
        worker_exe_sha256: worker_exe_sha256(),
        backend: backend.to_string(),
    }
}

/// Content hash of THIS executable (cached): the compiler/worker recipe
/// identity (R1 F07/F08 — also the cache compiler_fp).
fn worker_exe_sha256() -> String {
    static CACHE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| {
            std::env::current_exe()
                .and_then(|p| std::fs::read(p))
                .map(|b| sha256_hex(&b))
                .unwrap_or_default()
        })
        .clone()
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

/// Public re-exports for the syntax adapter (same crate).
pub fn sha256_hex_public(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

pub fn worker_exe_sha256_public() -> String {
    worker_exe_sha256()
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
