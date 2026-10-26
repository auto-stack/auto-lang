//! PLAN-824: the av-data-824 corpus — the frozen dynamic first case
//! (identity receipts in the corpus README) through the formal
//! `ac-av/dynamic-run-v1` recipe, judged PARENT-SIDE by the unified
//! runner: the dynamic face against the runner's OWN frozen expectation
//! table (result + live=0 + alloc/free order + the portable round-trip
//! axis). Mirrors the av-data-822 corpus gate shape.

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

fn temp_root(tag: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("atc-av-dynamic-824-{}-{}", tag, std::process::id()));
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

fn dynamic_suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../auto-lang/test/cases/av-data/824")
}

/// V-01/V-10 first case (formal path): the av-data-824 corpus runs END TO
/// END through the unified runner — a real av worker process, real store
/// on disk, parent-side oracle judgement (the frozen dynamic table lives
/// in the runner; the worker never sees expected outcomes). The suite's
/// checked population is asserted (an empty or shrunken suite can never
/// pass this gate silently).
#[test]
fn av_dynamic_824_corpus_green() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("corpus");
    let out = out_of(&root, "out");
    let report = run_suite(
        &dynamic_suite_dir().join("av-data-824.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("av-data-824 suite runs");
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
        "av-data-824 corpus all green ({} results): {:#?}",
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
    for required in ["run-dynamic-fixture", "run-dynamic-take-probe"] {
        assert!(
            ids.contains(&required),
            "required check {required}: {ids:?}"
        );
    }
}

/// V-10 negative oracle: with the parent's frozen expectation sabotaged
/// (test-only seam `AV824_DYNAMIC_EXPECT_SABOTAGE`), the LEGAL fixture run
/// must FAIL — a wrong result face AND a wrong ledger face (rotated free
/// order) each redden their own axis.
#[test]
fn av_dynamic_824_negative_oracle_fails() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    for (seam, want_axis) in [("result", "dynamic-result"), ("ledger", "dynamic-frees")] {
        let root = temp_root(&format!("neg-{seam}"));
        let out = out_of(&root, "out");
        std::env::set_var("AV824_DYNAMIC_EXPECT_SABOTAGE", seam);
        let report = run_suite(
            &dynamic_suite_dir().join("av-data-824.suite.ad"),
            &out,
            &Limits::defaults(),
        );
        std::env::remove_var("AV824_DYNAMIC_EXPECT_SABOTAGE");
        let report = report.expect("suite runs (the sabotage is judge-side)");
        let row = report
            .results
            .iter()
            .find(|r| r.check_id == "run-dynamic-fixture")
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

/// V-10 worker-contract faults on the DYNAMIC recipe: a non-zero worker
/// exit, a trailing protocol frame and a wrong source hash are INFRA
/// (Error) outcomes — never green language verdicts.
#[test]
fn av_dynamic_824_worker_faults_are_infra() {
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
            std::fs::read(dynamic_suite_dir().join("src/first-case.at")).expect("corpus seed");
        std::fs::create_dir_all(case_dir.join("src")).unwrap();
        std::fs::write(case_dir.join("src/first-case.at"), seed).unwrap();
        std::fs::write(
            case_dir.join("c.case.ad"),
            "# fault\n```test-case\ncase {\n  schema: \"autolang.case/2\",\n  id: \"av.dynamic.fault\",\n  source: { file: \"src/first-case.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"program\" },\n  checks: [\n    { id: \"run\", adapter: \"probe\", action: \"run\",\n      options: { recipe: \"ac-av/dynamic-run-v1\", av: { entry: \"dynamic:first-case\" }, steps: [ { op: \"run\" } ] },\n      expect: { completion: \"success\" } }\n  ]\n}\n```\n",
        )
        .unwrap();
        std::fs::write(
            case_dir.join("s.suite.ad"),
            "# s\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s824f\", include: [\"c.case.ad\"] }\n```\n",
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
    for fault in ["exit9", "extra-frame", "wrong-hash"] {
        std::env::set_var("AV798_FAULT", fault);
        let status = run_one(fault);
        std::env::remove_var("AV798_FAULT");
        assert_eq!(
            status,
            auto_test_case::report::Status::Error,
            "worker fault {fault} must be infra on the dynamic recipe"
        );
    }
}
