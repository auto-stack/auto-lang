//! PLAN-818 T-03 codegen tier (`mir_native_codegen_`): the full limited
//! instruction surface through the REAL new pipeline — arithmetic
//! boundaries (overflow / divide traps), negative truncation semantics,
//! comparisons, loops with break/continue, slot reassignment, call shapes
//! (>4 args, mixed bool/i32, recursion, mutual recursion), trace orders,
//! latent traps on unexecuted branches — plus MIR-level admission
//! negatives rejected with codegen count 0 (V-01/V-02/V-03 first halves).

#[path = "../common/mod.rs"]
mod common;

use ace_av_core::contract::AV_PROFILE;
use ace_av_core::mir::*;
use ace_native::{emit, MnError};

fn case(src: &str) -> String {
    src.to_string()
}

/// Arithmetic boundaries: every case is a whole program whose INDEPENDENT
/// expected result is the PE exit code (hand-computed; trap = 70).
#[test]
#[cfg(windows)]
fn mir_native_codegen_arithmetic_boundaries() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    // (name, source, expected exit)
    let cases: Vec<(&str, String, i32)> = vec![
        (
            "add-overflow",
            case("fn main() int {\n    var a = 2147483647\n    a = a + 1\n    return 9\n}\n"),
            70,
        ),
        (
            "sub-overflow",
            case("fn main() int {\n    var m = 0 - 2147483647\n    m = m - 1\n    return m - 1\n}\n"),
            70,
        ),
        (
            "mul-overflow",
            case("fn main() int {\n    var a = 65536\n    a = a * 65536\n    return a\n}\n"),
            70,
        ),
        (
            "neg-overflow",
            // 0 - MIN overflows (the negation guard fires on isub room).
            case("fn main() int {\n    var m = 0 - 2147483647\n    m = m - 1\n    return 0 - m\n}\n"),
            70,
        ),
        (
            "div-by-zero",
            case("fn main() int {\n    return 7 / 0\n}\n"),
            70,
        ),
        (
            "rem-by-zero",
            case("fn main() int {\n    return 7 % 0\n}\n"),
            70,
        ),
        (
            "div-min-minus-one",
            case("fn main() int {\n    var m = 0 - 2147483647\n    m = m - 1\n    var d = 0 - 1\n    return m / d\n}\n"),
            70,
        ),
        (
            "rem-min-minus-one",
            case("fn main() int {\n    var m = 0 - 2147483647\n    m = m - 1\n    var d = 0 - 1\n    return m % d\n}\n"),
            70,
        ),
        // Truncation toward zero, independent of host `%` semantics.
        (
            "neg-div-trunc",
            case("fn main() int {\n    return (0 - 7) / 2\n}\n"),
            -3,
        ),
        (
            "neg-rem-sign",
            case("fn main() int {\n    return (0 - 7) % 2\n}\n"),
            -1,
        ),
        (
            "pos-div-neg-divisor",
            case("fn main() int {\n    return 7 / (0 - 2)\n}\n"),
            -3,
        ),
        (
            "pos-rem-neg-divisor",
            case("fn main() int {\n    return 7 % (0 - 2)\n}\n"),
            1,
        ),
        // Exact boundaries that DO fit.
        (
            "add-at-max",
            case("fn main() int {\n    var a = 2147483646\n    return a + 1\n}\n"),
            2147483647,
        ),
        (
            "mul-exact",
            case("fn main() int {\n    var a = 46341\n    a = a - 1\n    return a * a\n}\n"),
            46340 * 46340,
        ),
        (
            "div-exact-neg",
            case("fn main() int {\n    var d = 0 - 4\n    return (0 - 8) / d\n}\n"),
            2,
        ),
        // Comparison boundaries (bool results folded into i32 returns).
        (
            "cmp-min-max",
            case("fn main() int {\n    var mn = 0 - 2147483647\n    mn = mn - 1\n    if mn < 2147483647 {\n        return 1\n    }\n    return 0\n}\n"),
            1,
        ),
        (
            "cmp-eq-ne",
            case("fn main() int {\n    var a = 3\n    var b = 3\n    var r = 0\n    if a == b {\n        r = r + 1\n    }\n    if a != b {\n        r = r + 10\n    }\n    if a >= b {\n        r = r + 100\n    }\n    if a <= b {\n        r = r + 1000\n    }\n    if a > b {\n        r = r + 10000\n    }\n    return r\n}\n"),
            1101,
        ),
        (
            "bool-not",
            case("fn main() int {\n    var t = true\n    var f = false\n    var r = 0\n    if !t {\n        r = r + 1\n    }\n    if !f {\n        r = r + 10\n    }\n    if !!f {\n        r = r + 100\n    }\n    return r\n}\n"),
            10,
        ),
    ];
    for (name, src, want) in cases {
        let r = common::run_mn(&src, &[], name);
        assert!(r.ok, "{name}: pipeline failed: {:?}", r.error);
        assert_eq!(
            r.exit_code,
            Some(want),
            "{name}: exit must match the independent expected"
        );
    }
}

/// A trap on an unexecuted branch stays LATENT — no folding into an
/// unconditional compile failure, no eager evaluation.
#[test]
#[cfg(windows)]
fn mir_native_codegen_latent_trap_unexecuted_branch() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    let src = "fn main() int {\n    if 1 > 2 {\n        return 1 / 0\n    }\n    return 41\n}\n";
    let r = common::run_mn(src, &[], "latent");
    assert!(r.ok, "latent trap case: {:?}", r.error);
    assert_eq!(r.exit_code, Some(41), "the untaken trap must not fire");
    // The trap on the taken path DOES fire.
    let src2 = "fn main() int {\n    if 2 > 1 {\n        return 1 / 0\n    }\n    return 41\n}\n";
    let r2 = common::run_mn(src2, &[], "latent-taken");
    assert!(r2.ok);
    assert_eq!(r2.exit_code, Some(70), "the taken trap fires as exit 70");
}

/// Guard loops with break/continue, slot reassignment and bool slots
/// (V-03 slot axis; expected values hand-derived, mirroring the 798
/// frozen loop shapes but with fresh constants).
#[test]
#[cfg(windows)]
fn mir_native_codegen_loops_break_continue_slots() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    // break/continue: i=1..4, skip 2, break at 4 → acc = 1 + 3 = 4.
    let src = "fn main() int {\n    var i = 0\n    var acc = 0\n    for i < 4 {\n        i = i + 1\n        if i == 2 {\n            continue\n        }\n        if i == 4 {\n            break\n        }\n        acc = acc + i\n    }\n    return acc\n}\n";
    let r = common::run_mn(src, &[], "loop-bc");
    assert!(r.ok, "{:?}", r.error);
    assert_eq!(r.exit_code, Some(4), "skip 2, break at 4: acc = 1 + 3");

    // Slot reassignment across a loop with a bool latch and a zero-iteration
    // path (initialization does not escape the loop body).
    let src2 = "fn main() int {\n    var n = 0\n    var total = 100\n    var run = false\n    for run {\n        n = n + 5\n    }\n    return total + n\n}\n";
    let r2 = common::run_mn(src2, &[], "loop-zero");
    assert!(r2.ok, "{:?}", r2.error);
    assert_eq!(
        r2.exit_code,
        Some(100),
        "zero-iteration loop leaves n untouched"
    );

    // Loop-carried slot with reassignment and a merge join.
    let src3 = "fn main() int {\n    var i = 0\n    var acc = 1\n    for i < 5 {\n        if acc > 8 {\n            acc = acc + 100\n        } else {\n            acc = acc * 2\n        }\n        i = i + 1\n    }\n    return acc\n}\n";
    // acc: 1→2→4→8→16→116 (i=4 sees acc=16>8 → +100).
    let r3 = common::run_mn(src3, &[], "loop-merge");
    assert!(r3.ok, "{:?}", r3.error);
    assert_eq!(r3.exit_code, Some(116), "1,2,4,8,16 then +100");
}

/// Call shapes: >4 arguments (stack-passed under Win x64), mixed
/// bool/i32, asymmetric argument expressions, direct and mutual
/// recursion, nested calls (V-03 call axis).
#[test]
#[cfg(windows)]
fn mir_native_codegen_call_shapes() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    // 7 args (3 registers + 4 stack under Win x64), mixed bool/i32,
    // asymmetric expressions. sum = (2+3) + 10 + 20 + 30 + 40 + 50 + 1(b)=156.
    let src = "fn add7(a int, b int, c int, d int, e int, f int, g bool) int {\n    var t = a + b + c + d + e + f\n    if g {\n        t = t + 1\n    }\n    return t\n}\n\nfn main() int {\n    return add7(2 + 3, 10, 20, 30, 40, 50, true)\n}\n";
    let r = common::run_mn(src, &[], "call-7args");
    assert!(r.ok, "{:?}", r.error);
    assert_eq!(r.exit_code, Some(156), "7-arg mixed-signature call");

    // Direct self recursion, bounded: f(6) = 0+1+...? f(n)=f(n-1)+n, f(0)=7 → 7+1+2+3+4+5+6=28.
    let src2 = "fn f(n int) int {\n    if n <= 0 {\n        return 7\n    }\n    return f(n - 1) + n\n}\n\nfn main() int {\n    return f(6)\n}\n";
    let r2 = common::run_mn(src2, &[], "call-selfrec");
    assert!(r2.ok, "{:?}", r2.error);
    assert_eq!(r2.exit_code, Some(28), "7+1+2+3+4+5+6");

    // Mutual recursion: f(4)→g(3)→f(2)→g(1)→f(0)=1.
    let src3 = "fn f(n int) int {\n    if n == 0 {\n        return 1\n    }\n    return g(n - 1)\n}\n\nfn g(n int) int {\n    if n == 0 {\n        return 2\n    }\n    return f(n - 1)\n}\n\nfn main() int {\n    return f(4)\n}\n";
    let r3 = common::run_mn(src3, &[], "call-mutual");
    assert!(r3.ok, "{:?}", r3.error);
    assert_eq!(r3.exit_code, Some(1), "mutual recursion bottoms at f(0)=1");

    // Nested calls with asymmetric evaluation order and a bool helper.
    let src4 = "fn w(x int) int {\n    return x * x\n}\n\nfn flag(v int) bool {\n    return v < 10\n}\n\nfn main() int {\n    var r = 0\n    if flag(w(3)) {\n        r = w(w(2))\n    }\n    return r\n}\n";
    let r4 = common::run_mn(src4, &[], "call-nested");
    assert!(r4.ok, "{:?}", r4.error);
    assert_eq!(r4.exit_code, Some(16), "w(w(2)) = 16, flag(9)=true");
}

/// Trace orders: marks around branches and loops; the untaken arm's mark
/// must NOT appear (stderr compared byte-exact, hand-computed).
#[test]
#[cfg(windows)]
fn mir_native_codegen_trace_orders() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    // First-case shape: "aab" (both gate invocations emit mark_a; only the
    // >40 arm emits mark_b).
    let src = "fn gate(v int) int {\n    let a = mark_a()\n    let t = v * 3 + a\n    if t > 40 {\n        return t - mark_b()\n    }\n    return t + a\n}\n\nfn main() int {\n    let x = gate(9)\n    let y = gate(15)\n    if x + y != 73 {\n        return 1\n    }\n    return 0\n}\n";
    let r = common::run_mn(src, &["hir.test.trace.v1"], "trace-first");
    assert!(r.ok, "{:?}", r.error);
    assert_eq!(r.exit_code, Some(0));
    assert_eq!(r.stderr, b"aab".as_slice(), "marks in source order");

    // Loop with marks: 3 iterations emit "a" each, then "b" once → "aaab".
    let src2 = "fn main() int {\n    var i = 0\n    var acc = 0\n    for i < 3 {\n        let m = mark_a()\n        acc = acc + m\n        i = i + 1\n    }\n    let b = mark_b()\n    return acc * 10 + b\n}\n";
    // acc = 3 (three mark_a=1), exit = 3*10+2 = 32; stderr "aaab".
    let r2 = common::run_mn(src2, &["hir.test.trace.v1"], "trace-loop");
    assert!(r2.ok, "{:?}", r2.error);
    assert_eq!(r2.exit_code, Some(32));
    assert_eq!(r2.stderr, b"aaab".as_slice(), "three loop marks then one b");

    // Untaken arm's mark stays silent: "a" only.
    let src3 = "fn main() int {\n    let a = mark_a()\n    if a > 5 {\n        let b = mark_b()\n        return b\n    }\n    return a\n}\n";
    let r3 = common::run_mn(src3, &["hir.test.trace.v1"], "trace-silent");
    assert!(r3.ok, "{:?}", r3.error);
    assert_eq!(r3.exit_code, Some(1));
    assert_eq!(
        r3.stderr,
        b"a".as_slice(),
        "the untaken arm's mark is silent"
    );
}

/// MIR-level admission negatives: out-of-surface content is rejected with
/// codegen count 0 — precise `mn.unsupported.*` codes, never NOP'd past,
/// never routed to the old lowerer (V-01 negative half).
#[test]
fn mir_native_codegen_bad_mir_admission_rejects_codegen_zero() {
    fn owner() -> OwnerScope {
        OwnerScope {
            schema: "auto.mir.av-probe/1".to_string(),
            module: "m".to_string(),
        }
    }
    fn minimal_unit(profile: &str, is_unit: bool) -> MirModule {
        let mut m = if is_unit {
            MirModule::unit(owner())
        } else {
            MirModule::new(owner())
        };
        m.profile = profile.to_string();
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        let mk = |name: &str| MirFunction {
            env: None,
            owner: owner(),
            name: name.to_string(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: None,
            }],
            blocks: vec![MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
        };
        m.funcs.push(mk("d_main"));
        m.entry = 0;
        m
    }

    // Wrong profile → mn.unsupported.profile, before any codegen.
    let bad = minimal_unit("av.core-i32/9", true);
    match emit::emit_unit_coff(&bad) {
        Err(MnError::Unsupported(k, _)) => assert_eq!(k, "profile"),
        other => panic!("wrong profile must reject as mn.unsupported.profile: {other:?}"),
    }
    // Non-unit module (whole-module shape at the unit door) → unit-shape.
    let bad2 = {
        let mut m = minimal_unit(AV_PROFILE, false);
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        m.funcs.push(MirFunction {
            env: None,
            owner: owner(),
            name: "d_other".to_string(),
            sig,
            slots: vec![MirSlot {
                kind: SlotKind::Local,
                ty: ti,
                name: None,
            }],
            blocks: vec![MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 0 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
        });
        m
    };
    match emit::emit_unit_coff(&bad2) {
        Err(MnError::Unsupported(k, _)) => assert_eq!(k, "unit-shape"),
        other => panic!("whole-module input must reject as unit-shape: {other:?}"),
    }
    // Unknown runtime import → mn.unsupported.import (symbol resolution
    // refuses before any object is built).
    let bad3 = {
        let mut m = minimal_unit(AV_PROFILE, true);
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: ti,
        }));
        m.imports.push(MirImport {
            env: None,
            symbol: "evil.native.ffi".to_string(),
            effect: Effect::Pure,
            sig,
        });
        m
    };
    match emit::emit_unit_coff(&bad3) {
        Err(MnError::Unsupported(k, msg)) => {
            assert_eq!(k, "import");
            assert!(msg.contains("evil.native.ffi"), "{msg}");
        }
        other => panic!("unknown import must reject as mn.unsupported.import: {other:?}"),
    }
    // Driver-level: an entry that does not exist fails at the adapter with
    // codegen_emit_units absent (=0) — the old lowerer is never invoked as
    // a fallback.
    let mut d = ace_native::driver::MnDriver::ephemeral();
    let workdir = std::env::temp_dir().join(format!("mn8-t03-badentry-{}", std::process::id()));
    let r = d.request(
        "fn main() int {\n    return 1\n}\n",
        "d_missing",
        &[],
        &workdir,
        None,
    );
    assert!(!r.ok, "missing entry must fail");
    assert!(
        r.error
            .as_ref()
            .unwrap()
            .code()
            .starts_with("mn.unsupported.adapter"),
        "precise source: {:?}",
        r.error
    );
    assert_eq!(
        *r.obs.counters.get("codegen_emit_units").unwrap_or(&0),
        0,
        "a rejected request compiles nothing (no old-lowerer fallback)"
    );
}
