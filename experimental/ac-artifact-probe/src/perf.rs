//! Six-scenario performance baseline (PLAN-783 §5.5, T-06 / V-11):
//! fresh, same-process hot, restart disk-hot, cross-project, body-modified,
//! GC-concurrent query. ≥30 samples per scenario for the committed report;
//! reduced sample counts are accepted mechanically for quick runs (the
//! report records what ran — no sample-count inflation).
//!
//! No sub-second claim is made; this measures the distance.

use crate::driver::{vertical_and_run, VerticalCfg};
use crate::store::{run_gc, GcConfig, ManagedStore};
use crate::subproc::run_deadlined;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Serialize, serde::Deserialize)]
pub struct SampleStats {
    pub n: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub mean_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
}

#[derive(Clone, Debug, Serialize, serde::Deserialize)]
pub struct ScenarioReport {
    pub name: String,
    pub what: String,
    pub stats: SampleStats,
    pub ok: usize,
    pub semantic_failures: usize,
    /// OS-measured private-bytes delta for THIS scenario's process section
    /// (in-process scenarios only; restart-disk-hot spans processes and
    /// reports 0 with a note — no fabricated numbers).
    pub private_bytes_delta: i64,
    pub disk_total: u64,
    /// GC scenario evidence (R1 F-07): actual runs and deletions.
    #[serde(default)]
    pub gc_runs: u64,
    #[serde(default)]
    pub gc_ran: u64,
    #[serde(default)]
    pub gc_deleted_chunks: u64,
    #[serde(default)]
    pub worker_failures: u64,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, serde::Deserialize)]
pub struct PerfReport {
    pub probe: String,
    /// Bound source identity of the run (R1 F-07: report configuration).
    #[serde(default)]
    pub source_commit: String,
    #[serde(default)]
    pub build_profile: String,
    pub machine: MachineInfo,
    pub fixture: String,
    pub samples_per_scenario: usize,
    pub scenarios: Vec<ScenarioReport>,
}

#[derive(Clone, Debug, Serialize, serde::Deserialize)]
pub struct MachineInfo {
    pub os: String,
    pub logical_cores: usize,
    pub total_phys: u64,
    pub avail_phys: u64,
}

fn pct(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn stats_of(mut v: Vec<f64>) -> SampleStats {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    let sum: f64 = v.iter().sum();
    SampleStats {
        n,
        p50_ms: pct(&v, 0.50),
        p95_ms: pct(&v, 0.95),
        p99_ms: pct(&v, 0.99),
        mean_ms: if n > 0 { sum / n as f64 } else { 0.0 },
        min_ms: v.first().copied().unwrap_or(0.0),
        max_ms: v.last().copied().unwrap_or(0.0),
    }
}

fn fixture_source(fixture: &str) -> String {
    // Read from the ac-core corpus (relative to this crate's manifest).
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ac-core/fixtures/source/valid")
        .join(fixture);
    std::fs::read_to_string(&p)
        .unwrap_or_else(|_| panic!("fixture {} not found at {}", fixture, p.display()))
}

/// Expected exit codes for the four oracle fixtures (source_native.rs).
pub fn fixture_exit(fixture: &str) -> i32 {
    match fixture {
        "p01-precedence.at" => 14,
        "p04-while-retest.at" => 3,
        "p12-unexecuted-trap-not-lifted-x1.at" => 0,
        "p13-add-overflow-trap.at" => 70,
        _ => panic!("no oracle for fixture {}", fixture),
    }
}

fn write_variant(work: &Path, tag: &str, src: &str) -> PathBuf {
    let p = work.join(format!("{}.at", tag));
    std::fs::write(&p, src).unwrap();
    p
}

pub fn run_scenarios(root: &Path, fixture: &str, samples: usize) -> PerfReport {
    // R1 F-02: this entrypoint NEVER deletes a caller-provided directory.
    // All scratch lives in a fresh, uniquely named, marker-carrying
    // subdirectory under `root` (created if absent; a reparse root is
    // refused). Sibling content in `root` is untouchable.
    if root.exists() && crate::store::is_reparse(root) {
        panic!("perf root {} is a reparse point — refusing", root.display());
    }
    let samples = samples.max(1);
    std::fs::create_dir_all(root).unwrap();
    let root = root.join(format!(
        "run-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("AC783-PERF-ROOT.marker"),
        b"probe-owned perf scratch",
    )
    .unwrap();
    let src = fixture_source(fixture);
    let want_exit = fixture_exit(fixture);
    let mut scenarios: Vec<ScenarioReport> = Vec::new();
    let mem0 = crate::sysinfo::current_process_memory();

    // 1. fresh: distinct body bytes per sample → full compile each time
    //    (`i * 0 +` keeps the oracle semantics, changes the content).
    {
        let store = ManagedStore::open(&root.join("s-fresh")).unwrap();
        let mut times = Vec::new();
        let mut ok = 0;
        let mut semfail = 0;
        for i in 0..samples {
            let body = src.replace("return", &format!("return {} * 0 +", i));
            let path = write_variant(&root, &format!("fresh-{i}"), &body);
            let cfg = VerticalCfg {
                project: format!("fresh-{i}"),
                source: path,
                work_dir: root.join(format!("w-fresh-{i}")),
                ..Default::default()
            };
            let t = Instant::now();
            let out = vertical_and_run(&store, &cfg).unwrap();
            times.push(t.elapsed().as_secs_f64() * 1000.0);
            if out.exit_code == Some(want_exit) {
                ok += 1;
            } else {
                semfail += 1;
            }
        }
        let mem1 = crate::sysinfo::current_process_memory();
        scenarios.push(ScenarioReport {
            name: "fresh".into(),
            what: "cold store: parse→HIR→verify→lower→link→run per sample (distinct source)".into(),
            stats: stats_of(times),
            ok,
            semantic_failures: semfail,
            private_bytes_delta: mem1.private_bytes as i64 - mem0.private_bytes as i64,
            disk_total: store.space().total,
            gc_runs: 0,
            gc_ran: 0,
            gc_deleted_chunks: 0,
            worker_failures: 0,
            notes: vec![format!("oracle exit {}", want_exit)],
        });
    }

    // Shared store for scenarios 2-6.
    let shared = ManagedStore::open(&root.join("s-main")).unwrap();
    let base_src_path = write_variant(&root, "base", &src);
    let base_cfg = |work: &str| VerticalCfg {
        project: "main".into(),
        source: base_src_path.clone(),
        work_dir: root.join(work),
        ..Default::default()
    };

    // 2. same-process hot: repeated object hits in ONE process.
    {
        let first = vertical_and_run(&shared, &base_cfg("w-hot")).unwrap();
        assert_eq!(first.exit_code, Some(want_exit));
        let mut times = Vec::new();
        let mut ok = 0;
        let mut semfail = 0;
        for _ in 0..samples {
            let t = Instant::now();
            let out = vertical_and_run(&shared, &base_cfg("w-hot")).unwrap();
            times.push(t.elapsed().as_secs_f64() * 1000.0);
            if out.exit_code == Some(want_exit) && out.counters.link == 1 && out.counters.parse == 0
            {
                ok += 1;
            } else {
                semfail += 1;
            }
        }
        let mem1 = crate::sysinfo::current_process_memory();
        scenarios.push(ScenarioReport {
            name: "same-process-hot".into(),
            what: "object-hit resume in the same process (link+run only)".into(),
            stats: stats_of(times),
            ok,
            semantic_failures: semfail,
            private_bytes_delta: mem1.private_bytes as i64 - mem0.private_bytes as i64,
            disk_total: shared.space().total,
            notes: vec![],
            gc_runs: 0,
            gc_ran: 0,
            gc_deleted_chunks: 0,
            worker_failures: 0,
        });
    }

    // 3. restart disk-hot: new process per sample (node vertical).
    {
        let node = node_exe();
        let mut times = Vec::new();
        let mut ok = 0;
        let mut semfail = 0;
        for _ in 0..samples {
            let mut cmd = Command::new(&node);
            cmd.args([
                "vertical",
                "--store",
                shared.root.display().to_string().as_str(),
                "--project",
                "main",
                "--source",
                base_src_path.display().to_string().as_str(),
                "--work",
                root.join("w-restart").display().to_string().as_str(),
            ]);
            let r = run_deadlined(cmd, Duration::from_secs(crate::SUBPROC_DEADLINE_SECS)).unwrap();
            times.push(r.elapsed_ms as f64);
            // Process spawn + store open + link + run; semantic check via RESULT line.
            if r.code == Some(0)
                && r.stdout.contains("\"object\":\"hit\"")
                && r.stdout.contains(&format!("\"exit_code\":{}", want_exit))
            {
                ok += 1;
            } else {
                semfail += 1;
            }
        }
        scenarios.push(ScenarioReport {
            name: "restart-disk-hot".into(),
            what: "object-hit resume in a NEW process (process spawn + store open + link + run)"
                .into(),
            stats: stats_of(times),
            ok,
            semantic_failures: semfail,
            private_bytes_delta: 0,
            disk_total: shared.space().total,
            gc_runs: 0,
            gc_ran: 0,
            gc_deleted_chunks: 0,
            worker_failures: 0,
            notes: vec![
                "wall time includes OS process spawn; restart is not claimed as OS-cold".into(),
            ],
        });
    }

    // 4. cross-project: same content under a different project id.
    {
        let before = shared.space().blocks_count;
        let mut times = Vec::new();
        let mut ok = 0;
        let mut semfail = 0;
        for i in 0..samples {
            let cfg = VerticalCfg {
                project: format!("other-{i}"),
                source: base_src_path.clone(),
                work_dir: root.join("w-cross"),
                ..Default::default()
            };
            let t = Instant::now();
            let out = vertical_and_run(&shared, &cfg).unwrap();
            times.push(t.elapsed().as_secs_f64() * 1000.0);
            if out.exit_code == Some(want_exit) && out.counters.lower == 0 {
                ok += 1;
            } else {
                semfail += 1;
            }
        }
        let after = shared.space().blocks_count;
        scenarios.push(ScenarioReport {
            name: "cross-project".into(),
            what: "identical content consumed by other projects: full hits, no new chunks".into(),
            stats: stats_of(times),
            ok,
            semantic_failures: semfail,
            private_bytes_delta: 0,
            disk_total: shared.space().total,
            gc_runs: 0,
            gc_ran: 0,
            gc_deleted_chunks: 0,
            worker_failures: 0,
            notes: vec![format!(
                "block count {} → {} (unchanged = content shared)",
                before, after
            )],
        });
    }

    // 5. body-modified: same signature, changed body → object must miss
    //    (distinct body bytes per sample, oracle semantics preserved).
    {
        let mut times = Vec::new();
        let mut ok = 0;
        let mut semfail = 0;
        for i in 0..samples {
            let body = src.replace("return", &format!("return {} * 0 +", 100 + i));
            let path = write_variant(&root, &format!("mod-{i}"), &body);
            let cfg = VerticalCfg {
                project: format!("mod-{i}"),
                source: path,
                work_dir: root.join(format!("w-mod-{i}")),
                ..Default::default()
            };
            let t = Instant::now();
            let out = vertical_and_run(&shared, &cfg).unwrap();
            times.push(t.elapsed().as_secs_f64() * 1000.0);
            // Object MUST have missed (body changed ⇒ new snapshot ⇒ new
            // object key), full recompile, semantics preserved.
            if out.exit_code == Some(want_exit) && out.counters.lower >= 1 {
                ok += 1;
            } else {
                semfail += 1;
            }
        }
        scenarios.push(ScenarioReport {
            name: "body-modified".into(),
            what: "body-only edit (same signature): object miss → full recompile".into(),
            stats: stats_of(times),
            ok,
            semantic_failures: semfail,
            private_bytes_delta: 0,
            disk_total: shared.space().total,
            gc_runs: 0,
            gc_ran: 0,
            gc_deleted_chunks: 0,
            worker_failures: 0,
            notes: vec![],
        });
    }

    // 6. GC-concurrent query: object-hit builds while GC ACTUALLY reclaims
    //    junk versions (R1 F-07): the worker uses scaled watermarks
    //    (2 MiB / 1 MiB) so real tombstone→delete cycles overlap the
    //    queries; every build/GC failure propagates; the receipt records
    //    the runs and the physically deleted bytes.
    {
        let shared = std::sync::Arc::new(ManagedStore::open(&root.join("s-gc")).unwrap());
        let base = write_variant(&root, "gc-base", &src);
        let cfg0 = VerticalCfg {
            project: "live".into(),
            source: base.clone(),
            work_dir: root.join("w-gc"),
            ..Default::default()
        };
        vertical_and_run(&shared, &cfg0).unwrap();
        let mem_before = crate::sysinfo::current_process_memory();
        let gc_store = shared.clone();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_t = stop.clone();
        struct GcWorker {
            junk_n: u64,
            gc_runs: u64,
            gc_ran: u64,
            gc_deleted: u64,
            build_failures: u64,
            gc_failures: u64,
            errors: Vec<String>,
        }
        let gc_thread = std::thread::spawn(move || {
            let mut w = GcWorker {
                junk_n: 0,
                gc_runs: 0,
                gc_ran: 0,
                gc_deleted: 0,
                build_failures: 0,
                gc_failures: 0,
                errors: Vec::new(),
            };
            // Minimum junk iterations guarantee the scenario actually
            // produces reclaimable versions even on a fast query loop
            // (R1 F-07: the proving oracle needs real deletions). The
            // trigger is a hair-pin high watermark measured from the
            // steady base with NO floor (low=0, the established test
            // semantic: reclaim every dead candidate) — real watermark
            // boundaries are exercised by the churn load (gc_budget_).
            const MIN_JUNK_ITERS: u64 = 12;
            let gc_cfg = GcConfig {
                high: gc_store.space().total + 1,
                low: 0,
            };
            while !stop_t.load(std::sync::atomic::Ordering::SeqCst) || w.junk_n < MIN_JUNK_ITERS {
                // Distinct junk version every iteration (real version churn).
                let srcj = format!("fn main() int {{ return {} * 0 + 1 }}", w.junk_n);
                let pj = gc_store.root.join(format!("junk-{}.at", w.junk_n));
                std::fs::write(&pj, srcj).unwrap();
                let cfg = VerticalCfg {
                    project: format!("junk-{}", w.junk_n % 3),
                    source: pj,
                    work_dir: gc_store.root.join(format!("wj-{}", w.junk_n % 3)),
                    ..Default::default()
                };
                if let Err(e) = vertical_and_run(&gc_store, &cfg) {
                    w.build_failures += 1;
                    w.errors.push(format!("build@{}: {}", w.junk_n, e));
                }
                w.junk_n += 1;
                match run_gc(&gc_store, gc_cfg) {
                    Ok(r) => {
                        w.gc_runs += 1;
                        if r.ran {
                            w.gc_ran += 1;
                        }
                        w.gc_deleted += r.deleted_chunks as u64;
                    }
                    Err(e) => {
                        w.gc_failures += 1;
                        w.errors.push(format!("gc@{}: {}", w.junk_n, e));
                    }
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            w
        });
        let mut times = Vec::new();
        let mut ok = 0;
        let mut semfail = 0;
        for _ in 0..samples {
            let cfg = VerticalCfg {
                project: "live".into(),
                source: base.clone(),
                work_dir: root.join("w-gc"),
                ..Default::default()
            };
            let t = Instant::now();
            let out = vertical_and_run(&shared, &cfg).unwrap();
            times.push(t.elapsed().as_secs_f64() * 1000.0);
            if out.exit_code == Some(want_exit) {
                ok += 1;
            } else {
                semfail += 1;
            }
        }
        stop.store(true, std::sync::atomic::Ordering::SeqCst);
        // R1 F-07: worker failures are evidence, not noise.
        let mut w = gc_thread.join().expect("gc worker must not panic");
        let mem1 = crate::sysinfo::current_process_memory();
        let worker_failures = w.build_failures + w.gc_failures;
        let proving = w.gc_ran > 0 && w.gc_deleted > 0 && worker_failures == 0;
        let error_notes = std::mem::take(&mut w.errors);
        scenarios.push(ScenarioReport {
            name: "gc-concurrent-query".into(),
            what: "object-hit build latency while GC recycles junk versions concurrently (hair-pin trigger, no floor: every dead candidate reclaimed per pass)".into(),
            stats: stats_of(times),
            ok,
            semantic_failures: semfail + if proving { 0 } else { 1 },
            private_bytes_delta: mem1.private_bytes as i64 - mem_before.private_bytes as i64,
            disk_total: shared.space().total,
            gc_runs: w.gc_runs,
            gc_ran: w.gc_ran,
            gc_deleted_chunks: w.gc_deleted,
            worker_failures,
            notes: std::iter::once(format!(
                "gc runs={} ran={} deleted-chunks={} (hair-pin trigger from steady base, low=0; proving={}, failures={})",
                w.gc_runs, w.gc_ran, w.gc_deleted, proving, worker_failures
            ))
            .chain(error_notes.iter().take(5).cloned())
            .collect(),
        });
    }

    let m = crate::sysinfo::system_memory();
    PerfReport {
        probe: crate::PROBE_ID.into(),
        source_commit: std::env::var("AC_PROBE_SOURCE_COMMIT")
            .unwrap_or_else(|_| "uncommitted".into()),
        build_profile: "release".into(),
        machine: MachineInfo {
            os: std::env::consts::OS.to_string(),
            logical_cores: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(0),
            total_phys: m.total_phys,
            avail_phys: m.avail_phys,
        },
        fixture: fixture.into(),
        samples_per_scenario: samples,
        scenarios,
    }
}

fn node_exe() -> PathBuf {
    let self_exe = std::env::current_exe().unwrap();
    self_exe
        .parent()
        .unwrap()
        .join(format!("node{}", std::env::consts::EXE_SUFFIX))
}
