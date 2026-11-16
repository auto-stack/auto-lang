//! PLAN-1035 (F34) HIR negative control: the core collections verifier must
//! classify a LIST-ONLY record element as OWNED. A descriptor-authored (or
//! mutated) bundle that reads a whole owned element with a BARE `ListGet` —
//! through no legal view consumer — is rejected with `verify.view-escape`.
//!
//! The source resolver rejects the same shape, but the HIR gate is the
//! defense for bundles that did not come from it (the T-01 finding: the
//! element rule only walked towards `Str`, so a `List<Bag{items List<int>}>`
//! element looked copyable and slipped past this gate). The pair below is the
//! discriminating control: the SAME mutation over a pure-scalar record
//! element stays legal, so the rule really keys on the resource closure.

use ace_core::atom_text::Diagnostic;
use ace_core::{hir, source as ac, verify};

fn env(source: &str) -> ac::CheckedEnv {
    let parsed = ac::parse_program_with(source, ac::SourceProfile::Aggregates).unwrap();
    let mut env = ac::resolve_env(&parsed, &[], ac::SourceProfile::Aggregates).unwrap();
    env.typecheck_all().unwrap();
    env
}

/// Compile one body, then strip the field projection off the list read: the
/// `FieldGet` over the `ListGet` becomes a plain constant, leaving the
/// `ListGet` as a bare whole-element value. Everything else (types, blocks,
/// the return) stays what the real front end produced.
fn bare_element_bundle(source: &str) -> ac::SourceParts {
    let env = env(source);
    let (parts, receipt) = ac::emit_function_body(&env, 0, source, "p1035.at").unwrap();
    assert!(receipt.real_body_plans >= 1, "the body really planned");
    let mut parts = parts;
    let body = &mut parts.bundle.modules[0].bodies[0];
    let list_get = body
        .exprs
        .iter()
        .position(|e| matches!(e.kind, hir::ExprKind::ListGet { .. }))
        .expect("the legal program has a projected ListGet");
    let field_get = body
        .exprs
        .iter()
        .position(|e| matches!(&e.kind, hir::ExprKind::FieldGet { base, .. } if base.0 as usize == list_get))
        .expect("the legal program projects a field off the element");
    body.exprs[field_get].kind = hir::ExprKind::Constant { value: 0 };
    body.exprs[field_get].ty = hir::TypeUse::Builtin(hir::BuiltinType::I32);
    parts
}

fn codes(ds: &[Diagnostic]) -> Vec<String> {
    ds.iter().map(|d| d.code.clone()).collect()
}

/// The discriminator: same mutation, List-only record element vs pure-scalar
/// record element.
#[test]
fn list_only_record_element_is_owned_at_the_hir_gate() {
    // List-only record element: `Bag` owns its inner List, so the bare
    // element read is a view escape.
    let list_only = "type Bag { items List<int>; code int }\nfn main() int {\n    var xs = List<Bag>.new()\n    var inner = [42]\n    xs.push(Bag(inner.move, 7))\n    return xs.get(0).code\n}";
    let parts = bare_element_bundle(list_only);
    let errs = verify::verify_closures(parts.bundle)
        .err()
        .expect("the bare owned-element read must reject");
    assert!(
        errs.iter().any(|d| d.code == "verify.view-escape"),
        "expected verify.view-escape for the List-only record element, got {:?}",
        codes(&errs)
    );

    // Pure-scalar record element: `Pair` owns nothing, so the identical
    // bare read is a legal copy-out — no view escape.
    let scalar = "type Pair { a int; b int }\nfn main() int {\n    var xs = List<Pair>.new()\n    xs.push(Pair(40, 2))\n    return xs.get(0).a\n}";
    let parts = bare_element_bundle(scalar);
    match verify::verify_closures(parts.bundle) {
        Ok(_) => {}
        Err(errs) => assert!(
            !errs.iter().any(|d| d.code == "verify.view-escape"),
            "a pure-scalar element is copyable; got {:?}",
            codes(&errs)
        ),
    }
}

/// The record closure test itself: a record whose ONLY resource carrier is a
/// nested record that eventually holds a List is owned; the identical bare
/// element read over a scalar-only chain stays legal. Exercised through the
/// same verifier gate so the derivation, not the spelling, decides.
#[test]
fn nested_list_only_record_chain_is_owned_at_the_hir_gate() {
    let nested = "type Inner { items List<int> }\ntype Outer { inner Inner; code int }\nfn main() int {\n    var items = [42]\n    var iv = Inner(items.move)\n    var outer = Outer(iv.move, 7)\n    var xs = List<Outer>.new()\n    xs.push(outer.move)\n    return xs.get(0).code\n}";
    let parts = bare_element_bundle(nested);
    let errs = verify::verify_closures(parts.bundle)
        .err()
        .expect("the bare nested owned-element read must reject");
    assert!(
        errs.iter().any(|d| d.code == "verify.view-escape"),
        "expected verify.view-escape through the nested record chain, got {:?}",
        codes(&errs)
    );

    // Scalar-only chain: the same mutation is a legal copy-out.
    let scalar = "type Inner2 { a int }\ntype Outer2 { inner Inner2; code int }\nfn main() int {\n    var iv = Inner2(1)\n    var outer = Outer2(iv.move, 7)\n    var xs = List<Outer2>.new()\n    xs.push(outer.move)\n    return xs.get(0).code\n}";
    let parts = bare_element_bundle(scalar);
    match verify::verify_closures(parts.bundle) {
        Ok(_) => {}
        Err(errs) => assert!(
            !errs.iter().any(|d| d.code == "verify.view-escape"),
            "a scalar-only chain stays copyable; got {:?}",
            codes(&errs)
        ),
    }
}
