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

/// PLAN-962 unit-2: whole-element VIEW reads — `xs.get(0)` over a nested
/// list borrows the element (LIST_PLACE + PLACE_GET under the hood);
/// read-through chains execute and the borrow never escapes.
#[test]
fn nested_element_view_read_runs() {
    let src = "fn main() int {\n    var xs = List<List<int>>.new()\n    var inner = List<int>.new()\n    inner.push(38)\n    inner.push(4)\n    xs.push(inner.move)\n    return xs.get(0).get(0) + xs.get(0).get(1)\n}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("chained whole-element view read executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
}

/// PLAN-962 unit-2: for-in element views of a nested list — the binding
/// reads through as a view (`row.get(i)`); the list is not emptied.
#[test]
fn for_in_nested_list_view_binding_reads() {
    let src = "fn main() int {\n    var xs = List<List<int>>.new()\n    var a = List<int>.new()\n    a.push(20)\n    a.push(1)\n    xs.push(a.move)\n    var b = List<int>.new()\n    b.push(21)\n    xs.push(b.move)\n    var s = 0\n    for row in xs {\n        s = s + row.get(0)\n    }\n    return s + 1\n}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("for-in view binding over a nested list executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
}

/// PLAN-962 unit-2 negative controls: every materializing use of a
/// whole-element view rejects with `source.view-escape`.
#[test]
fn element_view_materialization_rejects() {
    let base = "fn main() int {\n    var xs = List<List<int>>.new()\n    var inner = List<int>.new()\n    inner.push(1)\n    xs.push(inner.move)\n    RETURN\n}";
    for (name, ret) in [
        ("let-bind", "let y = xs.get(0)\n    return y.get(0)"),
        (
            "assign",
            "var y = List<List<int>>.new()\n    y = xs.get(0)\n    return 1",
        ),
    ] {
        let src = base.replace("RETURN", ret);
        let err = ace_core::source::compile_aggregates_source(src.as_str(), "p.at", &[])
            .err()
            .unwrap_or_else(|| panic!("{name} must reject"));
        assert!(
            err.iter().any(|d| d.code == "source.view-escape"),
            "{name}: expected view-escape, got {:?}",
            err.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
        );
    }
    // Storing a view into another list rejects too.
    let src = "fn main() int {\n    var xs = List<List<int>>.new()\n    var inner = List<int>.new()\n    inner.push(1)\n    xs.push(inner.move)\n    var ys = List<List<int>>.new()\n    ys.push(xs.get(0))\n    return 1\n}";
    let err = ace_core::source::compile_aggregates_source(src, "p.at", &[])
        .err()
        .expect("pushing a view must reject");
    assert!(err.iter().any(|d| d.code == "source.view-escape"));
}

/// PLAN-962 unit-3 (L-19): range for-in `for i in a .. b` / `a ..= b`
/// desugars to a counted i32 loop. Bounds need spaces around `..` in
/// this profile (the affix-num scanner takes `1..5` as one token —
/// documented boundary, see the negative control below).
#[test]
fn range_for_in_counts_and_executes() {
    let src = "fn main() int {\n    var s = 0\n    for i in 1 .. 5 {\n        s = s + i\n    }\n    for j in 1 ..= 3 {\n        s = s + j * 10\n    }\n    return s\n}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("range for-in executes");
    assert_eq!(run.value, 10 + 60); // 1+2+3+4 + 10+20+30
    assert_eq!(run.live_after, 0);

    // Negative: a range outside the iterable position rejects; the
    // compact unspaced form is a named affix rejection, not a pass.
    let bad = "fn main() int {\n    let r = 1 .. 5\n    return 1\n}";
    let err = ace_core::source::compile_aggregates_source(bad, "p.at", &[])
        .err()
        .expect("range-as-value must reject");
    // `..` outside for-range rejects at the grammar layer; the resolver's
    // source.range-position is defense-in-depth for programmatic ASTs.
    assert!(
        err.iter()
            .any(|d| d.code == "source.syntax" || d.code == "source.range-position")
    );
}

/// PLAN-962 unit-4 (L-16/L-48): array literals bind through the
/// statement-level desugar (List<T>.new() + pushes); index ASSIGN
/// `xs[i] = v` replaces elements in place. Positional index READS
/// (`xs[i]`) ride the `.get(i)` method form — the postfix-bracket shape
/// is a named pending item (CST pratt machinery, see the plan §9).
#[test]
fn array_literal_and_index_assign_execute() {
    let src = "fn main() int {
    var xs = [20, 22]
    xs[0] = 20
    var ys = [10, 5, 3]
    ys[1] = 0
    return xs.get(0) + xs.get(1) + ys.get(1)
}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("literal + index-assign chain executes");
    assert_eq!(run.value, 42); // 20 + 22 + 0
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
}

/// PLAN-962 unit-4 negative controls: empty literals, mixed element
/// types and non-List index bases reject with named diagnostics.
#[test]
fn array_literal_negative_controls() {
    for (name, src) in [
        (
            "empty",
            "fn main() int {\n    let a = []\n    return 1\n}",
        ),
        (
            "mixed",
            "fn main() int {\n    let a = [1, \"x\"]\n    return 1\n}",
        ),
        (
            "position",
            "fn main() int {\n    let a = [1, 2]\n    return [3].get(0)\n}",
        ),
    ] {
        let err = ace_core::source::compile_aggregates_source(src, "p.at", &[])
            .err()
            .unwrap_or_else(|| panic!("{name} must reject"));
        // `[]` rejects at the grammar layer (array-elems requires a
        // head); the resolver's array-lit-empty is defense-in-depth.
        assert!(
            err.iter().any(|d| {
                d.code == "source.array-lit-empty"
                    || d.code == "source.type-mismatch"
                    || d.code == "source.array-lit-position"
                    || (name == "empty" && d.code == "source.syntax")
            }),
            "{name}: unexpected diagnostics {:?}",
            err.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
        );
    }
}

/// PLAN-962 unit-5 (L-15): tuple type/literal + flat let-destructuring —
/// structural shapes intern in the resolver; the adapter materializes the
/// anonymous positional record (`t_rec___tuple_<i>`, fields "0".."n-1"),
/// so descriptor/verify/lowering/backends all ride the record machinery.
#[test]
fn tuple_literal_and_destructuring_execute() {
    let src = "fn main() int {\n    let t = (40, 2)\n    let (a, b) = t\n    return a + b\n}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("tuple construct+destructure executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);

    // annotated form + nested shape round-trips through a function param
    // annotated local + re-construct from the destructured parts
    let src2 = "fn main() int {
    let t (int, int) = (19, 21)
    let (x, y) = t
    let u = (x + 1, y + 1)
    let (p, q) = u
    return p + q
}";
    let run2 = ace_av::collections_864_support::compile_and_execute_aggregates(src2, "d_main")
        .expect("annotated tuple + re-construct executes");
    assert_eq!(run2.value, 42);
    // tuple as a CALL ARGUMENT: record params arrive as views (771
    // semantics); destructuring projects off the param directly (no
    // view->owned materialization).
    let src3 = "fn sum(t (int, int)) int {
    let (x, y) = t
    return x + y
}
fn main() int {
    return sum((20, 22))
}";
    let run3 = ace_av::collections_864_support::compile_and_execute_aggregates(src3, "d_main")
        .expect("tuple call-argument executes");
    assert_eq!(run3.value, 42);
}

/// PLAN-962 unit-5 negative controls: arity mismatch and non-tuple
/// destructuring reject with named diagnostics.
#[test]
fn tuple_destructuring_negative_controls() {
    for (name, src) in [
        (
            "arity",
            "fn main() int {\n    let (a, b, c) = (1, 2)\n    return a\n}",
        ),
        (
            "non-tuple",
            "fn main() int {\n    let (a, b) = 3\n    return a\n}",
        ),
    ] {
        let err = ace_core::source::compile_aggregates_source(src, "p.at", &[])
            .err()
            .unwrap_or_else(|| panic!("{name} must reject"));
        assert!(
            err.iter().any(|d| d.code == "source.type-mismatch"),
            "{name}: unexpected {:?}",
            err.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
        );
    }
}

/// PLAN-962 unit-6: composite record fields — a record owning a List
/// field constructs with an owned rvalue (`k.move`), the field read is a
/// whole-element view path (`n.kids.get(i)` reads through), and the
/// record's release cascades to the field's arena buffer.
#[test]
fn composite_record_list_field_executes() {
    let src = "type Node { name Str; kids List<int> }\nfn main() int {\n    var k = [19, 23]\n    var n = Node(\"x\", k.move)\n    return n.kids.get(0) + n.kids.get(1)\n}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("composite record field chain executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
}

/// PLAN-962 unit-6: named-constructor verification (R03, L-10) on the
/// aggregates surface — out-of-order named fields construct correctly.
#[test]
fn named_ctor_out_of_order_constructs() {
    let src = "type Pair { left int; right int }\nfn main() int {\n    let p = Pair { right: 2, left: 40 }\n    return p.left + p.right\n}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("named ctor executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
}

/// PLAN-962 unit-7 (T-08, L-13/L-12/L-09): `tag N {...}` aliases an enum
/// declaration; `union N {...}` declares a tagged safe union (one arm per
/// field — reading an inactive arm simply fails the is-guard); `take(x)`
/// is the deprecated Move alias.
#[test]
fn tag_union_and_take_execute() {
    let src = "tag Mark { Hit(int); Miss(int) }\nunion Val { Num(int); Flag(int) }\nfn main() int {\n    let m = Mark.Hit(20)\n    let v = Val.Num(22)\n    is m {\n        Mark.Hit(a) -> {\n            is v {\n                Val.Num(b) -> { return a + b }\n                Val.Flag(b) -> { return b }\n            }\n        }\n        Mark.Miss(a) -> { return a }\n    }\n}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("tag/union/take chain executes");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);

    // take() transfers ownership like .move
    let src2 = "fn main() int {\n    var k = [20, 22]\n    let n = take(k)\n    return n.get(0) + n.get(1)\n}";
    let run2 = ace_av::collections_864_support::compile_and_execute_aggregates(src2, "d_main")
        .expect("take alias executes");
    assert_eq!(run2.value, 42);
    assert_eq!(run2.live_after, 0);
    assert_eq!(run2.allocs, run2.frees);
}

/// PLAN-962 unit-8 (T-08, L-08): Box<T> explicit indirection — a
/// single-element List cell (t01-freeze D9). `type Node { next Box<Node> }`
/// is the LEGAL recursion form (the record-cycle walk does not follow the
/// list-typed field edge; a direct `next Node` still rejects — see
/// recursive_record_rejected_before_layout); the chain's last owner frees
/// every cell (V-08 Box destruct slice).
#[test]
#[test]
fn box_indirection_legalizes_recursion_and_frees() {
    // V-04 both sides at the VERIFY layer: the Box-typed field is a
    // list-typed edge the record containment cycle walk does not follow.
    let ok = "type Node { v int; next Box<Node> }
fn main() int { return 1 }";
    ace_core::source::compile_aggregates_source(ok, "p.at", &[])
        .expect("Box recursion legalizes through verify");
    // The recursive chain lowers through the opaque-gated two-phase path
    // (round-11 refined fix): the type table stays byte-identical for
    // every non-recursive module.
    let src = "type Node { v int; next Box<Node> }
fn main() int {
    let tail = Node(41, List<Node>.new())
    let cell = Box(tail)
    let n = Node(1, cell.move)
    return n.v + n.next.get(0).v
}";
    let run = ace_av::collections_864_support::compile_and_execute_aggregates(src, "d_main")
        .expect("recursive Box chain executes");
    assert_eq!(run.value, 42); // n.v(1) + tail.v(41)
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
}
