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
