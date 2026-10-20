//! PLAN-798 bounded worker: cross-process restore + re-verify + execute.
//!
//! Subcommands:
//! - `restore-run`: open the managed store, load the module manifest and its
//!   ABC units BY ARTIFACT ID (the store's address admission guards every
//!   read), re-validate every envelope, re-decode and re-verify each unit,
//!   re-bind diagnostics sidecars, assemble + verify the closure, execute
//!   the entry. Never receives the source; prints one JSON result line.
//! - `request`: ONE driver request in THIS fresh process (restart-scenario
//!   entrypoint); prints the outcome JSON and exits 0 only on success.
//! - `gc-loop`: run store GC repeatedly until a stop file appears
//!   (concurrent-GC scenario driver).
//! - `bench-turnaround`: the six-scenario turnaround tier (plan §5.5 r2
//!   semantics). Enforces the hard gate: `--gate-ms` is the enforced P95
//!   bound; `--gate-ms 0` MUST fail (review F798-R1-03 countercontrol).
//!   Writes raw JSONL samples, prints a REAL summary object, exits non-zero
//!   on any gate/validation failure.

use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    match args[0].as_str() {
        "restore-run" => restore_run(&args[1..]),
        "request" => request(&args[1..]),
        "gc-loop" => gc_loop(&args[1..]),
        "case-serve" => case_serve(),
        "load-revision" => load_revision(&args[1..]),
        "bench-turnaround" => bench_turnaround(&args[1..]),
        "native-build" => native_build(&args[1..]),
        "native-run" => native_run(&args[1..]),
        other => {
            eprintln!("unknown subcommand `{}`", other);
            usage();
        }
    }
}

fn usage() -> ! {
    eprintln!("usage: av_probe_worker restore-run --store DIR --owner NAME --entry FN --manifest-artifact ID [--snapshot HEX] [--caps C1,C2]");
    eprintln!("       av_probe_worker request --store DIR --src-file FILE --entry FN [--project P] [--caps C1,C2]");
    eprintln!("       av_probe_worker gc-loop --store DIR --stop-file FILE [--interval-ms N]");
    eprintln!("       av_probe_worker bench-turnaround --samples N --gate-ms MS --out FILE");
    eprintln!(
        "       av_probe_worker case-serve  (PLAN-798 T-07: AutoDown probe envelope on stdin)"
    );
    eprintln!("       av_probe_worker native-build --src-file F --entry FN --workdir DIR [--trace-lib P] [--caps C1,C2]");
    eprintln!(
        "       av_probe_worker native-run --exe P --budget-ms N --stdout-cap N --stderr-cap N"
    );
    exit(2)
}

fn flag(args: &[String], name: &str) -> Option<String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == name {
            return it.next().cloned();
        }
    }
    None
}

fn restore_run(args: &[String]) -> ! {
    let store_dir = flag(args, "--store").unwrap_or_else(|| usage());
    let owner = flag(args, "--owner").unwrap_or_else(|| usage());
    let entry = flag(args, "--entry").unwrap_or_else(|| usage());
    let manifest_artifact = flag(args, "--manifest-artifact").unwrap_or_else(|| usage());
    let snapshot = flag(args, "--snapshot")
        .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).expect("snapshot hex"))
        .unwrap_or(0);
    // The CURRENT request's capability authorization (F798-R3-02): a
    // trace-requiring module restores only under a live grant.
    let caps: Vec<String> = flag(args, "--caps")
        .map(|c| {
            c.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let outcome = auto_av_probe::worker_restore::restore_and_run(
        &PathBuf::from(store_dir),
        &owner,
        &entry,
        &manifest_artifact,
        snapshot,
        &caps,
    );
    let mut line = serde_json::to_string(&outcome).unwrap_or_else(|e| {
        eprintln!("serialize: {}", e);
        exit(3)
    });
    line.push('\n');
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(if outcome.executed_ok { 0 } else { 4 })
}

/// One request in a fresh process. The wall time covers the full
/// request → outcome span inside this process (store open included).
fn request(args: &[String]) -> ! {
    let store_dir = flag(args, "--store").unwrap_or_else(|| usage());
    let src_file = flag(args, "--src-file").unwrap_or_else(|| usage());
    let entry = flag(args, "--entry").unwrap_or_else(|| usage());
    let project = flag(args, "--project").unwrap_or_else(|| "restart".into());
    let caps: Vec<String> = flag(args, "--caps")
        .map(|c| {
            c.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let src = std::fs::read_to_string(&src_file).unwrap_or_else(|e| {
        eprintln!("read {}: {}", src_file, e);
        exit(2)
    });
    let started = std::time::Instant::now();
    let sampler = auto_av_probe::mempeek::PeakSampler::start(5);
    let mut d = auto_av_probe::driver::Driver::with_store(&PathBuf::from(store_dir), &project)
        .unwrap_or_else(|e| {
            eprintln!("store: {}", e);
            exit(3)
        });
    let r = d.request(&src, &entry, &caps);
    let wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let private_bytes_peak = sampler.stop();
    let payload = serde_json::json!({
        "wall_ms": wall_ms,
        "private_bytes_peak": private_bytes_peak,
        "ok": r.ok(),
        "value": r.value_i32(),
        "diag": r.diag.as_ref().map(|d| format!("{}:{}", d.stage, d.code)),
        "counters": r.counters.iter().map(|(k, v)| (k.clone(), *v)).collect::<std::collections::BTreeMap<String, u64>>(),
        "source_hash": r.source_hash,
    });
    let mut line = serde_json::to_string(&payload).unwrap_or_default();
    line.push('\n');
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(if r.ok() { 0 } else { 4 })
}

/// Repeated GC until the stop file appears (real cross-process concurrency
/// against requests in other processes).
fn gc_loop(args: &[String]) -> ! {
    let store_dir = flag(args, "--store").unwrap_or_else(|| usage());
    let stop_file = flag(args, "--stop-file").unwrap_or_else(|| usage());
    let interval_ms: u64 = flag(args, "--interval-ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(25);
    let store = ac_artifact_probe::store::ManagedStore::open(&PathBuf::from(store_dir))
        .unwrap_or_else(|e| {
            eprintln!("store: {}", e);
            exit(3)
        });
    let sampler = auto_av_probe::mempeek::PeakSampler::start(10);
    let cfg = ac_artifact_probe::store::gc::gc_config_default();
    let mut passes = 0u64;
    while !std::path::Path::new(&stop_file).exists() {
        let _ = ac_artifact_probe::store::gc::run_gc(&store, cfg);
        passes += 1;
        std::thread::sleep(std::time::Duration::from_millis(interval_ms));
    }
    let peak = sampler.stop();
    // Machine-readable peak line for the parent bench to fold in.
    eprintln!("GCPeak={}", peak);
    eprintln!("gc-loop: {} passes", passes);
    exit(0)
}

/// PLAN-798 T-07 AutoDown probe envelope (`avworker/1`): one bounded
/// request on stdin → one response on stdout. The worker NEVER sees the
/// check's expected outcomes — it reports the pipeline's observed result
/// and the parent-side runner judges. Request cap 4 MiB (probe metadata
/// budget); the run store is a fresh per-request scratch under the given
/// store root.
fn case_serve() -> ! {
    use std::io::Read;
    const REQUEST_CAP: usize = 4 * 1024 * 1024;
    let mut buf = Vec::new();
    match std::io::stdin()
        .take(REQUEST_CAP as u64 + 1)
        .read_to_end(&mut buf)
    {
        Ok(n) if n > REQUEST_CAP => {
            eprintln!("request exceeds {REQUEST_CAP} bytes");
            exit(2);
        }
        Ok(_) => {}
        Err(e) => {
            eprintln!("read stdin: {e}");
            exit(2);
        }
    }
    let req: serde_json::Value = match serde_json::from_slice(&buf) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("bad request json: {e}");
            exit(2);
        }
    };
    let recipe = req["recipe"].as_str().unwrap_or_default().to_string();
    let nonce = req["nonce"].as_str().unwrap_or_default().to_string();
    let request_hash = req["request_hash"].as_str().unwrap_or_default().to_string();
    let echo = |ok: serde_json::Value| -> ! {
        let out = serde_json::json!({
            "schema": "avworker/1",
            "nonce": nonce,
            "request_hash": request_hash,
            "ok": ok,
        });
        let mut line = serde_json::to_string(&out).unwrap_or_default();
        line.push('\n');
        std::io::stdout().write_all(line.as_bytes()).ok();
        exit(0)
    };
    if recipe != "ac-av/core-run-v1" {
        eprintln!("unknown recipe {recipe}");
        exit(2);
    }
    // Steps contract (T-07 edit-session, F798-R3-07): run + edit-source
    // steps; a sequence may LEAD with edits (the measured formal window)
    // but must END with the judged run; edits apply to the materialized
    // source before any leading run executes.
    let steps_json = match req["steps"].as_array() {
        Some(a) if !a.is_empty() && a[a.len() - 1]["op"].as_str() == Some("run") => a.clone(),
        _ => {
            eprintln!("ac-av/core-run-v1 step sequences must end with a run step");
            exit(2);
        }
    };
    let source = match req["source_b64"].as_str().and_then(|b| {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.decode(b).ok()
    }) {
        Some(s) => s,
        None => {
            eprintln!("missing/invalid source_b64");
            exit(2);
        }
    };
    let source = match String::from_utf8(source) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("source is not utf-8");
            exit(2);
        }
    };
    let entry = req["entry"].as_str().unwrap_or("d_main").to_string();
    let caps: Vec<String> = req["caps"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let store_root = req["store_root"].as_str().unwrap_or_default().to_string();
    if store_root.is_empty() {
        eprintln!("missing store_root");
        exit(2);
    }
    // Session model (F798-R3-07): with an explicit `session` id the store
    // directory is SHARED across worker calls of that session (the formal
    // two-call window: warm-up call, then the measured edit+run call over
    // the warm store) and the PARENT owns cleanup — the worker never
    // removes it. Without a session the store stays private per request
    // and is removed on completion.
    let session = req["session"].as_str().map(String::from);
    if let Some(s) = &session {
        if s.is_empty()
            || s.len() > 64
            || !s
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            eprintln!("invalid session id");
            exit(2);
        }
    }
    let mut store_dir = std::path::PathBuf::from(&store_root);
    match &session {
        Some(s) => store_dir.push(format!("case-{s}")),
        None => store_dir.push(format!("case-{}", nonce.get(0..16).unwrap_or("x"))),
    }
    if session.is_none() {
        let _ = std::fs::remove_dir_all(&store_dir);
    }
    let mut d = auto_av_probe::driver::Driver::with_store(&store_dir, "autodown-case")
        .unwrap_or_else(|e| {
            eprintln!("store: {e}");
            exit(3)
        });

    let replace_once = |src: &str, from: &str, to: &str| -> Result<String, String> {
        let hits = src.matches(from).count();
        if hits != 1 {
            return Err(format!(
                "edit-source literal matched {hits} times (expected exactly once)"
            ));
        }
        Ok(src.replacen(from, to, 1))
    };

    let mut current_source = source;
    let mut step_outs: Vec<serde_json::Value> = Vec::new();
    for st in steps_json.iter() {
        let step_started = std::time::Instant::now();
        match st["op"].as_str() {
            Some("edit-source") => {
                let from = st["from"].as_str().unwrap_or_default();
                let to = st["to"].as_str().unwrap_or_default();
                match replace_once(&current_source, from, to) {
                    Ok(next) => {
                        current_source = next;
                        step_outs.push(serde_json::json!({
                            "op": "edit-source",
                            "outcome": "edited",
                            "wall_ms": step_started.elapsed().as_secs_f64() * 1000.0,
                        }));
                    }
                    Err(e) => {
                        // A non-discriminating edit literal is a request
                        // error (the parent validates the same rule).
                        eprintln!("{e}");
                        exit(2);
                    }
                }
            }
            Some("run") => {
                let r = d.request(&current_source, &entry, &caps);
                let (outcome, value, trap, diag) = if let Some(dg) = &r.diag {
                    (
                        "diag".to_string(),
                        None,
                        None,
                        Some(serde_json::json!({
                            "stage": dg.stage, "code": dg.code, "message": dg.message,
                        })),
                    )
                } else {
                    match &r.av {
                        Some(auto_av_core::contract::ExecOutcome::Success { value, .. }) => {
                            ("success".to_string(), value.as_i32(), None, None)
                        }
                        Some(auto_av_core::contract::ExecOutcome::Trap { kind, .. }) => (
                            "trap".to_string(),
                            None,
                            Some(kind.code().to_string()),
                            None,
                        ),
                        other => (
                            "infra".to_string(),
                            None,
                            None,
                            Some(serde_json::json!({ "detail": format!("{:?}", other) })),
                        ),
                    }
                };
                let counters: std::collections::BTreeMap<String, u64> =
                    r.counters.iter().map(|(k, v)| (k.clone(), *v)).collect();
                // Stage spans (bounded observability, plan 5.5): each span's
                // own duration in ms — nested spans never summed.
                let spans: Vec<(String, f64)> = r
                    .spans
                    .iter()
                    .map(|sp| (sp.name.clone(), sp.dur.as_secs_f64() * 1000.0))
                    .collect();
                step_outs.push(serde_json::json!({
                    "op": "run",
                    "outcome": outcome,
                    "value": value,
                    "trap": trap,
                    "diag": diag,
                    "counters": counters,
                    "spans": spans,
                    "source_hash": r.source_hash,
                    "wall_ms": r.wall_ms,
                    "trace": r.trace.iter().map(|e| match e {
                        auto_av_core::mir::TraceEvent::MarkA => "a",
                        auto_av_core::mir::TraceEvent::MarkB => "b",
                    }).collect::<Vec<&str>>(),
                }));
            }
            other => {
                eprintln!("unknown step op {other:?}");
                exit(2);
            }
        }
    }
    if session.is_none() {
        let _ = std::fs::remove_dir_all(&store_dir);
    }

    // Test-only fault injection seam (F798-R3-05/06 counter-controls; the
    // same pattern the sema worker's FAULT env uses). The parent runner's
    // formal negatives set AV798_FAULT to verify that worker-contract
    // violations are INFRA, never green language verdicts. Never set in
    // production paths.
    if let Ok(fault) = std::env::var("AV798_FAULT") {
        match fault.as_str() {
            "exit9" => {
                let mut line = serde_json::to_string(&serde_json::json!({
                    "schema": "avworker/1", "nonce": nonce, "request_hash": request_hash,
                }))
                .unwrap_or_default();
                line.push('\n');
                use std::io::Write;
                std::io::stdout().write_all(line.as_bytes()).ok();
                std::io::stdout().flush().ok();
                exit(9);
            }
            "extra-frame" => {
                use std::io::Write;
                let ok_out = serde_json::json!({
                    "schema": "avworker/1",
                    "nonce": nonce,
                    "request_hash": request_hash,
                    "ok": {
                        "recipe": recipe,
                        "steps": step_outs,
                        "worker_version": EXPECTED_WORKER_VERSION,
                        "av_core_version": auto_av_core::contract::AV_PROFILE,
                    },
                });
                let mut line = serde_json::to_string(&ok_out).unwrap_or_default();
                line.push('\n');
                line.push_str("NOT_A_VALID_PROTOCOL_FRAME\n");
                std::io::stdout().write_all(line.as_bytes()).ok();
                std::io::stdout().flush().ok();
                exit(0);
            }
            "wrong-hash" => {
                for st in step_outs.iter_mut() {
                    if st["op"] == "run" {
                        st["source_hash"] = serde_json::json!("ffffffffffffffff");
                    }
                }
            }
            other if other.starts_with("delay-ms:") => {
                let ms: u64 = other["delay-ms:".len()..].trim().parse().unwrap_or(0);
                std::thread::sleep(std::time::Duration::from_millis(ms));
            }
            _ => {}
        }
    }

    echo(serde_json::json!({
        "recipe": recipe,
        "steps": step_outs,
        "worker_version": EXPECTED_WORKER_VERSION,
        "av_core_version": auto_av_core::contract::AV_PROFILE,
    }))
}

pub const EXPECTED_WORKER_VERSION: &str = "avworker/1.0";

/// Emit the frozen load's revision `n` (0 = base) source and independent
/// expected value as JSON on stdout — the single source of truth for the
/// frozen load stays in this workspace (the formal AutoDown turnaround
/// harness in auto-lang fetches revisions through this command instead of
/// duplicating the generator).
fn load_revision(args: &[String]) -> ! {
    let rev: u32 = flag(args, "--rev")
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            eprintln!("load-revision needs --rev N");
            exit(2)
        });
    let load = auto_av_probe::load256::generate_revision(rev);
    let out = serde_json::json!({
        "revision": rev,
        "source": load.source,
        "expected_main": load.expected_main,
    });
    let mut line = serde_json::to_string(&out).unwrap_or_default();
    line.push('\n');
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(0)
}

fn bench_turnaround(args: &[String]) -> ! {
    let samples: usize = flag(args, "--samples")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let gate_ms: f64 = flag(args, "--gate-ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000.0);
    let out_path = flag(args, "--out").unwrap_or_else(|| "-".into());
    let self_exe = std::env::current_exe().expect("self exe");

    // Worker private-bytes PEAK (F798-R2-02): a background sampler polls
    // PrivateUsage for the whole bench; child processes report their own
    // peaks and the bench folds in the maximum. The endpoint delta stays
    // as informational context only — the budget gates on the peak.
    #[cfg(windows)]
    let mem0 = ac_artifact_probe::sysinfo::current_process_memory();
    let sampler = auto_av_probe::mempeek::PeakSampler::start(5);

    let load = auto_av_probe::load256::generate();
    // Frontend attribution Line A (r5 §5.9, ac-core owned): measure the
    // LANDED scalar frontend's own transient peak on the frozen load, in
    // isolation. HARD-gated at 224 MiB (anchored to the measured
    // ~200-212 MiB; descriptor::bind alone ≈158.9 MiB per the R12-04
    // stage attribution). ac-core is read-only for this plan — the line
    // reports and gates the frontend's own cost; the probe's residual
    // (Line B) is gated separately.
    let frontend_peak = {
        let s = auto_av_probe::mempeek::PeakSampler::start(5);
        let _ =
            auto_ac_prototype::source::compile_scalar_source(&load.source, "attribution.at", &[]);
        s.stop()
    };
    let bench = auto_av_probe::bench::BenchInputs {
        conflict: auto_av_probe::load256::edit_signature_conflict(
            &auto_av_probe::load256::generate().source,
        ),
        load,
        samples,
        gate_ms,
        self_exe: self_exe.clone(),
        revision_target: 256 * 1024,
    };
    let (rows, mut summary) = auto_av_probe::bench::run_six_scenarios(&bench);
    summary.frontend_peak_bytes = Some(frontend_peak);

    let mut out: Box<dyn Write> = if out_path == "-" {
        Box::new(std::io::stdout())
    } else {
        Box::new(std::fs::File::create(&out_path).expect("out file"))
    };
    for row in &rows {
        let mut l = serde_json::to_string(row).unwrap_or_default();
        l.push('\n');
        out.write_all(l.as_bytes()).ok();
    }

    let own_peak = sampler.stop();
    #[cfg(windows)]
    {
        let mem1 = ac_artifact_probe::sysinfo::current_process_memory();
        // Informational endpoint delta; both r5 budget lines gate on the
        // sampled PEAKS (own process + max child), per F798-R2-02/§5.9.
        summary.worker_private_bytes_delta =
            Some((mem1.private_bytes as f64 - mem0.private_bytes as f64).max(0.0));
        let overall = own_peak.max(summary.child_private_bytes_peak);
        summary.private_bytes_peak = Some(overall);
        // r5 two-line targets (§5.9): Line A frontend transient (ac-core
        // owned) ≤224 MiB; Line B probe-owned residual ≤128 MiB.
        summary.frontend_transient_target_met =
            Some(frontend_peak <= auto_av_probe::bench::BUDGET_FRONTEND_TRANSIENT_PEAK_BYTES);
        let residual = overall.saturating_sub(frontend_peak);
        summary.worker_private_bytes_target_met =
            Some(residual <= auto_av_probe::bench::BUDGET_WORKER_PRIVATE_PEAK_BYTES);
    }
    #[cfg(not(windows))]
    {
        let _ = own_peak;
    }

    // Real summary + gate enforcement (F798-R1-03): gate failures make the
    // exit non-zero. `--gate-ms 0` always fails by construction.
    let failures = auto_av_probe::bench::validate_and_gate(&rows, &summary, samples);
    let mut summary_value = serde_json::to_value(&summary).unwrap_or_default();
    if let serde_json::Value::Object(map) = &mut summary_value {
        map.insert("hard_gate_failures".into(), serde_json::json!(failures));
    }
    let mut line = serde_json::to_string(&summary_value).unwrap_or_default();
    line.push('\n');
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(if failures.is_empty() { 0 } else { 5 })
}

/// F798-R7-01 helper: the UNSUPERVISED native build stages, run inside this
/// worker so the PARENT can kill the whole tree (including a stalled
/// linker) at the remaining-budget deadline. Prints one JSON line.
fn native_build(args: &[String]) -> ! {
    let src_file = flag(args, "--src-file").unwrap_or_else(|| usage());
    let entry = flag(args, "--entry").unwrap_or_else(|| usage());
    let workdir = flag(args, "--workdir").unwrap_or_else(|| usage());
    let trace_lib = flag(args, "--trace-lib");
    let caps: Vec<String> = flag(args, "--caps")
        .map(|c| {
            c.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let out = match (|| -> Result<serde_json::Value, String> {
        let src = std::fs::read_to_string(&src_file).map_err(|e| format!("build: {}", e))?;
        let compiled = auto_ac_prototype::source::compile_scalar_source(&src, "native.at", &caps)
            .map_err(|ds| {
            format!(
                "build: {:?}",
                ds.first().map(|d| d.message.clone()).unwrap_or_default()
            )
        })?;
        let (obj, exe, receipt) = auto_av_probe::native_reference::build_stages(
            &compiled.checked,
            &entry,
            std::path::Path::new(&workdir),
            trace_lib.as_deref().map(std::path::Path::new),
        )?;
        Ok(serde_json::json!({
            "ok": true,
            "obj": obj.display().to_string(),
            "exe": exe.display().to_string(),
            "receipt": receipt.display().to_string(),
        }))
    })() {
        Ok(v) => v,
        Err(e) => serde_json::json!({ "ok": false, "error": e }),
    };
    let mut line = serde_json::to_string(&out).unwrap_or_default();
    line.push('\n');
    use std::io::Write;
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(0)
}

/// F798-R7-01 helper: the contained, capped run stage. Prints one JSON line
/// (streams base64 — exact bytes, invalid UTF-8 included).
fn native_run(args: &[String]) -> ! {
    let exe = flag(args, "--exe").unwrap_or_else(|| usage());
    let budget_ms: u64 = flag(args, "--budget-ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);
    let stdout_cap: usize = flag(args, "--stdout-cap")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024 * 1024);
    let stderr_cap: usize = flag(args, "--stderr-cap")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024 * 1024);
    let out = match auto_av_probe::native_reference::run_stage(
        std::path::Path::new(&exe),
        std::time::Duration::from_millis(budget_ms),
        stdout_cap,
        stderr_cap,
    ) {
        Ok((code, so, se)) => {
            use base64::Engine as _;
            serde_json::json!({
                "ok": true,
                "exit_code": code,
                "stdout_b64": base64::engine::general_purpose::STANDARD.encode(&so),
                "stderr_b64": base64::engine::general_purpose::STANDARD.encode(&se),
            })
        }
        Err(e) => serde_json::json!({ "ok": false, "error": e }),
    };
    let mut line = serde_json::to_string(&out).unwrap_or_default();
    line.push('\n');
    use std::io::Write;
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(0)
}
