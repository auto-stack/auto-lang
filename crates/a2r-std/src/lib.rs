//! Auto Language Standard Library for a2r (Auto-to-Rust Transpiler)
//!
//! This crate provides Rust implementations of AutoLang's standard library
//! modules so that transpiled code can compile and run.
//!
//! Generated from: auto-lang/stdlib/auto/*.at + *.rs.at

pub mod math;
pub mod str;
pub mod time;
pub mod env;
pub mod json;
pub mod fs;
pub mod list;
pub mod hashmap;
pub mod http;

// Re-export commonly used types
pub use list::List;

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
pub use str::str_find;
pub use str::str_find_from;
pub use str::str_substr;
pub use str::str_ends_with;
pub use str::str_starts_with;
pub use str::str_split;
pub use str::str_trim;
pub use str::str_to_lower;
pub use str::str_to_upper;

/// Generate a UUID string
pub fn uuid() -> String {
    format!("{:x}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos())
}
