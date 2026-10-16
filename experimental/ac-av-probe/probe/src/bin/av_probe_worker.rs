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
