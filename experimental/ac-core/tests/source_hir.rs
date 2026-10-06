//! PLAN-745 T-03 source→HIR tests: the full adapter pipeline
//! (parse → resolve → typecheck → Atom emission → bind → span re-anchor →
//! verify) over the materialized corpus fixtures.
//!
//! Positive cases must produce a `CheckedModule`; negative cases must be
//! rejected with the corpus-pinned (stage, code, token) diagnostic present.
//! Mutation cases tamper with the generated Atom text (tests only) and must
//! be rejected by the real binder/verifier — a source `checked` marker or a
//! hand-edited bundle cannot reach native lowering.

use auto_ac_prototype::atom_text::{line_col, Stage};
use auto_ac_prototype::source::{compile_source, CompiledSource, TRACE_CAPABILITY};

fn fixture(kind: &str, name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/source")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn compile(
    src: &str,
    caps: &[&str],
) -> Result<CompiledSource, Vec<auto_ac_prototype::atom_text::Diagnostic>> {
    let caps: Vec<String> = caps.iter().map(|s| s.to_string()).collect();
    compile_source(src, "test.at", &caps)
}

const TRACE: &[&str] = &[TRACE_CAPABILITY];
const NO_CAPS: &[&str] = &[];

const VALID_PLAIN: &[&str] = &[
    "p01-precedence.at",
    "p02-signed-positional.at",
    "p03-bool-param-constant-encoding.at",
    "p04-while-retest.at",
    "p05-while-zero.at",
    "p06-continue-retests-condition.at",
    "p07-break-nearest-loop.at",
    "p08-recursion.at",
    "p09-forward-signature.at",
    "p10-shadow-rhs-outer.at",
    "p11-literal-min-max.at",
    "p12-unexecuted-trap-not-lifted-x1.at",
    "p13-add-overflow-trap.at",
    "p14-mul-overflow-trap.at",
    "p17-heterogeneous-binding-map.at",
    "p18-bool-return-local-branch.at",
    "p20-utf8-crlf-negative-min.at",
    "p21-mutual-recursion.at",
];

const VALID_TRACE: &[&str] = &[
    "p15-named-eval-order-x2.at",
    "p16-observable-before-trap-x9.at",
    "p19-user-function-not-intrinsic.at",
];

/// (fixture, stage, code, token, needs-trace-capability)
const NEGATIVE: &[(&str, &str, &str, &str, bool)] = &[
    (
        "n03-undefined-name.at",
        "bind",
        "source.undefined",
        "missing",
        false,
    ),
    (
        "n04-type-mismatch-return.at",
        "verify",
        "source.type-mismatch",
        "true",
        false,
    ),
    (
        "n05-type-mismatch-if-cond.at",
        "verify",
        "source.type-mismatch",
        "1",
        false,
    ),
    (
        "n06-type-mismatch-bool-add.at",
        "verify",
        "source.type-mismatch",
        "true",
        false,
    ),
    (
        "n07-immutable-write.at",
        "verify",
        "verify.immutable-write",
        "x = 2",
        false,
    ),
    (
        "n08-duplicate-local.at",
        "bind",
        "source.duplicate",
        "x = 2",
        false,
    ),
    (
        "n09-undefined-scope-escape.at",
        "bind",
        "source.undefined",
        // r1 review QA-03: the diagnostic anchors the NAME, not the
        // enclosing statement (precision per §5.3/AC-04; supersedes the
        // corpus draft's "return x" oracle).
        "x",
        false,
    ),
    (
        "n10-arg-missing.at",
        "bind",
        "source.argument",
        "f()",
        false,
    ),
    (
        "n11-arg-duplicate.at",
        "bind",
        "source.argument",
        "x: 2",
        false,
    ),
    (
        "n12-arg-unknown-name.at",
        "bind",
        "source.argument",
        "y",
        false,
    ),
    ("n13-arg-mixed.at", "bind", "source.argument", "y: 2", false),
    (
        "n14-type-mismatch-arg.at",
        "verify",
        "source.type-mismatch",
        "true",
        false,
    ),
    (
        "n15-break-outside-loop.at",
        "verify",
        "verify.loop-target-scope",
        "break",
        false,
    ),
    (
        "n16-continue-outside-loop.at",
        "verify",
        "verify.loop-target-scope",
        "continue",
        false,
    ),
    (
        "n17-no-return-path.at",
        "verify",
        "verify.no-return-path",
        "main",
        false,
    ),
    (
        "n18-stmt-after-return.at",
        "verify",
        "verify.block-structure",
        "return 1",
        false,
    ),
    (
        "n27-not-callable-local.at",
        "bind",
        "source.not-callable",
        "f(1)",
        false,
    ),
    (
        "n28-not-callable-param.at",
        "bind",
        "source.not-callable",
        "mark_a()",
        true,
    ),
    (
        "n29-not-callable-shadow.at",
        "bind",
        "source.not-callable",
        "mark_b()",
        true,
    ),
    (
        "n32-duplicate-fn.at",
        "bind",
        "source.duplicate",
        "fn f() int { return 1 }",
        false,
    ),
    (
        "n33-duplicate-param.at",
        "bind",
        "source.duplicate",
        "x int, x int",
        false,
    ),
    (
        "n50-undefined-callee.at",
        "bind",
        "source.undefined",
        "absent",
        false,
    ),
    (
        "n51-type-mismatch-lt-bool.at",
        "verify",
        "source.type-mismatch",
        "false",
        false,
    ),
    // review r1 QA-03: dead statements after a both-arms-returning if are
    // anchored AT the illegal statement
    (
        "c09-both-return-trailing.at",
        "verify",
        "verify.block-structure",
        "let x = 2",
        false,
    ),
    (
        "c10-scope-escape-token.at",
        "bind",
        "source.undefined",
        "x",
        false,
    ),
];

#[test]
fn valid_corpus_compiles_to_checked() {
    for name in VALID_PLAIN {
        let src = fixture("valid", name);
        match compile(&src, NO_CAPS) {
            Ok(_) => {}
            Err(ds) => panic!(
                "{}: unexpected rejection:\n{}",
                name,
                render_all(name, &src, &ds)
            ),
        }
    }
    for name in VALID_TRACE {
        let src = fixture("valid", name);
        match compile(&src, TRACE) {
            Ok(_) => {}
            Err(ds) => panic!(
                "{}: unexpected rejection:\n{}",
                name,
                render_all(name, &src, &ds)
            ),
        }
    }
}

#[test]
fn negative_corpus_rejected_with_pinned_diagnostics() {
    for (name, stage, code, token, trace) in NEGATIVE {
        let src = fixture("invalid", name);
        let caps: &[&str] = if *trace { TRACE } else { NO_CAPS };
        let ds = match compile(&src, caps) {
            Ok(_) => panic!("{}: expected rejection, got a Checked module", name),
            Err(ds) => ds,
        };
        let hit = ds
            .iter()
            .any(|d| d.stage.as_str() == *stage && d.code == *code && d.span.slice(&src) == *token);
        assert!(
            hit,
            "{}: no diagnostic with stage={} code={} token=`{}`; got:\n{}",
            name,
            stage,
            code,
            token,
            render_all(name, &src, &ds)
        );
    }
}

#[test]
fn n03_diagnostic_coordinates() {
    let src = fixture("invalid", "n03-undefined-name.at");
    let ds = compile(&src, NO_CAPS).unwrap_err();
    let d = ds
        .iter()
        .find(|d| d.code == "source.undefined")
        .expect("undefined diagnostic");
    assert_eq!(d.stage, Stage::Bind);
    assert_eq!(line_col(&src, d.span.start), (2, 24));
}

// -- structure spot checks ---------------------------------------------------

#[test]
fn p15_eval_order_and_binding_map() {
    let src = fixture("valid", "p15-named-eval-order-x2.at");
    let c = compile(&src, TRACE).unwrap();
    assert_eq!(
        c.checked.required_capabilities(),
        &[TRACE_CAPABILITY.to_string()]
    );
    let b = &c.checked.bundle().modules[0].bodies[1]; // main's body
    let ret = match &b.blocks[0].stmts[0] {
        auto_ac_prototype::hir::Stmt::Return { value } => *value,
        _ => panic!("expected return"),
    };
    let e = &b.exprs[ret.0 as usize];
    let auto_ac_prototype::hir::ExprKind::Call {
        callee,
        eval_args,
        bindings,
    } = &e.kind
    else {
        panic!("expected call");
    };
    // callee is the user `pair` function; args evaluated b then a (source
    // order) and bound through the name map, not by position.
    assert_eq!(b.exprs.len(), 3);
    assert_eq!(eval_args.len(), 2);
    let arg0 = &b.exprs[eval_args[0].0 as usize];
    let arg1 = &b.exprs[eval_args[1].0 as usize];
    let auto_ac_prototype::hir::ExprKind::Call { callee: c0, .. } = &arg0.kind else {
        panic!("arg0 is a call");
    };
    let auto_ac_prototype::hir::ExprKind::Call { callee: c1, .. } = &arg1.kind else {
        panic!("arg1 is a call");
    };
    let d0 = &c.checked.bundle().modules[0].defs[c0.0 as usize];
    let d1 = &c.checked.bundle().modules[0].defs[c1.0 as usize];
    assert_eq!(d0.name_for_diagnostic(), "hir.test.mark_b");
    assert_eq!(d1.name_for_diagnostic(), "hir.test.mark_a");
    // param 0 (a) ← arg 1 (mark_a); param 1 (b) ← arg 0 (mark_b); the map
    // is listed in argument order, so arg 0's binding comes first.
    assert_eq!(bindings.len(), 2);
    assert!(bindings.contains(&auto_ac_prototype::hir::CallBinding { param: 0, arg: 1 }));
    assert!(bindings.contains(&auto_ac_prototype::hir::CallBinding { param: 1, arg: 0 }));
    let _ = callee;
}

#[test]
fn p04_while_lowering_shape() {
    let src = fixture("valid", "p04-while-retest.at");
    let c = compile(&src, NO_CAPS).unwrap();
    let b = &c.checked.bundle().modules[0].bodies[0];
    // blocks: entry, while body, stop, re-test
    assert_eq!(b.blocks.len(), 4);
    assert_eq!(b.blocks[0].stmts.len(), 3); // let, loop, return
    let (l_loop, k_test) = match &b.blocks[0].stmts[1] {
        auto_ac_prototype::hir::Stmt::Loop { id, body } => (*id, *body),
        _ => panic!("expected loop"),
    };
    // the loop body is the re-test if; the cond is owned by that if, so it
    // re-evaluates every iteration (fall-through repeats the loop body)
    match &b.blocks[k_test.0 as usize].stmts[0] {
        auto_ac_prototype::hir::Stmt::If { cond, then, els } => {
            let e = &b.exprs[cond.0 as usize];
            let auto_ac_prototype::hir::ExprKind::Binary { op, .. } = &e.kind else {
                panic!("cond is a binary");
            };
            assert_eq!(op.as_str(), "lt_i32");
            // the while body block holds exactly the source assignment
            let body = &b.blocks[then.0 as usize].stmts;
            assert_eq!(body.len(), 1);
            assert!(matches!(
                body[0],
                auto_ac_prototype::hir::Stmt::Assign { .. }
            ));
            // the else arm breaks the same loop: zero iterations possible
            match &b.blocks[els.unwrap().0 as usize].stmts[0] {
                auto_ac_prototype::hir::Stmt::Break { target } => assert_eq!(*target, l_loop),
                _ => panic!("expected break"),
            }
        }
        _ => panic!("expected if in loop body"),
    }
}

#[test]
fn p03_bool_literal_encoding() {
    let src = fixture("valid", "p03-bool-param-constant-encoding.at");
    let c = compile(&src, NO_CAPS).unwrap();
    // choose's body: params c(bool); the call argument `true` becomes a
    // fresh 0<1 subgraph of type bool.
    let m = &c.checked.bundle().modules[0];
    let b = &m.bodies[1]; // main
    let ret = match &b.blocks[0].stmts[0] {
        auto_ac_prototype::hir::Stmt::Return { value } => *value,
        _ => panic!("return"),
    };
    let call = &b.exprs[ret.0 as usize];
    let auto_ac_prototype::hir::ExprKind::Call { eval_args, .. } = &call.kind else {
        panic!("call");
    };
    let bool_expr = &b.exprs[eval_args[0].0 as usize];
    let auto_ac_prototype::hir::ExprKind::Binary { op, lhs, rhs, .. } = &bool_expr.kind else {
        panic!("bool literal must encode as a comparison subgraph");
    };
    assert_eq!(op.as_str(), "lt_i32");
    let (l, r) = (&b.exprs[lhs.0 as usize], &b.exprs[rhs.0 as usize]);
    let auto_ac_prototype::hir::ExprKind::Constant { value: a } = &l.kind else {
        panic!("lhs const");
    };
    let auto_ac_prototype::hir::ExprKind::Constant { value: bb } = &r.kind else {
        panic!("rhs const");
    };
    assert_eq!((*a, *bb), (0, 1)); // true = 0 < 1
                                   // spans of the synthesized subgraph point at the literal token
    assert_eq!(bool_expr.span.slice(&src), "true");
    assert_eq!(l.span.slice(&src), "true");
    assert_eq!(r.span.slice(&src), "true");
}

#[test]
fn p11_literal_domain_in_hir() {
    let src = fixture("valid", "p11-literal-min-max.at");
    let c = compile(&src, NO_CAPS).unwrap();
    let b = &c.checked.bundle().modules[0].bodies[0];
    let mut consts = Vec::new();
    for e in &b.exprs {
        if let auto_ac_prototype::hir::ExprKind::Constant { value } = &e.kind {
            consts.push(*value);
        }
    }
    assert!(consts.contains(&-2147483648), "consts: {:?}", consts);
    assert!(consts.contains(&2147483647), "consts: {:?}", consts);
}

#[test]
fn p19_user_mark_wins_over_intrinsic() {
    let src = fixture("valid", "p19-user-function-not-intrinsic.at");
    let c = compile(&src, TRACE).unwrap();
    // No trace capability required: the user's mark_a is an ordinary
    // function and no intrinsic declaration exists.
    assert!(c.checked.required_capabilities().is_empty());
    let m = &c.checked.bundle().modules[0];
    assert!(m
        .defs
        .iter()
        .all(|d| !matches!(d.kind, auto_ac_prototype::hir::DefKind::Intrinsic { .. })));
    // the user function wins; its native symbol carries the sf_ prefix
    // (QA-02 namespace isolation)
    assert!(m.defs.iter().any(|d| d.name_for_diagnostic() == "sf_mark_a"
        && matches!(d.kind, auto_ac_prototype::hir::DefKind::Function { .. })));
}

#[test]
fn marks_without_capability_are_undefined() {
    // §5.4: without the trace capability, a mark call is source.undefined.
    let src = fixture("valid", "p16-observable-before-trap-x9.at");
    let ds = compile(&src, NO_CAPS).unwrap_err();
    assert!(ds.iter().any(|d| d.stage == Stage::Bind
        && d.code == "source.undefined"
        && d.span.slice(&src) == "mark_a"));
}

// -- mutation: tampered Atom text never reaches lowering --------------------

#[test]
fn tampered_atom_is_rejected() {
    let src = fixture("valid", "p04-while-retest.at");
    let c = compile(&src, NO_CAPS).unwrap();
    let atom = &c.atom_text;
    assert!(
        atom.contains("let(local: l0, value: e0) {}"),
        "emit shape changed; atom:\n{}",
        atom
    );

    // 1. deleting a let statement leaves a dead expression
    let mutated = atom.replacen("let(local: l0, value: e0) {}", "", 1);
    assert_bind_verify_rejects(&mutated, "dead expr");

    // 2. changing a constant's declared type corrupts the type table —
    // the mutation MUST actually happen (QA-06: no conditional skip)
    let mutated = atom.replace(
        "expr e0 constant { type: t_i32;",
        "expr e0 constant { type: t_bool;",
    );
    assert_ne!(&mutated, atom, "type mutation did not change the input");
    assert_bind_verify_rejects(&mutated, "const type");

    // 3. a dangling expression reference
    let mutated = atom.replacen("value: e0", "value: e999", 1);
    assert_bind_verify_rejects(&mutated, "dangling ref");

    // 4. a fabricated `checked` marker is an unknown field
    let mutated = atom.replacen("hir(schema:", "hir(checked: true, schema:", 1);
    assert_bind_verify_rejects(&mutated, "fake checked marker");
}

fn assert_bind_verify_rejects(atom: &str, what: &str) {
    let bound = auto_ac_prototype::bind_source(atom);
    let bundle = match bound {
        Ok(b) => b,
        Err(_) => return, // rejected at bind: fine
    };
    match auto_ac_prototype::verify::verify(bundle) {
        Ok(_) => panic!("tampered Atom ({}) passed bind+verify", what),
        Err(_) => {}
    }
}

fn render_all(name: &str, src: &str, ds: &[auto_ac_prototype::atom_text::Diagnostic]) -> String {
    ds.iter()
        .map(|d| d.render(name, src))
        .collect::<Vec<_>>()
        .join("\n")
}

/// QA-06: real HIR structure oracle for the runtime-trap cases — the
/// add/mul expressions must carry `overflow: trap` (no compile-time
/// folding: operands stay separate constant exprs).
#[test]
fn overflow_trap_structure_in_hir() {
    for (name, want_op) in [
        ("p13-add-overflow-trap.at", "add_i32"),
        ("p14-mul-overflow-trap.at", "mul_i32"),
    ] {
        let src = fixture("valid", name);
        let c = compile(&src, NO_CAPS).unwrap();
        let b = &c.checked.bundle().modules[0].bodies[0];
        let mut traps = 0;
        for e in &b.exprs {
            if let auto_ac_prototype::hir::ExprKind::Binary { op, overflow, .. } = &e.kind {
                assert_eq!(op.as_str(), want_op, "{}: op", name);
                assert_eq!(
                    format!("{:?}", overflow),
                    "Trap",
                    "{}: overflow policy must be trap",
                    name
                );
                traps += 1;
            }
        }
        assert!(traps > 0, "{}: no binary expr found", name);
    }
}

/// QA-06: the §6 Bundle mutation matrix — tamper the PROJECTED bundle (not
/// the Atom text), re-run the real verifier, and require rejection matched
/// to the invariant; every mutation is asserted to actually change input.
#[test]
fn tampered_bundle_is_rejected_by_real_verify() {
    use auto_ac_prototype::hir::{BuiltinType, ExprKind, Stmt, TypeUse};
    use auto_ac_prototype::source::compile_parts;

    let src = fixture("valid", "p04-while-retest.at");
    let caps: Vec<String> = Vec::new();
    let parts = compile_parts(&src, "test.at", &caps).unwrap();
    // baseline: the untouched projected bundle verifies OK
    assert!(auto_ac_prototype::verify::verify(parts.bundle.clone()).is_ok());

    // (1) delete the `let` statement → its value expr loses its only owning
    // position → verify.eval-position
    let mut b1 = parts.bundle.clone();
    let stmts = &mut b1.modules[0].bodies[0].blocks[0].stmts;
    assert!(
        matches!(stmts[0], Stmt::Let { .. }),
        "first entry statement must be the let being deleted"
    );
    stmts.remove(0);
    let ds = auto_ac_prototype::verify::verify(b1).unwrap_err();
    assert!(
        ds.iter().any(|d| d.code == "verify.eval-position"),
        "delete-let: expected eval-position, got: {:?}",
        ds.iter().map(|d| d.code.clone()).collect::<Vec<_>>()
    );

    // (2) wrong declared type on a read_local expr (i32 local declared as
    // builtin bool) → verify.type-mismatch
    let mut b2 = parts.bundle.clone();
    let exprs = &mut b2.modules[0].bodies[0].exprs;
    let read = exprs
        .iter_mut()
        .find(|e| matches!(e.kind, ExprKind::ReadLocal { .. }))
        .expect("a read_local expr");
    let before = read.ty;
    read.ty = TypeUse::Builtin(BuiltinType::Bool);
    assert_ne!(read.ty, before, "type mutation must change the input");
    let ds = auto_ac_prototype::verify::verify(b2).unwrap_err();
    assert!(
        ds.iter().any(|d| d.code == "verify.type-mismatch"),
        "wrong-expr-type: expected type-mismatch, got: {:?}",
        ds.iter().map(|d| d.code.clone()).collect::<Vec<_>>()
    );

    // (3) redirect a read to the WRONG local (swap a read of the int
    // param for the bool param) → the read/type consistency check fires.
    // p17's `choose(n int, flag bool)` has both types in one body.
    let src17 = fixture("valid", "p17-heterogeneous-binding-map.at");
    let parts17 = compile_parts(&src17, "test.at", &caps).unwrap();
    let mut b3 = parts17.bundle.clone();
    // locate an i32 and a bool local through the module's type table
    // (the adapter emits TypeUse::Ref ids)
    let (i32_local, bool_local) = {
        let m = &b3.modules[0];
        let mut i32l = None;
        let mut booll = None;
        for (i, l) in m.bodies[0].locals.iter().enumerate() {
            let vt = match l.ty {
                auto_ac_prototype::hir::TypeUse::Builtin(
                    auto_ac_prototype::hir::BuiltinType::I32,
                ) => "i32",
                auto_ac_prototype::hir::TypeUse::Builtin(
                    auto_ac_prototype::hir::BuiltinType::Bool,
                ) => "bool",
                auto_ac_prototype::hir::TypeUse::Ref(t) => match &m.types[t.0 as usize].kind {
                    auto_ac_prototype::hir::TypeKind::I32 => "i32",
                    auto_ac_prototype::hir::TypeKind::Bool => "bool",
                    _ => "fn",
                },
            };
            if vt == "i32" && i32l.is_none() {
                i32l = Some(i);
            }
            if vt == "bool" && booll.is_none() {
                booll = Some(i);
            }
        }
        (i32l.expect("i32 local"), booll.expect("bool local"))
    };
    let body = &mut b3.modules[0].bodies[0];
    let read = body
        .exprs
        .iter_mut()
        .find(|e| matches!(e.kind, ExprKind::ReadLocal { local } if local.0 as usize == i32_local))
        .expect("read of the i32 local");
    let ExprKind::ReadLocal { local } = &mut read.kind else {
        unreachable!()
    };
    *local = auto_ac_prototype::hir::LocalId(bool_local as u32);
    let ds = auto_ac_prototype::verify::verify(b3).unwrap_err();
    assert!(
        ds.iter().any(|d| d.code == "verify.type-mismatch"),
        "wrong-local-ref: expected type-mismatch, got: {:?}",
        ds.iter().map(|d| d.code.clone()).collect::<Vec<_>>()
    );
}
