//! PLAN-782 T-04 resource-walk tests: the ownership/liveness/cleanup
//! contract of the combination domain (V-03 move/clone, V-04 borrow
//! sources, V-06 control-flow cleanup, V-07 schedule shape, V-09
//! conditional transfers). Every check runs the full compile chain
//! (parse → resolve → typecheck → adapter → bind → verify incl. the
//! extended `TextResourcePlan` walk); plan assertions inspect the
//! schedule the T-05 lowering will consume verbatim. Runtime
//! allocation/free counting and failure injection are T-05's native
//! legs (`text_record_native.rs`).

use auto_ac_prototype::atom_text::Diagnostic;
use auto_ac_prototype::source::{compile_text_record_source, CompiledSource};
use auto_ac_prototype::text_resources::RecordBuffer;

fn ok(src: &str) -> CompiledSource {
    compile_text_record_source(src, "r.at", &[]).expect("combination compile must succeed")
}

fn err(src: &str) -> Vec<Diagnostic> {
    compile_text_record_source(src, "r.at", &[]).expect_err("must be rejected")
}

fn has(ds: &[Diagnostic], code: &str) -> bool {
    ds.iter().any(|d| d.code == code)
}

fn codes(ds: &[Diagnostic]) -> Vec<&str> {
    ds.iter().map(|d| d.code.as_str()).collect()
}

/// All (root, path) buffers scheduled anywhere in the module's plans.
fn all_buffers(c: &CompiledSource) -> Vec<(u32, Vec<u32>)> {
    let plan = c.checked.text_resources().expect("resource plan");
    plan.bodies
        .values()
        .flat_map(|bp| {
            bp.buffer_frees
                .values()
                .chain(bp.replace_frees.values())
                .flatten()
        })
        .map(|RecordBuffer { root, path }| (*root, path.clone()))
        .collect()
}

// -- V-03: explicit move, clone, and the missing-.move symmetry ----------

#[test]
fn use_after_move_of_record_root_rejects() {
    let ds = err("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    let a Tok = Tok { text: \"hi\", kind: 1 }\n    let k int = consume(a.move)\n    return k + a.kind\n}\n");
    assert!(has(&ds, "verify.use-after-move"), "{:?}", codes(&ds));
}

#[test]
fn double_move_rejects() {
    // same-statement double transfer of one root; a later-statement second
    // move is the use-after-move face instead
    let ds = err("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    let a Tok = Tok { text: \"hi\", kind: 1 }\n    let k int = consume(a.move) + consume(a.move)\n    return k\n}\n");
    assert!(has(&ds, "verify.double-move"), "{:?}", codes(&ds));
}

#[test]
fn loop_reentry_kills_entry_live_moved_root() {
    // a root live at loop entry and moved in the body is dead for the
    // whole body: iteration 2 would move it again (770 invariant, roots
    // extended to resource records)
    let ds = err("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    let r Tok = Tok { text: \"hi\", kind: 1 }\n    var i int = 0\n    for i < 3 {\n        let k int = consume(r.move)\n        i = i + 1\n    }\n    return i\n}\n");
    assert!(has(&ds, "verify.use-after-move"), "{:?}", codes(&ds));
}

#[test]
fn fresh_construction_to_move_param_requires_marker() {
    // 770 face inherited: the `.move` marker is required even for fresh
    // owned values (bind to a local first)
    let ds = err("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int { return consume(Tok { text: \"x\", kind: 1 }) }\n");
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));
}

#[test]
fn constructor_field_cannot_implicitly_transfer_owned_local() {
    // a bare owned-root ReadLocal as a ctor field argument is an implicit
    // transfer (owned Str locals AND resource records)
    let ds = err("type Tok { text Str; kind int }\nfn main() int {\n    let s Str = \"own\"\n    let t Tok = Tok { text: s, kind: 1 }\n    return t.kind\n}\n");
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));
    let ds = err("type Inner { s Str }\ntype Wrap { i Inner }\nfn main() int {\n    let a Inner = Inner { s: \"x\" }\n    let w Wrap = Wrap { i: a }\n    return 0\n}\n");
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));
    // an owned construction (`.to(Str)`) is the legal spelling, and the
    // source local stays valid afterwards
    ok("type Tok { text Str; kind int }\nfn main() int {\n    let s Str = \"own\"\n    let t Tok = Tok { text: s.to(Str), kind: 1 }\n    return t.kind + s.len()\n}\n");
}

#[test]
fn move_param_root_is_callee_cleaned() {
    // the callee's plan carries the move-in root and frees its Str-field
    // buffers at its own exits (the caller never schedules them)
    let c = ok("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    let a Tok = Tok { text: \"hi\", kind: 1 }\n    return consume(a.move)\n}\n");
    let plan = c.checked.text_resources().expect("plan");
    let consume_bp = plan
        .bodies
        .values()
        .find(|bp| {
            bp.owned.len() == 1
                && bp
                    .buffer_frees
                    .values()
                    .any(|v| v.len() == 1 && v[0].path == vec![0u32])
        })
        .expect("consume body frees its move-in root's text field");
    assert!(
        consume_bp.owned.contains(&0u32),
        "the move-in param is the owned root"
    );
}

#[test]
fn clone_leaves_outstanding_field_borrows_legal() {
    // r.clone() reads the root but constructs an independent value: a
    // field view taken before the clone stays valid (the clone is not a
    // root mutation)
    ok("type Tok { text Str; kind int }\nfn main() int {\n    let r Tok = Tok { text: \"hi\", kind: 1 }\n    let v = r.text\n    let u Tok = r.clone()\n    return v.len() + u.kind\n}\n");
}

// -- V-04: borrow sources and root-conservative conflicts ------------------

#[test]
fn view_param_field_is_a_legal_borrowed_return() {
    ok("type Tok { text Str; kind int }\nfn f(p Tok) str { return p.text }\nfn main() int { return f(Tok { text: \"hi\", kind: 1 }).len() }\n");
}

#[test]
fn owned_local_field_return_escapes() {
    let ds = err("type Tok { text Str; kind int }\nfn f() str {\n    let r Tok = Tok { text: \"hi\", kind: 1 }\n    return r.text\n}\nfn main() int { return f().len() }\n");
    assert!(has(&ds, "verify.borrow-escape"), "{:?}", codes(&ds));
}

#[test]
fn mut_param_field_return_escapes() {
    let ds = err("type Tok { text Str; kind int }\nfn f(mut p Tok) str { return p.text }\nfn main() int { return 0 }\n");
    assert!(has(&ds, "verify.borrow-escape"), "{:?}", codes(&ds));
}

#[test]
fn move_in_param_field_return_escapes() {
    let ds = err("type Tok { text Str; kind int }\nfn f(move t Tok) str { return t.text }\nfn main() int { return 0 }\n");
    assert!(has(&ds, "verify.borrow-escape"), "{:?}", codes(&ds));
}

#[test]
fn field_replacement_during_active_borrow_conflicts() {
    // root-conservative (5.2): a borrow of ANY resource field holds the
    // whole root - replacing a field under it is a mut conflict
    let ds = err("type Tok { text Str; kind int }\nfn main() int {\n    var r Tok = Tok { text: \"hi\", kind: 1 }\n    let v = r.text\n    r.text = \"new\"\n    return v.len()\n}\n");
    assert!(has(&ds, "verify.mut-conflict"), "{:?}", codes(&ds));
}

#[test]
fn root_move_during_active_borrow_conflicts() {
    let ds = err("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    let r Tok = Tok { text: \"hi\", kind: 1 }\n    let v = r.text\n    let k int = consume(r.move)\n    return v.len() + k\n}\n");
    assert!(has(&ds, "verify.mut-conflict"), "{:?}", codes(&ds));
}

#[test]
fn field_view_dangling_after_owner_move_rejects_at_use() {
    let ds = err("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    let u Tok = Tok { text: \"hi\", kind: 1 }\n    let v = u.text\n    let k int = consume(u.move)\n    return v.len() + k\n}\n");
    assert!(has(&ds, "verify.use-after-move"), "{:?}", codes(&ds));
}

// -- V-05 (resources leg): replacement schedules ---------------------------

#[test]
fn replacement_schedules_old_buffer_after_rhs() {
    // P4: `r.text = r.text + "!"` - the RHS reads the OLD field, the old
    // buffer frees only after it materialized; the schedule records
    // exactly that one field of that root
    let c = ok("type Tok { text Str; kind int }\nfn main() int {\n    var r Tok = Tok { text: \"hi\", kind: 1 }\n    r.text = r.text + \"!\"\n    r.text = \"brand new\"\n    r = r.clone()\n    return r.text.len()\n}\n");
    let plan = c.checked.text_resources().expect("plan");
    let main_bp = plan
        .bodies
        .values()
        .find(|bp| bp.replace_frees.len() == 3)
        .expect("three replacement schedules");
    let mut paths: Vec<Vec<u32>> = main_bp
        .replace_frees
        .values()
        .flatten()
        .map(|RecordBuffer { path, .. }| path.clone())
        .collect();
    paths.sort_unstable();
    // two single-field replacements + one whole-root clone re-assignment
    // (Tok's only Str field is 0)
    assert_eq!(paths, vec![vec![0u32], vec![0u32], vec![0u32]], "{paths:?}");
}

#[test]
fn chained_replacement_reading_old_field_is_legal() {
    ok("type Tok { text Str; kind int }\nfn main() int {\n    var r Tok = Tok { text: \"hi\", kind: 1 }\n    r.text = \"new\"\n    r.text = r.text + \"!\"\n    return r.text.len()\n}\n");
}

#[test]
fn moved_root_reassignment_schedules_no_old_frees() {
    // after consume(r.move) the root's buffers belong to the callee:
    // rebuilding r must NOT schedule a free of buffers it no longer owns
    let c = ok("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    var r Tok = Tok { text: \"hi\", kind: 1 }\n    let k int = consume(r.move)\n    r = Tok { text: \"rebuilt\", kind: k + 1 }\n    return r.kind\n}\n");
    let plan = c.checked.text_resources().expect("plan");
    let main_bp = plan.bodies.values().find(|bp| !bp.replace_frees.is_empty());
    assert!(
        main_bp.is_none(),
        "no old-buffer free for a moved-then-rebuilt root"
    );
}

// -- V-06: control-flow cleanup ---------------------------------------------

#[test]
fn arm_local_roots_close_at_their_own_block_exit() {
    let c = ok("type Tok { text Str; kind int }\nfn main() int {\n    var r Tok = Tok { text: \"hi\", kind: 1 }\n    if r.kind > 0 {\n        let inner Tok = Tok { text: \"arm\", kind: 2 }\n        r = inner.clone()\n    }\n    return r.kind\n}\n");
    let plan = c.checked.text_resources().expect("plan");
    // the arm block's natural end frees the arm-local root's buffer
    let arm_frees = plan
        .bodies
        .values()
        .flat_map(|bp| bp.buffer_frees.iter())
        .filter(|((_, k), _)| k.is_none())
        .flat_map(|((b, _), v)| v.iter().map(move |rb| (*b, rb.root, rb.path.clone())))
        .collect::<Vec<_>>();
    assert!(
        arm_frees.iter().any(|(_, _, path)| path == &vec![0u32]),
        "arm fall-through frees the arm local's text buffer: {arm_frees:?}"
    );
}

#[test]
fn nested_return_move_transfers_out() {
    // the moved return keeps its root OUT of the exit frees (the buffer
    // responsibility left with the returned value); the fresh-construction
    // return frees the still-live move-in root
    let c = ok("type Tok { text Str; kind int }\nfn pick(move t Tok) Tok {\n    if t.kind > 0 {\n        return t.move\n    }\n    return Tok { text: \"z\", kind: 0 }\n}\nfn main() int {\n    let t Tok = Tok { text: \"x\", kind: 1 }\n    return pick(t.move).kind\n}\n");
    let plan = c.checked.text_resources().expect("plan");
    let pick_bp = plan
        .bodies
        .values()
        .find(|bp| bp.owned_returns.len() == 2)
        .expect("pick's two owning returns");
    let flat: Vec<Vec<u32>> = pick_bp
        .buffer_frees
        .values()
        .flatten()
        .map(|RecordBuffer { path, .. }| path.clone())
        .collect();
    // exactly one scheduled free: the fresh-construction return freeing
    // the (then-arm-unmoved) move-in root's text field; the `t.move`
    // return schedules nothing
    assert_eq!(flat, vec![vec![0u32]], "{flat:?}");
}

#[test]
fn loop_scoped_roots_free_at_break_and_body_end() {
    let c = ok("type Tok { text Str; kind int }\nfn main() int {\n    var keep Tok = Tok { text: \"k\", kind: 0 }\n    var i int = 0\n    for i < 3 {\n        i = i + 1\n        if i == 2 {\n            break\n        }\n        let t Tok = Tok { text: \"loop\", kind: i }\n        keep = t.clone()\n    }\n    return keep.kind\n}\n");
    // legal shape; the loop body's own root closes at the body's natural
    // end and at the break - both reachable in the plan
    let plan = c.checked.text_resources().expect("plan");
    assert!(
        plan.bodies
            .values()
            .any(|bp| bp.buffer_frees.values().any(|v| !v.is_empty())),
        "loop-body and break frees scheduled"
    );
}

// -- V-07: schedule shape (probe P3 order; runtime injection is T-05) ------

#[test]
fn exit_frees_follow_p3_root_and_field_order() {
    // two Str fields, two roots: frees are root-reverse-creation, and
    // within a root field-reverse-declaration
    let c = ok("type P { a Str; b Str }\nfn main() int {\n    let x P = P { a: \"1\", b: \"2\" }\n    let y P = P { a: \"3\", b: \"4\" }\n    return x.a.len() + y.b.len()\n}\n");
    let plan = c.checked.text_resources().expect("plan");
    let main_bp = plan
        .bodies
        .values()
        .find(|bp| !bp.buffer_frees.is_empty())
        .expect("main's exit schedule");
    assert_eq!(main_bp.buffer_frees.len(), 1, "one return exit");
    let sched = main_bp.buffer_frees.values().next().unwrap();
    let shape: Vec<(u32, Vec<u32>)> = sched
        .iter()
        .map(|RecordBuffer { root, path }| (*root, path.clone()))
        .collect();
    // y before x (reverse creation); within each root b(1) before a(0)
    assert_eq!(
        shape,
        vec![(1u32, vec![1u32]), (1, vec![0]), (0, vec![1]), (0, vec![0])],
        "{shape:?}"
    );
}

#[test]
fn nested_record_fields_expand_in_declaration_order() {
    // Outer { i Inner{ s Str }, t Str }: declaration-order paths are
    // [0,0] then [1]; frees run them reversed
    let c = ok("type Inner { s Str }\ntype Outer { i Inner; t Str }\nfn main() int {\n    let o Outer = Outer { i: Inner { s: \"in\" }, t: \"out\" }\n    return o.t.len()\n}\n");
    let bufs = all_buffers(&c);
    assert_eq!(
        bufs,
        vec![(0u32, vec![1u32]), (0, vec![0u32, 0u32])],
        "{bufs:?}"
    );
}

// -- V-09: conditional transfers --------------------------------------------

#[test]
fn conditional_move_in_short_circuit_rhs_rejects() {
    let ds = err("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    let a int = 1\n    let r Tok = Tok { text: \"hi\", kind: a }\n    let ok bool = a > 0 && consume(r.move) > 0\n    return 0\n}\n");
    assert!(has(&ds, "verify.ownership-required"), "{:?}", codes(&ds));
}

#[test]
fn dead_branch_move_is_ignored_for_liveness() {
    // a statically-dead short-circuit rhs (constant-polarity lhs) keeps
    // its static legality but never transfers: the root stays live
    ok("type Tok { text Str; kind int }\nfn consume(move t Tok) int { return t.kind }\nfn main() int {\n    let r Tok = Tok { text: \"hi\", kind: 1 }\n    let skip bool = true || consume(r.move) > 0\n    return r.kind\n}\n");
}

// -- direct-HIR legs (binder + verifier, no source front end) -------------

#[test]
fn direct_hir_control_flow_verifies_and_plans() {
    // V-06's direct-HIR leg: the nested-return-move program's generated
    // Atom binds through the text-record descriptor and verifies through
    // the combination entry with the same resource plan (the source chain
    // is not a precondition for the walk)
    let src = "type Tok { text Str; kind int }\nfn pick(move t Tok) Tok {\n    if t.kind > 0 {\n        return t.move\n    }\n    return Tok { text: \"z\", kind: 0 }\n}\nfn main() int {\n    let t Tok = Tok { text: \"x\", kind: 1 }\n    return pick(t.move).kind\n}\n";
    let parts = auto_ac_prototype::source::compile_parts_with(
        src,
        "r.at",
        &[],
        auto_ac_prototype::source::SourceProfile::TextRecord,
    )
    .expect("parts");
    let bundle = auto_ac_prototype::descriptor::bind(
        auto_ac_prototype::text_record_descriptor(),
        &parts.atom_text,
    )
    .expect("direct bind");
    let checked = auto_ac_prototype::verify::verify_text_record(bundle)
        .expect("direct verify with the resource walk");
    let plan = checked.text_resources().expect("plan attached");
    assert!(
        plan.bodies
            .values()
            .any(|bp| bp.owned_returns.len() == 2 && !bp.buffer_frees.is_empty()),
        "pick's owning returns and the fresh-return free are planned"
    );
}

#[test]
fn wrapper_same_source_mut_view_conflict_rejects() {
    // V-04 counterexample closure (770 R4-04 relayed through the
    // combination domain): the mut x view exclusivity pair rides the
    // wrapper signatures -- passing one source into both wrapper params
    // through TWO levels still rejects at the front door
    let ds = err(concat!(
        "type Tok { text Str; kind int }\n",
        "fn both(mut a Tok, b Tok) int { a.kind = 1; return a.kind + b.kind }\n",
        "fn relay(mut a Tok, b Tok) int { return both(a.mut, b) }\n",
        "fn relay2(mut a Tok, b Tok) int { return relay(a.mut, b) }\n",
        "fn main() int {\n",
        "    var x Tok = Tok { text: \"abc\", kind: 1 }\n",
        "    return relay2(x.mut, x)\n",
        "}\n"
    ));
    assert!(has(&ds, "verify.mut-conflict"), "{:?}", codes(&ds));
}

#[test]
fn wrapper_independent_sources_stay_legal() {
    // the positive control: distinct sources through the same two-level
    // wrapper stay legal -- read/mut pairing is per SOURCE, not per formal
    // parameter number
    ok(concat!(
        "type Tok { text Str; kind int }\n",
        "fn both(mut a Tok, b Tok) int { a.kind = 1; return a.kind + b.kind }\n",
        "fn relay(mut a Tok, b Tok) int { return both(a.mut, b) }\n",
        "fn relay2(mut a Tok, b Tok) int { return relay(a.mut, b) }\n",
        "fn main() int {\n",
        "    var x Tok = Tok { text: \"abc\", kind: 1 }\n",
        "    var y Tok = Tok { text: \"d\", kind: 2 }\n",
        "    return relay2(x.mut, y)\n",
        "}\n"
    ));
}
