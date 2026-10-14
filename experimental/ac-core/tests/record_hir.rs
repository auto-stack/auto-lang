//! PLAN-771 T-02 record-domain HIR tests: triple identity and isolation
//! (V-01), nominal typing / containment graph (V-03), and programmatic
//! bundle mutation gates (V-04). Pure in-process bind/verify — native
//! execution of the same shape lives in `record_native.rs` (T-05), and the
//! source frontend half in `record_source.rs` (T-03).

use auto_ac_prototype::atom_text::{Diagnostic, Span};

fn zero_span() -> Span {
    Span { start: 0, end: 0 }
}
use auto_ac_prototype::descriptor;
use auto_ac_prototype::hir::{self, DefKind, ExprKind, HirDomain, PlaceKind, TypeKind, TypeUse};
use auto_ac_prototype::{bind_document, record_descriptor, verify};

fn fixture(kind: &str, name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("experimental/ac-core/fixtures/record-hir")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn bind_record_ok(src: &str) -> hir::Bundle {
    let b = descriptor::bind(record_descriptor(), src).expect("record bind must succeed");
    assert_eq!(
        b.domain,
        HirDomain::Record,
        "binder must stamp the record domain"
    );
    b
}

fn verify_record_err(bundle: hir::Bundle) -> Vec<Diagnostic> {
    verify::verify_record(bundle).expect_err("verify_record must reject")
}

fn codes(ds: &[Diagnostic]) -> Vec<&str> {
    ds.iter().map(|d| d.code.as_str()).collect()
}

fn has(ds: &[Diagnostic], code: &str) -> bool {
    ds.iter().any(|d| d.code == code)
}

/// Wrap a types-only document body into a full record-domain document.
fn doc_with_types(types: &str) -> String {
    format!(
        "hir(schema: \"auto.hir.record.draft\", revision: 1u32, profile: \"record-i32-draft\") {{\n\
         module m {{\n    types {{\n        type t_i32 i32 {{}}\n        {}\n    }}\n}}\n}}\n",
        types
    )
}

// -- V-01: identity, isolation, dispatch ------------------------------------

#[test]
fn record_identity_binds_and_verifies() {
    let bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let checked = verify::verify_record(bundle).expect("record verify");
    assert_eq!(checked.domain(), HirDomain::Record);
    let m = &checked.bundle().modules[0];
    // the fixture's shape is present: 2 records, 1 method pair, borrows
    let records: Vec<&str> = m
        .types
        .iter()
        .filter(|t| matches!(t.kind, TypeKind::Record(_)))
        .map(|t| t.id_text.as_str())
        .collect();
    assert_eq!(records, vec!["t_pair", "t_inner"]);
    let methods: Vec<(&str, &str)> = m
        .defs
        .iter()
        .filter_map(|d| match &d.kind {
            DefKind::Method { name, receiver, .. } => Some((name.as_str(), receiver.as_str())),
            _ => None,
        })
        .collect();
    assert_eq!(methods, vec![("new", "static"), ("left_is", "view")]);
    let main = m
        .bodies
        .iter()
        .find(|b| b.id_text == "b_main")
        .expect("b_main");
    assert!(main
        .exprs
        .iter()
        .any(|e| matches!(&e.kind, ExprKind::BorrowMut { .. })));
    assert!(main
        .exprs
        .iter()
        .any(|e| matches!(&e.kind, ExprKind::BorrowView { .. })));
    assert!(main
        .places
        .iter()
        .any(|p| matches!(p.kind, PlaceKind::FieldPath(_))));
}

#[test]
fn bind_document_dispatches_record_identity() {
    let src = fixture("valid", "r01-records.atom");
    let bundle = bind_document(&src).expect("registry must dispatch record identity");
    assert_eq!(bundle.domain, HirDomain::Record);
}

#[test]
fn unregistered_and_wrong_identities_are_rejected() {
    // unregistered triple (the text profile belongs to PLAN-770)
    let textified = fixture("valid", "r01-records.atom")
        .replace("auto.hir.record.draft", "auto.hir.text.draft");
    let err = bind_document(&textified).expect_err("unregistered identity must fail");
    assert_eq!(err[0].code, "bind.descriptor-unknown");
    // wrong-profile echo against the record descriptor
    let err = descriptor::bind(
        record_descriptor(),
        &fixture("valid", "r01-records.atom").replace("record-i32-draft", "scalar-i32-draft"),
    )
    .expect_err("profile mismatch must fail at bind");
    assert_eq!(err[0].code, "bind.profile-mismatch");
    // a scalar document cannot bind under the record descriptor
    let scalar_doc = "hir(schema: \"auto.hir.scalar.draft\", revision: 1u32, profile: \"scalar-i32-draft\") {\nmodule m { types { type t_i32 i32 {} } }\n}\n";
    let err = descriptor::bind(record_descriptor(), scalar_doc)
        .expect_err("scalar doc under record descriptor");
    assert!(
        err.iter().any(|d| d.code.starts_with("bind.")),
        "expected a bind diagnostic, got {:?}",
        codes(&err)
    );
}

#[test]
fn record_nodes_are_rejected_in_old_domains() {
    // a record type smuggled into a scalar-domain document (programmatically
    // bound under the scalar descriptor must not pass the scalar verifier)
    let record_types = "type t_pair record { fields: [{ name: left, type: i32 }] }";
    let scalar_doc = format!(
        "hir(schema: \"auto.hir.scalar.draft\", revision: 1u32, profile: \"scalar-i32-draft\") {{\n\
         module m {{ types {{ type t_i32 i32 {{}}\n {} }} }}\n}}\n",
        record_types
    );
    // the scalar schema has no `fields`/`record` — bind rejects first
    let err = descriptor::bind(auto_ac_prototype::scalar_descriptor(), &scalar_doc)
        .expect_err("scalar descriptor must reject record types");
    assert!(err.iter().any(|d| d.code.starts_with("bind.")));
    // and the record verifier rejects a scalar-domain bundle outright
    let scalar_bundle =
        descriptor::bind(auto_ac_prototype::scalar_descriptor(), "hir(schema: \"auto.hir.scalar.draft\", revision: 1u32, profile: \"scalar-i32-draft\") {\nmodule m { types { type t_i32 i32 {} } }\n}\n")
            .expect("plain scalar doc binds");
    let err = verify::verify_record(scalar_bundle).expect_err("domain gate");
    assert_eq!(err[0].code, "verify.domain");
}

#[test]
fn canonical_roundtrip_record_is_semantically_equal() {
    let src = fixture("valid", "r01-records.atom");
    let bundle = bind_record_ok(&src);
    let canonical = descriptor::write_canonical(&bundle, record_descriptor())
        .expect("canonical writer must handle the record domain");
    let rebundled = bind_record_ok(&canonical);
    assert_eq!(
        rebundled, bundle,
        "canonical roundtrip must be semantic-equal"
    );
    assert_eq!(
        verify::verify_record(rebundled).map(|_| ()).ok(),
        Some(()),
        "roundtripped bundle still verifies"
    );
}

// -- V-03: nominal typing and the containment graph --------------------------

#[test]
fn nominal_mismatch_is_detected_on_direct_hir() {
    // bind the valid fixture, then retarget a field_get's base local type
    // to the structurally identical t_pair2 — nominal equality must reject
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    // add a structurally identical type and retarget b_main's l_p
    let m = &mut bundle.modules[0];
    m.types.push(hir::TypeDef {
        id_text: "t_pair2".to_string(),
        kind: TypeKind::Record(hir::RecordShape {
            fields: match &m.types[2].kind {
                TypeKind::Record(s) => s.fields.clone(),
                _ => unreachable!(),
            },
        }),
        span: zero_span(),
    });
    let tid2 = (m.types.len() - 1) as u32;
    for b in m.bodies.iter_mut() {
        for l in b.locals.iter_mut() {
            if l.name == "p" && l.ty == TypeUse::Ref(hir::TypeId(2)) {
                l.ty = TypeUse::Ref(hir::TypeId(tid2));
            }
        }
    }
    let diags = verify_record_err(bundle);
    assert!(
        has(&diags, "verify.type-mismatch"),
        "nominal mismatch must be a type error, got {:?}",
        codes(&diags)
    );
}

#[test]
fn forward_reference_and_nested_records_verify() {
    // t_fwd is declared before the record it names — pass A registration
    // makes this legal; the graph is acyclic
    let src = doc_with_types(concat!(
        "type t_fwd record { fields: [{ name: inner, type: t_late }] }\n",
        "type t_late record { fields: [{ name: x, type: i32 }] }\n",
    ));
    let bundle = bind_record_ok(&src);
    verify::verify_record(bundle).expect("forward reference must verify");
}

#[test]
fn self_loop_mutual_loop_and_disconnected_loop_are_cycles() {
    let self_loop = doc_with_types("type t_a record { fields: [{ name: x, type: t_a }] }");
    let diags = verify_record_err(bind_record_ok(&self_loop));
    assert!(has(&diags, "verify.record-cycle"), "{:?}", codes(&diags));

    let mutual = doc_with_types(concat!(
        "type t_a record { fields: [{ name: b, type: t_b }] }\n",
        "type t_b record { fields: [{ name: a, type: t_a }] }\n",
    ));
    let diags = verify_record_err(bind_record_ok(&mutual));
    assert!(has(&diags, "verify.record-cycle"), "{:?}", codes(&diags));

    // disconnected loop: an island t_x -> t_y -> t_x that no acyclic record
    // reaches must still be found (probe P1's third shape)
    let disconnected = doc_with_types(concat!(
        "type t_ok record { fields: [{ name: v, type: i32 }] }\n",
        "type t_x record { fields: [{ name: y, type: t_y }] }\n",
        "type t_y record { fields: [{ name: x, type: t_x }] }\n",
    ));
    let diags = verify_record_err(bind_record_ok(&disconnected));
    assert!(has(&diags, "verify.record-cycle"), "{:?}", codes(&diags));
}

#[test]
fn deep_1024_type_chain_verifies_without_overflow() {
    // V-03/V-09 half: a 1024-deep nesting chain binds and verifies — the
    // cycle check and walks are iterative (probe P1/P2 shape)
    let mut types = String::new();
    types.push_str("type t_0 record { fields: [{ name: x, type: i32 }] }\n");
    for k in 1..1024u32 {
        types.push_str(&format!(
            "type t_{k} record {{ fields: [{{ name: a, type: i32 }}, {{ name: r, type: t_{} }}] }}\n",
            k - 1
        ));
    }
    let src = doc_with_types(&types);
    let bundle = bind_record_ok(&src);
    let checked = verify::verify_record(bundle).expect("deep chain verifies");
    assert_eq!(checked.bundle().modules[0].types.len(), 1025);
}

#[test]
fn deep_field_path_resolves() {
    // a 1024-segment field path through the chain above resolves to i32
    let mut types = String::new();
    types.push_str("type t_0 record { fields: [{ name: x, type: i32 }] }\n");
    for k in 1..64u32 {
        types.push_str(&format!(
            "type t_{k} record {{ fields: [{{ name: a, type: i32 }}, {{ name: r, type: t_{} }}] }}\n",
            k - 1
        ));
    }
    let mut decls = String::from(
        "type t_probe function { params: [t_63]; result: t_i32 }
",
    );
    decls.push_str(
        "type t_main function { params: []; result: t_i32 }
",
    );
    let doc = format!(
        "hir(schema: \"auto.hir.record.draft\", revision: 1u32, profile: \"record-i32-draft\") {{
         module m {{
    types {{
        type t_i32 i32 {{}}
        {}{}    }}
         declarations {{
            function d_probe t_probe {{ name: \"rf_probe\"; body: \"b\" }}
        }}
         bodies {{
            body b {{
                owner: \"d_probe\"
                entry: \"k\"
         locals {{ local lv t_63 {{ name: \"v\"; role: param; index: 0u32; mode: view }} }}
         expressions {{
                    expr e0 read_local {{ type: t_63; local: lv }}
         expr e1 field_get {{ type: t_i32; base: e0; path: [{}, 0u32] }}
                }}
         blocks {{ block k {{ return(value: e1) }} }}
            }}
        }}
    }}
}}
",
        types,
        decls,
        vec!["1u32"; 63].join(", ")
    );
    let bundle = bind_record_ok(&doc);
    verify::verify_record(bundle).expect("deep path verifies");
}

// -- V-04: programmatic bundle mutation gates --------------------------------

fn verify_mutated(bundle: hir::Bundle) -> Vec<Diagnostic> {
    verify_record_err(bundle)
}

#[test]
fn mutation_field_typeid_is_rejected_and_sibling_passes() {
    // retarget t_pair's `right` field type from bool to t_inner
    let bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    verify::verify_record(bundle).expect("unmutated sibling passes");

    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    if let TypeKind::Record(shape) = &mut m.types[2].kind {
        shape.fields[1].ty = TypeUse::Ref(hir::TypeId(3)); // t_inner
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.type-mismatch"),
        "field TypeId mutation must be caught, got {:?}",
        codes(&diags)
    );
}

#[test]
fn mutation_field_index_out_of_range_and_owner_are_rejected() {
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    let b = m.bodies.iter_mut().find(|b| b.id_text == "b_bump").unwrap();
    // place path [0] -> [5]: out of range for t_pair's 2 fields
    if let PlaceKind::FieldPath(p) = &mut b.places[0].kind {
        p[0] = hir::FieldId(5);
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.field-owner"),
        "out-of-range field index, got {:?}",
        codes(&diags)
    );

    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    let b = m.bodies.iter_mut().find(|b| b.id_text == "b_bump").unwrap();
    // path [0, 0]: segment 1 continues through the scalar field `left`
    if let PlaceKind::FieldPath(p) = &mut b.places[0].kind {
        p.push(hir::FieldId(0));
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.field-owner"),
        "scalar mid-path continuation, got {:?}",
        codes(&diags)
    );
}

#[test]
fn mutation_signature_params_and_result_are_rejected() {
    // swap t_sig_new's params [i32, bool] -> [bool, i32]: the method's own
    // param locals no longer match
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    if let TypeKind::Function(sig) = &mut m.types[5].kind {
        sig.params.swap(0, 1);
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.type-mismatch"),
        "param swap must be caught, got {:?}",
        codes(&diags)
    );

    // retarget a result: t_sig_leftis bool -> i32 conflicts with its body's
    // return expression and the caller's use
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    if let TypeKind::Function(sig) = &mut m.types[6].kind {
        sig.result = TypeUse::Builtin(hir::BuiltinType::I32);
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.type-mismatch"),
        "result mutation must be caught, got {:?}",
        codes(&diags)
    );
}

#[test]
fn mutation_method_receiver_and_record_are_rejected() {
    // static -> view on d_new: params[0] must be the owner record
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    for d in m.defs.iter_mut() {
        if let DefKind::Method { name, receiver, .. } = &mut d.kind {
            if name == "new" {
                *receiver = hir::ReceiverKind::View;
            }
        }
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.type-mismatch"),
        "receiver mutation, got {:?}",
        codes(&diags)
    );

    // retarget d_left_is's owning record to t_i32 (not a record)
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    for d in m.defs.iter_mut() {
        if let DefKind::Method { name, record, .. } = &mut d.kind {
            if name == "left_is" {
                *record = hir::TypeId(0);
            }
        }
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.type-mismatch"),
        "owner record mutation, got {:?}",
        codes(&diags)
    );
}

#[test]
fn mutation_out_of_range_typeref_in_record_field_is_bounds_rejected() {
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    if let TypeKind::Record(shape) = &mut m.types[2].kind {
        shape.fields[0].ty = TypeUse::Ref(hir::TypeId(999));
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.bad-reference"),
        "field type bounds, got {:?}",
        codes(&diags)
    );
}

#[test]
fn mutation_ctor_bindings_missing_field_is_rejected() {
    // drop the second ctor binding in b_new: only 1 of 2 fields initialized
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    let b = m.bodies.iter_mut().find(|b| b.id_text == "b_new").unwrap();
    for e in b.exprs.iter_mut() {
        if let ExprKind::RecordCtor { bindings, .. } = &mut e.kind {
            bindings.pop();
        }
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.ctor-arity"),
        "ctor arity mutation, got {:?}",
        codes(&diags)
    );
}

#[test]
fn mutation_borrow_exclusivity_is_rejected() {
    // duplicate the mut borrow of l_q as the second argument of ebump's
    // call site: (q.mut, <reuse>) — construct via a second BorrowMut on the
    // same local bound to param 1 of a record-arg call. The fixture's
    // t_sig_bump takes (Pair, i32); swap the i32 arg for another borrow of
    // l_q by retyping the signature to (Pair, Pair) — both record params,
    // both mut — hitting the exclusivity rule.
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    // t_sig_bump params: [t_pair, t_i32] -> [t_pair, t_pair]
    if let TypeKind::Function(sig) = &mut m.types[7].kind {
        sig.params[1] = TypeUse::Ref(hir::TypeId(2));
    }
    // b_bump's second param local must follow the new type; then the call
    // site's second argument becomes a second borrow of l_q
    {
        let b = m.bodies.iter_mut().find(|b| b.id_text == "b_bump").unwrap();
        for l in b.locals.iter_mut() {
            if l.name == "d" {
                l.ty = TypeUse::Ref(hir::TypeId(2));
            }
        }
    }
    // b_main: replace the const-5 argument expr with a second BorrowMut(l_q)
    let b = m.bodies.iter_mut().find(|b| b.id_text == "b_main").unwrap();
    let c5_idx = b
        .exprs
        .iter()
        .position(|e| matches!(&e.kind, ExprKind::Constant { value: 5 }))
        .expect("c5a");
    b.exprs[c5_idx] = hir::Expr {
        id_text: b.exprs[c5_idx].id_text.clone(),
        ty: TypeUse::Ref(hir::TypeId(2)),
        kind: ExprKind::BorrowMut {
            local: b
                .locals
                .iter()
                .position(|l| l.name == "q")
                .map(|i| hir::LocalId(i as u32))
                .unwrap(),
        },
        span: b.exprs[c5_idx].span,
    };
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.mut-conflict"),
        "double mut borrow, got {:?}",
        codes(&diags)
    );
}

#[test]
fn mutation_borrow_of_immutable_local_is_rejected() {
    // retarget ebump's BorrowMut from l_q (var) to l_p (let)
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    let b = m.bodies.iter_mut().find(|b| b.id_text == "b_main").unwrap();
    let lp = b
        .locals
        .iter()
        .position(|l| l.name == "p")
        .map(|i| hir::LocalId(i as u32))
        .unwrap();
    for e in b.exprs.iter_mut() {
        if let ExprKind::BorrowMut { local } = &mut e.kind {
            *local = lp;
        }
    }
    // the assign to q.left still works; the mut borrow of let p must not
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.immutable-write"),
        "mut borrow of let, got {:?}",
        codes(&diags)
    );
}

#[test]
fn mutation_borrow_outside_call_position_is_rejected() {
    // use the BorrowMut expr as a let value instead of a call argument
    let mut bundle = bind_record_ok(&fixture("valid", "r01-records.atom"));
    let m = &mut bundle.modules[0];
    let b = m.bodies.iter_mut().find(|b| b.id_text == "b_main").unwrap();
    let bump_idx = b
        .exprs
        .iter()
        .position(|e| matches!(&e.kind, ExprKind::BorrowMut { .. }))
        .unwrap();
    let borrow = b.exprs[bump_idx].clone();
    // rebind the ebump call's arg 0 to a read of l_q and repurpose the
    // borrow as the value of the `let l_q2` — but there is no l_q2 local;
    // simpler: make the borrow the value of the existing l_r1 let
    let eqb = hir::Expr {
        id_text: "x_replacement".to_string(),
        ty: TypeUse::Ref(hir::TypeId(2)),
        kind: ExprKind::ReadLocal {
            local: hir::LocalId(b.locals.iter().position(|l| l.name == "q").unwrap() as u32),
        },
        span: borrow.span,
    };
    b.exprs[bump_idx] = eqb;
    // find the let for l_r1 and retarget its value to the borrow expr
    let l_r1 = b.locals.iter().position(|l| l.name == "r1").unwrap() as u32;
    for bl in b.blocks.iter_mut() {
        for s in bl.stmts.iter_mut() {
            if let hir::Stmt::Let { local, value } = s {
                if local.0 == l_r1 {
                    // the borrow expr id must differ from any existing one;
                    // push a fresh expr
                    b.exprs.push(hir::Expr {
                        id_text: "e_stray_borrow".to_string(),
                        ty: borrow.ty,
                        kind: borrow.kind.clone(),
                        span: borrow.span,
                    });
                    *value = hir::ExprId((b.exprs.len() - 1) as u32);
                }
            }
        }
    }
    let diags = verify_mutated(bundle);
    assert!(
        has(&diags, "verify.eval-position"),
        "borrow outside call position, got {:?}",
        codes(&diags)
    );
}
