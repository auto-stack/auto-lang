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

/// V-10: an unregistered recipe is a DECODE admission error — a case can
/// never name an executable; lint rejects the suite outright.
#[test]
fn cte_adapter_unknown_recipe_rejected_at_lint() {
    let root = temp_root("unknown-recipe");
    let cte_dir = root.join("cte");
    std::fs::create_dir_all(&cte_dir).unwrap();
    std::fs::copy(
        suite_dir_path().join("cte").join("first-sum42.atomdata"),
        cte_dir.join("first-sum42.atomdata"),
    )
    .unwrap();
    let bad = std::fs::read_to_string(suite_dir_path().join("cte-first-sum42.case.ad"))
        .unwrap()
        .replace("cte/scalar-query-v1", "cte/not-registered-v9")
        .replace("cte.first.sum42", "cte.first.unknownrecipe");
    std::fs::write(root.join("bad.case.ad"), bad).unwrap();
    std::fs::write(
        root.join("bad.suite.ad"),
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"bad-recipe\", include: [ \"bad.case.ad\" ] }\n```\n",
    )
    .unwrap();
    let out = root.parent().unwrap().join("atc-ac-cte-unknown-out");
    let err = run_suite(&root.join("bad.suite.ad"), &out, &Limits::defaults())
        .expect_err("load must fail");
    assert!(
        err.contains("not-registered") || err.contains("registered"),
        "the rejection names the registered recipes: {err}"
    );
}

fn suite_dir_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../auto-lang/test/cases/ac-cte")
}

/// V-10 ERROR control: a broken worker tool identity surfaces as an
/// ERROR row (infrastructure), never a language FAIL and never green.
#[test]
fn cte_adapter_bogus_worker_is_error_not_green() {
    let _guard = CTE_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // cmd.exe answers nothing on stdout: the protocol admission fails.
    std::env::set_var("AC_CTE_CASE_WORKER", r"C:\Windows\System32\cmd.exe");
    let out = temp_root("bogus-worker")
        .parent()
        .unwrap()
        .join("atc-ac-cte-bogus-out");
    let report = run_suite(&suite_dir_path().join("ac-cte.suite.ad"), &out, &Limits::defaults());
    std::env::remove_var("AC_CTE_CASE_WORKER");
    let report = report.expect("the suite still runs");
    let failing: Vec<_> = report
        .results
        .iter()
        .filter(|r| r.status != Status::Pass)
        .collect();
    assert!(
        !failing.is_empty(),
        "a broken tool identity can never be green"
    );
    assert!(
        failing
            .iter()
            .all(|r| r.status == Status::Error),
        "every failing row is an ERROR (infra), not a language FAIL: {:?}",
        failing.iter().map(|r| (&r.status, &r.detail)).collect::<Vec<_>>()
    );
}

/// V-10 stale: mixed cte/av rows keep their OWN worker identity axes
/// regardless of row order, and a drifted axis goes stale.
#[test]
fn cte_adapter_stale_identity_order_independent_and_drift_red() {
    use crate::test_cases::{ac_av, ac_cte, stale};
    // PLAN-881 review F-2: this test hashes the cte worker binary TWICE
    // (report axes, then stale::check's current axes) while the
    // bogus-worker test mutates AC_CTE_CASE_WORKER mid-flight — an
    // unlocked parallel run hashed cmd.exe (15cd209c) once and the real
    // exe (2c79bac2) the second time, a deterministic false drift. The
    // file's own lock discipline applies here too.
    let _guard = CTE_CASE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = temp_root("stale-mixed");
    let deck = std::fs::read_to_string(
        suite_dir_path().join("cte").join("first-sum42.atomdata"),
    )
    .unwrap();
    let av_src = "fn main() int {\n    return 1\n}\n";
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src").join("cte.atomdata"), &deck).unwrap();
    std::fs::write(root.join("src").join("av.at"), av_src).unwrap();
    std::fs::write(
        root.join("av.case.ad"),
        format!(
            "# av\n```test-case\ncase {{ schema: \"autolang.case/2\", id: \"avrow\", source: {{ file: \"src/av.at\", syntax: \"auto\", path: \"m.at\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"run\", adapter: \"probe\", action: \"run\", options: {{ recipe: \"ac-av/core-run-v1\", av: {{ entry: \"d_main\" }}, steps: [ {{ op: \"run\" }} ] }}, expect: {{ completion: \"success\", exit: 1 }} }} ] }}\n```\n"
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("cte.case.ad"),
        format!(
            "# cte\n```test-case\ncase {{ schema: \"autolang.case/2\", id: \"cterow\", source: {{ file: \"src/cte.atomdata\", syntax: \"atom-data\", path: \"deck.atomdata\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"query\", adapter: \"probe\", action: \"run\", options: {{ recipe: \"cte/scalar-query-v1\", cte: {{ entry: \"main\" }}, steps: [ {{ op: \"query\" }} ] }}, expect: {{ completion: \"success\", exit: 42, probe: [ {{ cache: \"published\" }} ] }} }} ] }}\n```\n"
        ),
    )
    .unwrap();
    std::fs::write(
        root.join("mixed.suite.ad"),
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"cte-mixed\", include: [\"av.case.ad\", \"cte.case.ad\"] }\n```\n",
    )
    .unwrap();
    let suite = root.join("mixed.suite.ad");
    let limits = Limits::defaults();
    let identities = stale::current_identities(&suite, &limits).unwrap();
    assert_eq!(identities.len(), 2, "both checks resolve");
    let axes = [
        ac_av::current_toolchain_identity(),
        ac_cte::current_toolchain_identity(),
    ];
    let rows: Vec<_> = [("avrow", "run", 0), ("cterow", "query", 1)]
        .iter()
        .map(|(case, check, i)| {
            let hashes = &identities[&format!("{case}:{check}")].0;
            serde_json::json!({
                "case": case, "check": check, "adapter": "probe",
                "identity": {
                    "suite": hashes.suite, "case_file": hashes.case_file,
                    "source": hashes.source, "oracle": hashes.oracle,
                    "contract": hashes.contract,
                },
                "native": {"toolchain": axes[*i].iter().collect::<Vec<_>>()},
            })
        })
        .collect();
    for order in [[0, 1], [1, 0]] {
        let report = serde_json::json!({
            "toolchain": stale::toolchain_axes(&limits),
            "results": order.map(|i| rows[i].clone()),
        });
        let verdicts = stale::check(&suite, &report).unwrap();
        assert!(
            verdicts.iter().all(|v| matches!(v, stale::RowVerdict::Fresh { .. })),
            "each recipe keeps its own axes, order={order:?}: {verdicts:?}"
        );
    }
    // A drifted cte worker hash goes stale.
    let mut bad = rows.clone();
    for pair in bad[1]["native"]["toolchain"].as_array_mut().unwrap() {
        if pair[0] == "cte-worker-exe-sha256" {
            pair[1] = serde_json::json!("changed-worker-content");
        }
    }
    let report = serde_json::json!({
        "toolchain": stale::toolchain_axes(&limits),
        "results": bad,
    });
    let verdicts = stale::check(&suite, &report).unwrap();
    assert!(
        verdicts.iter().any(|v| !matches!(v, stale::RowVerdict::Fresh { .. })),
        "the drifted cte worker axis is stale: {verdicts:?}"
    );
}
