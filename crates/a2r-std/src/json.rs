/// JSON module - JSON encoding, decoding, and querying
/// Transpiled from auto-lang/stdlib/auto/json.at + json.rs.at

use serde_json::Value;

// ═══════════════════════════════════════════════════════════
// Encoding
// ═══════════════════════════════════════════════════════════

pub fn encode(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

pub fn encode_pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

// ═══════════════════════════════════════════════════════════
// Decoding
// ═══════════════════════════════════════════════════════════

pub fn parse(s: &str) -> Option<Value> {
    serde_json::from_str(s).ok()
}

pub fn is_valid(s: &str) -> bool {
    serde_json::from_str::<Value>(s).is_ok()
}

// ═══════════════════════════════════════════════════════════
// JsonValue Operations
// ═══════════════════════════════════════════════════════════

pub fn value_type(val: &Value) -> String {
    match val {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
    .to_string()
}

pub fn is_null(val: &Value) -> bool {
    val.is_null()
}

pub fn as_string(val: &Value) -> Option<String> {
    val.as_str().map(|s| s.to_string())
}

pub fn as_number(val: &Value) -> f64 {
    val.as_f64().unwrap_or(0.0)
}

pub fn as_int(val: &Value) -> i64 {
    val.as_i64().unwrap_or(0)
}

pub fn as_bool(val: &Value) -> bool {
    val.as_bool().unwrap_or(false)
}

pub fn json_get(val: &Value, key: &str) -> Option<Value> {
    val.get(key).cloned()
}

pub fn get_at(val: &Value, idx: usize) -> Option<Value> {
    val.get(idx).cloned()
}

pub fn json_len(val: &Value) -> usize {
    match val {
        Value::Array(a) => a.len(),
        Value::Object(o) => o.len(),
        _ => 0,
    }
}

pub fn has_key(val: &Value, key: &str) -> bool {
    val.get(key).is_some()
}

pub fn keys(val: &Value) -> Vec<String> {
    val.as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

pub fn as_array(val: &Value) -> Vec<Value> {
    val.as_array().cloned().unwrap_or_default()
}

// ═══════════════════════════════════════════════════════════
// Utility
// ═══════════════════════════════════════════════════════════

pub fn prettify(s: &str) -> String {
    serde_json::from_str::<Value>(s)
        .ok()
        .map(|v| serde_json::to_string_pretty(&v).unwrap_or(s.to_string()))
        .unwrap_or_else(|| s.to_string())
}

pub fn minify(s: &str) -> String {
    serde_json::from_str::<Value>(s)
        .ok()
        .map(|v| serde_json::to_string(&v).unwrap_or(s.to_string()))
        .unwrap_or_else(|| s.to_string())
}
