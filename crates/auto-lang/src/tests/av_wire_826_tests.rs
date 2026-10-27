//! PLAN-826 formal compact-wire corpus and parent-side sensitivity gates.

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use auto_test_case::report::{RunReport, Status};
use std::path::PathBuf;

fn suite() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../auto-lang/test/cases/av-abc/826/av-abc-826.suite.ad")
}

fn run(tag: &str) -> RunReport {
    let out = std::env::temp_dir().join(format!("atc-av826-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let report = run_suite(&suite(), &out, &Limits::defaults()).expect("av-abc-826 suite runs");
    assert!(report.load_errors.is_empty(), "{:#?}", report.load_errors);
    report
}

#[test]
fn av_wire_826_first_case_corpus_green() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let report = run("first");
    let row = report
        .results
        .iter()
        .find(|row| row.check_id == "run-compact-first")
        .expect("required first case");
    assert_eq!(
        row.status,
        Status::Pass,
        "{:?} {:?}",
        row.detail,
        row.mismatches
    );
}

#[test]
fn av_wire_826_parent_sensitivity_controls_non_green() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    // Sabotaging any compact-wire minimum axis (the compact unit/closure
    // counters or the loaded representation counters) must redden the
    // parent gate — a legacy-wire run of the same source cannot pass as
    // compact. (The reference-path zero axes are positive invariants:
    // they are judged on every corpus run and cannot be sabotaged by
    // zeroing.)
    for key in [
        "abc_wire_compact_units",
        "abc_wire_compact_closure",
        "loaded_index_builds",
        "loaded_pc_lookups",
    ] {
        std::env::set_var("AV826_JUDGE_SABOTAGE", key);
        let report = run(&format!("sabotage-{key}"));
        std::env::remove_var("AV826_JUDGE_SABOTAGE");
        assert!(
            report.results.iter().any(|row| {
                row.status == Status::Fail && row.mismatches.iter().any(|m| m.kind == key)
            }),
            "missing {key} must redden the parent gate"
        );
    }

    // A wrong oracle on the frozen first case must fail on the value axis.
    let root = std::env::temp_dir().join(format!("av826-wrong-oracle-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::copy(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../auto-lang/test/cases/av-abc/826/src/first-case.at"),
        root.join("src/first-case.at"),
    )
    .unwrap();
    std::fs::write(
        root.join("wrong.case.ad"),
        "```test-case\ncase { schema: \"autolang.case/2\", id: \"av.compact.wrong\", source: { file: \"src/first-case.at\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"program\" }, checks: [{ id: \"run\", adapter: \"probe\", action: \"run\", options: { recipe: \"ac-av/compact-run-v1\", av: { entry: \"d_main\", caps: [\"hir.test.trace.v1\"] }, steps: [{ op: \"run\" }] }, expect: { completion: \"success\", exit: 15 } }] }\n```\n",
    )
    .unwrap();
    std::fs::write(
        root.join("wrong.suite.ad"),
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"av826-wrong\", include: [\"wrong.case.ad\"] }\n```\n",
    )
    .unwrap();
    let report = run_suite(
        &root.join("wrong.suite.ad"),
        &root.with_extension("out"),
        &Limits::defaults(),
    )
    .expect("wrong-oracle suite runs");
    assert!(report.results.iter().any(|row| {
        row.status == Status::Fail && row.mismatches.iter().any(|m| m.kind == "exit")
    }));
    let _ = std::fs::remove_dir_all(&root);
}
