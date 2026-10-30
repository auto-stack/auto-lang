//! PLAN-864 AC-04: the incremental source session — fresh / body-edit /
//! hit (unchanged) shapes over real .at sources, with the counter evidence:
//! unchanged functions ride the carry path (rebuilt stays 0 for that
//! revision), a body edit rebuilds exactly the edited function, and an
//! interface edit (a record field type) invalidates exactly its consumers.

use ace_av::collections_864_session::CollectionsSourceSession;

const BASE: &str = "type Item { name Str; code int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).code + helper()\n}\n";

const BODY_EDIT: &str = "type Item { name Str; code int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).code + helper() + 0\n}\n";

const HELPER_EDIT: &str = "type Item { name Str; code int }\nfn helper() int { return 6 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).code + helper()\n}\n";

const IFACE_EDIT: &str = "type Item { name Str; other int }\nfn helper() int { return 5 }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"ab\", 1))\n    return xs.get(0).other + helper()\n}\n";

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
