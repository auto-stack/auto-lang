//! PLAN-881 T-03 semantics tier (`av_method_881` semantics family):
//! V-01/02/04/13 — declaration+call coverage, evaluation-order/identity,
//! admission/isolation negatives, recursion/mutual-call closure — for the
//! record (av.data/1) domain through the SHARED lowering. Oracles are
//! frozen hand-audited values and mark sequences, never outputs of the
//! implementation under test.

use auto_av_core::abc::encode_abc;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value, TRACE_CAPABILITY};
use auto_av_core::mir::TraceEvent;
use auto_av_core::{abc_verify, mir_verify};
use auto_av_probe::data_session::DataIncrementalSession;

const P01: &str = include_str!("../../../ac-core/fixtures/record-source/valid/p01-records.at");

/// Direct data-pipeline run (lower → verify → encode → admit → execute)
/// with trace capability; returns (outcome, trace events).
fn run_data_traced(src: &str) -> (ExecOutcome, Vec<TraceEvent>) {
    let caps = [TRACE_CAPABILITY.to_string()];
    let compiled = auto_ac_prototype::source::compile_record_source(src, "sem.at", &caps)
        .expect("record source compiles");
    let mir = auto_mir_lower::lower_checked_data(&compiled.checked, "d_main", 0x881)
        .expect("data lowering");
    mir_verify::verify_mir(&mir).expect("data MIR verifies");
    let wire = encode_abc(&mir).expect("AVAD encode");
    let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &mir.owner).expect("AVAD admits");
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    (out, trace.events)
}

fn success_i32(src: &str) -> i32 {
    match run_data_traced(src).0 {
        ExecOutcome::Success { value: Value::I32(v), .. } => v,
        other => panic!("expected i32 success, got {other:?}"),
    }
}

fn compile_err(src: &str) -> String {
    auto_ac_prototype::source::compile_record_source(src, "neg.at", &[])
        .err()
        .and_then(|ds| ds.first().map(|d| d.code.clone()))
        .unwrap_or_else(|| "<compiled>".to_string())
}

// ---------------------------------------------------------------------------
// V-01: declaration and call coverage (whole + unit), 771 flagship
// ---------------------------------------------------------------------------

/// PLAN-881 V-09 closed the ORIGINAL flagship's shared-path boundary:
/// the free-fn mut-record borrow (`bump(q.mut, 5)`) now rides the
/// by-value t_MutRet write-back ABI on av.data/1 (records are pure
/// values — Copy temp + Copy write-back; exclusivity stays the
/// source-level `verify.mut-conflict` contract). The ORIGINAL fixture
/// runs whole AND unit below; the old-native differential lives in the
/// mn tier.

#[test]
fn v01_771_p01_whole_path_zero() {
    assert_eq!(success_i32(P01), 0, "771 flagship p01 = 0 through shared lowering");
}

#[test]
fn v01_771_p01_unit_path_zero() {
    let sess = DataIncrementalSession::warmup(P01, "d_main", &[]).expect("warmup");
    assert_eq!(sess.run().expect("run"), Value::I32(0));
    // The fixture's methods are residents alongside the free functions.
    let order = sess.fn_order();
    assert!(order.contains(&"m.Pair.new".to_string()), "{order:?}");
    assert!(order.contains(&"m.Pair.left_is".to_string()), "{order:?}");
    assert!(order.contains(&"d_bump".to_string()), "{order:?}");
}

// ---------------------------------------------------------------------------
// V-02: evaluation order + identity
// ---------------------------------------------------------------------------

/// The 771 receiver-once oracle through the SHARED chain: the receiver
/// (ctor arg mark_a) evaluates exactly once FIRST, then mark_a, mark_b —
/// trace `aab`, exit 0.
#[test]
fn v02_receiver_once_then_args_in_order_trace_aab() {
    let src = "\
type P {
    v int
    static fn mk(x int) P { return P { v: x } }
    fn plus(a int, b int) int { return .v + a + b }
}
fn main() int {
    let r = P.mk(mark_a()).plus(mark_a(), mark_b())
    if r != 4 { return 9 }
    return 0
}
";
    let (out, trace) = run_data_traced(src);
    assert!(
        matches!(out, ExecOutcome::Success { value: Value::I32(0), .. }),
        "exit 0, got {out:?}"
    );
    assert_eq!(
        trace,
        vec![TraceEvent::MarkA, TraceEvent::MarkA, TraceEvent::MarkB],
        "receiver once FIRST, then args in source order (aab)"
    );
}

/// Named args bind by name on method calls, evaluated in SOURCE order.
#[test]
fn v02_method_named_args_reverse_order() {
    let src = "\
type P {
    v int
    w int
    static fn mk(a int, b int) P { return P { v: a, w: b } }
    fn pick(k int, j int) int { return .v * 100 + .w * 10 + k + j }
}
fn main() int {
    let p = P.mk(7, 3)
    return p.pick(j: 1, k: 2)
}
";
    // v=7, w=3 → 700 + 30 + k(2) + j(1) = 733
    assert_eq!(success_i32(src), 733);
}

/// Same-named methods on different records and a same-named free function
/// never cross-wire (the owner-based `m.<Rec>.<name>` identity).
#[test]
fn v02_same_name_method_free_fn_no_cross_wiring() {
    let src = "\
type A { v int
    fn f() int { return .v + 1 }
    static fn mk(n int) A { return A { v: n } }
}
type B { v int
    fn f() int { return .v + 2 }
    static fn mk(n int) B { return B { v: n } }
}
fn f(x int) int { return x + 100 }
fn main() int {
    let a = A.mk(10)
    let b = B.mk(10)
    return a.f() * 1000 + b.f() * 10 + f(0)
}
";
    // 11 * 1000 + 12 * 10 + 100 = 11120 + 120... hand-audit: 11000+120+100 = 11220
    assert_eq!(success_i32(src), 11220);
}

/// A method invoking another method on a field path (nested receiver),
/// and a method invoked before its declaring type appears in source
/// order are both just call-graph facts of the shared lowering.
#[test]
fn v02_nested_field_receiver_and_forward_calls() {
    let src = "\
type Leaf { v int
    fn get() int { return .v }
    static fn mk(n int) Leaf { return Leaf { v: n } }
}
type Node { leaf Leaf
    fn leaf_get() int { return .leaf.get() }
    static fn mk(l Leaf) Node { return Node { leaf: l } }
}
fn use_node(n Node) int { return n.leaf_get() }
fn main() int {
    let n = Node.mk(Leaf.mk(40))
    return use_node(n) + 2
}
";
    assert_eq!(success_i32(src), 42);
}

// ---------------------------------------------------------------------------
// V-04: admission / isolation negatives (old rejection set preserved)
// ---------------------------------------------------------------------------

/// `Pair.get()` — calling an instance method through the TYPE (static
/// mis-call) is rejected with the existing not-callable family.
#[test]
fn v04_static_miscall_of_instance_method_rejected() {
    let src = "\
type P { v int
    fn get() int { return .v }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    let p = P.mk(1)
    return P.get(p)
}
";
    let code = compile_err(src);
    assert!(
        code.starts_with("source."),
        "a source-level rejection (got `{code}`), never an unsupported escape"
    );
}

/// Writing a field through an immutable (let/view) binding INSIDE a
/// method body is the frozen immutable-write family (i01 shape). The
/// receiver itself has no write syntax at all — `self` is implicit and
/// `.v = n` does not parse — which is the strongest form of the view
/// contract; the HIR-level gate additionally rejects a mutated receiver
/// local (verify.rs).
#[test]
fn v04_view_write_inside_method_rejected() {
    let src = "type P { v int
    fn bad(n int) int {
        let q = P { v: n }
        q.v = n + 1
        return q.v
    }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    let p = P.mk(1)
    return p.bad(2)
}
";
    assert_eq!(compile_err(src), "verify.immutable-write");
}

/// `mut fn` (mutable instance methods) stays outside the profile — the
/// old rejection, now at the method-declaration door.
#[test]
fn v04_mut_instance_method_rejected() {
    let src = "type P { v int
    mut fn bad() int { return .v }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    let p = P.mk(1)
    return p.bad()
}
";
    let code = compile_err(src);
    assert!(
        code.starts_with("source."),
        "mut fn rejected at the source door (got `{code}`)"
    );
}

/// mut-conflict: passing the same root as view receiver AND a mut arg.
#[test]
fn v04_receiver_and_mut_arg_same_root_rejected() {
    let src = "\
type P { v int
    fn mix(mut q P, d int) int { q.v = q.v + d; return .v * 10 + q.v }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    let p = P.mk(7)
    return p.mix(p.mut, 3)
}
";
    assert_eq!(compile_err(src), "verify.mut-conflict");
}

/// Same-named method on two types: calling through the WRONG owner's
/// value does not resolve (nominal dispatch).
#[test]
fn v04_wrong_owner_method_rejected() {
    let src = "\
type A { v int
    fn f() int { return 1 }
    static fn mk(n int) A { return A { v: n } }
}
type B { v int
    static fn mk(n int) B { return B { v: n } }
}
fn main() int {
    let b = B.mk(1)
    return b.f()
}
";
    let code = compile_err(src);
    assert!(
        code.starts_with("source."),
        "record B has no method f (got `{code}`)"
    );
}

/// The scalar/core surface keeps rejecting type-bound methods — the
/// SHARED lowering stays unique; no second method runtime grows there.
#[test]
fn v04_scalar_and_collections_surfaces_still_reject_methods() {
    let src = "\
type C { v int
    static fn new(n int) C { return C(n) }
}
fn main() int {
    let c = C.new(1)
    return c.v
}
";
    // The SCALAR front door rejects the record surface outright.
    let code = auto_ac_prototype::source::compile_scalar_source(src, "s.at", &[])
        .err()
        .and_then(|ds| ds.first().map(|d| d.code.clone()))
        .unwrap_or_default();
    assert!(
        code.starts_with("source."),
        "scalar profile rejects records (got `{code}`)"
    );
    // And the scalar LOWERING of a method-bearing record HIR (escalated
    // domain) is a controlled `lower.method` — the 871 gap row's guard.
    let compiled = auto_ac_prototype::source::compile_record_source(src, "s.at", &[])
        .expect("record compiles");
    // Source-driven scalar lowering refuses at the DOMAIN gate (records
    // are outside av.core-i32/1); the method-level guard additionally
    // covers hand-made scalar HIR. Either way nothing method-shaped ever
    // passes the scalar surface.
    let err = auto_mir_lower::lower_checked(&compiled.checked, "d_main", 0)
        .expect_err("scalar lowering refuses record modules");
    assert!(
        err.code() == "lower.domain" || err.code() == "lower.method",
        "got {}",
        err.code()
    );
}

// ---------------------------------------------------------------------------
// V-13: recursion, mutual calls, call closure
// ---------------------------------------------------------------------------

/// Recursive static and view methods with a termination condition
/// (base 42, step counter −1); whole-module AND unit paths truly link the
/// self import and the cross-method closure.
#[test]
fn v13_recursive_static_and_view_methods() {
    let src = "\
type R {
    n int
    static fn down(k int) int {
        if k <= 0 { return 42 }
        return R.down(k - 1)
    }
    fn vdown(k int) int {
        if k <= 0 { return .n }
        return self.vdown(k - 1)
    }
    static fn mk(n int) R { return R { n: n } }
}
fn main() int {
    let r = R.mk(42)
    return R.down(9) + r.vdown(9)
}
";
    assert_eq!(success_i32(src), 84, "42 + 42 through true recursive links");
    let sess = DataIncrementalSession::warmup(src, "d_main", &[]).expect("warmup");
    assert_eq!(sess.run().expect("unit run"), Value::I32(84));
}

/// method → free function → method chains link through the unit closure.
#[test]
fn v13_method_free_method_chain() {
    let real = "\
type T {
    v int
    fn outer() int { return helper(.v) }
    fn inner() int { return .v * 2 }
    static fn mk(n int) T { return T { v: n } }
}
fn helper(x int) int { return T.mk(x + 1).inner() }
fn main() int {
    let t = T.mk(20)
    return t.outer()
}
";
    // outer(20) → helper(20) → T.mk(21).inner() = 42
    assert_eq!(success_i32(real), 42);
    let sess = DataIncrementalSession::warmup(real, "d_main", &[]).expect("warmup");
    assert_eq!(sess.run().expect("unit run"), Value::I32(42));
}
