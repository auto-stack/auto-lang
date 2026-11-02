//! PLAN-914 (I04): the declaration/layout invalidation oracle
//! (`mn.declaration` + the `declaration-invalidate` round).
//!
//! The round is admitted on the collections incremental recipe only (the
//! scalar profile has no declaration surface), the oracle must be present
//! exactly when a declaration round is declared, and the oracle fields are
//! bounded integers / non-empty unit keys.

use auto_test_case::admission::Limits;
use auto_test_case::atom::parse_contract;
use auto_test_case::contract::decode_case;

fn case(recipe: &str, mn: &str, steps: &str) -> String {
    format!(
        "case {{ schema: \"autolang.case/2\", id: \"f914.decl\", \
         source: {{ text: \"type P {{ x int }}\", syntax: \"auto\", path: \"main.at\" }}, \
         entry: {{ mode: \"program\" }}, \
         checks: [ {{ id: \"mn\", adapter: \"probe\", action: \"run\", \
         options: {{ recipe: \"{}\", mn: {{ {} }}, steps: [ {} ] }}, \
         expect: {{ completion: \"success\", exit: 0 }} }} ] }}",
        recipe, mn, steps
    )
}

fn decode(src: &str) -> Result<auto_test_case::contract::CaseSpec, auto_test_case::contract::ContractError> {
    let v = parse_contract(src).unwrap();
    decode_case(&v, &Limits::defaults())
}

const DECL_STEP: &str =
    "{ op: \"declaration-invalidate\", from: \"x int\", to: \"x int\\n    y int\" }, { op: \"run\" }";

#[test]
fn declaration_oracle_decodes_on_the_collections_recipe() {
    let spec = decode(&case(
        "ac/mir-native-collections-v1",
        "entry: \"d_main\", declaration: { min_frontend_invalidations: 2, max_codegen_units: 3, stable_defs: [\"unrelated\"] }",
        &format!("{{ op: \"run\" }}, {}", DECL_STEP),
    ))
    .unwrap();
    let decl = spec.checks[0]
        .options
        .probe_mn
        .as_ref()
        .and_then(|mn| mn.declaration.as_ref())
        .expect("declaration oracle decoded");
    assert_eq!(decl.min_frontend_invalidations, 2);
    assert_eq!(decl.max_codegen_units, Some(3));
    assert_eq!(decl.stable_defs, vec!["unrelated".to_string()]);
    assert!(matches!(
        spec.checks[0].options.probe_steps.as_deref(),
        Some([
            auto_test_case::contract::ProbeStep::Run,
            auto_test_case::contract::ProbeStep::DeclarationInvalidate { .. },
            auto_test_case::contract::ProbeStep::Run
        ])
    ));
}

#[test]
fn declaration_oracle_is_collections_only() {
    // With the round declared, the scalar recipe rejects the STEP vocabulary
    // first (it has no declaration surface at all).
    let scalar = decode(&case(
        "ac/mir-native-scalar-v1",
        "entry: \"d_main\", declaration: { min_frontend_invalidations: 1 }",
        &format!("{{ op: \"run\" }}, {}", DECL_STEP),
    ))
    .unwrap_err();
    assert!(
        scalar.message.contains("not valid for recipe"),
        "{scalar}"
    );

    // The typed recipe admits only a single run step; a bare oracle (no
    // round) is rejected for every non-collections recipe by the oracle gate.
    let bare = decode(&case(
        "ac/mir-native-scalar-v1",
        "entry: \"d_main\", declaration: { min_frontend_invalidations: 1 }",
        "{ op: \"run\" }",
    ))
    .unwrap_err();
    assert!(
        bare.message.contains("declaration oracle belongs to"),
        "{bare}"
    );
}

#[test]
fn declaration_round_requires_the_oracle_and_vice_versa() {
    // A declaration round without the oracle: never merely executed.
    let err = decode(&case(
        "ac/mir-native-collections-v1",
        "entry: \"d_main\"",
        &format!("{{ op: \"run\" }}, {}", DECL_STEP),
    ))
    .unwrap_err();
    assert!(
        err.message.contains("requires the mn.declaration oracle"),
        "{err}"
    );

    // An oracle without the round: never a parsed-then-ignored field.
    let err = decode(&case(
        "ac/mir-native-collections-v1",
        "entry: \"d_main\", declaration: { min_frontend_invalidations: 1 }",
        "{ op: \"run\" }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("without a declaration-invalidate step"),
        "{err}"
    );
}

#[test]
fn declaration_round_is_not_admitted_on_the_scalar_recipe() {
    let err = decode(&case(
        "ac/mir-native-scalar-v1",
        "entry: \"d_main\"",
        &format!("{{ op: \"run\" }}, {}", DECL_STEP),
    ))
    .unwrap_err();
    assert!(
        err.message.contains("not valid for recipe") || err.message.contains("unknown probe op"),
        "{err}"
    );
}


