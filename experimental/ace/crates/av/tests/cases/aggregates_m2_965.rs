//! PLAN-965 (M2-A): Control flow, error propagation (`try/catch/finally`,
//! `fail`, `rethrow`, `.?`, `.?(default)`, `??`, `is` expr/stmt) and
//! generators (`~Iter<T>`, `yield`, `.next()`, `.close()`, `for n in gen()`).
//! Covers independent oracles, V-01..V-04, V-08, AC-06, and negative controls.

use ace_av::console_host::ConsoleCapture;
use ace_av_core::abc::encode_abc;
use ace_av_core::av::{execute, execute_console, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::verify_mir;
use ace_core::source::compile_aggregates_source;

fn run_ag_outcome(
    src: &str,
    entry_name: &str,
    caps: &[String],
    limits: &ExecLimits,
    cancel: &CancelFlag,
) -> Result<(ExecOutcome, usize, usize, usize), String> {
    let compiled = compile_aggregates_source(src, "probe.at", caps).map_err(|ds| {
        ds.iter()
            .map(|d| d.render("probe.at", src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let mir = ace_lowering::lower_checked_closures(&compiled.checked, entry_name, 0)
        .map_err(|e| format!("lower: {e:?}"))?;
    let _checked = verify_mir(&mir).map_err(|e| format!("verify_mir: {e:?}"))?;
    let abc = encode_abc(&mir).map_err(|e| format!("encode: {e:?}"))?;
    let flat = abc.to_flat();
    let loaded =
        ace_av_core::abc_verify::verify_abc_bytes(&flat, &mir.owner).map_err(|e| e.to_string())?;
    let (out, trace) = execute(&loaded, limits, cancel);
    let live = trace.arena.as_ref().map(|a| a.live()).unwrap_or(0);
    let allocs = trace
        .arena
        .as_ref()
        .map(|a| a.alloc_events().len())
        .unwrap_or(0);
    let frees = trace
        .arena
        .as_ref()
        .map(|a| a.free_events().len())
        .unwrap_or(0);
    Ok((out, live, allocs, frees))
}

fn expect_i32(src: &str, entry_name: &str) -> i32 {
    let (out, live, allocs, frees) = run_ag_outcome(
        src,
        entry_name,
        &[],
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .expect("compile+lower+verify+execute should succeed");
    assert_eq!(live, 0, "StrArena must have 0 live buffers at exit");
    assert_eq!(allocs, frees, "StrArena allocs must equal frees");
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected Success(I32), got {other:?}"),
    }
}

#[test]
fn m2_oracle_sum_script_generator() {
    let src = "\
fn gen() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
var s = 0
for n in gen() {
    s = s + n
}
s";
    assert_eq!(expect_i32(src, "d_main"), 6);
}

#[test]
fn m2_oracle_coalesce_some_and_none() {
    let coalesce_some = "\
let x ?int = Some(42)
x ?? 99";
    assert_eq!(expect_i32(coalesce_some, "d_main"), 42);

    let coalesce_none = "\
let x ?int = None
x ?? 99";
    assert_eq!(expect_i32(coalesce_none, "d_main"), 99);
}

#[test]
fn m2_oracle_propagate_err_script() {
    let fail_script = "fail(\"error\")";
    let (out, live, allocs, frees) = run_ag_outcome(
        fail_script,
        "d_main",
        &[],
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .expect("fail script compiles");
    assert_eq!(live, 0);
    assert_eq!(allocs, frees);
    assert!(
        matches!(&out, ExecOutcome::Error { message, .. } if message == "error"),
        "expected ExecOutcome::Error(\"error\"), got {out:?}"
    );

    let err_prop_script = "\
fn bad() !int {
    return Err(\"error\")
}
bad().?";
    let (out2, live2, allocs2, frees2) = run_ag_outcome(
        err_prop_script,
        "d_main",
        &[],
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .expect("propagate err script compiles");
    assert_eq!(live2, 0);
    assert_eq!(allocs2, frees2);
    assert!(
        matches!(&out2, ExecOutcome::Error { message, .. } if message == "error"),
        "expected ExecOutcome::Error(\"error\"), got {out2:?}"
    );
}

#[test]
fn m2_oracle_result_and_question_propagate_with_may() {
    let src = "\
fn parse(ok bool) !int {
    if ok { return Ok(20) }
    Err(\"bad\")
}
fn add_two(ok bool) !int {
    let v = parse(ok).?
    Ok(v + 2)
}
fn pick(ok bool) ?int {
    var m ?int = May.nil
    if ok {
        m = May.val(19)
    }
    let x = m.?
    May.val(x + 1)
}
type Log { items List<int> }
fn log_inner(mut log Log) ?int {
    let n ?int = None
    try {
        let v = n.?
        log.items.push(v)
    } catch e {
        log.items.push(99)
    }
    Some(1)
}
fn none_bypasses_catch() int {
    var flag = 0
    var log = Log(List<int>.new())
    let r = log_inner(log.mut)
    if log.items.len() == 0 { flag = flag + 1 }
    let x = pick(true) ?? 0
    let y = pick(false) ?? 10
    let z = add_two(true).?(0)
    let w = add_two(false).?(100)
    // 1 + 20 + 10 + 22 + 100 = 153 -> make sure each sub-oracle contributes distinctly
    flag * 10000 + x * 100 + y + z * 1000 + w
}
fn main() int {
    none_bypasses_catch()
}";
    // Wait: let's check exact arithmetic:
    // flag = 1 -> 10000
    // z = 22 -> 22000
    // x = 20 -> 2000
    // y = 10 -> 10
    // w = 100 -> 100
    // total = 34110
    assert_eq!(expect_i32(src, "d_main"), 34110);
}

#[test]
fn m2_v01_nested_try_finally_runs_once_per_frame_and_loop_ctrl() {
    let src = "\
type LogBox { items List<int> }
fn inner(mut log LogBox, stop bool) int {
    try {
        try {
            log.items.push(1)
            if stop { return 77 }
            log.items.push(2)
        } finally {
            log.items.push(10)
        }
    } finally {
        log.items.push(20)
    }
    return 99
}
fn loop_test(mut log LogBox) int {
    var total = 0
    var i = 0
    for i < 4 {
        i = i + 1
        try {
            if i == 2 { continue }
            if i == 4 { break }
            total = total + i
        } finally {
            log.items.push(100 + i)
        }
    }
    return total
}
fn main() int {
    var log = LogBox(List<int>.new())
    let r1 = inner(log.mut, true)
    // log should be [1, 10, 20] -> encode to verify order & exact count
    let a = log.items.get(0) * 10000 + log.items.get(1) * 100 + log.items.get(2)
    let l_len1 = log.items.len()
    let lt = loop_test(log.mut)
    // loop_test iterations 1, 2(continue), 3, 4(break): total = 1+3 = 4
    // log gets 101, 102, 103, 104 => len moves from 3 to 7
    let l_len2 = log.items.len()
    let b = log.items.get(3) + log.items.get(4) + log.items.get(5) + log.items.get(6) // 410
    if r1 != 77 || l_len1 != 3 || l_len2 != 7 { return -1 }
    return a * 1000 + lt * 100 + (b - 400)
}";
    // a = 11020; lt = 4; b - 400 = 10 => 11020 * 1000 + 400 + 10 = 11020410
    assert_eq!(expect_i32(src, "d_main"), 11020410);
}

#[test]
fn m2_v02_lazy_short_circuit_coalesce_and_unwrap_or() {
    let src = "\
type Hits { items List<int> }
fn boom(mut hits Hits) int {
    hits.items.push(1)
    fail(\"must not run\")
}
fn ok_val() !int { return Ok(21) }
fn err_val() !int { return Err(\"nope\") }
fn main() int {
    var hits = Hits(List<int>.new())
    let opt ?int = Some(19)
    let a = opt ?? boom(hits.mut)
    let b = ok_val().?(boom(hits.mut))
    // Negative control leg: if None/Err, RHS DOES run
    let c ?int = None
    let fallback_none = c ?? 2
    let fallback_err = err_val().?(0)
    return a + b + fallback_none + fallback_err + hits.items.len() * 1000
}";
    // 19 + 21 + 2 + 0 + 0 = 42
    assert_eq!(expect_i32(src, "d_main"), 42);
}

#[test]
fn m2_v03_generator_lazy_suspend_finally_and_idempotent_close() {
    let src = "\
// PLAN-965 V-03: a `List` handle is an owned parameter, so the suspended
// generator keeps it across `yield` and the close `finally` is observable
// through the caller's handle (borrowed record/`mut` params are rejected by
// `source.generator-borrow-param`).
fn seq(events List<int>, limit int) ~Iter<int> {
    events.push(1)
    try {
        var i = 0
        for i < limit {
            yield i * 10
            i = i + 1
        }
    } finally {
        events.push(99)
    }
}
fn main() int {
    var ev = List<int>.new()
    var g = seq(ev, 5)
    // Lazy start: body must NOT run before first next()
    let before = ev.len()
    let n0 = g.next() ?? -1
    let after_first = ev.len()
    let n1 = g.next() ?? -1
    // Idempotent explicit close runs finally ONCE
    g.close()
    g.close()
    let after_close = g.next() ?? -1
    let close_ev = ev.get(ev.len() - 1)
    let close_len = ev.len()
    // for-in early break must also close generator & trigger finally
    var ev2 = List<int>.new()
    var sum = 0
    for x in seq(ev2, 10) {
        if x == 20 { break }
        sum = sum + x
    }
    // sum = 0 + 10 = 10, ev2 = [1, 99]
    if before != 0 || after_first != 1 || close_len != 2 || close_ev != 99 { return -1 }
    if ev2.len() != 2 || ev2.get(1) != 99 || after_close != -1 { return -2 }
    return n0 + n1 + sum + after_close + 1
}";
    // 0 + 10 + 10 + (-1) + 1 = 20
    assert_eq!(expect_i32(src, "d_main"), 20);
}

#[test]
fn m2_try_catch_rethrow_fail_shadow_and_is_expr() {
    let src = "\
fn fail(n int) int { return n + 1 }
fn rethrower() int {
    try {
        try {
            let r !int = Err(\"root\")
            let _ = r.?
        } catch (e) {
            rethrow(\"wrap:\" + e)
        }
    } catch e2 {
        if e2 == \"wrap:root\" { return 40 }
        return 0
    }
    return 0
}
fn is_variants(v int) int {
    let label = is v {
        1 -> 10
        2 => { 20 }
        else -> 30
    }
    let b = is (v == 2) {
        true -> 100
        false -> 200
    }
    return label + b
}
fn main() int {
    // 40 + 120 + 1 = 161
    return rethrower() + is_variants(2) + fail(0)
}";
    assert_eq!(expect_i32(src, "d_main"), 161);
}

#[test]
fn m2_v04_ac06_v08_limits_trap_cancel_host_not_caught() {
    // 1. Infinite generator hitting ExecLimits::max_steps -> ExecOutcome::Resource (never Error, never caught)
    let inf_gen = "\
fn inf() ~Iter<int> {
    var i = 0
    for true {
        yield i
        i = i + 1
    }
}
fn main() int {
    try {
        var s = 0
        for x in inf() {
            s = s + x
        }
        return s
    } catch e {
        return 999
    }
}";
    let limits = ExecLimits {
        max_steps: 250,
        ..ExecLimits::default()
    };
    let (out_lim, live_lim, alloc_lim, free_lim) =
        run_ag_outcome(inf_gen, "d_main", &[], &limits, &CancelFlag::new(false)).unwrap();
    assert!(
        matches!(out_lim, ExecOutcome::Resource { .. }),
        "expected Resource exhaustion, got {out_lim:?}"
    );
    assert_eq!(live_lim, 0);
    assert_eq!(alloc_lim, free_lim);

    // 2. Trap (DivByZero) is never caught by `catch`
    let div_zero = "\
fn main() int {
    try {
        var z = 0
        return 10 / z
    } catch e {
        return 999
    }
}";
    let (out_trap, _, _, _) = run_ag_outcome(
        div_zero,
        "d_main",
        &[],
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .unwrap();
    assert!(
        matches!(out_trap, ExecOutcome::Trap { .. }),
        "expected Trap, got {out_trap:?}"
    );

    // 3. CancelFlag is never caught by `catch`
    let cancel_src = "\
fn main() int {
    try {
        var i = 0
        for i < 2000 {
            i = i + 1
        }
        return i
    } catch e {
        return 999
    }
}";
    let (out_cancel, _, _, _) = run_ag_outcome(
        cancel_src,
        "d_main",
        &[],
        &ExecLimits::default(),
        &CancelFlag::new(true),
    )
    .unwrap();
    assert!(
        matches!(out_cancel, ExecOutcome::Cancelled),
        "expected Cancelled, got {out_cancel:?}"
    );

    // 4. host.unauthorized inherits HostConsole policy & dispatches == 0 (never enters catch)
    let host_src = "\
fn main() int {
    print(1)
    return 1
}";
    let compiled = ace_core::source::compile_host_console_source(host_src, "probe.at", &[])
        .expect("host console script compiles");
    let mir = ace_lowering::lower_checked_owned(&compiled.checked, "d_main", 0).unwrap();
    let _ = verify_mir(&mir).unwrap();
    let abc = encode_abc(&mir).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_bytes(&abc.to_flat(), &mir.owner).unwrap();
    let mut cap = ConsoleCapture::default();
    let err = execute_console(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &[],
        &mut cap,
    )
    .unwrap_err();
    assert_eq!(err.code, "host.unauthorized");
    assert_eq!(cap.dispatches, 0);
}

#[test]
fn m2_negative_compile_controls() {
    for (name, src, want_code) in [
        (
            "propagate-incompatible",
            "fn main() int {\n    let x !int = Ok(1)\n    return x.?\n}",
            "source.propagate-incompatible",
        ),
        (
            "yield-outside-generator",
            "fn main() int {\n    yield 1\n    return 0\n}",
            "source.yield-outside-generator",
        ),
        (
            "rethrow-outside-catch",
            "fn main() int {\n    rethrow()\n    return 0\n}",
            "source.rethrow-outside-catch",
        ),
        (
            "non-exhaustive-is",
            "fn main() int {\n    return is 1 { 1 -> 10 }\n}",
            "source.pattern-nonexhaustive",
        ),
        // PLAN-965 V-03: borrowed generator parameters cannot cross `yield`
        // (the suspended state record stores values, not borrows).
        (
            "generator-mut-borrow",
            "type Flag { n int }\nfn g(mut f Flag) ~Iter<int> {\n    yield 1\n}\nfn main() int {\n    return 0\n}",
            "source.generator-borrow-param",
        ),
        (
            "generator-view-param",
            "type Flag { n int }\nfn g(f Flag) ~Iter<int> {\n    yield 1\n}\nfn main() int {\n    return 0\n}",
            "source.generator-borrow-param",
        ),
    ] {
        let errs = compile_aggregates_source(src, "neg.at", &[])
            .err()
            .unwrap_or_else(|| panic!("{name} must fail to compile"));
        assert!(
            errs.iter().any(|d| d.code == want_code),
            "{name}: expected {want_code}, got {:?}",
            errs.iter().map(|d| d.code.as_str()).collect::<Vec<_>>()
        );
    }
}

#[test]
fn m2_v08_close_fail_during_drain_propagates_as_language_error() {
    // V-08 (cleanup-error exit): the `finally` of a suspended generator runs
    // when the iterator drains, and a language `fail` raised there surfaces as
    // `ExecOutcome::Error` (never a trap, a resource, or a silent success)
    // with no live arena buffer left behind.
    let src = "\
fn guarded() ~Iter<int> {
    try {
        yield 10
    } finally {
        fail(\"closed965\")
    }
}
fn main() int {
    var part = 0
    for x in guarded() {
        part = part + x
    }
    return part
}";
    let (out, live, allocs, frees) = run_ag_outcome(
        src,
        "d_main",
        &[],
        &ExecLimits::default(),
        &CancelFlag::new(false),
    )
    .expect("close-fail script compiles");
    assert_eq!(live, 0, "close cleanup must release the suspended frame");
    assert_eq!(allocs, frees);
    assert!(
        matches!(&out, ExecOutcome::Error { message, .. } if message == "closed965"),
        "expected ExecOutcome::Error(closed965), got {out:?}"
    );
}

#[test]
fn m2_oracle_indirect_generator_return_and_coexisting_instances() {
    // D4: `~Iter<T>` is a first-class value (legacy `22_generator/004_indirect_return`):
    // another function may hand back a generator it obtained from one, and two
    // generators of the same element type coexist in one program.
    let src = "\
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
fn get_counter() ~Iter<int> {
    return counter()
}
fn main() int {
    var a = 0
    for n in get_counter() {
        a = a + n
    }
    var b = 0
    for n in counter() {
        b = b + n
    }
    return a * 10 + b
}";
    assert_eq!(expect_i32(src, "d_main"), 66);
}
