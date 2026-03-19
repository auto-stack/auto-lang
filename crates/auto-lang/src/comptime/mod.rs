//! Compile-Time Execution Engine (CTEE) - Plan 095
//!
//! This module implements the compile-time execution system for AutoLang.
//! It provides AST transformation for:
//! - `#if` - Conditional compilation
//! - `#for` - Loop unrolling at compile time
//! - `#is` - Type pattern matching at compile time
//! - `#{}` - Compile-time code block execution
//!
//! # Architecture
//!
//! The CTEE uses the existing `VmInterpreter` for expression evaluation, rather than
//! implementing a separate evaluator. This approach:
//! - Reuses the mature, tested VM infrastructure
//! - Supports all language features automatically
//! - Distinguishes compile-time vs runtime via `comptime_mode` flag
//!
//! # Example
//!
//! ```auto
//! #if OS == "windows" {
//!     fn init() { init_win32() }
//! } else {
//!     fn init() { init_linux() }
//! }
//! ```

pub mod transformer;

pub use transformer::*;
