//! PLAN-911 T-03: the `mn.diagnostics` decoder admission — an explicit,
//! optional, scalar-only observation mode ("source-v1"); anything else is
//! a named decode error, and omitted keeps the pre-911 contract.

use auto_test_case::admission::Limits;
use auto_test_case::atom::parse_contract;
use auto_test_case::contract::{decode_case, CASE_SCHEMA};

fn mn_case(mn_extra: &str, recipe: &str) -> String {
    format!(
        "case {{ schema: \"autolang.case/2\", id: \"f911.mn\", \
         source: {{ text: \"fn main() int {{ return 1 }}\", syntax: \"auto\", path: \"main.at\" }}, \
         entry: {{ mode: \"program\" }}, \
         checks: [ {{ id: \"ac-native\", adapter: \"probe\", action: \"run\", \
         options: {{ recipe: \"{}\", mn: {{ entry: \"d_main\"{} }}, steps: [ {{ op: \"run\" }} ] }}, \
         expect: {{ completion: {{ stage: \"typecheck\", code: \"source.type-mismatch\" }} }} }} ] }}",
        recipe, mn_extra
    )
}

fn decode(
    src: &str,
) -> Result<auto_test_case::contract::CaseSpec, auto_test_case::contract::ContractError> {
    let v = parse_contract(src).unwrap();
    decode_case(&v, &Limits::defaults())
}

#[test]
fn mn_diagnostics_accepts_only_source_v1_on_scalar() {
    let good = mn_case(", diagnostics: \"source-v1\"", "ac/mir-native-scalar-v1");
    let decoded = decode(&good).expect("source-v1 on the scalar recipe decodes");
    assert_eq!(
        decoded.checks[0]
            .options
            .probe_mn
            .as_ref()
            .and_then(|m| m.diagnostics.as_deref()),
        Some("source-v1")
    );
    // Omitted keeps the legacy contract (None — the pre-911 behavior).
    let plain = decode(&mn_case("", "ac/mir-native-scalar-v1")).expect("omitted decodes");
    assert_eq!(
        plain.checks[0]
            .options
            .probe_mn
            .as_ref()
            .and_then(|m| m.diagnostics.as_ref()),
        None
    );
    for bad in [
        // Wrong value.
        mn_case(", diagnostics: \"source-v2\"", "ac/mir-native-scalar-v1"),
        mn_case(", diagnostics: \"\"", "ac/mir-native-scalar-v1"),
        // Wrong type.
        mn_case(", diagnostics: 1", "ac/mir-native-scalar-v1"),
        mn_case(", diagnostics: [\"source-v1\"]", "ac/mir-native-scalar-v1"),
        // Not the scalar recipe's field.
        mn_case(
            ", diagnostics: \"source-v1\", typed: { fixture: \"f\", args: [] }",
            "ac/mir-native-typed-v1",
        ),
    ] {
        let e = decode(&bad).expect_err("must reject");
        assert!(
            e.to_string().contains("mn.diagnostics"),
            "named error mentioning mn.diagnostics: {e}"
        );
    }
}
