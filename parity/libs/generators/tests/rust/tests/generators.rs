//! Native Rust oracle for the generators parity lib (Plan 359 D2 / 417-D2).
//!
//! Mirrors parity/libs/generators/tests/auto/basic.at test-for-test:
//! same TAP test names, same expected values. The generator bodies use
//! std::iter::from_fn with captured state — the canonical eager-to-lazy
//! bridge — while the Auto side uses yield.

fn tap_ok(n: i64, name: &str) {
    println!("ok {} - {}", n, name);
}

fn tap_not_ok(n: i64, name: &str, diag: &str) {
    println!("not ok {} - {} # {}", n, name, diag);
}

fn check_int(n: i64, name: &str, actual: i64, expected: i64) {
    if actual == expected {
        tap_ok(n, name);
    } else {
        tap_not_ok(n, name, &format!("got {} expected {}", actual, expected));
    }
}

/// Oracle for `three_yields`: a fixed three-item lazy sequence.
fn three_yields() -> impl Iterator<Item = i64> {
    let mut items = vec![10i64, 20, 30].into_iter();
    std::iter::from_fn(move || items.next())
}

/// Oracle for `counter(start, count)`: yields start, start+1, ... (count items).
fn counter(start: i64, count: i64) -> impl Iterator<Item = i64> {
    let mut i = 0i64;
    std::iter::from_fn(move || {
        if i < count {
            let v = start + i;
            i += 1;
            Some(v)
        } else {
            None
        }
    })
}

/// Oracle for `evens_up_to(limit)`: yields 0, 2, 4, ... < limit.
fn evens_up_to(limit: i64) -> impl Iterator<Item = i64> {
    (0..limit).step_by(2)
}

// Each #[test] mirrors ONE Auto TAP test (name-exact) so the comparator
// keys results by name.

#[test]
fn sum_three_yields() {
    check_int(1, "sum_three_yields", three_yields().sum(), 60);
}

#[test]
fn sum_counter_10_3() {
    check_int(2, "sum_counter_10_3", counter(10, 3).sum(), 33);
}

#[test]
fn empty_counter_zero_items() {
    check_int(3, "empty_counter_zero_items", counter(5, 0).count() as i64, 0);
}

#[test]
fn sum_evens_up_to_9() {
    check_int(4, "sum_evens_up_to_9", evens_up_to(9).sum(), 20);
}

#[test]
fn counter_first_value() {
    let first = counter(100, 5).next().unwrap_or(-1);
    check_int(5, "counter_first_value", first, 100);
}

#[test]
fn counter_item_count() {
    check_int(6, "counter_item_count", counter(100, 5).count() as i64, 5);
}
