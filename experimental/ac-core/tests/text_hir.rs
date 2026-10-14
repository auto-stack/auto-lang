//! PLAN-770 T-02 text-domain HIR tests: triple-identity isolation (V-01,
//! HIR half), programmatic bundle mutation gates (V-06), the text typing
//! matrix (V-03 static legs), exact resource-plan contents (exit frees in
//! LIFO order, owned returns) and canonical roundtrip. Pure in-process
//! verification — native execution lives in text_native.rs (T-05).

use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::descriptor;
use auto_ac_prototype::hir::{self, ExprKind, HirDomain};
use auto_ac_prototype::{bind_document, text_descriptor, verify};

fn fixture(kind: &str, name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("experimental/ac-core/fixtures/text-hir")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn bind_text_ok(src: &str) -> hir::Bundle {
    let b = descriptor::bind(text_descriptor(), src).expect("text bind must succeed");
    assert_eq!(
        b.domain,
        HirDomain::Text,
        "binder must stamp the text domain"
    );
    b
}

fn verify_text_ok(src: &str) -> verify::CheckedModule {
    let checked = verify::verify_text(bind_text_ok(src)).expect("text verify must succeed");
    assert_eq!(checked.domain(), HirDomain::Text);
    checked
}

fn verify_text_err(bundle: hir::Bundle) -> Vec<Diagnostic> {
    verify::verify_text(bundle).expect_err("verify_text must reject")
}

fn codes(ds: &[Diagnostic]) -> Vec<&str> {
    ds.iter().map(|d| d.code.as_str()).collect()
}

fn has(ds: &[Diagnostic], code: &str) -> bool {
    ds.iter().any(|d| d.code == code)
}

fn body_of_mut(b: &mut hir::Bundle, idx: usize) -> &mut hir::Body {
    &mut b.modules[0].bodies[idx]
}

// -- V-01: identity, isolation, dispatch --------------------------------------

#[test]
fn text_identity_binds_and_verifies() {
    let checked = verify_text_ok(&fixture("valid", "t01-text-ops.atom"));
    let m = &checked.bundle().modules[0];
    // five functions, all verified; the text kinds are present exactly once
    assert_eq!(m.defs.len(), 5);
    let mut kinds: Vec<&str> = m.bodies[0]
        .exprs
        .iter()
        .filter_map(|e| match &e.kind {
            ExprKind::TextLen { .. } => Some("text_len"),
            _ => None,
        })
        .collect();
    kinds.sort();
    assert_eq!(kinds, vec!["text_len"]);
    // str/Str/char all appear among local types (b_lens has str, b_consume
    // has Str+move mode, b_char has char)
    let tys: Vec<&str> = m
        .bodies
        .iter()
        .flat_map(|b| b.locals.iter())
        .map(|l| match l.ty {
            hir::TypeUse::Builtin(hir::BuiltinType::StrSlice) => "str",
            hir::TypeUse::Builtin(hir::BuiltinType::StrOwned) => "Str",
            hir::TypeUse::Builtin(hir::BuiltinType::Char) => "char",
            hir::TypeUse::Builtin(hir::BuiltinType::I32) => "i32",
            hir::TypeUse::Builtin(hir::BuiltinType::Bool) => "bool",
            hir::TypeUse::Ref(_) => "ref",
        })
        .collect();
    assert!(tys.contains(&"Str"), "owned local present: {:?}", tys);
    assert!(tys.contains(&"str"), "borrowed local present: {:?}", tys);
    // the move-mode param carries its marker through the binder
    assert_eq!(
        m.bodies[3].locals[0].param_mode,
        hir::ParamMode::MoveOwn,
        "l_o binds mode move"
    );
}

#[test]
fn bind_document_dispatches_text_triple() {
    let src = fixture("valid", "t01-text-ops.atom");
    let b = bind_document(&src).expect("registry dispatch must accept the text triple");
    assert_eq!(b.domain, HirDomain::Text);
    // unknown/wrong-dimension triples name the accepted set, never guess
    let mut wrong = src.replace("profile: \"text-i32-draft\"", "profile: \"text-draft\"");
    wrong = wrong.replace(
        "schema: \"auto.hir.text.draft\"",
        "schema: \"auto.hir.text.draft\"",
    );
    let err = bind_document(&wrong).expect_err("wrong profile must not dispatch");
    assert_eq!(err[0].code, "bind.descriptor-unknown");
    assert!(
        err[0].message.contains("text-i32-draft"),
        "accepted identities listed: {}",
        err[0].message
    );
    // one-dimension-off on revision is equally rejected
    let wrong_rev = src.replace("revision: 1u32", "revision: 2u32");
    let err = bind_document(&wrong_rev).expect_err("wrong revision must not dispatch");
    assert_eq!(err[0].code, "bind.descriptor-unknown");
}

#[test]
fn old_verifier_entries_reject_text_domain() {
    let b = bind_text_ok(&fixture("valid", "t01-text-ops.atom"));
    let ds = verify::verify(b.clone()).expect_err("core entry must reject text bundle");
    assert!(has(&ds, "verify.domain"), "{:?}", codes(&ds));
    let ds = verify::verify_scalar(b).expect_err("scalar entry must reject text bundle");
    assert!(has(&ds, "verify.domain"), "{:?}", codes(&ds));
}

#[test]
fn text_descriptor_rejects_scalar_identity() {
    // a scalar-header document bound under the TEXT descriptor is an
    // identity mismatch, not a re-shelve
    let scalar_src = r#"
hir(schema: "auto.hir.scalar.draft", revision: 1u32, profile: "scalar-i32-draft") {
    module(id: "m") {
        types { type(id: "t0", kind: function) { params: []; result: i32 } }
        declarations { function(id: "d0", type: "t0") { name: "f"; body: "b0" } }
        bodies {
            body(id: "b0") {
                owner: "d0"
                entry: "k0"
                expressions {
                    expr(id: "e0", kind: constant) { type: i32; value: 1i32 }
                }
                blocks { block(id: "k0") { return(value: "e0") {} } }
            }
        }
    }
}
"#;
    let ds = descriptor::bind(text_descriptor(), scalar_src)
        .expect_err("text descriptor must reject the scalar triple");
    assert!(
        has(&ds, "bind.schema-mismatch") || has(&ds, "bind.profile-mismatch"),
        "{:?}",
        codes(&ds)
    );
}

#[test]
fn text_nodes_injected_into_old_profiles_are_rejected() {
    // core profile: a text expr smuggled into a bound core bundle
    let core_src = r#"
hir(schema: "auto.hir.core.draft", revision: 1u32, profile: "core-i32-draft") {
    module(id: "m") {
        types { type(id: "t0", kind: function) { params: []; result: i32 } }
        declarations { function(id: "d0", type: "t0") { name: "f"; body: "b0" } }
        bodies {
            body(id: "b0") {
                owner: "d0"
                entry: "k0"
                locals { local(id: "l0", type: str) { name: "s"; role: binding } }
                expressions {
                    expr(id: "e0", kind: read_local) { type: str; local: "l0" }
                    expr(id: "e1", kind: text_len) { type: i32; operand: "e0" }
                }
                blocks { block(id: "k0") { return(value: "e1") {} } }
            }
        }
    }
}
"#;
    // str is not even bindable in core (dangling type ref — isolation at
    // the binder level, 770 design §4.5)
    let ds = descriptor::bind(auto_ac_prototype::descriptor(), core_src)
        .expect_err("str typeuse must not bind under the core descriptor");
    assert!(has(&ds, "bind.dangling-type-ref"), "{:?}", codes(&ds));

    // programmatic injection: mutate a valid scalar bundle into a text node
    let scalar = r#"
hir(schema: "auto.hir.scalar.draft", revision: 1u32, profile: "scalar-i32-draft") {
    module(id: "m") {
        types { type(id: "t0", kind: function) { params: []; result: i32 } }
        declarations { function(id: "d0", type: "t0") { name: "f"; body: "b0" } }
        bodies {
            body(id: "b0") {
                owner: "d0"
                entry: "k0"
                expressions {
                    expr(id: "e0", kind: constant) { type: i32; value: 1i32 }
                    expr(id: "e1", kind: constant) { type: i32; value: 2i32 }
                    expr(id: "e2", kind: binary) { type: i32; op: add_i32; overflow: trap; lhs: "e0"; rhs: "e1" }
                }
                blocks { block(id: "k0") { return(value: "e2") {} } }
            }
        }
    }
}
"#;
    let mut b =
        descriptor::bind(auto_ac_prototype::scalar_descriptor(), scalar).expect("scalar bind");
    if let ExprKind::Binary { lhs, .. } = &mut b.modules[0].bodies[0].exprs[2].kind {
        *lhs = hir::ExprId(1);
    }
    // inject a text node shape by replacing e2 with a TextLen
    b.modules[0].bodies[0].exprs[2].kind = ExprKind::TextLen {
        operand: hir::ExprId(0),
    };
    let ds = verify::verify_scalar(b).expect_err("scalar must reject text nodes");
    assert!(has(&ds, "verify.profile-op"), "{:?}", codes(&ds));
}

#[test]
fn canonical_roundtrip_text() {
    let src = fixture("valid", "t02-lifecycle.atom");
    let b = bind_text_ok(&src);
    let out = descriptor::write_canonical(&b, text_descriptor()).expect("canonical write");
    let re = descriptor::bind(text_descriptor(), &out).expect("canonical rebind");
    assert_eq!(
        re.modules, b.modules,
        "canonical roundtrip must be semantically equal"
    );
    assert_eq!(re.domain, HirDomain::Text);
    // writer refuses to re-shelve text under another identity
    let err = descriptor::write_canonical(&b, auto_ac_prototype::scalar_descriptor())
        .expect_err("identity laundering must be rejected");
    assert_eq!(err.code, "canonical.identity-conflict");
}

// -- resource plan contents (exact) -------------------------------------------

#[test]
fn lifecycle_resource_plan_is_exact() {
    let checked = verify_text_ok(&fixture("valid", "t02-lifecycle.atom"));
    let plan = checked.text_resources().expect("text plan present");
    // module 0 bodies: 0=play, 1=looper, 2=dropper, 3=main
    let play = &plan.bodies[&(0, 0)];
    // locals: 0=n(param) 1=a 2=b -> owned {1,2}
    assert_eq!(
        play.owned,
        [1, 2]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
    );
    // bk_then return (block 1 stmt 1): live {a,b} -> LIFO [2,1]
    assert_eq!(play.exit_frees[&(1, Some(1))], vec![2, 1]);
    // bk_play return (block 0 stmt 2): after the join only a is live
    assert_eq!(play.exit_frees[&(0, Some(2))], vec![1]);
    // no owned returns, no natural-end frees in play (then-arm returns)
    assert!(play.owned_returns.is_empty());
    assert!(!play.exit_frees.contains_key(&(0, None)));

    let looper = &plan.bodies[&(0, 1)];
    // locals: 0=n(param) 1=c -> owned {1}
    assert_eq!(
        looper.owned,
        [1].into_iter().collect::<std::collections::BTreeSet<_>>()
    );
    // loop body (block 1) natural end frees c; break (block 2 stmt 0) frees c
    assert_eq!(looper.exit_frees[&(1, None)], vec![1]);
    assert_eq!(looper.exit_frees[&(2, Some(0))], vec![1]);
    // final return has no owned live
    assert!(!looper.exit_frees.contains_key(&(0, Some(1))));

    let dropper = &plan.bodies[&(0, 2)];
    // locals: 0=s(param view) 1=d 2=n -> owned {1}; d is dropped at stmt 2
    assert_eq!(
        dropper.owned,
        [1].into_iter().collect::<std::collections::BTreeSet<_>>()
    );
    assert_eq!(dropper.exit_frees[&(0, Some(2))], vec![1]);
    // return after the drop frees nothing
    assert!(!dropper.exit_frees.contains_key(&(0, Some(3))));

    let main = &plan.bodies[&(0, 3)];
    assert!(main.owned.is_empty());
    assert!(main.exit_frees.is_empty());
}

#[test]
fn ops_resource_plan_is_exact() {
    let checked = verify_text_ok(&fixture("valid", "t01-text-ops.atom"));
    let plan = checked.text_resources().expect("text plan present");
    // bodies: 0=lens 1=make 2=char 3=consume 4=main
    let make = &plan.bodies[&(0, 1)];
    assert!(make.owned.is_empty()); // the literal is a temp, not a local
    assert_eq!(
        make.owned_returns,
        [(0, 0)]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
    );
    let consume = &plan.bodies[&(0, 3)];
    // move-in param l_o (local 0) is owned and freed by the callee
    assert_eq!(
        consume.owned,
        [0].into_iter().collect::<std::collections::BTreeSet<_>>()
    );
    assert_eq!(consume.exit_frees[&(0, Some(0))], vec![0]);
    let main = &plan.bodies[&(0, 4)];
    // l_t (local 0) is moved into consume at stmt 0 -> dead, no frees
    assert_eq!(
        main.owned,
        [0].into_iter().collect::<std::collections::BTreeSet<_>>()
    );
    assert!(!main.exit_frees.contains_key(&(0, Some(2))));
}

// -- V-03 static legs: char conversions and typing ----------------------------

#[test]
fn char_conversion_typing_matrix() {
    // from_codepoint on an i32 operand: valid (t03 b_fromcp)
    verify_text_ok(&fixture("valid", "t03-text-matrix.atom"));

    // from_codepoint on a char operand -> type-mismatch (mutate t03)
    let src = fixture("valid", "t03-text-matrix.atom");
    let mut b = bind_text_ok(&src);
    // b_fromcp is body 1: e_fcp operand e_rn4 (i32) -> swap to a char read
    // by mutating the operand expr's kind to char_lit (char-typed)
    let body = body_of_mut(&mut b, 1);
    if let ExprKind::CharFromCodepoint { operand } = &mut body.exprs[1].kind {
        assert_eq!(*operand, hir::ExprId(0));
    } else {
        panic!("e_fcp must be char_from_codepoint");
    }
    body.exprs[0].kind = ExprKind::CharLit { scalar: 97 };
    body.exprs[0].ty = hir::TypeUse::Builtin(hir::BuiltinType::Char);
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.type-mismatch"), "{:?}", codes(&ds));

    // char literal carrying an invalid scalar (surrogate) -> rejected
    let mut b = bind_text_ok(&src);
    let body = body_of_mut(&mut b, 1);
    body.exprs[0].kind = ExprKind::CharLit { scalar: 0xD800 };
    body.exprs[0].ty = hir::TypeUse::Builtin(hir::BuiltinType::Char);
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.type-mismatch"), "{:?}", codes(&ds));

    // binder rejects a surrogate scalar literal outright (t01's char lit)
    let bad = fixture("valid", "t01-text-ops.atom").replace("scalar: 65u32", "scalar: 0xD800u32");
    let ds = descriptor::bind(text_descriptor(), &bad).expect_err("surrogate rejected at bind");
    assert!(has(&ds, "bind.literal-range"), "{:?}", codes(&ds));
}

#[test]
fn text_eq_typing_matrix() {
    let src = fixture("valid", "t03-text-matrix.atom");
    // eq(str, Str) mixed -> type-mismatch: mutate chareq's lhs to a str read
    // (char_eq body 2, exprs 0..2)
    let mut b = bind_text_ok(&src);
    let body = body_of_mut(&mut b, 2);
    body.exprs[0].kind = ExprKind::StrLit {
        value: "x".to_string(),
    };
    body.exprs[0].ty = hir::TypeUse::Builtin(hir::BuiltinType::StrSlice);
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.type-mismatch"), "{:?}", codes(&ds));
}

// -- V-06: programmatic bundle mutation gates ---------------------------------

#[test]
fn invalid_fixture_borrow_escape() {
    // binds fine; the resource walk rejects returning a view of an owned
    // local (verify.borrow-escape) — no Checked module is produced
    let b = bind_text_ok(&fixture("invalid", "borrow-escape.atom"));
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.borrow-escape"), "{:?}", codes(&ds));
}

#[test]
fn mutation_move_marker_matrix() {
    let src = fixture("valid", "t01-text-ops.atom");
    // (a) strip the .move marker: move Str param bound without moved:true
    let mut b = bind_text_ok(&src);
    let body = body_of_mut(&mut b, 4);
    if let ExprKind::Call { bindings, .. } = &mut body.exprs[5].kind {
        bindings[0].moved = false;
    } else {
        panic!("e_call3 must be the consuming call");
    }
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));

    // (b) `.move` marker on a view-mode parameter: demote the callee's
    // move param to view (types unchanged, so only the mode rule fires)
    let mut b = bind_text_ok(&src);
    let body = body_of_mut(&mut b, 3);
    body.locals[0].param_mode = hir::ParamMode::View;
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));
}

#[test]
fn invalid_fixture_move_gates() {
    // double-move: the same local moved by two bindings of one call
    let b = bind_text_ok(&fixture("invalid", "double-move.atom"));
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.double-move"), "{:?}", codes(&ds));

    // mut-conflict: the same call views (to(str)) and moves one local
    let b = bind_text_ok(&fixture("invalid", "mut-conflict.atom"));
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.mut-conflict"), "{:?}", codes(&ds));

    // use-after-move: read of l_x after the consuming call
    let b = bind_text_ok(&fixture("invalid", "use-after-move.atom"));
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.use-after-move"), "{:?}", codes(&ds));
}

#[test]
fn mutation_owned_scope_and_assign_gates() {
    let src = fixture("valid", "t02-lifecycle.atom");
    // (a) owned-scope-escape: read l_b (let in bk_then) after the join.
    // t02 play: locals 0=n 1=a 2=b; bk_play stmts: let a, if, return(len a).
    let mut b = bind_text_ok(&src);
    {
        let body = body_of_mut(&mut b, 0);
        let n = body.exprs.len() as u32;
        body.exprs.push(hir::Expr {
            id_text: "e_bad_b".to_string(),
            ty: hir::TypeUse::Builtin(hir::BuiltinType::StrOwned),
            kind: ExprKind::ReadLocal {
                local: hir::LocalId(2),
            },
            span: body.span,
        });
        body.exprs.push(hir::Expr {
            id_text: "e_bad_b_len".to_string(),
            ty: hir::TypeUse::Builtin(hir::BuiltinType::I32),
            kind: ExprKind::TextLen {
                operand: hir::ExprId(n),
            },
            span: body.span,
        });
        if let hir::Stmt::Return { value } = &mut body.blocks[0].stmts[2] {
            *value = hir::ExprId(n + 1);
        }
    }
    let ds = verify_text_err(b);
    assert!(
        has(&ds, "verify.owned-scope-escape") || has(&ds, "verify.uninitialized-read"),
        "{:?}",
        codes(&ds)
    );

    // (b) resource-assign: `=` to the owned local a (needs a place; t02 has
    // none — build the place programmatically)
    let mut b = bind_text_ok(&src);
    {
        let body = body_of_mut(&mut b, 0);
        let n = body.exprs.len() as u32;
        body.exprs.push(hir::Expr {
            id_text: "e_newlit".to_string(),
            ty: hir::TypeUse::Builtin(hir::BuiltinType::StrOwned),
            kind: ExprKind::StrLit {
                value: "zz".to_string(),
            },
            span: body.span,
        });
        body.places.push(hir::Place {
            id_text: "p_a".to_string(),
            kind: hir::PlaceKind::Local,
            local: hir::LocalId(1),
            ty: hir::TypeUse::Builtin(hir::BuiltinType::StrOwned),
            access: hir::Access::Mutable,
            span: body.span,
        });
        body.blocks[0].stmts.insert(
            1,
            hir::Stmt::Assign {
                place: hir::PlaceId(0),
                value: hir::ExprId(n),
            },
        );
        // make a mutable so only the resource rule fires
        body.locals[1].mutable = true;
    }
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.resource-assign"), "{:?}", codes(&ds));

    // (c) implicit transfer: let y Str = <read of owned x>
    let mut b = bind_text_ok(&src);
    {
        let body = body_of_mut(&mut b, 0);
        let n = body.exprs.len() as u32;
        body.exprs.push(hir::Expr {
            id_text: "e_read_a".to_string(),
            ty: hir::TypeUse::Builtin(hir::BuiltinType::StrOwned),
            kind: ExprKind::ReadLocal {
                local: hir::LocalId(1),
            },
            span: body.span,
        });
        body.locals.push(hir::LocalDef {
            id_text: "l_y".to_string(),
            ty: hir::TypeUse::Builtin(hir::BuiltinType::StrOwned),
            name: "y".to_string(),
            role: hir::LocalRole::Binding,
            index: None,
            mutable: false,
            mutable_explicit: false,
            param_mode: hir::ParamMode::View,
            span: body.span,
        });
        body.blocks[0].stmts.insert(
            1,
            hir::Stmt::Let {
                local: hir::LocalId(3),
                value: hir::ExprId(n),
            },
        );
    }
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));
}

#[test]
fn mutation_drop_gates() {
    let src = fixture("valid", "t02-lifecycle.atom");
    // (a) drop of a non-owned local (the i32 param) — before the return
    let mut b = bind_text_ok(&src);
    {
        let body = body_of_mut(&mut b, 0);
        body.blocks[0].stmts.insert(
            2,
            hir::Stmt::Drop {
                local: hir::LocalId(0),
            },
        );
    }
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.resource-assign"), "{:?}", codes(&ds));

    // (b) double drop
    let mut b = bind_text_ok(&src);
    {
        let body = body_of_mut(&mut b, 2);
        body.blocks[0].stmts.insert(
            3,
            hir::Stmt::Drop {
                local: hir::LocalId(1),
            },
        );
    }
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.use-after-free"), "{:?}", codes(&ds));

    // (c) read after drop: dedicated fixture (mutation rewiring would
    // orphan exprs and mask the resource rule with eval-position)
    let b = bind_text_ok(&fixture("invalid", "read-after-drop.atom"));
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.use-after-free"), "{:?}", codes(&ds));
}

#[test]
fn mutation_bad_references_reach_controlled_rejections() {
    let src = fixture("valid", "t01-text-ops.atom");
    // out-of-range expr operand (text_len operand -> 99)
    let mut b = bind_text_ok(&src);
    let body = body_of_mut(&mut b, 0);
    if let ExprKind::TextLen { operand } = &mut body.exprs[1].kind {
        *operand = hir::ExprId(99);
    }
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.bad-reference"), "{:?}", codes(&ds));

    // signature result type mutation: t_lens result -> Str makes the return
    // type mismatch (resource walk must also accept the borrowed-return rule)
    let mut b = bind_text_ok(&src);
    if let hir::TypeKind::Function(sig) = &mut b.modules[0].types[0].kind {
        sig.result = hir::TypeUse::Builtin(hir::BuiltinType::StrOwned);
    }
    let ds = verify_text_err(b);
    assert!(has(&ds, "verify.type-mismatch"), "{:?}", codes(&ds));
}
