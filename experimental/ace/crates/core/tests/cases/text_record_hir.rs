//! PLAN-782 T-02 text-record-domain HIR tests: triple identity and
//! five-profile isolation (V-01), combination construction / nominal
//! typing / derived resource classification (V-02), and programmatic
//! bundle mutation gates (V-08). Pure in-process bind/verify — the
//! resource walk lands in T-04 (`text_record_resources.rs`), the source
//! frontend in T-03, native execution in T-05.

use ace_core::atom_text::Diagnostic;
use ace_core::descriptor;
use ace_core::hir::{self, HirDomain, TypeKind};
use ace_core::{bind_document, text_record_descriptor, verify};

fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/text-record-hir/valid")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn bind_tr(src: &str) -> hir::Bundle {
    let b = descriptor::bind(text_record_descriptor(), src).expect("text-record bind");
    assert_eq!(b.domain, HirDomain::TextRecord);
    b
}

fn codes(ds: &[Diagnostic]) -> Vec<&str> {
    ds.iter().map(|d| d.code.as_str()).collect()
}

fn has(ds: &[Diagnostic], code: &str) -> bool {
    ds.iter().any(|d| d.code == code)
}

/// A full document with one record shape + an empty-ish main, used as the
/// carrier for shape-level negatives.
fn doc_with_record(fields: &str) -> String {
    format!(
        "hir(schema: \"auto.hir.text-record.draft\", revision: 1u32, profile: \"text-record-i32-draft\") {{\n\
         module m {{\n    types {{\n        type t_i32 i32 {{}}\n        type t_r record {{ fields: [{}] }}\n\
         type t_sig function {{ params: []; result: i32 }}\n    }}\n\
         declarations {{ function d t_sig {{ name: \"f\"; body: \"b\" }} }}\n\
         bodies {{ body b {{ owner: \"d\"; entry: \"k\"; expressions {{ expr z constant {{ type: i32; value: 0i32 }} }} blocks {{ block k {{ return(value: z) }} }} }} }}\n\
         }}\n}}\n",
        fields
    )
}

// -- V-01: identity, isolation, dispatch ------------------------------------

#[test]
fn text_record_identity_binds_and_verifies() {
    let bundle = bind_tr(&fixture("tr01-combination.atom"));
    let checked = verify::verify_text_record(bundle).expect("text-record verify");
    assert_eq!(checked.domain(), HirDomain::TextRecord);
    let m = &checked.bundle().modules[0];
    let token = m
        .types
        .iter()
        .find(|t| matches!(t.kind, TypeKind::Record(_)))
        .expect("Token record present");
    assert_eq!(token.id_text, "t_token");
}

#[test]
fn canonical_writer_roundtrip_is_semantically_equal() {
    let bundle = bind_tr(&fixture("tr01-combination.atom"));
    let out =
        descriptor::write_canonical(&bundle, text_record_descriptor()).expect("canonical write");
    let reb = bind_tr(&out);
    assert!(
        reb == *checked_bundle(&fixture("tr01-combination.atom")),
        "writer roundtrip must be semantically equal"
    );
}

fn checked_bundle(src: &str) -> Box<hir::Bundle> {
    // bind twice-independent copy for equality comparison
    Box::new(descriptor::bind(text_record_descriptor(), src).expect("rebind"))
}

#[test]
fn writer_rejects_foreign_domain() {
    let bundle = bind_tr(&fixture("tr01-combination.atom"));
    let err = descriptor::write_canonical(&bundle, ace_core::record_descriptor())
        .expect_err("record writer must reject a text-record bundle");
    assert_eq!(err.code, "canonical.identity-conflict");
}

#[test]
fn unregistered_triple_rejected_and_new_identity_listed() {
    let src = "hir(schema: \"auto.hir.text-record.draft\", revision: 1u32, profile: \"text-record-i32-draft\") {\n    module m {}\n}\n";
    // registered via bind_document: now it BINDS (this is the T-01 N07
    // probe inverted — registration turned the rejection into acceptance)
    bind_document(src).expect("registered triple must bind");
    // changing any single dimension of the triple stays rejected
    let bad = "hir(schema: \"auto.hir.text-record.draft\", revision: 2u32, profile: \"text-record-i32-draft\") {\n    module m {}\n}\n";
    let diags = bind_document(bad).expect_err("revision drift must reject");
    assert_eq!(diags[0].code, "bind.descriptor-unknown");
    assert!(
        diags[0]
            .message
            .contains("auto.hir.text-record.draft/1/text-record-i32-draft"),
        "accepted list names the combination triple: {}",
        diags[0].message
    );
}

#[test]
fn old_domain_entries_reject_the_combination_document() {
    let src = fixture("tr01-combination.atom");
    // the OLD descriptors do not know the combination surface: the record
    // descriptor rejects the header triple outright
    let diags = descriptor::bind(ace_core::record_descriptor(), &src)
        .expect_err("record descriptor must reject the text-record triple");
    assert!(
        diags[0].code == "bind.schema-mismatch" || diags[0].code == "bind.descriptor-unknown",
        "header triple rejection, got {}",
        diags[0].code
    );
    // and the text-record verifier rejects bundles of foreign domains
    let text_bundle = descriptor::bind(
        ace_core::text_descriptor(),
        "hir(schema: \"auto.hir.text.draft\", revision: 1u32, profile: \"text-i32-draft\") {\n    module m {}\n}\n",
    )
    .expect("text bind");
    let diags = verify::verify_text_record(text_bundle).expect_err("domain mismatch");
    assert_eq!(diags[0].code, "verify.domain");
}

#[test]
fn text_record_verify_entry_rejects_text_bundle() {
    let bundle = bind_tr("hir(schema: \"auto.hir.text-record.draft\", revision: 1u32, profile: \"text-record-i32-draft\") {\n    module m {}\n}\n");
    let diags = verify::verify_text(bundle).expect_err("text entry rejects foreign domain");
    assert_eq!(diags[0].code, "verify.domain");
}

// -- V-02: construction, nominal typing, classification ----------------------

#[test]
fn str_field_is_the_resource_face_and_str_field_borrow_rejects() {
    // Str field binds + verifies (classification: resource record)
    let ok = doc_with_record("{ name: text, type: Str }");
    verify::verify_text_record(bind_tr(&ok)).expect("Str field is the combination face");
    // str field is the 774 D-03 rejection at the verify door
    let bad = doc_with_record("{ name: text, type: str }");
    let diags = verify::verify_text_record(bind_tr(&bad)).expect_err("str field rejects");
    assert!(has(&diags, "verify.field-borrow"), "{:?}", codes(&diags));
}

#[test]
fn char_field_allowed_and_classification_is_transitive() {
    let doc = doc_with_record("{ name: c, type: char }, { name: meta, type: t_meta }").replace(
        "type t_r record",
        "type t_meta record { fields: [{ name: s, type: Str }] }\n        type t_r record",
    );
    let checked = verify::verify_text_record(bind_tr(&doc)).expect("transitive resource ok");
    let m = &checked.bundle().modules[0];
    let resource = verify::record_resource_map(m);
    let idx = |name: &str| {
        m.types
            .iter()
            .position(|t| t.id_text == name)
            .unwrap_or_else(|| panic!("type {}", name))
    };
    assert!(
        !resource[idx("t_i32")],
        "function type is never a resource record"
    );
    assert!(resource[idx("t_meta")], "direct Str field -> resource");
    assert!(
        resource[idx("t_r")],
        "transitive through nesting -> resource"
    );
}

#[test]
fn containment_cycle_rejects() {
    let doc = doc_with_record("{ name: a, type: t_r }");
    let diags = verify::verify_text_record(bind_tr(&doc)).expect_err("self-loop rejects");
    assert!(has(&diags, "verify.record-cycle"), "{:?}", codes(&diags));
}

#[test]
fn copyable_record_keeps_value_copy_semantics() {
    // Meta{i32} stays copyable: bare let-of-local transfer is NOT a
    // resource-assign rejection (771 value copy preserved)
    let src = r#"
hir(schema: "auto.hir.text-record.draft", revision: 1u32, profile: "text-record-i32-draft") {
    module m {
        types {
            type t_i32 i32 {}
            type t_m record { fields: [{ name: a, type: i32 }] }
            type t_sig function { params: [t_m]; result: i32 }
        }
        declarations { function d t_sig { name: "f"; body: "b" } }
        bodies {
            body b {
                owner: "d"
                entry: "k"
                locals {
                    local l_p t_m { name: "p"; role: param; index: 0u32; mode: view }
                    local l_q t_m { name: "q"; role: binding }
                }
                expressions {
                    expr ep read_local { type: t_m; local: l_p }
                    expr eq2 read_local { type: t_m; local: l_q }
                    expr ef field_get { type: i32; base: eq2; path: [0u32] }
                }
                blocks { block k { let(local: l_q, value: ep); return(value: ef) } }
            }
        }
    }
}
"#;
    verify::verify_text_record(bind_tr(src))
        .expect("copyable record: bare let-of-local is a value copy, not a rejection");
}

// -- V-08: programmatic bundle mutation gates ---------------------------------

#[test]
fn bare_transfer_of_resource_record_rejects() {
    // let u = t (bare ReadLocal of a resource-record local) — the
    // classification-based rejection (resource-assign).
    let src = r#"
hir(schema: "auto.hir.text-record.draft", revision: 1u32, profile: "text-record-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m {
        types {
            type t_i32 i32 {}
            type t_t record { fields: [{ name: text, type: Str }] }
            type t_sig function { params: [t_t]; result: i32 }
        }
        declarations { function d t_sig { name: "f"; body: "b" } }
        bodies {
            body b {
                owner: "d"
                entry: "k"
                locals {
                    local l_t t_t { name: "t"; role: param; index: 0u32; mode: view }
                    local l_u t_t { name: "u"; role: binding }
                }
                expressions {
                    expr et read_local { type: t_t; local: l_t }
                    expr ef field_get { type: Str; base: et; path: [0u32] }
                    expr elen text_len { type: i32; operand: ef }
                }
                blocks { block k { let(local: l_u, value: et); return(value: elen) } }
            }
        }
    }
}
"#;
    let diags = verify::verify_text_record(bind_tr(src)).expect_err("bare let of resource record");
    assert!(has(&diags, "verify.resource-assign"), "{:?}", codes(&diags));
}

#[test]
fn partial_move_rejects_and_move_param_type_gate() {
    // a `move` wrapper around a field_get is a partial move
    let src = r#"
hir(schema: "auto.hir.text-record.draft", revision: 1u32, profile: "text-record-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m {
        types {
            type t_i32 i32 {}
            type t_t record { fields: [{ name: text, type: Str }] }
            type t_sig function { params: [t_t]; result: i32 }
        }
        declarations { function d t_sig { name: "f"; body: "b" } }
        bodies {
            body b {
                owner: "d"
                entry: "k"
                locals { local l_t t_t { name: "t"; role: param; index: 0u32; mode: view } }
                expressions {
                    expr et read_local { type: t_t; local: l_t }
                    expr ef field_get { type: Str; base: et; path: [0u32] }
                    expr emv move { type: Str; operand: ef }
                    expr elen text_len { type: i32; operand: emv }
                }
                blocks { block k { return(value: elen) } }
            }
        }
    }
}
"#;
    let diags = verify::verify_text_record(bind_tr(src)).expect_err("field move rejects");
    assert!(has(&diags, "verify.partial-move"), "{:?}", codes(&diags));

    // `move` mode on a copyable record parameter is rejected
    let src2 = r#"
hir(schema: "auto.hir.text-record.draft", revision: 1u32, profile: "text-record-i32-draft") {
    module m {
        types {
            type t_i32 i32 {}
            type t_m record { fields: [{ name: a, type: i32 }] }
            type t_sig function { params: [t_m]; result: i32 }
        }
        declarations { function d t_sig { name: "f"; body: "b" } }
        bodies {
            body b {
                owner: "d"
                entry: "k"
                locals { local l_m t_m { name: "m"; role: param; index: 0u32; mode: move } }
                expressions { expr z constant { type: i32; value: 0i32 } }
                blocks { block k { return(value: z) } }
            }
        }
    }
}
"#;
    let diags =
        verify::verify_text_record(bind_tr(src2)).expect_err("move mode on copyable rejects");
    assert!(
        has(&diags, "verify.ownership-required"),
        "{:?}",
        codes(&diags)
    );
}

#[test]
fn record_clone_expression_type_checks() {
    // record_clone of the same record type verifies; of a different record
    // type is a type mismatch (structural face of V-02; deep-copy resource
    // semantics land with the T-04 walk + T-05 lowering).
    let src = r#"
hir(schema: "auto.hir.text-record.draft", revision: 1u32, profile: "text-record-i32-draft") {
    requires: ["ac.runtime.text.v1"]
    module m {
        types {
            type t_i32 i32 {}
            type t_t record { fields: [{ name: text, type: Str }] }
            type t_o record { fields: [{ name: text, type: Str }] }
            type t_sig function { params: [t_t]; result: i32 }
        }
        declarations { function d t_sig { name: "f"; body: "b" } }
        bodies {
            body b {
                owner: "d"
                entry: "k"
                locals { local l_t t_t { name: "t"; role: param; index: 0u32; mode: view } }
                expressions {
                    expr et read_local { type: t_t; local: l_t }
                    expr ec record_clone { type: t_t; record: t_t; base: et }
                    expr ef field_get { type: Str; base: ec; path: [0u32] }
                    expr elen text_len { type: i32; operand: ef }
                }
                blocks { block k { return(value: elen) } }
            }
        }
    }
}
"#;
    let checked = verify::verify_text_record(bind_tr(src)).expect("record clone verifies");
    let _ = checked;
    // wrong-record clone: mutate record target to the same-shape other type
    let mut bundle = bind_tr(src);
    let m = &mut bundle.modules[0];
    let o_idx = m
        .types
        .iter()
        .position(|t| t.id_text == "t_o")
        .expect("t_o present") as u32;
    let mut hit = false;
    for body in &mut m.bodies {
        for e in &mut body.exprs {
            if let hir::ExprKind::RecordClone { record, .. } = &mut e.kind {
                record.0 = o_idx; // same shape, different name — nominal!
                hit = true;
            }
        }
    }
    assert!(hit, "clone expr present");
    let diags = verify::verify_text_record(bundle).expect_err("cross-record clone rejects");
    assert!(has(&diags, "verify.type-mismatch"), "{:?}", codes(&diags));
}

// -- V-08 (F-782-R1-05 repair): REAL programmatic mutations — bind a valid
// -- document, mutate the Bundle in Rust, re-verify: every mutation is a
// -- controlled rejection, never a panic (completing IS the no-panic
// -- proof). The Move.inner case panicked before the structural-bounds
// -- ordering fix (index 4294967295 out of bounds).

fn tr01_mutated<F: FnOnce(&mut hir::Bundle)>(f: F) -> Vec<Diagnostic> {
    let mut b = bind_tr(&fixture("tr01-combination.atom"));
    let before = format!("{:?}", b.modules[0].bodies);
    f(&mut b);
    // the mutation must actually change the bundle
    assert_ne!(
        before,
        format!("{:?}", b.modules[0].bodies),
        "mutation no-op"
    );
    verify::verify_text_record(b).expect_err("mutated bundle must reject")
}

#[test]
fn mutation_move_inner_oob_rejects_without_panic() {
    // F-782-R1-05 original counterexample: Move.inner = ExprId(u32::MAX)
    // panicked in check_text_record_rules before the bounds gate; now the
    // structural walk reports it first
    let ds = tr01_mutated(|b| {
        for body in b.modules[0].bodies.iter_mut() {
            for e in body.exprs.iter_mut() {
                if let hir::ExprKind::Move { inner } = &mut e.kind {
                    *inner = hir::ExprId(u32::MAX);
                }
            }
        }
    });
    assert!(has(&ds, "verify.bad-reference"), "{:?}", codes(&ds));
}

#[test]
fn mutation_field_get_oob_base_and_path_reject() {
    let ds = tr01_mutated(|b| {
        for body in b.modules[0].bodies.iter_mut() {
            for e in body.exprs.iter_mut() {
                if let hir::ExprKind::FieldGet { base, .. } = &mut e.kind {
                    *base = hir::ExprId(u32::MAX);
                }
            }
        }
    });
    assert!(has(&ds, "verify.bad-reference"), "{:?}", codes(&ds));
    let ds2 = tr01_mutated(|b| {
        for body in b.modules[0].bodies.iter_mut() {
            for e in body.exprs.iter_mut() {
                if let hir::ExprKind::FieldGet { path, .. } = &mut e.kind {
                    path.push(hir::FieldId(u32::MAX));
                }
            }
        }
    });
    assert!(!ds2.is_empty(), "field-path oob must reject, got none");
}

#[test]
fn mutation_ctor_and_call_binding_oob_reject() {
    let ds = tr01_mutated(|b| {
        for body in b.modules[0].bodies.iter_mut() {
            for e in body.exprs.iter_mut() {
                if let hir::ExprKind::RecordCtor { bindings, .. } = &mut e.kind {
                    for cb in bindings.iter_mut() {
                        cb.arg = u32::MAX;
                    }
                }
            }
        }
    });
    assert!(!ds.is_empty(), "ctor binding arg oob must reject");
    let ds2 = tr01_mutated(|b| {
        for body in b.modules[0].bodies.iter_mut() {
            for e in body.exprs.iter_mut() {
                if let hir::ExprKind::Call { bindings, .. } = &mut e.kind {
                    for cb in bindings.iter_mut() {
                        cb.arg = u32::MAX;
                    }
                }
            }
        }
    });
    assert!(!ds2.is_empty(), "call binding arg oob must reject");
}

#[test]
fn mutation_requires_removal_rejects() {
    // the module derives ac.runtime.text.v1 from its resource ops; a
    // bundle that silently drops the requires line fails the derivation
    let mut b = bind_tr(&fixture("tr01-combination.atom"));
    b.requires.clear();
    let ds = verify::verify_text_record(b).expect_err("requires removal must reject");
    assert!(
        has(&ds, "capability.missing") || has(&ds, "verify.profile-op"),
        "{:?}",
        codes(&ds)
    );
}

#[test]
fn mutation_type_id_remap_rejects() {
    // retarget a local's type to another record type id: nominal typing
    // must reject rather than misinterpret the aggregate
    let ds = tr01_mutated(|b| {
        let l = b.modules[0].bodies[2].locals.iter_mut().next().unwrap();
        l.ty = hir::TypeUse::Ref(hir::TypeId(u32::MAX));
    });
    assert!(has(&ds, "verify.bad-reference"), "{:?}", codes(&ds));
}
