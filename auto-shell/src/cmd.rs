//! Command execution module
//!
//! Handles execution of built-in commands, external commands, and Auto functions.

use miette::Result;
use std::path::Path;

pub mod auto;
pub mod builtin;
pub mod data;
pub mod external;
pub mod fs;
pub mod pipeline;

pub use pipeline::execute_pipeline;

/// Execute a command (built-in or external)
pub fn execute_command(input: &str, current_dir: &Path) -> Result<Option<String>> {
    let input = input.trim();

    // Check for built-in commands
    if let Some(output) = builtin::execute_builtin(input, current_dir)? {
        return Ok(Some(output));
    }

    // Otherwise, execute as external command
    external::execute_external(input, current_dir)
}
