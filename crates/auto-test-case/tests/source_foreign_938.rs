use auto_test_case::{admission::Limits, atom::parse_contract, contract::decode_case};
fn decode(recipe: &str, leg: &str, oracle: &str) -> bool {
    let text=format!("case {{schema:\"autolang.case/2\",id:\"source938\",source:{{text:\"{{}}\",syntax:\"atom-data\",path:\"source.atomdata\"}},entry:{{mode:\"program\"}},checks:[{{id:\"certify\",adapter:\"probe\",action:\"run\",options:{{recipe:\"{recipe}\",abi:{{leg:\"{leg}\"}},steps:[{{op:\"query\"}}]}},expect:{{{oracle},probe:[{{}}]}}}}]}}");
    decode_case(&parse_contract(&text).unwrap(), &Limits::defaults()).is_ok()
}
#[test]
fn source_foreign_recipe_keeps_explicit_consumer_parent_oracle_and_old_leg_boundaries() {
    let recipe = "abi/source-foreign-certify-v1";
    for leg in ["native", "c"] {
        assert!(decode(recipe, leg, "completion:\"success\",exit:42"));
    }
    assert!(!decode(recipe, "native", "completion:\"success\""));
    for leg in ["a", "b", "c2n", "source", "typed-n2c", "av", "rust", ""] {
        assert!(!decode(recipe, leg, "completion:\"success\",exit:42"));
    }
    assert!(!decode(
        "abi/interop-v1",
        "native",
        "completion:\"success\",exit:42"
    ));
    assert!(!decode(
        "abi/ca-d1-bridge-v1",
        "native",
        "completion:\"success\",exit:42"
    ));
}

#[test]
fn source_foreign_restart_only_sequence_cannot_pass_without_a_judged_query() {
    let text="case {schema: \"autolang.case/2\",id:\"empty-query\",source:{text:\"{}\",syntax:\"atom-data\",path:\"source.atomdata\"},entry:{mode:\"program\"},checks:[{id:\"certify\",adapter:\"probe\",action:\"run\",options:{recipe:\"abi/source-foreign-certify-v1\",abi:{leg:\"native\"},steps:[{op:\"restart\"}]},expect:{completion:\"success\",exit:42}}]}";
    assert!(decode_case(&parse_contract(text).unwrap(), &Limits::defaults()).is_err());
}
