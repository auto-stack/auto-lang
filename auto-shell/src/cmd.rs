//! Command execution module
//!
//! Handles execution of built-in commands, external commands, and Auto functions.

use miette::Result;
use std::path::Path;

pub mod auto;
pub mod builtin;
pub mod commands;
pub mod data;
pub mod external;
pub mod fs;
pub mod parser;
pub mod pipeline;
pub mod registry;

pub use pipeline::execute_pipeline;
pub use registry::CommandRegistry;

use crate::shell::Shell;

/// Argument type for command signatures
#[derive(Clone, Debug)]
pub struct Argument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub is_flag: bool,
}

/// Command signature for help generation and validation
#[derive(Clone, Debug)]
pub struct Signature {
    pub name: String,
    pub description: String,
    pub arguments: Vec<Argument>,
}

impl Signature {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            arguments: Vec::new(),
        }
    }

    pub fn required(mut self, name: &str, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: true,
            is_flag: false,
        });
        self
    }

    pub fn optional(mut self, name: &str, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            is_flag: false,
        });
        self
    }

    pub fn flag(mut self, name: &str, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            is_flag: true,
        });
        self
    }
}

/// Trait that all shell commands must implement
pub trait Command {
    /// Get the command name
    fn name(&self) -> &str;

    /// Get the command signature
    fn signature(&self) -> Signature;

    /// Execute the command
    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: Option<&str>,
        shell: &mut Shell,
    ) -> Result<Option<String>>;
}

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
