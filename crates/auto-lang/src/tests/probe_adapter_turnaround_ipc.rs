//! PLAN-796 F-796-R4-10: the turnaround hard gate over the REAL user
//! path — actual adapter/IPC: each sample runs a case through the real
//! `sema::run_check` (worker process spawn, JSON-lines IPC, parent-side
//! admission and judging), 30 raw samples, P95 < 1000ms, with the
//! stage spans (request/cache/infer/publish) recorded per sample. The
//! report lands in `docs/reports/plan796/turnaround-ipc.json`.

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use auto_test_case::admission::Limits;
use auto_test_case::report::Status;

use crate::test_cases::run_suite;

/// Serialized with the fault tests (they hold process-global state):
/// the same lock guards every sema-worker-spawning test.
pub(crate) use super::probe_adapter_tests::FAULT_LOCK as IPC_LOCK;

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-ipc-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// One sample = one full case execution through the unified runner
/// (real worker process, real IPC, real parent-side judging).
fn sample_ms(root: &Path, idx: usize) -> (f64, Status, Vec<(String, u64)>) {
    let doc = format!(
        "# ipc sample {idx}\n```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"ipc{idx}\", include: [\"c{idx}.case.ad\"] }}\n```\n\n```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ipc.case\",\n  source: {{ text: \"fn main() int {{ let x = {idx}\\nreturn x + 1 }}\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [ {{ id: \"q\", adapter: \"probe\", action: \"run\",\n    options: {{ recipe: \"sema/query-v1\", sema: {{ package: \"ipc\", module: \"m{idx}\", kind: \"infer-body\", fn: \"main\" }}, steps: [ {{ op: \"query\" }} ] }},\n    expect: {{ completion: \"success\", probe: [ {{ cache: \"published\", compute_calls: 1 }} ] }} }} ] }}\n```\n"
    );
    std::fs::write(root.join(format!("s{idx}.suite.ad")), doc.clone()).unwrap();
    std::fs::write(root.join(format!("c{idx}.case.ad")), doc).unwrap();
    let t = Instant::now();
    let out_dir = root.parent().unwrap().join(format!(
        "atc-ipc-gate-out-{}-{idx}",
        root.file_name().unwrap().to_string_lossy()
    ));
    let report = run_suite(
        &root.join(format!("s{idx}.suite.ad")),
        &out_dir,
        &Limits::defaults(),
    )
    .expect("suite runs");
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let row = report.results.first().expect("one check");
    let spans = row
        .native
        .as_ref()
        .map(|n| n.stage_spans.clone())
        .unwrap_or_default();
    (ms, row.status, spans)
}

fn pct(mut v: Vec<f64>, p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[idx.min(v.len() - 1)]
}

/// The hard gate over the REAL adapter/IPC path: 30 raw samples, every
/// one must PASS (a red sample is a failure, not a timing footnote),
/// P95 < 1000ms, and each sample's stage spans are recorded.
#[test]
fn probe_adapter_turnaround_ipc_hard_gate_p95() {
    let _guard = IPC_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = temp_root("gate");
    let mut samples: Vec<(f64, Status, Vec<(String, u64)>)> = Vec::new();
    for i in 0..30usize {
        samples.push(sample_ms(&root, i));
    }
    let times: Vec<f64> = samples.iter().map(|(ms, _, _)| *ms).collect();
    for (i, (_, status, _)) in samples.iter().enumerate() {
        assert_eq!(
            *status,
            Status::Pass,
            "sample {i} must pass on the real path"
        );
    }
    let p95 = pct(times.clone(), 0.95);
    assert!(
        p95 < 1000.0,
        "real-path P95 {p95:.2}ms exceeds the 1000ms gate"
    );
    // every sample carries the real stage spans (request/infer/publish)
    for (i, (_, _, spans)) in samples.iter().enumerate() {
        let names: Vec<&str> = spans.iter().map(|(n, _)| n.as_str()).collect();
        for want in ["request", "cache-read-validate", "infer", "publish"] {
            assert!(
                names.contains(&want),
                "sample {i} stage spans missing {want} (has {names:?})"
            );
        }
    }
    // write the committed report (raw samples, never filtered)
    let report = serde_json::json!({
        "path": "real adapter/IPC (sema::run_check: spawn+JSON-lines+parent judging)",
        "samples": samples.iter().map(|(ms, s, spans)| serde_json::json!({
            "total_ms": ms, "status": format!("{s:?}"),
            "stages": spans,
        })).collect::<Vec<_>>(),
        "p50_ms": pct(times.clone(), 0.50),
        "p95_ms": pct(times.clone(), 0.95),
        "p99_ms": pct(times.clone(), 0.99),
        "hard_gate_ms": 1000,
        "verdict": if p95 < 1000.0 { "PASS" } else { "FAIL" },
    });
    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/reports/plan796/turnaround-ipc.json");
    std::fs::write(&out, serde_json::to_string_pretty(&report).unwrap()).expect("report write");
}
