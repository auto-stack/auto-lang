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
    /// Named options (key-value pairs)
    pub named: HashMap<String, String>,
    /// Whether --help was requested
    pub help_requested: bool,
}

impl ParsedArgs {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a flag is set
    pub fn has_flag(&self, name: &str) -> bool {
        *self.flags.get(name).unwrap_or(&false)
    }

    /// Get a named option value
    pub fn get_option(&self, name: &str) -> Option<&String> {
        self.named.get(name)
    }

    /// Get a named option value or default
    pub fn option_or<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        self.named.get(name).map(|s| s.as_str()).unwrap_or(default)
    }

    /// Get a positional argument by index
    pub fn get_positional(&self, index: usize) -> Option<&String> {
        self.positionals.get(index)
    }

    /// Get positional arg as str, with fallback default
    pub fn positional_or<'a>(&'a self, index: usize, default: &'a str) -> &'a str {
        self.positionals.get(index).map(|s| s.as_str()).unwrap_or(default)
    }

    /// Get the first positional arg as str
    pub fn first(&self) -> Option<&str> {
        self.positionals.first().map(|s| s.as_str())
    }

    /// Get the second positional arg as str
    pub fn second(&self) -> Option<&str> {
        self.positionals.get(1).map(|s| s.as_str())
    }

    /// Number of positional arguments
    pub fn positional_count(&self) -> usize {
        self.positionals.len()
    }
}

/// Parse raw string arguments according to a command signature.
///
/// Handles:
/// - `--flag` / `-f` boolean flags
/// - `--option VALUE` / `-o VALUE` named options
/// - `--option=VALUE` equals syntax
/// - `--help` auto-help
/// - Positional arguments (everything else)
pub fn parse_args(signature: &Signature, raw_args: &[String]) -> Result<ParsedArgs> {
    let mut parsed = ParsedArgs::new();
    let mut positionals = Vec::new();

    // Map of valid flags/options for quick lookup
    let mut valid_flags: HashMap<String, &Argument> = HashMap::new();
    let mut valid_options: HashMap<String, &Argument> = HashMap::new();
    let mut short_aliases: HashMap<char, String> = HashMap::new();

    for arg in &signature.arguments {
        if arg.is_flag {
            valid_flags.insert(arg.name.clone(), arg);
            if let Some(short) = arg.short {
                short_aliases.insert(short, arg.name.clone());
            }
        } else if arg.is_option {
            valid_options.insert(arg.name.clone(), arg);
            if let Some(short) = arg.short {
                short_aliases.insert(short, arg.name.clone());
            }
        }
    }

    let mut arg_iter = raw_args.iter();
    while let Some(arg_str) = arg_iter.next() {
        if arg_str == "--help" || arg_str == "-h" {
            parsed.help_requested = true;
            continue;
        }

        if arg_str.starts_with("--") {
            let after_dashes = arg_str.trim_start_matches("--");

            // Check for --option=VALUE syntax
            if let Some(eq_pos) = after_dashes.find('=') {
                let opt_name = &after_dashes[..eq_pos];
                let opt_value = &after_dashes[eq_pos + 1..];
                if valid_options.contains_key(opt_name) {
                    parsed.named.insert(opt_name.to_string(), opt_value.to_string());
                } else {
                    return Err(miette::miette!("Unknown option: --{}", opt_name));
                }
            } else if valid_flags.contains_key(after_dashes) {
                parsed.flags.insert(after_dashes.to_string(), true);
            } else if valid_options.contains_key(after_dashes) {
                // Next arg is the value
                if let Some(value) = arg_iter.next() {
                    parsed.named.insert(after_dashes.to_string(), value.clone());
                } else {
                    return Err(miette::miette!("Option --{} requires a value", after_dashes));
                }
            } else {
                return Err(miette::miette!("Unknown flag: --{}", after_dashes));
            }
        } else if arg_str.starts_with('-') && arg_str.len() > 1 {
            let flag_short = arg_str.trim_start_matches('-');

            // Handle combined short flags like -al
            let chars: Vec<char> = flag_short.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                let ch = chars[i];

                if let Some(name) = short_aliases.get(&ch) {
                    if valid_flags.contains_key(name) {
                        parsed.flags.insert(name.clone(), true);
                    } else if valid_options.contains_key(name) {
                        // Option: next token or rest of combined string is the value
                        let rest_of_combined: String = chars[i + 1..].iter().collect();
                        if !rest_of_combined.is_empty() {
                            parsed.named.insert(name.clone(), rest_of_combined);
                            break;
                        } else if let Some(value) = arg_iter.next() {
                            parsed.named.insert(name.clone(), value.clone());
                        } else {
                            return Err(miette::miette!("Option -{} requires a value", ch));
                        }
                        break;
                    }
                } else {
                    return Err(miette::miette!("Unknown flag: -{}", ch));
                }
                i += 1;
            }
        } else {
            positionals.push(arg_str.clone());
        }
    }

    // Apply defaults for options not provided
    for arg in &signature.arguments {
        if arg.is_option && !parsed.named.contains_key(&arg.name) {
            if let Some(ref default) = arg.default {
                parsed.named.insert(arg.name.clone(), default.clone());
            }
        }
        if !arg.is_flag && !arg.is_option && !arg.required {
            // Apply default for optional positionals not provided
        }
    }

    // Validate required positionals
    let required_positionals: Vec<&Argument> = signature
        .arguments
        .iter()
        .filter(|a| !a.is_flag && !a.is_option && a.required)
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
