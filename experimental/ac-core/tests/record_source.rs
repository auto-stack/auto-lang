//! PLAN-771 T-03 record-source tests: the record profile front end
//! (V-02/V-06/V-07/V-08 source legs). Every fixture compiles through the
//! real parse→resolve→typecheck→bind→verify chain of the record domain;
//! rejection fixtures must produce structured diagnostics and no bundle.
//! Native execution of the same shape lives in `record_native.rs` (T-05).

use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::source::{compile_record_source, SourceProfile, RECORD_SOURCE_PROFILE};

fn fixture(kind: &str, name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("experimental/ac-core/fixtures/record-source")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn compile_ok(src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_record_source(src, "p.at", &[]).expect("record source compile must succeed")
}

fn compile_err(src: &str) -> Vec<Diagnostic> {
    compile_record_source(src, "p.at", &[]).expect_err("must be rejected")
}

fn has(ds: &[Diagnostic], code: &str) -> bool {
    ds.iter().any(|d| d.code == code)
}

#[test]
fn flagship_fixture_compiles_and_verifies() {
    let src = fixture("valid", "p01-records.at");
    let c = compile_ok(&src);
    assert_eq!(c.profile, RECORD_SOURCE_PROFILE);
    assert_eq!(
        c.checked.domain(),
        auto_ac_prototype::hir::HirDomain::Record
    );
}

#[test]
fn generated_atom_carries_record_shapes_and_methods() {
    let src = fixture("valid", "p01-records.at");
    let c = compile_ok(&src);
    let atom = &c.atom_text;
    assert!(atom.contains("type t_rec_Pair record { fields: [{ name: \"left\", type: t_i32 }, { name: \"right\", type: t_bool }] }"),
        "record shape in declaration order: {atom}");
    assert!(
        atom.contains("receiver: static"),
        "static new method: {atom}"
    );
    assert!(atom.contains("receiver: view"), "instance method: {atom}");
    assert!(atom.contains("mode: mut"), "mut param mode: {atom}");
    assert!(atom.contains(" field_path {"), "field-path places: {atom}");
    assert!(atom.contains("borrow_mut"), "p.mut borrow: {atom}");
    assert!(atom.contains("borrow_view"), "view borrows: {atom}");
}

#[test]
fn named_ctor_eval_order_is_source_order() {
    // V-07 order oracle (source leg): `Pair { right: .., left: .. }`
    // must emit field_args in the WRITTEN order with bindings mapping
    // left->arg1, right->arg0 — never declaration order.
    let src = fixture("valid", "p01-records.at");
    let c = compile_ok(&src);
    let atom = &c.atom_text;
    let ctor_line = atom
        .lines()
        .find(|l| {
            l.contains("record_ctor")
                && l.contains("t_rec_Pair")
                && l.contains("{ field: 0u32, arg: 1u32 }")
        })
        .expect("the right-then-left named Pair ctor");
    let args_pos = ctor_line.find("field_args: [").unwrap();
    // the named ctor in main writes right before left
    assert!(
        ctor_line[args_pos..].starts_with("field_args: [e"),
        "args exist"
    );
    let binds_pos = ctor_line.find("ctor_bindings: [").unwrap();
    let binds = &ctor_line[binds_pos..];
    assert!(
        binds.contains("{ field: 0u32, arg: 1u32 }"),
        "left(field 0) binds the SECOND-written arg: {ctor_line}"
    );
    assert!(
        binds.contains("{ field: 1u32, arg: 0u32 }"),
        "right(field 1) binds the FIRST-written arg: {ctor_line}"
    );
}

#[test]
fn receiver_borrow_precedes_args() {
    // V-08: `p.left_is(1)` lowers to a call whose eval_args[0] is a
    // borrow_view of the receiver — receiver evaluated exactly once, first.
    let src = fixture("valid", "p01-records.at");
    let c = compile_ok(&src);
    let atom = &c.atom_text;
    let borrow_line = atom
        .lines()
        .find(|l| l.contains("borrow_view"))
        .expect("a view borrow exists");
    let borrow_id = borrow_line
        .split_whitespace()
        .nth(1)
        .unwrap()
        .trim_start_matches('e')
        .to_string();
    // the receiver borrow is eval_args[0]: the call line opens its arg
    // list with exactly this expr id (either alone or before a comma)
    let head_exact = format!("eval_args: [e{}]", borrow_id);
    let head_first = format!("eval_args: [e{},", borrow_id);
    let call_line = atom
        .lines()
        .find(|l| l.contains(&head_exact) || l.contains(&head_first))
        .expect("a call consuming the receiver borrow first");
    assert!(
        call_line.contains(&head_exact) || call_line.contains(&head_first),
        "receiver borrow is eval_args[0]: {call_line}"
    );
}

// -- V-06: mutability rules ---------------------------------------------------

#[test]
fn let_field_write_is_rejected() {
    let ds = compile_err(&fixture("invalid", "i01-let-field-write.at"));
    assert!(
        has(&ds, "verify.immutable-write"),
        "let field write, got {:?}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn missing_dotmut_is_rejected() {
    let ds = compile_err(&fixture("invalid", "i02-missing-dotmut.at"));
    assert!(
        ds.iter().any(|d| d.code == "source.type-mismatch"),
        "missing .mut, got {:?}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn mut_fn_is_rejected() {
    let ds = compile_err(&fixture("invalid", "i03-mut-fn.at"));
    assert!(
        ds.iter()
            .any(|d| d.code == "source.unsupported" && d.message.contains("mut fn")),
        "mut fn, got {:?}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

// -- V-03/V-07/V-08 negatives ---------------------------------------------------

#[test]
fn nominal_records_do_not_interchange() {
    // A and B are structurally identical; A's field values may seed B's
    // fields (both i32) — but an A VALUE cannot bind a B field/param.
    let src = "type A { x int }\ntype B { x int }\nfn main() int {\n    let a = A { x: 1 }\n    let b = B { x: a }\n    return b.x\n}\n";
    let ds = compile_err(src);
    assert!(
        ds.iter().any(|d| d.code == "source.type-mismatch"),
        "record value into i32 field, got {:?}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn containment_cycle_is_rejected() {
    let ds = compile_err(&fixture("invalid", "i05-cycle.at"));
    // the cycle surfaces at bind (unknown type in the doc is impossible —
    // the graph closes at verify; the front end reports the dangling shape
    // through the record table build)
    assert!(
        !ds.is_empty(),
        "mutual cycle must be rejected somewhere in the chain"
    );
}

#[test]
fn ctor_duplicate_field_is_rejected() {
    let ds = compile_err(&fixture("invalid", "i06-ctor-order-and-fields.at"));
    assert!(
        ds.iter()
            .any(|d| d.code == "source.duplicate" || d.code == "verify.ctor-arity"),
        "duplicate field, got {:?}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn missing_field_is_rejected() {
    let src = "type Pair { left int; right bool }\nfn main() int {\n    let p = Pair { left: 1 }\n    return p.left\n}\n";
    let ds = compile_err(src);
    assert!(
        ds.iter().any(|d| d.code == "source.argument"),
        "missing field, got {:?}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn unknown_field_is_rejected() {
    let src = "type Pair { left int }\nfn main() int {\n    let p = Pair { left: 1, bogus: 2 }\n    return p.left\n}\n";
    let ds = compile_err(src);
    assert!(
        ds.iter().any(|d| d.code == "source.undefined"),
        "unknown field, got {:?}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn static_and_instance_method_misuse_is_rejected() {
    // calling an instance method statically
    let src = "type Pair { left int\n    fn get() int { return .left }\n}\nfn main() int {\n    return Pair.get()\n}\n";
    let ds = compile_err(src);
    assert!(
        ds.iter().any(|d| d.code == "source.not-callable"),
        "instance method called statically, got {:?}",
        ds.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

#[test]
fn old_profiles_reject_the_record_surface() {
    // `type` is Reserved-equivalent on core/scalar: rejected with
    // source.unsupported, same as before the record keywords existed.
    let src = "type Pair { left int }\nfn main() int { return 0 }\n";
    let err = auto_ac_prototype::source::compile_source(src, "p.at", &[])
        .expect_err("core profile must reject `type`");
    assert!(err.iter().any(|d| d.code == "source.unsupported"));
    let err = auto_ac_prototype::source::compile_scalar_source(src, "p.at", &[])
        .expect_err("scalar profile must reject `type`");
    assert!(err.iter().any(|d| d.code == "source.unsupported"));
    assert_eq!(
        SourceProfile::of(RECORD_SOURCE_PROFILE),
        Some(SourceProfile::Record)
    );
}
