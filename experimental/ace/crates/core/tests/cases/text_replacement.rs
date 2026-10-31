//! PLAN-894 T-03 frontend tests: the Str LOCAL full-root replacement
//! (§5.1) at the verify layer — the admitted shapes (V-02) and the named
//! rejections (V-03). The fixtures run the real
//! parse → resolve → typecheck → adapter → bind → verify chain of BOTH
//! text doors (the plain text profile and the text-record combination
//! profile — the recipe front door); execution-level oracles
//! (free-once events, values) live in the AV/native tiers
//! (`text_source_894*`), the lowering fixes in T-04.

use ace_core::hir::HirDomain;
use ace_core::source::{compile_text_record_source, compile_text_source};

fn compile_ok_rec(src: &str) -> ace_core::source::CompiledSource {
    compile_text_record_source(src, "p.at", &[]).expect("combination compile must succeed")
}
fn compile_ok_text(src: &str) -> ace_core::source::CompiledSource {
    compile_text_source(src, "p.at", &[]).expect("text compile must succeed")
}
fn compile_err_rec(src: &str) -> Vec<ace_core::atom_text::Diagnostic> {
    compile_text_record_source(src, "p.at", &[]).expect_err("must be rejected")
}
fn has(ds: &[ace_core::atom_text::Diagnostic], code: &str) -> bool {
    ds.iter().any(|d| d.code == code)
}

/// Every scheduled plain-root (empty path) replacement across all bodies
/// of the module: (block id, stmt idx) pairs.
fn plain_replace_frees(c: &ace_core::source::CompiledSource) -> Vec<(u32, u32)> {
    let plan = c.checked.text_resources().expect("text plan present");
    let mut out = Vec::new();
    for bp in plan.bodies.values() {
        for ((bid, idx), bufs) in &bp.replace_frees {
            assert!(
                bufs.iter().all(|b| b.path.is_empty()),
                "plain-root replacements only in these fixtures"
            );
            out.push((*bid, *idx as u32));
        }
    }
    out
}

// -- V-02: the admitted replacement shapes ----------------------------------

#[test]
fn t03_shape_concat_replacement_admitted_with_plan() {
    // The frozen ACE-M1 T03 body shape: RHS reads the OLD value.
    let src = "\
fn main() int {
    let a Str = \"ab\"
    let b Str = a.clone()
    var c Str = b.move
    c = c + \"中\"
    return a.len() * 10 + c.len()
}
";
    let c = compile_ok_rec(src);
    assert_eq!(c.checked.domain(), HirDomain::TextRecord);
    let frees = plain_replace_frees(&c);
    assert_eq!(frees.len(), 1, "exactly one replacement free scheduled");
}

#[test]
fn replacement_shapes_literal_clone_return_and_cross_move() {
    let src = "\
fn make() Str { return \"xy\" }
fn main() int {
    var a Str = \"1\"
    a = \"22\"
    a = a.clone()
    a = make()
    var b Str = \"b\"
    a = b.move
    return a.len() * 10
}
";
    let c = compile_ok_rec(src);
    assert_eq!(plain_replace_frees(&c).len(), 4, "four replacements");
}

#[test]
fn replacement_empty_string_and_branch_loop_repetition() {
    let src = "\
fn main() int {
    var c Str = \"x\"
    var n int = 0
    for n < 3 {
        c = c + \"!\"
        n = n + 1
    }
    if c.len() > 3 {
        c = \"\"
    } else {
        c = c.clone()
    }
    return c.len()
}
";
    let c = compile_ok_rec(src);
    assert!(
        plain_replace_frees(&c).len() >= 3,
        "loop + branch replacements"
    );
}

#[test]
fn text_profile_same_local_rules() {
    // §5.1: text and text-record share the SAME local rule.
    let src = "\
fn main() int {
    var c Str = \"ab\"
    c = c + \"中\"
    return c.len()
}
";
    let c = compile_ok_text(src);
    assert_eq!(c.checked.domain(), HirDomain::Text);
    assert_eq!(plain_replace_frees(&c).len(), 1);
}

#[test]
fn rhs_read_of_old_value_and_borrowed_view_temporary_is_legal() {
    // The RHS's own view of the target is statement-temporary: it does
    // NOT block the replacement (only a PRE-EXISTING borrow does).
    let src = "\
fn main() int {
    var c Str = \"ab\"
    c = c + c.clone()
    return c.len()
}
";
    compile_ok_rec(src);
}

// -- V-03: the named rejections ----------------------------------------------

#[test]
fn bare_owned_copy_rhs_rejected_ownership_required() {
    let src = "\
fn main() int {
    var c Str = \"a\"
    let d Str = c.clone()
    c = d
    return c.len()
}
";
    let ds = compile_err_rec(src);
    assert!(has(&ds, "verify.ownership-required"), "{ds:?}");
}

#[test]
fn self_move_rhs_rejected_reassign_conflict() {
    let src = "\
fn main() int {
    var c Str = \"a\"
    c = c.move
    return 0
}
";
    let ds = compile_err_rec(src);
    assert!(has(&ds, "verify.reassign-conflict"), "{ds:?}");
}

#[test]
fn rhs_consuming_target_via_call_rejected_reassign_conflict() {
    let src = "\
fn rebody(move s Str) Str { return s }
fn main() int {
    var c Str = \"abc\"
    c = rebody(c.move)
    return 0
}
";
    let ds = compile_err_rec(src);
    assert!(has(&ds, "verify.reassign-conflict"), "{ds:?}");
}

#[test]
fn moved_out_target_rebuild_rejected_use_after_move() {
    // Covers every source-level dead cause: a moved root is rebuilt only
    // by a fresh binding (drop/scope-exit deaths are HIR-fixture faces —
    // the source profile has no `drop` statement and block-lexical scoping
    // makes scope-exit reassignment unparseable).
    let src = "\
fn consume(move s Str) int { return s.len() }
fn main() int {
    var c Str = \"abc\"
    let n int = consume(c.move)
    c = \"x\"
    return n
}
";
    let ds = compile_err_rec(src);
    assert!(has(&ds, "verify.use-after-move"), "{ds:?}");
}

#[test]
fn replacement_under_live_borrow_rejected_mut_conflict() {
    let src = "\
fn main() int {
    var c Str = \"abc\"
    let v str = c.to(str)
    c = \"x\"
    return v.len()
}
";
    let ds = compile_err_rec(src);
    assert!(has(&ds, "verify.mut-conflict"), "{ds:?}");
}

#[test]
fn let_target_still_immutable_structural_gate() {
    let src = "\
fn main() int {
    let c Str = \"a\"
    c = \"b\"
    return c.len()
}
";
    let ds = compile_err_rec(src);
    assert!(has(&ds, "verify.immutable-write"), "{ds:?}");
}

#[test]
fn view_target_face_unchanged() {
    // A view (str) target still cannot take an owned RHS: the exact-match
    // type rule rejects it before the resource walk (the HIR-level
    // borrow-escape invariant is exercised by the atom-fixture suites).
    let src = "\
fn make() Str { return \"tmp\" }
fn main() int {
    var v str = \"a\"
    v = make()
    return v.len()
}
";
    let ds = compile_err_rec(src);
    assert!(has(&ds, "source.type-mismatch"), "{ds:?}");
}

#[test]
fn cross_move_source_dead_afterwards() {
    let src = "\
fn main() int {
    var a Str = \"abc\"
    var b Str = \"x\"
    a = b.move
    let n int = a.len()
    let m int = b.len()
    return n + m
}
";
    let ds = compile_err_rec(src);
    assert!(has(&ds, "verify.use-after-move"), "{ds:?}");
}

#[test]
fn replacement_after_rhs_failure_keeps_target_live() {
    // A FOLLOWING statement still sees the target live (the failure
    // boundary is the runtime's; statically the replacement completes).
    let src = "\
fn main() int {
    var c Str = \"ab\"
    c = c + \"中\"
    c = c + \"!\"
    return c.len()
}
";
    let c = compile_ok_rec(src);
    assert_eq!(plain_replace_frees(&c).len(), 2);
}
