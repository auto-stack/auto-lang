//! PLAN-825 formal LoadedProgram corpus and parent-side sensitivity gates.

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use auto_test_case::report::{RunReport, Status};
use std::path::PathBuf;

fn suite() -> PathBuf {
    PathBuf::from(env!("AUTO_LANG_HOST_DIR"))
        .join("../auto-lang/test/cases/av-abc/825/av-abc-825.suite.ad")
}

fn run(tag: &str) -> RunReport {
    let out = std::env::temp_dir().join(format!("atc-av825-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let report = run_suite(&suite(), &out, &Limits::defaults()).expect("av-abc-825 suite runs");
    assert!(report.load_errors.is_empty(), "{:#?}", report.load_errors);
    report
}

#[test]
fn av_runtime_first_case_corpus_green() {
    let _guard = crate::tests::av_protocol::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let report = run("first");
    let row = report
        .results
        .iter()
        .find(|row| row.check_id == "run-loaded-first")
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
fn av_runtime_baseline_corpus_green() {
    let _guard = crate::tests::av_protocol::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let report = run("baseline");
    assert_eq!(report.results.len(), 10, "all mapped checks must execute");
    let failing: Vec<_> = report
        .results
        .iter()
        .filter(|row| row.status != Status::Pass)
        .map(|row| format!("{} {:?} {:?}", row.check_id, row.status, row.mismatches))
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
    for id in [
        "run-loaded-b01",
        "run-loaded-b04",
        "run-loaded-b09",
        "run-loaded-b13",
        "run-loaded-b14",
        "loaded-warm-run",
        "loaded-hot-run",
        "loaded-body-edit-run",
        "loaded-edit-persist-across-workers",
    ] {
        assert!(
            report.results.iter().any(|row| row.check_id == id),
            "missing {id}"
        );
    }
}

#[test]
fn av_runtime_parent_sensitivity_controls_non_green() {
    let _guard = crate::tests::av_protocol::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    for key in [
        "loaded_index_builds",
        "loaded_pc_lookups",
        "loaded_param_map_queries",
    ] {
        std::env::set_var("AV825_JUDGE_SABOTAGE", key);
        let report = run(&format!("sabotage-{key}"));
        std::env::remove_var("AV825_JUDGE_SABOTAGE");
        assert!(
            report.results.iter().any(|row| {
                row.status == Status::Fail && row.mismatches.iter().any(|m| m.kind == key)
            }),
            "missing {key} must redden the parent gate"
        );
    }

    let root = std::env::temp_dir().join(format!("av825-wrong-oracle-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::copy(
        PathBuf::from(env!("AUTO_LANG_HOST_DIR")).join("../auto-lang/test/cases/ace-m1/src/B01.at"),
        root.join("src/B01.at"),
    )
    .unwrap();
    std::fs::write(
        root.join("wrong.case.ad"),
        "```test-case\ncase { schema: \"autolang.case/2\", id: \"av.loaded.wrong\", source: { file: \"src/B01.at\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"program\" }, checks: [{ id: \"run\", adapter: \"probe\", action: \"run\", options: { recipe: \"ac-av/loaded-run-v1\", av: { entry: \"d_main\" }, steps: [{ op: \"run\" }] }, expect: { completion: \"success\", exit: 15 } }] }\n```\n",
    )
    .unwrap();
    std::fs::write(
        root.join("wrong.suite.ad"),
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"av825-wrong\", include: [\"wrong.case.ad\"] }\n```\n",
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
