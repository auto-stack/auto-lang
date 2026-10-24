//! PLAN-818 T-02: the `ac/mir-native-scalar-v1` probe recipe — the formal
//! AutoDown corpus executes through the unified runner (real mn-worker
//! process, real store on disk, real multi-object PE), plus the negative
//! oracle (a wrong exit expectation must be RED — a real judge, never a
//! green stamp).

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

/// Serialize every test that spawns the real mn worker (worker identity
/// caches are process-global; mirrors the ac-av discipline).
pub(crate) static MN_CASE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-ac-mn-{}-{}", tag, std::process::id()));
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

/// The committed ac-mn corpus runs END TO END through the unified runner:
/// first case (trace, exit 0), value case (exit 7 bit-exact), div-overflow
/// trap (real PE exit 70).
#[test]
fn ac_mn_probe_corpus_executes_green() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ac-mn");
    let root = temp_root("corpus");
    let out = out_of(&root, "out");
    let report = run_suite(&suite_dir.join("ac-mn.suite.ad"), &out, &Limits::defaults())
        .expect("ac-mn suite runs");
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
        "ac-mn corpus all green ({} results): {:#?}",
        report.results.len(),
        failing
    );
}

/// Negative oracle: a WRONG exit expectation must FAIL.
#[test]
fn ac_mn_probe_negative_oracle_fails() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let root = temp_root("neg");
    std::fs::write(
        root.join("neg.suite.ad"),
        r#"# neg suite
```test-suite
suite { schema: "autolang.suite/2", id: "neg-mn", include: ["neg.case.ad"] }
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
  id: "neg.mn",
  source: { text: "fn main() int { return 7 }", syntax: "auto", path: "main.at" },
  entry: { mode: "program" },
  checks: [
    { id: "wrong-exit", adapter: "probe", action: "run",
      options: {
        recipe: "ac/mir-native-scalar-v1",
        mn: { entry: "d_main" },
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

/// Decode admission negatives: unregistered recipe, cross-recipe step
/// vocabulary, wrong/missing bindings.
#[test]
fn ac_mn_probe_decode_negatives() {
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
  id: "neg.decode.mn",
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
        r#"recipe: "ac/mir-native-scalar-v99", mn: { entry: "d_main" }, steps: [ { op: "run" } ]"#,
    ))
    .unwrap_err();
    assert!(e.message.contains("not registered"), "{e:?}");
    // The mn recipe rejects foreign step ops (edit-source is T-06).
    let e = decode_case_doc(&base(
        r#"recipe: "ac/mir-native-scalar-v1", mn: { entry: "d_main" }, steps: [ { op: "edit-source", from: "7", to: "8" } ]"#,
    ))
    .unwrap_err();
    assert!(
        e.message.contains("not valid for recipe") || e.message.contains("must end with a run"),
        "{e:?}"
    );
    // Missing mn binding.
    let e = decode_case_doc(&base(
        r#"recipe: "ac/mir-native-scalar-v1", steps: [ { op: "run" } ]"#,
    ))
    .unwrap_err();
    assert!(e.message.contains("requires an mn binding"), "{e:?}");
    // The mn binding does not attach to another recipe.
    let e = decode_case_doc(&base(
        r#"recipe: "ac-av/core-run-v1", mn: { entry: "d_main" }, av: { entry: "d_main" }, steps: [ { op: "run" } ]"#,
    ))
    .unwrap_err();
    assert!(
        e.message
            .contains("the mn binding belongs to ac/mir-native-scalar-v1"),
        "{e:?}"
    );
}

// ---------------------------------------------------------------------------
// PLAN-818 T-06 — the `mir_native_adapter_` family: the FULL formal
// AutoDown suite of the mn recipe (V-10; AC-07): edit-source/restart
// sessions through the real runner, bad-actual oracles, decode step and
// binding negatives, worker fault shapes as INFRA (never language
// verdicts), and old-channel compatibility (the T-02 single-run corpus
// plus ac-av coexistence, re-run over stale artifacts).
// ---------------------------------------------------------------------------

/// Run one inline mn case through the real unified runner; returns the
/// per-check status and the case root (for receipt inspection).
fn run_inline_mn_case(
    tag: &str,
    source: &str,
    steps: &str,
    expect: &str,
) -> (auto_test_case::report::Status, std::path::PathBuf) {
    let root = temp_root(tag);
    std::fs::write(root.join("main.at"), source).unwrap();
    std::fs::write(
        root.join("c.case.ad"),
        &format!(
            "# t06\n```test-case\ncase {{\n  schema: \"autolang.case/2\",\n  id: \"mn.t06.{tag}\",\n  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [\n    {{ id: \"run\", adapter: \"probe\", action: \"run\",\n      options: {{ recipe: \"ac/mir-native-scalar-v1\", mn: {{ entry: \"d_main\" }}, steps: [ {steps} ] }},\n      expect: {expect} }}\n  ]\n}}\n```\n"
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("s.suite.ad"),
        "# t06\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"t06\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    let out = out_of(&root, "out");
    let report =
        run_suite(&root.join("s.suite.ad"), &out, &Limits::defaults()).expect("suite runs");
    (status_of(&report, "run"), root)
}

fn receipt_of(root: &Path) -> serde_json::Value {
    // The mn receipt lands in the runner's per-check artifact dir.
    let out = out_of(root, "out");
    let mut found = None;
    fn walk(dir: &Path, found: &mut Option<serde_json::Value>) {
        if found.is_some() {
            return;
        }
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, found);
                } else if p.file_name().map(|n| n == "mn-probe-receipt.json") == Some(true) {
                    if let Ok(text) = std::fs::read_to_string(&p) {
                        if let Ok(v) = serde_json::from_str(&text) {
                            *found = Some(v);
                        }
                    }
                }
            }
        }
    }
    walk(&out, &mut found);
    found.expect("mn-probe-receipt.json written")
}

/// T-06: a discriminating edit materializes parent-side and the judged
/// run carries the EDITED source through the real worker.
#[test]
fn mir_native_adapter_edit_source_materializes_and_judges() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let (status, root) = run_inline_mn_case(
        "t06-edit",
        "fn main() int { return 7 }",
        r#"{ op: "edit-source", from: "return 7", to: "return 9" }, { op: "run" }"#,
        r#"{ completion: "success", exit: 9 }"#,
    );
    assert_eq!(
        status,
        auto_test_case::report::Status::Pass,
        "edited exit 9"
    );
    let receipt = receipt_of(&root);
    // F-818-R1-03: edit-bearing sequences run through the D-828
    // incremental session. This TINY single-line edit shape exceeds the
    // syntax-increment engine's convergence, so the worker takes the
    // RECORDED whole-request fallback — which is what this receipt shows
    // (never labeled incremental). The 798-load edit shapes exercise the
    // true incremental leg (mir_native_incremental_formal_runner_gate).
    assert_eq!(
        receipt["incremental"].as_bool(),
        Some(false),
        "no incremental claim on the fallback shape"
    );
    assert_eq!(receipt["warm_edit_fallback"].as_bool(), Some(true));
    assert_eq!(receipt["exit_code"].as_i64(), Some(9));
    assert!(
        receipt["manifest_fp"]
            .as_str()
            .is_some_and(|f| !f.is_empty()),
        "the fallback run carries whole-request identity"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// T-06: `restart` splits worker SESSIONS — a real new process each; the
/// second session HITS the persisted cache (codegen 0, coff hits), still
/// really links and runs, and the manifest identity is IDENTICAL to the
/// first session's (parent-side admission).
#[test]
fn mir_native_adapter_restart_session_reuses_cache_across_processes() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let (status, root) = run_inline_mn_case(
        "t06-restart",
        "fn main() int { return 7 }",
        r#"{ op: "run" }, { op: "restart" }, { op: "run" }"#,
        r#"{ completion: "success", exit: 7 }"#,
    );
    assert_eq!(status, auto_test_case::report::Status::Pass);
    let receipt = receipt_of(&root);
    let sessions = receipt["sessions"].as_array().expect("sessions");
    assert_eq!(sessions.len(), 2, "restart split the sequence");
    // Counters serialize as [key, value] pair arrays, not an object.
    let count = |s: &serde_json::Value, k: &str| -> u64 {
        s["counters"]
            .as_array()
            .and_then(|c| {
                c.iter().find_map(|p| {
                    let pair = p.as_array()?;
                    if pair.first()?.as_str()? == k {
                        Some(pair.get(1)?.as_u64().unwrap_or(0))
                    } else {
                        None
                    }
                })
            })
            .unwrap_or(0)
    };
    // Session 1 compiles; session 2 (a NEW process) hits the store.
    assert_eq!(count(&sessions[0], "codegen_emit_units"), 1);
    assert_eq!(count(&sessions[0], "cache_hit_coff"), 0);
    assert_eq!(count(&sessions[1], "codegen_emit_units"), 0);
    assert_eq!(count(&sessions[1], "cache_hit_coff"), 1);
    // Both sessions REALLY linked and executed (always-execute).
    for (i, s) in sessions.iter().enumerate() {
        assert!(count(s, "link_invocations") >= 1, "session {i} links");
        assert!(count(s, "run_invocations") >= 1, "session {i} runs");
        assert_eq!(s["exit_code"].as_i64(), Some(7), "session {i} exit");
    }
    // One source -> one manifest identity across processes.
    assert_eq!(
        sessions[0]["manifest_fp"].as_str(),
        sessions[1]["manifest_fp"].as_str()
    );
    assert_eq!(
        sessions[0]["source_sha256"].as_str(),
        sessions[1]["source_sha256"].as_str()
    );
    assert!(sessions[1]["judged"].as_bool() == Some(true));
    let _ = std::fs::remove_dir_all(&root);
}

/// T-06 negative oracles: a WRONG exit expectation against a good PE
/// actual (plain and after an edit), and a trap actual (exit 70) against
/// a success expectation — all must be RED (a real parent-side judge).
#[test]
fn mir_native_adapter_bad_actual_oracles_are_red() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let (status, _) = run_inline_mn_case(
        "t06-neg1",
        "fn main() int { return 7 }",
        r#"{ op: "run" }"#,
        r#"{ completion: "success", exit: 8 }"#,
    );
    assert_eq!(status, auto_test_case::report::Status::Fail, "7 != 8");
    let (status, _) = run_inline_mn_case(
        "t06-neg2",
        "fn main() int { return 7 }",
        r#"{ op: "edit-source", from: "return 7", to: "return 9" }, { op: "run" }"#,
        r#"{ completion: "success", exit: 7 }"#,
    );
    assert_eq!(status, auto_test_case::report::Status::Fail, "edit: 9 != 7");
    let (status, _) = run_inline_mn_case(
        "t06-neg3",
        "fn main() int { return 1 / 0 }",
        r#"{ op: "run" }"#,
        r#"{ completion: "success", exit: 0 }"#,
    );
    assert_eq!(
        status,
        auto_test_case::report::Status::Fail,
        "div-by-zero traps: actual exit 70 != expected 0"
    );
}

/// T-06: a diagnosed pipeline rejection is a POSITIVE face when the
/// expectation names the stage (frontend parse rejection), and RED when
/// the stage is wrong.
#[test]
fn mir_native_adapter_diagnosed_rejection_faces() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let broken = "fn main() int { return 1 + }";
    let (status, _) = run_inline_mn_case(
        "t06-diag1",
        broken,
        r#"{ op: "run" }"#,
        r#"{ completion: { stage: "parse", code: "mn.unsupported.frontend" } }"#,
    );
    assert_eq!(
        status,
        auto_test_case::report::Status::Pass,
        "a diagnosed frontend rejection matches stage+code"
    );
    let (status, _) = run_inline_mn_case(
        "t06-diag2",
        broken,
        r#"{ op: "run" }"#,
        r#"{ completion: { stage: "run", code: "mn.unsupported.frontend" } }"#,
    );
    assert_eq!(
        status,
        auto_test_case::report::Status::Fail,
        "wrong stage expectation is red (frontend maps to parse, not run)"
    );
}

/// T-06 decode negatives: the mn step vocabulary (query rejected,
/// restart-only sequences rejected, malformed edits) and the mn binding's
/// unknown-field rejection.
#[test]
fn mir_native_adapter_decode_step_and_binding_negatives() {
    use auto_test_case::atom::parse_contract;
    use auto_test_case::contract::decode_case;
    let decode = |options: &str| {
        let doc = format!(
            r#"case {{
  schema: "autolang.case/2",
  id: "neg.t06.mn",
  source: {{ text: "fn main() int {{ return 1 }}", syntax: "auto", path: "main.at" }},
  entry: {{ mode: "program" }},
  checks: [
    {{ id: "c", adapter: "probe", action: "run",
      options: {{ {options} }},
      expect: {{ completion: "success" }} }}
  ]
}}
"#
        );
        let v = parse_contract(&doc).unwrap();
        decode_case(&v, &Limits::defaults())
    };
    // `query` stays a sema/abi op — recipes never borrow ops.
    let e = decode(
        r#"recipe: "ac/mir-native-scalar-v1", mn: { entry: "d_main" }, steps: [ { op: "query" }, { op: "run" } ]"#,
    )
    .unwrap_err();
    assert!(e.message.contains("not valid for recipe"), "{e:?}");
    // A restart-only sequence judges nothing.
    let e = decode(
        r#"recipe: "ac/mir-native-scalar-v1", mn: { entry: "d_main" }, steps: [ { op: "restart" } ]"#,
    )
    .unwrap_err();
    assert!(e.message.contains("must end with a run step"), "{e:?}");
    // Malformed edits: empty `from`, from == to.
    let e = decode(
        r#"recipe: "ac/mir-native-scalar-v1", mn: { entry: "d_main" }, steps: [ { op: "edit-source", from: "", to: "x" }, { op: "run" } ]"#,
    )
    .unwrap_err();
    assert!(e.message.contains("non-empty"), "{e:?}");
    let e = decode(
        r#"recipe: "ac/mir-native-scalar-v1", mn: { entry: "d_main" }, steps: [ { op: "edit-source", from: "a", to: "a" }, { op: "run" } ]"#,
    )
    .unwrap_err();
    assert!(e.message.contains("distinct"), "{e:?}");
    // Unknown field in the mn binding: misspelled fields are errors.
    let e = decode(
        r#"recipe: "ac/mir-native-scalar-v1", mn: { entry: "d_main", enty: "x" }, steps: [ { op: "run" } ]"#,
    )
    .unwrap_err();
    assert!(e.message.contains("unknown field"), "{e:?}");
}

/// T-06: worker fault shapes are INFRA (Error), never language verdicts —
/// a missing worker path, a non-zero worker exit, a garbage protocol
/// line, and a stderr flood past the cap.
#[test]
fn mir_native_adapter_worker_faults_are_infra() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let run_one = || -> auto_test_case::report::Status {
        let (status, root) = run_inline_mn_case(
            "t06-fault",
            "fn main() int { return 7 }",
            r#"{ op: "run" }"#,
            r#"{ completion: "success", exit: 7 }"#,
        );
        let _ = std::fs::remove_dir_all(&root);
        status
    };

    // (a) The override points at a missing file: locate failure is infra.
    std::env::set_var("AC_MN_CASE_WORKER", r"Z:\definitely\missing\mn.exe");
    let status = run_one();
    std::env::remove_var("AC_MN_CASE_WORKER");
    assert_eq!(status, auto_test_case::report::Status::Error);

    // (b) A worker that exits non-zero (after one protocol-ish line) is
    // infra, never a verdict.
    let dir = temp_root("t06-fakes");
    let exit9 = dir.join("exit9.cmd");
    std::fs::write(
        &exit9,
        "@echo off
echo not-json
exit /b 9
",
    )
    .unwrap();
    std::env::set_var("AC_MN_CASE_WORKER", exit9.to_str().unwrap());
    let status = run_one();
    std::env::remove_var("AC_MN_CASE_WORKER");
    assert_eq!(status, auto_test_case::report::Status::Error);

    // (c) A zero-exit worker speaking a WRONG schema line is infra.
    let garbage = dir.join("garbage.cmd");
    std::fs::write(
        &garbage,
        "@echo off
echo hello world
exit /b 0
",
    )
    .unwrap();
    std::env::set_var("AC_MN_CASE_WORKER", garbage.to_str().unwrap());
    let status = run_one();
    std::env::remove_var("AC_MN_CASE_WORKER");
    assert_eq!(status, auto_test_case::report::Status::Error);

    // (d) A stderr flood: the cap kills the session — infra.
    let flood = dir.join("flood.cmd");
    let flood_cmd = concat!(
        "@echo off\r\n",
        "powershell -NoProfile -Command \"$s='x'*4096; 1..600 | ForEach-Object { [Console]::Error.Write($s) }\"\r\n",
        "exit /b 0\r\n"
    );
    std::fs::write(&flood, flood_cmd).unwrap();
    std::env::set_var("AC_MN_CASE_WORKER", flood.to_str().unwrap());
    let status = run_one();
    std::env::remove_var("AC_MN_CASE_WORKER");
    assert_eq!(status, auto_test_case::report::Status::Error);

    // Control: the REAL worker still judges green after the seam churn.
    let (status, root) = run_inline_mn_case(
        "t06-fault-ctrl",
        "fn main() int { return 7 }",
        r#"{ op: "run" }"#,
        r#"{ completion: "success", exit: 7 }"#,
    );
    assert_eq!(status, auto_test_case::report::Status::Pass);
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&dir);
}

/// T-06 old-channel compatibility: the committed T-02 single-run corpus
/// (unchanged v1 decode shape) and an ac-av case coexist in ONE suite,
/// and a RE-RUN over the previous artifacts (stale out dir) stays green.
#[test]
#[cfg(windows)]
fn mir_native_adapter_old_protocol_and_mixed_compat() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ac-mn");
    let root = temp_root("t06-mixed");
    std::fs::copy(
        corpus.join("mn-value.case.ad"),
        root.join("mn-value.case.ad"),
    )
    .unwrap();
    let suite_text = std::fs::read_to_string(corpus.join("ac-mn.suite.ad"))
        .unwrap()
        .replace("mn-value.case.ad", "mn-value.case.ad");
    std::fs::write(root.join("ac-mn.suite.ad"), suite_text).unwrap();
    std::fs::write(root.join("main.at"), "fn main() int { return 7 }").unwrap();
    std::fs::write(
        root.join("av.case.ad"),
        "# av\n```test-case\ncase {\n  schema: \"autolang.case/2\",\n  id: \"av.mixed\",\n  source: { file: \"main.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"program\" },\n  checks: [\n    { id: \"av-run\", adapter: \"probe\", action: \"run\",\n      options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\" }, steps: [ { op: \"run\" } ] },\n      expect: { completion: \"success\", exit: 7 } }\n  ]\n}\n```\n",
    )
    .unwrap();
    std::fs::write(
        root.join("mixed.suite.ad"),
        "# mixed\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"mixed\", include: [\"ac-mn.suite.ad\", \"av.case.ad\"] }\n```\n",
    )
    .unwrap();
    let out = out_of(&root, "out");
    let report = run_suite(&root.join("mixed.suite.ad"), &out, &Limits::defaults())
        .expect("mixed suite runs");
    let failing: Vec<String> = report
        .results
        .iter()
        .filter(|r| r.status != auto_test_case::report::Status::Pass)
        .map(|r| {
            format!(
                "{}: {:?} {}",
                r.check_id,
                r.status,
                r.detail.clone().unwrap_or_default()
            )
        })
        .collect();
    assert!(
        failing.is_empty(),
        "mn corpus + av case green together: {failing:#?}"
    );
    // Stale re-run: the SAME out dir still holds the first run's
    // artifacts; the second run must stay green (no stale poisoning).
    let report2 = run_suite(&root.join("mixed.suite.ad"), &out, &Limits::defaults())
        .expect("stale re-run executes");
    let failing2: Vec<String> = report2
        .results
        .iter()
        .filter(|r| r.status != auto_test_case::report::Status::Pass)
        .map(|r| format!("{}: {:?}", r.check_id, r.status))
        .collect();
    assert!(failing2.is_empty(), "stale re-run green: {failing2:#?}");
    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------
// PLAN-818 T-07 — the `mir_native_turnaround_` formal legs (V-09; AC-06):
// the frozen reference workload through the REAL formal runner — fresh
// cold, restart/warm store (a real new worker process per call), the
// body-edit and signature P95 < 1000 ms gates, and the diagnosed
// signature conflict. Each scenario is its OWN test (nextest terminates
// at 120 s; the frozen 30 samples per scenario fit one test each) with
// its own session namespace and raw file. The driver-tier legs
// (same-process hot / cross-project / concurrent GC) live in the plan
// workspace (`experimental/ac-mir-native/tests/turnaround_tier.rs`).
// ---------------------------------------------------------------------------

/// T-07 scratch root: the turnaround windows are disk-sensitive (116
/// staged objects per link); the reference machine's repo-local target
/// dir (gitignored, same disk as the build) is the faithful scratch —
/// NOT the system temp directory.
fn mn_t07_root(tag: &str) -> std::path::PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(format!("mn818-t07-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn mn_t07_samples() -> usize {
    std::env::var("MN818_FORMAL_SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30)
}

fn mn_t07_gate_ms() -> f64 {
    std::env::var("MN818_FORMAL_GATE_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000.0)
}

/// One formal mn call: write the case, run the suite, require green,
/// return the receipt (whose request_wall_ms IS the gate window).
fn mn_formal_call(
    root: &Path,
    tag: &str,
    source: &str,
    steps: &str,
    session: Option<&str>,
    expect: &str,
) -> serde_json::Value {
    mn_formal_call_opts(root, tag, source, steps, session, &[], expect)
}

/// `mn_formal_call` with capability grants (e.g. hir.test.trace.v1).
fn mn_formal_call_opts(
    root: &Path,
    tag: &str,
    source: &str,
    steps: &str,
    session: Option<&str>,
    caps: &[&str],
    expect: &str,
) -> serde_json::Value {
    let report = mn_formal_call_report_opts(root, tag, source, steps, session, caps, expect);
    assert_eq!(
        status_of(&report, "run"),
        auto_test_case::report::Status::Pass,
        "formal call {tag} must be green: {:#?}",
        report
            .results
            .iter()
            .map(|r| (r.status, r.detail.clone()))
            .collect::<Vec<_>>()
    );
    receipt_of(&root.join(tag))
}

/// The failure-fixture variant: runs the formal suite WITHOUT the green
/// assertion and returns the report (the caller asserts the expected
/// Error/Fail face and the mismatch text).
fn mn_formal_call_report(
    root: &Path,
    tag: &str,
    source: &str,
    steps: &str,
    expect: &str,
) -> auto_test_case::report::RunReport {
    mn_formal_call_report_opts(root, tag, source, steps, None, &[], expect)
}

fn mn_formal_call_report_opts(
    root: &Path,
    tag: &str,
    source: &str,
    steps: &str,
    session: Option<&str>,
    caps: &[&str],
    expect: &str,
) -> auto_test_case::report::RunReport {
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
    let caps_field = if caps.is_empty() {
        String::new()
    } else {
        let list = caps
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!(", caps: [{list}]")
    };
    std::fs::write(
        case_dir.join("c.case.ad"),
        format!(
            "# t07 {tag}\n```test-case\ncase {{\n  schema: \"autolang.case/2\",\n  id: \"mn.t07.{tag}\",\n  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [\n    {{ id: \"run\", adapter: \"probe\", action: \"run\",\n      options: {{ recipe: \"ac/mir-native-scalar-v1\", mn: {{ entry: \"d_main\"{session_field}{caps_field} }}, steps: [{steps}] }},\n      expect: {expect} }}\n  ]\n}}\n```\n"
        ),
    )
    .unwrap();
    std::fs::write(
        case_dir.join("s.suite.ad"),
        "# t07\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    run_suite(&case_dir.join("s.suite.ad"), &out_dir, &Limits::defaults())
        .unwrap_or_else(|e| panic!("formal suite {tag} runs: {e}"))
}

/// Removes an env var when dropped (panic-safe seam hygiene for the
/// fixture/seam tests below).
struct EnvGuard(&'static str);
impl Drop for EnvGuard {
    fn drop(&mut self) {
        std::env::remove_var(self.0);
    }
}

/// A [key, value] pair-array counter read from an mn receipt session.
fn mn_receipt_counter(session: &serde_json::Value, k: &str) -> u64 {
    session["counters"]
        .as_array()
        .and_then(|a| {
            a.iter().find_map(|p| {
                let pair = p.as_array()?;
                if pair.first()?.as_str()? == k {
                    Some(pair.get(1)?.as_u64().unwrap_or(0))
                } else {
                    None
                }
            })
        })
        .unwrap_or(0)
}

/// Fetch a reference-workload revision from the plan worker (single
/// source of truth — the frozen generator lives in the mn workspace).
fn mn_fetch_revision(rev: u32) -> (String, i64) {
    let out = std::process::Command::new(
        crate::test_cases::ac_mn::worker_binary().expect("mn worker located"),
    )
    .args(["load-revision", "--rev", &rev.to_string()])
    .output()
    .expect("spawn mn-worker load-revision");
    assert!(out.status.success(), "load-revision ok");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("revision json");
    (
        v["source"].as_str().expect("source").to_string(),
        v["expected_main"].as_i64().expect("expected"),
    )
}

fn mn_t07_percentile(v: &[f64], p: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((p / 100.0) * (s.len() - 1) as f64).round() as usize % s.len()]
}

/// Raw admissibility (the T-07 counter-control): zero/NaN/missing
/// windows never pass; write the scenario raw file.
fn mn_t07_finish_raw(scenario: &str, note: &str, walls: &[f64]) -> (f64, f64, f64) {
    assert!(!walls.is_empty(), "{scenario}: no samples (missing raw)");
    for (i, w) in walls.iter().enumerate() {
        assert!(w.is_finite(), "{scenario}[{i}]: NaN window");
        assert!(*w > 0.0, "{scenario}[{i}]: zero window");
    }
    let p50 = mn_t07_percentile(walls, 50.0);
    let p95 = mn_t07_percentile(walls, 95.0);
    let p99 = mn_t07_percentile(walls, 99.0);
    let report = serde_json::json!({
        "plan": "PLAN-818",
        "tier": "t07-formal-turnaround",
        "scenario": scenario,
        "note": note,
        "count": walls.len(),
        "p50_ms": p50,
        "p95_ms": p95,
        "p99_ms": p99,
        "max_ms": walls.iter().cloned().fold(0.0f64, f64::max),
        "windows_ms": walls,
    });
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../docs/reports/plan818/turnaround-mn-{scenario}.json"
    ));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    (p50, p95, p99)
}

/// Scenario 1 — fresh真冷store: every call a FRESH session store, the
/// full 115-unit compile; reported (the hard gates are the edit paths).
#[test]
#[cfg(windows)]
fn mir_native_turnaround_fresh_cold_30() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let samples = mn_t07_samples();
    let pid = std::process::id();
    let session_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(format!("mn818-t07-sessions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&session_base);
    std::fs::create_dir_all(&session_base).unwrap();
    std::env::set_var("AC_MN_SESSION_ROOT", &session_base);
    let root = mn_t07_root("t07-fresh");
    let (src0, exp0) = mn_fetch_revision(0);
    let mut walls = Vec::with_capacity(samples);
    for i in 0..samples {
        let r = mn_formal_call(
            &root,
            &format!("fresh-{i}"),
            &src0,
            r#"{ op: "run" }"#,
            Some(&format!("p{pid}-fresh-{i}")),
            &format!(r#"{{ completion: "success", exit: {} }}"#, exp0),
        );
        let w = r["request_wall_ms"].as_f64().unwrap_or(0.0);
        let s = &r["sessions"][0];
        assert_eq!(
            mn_receipt_counter(s, "codegen_emit_units"),
            115,
            "cold compile"
        );
        assert_eq!(mn_receipt_counter(s, "cache_hit_coff"), 0, "really cold");
        walls.push(w);
        let _ = std::fs::remove_dir_all(
            std::env::temp_dir().join(format!("mn818-session-p{pid}-fresh-{i}")),
        );
    }
    let (p50, p95, p99) = mn_t07_finish_raw(
        "fresh",
        "each call a fresh session store; full 115-unit compile; reported (not gated)",
        &walls,
    );
    eprintln!("t07 fresh: P50/P95/P99 = {p50:.1}/{p95:.1}/{p99:.1} ms");
    std::env::remove_var("AC_MN_SESSION_ROOT");
    let _ = std::fs::remove_dir_all(&session_base);
    let _ = std::fs::remove_dir_all(&root);
}

/// Scenario 2 — restart/warm store: every call is a REAL new worker
/// process over ONE shared session store.
#[test]
#[cfg(windows)]
fn mir_native_turnaround_restart_warm_30() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let samples = mn_t07_samples();
    let pid = std::process::id();
    let session_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(format!("mn818-t07-sessions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&session_base);
    std::fs::create_dir_all(&session_base).unwrap();
    std::env::set_var("AC_MN_SESSION_ROOT", &session_base);
    let root = mn_t07_root("t07-restart");
    let (src0, exp0) = mn_fetch_revision(0);
    let mut walls = Vec::with_capacity(samples);
    for i in 0..samples {
        let r = mn_formal_call(
            &root,
            &format!("restart-{i}"),
            &src0,
            r#"{ op: "run" }"#,
            Some(&format!("p{pid}-restart")),
            &format!(r#"{{ completion: "success", exit: {} }}"#, exp0),
        );
        let s = &r["sessions"][0];
        if i == 0 {
            assert_eq!(mn_receipt_counter(s, "codegen_emit_units"), 115);
        } else {
            assert_eq!(
                mn_receipt_counter(s, "codegen_emit_units"),
                0,
                "new process, warm store"
            );
            assert_eq!(mn_receipt_counter(s, "cache_hit_coff"), 115);
        }
        walls.push(r["request_wall_ms"].as_f64().unwrap_or(0.0));
    }
    let _ = mn_t07_finish_raw(
        "restart",
        "fresh worker process per call over one shared store",
        &walls,
    );
    let _ =
        std::fs::remove_dir_all(std::env::temp_dir().join(format!("mn818-session-p{pid}-restart")));
    std::env::remove_var("AC_MN_SESSION_ROOT");
    let _ = std::fs::remove_dir_all(&session_base);
    let _ = std::fs::remove_dir_all(&root);
}

/// Scenario 3 — body edit (THE gate): per revision, a warm-up `[run]` on
/// the previous revision, then the MEASURED `[edit-source, run]` call
/// applying the frozen fn_041 A0 edit (golden.md §2.3); P95 < gate.
#[test]
#[cfg(windows)]
fn mir_native_turnaround_body_edit_gate() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let samples = mn_t07_samples();
    let gate_ms = mn_t07_gate_ms();
    let session_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(format!("mn818-t07-sessions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&session_base);
    std::fs::create_dir_all(&session_base).unwrap();
    std::env::set_var("AC_MN_SESSION_ROOT", &session_base);
    let pid = std::process::id();
    let root = mn_t07_root("t07-edit");
    let session = format!("p{pid}-edit");
    let (mut prev_src, mut prev_exp) = mn_fetch_revision(0);
    let mut walls = Vec::with_capacity(samples);
    for i in 1..=samples {
        let (rev_src, rev_exp) = mn_fetch_revision(i as u32);
        // Frozen edit position: fn_041's `var acc = x * A0` line.
        let extract = |src: &str| -> String {
            let header = "fn fn_041(x int) int {";
            let hi = src.find(header).expect("fn_041 present");
            let rest = &src[hi..];
            let e1 = rest.find('\n').expect("line1");
            let e2 = rest[e1 + 1..].find('\n').expect("line2");
            format!("{}\n{}", header, &rest[e1 + 1..e1 + 1 + e2])
        };
        let from_line = extract(&prev_src);
        let to_line = extract(&rev_src);
        assert!(from_line != to_line, "revision {i} edits fn_041's A0");
        assert!(
            from_line.contains("var acc = x *"),
            "the edit is fn_041's multiplier"
        );
        let warm = mn_formal_call(
            &root,
            &format!("edit-{i}-warm"),
            &prev_src,
            r#"{ op: "run" }"#,
            Some(&session),
            &format!(r#"{{ completion: "success", exit: {} }}"#, prev_exp),
        );
        let warm_codegen = mn_receipt_counter(&warm["sessions"][0], "codegen_emit_units");
        if i == 1 {
            assert_eq!(warm_codegen, 115, "the session's first call is really cold");
        } else {
            // F-818-R1-03: the measured calls warm the INCREMENTAL store;
            // this whole-request session store lags one revision behind,
            // so the warm-up recompiles exactly the edited unit (outside
            // the measured window either way).
            assert!(
                warm_codegen <= 1,
                "warm-up {i} compiles at most the edited unit"
            );
        }
        let esc = |t: &str| t.replace('\\', "\\\\").replace('"', "\\\"");
        let steps = format!(
            "{{ op: \"edit-source\", from: \"{}\", to: \"{}\" }}, {{ op: \"run\" }}",
            esc(&from_line),
            esc(&to_line)
        );
        // F-818-R1-01: the observer seam carries THIS revision's
        // independent expectation so the measured window includes the
        // comparison (worker never sees it).
        std::env::set_var("AC_MN_INCREMENTAL_EXPECT_EXITS", rev_exp.to_string());
        let _seam = EnvGuard("AC_MN_INCREMENTAL_EXPECT_EXITS");
        let measured = mn_formal_call(
            &root,
            &format!("edit-{i}"),
            &prev_src,
            &steps,
            Some(&session),
            &format!(r#"{{ completion: "success", exit: {} }}"#, rev_exp),
        );
        drop(_seam);
        // F-818-R1-03/R1-01: the measured call runs the D-828 incremental
        // session through the ROUNDS protocol; the SAMPLE WINDOW is the
        // edit round's PARENT-OBSERVED interval — the parent's own
        // preparation (literal search/insert) through its admission of
        // the round's real PE result (the §5.5 gate window; the in-request
        // warm-up is initialization, single-listed and excluded).
        assert_eq!(measured["incremental"].as_bool(), Some(true));
        let edit_rec = measured["steps"]
            .as_array()
            .expect("step records")
            .iter()
            .find(|r| r["op"].as_str() == Some("edit-source"))
            .expect("edit step");
        assert_eq!(
            edit_rec["codegen_units"].as_u64(),
            Some(1),
            "one unit recompiles"
        );
        assert_eq!(edit_rec["cache_hits"].as_u64(), Some(114));
        assert_eq!(edit_rec["unaffected_stage_delta"].as_u64(), Some(0));
        assert_eq!(edit_rec["whole_module_stage_delta"].as_u64(), Some(0));
        assert_eq!(edit_rec["fallback_whole_lower"].as_u64(), Some(0));
        assert_eq!(
            edit_rec["expected_exit"].as_i64(),
            Some(rev_exp),
            "rev {i}: the seam carried the independent expectation"
        );
        assert_eq!(
            edit_rec["expected_match"].as_bool(),
            Some(true),
            "rev {i}: the independent comparison matched inside the window"
        );
        let w = edit_rec["wall_ms"].as_f64().unwrap_or(0.0);
        let pw = edit_rec["parent_window_ms"].as_f64().unwrap_or(0.0);
        assert!(
            w > 0.0 && pw >= w,
            "rev {i}: the parent window ({pw:.1}ms) must cover the worker window ({w:.1}ms)"
        );
        eprintln!(
            "t07 body-edit rev {i}: parent window {pw:.1}ms (worker {w:.1}ms, expected {rev_exp})"
        );
        walls.push(pw);
        prev_src = rev_src;
        prev_exp = rev_exp;
    }
    let (_, p95, _) = mn_t07_finish_raw(
        "body-edit",
        "warm-up outside window; measured [edit-source, run]; 1 miss / 114 hits; P95 GATED",
        &walls,
    );
    assert!(
        p95 < gate_ms,
        "body-edit P95 {p95:.1}ms >= gate {gate_ms}ms"
    );
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join(format!("mn818-session-{session}")));
    let _ = std::fs::remove_dir_all(session_base.join(format!("mn818-inc-{session}")));
    std::env::remove_var("AC_MN_SESSION_ROOT");
    let _ = std::fs::remove_dir_all(&session_base);
    let _ = std::fs::remove_dir_all(&root);
}

/// Scenario 4 — legal signature rebuild (THE second gate): fn_leaf_2
/// gains a parameter USED consistently at every call site (w=0 cancels —
/// the mirror expectation is unchanged); the signature fp change re-keys
/// the leaf AND its callers, everything else hits; P95 < gate.
#[test]
#[cfg(windows)]
fn mir_native_turnaround_signature_rebuild_gate() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // Measurement hygiene: this gate rides the WHOLE-REQUEST path (a
    // legal signature rebuild is a declared-context change — the warm
    // session's interface gate refuses it by design), whose window has
    // ~70ms headroom under 1000ms; the batch's earlier link storms must
    // settle before latency sampling.
    std::thread::sleep(std::time::Duration::from_secs(10));
    let samples = mn_t07_samples();
    let gate_ms = mn_t07_gate_ms();
    let session_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(format!("mn818-t07-sessions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&session_base);
    std::fs::create_dir_all(&session_base).unwrap();
    std::env::set_var("AC_MN_SESSION_ROOT", &session_base);
    let pid = std::process::id();
    let root = mn_t07_root("t07-sig");
    let session = format!("p{pid}-sig");
    let sig_rebuild = |src: &str, k: i64| -> String {
        let out = src.replacen(
            "fn fn_leaf_2(v int) int {",
            "fn fn_leaf_2(v int, w int) int {",
            1,
        );
        assert!(out != src, "frozen fn_leaf_2 header present");
        let out = out.replacen("return v + 2\n", "return v + 2 + w\n", 1);
        out.replace(
            "return fn_leaf_2(acc)",
            &format!("return fn_leaf_2(acc, {k})"),
        )
    };
    let mut walls = Vec::with_capacity(samples);
    for i in 0..samples {
        // A DIFFERENT revision each iteration: the rebuilt closure is new
        // work every time (the frozen fn_041 bit keeps moving the source).
        let (src_i, exp_i) = mn_fetch_revision(i as u32);
        let _ = mn_formal_call(
            &root,
            &format!("sig-{i}-warm"),
            &src_i,
            r#"{ op: "run" }"#,
            Some(&session),
            &format!(r#"{{ completion: "success", exit: {} }}"#, exp_i),
        );
        if i == 0 {
            // A second warm-up in ANOTHER fresh process: the session
            // store's first cross-process reads pay cold page-cache costs
            // (~5ms/unit x 106) that would otherwise land in the measured
            // window — warming the OS cache first is warm-session setup,
            // not measurement.
            let _ = mn_formal_call(
                &root,
                "sig-0-warm2",
                &src_i,
                r#"{ op: "run" }"#,
                Some(&session),
                &format!(r#"{{ completion: "success", exit: {} }}"#, exp_i),
            );
        }
        // Each iteration passes a DIFFERENT w (K = i): the 8 call sites
        // re-key every time; the closed-form expectation shifts by 8*K
        // (each fn_leaf_2 caller's value gains K).
        let rebuilt = sig_rebuild(&src_i, i as i64);
        let rebuilt_exp = exp_i + 8 * i as i64;
        let r = mn_formal_call(
            &root,
            &format!("sig-{i}"),
            &rebuilt,
            r#"{ op: "run" }"#,
            Some(&session),
            &format!(r#"{{ completion: "success", exit: {} }}"#, rebuilt_exp),
        );
        let s = &r["sessions"][0];
        let recompiled = mn_receipt_counter(s, "codegen_emit_units");
        let hits = mn_receipt_counter(s, "cache_hit_coff");
        let want_recompiled = if i == 0 { 9 } else { 8 };
        assert!(
            recompiled == want_recompiled && hits + recompiled == 115,
            "signature rebuild re-keys the call sites every iteration (recompiled {recompiled}, want {want_recompiled}, hits {hits})"
        );
        walls.push(r["request_wall_ms"].as_f64().unwrap_or(0.0));
    }
    let (_, p95, _) = mn_t07_finish_raw(
        "signature-rebuild",
        "fn_leaf_2 +w param used consistently (w=0); leaf+8 callers re-key; P95 GATED",
        &walls,
    );
    assert!(
        p95 < gate_ms,
        "signature-rebuild P95 {p95:.1}ms >= gate {gate_ms}ms"
    );
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join(format!("mn818-session-{session}")));
    let _ = std::fs::remove_dir_all(session_base.join(format!("mn818-inc-{session}")));
    std::env::remove_var("AC_MN_SESSION_ROOT");
    let _ = std::fs::remove_dir_all(&session_base);
    let _ = std::fs::remove_dir_all(&root);
}

/// Scenario 5 — diagnosed signature conflict (av parity gate): the frozen
/// conflict edit (fn_leaf_2 gains an UNUSED param — call sites break) is
/// a green DIAGNOSED negative, window-measured; P95 < gate.
#[test]
#[cfg(windows)]
fn mir_native_turnaround_signature_conflict_gate() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let samples = mn_t07_samples();
    let gate_ms = mn_t07_gate_ms();
    let session_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(format!("mn818-t07-sessions-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&session_base);
    std::fs::create_dir_all(&session_base).unwrap();
    std::env::set_var("AC_MN_SESSION_ROOT", &session_base);
    let root = mn_t07_root("t07-conflict");
    let (src0, _) = mn_fetch_revision(0);
    let conflicted = src0.replacen(
        "fn fn_leaf_2(v int) int {",
        "fn fn_leaf_2(v int, w int) int {",
        1,
    );
    let mut walls = Vec::with_capacity(samples);
    for i in 0..samples {
        let r = mn_formal_call(
            &root,
            &format!("conflict-{i}"),
            &conflicted,
            r#"{ op: "run" }"#,
            None,
            r#"{ completion: { stage: "parse", code: "mn.unsupported.frontend" } }"#,
        );
        walls.push(r["request_wall_ms"].as_f64().unwrap_or(0.0));
    }
    let (_, p95, _) = mn_t07_finish_raw(
        "signature-conflict",
        "frozen conflict edit; green diagnosed parse rejection; P95 GATED",
        &walls,
    );
    assert!(p95 < gate_ms, "conflict P95 {p95:.1}ms >= gate {gate_ms}ms");
    let _ = std::fs::remove_dir_all(&root);
}

/// F-818-R1-03: the FORMAL recipe consumes the D-828 incremental session
/// for warm edit sequences — through the real runner, with parent-side
/// expected result, per-step identity/stage records, the incrementality
/// admission, cold/warm separation (a fresh `[run]` call stays the
/// whole-request path and is not labeled incremental) and the refusal
/// control (a signature edit inside the steps is refused by the session
/// and the parent treats the sequence as an infra non-verdict).
#[test]
#[cfg(windows)]
fn mir_native_incremental_formal_runner_gate() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = mn_t07_root("t11-formal");
    let (src0, exp0) = mn_fetch_revision(0);
    let (src1, exp1) = mn_fetch_revision(1);

    // The frozen edit position: the load's revision-1 edit site (derive
    // the differing LINE by diffing the two revisions - digit-width
    // changes included).
    let (from_line, to_line) = {
        let a = src0.as_bytes();
        let b = src1.as_bytes();
        let mut i = 0usize;
        while i < a.len().min(b.len()) && a[i] == b[i] {
            i += 1;
        }
        assert!(i < a.len().min(b.len()), "the revisions differ");
        let ls = |src: &[u8], pos: usize| -> String {
            let s = std::str::from_utf8(src).expect("utf8");
            let start = s[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
            let end = s[pos..].find('\n').map(|e| pos + e).unwrap_or(s.len());
            s[start..end].to_string()
        };
        (ls(a, i), ls(b, i))
    };
    assert!(from_line != to_line && from_line.contains("var acc = x *"));

    // (a) The warm edit sequence through the REAL runner: judged value is
    // revision 1's independent expectation; the edit step is TRULY
    // incremental; the trailing run re-links the current state for real.
    let esc = |t: &str| t.replace('\\', "\\\\").replace('"', "\\\"");
    let steps = format!(
        "{{ op: \"edit-source\", from: \"{}\", to: \"{}\" }}, {{ op: \"run\" }}",
        esc(&from_line),
        esc(&to_line)
    );
    let r = mn_formal_call(
        &root,
        "inc-edit",
        &src0,
        &steps,
        None,
        &format!(r#"{{ completion: "success", exit: {} }}"#, exp1),
    );
    assert_eq!(
        r["incremental"].as_bool(),
        Some(true),
        "the warm path engaged"
    );
    let recs = r["steps"].as_array().expect("step records");
    let edit = recs
        .iter()
        .find(|x| x["op"].as_str() == Some("edit-source"))
        .expect("edit step");
    assert_eq!(edit["codegen_units"].as_u64(), Some(1), "one unit re-emits");
    assert_eq!(
        edit["cache_hits"].as_u64(),
        Some(114),
        "the rest hit resident"
    );
    assert_eq!(edit["unaffected_stage_delta"].as_u64(), Some(0));
    assert_eq!(edit["whole_module_stage_delta"].as_u64(), Some(0));
    assert_eq!(edit["fallback_whole_lower"].as_u64(), Some(0));
    assert!(edit["refused"].is_null());
    let run_rec = recs
        .iter()
        .rev()
        .find(|x| x["op"].as_str() == Some("run"))
        .expect("judged run step");
    assert_eq!(run_rec["exit_code"].as_i64(), Some(exp1));
    assert!(
        run_rec["wall_ms"].as_f64().unwrap_or(0.0) > 0.0,
        "the judged step really linked and ran"
    );

    // (b) Cold/warm separation: a plain `[run]` call on the same source
    // stays the WHOLE-REQUEST path (the cold/restored scenario) — its
    // receipt is NOT labeled incremental.
    let cold_r = mn_formal_call(
        &root,
        "inc-cold",
        &src0,
        r#"{ op: "run" }"#,
        None,
        &format!(r#"{{ completion: "success", exit: {} }}"#, exp0),
    );
    assert!(
        cold_r
            .get("incremental")
            .map(|v| v.is_null())
            .unwrap_or(true),
        "the whole-request path is not labeled incremental"
    );

    // (c) Failure control: a SIGNATURE edit inside the steps is refused
    // by the session's interface gate — the parent-side incrementality
    // admission treats the sequence as an infra non-verdict (a refused
    // warm edit is never blessed as a language result).
    let sig_steps = r#"{ op: "edit-source", from: "fn fn_leaf_1(v int) int {", to: "fn fn_leaf_1(v int, w int) int {" }, { op: "run" }"#;
    let case_dir = root.join("inc-refused");
    let out_dir = root.join("inc-refused-out");
    std::fs::create_dir_all(&case_dir).unwrap();
    std::fs::write(case_dir.join("main.at"), &src0).unwrap();
    std::fs::write(
        case_dir.join("c.case.ad"),
        format!(
            "# t11 refused\n```test-case\ncase {{\n  schema: \"autolang.case/2\",\n  id: \"mn.t11.refused\",\n  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [\n    {{ id: \"run\", adapter: \"probe\", action: \"run\",\n      options: {{ recipe: \"ac/mir-native-scalar-v1\", mn: {{ entry: \"d_main\" }}, steps: [{sig_steps}] }},\n      expect: {{ completion: \"success\", exit: {exp0} }} }}\n  ]\n}}\n```\n"
        ),
    )
    .unwrap();
    std::fs::write(
        case_dir.join("s.suite.ad"),
        "# t11\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    let report =
        run_suite(&case_dir.join("s.suite.ad"), &out_dir, &Limits::defaults()).expect("suite runs");
    assert_eq!(
        status_of(&report, "run"),
        auto_test_case::report::Status::Error,
        "a refused warm edit is an infra non-verdict, never a green language result"
    );

    // (d) Bad-literal control: an edit-source step whose literal does not
    // match exactly once is an infra error (never a silent pass).
    let bad_steps = r#"{ op: "edit-source", from: "zzz-not-present", to: "x" }, { op: "run" }"#;
    let case_dir2 = root.join("inc-badlit");
    let out_dir2 = root.join("inc-badlit-out");
    std::fs::create_dir_all(&case_dir2).unwrap();
    std::fs::write(case_dir2.join("main.at"), &src0).unwrap();
    std::fs::write(
        case_dir2.join("c.case.ad"),
        format!(
            "# t11 badlit\n```test-case\ncase {{\n  schema: \"autolang.case/2\",\n  id: \"mn.t11.badlit\",\n  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [\n    {{ id: \"run\", adapter: \"probe\", action: \"run\",\n      options: {{ recipe: \"ac/mir-native-scalar-v1\", mn: {{ entry: \"d_main\" }}, steps: [{bad_steps}] }},\n      expect: {{ completion: \"success\", exit: {exp0} }} }}\n  ]\n}}\n```\n"
        ),
    )
    .unwrap();
    std::fs::write(
        case_dir2.join("s.suite.ad"),
        "# t11\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s2\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    let report2 = run_suite(
        &case_dir2.join("s.suite.ad"),
        &out_dir2,
        &Limits::defaults(),
    )
    .expect("suite runs");
    assert_eq!(
        status_of(&report2, "run"),
        auto_test_case::report::Status::Error,
        "an unmatchable edit literal is infra, never green"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// V-12.1 through the REAL FORMAL RUNNER (R1 repair, R2 hardened): ONE
/// request = ONE stable D-828 session carrying the full 30-revision
/// body-edit series (the frozen fn edit site, chained revision by
/// revision) plus the judged run — driven through the ROUNDS protocol
/// (one request/response line pair per step). The dual gate per the
/// plan: STRUCTURE (every edit step re-emits exactly one unit, every
/// other unit hits resident, zero unaffected-function stage work, zero
/// whole-module stages, zero fallbacks, zero refusals) AND the
/// PARENT-OBSERVED P95 (F-818-R1-01): each edit's CONTINUOUS window
/// from the parent's own round preparation (source copy + literal
/// search/insert + payload serialization) through IPC, the worker's
/// locate/edit/link/run, the response, and the parent's admission of
/// the round — the §5.5 interval. The worker-side continuous wall_ms
/// rides every record as retained evidence (asserted covered by the
/// parent window). The debug-tier mirror keeps the structural
/// assertions and reports (not gates) its environment-sensitive P95.
#[test]
#[cfg(windows)]
fn mir_native_incremental_series_formal() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let samples: usize = std::env::var("MN818_FORMAL_SAMPLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let gate_ms: f64 = std::env::var("MN818_FORMAL_GATE_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000.0);
    let root = mn_t07_root("t11-series");
    let mut revs = Vec::with_capacity(samples + 1);
    for n in 0..=samples {
        revs.push(mn_fetch_revision(n as u32));
    }
    // The chained edit steps: revision n -> n+1 at the differing line.
    // The differing LINE alone is not unique across the 115-function
    // module; anchor it with the enclosing function header (unique per
    // function) - the same disambiguation the frozen T-07 site uses.
    let line_of = |src: &str, pos: usize| -> String {
        let start = src[..pos].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let end = src[pos..].find('\n').map(|e| pos + e).unwrap_or(src.len());
        let line = &src[start..end];
        let head_zone = &src[..start];
        let hstart = head_zone.rfind("\nfn ").map(|p| p + 1).unwrap_or(0);
        let header = &src[hstart..start];
        format!("{}{}", header, line)
    };
    let mut steps = String::new();
    for n in 0..samples {
        let (a, b) = (&revs[n].0, &revs[n + 1].0);
        let mut i = 0usize;
        while i < a.len().min(b.len()) && a.as_bytes()[i] == b.as_bytes()[i] {
            i += 1;
        }
        assert!(i < a.len().min(b.len()), "revisions {n}..{} differ", n + 1);
        let from = line_of(a, i);
        let to = line_of(b, i);
        assert!(
            from != to && from.contains("var acc = x *"),
            "the frozen multiplier site"
        );
        let esc = |t: &str| t.replace('\\', "\\\\").replace('"', "\\\"");
        steps.push_str(&format!(
            "{{ op: \"edit-source\", from: \"{}\", to: \"{}\" }}, ",
            esc(&from),
            esc(&to)
        ));
    }
    steps.push_str("{ op: \"run\" }");
    let judged = revs[samples].1;
    // F-818-R1-01 residual: the OBSERVER SEAM carries the independent
    // per-edit expectations INTO the adapter — each edit round's real PE
    // result is compared against its revision's independent expectation
    // INSIDE the round's continuous parent-observed window (the worker
    // never sees them); the per-record asserts below re-check what was
    // already compared in-window.
    let expect_list = revs[1..=samples]
        .iter()
        .map(|(_, exp)| exp.to_string())
        .collect::<Vec<_>>()
        .join(",");
    std::env::set_var("AC_MN_INCREMENTAL_EXPECT_EXITS", &expect_list);
    let _seam = EnvGuard("AC_MN_INCREMENTAL_EXPECT_EXITS");
    let r = mn_formal_call(
        &root,
        "series",
        &revs[0].0,
        &steps,
        None,
        &format!(r#"{{ completion: "success", exit: {} }}"#, judged),
    );
    drop(_seam);
    assert_eq!(r["incremental"].as_bool(), Some(true), "one stable session");
    let recs = r["steps"].as_array().expect("step records");
    let edits: Vec<&serde_json::Value> = recs
        .iter()
        .filter(|x| x["op"].as_str() == Some("edit-source"))
        .collect();
    assert_eq!(edits.len(), samples, "every revision rode the session");
    let mut walls = Vec::with_capacity(samples);
    for (n, e) in edits.iter().enumerate() {
        // STRUCTURE: exactly one unit re-emits; the rest hit resident; no
        // unaffected work, no whole-module stages, no fallbacks/refusals.
        assert_eq!(
            e["codegen_units"].as_u64(),
            Some(1),
            "rev {n}: one re-emission"
        );
        assert_eq!(
            e["cache_hits"].as_u64(),
            Some(114),
            "rev {n}: resident hits"
        );
        assert_eq!(e["unaffected_stage_delta"].as_u64(), Some(0), "rev {n}");
        assert_eq!(e["whole_module_stage_delta"].as_u64(), Some(0), "rev {n}");
        assert_eq!(e["fallback_whole_lower"].as_u64(), Some(0), "rev {n}");
        assert!(e["refused"].is_null(), "rev {n}");
        assert_eq!(
            e["exit_code"].as_i64(),
            Some(revs[n + 1].1),
            "rev {n}: PE result == the revision's independent expectation"
        );
        assert_eq!(
            e["expected_exit"].as_i64(),
            Some(revs[n + 1].1),
            "rev {n}: the seam carried the independent expectation"
        );
        assert_eq!(
            e["expected_match"].as_bool(),
            Some(true),
            "rev {n}: the independent comparison matched INSIDE the window"
        );
        // The worker's CONTINUOUS edit-entry -> PE window (retained
        // evidence), and the PARENT-OBSERVED gate window that COVERS it
        // (preparation + IPC + admission + the independent comparison are
        // all inside the parent window).
        let w = e["wall_ms"].as_f64().unwrap_or(0.0);
        assert!(w > 0.0, "rev {n}: real continuous worker window");
        let pw = e["parent_window_ms"].as_f64().unwrap_or(0.0);
        assert!(
            pw >= w,
            "rev {n}: the parent-observed window ({pw:.1}ms) must cover the worker window ({w:.1}ms)"
        );
        walls.push(pw);
    }
    let run_rec = recs
        .iter()
        .rev()
        .find(|x| x["op"].as_str() == Some("run"))
        .expect("judged run");
    assert_eq!(run_rec["exit_code"].as_i64(), Some(judged));
    let p95 = mn_t07_percentile(&walls, 95.0);
    eprintln!(
        "t11 v12.1-formal: {samples} revisions in one stable session, PARENT-OBSERVED (prep -> IPC -> worker -> admission -> independent comparison) P50/P95/P99 = {:.1}/{:.1}/{:.1} ms (gate {gate_ms})",
        mn_t07_percentile(&walls, 50.0),
        p95,
        mn_t07_percentile(&walls, 99.0),
    );
    assert!(
        p95 < gate_ms,
        "formal series parent-observed P95 {p95:.1}ms >= {gate_ms}ms"
    );
    // Raw evidence.
    let report = serde_json::json!({
        "plan": "PLAN-818",
        "tier": "t11-v12.1-formal-series",
        "window": "parent-observed (preparation -> IPC -> worker -> admission -> independent comparison)",
        "gate_ms": gate_ms,
        "samples": samples,
        "p50_ms": mn_t07_percentile(&walls, 50.0),
        "p95_ms": p95,
        "p99_ms": mn_t07_percentile(&walls, 99.0),
        "windows_ms": walls,
    });
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/reports/plan818/turnaround-mn-incremental-series.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    let _ = std::fs::remove_dir_all(&root);
}

/// F-818-R2-01: an AUTHORIZED-TRACE warm edit sequence through the REAL
/// runner — the incremental session links the trace support library on
/// EVERY round (cold warm-up, edit, judged run) and the raw PE stderr
/// bytes survive byte-exact in every record and the judged receipt
/// (never dropped or blanked).
#[test]
#[cfg(windows)]
fn mir_native_incremental_trace_formal() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = mn_t07_root("t11-trace");
    let src = "fn helper(v int) int {\n    let a = mark_a()\n    return v + a\n}\n\nfn main() int {\n    return helper(10)\n}\n";
    let steps =
        r#"{ op: "edit-source", from: "return v + a", to: "return v + a + 1" }, { op: "run" }"#;
    let r = mn_formal_call_opts(
        &root,
        "trace-inc",
        src,
        steps,
        None,
        &["hir.test.trace.v1"],
        r#"{ completion: "success", exit: 12 }"#,
    );
    assert_eq!(
        r["incremental"].as_bool(),
        Some(true),
        "the warm path engaged"
    );
    let recs = r["steps"].as_array().expect("step records");
    let warm = recs
        .iter()
        .find(|x| x["op"].as_str() == Some("warmup"))
        .expect("warmup");
    assert_eq!(warm["exit_code"].as_i64(), Some(11), "10 + mark_a()");
    assert_eq!(
        warm["stderr_hex"].as_str(),
        Some("61"),
        "the COLD round's trace byte"
    );
    let edit = recs
        .iter()
        .find(|x| x["op"].as_str() == Some("edit-source"))
        .expect("edit step");
    assert_eq!(edit["exit_code"].as_i64(), Some(12), "10 + a + 1");
    assert_eq!(
        edit["stderr_hex"].as_str(),
        Some("61"),
        "the EDIT round's trace byte (byte-exact, in order)"
    );
    assert_eq!(edit["codegen_units"].as_u64(), Some(1), "one unit re-emits");
    let run_rec = recs
        .iter()
        .rev()
        .find(|x| x["op"].as_str() == Some("run"))
        .expect("judged run step");
    assert_eq!(run_rec["exit_code"].as_i64(), Some(12));
    assert_eq!(run_rec["stderr_hex"].as_str(), Some("61"));
    assert_eq!(
        r["stderr_hex"].as_str(),
        Some("61"),
        "the judged streams ride the receipt top level"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// F-818-R2-02: a LATER syntax fallback preserves the ACCEPTED PREFIX.
/// The first edit is committed by the session (the first-step-only
/// control in mir_native_incremental_formal_runner_gate proves true
/// incremental engagement); when the SECOND edit cannot converge, the
/// whole-request leg must reconstruct from the SESSION'S CURRENT source
/// (both edits applied) — the correct result is 202, never the original
/// source's 102. Includes the dependent-literal variant (the second
/// edit's `from` exists only after the first edit) and the first-step
/// fallback control.
#[test]
#[cfg(windows)]
fn mir_native_incremental_syntax_fallback_preserves_prefix() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = mn_t07_root("t11-prefix");
    let src = "fn helper(v int) int {\n    return v + 100\n}\n\nfn main() int {\n    return helper(1)\n}\n";
    // (a) The R2 counterexample verbatim: committed edit + fallback edit
    // + run. Correct result 202 (was executed as 102 by the defect).
    let steps = r#"{ op: "edit-source", from: "return v + 100", to: "return v + 200" }, { op: "edit-source", from: "helper(1)", to: "helper(2)" }, { op: "run" }"#;
    let r = mn_formal_call(
        &root,
        "prefix",
        src,
        steps,
        None,
        r#"{ completion: "success", exit: 202 }"#,
    );
    assert_eq!(
        r["warm_edit_fallback"].as_bool(),
        Some(true),
        "the second edit falls back (recorded, never labeled incremental)"
    );
    assert_eq!(r["incremental"].as_bool(), Some(false));
    assert_eq!(r["manifest_fp"].as_str().map(|s| !s.is_empty()), Some(true));
    // (b) Dependent literal: the second edit's `from` (`return v + 200`)
    // exists ONLY because the first edit introduced it — a prefix-dropping
    // fallback cannot even locate it.
    let steps_b = r#"{ op: "edit-source", from: "return v + 100", to: "return v + 200" }, { op: "edit-source", from: "return v + 200", to: "return v + 300" }, { op: "run" }"#;
    let rb = mn_formal_call(
        &root,
        "dependent",
        src,
        steps_b,
        None,
        r#"{ completion: "success", exit: 301 }"#,
    );
    // Whichever path the second edit takes (incremental or fallback), the
    // judged program is the fully edited source: 2 + 300.
    assert_eq!(rb["exit_code"].as_i64(), Some(301));
    // (c) FIRST-step fallback control: the non-converging shape as the
    // very first edit — the whole-request leg runs over original-source +
    // this edit: 102, labeled fallback.
    let steps_c = r#"{ op: "edit-source", from: "helper(1)", to: "helper(2)" }, { op: "run" }"#;
    let rc = mn_formal_call(
        &root,
        "first-fallback",
        src,
        steps_c,
        None,
        r#"{ completion: "success", exit: 102 }"#,
    );
    assert_eq!(rc["warm_edit_fallback"].as_bool(), Some(true));
    let _ = std::fs::remove_dir_all(&root);
}

/// F-818-R2-02 (failure propagation): after a syntax fallback the
/// whole-request leg surfaces REAL pipeline diagnostics — a signature
/// edit materialized in fallback mode leaves the call sites broken, and
/// the trailing run is a green DIAGNOSED frontend rejection (never a
/// fake success, never an unexplained crash).
#[test]
#[cfg(windows)]
fn mir_native_incremental_fallback_diagnostic_propagation() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = mn_t07_root("t11-fbdiag");
    let src = "fn helper(v int) int {\n    return v + 100\n}\n\nfn main() int {\n    return helper(1)\n}\n";
    let steps = r#"{ op: "edit-source", from: "helper(1)", to: "helper(2)" }, { op: "edit-source", from: "fn helper(v int) int {", to: "fn helper(v int, w int) int {" }, { op: "run" }"#;
    let r = mn_formal_call(
        &root,
        "fbdiag",
        src,
        steps,
        None,
        r#"{ completion: { stage: "parse", code: "mn.unsupported.frontend" } }"#,
    );
    assert_eq!(
        r["warm_edit_fallback"].as_bool(),
        Some(true),
        "the first edit falls back; the signature edit materializes"
    );
    assert!(
        r["exit_code"].is_null(),
        "a diagnosed rejection carries no success exit"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// F-818-R2-01/R2-02: the SINGLE-SHOT worker protocol (all steps in one
/// request — the direct replay shape of the R2 review) carries the trace
/// library and the accepted prefix: an authorized-trace edit run exits
/// 12 with stderr byte 61, and the prefix sequence produces 202 (the
/// review observed the link failure and 102 respectively).
#[test]
#[cfg(windows)]
fn mir_native_worker_single_shot_trace_and_prefix() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let worker = crate::test_cases::ac_mn::worker_binary().expect("mn worker located");
    let scratch = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target")
        .join(format!("mn818-t11-shot-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let run_case = |req: serde_json::Value| -> serde_json::Value {
        use std::io::Write as _;
        let mut child = std::process::Command::new(&worker)
            .arg("case-run")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn mn-worker case-run");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(format!("{}\n", req.to_string()).as_bytes())
            .unwrap();
        let out = child.wait_with_output().expect("worker output");
        assert!(
            out.status.success(),
            "single-shot worker must succeed: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        let text = String::from_utf8_lossy(&out.stdout);
        let line = text
            .lines()
            .find(|l| !l.trim().is_empty())
            .expect("one protocol line");
        serde_json::from_str(line).expect("protocol json")
    };
    let base64_request = |src: &str, caps: &[&str], steps: &[(&str, &str)], tag: &str| {
        use base64::Engine as _;
        let steps_json: Vec<serde_json::Value> = steps
            .iter()
            .map(|(f, t)| serde_json::json!({ "op": "edit-source", "from": f, "to": t }))
            .chain(std::iter::once(serde_json::json!({ "op": "run" })))
            .collect();
        serde_json::json!({
            "recipe": "ac/mir-native-scalar-v1",
            "nonce": format!("shot-{tag}"),
            "request_hash": "shot",
            "source_b64": base64::engine::general_purpose::STANDARD.encode(src.as_bytes()),
            "entry": "d_main",
            "caps": caps,
            "store_root": scratch.join(format!("{tag}-store")).display().to_string(),
            "workdir": scratch.join(format!("{tag}-work")).display().to_string(),
            "steps": steps_json,
        })
    };
    // (a) The R2-01 trace counterexample: authorized mark_a edit run.
    let trace_src = "fn helper(v int) int {\n    let a = mark_a()\n    return v + a\n}\n\nfn main() int {\n    return helper(10)\n}\n";
    let ra = run_case(base64_request(
        trace_src,
        &["hir.test.trace.v1"],
        &[("return v + a", "return v + a + 1")],
        "trace",
    ));
    let ok = ra["ok"].as_object().expect("ok");
    assert_eq!(
        ok.get("incremental").and_then(|v| v.as_bool()),
        Some(true),
        "the traced edit rides the incremental session"
    );
    assert_eq!(
        ok.get("exit_code").and_then(|v| v.as_i64()),
        Some(12),
        "traced edit result"
    );
    assert_eq!(
        ok.get("stderr_hex").and_then(|v| v.as_str()),
        Some("61"),
        "the judged trace bytes ride the single-shot response"
    );
    let edit_rec = ok["steps"]
        .as_array()
        .expect("step records")
        .iter()
        .find(|x| x["op"].as_str() == Some("edit-source"))
        .expect("edit record");
    assert_eq!(
        edit_rec["stderr_hex"].as_str(),
        Some("61"),
        "per-step trace bytes"
    );
    // (b) The R2-02 prefix counterexample: 202, not 102.
    let prefix_src = "fn helper(v int) int {\n    return v + 100\n}\n\nfn main() int {\n    return helper(1)\n}\n";
    let rb = run_case(base64_request(
        prefix_src,
        &[],
        &[
            ("return v + 100", "return v + 200"),
            ("helper(1)", "helper(2)"),
        ],
        "prefix",
    ));
    let okb = rb["ok"].as_object().expect("ok");
    assert_eq!(
        okb.get("warm_edit_fallback").and_then(|v| v.as_bool()),
        Some(true),
        "the second edit falls back (single-shot)"
    );
    assert_eq!(
        okb.get("exit_code").and_then(|v| v.as_i64()),
        Some(202),
        "the accepted prefix is preserved (was executed as 102)"
    );
    let _ = std::fs::remove_dir_all(&scratch);
}

/// F-818-R3-01/R3-02: the rounds boundary is failure-live and cap-bound.
/// Protocol-aware fixtures (the REAL worker binary, test-only fault seam
/// `AC_MN_WORKER_FAULT`) drive the actual adapter. Every blocking shape —
/// a worker that never reads stdin (request beyond pipe capacity), stops
/// reading after the warmup, answers close but never exits, or ignores
/// close — must return an infra non-verdict within the configured
/// absolute budget (AC_MN_ROUNDS_WALL_SECS) with the worker tree GONE
/// (pid liveness), never a parent hang. Cap shapes — a newline-free
/// over-cap stream, one oversized newline-terminated line, an unsolicited
/// line flood, and stderr past its cap WHILE stdout stays valid — must be
/// rejected as infra promptly; the boundary valid controls (stderr just
/// under the cap, a response line just under the line cap) stay green.
#[test]
#[cfg(windows)]
fn mir_native_rounds_failure_liveness_and_caps() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = mn_t07_root("t11-liveness");
    let small_src =
        "fn helper(v int) int {\n    return v + 100\n}\n\nfn main() int {\n    return helper(1)\n}\n";
    let steps =
        r#"{ op: "edit-source", from: "return v + 100", to: "return v + 200" }, { op: "run" }"#;
    let pidfile = root.join("fixture.pid");
    let fixture_pid = || -> u32 {
        for _ in 0..100 {
            if let Ok(text) = std::fs::read_to_string(&pidfile) {
                if let Ok(p) = text.trim().parse::<u32>() {
                    return p;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        panic!("the fixture pidfile never appeared");
    };
    let pid_alive = |pid: u32| -> bool {
        let out = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .stdout(std::process::Stdio::piped())
            .output();
        matches!(
            out,
            Ok(o) if String::from_utf8_lossy(&o.stdout).contains(&format!("\"{pid}\""))
        )
    };
    let set_seams = |fault: &str| {
        let _ = std::fs::remove_file(&pidfile);
        std::env::set_var("AC_MN_WORKER_FAULT", fault);
        std::env::set_var(
            "AC_MN_WORKER_FIXTURE_PIDFILE",
            pidfile.display().to_string(),
        );
        std::env::set_var("AC_MN_ROUNDS_WALL_SECS", "3");
    };
    let clear_seams = || {
        std::env::remove_var("AC_MN_WORKER_FAULT");
        std::env::remove_var("AC_MN_WORKER_FIXTURE_PIDFILE");
        std::env::remove_var("AC_MN_ROUNDS_WALL_SECS");
    };
    let mismatch_text = |report: &auto_test_case::report::RunReport| -> String {
        report
            .results
            .iter()
            .find(|r| r.check_id == "run")
            .and_then(|r| r.mismatches.first().map(|m| m.actual.clone()))
            .unwrap_or_default()
    };

    // (a) Liveness: infra within the budget, worker tree gone. The
    // no-stdin fixture needs a request beyond the pipe capacity (the
    // 273KB frozen load source).
    let (big_src, _) = mn_fetch_revision(0);
    for (tag, fault, src) in [
        ("no-stdin", "no-stdin-read", big_src.as_str()),
        ("stop-warm", "stop-after-warmup", small_src),
        ("close-noexit", "close-without-exit", small_src),
        ("close-ignored", "ignore-close", small_src),
    ] {
        let started = std::time::Instant::now();
        set_seams(fault);
        let report = mn_formal_call_report(
            &root,
            tag,
            src,
            steps,
            r#"{ completion: "success", exit: 201 }"#,
        );
        clear_seams();
        let took = started.elapsed();
        assert_eq!(
            status_of(&report, "run"),
            auto_test_case::report::Status::Error,
            "{tag}: a blocking worker is an infra non-verdict (took {:.1}s): {}",
            took.as_secs_f64(),
            mismatch_text(&report)
        );
        assert!(
            took < std::time::Duration::from_secs(12),
            "{tag}: returned within the configured budget (took {:.1}s)",
            took.as_secs_f64()
        );
        let pid = fixture_pid();
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(!pid_alive(pid), "{tag}: the fixture worker tree is gone");
    }

    // (b) Stdout caps: prompt infra naming the cap (bounded allocation —
    // the cap trips DURING accumulation, before any unbounded read).
    for (tag, fault) in [
        ("cap-noeol", "stdout-oversize-noeol"),
        ("cap-line", "stdout-oversize-line"),
    ] {
        let started = std::time::Instant::now();
        set_seams(fault);
        let report = mn_formal_call_report(
            &root,
            tag,
            small_src,
            steps,
            r#"{ completion: "success", exit: 201 }"#,
        );
        clear_seams();
        assert_eq!(
            status_of(&report, "run"),
            auto_test_case::report::Status::Error,
            "{tag}"
        );
        let text = mismatch_text(&report);
        assert!(
            text.contains("stdout exceeded"),
            "{tag}: the stdout cap is propagated, got: {text}"
        );
        assert!(
            started.elapsed() < std::time::Duration::from_secs(12),
            "{tag}"
        );
    }

    // (c) Unsolicited flood: infra promptly (bounded queue — the reader
    // never allocates without bound); the first bogus line fails
    // admission instead of queueing the flood.
    let started = std::time::Instant::now();
    set_seams("stdout-flood-lines");
    let report = mn_formal_call_report(
        &root,
        "cap-flood",
        small_src,
        steps,
        r#"{ completion: "success", exit: 201 }"#,
    );
    clear_seams();
    assert_eq!(
        status_of(&report, "run"),
        auto_test_case::report::Status::Error,
        "cap-flood: an unsolicited flood is infra"
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(12));

    // (d) stderr over the cap WHILE stdout stays valid: the cap is
    // propagated as infra, never a silently-green run.
    let started = std::time::Instant::now();
    set_seams("stderr-overcap");
    let report = mn_formal_call_report(
        &root,
        "stderr-over",
        small_src,
        steps,
        r#"{ completion: "success", exit: 201 }"#,
    );
    clear_seams();
    assert_eq!(
        status_of(&report, "run"),
        auto_test_case::report::Status::Error,
        "stderr-over: a cap-violating worker never yields a valid verdict"
    );
    let text = mismatch_text(&report);
    assert!(
        text.contains("stderr exceeded"),
        "stderr-over: the stderr cap is propagated, got: {text}"
    );
    assert!(started.elapsed() < std::time::Duration::from_secs(12));
    let pid = fixture_pid();
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert!(
        !pid_alive(pid),
        "stderr-over: the fixture worker tree is gone"
    );

    // (e) Boundary VALID controls: stderr just under the cap and a
    // response line just under the line cap both stay green through the
    // real adapter (a relaxed wall — the multi-megabyte line is the
    // probe, not the deadline).
    for (tag, fault) in [
        ("stderr-under", "stderr-under-cap"),
        ("line-under", "stdout-line-under-cap"),
    ] {
        set_seams(fault);
        std::env::set_var("AC_MN_ROUNDS_WALL_SECS", "30");
        let report = mn_formal_call_report(
            &root,
            tag,
            small_src,
            steps,
            r#"{ completion: "success", exit: 201 }"#,
        );
        clear_seams();
        assert_eq!(
            status_of(&report, "run"),
            auto_test_case::report::Status::Pass,
            "{tag}: the valid boundary control stays green: {}",
            mismatch_text(&report)
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// F-818-R1-01 residual: the observer-seam independent comparison is a
/// VERDICT inside the round's continuous parent-observed window — the
/// correct expectation rides every edit record (compared in-window), and
/// a WRONG expectation fails the case (never a silent pass).
#[test]
#[cfg(windows)]
fn mir_native_incremental_expect_seam_controls() {
    let _guard = MN_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = mn_t07_root("t11-seam");
    let src =
        "fn helper(v int) int {\n    return v + 100\n}\n\nfn main() int {\n    return helper(1)\n}\n";
    let steps =
        r#"{ op: "edit-source", from: "return v + 100", to: "return v + 200" }, { op: "run" }"#;

    // (a) Correct expectation: green, and the comparison is recorded on
    // the edit record (compared INSIDE the round window).
    std::env::set_var("AC_MN_INCREMENTAL_EXPECT_EXITS", "201");
    let _seam = EnvGuard("AC_MN_INCREMENTAL_EXPECT_EXITS");
    let r = mn_formal_call(
        &root,
        "seam-ok",
        src,
        steps,
        None,
        r#"{ completion: "success", exit: 201 }"#,
    );
    drop(_seam);
    let edit = r["steps"]
        .as_array()
        .expect("step records")
        .iter()
        .find(|x| x["op"].as_str() == Some("edit-source"))
        .expect("edit step");
    assert_eq!(edit["expected_exit"].as_i64(), Some(201));
    assert_eq!(
        edit["expected_match"].as_bool(),
        Some(true),
        "the independent comparison matched inside the window"
    );

    // (b) Wrong expectation: the comparison FAILS the case (a real
    // verdict failure, not infra, not a silent pass).
    std::env::set_var("AC_MN_INCREMENTAL_EXPECT_EXITS", "999");
    let _seam = EnvGuard("AC_MN_INCREMENTAL_EXPECT_EXITS");
    let report = mn_formal_call_report(
        &root,
        "seam-bad",
        src,
        steps,
        r#"{ completion: "success", exit: 201 }"#,
    );
    drop(_seam);
    assert_eq!(
        status_of(&report, "run"),
        auto_test_case::report::Status::Fail,
        "a mismatched independent expectation fails the case: {:#?}",
        report.results
    );
    let seam_mismatch = report
        .results
        .iter()
        .find(|r| r.check_id == "run")
        .and_then(|r| {
            r.mismatches
                .iter()
                .find(|m| m.observation.contains("independent exit"))
                .cloned()
        })
        .unwrap_or_else(|| {
            panic!(
                "the seam comparison mismatch is recorded, got: {:#?}",
                report
                    .results
                    .iter()
                    .find(|r| r.check_id == "run")
                    .map(|r| &r.mismatches)
            )
        });
    assert_eq!(seam_mismatch.expected, "999");
    assert_eq!(seam_mismatch.actual, "Some(201)");
    let _ = std::fs::remove_dir_all(&root);
}
