//! Native Rust oracle tests for the cli_app replication.
//!
//! These assert the same input -> output mapping that the Auto implementation
//! must match. Test names EXACTLY mirror the TAP test names in
//! `tests/auto/{basic,edge_cases}.at` so the parity framework can compare
//! them three-way (AutoVM vs a2r vs native Rust std).
//!
//! The implementation uses only `std` (no external crates), which is the whole
//! purpose of the cli_app use case: it exercises a2r's "pure Rust output,
//! no external dependency" path and gives the parity framework a
//! fully-deterministic, IO-free oracle.

/// Count lines: empty string = 0; a trailing newline does not add a line;
/// otherwise the number of lines is count('\n') + 1. This is exactly
/// `str::lines().count()`:
///   "" -> 0, "a" -> 1, "a\n" -> 1, "a\nb" -> 2, "\n" -> 1, "\n\n" -> 2.
fn count_lines(s: &str) -> i32 {
    s.lines().count() as i32
}

/// Count words: split on ASCII whitespace, count non-empty tokens. This is
/// exactly `str::split_whitespace().count()`.
fn count_words(s: &str) -> i32 {
    s.split_whitespace().count() as i32
}

/// Count characters (byte length for ASCII). This is exactly `str::len()`.
/// Tests are ASCII-only to avoid UTF-8 byte-vs-char ambiguity.
fn count_chars(s: &str) -> i32 {
    s.len() as i32
}

// ============================================================================
// basic tests (mirror tests/auto/basic.at)
// ============================================================================

// ---- count_lines -----------------------------------------------------------

#[test]
fn test_lines_empty() {
    assert_eq!(count_lines(""), 0);
}

#[test]
fn test_lines_single_no_nl() {
    assert_eq!(count_lines("a"), 1);
}

#[test]
fn test_lines_one_trailing_nl() {
    assert_eq!(count_lines("a\n"), 1);
}

#[test]
fn test_lines_two_no_trailing() {
    assert_eq!(count_lines("a\nb"), 2);
}

#[test]
fn test_lines_two_trailing_nl() {
    assert_eq!(count_lines("a\nb\n"), 2);
}

#[test]
fn test_lines_single_newline() {
    assert_eq!(count_lines("\n"), 1);
}

#[test]
fn test_lines_two_newlines() {
    assert_eq!(count_lines("\n\n"), 2);
}

#[test]
fn test_lines_paragraph() {
    assert_eq!(count_lines("foo\nbar\nbaz"), 3);
}

// ---- count_words -----------------------------------------------------------

#[test]
fn test_words_empty() {
    assert_eq!(count_words(""), 0);
}

#[test]
fn test_words_only_spaces() {
    assert_eq!(count_words("   "), 0);
}

#[test]
fn test_words_single() {
    assert_eq!(count_words("hello"), 1);
}

#[test]
fn test_words_three_spaces() {
    assert_eq!(count_words("a b c"), 3);
}

#[test]
fn test_words_leading_trailing_ws() {
    assert_eq!(count_words("  a b  c  "), 3);
}

#[test]
fn test_words_newline_separated() {
    assert_eq!(count_words("foo\nbar"), 2);
}

// ---- count_chars -----------------------------------------------------------

#[test]
fn test_chars_empty() {
    assert_eq!(count_chars(""), 0);
}

#[test]
fn test_chars_single() {
    assert_eq!(count_chars("a"), 1);
}

#[test]
fn test_chars_word() {
    assert_eq!(count_chars("hello"), 5);
}

#[test]
fn test_chars_with_newline() {
    assert_eq!(count_chars("ab\nc"), 4);
}

// ============================================================================
// edge_cases tests (mirror tests/auto/edge_cases.at)
// ============================================================================

// ---- count_lines edge cases ------------------------------------------------

#[test]
fn test_edge_lines_three_newlines() {
    assert_eq!(count_lines("\n\n\n"), 3);
}

#[test]
fn test_edge_lines_crlf() {
    // `str::lines()` splits on both `\n` and `\r\n`, so "a\r\nb" is 2 lines.
    assert_eq!(count_lines("a\r\nb"), 2);
}

#[test]
fn test_edge_lines_leading_nl() {
    assert_eq!(count_lines("\nabc"), 2);
}

#[test]
fn test_edge_lines_many() {
    assert_eq!(count_lines("1\n2\n3\n4\n5\n6\n7\n8\n9\n10"), 10);
}

// ---- count_words edge cases ------------------------------------------------

#[test]
fn test_edge_words_tab_separated() {
    assert_eq!(count_words("a\tb\tc"), 3);
}

#[test]
fn test_edge_words_mixed_ws() {
    assert_eq!(count_words("a  \tb\n c"), 3);
}

#[test]
fn test_edge_words_cr_separated() {
    // `split_whitespace` treats `\r` as whitespace.
    assert_eq!(count_words("a\rb"), 2);
}

#[test]
fn test_edge_words_many() {
    assert_eq!(count_words("the quick brown fox jumps over the lazy dog"), 9);
}

// ---- count_chars edge cases ------------------------------------------------

#[test]
fn test_edge_chars_with_tab() {
    assert_eq!(count_chars("a\tb"), 3);
}

#[test]
fn test_edge_chars_crlf() {
    assert_eq!(count_chars("a\r\nb"), 4);
}

#[test]
fn test_edge_chars_long() {
    assert_eq!(count_chars("abcdefghijklmnopqrstuvwxyz"), 26);
}

// ---- combined sanity (all three over the same input) -----------------------

#[test]
fn test_edge_combined_lines() {
    assert_eq!(count_lines("one two\nthree four\nfive six"), 3);
}

#[test]
fn test_edge_combined_words() {
    assert_eq!(count_words("one two\nthree four\nfive six"), 6);
}

#[test]
fn test_edge_combined_chars() {
    // 7 + 1 + 10 + 1 + 8 = 27
    assert_eq!(count_chars("one two\nthree four\nfive six"), 27);
}
