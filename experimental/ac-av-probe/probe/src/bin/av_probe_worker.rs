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
//!   semantics). Enforces the hard gate: `--gate-ms` is the enforced P90
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
        "data-restore-run" => data_restore_run(&args[1..]),
        "data-mem-envelope" => data_mem_envelope(&args[1..]),
        "request" => request(&args[1..]),
        "owned-request" => owned_request(&args[1..]),
        "collections-request" => collections_request(&args[1..]),
        "dynamic-request" => dynamic_request(&args[1..]),
        "owned-restore-run" => owned_restore_run(&args[1..]),
        "gc-loop" => gc_loop(&args[1..]),
        "case-serve" => case_serve(),
        "load-revision" => load_revision(&args[1..]),
        "bench-turnaround" => bench_turnaround(&args[1..]),
        "frontend-attribution" => frontend_attribution(),
        "session-serve" => session_serve(),
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
    eprintln!("       av_probe_worker data-restore-run --store DIR --owner NAME --entry FN --snapshot HEX --types-fp FP   (PLAN-820: data-profile module restore, re-verify, execute)");
    eprintln!("       av_probe_worker data-mem-envelope [--rounds N] [--store DIR] [--transient MIB] [--hold-ms N]   (PLAN-820 F820-R1-02: parent-sampled memory-envelope worker modes)");
    eprintln!("       av_probe_worker request --store DIR --src-file FILE --entry FN [--project P] [--caps C1,C2]");
    eprintln!("       av_probe_worker owned-request --store DIR --src-file FILE --entry FN   (PLAN-821 T-02: owned whole path + arena ledger + sampled peak, one JSON line)");
    eprintln!("       av_probe_worker collections-request --store DIR --fixture ID   (PLAN-822 T-09: collections whole path + ledger face + sampled peaks, one JSON line)");
    eprintln!("       av_probe_worker dynamic-request --store DIR --fixture ID   (PLAN-824 T-02: dynamic whole path + portable round + sampled peaks, one JSON line)");
    eprintln!("       av_probe_worker owned-restore-run --store DIR --owner NAME --entry FN --snapshot HEX --types-fp FP   (PLAN-821 R1-F10: owned new-process restore, re-verify + execute)");
    eprintln!("       av_probe_worker gc-loop --store DIR --stop-file FILE [--interval-ms N]");
    eprintln!("       av_probe_worker bench-turnaround --samples N --gate-ms MS --out FILE");
    eprintln!("       av_probe_worker frontend-attribution  (PLAN-828 T-07: Line-A frontend transient, isolated, one JSON line)");
    eprintln!("       av_probe_worker session-serve  (PLAN-828 T-08: persistent formal-session worker, JSON-lines on stdio)");
    eprintln!(
        "       av_probe_worker case-serve  (PLAN-798 T-07: AutoDown probe envelope on stdin)"
    );
    eprintln!("       av_probe_worker native-build --src-file F --entry FN --workdir DIR [--trace-lib P] [--caps C1,C2]");
    eprintln!(
        "       av_probe_worker native-run --exe P --budget-ms N --stdout-cap N --stderr-cap N"
    );
    exit(2)
}

/// PLAN-820 F820-R1-02: the data-pipeline memory-envelope worker modes.
/// The PARENT samples THIS process's private-bytes peak externally
/// (`mempeek::ChildPeakSampler`) for the child's whole lifetime; this
/// child never self-reports a verdict. Composable modes:
///   --rounds N       run N data requests (the load; a varying record
///                    source per round — the exact pipeline the old
///                    in-process envelope ran, store round trip included)
///   --store DIR      store directory for the rounds
///   --transient MIB  allocate + touch + free an MIB-sized staging buffer
///                    mid-life (the sampler-capture counter-control)
///   --hold-ms N      sleep before exit (a stable observation window)
fn data_mem_envelope(args: &[String]) -> ! {
    let rounds: usize = flag(args, "--rounds")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let transient_mib: usize = flag(args, "--transient")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let hold_ms: u64 = flag(args, "--hold-ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let store = flag(args, "--store").map(std::path::PathBuf::from);
    let mut failures: Vec<String> = Vec::new();
    let mut ok_rounds = 0usize;
    for i in 0..rounds {
        let src = format!(
            "type Inner {{ x int }}
type Outer {{ inner Inner }}
fn main() int {{
    var o = Outer {{ inner: Inner {{ x: {i} }} }}
    o.inner.x = o.inner.x + 2
    return o.inner.x
}}
"
        );
        let dir = store
            .clone()
            .unwrap_or_else(|| std::env::temp_dir().join("av820-mem-envelope"));
        match auto_av_probe::data_driver::request_data(Some(&dir), "mem", &src, "d_main", &[]) {
            Ok(r)
                if matches!(
                    r.av,
                    Some(auto_av_core::contract::ExecOutcome::Success { .. })
                ) =>
            {
                ok_rounds += 1;
            }
            Ok(r) => failures.push(format!("round {i}: non-success outcome {:?}", r.av)),
            Err(e) => failures.push(format!("round {i}: {e}")),
        }
    }
    if transient_mib > 0 {
        // The staging buffer must stay ALIVE across several parent-side
        // sampling periods before it is freed — a spike shorter than the
        // poll interval is legitimately missable (poll-quantized evidence).
        let mut v = vec![0u8; transient_mib * 1024 * 1024];
        for chunk in v.chunks_mut(4096 * 64) {
            chunk[0] = 1;
        }
        std::thread::sleep(std::time::Duration::from_millis(hold_ms.max(300)));
        drop(v);
    } else if hold_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(hold_ms));
    }
    let line = serde_json::json!({
        "ok": failures.is_empty(),
        "rounds": rounds,
        "ok_rounds": ok_rounds,
        "transient_mib": transient_mib,
        "hold_ms": hold_ms,
        "failures": failures,
    });
    println!("{line}");
    exit(if failures.is_empty() { 0 } else { 4 })
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

/// PLAN-821 R1-F10: restore a persisted owned module from the store in
/// THIS fresh process and execute — the real NEW-WORKER seam (envelope
/// admission, decode and the FULL deep verification re-run; nothing on
/// disk is trusted). Prints the outcome as one JSON line; non-zero exit
/// unless the restored module verifies AND executes to Success.
fn owned_restore_run(args: &[String]) -> ! {
    let mut store = None;
    let mut owner = String::new();
    let mut entry = String::new();
    let mut snapshot = None;
    let mut types_fp = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--store" => store = Some(args[i + 1].clone()),
            "--owner" => owner = args[i + 1].clone(),
            "--entry" => entry = args[i + 1].clone(),
            "--snapshot" => {
                snapshot = Some(
                    u64::from_str_radix(args[i + 1].trim_start_matches("0x"), 16)
                        .expect("snapshot hex"),
                )
            }
            "--types-fp" => types_fp = args[i + 1].clone(),
            other => {
                eprintln!("owned-restore-run: unknown flag `{other}`");
                exit(2);
            }
        }
        i += 2;
    }
    let (Some(store), Some(snapshot)) = (store, snapshot) else {
        eprintln!(
            "owned-restore-run: --store/--owner/--entry/--snapshot/--types-fp are all required"
        );
        exit(2);
    };
    let payload = match auto_av_probe::owned_driver::restore_owned(
        std::path::Path::new(&store),
        &owner,
        &entry,
        snapshot,
        &types_fp,
    ) {
        Ok(auto_av_core::contract::ExecOutcome::Success { value, .. }) => {
            let face = auto_av_probe::owned_fixture::canonical_owned(&value, &Default::default())
                .unwrap_or(serde_json::json!({ "note": "non-canonical" }));
            serde_json::json!({
                "ok": true,
                "restored": true,
                "result": face,
            })
        }
        Ok(other) => serde_json::json!({
            "ok": false,
            "restored": true,
            "outcome": format!("{other:?}"),
        }),
        Err(e) => serde_json::json!({
            "ok": false,
            "restored": false,
            "error": e,
            "dispatched": false,
        }),
    };
    let mut line = serde_json::to_string(&payload).unwrap_or_default();
    line.push('\n');
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(if payload["ok"].as_bool() == Some(true) {
        0
    } else {
        4
    })
}

/// PLAN-821 T-02: one owned-profile whole-path request in THIS fresh
/// process — the CPU/memory evidence entry. Wall covers the full
/// request → outcome span (store round trip included); the peak is the
/// parent-side-style sampled private-bytes high-water of THIS process.
/// The result face and post-release ledger mirror case-serve's owned
/// branch; the JUDGE is always the caller.
fn owned_request(args: &[String]) -> ! {
    let store_dir = flag(args, "--store").unwrap_or_else(|| usage());
    let src_file = flag(args, "--src-file").unwrap_or_else(|| usage());
    let entry = flag(args, "--entry").unwrap_or_else(|| usage());
    let src = std::fs::read_to_string(&src_file).unwrap_or_else(|e| {
        eprintln!("read {}: {}", src_file, e);
        exit(2)
    });
    let started = std::time::Instant::now();
    let sampler = auto_av_probe::mempeek::PeakSampler::start(5);
    let r = auto_av_probe::owned_driver::request_owned(
        Some(&PathBuf::from(&store_dir)),
        "owned-evidence",
        &src,
        &entry,
        &[],
    );
    let wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let peak = sampler.stop();
    let payload = match r {
        Ok(mut r) => {
            let av = r.av.take().expect("request_owned sets av");
            let ok = matches!(av, auto_av_core::contract::ExecOutcome::Success { .. });
            let owned = match av {
                auto_av_core::contract::ExecOutcome::Success { value, .. } => {
                    let face =
                        auto_av_probe::owned_fixture::canonical_owned(&value, &r.record_names);
                    match face {
                        Some(face) => {
                            // Host release before the ledger read (the
                            // live=0 oracle's ordering).
                            drop(value);
                            let arena = r.arena.as_ref().expect("owned arena");
                            serde_json::json!({
                                "result": face,
                                "live_after_release": arena.live(),
                                "allocs": arena.alloc_events().iter()
                                    .map(|a| serde_json::json!({ "id": a.id, "bytes": a.bytes }))
                                    .collect::<Vec<_>>(),
                                "frees": arena.free_events().iter()
                                    .map(|e| serde_json::json!({ "id": e.id }))
                                    .collect::<Vec<_>>(),
                            })
                        }
                        None => serde_json::json!({ "error": "non-canonicalizable owned result" }),
                    }
                }
                other => serde_json::json!({ "av": format!("{:?}", other) }),
            };
            serde_json::json!({
                "ok": ok,
                "wall_ms": wall_ms,
                "private_bytes_peak": peak,
                "peak_hot_bytes": r.peak_hot_bytes,
                "peak_staging_bytes": r.peak_staging_bytes,
                "peak_exec_slot_bytes": r.peak_exec_slot_bytes,
                "owned": owned,
                "snapshot": format!("{:016x}", r.snapshot),
            })
        }
        Err(e) => serde_json::json!({
            "ok": false,
            "wall_ms": wall_ms,
            "private_bytes_peak": peak,
            "error": e,
        }),
    };
    let mut line = serde_json::to_string(&payload).unwrap_or_default();
    line.push('\n');
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(if payload["ok"].as_bool() == Some(true) {
        0
    } else {
        4
    })
}

/// PLAN-824 T-02: one dynamic whole-path request in THIS fresh process -
/// the CPU/memory evidence entry over the frozen fixture (the dynamic
/// profile has no source surface; the input is the worker's programmatic
/// typed fixture admitted by the real gates). Mirrors collections-request
/// plus the portable round axes; the JUDGE is always the caller.
fn dynamic_request(args: &[String]) -> ! {
    let store_dir = flag(args, "--store").unwrap_or_else(|| usage());
    let fixture = flag(args, "--fixture").unwrap_or_else(|| usage());
    let started = std::time::Instant::now();
    let sampler = auto_av_probe::mempeek::PeakSampler::start(5);
    let r = auto_av_probe::dynamic_fixture::request_dynamic(
        Some(&PathBuf::from(&store_dir)),
        "dynamic-evidence",
        &fixture,
    );
    let wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let peak = sampler.stop();
    let payload = match r {
        Ok(mut r) => {
            let av = r.av.take().expect("request_dynamic sets av");
            let ok = matches!(av, auto_av_core::contract::ExecOutcome::Success { .. });
            let dynamic = match av {
                auto_av_core::contract::ExecOutcome::Success { value, .. } => {
                    let face = auto_av_probe::owned_fixture::canonical_owned(&value, &r.type_names);
                    match face {
                        Some(face) => {
                            drop(value);
                            let arena = r.arena.as_ref().expect("dynamic arena");
                            serde_json::json!({
                                "result": face,
                                "live_after_release": arena.live(),
                                "allocs": arena.alloc_events().iter()
                                    .map(|a| serde_json::json!({ "id": a.id, "bytes": a.bytes }))
                                    .collect::<Vec<_>>(),
                                "frees": arena.free_events().iter()
                                    .map(|e| serde_json::json!({ "id": e.id, "bytes": e.bytes }))
                                    .collect::<Vec<_>>(),
                                "portable_bytes": r.portable_bytes.len(),
                                "portable_roundtrip": r.portable_roundtrip_ok,
                            })
                        }
                        None => {
                            serde_json::json!({ "error": "non-canonicalizable dynamic result" })
                        }
                    }
                }
                other => serde_json::json!({ "av": format!("{:?}", other) }),
            };
            serde_json::json!({
                "ok": ok,
                "wall_ms": wall_ms,
                "private_bytes_peak": peak,
                "dynamic": dynamic,
                "snapshot": format!("{:016x}", r.snapshot),
            })
        }
        Err(e) => serde_json::json!({
            "ok": false,
            "wall_ms": wall_ms,
            "private_bytes_peak": peak,
            "error": e,
        }),
    };
    let mut line = serde_json::to_string(&payload).unwrap_or_default();
    line.push('\n');
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(if payload["ok"].as_bool() == Some(true) {
        0
    } else {
        4
    })
}

/// PLAN-822 T-09: one collections whole-path request in THIS fresh
/// process — the CPU/memory evidence entry over the frozen fixture (the
/// collections profile has no source surface; the input is the worker's
/// programmatic typed fixture admitted by the real gates). Mirrors
/// owned-request: the parent samples the private-bytes peak, the JUDGE is
/// always the caller.
fn collections_request(args: &[String]) -> ! {
    let store_dir = flag(args, "--store").unwrap_or_else(|| usage());
    let fixture = flag(args, "--fixture").unwrap_or_else(|| usage());
    let started = std::time::Instant::now();
    let sampler = auto_av_probe::mempeek::PeakSampler::start(5);
    let r = auto_av_probe::collections_fixture::request_collections(
        Some(&PathBuf::from(&store_dir)),
        "collections-evidence",
        &fixture,
    );
    let wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let peak = sampler.stop();
    let payload = match r {
        Ok(mut r) => {
            let av = r.av.take().expect("request_collections sets av");
            let ok = matches!(av, auto_av_core::contract::ExecOutcome::Success { .. });
            let collections = match av {
                auto_av_core::contract::ExecOutcome::Success { value, .. } => {
                    let face = auto_av_probe::owned_fixture::canonical_owned(&value, &r.type_names);
                    match face {
                        Some(face) => {
                            // Host release before the ledger read (the
                            // live=0 oracle's ordering).
                            drop(value);
                            let arena = r.arena.as_ref().expect("collections arena");
                            serde_json::json!({
                                "result": face,
                                "live_after_release": arena.live(),
                                "allocs": arena.alloc_events().iter()
                                    .map(|a| serde_json::json!({ "id": a.id, "bytes": a.bytes }))
                                    .collect::<Vec<_>>(),
                                "frees": arena.free_events().iter()
                                    .map(|e| serde_json::json!({ "id": e.id, "bytes": e.bytes }))
                                    .collect::<Vec<_>>(),
                            })
                        }
                        None => serde_json::json!({
                            "error": "non-canonicalizable collections result"
                        }),
                    }
                }
                other => serde_json::json!({ "av": format!("{:?}", other) }),
            };
            serde_json::json!({
                "ok": ok,
                "wall_ms": wall_ms,
                "private_bytes_peak": peak,
                "peak_hot_bytes": r.peak_hot_bytes,
                "peak_staging_bytes": r.peak_staging_bytes,
                "peak_exec_slot_bytes": r.peak_exec_slot_bytes,
                "collections": collections,
                "snapshot": format!("{:016x}", r.snapshot),
            })
        }
        Err(e) => serde_json::json!({
            "ok": false,
            "wall_ms": wall_ms,
            "private_bytes_peak": peak,
            "error": e,
        }),
    };
    let mut line = serde_json::to_string(&payload).unwrap_or_default();
    line.push('\n');
    std::io::stdout().write_all(line.as_bytes()).ok();
    exit(if payload["ok"].as_bool() == Some(true) {
        0
    } else {
        4
    })
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

/// The av-core identity a recipe's response must echo (admission mirror
/// of the parent-side constants): core=av.core-i32/1, data=av.data/1,
/// owned=av.owned/1 (PLAN-821).
fn core_version_for(recipe: &str) -> &'static str {
    match recipe {
        "ac-av/data-run-v1" => auto_av_core::contract::AV_DATA_PROFILE,
        "ac-av/owned-run-v1" => auto_av_core::contract::AV_OWNED_PROFILE,
        "ac-av/collections-run-v1" => auto_av_core::contract::AV_COLLECTIONS_PROFILE,
        "ac-av/dynamic-run-v1" => auto_av_core::contract::AV_DYNAMIC_PROFILE,
        _ => auto_av_core::contract::AV_PROFILE,
    }
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
    let data_recipe = recipe == "ac-av/data-run-v1";
    // PLAN-821 T-02: the owned-profile recipe (`avworker/1` envelope domain
    // — the recipe id itself is the discriminator, no field grows).
    let owned_recipe = recipe == "ac-av/owned-run-v1";
    // PLAN-822 T-02: the collections-profile recipe (avworker/1 envelope
    // domain; the recipe id is the discriminator).
    let collections_recipe = recipe == "ac-av/collections-run-v1";
    // PLAN-824: the dynamic-profile recipe (avworker/1 envelope domain).
    let dynamic_recipe = recipe == "ac-av/dynamic-run-v1";
    // PLAN-820 F820-R6-01: the typed-fixture entry (`av.entry` of the
    // form `typed:<id>`, validated by the parent against its OWN frozen
    // table before the request) selects a frozen av.data-fixture/1 input
    // instead of a record source.
    let typed_fixture = req["typed_fixture"].as_str().map(|s| s.to_string());
    // PLAN-821: the owned-fixture entry (`owned:<id>`, parent-validated
    // first) selects a frozen av.owned-fixture/1 source.
    let owned_fixture = req["owned_fixture"].as_str().map(|s| s.to_string());
    // PLAN-822: the collections-fixture entry (`collections:<id>`,
    // parent-validated first) selects a frozen av-collections fixture.
    let collections_fixture_id = req["collections_fixture"].as_str().map(|s| s.to_string());
    // PLAN-824: the dynamic-fixture entry (`dynamic:<id>`, parent-validated
    // first) selects a frozen av-dynamic fixture.
    let dynamic_fixture_id = req["dynamic_fixture"].as_str().map(|s| s.to_string());
    if recipe != "ac-av/core-run-v1"
        && !data_recipe
        && !owned_recipe
        && !collections_recipe
        && !dynamic_recipe
    {
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
    let mut d = if data_recipe {
        None
    } else {
        Some(
            auto_av_probe::driver::Driver::with_store(&store_dir, "autodown-case").unwrap_or_else(
                |e| {
                    eprintln!("store: {e}");
                    exit(3)
                },
            ),
        )
    };

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
                // PLAN-822 T-02: the collections-profile pipeline. A frozen
                // `collections:<id>` fixture carries its own face (result +
                // post-release ledger); there is NO source surface for this
                // profile (contract non-goal), so the fixture entry is the
                // only run shape. The PARENT judges against its own frozen
                // expectations — never the worker's word.
                // PLAN-824 T-02: the dynamic-profile pipeline. A frozen
                // `dynamic:<id>` fixture carries the judged face (result +
                // ledger + the Dyn/portable axes); NO source surface — the
                // fixture entry is the only run shape. The PARENT judges.
                if dynamic_recipe {
                    let Some(fid) = &dynamic_fixture_id else {
                        eprintln!("dynamic recipe requires a dynamic:<id> entry");
                        exit(2);
                    };
                    let f = match auto_av_probe::dynamic_fixture::dynamic_fixture(fid) {
                        Some(f) => f,
                        None => {
                            eprintln!("unknown dynamic fixture {fid}");
                            exit(2);
                        }
                    };
                    let started = std::time::Instant::now();
                    match auto_av_probe::dynamic_fixture::run_dynamic_fixture(
                        f.id,
                        Some(&store_dir),
                        "autodown-dynamic-case",
                    ) {
                        Ok(dynamic) => step_outs.push(serde_json::json!({
                            "op": "run",
                            "outcome": "success",
                            "value": null,
                            "collections": dynamic,
                            "dynamic": dynamic.get("dyn").cloned().unwrap_or(serde_json::Value::Null),
                            "trap": null,
                            "diag": null,
                            "source_hash": format!("{:016x}", auto_av_core::contract::fnv1a64(entry.as_bytes())),
                            "wall_ms": started.elapsed().as_secs_f64() * 1000.0,
                        })),
                        Err(e) => {
                            eprintln!("dynamic fixture {fid}: {e}");
                            exit(2);
                        }
                    }
                    continue;
                }
                if collections_recipe {
                    let Some(fid) = &collections_fixture_id else {
                        eprintln!("collections recipe requires a collections:<id> entry");
                        exit(2);
                    };
                    let f = match auto_av_probe::collections_fixture::collections_fixture(fid) {
                        Some(f) => f,
                        None => {
                            eprintln!("unknown collections fixture {fid}");
                            exit(2);
                        }
                    };
                    let started = std::time::Instant::now();
                    match auto_av_probe::collections_fixture::run_collections_fixture(
                        f.id,
                        Some(&store_dir),
                        "autodown-collections-case",
                    ) {
                        Ok(collections) => step_outs.push(serde_json::json!({
                            "op": "run",
                            "outcome": "success",
                            "value": null,
                            "collections": collections,
                            "trap": null,
                            "diag": null,
                            "source_hash": format!("{:016x}", auto_av_core::contract::fnv1a64(entry.as_bytes())),
                            "wall_ms": started.elapsed().as_secs_f64() * 1000.0,
                        })),
                        Err(e) => {
                            eprintln!("collections fixture {fid}: {e}");
                            exit(2);
                        }
                    }
                    continue;
                }
                if owned_recipe {
                    // PLAN-821 T-02: the owned-profile pipeline. A frozen
                    // `owned:<id>` fixture carries its own face (result +
                    // post-release ledger); a source run reports the same
                    // owned face for its success value. The PARENT judges
                    // against its own frozen expectations — never the
                    // worker's word.
                    if let Some(fid) = &owned_fixture {
                        let f = match auto_av_probe::owned_fixture::owned_fixture(fid) {
                            Some(f) => f,
                            None => {
                                // The parent validates ids too; an unknown
                                // id here is a protocol violation.
                                eprintln!("unknown owned fixture {fid}");
                                exit(2);
                            }
                        };
                        let started = std::time::Instant::now();
                        match auto_av_probe::owned_fixture::run_owned_fixture(
                            f,
                            Some(&store_dir),
                            "autodown-owned-case",
                        ) {
                            Ok(owned) => step_outs.push(serde_json::json!({
                                "op": "run",
                                "outcome": "success",
                                "value": null,
                                "owned": owned,
                                "trap": null,
                                "diag": null,
                                "source_hash": format!("{:016x}", auto_av_core::contract::fnv1a64(entry.as_bytes())),
                                "wall_ms": started.elapsed().as_secs_f64() * 1000.0,
                            })),
                            Err(e) => {
                                eprintln!("owned fixture {fid}: {e}");
                                exit(2);
                            }
                        }
                        continue;
                    }
                    match auto_av_probe::owned_driver::request_owned(
                        Some(&store_dir),
                        "autodown-owned-case",
                        &current_source,
                        &entry,
                        &caps,
                    ) {
                        Ok(mut r) => {
                            let av = r.av.take().expect("request_owned sets av");
                            let snapshot_hex = format!("{:016x}", r.snapshot);
                            match av {
                                auto_av_core::contract::ExecOutcome::Success { value, .. } => {
                                    let value_i32 = value.as_i32();
                                    let face = auto_av_probe::owned_fixture::canonical_owned(
                                        &value,
                                        &r.record_names,
                                    );
                                    let owned = match face {
                                        Some(face) => {
                                            // Host release BEFORE the ledger
                                            // read — the live=0 oracle's
                                            // ordering is part of the face.
                                            drop(value);
                                            let arena = r
                                                .arena
                                                .as_ref()
                                                .expect("owned execution carries the arena");
                                            serde_json::json!({
                                                "result": face,
                                                "live_after_release": arena.live(),
                                                "allocs": arena.alloc_events().iter()
                                                    .map(|a| serde_json::json!({ "id": a.id, "bytes": a.bytes }))
                                                    .collect::<Vec<_>>(),
                                                "frees": arena.free_events().iter()
                                                    .map(|e| serde_json::json!({ "id": e.id }))
                                                    .collect::<Vec<_>>(),
                                            })
                                        }
                                        // A view/token crossing the entry is
                                        // a protocol violation, never green.
                                        None => {
                                            eprintln!("owned result is not canonicalizable (view/token crossed entry)");
                                            exit(2);
                                        }
                                    };
                                    step_outs.push(serde_json::json!({
                                        "op": "run",
                                        "outcome": "success",
                                        "value": value_i32,
                                        "owned": owned,
                                        "trap": null,
                                        "diag": null,
                                        "source_hash": snapshot_hex,
                                        "wall_ms": r.wall_ms,
                                    }));
                                }
                                auto_av_core::contract::ExecOutcome::Trap { kind, .. } => {
                                    step_outs.push(serde_json::json!({
                                        "op": "run",
                                        "outcome": "trap",
                                        "value": null,
                                        "owned": null,
                                        "trap": kind.code().to_string(),
                                        "diag": null,
                                        "source_hash": snapshot_hex,
                                        "wall_ms": r.wall_ms,
                                    }));
                                }
                                other => {
                                    step_outs.push(serde_json::json!({
                                        "op": "run",
                                        "outcome": "infra",
                                        "value": null,
                                        "owned": null,
                                        "trap": null,
                                        "diag": { "detail": format!("{other:?}") },
                                        "source_hash": snapshot_hex,
                                        "wall_ms": r.wall_ms,
                                    }));
                                }
                            }
                        }
                        Err(e) => {
                            // Same stage mapping as the data recipe: source
                            // gate rejections are frontend; admission
                            // failures are verify; anything else is a
                            // request error (infra, never a language face).
                            let stage = if e.starts_with("front door")
                                || e.starts_with("frontend")
                                || e.starts_with("lower")
                            {
                                "frontend"
                            } else if e.starts_with("mir verify")
                                || e.starts_with("encode")
                                || e.starts_with("abc admit")
                            {
                                "verify"
                            } else {
                                eprintln!("owned pipeline: {e}");
                                exit(2);
                            };
                            // The source-gate rejections carry their
                            // diagnostic code as the first token after the
                            // layer prefix ("frontend: <code> <message>")
                            // — surface it for parent-side code judgement.
                            let code = e
                                .strip_prefix("frontend: ")
                                .or_else(|| e.strip_prefix("front door: "))
                                .and_then(|rest| rest.split_whitespace().next())
                                .unwrap_or("");
                            step_outs.push(serde_json::json!({
                                "op": "run",
                                "outcome": "diag",
                                "value": null,
                                "owned": null,
                                "trap": null,
                                "diag": { "stage": stage, "code": code, "message": e },
                                "source_hash": format!(
                                    "{:016x}",
                                    auto_av_core::contract::fnv1a64(current_source.as_bytes())
                                ),
                                "wall_ms": 0.0,
                            }));
                        }
                    }
                    continue;
                }
                if data_recipe {
                    if let Some(fid) = &typed_fixture {
                        // F820-R6-01: frozen typed fixture through the REAL
                        // admission chain; the produced value is reported
                        // as canonical {kind, bits} — the PARENT compares
                        // tag and exact raw bits against its own frozen
                        // expectation (never the worker's word).
                        match auto_av_probe::typed_fixture::typed_fixture(fid) {
                            None => {
                                // The parent validates ids too; an unknown
                                // id here is a protocol violation.
                                eprintln!("unknown typed fixture {fid}");
                                exit(2);
                            }
                            Some(f) => {
                                let started = std::time::Instant::now();
                                match auto_av_probe::typed_fixture::run_fixture(f)
                                    .and_then(|v| {
                                        auto_av_probe::typed_fixture::canonical_typed(&v)
                                            .map(|(kind, bits)| (kind, bits))
                                            .ok_or_else(|| {
                                                "non-scalar fixture result".to_string()
                                            })
                                    }) {
                                    Ok((kind, bits)) => step_outs.push(serde_json::json!({
                                        "op": "run",
                                        "outcome": "success",
                                        "value": null,
                                        "typed": { "kind": kind, "bits": bits },
                                        "trap": null,
                                        "diag": null,
                                        "source_hash": format!("{:016x}", auto_av_core::contract::fnv1a64(entry.as_bytes())),
                                        "wall_ms": started.elapsed().as_secs_f64() * 1000.0,
                                    })),
                                    Err(e) => {
                                        eprintln!("typed fixture {fid}: {e}");
                                        exit(2);
                                    }
                                }
                            }
                        }
                        continue;
                    }
                    // PLAN-820: the data-profile pipeline (record source →
                    // data MIR → AVAD → AV) with the same store model; the
                    // first case's oracle is parent-side like the core
                    // recipe. Wide scalars have no source syntax — data
                    // cases are record-source programs with scalar exits.
                    match auto_av_probe::data_driver::request_data(
                        Some(&store_dir),
                        "autodown-data-case",
                        &current_source,
                        &entry,
                        &caps,
                    ) {
                        Ok(r) => {
                            let (outcome, value, trap, diag, typed) = match &r.av {
                                Some(auto_av_core::contract::ExecOutcome::Success {
                                    value,
                                    ..
                                }) => {
                                    // F820-R6-01: the canonical typed report
                                    // rides EVERY data success — bool/wide
                                    // results are no longer lost to the
                                    // i32-only `value` channel (records
                                    // report no typed face).
                                    let typed = auto_av_probe::typed_fixture::canonical_typed(
                                        value,
                                    )
                                    .map(|(kind, bits)| serde_json::json!({ "kind": kind, "bits": bits }));
                                    ("success".to_string(), value.as_i32(), None, None, typed)
                                }
                                Some(auto_av_core::contract::ExecOutcome::Trap {
                                    kind, ..
                                }) => (
                                    "trap".to_string(),
                                    None,
                                    Some(kind.code().to_string()),
                                    None,
                                    None,
                                ),
                                other => (
                                    "infra".to_string(),
                                    None,
                                    None,
                                    Some(serde_json::json!({ "detail": format!("{:?}", other) })),
                                    None,
                                ),
                            };
                            step_outs.push(serde_json::json!({
                                "op": "run",
                                "outcome": outcome,
                                "value": value,
                                "typed": typed,
                                "trap": trap,
                                "diag": diag,
                                "source_hash": format!("{:016x}", r.snapshot),
                                "wall_ms": r.wall_ms,
                            }));
                        }
                        Err(e) => {
                            // Front-door/frontend/lowering rejections are
                            // PARSE-stage diagnoses (same mapping as the
                            // core recipe); admission failures (mir
                            // verify/encode/abc admit) are VERIFY stage.
                            // Every other failure shape is a request error.
                            let stage = if e.starts_with("front door")
                                || e.starts_with("frontend")
                                || e.starts_with("lower")
                            {
                                "frontend"
                            } else if e.starts_with("mir verify")
                                || e.starts_with("encode")
                                || e.starts_with("abc admit")
                            {
                                "verify"
                            } else {
                                eprintln!("data pipeline: {e}");
                                exit(2);
                            };
                            step_outs.push(serde_json::json!({
                                "op": "run",
                                "outcome": "diag",
                                "value": null,
                                "trap": null,
                                "diag": { "stage": stage, "code": "", "message": e },
                                "source_hash": format!(
                                    "{:016x}",
                                    auto_av_core::contract::fnv1a64(current_source.as_bytes())
                                ),
                                "wall_ms": 0.0,
                            }));
                        }
                    }
                    continue;
                }
                let r = d
                    .as_mut()
                    .expect("core driver")
                    .request(&current_source, &entry, &caps);
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
                        "av_core_version": core_version_for(&recipe),
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
        "av_core_version": core_version_for(&recipe),
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

/// PLAN-828 T-07 dual-line budget verdict, Line A: the LANDED scalar
/// frontend's own transient peak on the frozen load, measured in
/// ISOLATION in this fresh process — the same attribution 口径 as
/// `bench-turnaround`'s `frontend_peak_bytes` (798 r5 §5.9). One JSON
/// line on stdout; the caller (the T-08 campaign test) gates Line A at
/// 224 MiB and computes Line B (probe-owned residual = worker sampled
/// peak − Line A) at 128 MiB.
fn frontend_attribution() -> ! {
    let load = auto_av_probe::load256::generate_revision(0);
    let sampler = auto_av_probe::mempeek::PeakSampler::start(5);
    let compiled =
        auto_ac_prototype::source::compile_scalar_source(&load.source, "attribution.at", &[]);
    let frontend_peak = sampler.stop();
    let ok = compiled.is_ok();
    let line = serde_json::json!({
        "ok": ok,
        "frontend_peak_bytes": frontend_peak,
    });
    println!("{line}");
    exit(if ok { 0 } else { 5 })
}

fn bench_turnaround(args: &[String]) -> ! {
    let samples: usize = flag(args, "--samples")
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let gate_ms: f64 = flag(args, "--gate-ms")
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000.0);
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
    // PLAN-820 F820-R1-03: `--profile record` compiles the same front door
    // under the record domain (the old HIR-native lowering has supported
    // nominal records since PLAN-771); the default stays scalar.
    let record_profile = flag(args, "--profile").as_deref() == Some("record");
    // PLAN-821 T-02: the text-record domain (782's own native chain —
    // tr_layouts/rollback live in ac-core native.rs since 782).
    let text_record_profile = flag(args, "--profile").as_deref() == Some("text-record");
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
        let compiled = if text_record_profile {
            auto_ac_prototype::source::compile_text_record_source(&src, "native.at", &caps)
        } else if record_profile {
            auto_ac_prototype::source::compile_record_source(&src, "native.at", &caps)
        } else {
            auto_ac_prototype::source::compile_scalar_source(&src, "native.at", &caps)
        }
        .map_err(|ds| {
            format!(
                "build: {:?}",
                ds.first().map(|d| d.message.clone()).unwrap_or_default()
            )
        })?;
        let support_lib = flag(args, "--support-lib");
        let (obj, exe, receipt) = auto_av_probe::native_reference::build_stages_with_support(
            &compiled.checked,
            &entry,
            std::path::Path::new(&workdir),
            trace_lib.as_deref().map(std::path::Path::new),
            support_lib.as_deref().map(std::path::Path::new),
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

/// PLAN-828 T-08 (F-12): the PERSISTENT formal-session worker. One
/// process, ONE resident session: `init` warms it up once (the ready
/// handshake carries the CONSTANT worker identity — session id —
/// verified once per §5.5), then a stream of real body edits. Each
/// publication carries the verified CURRENT result, the worker-internal
/// phase timings (attribution only — the judge owns the measured
/// window), the cumulative counter snapshot (the judge takes deltas) and
/// the sampled worker-line memory peak. EOF or `close` terminates. This
/// side never decides the verdict.
fn session_serve() -> ! {
    use auto_av_core::av::{CancelFlag, ExecLimits};
    use auto_av_core::contract::ExecOutcome;
    use auto_av_probe::formal_worker::{
        ClosedMsg, DispatchedIdentity, EditOp, EditResultMsg, ReadyMsg, WorkerReply, WorkerRequest,
    };
    use auto_av_probe::session::{warmup, FunctionIncrementalSession};
    use std::io::BufRead as _;
    use std::time::Instant;

    let stdin = std::io::stdin();
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    let peak = auto_av_probe::mempeek::PeakSampler::start(20);
    let mut session: Option<FunctionIncrementalSession> = None;
    let cancel = CancelFlag::new(false);

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let req: WorkerRequest = match serde_json::from_str(line.trim()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("session-serve: bad request line: {e}");
                break;
            }
        };
        match req {
            WorkerRequest::Init(op) => {
                if session.is_some() {
                    eprintln!("session-serve: double init refused");
                    break;
                }
                let s = match warmup(&op.src, &op.entry, &op.caps) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("session-serve: warm-up failed: {e}");
                        std::process::exit(3);
                    }
                };
                let (exec, _) = s.run(&ExecLimits::default(), &cancel);
                if !matches!(exec, ExecOutcome::Success { .. }) {
                    eprintln!("session-serve: initial run is not a success: {exec:?}");
                    std::process::exit(3);
                }
                // B3-C03: the published value AND the dispatched
                // identities come from the SAME execution's record (one
                // `run` call — the executor's own trace evidence).
                let (dispatched_record, value, run_seq) = s.last_run_publication();
                let value = value.expect("checked Success above");
                let reply = WorkerReply::Ready(ReadyMsg {
                    session_id: s.session_id(),
                    functions: s.function_names().len(),
                    revision: s.revision(),
                    value,
                    // B3-C03: the ready baseline directory — identities
                    // from the initial run's own trace evidence.
                    dispatched: dispatched_record
                        .into_iter()
                        .map(
                            |(name, ptr, generation, content, resident)| DispatchedIdentity {
                                name,
                                ptr,
                                generation,
                                content,
                                resident,
                            },
                        )
                        .collect(),
                    run_seq,
                    av_seq: auto_av_core::av::execution_seq(),
                });
                let _ = session.insert(s);
                writeln!(out, "{}", serde_json::to_string(&reply).unwrap()).unwrap();
                out.flush().unwrap();
            }
            WorkerRequest::Edit(op) => {
                let Some(s) = session.as_mut() else {
                    eprintln!("session-serve: edit before init refused");
                    break;
                };
                let EditOp {
                    req_id,
                    start,
                    end,
                    insert,
                } = op;
                let t0 = Instant::now();
                let outcome = s.edit(start, end, insert.into_bytes());
                let worker_edit_us = t0.elapsed().as_micros() as u64;
                let msg = match outcome {
                    Ok(o) => {
                        let (exec, _) = s.run(&ExecLimits::default(), &cancel);
                        let total = t0.elapsed().as_micros() as u64;
                        let _ = &exec; // non-success outcomes surface via the record below
                        let (dispatched_record, value, run_seq) = s.last_run_publication();
                        EditResultMsg {
                            req_id,
                            ok: o.ok,
                            revision: s.revision(),
                            changed_fn: o.changed_fn.clone(),
                            install_set: o.install_set.clone(),
                            diagnostics: o.diagnostics.clone(),
                            non_fast_path_reason: o.non_fast_path_reason.clone(),
                            worker_edit_us,
                            worker_edit_to_result_us: total,
                            counters: s.counters().snapshot(),
                            worker_peak_bytes: peak.current_peak(),
                            // B3-C03: value AND identities from the
                            // SAME run's record — the executor's trace
                            // evidence, never a wrapper's view.
                            value,
                            dispatched: dispatched_record
                                .into_iter()
                                .map(|(name, ptr, generation, content, resident)| {
                                    DispatchedIdentity {
                                        name,
                                        ptr,
                                        generation,
                                        content,
                                        resident,
                                    }
                                })
                                .collect(),
                            phase_walls: s.phase_walls().clone(),
                            phase_io: s.phase_io().clone(),
                            store_attached: s.has_store(),
                            run_seq,
                            av_seq: auto_av_core::av::execution_seq(),
                        }
                    }
                    Err(e) => {
                        eprintln!("session-serve: edit infrastructure failure: {e}");
                        std::process::exit(4);
                    }
                };
                writeln!(
                    out,
                    "{}",
                    serde_json::to_string(&WorkerReply::EditResult(msg)).unwrap()
                )
                .unwrap();
                out.flush().unwrap();
            }
            WorkerRequest::Close(_) => {
                if let Some(s) = session.as_mut() {
                    s.close();
                }
                writeln!(
                    out,
                    "{}",
                    serde_json::to_string(&WorkerReply::Closed(ClosedMsg {})).unwrap()
                )
                .unwrap();
                out.flush().unwrap();
                break;
            }
        }
    }
    exit(0)
}

// ---------------------------------------------------------------------------
// PLAN-820: data-profile module restore + run (new-worker seam)
// ---------------------------------------------------------------------------

fn data_restore_run(args: &[String]) -> ! {
    let mut store: Option<PathBuf> = None;
    let mut owner = String::new();
    let mut entry = String::new();
    let mut snapshot: Option<u64> = None;
    let mut types_fp = String::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--store" => store = Some(PathBuf::from(&args[i + 1])),
            "--owner" => owner = args[i + 1].clone(),
            "--entry" => entry = args[i + 1].clone(),
            "--snapshot" => {
                snapshot = Some(
                    u64::from_str_radix(args[i + 1].trim_start_matches("0x"), 16)
                        .expect("hex snapshot"),
                )
            }
            "--types-fp" => types_fp = args[i + 1].clone(),
            other => {
                eprintln!("data-restore-run: unknown flag `{}`", other);
                exit(2);
            }
        }
        i += 2;
    }
    let (Some(store), Some(snapshot)) = (store, snapshot) else {
        eprintln!(
            "data-restore-run: --store/--owner/--entry/--snapshot/--types-fp are all required"
        );
        exit(2);
    };
    match auto_av_probe::data_driver::restore_data(&store, &owner, &entry, snapshot, &types_fp) {
        Ok(out) => {
            println!("{:?}", out);
            match out {
                auto_av_core::contract::ExecOutcome::Success { .. } => exit(0),
                _ => exit(3),
            }
        }
        Err(e) => {
            eprintln!("data-restore-run failed: {}", e);
            exit(1);
        }
    }
}
