//! PLAN-817 T-02: the `cte/scalar-query-v1` probe recipe — minimal
//! official corpus execution through the unified runner plus the
//! wrong-exit control (errors never turn green). The full positive/
//! negative matrix (identity/stale/restart/compat) arrives with T-06.

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use auto_test_case::report::Status;
use std::path::PathBuf;

/// Serialize tests that spawn the real cte worker (worker identity
/// caches are process-global — same discipline as the av/sema locks).
pub(crate) static CTE_CASE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-ac-cte-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn status_of(report: &auto_test_case::report::RunReport, check: &str) -> Status {
    report
        .results
        .iter()
        .find(|r| r.check_id == check)
        .unwrap_or_else(|| panic!("check {} present in {:?}", check, report.results))
        .status
}

/// The committed ac-cte corpus runs END TO END through the unified
/// runner — a real cte_probe_worker process, the frozen store on disk,
/// REAL AV execution on the miss, consumer backfill re-derived
/// parent-verified, and the native observable judged against the
/// hand-computed oracle.
#[test]
fn cte_probe_corpus_executes_green() {
    let _guard = CTE_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../auto-lang/test/cases/ac-cte");
    let root = temp_root("corpus");
    let out = root.parent().unwrap().join("atc-ac-cte-corpus-out");
    let report = run_suite(&suite_dir.join("ac-cte.suite.ad"), &out, &Limits::defaults())
        .expect("ac-cte suite runs");
    let failing: Vec<String> = report
        .results
        .iter()
        .filter(|r| r.status != Status::Pass)
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
        "ac-cte corpus all green ({} results): {:#?}",
        report.results.len(),
        failing
    );
    assert_eq!(
        status_of(&report, "query-sum42"),
        Status::Pass,
        "sum42 official check green"
    );
    assert_eq!(
        status_of(&report, "query-bool-branch"),
        Status::Pass,
        "bool-branch official check green"
    );
}

/// Wrong-exit control: the SAME deck with a wrong hand-computed oracle
/// (exit 43) must FAIL — a bad actual can never ride a green banner.
#[test]
fn cte_probe_wrong_exit_not_green() {
    let _guard = CTE_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let suite_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../auto-lang/test/cases/ac-cte");
    let root = temp_root("wrongexit");
    let cte_dir = root.join("cte");
    std::fs::create_dir_all(&cte_dir).unwrap();
    std::fs::copy(
        suite_dir.join("cte").join("first-sum42.atomdata"),
        cte_dir.join("first-sum42.atomdata"),
    )
    .unwrap();
    let bad_case = std::fs::read_to_string(suite_dir.join("cte-first-sum42.case.ad"))
        .unwrap()
        .replace("exit: 42", "exit: 43")
        .replace("cte.first.sum42", "cte.first.sum42.wrongexit");
    std::fs::write(root.join("cte-first-sum42-wrong.case.ad"), bad_case).unwrap();
    std::fs::write(
        root.join("ac-cte-wrong.suite.ad"),
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"ac-cte-wrong\", include: [ \"cte-first-sum42-wrong.case.ad\" ] }\n```\n",
    )
    .unwrap();
    let out = root.parent().unwrap().join("atc-ac-cte-wrongexit-out");
    let report = run_suite(
        &root.join("ac-cte-wrong.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("wrong-exit suite runs");
    assert_eq!(
        status_of(&report, "query-sum42"),
        Status::Fail,
        "wrong oracle must be a language FAIL: {:?}",
        report.results
    );
}
