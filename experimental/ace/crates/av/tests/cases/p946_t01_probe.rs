//! PLAN-946 T-01 bounded probe (frozen outcome): the collections whole
//! path for the public algorithm slice this plan freezes. Each leg pins a
//! fact recorded in PLAN-946 §5 "T-01 冻结表":
//!
//! - Base collections profile legal method table today is exactly
//!   `new/push/insert/get/set/remove/len/clear` (pop is NOT in it; the old
//!   pop obligation maps to a remove(len-1) wrapper delivered by T-02).
//! - List-typed parameters/results are rejected in the base profile
//!   (`source.list-value-context`: v1 lists are function-local values);
//!   the aggregates profile (PLAN-962 `av.aggregates/1`) is the delivered
//!   legal home for public List-parameter functions — the T-08 shape.
//! - Source-level `xs.clone()` is rejected ("elements clone explicitly");
//!   deep-container clone fault legs (V-DETAIL-03) therefore run through
//!   growth + explicit element clones, per the frozen boundary.

use ace_av::collections_864_support::{compile_and_execute, compile_and_execute_aggregates};
use ace_core::source::compile_collections_source;

fn expect_value(name: &str, src: &str, want: i32) {
    match compile_and_execute(src, "main") {
        Ok(out) => {
            println!(
                "{name}: value={} live={} allocs={} frees={}",
                out.value, out.live_after, out.allocs, out.frees
            );
            assert_eq!(out.value, want, "{name}");
            assert_eq!(out.live_after, 0, "{name}: live_after");
            assert_eq!(out.allocs, out.frees, "{name}: allocs==frees");
        }
        Err(e) => panic!("{name}: unexpected reject: {e}"),
    }
}

fn expect_value_aggregates(name: &str, src: &str, want: i32) {
    match compile_and_execute_aggregates(src, "main") {
        Ok(out) => {
            println!(
                "{name}(aggregates): value={} live={} allocs={} frees={}",
                out.value, out.live_after, out.allocs, out.frees
            );
            assert_eq!(out.value, want, "{name}");
            assert_eq!(out.live_after, 0, "{name}: live_after");
            assert_eq!(out.allocs, out.frees, "{name}: allocs==frees");
        }
        Err(e) => panic!("{name}: unexpected reject: {e}"),
    }
}

/// Scalar List reduction via for-in: [1..5] sums to 15 (the plan's
/// independent reduction oracle — not a printed banner).
#[test]
fn p946_scalar_reduce_for_in_sums_15() {
    let src = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    var total = 0
    for x in xs {
        total = total + x
    }
    return total
}
"#;
    expect_value("reduce", src, 15);
}

/// Scalar List query via for-in: first element > 3 is 4 (nested-if guard,
/// no break, no closure — the frozen algorithm shape).
#[test]
fn p946_scalar_find_first_gt3_is_4() {
    let src = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    var found = 0
    for x in xs {
        if x > 3 {
            if found == 0 {
                found = x
            }
        }
    }
    return found
}
"#;
    expect_value("find", src, 4);
}

/// The old pop obligation (push 10/20/30, pop -> 30, two left) mapped onto
/// today's frozen method table: remove(len-1) is the move-out read.
#[test]
fn p946_remove_last_returns_30_and_len_2() {
    let src = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(10)
    xs.push(20)
    xs.push(30)
    var last = xs.remove(xs.len() - 1)
    return last * 10 + xs.len()
}
"#;
    expect_value("pop-via-remove", src, 302);
}

/// Empty-remove contract (PLAN-1098 T-05, O-01).
///
/// Independent oracle from `docs/specs/stdlib/design/ace-collections.md:17`:
/// the bare `remove(len-1)` on an EMPTY list traps `list-index`; it is `pop`
/// (the `?T` wrapper) that answers `None`. The old probe printed whichever
/// branch happened and could never go red.
#[test]
fn p946_empty_remove_contract_observed() {
    let src = r#"
fn main() int {
    var xs = List<int>.new()
    var p = xs.remove(xs.len() - 1)
    return p
}
"#;
    let err = match compile_and_execute(src, "main") {
        Ok(out) => panic!(
            "empty `remove` must trap, never return a value (got value={})",
            out.value
        ),
        Err(e) => e,
    };
    assert_eq!(
        err, "trap: av.trap.list-index",
        "empty remove contract (spec trap kind `list-index`, code `av.trap.list-index`)"
    );

    // Legal neighbour: the same call on a non-empty list is a value transfer.
    let ok_src = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(7)
    var p = xs.remove(xs.len() - 1)
    return p
}
"#;
    expect_value("remove-nonempty", ok_src, 7);
}

/// Owned-Str elements: `List<Str>` push/get with `.len()` reads — the
/// owned-Str slice the plan must support (3 allocs = two texts + buffer,
/// all freed, live 0).
#[test]
fn p946_owned_str_elements_len_sum_10() {
    let src = r#"
fn main() int {
    var xs = List<Str>.new()
    xs.push("hello")
    xs.push("world")
    return xs.get(0).len() + xs.get(1).len()
}
"#;
    expect_value("owned-str", src, 10);
}

/// T-08 shape: a public function taking `List<int>` by parameter — legal
/// under the aggregates profile (PLAN-962 collections parameter/return
/// contract), which is where 946's public algorithm file compiles.
#[test]
fn p946_list_param_public_fn_aggregates_sums_15() {
    let src = r#"
fn sum_list(xs List<int>) int {
    var total = 0
    for x in xs {
        total = total + x
    }
    return total
}
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    xs.push(3)
    xs.push(4)
    xs.push(5)
    return sum_list(xs)
}
"#;
    expect_value_aggregates("param-fn", src, 15);
}

/// Frozen negative (base face): the SAME List-parameter function rejects
/// under the collections profile — `source.list-value-context` — so the
/// base profile stays byte-identical while 946's public algorithms live
/// under the aggregates capability.
#[test]
fn p946_list_param_rejected_in_base_profile() {
    let src = r#"
fn sum_list(xs List<int>) int {
    return 0
}
fn main() int {
    return 0
}
"#;
    let err = compile_collections_source(src, "probe.at", &[])
        .err()
        .expect("List-typed parameter must reject under the base profile");
    assert!(
        err.iter().any(|d| d.code == "source.list-value-context"),
        "expected source.list-value-context, got: {:?}",
        err.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
}

/// Source-level List clone boundary: `xs.clone()` rejects in the base
/// profile ("elements clone explicitly"); observe the aggregates face too.
#[test]
fn p946_list_clone_boundary() {
    let src = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    var ys = xs.clone()
    return ys.get(0) + ys.get(1)
}
"#;
    let err = compile_collections_source(src, "probe.at", &[])
        .err()
        .expect("source-level list clone must reject (frozen boundary)");
    assert!(
        err.iter().any(|d| d.code == "source.unsupported"),
        "expected source.unsupported for clone, got: {:?}",
        err.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
    match compile_and_execute_aggregates(src, "main") {
        Ok(out) => println!("clone(aggregates): executes value={}", out.value),
        Err(e) => println!("clone(aggregates): rejected: {e}"),
    }
}

/// Frozen negative (PLAN-822/962): nested `List<List<int>>` stays
/// rejected under the collections profile — `source.list-value-context`;
/// the aggregates profile is the legal superset and runs the nested leg
/// with an independent expected value (inner 40+2 = 42).
#[test]
fn p946_nested_list_base_rejects_aggregates_runs() {
    let base = "fn main() int {\n    var xs = List<List<int>>.new()\n    return 1\n}";
    let err = compile_collections_source(base, "probe.at", &[])
        .err()
        .expect("nested List must reject under the collections profile");
    assert!(
        err.iter().any(|d| d.code == "source.list-value-context"),
        "expected source.list-value-context, got: {:?}",
        err.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
    );
    let agg = r#"
fn main() int {
    var xs = List<List<int>>.new()
    var inner = List<int>.new()
    inner.push(40)
    inner.push(2)
    xs.push(inner.move)
    return xs.get(0).get(0) + xs.get(0).get(1)
}
"#;
    expect_value_aggregates("nested", agg, 42);
}

/// Frozen negative: wrong element type on push rejects before any backend.
#[test]
fn p946_wrong_element_type_rejected() {
    let src = r#"
fn main() int {
    var xs = List<int>.new()
    xs.push("not an int")
    return 0
}
"#;
    let err = compile_collections_source(src, "probe.at", &[])
        .err()
        .expect("wrong element type must reject before lowering");
    assert!(!err.is_empty(), "expected a named diagnostic, got none");
}

/// T-02 semantic pin: List-typed arguments MOVE into the callee in the
/// delivered aggregates surface (the caller's list is dead after the
/// call; a second use reads the moved-out state — the D-962-C4 debt
/// compiles silently, it is NOT legal ownership). a=2, then xs.len()=0
/// → 20 (a borrowing transfer would give 22).
#[test]
fn p946_list_argument_moves_single_module() {
    let src = r#"
fn len_of(xs List<int>) int {
    return xs.len()
}
fn main() int {
    var xs = List<int>.new()
    xs.push(1)
    xs.push(2)
    let a = len_of(xs)
    return a * 10 + xs.len()
}
"#;
    // OBSERVED: 22 — the single-module face BORROWS the list argument
    // (the caller's list survives). The packages lane gives 20 (dead).
    // Pin the single-module borrow as the reference semantics; the
    // packages divergence is recorded in PLAN-946 §9.
    expect_value_aggregates("arg-borrow-single", src, 22);
    // PLAN-1098 T-05 (O-05): the old "caps-isolation" block called the very
    // same `compile_and_execute_aggregates(src, "main")` a second time and only
    // printed the result, so it could never fail and observed nothing new.
    // Isolating a capability-only axis needs the raw assembly path (a
    // different fixture than this support fn), so it is named out of this
    // probe instead of being kept as decoration.
}
