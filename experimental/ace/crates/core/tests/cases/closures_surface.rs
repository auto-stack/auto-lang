//! PLAN-865 T-03: the closure source surface has its own descriptor and
//! verifier identity while retaining the accepted collections subset.

use ace_core::source::ast::{ClosureBody, ExprKind, Item, Stmt};
use ace_core::source::{
    compile_closures_source, compile_collections_source, parse_program_with, SourceProfile,
};
use ace_core::{closures_descriptor, collections_descriptor, hir::HirDomain};

const SIMPLE: &str = "fn main() int { return 42 }";

#[test]
fn closures_has_distinct_descriptor_and_verifier_domain() {
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
fn closures_resolves_and_emits_higher_order_function_types() {
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
fn closures_checks_and_emits_indirect_calls_through_function_params() {
    let source = "fn apply(f fn(int) -> int, value int) int { return f(value) }\nfn main() int { return 42 }";
    let compiled = compile_closures_source(source, "closures-call.at", &[])
        .expect("indirect call through a function-typed local compiles");
    assert!(compiled.atom_text.contains("call_closure"));
}

#[test]
fn closures_rejects_bad_call_arity_signature_and_async_closures() {
    let wrong_arity = "fn main() int { let f = (n int) => n + 1; return f() }";
    let diagnostics = compile_closures_source(wrong_arity, "closures-arity.at", &[])
        .expect_err("a closure call with the wrong number of arguments is rejected");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "source.argument"
                && diagnostic.message.contains("closure expects 1 argument")
        }),
        "wrong-arity diagnostics: {diagnostics:#?}"
    );

    let wrong_signature = "fn main() int { let f fn(bool) -> int = (n int) => n; return f(true) }";
    let diagnostics = compile_closures_source(wrong_signature, "closures-signature.at", &[])
        .expect_err("a closure with a different function signature is rejected");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "source.type-mismatch"),
        "signature diagnostics: {diagnostics:#?}"
    );

    let async_closure = "fn main() int { let f = async (n int) => n + 1; return 0 }";
    let diagnostics = compile_closures_source(async_closure, "closures-async.at", &[])
        .expect_err("async closures remain unsupported");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "source.closure-unsupported" && diagnostic.message.contains("async")
        }),
        "async-closure diagnostics: {diagnostics:#?}"
    );
}

#[test]
fn closures_parses_move_expression_and_zero_arg_block_lambdas() {
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
fn closures_compiles_capture_environment_and_invocation() {
    let source =
        "fn main() int { let base = 40; let add = (n int) => { return base + n }; return add(2) }";
    let compiled = compile_closures_source(source, "closures-capture.at", &[])
        .expect("copy capture closure compiles");
    assert!(compiled.atom_text.contains("role: capture; index: 0u32"));
    assert!(compiled.atom_text.contains("create_closure"));
    assert!(compiled.atom_text.contains("call_closure"));
}

#[test]
fn closures_moves_owned_capture_and_rejects_source_reuse() {
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
fn closures_allows_local_borrow_calls_and_rejects_borrow_escape() {
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
fn closures_hir_verifier_rejects_borrow_closure_escape_bypass() {
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
        ace_core::hir::DefKind::Function { body, .. } => *body,
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
            ace_core::hir::Stmt::Return { value } => Some(*value),
            _ => None,
        })
        .expect("main return");
    let span = body.exprs[return_value.0 as usize].span;
    let local_id = body
        .locals
        .iter()
        .position(|local| local.id_text == borrowed_local)
        .unwrap() as u32;
    let escape_expr = ace_core::hir::ExprId(body.exprs.len() as u32);
    body.exprs.push(ace_core::hir::Expr {
        id_text: "e_borrow_escape_bypass".to_string(),
        ty: closure_ty,
        kind: ace_core::hir::ExprKind::ReadLocal {
            local: ace_core::hir::LocalId(local_id),
        },
        span,
    });
    let return_stmt = body.blocks[body.entry.0 as usize]
        .stmts
        .iter_mut()
        .find(|stmt| matches!(stmt, ace_core::hir::Stmt::Return { .. }))
        .expect("main return");
    if let ace_core::hir::Stmt::Return { value } = return_stmt {
        *value = escape_expr;
    }
    let diagnostics = ace_core::verify::verify_closures(bundle)
        .expect_err("descriptor-authored borrow closure escapes are rejected by HIR verify");
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "verify.closure-borrow-escape"));
}

// -- PLAN-909 shared mutable captures: lexical facts and checked-fact gates --

/// V-04: the shared-cell group is LEXICAL (binding identity), not by name —
/// two sibling `var x` bindings: only the one a non-move closure writes
/// becomes `shared`; the read-only sibling stays a `move` snapshot.
#[test]
fn closures_909_shared_group_is_lexical_not_by_name() {
    let source = "fn main() int { var r int = 0; if 1 < 2 { var x int = 1; let f = () => { return x }; r = r + f() }; if 1 < 2 { var x int = 10; let g = () => { x = x + 5; return x }; r = r + g() }; return r }";
    let compiled = compile_closures_source(source, "closures-909-lexical.at", &[])
        .expect("sibling shadowed vars compile");
    let shared = compiled.atom_text.matches("mode: shared").count();
    let moved = compiled.atom_text.matches("mode: move").count();
    assert_eq!(
        shared, 1,
        "exactly the written sibling is shared:\n{}",
        compiled.atom_text
    );
    assert_eq!(
        moved, 1,
        "the read-only sibling stays a snapshot:\n{}",
        compiled.atom_text
    );
}

/// K06 fact shape: a move closure that writes a scalar capture declares the
/// callee capture local `mutable: true` (the checked private-cell fact) and
/// keeps `mode: move` at the creating site.
#[test]
fn closures_909_k06_private_writable_capture_fact() {
    let source = "fn main() int { var base int = 1; var f = move (n int) => { base = base + n; return base }; return f(20) + f(0) }";
    let compiled =
        compile_closures_source(source, "closures-909-k06.at", &[]).expect("K06 compiles");
    assert!(
        compiled
            .atom_text
            .contains("role: capture; index: 0u32; mutable: true"),
        "{}",
        compiled.atom_text
    );
    assert!(
        compiled.atom_text.contains("mode: move"),
        "{}",
        compiled.atom_text
    );
    assert!(
        !compiled.atom_text.contains("mode: shared"),
        "{}",
        compiled.atom_text
    );
}

fn bundle_of(source: &str, label: &str) -> ace_core::hir::Bundle {
    compile_closures_source(source, label, &[])
        .expect("valid source compiles")
        .checked
        .bundle()
        .clone()
}

fn flip_capture_mode(bundle: &mut ace_core::hir::Bundle, mode: ace_core::hir::CaptureMode) {
    let module = &mut bundle.modules[0];
    for body in &mut module.bodies {
        for expr in &mut body.exprs {
            if let ace_core::hir::ExprKind::CreateClosure { captures, .. } = &mut expr.kind {
                for capture in captures.iter_mut() {
                    capture.mode = mode;
                }
            }
        }
    }
}

/// V-10: a descriptor-authored `shared` capture over an IMMUTABLE `let`
/// source is rejected by HIR verify (the cell domain is `var` only).
#[test]
fn closures_909_hir_verifier_rejects_shared_over_immutable_source() {
    let mut bundle = bundle_of(
        "fn main() int { let base int = 40; let f = (n int) => { return base + n }; return f(2) }",
        "closures-909-fake-shared-let.at",
    );
    flip_capture_mode(&mut bundle, ace_core::hir::CaptureMode::Shared);
    let diagnostics = ace_core::verify::verify_closures(bundle)
        .expect_err("shared over an immutable source is outside the cell domain");
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == "verify.closure-shared-domain"),
        "{diagnostics:#?}"
    );
}

/// V-10: a descriptor-authored `shared` capture over a NON-SCALAR (owned
/// Str) `var` source is rejected — no owned aggregate rides the cell.
#[test]
fn closures_909_hir_verifier_rejects_shared_over_non_scalar() {
    let mut bundle = bundle_of(
        "fn make(move s Str) fn(int) int { return move (n int) => { return s.len() + n } } fn main() int { let s Str = \"hello\"; let f = make(s.move); return f(37) }",
        "closures-909-fake-shared-str.at",
    );
    flip_capture_mode(&mut bundle, ace_core::hir::CaptureMode::Shared);
    let diagnostics = ace_core::verify::verify_closures(bundle)
        .expect_err("shared over an owned Str is outside the cell domain");
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == "verify.closure-shared-domain"),
        "{diagnostics:#?}"
    );
}

/// V-10: a descriptor-authored WRITABLE (`mutable: true`) capture local
/// behind a `borrow` capture is rejected — borrow captures are never
/// writable, and a writable move capture must be scalar.
#[test]
fn closures_909_hir_verifier_rejects_writable_borrow_capture() {
    let mut bundle = bundle_of(
        "fn main() int { let s Str = \"ok\"; let size = () => s.len(); return size() }",
        "closures-909-fake-writable-borrow.at",
    );
    let module = &mut bundle.modules[0];
    let mut flipped = 0;
    for body in &mut module.bodies {
        for local in &mut body.locals {
            if local.role == ace_core::hir::LocalRole::Capture {
                local.mutable = true;
                flipped += 1;
            }
        }
    }
    assert!(flipped > 0, "the fixture has a borrow capture local");
    let diagnostics = ace_core::verify::verify_closures(bundle)
        .expect_err("a writable borrow capture is rejected");
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == "verify.closure-shared-domain"),
        "{diagnostics:#?}"
    );
}
