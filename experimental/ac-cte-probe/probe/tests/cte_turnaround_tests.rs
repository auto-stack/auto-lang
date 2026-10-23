//! cte_turnaround_ — the turnaround tier (PLAN-817 T-07, V-09): the
//! bench-turnaround worker runs six scenarios x >=30 raw samples with
//! the two EDIT hard gates (helper body + Meta set, warm store,
//! P95 < 1000ms), every row bound to its input hash; the gate-ms-0 and
//! injected-spike controls turn the SAME machinery deterministically
//! red. Independent expectations: 42+pad and 2+extra_fields.

use std::process::{Command, Stdio};

fn run_bench(args: &[&str]) -> (i32, serde_json::Value, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
        .arg("bench-turnaround")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("bench spawns");
    let code = out.status.code().unwrap_or(-1);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let summary = serde_json::from_str(text.trim()).unwrap_or(serde_json::Value::Null);
    (code, summary, text)
}

#[test]
fn cte_turnaround_six_scenarios_hard_gates_pass() {
    let tmp = tempfile::tempdir().unwrap();
    let raw = tmp.path().join("raw.jsonl");
    // The formal tier is RELEASE (798 discipline): debug runs the same
    // machinery at smoke scale under a smoke gate; the 1000ms hard gate
    // is asserted in release (and by the verify script's formal bench).
    let samples = if cfg!(debug_assertions) { "3" } else { "30" };
    let gate = if cfg!(debug_assertions) {
        "5000"
    } else {
        "1000"
    };
    let want_samples: u64 = if cfg!(debug_assertions) { 3 } else { 30 };
    let (code, s, _) = run_bench(&[
        "--samples",
        samples,
        "--gate-ms",
        gate,
        "--out",
        raw.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "gate green: {s}");
    assert_eq!(s["target_met"], serde_json::json!(true));
    for sc in [
        "fresh",
        "hot",
        "restart",
        "body-edit",
        "meta-edit",
        "gc",
        "worker-edit",
    ] {
        assert_eq!(s[sc]["samples"], want_samples, "{sc} sample count");
        assert_eq!(s[sc]["all_values_ok"], true, "{sc} values correct");
        assert!(s[sc]["p95_ms"].as_f64().unwrap() > 0.0, "{sc} measured");
    }
    // The two EDIT hard gates specifically.
    let body_gate = if cfg!(debug_assertions) {
        5000.0
    } else {
        1000.0
    };
    assert!(
        s["body-edit"]["p95_ms"].as_f64().unwrap() < body_gate,
        "helper body edit P95 < {body_gate}ms (formal 1000ms gate is the release tier): {}",
        s["body-edit"]
    );
    assert!(
        s["meta-edit"]["p95_ms"].as_f64().unwrap() < 1000.0,
        "Meta set edit P95 < 1000ms: {}",
        s["meta-edit"]
    );
    // Raw report: >= 6*30 rows + warmups + summary line, every row with
    // an input hash; edit scenarios carry FRESH hashes per sample.
    let text = std::fs::read_to_string(&raw).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines.len() >= 7 * want_samples as usize + 1,
        "raw rows present: {}",
        lines.len()
    );
    let mut body_hashes = std::collections::BTreeSet::new();
    let mut meta_hashes = std::collections::BTreeSet::new();
    let mut rows = 0usize;
    for l in &lines[..lines.len() - 1] {
        let v: serde_json::Value = serde_json::from_str(l).expect("row json");
        rows += 1;
        assert!(v["input_hash"].as_str().is_some(), "row bound to hash");
        if v["scenario"] == "body-edit" && v["sample"].is_u64() {
            body_hashes.insert(v["input_hash"].as_str().unwrap().to_string());
        }
        if v["scenario"] == "meta-edit" && v["sample"].is_u64() {
            meta_hashes.insert(v["input_hash"].as_str().unwrap().to_string());
        }
    }
    assert_eq!(
        body_hashes.len() as u64,
        want_samples,
        "every body-edit sample is a distinct input"
    );
    assert_eq!(
        meta_hashes.len() as u64,
        want_samples,
        "every meta-edit sample is a distinct input"
    );
    let last: serde_json::Value = serde_json::from_str(lines[lines.len() - 1]).unwrap();
    assert_eq!(last["target_met"], serde_json::json!(true), "summary row");
    let _ = rows;
}

#[test]
fn cte_turnaround_gate_ms_zero_control_is_red() {
    let (code, s, _) = run_bench(&["--samples", "3", "--gate-ms", "0"]);
    assert_ne!(code, 0, "gate-ms 0 always fails");
    assert_eq!(s["target_met"], serde_json::json!(false));
    assert!(
        s["hard_gate_failures"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f.as_str().unwrap_or_default().contains("counter-control")),
        "the failure names the control: {s}"
    );
}

#[test]
fn cte_turnaround_injected_spike_control_is_red() {
    // A deterministic 1500ms spike on the first TWO body-edit samples drives
    // its P95 over the gate — the SAME gate that was green.
    let (code, s, _) = run_bench(&["--samples", "30", "--inject-delay-ms", "1500"]);
    assert_ne!(code, 0, "the spike turns the gate red");
    assert_eq!(s["target_met"], serde_json::json!(false));
    assert!(
        s["body-edit"]["p95_ms"].as_f64().unwrap() >= 1500.0,
        "the spike is visible in the P95: {}",
        s["body-edit"]
    );
}

/// F-817-R1-04 closure: the malformed-input gates (0 / negative / NaN /
/// Inf / negative sample counts) all exit non-zero with target_met=false
/// — never a green empty run.
#[test]
fn cte_turnaround_malformed_inputs_all_red() {
    for (name, args) in [
        ("zero-samples", vec!["--samples", "0"]),
        ("negative-samples", vec!["--samples", "-3"]),
        ("nan-gate", vec!["--samples", "3", "--gate-ms", "NaN"]),
        ("inf-gate", vec!["--samples", "3", "--gate-ms", "inf"]),
        ("neg-gate", vec!["--samples", "3", "--gate-ms", "-5"]),
    ] {
        let (code, s, _) = run_bench(&args);
        assert_ne!(code, 0, "{name} exits non-zero");
        assert_eq!(
            s["target_met"],
            serde_json::json!(false),
            "{name} not green"
        );
        assert!(
            s["hard_gate_failures"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f.as_str().unwrap_or_default().contains("invalid input")),
            "{name} names the invalid input: {s}"
        );
    }
}
