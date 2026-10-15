//! V-11 perf tests (§5.5): scenario machinery runs end to end with zero
//! semantic failures; when a full report is supplied (acceptance/closeout),
//! its contract is enforced (six scenarios, ≥ required samples, ordered
//! percentiles). No sub-second claim is asserted.

#![cfg(windows)]

mod common;

use ac_artifact_probe::perf;
use common::scratch;

#[test]
fn perf_scenarios_execute_with_zero_semantic_failures() {
    // Quick machinery run (3 samples per scenario); full-sample runs are
    // produced by the acceptance script / closeout via the node perf bin.
    let root = scratch("perf-quick");
    let report = perf::run_scenarios(&root, "p04-while-retest.at", 3);
    assert_eq!(report.scenarios.len(), 6, "six scenarios");
    let names: Vec<&str> = report.scenarios.iter().map(|s| s.name.as_str()).collect();
    for expected in [
        "fresh",
        "same-process-hot",
        "restart-disk-hot",
        "cross-project",
        "body-modified",
        "gc-concurrent-query",
    ] {
        assert!(
            names.contains(&expected),
            "missing scenario {} in {:?}",
            expected,
            names
        );
    }
    for s in &report.scenarios {
        assert_eq!(s.semantic_failures, 0, "scenario {}: {}", s.name, s.what);
        assert!(s.ok >= 3, "scenario {} ran its samples", s.name);
        assert!(s.stats.p50_ms > 0.0);
        assert!(s.stats.p50_ms <= s.stats.p95_ms);
        assert!(s.stats.p95_ms <= s.stats.p99_ms);
        assert!(s.stats.max_ms >= s.stats.min_ms);
    }
    // R1 F-07: the GC scenario must PROVE real reclamation — GC ran,
    // deleted physical chunks, no worker failures — or it counts as a
    // semantic failure above; here assert the recorded evidence directly.
    let gc = report
        .scenarios
        .iter()
        .find(|s| s.name == "gc-concurrent-query")
        .unwrap();
    assert!(
        gc.gc_ran >= 1,
        "GC ran during the scenario: {}",
        gc.notes.join("; ")
    );
    assert!(
        gc.gc_deleted_chunks > 0 && gc.gc_deleted_during_queries > 0,
        "GC deleted for real: {}",
        gc.notes.join("; ")
    );
    assert_eq!(gc.worker_failures, 0);
}

#[test]
fn perf_full_report_contract_when_provided() {
    // The committed/produced full report (≥30 samples per scenario) is
    // validated when its path is given via AC_PROBE_PERF_REPORT; the
    // acceptance script always sets it, so a missing file there fails.
    let Ok(path) = std::env::var("AC_PROBE_PERF_REPORT") else {
        eprintln!("AC_PROBE_PERF_REPORT not set — skipping contract check (quick machinery test covers execution)");
        return;
    };
    let min_samples: usize = std::env::var("AC_PROBE_PERF_MIN_SAMPLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        panic!(
            "perf report {} missing ({}): run the perf pass first",
            path, e
        )
    });
    let report: perf::PerfReport = serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("perf report not valid JSON: {}", e));
    assert_eq!(report.scenarios.len(), 6);
    for s in &report.scenarios {
        assert!(
            s.stats.n >= min_samples,
            "scenario {} has {} samples < required {}",
            s.name,
            s.stats.n,
            min_samples
        );
        assert_eq!(s.semantic_failures, 0, "scenario {}", s.name);
        assert!(s.stats.p50_ms <= s.stats.p95_ms && s.stats.p95_ms <= s.stats.p99_ms);
    }
    assert!(report.samples_per_scenario >= min_samples);
    assert!(!report.machine.os.is_empty());
    assert!(report.machine.logical_cores > 0);
}
