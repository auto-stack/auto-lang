//! PLAN-825 formal LoadedProgram corpus gate.

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::PathBuf;

#[test]
fn av_runtime_825_first_case_corpus_green() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let suite = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../auto-lang/test/cases/av-abc/825/av-abc-825.suite.ad");
    let out = std::env::temp_dir().join(format!("atc-av825-first-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let report = run_suite(&suite, &out, &Limits::defaults()).expect("av-abc-825 suite runs");
    assert!(report.load_errors.is_empty(), "{:#?}", report.load_errors);
    assert_eq!(
        report.results.len(),
        1,
        "the required first case is present"
    );
    let row = &report.results[0];
    assert_eq!(row.check_id, "run-loaded-first");
    assert_eq!(
        row.status,
        auto_test_case::report::Status::Pass,
        "detail={:?} mismatches={:?}",
        row.detail,
        row.mismatches
    );
}
