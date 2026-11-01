//! PLAN-904 T-04/T-05/T-08 face conformance (V-03/V-07): the collections
//! channel's FRONT-DOOR rejection set (the named source-domain gates the
//! 864/889 contracts freeze — the channel surfaces them as compile
//! diagnostics, never as rustc errors or silent acceptance) and the
//! dependency-invalidation boundary (element/arm/field changes move the
//! fingerprint; equivalent table order does not).

use ace_a2x_core::{SemanticView, Ty};
use ace_core::source::compile_collections_source;

fn reject(src: &str, want_code: &str) {
    let err = compile_collections_source(src, "face", &[])
        .expect_err("expected a named frontend rejection");
    assert!(
        err.iter().any(|d| d.code == want_code),
        "expected `{want_code}`, got: {err:?}\nsrc:\n{src}"
    );
}

#[test]
fn v03_front_door_named_rejections() {
    // source-level container clone is rejected (List/Variant own identity)
    reject(
        "fn main() int {\n    var xs = List<int>.new()\n    let ys = xs.clone()\n    return 0\n}",
        "source.unsupported",
    );
    // List in param position is a named value-context rejection
    reject(
        "fn consume(xs List<int>) int { return 0 }\nfn main() int { return 1 }",
        "source.list-value-context",
    );
    // variant in record-field position
    reject(
        "enum E { A\nEnd }\ntype W { v E }\nfn main() int { return 0 }",
        "source.list-value-context",
    );
    // missing arm in `is`
    reject(
        "enum E { A(int)\nB }\nfn main() int {\n    let e = E.A(1)\n    is e {\n        E.A(v) -> { return v }\n    }\n}",
        "source.pattern-non-exhaustive",
    );
    // duplicate arm
    reject(
        "enum E { A(int)\nB }\nfn main() int {\n    let e = E.A(1)\n    is e {\n        E.A(v) -> { return v }\n        E.A(v) -> { return v }\n        E.B -> { return 0 }\n    }\n}",
        "source.pattern-duplicate",
    );
    // owned-element push without .move
    reject(
        "type I { n Str }\nfn main() int {\n    var xs = List<I>.new()\n    let x I = I(\"a\")\n    xs.push(x)\n    return 0\n}",
        "source.move-required",
    );
    // owned-element view escape (whole-element value use of a projection)
    reject(
        "type I { n Str }\nfn main() int {\n    var xs = List<I>.new()\n    xs.push(I(\"a\"))\n    let v I = xs.get(0)\n    return 0\n}",
        "source.view-escape",
    );
    // structural mutation of the iterated list
    reject(
        "fn main() int {\n    var xs = List<int>.new()\n    xs.push(1)\n    var t = 0\n    for x in xs {\n        xs.push(2)\n        t = t + x\n    }\n    return t\n}",
        "source.iterate-mutate",
    );
}

#[test]
fn v07_dependency_invalidation_boundary() {
    // The same module with an ELEMENT TYPE change vs a pure table reorder:
    // the element change moves main's fingerprint; the reorder does not.
    let base = "fn helper(v int) int { return v * 2 }\ntype W { n Str }\nfn main() int {\n    var xs = List<W>.new()\n    xs.push(W(\"ab\"))\n    return xs.get(0).n.len() + helper(20)\n}";
    let reordered = "type W { n Str }\nfn helper(v int) int { return v * 2 }\nfn main() int {\n    var xs = List<W>.new()\n    xs.push(W(\"ab\"))\n    return xs.get(0).n.len() + helper(20)\n}";
    let elem_change = "fn helper(v int) int { return v * 2 }\ntype W { n int }\nfn main() int {\n    var xs = List<int>.new()\n    xs.push(3)\n    return xs.get(0) + helper(20)\n}";

    let fp = |src: &str, name: &str| {
        let compiled = compile_collections_source(src, "dep", &[]).unwrap();
        let view = SemanticView::from_checked(&compiled.checked).unwrap();
        let f = view.functions().iter().find(|f| f.source_name == name).unwrap();
        ace_a2x_core::semantic_fingerprint(&view, f).unwrap()
    };
    let fp_base = fp(base, "main");
    // pure declaration reorder: same semantic identity
    assert_eq!(fp(reordered, "main"), fp_base, "table reorder must not move");
    // element/type change: different identity (nominal digest + list elem)
    assert_ne!(
        fp(elem_change, "main"),
        fp_base,
        "an element/field change must invalidate"
    );

    // the view exposes the nominal graph the fingerprint digests
    let compiled = compile_collections_source(base, "dep", &[]).unwrap();
    let view = SemanticView::from_checked(&compiled.checked).unwrap();
    assert_eq!(view.variants().len(), 0, "no variants in this module");
    assert!(view
        .functions()
        .iter()
        .any(|f| f
            .body
            .exprs
            .iter()
            .any(|e| matches!(e.kind, ace_core::hir::ExprKind::ListGet { .. }))));
    // the List element type resolves through the view
    let main = view.functions().iter().find(|f| f.source_name == "main").unwrap();
    let list_local = main
        .body
        .locals
        .iter()
        .find_map(|l| match view.resolve_use_ty(l.ty) {
            Ok(Ty::List(id)) => Some(view.list_elem(id).unwrap()),
            _ => None,
        })
        .expect("a list local");
    assert!(matches!(list_local, Ty::Record(_)));
}

/// PLAN-904 formal-load finding (durable counterexample): the `.len()`
/// view fusion over `List<Str>` mis-ordered its deferred index visit —
/// the SECOND fused view rendered index 0. The fix carries the index task
/// in the node's subtask list; both indices must survive to the HIR.
#[test]
fn v02_fused_view_indices_preserved() {
    let compiled = compile_collections_source(
        "fn main() int {\n    var zs = List<Str>.new()\n    let a Str = \"aaaa\"\n    zs.push(a.move)\n    let b Str = \"bb\"\n    zs.push(b.move)\n    return zs.get(0).len() + zs.get(1).len()\n}",
        "face",
        &[],
    )
    .expect("compile");
    let view = SemanticView::from_checked(&compiled.checked).unwrap();
    let main = view.functions().iter().find(|f| f.source_name == "main").unwrap();
    let indices: Vec<i32> = main
        .body
        .exprs
        .iter()
        .filter_map(|e| match e.kind {
            ace_core::hir::ExprKind::ListGet { index, .. } => match main.body.exprs[index.0 as usize].kind {
                ace_core::hir::ExprKind::Constant { value } => Some(value),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(indices, vec![0, 1], "fused views must keep their own indices");
}
