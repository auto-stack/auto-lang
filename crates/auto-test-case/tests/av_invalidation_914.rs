//! PLAN-914 (I04/I09): the AV data-incremental recipe admission — the
//! `partial-rebuild` step, the `av.invalidation` oracle and their two-way
//! binding to the recipe.

use auto_test_case::admission::Limits;
use auto_test_case::atom::parse_contract;
use auto_test_case::contract::decode_case;

fn case(recipe: &str, av: &str, steps: &str) -> String {
    format!(
        "case {{ schema: \"autolang.case/2\", id: \"f914.avdec\", \
         source: {{ text: \"type P {{ x int }}\", syntax: \"auto\", path: \"main.at\" }}, \
         entry: {{ mode: \"program\" }}, \
         checks: [ {{ id: \"av\", adapter: \"probe\", action: \"run\", \
         options: {{ recipe: \"{}\", av: {{ entry: \"d_main\", {} }}, steps: [ {} ] }}, \
         expect: {{ completion: \"success\", exit: 0 }} }} ] }}",
        recipe, av, steps
    )
}

fn decode(src: &str) -> Result<auto_test_case::contract::CaseSpec, auto_test_case::contract::ContractError> {
    let v = parse_contract(src).unwrap();
    decode_case(&v, &Limits::defaults())
}

#[test]
fn av_invalidation_oracle_and_partial_rebuild_decode() {
    let spec = decode(&case(
        "ac-av/data-incremental-v1",
        "invalidation: { stable_defs: [\"d_unrelated\"], rebuilt_defs: [\"d_fan\"] }",
        "{ op: \"run\" }, { op: \"edit-source\", from: \"x + 1\", to: \"x + 2\" }, { op: \"run\" }",
    ))
    .unwrap();
    let av = spec.checks[0].options.probe_av.as_ref().unwrap();
    let inv = av.invalidation.as_ref().expect("invalidation decoded");
    assert_eq!(inv.stable_defs, vec!["d_unrelated".to_string()]);
    assert_eq!(inv.rebuilt_defs, vec!["d_fan".to_string()]);

    let spec = decode(&case(
        "ac-av/data-incremental-v1",
        "",
        "{ op: \"run\" }, { op: \"partial-rebuild\", from: \"x int\", to: \"x i64\", defs: [\"d_fan\"] }, { op: \"run\" }",
    ))
    .unwrap();
    assert!(matches!(
        spec.checks[0].options.probe_steps.as_deref(),
        Some([
            auto_test_case::contract::ProbeStep::Run,
            auto_test_case::contract::ProbeStep::PartialRebuild { .. },
            auto_test_case::contract::ProbeStep::Run
        ])
    ));
}

#[test]
fn av_invalidation_oracle_is_recipe_and_step_bound() {
    // The oracle is the data-incremental recipe's only.
    let err = decode(&case(
        "ac-av/data-run-v1",
        "invalidation: { stable_defs: [\"d_fan\"] }",
        "{ op: \"run\" }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("belongs to ac-av/data-incremental-v1"),
        "{err}"
    );

    // An edit step without the oracle is refused (judged, never executed).
    let err = decode(&case(
        "ac-av/data-incremental-v1",
        "",
        "{ op: \"run\" }, { op: \"edit-source\", from: \"a\", to: \"b\" }, { op: \"run\" }",
    ))
    .unwrap_err();
    assert!(err.message.contains("requires the av.invalidation oracle"), "{err}");

    // An oracle without an edit step dangles.
    let err = decode(&case(
        "ac-av/data-incremental-v1",
        "invalidation: { stable_defs: [\"d_fan\"] }",
        "{ op: \"run\" }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("without an edit-source step"),
        "{err}"
    );

    // A def can never be both stable and rebuilt.
    let err = decode(&case(
        "ac-av/data-incremental-v1",
        "invalidation: { stable_defs: [\"d_fan\"], rebuilt_defs: [\"d_fan\"] }",
        "{ op: \"run\" }, { op: \"edit-source\", from: \"a\", to: \"b\" }, { op: \"run\" }",
    ))
    .unwrap_err();
    assert!(err.message.contains("both stable and rebuilt"), "{err}");

    // partial-rebuild needs a non-empty defs list.
    let err = decode(&case(
        "ac-av/data-incremental-v1",
        "",
        "{ op: \"run\" }, { op: \"partial-rebuild\", from: \"x\", to: \"y\", defs: [] }, { op: \"run\" }",
    ))
    .unwrap_err();
    assert!(err.message.contains("defs must not be empty"), "{err}");
}

#[test]
fn partial_rebuild_is_data_incremental_only() {
    let err = decode(&case(
        "ac-av/core-run-v1",
        "",
        "{ op: \"run\" }, { op: \"partial-rebuild\", from: \"x\", to: \"y\", defs: [\"d_f\"] }, { op: \"run\" }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("not valid for recipe") || err.message.contains("unknown probe op"),
        "{err}"
    );
}
