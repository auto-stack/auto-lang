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
        other => {
            eprintln!("unknown subcommand `{}`", other);
            usage();
        }
    }
}

fn usage() -> ! {
    eprintln!("usage: av_probe_worker restore-run --store DIR --owner NAME --entry FN --manifest-artifact ID [--snapshot HEX]");
    eprintln!("       av_probe_worker request --store DIR --src-file FILE --entry FN [--project P] [--caps C1,C2]");
    eprintln!("       av_probe_worker gc-loop --store DIR --stop-file FILE [--interval-ms N]");
    eprintln!("       av_probe_worker bench-turnaround --samples N --gate-ms MS --out FILE");
    eprintln!(
        "       av_probe_worker case-serve  (PLAN-798 T-07: AutoDown probe envelope on stdin)"
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

    let outcome = auto_av_probe::worker_restore::restore_and_run(
        &PathBuf::from(store_dir),
        &owner,
        &entry,
        &manifest_artifact,
        snapshot,
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
    // Steps contract (T-07 edit-session): starts with `run`; edit-source
    // steps (literal must match exactly once) interleave with re-runs,
    // all inside this ONE session over a persistent per-request store.
    let steps_json = match req["steps"].as_array() {
        Some(a) if !a.is_empty() && a[0]["op"] == "run" => a.clone(),
        _ => {
            eprintln!("ac-av/core-run-v1 step sequences must start with a run step");
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
    // One persistent per-request session store: warm across the session's
    // steps (the formal warm-edit primitive); removed afterwards.
    let mut store_dir = std::path::PathBuf::from(&store_root);
    store_dir.push(format!("case-{}", nonce.get(0..16).unwrap_or("x")));
    let _ = std::fs::remove_dir_all(&store_dir);
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
                step_outs.push(serde_json::json!({
                    "op": "run",
                    "outcome": outcome,
                    "value": value,
                    "trap": trap,
                    "diag": diag,
                    "counters": counters,
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
    let _ = std::fs::remove_dir_all(&store_dir);

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
    // Frontend attribution (F798-R2-02): measure the LANDED scalar
    // frontend's own transient peak on the frozen load, in isolation. The
    // observed spike (~180 MiB) dominates the worker peak while the probe
    // layers stay single-digit MB; ac-core is read-only for this plan, so
    // the §3 128 MiB target miss is reported with attribution instead of
    // being silently gated or silently blamed on the AV pipeline.
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
        // Informational endpoint delta; the budget gates on the sampled
        // PEAK (own process + max child), per F798-R2-02.
        summary.worker_private_bytes_delta =
            Some((mem1.private_bytes as f64 - mem0.private_bytes as f64).max(0.0));
        let overall = own_peak.max(summary.child_private_bytes_peak);
        summary.private_bytes_peak = Some(overall);
        // §3 TARGET (advisory with attribution): increment vs idle.
        summary.worker_private_bytes_target_met = Some(
            overall.saturating_sub(mem0.private_bytes)
                <= auto_av_probe::bench::BUDGET_WORKER_PRIVATE_PEAK_BYTES,
        );
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
