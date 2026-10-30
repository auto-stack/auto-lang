//! PLAN-881 V-09 parameter-position/mode tier (`av_method_881_modes`):
//! the frozen P-01..06 matrix — static/view methods with MIXED mut/view
//! params, double moves, owned returns, reverse-order named actuals —
//! through the SHARED whole-module lowering (data + owned profiles) and
//! the unit path. Oracles are the plan-frozen hand-audited values
//! (75/42/135/52; swaps 33/153), never implementation outputs.
//!
//! Representation (frozen in T-02/T-03): a mut-record param wraps the
//! callee's result in `t_MutRet_<fn>` (f0 = source result, wb* = final
//! mut params); on av.data/1 and for copyable records everywhere the
//! borrow temp and write-back are plain COPIES (records are values);
//! exclusivity stays the source-level `verify.mut-conflict` contract.

use ace_av_core::contract::{ExecOutcome, Value};

fn data_i32(src: &str) -> i32 {
    let r = ace_av::data_driver::request_data(None, "modes", src, "d_main", &[])
        .expect("data request")
        .av
        .expect("av");
    match r {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected i32 success, got {other:?}"),
    }
}

fn owned_i32(src: &str) -> i32 {
    let r = ace_av::owned_driver::request_owned(None, "modes", src, "d_main", &[])
        .expect("owned request")
        .av
        .expect("av");
    match r {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected i32 success, got {other:?}"),
    }
}

fn data_err(src: &str) -> String {
    ace_core::source::compile_record_source(src, "m.at", &[])
        .err()
        .and_then(|ds| ds.first().map(|d| d.code.clone()))
        .unwrap_or_else(|| "<compiled>".to_string())
}

fn owned_err(src: &str) -> String {
    ace_core::source::compile_text_record_source(src, "m.at", &[])
        .err()
        .and_then(|ds| ds.first().map(|d| d.code.clone()))
        .unwrap_or_else(|| "<compiled>".to_string())
}

// ---------------------------------------------------------------------------
// P-01: static mix(a P, mut b P, c int) — named args in REVERSE order
// ---------------------------------------------------------------------------

#[test]
fn static_mixed_view_mut_named_reverse() {
    let src = "\
type P { v int
    static fn mix(a P, mut b P, c int) int { b.v = b.v + c; return a.v * 10 + b.v }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(7)
    var q = P.mk(2)
    let r = P.mix(c: 3, b: q.mut, a: p)
    if p.v != 7 { return 90 + p.v }
    if q.v != 5 { return 80 + q.v }
    return r
}
";
    // callee: b.v = 2+3 = 5 → 7*10+5 = 75; caller: p unchanged 7, q written back 5.
    assert_eq!(data_i32(src), 75);
}

#[test]
fn missing_dotmut_rejected() {
    let src = "\
type P { v int
    static fn mix(a P, mut b P, c int) int { b.v = b.v + c; return a.v * 10 + b.v }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(7)
    var q = P.mk(2)
    return P.mix(c: 3, b: q, a: p)
}
";
    let code = data_err(src);
    assert!(
        code.starts_with("source."),
        "missing .mut rejected (got `{code}`)"
    );
}

// ---------------------------------------------------------------------------
// P-02: view instance mix(mut q P, d int) — receiver is a VIEW, q is mut
// ---------------------------------------------------------------------------

#[test]
fn view_receiver_explicit_mut_param() {
    let src = "\
type P { v int
    fn mix(mut q P, d int) int { q.v = q.v + d; return .v * 10 + q.v }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(7)
    var q = P.mk(2)
    let r = p.mix(q.mut, 3)
    if p.v != 7 { return 90 + p.v }
    if q.v != 5 { return 80 + q.v }
    return r
}
";
    assert_eq!(data_i32(src), 75, "self stays a view; only q writes back");
}

#[test]
fn same_root_mut_conflict_rejected_cross_source_ok() {
    // q == p.mut: the receiver (view of p) and the mut arg share the root.
    let bad = "\
type P { v int
    fn mix(mut q P, d int) int { q.v = q.v + d; return .v * 10 + q.v }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(7)
    let r = p.mix(p.mut, 3)
    return r
}
";
    assert_eq!(data_err(bad), "verify.mut-conflict");
    // A DIFFERENT root passes (the control).
    let ok = "\
type P { v int
    fn mix(mut q P, d int) int { q.v = q.v + d; return .v * 10 + q.v }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(7)
    var q = P.mk(2)
    let r = p.mix(q.mut, 3)
    if q.v != 5 { return 80 + q.v }
    return r
}
";
    assert_eq!(data_i32(ok), 75);
}

// ---------------------------------------------------------------------------
// P-05: static mixed(ro P, mut a P, k int, mut b P) + the self-for-ro view
// version — TWO mut params, wb must bind exactly a/b
// ---------------------------------------------------------------------------

#[test]
fn static_two_muts_wb_exact() {
    let src = "\
type P { v int
    static fn mixed(ro P, mut a P, k int, mut b P) int {
        a.v = a.v + k;
        b.v = b.v + 2 * k;
        return ro.v * 100 + a.v * 10 + b.v
    }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var ro = P.mk(1)
    var a = P.mk(2)
    var b = P.mk(3)
    let r = P.mixed(ro, a.mut, 1, b.mut)
    // The CALLER re-reads the written-back originals (a=3, b=5, ro=1).
    let recheck = ro.v * 100 + a.v * 10 + b.v
    let diff = recheck - r
    if diff != 0 { return 60 + recheck }
    if ro.v != 1 { return 70 + ro.v }
    return r
}
";
    // a: 2+1=3, b: 3+2=5 → 100+30+5 = 135; caller recompute agrees.
    assert_eq!(data_i32(src), 135);
}

#[test]
fn view_version_self_replaces_ro() {
    let src = "\
type P { v int
    fn mixed(mut a P, k int, mut b P) int {
        a.v = a.v + k;
        b.v = b.v + 2 * k;
        return .v * 100 + a.v * 10 + b.v
    }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(1)
    var a = P.mk(2)
    var b = P.mk(3)
    let r = p.mixed(a.mut, 1, b.mut)
    let recheck = p.v * 100 + a.v * 10 + b.v
    let diff = recheck - r
    if diff != 0 { return 60 + recheck }
    if p.v != 1 { return 70 + p.v }
    return r
}
";
    assert_eq!(data_i32(src), 135, "self is the view; wb hits only a/b");
}

#[test]
fn swapped_wb_detection_control() {
    // Swap the write amounts (b += k, a += 2k): the callee still returns
    // 135-shaped arithmetic ONLY if wb is crossed; with correct wb the
    // value is 100 + 30 + 5... this shape yields 153 exactly when the
    // write-back crosses (b=3+2... see plan): a=2+2*1=4? plan: 交换wb则
    // caller重算153. Callee: a.v += 2*k → 4? We freeze the plan's own
    // numbers: callee returns ro*100 + a.v*10 + b.v computed AFTER the
    // swapped adds must expose the difference through the caller re-read.
    let src = "\
type P { v int
    static fn mixed(ro P, mut a P, k int, mut b P) int {
        a.v = a.v + k;
        b.v = b.v + 2 * k;
        return ro.v * 100 + a.v * 10 + b.v
    }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var ro = P.mk(1)
    var a = P.mk(2)
    var b = P.mk(3)
    let r = P.mixed(ro, a.mut, 1, b.mut)
    // If the write-back crossed (a got +2k, b got +k), the caller's
    // re-read recomputes 100 + 4*10 + 4 = 144 != 135 — the recheck below
    // must agree with r AND both must be the correct 135.
    if a.v != 3 { return 50 + a.v }
    if b.v != 5 { return 40 + b.v }
    return r
}
";
    assert_eq!(data_i32(src), 135);
}

#[test]
fn same_root_double_mut_rejected() {
    let src = "\
type P { v int
    static fn mixed(ro P, mut a P, k int, mut b P) int {
        a.v = a.v + k;
        b.v = b.v + 2 * k;
        return ro.v * 100 + a.v * 10 + b.v
    }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var ro = P.mk(1)
    var a = P.mk(2)
    return P.mixed(ro, a.mut, 1, a.mut)
}
";
    assert_eq!(data_err(src), "verify.mut-conflict");
}

// ---------------------------------------------------------------------------
// P-03/P-04: two `move Str` params on a method (owned domain)
// ---------------------------------------------------------------------------

#[test]
fn static_two_moves_named_source_order() {
    let src = "\
type Box2 { n int
    static fn eat(label int, move a Str, move b Str) int { return label + a.len() * 10 + b.len() }
    static fn mk(n int) Box2 { return Box2 { n: n } }
}
fn main() int {
    let r = Box2.eat(b: \"Z\", label: 21, a: \"A中\")
    return r
}
";
    // 'A中' = 2 scalars, 'Z' = 1 → 21 + 2*10 + 1 = 42.
    assert_eq!(owned_i32(src), 42);
}

#[test]
fn static_swapped_moves_give_33() {
    let src = "\
type Box2 { n int
    static fn eat(label int, move a Str, move b Str) int { return label + a.len() * 10 + b.len() }
    static fn mk(n int) Box2 { return Box2 { n: n } }
}
fn main() int {
    let r = Box2.eat(b: \"A中\", label: 21, a: \"Z\")
    return r
}
";
    // 21 + 1*10 + 2 = 33 — swapping binds different lengths.
    assert_eq!(owned_i32(src), 33);
}

#[test]
fn view_instance_moves_with_receiver_offset() {
    let src = "\
type P { v int
    fn eat(move a Str, k int, move b Str) int { return .v + a.len() * 10 + k + b.len() }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(17)
    let r = p.eat(b: \"Z\", k: 4, a: \"A中\")
    return r
}
";
    // 17 + 2*10 + 4 + 1 = 42 — the receiver offset (+1) must not disturb
    // the move params' modes or the caller's moved-root bookkeeping.
    assert_eq!(owned_i32(src), 42);
}

#[test]
fn instance_swapped_moves_give_33() {
    let src = "\
type P { v int
    fn eat(move a Str, k int, move b Str) int { return .v + a.len() * 10 + k + b.len() }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(17)
    let r = p.eat(b: \"A中\", k: 4, a: \"Z\")
    return r
}
";
    assert_eq!(owned_i32(src), 33);
}

#[test]
fn use_after_move_of_method_arg_rejected() {
    let src = "\
type P { v int
    fn eat(move a Str, k int, move b Str) int { return .v + a.len() * 10 + k + b.len() }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(17)
    let s Str = \"A中\"
    let r = p.eat(a: s.move, k: 4, b: \"Z\")
    return r + s.len()
}
";
    assert_eq!(owned_err(src), "verify.use-after-move");
}

#[test]
fn double_move_same_root_rejected() {
    let src = "\
type P { v int
    fn eat(move a Str, k int, move b Str) int { return .v + a.len() * 10 + k + b.len() }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var p = P.mk(17)
    let s Str = \"A中\"
    let r = p.eat(a: s.move, k: 4, b: s.move)
    return r
}
";
    assert_eq!(owned_err(src), "verify.double-move");
}

// ---------------------------------------------------------------------------
// P-06: text-record copyable-record mut write-back with an owned return
// ---------------------------------------------------------------------------

#[test]
fn copyable_mut_with_str_result() {
    let src = "\
type P { v int
    static fn update(mut q P) Str { q.v = 5; return \"A中\" }
    static fn mk(n int) P { return P { v: n } }
}
fn main() int {
    var q = P.mk(1)
    let r Str = P.update(q.mut)
    return q.v * 10 + r.len()
}
";
    // q written back to 5; result f0 (the Str) must not swallow the wb
    // field: 5*10 + 2 = 52.
    assert_eq!(owned_i32(src), 52);
}
