//! PLAN-965 (T-03, native leg): real `MnDriver::request_checked` PE builds &
//! executions for M2-A control flow, error propagation, `??`, `.?`, `.?(d)`,
//! `is` expression/statement, `try/catch/finally` with nested `break`/`continue`/
//! early `return`, `~Iter<T>` lazy generators + early `break` `finally` cleanup,
//! and uncaught `fail`/`.?` mapped to non-zero `Outcome::Trap`.

#[path = "../common/mod.rs"]
mod common;

use ace_native::{
    collections_observe::{decode, Outcome},
    driver::MnDriver,
};

fn run_ag_native(src: &str, tag: &str) -> Outcome {
    let checked = ace_core::source::compile_aggregates_source(src, "p965.at", &[])
        .unwrap_or_else(|ds| panic!("{tag}: source compile failed: {ds:?}"))
        .checked;
    let dir = std::env::temp_dir().join(format!("mn965-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let snapshot = ace_av_core::contract::fnv1a64(src.as_bytes());
    let mut driver = MnDriver::ephemeral();
    let res = driver.request_checked(&checked, "d_main", &dir, None, snapshot);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(res.ok, "{tag}: mn driver failed: {:?}", res.error);
    decode(res.exit_code, &res.stderr)
        .unwrap_or_else(|e| panic!("{tag}: decode observation failed: {e:?}"))
        .outcome
}

#[test]
#[cfg(windows)]
fn mn_965_try_catch_finally_nested_break_continue_return() {
    let src = "\
type Trace { sum int; log List<int> }
fn inner_loop(mut tr Trace) int {
    var i = 0
    for i < 5 {
        i = i + 1
        try {
            if i == 2 {
                continue
            }
            if i == 4 {
                break
            }
            tr.sum = tr.sum + i
        } finally {
            tr.log.push(i)
        }
    }
    tr.sum
}
fn early_ret(mut tr Trace) int {
    try {
        try {
            return 10
        } finally {
            tr.log.push(1)
        }
    } finally {
        tr.log.push(2)
    }
    return 0
}
fn main() int {
    var tr = Trace(0, List<int>.new())
    let s = inner_loop(tr.mut)
    let r = early_ret(tr.mut)
    // inner_loop: i=1 (sum=1, log=[1]), i=2 continue (log=[1,2]), i=3 (sum=4, log=[1,2,3]), i=4 break (log=[1,2,3,4])
    // early_ret: inner finally pushes 1, outer finally pushes 2 -> len = 6
    if s != 4 { return 101 }
    if r != 10 { return 102 }
    if tr.log.len() != 6 { return 103 }
    if tr.log.get(0) != 1 { return 104 }
    if tr.log.get(1) != 2 { return 105 }
    if tr.log.get(2) != 3 { return 106 }
    if tr.log.get(3) != 4 { return 107 }
    if tr.log.get(4) != 1 { return 108 }
    if tr.log.get(5) != 2 { return 109 }
    return 42
}";
    assert_eq!(run_ag_native(src, "v01-try-finally"), Outcome::Return(42));
}

#[test]
#[cfg(windows)]
fn mn_965_coalesce_propagate_and_is() {
    let src = "\
type BoxCounter { ticks int }
fn bump(mut c BoxCounter) int {
    c.ticks = c.ticks + 1
    99
}
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
fn main() int {
    var ctr = BoxCounter(0)
    let a = Some(10) ?? bump(ctr.mut)
    let b = None ?? 5
    let c = add_two(true).?(0)
    let d = add_two(false).?(7)
    let e = pick(true) ?? 0
    let f = pick(false) ?? 1
    let g = is Some(4) {
        Some(v) -> v * 2
        None -> 0
    }
    if ctr.ticks != 0 { return 101 }
    // a=10, b=5, c=22, d=7, e=20, f=1, g=8 -> sum = 73
    return a + b + c + d + e + f + g
}";
    assert_eq!(run_ag_native(src, "v02-coalesce-is"), Outcome::Return(73));
}

#[test]
#[cfg(windows)]
fn mn_965_generator_sum_and_close_cleanup() {
    // Native's source ABI carries scalars and owned records only (a container
    // parameter is `mn.unsupported`), so this leg observes generator VALUES —
    // drain and early `break` must not corrupt them. The close EFFECT of a
    // `finally` is observed through the language-error channel by
    // `mn_965_generator_close_fail_on_break` / `..._on_drain` below; the
    // AV-only `List`-handle observation lives in `ace-av`'s V-03.
    let src = "\
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
fn guarded() ~Iter<int> {
    try {
        yield 10
        yield 20
        yield 30
    } finally {
        let closed965 = 1
    }
}
fn main() int {
    var sum = 0
    for n in counter() {
        sum = sum + n
    }
    var part = 0
    for x in guarded() {
        part = part + x
        if x == 20 {
            break
        }
    }
    if sum != 6 { return 101 }
    if part != 30 { return 102 }
    return sum + part + sum * 6
}";
    // 6 + 30 + 36 = 72
    assert_eq!(run_ag_native(src, "v03-gen-close"), Outcome::Return(72));
}

#[test]
#[cfg(windows)]
fn mn_965_generator_close_fail_on_break() {
    // F-965-R1-03 (native leg): the early-`break` close of a suspended
    // generator runs its `finally`; a language `fail` there must surface as the
    // language-error trap instead of vanishing.
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
        if x == 10 { break }
    }
    return part
}";
    assert_eq!(run_ag_native(src, "v03-gen-close-break"), Outcome::Trap(9));
}

#[test]
#[cfg(windows)]
fn mn_965_call_statement_is_evaluated() {
    // F-965-R1-01 (native leg): a non-tail call statement is a call for
    // effect; the old adapter dropped it, so `bump` never ran.
    let src = "\
type Ctr { hits int }
fn bump(mut c Ctr) int {
    c.hits = c.hits + 1
    7
}
fn main() int {
    var c = Ctr(0)
    bump(c.mut)
    if c.hits != 1 { return 11 }
    return 42
}";
    assert_eq!(run_ag_native(src, "f01-stmt-call"), Outcome::Return(42));
}

#[test]
#[cfg(windows)]
fn mn_965_generator_close_fail_on_drain() {
    // V-08 (cleanup-error exit): the `finally` of a suspended generator runs
    // when the iterator drains; a language `fail` there is an uncaught
    // language error and reaches the native observation channel as the
    // language-error trap code rather than a normal return.
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
    assert_eq!(run_ag_native(src, "v08-close-fail"), Outcome::Trap(9));
}

#[test]
#[cfg(windows)]
fn mn_965_uncaught_fail_maps_to_non_zero_trap() {
    let src = "\
fn main() int {
    fail(\"boom\")
}";
    assert_eq!(run_ag_native(src, "v04-lang-fail"), Outcome::Trap(9));
}

#[test]
#[cfg(windows)]
fn mn_965_indirect_generator_return_and_coexisting_instances() {
    // D4: `~Iter<T>` is a first-class value (legacy `004_indirect_return`); a
    // yield-less passthrough function hands back another generator and two
    // generators of the same element type coexist.
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
    assert_eq!(run_ag_native(src, "d4-indirect-gen"), Outcome::Return(66));
}
