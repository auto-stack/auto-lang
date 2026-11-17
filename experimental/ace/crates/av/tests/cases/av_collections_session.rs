//! PLAN-864 AC-04: the incremental source session — fresh / body-edit /
//! hit (unchanged) shapes over real .at sources, with the counter evidence:
//! unchanged functions ride the carry path (rebuilt stays 0 for that
//! revision), a body edit rebuilds exactly the edited function, and an
//! interface edit (a record field type) invalidates exactly its consumers.
//!
//! PLAN-1030 AVTX-01 adds the FAILED-revision face of the same entry: a
//! source that compiles and admits but traps at execution must not advance
//! the session, and re-submitting it must be rejected again rather than
//! read as a hit over the previous revision.

use ace_av::collections_864_session::CollectionsSourceSession;

const BASE: &str = "type Item { name Str; code int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).code + helper()\n}\n";

const BODY_EDIT: &str = "type Item { name Str; code int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).code + helper() + 0\n}\n";

const HELPER_EDIT: &str = "type Item { name Str; code int }\nfn helper() int { return 6 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).code + helper()\n}\n";

const IFACE_EDIT: &str = "type Item { name Str; other int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).other + helper()\n}\n";

/// PLAN-1030: BASE with only the element index changed. It passes the
/// source front end and the three admission gates and only then traps
/// (`av.trap.list-index`) during the revision's execution.
const REJECTING: &str = "type Item { name Str; code int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(1).code + helper()\n}\n";

#[test]
fn session_fresh_then_body_edit_rebuilds_one() {
    let mut s = CollectionsSourceSession::new(None, "d_main").expect("session");
    let cold = s.warmup(BASE).expect("cold");
    // fresh: every function rebuilds (helper + main = 2 units)
    assert!(
        cold.gate.pass(),
        "cold admission passes: {:?}",
        cold.gate.rebuilt_units
    );
    assert_eq!(cold.gate.rebuilt_units, 2, "cold rebuilds both functions");

    // body edit of main only: helper carries, main rebuilds
    let (warm, changed, carried) = s.edit(BODY_EDIT).expect("body edit");
    assert!(warm.gate.pass());
    assert_eq!(changed, 1, "a body edit rebuilds exactly main");
    assert_eq!(carried, 1, "the unchanged helper rides the carry path");
    assert_eq!(warm.value, Some(6), "code(1) + helper(5)");

    // unchanged resubmission: pure hit — nothing rebuilds
    let (hit, changed_hit, carried_hit) = s.edit(BODY_EDIT).expect("hit");
    assert!(hit.gate.pass());
    assert_eq!(changed_hit, 0, "an unchanged source rebuilds nothing");
    assert_eq!(carried_hit, 2, "both units carry");
}

#[test]
fn session_helper_edit_invalidates_consumer() {
    let mut s = CollectionsSourceSession::new(None, "d_main").expect("session");
    s.warmup(BASE).expect("cold");
    let (out, changed, carried) = s.edit(HELPER_EDIT).expect("helper edit");
    assert!(out.gate.pass());
    assert_eq!(changed, 1, "the edited helper rebuilds");
    assert_eq!(carried, 1, "main carries (its own bytes unchanged)");
    assert_eq!(out.value, Some(7), "code(1) + helper(6)");
}

#[test]
fn session_interface_edit_invalidates_consumers() {
    let mut s = CollectionsSourceSession::new(None, "d_main").expect("session");
    s.warmup(BASE).expect("cold");
    let (out, changed, _carried) = s.edit(IFACE_EDIT).expect("interface edit");
    assert!(out.gate.pass());
    assert!(
        changed >= 2,
        "a record-field rename changes the record shape and every function that constructs it (main) — and the type table rides the per-fn unit: {changed}"
    );
    assert_eq!(out.value, Some(6), "other(1) + helper(5)");
}

/// PLAN-1030 AVTX-01: A → failing B → retrying B → C(=A) → a real edit.
/// The injected input is a source that only fails DURING execution, so the
/// failure is a revision rejection and not a front-end diagnostic.
#[test]
fn audit1030_avtx_01() {
    let mut s = CollectionsSourceSession::new(None, "d_main").expect("session");
    let cold = s.warmup(BASE).expect("cold A");
    assert_eq!(cold.value, Some(6), "A = code(1) + helper(5)");
    assert_eq!(cold.revision, 1);

    // (1) B fails at execution: the error is the run-phase trap, NOT a
    // parse/verify/admission refusal (which is what a "wrong" test input
    // would have shown).
    let first = s.edit(REJECTING).expect_err("B traps at execution");
    assert!(
        first.starts_with("revision did not execute"),
        "B was admitted through every front gate and rejected by execution: {first}"
    );
    assert!(first.contains("ListIndex"), "the controlled trap: {first}");

    // (2) THE regression: re-submitting B must be rejected again. Pre-fix
    // B's fingerprint had already been accepted, so the retry was read as
    // a pure hit and returned A's value 6 at a fresh revision.
    let retry = s.edit(REJECTING).expect_err("the retry of B is still B");
    assert!(retry.contains("ListIndex"), "same trap: {retry}");

    // (3) A is still the accepted revision: C (identical to A) is a pure
    // carry with A's value — it does not smuggle B's rejected shape in.
    let (hit, changed, carried) = s.edit(BASE).expect("C = A is a hit");
    assert_eq!((changed, carried), (0, 2), "C's units are A's units");
    assert_eq!(hit.value, Some(6), "A's 6, from A's code objects");
    assert_eq!(
        hit.revision, 2,
        "only the accepted hit advanced the revision"
    );
    assert_eq!(hit.gate.rebuilt_units, 0, "a hit produces nothing");

    // (4) the session still accepts real edits after the failures.
    let (body, changed_body, carried_body) = s.edit(BODY_EDIT).expect("body edit");
    assert_eq!(
        (changed_body, carried_body),
        (1, 1),
        "main rebuilds, helper carries"
    );
    assert_eq!(body.value, Some(6));
    assert_eq!(body.revision, 3);
    assert_eq!(
        body.production.verified, 1,
        "only the submitted unit was verified"
    );
}
