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
