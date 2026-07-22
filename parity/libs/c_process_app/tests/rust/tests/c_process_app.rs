// Rust oracle for c_process_app consumer parity.
//
// Mirrors parity/libs/c_process_app/auto/c_process_app.at: splits an args
// string on single spaces and drops empty segments (the same logic the Auto
// side implements with str.split(" ") + len>0 filter). Test names EXACTLY
// mirror tests/auto/basic.at so the parity framework can compare three-way.
//
// Design doc §F4: real argv differs per process, so we test the *parsing
// logic* over a FIXED args string. This is the core of any "read argv then
// parse/dispatch" CLI app. Pure computation — race-free under parallel tests.

/// Count args: split on ' ', drop empty segments. Mirrors Auto parse_count.
fn parse_count(args_str: &str) -> i32 {
    args_str.split(' ').filter(|p| !p.is_empty()).count() as i32
}

/// Nth non-empty arg (0-based), "<none>" if out of range. Mirrors Auto parse_nth
/// (Auto uses "<none>" rather than "" to avoid a VM `print("")` quirk that
/// swallows the preceding output line — see c_process_app.at notes).
fn parse_nth(args_str: &str, n: i32) -> String {
    let mut idx = 0i32;
    for p in args_str.split(' ').filter(|p| !p.is_empty()) {
        if idx == n {
            return p.to_string();
        }
        idx += 1;
    }
    "<none>".to_string()
}

fn tap_ok(n: u32, name: &str) {
    println!("ok {} - {}", n, name);
}

#[test]
fn test_count_basic() {
    assert_eq!(parse_count("cmd arg1 arg2 arg3"), 4);
    tap_ok(1, "test_count_basic");
}

#[test]
fn test_count_single() {
    assert_eq!(parse_count("cmd"), 1);
    tap_ok(2, "test_count_single");
}

#[test]
fn test_count_two() {
    assert_eq!(parse_count("cmd arg1"), 2);
    tap_ok(3, "test_count_two");
}

#[test]
fn test_count_empty() {
    assert_eq!(parse_count(""), 0);
    tap_ok(4, "test_count_empty");
}

#[test]
fn test_count_extra_spaces() {
    // "cmd  arg1   arg2" -> split on ' ', drop empties -> 3 args.
    assert_eq!(parse_count("cmd  arg1   arg2"), 3);
    tap_ok(5, "test_count_extra_spaces");
}

#[test]
fn test_nth_first() {
    assert_eq!(parse_nth("cmd arg1 arg2", 0), "cmd");
    tap_ok(6, "test_nth_first");
}

#[test]
fn test_nth_middle() {
    assert_eq!(parse_nth("cmd arg1 arg2", 1), "arg1");
    tap_ok(7, "test_nth_middle");
}

#[test]
fn test_nth_last() {
    assert_eq!(parse_nth("cmd arg1 arg2", 2), "arg2");
    tap_ok(8, "test_nth_last");
}

#[test]
fn test_nth_out_of_range() {
    assert_eq!(parse_nth("cmd arg1", 5), "<none>");
    tap_ok(9, "test_nth_out_of_range");
}
