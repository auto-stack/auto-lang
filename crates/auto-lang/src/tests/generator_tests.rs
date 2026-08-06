//! Plan 326: generator + for-loop regression tests.
//!
//! Verifies that `for n in gen()` consumes each yielded value exactly once,
//! with no duplicates and no skips. Covers the root-cause scenario from
//! Plan 326 §1 item 1 (generator task sp management / eager collection).

use crate::run_with_capture;

/// Baseline: `fn counter() ~Iter<int> { yield 1; yield 2; yield 3 }`
/// summed in a for-loop should total 6.
#[test]
fn generator_for_loop_sum() {
    let code = r#"
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}
fn main() {
    var sum = 0
    for n in counter() {
        sum = sum + n
    }
    print(sum)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("Error: {}", e), String::new()));
    assert_eq!(stdout.trim(), "6", "counter sum: got stdout={:?} result={:?}", stdout, result);
}

/// Each yielded value must appear exactly once when collected into a list.
#[test]
fn generator_values_no_duplicates() {
    let code = r#"
fn three() ~Iter<int> {
    yield 10
    yield 20
    yield 30
}
fn main() {
    var seen = ""
    for n in three() {
        seen = seen + n.to_str() + ","
    }
    print(seen)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("Error: {}", e), String::new()));
    // Must be "10,20,30," — no duplicate 10, no skip.
    assert_eq!(stdout.trim(), "10,20,30,", "no duplicates: got stdout={:?} result={:?}", stdout, result);
}

/// String yields (not just int).
#[test]
fn generator_string_yields() {
    let code = r#"
fn words() ~Iter<str> {
    yield "a"
    yield "b"
    yield "c"
}
fn main() {
    var acc = ""
    for w in words() {
        acc = acc + w
    }
    print(acc)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("Error: {}", e), String::new()));
    assert_eq!(stdout.trim(), "abc", "string yields: got stdout={:?} result={:?}", stdout, result);
}

/// Plan 317 §11 Phase 7 (P3): infinite generator inside `for {}` must not hang.
///
/// Root cause (pre-fix): `Expr::Yield` emits `compile_expr(inner)` (push) +
/// `YIELD_VAL` (pop) = net stack-neutral, but `Stmt::Expr` then emitted a
/// trailing POP (because `should_pop_expr_result` is true in a `for {}` body).
/// That POP ate a value from below the yield — a per-iteration underflow that
/// accumulated across `next()` resumptions in the lazy generator and hung the
/// VM. Finite generators hid this (their RET clears the stack once). Fix:
/// `Expr::Yield` sets `last_was_self_balanced`, and `Stmt::Expr` skips the POP
/// when that flag is set.
#[test]
fn generator_infinite_break() {
    let code = r#"
fn counter() ~Iter<int> {
    var i = 0
    for {
        yield i
        i = i + 1
    }
}
fn main() {
    var sum = 0
    for n in counter() {
        sum = sum + n
        if sum >= 6 {
            break
        }
    }
    print(sum)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("Error: {}", e), String::new()));
    // i yields 0,1,2,3,...; sum accumulates 0,1,3,6; breaks when sum>=6 → "6".
    // Pre-fix: this hung forever (stack underflow across lazy resumptions).
    assert_eq!(stdout.trim(), "6", "infinite generator break: got stdout={:?} result={:?}", stdout, result);
}

/// Plan 317 §11 Phase 7 (P3) regression: infinite generator consumed with a
/// fixed take-count via break, verifying multiple yields resolve correctly and
/// the generator terminates cleanly (no hang, no crash) once the consumer stops.
#[test]
fn generator_infinite_take_three() {
    let code = r#"
fn ones() ~Iter<int> {
    for {
        yield 1
    }
}
fn main() {
    var seen = 0
    var total = 0
    for v in ones() {
        total = total + v
        seen = seen + 1
        if seen >= 3 {
            break
        }
    }
    print(total)
}
"#;
    let (result, stdout) = run_with_capture(code)
        .unwrap_or_else(|e| (format!("Error: {}", e), String::new()));
    // Three 1s → total 3.
    assert_eq!(stdout.trim(), "3", "infinite generator take-3: got stdout={:?} result={:?}", stdout, result);
}
