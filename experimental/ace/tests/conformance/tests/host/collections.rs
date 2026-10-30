//! PLAN-864 file-backed collections-source corpus: the 852 L/V rows plus
//! the append-only N01..N03 variants run END-TO-END through the unified
//! runner — a real av worker process, parent-side oracle judgement (the
//! worker never sees an expectation).

use auto_lang_core::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

fn temp_root(tag: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "av864-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("scratch root");
    root
}

fn out_of(root: &Path, sub: &str) -> PathBuf {
    let out = root.join(sub);
    std::fs::create_dir_all(&out).expect("scratch out");
    out
}

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("AUTO_LANG_HOST_DIR")).join("test/cases/ace-m1")
}

/// AC-01/AC-05: the file-backed corpus runs green — 17 checks (one per
/// bound row), all PASS, no load errors, no duplicated ids. The L/V rows
/// keep their 852 denominator values; L06 keeps the frozen SEM-02 trap.
#[test]
fn av_collections_corpus_green() {
    let _guard = crate::tests::av_protocol::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("corpus");
    let out = out_of(&root, "out");
    let report = run_suite(
        &suite_dir().join("collections-source-864.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("collections-864 suite runs");
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
        "collections-864 corpus all green ({} results): {:#?}",
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
    // the frozen denominator: exactly the 17 bound rows
    assert_eq!(
        ids.len(),
        17,
        "the frozen collections-source denominator is 17 rows: {ids:?}"
    );
}
