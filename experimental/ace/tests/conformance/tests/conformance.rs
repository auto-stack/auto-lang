pub use auto_lang_core::*;
#[cfg(feature = "test-case-batch")]
mod batch;
mod console;
#[path = "host/mod.rs"]
mod tests;

mod governance;
mod native_lifecycle;
