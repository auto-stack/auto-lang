//! cte_turnaround_ — the turnaround tier (PLAN-817 T-07, V-09): the
//! bench-turnaround worker runs six scenarios x >=30 raw samples with
//! the two EDIT hard gates (helper body + Meta set, warm store,
//! P90 <= 2000ms), every row bound to its input hash; the gate-ms-0 and
//! injected-spike controls turn the SAME machinery deterministically
//! red. Independent expectations: 42+pad and 2+extra_fields.
//!
//! F-817-R5 closures: the REAL wire-mutation matrix (actual mutated
//! worker responses through the shared admission), the child memory
//! transient / tree aggregation / managed overage / sampling
//! unavailability counter-controls — each turns the same formal gates
//! red, and a legal positive control stays green.

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
    // machinery at smoke scale under a smoke gate; the 2000ms hard gate
    // is asserted in release (and by the verify script's formal bench).
    let samples = if cfg!(debug_assertions) { "3" } else { "30" };
    let gate = if cfg!(debug_assertions) {
        "5000"
    } else {
        "2000"
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
        "resident-edit",
    ] {
        assert_eq!(s[sc]["samples"], want_samples, "{sc} sample count");
        assert_eq!(s[sc]["all_values_ok"], true, "{sc} values correct");
        assert!(s[sc]["p95_ms"].as_f64().unwrap() > 0.0, "{sc} measured");
    }
    // The two EDIT hard gates specifically.
    let body_gate = if cfg!(debug_assertions) {
        5000.0
    } else {
        2000.0
    };
    assert!(
        s["body-edit"]["p90_ms"].as_f64().unwrap() <= body_gate,
        "helper body edit P90 <= {body_gate}ms (formal 2000ms gate is the release tier): {}",
        s["body-edit"]
    );
    let meta_gate = if cfg!(debug_assertions) {
        5000.0
    } else {
        2000.0
    };
    assert!(
        s["meta-edit"]["p90_ms"].as_f64().unwrap() <= meta_gate,
        "Meta set edit P90 <= {meta_gate}ms (formal 2000ms gate is the release tier): {}",
        s["meta-edit"]
    );
    // Raw report: >= 6*30 rows + warmups + summary line, every row with
    // an input hash; edit scenarios carry FRESH hashes per sample.
    let text = std::fs::read_to_string(&raw).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        lines.len() >= 8 * want_samples as usize + 1,
        "raw rows present: {}",
        lines.len()
    );
    let mut body_hashes = std::collections::BTreeSet::new();
    let mut meta_hashes = std::collections::BTreeSet::new();
    for l in &lines[..lines.len() - 1] {
        let v: serde_json::Value = serde_json::from_str(l).expect("row json");
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
    // F-817-R2/R1-03: every restart row ran in a DISTINCT real process.
    let mut restart_pids = std::collections::BTreeSet::new();
    for l in &lines[..lines.len() - 1] {
        let v: serde_json::Value = serde_json::from_str(l).expect("row json");
        if v["scenario"] == "restart" && v["sample"].is_u64() {
            let pid = v["pid"].as_u64().expect("process rows carry a pid");
            restart_pids.insert(pid);
        }
    }
    assert_eq!(
        restart_pids.len() as u64,
        want_samples,
        "restart PIDs are all distinct: {restart_pids:?}"
    );
}

// The real worker and its ordinary body-edit oracle exercise both sides
// of the release threshold; injected delays affect only the first two rows.
#[test]
fn cte_turnaround_release_gate_2000_boundary_control() {
    for (delay, should_pass) in [("1100", true), ("2200", false)] {
        let (code, summary, _) = run_bench(&[
            "--samples",
            "10",
            "--gate-ms",
            "2000",
            "--only",
            "body-edit",
            "--inject-delay-ms",
            delay,
        ]);
        assert_eq!(summary["body-edit"]["samples"], 10);
        assert_eq!(summary["body-edit"]["all_values_ok"], true);
        let p90 = summary["body-edit"]["p90_ms"].as_f64().unwrap();
        if should_pass {
            assert!(p90 > 1000.0 && p90 <= 2000.0, "legal measured window {p90}");
            assert_eq!(code, 0, "legal release gate: {summary}");
            assert_eq!(summary["target_met"], true);
        } else {
            assert!(p90 > 2000.0, "out-of-bound measured window {p90}");
            assert_ne!(code, 0, "oversized release gate must fail: {summary}");
            assert_eq!(summary["target_met"], false);
        }
    }
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
    // A deterministic 2500ms spike on four body-edit samples drives
    // its P90 over the gate — the SAME gate that was green.
    let (code, s, _) = run_bench(&["--samples", "30", "--inject-delay-ms", "2500"]);
    assert_ne!(code, 0, "the spike turns the gate red");
    assert_eq!(s["target_met"], serde_json::json!(false));
    assert!(
        s["body-edit"]["p90_ms"].as_f64().unwrap() >= 2500.0,
        "the spike is visible in the P90: {}",
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

/// F-817-R1-02 closure: the resident-edit degrade gate — green in normal
/// incremental traffic (0 re-warms), RED under the forced-rebuild
/// control (exit non-zero, re-warm counted).
#[test]
fn cte_turnaround_resident_degrade_gate_and_control() {
    let samples = if cfg!(debug_assertions) { "3" } else { "10" };
    let gate = if cfg!(debug_assertions) {
        "5000"
    } else {
        "2000"
    };
    let exe = env!("CARGO_BIN_EXE_cte_probe_worker");

    // (a) Normal incremental traffic: green, zero re-warms.
    let (code, s, _) = run_bench(&["--samples", samples, "--gate-ms", gate]);
    assert_eq!(code, 0, "normal resident traffic green: {s}");
    assert_eq!(s["resident_edit_rewarms"], serde_json::json!(0));
    assert!(!serde_json::to_string(&s["hard_gate_failures"])
        .unwrap_or_default()
        .contains("resident-edit degraded"));

    // (b) Forced structural change: the DETECTOR VALUE fires the normal
    // gate (no marker) — the failure names the re-warm count.
    let out = std::process::Command::new(exe)
        .args([
            "bench-turnaround",
            "--samples",
            samples,
            "--gate-ms",
            gate,
            "--inject-rebuild",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("control spawns");
    assert_ne!(out.status.code().unwrap_or(0), 0, "the control is red");
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert!(
        v["resident_edit_rewarms"].as_u64().unwrap_or(0) > 0,
        "the forced rebuild is counted: {}",
        v["resident_edit_rewarms"]
    );
    let failures = serde_json::to_string(&v["hard_gate_failures"]).unwrap_or_default();
    assert!(
        failures.contains("resident-edit degraded"),
        "the DETECTOR message fires (not a marker): {failures}"
    );

    // (c) Detector-disabled control (env): the forced input is present,
    // but the degrade-specific red VANISHES — proving the gate failure
    // comes from the detector, never from the inject flag itself. (An
    // independent memory-gate failure may legitimately remain: a forced
    // rebuild while resident does exceed the worker peak gate.)
    let out = std::process::Command::new(exe)
        .args([
            "bench-turnaround",
            "--samples",
            samples,
            "--gate-ms",
            gate,
            "--inject-rebuild",
        ])
        .env("AUTO_CTE_DISABLE_RESIDENT_GATE", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("disabled control spawns");
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    let failures = serde_json::to_string(&v["hard_gate_failures"]).unwrap_or_default();
    assert!(
        !failures.contains("resident-edit degraded"),
        "with the detector disabled the degrade red is gone (the gate was load-bearing): {failures}"
    );
}

// ---- F-817-R5-03: the REAL wire-mutation matrix ------------------------
//
// Every mode below makes the WORKER emit actually-mutated response
// bytes (env seam inside the worker, never active in production); the
// shared parent admission must REJECT each one with a named reason —
// no panic, no swallow — and the legal control stays green.

fn serve_one_with_env(envs: &[(&str, &str)], store: &std::path::Path) -> (String, bool) {
    let kind = auto_cte_probe::bench::DeckKind::EditRev(0);
    let req =
        auto_cte_probe::bench::build_worker_request_line("mut", "rh-mut", store, "mut", &kind);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"));
    cmd.arg("case-serve")
        .envs(envs.iter().copied())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("worker spawn");
    use std::io::Write as _;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(req.as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("worker completes");
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        out.status.success(),
    )
}

fn admit_edit_rev(raw: &str, exit_ok: bool) -> auto_cte_probe::bench::AdmissionOutcome {
    let kind = auto_cte_probe::bench::DeckKind::EditRev(0);
    let (deck, _) = kind.materialize();
    auto_cte_probe::bench::admit_response(raw, "mut", "rh-mut", &deck, &kind.expects(), exit_ok)
}

#[test]
fn cte_turnaround_wire_mutations_rejected_no_panic() {
    // (mode, a substring the rejection reason must carry)
    let modes: &[(&str, &str)] = &[
        ("wrong-value", "wrong key/set or wrong value"),
        ("wrong-hir", "hir[main.ct] != constant"),
        ("wrong-key", "values[binding main.ct] wrong key/set"),
        ("wrong-type", "consumer_types[main.ct] != i32"),
        // The FIRST query's value (a PRIOR step, not the final one).
        ("wrong-first-value", "step 0:"),
        ("diag", "inside a valid envelope"),
        ("no-hir", "hir[main.ct]"),
        ("extra-step", "step count 4 != expected 3"),
        ("missing-step", "step count 2 != expected 3"),
        ("bad-hash", "source_hash mismatch"),
        ("omit-peak", "peak sampling unavailable"),
    ];
    for (mode, needle) in modes {
        let store = tempfile::tempdir().unwrap();
        let (raw, exit_ok) = serve_one_with_env(&[("AUTO_CTE_BENCH_MUTATE", mode)], store.path());
        let adm = admit_edit_rev(&raw, exit_ok);
        assert!(
            !adm.ok,
            "{mode} must be REJECTED (raw: {})",
            &raw[..raw.len().min(400)]
        );
        let all = adm.failures.join("; ");
        assert!(
            all.contains(needle),
            "{mode} rejection names the cause ({needle}): {all}"
        );
        // The bad response's raw bytes exist for the failure report.
        assert!(raw.contains("cteworker/1"), "{mode} kept real wire bytes");
    }
    // Legal positive control: no mutation env -> green.
    let store = tempfile::tempdir().unwrap();
    let (raw, exit_ok) = serve_one_with_env(&[], store.path());
    let adm = admit_edit_rev(&raw, exit_ok);
    assert!(adm.ok, "legal response stays green: {:?}", adm.failures);
    assert!(adm.worker_peak.unwrap_or(0) > 0, "real child peak present");
}

/// The WARM path (case-serve-stream) speaks the same wire and feeds
/// the same admission — representative stream mutations reject too.
#[test]
fn cte_turnaround_stream_mutations_share_admission() {
    for (mode, needle) in [
        ("wrong-key", "values[binding main.ct] wrong key/set"),
        ("missing-step", "step count"),
    ] {
        let store = tempfile::tempdir().unwrap();
        let kind = auto_cte_probe::bench::DeckKind::EditRev(0);
        let req = auto_cte_probe::bench::build_worker_request_line(
            "sm",
            "rh-sm",
            store.path(),
            "sm",
            &kind,
        );
        let mut child = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
            .arg("case-serve-stream")
            .env("AUTO_CTE_BENCH_MUTATE", mode)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("stream worker spawn");
        use std::io::{BufRead as _, BufReader, Write as _};
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(format!("{req}\n").as_bytes()).unwrap();
        stdin.flush().unwrap();
        let mut reader = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        reader.read_line(&mut line).expect("stream response");
        let adm = admit_edit_rev(&line, true);
        assert!(!adm.ok, "{mode} rejected on the stream path");
        assert!(adm.failures.join("; ").contains(needle), "{mode}: {needle}");
        drop(stdin);
        let _ = child.wait();
    }
}

// ---- F-817-R5-01: the memory counter-controls --------------------------

/// A REAL child memory transient (the worker holds 420MiB for the
/// request's lifetime, self-reported through its own 1ms sampler)
/// must turn the SAME formal worker gate red via the bench.
#[test]
fn cte_turnaround_child_memory_spike_control_is_red() {
    let out = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
        .args([
            "bench-turnaround",
            "--samples",
            "2",
            "--gate-ms",
            "100000",
            "--only",
            "restart",
        ])
        .env("AUTO_CTE_BENCH_INJECT_ALLOC_MIB", "420")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("control spawns");
    assert_ne!(out.status.code().unwrap_or(0), 0, "the spike is red");
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert_eq!(v["target_met"], serde_json::json!(false));
    let failures = serde_json::to_string(&v["hard_gate_failures"]).unwrap_or_default();
    assert!(
        failures.contains("restart worker peak") && failures.contains("> 384MiB gate"),
        "the failure names the child's self-reported peak: {failures}"
    );
    // The child really reported an elevated peak (>420MiB committed).
    let peak = v["scenario_peaks"]["restart"]["bytes"]
        .as_u64()
        .unwrap_or(0);
    assert!(
        peak > 420 * 1024 * 1024,
        "the recorded peak is the CHILD's own reading: {:.1}MiB",
        peak as f64 / 1048576.0
    );
}

/// The tree gate sees the SIMULTANEOUS sum: two children each under
/// the 384MiB worker gate (a small real spike on top of the debug
/// baseline keeps them distinguishable from an unspiked run), together
/// with the parent over the 512MiB tree cap — aggregation, not
/// per-process maxima.
#[test]
fn cte_turnaround_tree_gate_aggregates_concurrent_children() {
    use std::io::Write as _;
    let sampler = auto_cte_probe::procmem::TreeSampler::start(5);
    let mut children = Vec::new();
    for k in 0..2 {
        let store = tempfile::tempdir().unwrap();
        let kind = auto_cte_probe::bench::DeckKind::QueryBig { rev: 0 };
        let req = auto_cte_probe::bench::build_worker_request_line(
            "tree",
            &format!("rh-tree-{k}"),
            store.path(),
            "tree",
            &kind,
        );
        let mut child = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
            .arg("case-serve")
            .env("AUTO_CTE_BENCH_INJECT_ALLOC_MIB", "10")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("tree child spawn");
        sampler.register(child.id());
        child
            .stdin
            .take()
            .unwrap()
            .write_all(req.as_bytes())
            .unwrap();
        children.push(child);
    }
    let mut child_peaks = Vec::new();
    for child in children {
        let out = child.wait_with_output().expect("tree child completes");
        let v: serde_json::Value =
            serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
        child_peaks.push(v["ok"]["worker_private_bytes_peak"].as_u64().unwrap_or(0));
    }
    let tree = sampler.stop();
    for (k, p) in child_peaks.iter().enumerate() {
        assert!(
            *p > 300 * 1024 * 1024 && *p < 384 * 1024 * 1024,
            "child {k} carries a real workload peak UNDER the worker gate: {:.1}MiB",
            *p as f64 / 1048576.0
        );
    }
    assert!(
        tree.tree_peak > 512 * 1024 * 1024,
        "the tree gate's input sees the SUM (each child <384MiB, together >512MiB): {:.1}MiB",
        tree.tree_peak as f64 / 1048576.0
    );
    assert!(
        tree.tree_peak - tree.parent_peak > child_peaks.iter().copied().max().unwrap_or(0),
        "the children's summed contribution exceeds the largest single child — aggregation, not a per-process maximum (tree {:.1}MiB, parent {:.1}MiB)",
        tree.tree_peak as f64 / 1048576.0,
        tree.parent_peak as f64 / 1048576.0
    );
}

/// A real managed-budget overage inside the warm worker (staging
/// partition shrunk below the request size) must turn the same bench
/// red through the wire's managed verdict.
#[test]
fn cte_turnaround_managed_budget_control_is_red() {
    let out = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
        .args([
            "bench-turnaround",
            "--samples",
            "2",
            "--gate-ms",
            "100000",
            "--only",
            "body-edit",
        ])
        .env("AUTO_CTE_BENCH_SHRINK_STAGING_KIB", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("control spawns");
    assert_ne!(out.status.code().unwrap_or(0), 0, "the overage is red");
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert_eq!(v["target_met"], serde_json::json!(false));
    let failures = serde_json::to_string(&v["hard_gate_failures"]).unwrap_or_default();
    assert!(
        failures.contains("managed budget exceeded"),
        "the failure names the managed verdict: {failures}"
    );
}

/// Sampling unavailability (the worker reports peak 0 / omits it) is a
/// RED, never a silent green — and the failed raw row carries the
/// response excerpt + failure reasons (the failure report travels).
#[test]
fn cte_turnaround_sampling_unavailable_control_is_red() {
    let tmp = tempfile::tempdir().unwrap();
    let raw = tmp.path().join("omit.jsonl");
    let out = Command::new(env!("CARGO_BIN_EXE_cte_probe_worker"))
        .args([
            "bench-turnaround",
            "--samples",
            "2",
            "--gate-ms",
            "100000",
            "--only",
            "restart",
            "--out",
            raw.to_str().unwrap(),
        ])
        .env("AUTO_CTE_BENCH_MUTATE", "omit-peak")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("control spawns");
    assert_ne!(out.status.code().unwrap_or(0), 0, "unavailable is red");
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    let failures = serde_json::to_string(&v["hard_gate_failures"]).unwrap_or_default();
    assert!(
        failures.contains("sampling unavailable"),
        "the failure names unavailability: {failures}"
    );
    // The failed rows preserve the raw response excerpt + reasons.
    let text = std::fs::read_to_string(&raw).unwrap();
    let failed_row = text
        .lines()
        .find(|l| l.contains("\"ok\":false"))
        .expect("a failed row is present in the report");
    assert!(
        failed_row.contains("peak sampling unavailable") && failed_row.contains("cteworker/1"),
        "the row carries the failure report and the raw bytes: {failed_row}"
    );
}
