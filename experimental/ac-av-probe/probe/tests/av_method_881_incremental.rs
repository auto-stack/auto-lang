//! PLAN-881 T-04 incremental tier (`av_method_881_incremental`): V-05
//! (method body edits round-trip, interface dependencies, restart), V-11
//! (true single-method frontend: sibling identity preservation, growing
//! bodies with UTF-8, candidate atomicity) and V-12 (snapshot lifetime:
//! the pinned old snapshot keeps its code; the new one runs the edit) —
//! over BOTH sessions (data `DataIncrementalSession`, owned
//! `OwnedFunctionSession`).
//!
//! Oracles: 42/43 round trips; production counters move ONLY the edited
//! unit; a failed candidate leaves the previous program runnable.

use auto_av_core::contract::Value;
use auto_av_probe::data_session::DataIncrementalSession;
use auto_av_probe::owned_session::OwnedFunctionSession;

const MIXED: &str = "\
type Counter {
    value int
    static fn new(n int) Counter { return Counter(n) }
    fn read() int { return .value }
    fn bump() int { return .value + 1 }
}
type Other {
    k int
    fn read() int { return .k }
    static fn mk(n int) Other { return Other { k: n } }
}
fn free_read(c Counter) int { return c.value }
fn main() int {
    let x Counter = Counter.new(42)
    return x.read()
}
";

// ---------------------------------------------------------------------------
// V-05 / V-11: the method body edit round-trip on the DATA session
// ---------------------------------------------------------------------------

#[test]
fn v05_method_body_round_trip_42_43_42() {
    let mut sess = DataIncrementalSession::warmup(MIXED, "d_main", &[]).expect("warmup");
    assert_eq!(sess.run().expect("run"), Value::I32(42));

    // 42 -> 43: ONLY the edited method's unit re-produces.
    let edited = MIXED.replace(
        "fn read() int { return .value }",
        "fn read() int { return .value + 1 }",
    );
    let r1 = sess.edit_body(&edited, "m.Counter.read").expect("edit 1");
    assert_eq!(r1.facts.receipt.real_body_plans, 1, "one real body");
    assert_eq!(sess.run().expect("run 1"), Value::I32(43));
    let lowered = r1.lowered.clone();
    assert_eq!(lowered.get("lower:m.Counter.read"), Some(&2));
    for other in [
        "lower:m.Counter.new",
        "lower:m.Counter.bump",
        "lower:m.Other.read",
        "lower:m.Other.mk",
        "lower:d_free_read",
        "lower:d_main",
    ] {
        assert_eq!(lowered.get(other), Some(&1), "{other} NOT re-lowered");
    }

    // 43 -> 42: back to the original body — same precision.
    let r2 = sess.edit_body(MIXED, "m.Counter.read").expect("edit 2");
    assert_eq!(sess.run().expect("run 2"), Value::I32(42));
    assert_eq!(r2.lowered.get("lower:m.Counter.read"), Some(&3));
    for other in ["lower:m.Counter.new", "lower:d_main"] {
        assert_eq!(r2.lowered.get(other), Some(&1), "{other} still untouched");
    }
}

#[test]
fn v11_sibling_identity_preserved_across_method_edit() {
    let mut sess = DataIncrementalSession::warmup(MIXED, "d_main", &[]).expect("warmup");
    let before_hir = sess.hir_identity_all();
    let before_code = sess.identity_all();

    let edited = MIXED.replace(
        "fn bump() int { return .value + 1 }",
        "fn bump() int { return .value + 2 }",
    );
    let _ = sess.edit_body(&edited, "m.Counter.bump").expect("edit bump");

    let after_hir = sess.hir_identity_all();
    let after_code = sess.identity_all();
    // Same-type sibling, same-NAME method on ANOTHER type, the free
    // caller and main all keep BOTH identities.
    for def in [
        "m.Counter.new",
        "m.Counter.read",
        "m.Other.read",
        "m.Other.mk",
        "d_free_read",
        "d_main",
    ] {
        assert_eq!(
            after_hir.get(def),
            before_hir.get(def),
            "HIR identity moved for `{def}`"
        );
        assert_eq!(
            after_code.get(def),
            before_code.get(def),
            "code identity moved for `{def}`"
        );
    }
    assert_ne!(
        after_hir.get("m.Counter.bump"),
        before_hir.get("m.Counter.bump"),
        "the edited method's HIR object was replaced"
    );
}

#[test]
fn v11_growing_utf8_body_edits_keep_coordinates() {
    let mut sess = DataIncrementalSession::warmup(MIXED, "d_main", &[]).expect("warmup");
    // 42 -> 123: a much longer body with a UTF-8 COMMENT carrying braces
    // (the record profile has no string literals; braces in comments are
    // the coordinate-tracking hazard this edit exercises).
    let long_body = "fn read() int { // 注释 A中 { }\n        if 1 != 2 { return .value + 81 }\n        return .value }";
    let s1 = MIXED.replace("fn read() int { return .value }", long_body);
    sess.edit_body(&s1, "m.Counter.read").expect("edit long");
    assert_eq!(sess.run().expect("run long"), Value::I32(123));

    // Then 123 -> 7: a SHORT body — the coordinates tracked the shrink.
    let s2 = s1.replace(long_body, "fn read() int { return 7 }");
    sess.edit_body(&s2, "m.Counter.read").expect("edit short");
    assert_eq!(sess.run().expect("run short"), Value::I32(7));

    // And a THIRD edit of a DIFFERENT method still lands precisely.
    let s3 = s2.replace(
        "fn bump() int { return .value + 1 }",
        "fn bump() int { return .value + 5 }",
    );
    let r3 = sess.edit_body(&s3, "m.Counter.bump").expect("edit bump after shifts");
    assert_eq!(r3.lowered.get("lower:m.Counter.read"), Some(&3), "read untouched by the bump edit");
    // main still calls read() -> 7.
    assert_eq!(sess.run().expect("run 3"), Value::I32(7));
}

#[test]
fn v11_failed_candidate_installs_nothing() {
    let mut sess = DataIncrementalSession::warmup(MIXED, "d_main", &[]).expect("warmup");
    let before_code = sess.identity_all();
    // A body that fails the frontend (type error) — the session must Err
    // and stay exactly at its pre-edit state.
    let bad = MIXED.replace(
        "fn read() int { return .value }",
        "fn read() int { return .value + true }",
    );
    assert!(sess.edit_body(&bad, "m.Counter.read").is_err(), "bad body rejected");
    assert_eq!(sess.run().expect("old program still runs"), Value::I32(42));
    assert_eq!(sess.identity_all(), before_code, "no identity moved");
}

#[test]
fn v11_signature_edit_is_not_the_fast_path() {
    let mut sess = DataIncrementalSession::warmup(MIXED, "d_main", &[]).expect("warmup");
    // A SIGNATURE change (param added) must not ride the single-body
    // channel: it is a controlled non-fast rejection.
    let sig = MIXED.replace(
        "fn read() int { return .value }",
        "fn read(k int) int { return .value + k }",
    );
    let err = sess
        .edit_body(&sig, "m.Counter.read")
        .expect_err("signature edits are the non-fast path");
    assert!(err.contains("non-fast"), "got: {err}");
    assert_eq!(sess.run().expect("unchanged"), Value::I32(42));
}

// ---------------------------------------------------------------------------
// V-05: interface dependencies (apply_source channel)
// ---------------------------------------------------------------------------

#[test]
fn v05_type_shape_change_invalidates_method_units_precisely() {
    let mut sess = DataIncrementalSession::warmup(MIXED, "d_main", &[]).expect("warmup");
    let before_hir = sess.hir_identity_all();

    // Add a FIELD to Counter: every unit mentioning Counter re-builds;
    // Other's untouched units keep both identities.
    let grown = MIXED.replace(
        "type Counter {\n    value int",
        "type Counter {\n    value int\n    extra int",
    );
    // The constructor initializes the new field (a valid grown source).
    let grown = grown.replace(
        "static fn new(n int) Counter { return Counter(n) }",
        "static fn new(n int) Counter { return Counter { value: n, extra: 0 } }",
    );
    let rebuilt = sess.apply_source(&grown).expect("apply");
    let after_hir = sess.hir_identity_all();
    // The Counter family + free_read (mentions Counter) + main re-build.
    for def in [
        "m.Counter.new",
        "m.Counter.read",
        "m.Counter.bump",
        "d_free_read",
        "d_main",
    ] {
        assert!(
            rebuilt.contains(&def.to_string()),
            "`{def}` invalidated by the Counter shape change: {rebuilt:?}"
        );
    }
    // Other's units survive untouched.
    for def in ["m.Other.read", "m.Other.mk"] {
        assert_eq!(
            after_hir.get(def),
            before_hir.get(def),
            "`{def}` untouched by an unrelated type's change"
        );
    }
    // Constructor updated — a fresh construction sets the new field.
    assert_eq!(sess.run().expect("run after growth"), Value::I32(42));
}

#[test]
fn v05_restart_fresh_session_reruns() {
    // A FRESH session over the (edited) source re-decodes and re-verifies
    // everything — restart never trusts prior residents.
    let edited = MIXED.replace(
        "fn read() int { return .value }",
        "fn read() int { return .value + 1 }",
    );
    let mut sess = DataIncrementalSession::warmup(&edited, "d_main", &[]).expect("warmup");
    assert_eq!(sess.run().expect("run"), Value::I32(43));
    let _ = &mut sess;
    // And the identity maps name every method resident.
    let ids = sess.identity_all();
    for def in ["m.Counter.new", "m.Counter.read", "m.Other.read", "d_main"] {
        assert!(ids.contains_key(def), "resident `{def}` missing");
    }
}

// ---------------------------------------------------------------------------
// V-12: snapshot lifetime — the pinned old snapshot keeps its code
// ---------------------------------------------------------------------------

#[test]
fn v12_old_snapshot_pin_keeps_old_code() {
    let mut sess = DataIncrementalSession::warmup(MIXED, "d_main", &[]).expect("warmup");
    assert_eq!(sess.run().expect("run"), Value::I32(42));

    // PIN the current snapshot (an external observer holding the Arc).
    let pinned = sess.snapshot_arc();

    let edited = MIXED.replace(
        "fn read() int { return .value }",
        "fn read() int { return .value + 1 }",
    );
    sess.edit_body(&edited, "m.Counter.read").expect("edit");
    assert_eq!(sess.run().expect("new"), Value::I32(43), "the session runs the new code");

    // The PINNED (old) snapshot still executes the OLD program — the
    // resident code it captured was not mutated in place.
    let old = {
        let (out, _) = auto_av_core::av::execute_program_units(
            &pinned,
            &auto_av_core::av::ExecLimits::default(),
            &auto_av_core::av::CancelFlag::new(false),
        );
        out
    };
    match old {
        auto_av_core::contract::ExecOutcome::Success { value: Value::I32(42), .. } => {}
        other => panic!("pinned old snapshot must still return 42, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// V-05/V-11 (owned profile): the method single-body channel
// ---------------------------------------------------------------------------

const OWNED_SRC: &str = r#"type Tok {
    label Str
    n int
    static fn make(move s Str, k int) Tok { return Tok { label: s.move, n: k } }
    fn score() int { return .label.len_bytes() * 4 + .n }
}
fn main() int {
    let t = Tok.make("A中", 5)
    return t.score()
}
"#;

#[test]
fn v05_owned_method_body_edit_single_channel() {
    let mut sess = OwnedFunctionSession::warmup(OWNED_SRC, "d_main", &[]).expect("warmup");
    assert_eq!(sess.run_i32(), 21);

    // Body edit inside the METHOD: the single-body channel (no whole
    // frontend round), production isolated to the method's unit.
    let edited = OWNED_SRC.replace(
        "fn score() int { return .label.len_bytes() * 4 + .n }",
        "fn score() int { return .label.len_bytes() * 4 + .n + 1 }",
    );
    // Replace the body tail (`+ .n }` -> `+ .n + 1 }`) via a byte range
    // fully inside the method's declaration.
    let tail = OWNED_SRC.find("+ .n }").expect("tail");
    let inv = sess
        .edit(tail as u64, (tail + 6) as u64, "+ .n + 1 }".as_bytes())
        .expect("owned method body edit");
    assert!(
        inv.iter().all(|d| d.contains("score") || d == "d_main"),
        "invalidated set is the method (+ importers): {inv:?}"
    );
    assert_eq!(sess.run_i32(), 22, "edited method = 22");
    assert_eq!(sess.last_channel(), auto_av_probe::owned_session::EditChannel::SingleBody);
}

#[test]
fn v05_owned_interface_edit_takes_whole_channel() {
    let mut sess = OwnedFunctionSession::warmup(OWNED_SRC, "d_main", &[]).expect("warmup");
    // Editing the TYPE SHELL (a field add) meets the type declaration —
    // the whole-module channel owns it.
    let grown = OWNED_SRC.replace(
        "type Tok {\n    label Str\n    n int",
        "type Tok {\n    label Str\n    n int\n    pad int",
    );
    let grown = grown.replace(
        "return Tok { label: s.move, n: k }",
        "return Tok { label: s.move, n: k, pad: 0 }",
    );
    // Rewrite the whole source via one full-range edit.
    let inv = sess
        .edit(0, OWNED_SRC.len() as u64, grown.as_bytes())
        .expect("type edit");
    assert_eq!(
        sess.last_channel(),
        auto_av_probe::owned_session::EditChannel::WholeModule
    );
    // make/score/main all rebuilt (the type context changed).
    assert!(!inv.is_empty(), "interface change invalidates: {inv:?}");
    // make still sets label/n (pad defaults to 0) — same observable 21.
    assert_eq!(sess.run_i32(), 21);
}
