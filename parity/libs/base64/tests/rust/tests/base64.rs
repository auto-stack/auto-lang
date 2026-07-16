//! Native Rust oracle tests for the base64 replication.
//!
//! These assert the same input -> output mapping that the Auto implementation
//! must match. Test names EXACTLY mirror the TAP test names in
//! `tests/auto/{encode,decode,edge_cases}.at` so the parity framework can
//! compare them three-way (AutoVM vs a2r vs native Rust).

use base64::{engine::general_purpose::STANDARD, Engine};

/// Encode a string to standard-alphabet base64 (padded).
fn encode(input: &str) -> String {
    STANDARD.encode(input.as_bytes())
}

/// Decode standard-alphabet base64 (padded) to a string, mirroring the Auto
/// `decode` signature which returns Result<String, String>.
fn decode(input: &str) -> Result<String, String> {
    STANDARD
        .decode(input.as_bytes())
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| format!("invalid: {}", e))
}

// ============================================================================
// encode tests (mirror tests/auto/encode.at)
// ============================================================================

#[test]
fn test_encode_empty() {
    assert_eq!(encode(""), "");
}

#[test]
fn test_encode_f() {
    assert_eq!(encode("f"), "Zg==");
}

#[test]
fn test_encode_fo() {
    assert_eq!(encode("fo"), "Zm8=");
}

#[test]
fn test_encode_foo() {
    assert_eq!(encode("foo"), "Zm9v");
}

#[test]
fn test_encode_foob() {
    assert_eq!(encode("foob"), "Zm9vYg==");
}

#[test]
fn test_encode_fooba() {
    assert_eq!(encode("fooba"), "Zm9vYmE=");
}

#[test]
fn test_encode_foobar() {
    assert_eq!(encode("foobar"), "Zm9vYmFy");
}

#[test]
fn test_encode_hello_world() {
    assert_eq!(encode("hello world"), "aGVsbG8gd29ybGQ=");
}

#[test]
fn test_encode_a() {
    assert_eq!(encode("a"), "YQ==");
}

#[test]
fn test_encode_ab() {
    assert_eq!(encode("ab"), "YWI=");
}

#[test]
fn test_encode_abc() {
    assert_eq!(encode("abc"), "YWJj");
}

#[test]
fn test_encode_number() {
    assert_eq!(encode("123"), "MTIz");
}

// ============================================================================
// decode tests (mirror tests/auto/decode.at)
// ============================================================================

#[test]
fn test_decode_empty() {
    assert_eq!(decode("").unwrap(), "");
}

#[test]
fn test_decode_Zg() {
    assert_eq!(decode("Zg==").unwrap(), "f");
}

#[test]
fn test_decode_Zm8() {
    assert_eq!(decode("Zm8=").unwrap(), "fo");
}

#[test]
fn test_decode_Zm9v() {
    assert_eq!(decode("Zm9v").unwrap(), "foo");
}

#[test]
fn test_decode_Zm9vYg() {
    assert_eq!(decode("Zm9vYg==").unwrap(), "foob");
}

#[test]
fn test_decode_Zm9vYmE() {
    assert_eq!(decode("Zm9vYmE=").unwrap(), "fooba");
}

#[test]
fn test_decode_Zm9vYmFy() {
    assert_eq!(decode("Zm9vYmFy").unwrap(), "foobar");
}

#[test]
fn test_decode_hello_world() {
    assert_eq!(decode("aGVsbG8gd29ybGQ=").unwrap(), "hello world");
}

#[test]
fn test_decode_invalid_chars() {
    // '!' is not a valid base64 symbol -> must error.
    assert!(decode("!!!!").is_err());
}

#[test]
fn test_decode_bad_length() {
    // Length not a multiple of 4 -> must error.
    assert!(decode("abc").is_err());
}

// ============================================================================
// edge_cases / roundtrip tests (mirror tests/auto/edge_cases.at)
// ============================================================================

fn roundtrip(s: &str) -> bool {
    decode(&encode(s)).map(|d| d == s).unwrap_or(false)
}

#[test]
fn test_roundtrip_empty() {
    assert!(roundtrip(""));
}

#[test]
fn test_roundtrip_a() {
    assert!(roundtrip("a"));
}

#[test]
fn test_roundtrip_ab() {
    assert!(roundtrip("ab"));
}

#[test]
fn test_roundtrip_abc() {
    assert!(roundtrip("abc"));
}

#[test]
fn test_roundtrip_foobar() {
    assert!(roundtrip("foobar"));
}

#[test]
fn test_roundtrip_hello_world() {
    assert!(roundtrip("hello world"));
}

#[test]
fn test_roundtrip_123456() {
    assert!(roundtrip("123456"));
}

#[test]
fn test_roundtrip_abcdefg() {
    assert!(roundtrip("abcdefg"));
}

#[test]
fn test_known_zero_pad() {
    assert_eq!(encode("M"), "TQ==");
}

#[test]
fn test_known_two_pad() {
    assert_eq!(encode("Ma"), "TWE=");
}

#[test]
fn test_known_no_pad() {
    assert_eq!(encode("Man"), "TWFu");
}
