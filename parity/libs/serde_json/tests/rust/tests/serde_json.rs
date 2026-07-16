//! Native Rust oracle tests for the serde_json replication.
//!
//! These assert the same input -> output mapping that the Auto implementation
//! must match. Test names EXACTLY mirror the TAP test names in
//! `tests/auto/{parse,roundtrip}.at` so the parity framework can compare them
//! three-way (AutoVM vs a2r vs native Rust).
//!
//! Mapping to the Auto helpers:
//!   parse(input)      -> Ok(canonical) | Err  : `serde_json::from_str::<Value>`
//!                                          then `Value::to_string` for the Ok payload.
//!   to_string(input)  -> canonical | ""      : parse-or-empty, then `to_string`.
//!
//! NOTE on number formatting: the Auto parser emits number literals verbatim
//! from the source. serde_json re-formats floats (e.g. `1e3` -> `1000.0`). To
//! keep three-way parity, the test set uses only numbers that serialize
//! identically under both (integers and simple decimals). See
//! `parity/docs/known-divergences.md` for the documented exponent divergence.

use serde_json::{from_str, Value};

/// Mirror of the Auto `parse`: Ok(canonical_text) on success, Err on failure.
/// The canonical text is serde_json's `Value::to_string` of the parsed value.
fn parse(input: &str) -> Result<String, String> {
    match from_str::<Value>(input) {
        Ok(v) => Ok(v.to_string()),
        Err(_) => Err("invalid JSON".to_string()),
    }
}

/// Mirror of the Auto `to_string`: canonical re-serialization, or "" on error.
fn to_string(input: &str) -> String {
    match from_str::<Value>(input) {
        Ok(v) => v.to_string(),
        Err(_) => String::new(),
    }
}

// ============================================================================
// parse tests (mirror tests/auto/parse.at)
// ============================================================================

#[test]
fn test_parse_null() {
    assert_eq!(parse("null").unwrap(), "null");
}

#[test]
fn test_parse_true() {
    assert_eq!(parse("true").unwrap(), "true");
}

#[test]
fn test_parse_false() {
    assert_eq!(parse("false").unwrap(), "false");
}

#[test]
fn test_parse_int_zero() {
    assert_eq!(parse("0").unwrap(), "0");
}

#[test]
fn test_parse_int_positive() {
    assert_eq!(parse("42").unwrap(), "42");
}

#[test]
fn test_parse_int_negative() {
    assert_eq!(parse("-7").unwrap(), "-7");
}

#[test]
fn test_parse_float() {
    assert_eq!(parse("3.14").unwrap(), "3.14");
}

#[test]
fn test_parse_float_negative() {
    assert_eq!(parse("-0.5").unwrap(), "-0.5");
}

#[test]
fn test_parse_float_frac() {
    assert_eq!(parse("10.25").unwrap(), "10.25");
}

#[test]
fn test_parse_big_int() {
    assert_eq!(parse("123456789").unwrap(), "123456789");
}

#[test]
fn test_parse_string_empty() {
    assert_eq!(parse("\"\"").unwrap(), "\"\"");
}

#[test]
fn test_parse_string_simple() {
    assert_eq!(parse("\"hello\"").unwrap(), "\"hello\"");
}

#[test]
fn test_parse_string_spaces() {
    assert_eq!(parse("\"a b c\"").unwrap(), "\"a b c\"");
}

#[test]
fn test_parse_array_empty() {
    assert_eq!(parse("[]").unwrap(), "[]");
}

#[test]
fn test_parse_array_numbers() {
    assert_eq!(parse("[1,2,3]").unwrap(), "[1,2,3]");
}

#[test]
fn test_parse_array_mixed() {
    assert_eq!(parse("[1,true,null]").unwrap(), "[1,true,null]");
}

#[test]
fn test_parse_array_nested() {
    assert_eq!(parse("[[1,2],[3,4]]").unwrap(), "[[1,2],[3,4]]");
}

#[test]
fn test_parse_object_empty() {
    assert_eq!(parse("{}").unwrap(), "{}");
}

#[test]
fn test_parse_object_simple() {
    assert_eq!(parse("{\"a\":1}").unwrap(), "{\"a\":1}");
}

#[test]
fn test_parse_object_multi() {
    assert_eq!(parse("{\"a\":1,\"b\":2}").unwrap(), "{\"a\":1,\"b\":2}");
}

#[test]
fn test_parse_object_string_val() {
    assert_eq!(parse("{\"k\":\"v\"}").unwrap(), "{\"k\":\"v\"}");
}

#[test]
fn test_parse_object_nested() {
    assert_eq!(parse("{\"a\":{\"b\":1}}").unwrap(), "{\"a\":{\"b\":1}}");
}

#[test]
fn test_parse_object_array_val() {
    assert_eq!(parse("{\"a\":[1,2]}").unwrap(), "{\"a\":[1,2]}");
}

#[test]
fn test_parse_ws_array() {
    assert_eq!(parse("  [ 1 , 2 ]  ").unwrap(), "[1,2]");
}

#[test]
fn test_parse_ws_object() {
    assert_eq!(parse(" { \"a\" : 1 } ").unwrap(), "{\"a\":1}");
}

#[test]
fn test_to_string_ws_nested() {
    assert_eq!(to_string("  { \"a\" : [ 1 , 2 ] }  "), "{\"a\":[1,2]}");
}

#[test]
fn test_to_string_null() {
    assert_eq!(to_string("null"), "null");
}

#[test]
fn test_to_string_number() {
    assert_eq!(to_string("42"), "42");
}

#[test]
fn test_to_string_array() {
    assert_eq!(to_string("[1, 2, 3]"), "[1,2,3]");
}

#[test]
fn test_to_string_object() {
    assert_eq!(to_string("{\"x\": 1}"), "{\"x\":1}");
}

#[test]
fn test_parse_err_empty() {
    assert!(parse("").is_err());
}

#[test]
fn test_parse_err_trailing_data() {
    assert!(parse("1 2").is_err());
}

#[test]
fn test_parse_err_unclosed_array() {
    assert!(parse("[1,2").is_err());
}

#[test]
fn test_parse_err_unclosed_object() {
    assert!(parse("{\"a\":1").is_err());
}

#[test]
fn test_parse_err_unclosed_string() {
    assert!(parse("\"abc").is_err());
}

#[test]
fn test_parse_err_bad_literal() {
    assert!(parse("tru").is_err());
}

#[test]
fn test_parse_err_double_comma() {
    assert!(parse("[1,,2]").is_err());
}

#[test]
fn test_parse_err_missing_colon() {
    assert!(parse("{\"a\" 1}").is_err());
}

#[test]
fn test_parse_err_leading_comma() {
    assert!(parse("[,1]").is_err());
}

#[test]
fn test_parse_err_garbage() {
    assert!(parse("@#$").is_err());
}

// ============================================================================
// roundtrip tests (mirror tests/auto/roundtrip.at)
//
// For each input: first = parse(input).unwrap(); second = to_string(first);
// assert first == second (idempotent) and first == compact.
// ============================================================================

fn roundtrip_check(input: &str, compact: &str) {
    let first = parse(input).expect("parse failed");
    let second = to_string(&first);
    assert_eq!(first, second, "not idempotent for input {:?}", input);
    assert_eq!(first, compact, "canonical mismatch for input {:?}", input);
}

#[test]
fn test_roundtrip_null() {
    roundtrip_check("null", "null");
}

#[test]
fn test_roundtrip_true() {
    roundtrip_check("true", "true");
}

#[test]
fn test_roundtrip_false() {
    roundtrip_check("false", "false");
}

#[test]
fn test_roundtrip_int() {
    roundtrip_check("42", "42");
}

#[test]
fn test_roundtrip_float() {
    roundtrip_check("3.14", "3.14");
}

#[test]
fn test_roundtrip_string() {
    roundtrip_check("\"hello\"", "\"hello\"");
}

#[test]
fn test_roundtrip_array() {
    roundtrip_check("[1,2,3]", "[1,2,3]");
}

#[test]
fn test_roundtrip_object() {
    roundtrip_check("{\"a\":1}", "{\"a\":1}");
}

#[test]
fn test_roundtrip_nested_array() {
    roundtrip_check("[[1],[2,3]]", "[[1],[2,3]]");
}

#[test]
fn test_roundtrip_nested_object() {
    roundtrip_check("{\"a\":{\"b\":1}}", "{\"a\":{\"b\":1}}");
}

#[test]
fn test_roundtrip_mixed() {
    roundtrip_check("{\"a\":[1,2],\"b\":true}", "{\"a\":[1,2],\"b\":true}");
}

#[test]
fn test_roundtrip_empty_array() {
    roundtrip_check("[]", "[]");
}

#[test]
fn test_roundtrip_empty_object() {
    roundtrip_check("{}", "{}");
}

#[test]
fn test_roundtrip_ws_array() {
    roundtrip_check("  [ 1 , 2 ]  ", "[1,2]");
}

#[test]
fn test_roundtrip_ws_object() {
    roundtrip_check("{ \"a\" : 1 , \"b\" : 2 }", "{\"a\":1,\"b\":2}");
}

#[test]
fn test_roundtrip_ws_nested() {
    roundtrip_check("  { \"a\" : [ 1 , 2 ] }  ", "{\"a\":[1,2]}");
}
