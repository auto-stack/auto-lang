//! PLAN-796 T-05 turnaround tests (`sema_turnaround_` filter, V-07):
//! stage traces from the first case, the six measured scenarios with
//! RAW samples (all statuses kept), and the §5.5 hard gate — a single
//! function body edit on the 1 MiB reference module must go
//! request → current correct inference published at P95 < 1000 ms.

#![cfg(windows)]

use ac_sema_probe::turnaround::{stats_of, Harness, HARD_GATE, SAMPLES};
use ac_sema_probe::open_store;

fn harness(family: &str) -> (Harness, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("ac796-{}", family));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    (Harness::new(root.join("store")), root)
}

/// The reference load is a real ~1 MiB module with a real function
/// population (tool preparation, excluded from measured windows).
#[test]
fn sema_turnaround_reference_load_shape() {
    let src = ac_sema_probe::turnaround::reference_module(1024 * 1024);
    assert!(
        src.len() >= 1024 * 1024,
        "reference module must reach 1MiB, got {}",
        src.len()
    );
    let input = ac_sema_probe::turnaround::reference_input(&src);
    assert!(input.fns.len() > 1000, "substantial fn population, got {}", input.fns.len());
    // deterministic: same generator call → identical bytes
    let src2 = ac_sema_probe::turnaround::reference_module(1024 * 1024);
    assert_eq!(src, src2);
}

/// Every sample carries the full stage trace (request / cache read +
/// validation / infer / publish) and its status — traces exist from
/// the first case, nesting never double-counted (stages are disjoint).
#[test]
fn sema_turnaround_stage_traces_present() {
    let (h, _root) = harness("turnaround-trace");
    let store = open_store(&h.store_root).unwrap();
    let samples = h.scenario_local_body_edit(&store);
    for s in &samples {
        assert!(s.total_ns > 0);
        let names: Vec<&str> = s.stages.iter().map(|st| st.name).collect();
        assert!(names.contains(&"request"), "request stage in {:?}", names);
        assert!(names.contains(&"cache-read-validate"), "cache stage in {:?}", names);
        // every sample either hit or published — with a real status
        assert!(
            s.status == "published" || s.status == "hit",
            "status {}",
            s.status
        );
        if s.status == "published" {
            assert!(names.contains(&"infer"), "infer stage in {:?}", names);
            assert!(names.contains(&"publish"), "publish stage in {:?}", names);
            assert_eq!(s.compute_calls, 1, "published means exactly one real compute");
        }
        if s.status == "hit" {
            assert_eq!(s.compute_calls, 0, "hit never computes");
        }
    }
}

/// THE HARD GATE (§5.5): local body edits on the 1 MiB reference
/// module — P95 < 1000 ms over 30 raw samples.
#[test]
fn sema_turnaround_hard_gate_local_body_edit_p95() {
    let (h, _root) = harness("turnaround-gate");
    let store = open_store(&h.store_root).unwrap();
    let samples = h.scenario_local_body_edit(&store);
    assert_eq!(samples.len(), SAMPLES);
    let stats = stats_of("local-body-edit", &samples);
    assert!(
        stats.p95_ns < HARD_GATE.as_nanos(),
        "P95 {:.2}ms exceeds the {}ms gate",
        stats.p95_ns as f64 / 1e6,
        HARD_GATE.as_millis()
    );
    assert!(stats.compute_calls >= SAMPLES as u64, "each sample performed a real edit+recompute");
}

/// All six scenarios run with 30 raw samples each; normal samples
/// dominate and every sample is retained (statuses counted, not
/// dropped).
#[test]
fn sema_turnaround_six_scenarios_30_samples() {
    let (h, _root) = harness("turnaround-six");
    let store = open_store(&h.store_root).unwrap();
    let mut all: Vec<(&'static str, Vec<ac_sema_probe::turnaround::Sample>)> = Vec::new();
    all.push(("no-op", h.scenario_no_op(&store)));
    all.push(("local-body-edit", h.scenario_local_body_edit(&store)));
    all.push(("sig-change", h.scenario_sig_change(&store)));
    all.push(("restart-disk-hot", h.scenario_restart_disk_hot()));
    all.push(("cross-project", h.scenario_cross_project(&store)));
    all.push(("gc-concurrent", h.scenario_gc_concurrent(&store)));
    for (name, samples) in &all {
        assert_eq!(samples.len(), SAMPLES, "{name}: 30 raw samples");
        let stats = stats_of(name, samples);
        // sample count == status count (nothing dropped)
        let counted: usize = stats.statuses.iter().map(|(_, n)| n).sum();
        assert_eq!(counted, samples.len(), "{name}: all samples accounted");
        // no infrastructure failures in these healthy scenarios
        for (st, _) in &stats.statuses {
            assert!(
                *st == "hit" || *st == "published",
                "{name}: unexpected status {st}"
            );
        }
    }
    // no-op turns into hits after the first sample
    let stats = stats_of("no-op", &all[0].1);
    assert!(stats.cache_hits >= SAMPLES as u64 - 1, "no-op hits after warm-up");
    // restart-disk-hot: every sample is a disk hit through a FRESH store
    // handle (cross-process persistence itself is proven in
    // sema_cache_first_case with real child processes)
    let stats = stats_of("restart-disk-hot", &all[3].1);
    assert_eq!(stats.cache_hits, SAMPLES as u64, "disk-hot restarts all hit");
}

/// The report writer emits machine info, per-scenario stats and RAW
/// samples (JSON) plus the markdown summary with the hard-gate
/// verdict; tool-prep notes are separate, not folded into samples.
#[test]
fn sema_turnaround_report_written() {
    let (h, root) = harness("turnaround-report");
    let store = open_store(&h.store_root).unwrap();
    let all = vec![
        ("local-body-edit", h.scenario_local_body_edit(&store)),
    ];
    let out = root.join("report");
    let machine = ac_sema_probe::turnaround::machine_info();
    let path = ac_sema_probe::turnaround::write_report(&out, &machine, &all).unwrap();
    let md = std::fs::read_to_string(&path).unwrap();
    assert!(md.contains("硬门判定"));
    assert!(md.contains("local-body-edit"));
    assert!(md.contains("PASS") || md.contains("FAIL"));
    let json = std::fs::read_to_string(out.join("turnaround.json")).unwrap();
    assert!(json.contains("\"raw\""));
    assert!(json.contains("\"stages\""));
}
