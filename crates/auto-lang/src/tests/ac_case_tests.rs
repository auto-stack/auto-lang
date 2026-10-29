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
fn syntax_parse_and_edit_dispatch_with_goldens() {
    // T-06: syntax parse/edit run through the real dispatch (the merged
    // 787 runtime); the lossless reconstruction golden IS the input bytes,
    // and an edit's golden is the edited text. A WRONG golden must FAIL.
    let root = temp_root("syntax-dispatch");
    write(
        &root,
        "t04ac.suite.ad",
        &format!(
            "```test-suite
suite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"syn.case.ad\"] }}
```
"
        ),
    );
    // Golden sidecars live next to the suite (fixture_root default ".").
    write(&root, "p04-reconstructed.atom", P04);
    let p04_edited = P04.replace("3", "4");
    write(&root, "p04-edited.atom", &p04_edited);
    // Byte offset of the loop bound for the edit step.
    let pos3 = P04.find('3').unwrap();
    write(&root, "syn.case.ad", &format!(
        "```test-case
case {{ schema: \"autolang.case/2\", id: \"ac.syn\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"parse-ok\", adapter: \"syntax\", action: \"parse\", expect: {{ completion: \"success\", artifacts: [ {{ path: \"reconstructed.at\", file: \"p04-reconstructed.atom\" }} ] }} }}, {{ id: \"edit-ok\", adapter: \"syntax\", action: \"edit\", options: {{ grammar: \"auto-core\", edits: [ {{ start: {pos3}, end: {}, insert: \"4\" }} ] }}, expect: {{ completion: \"success\", artifacts: [ {{ path: \"reconstructed.at\", file: \"p04-edited.atom\" }} ] }} }}, {{ id: \"edit-red\", adapter: \"syntax\", action: \"parse\", expect: {{ completion: \"success\", artifacts: [ {{ path: \"reconstructed.at\", file: \"p04-edited.atom\" }} ] }} }} ] }}
```
",
        pos3 + 1
    ));
    let report = run(&root, "out");
    let rows: Vec<_> = report
        .results
        .iter()
        .map(|r| (r.check_id.clone(), r.status, r.detail.clone()))
        .collect();
    assert_eq!(status_of(&report, "parse-ok"), Status::Pass, "{:?}", rows);
    assert_eq!(status_of(&report, "edit-ok"), Status::Pass, "{:?}", rows);
    // Red control: the unedited reconstruction against the EDITED golden
    // must FAIL on bytes — a good expectation never passes on bad bytes.
    let red = report
        .results
        .iter()
        .find(|r| r.check_id == "edit-red")
        .unwrap();
    assert_eq!(red.status, Status::Fail);
    assert!(
        red.mismatches
            .iter()
            .any(|m| m.observation.contains("reconstructed")),
        "{:#?}",
        red.mismatches
    );
    // Native records carry the syntax observation axes (revision/grammar).
    let edit_row = report
        .results
        .iter()
        .find(|r| r.check_id == "edit-ok")
        .unwrap();
    let native = edit_row.native.as_ref().expect("native record");
    let rev = native
        .counters
        .iter()
        .find(|(k, _)| k == "revision")
        .map(|(_, v)| *v);
    assert_eq!(rev, Some(1), "{:?}", native.counters);
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

#[test]
fn native_cache_runner_reuses_store_within_run() {
    let root = temp_root("cache-run");
    write(
        &root,
        "t04ac.suite.ad",
        &format!(
            "```test-suite
suite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"two.case.ad\"] }}
```
"
        ),
    );
    // Two independent ac/run checks over the SAME source: the per-run
    // unified store makes the second an object hit (frontend skipped,
    // link+run real) — the native records must show both truths.
    write(&root, "two.case.ad", &format!(
        "```test-case
case {{ schema: \"autolang.case/2\", id: \"ac.two\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"r1\", adapter: \"ac\", action: \"run\", expect: {{ completion: \"success\", exit: 3 }} }}, {{ id: \"r2\", adapter: \"ac\", action: \"run\", expect: {{ completion: \"success\", exit: 3 }} }} ] }}
```
"
    ));
    let report = run(&root, "out");
    let rows: Vec<_> = report
        .results
        .iter()
        .map(|r| (r.check_id.clone(), r.status, r.detail.clone()))
        .collect();
    assert_eq!(status_of(&report, "r1"), Status::Pass, "{:?}", rows);
    assert_eq!(status_of(&report, "r2"), Status::Pass, "{:?}", rows);
    let object_hit = |check: &str| {
        report
            .results
            .iter()
            .find(|r| r.check_id == check)
            .unwrap()
            .native
            .as_ref()
            .unwrap()
            .cache_hits
            .iter()
            .find(|(k, _)| k == "object")
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    assert_eq!(object_hit("r1"), "lowered", "first check computes");
    assert_eq!(
        object_hit("r2"),
        "hit",
        "second check consumes the store hit"
    );
    // PASS is never cached: both checks re-observed the exit oracle.
    for check in ["r1", "r2"] {
        let row = report.results.iter().find(|r| r.check_id == check).unwrap();
        assert_eq!(row.native.as_ref().unwrap().exit_code, Some(3));
    }
}

// ---- PLAN-788 R1 repairs: parent-side admission, consumer reds, report --

#[test]
fn r1_response_admission_table() {
    use crate::test_cases::ac::validate_response;
    let good = serde_json::json!({
        "schema": "acworker/1", "nonce": "n", "request_hash": "h",
        "ok": {
            "diagnostics": [],
            "artifacts": [],
            "run": { "exit_code": 3, "stdout_b64": "", "stderr_b64": "" },
            "trace": [],
            "toolchain": { "lld_path": "", "lld_sha256": "", "sdk_um": "", "ac_core_version": "", "worker_version": "", "worker_exe_sha256": "abc" },
            "metric_gaps": []
        },
    });
    validate_response(&good, "n", "h", "run").expect("well-formed response accepted");
    // F03: wrong schema, ok+error both, neither, missing run on a run
    // request, identity mismatch — each is a controlled refusal.
    let bad_schema = serde_json::json!({ "schema": "acworker/999", "nonce": "n", "request_hash": "h", "ok": {} });
    assert!(validate_response(&bad_schema, "n", "h", "check").is_err());
    let both = serde_json::json!({ "schema": "acworker/1", "nonce": "n", "request_hash": "h", "ok": {}, "error": {"code":"x","message":"y"} });
    assert!(validate_response(&both, "n", "h", "check").is_err());
    let neither = serde_json::json!({ "schema": "acworker/1", "nonce": "n", "request_hash": "h" });
    assert!(validate_response(&neither, "n", "h", "check").is_err());
    let no_run =
        serde_json::json!({ "schema": "acworker/1", "nonce": "n", "request_hash": "h", "ok": {} });
    assert!(validate_response(&no_run, "n", "h", "run").is_err());
    let foreign = serde_json::json!({ "schema": "acworker/1", "nonce": "other", "request_hash": "h", "ok": {} });
    assert!(validate_response(&foreign, "n", "h", "check").is_err());
}

#[test]
fn r1_consumer_invalid_exe_is_error_not_pass() {
    // F02 red shape: an invalid product exe makes the consumer verdict an
    // ERROR — the runner must propagate it (gate 2), never a PASS.
    use crate::test_cases::ac::consume_and_run;
    use auto_test_case::contract::{CacheMode, Options};
    let root = temp_root("bad-product");
    let exe = root.join("app.exe");
    std::fs::write(&exe, b"not a PE executable").unwrap();
    let check = auto_test_case::contract::CheckSpec {
        id: "red".into(),
        adapter: auto_test_case::contract::Adapter::Ac,
        action: auto_test_case::contract::Action::Run,
        tier: None,
        options: Options::default(),
        expect: auto_test_case::contract::ExpectSpec {
            completion: auto_test_case::contract::Completion::Success,
            result: None,
            stdout: None,
            stderr: None,
            artifacts: vec![],
            exit: Some(3),
            perf: None,
            probe: None,
            av_channel: None,
        },
        product_from: Some("upstream".into()),
        skip: None,
    };
    let _ = CacheMode::Reuse;
    let verdict = consume_and_run(&exe, &check, Some(3), None, None, None)
        .expect("consumer execution completes with a verdict");
    assert_eq!(verdict.status, Status::Error, "{:?}", verdict.detail);
    assert!(
        verdict
            .detail
            .as_deref()
            .unwrap_or_default()
            .contains("ac consumer run failed")
            || verdict
                .detail
                .as_deref()
                .unwrap_or_default()
                .contains("run"),
        "{:?}",
        verdict.detail
    );
}

#[test]
fn r1_runner_report_v2_legal_json_schema_and_native_axes() {
    // F06/F07: a real v2 run emits autolang.test-report/2, the JSON parses
    // with a standard parser, native rows carry the verifiable identity
    // axes, and the stale checker reads the report back.
    let root = temp_root("report-v2");
    write(&root, "t04ac.suite.ad", &format!(
        "```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"r.case.ad\"] }}\n```\n"
    ));
    write(&root, "r.case.ad", &format!(
        "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ac.r\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ {{ id: \"run\", adapter: \"ac\", action: \"run\", expect: {{ completion: \"success\", exit: 3 }} }} ] }}\n```\n"
    ));
    let report = run(&root, "out");
    assert_eq!(
        report.schema, "autolang.test-report/2",
        "suite/2 → report/2"
    );
    let json = report.to_json();
    let v: serde_json::Value = serde_json::from_str(&json).expect("runner report is legal JSON");
    assert_eq!(v["schema"], "autolang.test-report/2");
    let row = &v["results"][0];
    assert!(row["native"]["exit_code"].is_i64(), "native record present");
    let axes: Vec<(String, String)> = row["native"]["toolchain"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p[0].as_str().unwrap().to_string(),
                p[1].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert!(
        axes.iter()
            .any(|(k, val)| k == "worker-exe-sha256" && !val.is_empty()),
        "identity axes: {:?}",
        axes
    );
    // Stale reads the SAME report: identical pinned axes → Fresh; a
    // different worker → Stale with the axis named (F07 red control,
    // without env mutation).
    let same: std::collections::BTreeMap<String, String> = axes.clone().into_iter().collect();
    let verdicts = crate::test_cases::stale::check_with_native(
        &root.join("t04ac.suite.ad"),
        &v,
        Some(same.clone()),
    )
    .expect("stale reads the report");
    assert!(
        verdicts
            .iter()
            .all(|x| matches!(x, crate::test_cases::stale::RowVerdict::Fresh { .. })),
        "matching axes stay fresh: {:?}",
        verdicts
    );
    let mut drifted = same.clone();
    drifted.insert("worker-exe-sha256".into(), "differentworkerhash0000".into());
    let verdicts = crate::test_cases::stale::check_with_native(
        &root.join("t04ac.suite.ad"),
        &v,
        Some(drifted),
    )
    .expect("stale reads the report again");
    assert!(
        verdicts
            .iter()
            .any(|x| matches!(x, crate::test_cases::stale::RowVerdict::Stale { .. })),
        "a changed worker must be stale: {:?}",
        verdicts
    );
}

#[test]
fn r1_backend_axis_mirror_matches_the_real_worker() {
    // PLAN-871 T-05: the parent recomputes the native backend axis for stale
    // comparison WITHOUT linking ac-mn (zero-Cranelift boundary), so its
    // mirror must reproduce the worker's own tag exactly — on both backends
    // and under a profile-scoped rollback. Env travels per CHILD process,
    // never through this test process.
    for envv in [None, Some("legacy"), Some("mn")] {
        let tag = crate::test_cases::ac::probe_backend_tag("core", envv)
            .unwrap_or_else(|e| panic!("probe {envv:?}: {e}"));
        let mirrored = crate::test_cases::ac::backend_tag_for_env(envv, "core");
        assert_eq!(tag, mirrored, "env {envv:?}: worker vs mirror");
    }
    // A profile-scoped rollback: core back on the legacy lowering, scalar
    // still on the switched default.
    let core = crate::test_cases::ac::probe_backend_tag("core", Some("core=legacy"))
        .expect("scoped core probe");
    let scalar = crate::test_cases::ac::probe_backend_tag("scalar", Some("core=legacy"))
        .expect("scoped scalar probe");
    assert_eq!(core, crate::test_cases::ac::AC_LEGACY_BACKEND_TAG);
    assert_eq!(scalar, crate::test_cases::ac::mn_backend_tag());
    assert_eq!(
        crate::test_cases::ac::backend_tag_for_env(Some("core=legacy"), "core"),
        core
    );
    assert_eq!(
        crate::test_cases::ac::backend_tag_for_env(Some("core=legacy"), "scalar"),
        scalar
    );
    // A malformed value must never match a real tag (every row goes stale).
    let bad = crate::test_cases::ac::backend_tag_for_env(Some("core=clif"), "core");
    assert!(bad.starts_with("invalid:"), "{bad}");
    assert_ne!(bad, core);
    assert_ne!(bad, scalar);
}

#[test]
fn r1_consumer_streams_observed_not_rejected() {
    // F02: an ac/run product consumer's stdout observation is supported
    // (the old generic guard rejected it as an admission error).
    let root = temp_root("consumer-streams");
    write(&root, "t04ac.suite.ad", &format!(
        "```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"t04ac\", include: [\"cs.case.ad\"] }}\n```\n"
    ));
    write(&root, "cs.case.ad", &format!(
        "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ac.cs\", source: {{ text: \"{P04}\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [ \
{{ id: \"bld\", adapter: \"ac\", action: \"build\", expect: {{ completion: \"success\" }} }}, \
{{ id: \"cns\", adapter: \"ac\", action: \"run\", product_from: \"bld\", expect: {{ completion: \"success\", exit: 3, stdout: {{ text: \"\" }} }} }} ] }}\n```\n"
    ));
    let report = run(&root, "out");
    let rows: Vec<_> = report
        .results
        .iter()
        .map(|r| (r.check_id.clone(), r.status, r.detail.clone()))
        .collect();
    assert_eq!(status_of(&report, "bld"), Status::Pass, "{:?}", rows);
    assert_eq!(status_of(&report, "cns"), Status::Pass, "{:?}", rows);
}

#[test]
fn ac_native_suite_full_run_and_ledger_reconcile() {
    // T-07: the ≥24-case corpus through the REAL runner. The two red
    // controls FAIL by design; everything else must PASS; the ledger maps
    // every case to its legacy source and gates.
    use std::collections::BTreeSet;
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ac-native");
    let out = std::env::temp_dir().join(format!("ac-native-out-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    let report = crate::test_cases::run_suite(
        &suite_dir.join("ac-native.suite.ad"),
        &out,
        &auto_test_case::admission::Limits::defaults(),
    )
    .expect("ac-native suite runs");
    let case_count = report
        .results
        .iter()
        .map(|r| r.case_id.clone())
        .collect::<BTreeSet<_>>()
        .len();
    assert!(case_count >= 24, "corpus floor: got {} cases", case_count);
    let mut fails = Vec::new();
    for r in &report.results {
        let expected_fail = r.check_id == "bld-bad-golden"
            || r.check_id == "run-wrong-exit"
            || r.check_id == "bld-red";
        let ok = match (expected_fail, r.status) {
            (true, Status::Fail) => true,
            (false, Status::Pass) => true,
            _ => false,
        };
        if !ok {
            fails.push((
                r.case_id.clone(),
                r.check_id.clone(),
                r.status,
                r.detail.clone(),
            ));
        }
    }
    assert!(
        fails.is_empty(),
        "{} non-conforming results: {:#?}",
        fails.len(),
        fails
    );
    // The ledger maps EVERY case id; each row names a legacy source and a
    // legacy gate (V-14 reconciliation).
    let map = std::fs::read_to_string(suite_dir.join("legacy-map.tsv")).unwrap();
    let mut mapped = BTreeSet::new();
    for (i, line) in map.lines().enumerate() {
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert!(cols.len() >= 9, "ledger row {} column count", i);
        assert!(
            !cols[0].is_empty() && !cols[4].is_empty(),
            "ledger row {} source/gate",
            i
        );
        mapped.insert(cols[5].to_string());
    }
    let suite_cases: BTreeSet<String> = report.results.iter().map(|r| r.case_id.clone()).collect();
    let unmapped: Vec<_> = suite_cases.difference(&mapped).cloned().collect();
    assert!(
        unmapped.is_empty(),
        "cases absent from the ledger: {:?}",
        unmapped
    );
    let stale_rows: Vec<_> = mapped.difference(&suite_cases).cloned().collect();
    assert!(
        stale_rows.is_empty(),
        "ledger rows naming no case: {:?}",
        stale_rows
    );
    // Aggregate gate is 1 (the two by-design FAILs break green exactly as
    // intended — the acceptance script asserts the same shape).
    assert_eq!(report.exit_code(), 1);
}
