//! Probe node CLI: the cross-process actor for integration tests and the
//! perf harness (T-04/T-05/T-06). Every subcommand prints one JSON line
//! (`KEY ` prefix) so tests consume structured results.

use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::identity::{SourceSnapshotId, SUPPORTED_TARGET};
use ac_artifact_probe::store::{gc_config_default, recover, run_gc, ManagedStore};
use auto_ac_prototype::source::SourceProfile;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().cloned().unwrap_or_default();
    let rest = &args[1..];
    let code = match cmd.as_str() {
        "vertical" => cmd_vertical(rest),
        "crash" => cmd_crash(rest),
        "gc" => cmd_gc(rest),
        "churn" => cmd_churn(rest),
        "lease" => cmd_lease(rest),
        "space" => cmd_space(rest),
        "unroot" => cmd_unroot(rest),
        "recover" => cmd_recover(rest),
        "perf" => cmd_perf(rest),
        _ => {
            eprintln!("usage: node <vertical|crash|gc|churn|lease|space|recover|perf> [args]");
            2
        }
    };
    std::process::exit(code);
}

struct Args<'a> {
    rest: &'a [String],
}

impl<'a> Args<'a> {
    fn flag(&self, name: &str) -> Option<String> {
        let mut it = self.rest.iter();
        while let Some(a) = it.next() {
            if a == name {
                return it.next().cloned();
            }
        }
        None
    }

    /// Valueless boolean flag.
    fn has(&self, name: &str) -> bool {
        self.rest.iter().any(|a| a == name)
    }
}

fn emit(key: &str, v: impl serde::Serialize) {
    println!("{} {}", key, serde_json::to_string(&v).unwrap());
}

fn open_store(a: &Args) -> ManagedStore {
    let root = a.flag("--store").expect("--store");
    ManagedStore::open(PathBuf::from(&root).as_path()).expect("open store")
}

fn cfg_from(a: &Args) -> VerticalCfg {
    let source = a.flag("--source").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("main.at"));
    let mut cfg = VerticalCfg {
        project: a.flag("--project").unwrap_or_else(|| "proj".into()),
        source,
        snapshot_override: None,
        logical_name: a.flag("--name").unwrap_or_else(|| "main".into()),
        profile: match a.flag("--profile").as_deref() {
            Some("scalar") => SourceProfile::Scalar,
            _ => SourceProfile::Core,
        },
        target: a.flag("--target").unwrap_or_else(|| SUPPORTED_TARGET.into()),
        entry: a.flag("--entry").unwrap_or_else(|| "main".into()),
        toolchain_fp: a.flag("--toolchain-fp").unwrap_or_else(|| "default-lld".into()),
        work_dir: a.flag("--work").map(PathBuf::from).unwrap_or_else(std::env::temp_dir),
        ..Default::default()
    };
    if let Some(snap) = a.flag("--snapshot-id") {
        cfg.snapshot_override = Some(SourceSnapshotId(snap));
    }
    cfg
}

fn cmd_vertical(rest: &[String]) -> i32 {
    let a = Args { rest };
    let store = open_store(&a);
    let cfg = cfg_from(&a);
    match vertical_and_run(&store, &cfg) {
        Ok(out) => {
            emit("RESULT", &out);
            0
        }
        Err(e) => {
            emit("ERROR", &serde_json::json!({ "error": e.to_string() }));
            1
        }
    }
}

fn cmd_crash(rest: &[String]) -> i32 {
    let a = Args { rest };
    let at = a.flag("--at").expect("--at after-staging|after-chunk-move|after-tombstone");
    std::env::set_var("AC_PROBE_FAULT", &at);
    let store = open_store(&a);
    if at == "after-tombstone" {
        // Crash inside GC's tombstone window: one real build, then make it
        // unreachable and run GC with a hair-trigger watermark so the very
        // first candidate tombstone is the fault point.
        let cfg = cfg_from(&a);
        vertical_and_run(&store, &cfg).expect("junk build");
        store.remove_project_roots(&cfg.project).expect("roots");
        let r = run_gc(&store, ac_artifact_probe::store::GcConfig { high: 1, low: 0 }).expect("gc");
        emit("GC", &r);
        unreachable!("AC_PROBE_FAULT must abort inside run_gc");
    }
    let cfg = cfg_from(&a);
    match vertical_and_run(&store, &cfg) {
        Ok(_) => unreachable!("AC_PROBE_FAULT must abort"),
        Err(e) => {
            eprintln!("unexpected error before fault: {}", e);
            1
        }
    }
}

fn cmd_gc(rest: &[String]) -> i32 {
    let a = Args { rest };
    let store = open_store(&a);
    let mut cfg = gc_config_default();
    if let Some(h) = a.flag("--high").and_then(|v| v.parse().ok()) {
        cfg.high = h;
    }
    if let Some(l) = a.flag("--low").and_then(|v| v.parse().ok()) {
        cfg.low = l;
    }
    match run_gc(&store, cfg) {
        Ok(r) => {
            emit("GC", &r);
            0
        }
        Err(e) => {
            emit("ERROR", &serde_json::json!({ "error": e.to_string() }));
            1
        }
    }
}

fn cmd_churn(rest: &[String]) -> i32 {
    let a = Args { rest };
    let store = open_store(&a);
    let iterations: u64 = a.flag("--iterations").and_then(|v| v.parse().ok()).unwrap_or(32);
    let ops: u64 = a.flag("--ops").and_then(|v| v.parse().ok()).unwrap_or(300);
    let gc_every: u64 = a.flag("--gc-every").and_then(|v| v.parse().ok()).unwrap_or(8);
    let work = a.flag("--work").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&work).expect("create work dir");
    let projects = 3;
    let mut gc_runs = 0u64;
    let mut peak_total = 0u64;
    let mut gc_backoffs = 0u64;
    let t0 = std::time::Instant::now();
    for i in 0..iterations {
        let project = format!("churn-{}", i % projects);
        // Real .at source: an add chain whose depth and constant vary per
        // version (body-only changes; signature constant).
        let constant = 1 + (i % 7);
        let body: String = std::iter::repeat("1 + ").take(ops as usize).collect();
        let src = format!("fn main() int {{ return {} + {} }}", constant, body.trim_end_matches(" + "));
        let src_path = work.join(format!("churn-{}.at", i));
        std::fs::write(&src_path, &src).unwrap();
        let cfg = VerticalCfg {
            project: project.clone(),
            source: src_path,
            work_dir: work.join(format!("churn-w-{}", i % projects)),
            ..Default::default()
        };
        match vertical_and_run(&store, &cfg) {
            Ok(out) => {
                // Semantic oracle: the chain returns constant + ops × 1.
                let expected = (constant as i64 + ops as i64) as i32;
                if out.exit_code != Some(expected) {
                    emit("ERROR", &serde_json::json!({ "iteration": i, "exit": out.exit_code, "expected": expected }));
                    return 1;
                }
            }
            Err(e) => {
                emit("ERROR", &serde_json::json!({ "iteration": i, "error": e.to_string() }));
                return 1;
            }
        }
        if (i + 1) % gc_every == 0 {
            let r = run_gc(&store, gc_config_default()).expect("gc");
            gc_runs += 1;
            if r.backoff.is_some() {
                gc_backoffs += 1;
            }
        }
        peak_total = peak_total.max(store.space().total);
    }
    let final_space = store.space();
    emit(
        "CHURN",
        &serde_json::json!({
            "iterations": iterations,
            "ops": ops,
            "gc_runs": gc_runs,
            "gc_backoffs": gc_backoffs,
            "peak_total": peak_total,
            "final_total": final_space.total,
            "blocks_count": final_space.blocks_count,
            "duration_ms": t0.elapsed().as_millis() as u64,
        }),
    );
    0
}

fn cmd_lease(rest: &[String]) -> i32 {
    let a = Args { rest };
    let store = open_store(&a);
    let purpose = a.flag("--purpose").unwrap_or_else(|| "test".into());
    let hold_ms: u64 = a.flag("--hold-ms").and_then(|v| v.parse().ok()).unwrap_or(1000);
    let artifact = a.flag("--artifact");
    let lease = store
        .acquire_lease(&purpose, 60, artifact.as_deref())
        .expect("lease");
    let lease_id = lease.lease_id.clone();
    if let Some(id_file) = a.flag("--id-file") {
        std::fs::write(&id_file, &lease_id).expect("write id file");
    }
    emit("LEASE", &serde_json::json!({ "lease_id": lease_id, "purpose": purpose }));
    std::thread::sleep(Duration::from_millis(hold_ms));
    if a.has("--crash") {
        // Simulate a crashed holder: abort WITHOUT releasing — the OS lock
        // dies with the process, the row stays unreleased (sweep test).
        std::process::abort();
    }
    let live = store.lease_live(&lease_id).expect("liveness");
    lease.release();
    let live_after = store.lease_live(&lease_id).unwrap_or(false);
    emit("LEASE-END", &serde_json::json!({ "lease_id": lease_id, "live_during": live, "live_after": live_after }));
    0
}

fn cmd_space(rest: &[String]) -> i32 {
    let a = Args { rest };
    let store = open_store(&a);
    emit("SPACE", &store.space());
    0
}

fn cmd_unroot(rest: &[String]) -> i32 {
    let a = Args { rest };
    let store = open_store(&a);
    let project = a.flag("--project").expect("--project");
    match store.remove_project_roots(&project) {
        Ok(n) => {
            emit("UNROOT", &serde_json::json!({ "project": project, "removed": n }));
            0
        }
        Err(e) => {
            emit("ERROR", &serde_json::json!({ "error": e.to_string() }));
            1
        }
    }
}

fn cmd_recover(rest: &[String]) -> i32 {
    let a = Args { rest };
    let store = open_store(&a);
    let exclusive = a.flag("--exclusive").is_some();
    match recover(&store, exclusive) {
        Ok(r) => {
            emit("RECOVER", &r);
            0
        }
        Err(e) => {
            emit("ERROR", &serde_json::json!({ "error": e.to_string() }));
            1
        }
    }
}

fn cmd_perf(rest: &[String]) -> i32 {
    let a = Args { rest };
    let out_path = a.flag("--out").expect("--out");
    let samples: usize = a.flag("--samples").and_then(|v| v.parse().ok()).unwrap_or(3);
    let root = a.flag("--root").map(PathBuf::from).unwrap_or_else(|| std::env::temp_dir().join("ac783-perf"));
    let fixture = a.flag("--fixture").unwrap_or_else(|| "p04-while-retest.at".into());
    let report = ac_artifact_probe::perf::run_scenarios(&root, &fixture, samples);
    let ok = std::fs::write(&out_path, serde_json::to_vec_pretty(&report).unwrap()).is_ok();
    emit("PERF", &serde_json::json!({ "written": ok, "out": out_path, "samples_per_scenario": samples }));
    if ok {
        0
    } else {
        1
    }
}

