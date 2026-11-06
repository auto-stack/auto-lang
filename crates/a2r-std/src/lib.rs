//! Auto Language Standard Library for a2r (Auto-to-Rust Transpiler)
//!
//! This crate provides Rust implementations of AutoLang's standard library
//! modules so that transpiled code can compile and run.
//!
//! Generated from: auto-lang/stdlib/auto/*.at + *.rs.at

pub mod env;
pub mod fs;
pub mod hash;
pub mod hashmap;
pub mod http;
pub mod json;
pub mod zip;

/// Auto's runtime type predicate, including the distinction between `0` and
/// `0.0` in parsed JSON. Borrowing keeps the inspected value usable afterwards.
pub fn type_of<T: serde::Serialize + ?Sized>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(serde_json::Value::Null) => "null",
        Ok(serde_json::Value::Bool(_)) => "bool",
        Ok(serde_json::Value::Number(n)) if n.is_i64() || n.is_u64() => "int",
        Ok(serde_json::Value::Number(_)) => "float",
        Ok(serde_json::Value::String(_)) => "str",
        // VM JSON arrays and List.new values use its object heap tag.
        Ok(serde_json::Value::Array(_)) => "object",
        Ok(serde_json::Value::Object(_)) => "object",
        Err(_) => "unknown",
    }
    .to_string()
}

/// The VM's branch condition for dynamic JSON values. Strings and containers
/// are reference values, including empty ones; numeric zero and null are false.
pub fn json_condition(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::Bool(value) => *value,
        serde_json::Value::Number(value) => value.as_f64().is_some_and(|n| n != 0.0),
        _ => true,
    }
}

#[cfg(test)]
mod type_of_tests {
    use super::type_of;

    #[test]
    fn dynamic_json_preserves_runtime_kinds() {
        for (source, expected) in [
            ("0", "int"),
            ("-1", "int"),
            ("2147483647", "int"),
            ("0.0", "float"),
            ("1e0", "float"),
            ("-0.0", "float"),
            ("true", "bool"),
            ("false", "bool"),
            ("null", "null"),
            ("\"中文\"", "str"),
            ("[]", "object"),
            ("{}", "object"),
        ] {
            let value: serde_json::Value = serde_json::from_str(source).unwrap();
            let before = value.clone();
            assert_eq!(type_of(&value), expected, "source={source}");
            assert_eq!(value, before, "inspection consumed or mutated its input");
        }
    }

    #[test]
    fn scalar_and_collection_inputs_share_the_json_kind_names() {
        assert_eq!(type_of(&0_i32), "int");
        assert_eq!(type_of(&0.0_f64), "float");
        assert_eq!(type_of("text"), "str");
        assert_eq!(type_of(&true), "bool");
        assert_eq!(type_of(&Option::<i32>::None), "null");
        assert_eq!(type_of(&vec![1, 2]), "object");
    }

    #[test]
    fn dynamic_conditions_keep_zero_false_and_reference_values_true() {
        for (source, expected) in [
            ("null", false),
            ("false", false),
            ("0", false),
            ("0.0", false),
            ("-1", true),
            ("true", true),
            ("\"\"", true),
            ("[]", true),
            ("{}", true),
        ] {
            assert_eq!(
                super::json_condition(&serde_json::from_str(source).unwrap()),
                expected,
                "{source}"
            );
        }
    }
}
pub mod list;
pub mod math;
pub mod process;
pub mod rand;
pub mod redis;
pub mod sqlite;
pub mod sse;
pub mod str;
pub mod string_builder;
pub mod sys;
pub mod task;
pub mod time;

// Re-export commonly used types
pub use list::List;
pub use str::str_contains;
pub use string_builder::StringBuilder;

/// May<T> - AutoLang's optional type (alias for Option<T>)
pub type May<T> = Option<T>;

// Type aliases for May<T> with specific types
pub type MayInt = Option<i32>;
pub type MayUint = Option<u32>;
pub type MayFloat = Option<f64>;
pub type MayDouble = Option<f64>;
pub type MayChar = Option<char>;
pub type MayBool = Option<bool>;
pub type MayStr = Option<String>;

/// Nil - AutoLang's nil value type marker
pub struct Nil;

/// Create a Nil value (None)
pub fn nil<T>() -> Option<T> {
    None
}

/// Sleep for the specified number of milliseconds
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

/// Simple hash function (transpiler compatibility)
pub fn simple_hash(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Value length — alias for json::len for transpiler compatibility
pub fn value_len(val: &serde_json::Value) -> usize {
    json::json_len(val)
}

// Re-export string functions at crate root for transpiler compatibility
pub use str::str_ends_with;
pub use str::str_find;
pub use str::str_find_from;
pub use str::str_split;
pub use str::str_starts_with;
pub use str::str_substr;
pub use str::str_to_lower;
pub use str::str_to_upper;
pub use str::str_trim;

/// Generate a UUID string
pub fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos: u64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    // Simple UUID v4-like: 8-4-4-4-12 hex format using nanos + LCG
    let r1: u64 = nanos;
    let r2: u64 = nanos
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let r3: u64 = nanos
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
        .rotate_left(17);
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:08x}{:04x}",
        (r1 >> 32) as u32,
        (r1 >> 16) as u16 & 0xFFFF,
        ((r2 >> 16) as u16 & 0x0FFF) | 0x4000, // version 4
        ((r2 >> 0) as u16 & 0x3FFF) | 0x8000,  // variant 1
        (r3 >> 32) as u32,
        r3 as u16,
    )
}
