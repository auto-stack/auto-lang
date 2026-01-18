//! System operation functions for AutoLang
//!
//! Provides built-in functions for system-level operations.

use auto_val::{Args, Value};

/// Get the process ID
///
/// # Arguments
/// * `args` - Expected: (no arguments)
///
/// # Example
/// ```auto
/// let pid = getpid()
/// ```
pub fn sys_getpid(_args: &Args) -> Value {
    use std::process;

    // Return the process ID as an integer
    Value::Int(process::id() as i32)
}
