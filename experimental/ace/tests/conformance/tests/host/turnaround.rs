//! PLAN-796 F-796-R4-10 + F-796-R6-06 + F-796-R10-01: the turnaround
//! hard gate over the REAL user path on the FROZEN reference load —
//! actual adapter/IPC: each sample runs a case through the real
//! `sema::run_check` (worker process spawn, JSON-lines IPC, parent-side
//! admission and judging) on a ~1 MiB-equivalent core module (the SAME
//! `gen796-1mi/v1` generator as the library report), performs a LOCAL
//! `main` body edit via the `edit-source` step, and queries again. 30
//! raw samples, P90 <= 2000ms, stage spans per query, statuses kept. The
//! report lands in `docs/reports/plan796/turnaround-ipc.json`.

#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use auto_test_case::admission::Limits;
use auto_test_case::report::Status;

use crate::test_cases::run_suite;

/// Serialized with the fault tests (they hold process-global state):
/// the same lock guards every sema-worker-spawning test.
pub(crate) use super::semantics_protocol::FAULT_LOCK as IPC_LOCK;

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-ipc-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// The frozen 1 MiB-equivalent reference module (`gen796-1mi/v1`) —
/// byte-identical generator formula to `ace_semantics_store::turnaround::
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

/// One sample's measurements from a full case execution through the
/// unified runner (real worker process, real IPC, real parent-side
/// judging) on the reference module with a local body edit.
struct IpcSample {
    /// whole `run_suite` wall (includes the cold warm-up session)
    total_ms: f64,
    status: Status,
    /// per-query stage spans reported by the worker (both queries)
    spans: Vec<(String, u64)>,
    /// stage spans of the second (edit) query
    edit_spans: Vec<(String, u64)>,
    /// the FULL parent-side wall of the edit session (session 1):
    /// request preparation (steps/nonce/1MiB base64/JSON/hash) + worker
    /// call (spawn+IPC+parse+edit+query+exit) + parent-side admission
    /// until the response's current result is available — the local-edit
    /// request window; session 0 is the cold warm-up reported only in
    /// `total_ms` (F-796-R10-01)
    edit_session_ms: f64,
    /// the three CONSECUTIVE window segments in µs — they tile the
    /// window: prep + worker + admit ≈ wall (F-796-R10-01)
    prep_us: u64,
    worker_us: u64,
    admit_us: u64,
    /// the edit-session wall at µs resolution (ms is derived from it)
    wall_us: u64,
}

fn sample_ms(root: &Path, idx: usize) -> IpcSample {
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
        "# ipc sample {idx}\n```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"ipc{idx}\", include: [\"c{idx}.case.ad\"] }}\n```\n\n```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ipc.case\",\n  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [ {{ id: \"q\", adapter: \"probe\", action: \"run\",\n    options: {{ recipe: \"sema/query-v1\", sema: {{ package: \"ipc\", module: \"m{idx}\", kind: \"infer-body\", fn: \"main\" }}, steps: [ {{ op: \"query\" }}, {{ op: \"restart\" }}, {{ op: \"edit-source\", from: \"let x = 1\", to: \"{to}\" }}, {{ op: \"query\" }} ] }},\n    expect: {{ completion: \"success\", probe: [ {{ cache: \"published\", compute_calls: 1, bindings_file: \"oracle/ipc-main.bindings.json\" }}, {{ cache: \"published\", compute_calls: 1, bindings_file: \"oracle/ipc-main.bindings.json\" }} ] }} }} ] }}\n```\n"
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
    let counter = |name: &str| {
        row.native
            .as_ref()
            .and_then(|n| {
                n.counters
                    .iter()
                    .find(|(k, _)| k == name)
                    .map(|(_, v)| *v as u64)
            })
            .unwrap_or(0)
    };
    let edit_session_ms = counter("session_wall_ms_1") as f64;
    assert!(
        edit_session_ms > 0.0,
        "sample {idx}: the edit session wall must be reported"
    );
    IpcSample {
        total_ms: ms,
        status: row.status.clone(),
        spans,
        edit_spans,
        edit_session_ms,
        prep_us: counter("session_prep_us_1"),
        worker_us: counter("session_worker_us_1"),
        admit_us: counter("session_admit_us_1"),
        wall_us: counter("session_wall_us_1"),
    }
}

/// F-796-R10-01 early/late-stage counter-controls: the three
/// consecutive segments must TILE the reported edit-session wall —
/// every segment measured and positive, request preparation (early) and
/// parent-side admission (late) both INSIDE the window, and no
/// unmeasured gap between segments. A window that starts after request
/// preparation or stops when `call_worker` returns fails here.
fn assert_edit_window_tiles(idx: usize, s: &IpcSample) {
    assert!(
        s.prep_us > 0,
        "sample {idx}: request preparation must be measured ({}µs)",
        s.prep_us
    );
    assert!(
        s.worker_us > 0,
        "sample {idx}: the worker window must be measured ({}µs)",
        s.worker_us
    );
    assert!(
        s.admit_us > 0,
        "sample {idx}: parent-side admission must be measured ({}µs)",
        s.admit_us
    );
    assert!(s.wall_us > 0, "sample {idx}: the wall must be measured");
    // early-stage control: preparation is inside the window
    assert!(
        s.wall_us >= s.prep_us + s.worker_us,
        "sample {idx}: wall {}µs excludes request preparation (prep {}µs + worker {}µs)",
        s.wall_us,
        s.prep_us,
        s.worker_us
    );
    // late-stage control: admission is inside the window, and the three
    // consecutive segments leave no unmeasured gap (only µs truncation
    // and the two inter-segment Instant marks may differ)
    let sum = s.prep_us + s.worker_us + s.admit_us;
    assert!(
        s.wall_us >= sum,
        "sample {idx}: wall {}µs excludes parent-side admission (segments sum {sum}µs)",
        s.wall_us
    );
    assert!(
        s.wall_us - sum <= 100,
        "sample {idx}: wall {}µs carries an unmeasured {}µs gap between segments (sum {sum}µs)",
        s.wall_us,
        s.wall_us - sum
    );
    // the ms gate counter is the SAME window at coarser resolution
    assert_eq!(
        s.edit_session_ms as u64,
        s.wall_us / 1000,
        "sample {idx}: session_wall_ms_1 and session_wall_us_1 disagree"
    );
}

fn pct(mut v: Vec<f64>, p: f64) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[idx.min(v.len() - 1)]
}

/// F-796-R10-01: single-real-sample counter-control for the edit-session
/// window boundary — the prep/worker/admit segments must tile the wall
/// (preparation and parent-side admission both inside the window).
#[test]
fn probe_adapter_edit_session_window_tiles_prep_worker_admit() {
    let _guard = IPC_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = temp_root("window");
    let s = sample_ms(&root, 0);
    assert_eq!(
        s.status,
        Status::Pass,
        "the control sample must pass on the real path"
    );
    assert_edit_window_tiles(0, &s);
}

// Original 30-sample 1MiB IPC timing contract.
mod performance {
    use super::*;

    /// The hard gate over the REAL adapter/IPC path on the frozen 1MiB
    /// reference load with a local body edit: 30 raw samples, every one
    /// must PASS (a red sample is a failure, not a timing footnote),
    /// P90 <= 2000ms over the FULL edit-request window, per-query stage
    /// spans recorded, and every sample's window tiled by its segments.
    #[test]
    fn probe_adapter_turnaround_ipc_hard_gate_p95() {
        let _guard = IPC_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let root = temp_root("gate");
        let mut samples: Vec<IpcSample> = Vec::new();
        for i in 0..30usize {
            samples.push(sample_ms(&root, i));
        }
        let total_times: Vec<f64> = samples.iter().map(|s| s.total_ms).collect();
        let edit_times: Vec<f64> = samples.iter().map(|s| s.edit_session_ms).collect();
        for (i, s) in samples.iter().enumerate() {
            assert_eq!(
                s.status,
                Status::Pass,
                "sample {i} must pass on the real path"
            );
            // F-796-R10-01: per-sample window tiling (early/late controls)
            assert_edit_window_tiles(i, s);
        }
        let p90 = super::super::latency::p90_ms(&edit_times).expect("valid edit windows");
        assert!(
            p90 <= 2000.0,
            "edit-session P90 {p90:.2}ms exceeds the 2000ms gate"
        );
        // every sample carries the real stage spans for BOTH queries, and
        // the edit-query group is complete (request/cache/infer/publish)
        for (i, s) in samples.iter().enumerate() {
            let names: Vec<&str> = s.spans.iter().map(|(n, _)| n.as_str()).collect();
            assert!(
                s.spans.len() >= 8,
                "sample {i} must carry two query step traces (has {names:?})"
            );
            for want in ["request", "cache-read-validate", "infer", "publish"] {
                assert!(
                    names.contains(&want),
                    "sample {i} stage spans missing {want} (has {names:?})"
                );
                assert!(
                    s.edit_spans.iter().any(|(n, _)| n == want),
                    "sample {i} edit-query stages missing {want} (has {:?})",
                    s.edit_spans
                );
            }
        }
        // write the committed report (raw samples, never filtered)
        let reference_bytes = reference_module(1024 * 1024).len();
        let report = serde_json::json!({
            "path": "real adapter/IPC (sema::run_check: request preparation+spawn+JSON-lines+parent admission), 1MiB-equivalent module, local main body edit via edit-source",
            "generator": "gen796-1mi/v1",
            "reference_bytes": reference_bytes,
            "samples": samples.iter().map(|s| serde_json::json!({
                "total_ms": s.total_ms, "edit_session_ms": s.edit_session_ms, "status": format!("{:?}", s.status),
                "stages": s.spans, "edit_query_stages": s.edit_spans,
                "edit_session_prep_us": s.prep_us, "edit_session_worker_us": s.worker_us,
                "edit_session_admit_us": s.admit_us, "edit_session_wall_us": s.wall_us,
            })).collect::<Vec<_>>(),
            "gate": "edit session (request preparation + spawn/IPC + worker execution + parent-side admission), cold warm-up query excluded; prep/worker/admit segments tile the window",
            "p50_ms": pct(edit_times.clone(), 0.50),
            "p90_ms": p90,
            "p95_ms": pct(edit_times.clone(), 0.95),
            "p99_ms": pct(edit_times.clone(), 0.99),
            "total_p95_ms": pct(total_times, 0.95),
            "hard_gate_ms": 2000,
            "verdict": if p90 <= 2000.0 { "PASS" } else { "FAIL" },
        });
        let out = PathBuf::from(env!("AUTO_LANG_HOST_DIR"))
            .join("../../docs/reports/plan796/turnaround-ipc.json");
        std::fs::write(&out, serde_json::to_string_pretty(&report).unwrap()).expect("report write");
    }
}
