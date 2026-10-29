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
