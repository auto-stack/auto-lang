use crate::cmd::{Argument, Signature};
use miette::Result;
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
    // Map of short aliases to flag names
    let mut short_aliases: HashMap<char, String> = HashMap::new();

    for arg in &signature.arguments {
        if arg.is_flag {
            valid_flags.insert(arg.name.clone(), arg);
            if let Some(short) = arg.short {
                short_aliases.insert(short, arg.name.clone());
            }
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
            // Short flag(s)
            let flag_short = arg_str.trim_start_matches('-');

            // Handle combined short flags like -al, -ltr
            if flag_short.len() > 1 {
                // Split into individual flags
                let chars: Vec<char> = flag_short.chars().collect();
                for ch in chars {
                    // Look up each character in short_aliases
                    if let Some(name) = short_aliases.get(&ch) {
                        parsed.flags.insert(name.clone(), true);
                    } else {
                        return Err(miette::miette!("Unknown flag: -{}", ch));
                    }
                }
            } else {
                // Single character flag
                let ch = flag_short.chars().next().unwrap();
                let flag_name = short_aliases.get(&ch).cloned();

                if let Some(name) = flag_name {
                    parsed.flags.insert(name, true);
                } else {
                    return Err(miette::miette!("Unknown flag: -{}", ch));
                }
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
