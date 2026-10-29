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
use auto_ac_prototype::{link, verify};
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
    // PLAN-871 T-06: the legacy HIR→CLIF backend is RETIRED. The switch is
    // still readable (reports/diagnostics must show the resolved value), but
    // a request that resolves to it is refused EXPLICITLY — never silently
    // served by another backend.
    if backend == ac_mn::backend::Backend::Legacy {
        return Err(WorkerError::admission(
            "request.backend",
            "the legacy HIR→CLIF backend was retired (PLAN-871 T-06); unset AC_NATIVE_BACKEND or set it to `mn`",
        ));
    }
    // Build/run always ride the shared-MIR driver (its A09'/A10' cache is
    // structural in the backend identity); the run leg stays HERE so
    // budgets/caps/raw bytes hold on every path.
    if matches!(req.kind.as_str(), "build" | "run") {
        return exec_mn(req, profile, cache_mode, &backend_tag);
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
    // Every build/run request was routed to `exec_mn` above; reaching this
    // point means an unexpected kind slipped through the dispatch.
    Err(infra(
        "request.kind",
        format!("unhandled request kind `{}`", req.kind),
    ))
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
