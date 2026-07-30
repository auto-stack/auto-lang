/// Rust oracle for c_json_app consumer parity.
fn get_str(json_str: &str, key: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}
fn check_valid(json_str: &str) -> i32 {
    if serde_json::from_str::<serde_json::Value>(json_str).is_ok() { 1 } else { 0 }
}
fn get_array_len(json_str: &str) -> i32 {
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();
    v.as_array().map(|a| a.len() as i32).unwrap_or(-1)
}
fn get_type(json_str: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap_or_default();
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }.to_string()
}
fn tap_ok(n: i32, name: &str) { println!("ok {} - {}", n, name); }

#[test] fn test_get_str() { assert_eq!(get_str(r#"{"name":"hello","n":42,"arr":[1,2,3]}"#, "name"), "hello"); tap_ok(1, "test_get_str"); }
#[test] fn test_valid_json() { assert_eq!(check_valid(r#"{"a":1}"#), 1); tap_ok(2, "test_valid_json"); }
#[test] fn test_invalid_json() { assert_eq!(check_valid("{broken}"), 0); tap_ok(3, "test_invalid_json"); }
#[test] fn test_array_len() { assert_eq!(get_array_len("[1,2,3]"), 3); tap_ok(4, "test_array_len"); }
#[test] fn test_type_object() { assert_eq!(get_type(r#"{"a":1}"#), "object"); tap_ok(5, "test_type_object"); }
