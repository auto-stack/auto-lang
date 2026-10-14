//! PLAN-782 T-03 text-record source frontend tests: the check-source legs
//! of V-02 (construction / nominal typing / field faces), V-03 (explicit
//! move and clone — the acceptance side; resource counting lands with the
//! T-04 walk) and V-05 (replacement and overlap shapes), plus the frozen
//! source-face gates: partial-move rejection (including never-executed
//! branches), the user-declared `clone` duplicate, and old-domain face
//! preservation. Every fixture compiles through the real
//! parse → resolve → typecheck → adapter → bind → verify chain of the
//! combination domain; rejection fixtures must produce structured
//! diagnostics and no bundle. Native execution lives in T-05
//! (`text_record_native.rs`), the resource walk in T-04.

use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::source::{
    compile_record_source, compile_text_record_source, SourceProfile, TEXT_RECORD_SOURCE_PROFILE,
};

fn fixture(kind: &str, name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("experimental/ac-core/fixtures/text-record-source")
        .join(kind)
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

fn compile_ok(src: &str) -> auto_ac_prototype::source::CompiledSource {
    compile_text_record_source(src, "p.at", &[]).expect("combination source compile must succeed")
}

fn compile_err(src: &str) -> Vec<Diagnostic> {
    compile_text_record_source(src, "p.at", &[]).expect_err("must be rejected")
}

fn has(ds: &[Diagnostic], code: &str) -> bool {
    ds.iter().any(|d| d.code == code)
}

fn codes(ds: &[Diagnostic]) -> Vec<&str> {
    ds.iter().map(|d| d.code.as_str()).collect()
}

// -- V-02: construction, typing, field faces --------------------------------

#[test]
fn flagship_combination_compiles_and_verifies() {
    let c = compile_ok(&fixture("valid", "p01-combination.at"));
    assert_eq!(c.profile, TEXT_RECORD_SOURCE_PROFILE);
    assert_eq!(
        c.checked.domain(),
        auto_ac_prototype::hir::HirDomain::TextRecord
    );
}

#[test]
fn generated_atom_carries_the_combination_surface() {
    let c = compile_ok(&fixture("valid", "p01-combination.at"));
    let atom = &c.atom_text;
    // resource record shape with heterogeneous fields
    assert!(
        atom.contains("type t_rec_Tok record { fields: [{ name: \"text\", type: t_Str }, { name: \"kind\", type: t_i32 }, { name: \"c\", type: t_char }, { name: \"meta\", type: t_rec_Tag }, { name: \"note\", type: t_rec_Wrap }] }"),
        "field kinds in declaration order: {atom}"
    );
    // builtin deep clone
    assert!(atom.contains("record_clone"), "clone expr: {atom}");
    // whole-root move wrapper (argument value AND let position)
    let move_roots = atom
        .lines()
        .filter(|l| l.contains(" move { type: t_rec_Tok;"))
        .count();
    assert!(
        move_roots >= 3,
        "move wrappers for call args and the let move (found {move_roots}): {atom}"
    );
    // move-in parameter locals carry their mode
    assert!(
        atom.contains("role: param; index: 0u32; mode: move }"),
        "move param mode: {atom}"
    );
    // 771 method faces survive on the union surface
    assert!(atom.contains("receiver: static"), "static method: {atom}");
    assert!(atom.contains("receiver: view"), "instance method: {atom}");
    assert!(atom.contains("borrow_mut"), "p.mut borrow: {atom}");
}

#[test]
fn named_ctor_source_order_is_the_written_order() {
    // V-02 order oracle (source leg): the flagship's first Tok ctor is
    // written note/meta/c/kind/text — REVERSE declaration order. The Atom
    // must carry the field_args in that written sequence with bindings
    // mapping each declaration field to the arg that initialized it.
    let c = compile_ok(&fixture("valid", "p01-combination.at"));
    let atom = &c.atom_text;
    let ctor_line = atom
        .lines()
        .find(|l| {
            l.contains("record_ctor")
                && l.contains("t_rec_Tok")
                && l.contains("{ field: 0u32, arg: 4u32 }")
        })
        .expect("the reverse-written named Tok ctor");
    // written order: note(arg0) meta(arg1) c(arg2) kind(arg3) text(arg4)
    // -> declaration fields: text=0<-arg4, kind=1<-arg3, c=2<-arg2,
    //    meta=3<-arg1, note=4<-arg0
    for (field, arg) in [(0, 4), (1, 3), (2, 2), (3, 1), (4, 0)] {
        assert!(
            ctor_line.contains(&format!("{{ field: {field}u32, arg: {arg}u32 }}")),
            "field {field} binds written arg {arg}: {ctor_line}"
        );
    }
}

#[test]
fn positional_ctor_and_owning_contexts_promote_literals() {
    // AP-4 in every owning context: positional ctor fields, named ctor
    // fields, and static-method call formals — a borrowing string literal
    // retags to the owning Str type of the receiving position.
    let c = compile_ok(&fixture("valid", "p01-combination.at"));
    let atom = &c.atom_text;
    // positional ctor: the ONLY identity binding over all five fields is
    // the positional form (`Tok("tail", 2, 'B', Tag(8), Wrap{...})`)
    let pos_line = atom
        .lines()
        .find(|l| {
            l.contains("record_ctor")
                && l.contains("t_rec_Tok")
                && l.contains("{ field: 0u32, arg: 0u32 }, { field: 1u32, arg: 1u32 }, { field: 2u32, arg: 2u32 }, { field: 3u32, arg: 3u32 }, { field: 4u32, arg: 4u32 }")
        })
        .expect("the positional Tok ctor");
    assert!(
        pos_line.contains("field_args: [e")
            && atom.contains("expr e14 str_lit { type: t_Str; text: \"tail\" }"),
        "positional ctor arg promotes to t_Str: {pos_line}"
    );
    // static method call: Tok.new("fresh", 9) — literal to the Str formal
    assert!(
        atom.contains("str_lit { type: t_Str; text: \"fresh\" }"),
        "static-method Str formal promotes its literal: {atom}"
    );
}

#[test]
fn move_arguments_transfer_whole_roots_not_views() {
    // the combination surface keeps `.move` wrappers as argument VALUES
    // (CallBinding.moved is domain-forbidden): eval_args carries the move
    // wrapper itself, never a borrow_view of it; plain view params keep
    // the 771 borrow_view wrap.
    let c = compile_ok(&fixture("valid", "p01-combination.at"));
    let atom = &c.atom_text;
    for callee in ["d_eat", "d_consume"] {
        let call_line = atom
            .lines()
            .find(|l| l.contains("call") && l.contains(callee))
            .unwrap_or_else(|| panic!("call to {callee}: {atom}"));
        let arg_id: String = call_line
            .split("eval_args: [")
            .nth(1)
            .and_then(|rest| rest.split(']').next())
            .and_then(|s| s.trim().strip_prefix('e').map(|n| n.to_string()))
            .unwrap_or_else(|| panic!("arg id in {call_line}"));
        assert!(
            atom.contains(&format!("expr e{arg_id} move {{ type: t_rec_Tok;")),
            "{callee}'s argument is the move wrapper itself: {call_line}"
        );
    }
    // receiver view of the instance method stays a borrow
    assert!(
        atom.contains("borrow_view { type: t_rec_Tok;"),
        "view receiver discipline preserved: {atom}"
    );
}

// -- V-05: replacement and overlap acceptance ---------------------------------

#[test]
fn overlap_shapes_compile() {
    // r=r.clone() / r=transform(r) / consume through a move param whose
    // result rebuilds r / field replacement with a concat RHS reading the
    // old field — the front door accepts exactly these shapes (the
    // value/free-count oracles are T-04/T-05).
    let c = compile_ok(&fixture("valid", "p02-overlap.at"));
    assert!(
        c.atom_text.contains("record_clone"),
        "clone-assignment present: {}",
        c.atom_text
    );
    assert!(
        c.atom_text.contains("text_concat"),
        "field-replacement RHS concat present: {}",
        c.atom_text
    );
}

#[test]
fn copyable_record_keeps_value_copy_semantics() {
    // 771 value copy survives on the union surface: a copyable record
    // (no Str anywhere in its closure) transfers by plain assignment
    // without the resource-assign rejection.
    compile_ok(
        "type Meta { num int }\nfn main() int {\n    let p Meta = Meta { num: 1 }\n    var q Meta = p\n    q.num = 2\n    return p.num + q.num\n}\n",
    );
}

// -- frozen rejection faces ----------------------------------------------------

#[test]
fn bare_resource_assignment_rejects() {
    let ds = compile_err(&fixture("invalid", "i01-bare-resource-assign.at"));
    assert!(has(&ds, "verify.resource-assign"), "{:?}", codes(&ds));
}

#[test]
fn bare_resource_return_rejects() {
    let ds = compile_err(&fixture("invalid", "i05-bare-return-resource.at"));
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));
}

#[test]
fn partial_move_of_a_field_rejects() {
    let ds = compile_err(&fixture("invalid", "i02-partial-move-field.at"));
    assert!(has(&ds, "verify.partial-move"), "{:?}", codes(&ds));
}

#[test]
fn dead_branch_is_still_statically_checked() {
    // `if false` exempts nothing: the ownership rules are lexical static
    // checks, not runtime observations.
    let ds = compile_err(&fixture("invalid", "i07-nonexecuted-branch.at"));
    assert!(has(&ds, "verify.partial-move"), "{:?}", codes(&ds));
}

#[test]
fn user_declared_clone_method_rejects_as_duplicate() {
    let ds = compile_err(&fixture("invalid", "i03-user-clone-method.at"));
    assert!(has(&ds, "source.duplicate"), "{:?}", codes(&ds));
    let d = ds.iter().find(|d| d.code == "source.duplicate").unwrap();
    assert!(
        d.message.contains("builtin deep-clone"),
        "guidance names the builtin: {}",
        d.message
    );
}

#[test]
fn free_function_named_clone_stays_legal() {
    // methods bind on (receiver type, name); a FREE function named clone
    // lives in the function namespace and never collides with the builtin
    compile_ok(
        "type Tok { text Str; kind int }\nfn clone(n int) int { return n }\nfn main() int {\n    let t Tok = Tok { text: \"hi\", kind: clone(3) }\n    return t.kind\n}\n",
    );
}

#[test]
fn str_field_rejects_through_the_source_door() {
    // `str` parses and typechecks; verify owns the 774 D-03 rejection
    let ds = compile_err(&fixture("invalid", "i04-str-field.at"));
    assert!(has(&ds, "verify.field-borrow"), "{:?}", codes(&ds));
}

#[test]
fn nominal_records_do_not_interchange() {
    let ds = compile_err(&fixture("invalid", "i06-nominal-mismatch.at"));
    assert!(
        ds.iter().any(|d| d.code == "source.type-mismatch"
            && d.message.contains("record `A`")
            && d.message.contains("record `B`")),
        "nominal mismatch names both records: {:?}",
        ds.iter()
            .map(|d| (d.code.as_str(), d.message.as_str()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn move_param_on_copyable_record_rejects() {
    let ds = compile_err(&fixture("invalid", "i08-move-param-on-copyable.at"));
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));
}

#[test]
fn missing_dotmove_at_call_site_rejects() {
    // mode/marker symmetry (771 double gate): a `move` param takes the
    // whole root through an explicit `.move` argument
    let ds = compile_err(
        "type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    let c Tok = Tok { text: \"hi\", kind: 1 }\n    return consume(c)\n}\n",
    );
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));
}

// -- old-domain face preservation (frozen: zero acceptance change) -----------

#[test]
fn record_domain_clone_face_unchanged() {
    // N06 lock: the plain record profile keeps `type X has no method named
    // clone` — the builtin deep-clone binds on the combination surface only
    let ds = compile_record_source(
        "type Q { v int }\nfn main() int {\n    let q Q = Q(1)\n    let r Q = q.clone()\n    return r.v\n}\n",
        "q.at",
        &[],
    )
    .expect_err("record profile must reject .clone()");
    assert!(
        ds.iter()
            .any(|d| d.code == "source.undefined" && d.message.contains("no method named `clone`")),
        "{:?}",
        codes(&ds)
    );
}

#[test]
fn text_domain_move_face_unchanged() {
    // the plain text profile keeps its 770 front-door `.move` rule (a
    // named local) — the combination surface is where the wrapper stays
    // first-class and verify owns the partial-move oracle
    let ds = auto_ac_prototype::source::compile_text_source(
        "fn main() int {\n    let a int = 1\n    return (a + 1).move\n}\n",
        "t.at",
        &[],
    )
    .expect_err("text profile must reject .move on a sub-expression");
    assert!(has(&ds, "source.unsupported"), "{:?}", codes(&ds));
    assert_eq!(
        SourceProfile::of(TEXT_RECORD_SOURCE_PROFILE),
        Some(SourceProfile::TextRecord)
    );
}
