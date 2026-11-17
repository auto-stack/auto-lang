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

// ---------------------------------------------------------------------------
// PLAN-1035 (T-02): RES-03 — the REAL mid-failure evidence over the new
// aggregate shapes (F30). The 915 seam helpers only carried the
// Collections/Closures frontends; this aggregate leg runs the same
// two-seam per-attempt sweep through `compile_aggregates_source`, plus the
// commit-seam (no-partial-commit) observation on the real destination.
// ---------------------------------------------------------------------------

/// The aggregate replacement program: build a List-only record into a List,
/// then REPLACE it with a fresh record (the atomic store whose commit seam
/// RES-03 observes). The entry value comes from a scalar bound before the
/// replacement, so every attempt's fault still yields the frozen 42 when the
/// seam is one past the last attempt.
const RES03_REPLACE: &str = "type Bag { items List<int> }\nfn main() int {\n    var r = 42\n    var xs = List<Bag>.new()\n    var inner = List<int>.new()\n    inner.push(r)\n    xs.push(Bag(inner.move))\n    var fresh = List<int>.new()\n    fresh.push(7)\n    xs[0] = Bag(fresh.move)\n    return r\n}";

/// The SLOT-replacement program: the destination is a whole `var Bag` slot
/// whose assignment goes through the executor's commit write (the seam that
/// carries the `replaced` lifecycle fact). The new value is deliberately
/// larger, so the commit is a real growth attempt on the value seam.
const RES03_SLOT_REPLACE: &str = "type Bag { items List<int> }\nfn main() int {\n    var r = 42\n    var small = List<int>.new()\n    small.push(1)\n    var b = Bag(small.move)\n    var big = List<int>.new()\n    big.push(1)\n    big.push(2)\n    big.push(3)\n    big.push(4)\n    b = Bag(big.move)\n    return r\n}";

/// RES-03: both seams sweep every REAL attempt of the aggregate program.
/// Each injected fault is the fault-injected Infra (never a language
/// verdict), the ledger closes (nothing half-constructed survives, each
/// buffer releases exactly once), the N+1 control and the post-fault rerun
/// still produce 42.
#[test]
fn audit1035_res_03_aggregate_fault_sweep() {
    let facts = ace_av::resource_915::aggregates_fault_sweep(RES03_REPLACE, "d_main")
        .expect("aggregate fault sweep runs");

    assert_eq!(facts["source_value"], 42, "{facts:#}");
    assert_eq!(facts["clean_ledger_closes"], true, "{facts:#}");
    assert!(facts["arena_attempts"].as_u64().unwrap() >= 2, "{facts:#}");
    assert!(facts["value_attempts"].as_u64().unwrap() >= 1, "{facts:#}");

    let arena_points = facts["arena_points"].as_array().expect("arena points");
    assert!(!arena_points.is_empty(), "{facts:#}");
    for p in arena_points {
        assert_eq!(p["faulted"], true, "arena point must hit the seam: {p}");
        assert_eq!(p["live"], 0, "arena point leaked buffers: {p}");
        assert_eq!(p["free_ids_unique"], true, "arena point double-freed: {p}");
        let successful = p["successful"].as_u64().unwrap();
        let frees = p["frees"].as_u64().unwrap();
        assert_eq!(frees, successful, "arena point must free every buffer: {p}");
    }

    let value_points = facts["value_points"].as_array().expect("value points");
    assert!(!value_points.is_empty(), "{facts:#}");
    for p in value_points {
        assert_eq!(p["faulted"], true, "value point must hit the seam: {p}");
        assert_eq!(p["live"], 0, "value point leaked buffers: {p}");
    }

    // N+1 on both seams: the frozen value survives, the ledger closes.
    assert_eq!(facts["arena_control"]["value"], 42, "{facts:#}");
    assert_eq!(facts["arena_control"]["live"], 0, "{facts:#}");
    assert_eq!(facts["value_control"]["value"], 42, "{facts:#}");
    assert_eq!(facts["value_control"]["live"], 0, "{facts:#}");

    // Clearing the fault re-runs the same session, and a full recompile
    // reproduces the value: the failure cached no PASS and no half product.
    assert_eq!(facts["after"]["value"], 42, "{facts:#}");
    assert_eq!(facts["after"]["live"], 0, "{facts:#}");
    assert_eq!(facts["after_recompile"]["value"], 42, "{facts:#}");
    assert_eq!(facts["after_recompile"]["live"], 0, "{facts:#}");
}

/// RES-03: the COMMIT-SEAM observation on the real destination. The clean
/// control destroys the replaced value (>=1 `replaced` disown of the OLD
/// buffer), while every faulted attempt reports ZERO replaced disowns — the
/// write never committed, so the old destination value cannot have been
/// dropped first. The abort sweep releases the remaining owners exactly once
/// (no leak, no double free), and the old buffer keeps its identity until
/// that abort (it is freed there, exactly once).
#[test]
fn audit1035_res_03_commit_seam_keeps_old_destination() {
    let clean =
        ace_av::resource_915::aggregates_commit_fault_probe(RES03_SLOT_REPLACE, "d_main", None)
            .expect("clean observed run");
    assert_eq!(clean["outcome"], "success", "{clean:#}");
    assert_eq!(clean["value"], 42, "{clean:#}");
    assert_eq!(clean["arena_live"], 0, "{clean:#}");
    assert_eq!(clean["frees"], clean["arena_allocs"], "{clean:#}");
    assert_eq!(clean["free_ids_unique"], true, "{clean:#}");
    let replaced_ids: Vec<u64> = clean["replaced_ids"]
        .as_array()
        .expect("replaced ids")
        .iter()
        .filter_map(|v| v.as_u64())
        .collect();
    assert!(
        !replaced_ids.is_empty(),
        "the replacement must really destroy the old destination buffer: {clean:#}"
    );

    let sweep = ace_av::resource_915::aggregates_fault_sweep(RES03_SLOT_REPLACE, "d_main")
        .expect("sweep for attempt discovery");
    let n = sweep["value_attempts"].as_u64().expect("attempt count");
    assert!(
        n >= 1,
        "the replacement must expose a value attempt: {sweep:#}"
    );
    for attempt in 1..=n {
        let p = ace_av::resource_915::aggregates_commit_fault_probe(
            RES03_SLOT_REPLACE,
            "d_main",
            Some(attempt),
        )
        .expect("observed faulted run");
        assert_eq!(p["outcome"], "infra", "attempt {attempt}: {p:#}");
        let msg = p["message"].as_str().unwrap_or_default();
        assert!(
            msg.contains("fault-injected"),
            "attempt {attempt} must be the injected seam, got {msg:?}"
        );
        assert_eq!(
            p["replaced"], 0,
            "attempt {attempt} must not commit over the old destination: {p:#}"
        );
        assert_eq!(p["arena_live"], 0, "attempt {attempt} leaked: {p:#}");
        // The OLD destination buffer must survive to the abort and be
        // released there exactly once (its identity is the one the clean
        // run's `replaced` disown names).
        let frees: Vec<u64> = p["free_ids"]
            .as_array()
            .expect("free ids")
            .iter()
            .filter_map(|v| v.as_u64())
            .collect();
        for old in &replaced_ids {
            assert!(
                frees.contains(old),
                "attempt {attempt} lost the old destination buffer {old} (frees={frees:?}): {p:#}"
            );
        }
        let mut sorted = frees.clone();
        sorted.sort_unstable();
        let frees_len = sorted.len();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            frees_len,
            "attempt {attempt} double-freed a buffer: {p:#}"
        );
        assert_eq!(
            p["frees"], p["arena_allocs"],
            "attempt {attempt} must free exactly the allocated buffers: {p:#}"
        );
        assert_eq!(p["free_ids_unique"], true, "attempt {attempt}: {p:#}");
        assert_eq!(p["lifecycle_overflowed"], false, "attempt {attempt}: {p:#}");
    }

    // N+1 commits: the replacement really lands again.
    let control = ace_av::resource_915::aggregates_commit_fault_probe(
        RES03_SLOT_REPLACE,
        "d_main",
        Some(n + 1),
    )
    .expect("N+1 observed run");
    assert_eq!(control["outcome"], "success", "{control:#}");
    assert_eq!(control["value"], 42, "{control:#}");
    assert!(
        control["replaced"].as_u64().unwrap() >= 1,
        "the N+1 control commits the replacement: {control:#}"
    );
}
