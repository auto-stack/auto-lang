//! PLAN-798 T-07: the `ac-av/core-run-v1` probe recipe — corpus
//! execution through the unified runner, negative oracles, and decode
//! admission negatives (V-12).

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-ac-av-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn out_of(root: &Path, sub: &str) -> PathBuf {
    root.parent().unwrap_or(Path::new(".")).join(format!(
        "{}-{}",
        root.file_name().unwrap().to_string_lossy(),
        sub
    ))
}

/// Serialize every test that spawns the real av worker (mirrors the sema
/// FAULT_LOCK discipline: process-global worker identity caches).
pub(crate) static AV_CASE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn status_of(
    report: &auto_test_case::report::RunReport,
    check: &str,
) -> auto_test_case::report::Status {
    report
        .results
        .iter()
        .find(|r| r.check_id == check)
        .unwrap_or_else(|| panic!("check {} present in {:?}", check, report.results))
        .status
}

/// V-12: the committed ac-av corpus runs END TO END through the unified
/// runner — a real av worker process, real store on disk, parent-side
/// oracle judgement across all three oracle faces (success+exit, trap,
/// diagnosed admission rejection) plus the capability-gated trace case.
#[test]
fn ac_av_probe_corpus_executes_green() {
    let _guard = AV_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ac-av");
    let root = temp_root("corpus");
    let out = out_of(&root, "out");
    let report = run_suite(&suite_dir.join("ac-av.suite.ad"), &out, &Limits::defaults())
        .expect("ac-av suite runs");
    for check in ["run", "run", "run", "run", "run"] {
        // check ids are all "run" per case; assert every result passed
    }
    let failing: Vec<String> = report
        .results
        .iter()
        .filter(|r| r.status != auto_test_case::report::Status::Pass)
        .map(|r| {
            format!(
                "{}: {:?} {} | mismatches: {:?}",
                r.check_id,
                r.status,
                r.detail.clone().unwrap_or_default(),
                r.mismatches
            )
        })
        .collect();
    assert!(
        failing.is_empty(),
        "ac-av corpus all green ({} results): {:#?}",
        report.results.len(),
        failing
    );
    let ids: Vec<String> = report.results.iter().map(|r| r.check_id.clone()).collect();
    assert!(
        report.results.len() >= 6,
        "six checks across five cases + traps pair; got {:?}",
        ids
    );
}

/// V-12 negative oracle: a WRONG exit expectation must FAIL — the runner
/// is a real judge, not a green stamp (good actual + wrong expected red;
/// equivalently wrong actual + good expected).
#[test]
fn ac_av_probe_negative_oracle_fails() {
    let _guard = AV_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let root = temp_root("neg");
    std::fs::write(
        root.join("neg.suite.ad"),
        r#"# neg suite
```test-suite
suite { schema: "autolang.suite/2", id: "neg-av", include: ["neg.case.ad"] }
```
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("neg.case.ad"),
        r#"# neg case
```test-case
case {
  schema: "autolang.case/2",
  id: "neg.av",
  source: { text: "fn main() int { return 7 }", syntax: "auto", path: "main.at" },
  entry: { mode: "program" },
  checks: [
    { id: "wrong-exit", adapter: "probe", action: "run",
      options: {
        recipe: "ac-av/core-run-v1",
        av: { entry: "d_main" },
        steps: [ { op: "run" } ]
      },
      expect: { completion: "success", exit: 8 } }
  ]
}
```
"#,
    )
    .unwrap();
    let out = out_of(&root, "out");
    let report =
        run_suite(&root.join("neg.suite.ad"), &out, &Limits::defaults()).expect("suite runs");
    assert_eq!(
        status_of(&report, "wrong-exit"),
        auto_test_case::report::Status::Fail,
        "wrong exit oracle must be red (actual 7 vs expected 8)"
    );
}

/// V-12 decode admission negatives: unregistered recipes, cross-recipe
/// step vocabulary, cross-recipe bindings and missing bindings are
/// LOCATED decode errors.
#[test]
fn ac_av_probe_decode_negatives() {
    use auto_test_case::atom::parse_contract;
    use auto_test_case::contract::decode_case;

    let decode_case_doc = |doc: &str| -> Result<
        auto_test_case::contract::CaseSpec,
        auto_test_case::contract::ContractError,
    > {
        let v = parse_contract(doc).unwrap();
        decode_case(&v, &Limits::defaults())
    };
    let base = |options: &str| {
        format!(
            r#"case {{
  schema: "autolang.case/2",
  id: "neg.decode.av",
  source: {{ text: "fn main() int {{ return 1 }}", syntax: "auto", path: "main.at" }},
  entry: {{ mode: "program" }},
  checks: [
    {{ id: "c", adapter: "probe", action: "run",
      options: {{ {options} }},
      expect: {{ completion: "success" }} }}
  ]
}}
"#,
        )
    };
    // Unregistered recipe.
    let e = decode_case_doc(&base(
        r#"recipe: "ac-av/core-run-v99", av: { entry: "d_main" }, steps: [ { op: "run" } ]"#,
    ))
    .unwrap_err();
    assert!(e.message.contains("not registered"), "{e:?}");
    // ac-av recipe with a sema-only step op.
    let e = decode_case_doc(&base(
        r#"recipe: "ac-av/core-run-v1", av: { entry: "d_main" }, steps: [ { op: "query" } ]"#,
    ))
    .unwrap_err();
    assert!(
        e.message.contains("not valid for recipe")
            || e.message.contains("must start with a run step"),
        "{e:?}"
    );
    // ac-av recipe with the sema binding.
    let e = decode_case_doc(&base(
        r#"recipe: "ac-av/core-run-v1", av: { entry: "d_main" }, sema: { package: "x", module: "y", kind: "infer-body", fn: "f" }, steps: [ { op: "run" } ]"#,
    ))
    .unwrap_err();
    assert!(e.message.contains("not valid for recipe"), "{e:?}");
    // ac-av recipe without the av binding.
    let e = decode_case_doc(&base(
        r#"recipe: "ac-av/core-run-v1", steps: [ { op: "run" } ]"#,
    ))
    .unwrap_err();
    assert!(e.message.contains("requires an av binding"), "{e:?}");
    // sema recipe with the av binding.
    let e = decode_case_doc(&base(
        r#"recipe: "sema/query-v1", av: { entry: "d_main" }, sema: { package: "x", module: "y", kind: "infer-body", fn: "f" }, steps: [ { op: "query" } ]"#,
    ))
    .unwrap_err();
    assert!(e.message.contains("not valid for recipe"), "{e:?}");
    // The valid form decodes.
    decode_case_doc(&base(
        r#"recipe: "ac-av/core-run-v1", av: { entry: "d_main" }, steps: [ { op: "run" } ]"#,
    ))
    .expect("valid ac-av check decodes");
}

/// V-11 formal-runner leg (T-07): the hard gate measured END TO END
/// through the unified runner — materialize → worker spawn → full AV
/// pipeline → parent-side judge — on the FROZEN load, per-sample NEW
/// body revisions (edit path) and the signature-conflict diagnostic
/// path. The Rust-directed bench cannot substitute for this (plan §5.5).
#[test]
fn ac_av_probe_formal_turnaround_gates() {
    let _guard = AV_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    // Sample count/gate overridable by the verify script (release-grade
    // run keeps the frozen 30/1000ms).
    let samples: usize = std::env::var("AV798_FORMAL_SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let gate_ms: f64 = std::env::var("AV798_FORMAL_GATE_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000.0);
    assert!(samples >= 1);

    let root = temp_root("formal");
    let percentile = |v: &[f64], p: f64| {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        s[((p / 100.0) * (s.len() - 1) as f64).round() as usize % s.len()]
    };

    // The frozen load lives in the probe workspace; fetch each revision
    // from the worker (single source of truth, no product dependency).
    let fetch_rev = |rev: u32| -> (String, i32) {
        let out = std::process::Command::new(
            crate::test_cases::ac_av::worker_binary().expect("worker located"),
        )
        .args(["load-revision", "--rev", &rev.to_string()])
        .output()
        .expect("spawn worker load-revision");
        assert!(out.status.success(), "load-revision ok");
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("revision json");
        (
            v["source"].as_str().expect("source").to_string(),
            v["expected_main"].as_i64().expect("expected") as i32,
        )
    };
    let (conflict_src, _) = {
        // The frozen signature-conflict edit: derived from the base load.
        let (base, _) = fetch_rev(0);
        // Worker-side helper keeps the derivation frozen; reuse via env of
        // the load generator through the recipe: simplest is to fetch rev 0
        // and apply the frozen edit locally ONLY as bytes (the edit literal
        // is a frozen contract string).
        let edited = base.replacen(
            "fn fn_leaf_1(v int) int {",
            "fn fn_leaf_1(v int, w int) int {",
            1,
        );
        assert!(edited != base, "frozen sig-conflict marker present");
        (edited, 0)
    };

    // Edit path (formal warm-session primitive): one check per revision
    // with steps [run(rev_{i-1}) warm-up, edit-source(diff line), run] —
    // the worker keeps ONE session store, so the final run is a WARM
    // single-unit edit. The gate window = everything after the warm-up
    // (edit + full frontend + verify + rebuild + execute + IPC/admit).
    let mut edit_walls = Vec::new();
    let (mut prev_source, mut prev_expected) = fetch_rev(0);
    for i in 1..=samples {
        let (rev_source, rev_expected) = fetch_rev(i as u32);
        // The frozen edit position: fn_037's first multiplier line. The
        // literal includes the function header so it matches exactly once
        // (other functions may share the multiplier value).
        let extract = |src: &str| -> String {
            let header = "fn fn_037(x int) int {";
            let hi = src.find(header).expect("fn_037 present");
            let rest = &src[hi..];
            let acc_line_end = rest.find('\n').expect("acc line end");
            let acc_line_end2 = rest[acc_line_end + 1..].find('\n').expect("acc line end 2");
            format!(
                "{}
{}",
                header,
                &rest[acc_line_end + 1..acc_line_end + 1 + acc_line_end2]
            )
        };
        let from_line = extract(&prev_source);
        let to_line = extract(&rev_source);
        assert!(
            from_line != to_line,
            "revision {i} edits fn_037's multiplier"
        );
        assert!(
            from_line.contains("var acc = x *"),
            "the differing line is fn_037's multiplier"
        );
        let case_dir = root.join(format!("rev-{i}"));
        let out_dir = root.join(format!("rev-{i}-out"));
        std::fs::create_dir_all(&case_dir).unwrap();
        std::fs::write(case_dir.join("main.at"), &prev_source).unwrap();
        let esc = |t: &str| t.replace('\\', "\\\\").replace('"', "\\\"");
        std::fs::write(
            case_dir.join("c.case.ad"),
            format!(
                "# formal warm-edit rev {i}
```test-case
case {{
  schema: \"autolang.case/2\",
  id: \"av.formal.rev{i}\",
  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},
  entry: {{ mode: \"program\" }},
  checks: [
    {{ id: \"run\", adapter: \"probe\", action: \"run\",
      options: {{ recipe: \"ac-av/core-run-v1\", av: {{ entry: \"d_main\" }}, steps: [ {{ op: \"run\" }}, {{ op: \"edit-source\", from: \"{from}\", to: \"{to}\" }}, {{ op: \"run\" }} ] }},
      expect: {{ completion: \"success\", exit: {exit} }} }}
  ]
}}
```
",
                from = esc(&from_line),
                to = esc(&to_line),
                exit = rev_expected,
            ),
        )
        .unwrap();
        std::fs::write(
            case_dir.join("s.suite.ad"),
            "# s
```test-suite
suite { schema: \"autolang.suite/2\", id: \"s\", include: [\"c.case.ad\"] }
```
",
        )
        .unwrap();
        let started = std::time::Instant::now();
        let report = run_suite(&case_dir.join("s.suite.ad"), &out_dir, &Limits::defaults())
            .expect("formal suite runs");
        let full_wall = started.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(
            status_of(&report, "run"),
            auto_test_case::report::Status::Pass,
            "rev {i} must be green through the formal seam (value {rev_expected}): {:#?}",
            report
                .results
                .iter()
                .map(|r| (r.status, r.detail.clone(), r.mismatches.clone()))
                .collect::<Vec<_>>()
        );
        // Gate window: everything after the warm-up run. The receipt's
        // stage spans carry the worker session wall and the final run's
        // driver wall; the warm-up = the session wall minus the final run
        // is not directly separable runner-side, so measure the window as
        // the final run's driver wall + the per-request IPC overhead
        // estimate is avoided: use the WORKER-side final-step wall from
        // the receipt (probe-receipt.json) plus admit time is bounded by
        // the runner — the honest formal quantity is the final step wall
        // plus the request overhead measured once (spawn+IPC), recorded
        // below as `formal_overhead_ms` from the warm-up delta.
        // Locate this check's receipt (artifact layout is runner-owned).
        let mut receipt = String::new();
        let mut stack = vec![out_dir.clone()];
        while let Some(d) = stack.pop() {
            if let Ok(rd) = std::fs::read_dir(&d) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        stack.push(p);
                    } else if p
                        .file_name()
                        .map(|n| n == "probe-receipt.json")
                        .unwrap_or(false)
                    {
                        receipt = std::fs::read_to_string(&p).unwrap_or_default();
                    }
                }
            }
        }
        let rj: serde_json::Value = serde_json::from_str(&receipt)
            .unwrap_or_else(|e| panic!("receipt parses ({e}): {:.120}", receipt));
        let final_wall = rj["step"]["wall_ms"].as_f64().unwrap_or(0.0);
        let worker_us = rj["worker_us"].as_u64().unwrap_or(0) as f64 / 1000.0;
        // Warm-up wall = worker wall - final step wall (edit is ~0); the
        // edit-request window = final step wall + (spawn + IPC + admit),
        // estimated as full_wall - worker_us (runner-side fixed overhead)
        // applied once to the window.
        let overhead = (full_wall - worker_us).max(0.0);
        let window = final_wall + overhead;
        eprintln!(
            "formal warm-edit rev {i}: full {full_wall:.1}ms worker {worker_us:.1}ms final-step {final_wall:.1}ms overhead {overhead:.1}ms -> window {window:.1}ms"
        );
        edit_walls.push(window);
        prev_source = rev_source;
        prev_expected = rev_expected;
    }
    let _ = prev_expected;

    // Signature-conflict path: the frozen conflict edit diagnosed through
    // the formal seam (conflict_src derived above from the fetched base).
    let conflicted = conflict_src;
    let sig_dir = root.join("sig");
    std::fs::create_dir_all(&sig_dir).unwrap();
    std::fs::write(sig_dir.join("main.at"), &conflicted).unwrap();
    std::fs::write(
        sig_dir.join("c.case.ad"),
        "# formal sig conflict\n```test-case\ncase {\n  schema: \"autolang.case/2\",\n  id: \"av.formal.sig\",\n  source: { file: \"main.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"program\" },\n  checks: [\n    { id: \"run\", adapter: \"probe\", action: \"run\",\n      options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\" }, steps: [ { op: \"run\" } ] },\n      expect: { completion: { stage: \"parse\", code: \"source.argument\" } } }\n  ]\n}\n```\n",
    )
    .unwrap();
    std::fs::write(
        sig_dir.join("s.suite.ad"),
        "# s\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    let mut sig_walls = Vec::new();
    for i in 0..samples {
        let started = std::time::Instant::now();
        let report = run_suite(
            &sig_dir.join("s.suite.ad"),
            &root.join(format!("sig-out-{i}")),
            &Limits::defaults(),
        )
        .expect("formal sig suite runs");
        let wall = started.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(
            status_of(&report, "run"),
            auto_test_case::report::Status::Pass,
            "sig-conflict sample {i} must be a green DIAGNOSED negative: {:#?}",
            report
                .results
                .iter()
                .map(|r| (r.check_id.clone(), r.status, r.mismatches.clone()))
                .collect::<Vec<_>>()
        );
        sig_walls.push(wall);
    }

    let p95_edit = percentile(&edit_walls, 95.0);
    let p95_sig = percentile(&sig_walls, 95.0);
    eprintln!(
        "formal turnaround: edit P50/P95/P99 = {:.1}/{:.1}/{:.1} ms; sig P95 = {:.1} ms (gate {:.1} ms, {} samples)",
        percentile(&edit_walls, 50.0),
        p95_edit,
        percentile(&edit_walls, 99.0),
        p95_sig,
        gate_ms,
        samples
    );
    assert!(
        p95_edit < gate_ms,
        "formal edit-path P95 {p95_edit:.1}ms >= gate {gate_ms}ms"
    );
    assert!(
        p95_sig < gate_ms,
        "formal sig-path P95 {p95_sig:.1}ms >= gate {gate_ms}ms"
    );
    let _ = std::fs::remove_dir_all(&root);
}
