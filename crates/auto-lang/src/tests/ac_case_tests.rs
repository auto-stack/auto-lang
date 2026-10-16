//! PLAN-788 T-04: `ac` adapter integration through the REAL runner — the
//! ac-case-worker process is spawned, its observations judged parent-side
//! (controls V-03/V-04/V-07 slices at the auto-lang level). Windows-only:
//! the worker's native paths need the real toolchain; the protocol-level
//! negatives live in the worker's own suite.

#![cfg(windows)]

use std::path::{Path, PathBuf};

use auto_test_case::admission::Limits;
use auto_test_case::report::Status;

use crate::test_cases::run_suite;

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-ac-{}-{}", tag, std::process::id()));
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

fn write(root: &Path, name: &str, content: &str) {
    std::fs::write(root.join(name), content).unwrap();
}

const P04: &str = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
const BAD_NAME: &str = "fn main() int { return x }";

/// A minimal v2 suite with one case whose single check is supplied by the
/// caller.
fn v2_suite(check: &str) -> String {
    format!(
        concat!(
            "# T-04 v2 suite\n",
            "```test-suite\n",
            "suite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"cases.case.ad\"] }}\n",
            "```\n",
            "\n",
            "```test-case\n",
            "case {{ schema: \"autolang.case/2\", id: \"ac.case\", ",
            "source: {{ text: \"{}\", syntax: \"auto\", path: \"main.at\" }}, ",
            "entry: {{ mode: \"program\" }}, ",
            "checks: [ {} ] }}\n",
            "```\n"
        ),
        P04_ESC,
        check
    )
}

/// Escaped once for the two format layers (fixture text embeds into the
/// Atom string literal).
const P04_ESC: &str = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";

fn run(root: &Path, tag: &str) -> auto_test_case::report::RunReport {
    let out = out_of(root, tag);
    run_suite(&root.join("t04ac.suite.ad"), &out, &Limits::defaults()).expect("suite runs")
}

fn status_of(report: &auto_test_case::report::RunReport, check: &str) -> Status {
    report
        .results
        .iter()
        .find(|r| r.check_id == check)
        .unwrap_or_else(|| panic!("check {} present in {:?}", check, report.results))
        .status
}

#[test]
fn ac_native_run_exit_oracle_and_red_sibling() {
    let root = temp_root("exit");
    write(
        &root,
        "t04ac.suite.ad",
        &format!(
            concat!(
                "```test-suite\n",
                "suite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"ok.case.ad\", \"red.case.ad\"] }}\n",
                "```\n"
            )
        ),
    );
    let ok_case = format!(
        "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ac.ok\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"run\", adapter: \"ac\", action: \"run\", expect: {{ completion: \"success\", exit: 3 }} }} ] }}\n```\n"
    );
    let red_case = format!(
        "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ac.red\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"run\", adapter: \"ac\", action: \"run\", expect: {{ completion: \"success\", exit: 4 }} }} ] }}\n```\n"
    );
    write(&root, "ok.case.ad", &ok_case);
    write(&root, "red.case.ad", &red_case);

    let report = run(&root, "out");
    assert_eq!(
        status_of(&report, "run"),
        Status::Pass,
        "{:?}",
        report
            .results
            .iter()
            .map(|r| (r.check_id.clone(), r.status, r.detail.clone()))
            .collect::<Vec<_>>()
    );
    // The red sibling must FAIL on the exit mismatch — a good expectation
    // against a wrong exit is never green.
    let red = report
        .results
        .iter()
        .find(|r| r.case_id == "ac.red")
        .unwrap();
    assert_eq!(red.status, Status::Fail);
    assert!(
        red.mismatches.iter().any(|m| m.observation == "exit"),
        "{:#?}",
        red.mismatches
    );
    // v2 native record: exit observed, stage spans and toolchain recorded.
    let ok_row = report
        .results
        .iter()
        .find(|r| r.case_id == "ac.ok")
        .unwrap();
    let native = ok_row.native.as_ref().expect("native record on ac/run");
    assert_eq!(native.exit_code, Some(3));
    assert!(
        native.stage_spans.iter().any(|(s, _)| s == "run"),
        "run span present: {:?}",
        native.stage_spans
    );
    assert!(!native.toolchain.is_empty());
    // Products registered for handoff (exe manifest verified parent-side).
    assert!(
        ok_row.artifacts.iter().any(|a| a.logical_path == "app.exe"),
        "{:?}",
        ok_row.artifacts
    );
}

#[test]
fn ac_native_typed_negative_stage_attribution() {
    let root = temp_root("typed-neg");
    write(&root, "t04ac.suite.ad", &format!(
        "```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"neg.case.ad\"] }}\n```\n"
    ));
    write(&root, "neg.case.ad", &format!(
        "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ac.neg\", source: {{ text: \"{BAD_NAME}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ \
{{ id: \"ok-stage\", adapter: \"ac\", action: \"check\", expect: {{ completion: {{ stage: \"resolve\", code: \"source.undefined\" }} }} }}, \
{{ id: \"bad-stage\", adapter: \"ac\", action: \"check\", expect: {{ completion: {{ stage: \"parse\", code: \"source.syntax\" }} }} }} ] }}\n```\n"
    ));
    let report = run(&root, "out");
    // The typed resolve/undefined expectation PASSes…
    assert_eq!(
        status_of(&report, "ok-stage"),
        Status::Pass,
        "{:?}",
        report
            .results
            .iter()
            .map(|r| (r.check_id.clone(), r.status, r.detail.clone()))
            .collect::<Vec<_>>()
    );
    // …and a WRONG stage attribution FAILs — the oracle distinguishes the
    // real boundary (observer), not just "some error".
    let bad = report
        .results
        .iter()
        .find(|r| r.check_id == "bad-stage")
        .unwrap();
    assert_eq!(bad.status, Status::Fail);
    assert!(
        bad.mismatches
            .iter()
            .any(|m| m.observation == "completion.stage"),
        "{:#?}",
        bad.mismatches
    );
}

#[test]
fn ac_native_hir_atom_input_checks() {
    let root = temp_root("hir-input");
    // The REAL PLAN-741 wrapper fixture: add(2,3) via a zero-param entry.
    let hir = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experimental/ac-core/fixtures/valid/wrapper.atom"),
    )
    .unwrap();
    write(
        &root,
        "t04ac.suite.ad",
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"hir.case.ad\"] }\n```\n",
    );
    // Atom string literal: escape backslashes and quotes only — braces are
    // ordinary characters inside a quoted Atom string.
    let hir_lit = hir.replace('\\', "\\\\").replace('"', "\\\"");
    write(
        &root,
        "hir.case.ad",
        &format!(
            "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ac.hir\", source: {{ text: \"{hir_lit}\", syntax: \"hir-atom\", path: \"main.atom\" }}, entry: {{ mode: \"program\" }}, checks: [ \
{{ id: \"chk\", adapter: \"ac\", action: \"check\", expect: {{ completion: \"success\" }} }}, \
{{ id: \"rn\", adapter: \"ac\", action: \"run\", options: {{ entry: \"test_entry\" }}, expect: {{ completion: \"success\", exit: 5 }} }} ] }}\n```\n"
        ),
    );
    let report = run(&root, "out");
    let rows: Vec<_> = report
        .results
        .iter()
        .map(|r| {
            (
                r.check_id.clone(),
                r.status,
                r.detail.clone(),
                r.mismatches.clone(),
                r.native.as_ref().map(|n| n.exit_code),
            )
        })
        .collect();
    assert_eq!(status_of(&report, "chk"), Status::Pass, "{:?}", rows);
    assert_eq!(status_of(&report, "rn"), Status::Pass, "{:?}", rows);
}

#[test]
fn ac_native_syntax_check_is_admission_error_not_skip() {
    let root = temp_root("syntax-unbound");
    write(&root, "t04ac.suite.ad", &format!(
        "```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"sy.case.ad\"] }}\n```\n"
    ));
    write(&root, "sy.case.ad", &format!(
        "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ac.sy\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"sp\", adapter: \"syntax\", action: \"parse\", expect: {{ completion: \"success\" }} }} ] }}\n```\n"
    ));
    let report = run(&root, "out");
    let row = report.results.iter().find(|r| r.check_id == "sp").unwrap();
    // D-787 unmet: ERROR with the binding precondition named, never SKIP,
    // never green.
    assert_eq!(row.status, Status::Error);
    let detail = row.detail.clone().unwrap_or_default();
    assert!(detail.contains("PLAN-787"), "{}", detail);
    assert_eq!(
        report.exit_code(),
        2,
        "an unbound adapter is a contract error"
    );
}

#[test]
fn ac_product_handoff_run_consumes_real_build() {
    let root = temp_root("product");
    write(&root, "t04ac.suite.ad", &format!(
        "```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"dag.case.ad\"] }}\n```\n"
    ));
    // build → FAIL nothing (success), consumer ac/run product_from the build
    // with an exit oracle; plus a RED consumer expecting the wrong exit.
    write(&root, "dag.case.ad", &format!(
        "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ac.dag\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ \
{{ id: \"bld\", adapter: \"ac\", action: \"build\", expect: {{ completion: \"success\" }} }}, \
{{ id: \"cns\", adapter: \"ac\", action: \"run\", product_from: \"bld\", expect: {{ completion: \"success\", exit: 3 }} }}, \
{{ id: \"red\", adapter: \"ac\", action: \"run\", product_from: \"bld\", expect: {{ completion: \"success\", exit: 9 }} }} ] }}\n```\n"
    ));
    let report = run(&root, "out");
    let rows: Vec<_> = report
        .results
        .iter()
        .map(|r| (r.check_id.clone(), r.status, r.detail.clone()))
        .collect();
    assert_eq!(status_of(&report, "bld"), Status::Pass, "{:?}", rows);
    // The consumer RAN the manifest-verified exe (never recompiled).
    let cns = report.results.iter().find(|r| r.check_id == "cns").unwrap();
    assert_eq!(cns.status, Status::Pass, "{:?}", rows);
    assert_eq!(cns.native.as_ref().unwrap().exit_code, Some(3));
    assert!(cns
        .detail
        .as_deref()
        .unwrap_or_default()
        .contains("consumed"));
    // Red control: wrong expected exit on the consumed product is a FAIL.
    let red = report.results.iter().find(|r| r.check_id == "red").unwrap();
    assert_eq!(red.status, Status::Fail);
    assert!(
        red.mismatches.iter().any(|m| m.observation == "exit"),
        "{:#?}",
        red.mismatches
    );
}

#[test]
fn native_admission_v1_suite_rejects_ac_check_at_load() {
    let root = temp_root("v1-ac");
    write(&root, "t04ac.suite.ad",
        "```test-suite\nsuite { schema: \"autolang.suite/1\", id: \"t04\", include: [\"c.case.ad\"] }\n```\n");
    write(&root, "c.case.ad", &format!(
        "```test-case\ncase {{ schema: \"autolang.case/1\", id: \"ac.v1\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"r\", adapter: \"ac\", action: \"run\", expect: {{ completion: \"success\", exit: 3 }} }} ] }}\n```\n"
    ));
    let out = out_of(&root, "out");
    let err = run_suite(&root.join("t04ac.suite.ad"), &out, &Limits::defaults())
        .expect_err("v1 suite cannot carry ac checks");
    assert!(err.contains("not available"), "{}", err);
}

#[test]
fn native_admission_selection_excludes_ac_check_without_scratch() {
    let root = temp_root("select");
    write(&root, "t04ac.suite.ad", &format!(
        "```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"sel.case.ad\"] }}\n```\n"
    ));
    write(&root, "sel.case.ad", &format!(
        "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ac.sel\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"r\", adapter: \"ac\", action: \"run\", expect: {{ completion: \"success\", exit: 3 }} }} ] }}\n```\n"
    ));
    let out = out_of(&root, "out");
    let selection = crate::test_cases::Selection {
        case: Some("nothing-matches".into()),
        adapter: Some("ac".into()),
        check: None,
    };
    let report = crate::test_cases::run_suite_selected(
        &root.join("t04ac.suite.ad"),
        &out,
        &Limits::defaults(),
        &selection,
    )
    .expect_err("zero-match selector is a contract error");
    let _ = report;
    // Unselected (zero-match) is rejected before ANY scratch exists.
    assert!(!out.join("run-0").exists() || std::fs::read_dir(&out).unwrap().count() == 0);
}

// Silence the unused-helper warning for v2_suite (used compositionally by
// future T-05/T-08 tests in this family).
#[allow(dead_code)]
fn _keep_v2_suite() -> String {
    v2_suite(
        "{ id: \"x\", adapter: \"ac\", action: \"check\", expect: { completion: \"success\" } }",
    )
}
