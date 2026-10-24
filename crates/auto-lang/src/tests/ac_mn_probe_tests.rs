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
    assert_eq!(status, auto_test_case::report::Status::Pass, "edited exit 9");
    let receipt = receipt_of(&root);
    let s = &receipt["sessions"][0];
    assert_eq!(s["steps"][0].as_str(), Some("edit-source"));
    assert_eq!(s["exit_code"].as_i64(), Some(9));
    assert!(s["manifest_fp"].as_str().is_some_and(|f| !f.is_empty()));
    assert!(s["judged"].as_bool() == Some(true));
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
    std::fs::write(&exit9, "@echo off
echo not-json
exit /b 9
").unwrap();
    std::env::set_var("AC_MN_CASE_WORKER", exit9.to_str().unwrap());
    let status = run_one();
    std::env::remove_var("AC_MN_CASE_WORKER");
    assert_eq!(status, auto_test_case::report::Status::Error);

    // (c) A zero-exit worker speaking a WRONG schema line is infra.
    let garbage = dir.join("garbage.cmd");
    std::fs::write(&garbage, "@echo off
echo hello world
exit /b 0
").unwrap();
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
    std::fs::copy(corpus.join("mn-value.case.ad"), root.join("mn-value.case.ad")).unwrap();
    let suite_text =
        std::fs::read_to_string(corpus.join("ac-mn.suite.ad")).unwrap().replace(
            "mn-value.case.ad",
            "mn-value.case.ad",
        );
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
    assert!(failing.is_empty(), "mn corpus + av case green together: {failing:#?}");
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
