//! PLAN-820 T-09: V-13 — the data profile's function-incremental
//! compatibility over the 828 public interfaces (`av_data_820` family,
//! part 5). Four subcases (contract-decision §12):
//! 1. shared D828 scalar baseline — covered by the function_incremental_828
//!    suite green in this tree (its own family; not re-run here);
//! 2. a REAL data-program body edit re-lowers exactly the edited
//!    function; every other function's code identity (Arc pointer +
//!    generation) is unchanged and the semantics stay correct;
//! 3. a type/interface change invalidates precisely the affected
//!    functions; a signature-breaking edit is a controlled LINK rejection
//!    that leaves the previous snapshot executable;
//! 4. the forced whole-rebuild counter-control: rebuilding unchanged
//!    functions MOVES their identity — the structural axis a degradation
//!    gate reddens — proving subcase 2's stability is the increment, not
//!    an artifact.

use auto_av_core::contract::Value;
use auto_av_probe::data_session::DataIncrementalSession;

const SRC: &str = "\
type I { x int }
type O { inner I }
fn bump(o O) O {
    var c = o
    c.inner.x = c.inner.x + 1
    return c
}
fn main() int {
    var o = O { inner: I { x: 41 } }
    let b = bump(o)
    return b.inner.x
}
";

#[test]
fn av_data_820_v13_body_edit_is_function_level() {
    // Subcase 2: edit bump's body (+1 → +2 through the caller's constant)
    // — bump's code identity changes, main's is IDENTICAL, and the run
    // result reflects the edited body.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    assert!(matches!(s.run(), Ok(Value::I32(42))), "cold run 41+1=42");
    let before_main = s.identity_of("d_main").expect("main resident");
    let counters_before: Vec<(String, u64)> =
        s.counters().iter().map(|(k, v)| (k.clone(), *v)).collect();

    let edited = SRC.replace("+ 1", "+ 2");
    let report = s.edit_body(&edited, "d_bump").expect("body edit");
    assert_eq!(report.changed_def, "d_bump");
    // main's identity: unchanged to the byte.
    assert_eq!(
        s.identity_of("d_main"),
        Some(before_main),
        "unchanged function's code object/generation is reused as-is"
    );
    // Structural axis: only d_bump's lower counter moved.
    for (k, v) in counters_before {
        let want = if k == "d_bump" { v + 1 } else { v };
        assert_eq!(s.counters().get(&k), Some(&want), "counter axis for {k}");
    }
    // Semantics: 41 + 2 = 43.
    assert!(
        matches!(s.run(), Ok(Value::I32(43))),
        "edited body executes"
    );
}

#[test]
fn av_data_820_v13_type_change_invalidates_precisely() {
    // Subcase 3a: adding a field to O changes the TYPE table — every unit
    // whose types mention O must re-lower; identity of BOTH moves. The
    // precise-invalidation contract for a whole-type change is "all
    // dependents rebuild"; the KEY control is that a NON-dependent edit
    // still leaves others untouched (subcase 2 above).
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    let typed = SRC
        .replace(
            "type O { inner I }",
            "type O { inner I
    extra int }",
        )
        .replace(
            "var o = O { inner: I { x: 41 } }",
            "var o = O { inner: I { x: 41 }, extra: 0 }",
        )
        .replace(
            "    var c = o\n    c.inner.x = c.inner.x + 1\n    return c",
            "    var c = o\n    c.inner.x = c.inner.x + 1\n    c.extra = 7\n    return c",
        );
    let rebuilt = s
        .apply_source(&typed, &["d_bump", "d_main"])
        .expect("type-change edit links");
    assert_eq!(rebuilt.len(), 2, "both O-dependent functions re-lowered");
    assert!(matches!(s.run(), Ok(Value::I32(42))), "still 41+1=42");

    // Subcase 3b (rejection control): a CROSS-REVISION signature change.
    // The rev2 source is fully self-consistent (bump takes I and returns
    // it; main adapts to `b.x`) so the whole-source front door accepts it —
    // but only d_bump is rebuilt: the RESIDENT d_main (rev1, calling
    // bump(O)) now mismatches at the LINK. This is exactly the incremental
    // hazard the snapshot link gate owns; the previous snapshot survives.
    let broken = SRC
        .replace(
            "fn bump(o O) O {\n    var c = o\n    c.inner.x = c.inner.x + 1\n    return c\n}",
            "fn bump(o I) I {\n    var c = o\n    c.x = c.x + 1\n    return c\n}",
        )
        .replace("return b.inner.x", "return b.x")
        .replace("let b = bump(o)", "let b = bump(o.inner)");
    let err = s
        .apply_source(&broken, &["d_bump"])
        .expect_err("signature break must reject");
    assert!(err.contains("link rejected"), "got {err}");
    assert!(
        matches!(s.run(), Ok(Value::I32(42))),
        "the resident snapshot survives the rejected edit"
    );
}

#[test]
fn av_data_820_v13_forced_whole_rebuild_moves_identity() {
    // Subcase 4: the counter-control — force-rebuilding UNCHANGED
    // functions moves their identity axis. A degradation gate keying this
    // axis (as the 828 structural gate does) reddens; the pair with
    // subcase 2 proves the increment is real.
    let mut s = DataIncrementalSession::warmup(SRC, "d_main", &[]).expect("warmup");
    let moved = s
        .forced_whole_rebuild_counter()
        .expect("forced rebuild runs");
    assert!(
        moved >= 2,
        "a forced whole rebuild moves every function's identity (moved {moved})"
    );
    assert!(matches!(s.run(), Ok(Value::I32(42))), "semantics unchanged");
}
