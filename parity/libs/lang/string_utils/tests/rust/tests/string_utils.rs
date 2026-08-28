// Rust oracle for string_utils parity.
//
// Mirrors parity/libs/string_utils/auto/string_utils.at: hand-rolled ASCII
// string ops matching the Auto implementation exactly (not using std's own
// trim/contains/replace, so byte-for-byte agreement is guaranteed on the
// same definition). Test names EXACTLY mirror tests/auto/basic.at.
//
// NOTE: these deliberately mirror the Auto source's own semantics, not Rust's
// Unicode-aware str methods — e.g. to_lower only folds A-Z, and trim only
// strips the four ASCII whitespace bytes the Auto side strips.

fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

fn to_lower(s: &str) -> String {
    s.bytes()
        .map(|b| if b >= b'A' && b <= b'Z' { b + 32 } else { b })
        .map(|b| b as char)
        .collect()
}

fn to_upper(s: &str) -> String {
    s.bytes()
        .map(|b| if b >= b'a' && b <= b'z' { b - 32 } else { b })
        .map(|b| b as char)
        .collect()
}

fn is_ws(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\n' || b == b'\r'
}

fn trim(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && is_ws(bytes[start]) {
        start += 1;
    }
    while end > start && is_ws(bytes[end - 1]) {
        end -= 1;
    }
    String::from_utf8(bytes[start..end].to_vec()).unwrap()
}

fn contains(haystack: &str, needle: &str) -> i32 {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() {
        return 1;
    }
    if n.len() > h.len() {
        return 0;
    }
    let mut i = 0;
    while i <= h.len() - n.len() {
        let mut m = 1;
        let mut j = 0;
        while j < n.len() {
            if h[i + j] != n[j] {
                m = 0;
                break;
            }
            j += 1;
        }
        if m == 1 {
            return 1;
        }
        i += 1;
    }
    0
}

fn replace(s: &str, src: &str, dest: &str) -> String {
    let flen = src.len();
    if flen == 0 {
        return s.to_string();
    }
    let bytes = s.as_bytes();
    let srcb = src.as_bytes();
    let mut out = String::new();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        if i <= n - flen && {
            let mut m = 1;
            let mut j = 0;
            while j < flen {
                if bytes[i + j] != srcb[j] {
                    m = 0;
                    break;
                }
                j += 1;
            }
            m == 1
        } {
            out.push_str(dest);
            i += flen;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

fn tap_ok(n: u32, name: &str) {
    println!("ok {} - {}", n, name);
}

#[test]
fn test_reverse_basic() {
    assert_eq!(reverse("abc"), "cba");
    tap_ok(1, "test_reverse_basic");
}
#[test]
fn test_reverse_single() {
    assert_eq!(reverse("x"), "x");
    tap_ok(2, "test_reverse_single");
}
#[test]
fn test_reverse_empty() {
    assert_eq!(reverse(""), "");
    tap_ok(3, "test_reverse_empty");
}
#[test]
fn test_reverse_palindrome() {
    assert_eq!(reverse("racecar"), "racecar");
    tap_ok(4, "test_reverse_palindrome");
}
#[test]
fn test_lower_mixed() {
    assert_eq!(to_lower("Hello WORLD"), "hello world");
    tap_ok(5, "test_lower_mixed");
}
#[test]
fn test_lower_already() {
    assert_eq!(to_lower("abc"), "abc");
    tap_ok(6, "test_lower_already");
}
#[test]
fn test_lower_with_digits() {
    assert_eq!(to_lower("A1B2"), "a1b2");
    tap_ok(7, "test_lower_with_digits");
}
#[test]
fn test_upper_mixed() {
    assert_eq!(to_upper("Hello WORLD"), "HELLO WORLD");
    tap_ok(8, "test_upper_mixed");
}
#[test]
fn test_upper_already() {
    assert_eq!(to_upper("ABC"), "ABC");
    tap_ok(9, "test_upper_already");
}
#[test]
fn test_trim_both() {
    assert_eq!(trim("  hi  "), "hi");
    tap_ok(10, "test_trim_both");
}
#[test]
fn test_trim_leading() {
    assert_eq!(trim("\n\nhi"), "hi");
    tap_ok(11, "test_trim_leading");
}
#[test]
fn test_trim_none() {
    assert_eq!(trim("abc"), "abc");
    tap_ok(12, "test_trim_none");
}
#[test]
fn test_trim_all_ws() {
    assert_eq!(trim("   "), "");
    tap_ok(13, "test_trim_all_ws");
}
#[test]
fn test_contains_found() {
    assert_eq!(contains("hello world", "world"), 1);
    tap_ok(14, "test_contains_found");
}
#[test]
fn test_contains_missing() {
    assert_eq!(contains("hello", "xyz"), 0);
    tap_ok(15, "test_contains_missing");
}
#[test]
fn test_contains_empty_needle() {
    assert_eq!(contains("abc", ""), 1);
    tap_ok(16, "test_contains_empty_needle");
}
#[test]
fn test_contains_at_start() {
    assert_eq!(contains("abc", "ab"), 1);
    tap_ok(17, "test_contains_at_start");
}
#[test]
fn test_contains_too_long() {
    assert_eq!(contains("ab", "abc"), 0);
    tap_ok(18, "test_contains_too_long");
}
#[test]
fn test_replace_all() {
    assert_eq!(replace("a-b-c", "-", "+"), "a+b+c");
    tap_ok(19, "test_replace_all");
}
#[test]
fn test_replace_none() {
    assert_eq!(replace("abc", "x", "y"), "abc");
    tap_ok(20, "test_replace_none");
}
#[test]
fn test_replace_multi_char() {
    assert_eq!(replace("foo bar foo", "foo", "XX"), "XX bar XX");
    tap_ok(21, "test_replace_multi_char");
}
#[test]
fn test_replace_empty_src() {
    assert_eq!(replace("abc", "", "Z"), "abc");
    tap_ok(22, "test_replace_empty_src");
}
