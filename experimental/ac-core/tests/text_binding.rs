//! PLAN-741 T-02 text binding tests: the four existing design fixtures plus
//! enum-shorthand and swapped-field forms must bind through the real
//! reader/binder; rejection fixtures must produce structured diagnostics and
//! no bound module.

use auto_ac_prototype::atom_text::{Diagnostic, Stage};
use auto_ac_prototype::descriptor;
use auto_ac_prototype::hir::{self, BuiltinType, TypeUse};

fn repo_path(rel: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_path(rel)).unwrap_or_else(|e| panic!("read {}: {}", rel, e))
}

fn bind_ok(src: &str) -> hir::Bundle {
    match descriptor::bind(auto_ac_prototype::descriptor(), src) {
        Ok(b) => b,
        Err(diags) => {
            let rendered: Vec<String> = diags.iter().map(|d| d.render("input", src)).collect();
            panic!(
                "expected bind success, got {} diagnostics:\n{}",
                diags.len(),
                rendered.join("\n")
            )
        }
    }
}

fn bind_err(src: &str) -> Vec<Diagnostic> {
    descriptor::bind(auto_ac_prototype::descriptor(), src)
        .err()
        .unwrap_or_else(|| panic!("expected bind failure, got success:\n{}", src))
}

fn fixture(sub: &str, name: &str) -> String {
    read(&format!("experimental/ac-core/fixtures/{}/{}", sub, name))
}

#[test]
fn descriptor_schema_parses() {
    // The embedded descriptor must be structurally valid; bind_source would
    // panic otherwise. Force initialization here for a clear failure site.
    let _ = auto_ac_prototype::descriptor();
}

#[test]
fn four_existing_documents_bind() {
    for name in [
        "01-add.atom",
        "01-add.explicit.atom",
        "02-control.atom",
        "03-call-order.atom",
    ] {
        let src = read(&format!("docs/design/strategy/hir-examples/{}", name));
        let bundle = bind_ok(&src);
        assert_eq!(bundle.modules.len(), 1, "{}: one module", name);
    }
}

#[test]
fn author_and_explicit_forms_are_semantically_equal() {
    let author = bind_ok(&read("docs/design/strategy/hir-examples/01-add.atom"));
    let explicit = bind_ok(&read(
        "docs/design/strategy/hir-examples/01-add.explicit.atom",
    ));
    assert_eq!(author, explicit);
}

#[test]
fn canonical_roundtrip_is_semantically_equal() {
    for name in [
        "01-add.atom",
        "01-add.explicit.atom",
        "02-control.atom",
        "03-call-order.atom",
        "fixtures/valid/shorthand.atom",
    ] {
        let src = if name.starts_with("fixtures/") {
            fixture("valid", name.rsplit('/').next().unwrap())
        } else {
            read(&format!("docs/design/strategy/hir-examples/{}", name))
        };
        let first = bind_ok(&src);
        let canonical = descriptor::write_canonical(&first, auto_ac_prototype::descriptor());
        let second = bind_ok(&canonical);
        assert_eq!(first, second, "roundtrip mismatch for {}", name);
    }
}

#[test]
fn builtin_enum_shorthand_binds_to_builtin_type() {
    let bundle = bind_ok(&fixture("valid", "shorthand.atom"));
    let m = &bundle.modules[0];
    // locals use the bareword i32 shorthand
    let l_a = &m.bodies[0].locals[0];
    assert!(matches!(l_a.ty, TypeUse::Builtin(BuiltinType::I32)));
    // function signature params/result use the shorthand too
    match &m.types[1].kind {
        hir::TypeKind::Function(sig) => {
            assert!(matches!(sig.params[0], TypeUse::Builtin(BuiltinType::I32)));
            assert!(matches!(sig.result, TypeUse::Builtin(BuiltinType::I32)));
        }
        other => panic!("expected function type, got {:?}", other),
    }
    // mixed form expr (paren head with type: i32) also binds
    let exprs = &m.bodies[0].exprs;
    assert!(exprs
        .iter()
        .any(|e| e.id_text == "e_add" && matches!(e.ty, TypeUse::Builtin(BuiltinType::I32))));
}

#[test]
fn field_order_and_region_interchange_is_equal() {
    let author = bind_ok(&read("docs/design/strategy/hir-examples/01-add.atom"));
    let swapped = bind_ok(&fixture("valid", "swapped-fields.atom"));
    assert_eq!(author, swapped);
}

#[test]
fn mutable_presence_is_recorded_separately() {
    let bundle = bind_ok(&read("docs/design/strategy/hir-examples/01-add.atom"));
    let locals = &bundle.modules[0].bodies[0].locals;
    let l_sum = locals.iter().find(|l| l.id_text == "l_sum").unwrap();
    assert!(!l_sum.mutable);
    assert!(
        l_sum.mutable_explicit,
        "mutable: false was written explicitly"
    );
    // 02-control's l_i is explicitly mutable
    let ctrl = bind_ok(&read("docs/design/strategy/hir-examples/02-control.atom"));
    let l_i = &ctrl.modules[0].bodies[0].locals[1];
    assert!(l_i.mutable);
}

#[test]
fn requires_capability_is_carried_through() {
    let bundle = bind_ok(&read(
        "docs/design/strategy/hir-examples/03-call-order.atom",
    ));
    assert_eq!(bundle.requires, vec!["hir.test.trace.v1".to_string()]);
}

/// (fixture, expected bind diagnostic code)
const INVALID_CASES: &[(&str, &str)] = &[
    ("duplicate-field.atom", "bind.duplicate-field"),
    ("dangling-expr-ref.atom", "bind.dangling-ref"),
    ("category-mismatch.atom", "bind.category-mismatch"),
    ("dangling-type-ref.atom", "bind.dangling-type-ref"),
    ("unknown-field.atom", "bind.unknown-field"),
    ("unknown-enum-case.atom", "bind.unknown-case"),
    ("literal-range.atom", "bind.literal-range"),
    ("profile-mismatch.atom", "bind.profile-mismatch"),
    ("revision-mismatch.atom", "bind.revision-mismatch"),
    ("unknown-node.atom", "bind.unknown-node"),
    (
        "binding-local-with-index.atom",
        "bind.branch-forbidden-field",
    ),
    ("param-local-without-index.atom", "bind.missing-field"),
    ("add-without-overflow.atom", "bind.missing-field"),
    ("lt-with-overflow.atom", "bind.branch-forbidden-field"),
    ("duplicate-type-id.atom", "bind.duplicate-id"),
];

#[test]
fn invalid_fixtures_are_rejected_with_expected_codes() {
    for (name, expected) in INVALID_CASES {
        let src = fixture("invalid", name);
        let diags = bind_err(&src);
        assert!(
            diags.iter().any(|d| d.code == *expected),
            "{}: expected code {} in {:#?}",
            name,
            expected,
            diags.iter().map(|d| d.code.clone()).collect::<Vec<_>>()
        );
        // every diagnostic must carry the bind stage and a real span
        for d in &diags {
            assert_eq!(d.stage, Stage::Bind, "{}: stage", name);
            assert!(d.span.end >= d.span.start);
        }
    }
}

#[test]
fn rejected_documents_produce_no_bundle() {
    // The API shape itself guarantees it: bind returns Err and no caller can
    // reach lowering without a Bundle. Assert the type-level fact with one
    // case to keep the guarantee visible in the suite.
    let src = fixture("invalid", "dangling-expr-ref.atom");
    assert!(auto_ac_prototype::bind_source(&src).is_err());
}

#[test]
fn diagnostics_span_the_offense() {
    let src = fixture("invalid", "dangling-expr-ref.atom");
    let diags = bind_err(&src);
    let d = diags
        .iter()
        .find(|d| d.code == "bind.dangling-ref")
        .expect("dangling-ref diagnostic");
    let slice = src.get(d.span.start..d.span.end).expect("span in range");
    assert!(slice.contains("e_missing"), "span points at `{}`", slice);
    // render includes line:col and the code
    let rendered = d.render("test.atom", &src);
    assert!(rendered.contains("test.atom:"));
    assert!(rendered.contains("bind.dangling-ref"), "{}", rendered);
}
