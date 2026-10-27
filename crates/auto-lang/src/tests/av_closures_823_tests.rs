//! PLAN-823: the av-data-823 corpus — the frozen closures first case
//! (identity receipts in the corpus README) through the formal
//! `ac-av/closures-run-v1` recipe, judged PARENT-SIDE by the unified
//! runner: the closures face against the runner's OWN frozen
//! expectation table (result + live=0 + alloc/free order). Mirrors the
//! av-data-821 corpus gate shape.

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "atc-av-closures-823-{}-{}",
        tag,
        std::process::id()
    ));
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

fn closures_suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../auto-lang/test/cases/av-data/823")
}

/// V-01/V-10 first case (formal path): the av-data-823 corpus runs END TO
/// END through the unified runner — a real av worker process, real store
/// on disk, parent-side oracle judgement (the frozen closures table
/// lives in the runner; the worker never sees expected outcomes). The
/// suite's checked population is asserted (an empty or shrunken suite can
/// never pass this gate silently).
#[test]
fn av_closures_823_corpus_green() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("corpus");
    let out = out_of(&root, "out");
    let report = run_suite(
        &closures_suite_dir().join("av-data-823.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("av-data-823 suite runs");
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
        "av-data-823 corpus all green ({} results): {:#?}",
        report.results.len(),
        failing
    );
    assert!(
        !report.results.is_empty(),
        "an empty corpus is not a pass — the suite must carry its checks"
    );
    assert!(
        report.load_errors.is_empty(),
        "every included case must load — load errors: {:#?}",
        report.load_errors
    );
    let ids: Vec<&str> = report.results.iter().map(|r| r.check_id.as_str()).collect();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort_unstable();
    sorted_ids.dedup();
    assert_eq!(
        sorted_ids.len(),
        ids.len(),
        "duplicated check ids in the report: {ids:?}"
    );
    for required in ["run-closures-fixture"] {
        assert!(
            ids.contains(&required),
            "required check {required}: {ids:?}"
        );
    }
}

/// V-10 negative oracle: with the parent's frozen expectation sabotaged
/// (test-only seam `AV823_CLOSURES_EXPECT_SABOTAGE`), the LEGAL fixture
/// run must FAIL — a wrong result face AND a wrong ledger face (rotated
/// free order) each redden their own axis.
#[test]
fn av_closures_823_negative_oracle_fails() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    for (seam, want_axis) in [("result", "closures-result"), ("ledger", "closures-frees")] {
        let root = temp_root(&format!("neg-{seam}"));
        let out = out_of(&root, "out");
        std::env::set_var("AV823_CLOSURES_EXPECT_SABOTAGE", seam);
        let report = run_suite(
            &closures_suite_dir().join("av-data-823.suite.ad"),
            &out,
            &Limits::defaults(),
        );
        std::env::remove_var("AV823_CLOSURES_EXPECT_SABOTAGE");
        let report = report.expect("suite runs (the sabotage is judge-side)");
        let row = report
            .results
            .iter()
            .find(|r| r.check_id == "run-closures-fixture")
            .expect("fixture check present");
        assert_eq!(
            row.status,
            auto_test_case::report::Status::Fail,
            "the `{seam}` sabotage must redden the legal fixture run: {:?} {:?}",
            row.detail,
            row.mismatches
        );
        assert!(
            row.mismatches.iter().any(|m| m.kind == want_axis),
            "the mismatch must name the {want_axis} axis: {:?}",
            row.mismatches
        );
    }
}

/// V-10 worker-contract faults on the CLOSURES recipe: a non-zero
/// worker exit, a trailing protocol frame and a wrong source hash are
/// INFRA (Error) outcomes — never green language verdicts.
#[test]
fn av_closures_823_worker_faults_are_infra() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("faults");
    let run_one = |tag: &str| -> auto_test_case::report::Status {
        let case_dir = root.join(tag);
        let out_dir = root.join(format!("{tag}-out"));
        let _ = std::fs::remove_dir_all(&case_dir);
        let _ = std::fs::remove_dir_all(&out_dir);
        std::fs::create_dir_all(&case_dir).unwrap();
        let seed =
            std::fs::read(closures_suite_dir().join("src/first-case.at")).expect("corpus seed");
        std::fs::create_dir_all(case_dir.join("src")).unwrap();
        std::fs::write(case_dir.join("src/first-case.at"), seed).unwrap();
        std::fs::write(
            case_dir.join("c.case.ad"),
            "# fault\n```test-case\ncase {\n  schema: \"autolang.case/2\",\n  id: \"av.closures.fault\",\n  source: { file: \"src/first-case.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"program\" },\n  checks: [\n    { id: \"run\", adapter: \"probe\", action: \"run\",\n      options: { recipe: \"ac-av/closures-run-v1\", av: { entry: \"closures:first-case\" }, steps: [ { op: \"run\" } ] },\n      expect: { completion: \"success\" } }\n  ]\n}\n```\n",
        )
        .unwrap();
        std::fs::write(
            case_dir.join("s.suite.ad"),
            "# s\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s823f\", include: [\"c.case.ad\"] }\n```\n",
        )
        .unwrap();
        let report = run_suite(&case_dir.join("s.suite.ad"), &out_dir, &Limits::defaults())
            .expect("fault suite runs");
        report
            .results
            .iter()
            .find(|r| r.check_id == "run")
            .unwrap()
            .status
    };
    for fault in ["exit9", "extra-frame", "wrong-hash", "wrong-core-version"] {
        std::env::set_var("AV798_FAULT", fault);
        let status = run_one(fault);
        std::env::remove_var("AV798_FAULT");
        assert_eq!(
            status,
            auto_test_case::report::Status::Error,
            "worker fault {fault} must be infra on the closures recipe"
        );
    }
}

#[test]
fn av_closures_823_missing_case_is_not_green() {
    let root = temp_root("missing-case");
    std::fs::write(root.join("s.suite.ad"), "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"missing823\", include: [\"missing.case.ad\"] }\n```\n").unwrap();
    let result = run_suite(
        &root.join("s.suite.ad"),
        &root.join("out"),
        &Limits::defaults(),
    );
    match result {
        Err(_) => {}
        Ok(report) => assert!(
            !report.load_errors.is_empty()
                || report
                    .results
                    .iter()
                    .any(|r| r.status != auto_test_case::report::Status::Pass),
            "missing case must not pass"
        ),
    }
}

#[test]
fn av_closures_823_missing_worker_is_not_green() {
    const CHILD: &str = "AV823_MISSING_WORKER_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let root = temp_root("missing-worker");
        let result = run_suite(
            &closures_suite_dir().join("av-data-823.suite.ad"),
            &root.join("out"),
            &Limits::defaults(),
        );
        match result {
            Err(_) => {}
            Ok(report) => {
                assert!(!report.results.is_empty(), "case must have been selected");
                assert!(
                    report
                        .results
                        .iter()
                        .any(|r| r.status != auto_test_case::report::Status::Pass),
                    "missing worker must not pass"
                );
            }
        }
        return;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "tests::av_closures_823_tests::av_closures_823_missing_worker_is_not_green",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env(
            "AC_AV_CASE_WORKER",
            std::env::temp_dir().join(format!(
                "av823-definitely-missing-{}.exe",
                std::process::id()
            )),
        )
        .output()
        .expect("start isolated identity-cache process");
    assert!(
        output.status.success(),
        "isolated missing worker control: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "child must execute the control"
    );
}
