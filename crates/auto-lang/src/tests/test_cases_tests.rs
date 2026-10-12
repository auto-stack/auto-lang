//! Plan 768 T-04: AVM adapter + runner integration against the REAL VM
//! (controls V-03/V-05/V-08 slices). Everything runs through
//! `run_suite` — the same orchestration the pilot CLI will use.

use std::path::{Path, PathBuf};

use auto_test_case::admission::Limits;
use auto_test_case::report::Status;

use crate::test_cases::run_suite;

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-avm-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(root: &Path, name: &str, content: &str) {
    std::fs::write(root.join(name), content).unwrap();
}

fn case(id: &str, body: &str) -> String {
    format!(
        "```test-case\ncase {{ schema: \"autolang.case/1\", id: \"{}\", {} }}\n```\n",
        id, body
    )
}

const SUITE_HEAD: &str = concat!(
    "# T-04 suite\n",
    "```test-suite\n",
    "suite { schema: \"autolang.suite/1\", id: \"t04\", include: [\"cases.case.ad\"], defaults: { tier: \"vm-corpus\" } }\n",
    "```\n",
);

fn build_suite(root: &Path, cases_body: &str) {
    write(root, "t04.suite.ad", SUITE_HEAD);
    write(root, "cases.case.ad", cases_body);
}

fn expr_check(id: &str, expect_extra: &str) -> String {
    format!(
        "{{ id: \"{}\", adapter: \"avm\", action: \"run\", expect: {{ completion: \"success\" {} }} }}",
        id, expect_extra
    )
}

#[test]
fn avm_positive_negative_and_independence() {
    let root = temp_root("pos-neg");
    let mut doc = String::new();
    doc.push_str("# expr ^e1
```auto
1 + 2
```
## C1
");
    doc.push_str(&case("t04.expr",
        "source: { block: \"e1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ]"));
    doc.push_str("# prog ^p1
```auto
fn main() int { return 2 + 3 * 4 }
```
## C2
");
    doc.push_str(&case("t04.prog",
        "source: { block: \"p1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"program\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"14\" } } } ]"));
    doc.push_str("# out ^o1
```auto
var x = 1
print(x + 1)
```
## C3
");
    doc.push_str(&case("t04.print",
        "source: { block: \"o1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", stdout: { text: \"2\n\", compare: \"bytes\" } } } ]"));
    doc.push_str("# div ^d1
```auto
var a = 6
var b = 0
print(a / b)
```
## C4
");
    doc.push_str(&case("t04.div0",
        "source: { block: \"d1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"run\", code: \"DivisionByZero\" } } } ]"));
    doc.push_str("# par ^x1
```auto
fn (
```
## C5
");
    doc.push_str(&case("t04.parse-err",
        "source: { block: \"x1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"parse\", message: { contains: \"unexpected token\" } } } } ]"));
    build_suite(&root, &doc);

    let out = root.join("out");
    let report = run_suite(&root.join("t04.suite.ad"), &out, &Limits::defaults()).unwrap();

    let by: std::collections::BTreeMap<String, Status> = report
        .results
        .iter()
        .map(|r| (format!("{}:{}", r.case_id, r.check_id), r.status))
        .collect();
    assert_eq!(by.get("t04.expr:vm"), Some(&Status::Pass), "report: {}", report.to_console());
    assert_eq!(by.get("t04.prog:vm"), Some(&Status::Pass), "program entry result channel");
    assert_eq!(by.get("t04.print:vm"), Some(&Status::Pass), "stdout oracle");
    assert_eq!(by.get("t04.div0:vm"), Some(&Status::Pass), "strong run-stage negative");
    assert_eq!(by.get("t04.parse-err:vm"), Some(&Status::Pass), "strong parse-stage negative");
    assert_eq!(report.exit_code(), 0);

    // Identity binding: every outcome carries non-empty hashes and a
    // suite-scoped execution key.
    for r in &report.results {
        assert!(r.execution_key.starts_with("t04:"));
        assert_eq!(r.identity.suite.len(), 64);
        assert_eq!(r.identity.source.len(), 64);
        assert_eq!(r.identity.oracle.len(), 64);
    }

    // Read-only fixtures: source suite bytes untouched by the run.
    let suite_before = std::fs::read(root.join("cases.case.ad")).unwrap();
    assert_eq!(std::fs::read(root.join("cases.case.ad")).unwrap(), suite_before);

    // V-03 independence: a WRONG oracle on one check fails exactly that
    // check; other checks keep running and passing.
    let mut doc2 = String::new();
    doc2.push_str("# expr ^e1
```auto
1 + 2
```
## C1
");
    doc2.push_str(&case("t04.expr-bad",
        "source: { block: \"e1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"999\" } } } ]"));
    doc2.push_str("# div ^d1
```auto
var a = 6
var b = 0
print(a / b)
```
## C2
");
    doc2.push_str(&case("t04.div0-still-ok",
        "source: { block: \"d1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"run\", code: \"DivisionByZero\" } } } ]"));
    build_suite(&root, &doc2);
    let report2 = run_suite(&root.join("t04.suite.ad"), &out, &Limits::defaults()).unwrap();
    assert_eq!(report2.count(Status::Pass), 1);
    assert_eq!(report2.count(Status::Fail), 1);
    assert_eq!(report2.exit_code(), 1);
    let failed = report2.results.iter().find(|r| r.status == Status::Fail).unwrap();
    assert_eq!(failed.case_id, "t04.expr-bad");
    assert!(failed.mismatches[0].to_string().contains("result[text]"), "{}", failed.mismatches[0]);
}

#[test]
fn avm_wrong_stage_expectation_fails() {
    let root = temp_root("wrong-stage");
    let mut doc = String::new();
    doc.push_str("# div ^d1
```auto
var a = 6
var b = 0
print(a / b)
```
## C
");
    doc.push_str(&case("t04.wrong-stage",
        "source: { block: \"d1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"parse\", message: { contains: \"anything\" } } } } ]"));
    build_suite(&root, &doc);
    let report = run_suite(&root.join("t04.suite.ad"), &root.join("out"), &Limits::defaults()).unwrap();
    let r = &report.results[0];
    assert_eq!(r.status, Status::Fail);
    assert!(
        r.mismatches.iter().any(|m| m.observation == "completion.stage" && m.expected == "parse" && m.actual == "run"),
        "{:?}",
        r.mismatches
    );
}

#[test]
fn avm_as_source_lowers_and_records_hash() {
    let root = temp_root("as-src");
    let mut doc = String::new();
    doc.push_str("# src ^s1
```auto
var x = 1
print(x + 1)
```
## C
");
    doc.push_str(&case("t04.as-script",
        "source: { block: \"s1\", syntax: \"auto-script\", path: \"main.as\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", stdout: { text: \"2\n\" } } } ]"));
    build_suite(&root, &doc);
    let report = run_suite(&root.join("t04.suite.ad"), &root.join("out"), &Limits::defaults()).unwrap();
    assert_eq!(report.results[0].status, Status::Pass, "{}", report.to_console());
    let lowered = report
        .results[0]
        .artifacts
        .iter()
        .find(|a| a.logical_path == "<lowered-input>")
        .expect("lowered-input hash evidence recorded for .as sources");
    assert_eq!(lowered.sha256.len(), 64);
}

#[test]
fn skip_and_coverage_gap_semantics() {
    let root = temp_root("skip-gap");
    write(
        &root,
        "t04.suite.ad",
        concat!(
            "```test-suite\n",
            "suite { schema: \"autolang.suite/1\", id: \"t04b\", include: [\"a.case.ad\", \"b.case.ad\"], required: { adapters: [\"avm\", \"a2r\"] } }\n",
            "```\n",
        ),
    );
    write(
        &root,
        "a.case.ad",
        concat!(
            "# e ^e\n```auto\n1\n```\n## C\n",
            "```test-case\ncase { schema: \"autolang.case/1\", id: \"t04b.a\", source: { block: \"e\", syntax: \"auto\", path: \"m.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"1\" } } } ] }\n```\n",
        ),
    );
    // Case b declares avm only and skips it — required a2r missing AND its
    // avm check is SKIP (does not satisfy required coverage).
    write(
        &root,
        "b.case.ad",
        concat!(
            "# e ^e\n```auto\n2\n```\n## C\n",
            "```test-case\ncase { schema: \"autolang.case/1\", id: \"t04b.b\", source: { block: \"e\", syntax: \"auto\", path: \"m.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", skip: { reason: \"flaky infra\", issue: \"PLAN-768\" }, expect: { completion: \"success\" } } ] }\n```\n",
        ),
    );
    let report = run_suite(&root.join("t04.suite.ad"), &root.join("out"), &Limits::defaults()).unwrap();
    let skipped = report.results.iter().find(|r| r.case_id == "t04b.b").unwrap();
    assert_eq!(skipped.status, Status::Skip);
    assert!(skipped.detail.as_deref().unwrap().contains("flaky infra"));
    // Coverage gaps: both cases lack a2r; b's avm is skipped so it counts too.
    assert!(report.coverage_gaps.contains(&("t04b.a".to_string(), "a2r".to_string())));
    assert!(report.coverage_gaps.contains(&("t04b.b".to_string(), "a2r".to_string())));
    assert!(report.coverage_gaps.contains(&("t04b.b".to_string(), "avm".to_string())));
    assert_eq!(report.exit_code(), 2, "required gaps fail the gate as contract errors");
}

#[test]
fn infra_errors_do_not_satisfy_language_negatives() {
    // Parse error source, but the oracle demands a RUN-stage failure: the
    // honest result is FAIL (wrong stage), never PASS.
    let root = temp_root("infra-vs-lang");
    let mut doc = String::new();
    doc.push_str("# bad ^b
```auto
fn (
```
## C
");
    doc.push_str(&case("t04.parse-as-run",
        "source: { block: \"b\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"run\", code: \"DivisionByZero\" } } } ]"));
    build_suite(&root, &doc);
    let report = run_suite(&root.join("t04.suite.ad"), &root.join("out"), &Limits::defaults()).unwrap();
    assert_eq!(report.results[0].status, Status::Fail);
    assert_eq!(report.exit_code(), 1);
}
