//! Command execution module
//!
//! Handles execution of built-in commands, external commands, and Auto functions.
//!
//! Pure logic submodules (data, value_helpers, external) live in `core::cmd`.
//! Terminal-dependent submodules (builtin, fs, pipeline, etc.) stay here.

use miette::Result;
use std::path::Path;

// Re-export core cmd submodules for backward compatibility
pub use crate::core::cmd::data;
pub use crate::core::cmd::external;
pub use crate::core::cmd::value_helpers;

// Frontend-only submodules (have terminal deps or complex cross-deps)
pub mod auto;
pub mod builtin;
pub mod commands;
pub mod fs;
pub mod parser;
pub mod pipeline;
pub mod pipeline_convert;
pub mod pipeline_data;
pub mod registry;

pub use pipeline::execute_pipeline;
pub use pipeline_data::PipelineData;
pub use registry::CommandRegistry;

use crate::shell::Shell;

/// Argument type for command signatures
#[derive(Clone, Debug)]
pub struct Argument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub is_flag: bool,
    pub is_option: bool,  // Named option: --name VALUE (takes next arg as value)
    pub short: Option<char>,  // Short flag alias (e.g., 'a' for 'all')
    pub default: Option<String>, // Default value for optional positionals/options
}

// Bridge: auto-shell Signature → ash-core CompletionSignature
impl From<Signature> for crate::completions::CompletionSignature {
    fn from(sig: Signature) -> Self {
        Self {
            name: sig.name,
            description: sig.description,
            arguments: sig.arguments.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Argument> for crate::completions::CompletionArgument {
    fn from(arg: Argument) -> Self {
        Self {
            name: arg.name,
            description: arg.description,
            required: arg.required,
            is_flag: arg.is_flag,
            short: arg.short,
        }
    }
}

/// Command signature for help generation and validation
#[derive(Clone, Debug)]
pub struct Signature {
    pub name: String,
    pub description: String,
    pub arguments: Vec<Argument>,
    /// Extra text shown after the argument list in --help
    pub extra_help: Option<String>,
}

impl Signature {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            arguments: Vec::new(),
            extra_help: None,
        }
    }

    /// Add a required positional argument.
    pub fn required(mut self, name: &str, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: true,
            is_flag: false,
            is_option: false,
            short: None,
            default: None,
        });
        self
    }

    /// Add an optional positional argument.
    pub fn optional(mut self, name: &str, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            is_flag: false,
            is_option: false,
            short: None,
            default: None,
        });
        self
    }

    /// Add an optional positional argument with a default value.
    pub fn optional_default(mut self, name: &str, default: &str, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            is_flag: false,
            is_option: false,
            short: None,
            default: Some(default.to_string()),
        });
        self
    }

    /// Add a boolean flag (--name).
    pub fn flag(mut self, name: &str, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            is_flag: true,
            is_option: false,
            short: None,
            default: None,
        });
        self
    }

    /// Add a boolean flag with a short alias (-s, --short).
    pub fn flag_with_short(mut self, name: &str, short: char, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            is_flag: true,
            is_option: false,
            short: Some(short),
            default: None,
        });
        self
    }

    /// Add a named option that takes a value (--name VALUE).
    pub fn option(mut self, name: &str, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            is_flag: false,
            is_option: true,
            short: None,
            default: None,
        });
        self
    }

    /// Add a named option with a short alias (-n VALUE, --name VALUE).
    pub fn option_with_short(mut self, name: &str, short: char, description: &str) -> Self {
        self.arguments.push(Argument {
            name: name.to_string(),
            description: description.to_string(),
            required: false,
            is_flag: false,
            is_option: true,
            short: Some(short),
            default: None,
        });
        self
    }

    /// Add extra help text shown at the bottom of --help output.
    pub fn extra_help(mut self, text: &str) -> Self {
        self.extra_help = Some(text.to_string());
        self
    }

    /// Generate a --help text string from this signature.
    pub fn format_help(&self) -> String {
        let mut help = String::new();

        // Usage line
        help.push_str(&format!("{} — {}\n\n", self.name, self.description));
        help.push_str("USAGE:\n");
        help.push_str(&format!("  {}", self.name));

        for arg in &self.arguments {
            if arg.is_flag {
                // Flags shown in options section
            } else if arg.is_option {
                help.push_str(&format!(" [--{} <{}>]", arg.name, arg.name));
            } else if arg.required {
                help.push_str(&format!(" <{}>", arg.name));
            } else {
                help.push_str(&format!(" [{}]", arg.name));
            }
        }
        help.push('\n');

        // Arguments section
        let has_args = self.arguments.iter().any(|a| !a.is_flag && !a.is_option);
        if has_args {
            help.push_str("\nARGS:\n");
            for arg in &self.arguments {
                if !arg.is_flag && !arg.is_option {
                    let default = arg.default.as_ref().map(|d| format!(" [default: {}]", d)).unwrap_or_default();
                    help.push_str(&format!("  <{}>  {}{}\n", arg.name, arg.description, default));
                }
            }
        }

        // Options section
        let has_opts = self.arguments.iter().any(|a| a.is_flag || a.is_option);
        if has_opts {
            help.push_str("\nOPTIONS:\n");
            for arg in &self.arguments {
                if arg.is_flag {
                    let short = arg.short.map(|s| format!("-{}, ", s)).unwrap_or_default();
                    help.push_str(&format!("  {}--{}  {}\n", short, arg.name, arg.description));
                } else if arg.is_option {
                    let short = arg.short.map(|s| format!("-{}, ", s)).unwrap_or_default();
                    help.push_str(&format!("  {}--{} <{}>  {}\n", short, arg.name, arg.name, arg.description));
                }
            }
            help.push_str("  --help  Show this help message\n");
        }

        // Extra help
        if let Some(ref extra) = self.extra_help {
            help.push_str(&format!("\n{}\n", extra));
        }

        help
    }
}

/// Trait that all shell commands must implement
pub trait Command {
    /// Get the command name
    fn name(&self) -> &str;

    /// Get the command signature
    fn signature(&self) -> Signature;

    /// Execute the command (legacy path)
    ///
    /// Commands now receive PipelineData (structured Value or text) and return PipelineData.
    /// This enables zero-copy structured data pipelines between commands.
    fn run(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: PipelineData,
        shell: &mut Shell,
    ) -> Result<PipelineData>;

    /// Execute the command with typed AtomPipeline data
    ///
    /// Default implementation delegates to `run()` via the bridge layer.
    /// Commands should override this to produce typed Atoms directly.
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: ash_core::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> Result<ash_core::pipeline::AtomPipeline> {
        let legacy_in = pipeline_convert::atom_to_pipeline_data(input);
        let legacy_out = self.run(args, legacy_in, shell)?;
        Ok(pipeline_convert::pipeline_data_to_atom(legacy_out))
    }
}

/// Execute a command (built-in or external)
pub fn execute_command(input: &str, current_dir: &Path) -> Result<Option<String>> {
    let input = input.trim();

    // Check for built-in commands
    if let Some(output) = builtin::execute_builtin(input, current_dir)? {
        return Ok(Some(output));
    }

    // Otherwise, execute as external command
    external::execute_external(input, current_dir, false)
}
