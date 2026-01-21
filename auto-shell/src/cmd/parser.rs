use crate::cmd::{Argument, Signature};
use miette::{IntoDiagnostic, Result};
use std::collections::HashMap;

/// Parsed arguments ready for command consumption
#[derive(Debug, Clone, Default)]
pub struct ParsedArgs {
    /// Positional arguments
    pub positionals: Vec<String>,
    /// Flags (boolean options)
    pub flags: HashMap<String, bool>,
    /// Named options (key-value pairs) - placeholder for future
    pub named: HashMap<String, String>,
}

impl ParsedArgs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a flag is set
    pub fn has_flag(&self, name: &str) -> bool {
        *self.flags.get(name).unwrap_or(&false)
    }

    /// Get a positional argument by index
    pub fn get_positional(&self, index: usize) -> Option<&String> {
        self.positionals.get(index)
    }
}

/// Parse raw string arguments according to a command signature
pub fn parse_args(signature: &Signature, raw_args: &[String]) -> Result<ParsedArgs> {
    let mut parsed = ParsedArgs::new();
    let mut positionals = Vec::new();

    // Map of valid flags for quick lookup
    // Key: flag name, Value: Argument definition
    let mut valid_flags: HashMap<String, &Argument> = HashMap::new();

    for arg in &signature.arguments {
        if arg.is_flag {
            valid_flags.insert(arg.name.clone(), arg);
        }
    }

    let mut arg_iter = raw_args.iter();
    while let Some(arg_str) = arg_iter.next() {
        if arg_str.starts_with("--") {
            // Long flag
            let flag_name = arg_str.trim_start_matches("--");
            if valid_flags.contains_key(flag_name) {
                parsed.flags.insert(flag_name.to_string(), true);
            } else {
                return Err(miette::miette!("Unknown flag: --{}", flag_name));
            }
        } else if arg_str.starts_with('-') && arg_str.len() > 1 {
            // Short flag (currently treat same as long for simplicity or need aliasing?)
            // For now, let's assume we match against name directly or we need short alias support in Signature.
            // Nushell style often uses full names. Let's support -f if name is "f" or "force".
            // Implementation Plan didn't specify short aliases, so implementing basic exact match.
            // If name is "force", --force works.
            // If we want -f, we need short names in Signature.
            // For this iteration, let's treat -name same as --name for simplicity unless we add short alias field.
            let flag_name = arg_str.trim_start_matches('-');
            if valid_flags.contains_key(flag_name) {
                parsed.flags.insert(flag_name.to_string(), true);
            } else {
                return Err(miette::miette!("Unknown flag: -{}", flag_name));
            }
        } else {
            // Positional
            positionals.push(arg_str.clone());
        }
    }

    // Validate positionals
    // Count required positionals
    let required_positionals: Vec<&Argument> = signature
        .arguments
        .iter()
        .filter(|a| !a.is_flag && a.required)
        .collect();

    if positionals.len() < required_positionals.len() {
        let missing = &required_positionals[positionals.len()];
        return Err(miette::miette!(
            "Missing required argument: {}",
            missing.name
        ));
    }

    parsed.positionals = positionals;
    Ok(parsed)
}
