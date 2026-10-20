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
    // Extending the shared enum must not authorize AV run in ABI.
    let e = decode_case_doc(&base(
        r#"recipe: "abi/interop-v1", abi: { leg: "a" }, steps: [{ op: "run" }]"#,
    ))
    .unwrap_err();
    assert!(e.message.contains("not valid for recipe"), "{e:?}");
    // The valid form decodes.
    decode_case_doc(&base(
        r#"recipe: "ac-av/core-run-v1", av: { entry: "d_main" }, steps: [ { op: "run" } ]"#,
    ))
    .expect("valid ac-av check decodes");
}

/// V-11 formal-runner leg (T-07/F798-R3-07): the hard gate measured as a
/// CONTINUOUS parent-side window through the unified runner. Per revision
/// the harness issues TWO real calls sharing an explicit session store:
/// a warm-up `[run]` call, then the MEASURED `[edit-source, run]` call —
/// the window is the measured call's `request_wall_ms` (request build ->
/// spawn -> request write -> worker session -> edit + full pipeline +
/// execute -> response read -> admission/judgement). Nothing is
/// subtracted; IPC, spawn and admission are inside the window by
/// construction. The Rust-directed bench cannot substitute for this
/// (plan section 5.5).
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
    // Find this check's probe receipt (artifact layout is runner-owned).
    let find_receipt = |out_dir: &Path| -> serde_json::Value {
        let mut receipt = String::new();
        let mut stack = vec![out_dir.to_path_buf()];
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
        serde_json::from_str(&receipt)
            .unwrap_or_else(|e| panic!("receipt parses ({e}): {:.120}", receipt))
    };
    // One formal call: write the case, run the suite, require green, and
    // return the receipt (whose request_wall_ms IS the gate window).
    // `expect` is always success+value; diagnosed rejections have their
    // own oracle case.
    let formal_call = |tag: &str,
                       source: &str,
                       steps: &str,
                       session: Option<&str>,
                       expect_exit: Option<i64>|
     -> serde_json::Value {
        let case_dir = root.join(tag);
        let out_dir = root.join(format!("{tag}-out"));
        let _ = std::fs::remove_dir_all(&case_dir);
        let _ = std::fs::remove_dir_all(&out_dir);
        std::fs::create_dir_all(&case_dir).unwrap();
        std::fs::write(case_dir.join("main.at"), source).unwrap();
        let session_field = match session {
            Some(s) => format!(", session: \"{}\"", s),
            None => String::new(),
        };
        let expect_field = match expect_exit {
            Some(v) => format!(", exit: {}", v),
            None => String::new(),
        };
        std::fs::write(
            case_dir.join("c.case.ad"),
            format!(
                "# formal {tag}\n```test-case\ncase {{\n  schema: \"autolang.case/2\",\n  id: \"av.formal.{tag}\",\n  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [\n    {{ id: \"run\", adapter: \"probe\", action: \"run\",\n      options: {{ recipe: \"ac-av/core-run-v1\", av: {{ entry: \"d_main\"{session_field} }}, steps: [{steps}] }},\n      expect: {{ completion: \"success\"{expect_field} }} }}\n  ]\n}}\n```\n"
            ),
        )
        .unwrap();
        std::fs::write(
            case_dir.join("s.suite.ad"),
            "# s\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s\", include: [\"c.case.ad\"] }\n```\n",
        )
        .unwrap();
        let report = run_suite(&case_dir.join("s.suite.ad"), &out_dir, &Limits::defaults())
            .expect("formal suite runs");
        assert_eq!(
            status_of(&report, "run"),
            auto_test_case::report::Status::Pass,
            "formal call {tag} must be green: {:#?}",
            report
                .results
                .iter()
                .map(|r| (r.status, r.detail.clone(), r.mismatches.clone()))
                .collect::<Vec<_>>()
        );
        find_receipt(&out_dir)
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

    // Edit path: per revision — warm-up `[run]` call over the PREVIOUS
    // revision's source (the session store goes warm), then the MEASURED
    // `[edit-source, run]` call applying the frozen single-line edit.
    let mut edit_walls = Vec::new();
    let (mut prev_source, mut prev_expected) = fetch_rev(0);
    for i in 1..=samples {
        let (rev_source, rev_expected) = fetch_rev(i as u32);
        // The frozen edit position: fn_037's first multiplier line. The
        // literal includes the function header so it matches exactly once.
        let extract = |src: &str| -> String {
            let header = "fn fn_037(x int) int {";
            let hi = src.find(header).expect("fn_037 present");
            let rest = &src[hi..];
            let acc_line_end = rest.find('\n').expect("acc line end");
            let acc_line_end2 = rest[acc_line_end + 1..].find('\n').expect("acc line end 2");
            format!(
                "{}\n{}",
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
        let session = format!("p{}-rev{i}", std::process::id());
        // Warm-up call (outside the window).
        let warm = formal_call(
            &format!("rev-{i}-warm"),
            &prev_source,
            "{ op: \"run\" }",
            Some(&session),
            Some(prev_expected as i64),
        );
        assert_eq!(
            warm["step"]["counters"]["cache_hit_abc"]
                .as_u64()
                .unwrap_or(0),
            0,
            "warm-up on a fresh session store is a real cold run"
        );
        // Measured call: the whole edit request inside the window.
        let esc = |t: &str| t.replace('\\', "\\\\").replace('"', "\\\"");
        let steps = format!(
            "{{ op: \"edit-source\", from: \"{}\", to: \"{}\" }}, {{ op: \"run\" }}",
            esc(&from_line),
            esc(&to_line)
        );
        let measured = formal_call(
            &format!("rev-{i}"),
            &prev_source,
            &steps,
            Some(&session),
            Some(rev_expected as i64),
        );
        let window = measured["request_wall_ms"].as_f64().unwrap_or(0.0);
        assert!(window > 0.0, "request_wall_ms present in the receipt");
        // The measured final run IS the warm single-unit edit (one ABC
        // miss = the edited unit; the rest hit the session store).
        let hits = measured["step"]["counters"]["cache_hit_abc"]
            .as_u64()
            .unwrap_or(0);
        let misses = measured["step"]["counters"]["cache_miss_abc"]
            .as_u64()
            .unwrap_or(0);
        assert!(
            misses >= 1 && hits >= 1,
            "warm edit shape (hits {hits}, misses {misses})"
        );
        assert_eq!(
            measured["step"]["value"].as_i64().unwrap_or(i64::MIN),
            rev_expected as i64,
            "measured run publishes the current revision's correct value"
        );
        let spans: Vec<String> = measured["step"]["spans"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|s| {
                        Some(format!(
                            "{}={:.0}",
                            s.as_array()?.first()?.as_str()?,
                            s.as_array()?.get(1)?.as_f64()?
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        eprintln!(
            "formal warm-edit rev {i}: window {window:.1}ms (worker {:.1}ms, step {:.1}ms; hits {hits}, misses {misses}) spans: {}",
            measured["worker_us"].as_f64().unwrap_or(0.0) / 1000.0,
            measured["step"]["wall_ms"].as_f64().unwrap_or(0.0),
            spans.join(" "),
        );
        edit_walls.push(window);
        prev_source = rev_source;
        prev_expected = rev_expected;
    }
    let _ = prev_expected;

    // Signature-conflict path: the frozen conflict edit diagnosed through
    // the formal seam — the window is the diagnosed request's own wall,
    // and the oracle face is the green DIAGNOSED negative.
    let (base, _) = fetch_rev(0);
    let conflicted = base.replacen(
        "fn fn_leaf_1(v int) int {",
        "fn fn_leaf_1(v int, w int) int {",
        1,
    );
    assert!(conflicted != base, "frozen sig-conflict marker present");
    {
        let case_dir = root.join("sig-oracle");
        let _ = std::fs::remove_dir_all(&case_dir);
        std::fs::create_dir_all(&case_dir).unwrap();
        std::fs::write(case_dir.join("main.at"), &conflicted).unwrap();
        std::fs::write(
            case_dir.join("c.case.ad"),
            "# formal sig conflict\n```test-case\ncase {\n  schema: \"autolang.case/2\",\n  id: \"av.formal.sig\",\n  source: { file: \"main.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"program\" },\n  checks: [\n    { id: \"run\", adapter: \"probe\", action: \"run\",\n      options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\" }, steps: [ { op: \"run\" } ] },\n      expect: { completion: { stage: \"parse\", code: \"source.argument\" } } }\n  ]\n}\n```\n",
        )
        .unwrap();
        std::fs::write(
            case_dir.join("s.suite.ad"),
            "# s\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s\", include: [\"c.case.ad\"] }\n```\n",
        )
        .unwrap();
        let report = run_suite(
            &case_dir.join("s.suite.ad"),
            &root.join("sig-oracle-out"),
            &Limits::defaults(),
        )
        .expect("formal sig suite runs");
        assert_eq!(
            status_of(&report, "run"),
            auto_test_case::report::Status::Pass,
            "sig-conflict is a green DIAGNOSED negative: {:#?}",
            report.results
        );
    }
    let mut sig_walls = Vec::new();
    for i in 0..samples {
        // The same diagnosed rejection, window-measured under its own
        // private store (the diagnosis is stateless): the oracle face is
        // the frontend conflict, so the call is green AND windowed.
        let case_dir = root.join(format!("sig-w{i}"));
        let out_dir = root.join(format!("sig-w{i}-out"));
        let _ = std::fs::remove_dir_all(&case_dir);
        let _ = std::fs::remove_dir_all(&out_dir);
        std::fs::create_dir_all(&case_dir).unwrap();
        std::fs::write(case_dir.join("main.at"), &conflicted).unwrap();
        std::fs::write(
            case_dir.join("c.case.ad"),
            format!(
                "# formal sig window {i}
```test-case
case {{
  schema: \"autolang.case/2\",
  id: \"av.formal.sigw{i}\",
  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},
  entry: {{ mode: \"program\" }},
  checks: [
    {{ id: \"run\", adapter: \"probe\", action: \"run\",
      options: {{ recipe: \"ac-av/core-run-v1\", av: {{ entry: \"d_main\" }}, steps: [ {{ op: \"run\" }} ] }},
      expect: {{ completion: {{ stage: \"parse\", code: \"source.argument\" }} }} }}
  ]
}}
```
"
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
        let report = run_suite(&case_dir.join("s.suite.ad"), &out_dir, &Limits::defaults())
            .expect("formal sig window suite runs");
        assert_eq!(
            status_of(&report, "run"),
            auto_test_case::report::Status::Pass,
            "sig window sample {i} is a green DIAGNOSED negative"
        );
        let receipt = find_receipt(&out_dir);
        let window = receipt["request_wall_ms"].as_f64().unwrap_or(0.0);
        assert!(window > 0.0);
        sig_walls.push(window);
    }

    let p95_edit = percentile(&edit_walls, 95.0);
    let p95_sig = percentile(&sig_walls, 95.0);
    eprintln!(
        "formal turnaround (continuous request-to-publish windows): edit P50/P95/P99 = {:.1}/{:.1}/{:.1} ms; sig P95 = {:.1} ms (gate {:.1} ms, {} samples)",
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
    // Session stores are parent-owned scratch: remove the shared roots.
    for i in 1..=samples {
        let _ = std::fs::remove_dir_all(
            std::env::temp_dir().join(format!("av798-session-p{}-rev{i}", std::process::id())),
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R3-05/06 formal counter-controls: worker-contract violations are
/// INFRA errors, never green language verdicts — non-zero exit, trailing
/// protocol frames, a result not bound to the current revision's source
/// hash, and a wall breach are each rejected by the parent runner.
#[test]
fn ac_av_probe_worker_contract_faults_are_infra() {
    let _guard = AV_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let root = temp_root("faults");
    let run_one = |tag: &str| -> auto_test_case::report::Status {
        let case_dir = root.join(tag);
        let out_dir = root.join(format!("{tag}-out"));
        let _ = std::fs::remove_dir_all(&case_dir);
        let _ = std::fs::remove_dir_all(&out_dir);
        std::fs::create_dir_all(&case_dir).unwrap();
        std::fs::write(case_dir.join("main.at"), "fn main() int { return 7 }").unwrap();
        std::fs::write(
            case_dir.join("c.case.ad"),
            "# fault\n```test-case\ncase {\n  schema: \"autolang.case/2\",\n  id: \"av.fault\",\n  source: { file: \"main.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"program\" },\n  checks: [\n    { id: \"run\", adapter: \"probe\", action: \"run\",\n      options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\" }, steps: [ { op: \"run\" } ] },\n      expect: { completion: \"success\", exit: 7 } }\n  ]\n}\n```\n",
        )
        .unwrap();
        std::fs::write(
            case_dir.join("s.suite.ad"),
            "# s\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s\", include: [\"c.case.ad\"] }\n```\n",
        )
        .unwrap();
        let report = run_suite(&case_dir.join("s.suite.ad"), &out_dir, &Limits::defaults())
            .expect("fault suite runs");
        status_of(&report, "run")
    };

    for fault in ["exit9", "extra-frame", "wrong-hash"] {
        std::env::set_var("AV798_FAULT", fault);
        let status = run_one(fault);
        std::env::remove_var("AV798_FAULT");
        assert_eq!(
            status,
            auto_test_case::report::Status::Error,
            "fault `{fault}` must be an infra ERROR, not a language verdict"
        );
    }
    // Wall breach: a 5s delay against a 1.5s session wall — the deadline
    // is absolute and kills the tree; the outcome is infra.
    std::env::set_var("AV798_FAULT", "delay-ms:5000");
    std::env::set_var("AV798_SESSION_WALL_MS", "1500");
    let status = run_one("delay");
    std::env::remove_var("AV798_FAULT");
    std::env::remove_var("AV798_SESSION_WALL_MS");
    assert_eq!(
        status,
        auto_test_case::report::Status::Error,
        "a worker that stalls past the absolute wall is infra, never accepted ({status:?})"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R15-05: the formal runner's build PREPARER is a contained
/// executor — stdout backpressure surfaces as a stream-cap error (not a
/// wall timeout), the wall breach kills the whole TREE (descendants do
/// not survive), and the happy path still locates the real worker.
/// Driven through the AV798_BUILD_CARGO/AV798_BUILD_WALL_MS seams with a
/// fake launcher that reproduces the R15 probe shapes.
#[test]
fn ac_av_probe_prep_executor_contained_failure_shapes() {
    fn locate_with(seam_cargo: &str, wall_ms: u64) -> Result<std::path::PathBuf, String> {
        std::env::set_var("AV798_BUILD_CARGO", seam_cargo);
        std::env::set_var("AV798_BUILD_WALL_MS", wall_ms.to_string());
        let r = crate::test_cases::ac_av::tests_seam::locate_worker_for_test();
        std::env::remove_var("AV798_BUILD_CARGO");
        std::env::remove_var("AV798_BUILD_WALL_MS");
        r
    }
    let dir = std::env::temp_dir().join(format!("av798-prep-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // Shape 1: 2 MiB stdout (over the 1 MiB cap) — must be a stream-cap
    // error, never a backpressure wall timeout.
    let flood = dir.join("flood.cmd");
    std::fs::write(
        &flood,
        "@echo off\r\npowershell -NoProfile -Command \"$s='x'*4096; 1..600 | ForEach-Object { [Console]::Out.Write($s) }\"\r\n",
    )
    .unwrap();
    let wall = 60_000u64;
    match locate_with(flood.to_str().unwrap(), wall) {
        Err(e) => assert!(
            e.contains("stdout exceeded the cap"),
            "stdout cap error, got: {e}"
        ),
        Ok(_) => panic!("2MiB stdout must fail the cap"),
    }
    // Shape 2: a launcher that spawns a long-lived DESCENDANT then idles —
    // the wall breach must kill the whole tree; the descendant is gone.
    let marker = dir.join("descendant.pid");
    let _ = std::fs::remove_file(&marker);
    let spawner = dir.join("spawner.cmd");
    std::fs::write(
        &spawner,
        &format!(
            "@echo off\r\npowershell -NoProfile -Command \"$p = Start-Process powershell -ArgumentList '-NoProfile','-Command','Start-Sleep -Seconds 60' -PassThru; Set-Content -Path '{}' -Value $p.Id; Start-Sleep -Seconds 60\"\r\n",
            marker.display()
        ),
    )
    .unwrap();
    let started = std::time::Instant::now();
    match locate_with(spawner.to_str().unwrap(), 700) {
        Err(e) => assert!(e.contains("BUILD_WALL"), "wall error, got: {e}"),
        Ok(_) => panic!("idle launcher must hit the wall"),
    }
    assert!(
        started.elapsed() < std::time::Duration::from_secs(20),
        "wall enforced promptly"
    );
    // The descendant must NOT survive the tree kill.
    if let Ok(pid) = std::fs::read_to_string(&marker) {
        let pid: u32 = pid.trim().parse().unwrap_or(0);
        if pid > 0 {
            let alive = std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!("if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ 'alive' }} else {{ 'dead' }}"),
                ])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();
            assert_eq!(alive, "dead", "descendant {pid} reaped with the tree");
        }
    }
    // Shape 3 (control): the REAL happy path still builds/locates.
    std::env::remove_var("AV798_BUILD_CARGO");
    std::env::remove_var("AV798_BUILD_WALL_MS");
    let exe = crate::test_cases::ac_av::tests_seam::locate_worker_for_test()
        .expect("real locate still works");
    assert!(exe.is_file(), "worker binary at {:?}", exe);
    let _ = std::fs::remove_dir_all(&dir);
}

/// Rebase integration: ABI, sema and AV share probe/run but must never
/// share a cached identity map. Exercise all row orders through the real
/// stale adjudicator, with independently declared case/source identities.
#[test]
#[cfg(windows)]
fn ac_av_probe_mixed_recipe_stale_identity_is_order_independent() {
    use crate::test_cases::{abi, ac_av, sema, stale};

    let _guard = AV_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // Sema fault tests mutate process-wide worker settings; readers
    // share their serializer even in a bare multi-thread libtest run.
    let _sema_guard = crate::tests::probe_adapter_tests::FAULT_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("mixed-recipes");
    std::fs::write(
        root.join("all.suite.ad"),
        "# mixed\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"mixed3\", include: [\"av.case.ad\", \"sema.case.ad\", \"abi.case.ad\"] }\n```\n",
    )
    .unwrap();
    let deck = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/autoabi/src/p797-mix.atomdata"),
    )
    .unwrap();
    let cases = [
        (
            "av",
            "fn main() int { return 1 }".to_string(),
            "auto",
            r#"{ recipe: "ac-av/core-run-v1", av: { entry: "d_main" }, steps: [{ op: "run" }] }"#,
        ),
        (
            "sema",
            "fn main() int { return 1 }".to_string(),
            "auto",
            r#"{ recipe: "sema/query-v1", sema: { package: "mixed3", module: "m", kind: "infer-body", fn: "main" }, steps: [{ op: "query" }] }"#,
        ),
        (
            "abi",
            deck,
            "atom-data",
            r#"{ recipe: "abi/interop-v1", abi: { leg: "a" }, steps: [{ op: "query" }] }"#,
        ),
    ];
    for (id, source, syntax, options) in &cases {
        let virtual_path = if *syntax == "atom-data" {
            "input.atomdata"
        } else {
            "input.at"
        };
        let text = format!(
            "# {id}\n```test-case\ncase {{ schema: \"autolang.case/2\", id: \"{id}\", source: {{ text: {}, syntax: \"{syntax}\", path: \"{virtual_path}\" }}, entry: {{ mode: \"program\" }}, checks: [{{ id: \"run\", adapter: \"probe\", action: \"run\", options: {options}, expect: {{ completion: \"success\" }} }}] }}\n```\n",
            serde_json::to_string(source).unwrap()
        );
        std::fs::write(root.join(format!("{id}.case.ad")), text).unwrap();
    }
    let suite = root.join("all.suite.ad");
    let limits = Limits::defaults();
    let loaded = auto_test_case::discovery::load_suite(&suite, &limits).unwrap();
    assert!(
        loaded.errors.is_empty(),
        "all fixture cases decode: {:?}",
        loaded.errors
    );
    let identities = stale::current_identities(&suite, &limits).unwrap();
    assert_eq!(
        identities.len(),
        3,
        "all three declared checks are resolved"
    );
    let axes = [
        ac_av::current_toolchain_identity(),
        sema::current_toolchain_identity(),
        abi::current_toolchain_identity(),
    ];
    let rows: Vec<_> = cases
        .iter()
        .enumerate()
        .map(|(i, (id, ..))| {
            let hashes = &identities[&format!("{id}:run")].0;
            serde_json::json!({
                "case": id, "check": "run", "adapter": "probe",
                "identity": {
                    "suite": hashes.suite, "case_file": hashes.case_file,
                    "source": hashes.source, "oracle": hashes.oracle,
                    "contract": hashes.contract,
                },
                "native": {"toolchain": axes[i].iter().collect::<Vec<_>>()},
            })
        })
        .collect();
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let report = serde_json::json!({
            "toolchain": stale::toolchain_axes(&limits),
            "results": order.map(|i| rows[i].clone()),
        });
        let verdicts = stale::check(&suite, &report).unwrap();
        assert_eq!(verdicts.len(), 3);
        assert!(
            verdicts
                .iter()
                .all(|v| matches!(v, stale::RowVerdict::Fresh { .. })),
            "every recipe retains its own axes, order={order:?}: {verdicts:?}"
        );
    }
    for (target, axis) in [(0, "av-worker-exe-sha256"), (2, "abi-worker-exe-sha256")] {
        let mut bad = rows.clone();
        for pair in bad[target]["native"]["toolchain"].as_array_mut().unwrap() {
            if pair[0] == axis {
                pair[1] = serde_json::json!("changed-worker-content");
            }
        }
        let report = serde_json::json!({
            "toolchain": stale::toolchain_axes(&limits), "results": bad,
        });
        let verdicts = stale::check(&suite, &report).unwrap();
        for (i, verdict) in verdicts.iter().enumerate() {
            assert_eq!(
                matches!(verdict, stale::RowVerdict::Stale { .. }),
                i == target,
                "drift is confined to its recipe: {verdicts:?}"
            );
        }
    }
    let _ = std::fs::remove_dir_all(root);
}
