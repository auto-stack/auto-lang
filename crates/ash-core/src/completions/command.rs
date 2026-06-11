//! Command name completion
//!
//! Provides completion for built-in shell commands using CompletionSignature
//! metadata from the CommandRegistry. Falls back to a minimal hardcoded list
//! when no signatures are available (e.g., in tests without a registry).

use crate::completions::types::CompletionSignature;
use crate::completions::{Completion, CompletionKind};

/// Minimal fallback list when no CompletionSignatures are available.
const FALLBACK_COMMANDS: &[&str] = &[
    "ls", "cd", "pwd", "mkdir", "rm", "mv", "cp",
    "sort", "uniq", "head", "tail", "wc", "grep",
    "echo", "help", "exit",
];

/// Complete command names from registry signatures.
///
/// When `signatures` is non-empty, uses the full registry data (77+ commands
/// with descriptions). Otherwise falls back to a minimal hardcoded list.
pub fn complete_command(input: &str, signatures: &[CompletionSignature]) -> Vec<Completion> {
    let trimmed = input.trim();

    // Determine prefix: after pipe, use the text after the pipe
    let prefix = if let Some(pipe_idx) = trimmed.rfind('|') {
        trimmed[pipe_idx + 1..].trim()
    } else {
        trimmed
    };

    if !signatures.is_empty() {
        complete_from_signatures(prefix, signatures)
    } else {
        complete_from_fallback(prefix)
    }
}

fn complete_from_signatures(prefix: &str, signatures: &[CompletionSignature]) -> Vec<Completion> {
    let mut completions = Vec::new();

    for sig in signatures {
        if sig.name.starts_with(prefix) || prefix.is_empty() {
            completions.push(Completion::with_description(
                sig.name.clone(),
                sig.name.clone(),
                sig.description.clone(),
                CompletionKind::Command,
            ));
        }
    }

    completions
}

fn complete_from_fallback(prefix: &str) -> Vec<Completion> {
    let mut completions = Vec::new();

    for &cmd in FALLBACK_COMMANDS {
        if cmd.starts_with(prefix) || prefix.is_empty() {
            completions.push(Completion::with_kind(cmd, cmd, CompletionKind::Command));
        }
    }

    completions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completions::types::CompletionArgument;

    fn test_signatures() -> Vec<CompletionSignature> {
        vec![
            CompletionSignature {
                name: "ls".into(),
                description: "List directory contents".into(),
                arguments: vec![CompletionArgument {
                    name: "all".into(),
                    description: "Show all files".into(),
                    required: false,
                    is_flag: true,
                    short: Some('a'),
                }],
            },
            CompletionSignature {
                name: "grep".into(),
                description: "Search for patterns".into(),
                arguments: vec![],
            },
            CompletionSignature {
                name: "cd".into(),
                description: "Change directory".into(),
                arguments: vec![],
            },
        ]
    }

    #[test]
    fn test_complete_command_empty() {
        let sigs = test_signatures();
        let completions = complete_command("", &sigs);
        assert_eq!(completions.len(), 3);
    }

    #[test]
    fn test_complete_command_partial() {
        let sigs = test_signatures();
        let completions = complete_command("l", &sigs);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].display, "ls");
        assert_eq!(completions[0].description.as_deref(), Some("List directory contents"));
    }

    #[test]
    fn test_complete_command_no_match() {
        let sigs = test_signatures();
        let completions = complete_command("xyz", &sigs);
        assert!(completions.is_empty());
    }

    #[test]
    fn test_complete_command_after_pipe() {
        let sigs = test_signatures();
        let completions = complete_command("echo test | gr", &sigs);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "grep"));
    }

    #[test]
    fn test_fallback_no_signatures() {
        let completions = complete_command("ls", &[]);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "ls"));
    }
}
