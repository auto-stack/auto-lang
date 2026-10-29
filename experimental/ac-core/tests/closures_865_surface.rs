//! PLAN-865 T-03: the closure source surface has its own descriptor and
//! verifier identity while retaining the accepted collections subset.

use auto_ac_prototype::source::{compile_closures_source, compile_collections_source};
use auto_ac_prototype::{closures_descriptor, collections_descriptor, hir::HirDomain};

const SIMPLE: &str = "fn main() int { return 42 }";

#[test]
fn closures_865_has_distinct_descriptor_and_verifier_domain() {
    let closures = compile_closures_source(SIMPLE, "closures.at", &[]).expect("closures compile");
    assert_eq!(closures.profile, "auto.source.closures.draft");
    assert_eq!(closures.checked.domain(), HirDomain::Closures);
    assert_eq!(closures_descriptor().domain(), Some(HirDomain::Closures));
    assert_ne!(
        closures_descriptor().schema,
        collections_descriptor().schema
    );

    let collections =
        compile_collections_source(SIMPLE, "collections.at", &[]).expect("collections compile");
    assert_eq!(collections.checked.domain(), HirDomain::Collections);
}

#[test]
fn closures_865_resolves_and_emits_higher_order_function_types() {
    let source =
        "fn passthrough(f fn(int) -> int) fn(int) -> int { return f }\nfn main() int { return 42 }";
    let compiled = compile_closures_source(source, "closures-types.at", &[])
        .expect("higher-order function types compile");

    assert!(compiled
        .atom_text
        .contains("type t_closure_sig_0 function { params: [\"t_i32\"]; result: \"t_i32\" }"));
    assert!(compiled.atom_text.contains(
        "type t_f_passthrough function { params: [\"t_closure_sig_0\"]; result: \"t_closure_sig_0\" }"
    ));

    let legacy = compile_collections_source(source, "collections-types.at", &[])
        .expect_err("the collections profile does not accept function types");
    assert_eq!(legacy[0].code, "source.syntax");
}
