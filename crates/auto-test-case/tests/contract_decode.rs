//! Contract decode attack surface (V-01 core): strict schemas, unknown
//! fields, illegal combinations, weak-oracle rejection, DAG wiring.

use auto_test_case::admission::Limits;
use auto_test_case::atom::parse_contract;
use auto_test_case::contract::{decode_case, CASE_SCHEMA};

fn decode(
    src: &str,
) -> Result<auto_test_case::contract::CaseSpec, auto_test_case::contract::ContractError> {
    let v = parse_contract(src).unwrap();
    decode_case(&v, &Limits::defaults())
}

fn minimal_case() -> String {
    concat!(
        "case { schema: \"autolang.case/1\", id: \"ok.min\", ",
        "source: { text: \"1 + 2\", syntax: \"auto\", path: \"main.at\" }, ",
        "entry: { mode: \"expression\" }, ",
        "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", ",
        "expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ] }"
    )
    .to_string()
}

#[test]
fn minimal_case_decodes() {
    let spec = decode(&minimal_case()).unwrap();
    assert_eq!(spec.id, "ok.min");
    assert_eq!(spec.checks.len(), 1);
    assert_eq!(spec.checks[0].id, "vm");
    assert!(spec.status_skip.is_none());
}

#[test]
fn wrong_schema_is_rejected() {
    let err = decode(&minimal_case().replace(CASE_SCHEMA, "autolang.case/2")).unwrap_err();
    assert!(err.message.contains("unsupported schema"), "{}", err);
}

#[test]
fn unknown_fields_are_located_errors() {
    for (mangled, needle) in [
        (
            minimal_case().replace(
                "entry: { mode: \"expression\" }",
                "entry: { mode: \"expression\", extra: 1 }",
            ),
            "case.entry.extra",
        ),
        (
            minimal_case().replace("id: \"ok.min\"", "id: \"ok.min\", tier: \"x\""),
            "case.tier",
        ),
        (
            minimal_case().replacen("text: \"1 + 2\"", "text: \"1 + 2\", anchor: \"nope\"", 1),
            "unknown field",
        ),
    ] {
        let err = decode(&mangled).unwrap_err();
        assert!(
            err.path.contains(needle) || err.message.contains(needle),
            "{:?} -> {}",
            mangled,
            err
        );
    }
}

#[test]
fn typo_field_is_never_silently_ignored() {
    // "stdouts" instead of "stdout" must fail, not pass as unchecked stream.
    let src = minimal_case().replace(
        "expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } }",
        "expect: { completion: \"success\", stdouts: { text: \"\" } }",
    );
    let err = decode(&src).unwrap_err();
    assert!(err.path.contains("expect.stdouts"), "{}", err);
}

#[test]
fn illegal_adapter_action_combos_error() {
    for combo in [
        ("ac", "build-run"),
        ("avm", "emit"),
        ("a2r", "run"),
        ("acc", "emit"),
    ] {
        let src = minimal_case().replace(
            "adapter: \"avm\", action: \"run\"",
            &format!("adapter: \"{}\", action: \"{}\"", combo.0, combo.1),
        );
        let err = decode(&src).unwrap_err();
        assert!(!err.message.is_empty());
    }
}

#[test]
fn syntax_and_path_extension_must_agree() {
    let err =
        decode(&minimal_case().replace("path: \"main.at\"", "path: \"main.as\"")).unwrap_err();
    assert!(err.message.contains("does not match syntax"), "{}", err);
    let err =
        decode(&minimal_case().replace("path: \"main.at\"", "path: \"../up.at\"")).unwrap_err();
    assert!(err.message.contains("path"), "{}", err);
}

#[test]
fn weak_error_oracles_are_rejected() {
    // Bare stage without code/message = weak oracle = contract error.
    let src = minimal_case().replace(
        "expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } }",
        "expect: { completion: { stage: \"run\" } }",
    );
    let err = decode(&src).unwrap_err();
    assert!(err.message.contains("stable identification"), "{}", err);

    // Strong: stage + code is accepted.
    let src = minimal_case().replace(
        "expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } }",
        "expect: { completion: { stage: \"run\", code: \"DivisionByZero\" } }",
    );
    assert!(decode(&src).is_ok());

    // Strong: stage + contains.
    let src = minimal_case().replace(
        "expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } }",
        "expect: { completion: { stage: \"parse\", message: { contains: \"unexpected token\" } } }",
    );
    assert!(decode(&src).is_ok());
}

#[test]
fn error_completion_cannot_also_expect_result_or_artifacts() {
    let base = minimal_case().replace(
        "expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } }",
        "expect: { completion: { stage: \"run\", code: \"X\" }, RESULT }",
    );
    let err = decode(&base.replace(
        "RESULT",
        "result: { format: \"legacy-display\", text: \"1\" }",
    ))
    .unwrap_err();
    assert!(err.message.contains("completion \"success\""), "{}", err);
    let err = decode(&base.replace(
        "RESULT",
        "artifacts: [ { path: \"main.rs\", file: \"exp.rs\" } ]",
    ))
    .unwrap_err();
    assert!(err.message.contains("completion \"success\""), "{}", err);
}

#[test]
fn product_from_must_reference_earlier_checks() {
    let two_checks = minimal_case().replace(
        "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ]",
        "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } }, \
                  { id: \"emit\", adapter: \"a2r\", action: \"emit\", product_from: \"missing\", expect: { completion: \"success\", artifacts: [ { path: \"main.rs\", file: \"exp.rs\" } ] } } ]",
    );
    let err = decode(&two_checks).unwrap_err();
    assert!(err.message.contains("unknown check"), "{}", err);

    let cyclic = minimal_case().replace(
        "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ]",
        "checks: [ { id: \"emit\", adapter: \"a2r\", action: \"emit\", product_from: \"emit\", expect: { completion: \"success\", artifacts: [ { path: \"main.rs\", file: \"exp.rs\" } ] } } ]",
    );
    let err = decode(&cyclic).unwrap_err();
    assert!(err.message.contains("earlier check"), "{}", err);
}

#[test]
fn skip_is_explicit_xfail_is_unsupported() {
    let skipped = minimal_case().replace(
        "entry: { mode: \"expression\" }",
        "entry: { mode: \"expression\" }, status: { skip: { reason: \"infra\", issue: \"PLAN-999\" } }",
    );
    assert!(decode(&skipped).unwrap().status_skip.is_some());

    let xfail = minimal_case().replace(
        "entry: { mode: \"expression\" }",
        "entry: { mode: \"expression\" }, status: { xfail: { reason: \"known\" } }",
    );
    let err = decode(&xfail).unwrap_err();
    assert!(err.path.contains("status"), "{}", err);
}

#[test]
fn duplicate_check_ids_and_bad_ids_error() {
    let dup = minimal_case().replace(
        "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } } ]",
        "checks: [ { id: \"vm\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\", result: { format: \"legacy-display\", text: \"3\" } } }, \
                  { id: \"vm\", adapter: \"a2r\", action: \"emit\", expect: { completion: \"success\", artifacts: [ { path: \"m.rs\", file: \"e.rs\" } ] } } ]",
    );
    assert!(decode(&dup)
        .unwrap_err()
        .message
        .contains("duplicate check id"));

    let err = decode(&minimal_case().replace("id: \"ok.min\"", "id: \"Bad ID\"")).unwrap_err();
    assert!(err.path == "case.id", "{}", err);
}

#[test]
fn avm_options_are_closed() {
    let src = minimal_case().replace(
        "id: \"vm\", adapter: \"avm\", action: \"run\",",
        "id: \"vm\", adapter: \"avm\", action: \"run\", options: { anything: 1 },",
    );
    let err = decode(&src).unwrap_err();
    assert!(err.path.contains("options"), "{}", err);
}

#[test]
fn module_entry_requires_files() {
    let err =
        decode(&minimal_case().replace("mode: \"expression\"", "mode: \"module\"")).unwrap_err();
    assert!(err.message.contains("module entry requires"), "{}", err);
}

#[test]
fn check_count_limit_applies() {
    let one =
        " { id: \"c$i\", adapter: \"avm\", action: \"run\", expect: { completion: \"success\" } }";
    let checks: Vec<String> = (0..17).map(|i| one.replace("$i", &i.to_string())).collect();
    let src = format!(
        "case {{ schema: \"{}\", id: \"ok.many\", source: {{ text: \"1\", syntax: \"auto\", path: \"m.at\" }}, entry: {{ mode: \"expression\" }}, checks: [{}] }}",
        CASE_SCHEMA,
        checks.join(", ")
    );
    let err = decode(&src).unwrap_err();
    assert!(err.message.contains("max_checks_per_case"), "{}", err);
}
