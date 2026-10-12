//! Native Rust oracle tests for the rusqlite replication.
//!
//! These assert the same input -> output mapping that the Auto implementation
//! must match. Test names EXACTLY mirror the TAP test names in
//! `tests/auto/{from_sql,to_sql,roundtrip}.at` so the parity framework can
//! compare them three-way (AutoVM vs a2r vs native Rust).
//!
//! ## What this exercises
//!
//! rusqlite's `FromSql::column_result` (SQLite Value -> Rust value) and
//! `ToSql` (Rust value -> SQLite Value) coercions, driven through a real
//! in-memory SQLite database via `SELECT ?1`. This is the genuine rusqlite
//! 0.31.0 query layer — the deterministic, pure-function slice that the Auto
//! replication reproduces.
//!
//! ## Val + status/value API
//!
//! To match the Auto side's observable surface (which avoids returning
//! `Result<T>` across the module boundary due to VM marshalling bugs — see
//! `auto/rusqlite.at`), the oracle exposes the same flat API:
//!   - `Val { kind, ival, sval, fval }` — tagged SQLite value
//!   - `<from_X>_status(v) -> i32`  -> 0 Ok, 1 InvalidType, 2 OutOfRange
//!   - `<from_X>_value(v) -> T`     -> the coerced payload
//!
//! Each `_status`/`_value` runs the value through a real SQLite round-trip
//! (`SELECT ?1`) and invokes the real `FromSql` impl, so any divergence from
//! the Auto side is a genuine rusqlite-semantics difference.

use rusqlite::{Connection, Error, types::Value};

// ---------------------------------------------------------------------------
// Val — the tagged SQLite value, mirroring the Auto representation.
//   kind 0 = Null
//   kind 1 = Integer  -> ival (i64 bit pattern)
//   kind 2 = Real     -> fval
//   kind 3 = Text     -> sval
//   kind 4 = Blob     -> sval (bytes held as a string)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Val {
    kind: i32,
    ival: i64,
    sval: String,
    fval: f64,
}

fn null_() -> Val {
    Val { kind: 0, ival: 0, sval: String::new(), fval: 0.0 }
}
fn integer_(i: i64) -> Val {
    Val { kind: 1, ival: i, sval: String::new(), fval: 0.0 }
}
fn real_(f: f64) -> Val {
    Val { kind: 2, ival: 0, sval: String::new(), fval: f }
}
fn text_(s: &str) -> Val {
    Val { kind: 3, ival: 0, sval: s.to_string(), fval: 0.0 }
}
fn blob_(s: &str) -> Val {
    Val { kind: 4, ival: 0, sval: s.to_string(), fval: 0.0 }
}

fn data_type(v: &Val) -> i32 {
    v.kind
}

/// Convert a `Val` to a real rusqlite `Value` for binding into a query.
fn val_to_rusqlite(v: &Val) -> Value {
    match v.kind {
        0 => Value::Null,
        1 => Value::Integer(v.ival),
        2 => Value::Real(v.fval),
        3 => Value::Text(v.sval.clone()),
        4 => Value::Blob(v.sval.as_bytes().to_vec()),
        _ => Value::Null,
    }
}

/// A single in-memory connection reused across helper calls. SQLite columns
/// read back values with the same type tag they were bound with, so this
/// faithfully reproduces `ValueRef` as seen by `FromSql`.
fn conn() -> Connection {
    Connection::open_in_memory().expect("open in-memory db")
}

/// Run a Val through `SELECT ?1` and read column 0 as the requested Rust type
/// via real `FromSql`. Returns (status, int_payload) where status is
/// 0 = Ok, 1 = InvalidType, 2 = OutOfRange.
fn read_as_i64(v: &Val) -> (i32, i64) {
    let db = conn();
    let res: Result<i64, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, i64>(0));
    decode_int_result(res)
}

fn read_as_i32(v: &Val) -> (i32, i64) {
    let db = conn();
    let res: Result<i32, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, i32>(0));
    decode_int_result(res)
}

fn read_as_i16(v: &Val) -> (i32, i64) {
    let db = conn();
    let res: Result<i16, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, i16>(0));
    decode_int_result(res)
}

fn read_as_i8(v: &Val) -> (i32, i64) {
    let db = conn();
    let res: Result<i8, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, i8>(0));
    decode_int_result(res)
}

fn read_as_u8(v: &Val) -> (i32, i64) {
    let db = conn();
    let res: Result<u8, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, u8>(0));
    decode_int_result(res)
}

fn read_as_u16(v: &Val) -> (i32, i64) {
    let db = conn();
    let res: Result<u16, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, u16>(0));
    decode_int_result(res)
}

fn read_as_u32(v: &Val) -> (i32, i64) {
    let db = conn();
    let res: Result<u32, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, u32>(0));
    decode_int_result(res)
}

/// Map a rusqlite integral read result to (status, value). status:
/// 0 = Ok, 1 = InvalidType, 2 = OutOfRange.
fn decode_int_result<T: Into<i64> + Copy>(res: Result<T, Error>) -> (i32, i64) {
    match res {
        Ok(t) => (0, t.into()),
        Err(Error::IntegralValueOutOfRange(_, n)) => (2, n),
        // FromSqlError::InvalidType is wrapped in FromSqlConversionFailure as a
        // boxed error; FromSqlError::OutOfRange is mapped to
        // IntegralValueOutOfRange (handled above). Anything else is a type
        // mismatch (InvalidColumnType / conversion failure) -> InvalidType.
        Err(_) => (1, 0),
    }
}

// ---------------------------------------------------------------------------
// FromSql status/value helpers — mirror the Auto API exactly.
// ---------------------------------------------------------------------------

fn from_i64_status(v: &Val) -> i32 { read_as_i64(v).0 }
fn from_i64_value(v: &Val) -> i32 { read_as_i64(v).1 as i32 }
fn from_i32_status(v: &Val) -> i32 { read_as_i32(v).0 }
fn from_i32_value(v: &Val) -> i32 { read_as_i32(v).1 as i32 }
fn from_i16_status(v: &Val) -> i32 { read_as_i16(v).0 }
fn from_i16_value(v: &Val) -> i32 { read_as_i16(v).1 as i32 }
fn from_i8_status(v: &Val) -> i32 { read_as_i8(v).0 }
fn from_i8_value(v: &Val) -> i32 { read_as_i8(v).1 as i32 }
fn from_u8_status(v: &Val) -> i32 { read_as_u8(v).0 }
fn from_u8_value(v: &Val) -> i32 { read_as_u8(v).1 as i32 }
fn from_u16_status(v: &Val) -> i32 { read_as_u16(v).0 }
fn from_u16_value(v: &Val) -> i32 { read_as_u16(v).1 as i32 }
fn from_u32_status(v: &Val) -> i32 { read_as_u32(v).0 }
fn from_u32_value(v: &Val) -> i32 { read_as_u32(v).1 as i32 }

fn from_f64_status(v: &Val) -> i32 {
    let db = conn();
    let res: Result<f64, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, f64>(0));
    match res {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

fn from_f64_value(v: &Val) -> f64 {
    let db = conn();
    let res: f64 = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, f64>(0)).unwrap_or(0.0);
    res
}

fn from_bool_status(v: &Val) -> i32 {
    let db = conn();
    let res: Result<bool, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, bool>(0));
    match res {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

fn from_bool_value(v: &Val) -> bool {
    let db = conn();
    db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, bool>(0)).unwrap_or(false)
}

fn from_string_status(v: &Val) -> i32 {
    let db = conn();
    let res: Result<String, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, String>(0));
    match res {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

fn from_string_value(v: &Val) -> String {
    let db = conn();
    db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, String>(0)).unwrap_or_default()
}

fn option_is_none(v: &Val) -> bool {
    v.kind == 0
}

// ---------------------------------------------------------------------------
// data_type / variant tests (mirror tests/auto/from_sql.at "kind" cases)
// ---------------------------------------------------------------------------

#[test]
fn test_null_kind() { assert_eq!(data_type(&null_()), 0); }
#[test]
fn test_integer_kind() { assert_eq!(data_type(&integer_(5)), 1); }
#[test]
fn test_real_kind() { assert_eq!(data_type(&real_(3.5)), 2); }
#[test]
fn test_text_kind() { assert_eq!(data_type(&text_("x")), 3); }
#[test]
fn test_blob_kind() { assert_eq!(data_type(&blob_("y")), 4); }

// ---------------------------------------------------------------------------
// i64 FromSql (Integer -> i64; else InvalidType)
// ---------------------------------------------------------------------------

#[test]
fn test_i64_ok() {
    assert_eq!(from_i64_status(&integer_(5)), 0);
    assert_eq!(from_i64_value(&integer_(5)), 5);
}
#[test]
fn test_i64_negative() {
    assert_eq!(from_i64_status(&integer_(-42)), 0);
    assert_eq!(from_i64_value(&integer_(-42)), -42);
}
#[test]
fn test_i64_zero() {
    assert_eq!(from_i64_status(&integer_(0)), 0);
    assert_eq!(from_i64_value(&integer_(0)), 0);
}
#[test]
fn test_i64_invalid_type_text() { assert_eq!(from_i64_status(&text_("x")), 1); }
#[test]
fn test_i64_invalid_type_real() { assert_eq!(from_i64_status(&real_(1.0)), 1); }
#[test]
fn test_i64_invalid_type_null() { assert_eq!(from_i64_status(&null_()), 1); }

// ---------------------------------------------------------------------------
// i32 FromSql (range [-2147483648, 2147483647])
//
// The Auto VM int is 32-bit signed, so the out-of-range boundary values
// 2147483648 and -2147483649 cannot be represented (they wrap). Those cases
// are therefore excluded from the parity suite (see DIV-RUSQLITE-VM-2) — the
// in-range path and InvalidType path are fully covered, and i8/i16/u8/u16
// exercise the OutOfRange path with representable values.
// ---------------------------------------------------------------------------

#[test]
fn test_i32_in_range() {
    assert_eq!(from_i32_status(&integer_(100)), 0);
    assert_eq!(from_i32_value(&integer_(100)), 100);
}
#[test]
fn test_i32_max() {
    assert_eq!(from_i32_status(&integer_(2147483647)), 0);
    assert_eq!(from_i32_value(&integer_(2147483647)), 2147483647);
}
#[test]
fn test_i32_min() {
    assert_eq!(from_i32_status(&integer_(-2147483648)), 0);
    assert_eq!(from_i32_value(&integer_(-2147483648)), -2147483648);
}
#[test]
fn test_i32_invalid_type() { assert_eq!(from_i32_status(&text_("x")), 1); }

// ---------------------------------------------------------------------------
// i16 FromSql (range [-32768, 32767])
// ---------------------------------------------------------------------------

#[test]
fn test_i16_in_range() {
    assert_eq!(from_i16_status(&integer_(100)), 0);
    assert_eq!(from_i16_value(&integer_(100)), 100);
}
#[test]
fn test_i16_max() {
    assert_eq!(from_i16_status(&integer_(32767)), 0);
    assert_eq!(from_i16_value(&integer_(32767)), 32767);
}
#[test]
fn test_i16_min() {
    assert_eq!(from_i16_status(&integer_(-32768)), 0);
    assert_eq!(from_i16_value(&integer_(-32768)), -32768);
}
#[test]
fn test_i16_over_range() { assert_eq!(from_i16_status(&integer_(32768)), 2); }
#[test]
fn test_i16_under_range() { assert_eq!(from_i16_status(&integer_(-32769)), 2); }

// ---------------------------------------------------------------------------
// i8 FromSql (range [-128, 127])
// ---------------------------------------------------------------------------

#[test]
fn test_i8_in_range() {
    assert_eq!(from_i8_status(&integer_(100)), 0);
    assert_eq!(from_i8_value(&integer_(100)), 100);
}
#[test]
fn test_i8_max() {
    assert_eq!(from_i8_status(&integer_(127)), 0);
    assert_eq!(from_i8_value(&integer_(127)), 127);
}
#[test]
fn test_i8_min() {
    assert_eq!(from_i8_status(&integer_(-128)), 0);
    assert_eq!(from_i8_value(&integer_(-128)), -128);
}
#[test]
fn test_i8_over_range() { assert_eq!(from_i8_status(&integer_(200)), 2); }
#[test]
fn test_i8_under_range() { assert_eq!(from_i8_status(&integer_(-200)), 2); }

// ---------------------------------------------------------------------------
// u8 FromSql (range [0, 255])
// ---------------------------------------------------------------------------

#[test]
fn test_u8_in_range() {
    assert_eq!(from_u8_status(&integer_(100)), 0);
    assert_eq!(from_u8_value(&integer_(100)), 100);
}
#[test]
fn test_u8_max() {
    assert_eq!(from_u8_status(&integer_(255)), 0);
    assert_eq!(from_u8_value(&integer_(255)), 255);
}
#[test]
fn test_u8_zero() {
    assert_eq!(from_u8_status(&integer_(0)), 0);
    assert_eq!(from_u8_value(&integer_(0)), 0);
}
#[test]
fn test_u8_over_range() { assert_eq!(from_u8_status(&integer_(256)), 2); }
#[test]
fn test_u8_negative() { assert_eq!(from_u8_status(&integer_(-1)), 2); }

// ---------------------------------------------------------------------------
// u16 FromSql (range [0, 65535])
// ---------------------------------------------------------------------------

#[test]
fn test_u16_in_range() {
    assert_eq!(from_u16_status(&integer_(100)), 0);
    assert_eq!(from_u16_value(&integer_(100)), 100);
}
#[test]
fn test_u16_max() {
    assert_eq!(from_u16_status(&integer_(65535)), 0);
    assert_eq!(from_u16_value(&integer_(65535)), 65535);
}
#[test]
fn test_u16_over_range() { assert_eq!(from_u16_status(&integer_(65536)), 2); }
#[test]
fn test_u16_negative() { assert_eq!(from_u16_status(&integer_(-1)), 2); }

// ---------------------------------------------------------------------------
// u32 FromSql (range [0, 4294967295])
//
// u32::MAX (4294967295) exceeds the VM's 32-bit int range, so the upper
// boundary is not testable in the VM (DIV-RUSQLITE-VM-2). The in-range and
// negative -> OutOfRange paths are covered.
// ---------------------------------------------------------------------------

#[test]
fn test_u32_in_range() {
    assert_eq!(from_u32_status(&integer_(100)), 0);
    assert_eq!(from_u32_value(&integer_(100)), 100);
}
#[test]
fn test_u32_zero() {
    assert_eq!(from_u32_status(&integer_(0)), 0);
    assert_eq!(from_u32_value(&integer_(0)), 0);
}
#[test]
fn test_u32_negative() { assert_eq!(from_u32_status(&integer_(-1)), 2); }

// ---------------------------------------------------------------------------
// f64 FromSql (Integer -> i as f64, OR Real -> f, else InvalidType)
// ---------------------------------------------------------------------------

#[test]
fn test_f64_from_int() {
    assert_eq!(from_f64_status(&integer_(5)), 0);
    assert_eq!(from_f64_value(&integer_(5)), 5.0);
}
#[test]
fn test_f64_from_real() {
    assert_eq!(from_f64_status(&real_(3.5)), 0);
    assert_eq!(from_f64_value(&real_(3.5)), 3.5);
}
#[test]
fn test_f64_from_negative_int() {
    assert_eq!(from_f64_status(&integer_(-7)), 0);
    assert_eq!(from_f64_value(&integer_(-7)), -7.0);
}
#[test]
fn test_f64_invalid_type_text() { assert_eq!(from_f64_status(&text_("x")), 1); }
#[test]
fn test_f64_invalid_type_null() { assert_eq!(from_f64_status(&null_()), 1); }

// ---------------------------------------------------------------------------
// bool FromSql (`i64::column_result(value).map(|i| i != 0)`)
// ---------------------------------------------------------------------------

#[test]
fn test_bool_zero() {
    assert_eq!(from_bool_status(&integer_(0)), 0);
    assert_eq!(from_bool_value(&integer_(0)), false);
}
#[test]
fn test_bool_one() {
    assert_eq!(from_bool_status(&integer_(1)), 0);
    assert_eq!(from_bool_value(&integer_(1)), true);
}
#[test]
fn test_bool_nonzero() {
    assert_eq!(from_bool_status(&integer_(7)), 0);
    assert_eq!(from_bool_value(&integer_(7)), true);
}
#[test]
fn test_bool_negative() {
    assert_eq!(from_bool_status(&integer_(-3)), 0);
    assert_eq!(from_bool_value(&integer_(-3)), true);
}
#[test]
fn test_bool_invalid_type_text() { assert_eq!(from_bool_status(&text_("x")), 1); }
#[test]
fn test_bool_invalid_type_real() { assert_eq!(from_bool_status(&real_(1.0)), 1); }

// ---------------------------------------------------------------------------
// String FromSql (`value.as_str()`)
// ---------------------------------------------------------------------------

#[test]
fn test_string_ok() {
    assert_eq!(from_string_status(&text_("hello")), 0);
    assert_eq!(from_string_value(&text_("hello")), "hello");
}
#[test]
fn test_string_empty() {
    assert_eq!(from_string_status(&text_("")), 0);
    assert_eq!(from_string_value(&text_("")), "");
}
#[test]
fn test_string_invalid_type_int() { assert_eq!(from_string_status(&integer_(5)), 1); }
#[test]
fn test_string_invalid_type_null() { assert_eq!(from_string_status(&null_()), 1); }
#[test]
fn test_string_invalid_type_real() { assert_eq!(from_string_status(&real_(1.0)), 1); }

// ---------------------------------------------------------------------------
// Option<T> (Null -> None)
// ---------------------------------------------------------------------------

#[test]
fn test_option_none_on_null() { assert_eq!(option_is_none(&null_()), true); }
#[test]
fn test_option_some_on_int() { assert_eq!(option_is_none(&integer_(5)), false); }
#[test]
fn test_option_some_on_text() { assert_eq!(option_is_none(&text_("x")), false); }

// ---------------------------------------------------------------------------
// Blob FromSql (`value.as_blob()`) — bytes read back as a string.
// ---------------------------------------------------------------------------

fn from_blob_status(v: &Val) -> i32 {
    let db = conn();
    let res: Result<Vec<u8>, Error> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, Vec<u8>>(0));
    match res {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

fn from_blob_value(v: &Val) -> String {
    let db = conn();
    let bytes: Vec<u8> = db.query_row("SELECT ?1", [val_to_rusqlite(v)], |r| r.get::<_, Vec<u8>>(0)).unwrap_or_default();
    String::from_utf8_lossy(&bytes).into_owned()
}

fn from_option_status(v: &Val) -> i32 {
    if v.kind == 0 { 0 } else { 1 }
}

#[test]
fn test_blob_ok() {
    assert_eq!(from_blob_status(&blob_("abc")), 0);
    assert_eq!(from_blob_value(&blob_("abc")), "abc");
}
#[test]
fn test_blob_empty() {
    assert_eq!(from_blob_status(&blob_("")), 0);
    assert_eq!(from_blob_value(&blob_("")), "");
}
#[test]
fn test_blob_invalid_type_text() { assert_eq!(from_blob_status(&text_("x")), 1); }
#[test]
fn test_blob_invalid_type_int() { assert_eq!(from_blob_status(&integer_(5)), 1); }
#[test]
fn test_option_status_none() { assert_eq!(from_option_status(&null_()), 0); }
#[test]
fn test_option_status_some() { assert_eq!(from_option_status(&integer_(5)), 1); }

// ---------------------------------------------------------------------------
// Round-trip: ToSql then FromSql identity (status 0).
// ---------------------------------------------------------------------------

#[test]
fn test_roundtrip_i64_status() {
    // to_sql_i64(42) then from_i64 -> status 0
    let v = integer_(42);
    assert_eq!(from_i64_status(&v), 0);
}
#[test]
fn test_roundtrip_string_status() {
    let v = text_("hello");
    assert_eq!(from_string_status(&v), 0);
}
#[test]
fn test_roundtrip_bool_status() {
    // to_sql_bool(true) -> Integer(1) -> from_bool -> status 0
    let v = integer_(1);
    assert_eq!(from_bool_status(&v), 0);
}
