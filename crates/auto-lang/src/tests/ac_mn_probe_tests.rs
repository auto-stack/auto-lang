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
