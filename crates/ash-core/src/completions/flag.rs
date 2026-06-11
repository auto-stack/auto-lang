//! Flag completion from CompletionSignature
//!
//! Provides `--long-flag` and `-s` (short flag) completion for built-in
//! commands whose Signature is available via the CommandRegistry.

use crate::completions::types::CompletionSignature;
use crate::completions::{Completion, CompletionKind};

/// Complete flag names for a known command.
///
/// `prefix` — the text the user has typed so far (e.g. `--a` or `-l`).
/// `command_name` — the command whose flags we are completing.
/// `signatures` — all registered command signatures.
/// `already_set` — flags already present on the command line (to avoid duplicates).
pub fn complete_flags(
    prefix: &str,
    command_name: &str,
    signatures: &[CompletionSignature],
    already_set: &[String],
) -> Vec<Completion> {
    let Some(sig) = signatures.iter().find(|s| s.name == command_name) else {
        return Vec::new();
    };

    let mut completions = Vec::new();
    let is_long = prefix.starts_with("--");
    let is_short = !is_long && prefix.starts_with('-');

    for arg in &sig.arguments {
        if !arg.is_flag {
            continue;
        }

        if is_long {
            // Suggest --long-flag
            let long = &arg.name;
            let long_flag = format!("--{}", long);
            if long_flag.starts_with(prefix)
                && !already_set.contains(&long_flag)
                && !already_set.contains(long)
            {
                completions.push(Completion::with_description(
                    long_flag.clone(),
                    long_flag,
                    &arg.description,
                    CompletionKind::Flag,
                ));
            }
        } else if is_short {
            // Suggest -s (short flag)
            if let Some(short) = arg.short {
                let short_flag = format!("-{}", short);
                if short_flag.starts_with(prefix)
                    && !already_set.contains(&short_flag)
                    && !already_set.contains(&short.to_string())
                {
                    completions.push(Completion::with_description(
                        short_flag.clone(),
                        short_flag,
                        &arg.description,
                        CompletionKind::Flag,
                    ));
                }
            }
            // Also suggest --long when user typed `-`
            if prefix == "-" {
                let long = &arg.name;
                let long_flag = format!("--{}", long);
                if !already_set.contains(&long_flag)
                    && !already_set.contains(long)
                {
                    completions.push(Completion::with_description(
                        long_flag.clone(),
                        long_flag,
                        &arg.description,
                        CompletionKind::Flag,
                    ));
                }
            }
        }
    }

    completions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completions::types::CompletionArgument;

    fn ls_sig() -> CompletionSignature {
        CompletionSignature {
            name: "ls".into(),
            description: "List directory contents".into(),
            arguments: vec![
                CompletionArgument {
                    name: "all".into(),
                    description: "Show hidden files".into(),
                    required: false,
                    is_flag: true,
                    short: Some('a'),
                },
                CompletionArgument {
                    name: "long".into(),
                    description: "Long listing format".into(),
                    required: false,
                    is_flag: true,
                    short: Some('l'),
                },
                CompletionArgument {
                    name: "recursive".into(),
                    description: "List recursively".into(),
                    required: false,
                    is_flag: true,
                    short: Some('R'),
                },
                CompletionArgument {
                    name: "path".into(),
                    description: "Path to list".into(),
                    required: false,
                    is_flag: false,
                    short: None,
                },
            ],
        }
    }

    #[test]
    fn test_complete_long_flags() {
        let sigs = vec![ls_sig()];
        let result = complete_flags("--a", "ls", &sigs, &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].replacement, "--all");
        assert_eq!(result[0].kind, CompletionKind::Flag);
    }

    #[test]
    fn test_complete_short_flags() {
        let sigs = vec![ls_sig()];
        let result = complete_flags("-l", "ls", &sigs, &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].replacement, "-l");
    }

    #[test]
    fn test_complete_dash_shows_all() {
        let sigs = vec![ls_sig()];
        let result = complete_flags("-", "ls", &sigs, &[]);
        // Should include all 3 short flags + 3 long flags = 6
        assert!(result.len() >= 3);
        let values: Vec<&str> = result.iter().map(|c| c.replacement.as_str()).collect();
        assert!(values.contains(&"-a"));
        assert!(values.contains(&"-l"));
        assert!(values.contains(&"-R"));
    }

    #[test]
    fn test_exclude_already_set() {
        let sigs = vec![ls_sig()];
        let result = complete_flags("-", "ls", &sigs, &["--all".into()]);
        let values: Vec<&str> = result.iter().map(|c| c.replacement.as_str()).collect();
        assert!(!values.contains(&"--all"));
    }

    #[test]
    fn test_unknown_command() {
        let sigs = vec![ls_sig()];
        let result = complete_flags("--", "nonexistent", &sigs, &[]);
        assert!(result.is_empty());
    }
}
