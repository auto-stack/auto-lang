//! PLAN-865 T-03: the closure source surface has its own descriptor and
//! verifier identity while retaining the accepted collections subset.

use auto_ac_prototype::source::ast::{ClosureBody, ExprKind, Item, Stmt};
use auto_ac_prototype::source::{
    compile_closures_source, compile_collections_source, parse_program_with, SourceProfile,
};
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

#[test]
fn closures_865_parses_move_expression_and_zero_arg_block_lambdas() {
    let source = "fn main() int { let add = move (n int) => n + 1; let answer = () => { return 42 }; return add(3) }";
    let program =
        parse_program_with(source, SourceProfile::Closures).expect("lambda syntax parses");
    let Item::Fn(main) = &program.items[0] else {
        panic!("expected main function");
    };
    let Stmt::Let { value, .. } = &main.body.stmts[0] else {
        panic!("expected first lambda binding");
    };
    let ExprKind::Closure {
        move_capture,
        params,
        body,
    } = &value.kind
    else {
        panic!("expected move closure");
    };
    assert!(*move_capture);
    assert_eq!(params.len(), 1);
    assert!(matches!(body.as_ref(), ClosureBody::Expr(_)));

    let Stmt::Let { value, .. } = &main.body.stmts[1] else {
        panic!("expected second lambda binding");
    };
    let ExprKind::Closure {
        move_capture,
        params,
        body,
    } = &value.kind
    else {
        panic!("expected block closure");
    };
    assert!(!move_capture);
    assert!(params.is_empty());
    assert!(matches!(body.as_ref(), ClosureBody::Block(_)));
}
