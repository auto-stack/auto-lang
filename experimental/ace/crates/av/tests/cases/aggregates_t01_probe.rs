//! PLAN-962 T-01 bounded probe: the longest producer→consumer path
//! (source → CST front door → resolve → typecheck → HIR → verify →
//! lowering → MIR verify → AV execute) with the aggregate surface the
//! plan must lift, plus its same-type illegal neighbour and a
//! failure-recovery leg.
//!
//! Baseline semantics at T-01 (frozen in t01-freeze.md):
//! - `nested_list_rejected_today`: `List<List<int>>` is rejected by the
//!   822 element domain (`source.list-value-context`) — the red that
//!   T-02 turns green under the aggregates profile.
//! - `recursive_record_rejected_before_layout`: a self-referential
//!   record is rejected at bind (`source.record-cycle` family), i.e.
//!   before any layout/backend work — the pre-layout rejection phase
//!   V-04 relies on.
//! - `chain_health_and_recovery`: the full chain executes a supported
//!   aggregate program (record-in-List) with resource accounting, and a
//!   broken neighbour rejects with a diagnostic while the fixed program
//!   still runs — the failure-recovery leg.

use ace_av::collections_864_support::compile_and_execute;

#[test]
fn chain_health_and_recovery() {
    // Positive: supported aggregate composition today (record element).
    let good = "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    xs.push(Item(\"x\", 20))\n    xs.push(Item(\"y\", 22))\n    return xs.get(0).code + xs.get(1).code\n}";
    let run = compile_and_execute(good, "d_main").expect("supported aggregate chain");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);

    // Illegal neighbour of the same type family: record ctor arity error.
    let broken = good.replace("Item(\"y\", 22)", "Item(\"y\")");
    let err = ace_core::source::compile_collections_source(broken.as_str(), "probe.at", &[])
        .err()
        .expect("wrong arity must reject");
    assert!(!err.is_empty());

    // Recovery: the same program compiles and runs after the fix.
    let run2 = compile_and_execute(good, "d_main").expect("recovery leg");
    assert_eq!(run2.value, 42);
}

#[test]
fn nested_list_rejected_today() {
    let src = "fn main() int {\n    var xs = List<List<int>>.new()\n    var inner = List<int>.new()\n    inner.push(40)\n    inner.push(2)\n    xs.push(inner.move)\n    return xs.get(0).get(1)\n}";
    let err = ace_core::source::compile_collections_source(src, "probe.at", &[])
        .err()
        .expect("nested List is outside the 822 element domain today");
    assert!(
        err.iter().any(|d| d.code == "source.list-value-context"),
        "expected the 822 element-domain diagnostic, got: {:?}",
        err.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

/// PLAN-962 unit-1: the same source under the aggregates profile must
/// compile AND execute through the full chain (the frozen collections
/// face keeps rejecting — see `nested_list_rejected_today`). The
/// remove-based read moves the inner list OUT of the outer list (value
/// semantics the existing facts schedule already owns); whole-element
/// view reads (`xs.get(0)` aliasing) are the unit-2 view surface.
#[test]
fn nested_list_runs_under_aggregates_profile() {
    let src = "fn main() int {\n    var xs = List<List<int>>.new()\n    var inner = List<int>.new()\n    inner.push(40)\n    inner.push(2)\n    xs.push(inner.move)\n    var row = xs.remove(0)\n    return row.get(0) + row.get(1)\n}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("nested List compiles+runs under the aggregates profile");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
}

/// T-01 probe found a real pre-existing gap (t01-freeze.md §0): the
/// record containment-cycle walk only ran for the Record/TextRecord
/// domains. PLAN-962 unit-1 extends the cycle gate to the collection
/// family (V-04's pre-layout rejection) on BOTH the frozen collections
/// face and the aggregates face.
#[test]
fn recursive_record_rejected_before_layout() {
    type CompileFn =
        fn(
            &str,
            &str,
            &[String],
        )
            -> Result<ace_core::source::CompiledSource, Vec<ace_core::atom_text::Diagnostic>>;
    let compiles: [CompileFn; 2] = [
        ace_core::source::compile_collections_source,
        ace_core::source::compile_aggregates_source,
    ];
    for compile in compiles {
        let src = "type Node { v int; next Node }\nfn main() int { return 1 }";
        let err = compile(src, "probe.at", &[])
            .err()
            .expect("self-referential record must reject before layout");
        assert!(
            err.iter().any(|d| d.code == "verify.record-cycle"),
            "expected the record-cycle diagnostic, got: {:?}",
            err.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
        );
    }
}
