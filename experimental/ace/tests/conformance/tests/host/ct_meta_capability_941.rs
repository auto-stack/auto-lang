//! The real host recipes must carry the current capability to both consumers.
use auto_test_case::{admission::Limits, report::Status};
use std::path::PathBuf;

const FIELD: &str = include_str!(
    "../../../../../../crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta2.at"
);
const HAS: &str = "type Pair { x int\ny int }\nfn pick(b bool) int { if b { return 40 }; return 42 }\nfn main() int { return #{ pick(ct_has_field(Pair, \"z\")) } }";
const USER_FIELD: &str =
    "fn ct_field_count(x int) int { return x+7 }\nfn main() int { return #{ ct_field_count(35) } }";
const USER_HAS: &str =
    "fn ct_has_field(x int) int { return x+7 }\nfn main() int { return #{ ct_has_field(35) } }";
const TEMPLATE: &str =
    "fn ct_field_count<T>(x T) T { return x }\nfn main() int { return #{ ct_field_count(42) } }";

fn scratch() -> tempfile::TempDir {
    let parent = std::env::var_os("AUTO_PLAN_941_CONSUMER_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&parent).unwrap();
    tempfile::Builder::new()
        .prefix("host-ct-meta-941-")
        .tempdir_in(parent)
        .unwrap()
}

fn case(
    id: &str,
    source: &str,
    meta: bool,
    instances: bool,
    expected: Option<i32>,
    wrong_code: bool,
) -> String {
    let mut caps = vec!["ac.source.cte.scalar/v1"];
    if meta {
        caps.push("cte.meta.fields/v1");
    }
    if instances {
        caps.push("ac.source.instances.scalar/v1");
    }
    let caps = serde_json::to_string(&caps).unwrap();
    let source = serde_json::to_string(source).unwrap();
    let checks = [("av", "ac-av/core-run-v1", "av", "parse"), ("native", "ac/mir-native-scalar-v1", "mn", "resolve")]
        .map(|(id, recipe, option, stage)| {
            let expect = if let Some(n) = expected { format!("{{ completion: \"success\", exit: {n} }}") } else {
                let code = if wrong_code { "source.cte.wrong-control" } else { "source.cte.bind" };
                format!("{{ completion: {{ stage: \"{stage}\", code: \"{code}\" }} }}")
            };
            let diagnostic_mode = if option == "mn" { ", diagnostics: \"source-v1\"" } else { "" };
            format!("{{ id: \"{id}\", adapter: \"probe\", action: \"run\", options: {{ recipe: \"{recipe}\", {option}: {{ entry: \"d_main\", caps: {caps}{diagnostic_mode} }}, steps: [{{ op: \"run\" }}] }}, expect: {expect} }}")
        }).join(",\n");
    format!("```test-case\ncase {{ schema: \"autolang.case/2\", id: \"ace.941.meta.{id}\", source: {{ text: {source}, syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"program\" }}, checks: [{checks}] }}\n```\n")
}

fn run(cases: &str, expected_checks: usize, expected_status: Status) {
    let root = scratch();
    let inputs = root.path().join("inputs");
    std::fs::create_dir(&inputs).unwrap();
    std::fs::write(inputs.join("inputs.case.ad"), cases).unwrap();
    std::fs::write(inputs.join("gate.suite.ad"), "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"ace-941-meta\", include: [\"inputs.case.ad\"] }\n```\n").unwrap();
    let report = crate::test_cases::run_suite(
        &inputs.join("gate.suite.ad"),
        &root.path().join("out"),
        &Limits::defaults(),
    )
    .unwrap();
    assert!(report.load_errors.is_empty(), "{:?}", report.load_errors);
    assert_eq!(
        report.results.len(),
        expected_checks,
        "no missing consumer checks"
    );
    for result in &report.results {
        assert_eq!(
            result.status, expected_status,
            "{} {:?} {:?}",
            result.check_id, result.detail, result.mismatches
        );
    }
}

#[test]
fn ct_meta_capability_941_current_caps_in_av_and_native() {
    let _guard = super::av_protocol::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let mut cases = String::new();
    for (id, source, meta, instances, expected) in [
        ("field-closed", FIELD.to_string(), false, false, None),
        ("has-closed", HAS.to_string(), false, false, None),
        (
            "nested-closed",
            FIELD.replace("ct_field_count(Pair)", "#{ ct_field_count(Pair) }"),
            false,
            false,
            None,
        ),
        ("protected-field", USER_FIELD.to_string(), true, false, None),
        ("protected-has", USER_HAS.to_string(), true, false, None),
        ("protected-template", TEMPLATE.to_string(), true, true, None),
        ("field-open", FIELD.to_string(), true, false, Some(42)),
        ("has-absent", HAS.to_string(), true, false, Some(42)),
        (
            "has-present",
            HAS.replace("y int", "y int\nz int"),
            true,
            false,
            Some(40),
        ),
        (
            "ordinary-field",
            USER_FIELD.to_string(),
            false,
            false,
            Some(42),
        ),
        ("ordinary-has", USER_HAS.to_string(), false, false, Some(42)),
    ] {
        cases.push_str(&case(id, &source, meta, instances, expected, false));
    }
    run(&cases, 22, Status::Pass);
}

#[test]
fn ct_meta_capability_941_wrong_diagnostic_oracle_is_red() {
    let _guard = super::av_protocol::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    run(
        &case("wrong-code-control", FIELD, false, false, None, true),
        2,
        Status::Fail,
    );
}
