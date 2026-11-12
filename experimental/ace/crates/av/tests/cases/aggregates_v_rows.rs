//! PLAN-962 T-04: the V-row permanent fixtures (V-01..V-05, V-08) —
//! each `.at` under `fixtures/aggregates_962/` runs through the REAL
//! full chain (source → CST → resolve → typecheck → HIR → verify →
//! lower → MIR verify → ABC → AV execute) with independent value and
//! resource assertions. Negative controls compile-reject with the named
//! diagnostic.

use ace_av::collections_864_support::{compile_and_execute, compile_and_execute_aggregates};

macro_rules! fx {
    ($n:expr) => {
        include_str!(concat!("../../../core/tests/fixtures/aggregates_962/", $n))
    };
}

/// V-01: nested List<record<Str>> — the moved row stays valid after the
/// transfer out (value oracle 42; the frame releases both column lists).
#[test]
fn v01_nested_list_record_moves_out() {
    let src = fx!("v01_nested_clone.at");
    let run = compile_and_execute_aggregates(&src, "d_main").expect("V-01 executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
}

/// V-02: self-alias replacement — the RHS materializes before the store
/// (`xs[0] = n` with the pre-store `n`; the frozen collections family's
/// view of the same invariant).
#[test]
fn v02_self_alias_rhs_first() {
    let src = fx!("v02_self_alias.at");
    let run = compile_and_execute_aggregates(&src, "d_main").expect("V-02 executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
}

/// V-03: mid-construction failure / recovery — two independent builds
/// (fresh frames) land the same semantics; every frame's arena ledger
/// closes (the LIFO reclaim observable).
#[test]
fn v03_midfail_recovery_two_frames() {
    let src = fx!("v03_midfail_recovery.at");
    let run = compile_and_execute_aggregates(&src, "d_main").expect("V-03 executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
}

/// V-04 negative: a DIRECT recursive record rejects BEFORE layout
/// (`verify.record-cycle` — the T-01 probe gap, closed for the whole
/// collections family).
#[test]
fn v04_direct_recursion_rejects_before_layout() {
    let src = fx!("v04_recursive_reject.at");
    let err = ace_core::source::compile_aggregates_source(&src, "v04.at", &[])
        .err()
        .expect("direct recursion must reject");
    assert!(err.iter().any(|d| d.code == "verify.record-cycle"));
}

/// V-04 positive: recursion through the EXPLICIT indirection (Box = the
/// single-element List cell) executes; every cell frees (last-owner
/// destruct — the V-08 Box slice).
#[test]
fn v04_box_recursion_executes_and_frees() {
    let src = fx!("v04_box_legal.at");
    let run = compile_and_execute_aggregates(&src, "d_main").expect("V-04 box executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
}

/// V-05 (resource ledger slice, AV side): the grid's column lists and
/// the destructured reads close the frame ledger exactly (allocs ==
/// frees, live 0) — the AV end of the four-backend resource-event
/// comparison (native COFF emission + a2x true chains cover theirs).
#[test]
fn v05_grid_column_ledger_closes() {
    let src = fx!("v05_grid_columns.at");
    let run = compile_and_execute_aggregates(&src, "d_main").expect("V-05 executes");
    assert_eq!(run.value, 62);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
    // the collections face keeps rejecting the same nested shape (frozen
    // 822 one-layer domain — the aggregates capability is the divider)
    let nested = "fn main() int {\n    var xs = List<List<int>>.new()\n    return 1\n}";
    let err = compile_and_execute(nested, "d_main")
        .err()
        .expect("frozen face rejects");
    assert!(err.contains("source.list-value-context") || err.contains("822"));
}

/// V-08: union/tag branch dispatch — the tagged union's inactive arm
/// simply fails its is-guard (the t01-freeze D10 semantics; a dedicated
/// branch family the List tests cannot stand for).
#[test]
fn v08_union_tag_branch_family() {
    let src = fx!("v08_union_tag_branch.at");
    let run = compile_and_execute_aggregates(&src, "d_main").expect("V-08 union/tag executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
}

/// V-08: the whole-local hold — the alias reads through the block and
/// the binding dies with it (>= the old engine's MVP semantics).
#[test]
fn v08_hold_alias_block() {
    let src = fx!("v08_hold_alias.at");
    let run = compile_and_execute_aggregates(&src, "d_main").expect("V-08 hold executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
}
