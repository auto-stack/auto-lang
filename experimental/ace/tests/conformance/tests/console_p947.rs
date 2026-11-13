//! PLAN-947: the console/1 public-binding corpus on BOTH real lanes — AV
//! (`ac-av/console-run-v1`) and native mn (`ac/mn/console-run-v1`). Every
//! row must actually PASS; the wrong-implementation sensitivity controls
//! mutate one expectation face at a time and must FAIL independently.
use auto_lang_core::test_cases::{run_suite, stale};
use auto_test_case::{admission::Limits, discovery::load_suite, report::Status};
use std::path::{Path, PathBuf};

fn corpus() -> PathBuf {
    Path::new(env!("AUTO_LANG_CORE_DIR"))
        .join("../auto-lang/test/cases/ace-m2/p947-console/p947-console.suite.ad")
}

#[test]
fn p947_console_corpus_both_lanes_pass() {
    let temp = tempfile_dir();
    let suite = corpus();
    let report = run_suite(&suite, &temp.join("out"), &Limits::defaults()).unwrap();
    assert!(report.load_errors.is_empty(), "{:?}", report.load_errors);
    assert_eq!(report.results.len(), 7, "{:?}", report.load_errors);
    for row in &report.results {
        assert_eq!(
            row.status,
            Status::Pass,
            "{} {:?} {:?}",
            row.case_id,
            row.detail,
            row.mismatches
        );
    }
    // Freshness: every row's stored identity matches the current run.
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
    let fresh = stale::check(&suite, &json).unwrap();
    assert_eq!(fresh.len(), 7);
    assert!(
        fresh
            .iter()
            .all(|r| matches!(r, stale::RowVerdict::Fresh { .. })),
        "{fresh:?}"
    );
}

#[test]
fn p947_console_wrong_expectations_fail_independently() {
    // Each control edits exactly one expectation face inside the case file
    // (the sources are untouched); the mutated row must FAIL on that face.
    let cases = std::fs::read_to_string(
        Path::new(env!("AUTO_LANG_CORE_DIR"))
            .join("../auto-lang/test/cases/ace-m2/p947-console/p947-console.case.ad"),
    )
    .unwrap();
    for (needle, replacement, face) in [
        // mn stdout face (O03): a wrong first byte must be caught.
        ("stdout: { text: \"你好\\n\\n\", compare: \"bytes\" }, stderr: { text: \"é\\n\", compare: \"bytes\" }, exit: 0 } }]
}
```

```test-case
case {
  schema: \"autolang.case/2\", id: \"ace.m2.p947.o03-av\"",
         "stdout: { text: \"你坏\\n\\n\", compare: \"bytes\" }, stderr: { text: \"é\\n\", compare: \"bytes\" }, exit: 0 } }]
}
```

```test-case
case {
  schema: \"autolang.case/2\", id: \"ace.m2.p947.o03-av\"",
         "stdout-mn-bytes"),
        // mn stderr face (O03): wrong stderr text.
        ("stderr: { text: \"é\\n\", compare: \"bytes\" }, exit: 0 } }]
}
```

```test-case
case {
  schema: \"autolang.case/2\", id: \"ace.m2.p947.o03-av\"",
         "stderr: { text: \"e\\n\", compare: \"bytes\" }, exit: 0 } }]
}
```

```test-case
case {
  schema: \"autolang.case/2\", id: \"ace.m2.p947.o03-av\"",
         "stderr-mn-bytes"),
        // unauthorized face: granting the caps turns the named rejection
        // into a success — the control must FAIL (rejection is load-bearing).
        ("mn: { entry: \"d_main\", caps: [] }, steps: [{ op: \"run\" }] },
    expect: { completion: { stage: \"parse\", code: \"host.unauthorized\" }",
         "mn: { entry: \"d_main\", caps: [\"console.stdout\"] }, steps: [{ op: \"run\" }] },
    expect: { completion: { stage: \"parse\", code: \"host.unauthorized\" }",
         "unauthorized-grant-control"),
    ] {
        let mutated = cases.replacen(needle, replacement, 1);
        assert_ne!(&mutated as &str, &cases as &str, "{face} needle missing");
        let temp = tempfile_dir();
        std::fs::create_dir_all(temp.join("fixture/src")).unwrap();
        let root = Path::new(env!("AUTO_LANG_CORE_DIR"))
            .join("../auto-lang/test/cases/ace-m2/p947-console");
        for entry in std::fs::read_dir(root.join("src")).unwrap() {
            let entry = entry.unwrap();
            std::fs::copy(entry.path(), temp.join("fixture/src").join(entry.file_name())).unwrap();
        }
        std::fs::write(temp.join("fixture/p947-console.case.ad"), mutated).unwrap();
        std::fs::write(
            temp.join("fixture/p947-console.suite.ad"),
            "```test-suite\nsuite {schema: \"autolang.suite/2\", id: \"p947-console-ctl\", include: [\"p947-console.case.ad\"]}\n```\n",
        )
        .unwrap();
        let report = run_suite(
            &temp.join("fixture/p947-console.suite.ad"),
            &temp.join("out"),
            &Limits::defaults(),
        )
        .unwrap();
        assert!(
            report.results.iter().any(|r| r.status == Status::Fail),
            "{face}: the mutated expectation must fail, got {:?}",
            report
                .results
                .iter()
                .map(|r| (r.case_id.as_str(), r.status))
                .collect::<Vec<_>>()
        );
    }
}

fn tempfile_dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "p947-console-ctl-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
