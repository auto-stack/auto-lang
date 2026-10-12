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

    let out = root.join("out");
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
        &root.join("out"),
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
        &root.join("out"),
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
        &root.join("out"),
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
        &root.join("out"),
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
        &root.join("out"),
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
        &root.join("out"),
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
fn a2r_product_chain_blocked_on_failed_upstream() {
    let root = temp_root("a2r-blocked");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    let mut bad = golden.clone();
    bad.extend_from_slice(b"// tampered\n");
    fs::write(root.join("expected-hello.rs"), &bad).unwrap();
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "t05c");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05c.chain\",\n  source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"script\" },\n  checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" },\n      expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected-hello.rs\" } ] } },\n    { id: \"consumer\", adapter: \"a2r\", action: \"emit\", product_from: \"emit\", options: { module_name: \"hello2\" },\n      expect: { completion: \"success\", artifacts: [ { path: \"hello2.rs\", file: \"expected-hello.rs\" } ] } } ] }\n```\n",
    )
    .unwrap();

    let report = run_suite(
        &root.join("t05.suite.ad"),
        &root.join("out"),
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
    assert_eq!(
        consumer.status,
        Status::Blocked,
        "downstream of failed product is BLOCKED"
    );
    assert!(consumer
        .detail
        .as_deref()
        .unwrap()
        .contains("no usable product"));
    assert_eq!(
        report.exit_code(),
        2,
        "BLOCKED fails the gate as contract error"
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
        &root.join("out"),
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
fn proposals_are_written_to_proposal_dir_only() {
    use crate::test_cases::materialize::{collect_proposals, write_proposals};

    let root = temp_root("a2r-proposal");
    let fx = fixture("a2r/01_basics/001_hello");
    let src = fs::read_to_string(fx.join("hello.at")).unwrap();
    let golden = fs::read(fx.join("hello.expected.rs")).unwrap();
    let mut bad = golden.clone();
    bad.extend_from_slice(b"// drift\n");
    fs::write(root.join("expected-hello.rs"), &bad).unwrap();
    fs::write(root.join("hello.at"), &src).unwrap();
    mini_suite(&root, "t05e");
    fs::write(
        root.join("c.case.ad"),
        "# C\n```test-case\ncase { schema: \"autolang.case/1\", id: \"t05e.p\", source: { file: \"hello.at\", syntax: \"auto\", path: \"main.at\" }, entry: { mode: \"script\" }, checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", options: { module_name: \"hello\" }, expect: { completion: \"success\", artifacts: [ { path: \"hello.rs\", file: \"expected-hello.rs\" } ] } } ] }\n```\n",
    )
    .unwrap();

    let report = run_suite(
        &root.join("t05.suite.ad"),
        &root.join("out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert_eq!(report.results[0].status, Status::Fail);

    let proposals = collect_proposals(&report);
    assert_eq!(proposals.len(), 1);
    assert_eq!(
        proposals[0].candidate_sha256,
        report.results[0].artifacts[0].sha256
    );

    let proposal_dir = root.join("proposals");
    let n = write_proposals(&proposals, &proposal_dir).unwrap();
    assert_eq!(n, 1);
    let candidate = proposal_dir.join("t05e.p").join("emit").join("hello.rs");
    assert_eq!(
        fs::read(&candidate).unwrap(),
        golden,
        "candidate = current actual bytes"
    );
    assert!(proposal_dir.join("PROPOSALS.md").is_file());
    // Oracle untouched; nothing written next to the case.
    assert_eq!(fs::read(root.join("expected-hello.rs")).unwrap(), bad);
}

// ------------------------------------------------------- T-06 pilot ------

fn pilot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("test/cases/pilot")
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
    let mut mapped: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for line in map.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 12, "map row must have 12 columns: {:?}", cols);
        let case_id = cols[8];
        assert!(
            !case_id.is_empty(),
            "map row without new_case: {:?}",
            cols[0]
        );
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
    let proposals = crate::test_cases::materialize::collect_proposals(&report);
    assert!(
        !proposals.is_empty(),
        "nothing to propose — are the oracles already green?"
    );
    let n = crate::test_cases::materialize::write_proposals(
        &proposals,
        &std::env::temp_dir().join("pilot-oracle-proposals"),
    )
    .unwrap();
    panic!("{} proposal(s) written to {}/pilot-oracle-proposals — review, then copy into oracle/ deliberately", n, std::env::temp_dir().display());
}
