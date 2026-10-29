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
        .contains("type t_closure_sig_0 closure { params: [\"t_i32\"]; result: \"t_i32\" }"));
    assert!(compiled.atom_text.contains(
        "type t_f_passthrough function { params: [\"t_closure_sig_0\"]; result: \"t_closure_sig_0\" }"
    ));

    let legacy = compile_collections_source(source, "collections-types.at", &[])
        .expect_err("the collections profile does not accept function types");
    assert_eq!(legacy[0].code, "source.syntax");
}

#[test]
fn closures_865_checks_and_emits_indirect_calls_through_function_params() {
    let source = "fn apply(f fn(int) -> int, value int) int { return f(value) }\nfn main() int { return 42 }";
    let compiled = compile_closures_source(source, "closures-call.at", &[])
        .expect("indirect call through a function-typed local compiles");
    assert!(compiled.atom_text.contains("call_closure"));
}

#[test]
fn closures_865_parses_move_expression_and_zero_arg_block_lambdas() {
    let source = "fn main() int { let add = move (n int) => n + 1; let answer = () => { return 42 }; let direct = ((n int) => n + 1)(3); return add(3) }";
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

    let Stmt::Let { value, .. } = &main.body.stmts[2] else {
        panic!("expected direct lambda call binding");
    };
    assert!(matches!(value.kind, ExprKind::CallValue { .. }));
}

#[test]
fn closures_865_compiles_capture_environment_and_invocation() {
    let source =
        "fn main() int { let base = 40; let add = (n int) => { return base + n }; return add(2) }";
    let compiled = compile_closures_source(source, "closures-capture.at", &[])
        .expect("copy capture closure compiles");
    assert!(compiled.atom_text.contains("role: capture; index: 0u32"));
    assert!(compiled.atom_text.contains("create_closure"));
    assert!(compiled.atom_text.contains("call_closure"));
}

#[test]
fn closures_865_moves_owned_capture_and_rejects_source_reuse() {
    let accepted = "fn main() int { let s Str = \"ok\"; let size = move () => { return s.len() }; return size() }";
    compile_closures_source(accepted, "closures-move.at", &[])
        .expect("move capture of an owned string compiles");

    let rejected =
        "fn main() int { let s Str = \"ok\"; let size = move () => s.len(); return s.len() }";
    let diagnostics = compile_closures_source(rejected, "closures-move-reuse.at", &[])
        .expect_err("moved capture source reuse is rejected");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "source.closure-use-after-move"));
}

#[test]
fn closures_865_allows_local_borrow_calls_and_rejects_borrow_escape() {
    let local = "fn main() int { let s Str = \"ok\"; let size = () => s.len(); return size() }";
    compile_closures_source(local, "closures-borrow-local.at", &[])
        .expect("local synchronous borrow capture compiles");

    let escaping =
        "fn make(s Str) fn() -> int { return () => s.len() }\nfn main() int { return 0 }";
    let diagnostics = compile_closures_source(escaping, "closures-borrow-escape.at", &[])
        .expect_err("borrow capture cannot escape through a return");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "source.closure-borrow-escape"));
}

#[test]
fn closures_865_hir_verifier_rejects_borrow_closure_escape_bypass() {
    let source = "fn main() int { let s Str = \"ok\"; let size = () => s.len(); return size() }";
    let compiled = compile_closures_source(source, "closures-hir-escape.at", &[])
        .expect("valid synchronous borrow closure compiles");
    let mut bundle = compiled.checked.bundle().clone();
    let module = &mut bundle.modules[0];
    let main = module
        .defs
        .iter()
        .find(|def| def.id_text == "d_main")
        .expect("main definition");
    let body_id = match &main.kind {
        auto_ac_prototype::hir::DefKind::Function { body, .. } => *body,
        _ => panic!("main is a function"),
    };
    let body = &mut module.bodies[body_id.0 as usize];
    let local = body
        .locals
        .iter()
        .find(|local| local.name == "size")
        .expect("bound closure local");
    let borrowed_local = local.id_text.clone();
    let closure_ty = local.ty;
    let return_value = body.blocks[body.entry.0 as usize]
        .stmts
        .iter_mut()
        .find_map(|stmt| match stmt {
            auto_ac_prototype::hir::Stmt::Return { value } => Some(*value),
            _ => None,
        })
        .expect("main return");
    let span = body.exprs[return_value.0 as usize].span;
    let local_id = body
        .locals
        .iter()
        .position(|local| local.id_text == borrowed_local)
        .unwrap() as u32;
    let escape_expr = auto_ac_prototype::hir::ExprId(body.exprs.len() as u32);
    body.exprs.push(auto_ac_prototype::hir::Expr {
        id_text: "e_borrow_escape_bypass".to_string(),
        ty: closure_ty,
        kind: auto_ac_prototype::hir::ExprKind::ReadLocal {
            local: auto_ac_prototype::hir::LocalId(local_id),
        },
        span,
    });
    let return_stmt = body.blocks[body.entry.0 as usize]
        .stmts
        .iter_mut()
        .find(|stmt| matches!(stmt, auto_ac_prototype::hir::Stmt::Return { .. }))
        .expect("main return");
    if let auto_ac_prototype::hir::Stmt::Return { value } = return_stmt {
        *value = escape_expr;
    }
    let diagnostics = auto_ac_prototype::verify::verify_closures(bundle)
        .expect_err("descriptor-authored borrow closure escapes are rejected by HIR verify");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "verify.closure-borrow-escape"));
}
