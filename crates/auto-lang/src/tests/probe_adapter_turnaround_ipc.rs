//! PLAN-796 F-796-R4-10 + F-796-R6-06: the turnaround hard gate over
//! the REAL user path on the FROZEN reference load — actual adapter/IPC:
//! each sample runs a case through the real `sema::run_check` (worker
//! process spawn, JSON-lines IPC, parent-side admission and judging) on
//! a ~1 MiB-equivalent core module (the SAME `gen796-1mi/v1` generator
//! as the library report), performs a LOCAL `main` body edit via the
//! `edit-source` step, and queries again. 30 raw samples, P95 <
//! 1000ms, stage spans per query, statuses kept. The report lands in
//! `docs/reports/plan796/turnaround-ipc.json`.

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

/// The frozen 1 MiB-equivalent reference module (`gen796-1mi/v1`) —
/// byte-identical generator formula to `ac_sema_probe::turnaround::
/// reference_module` (the two reports measure the same load). `main` is
/// appended as the local-edit target.
fn reference_module(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 1024);
    let mut i = 0u32;
    while out.len() < target_bytes {
        let a = i % 977 + 1;
        let b = i % 131 + 1;
        out.push_str(&format!(
            "fn f{i}(x int) int {{ let y{i} = x * {a} + {b}\nvar z{i} = y{i}\nwhile z{i} < {a}00 {{ z{i} = z{i} + {b} }}\nreturn z{i} + x }}\n"
        ));
        i += 1;
    }
    out.push_str("fn main() int { let x = 1\nreturn x }\n");
    out
}

/// One sample = one full case execution through the unified runner
/// (real worker process, real IPC, real parent-side judging) on the
/// reference module with a local body edit. Returns
/// (total_ms, status, all stage spans, second-query stage spans).
fn sample_ms(root: &Path, idx: usize) -> (f64, Status, Vec<(String, u64)>, Vec<(String, u64)>) {
    let src = reference_module(1024 * 1024);
    std::fs::write(root.join("main.at"), &src).unwrap();
    std::fs::create_dir_all(root.join("oracle")).unwrap();
    std::fs::write(
        root.join("oracle/ipc-main.bindings.json"),
        r#"{ "bindings": [ { "name": "x", "ty": "i32" } ] }"#,
    )
    .unwrap();
    let to = format!("let x = {}", idx + 2);
    let doc = format!(
        "# ipc sample {idx}\n```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"ipc{idx}\", include: [\"c{idx}.case.ad\"] }}\n```\n\n```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ipc.case\",\n  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [ {{ id: \"q\", adapter: \"probe\", action: \"run\",\n    options: {{ recipe: \"sema/query-v1\", sema: {{ package: \"ipc\", module: \"m{idx}\", kind: \"infer-body\", fn: \"main\" }}, steps: [ {{ op: \"query\" }}, {{ op: \"edit-source\", from: \"let x = 1\", to: \"{to}\" }}, {{ op: \"query\" }} ] }},\n    expect: {{ completion: \"success\", probe: [ {{ cache: \"published\", compute_calls: 1, bindings_file: \"oracle/ipc-main.bindings.json\" }}, {{ cache: \"published\", compute_calls: 1, bindings_file: \"oracle/ipc-main.bindings.json\" }} ] }} }} ] }}\n```\n"
    );
    std::fs::write(root.join(format!("s{idx}.suite.ad")), doc.clone()).unwrap();
    std::fs::write(root.join(format!("c{idx}.case.ad")), doc).unwrap();
    let t = Instant::now();
    let out_dir = root.parent().unwrap().join(format!(
        "atc-ipc-gate-out-{}-{idx}",
        root.file_name().unwrap().to_string_lossy()
    ));
    // the reference load is deliberately 1 MiB — the default 1 MiB
    // sidecar bound is exclusive, so the harness raises ONLY that bound
    // (well under the frozen hard cap of 4 MiB)
    let limits = Limits {
        max_sidecar_bytes: 2 * 1024 * 1024,
        ..Limits::defaults()
    };
    let report =
        run_suite(&root.join(format!("s{idx}.suite.ad")), &out_dir, &limits).expect("suite runs");
    let ms = t.elapsed().as_secs_f64() * 1000.0;
    let row = report.results.first().expect("one check");
    if row.status != Status::Pass {
        eprintln!("sample {idx} failed: {:?}", row.detail);
    }
    // F-796-R6-05: the worker's SAME session meter actually ran — a
    // `meter=None` regression shows up as a zero peak here.
    let meter_peak = row
        .native
        .as_ref()
        .and_then(|n| {
            n.counters
                .iter()
                .find(|(k, _)| k == "meter_scratch_peak_bytes")
                .map(|(_, v)| *v)
        })
        .unwrap_or(0);
    assert!(
        meter_peak > 0,
        "sample {idx}: the worker session meter must be charged (peak {meter_peak})"
    );
    let spans = row
        .native
        .as_ref()
        .map(|n| n.stage_spans.clone())
        .unwrap_or_default();
    // two query steps × (request, cache-read-validate, infer, publish)
    let edit_spans = if spans.len() >= 8 {
        spans[spans.len() - 4..].to_vec()
    } else {
        Vec::new()
    };
    (ms, row.status, spans, edit_spans)
}

fn pct(mut v: Vec<f64>, p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[idx.min(v.len() - 1)]
}

/// The hard gate over the REAL adapter/IPC path on the frozen 1MiB
/// reference load with a local body edit: 30 raw samples, every one
/// must PASS (a red sample is a failure, not a timing footnote),
/// P95 < 1000ms, and each sample's per-query stage spans are recorded.
#[test]
fn probe_adapter_turnaround_ipc_hard_gate_p95() {
    let _guard = IPC_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = temp_root("gate");
    let mut samples: Vec<(f64, Status, Vec<(String, u64)>, Vec<(String, u64)>)> = Vec::new();
    for i in 0..30usize {
        samples.push(sample_ms(&root, i));
    }
    let times: Vec<f64> = samples.iter().map(|(ms, _, _, _)| *ms).collect();
    for (i, (_, status, _, _)) in samples.iter().enumerate() {
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
    // every sample carries the real stage spans for BOTH queries, and
    // the edit-query group is complete (request/cache/infer/publish)
    for (i, (_, _, spans, edit_spans)) in samples.iter().enumerate() {
        let names: Vec<&str> = spans.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            spans.len() >= 8,
            "sample {i} must carry two query step traces (has {names:?})"
        );
        for want in ["request", "cache-read-validate", "infer", "publish"] {
            assert!(
                names.contains(&want),
                "sample {i} stage spans missing {want} (has {names:?})"
            );
            assert!(
                edit_spans.iter().any(|(n, _)| n == want),
                "sample {i} edit-query stages missing {want} (has {edit_spans:?})"
            );
        }
    }
    // write the committed report (raw samples, never filtered)
    let reference_bytes = reference_module(1024 * 1024).len();
    let report = serde_json::json!({
        "path": "real adapter/IPC (sema::run_check: spawn+JSON-lines+parent judging), 1MiB-equivalent module, local main body edit via edit-source",
        "generator": "gen796-1mi/v1",
        "reference_bytes": reference_bytes,
        "samples": samples.iter().map(|(ms, s, spans, edit_spans)| serde_json::json!({
            "total_ms": ms, "status": format!("{s:?}"),
            "stages": spans, "edit_query_stages": edit_spans,
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
