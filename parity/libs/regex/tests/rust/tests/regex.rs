//! Native Rust oracle tests for the regex replication.
//!
//! These assert the same input -> output mapping that the Auto implementation
//! must match. Test names EXACTLY mirror the TAP test names in
//! `tests/auto/match.at` so the parity framework can compare them three-way
//! (AutoVM vs a2r vs native Rust).
//!
//! The `regex` crate (v1.10) is the oracle. To stay consistent with the
//! simplified Auto matcher:
//!   * `is_match` returns `bool`. (The Auto implementation used to return
//!     `int` (1/0) because a `bool` crossing the Auto module boundary was
//!     corrupted by the VM; fixed by Plan 359 / B4, so both backends now
//!     yield a plain `bool`.)
//!   * `find` returns the matched text as a `String`, or `""` when there is no
//!     match — matching the Auto `find`. An empty (zero-width) match also
//!     yields `""`.
//!
//! The test cases use unambiguous patterns so the regex crate's NFA
//! leftmost-longest semantics coincide with the Auto matcher's greedy
//! backtracking for this subset (`.`, `*`, `+`, `?`, `[...]`, `[a-z]`).

use regex::Regex;

/// `true` if `pattern` matches anywhere in `input`, else `false`.
fn is_match(pattern: &str, input: &str) -> bool {
    let re = Regex::new(pattern).expect("valid pattern");
    re.is_match(input)
}

/// The leftmost match of `pattern` in `input` as a string, or "" if none.
/// A zero-width match (e.g. `a*` at a non-a position) also returns "".
fn find(pattern: &str, input: &str) -> String {
    let re = Regex::new(pattern).expect("valid pattern");
    match re.find(input) {
        Some(m) => m.as_str().to_string(),
        None => String::new(),
    }
}

// ============================================================================
// literal characters (is_match) — mirror tests/auto/match.at
// ============================================================================

#[test]
fn test_literal_match_true() {
    assert_eq!(is_match("abc", "abc"), true);
}

#[test]
fn test_literal_no_match_false() {
    assert_eq!(is_match("abc", "xyz"), false);
}

#[test]
fn test_literal_substring_true() {
    assert_eq!(is_match("bc", "abcde"), true);
}

#[test]
fn test_literal_full_substring_true() {
    assert_eq!(is_match("cde", "abcde"), true);
}

#[test]
fn test_literal_at_end_true() {
    assert_eq!(is_match("de", "abcde"), true);
}

#[test]
fn test_literal_single_char_true() {
    assert_eq!(is_match("a", "a"), true);
}

// ============================================================================
// dot wildcard (is_match)
// ============================================================================

#[test]
fn test_dot_any_char_true() {
    assert_eq!(is_match("a.c", "abc"), true);
}

#[test]
fn test_dot_any_char_axc_true() {
    assert_eq!(is_match("a.c", "axc"), true);
}

#[test]
fn test_dot_no_match_wrong_len_false() {
    assert_eq!(is_match("a.c", "ac"), false);
}

#[test]
fn test_dot_matches_anywhere_true() {
    assert_eq!(is_match("x.y", "zxayz"), true);
}

// ============================================================================
// star (zero or more, is_match)
// ============================================================================

#[test]
fn test_star_zero_true() {
    assert_eq!(is_match("ab*c", "ac"), true);
}

#[test]
fn test_star_one_true() {
    assert_eq!(is_match("ab*c", "abc"), true);
}

#[test]
fn test_star_many_true() {
    assert_eq!(is_match("ab*c", "abbbbc"), true);
}

#[test]
fn test_star_no_match_false() {
    assert_eq!(is_match("ab*c", "axc"), false);
}

#[test]
fn test_star_only_true() {
    assert_eq!(is_match("a*", "bbba"), true);
}

#[test]
fn test_star_empty_match_true() {
    assert_eq!(is_match("a*", "bbb"), true);
}

// ============================================================================
// plus (one or more, is_match)
// ============================================================================

#[test]
fn test_plus_one_true() {
    assert_eq!(is_match("ab+c", "abc"), true);
}

#[test]
fn test_plus_many_true() {
    assert_eq!(is_match("ab+c", "abbbbc"), true);
}

#[test]
fn test_plus_zero_false() {
    assert_eq!(is_match("ab+c", "ac"), false);
}

#[test]
fn test_plus_no_match_false() {
    assert_eq!(is_match("ab+c", "axc"), false);
}

// ============================================================================
// question (zero or one, is_match)
// ============================================================================

#[test]
fn test_question_zero_true() {
    assert_eq!(is_match("colou?r", "color"), true);
}

#[test]
fn test_question_one_true() {
    assert_eq!(is_match("colou?r", "colour"), true);
}

#[test]
fn test_question_two_false() {
    assert_eq!(is_match("ab?c", "abbc"), false);
}

#[test]
fn test_question_no_match_false() {
    assert_eq!(is_match("ab?c", "axc"), false);
}

// ============================================================================
// character classes (is_match)
// ============================================================================

#[test]
fn test_class_single_true() {
    assert_eq!(is_match("[abc]", "b"), true);
}

#[test]
fn test_class_single_no_match_false() {
    assert_eq!(is_match("[abc]", "d"), false);
}

#[test]
fn test_class_in_word_true() {
    assert_eq!(is_match("x[abc]y", "xby"), true);
}

#[test]
fn test_class_in_word_no_match_false() {
    assert_eq!(is_match("x[abc]y", "xdy"), false);
}

#[test]
fn test_class_range_digit_true() {
    assert_eq!(is_match("[0-9]", "5"), true);
}

#[test]
fn test_class_range_digit_no_match_false() {
    assert_eq!(is_match("[0-9]", "a"), false);
}

#[test]
fn test_class_range_alpha_true() {
    assert_eq!(is_match("[a-z][0-9]", "k7"), true);
}

#[test]
fn test_class_range_alpha_no_match_false() {
    assert_eq!(is_match("[a-z][0-9]", "77"), false);
}

#[test]
fn test_class_with_star_true() {
    assert_eq!(is_match("[0-9]*", "abc"), true);
}

#[test]
fn test_class_with_plus_true() {
    assert_eq!(is_match("[0-9]+x", "12x"), true);
}

#[test]
fn test_class_with_plus_no_match_false() {
    assert_eq!(is_match("[0-9]+x", "abx"), false);
}

// ============================================================================
// find (leftmost match as string)
// ============================================================================

#[test]
fn test_find_literal() {
    assert_eq!(find("cde", "abcde"), "cde");
}

#[test]
fn test_find_dot() {
    assert_eq!(find("a.c", "xxabcxx"), "abc");
}

#[test]
fn test_find_star_greedy() {
    assert_eq!(find("ab*c", "abbbc"), "abbbc");
}

#[test]
fn test_find_star_zero() {
    assert_eq!(find("ab*c", "ac"), "ac");
}

#[test]
fn test_find_plus() {
    assert_eq!(find("a+b", "aaaab"), "aaaab");
}

#[test]
fn test_find_class() {
    assert_eq!(find("[0-9]+", "ab123cd"), "123");
}

#[test]
fn test_find_question() {
    assert_eq!(find("colou?r", "colour"), "colour");
}

#[test]
fn test_find_no_match_empty() {
    assert_eq!(find("xyz", "abc"), "");
}

#[test]
fn test_find_partial_word() {
    assert_eq!(find("cat", "the cat sat"), "cat");
}

#[test]
fn test_find_dot_greedy() {
    assert_eq!(find("a.*e", "abcde"), "abcde");
}
