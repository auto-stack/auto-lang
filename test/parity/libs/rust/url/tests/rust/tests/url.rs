//! Native Rust oracle tests for the url replication.
//!
//! These assert the same input -> output mapping that the Auto implementation
//! must match. Test names EXACTLY mirror the TAP test names in
//! `tests/auto/{parse,components}.at` so the parity framework can compare
//! them three-way (AutoVM vs a2r vs native Rust).
//!
//! The url crate (`url::Url`) is the oracle. To stay consistent with the
//! simplified Auto parser, these helpers apply the same representation
//! choices:
//!   * `port` absent -> -1 (sentinel), matching the Auto accessor.
//!   * `query` / `fragment` absent -> "" (empty string).
//! The url crate lower-cases scheme and host and normalises an empty path to
//! "/", which the Auto parser also does (for scheme and path).

use url::Url;

/// Scheme, lower-cased (the url crate already lower-cases it). "" on error.
fn scheme(input: &str) -> String {
    Url::parse(input).map(|u| u.scheme().to_string()).unwrap_or_default()
}

/// Host without port. The url crate lower-cases the host; the Auto tests only
/// use already-lower-case hosts, so this is consistent. "" on error.
fn host(input: &str) -> String {
    Url::parse(input)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default()
}

/// Path, normalised to "/" when empty (the url crate does this for special
/// schemes). "" on error (should not happen for the tested inputs).
fn path(input: &str) -> String {
    Url::parse(input)
        .map(|u| u.path().to_string())
        .unwrap_or_default()
}

/// Explicit port, or -1 when absent (sentinel matching the Auto accessor).
/// Note: the url crate strips *default* ports (80 for http, 443 for https),
/// returning None. The Auto parser keeps explicit ports. The tests therefore
/// use only non-default ports so both backends agree.
fn port(input: &str) -> i32 {
    Url::parse(input)
        .ok()
        .and_then(|u| u.port())
        .map(|p| p as i32)
        .unwrap_or(-1)
}

/// Query without the leading '?', or "" when absent.
fn query(input: &str) -> String {
    Url::parse(input)
        .ok()
        .and_then(|u| u.query().map(|q| q.to_string()))
        .unwrap_or_default()
}

/// Fragment without the leading '#', or "" when absent.
fn fragment(input: &str) -> String {
    Url::parse(input)
        .ok()
        .and_then(|u| u.fragment().map(|f| f.to_string()))
        .unwrap_or_default()
}

/// True when the url crate accepts the input (Ok).
fn parse_ok(input: &str) -> bool {
    Url::parse(input).is_ok()
}

// ============================================================================
// parse tests (mirror tests/auto/parse.at)
// ============================================================================

#[test]
fn test_scheme_http() {
    assert_eq!(scheme("http://example.com/path"), "http");
}

#[test]
fn test_scheme_https() {
    assert_eq!(scheme("https://example.com/path"), "https");
}

#[test]
fn test_scheme_ftp() {
    assert_eq!(scheme("ftp://files.example.com"), "ftp");
}

#[test]
fn test_scheme_uppercase_lowered() {
    assert_eq!(scheme("HTTP://Example.COM/p"), "http");
}

#[test]
fn test_host_simple() {
    assert_eq!(host("http://example.com/path"), "example.com");
}

#[test]
fn test_host_with_port_stripped() {
    assert_eq!(host("http://example.com:8080/path"), "example.com");
}

#[test]
fn test_host_localhost() {
    assert_eq!(host("http://localhost:3000/"), "localhost");
}

#[test]
fn test_path_simple() {
    assert_eq!(path("http://example.com/path"), "/path");
}

#[test]
fn test_path_multi_segment() {
    assert_eq!(path("http://example.com/a/b/c"), "/a/b/c");
}

#[test]
fn test_path_empty_normalised_to_root() {
    assert_eq!(path("http://example.com"), "/");
}

#[test]
fn test_path_root() {
    assert_eq!(path("http://example.com/"), "/");
}

#[test]
fn test_parse_valid_http() {
    assert!(parse_ok("http://example.com/path"));
}

#[test]
fn test_parse_valid_https() {
    assert!(parse_ok("https://example.com"));
}

#[test]
fn test_parse_missing_scheme_separator() {
    // Relative URL without a base -> the url crate rejects it.
    assert!(!parse_ok("example.com/path"));
}

#[test]
fn test_parse_empty_string() {
    assert!(!parse_ok(""));
}

#[test]
fn test_parse_only_separator() {
    assert!(!parse_ok("://example.com"));
}

// ============================================================================
// components tests (mirror tests/auto/components.at)
// ============================================================================

#[test]
fn test_port_explicit_8080() {
    assert_eq!(port("http://example.com:8080/path"), 8080);
}

#[test]
fn test_port_explicit_3000() {
    assert_eq!(port("http://localhost:3000/"), 3000);
}

#[test]
fn test_port_https_8443() {
    assert_eq!(port("https://example.com:8443/a"), 8443);
}

#[test]
fn test_port_absent_returns_neg1() {
    assert_eq!(port("http://example.com/path"), -1);
}

#[test]
fn test_query_simple() {
    assert_eq!(query("http://example.com/p?x=1"), "x=1");
}

#[test]
fn test_query_multi() {
    assert_eq!(query("http://example.com/p?a=1&b=2"), "a=1&b=2");
}

#[test]
fn test_query_with_fragment_present() {
    assert_eq!(query("http://example.com/p?x=1#frag"), "x=1");
}

#[test]
fn test_query_absent_empty() {
    assert_eq!(query("http://example.com/p"), "");
}

#[test]
fn test_fragment_simple() {
    assert_eq!(fragment("http://example.com/p#section"), "section");
}

#[test]
fn test_fragment_with_query_present() {
    assert_eq!(fragment("http://example.com/p?x=1#sec"), "sec");
}

#[test]
fn test_fragment_absent_empty() {
    assert_eq!(fragment("http://example.com/p"), "");
}

#[test]
fn test_full_url_port() {
    assert_eq!(port("http://example.com:9090/a/b?x=1#f"), 9090);
}

#[test]
fn test_full_url_query() {
    assert_eq!(query("http://example.com:9090/a/b?x=1#f"), "x=1");
}

#[test]
fn test_full_url_fragment() {
    assert_eq!(fragment("http://example.com:9090/a/b?x=1#f"), "f");
}
