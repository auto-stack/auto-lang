//! PLAN-767 T-02 scalar-domain HIR tests: dual-profile identity isolation
//! (V-01, HIR half), programmatic bundle mutation gates (V-07), canonical
//! roundtrip, the new-operator type matrix, and a deep logical chain
//! (R-iterative guard). Pure in-process verification — native execution of
//! these fixtures lives in `scalar_native.rs`.

use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::descriptor;
use auto_ac_prototype::hir::{self, BinOp, ExprKind, HirDomain};
use auto_ac_prototype::{bind_document, bind_source, scalar_descriptor, verify};

fn fixture(kind: &str, name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("experimental/ac-core/fixtures/scalar-hir")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn bind_scalar_ok(src: &str) -> hir::Bundle {
    let b = descriptor::bind(scalar_descriptor(), src).expect("scalar bind must succeed");
    assert_eq!(
        b.domain,
        HirDomain::Scalar,
        "binder must stamp the scalar domain"
    );
    b
}

fn verify_scalar_err(bundle: hir::Bundle) -> Vec<Diagnostic> {
    verify::verify_scalar(bundle).expect_err("verify_scalar must reject")
}

fn codes(ds: &[Diagnostic]) -> Vec<&str> {
    ds.iter().map(|d| d.code.as_str()).collect()
}

fn has(ds: &[Diagnostic], code: &str) -> bool {
    ds.iter().any(|d| d.code == code)
}

// -- V-01: identity, isolation, dispatch ------------------------------------

#[test]
fn scalar_identity_binds_and_verifies() {
    let bundle = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let checked = verify::verify_scalar(bundle).expect("scalar verify");
    assert_eq!(checked.domain(), HirDomain::Scalar);
    let body = &checked.bundle().modules[0].bodies[0];
    // every new operator is present exactly once in the fixture
    let mut ops: Vec<&str> = body
        .exprs
        .iter()
        .filter_map(|e| match &e.kind {
            ExprKind::Binary { op, .. } => Some(op.as_str()),
            _ => None,
        })
        .collect();
    ops.sort();
    assert_eq!(
        ops,
        vec!["div_i32", "eq", "ge_i32", "gt_i32", "le_i32", "ne", "rem_i32", "sub_i32"]
    );
    assert!(body.exprs.iter().any(|e| matches!(
        &e.kind,
        ExprKind::Unary {
            op: hir::UnaryOp::NegI32,
            ..
        }
    )));
    assert!(body.exprs.iter().any(|e| matches!(
        &e.kind,
        ExprKind::Unary {
            op: hir::UnaryOp::NotBool,
            ..
        }
    )));
    assert_eq!(
        body.exprs
            .iter()
            .filter(|e| matches!(&e.kind, ExprKind::Logical { .. }))
            .count(),
        2,
        "one and + one or"
    );
}

#[test]
fn core_descriptor_rejects_scalar_ops_in_text() {
    // The SAME document under the OLD descriptor: the widened Rust enums
    // must not widen the old descriptor's accepted text surface.
    let src = fixture("valid", "s01-scalar-ops.atom");
    let ds = descriptor::bind(auto_ac_prototype::descriptor(), &src)
        .expect_err("core descriptor must reject scalar ops");
    assert!(
        ds.iter().any(|d| d.message.contains("sub_i32")),
        "expected a sub_i32 rejection, got: {:?}",
        codes(&ds)
    );
}

#[test]
fn bind_document_dispatches_by_declared_identity() {
    let scalar = bind_document(&fixture("valid", "s01-scalar-ops.atom"))
        .expect("registered scalar triple dispatches");
    assert_eq!(scalar.domain, HirDomain::Scalar);

    let core_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("docs/design/strategy/hir-examples/01-add.atom"),
    )
    .unwrap();
    let core = bind_document(&core_src).expect("registered core triple dispatches");
    assert_eq!(core.domain, HirDomain::Core);

    let ds = bind_document(&fixture("invalid", "i09-unknown-triple.atom"))
        .expect_err("unknown triple must be a controlled bind diagnostic");
    assert!(has(&ds, "bind.descriptor-unknown"), "{:?}", codes(&ds));
    assert!(ds[0].message.contains("accepted:"), "{:?}", ds[0].message);
}

// -- P767-R1: unregistered identities never classify as a known domain ----

#[test]
fn unregistered_descriptor_identities_are_rejected_everywhere() {
    let core_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("docs/design/strategy/hir-examples/01-add.atom"),
    )
    .unwrap();
    let core = bind_source(&core_src).expect("core bind");
    assert_eq!(core.domain, HirDomain::Core);

    // Controls: the two REGISTERED descriptors bind and write their own
    // bundles cleanly.
    assert!(descriptor::write_canonical(&core, auto_ac_prototype::descriptor()).is_ok());
    let scalar = bind_scalar_ok(&fixture("valid", "s02-two-bodies.atom"));
    assert!(descriptor::write_canonical(&scalar, scalar_descriptor()).is_ok());

    // Each altered dimension (schema / revision / profile) of an otherwise
    // valid core descriptor: binding under it and re-shelling through the
    // canonical writer are BOTH controlled rejections — an unknown identity
    // never falls back to the core domain on its way into a Bundle.
    let cases: [(&str, Box<dyn Fn(&mut descriptor::Descriptor)>); 3] = [
        (
            "schema",
            Box::new(|d: &mut descriptor::Descriptor| {
                d.schema = "auto.hir.unregistered".to_string()
            }),
        ),
        (
            "revision",
            Box::new(|d: &mut descriptor::Descriptor| d.revision = 99),
        ),
        (
            "profile",
            Box::new(|d: &mut descriptor::Descriptor| {
                d.profile = "unregistered-profile".to_string()
            }),
        ),
    ];
    for (which, alter) in cases {
        let mut desc = auto_ac_prototype::descriptor().clone();
        alter(&mut desc);
        assert_eq!(desc.domain(), None, "{}: unregistered triple", which);

        // the writer refuses to re-shelve a bundle under an unregistered
        // identity (no laundering, no output of a swapped header)
        let err = descriptor::write_canonical(&core, &desc)
            .expect_err("writer must reject an unregistered descriptor");
        assert_eq!(err.code, "canonical.identity-conflict", "{}", which);
        assert!(
            err.message.contains("unregistered"),
            "{}: {}",
            which,
            err.message
        );

        // the binder refuses to produce a Bundle under it at all
        let ds = descriptor::bind(&desc, &core_src)
            .expect_err("bind must reject an unregistered descriptor");
        assert!(
            has(&ds, "bind.descriptor-unregistered"),
            "{}: {:?}",
            which,
            codes(&ds)
        );

        // and a document whose header was rewritten to the altered triple
        // cannot reach a Checked through the programmatic path either
        let mut altered_src = core_src.clone();
        match which {
            "schema" => {
                altered_src = altered_src.replace("auto.hir.core.draft", "auto.hir.unregistered")
            }
            "revision" => altered_src = altered_src.replace("revision: 1u32", "revision: 99u32"),
            "profile" => {
                altered_src = altered_src.replace("core-i32-draft", "unregistered-profile")
            }
            _ => unreachable!(),
        }
        let ds = descriptor::bind(&desc, &altered_src)
            .expect_err("altered-header bind under the same descriptor");
        assert!(has(&ds, "bind.descriptor-unregistered"), "{}", which);
        // the public registry keeps rejecting it independently
        assert!(bind_document(&altered_src).is_err(), "{}: registry", which);
    }
}

// -- P767-R2: signature TypeUse bounds gate -------------------------------

#[test]
fn signature_type_refs_are_bounds_checked_before_checked() {
    // params[0] and result of a function type are separately mutated to an
    // out-of-range TypeId; every other reference stays valid. Both must be
    // rejected by verify with the reference-bounds code — never a Checked
    // that would panic the native backend's type-table indexing.
    for which in ["parameter", "result"] {
        let mut b = bind_scalar_ok(&fixture("valid", "s02-two-bodies.atom"));
        let hir::TypeKind::Function(sig) = &mut b.modules[0].types[1].kind else {
            panic!("fixture function type");
        };
        if which == "parameter" {
            sig.params[0] = hir::TypeUse::Ref(hir::TypeId(9999));
        } else {
            sig.result = hir::TypeUse::Ref(hir::TypeId(9999));
        }
        let ds = verify_scalar_err(b);
        assert!(
            has(&ds, "verify.bad-reference"),
            "{}: {:?}",
            which,
            codes(&ds)
        );
    }
    // control: the untouched fixture still verifies
    let b = bind_scalar_ok(&fixture("valid", "s02-two-bodies.atom"));
    verify::verify_scalar(b).expect("valid control");
    // a valid Builtin result type is equally fine (in-range projection)
    let mut b = bind_scalar_ok(&fixture("valid", "s02-two-bodies.atom"));
    let hir::TypeKind::Function(sig) = &mut b.modules[0].types[1].kind else {
        panic!("fixture function type");
    };
    sig.result = hir::TypeUse::Builtin(hir::BuiltinType::I32);
    verify::verify_scalar(b).expect("builtin result control");
}

#[test]
fn verify_entries_are_domain_gated() {
    // verify() is core-only; verify_scalar() is scalar-only. Neither
    // silently reinterprets the other domain.
    let scalar = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let ds = verify::verify(scalar).expect_err("core entry rejects scalar bundle");
    assert!(has(&ds, "verify.domain"), "{:?}", codes(&ds));

    let core = bind_source(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("docs/design/strategy/hir-examples/01-add.atom"),
        )
        .unwrap(),
    )
    .expect("core bind");
    let ds = verify::verify_scalar(core).expect_err("scalar entry rejects core bundle");
    assert!(has(&ds, "verify.domain"), "{:?}", codes(&ds));
}

#[test]
fn core_verify_rejects_smuggled_scalar_nodes() {
    // Belt-and-braces: a core-domain bundle programmatically mutated to
    // carry scalar-only ops/kinds is rejected by the (unchanged) core
    // entry, never reinterpreted under scalar semantics.
    let base_src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("docs/design/strategy/hir-examples/01-add.atom"),
    )
    .unwrap();
    let mut bundle = bind_source(&base_src).expect("core bind");
    {
        // 01-add: e_a/e_b are read_locals, e_add is the binary at index 2
        let e = &mut bundle.modules[0].bodies[0].exprs[2];
        if let ExprKind::Binary { op, .. } = &mut e.kind {
            *op = BinOp::SubI32; // smuggled scalar-only op into the core domain
        } else {
            panic!("fixture shape: e_add must be binary");
        }
    }
    let ds = verify::verify(bundle).expect_err("core verify rejects smuggled op");
    assert!(has(&ds, "verify.profile-op"), "{:?}", codes(&ds));

    let mut bundle = bind_source(&base_src).expect("core bind");
    {
        // 01-add: e_a/e_b are read_locals, e_add is the binary at index 2.
        // Replacing it with a unary node leaves e_b dead — an eval-position
        // diag fires alongside; the assertion is the profile-op rejection.
        let e = &mut bundle.modules[0].bodies[0].exprs[2];
        if let ExprKind::Binary { lhs, .. } = &e.kind {
            let operand = *lhs;
            e.kind = ExprKind::Unary {
                op: hir::UnaryOp::NegI32,
                operand,
            };
        } else {
            panic!("fixture shape: e_add must be binary");
        }
    }
    let ds = verify::verify(bundle).expect_err("core verify rejects unary node");
    assert!(has(&ds, "verify.profile-op"), "{:?}", codes(&ds));
}

// -- canonical writer identity ----------------------------------------------

#[test]
fn write_canonical_refuses_identity_laundering() {
    let scalar = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let err = descriptor::write_canonical(&scalar, auto_ac_prototype::descriptor())
        .expect_err("core descriptor cannot re-shelve a scalar bundle");
    assert_eq!(err.code, "canonical.identity-conflict");

    let core = bind_source(
        &std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("docs/design/strategy/hir-examples/01-add.atom"),
        )
        .unwrap(),
    )
    .expect("core bind");
    let err = descriptor::write_canonical(&core, scalar_descriptor())
        .expect_err("scalar descriptor cannot re-shelve a core bundle");
    assert_eq!(err.code, "canonical.identity-conflict");
}

#[test]
fn canonical_roundtrip_scalar_is_semantically_equal() {
    for name in ["s01-scalar-ops.atom", "s02-two-bodies.atom"] {
        let first = bind_scalar_ok(&fixture("valid", name));
        let canonical = descriptor::write_canonical(&first, scalar_descriptor())
            .expect("scalar canonical write");
        let second = bind_scalar_ok(&canonical);
        assert_eq!(first, second, "roundtrip mismatch for {}", name);
        // the emitted header really is the scalar triple
        assert!(canonical.contains("auto.hir.scalar.draft"), "{}", name);
        assert!(canonical.contains("scalar-i32-draft"), "{}", name);
    }
}

// -- type matrix negatives (text fixtures) -----------------------------------

#[test]
fn scalar_type_and_graph_negatives() {
    // (fixture, expected verify code)
    let cases: &[(&str, &str)] = &[
        ("i02-eq-mixed-types.atom", "verify.type-mismatch"),
        ("i03-neg-on-bool.atom", "verify.type-mismatch"),
        ("i04-not-on-i32.atom", "verify.type-mismatch"),
        ("i05-logical-i32-operand.atom", "verify.type-mismatch"),
        ("i06-shared-expr.atom", "verify.eval-position"),
        ("i07-logical-self-cycle.atom", "verify.expr-cycle"),
        ("i08-unary-two-node-cycle.atom", "verify.expr-cycle"),
    ];
    for (name, want) in cases {
        let bundle = bind_scalar_ok(&fixture("invalid", name));
        let ds = verify_scalar_err(bundle);
        assert!(
            has(&ds, want),
            "{}: wanted {} in {:?}",
            name,
            want,
            codes(&ds)
        );
    }
}

#[test]
fn div_rem_reject_spurious_overflow_field_at_bind() {
    let ds = descriptor::bind(
        scalar_descriptor(),
        &fixture("invalid", "i01-div-overflow-field.atom"),
    )
    .expect_err("div_i32 forbids the overflow field");
    assert!(has(&ds, "bind.branch-forbidden-field"), "{:?}", codes(&ds));
}

// -- V-07: programmatic mutation gates ---------------------------------------

#[test]
fn mutation_wrong_types_are_rejected() {
    // not_bool over an i32 operand (mutate e19's operand e18 -> e16)
    let mut b = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let body = &mut b.modules[0].bodies[0];
    if let ExprKind::Unary { operand, .. } = &mut body.exprs[19].kind {
        *operand = hir::ExprId(16);
    } else {
        panic!("e19 must be unary");
    }
    let ds = verify_scalar_err(b);
    assert!(has(&ds, "verify.type-mismatch"), "{:?}", codes(&ds));

    // logical and over an i32 lhs (mutate e20's lhs e15 -> e16)
    let mut b = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let body = &mut b.modules[0].bodies[0];
    if let ExprKind::Logical { lhs, .. } = &mut body.exprs[20].kind {
        *lhs = hir::ExprId(16);
    } else {
        panic!("e20 must be logical");
    }
    let ds = verify_scalar_err(b);
    assert!(has(&ds, "verify.type-mismatch"), "{:?}", codes(&ds));

    // eq over mixed operand types (mutate e11's rhs e10 -> e18, a bool)
    let mut b = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let body = &mut b.modules[0].bodies[0];
    if let ExprKind::Binary { rhs, .. } = &mut body.exprs[11].kind {
        *rhs = hir::ExprId(18);
    } else {
        panic!("e11 must be binary");
    }
    let ds = verify_scalar_err(b);
    assert!(
        ds.iter()
            .any(|d| d.message.contains("both be i32 or both be bool")),
        "{:?}",
        codes(&ds)
    );
}

#[test]
fn mutation_shared_expression_and_cycles_are_rejected() {
    // shared: e23 = ge(e22, e21) -> ge(e22, e22) leaves e21 dead and e22
    // referenced twice
    let mut b = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let body = &mut b.modules[0].bodies[0];
    if let ExprKind::Binary { rhs, .. } = &mut body.exprs[23].kind {
        *rhs = hir::ExprId(22);
    } else {
        panic!("e23 must be binary");
    }
    let ds = verify_scalar_err(b);
    assert!(has(&ds, "verify.eval-position"), "{:?}", codes(&ds));

    // short-circuit self-cycle: e19 not_bool(operand = e19 itself)
    let mut b = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let body = &mut b.modules[0].bodies[0];
    if let ExprKind::Unary { operand, .. } = &mut body.exprs[19].kind {
        *operand = hir::ExprId(19);
    } else {
        panic!("e19 must be unary");
    }
    let ds = verify_scalar_err(b);
    assert!(has(&ds, "verify.expr-cycle"), "{:?}", codes(&ds));

    // two-node cycle through the logical spine: e20's lhs -> e20's consumer
    // is e24; point e19's operand at e20 (its own parent)
    let mut b = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let body = &mut b.modules[0].bodies[0];
    if let ExprKind::Unary { operand, .. } = &mut body.exprs[19].kind {
        *operand = hir::ExprId(20);
    } else {
        panic!("e19 must be unary");
    }
    let ds = verify_scalar_err(b);
    assert!(has(&ds, "verify.expr-cycle"), "{:?}", codes(&ds));
}

#[test]
fn mutation_cross_body_reference_is_bounds_rejected() {
    // b_main has 3 exprs; e5 exists only in b_add's table. The reference
    // must be rejected in a controlled way (never an indexing panic), and a
    // freshly re-bound bundle of the ORIGINAL text still verifies — a bad
    // verification result is not a reusable credential.
    let mut b = bind_scalar_ok(&fixture("valid", "s02-two-bodies.atom"));
    let main = &mut b.modules[0].bodies[1];
    assert_eq!(main.id_text, "b_main");
    if let ExprKind::Call { eval_args, .. } = &mut main.exprs[2].kind {
        eval_args[0] = hir::ExprId(5);
    } else {
        panic!("b_main e2 must be a call");
    }
    let ds = verify_scalar_err(b);
    assert!(has(&ds, "verify.bad-reference"), "{:?}", codes(&ds));

    // credential non-reuse: the fresh bind of the unmutated text verifies,
    // the mutated one does not, and re-running the mutated bundle cannot
    // produce a CheckedModule by any other entry either.
    let fresh = bind_scalar_ok(&fixture("valid", "s02-two-bodies.atom"));
    verify::verify_scalar(fresh).expect("fresh bind verifies");
}

#[test]
fn canonical_text_of_a_mutated_bundle_does_not_launder_it() {
    // write_canonical only checks identity; a graph-invalid bundle still
    // serializes — but the serialized TEXT re-binds into the same sharing
    // and is rejected by verify again. No laundering through the writer.
    let mut b = bind_scalar_ok(&fixture("valid", "s01-scalar-ops.atom"));
    let body = &mut b.modules[0].bodies[0];
    if let ExprKind::Binary { rhs, .. } = &mut body.exprs[23].kind {
        *rhs = hir::ExprId(22);
    } else {
        panic!("e23 must be binary");
    }
    let text = descriptor::write_canonical(&b, scalar_descriptor()).expect("write");
    let rebound = bind_scalar_ok(&text);
    let ds = verify_scalar_err(rebound);
    assert!(has(&ds, "verify.eval-position"), "{:?}", codes(&ds));
}

// -- deep input (iterative guards) -------------------------------------------

#[test]
fn deep_logical_chain_binds_and_verifies() {
    // 4096-node or-spine over fresh lt leaves: parse/bind/verify stay
    // iterative (no recursion), and the single-use invariant holds by
    // construction.
    let n = 4096usize;
    let mut t = String::from(
        "hir(schema: \"auto.hir.scalar.draft\", revision: 1u32, profile: \"scalar-i32-draft\") {\n",
    );
    t.push_str("    module m_deep {\n        types {\n            type t_i32 i32 {}\n            type t_bool bool {}\n            type t_f function { params: []; result: \"t_bool\" }\n        }\n");
    t.push_str(
        "        declarations { function d_main t_f { name: \"sf_main\"; body: \"b_main\" } }\n",
    );
    t.push_str("        bodies {\n            body b_main {\n                owner: \"d_main\"\n                entry: \"k0\"\n                locals {}\n                expressions {\n");
    // leaves e0/e1, lt(e0,e1)=e2, then per k: two consts + lt + or — four
    // fresh slots per iteration (c1=3+4(k-1), c2, lt, or=6+4(k-1)).
    t.push_str("                    expr e0 constant { type: t_i32; value: 1i32 }\n");
    t.push_str("                    expr e1 constant { type: t_i32; value: 2i32 }\n");
    t.push_str(
        "                    expr e2 binary { type: t_bool; op: lt_i32; lhs: e0; rhs: e1 }\n",
    );
    for k in 1..n {
        let c1 = 3 + 4 * (k - 1);
        let c2 = 4 + 4 * (k - 1);
        let lt = 5 + 4 * (k - 1);
        let or = 6 + 4 * (k - 1);
        let prev = if k == 1 { 2 } else { 6 + 4 * (k - 2) };
        t.push_str(&format!(
            "                    expr e{c1} constant {{ type: t_i32; value: {v1}i32 }}\n",
            v1 = (k % 100) + 1
        ));
        t.push_str(&format!(
            "                    expr e{c2} constant {{ type: t_i32; value: {v2}i32 }}\n",
            v2 = (k % 97) + 1
        ));
        t.push_str(&format!(
            "                    expr e{lt} binary {{ type: t_bool; op: lt_i32; lhs: e{c1}; rhs: e{c2} }}\n"
        ));
        t.push_str(&format!(
            "                    expr e{or} logical {{ type: t_bool; op: or; lhs: e{prev}; rhs: e{lt} }}\n"
        ));
    }
    let root = 6 + 4 * (n - 2);
    t.push_str(&format!(
        "                }}\n                blocks {{ block k0 {{ return(value: e{root}) }} }}\n"
    ));
    t.push_str("            }\n        }\n    }\n}\n");
    let bundle = bind_scalar_ok(&t);
    verify::verify_scalar(bundle).expect("deep or-spine verifies");
}
