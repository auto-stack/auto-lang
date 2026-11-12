//! PLAN-962 T-03 native leg: the aggregates surface through the REAL mn
//! pipeline (aggregates source profile → closures-family lowering → CLIF
//! → COFF → PE execution). The plan added ZERO new MIR instructions —
//! tuples ride records, Box/grid ride lists, index writes ride LIST_SET,
//! view reads ride the PLACE machinery — so the native lane needs only
//! the driver admission (the escalation ladder's new final rung).

#[path = "../common/mod.rs"]
mod common;

fn expect_exit(src: &str, want: i32, tag: &str) {
    if !common::tools_available() {
        panic!("infra-blocked: rust-lld/SDK unavailable");
    }
    let r = common::run_mn(src, &[], tag);
    assert!(r.ok, "{:?}: {:?}", tag, r.error);
    assert_eq!(r.exit_code, Some(want), "case {tag}");
}

#[test]
#[cfg(windows)]
fn mn_aggregates_arena_free_composites_execute() {
    // tuple + destructuring + named ctor + re-construct
    expect_exit(
        "fn main() int {
    let t = (19, 23)
    let (x, y) = t
    let p = Pair { right: 0, left: y }
    return x + p.left
}
type Pair { left int; right int }",
        42,
        "ag-tuple-named-ctor",
    );
}

/// PLAN-962 T-03 native collections lane: arena-carrying aggregates
/// programs (lists/tuples-of-lists/Box/grid) lower to the SAME frozen
/// collections MIR — every split unit ADMITS and EMITS a real COFF
/// object through collections_lower (the mn-collections worker lane's
/// emission half; worker-protocol execution is the batch tier's).
#[test]
#[cfg(windows)]
fn mn_aggregates_collections_lane_admits_and_emits() {
    if !common::tools_available() {
        panic!("infra-blocked: rust-lld/SDK unavailable");
    }
    let src = "fn main() int {
    var xs = [40, 2]
    xs[0] = 41
    var ys = List<List<int>>.new()
    var inner = [1, 2]
    ys.push(inner.move)
    return xs.get(0) + xs.get(1) + ys.get(0).get(1) - 2
}";
    let compiled = ace_core::source::compile_aggregates_source(src, "ag.at", &[]).unwrap();
    let m = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0).unwrap();
    ace_av_core::verify_mir(&m).unwrap();
    let units = ace_av::cache::split_units(&m).unwrap();
    assert_eq!(units.len(), 1);
    for u in &units {
        ace_native::collections_lower::admit_unit(u).unwrap();
        let _obj = ace_native::collections_lower::emit_unit_coff(u).unwrap();
    }
    // the range/tag/union shapes (arena-carrying variants) ride the lane too
    let src3 = "tag Mark { Hit(int); Miss(int) }
union Val { Num(int); Flag(int) }
fn main() int {
    var s = 0
    for i in 1 .. 5 {
        s = s + i
    }
    let m = Mark.Hit(20)
    let v = Val.Num(1)
    is m {
        Mark.Hit(a) -> {
            is v {
                Val.Num(b) -> { return s + a + b }
                Val.Flag(b) -> { return b }
            }
        }
        Mark.Miss(a) -> { return a }
    }
}";
    let compiled3 = ace_core::source::compile_aggregates_source(src3, "ag3.at", &[]).unwrap();
    let m3 = ace_lowering::lower_checked_closures(&compiled3.checked, "d_main", 0).unwrap();
    ace_av_core::verify_mir(&m3).unwrap();
    let units3 = ace_av::cache::split_units(&m3).unwrap();
    for u in &units3 {
        ace_native::collections_lower::admit_unit(u).unwrap();
        let _obj = ace_native::collections_lower::emit_unit_coff(u).unwrap();
    }
    // the Box cell + grid shapes ride the same lane. NAMED FINDING: a
    // RECURSIVE Box (Node { next Box<Node> }) lowers to a CYCLIC type
    // table which the unit-split fixpoint cannot localize (unresolved
    // type closure) — recorded in the plan §9; the non-recursive cell
    // and the grid column-tuple emit today.
    let src2 = "type P { v int }
fn main() int {
    let p = P(41)
    let cell = Box(p)
    let q = P(1)
    return cell.get(0).v + q.v
}";
    let src2b = "fn main() int {
    let g = grid (a, b) { [40, 20]; [0, 2] }
    let (ca, cb) = g
    return ca.get(0) + ca.get(1) + cb.get(0) + cb.get(1)
}";
    let compiled2 = ace_core::source::compile_aggregates_source(src2, "ag2.at", &[]).unwrap();
    let m2 = ace_lowering::lower_checked_closures(&compiled2.checked, "d_main", 0).unwrap();
    ace_av_core::verify_mir(&m2).unwrap();
    let units2 = ace_av::cache::split_units(&m2).unwrap();
    for u in &units2 {
        ace_native::collections_lower::admit_unit(u).unwrap();
        let _obj = ace_native::collections_lower::emit_unit_coff(u).unwrap();
    }
    let compiled2b = ace_core::source::compile_aggregates_source(src2b, "ag2b.at", &[]).unwrap();
    let m2b = ace_lowering::lower_checked_closures(&compiled2b.checked, "d_main", 0).unwrap();
    ace_av_core::verify_mir(&m2b).unwrap();
    let units2b = ace_av::cache::split_units(&m2b).unwrap();
    for u in &units2b {
        ace_native::collections_lower::admit_unit(u).unwrap();
        let _obj = ace_native::collections_lower::emit_unit_coff(u).unwrap();
    }
}
