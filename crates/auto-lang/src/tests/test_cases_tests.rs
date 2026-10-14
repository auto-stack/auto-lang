//! Plan 768 T-04: AVM adapter + runner integration against the REAL VM
//! (controls V-03/V-05/V-08 slices). Everything runs through
//! `run_suite` — the same orchestration the pilot CLI will use.

use std::path::{Path, PathBuf};

use auto_test_case::admission::Limits;
use auto_test_case::report::Status;

use crate::test_cases::run_suite;

/// Disjoint output directory for a fixture root (Plan 773 F-05: outputs
/// may not live inside the suite/fixture tree). Sibling of the fixture dir.
fn out_of(root: &Path, sub: &str) -> PathBuf {
    root.parent()
        .unwrap_or(Path::new("."))
        .join(format!("{}-{}", root.file_name().unwrap().to_string_lossy(), sub))
}

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
    doc.push_str(
        "# expr ^e1
```auto
1 + 2
```
## C1
",
    );
    doc.push_str(&case("t04.expr",
        "source: { block: \"e1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ]"));
    doc.push_str(
        "# prog ^p1
```auto
fn main() int { return 2 + 3 * 4 }
```
## C2
",
    );
    doc.push_str(&case("t04.prog",
        "source: { block: \"p1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"program\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"14\" } } } ]"));
    doc.push_str(
        "# out ^o1
```auto
var x = 1
print(x + 1)
```
## C3
",
    );
    doc.push_str(&case("t04.print",
        "source: { block: \"o1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", stdout: { text: \"2\n\", compare: \"bytes\" } } } ]"));
    doc.push_str(
        "# div ^d1
```auto
var a = 6
var b = 0
print(a / b)
```
## C4
",
    );
    doc.push_str(&case("t04.div0",
        "source: { block: \"d1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"run\", code: \"DivisionByZero\" } } } ]"));
    doc.push_str(
        "# par ^x1
```auto
fn (
```
## C5
",
    );
    doc.push_str(&case("t04.parse-err",
        "source: { block: \"x1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"parse\", message: { contains: \"unexpected token\" } } } } ]"));
    build_suite(&root, &doc);

    let out = out_of(&root, "out");
    let report = run_suite(&root.join("t04.suite.ad"), &out, &Limits::defaults()).unwrap();

    let by: std::collections::BTreeMap<String, Status> = report
        .results
        .iter()
        .map(|r| (format!("{}:{}", r.case_id, r.check_id), r.status))
        .collect();
    assert_eq!(
        by.get("t04.expr:vm"),
        Some(&Status::Pass),
        "report: {}",
        report.to_console()
    );
    assert_eq!(
        by.get("t04.prog:vm"),
        Some(&Status::Pass),
        "program entry result channel"
    );
    assert_eq!(by.get("t04.print:vm"), Some(&Status::Pass), "stdout oracle");
    assert_eq!(
        by.get("t04.div0:vm"),
        Some(&Status::Pass),
        "strong run-stage negative"
    );
    assert_eq!(
        by.get("t04.parse-err:vm"),
        Some(&Status::Pass),
        "strong parse-stage negative"
    );
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
    assert_eq!(
        std::fs::read(root.join("cases.case.ad")).unwrap(),
        suite_before
    );

    // V-03 independence: a WRONG oracle on one check fails exactly that
    // check; other checks keep running and passing.
    let mut doc2 = String::new();
    doc2.push_str(
        "# expr ^e1
```auto
1 + 2
```
## C1
",
    );
    doc2.push_str(&case("t04.expr-bad",
        "source: { block: \"e1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"999\" } } } ]"));
    doc2.push_str(
        "# div ^d1
```auto
var a = 6
var b = 0
print(a / b)
```
## C2
",
    );
    doc2.push_str(&case("t04.div0-still-ok",
        "source: { block: \"d1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"run\", code: \"DivisionByZero\" } } } ]"));
    build_suite(&root, &doc2);
    let report2 = run_suite(&root.join("t04.suite.ad"), &out, &Limits::defaults()).unwrap();
    assert_eq!(report2.count(Status::Pass), 1);
    assert_eq!(report2.count(Status::Fail), 1);
    assert_eq!(report2.exit_code(), 1);
    let failed = report2
        .results
        .iter()
        .find(|r| r.status == Status::Fail)
        .unwrap();
    assert_eq!(failed.case_id, "t04.expr-bad");
    assert!(
        failed.mismatches[0].to_string().contains("result[text]"),
        "{}",
        failed.mismatches[0]
    );
}

#[test]
fn avm_wrong_stage_expectation_fails() {
    let root = temp_root("wrong-stage");
    let mut doc = String::new();
    doc.push_str(
        "# div ^d1
```auto
var a = 6
var b = 0
print(a / b)
```
## C
",
    );
    doc.push_str(&case("t04.wrong-stage",
        "source: { block: \"d1\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"parse\", message: { contains: \"anything\" } } } } ]"));
    build_suite(&root, &doc);
    let report = run_suite(
        &root.join("t04.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    let r = &report.results[0];
    assert_eq!(r.status, Status::Fail);
    assert!(
        r.mismatches
            .iter()
            .any(|m| m.observation == "completion.stage"
                && m.expected == "parse"
                && m.actual == "run"),
        "{:?}",
        r.mismatches
    );
}

#[test]
fn avm_as_source_lowers_and_records_hash() {
    let root = temp_root("as-src");
    let mut doc = String::new();
    doc.push_str(
        "# src ^s1
```auto
var x = 1
print(x + 1)
```
## C
",
    );
    doc.push_str(&case("t04.as-script",
        "source: { block: \"s1\", syntax: \"auto-script\", path: \"main.as\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", stdout: { text: \"2\n\" } } } ]"));
    build_suite(&root, &doc);
    let report = run_suite(
        &root.join("t04.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(
        report.results[0].status,
        Status::Pass,
        "{}",
        report.to_console()
    );
    let lowered = report.results[0]
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
    let report = run_suite(
        &root.join("t04.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    let skipped = report
        .results
        .iter()
        .find(|r| r.case_id == "t04b.b")
        .unwrap();
    assert_eq!(skipped.status, Status::Skip);
    assert!(skipped.detail.as_deref().unwrap().contains("flaky infra"));
    // Coverage gaps: both cases lack a2r; b's avm is skipped so it counts too.
    assert!(report
        .coverage_gaps
        .contains(&("t04b.a".to_string(), "a2r".to_string())));
    assert!(report
        .coverage_gaps
        .contains(&("t04b.b".to_string(), "a2r".to_string())));
    assert!(report
        .coverage_gaps
        .contains(&("t04b.b".to_string(), "avm".to_string())));
    assert_eq!(
        report.exit_code(),
        2,
        "required gaps fail the gate as contract errors"
    );
}

#[test]
fn infra_errors_do_not_satisfy_language_negatives() {
    // Parse error source, but the oracle demands a RUN-stage failure: the
    // honest result is FAIL (wrong stage), never PASS.
    let root = temp_root("infra-vs-lang");
    let mut doc = String::new();
    doc.push_str(
        "# bad ^b
```auto
fn (
```
## C
",
    );
    doc.push_str(&case("t04.parse-as-run",
        "source: { block: \"b\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"run\", code: \"DivisionByZero\" } } } ]"));
    build_suite(&root, &doc);
    let report = run_suite(
        &root.join("t04.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(report.results[0].status, Status::Fail);
    assert_eq!(report.exit_code(), 1);
}

// ---------------------------------------------------------------- T-05 ---

use std::fs;

fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("test").join(rel)
}

fn mini_suite(root: &Path, id: &str) {
    fs::write(
        root.join("t05.suite.ad"),
        format!(
            "# s\n```test-suite\nsuite {{ schema: \"autolang.suite/1\", id: \"{}\", include: [\"c.case.ad\"] }}\n```\n",
            id
        ),
    )
    .unwrap();
}

#[test]
fn a2r_emit_matches_legacy_golden_bytes() {
    // Real legacy fixture (source + golden .expected.rs): the adapter path
    // (materialize -> emit -> compare) must be byte-identical to the legacy
    // test_a2r_with_base comparison.
    let root = temp_root("a2r-golden");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    fs::write(root.join("expected-hello.rs"), &golden).unwrap();
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "t05a");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05a.hello\",\n  source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"script\" },\n  checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" },\n    expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected-hello.rs\", compare: \"bytes\" } ] } } ] }\n```\n",
    )
    .unwrap();

    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(report.results.len(), 1, "{}", report.to_console());
    assert_eq!(
        report.results[0].status,
        Status::Pass,
        "{}",
        report.to_console()
    );
    let art = &report.results[0].artifacts[0];
    assert_eq!(art.logical_path, "hello.rs");
    assert_eq!(art.bytes, Some(golden.len()));
    assert_eq!(report.exit_code(), 0);

    // Actual persisted under the run scratch; oracle untouched.
    assert_eq!(fs::read(&art.stored_path).unwrap(), golden);
    assert_eq!(fs::read(root.join("expected-hello.rs")).unwrap(), golden);
}

#[test]
fn a2r_project_emission_multi_artifact_manifest() {
    let root = temp_root("a2r-proj");
    let fx = fixture("a2r/14_modules/005_multi_file");
    // Oracle sidecars are built here from ONE direct production call on a
    // MATERIALIZED copy of the same project (same layout the adapter
    // materializes): project-mode Cargo.toml metadata derives from the
    // entry's parent directory name, so the equivalence under test is
    // "adapter == direct call on the same materialized project" — path-
    // derived metadata is part of the input identity (feasibility §5 note).
    let oracle_dir = root.join("oracle").join("project");
    for name in ["main.at", "db.at", "api/mod.at", "api/handlers.at"] {
        let target = oracle_dir.join(name);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(fx.join(name), &target).unwrap();
    }
    let direct = crate::trans::rust::transpile_rust_project(
        &oracle_dir.join("main.at").display().to_string(),
    )
    .unwrap();
    let mut artifacts_decl = String::new();
    for key in [
        "main.rs",
        "db.rs",
        "api/mod.rs",
        "api/handlers.rs",
        "Cargo.toml",
    ] {
        let (bytes, _) = direct.get(key).expect(key);
        let sidecar = format!("expected-{}.txt", key.replace('/', "_"));
        fs::write(root.join(&sidecar), bytes).unwrap();
        artifacts_decl.push_str(&format!(
            "{{ path: \"{}\", file: \"{}\", compare: \"bytes\" }}, ",
            key, sidecar
        ));
    }

    mini_suite(&root, "t05b");
    let mut pd = String::new();
    for (anchor, name) in [
        ("m", "main.at"),
        ("d", "db.at"),
        ("am", "api/mod.at"),
        ("ah", "api/handlers.at"),
    ] {
        pd.push_str(&format!("# {} ^{}\n```auto\n", name, anchor));
        pd.push_str(&fs::read_to_string(fx.join(name)).unwrap());
        if !pd.ends_with('\n') {
            pd.push('\n');
        }
        pd.push_str("```\n");
    }
    pd.push_str(&format!(
        "## C\n```test-case\ncase {{ schema: \"autolang.case/1\", id: \"t05b.proj\",\n  source: {{ block: \"m\", syntax: \"auto\", path: \"main.at\" }},\n  files: [ {{ path: \"db.at\", block: \"d\" }}, {{ path: \"api/mod.at\", block: \"am\" }}, {{ path: \"api/handlers.at\", block: \"ah\" }} ],\n  entry: {{ mode: \"module\" }},\n  checks: [ {{ id: \"emit\", adapter: \"a2r\", action: \"emit\",\n    expect: {{ completion: \"success\", artifacts: [ {} ] }} }} ] }}\n```\n",
        artifacts_decl
    ));
    fs::write(root.join("c.case.ad"), &pd).unwrap();

    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(
        report.results[0].status,
        Status::Pass,
        "{}",
        report.to_console()
    );
    assert_eq!(
        report.results[0].artifacts.len(),
        5,
        "manifest lists every product"
    );
}

#[test]
fn a2r_product_handoff_golden_fail_does_not_block_consumer() {
    // Plan 773 F-03: a tampered golden makes the producer FAIL, but the
    // emission SUCCEEDED and its actual bytes exist — the consumer is NOT
    // blocked; it consumes the real actual and its own (correct) oracle
    // over those bytes PASSES.
    let root = temp_root("a2r-handoff");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    let mut bad = golden.clone();
    bad.extend_from_slice(b"// tampered\n");
    fs::write(root.join("expected-hello.rs"), &bad).unwrap();
    fs::write(root.join("good-hello.rs"), &golden).unwrap();
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "t05c");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05c.chain\",\n  source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"script\" },\n  checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" },\n      expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected-hello.rs\" } ] } },\n    { id: \"consumer\", adapter: \"a2r\", action: \"emit\", product_from: \"emit\",\n      expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"good-hello.rs\" } ] } } ] }\n```\n",
    )
    .unwrap();

    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    let emit = report
        .results
        .iter()
        .find(|r| r.check_id == "emit")
        .unwrap();
    assert_eq!(emit.status, Status::Fail, "tampered oracle must fail");
    assert!(
        fs::read(&emit.artifacts[0].stored_path).is_ok(),
        "actual persisted despite FAIL"
    );
    let consumer = report
        .results
        .iter()
        .find(|r| r.check_id == "consumer")
        .unwrap();
    assert_ne!(
        consumer.status,
        Status::Blocked,
        "a golden FAIL must NOT cut the product handoff (F-773-R1-03)"
    );
    assert_eq!(
        consumer.status,
        Status::Pass,
        "consumer judged the CONSUMED actual bytes: {}",
        consumer.detail.as_deref().unwrap_or("")
    );
    assert!(
        consumer
            .detail
            .as_deref()
            .unwrap()
            .contains("consumed 1 actual artifact(s) from \"emit\""),
        "consumer must record REAL consumption: {:?}",
        consumer.detail
    );
    assert_eq!(
        report.count(Status::Blocked),
        0,
        "no BLOCKED in a handoff with usable products"
    );
    assert_eq!(report.exit_code(), 1, "producer FAIL fails the gate");
}

#[test]
fn a2r_real_emit_failure_blocks_consumer() {
    // F-03 other side: a REAL execution failure produces no artifacts —
    // only then is the consumer BLOCKED.
    let root = temp_root("a2r-blocked");
    fs::write(root.join("expected.rs"), b"// never reached\n").unwrap();
    mini_suite(&root, "t05c2");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05c2.chain\",\n  source: { text: \"fn (\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"script\" },\n  checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\",\n      expect: { completion: \"success\", artifacts: [ { path: \"main.rs\", file: \"expected.rs\" } ] } },\n    { id: \"consumer\", adapter: \"a2r\", action: \"emit\", product_from: \"emit\",\n      expect: { completion: \"success\", artifacts: [ { path: \"main.rs\", file: \"expected.rs\" } ] } } ] }\n```\n",
    )
    .unwrap();

    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    let emit = report
        .results
        .iter()
        .find(|r| r.check_id == "emit")
        .unwrap();
    assert_eq!(
        emit.status,
        Status::Fail,
        "emit error vs success completion is a FAIL, not an error"
    );
    assert!(
        emit.artifacts.is_empty(),
        "a failed emission produced no artifacts"
    );
    let consumer = report
        .results
        .iter()
        .find(|r| r.check_id == "consumer")
        .unwrap();
    assert_eq!(
        consumer.status,
        Status::Blocked,
        "no usable product — consumer BLOCKED"
    );
    assert_eq!(report.exit_code(), 2, "BLOCKED fails the gate");
}

#[test]
fn product_consumer_unimplemented_modes_are_errors() {
    // F-03: unimplemented public downstream consumption is an explicit
    // ERROR — never a silent pass (the R04 "fake consumer" shape).
    let root = temp_root("a2r-consumer-modes");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    fs::write(root.join("expected-hello.rs"), &golden).unwrap();
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "t05f");
    fs::write(
        root.join("c.case.ad"),
        concat!(
            "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05f.modes\",\n",
            "  source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"script\" },\n  checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" },\n",
            "      expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected-hello.rs\" } ] } },\n",
            "    { id: \"bare\", adapter: \"avm\", action: \"run\", product_from: \"emit\", expect: { completion: \"success\" } },\n",
            "    { id: \"errconsumer\", adapter: \"a2r\", action: \"emit\", product_from: \"emit\", expect: { completion: { stage: \"parse\", code: \"auto_syntax_E0001\" } } } ] }\n```\n",
        ),
    )
    .unwrap();

    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    let get = |id: &str| {
        report
            .results
            .iter()
            .find(|r| r.check_id == id)
            .unwrap()
            .clone()
    };
    // A bare-success consumer runs the registered test consumer over the
    // REAL actual bytes — its PASS is backed by consumption evidence.
    let bare = get("bare");
    assert_eq!(
        bare.status,
        Status::Pass,
        "bare consumer verifies the consumed manifest: {:?}",
        bare.detail
    );
    assert!(bare.detail.unwrap().contains("verified against manifest"));
    // Error-completion consumers are unimplemented → ERROR.
    let errconsumer = get("errconsumer");
    assert_eq!(errconsumer.status, Status::Error);
    assert!(errconsumer.detail.unwrap().contains("unimplemented"));
}

#[test]
fn product_consumer_rejects_result_observation() {
    // F-03: result/stdout/stderr observations on a product consumer name a
    // consumption mode that does not exist in this pilot → ERROR.
    let root = temp_root("a2r-consumer-result");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    fs::write(root.join("expected-hello.rs"), &golden).unwrap();
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "t05g");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05g.r\",\n  source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"script\" },\n  checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" },\n      expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected-hello.rs\" } ] } },\n    { id: \"consumer\", adapter: \"avm\", action: \"run\", product_from: \"emit\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ] }\n```\n",
    )
    .unwrap();
    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    let consumer = report
        .results
        .iter()
        .find(|r| r.check_id == "consumer")
        .unwrap();
    assert_eq!(consumer.status, Status::Error);
    assert!(
        consumer
            .detail
            .as_deref()
            .unwrap()
            .contains("result/stdout/stderr observations on a product consumer"),
        "{:?}",
        consumer.detail
    );
}

#[test]
fn a2r_missing_manifest_is_contract_error_not_silent_output() {
    let root = temp_root("a2r-manifest");
    mini_suite(&root, "t05d");
    fs::write(
        root.join("c.case.ad"),
        concat!(
            "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05d.cabi\",\n",
            "  source: { text: \"use.c <no_such_manifest.json>\\n\\nfn main() { print(1) }\", syntax: \"auto\", path: \"main.at\" },\n",
            "  entry: { mode: \"script\" },\n",
            "  checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", expect: { completion: \"success\", artifacts: [ { path: \"main.rs\", file: \"expected.rs\" } ] } } ] }\n```\n",
        ),
    )
    .unwrap();
    fs::write(root.join("expected.rs"), b"// whatever\n").unwrap();

    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(report.results[0].status, Status::Error);
    assert!(
        report.results[0]
            .detail
            .as_deref()
            .unwrap()
            .contains("no_such_manifest"),
        "{}",
        report.to_console()
    );
}

#[test]
fn proposals_bind_old_and_new_identity_with_real_diff() {
    use crate::test_cases::materialize::{collect_proposals, write_proposals};

    let root = temp_root("a2r-proposal");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    let mut bad = golden.clone();
    bad.extend_from_slice(b"// drift\n");
    fs::write(root.join("expected-hello.rs"), &bad).unwrap();
    // A SECOND check shares the same oracle sidecar — it must be listed as
    // a shared consumer of the proposal (F-11).
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "t05e");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05e.p\", source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" }, expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected-hello.rs\" } ] } }, { id: \"second\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" }, expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected-hello.rs\" } ] } } ] }\n```\n",
    )
    .unwrap();

    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(report.count(Status::Fail), 2);

    let proposals =
        collect_proposals(&report, &root.join("t05.suite.ad"), &Limits::defaults()).unwrap();
    assert_eq!(proposals.len(), 2);
    let p = &proposals[0];
    // Both sides' identity is retained (F-11): old = tampered bytes hash,
    // candidate = current actual hash — and they differ.
    let sha = |b: &[u8]| {
        use auto_test_case::sha256::{hex, sha256};
        hex(&sha256(b))
    };
    assert_eq!(p.old_sha256, sha(&bad));
    assert_eq!(p.candidate_sha256, sha(&golden));
    assert_ne!(p.old_sha256, p.candidate_sha256);
    assert_eq!(p.oracle_file, "expected-hello.rs");
    // Shared consumers: each proposal lists the OTHER check.
    assert!(p.shared_consumers.contains(&"t05e.p:second".to_string()));

    let proposal_dir = out_of(&root, "proposals");
    let n = write_proposals(&proposals, &proposal_dir, Some(&out_of(&root, "out").join("report.json")))
        .unwrap();
    assert_eq!(n, 2);
    let candidate = proposal_dir.join("t05e.p").join("emit").join("hello.rs");
    assert_eq!(
        fs::read(&candidate).unwrap(),
        golden,
        "candidate = current actual bytes"
    );
    // Real content diff written next to the candidate (F-11).
    let diff = fs::read_to_string(candidate.with_extension("rs.diff")).unwrap();
    assert!(diff.contains("@@"), "unified hunk header: {}", diff);
    assert!(diff.contains("-// drift"), "{}", diff);
    let manifest = fs::read_to_string(proposal_dir.join("PROPOSALS.md")).unwrap();
    assert!(manifest.contains(&format!("old sha256 `{}`", sha(&bad))));
    assert!(manifest.contains(&format!("candidate sha256 `{}`", sha(&golden))));
    assert!(manifest.contains("t05e.p:second"));
    assert!(manifest.contains("report.json"));
    // Oracle untouched; nothing written next to the case.
    assert_eq!(fs::read(root.join("expected-hello.rs")).unwrap(), bad);
}

#[test]
fn proposal_read_failures_propagate() {
    use crate::test_cases::materialize::collect_proposals;

    // F-11: an unreadable actual is an ERROR, not a silently skipped row.
    let root = temp_root("a2r-proposal-err");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    let mut bad = golden.clone();
    bad.extend_from_slice(b"// drift\n");
    fs::write(root.join("expected-hello.rs"), &bad).unwrap();
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "t05h");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05h.p\", source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" }, expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected-hello.rs\" } ] } } ] }\n```\n",
    )
    .unwrap();
    let mut report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(report.count(Status::Fail), 1);
    // Sabotage the persisted actual's recorded path.
    report.results[0].artifacts[0].stored_path =
        out_of(&root, "out").join("no-such-file.rs").display().to_string();
    let err = collect_proposals(&report, &root.join("t05.suite.ad"), &Limits::defaults())
        .unwrap_err();
    assert!(err.contains("reading actual"), "{}", err);
}

// ------------------------------------------------------- T-06 pilot ------

fn pilot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("test/cases/pilot")
}

/// Shared runner for the fixed-name aggregate entries (Plan 773 F-10):
/// `vm_file_tests::pilot_aggregate_vm` (reached by the tv profile's
/// `vm_file_tests::` filter) and `a2r_tests::pilot_aggregate_a2r` (tt
/// tier) each run the pilot suite through this and assert their half.
pub(crate) fn run_pilot_for_aggregate(tag: &str) -> auto_test_case::report::RunReport {
    let out = std::env::temp_dir().join(format!("atc-pilot-agg-{}-{}", tag, std::process::id()));
    crate::test_cases::run_suite(&pilot_dir().join("pilot.suite.ad"), &out, &Limits::defaults())
        .expect("suite loads")
}

/// Full pilot run: 21 cases / 26 checks, everything green. This is also the
/// aggregation entry the daily tier consumes (fixed name).
#[test]
fn pilot_suite_full_run() {
    let out = std::env::temp_dir().join(format!("atc-pilot-{}", std::process::id()));
    let report = crate::test_cases::run_suite(
        &pilot_dir().join("pilot.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("suite loads");
    assert!(
        report.load_errors.is_empty(),
        "load errors: {:?}",
        report.load_errors
    );
    assert_eq!(report.results.len(), 24, "checks: {}", report.to_console());
    assert_eq!(report.count(Status::Pass), 24, "{}", report.to_console());
    assert_eq!(report.exit_code(), 0, "{}", report.to_console());

    // ≥3 dual-check cases: same source, independent avm/a2r verdicts.
    let dual = [
        "lang.basic.hello",
        "lang.basic.str-upper",
        "lang.basic.eq-false",
    ];
    for case_id in dual {
        let checks: Vec<&str> = report
            .results
            .iter()
            .filter(|r| r.case_id == case_id)
            .map(|r| r.check_id.as_str())
            .collect();
        assert_eq!(checks, vec!["vm", "rust"], "{}", case_id);
    }
    // Identity hashes bound on every outcome (stale-check basis).
    for r in &report.results {
        assert_eq!(r.identity.suite.len(), 64);
        assert_eq!(r.identity.source.len(), 64);
    }
}

/// V-07 reconciliation: legacy-map.tsv rows and the live report explain
/// each other completely — no unexplained legacy import, no phantom case.
/// Plan 773 F-11: the map also carries the legacy source/oracle CONTENT
/// hashes and the new-side entry/project context; every hash column is
/// RE-COMPUTED from the real legacy files so the map cannot drift from the
/// tree it claims to reconcile.
#[test]
fn pilot_legacy_map_reconciles() {
    let out = std::env::temp_dir().join(format!("atc-pilot-map-{}", std::process::id()));
    let report = crate::test_cases::run_suite(
        &pilot_dir().join("pilot.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("suite loads");

    let map = fs::read_to_string(pilot_dir().join("legacy-map.tsv")).unwrap();
    let sha = |b: &[u8]| {
        use auto_test_case::sha256::{hex, sha256};
        hex(&sha256(b))
    };
    let mut mapped: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (li, line) in map.lines().enumerate() {
        let cols: Vec<&str> = line.split('\t').collect();
        if li == 0 {
            assert_eq!(
                cols.len(),
                16,
                "map header must have 16 columns (Plan 773 F-11 traceability)"
            );
            assert_eq!(cols[12], "legacy_source_sha256");
            assert_eq!(cols[13], "legacy_oracle_sha256");
            assert_eq!(cols[14], "new_entry_mode");
            assert_eq!(cols[15], "new_project_context");
            continue;
        }
        assert_eq!(cols.len(), 16, "map row must have 16 columns: {:?}", cols);
        let case_id = cols[8];
        assert!(
            !case_id.is_empty(),
            "map row without new_case: {:?}",
            cols[0]
        );
        // Hash columns re-computed from the real legacy files (F-11): the
        // map stays honest against the imported tree.
        let test_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("test");
        let legacy_dir = test_root.join(cols[0].replace('/', std::path::MAIN_SEPARATOR_STR));
        let stem = cols[1];
        let src_at = legacy_dir.join(format!("{}.at", stem));
        let src_as = legacy_dir.join(format!("{}.as", stem));
        let src_bytes = if src_at.is_file() {
            fs::read(src_at).unwrap()
        } else {
            fs::read(src_as).expect("legacy source .at or .as exists")
        };
        assert_eq!(cols[12], sha(&src_bytes), "source hash drift: {}", cols[0]);
        let oracle_ext = ["expected.out", "expected.result", "expected.error", "expected.rs"]
            .iter()
            .find(|ext| legacy_dir.join(format!("{}.{}", stem, ext)).is_file())
            .expect("legacy oracle exists");
        let oracle_bytes =
            fs::read(legacy_dir.join(format!("{}.{}", stem, oracle_ext))).unwrap();
        assert_eq!(cols[13], sha(&oracle_bytes), "oracle hash drift: {}", cols[0]);
        // Engineering context columns are present and non-empty.
        assert!(!cols[14].is_empty(), "entry mode missing: {}", cols[0]);
        assert!(!cols[15].is_empty(), "project context missing: {}", cols[0]);
        for check in cols[9].split(';') {
            let check = check.trim();
            if check.is_empty() {
                continue;
            }
            let (check_id, adapter) = check.split_once(':').expect("vm:avm form");
            mapped.insert(format!("{}:{}", case_id, check_id));
            // The declared adapter matches the actual outcome adapter.
            let outcome = report
                .results
                .iter()
                .find(|r| r.case_id == case_id && r.check_id == check_id)
                .unwrap_or_else(|| {
                    panic!("map references missing outcome {}:{}", case_id, check_id)
                });
            assert_eq!(outcome.adapter, adapter, "{}:{}", case_id, check_id);
        }
    }
    let ran: std::collections::BTreeSet<String> = report
        .results
        .iter()
        .map(|r| format!("{}:{}", r.case_id, r.check_id))
        .collect();
    assert_eq!(
        mapped, ran,
        "legacy-map and report must cover exactly the same case/check set"
    );
    // 21 distinct cases; multi-file/multi-artifact + weak/strengthened +
    // unwired-revived classes are all present.
    let cases: std::collections::BTreeSet<&String> =
        report.results.iter().map(|r| &r.case_id).collect();
    assert_eq!(cases.len(), 21);
    assert!(cases.contains(&String::from("trans.project.multi-file")));
    assert!(cases.contains(&String::from("lang.error.out-of-bounds")));
    assert!(cases.contains(&String::from("lang.script.json-is-null")));
}

/// Oracle regeneration tool (bootstrap): reruns the suite and refreshes the
/// NEW oracle candidates under `oracle/` from the persisted actual bytes via
/// the proposal flow. NOT part of normal verification — run on demand, then
/// review the diff before accepting.
#[test]
#[ignore = "oracle regeneration tool: run explicitly, review diff before accepting"]
fn pilot_oracle_bootstrap() {
    let out = std::env::temp_dir().join(format!("atc-pilot-boot-{}", std::process::id()));
    let report = crate::test_cases::run_suite(
        &pilot_dir().join("pilot.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("suite loads");
    let proposals = crate::test_cases::materialize::collect_proposals(
        &report,
        &pilot_dir().join("pilot.suite.ad"),
        &Limits::defaults(),
    )
    .expect("collect");
    assert!(
        !proposals.is_empty(),
        "nothing to propose — are the oracles already green?"
    );
    let n = crate::test_cases::materialize::write_proposals(
        &proposals,
        &std::env::temp_dir().join("pilot-oracle-proposals"),
        None,
    )
    .unwrap();
    panic!("{} proposal(s) written to {}/pilot-oracle-proposals — review, then copy into oracle/ deliberately", n, std::env::temp_dir().display());
}

// ---------------------------------------------- Plan 773 repair controls ---
// Each test below pins one F-773-R1-* finding as a permanent regression:
// the counterexample shape comes from the review's replay set
// (docs/plans/attachments/773/review-r1/), adapted to disposable fixtures.

use crate::test_cases::{run_suite_selected, Selection};

fn sel(case: Option<&str>, adapter: Option<&str>, check: Option<&str>) -> Selection {
    Selection {
        case: case.map(String::from),
        adapter: adapter.map(String::from),
        check: check.map(String::from),
    }
}

/// F-773-R1-01: a successful emission can NEVER satisfy an error
/// completion — the R01 replay (`fn main(){print(1)}` + an oracle that
/// literally expects "emit succeeded") must FAIL, not PASS.
#[test]
fn success_never_satisfies_error_completion() {
    let root = temp_root("r01");
    mini_suite(&root, "r01");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r01.emit\", source: { text: \"fn main() { print(1) }\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", expect: { completion: { stage: \"run\", message: { exact: \"emit succeeded\" } } } } ] }\n```\n",
    )
    .unwrap();
    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    let r = &report.results[0];
    assert_ne!(r.status, Status::Pass, "{}", report.to_console());
    assert_eq!(r.status, Status::Fail);
    assert!(
        r.mismatches
            .iter()
            .any(|m| m.observation == "completion" && m.kind == "stage"),
        "{:?}",
        r.mismatches
    );
    assert_eq!(report.exit_code(), 1);
}

/// F-01 three-sided control: a REAL language failure satisfies a matching
/// error completion; the wrong stage does not.
#[test]
fn error_completion_three_sides() {
    let root = temp_root("r01-sides");
    mini_suite(&root, "r01b");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r01.parse\", source: { text: \"fn (\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"ok\", adapter: \"a2r\", action: \"emit\", expect: { completion: { stage: \"parse\", message: { contains: \"unexpected token\" } } } }, { id: \"wrong\", adapter: \"a2r\", action: \"emit\", expect: { completion: { stage: \"run\", message: { contains: \"unexpected token\" } } } } ] }\n```\n",
    )
    .unwrap();
    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    let get = |id: &str| report.results.iter().find(|r| r.check_id == id).unwrap();
    assert_eq!(get("ok").status, Status::Pass, "real parse failure matches");
    assert_eq!(
        get("wrong").status,
        Status::Fail,
        "wrong stage never satisfies"
    );
}

/// F-773-R1-02: unsupported observation channels and empty strong oracles
/// are ADMISSION errors — never decoded-then-ignored (R03/R08 shapes).
#[test]
fn unsupported_channels_and_empty_oracles_are_admission_errors() {
    let root = temp_root("r03");
    mini_suite(&root, "r03");
    let bad_contracts = [
        // R03-a2r: a2r cannot observe result/stdout/stderr.
        "case { schema: \"autolang.case/1\", id: \"r03.a\", source: { text: \"fn main() { print(1) }\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"IMPOSSIBLE\" } } } ] }",
        "case { schema: \"autolang.case/1\", id: \"r03.b\", source: { text: \"fn main() { print(1) }\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", expect: { completion: \"success\", stdout: { text: \"IMPOSSIBLE\" } } } ] }",
        // R03-avm: avm cannot observe artifacts.
        "case { schema: \"autolang.case/1\", id: \"r03.c\", source: { text: \"1+2\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", artifacts: [ { path: \"missing.bin\", file: \"missing-oracle.bin\" } ] } } ] }",
        // R08: streams declared on an error completion are not observable.
        "case { schema: \"autolang.case/1\", id: \"r03.d\", source: { text: \"fn (\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"parse\", message: { contains: \"token\" } }, stdout: { text: \"IMPOSSIBLE\" } } } ] }",
        // Empty code / empty matchers have no discriminating power.
        "case { schema: \"autolang.case/1\", id: \"r03.e\", source: { text: \"fn (\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"parse\", code: \"\" } } } ] }",
        "case { schema: \"autolang.case/1\", id: \"r03.f\", source: { text: \"fn (\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"parse\", message: { contains: \"\" } } } } ] }",
        // F-05: module_name must be a single safe path component.
        "case { schema: \"autolang.case/1\", id: \"r03.g\", source: { text: \"fn main() { print(1) }\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"../evil\" }, expect: { completion: \"success\" } } ] }",
    ];
    for (i, body) in bad_contracts.iter().enumerate() {
        let root_i = root.join(format!("c{}", i));
        std::fs::create_dir_all(&root_i).unwrap();
        fs::write(root_i.join("t05.suite.ad"), format!(
            "# s\n```test-suite\nsuite {{ schema: \"autolang.suite/1\", id: \"r03-{}\", include: [\"c.case.ad\"] }}\n```\n", i
        ))
        .unwrap();
        fs::write(
            root_i.join("c.case.ad"),
            format!("# C\n```test-case\n{}\n```\n", body),
        )
        .unwrap();
        let err = run_suite(
            &root_i.join("t05.suite.ad"),
            &out_of(&root_i, "out"),
            &Limits::defaults(),
        )
        .expect_err("contract must be rejected at admission");
        assert!(
            err.contains("not an observable channel")
                || err.contains("needs discriminating text")
                || err.contains("empty `code`")
                || err.contains("require completion \"success\"")
                || err.contains("single path component"),
            "counterexample #{} must be a located admission error, got: {}",
            i,
            err
        );
    }
}

/// F-773-R1-04: selectors validate and apply BEFORE execution — unselected
/// checks never run (no scratch, no side-effect files), typo/zero-match
/// selectors are contract errors, and a partial report never passes the
/// gate.
#[test]
fn selection_prevents_execution_and_validates_selectors() {
    let root = temp_root("r05");
    fs::write(
        root.join("t05.suite.ad"),
        "# s\n```test-suite\nsuite { schema: \"autolang.suite/1\", id: \"r05\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    fs::write(
        root.join("c.case.ad"),
        "# e ^e\n```auto\n1+2\n```\n## C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r05.case\", source: { block: \"e\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } }, { id: \"other\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ] }\n```\n",
    )
    .unwrap();

    // Typo'd adapter: contract error BEFORE any execution; fixtures intact.
    let before = fs::read(&root.join("c.case.ad")).unwrap();
    let err = run_suite_selected(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out1"),
        &Limits::defaults(),
        &sel(None, Some("typo"), None),
    )
    .unwrap_err();
    assert!(err.contains("bad selector"), "{}", err);
    assert!(!out_of(&root, "out1").exists(), "no scratch on bad selector");
    assert_eq!(fs::read(&root.join("c.case.ad")).unwrap(), before);

    // Zero-match selector: same treatment.
    let err = run_suite_selected(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out2"),
        &Limits::defaults(),
        &sel(Some("no-such-case"), None, None),
    )
    .unwrap_err();
    assert!(err.contains("matches no declared check"), "{}", err);

    // Partial selection: only `vm` executes; `other` is NOT_RUN with NO
    // side effects (the R05 repro found other/result.txt on disk).
    let report = run_suite_selected(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out3"),
        &Limits::defaults(),
        &sel(None, None, Some("vm")),
    )
    .unwrap();
    let vm = report.results.iter().find(|r| r.check_id == "vm").unwrap();
    let other = report
        .results
        .iter()
        .find(|r| r.check_id == "other")
        .unwrap();
    assert_eq!(vm.status, Status::Pass);
    assert_eq!(other.status, Status::NotRun);
    assert!(other.detail.as_deref().unwrap().contains("not selected"));
    // No artifact dir was ever created for the unselected check.
    let out3 = out_of(&root, "out3");
    let other_paths: Vec<_> = std::fs::read_dir(&out3)
        .unwrap()
        .flat_map(|run| {
            let run = run.unwrap().path();
            std::fs::read_dir(&run).unwrap().map(move |c| c.unwrap().path())
        })
        .filter(|p| p.to_string_lossy().contains("other"))
        .collect();
    assert!(
        other_paths.is_empty(),
        "unselected check must leave no scratch dirs: {:?}",
        other_paths
    );
    // A partial report never counts as a gate pass.
    assert_eq!(report.exit_code(), 1);
}

/// F-773-R1-05: output write ports are path-domain audited BEFORE
/// execution — the out directory may not overlap the suite/fixture roots
/// in either direction, and nothing is written when the audit rejects.
#[test]
fn output_domain_overlap_rejected_before_execution() {
    let root = temp_root("r06");
    fs::write(
        root.join("t05.suite.ad"),
        "# s\n```test-suite\nsuite { schema: \"autolang.suite/1\", id: \"r06\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    fs::write(
        root.join("c.case.ad"),
        "# e ^e\n```auto\n1+2\n```\n## C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r06.case\", source: { block: \"e\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ] }\n```\n",
    )
    .unwrap();
    // Sentinel: the R06 repro had --out pointed at the fixture root itself,
    // overwriting report.json.
    fs::write(root.join("report.json"), "ORACLE SENTINEL\n").unwrap();
    let snapshot: Vec<(PathBuf, Vec<u8>)> = ["t05.suite.ad", "c.case.ad", "report.json"]
        .iter()
        .map(|n| (root.join(n), fs::read(root.join(n)).unwrap()))
        .collect();

    // out == fixture/suite root → rejected, nothing executed or written.
    let err = run_suite(&root.join("t05.suite.ad"), &root, &Limits::defaults())
        .unwrap_err();
    assert!(err.contains("output path rejected"), "{}", err);
    // out == PARENT of the root (contains inputs) → rejected too.
    let err = run_suite(
        &root.join("t05.suite.ad"),
        &root.parent().unwrap(),
        &Limits::defaults(),
    )
    .unwrap_err();
    assert!(err.contains("output path rejected"), "{}", err);
    for (path, bytes) in &snapshot {
        assert_eq!(&fs::read(path).unwrap(), bytes, "{} untouched", path.display());
    }
    assert_eq!(
        fs::read_to_string(root.join("report.json")).unwrap(),
        "ORACLE SENTINEL\n"
    );
    // The rejected parent directory is not the caller's — verify no run
    // dirs appeared inside the fixture root.
    let runs: Vec<_> = std::fs::read_dir(&root)
        .unwrap()
        .filter_map(|e| {
            let p = e.unwrap().path();
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("run-")
                .then_some(p)
        })
        .collect();
    assert!(runs.is_empty(), "no run scratch after rejections: {:?}", runs);
}

/// F-773-R1-06: oracle identity tracks sidecar CONTENT — mutating expected
/// bytes changes the identity axis (the R02 repro kept it equal); the
/// report carries toolchain/config identity; zero dynamic checks exit 2.
#[test]
fn oracle_identity_tracks_content_and_toolchain_binds() {
    let root = temp_root("r02");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    let mut wrong = golden.clone();
    wrong.extend_from_slice(b"// deliberately wrong\n");
    fs::write(root.join("expected.rs"), &wrong).unwrap();
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "r02");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r02.case\", source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" }, expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected.rs\" } ] } } ] }\n```\n",
    )
    .unwrap();

    let before = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out1"),
        &Limits::defaults(),
    )
    .unwrap();
    // Fix the sidecar to the actual bytes: FAIL → would-be PASS.
    let actual = fs::read(
        before.results[0].artifacts[0]
            .stored_path
            .clone()
            .parse::<PathBuf>()
            .unwrap(),
    )
    .unwrap();
    fs::write(root.join("expected.rs"), &actual).unwrap();
    let after = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out2"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(before.results[0].status, Status::Fail);
    assert_eq!(after.results[0].status, Status::Pass);
    // The oracle identity axis CHANGED with the sidecar bytes; the rest of
    // the identity stayed stable.
    assert_ne!(
        before.results[0].identity.oracle, after.results[0].identity.oracle,
        "sidecar content must enter the oracle identity (F-773-R1-06)"
    );
    assert_eq!(
        before.results[0].identity.source, after.results[0].identity.source,
        "source identity is content of inputs, not of oracles"
    );
    assert_eq!(before.results[0].identity.suite, after.results[0].identity.suite);

    // Toolchain identity: present, and sensitive to the effective limits.
    for (k, _) in [
        ("auto-test-case", 0),
        ("vendored-parser", 0),
        ("auto-lang", 0),
        ("limits", 0),
    ] {
        assert!(
            after.toolchain.contains_key(k),
            "toolchain axis {} missing: {:?}",
            k,
            after.toolchain
        );
    }
    let tuned = Limits {
        max_expanded_checks_per_run: 8,
        ..Limits::defaults()
    };
    let tuned_report =
        run_suite(&root.join("t05.suite.ad"), &out_of(&root, "out3"), &tuned).unwrap();
    assert_ne!(
        after.toolchain.get("limits"),
        tuned_report.toolchain.get("limits"),
        "effective limits are part of the report identity"
    );

    // The report JSON is valid, and a report with ZERO dynamic checks is a
    // contract error, never a green run (F-06 empty-reducer).
    let json: serde_json::Value =
        serde_json::from_str(&after.to_json()).expect("report JSON parses");
    assert!(json["toolchain"].is_object());
    let mut empty = after.clone();
    empty.results.clear();
    assert_eq!(empty.exit_code(), 2, "zero checks never exit 0");
    let mut empty_counts = after.clone();
    empty_counts.results.clear();
    let j = empty_counts.to_json();
    assert!(
        serde_json::from_str::<serde_json::Value>(&j).is_ok(),
        "summary with zero counts must still be valid JSON: {}",
        j
    );
}

/// F-06 stale entry: an old report's PASS goes stale when the oracle
/// content drifts — through the real `stale::check` entry the CLI uses.
#[test]
fn stale_check_detects_oracle_drift() {
    let root = temp_root("r02-stale");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    fs::write(root.join("expected.rs"), &golden).unwrap();
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "r02s");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r02s.case\", source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" }, expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected.rs\" } ] } } ] }\n```\n",
    )
    .unwrap();
    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(report.results[0].status, Status::Pass);
    let old: serde_json::Value =
        serde_json::from_str(&report.to_json()).expect("report JSON parses");

    // Unmodified tree: everything FRESH.
    let verdicts = crate::test_cases::stale::check(&root.join("t05.suite.ad"), &old).unwrap();
    assert!(verdicts.iter().all(|v| matches!(
        v,
        crate::test_cases::stale::RowVerdict::Fresh { .. }
    )));

    // Mutate the oracle sidecar: the old PASS is STALE on the oracle axis.
    let mut drifted = golden.clone();
    drifted.extend_from_slice(b"// later edit\n");
    fs::write(root.join("expected.rs"), &drifted).unwrap();
    let verdicts = crate::test_cases::stale::check(&root.join("t05.suite.ad"), &old).unwrap();
    match &verdicts[0] {
        crate::test_cases::stale::RowVerdict::Stale { drifted_axes, .. } => {
            assert!(
                drifted_axes.iter().any(|a| a.starts_with("oracle ")),
                "{:?}",
                drifted_axes
            );
        }
        other => panic!("expected stale, got {:?}", other),
    }
}

/// F-773-R1-07: the whole-run expansion limit is enforced at LOAD (lint
/// sees it too), and oversized documents are rejected under bounded reads.
#[test]
fn expansion_and_byte_limits_are_enforced() {
    let root = temp_root("r09");
    fs::write(
        root.join("t05.suite.ad"),
        "# s\n```test-suite\nsuite { schema: \"autolang.suite/1\", id: \"r09\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    fs::write(
        root.join("c.case.ad"),
        "# e ^e\n```auto\n1+2\n```\n## C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r09.case\", source: { block: \"e\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"expression\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\" } }, { id: \"vm2\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\" } } ] }\n```\n",
    )
    .unwrap();
    // Runner-tuned limit: 2 expanded checks > 1 → resource error at load,
    // on the run path AND the lint path (shared load_suite).
    let tuned = Limits {
        max_expanded_checks_per_run: 1,
        ..Limits::defaults()
    };
    let err = run_suite(&root.join("t05.suite.ad"), &out_of(&root, "out"), &tuned).unwrap_err();
    assert!(
        err.contains("max_expanded_checks_per_run"),
        "expansion limit enforced: {}",
        err
    );
    let lint_errors =
        crate::test_cases::materialize::lint_resolve(&root.join("t05.suite.ad"), &tuned)
            .unwrap_err();
    assert!(lint_errors.contains("max_expanded_checks_per_run"), "{}", lint_errors);

    // Oversized document: bounded read rejects without full buffering.
    let tiny = Limits {
        max_document_bytes: 16,
        ..Limits::defaults()
    };
    let err = run_suite(&root.join("t05.suite.ad"), &out_of(&root, "out2"), &tiny).unwrap_err();
    assert!(
        err.contains("max_document_bytes"),
        "document size limit enforced: {}",
        err
    );
}

/// F-773-R1-09: lint runs the FULL admission — a missing expected sidecar
/// is a located lint error, not a run-time surprise (the R07 replay had
/// `lint` exit 0 for an oracle that does not exist).
#[test]
fn lint_resolves_references_or_fails_located() {
    let root = temp_root("r07");
    fs::write(root.join("expected.rs"), b"// present\n").unwrap();
    mini_suite(&root, "r07");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r07.a\", source: { text: \"fn main() { print(1) }\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", expect: { completion: \"success\", artifacts: [ { path: \"main.rs\", file: \"expected.rs\" } ] } } ] }\n```\n# D\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r07.missing\", source: { text: \"fn main() { print(2) }\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", expect: { completion: \"success\", artifacts: [ { path: \"main.rs\", file: \"no-such-expected.rs\" } ] } } ] }\n```\n",
    )
    .unwrap();
    let errors =
        crate::test_cases::materialize::lint_resolve(&root.join("t05.suite.ad"), &Limits::defaults())
            .unwrap();
    assert_eq!(errors.len(), 1, "{:?}", errors);
    assert!(
        errors[0].contains("no-such-expected.rs") && errors[0].contains("r07.missing"),
        "located error naming the missing ref and case: {}",
        errors[0]
    );
    // A bad anchor is caught by the same full admission.
    let root2 = temp_root("r07-anchor");
    mini_suite(&root2, "r07b");
    fs::write(
        root2.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r07b.a\", source: { block: \"no-such-anchor\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\" } } ] }\n```\n",
    )
    .unwrap();
    let errors =
        crate::test_cases::materialize::lint_resolve(&root2.join("t05.suite.ad"), &Limits::defaults())
            .unwrap();
    assert!(
        errors.iter().any(|e| e.contains("no-such-anchor")),
        "{:?}",
        errors
    );
}

/// F-773-R1-08 (runner side): real I/O failures surface as infrastructure
/// ERRORS on the run path — an oracle crafted to match the failure TEXT
/// still cannot turn them into a language negative.
#[test]
fn runner_io_failures_are_infrastructure_errors() {
    let root = temp_root("r08");
    mini_suite(&root, "r08");
    // `file:` source pointing at a DIRECTORY: a real read failure during
    // payload resolution — per-check ERROR outcomes, never PASS.
    std::fs::create_dir_all(root.join("not-a-file.at")).unwrap();
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"r08.case\", source: { file: \"not-a-file.at\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: { stage: \"run\", message: { contains: \"os error\" } } } } ] }\n```\n",
    )
    .unwrap();
    let report = run_suite(
        &root.join("t05.suite.ad"),
        &out_of(&root, "out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(
        report.results[0].status,
        Status::Error,
        "I/O failure is an infrastructure ERROR even when the oracle text would match: {}",
        report.to_console()
    );
    assert_eq!(report.exit_code(), 2);
}
