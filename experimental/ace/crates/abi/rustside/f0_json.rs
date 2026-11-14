//! auto-ffi provider f0_json/1 target=x86_64-pc-windows-msvc profile=f0-json.v1
//!
//! PLAN-967 T-15: the frozen REAL-LIBRARY Rust provider — pinned
//! serde_json (the `dep` annotation below is the whole dependency
//! configuration; the cargo build unit consumes it, the importer
//! ignores it). The annotations ARE the import metadata.

// auto-ffi dep serde_json =1.0.151

/// Top-level object key count; parse failure -> named `bad-json`.
/// A non-object document returns the VALUE -1 (shape observable without
/// the failure channel — the trap discriminates the two).
// auto-ffi fn json_object_len symbol=f0.json.object_len.v1 form=fallible errors=bad-json:-1
pub fn json_object_len(s: &str) -> Option<i32> {
    let v: serde_json::Value = serde_json::from_str(s).ok()?;
    match v {
        serde_json::Value::Object(map) => Some(map.len() as i32),
        _ => Some(-1),
    }
}

/// from_str -> to_string round-trip byte length (both public APIs in
/// one call); parse failure -> named `bad-json`.
// auto-ffi fn json_roundtrip_len symbol=f0.json.roundtrip_len.v1 form=fallible errors=bad-json:-1
pub fn json_roundtrip_len(s: &str) -> Option<i32> {
    let v: serde_json::Value = serde_json::from_str(s).ok()?;
    Some(v.to_string().len() as i32)
}

/// Document type as a stable id (0 object, 1 array, 2 string, 3 number,
/// 4 bool, 5 null, -1 parse failure) — the Value surface made visible.
// auto-ffi fn json_type_id symbol=f0.json.type_id.v1 form=pure ret=i32
pub fn json_type_id(s: &str) -> i32 {
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(v) => match v {
            serde_json::Value::Object(_) => 0,
            serde_json::Value::Array(_) => 1,
            serde_json::Value::String(_) => 2,
            serde_json::Value::Number(_) => 3,
            serde_json::Value::Bool(_) => 4,
            serde_json::Value::Null => 5,
        },
        Err(_) => -1,
    }
}
