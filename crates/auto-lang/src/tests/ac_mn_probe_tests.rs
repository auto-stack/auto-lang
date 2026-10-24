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
    std::fs::write(
        case_dir.join("c.case.ad"),
        format!(
            "# t07 {tag}\n```test-case\ncase {{\n  schema: \"autolang.case/2\",\n  id: \"mn.t07.{tag}\",\n  source: {{ file: \"main.at\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [\n    {{ id: \"run\", adapter: \"probe\", action: \"run\",\n      options: {{ recipe: \"ac/mir-native-scalar-v1\", mn: {{ entry: \"d_main\"{session_field} }}, steps: [{steps}] }},\n      expect: {expect} }}\n  ]\n}}\n```\n"
        ),
    )
    .unwrap();
    std::fs::write(
        case_dir.join("s.suite.ad"),
        "# t07\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s\", include: [\"c.case.ad\"] }\n```\n",
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
            .map(|r| (r.status, r.detail.clone()))
            .collect::<Vec<_>>()
    );
    receipt_of(&case_dir)
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
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../docs/reports/plan818/turnaround-mn-{scenario}.json"));
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
        assert_eq!(mn_receipt_counter(s, "codegen_emit_units"), 115, "cold compile");
        assert_eq!(mn_receipt_counter(s, "cache_hit_coff"), 0, "really cold");
        walls.push(w);
        let _ = std::fs::remove_dir_all(
            std::env::temp_dir().join(format!("mn818-session-p{pid}-fresh-{i}")),
        );
    }
    let (p50, p95, p99) = mn_t07_finish_raw("fresh", "each call a fresh session store; full 115-unit compile; reported (not gated)", &walls);
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
            assert_eq!(mn_receipt_counter(s, "codegen_emit_units"), 0, "new process, warm store");
            assert_eq!(mn_receipt_counter(s, "cache_hit_coff"), 115);
        }
        walls.push(r["request_wall_ms"].as_f64().unwrap_or(0.0));
    }
    let _ = mn_t07_finish_raw("restart", "fresh worker process per call over one shared store", &walls);
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join(format!("mn818-session-p{pid}-restart")));
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
        assert!(from_line.contains("var acc = x *"), "the edit is fn_041's multiplier");
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
            assert_eq!(warm_codegen, 0, "warm-up {i} is warm");
        }
        let esc = |t: &str| t.replace('\\', "\\\\").replace('"', "\\\"");
        let steps = format!(
            "{{ op: \"edit-source\", from: \"{}\", to: \"{}\" }}, {{ op: \"run\" }}",
            esc(&from_line),
            esc(&to_line)
        );
        let measured = mn_formal_call(
            &root,
            &format!("edit-{i}"),
            &prev_src,
            &steps,
            Some(&session),
            &format!(r#"{{ completion: "success", exit: {} }}"#, rev_exp),
        );
        let s = &measured["sessions"][0];
        assert_eq!(mn_receipt_counter(s, "codegen_emit_units"), 1, "one unit recompiles");
        assert_eq!(mn_receipt_counter(s, "cache_hit_coff"), 114);
        let w = measured["request_wall_ms"].as_f64().unwrap_or(0.0);
        eprintln!("t07 body-edit rev {i}: window {w:.1}ms (expected {rev_exp})");
        walls.push(w);
        prev_src = rev_src;
        prev_exp = rev_exp;
    }
    let (_, p95, _) = mn_t07_finish_raw(
        "body-edit",
        "warm-up outside window; measured [edit-source, run]; 1 miss / 114 hits; P95 GATED",
        &walls,
    );
    assert!(p95 < gate_ms, "body-edit P95 {p95:.1}ms >= gate {gate_ms}ms");
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join(format!("mn818-session-{session}")));
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
    assert!(p95 < gate_ms, "signature-rebuild P95 {p95:.1}ms >= gate {gate_ms}ms");
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join(format!("mn818-session-{session}")));
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
