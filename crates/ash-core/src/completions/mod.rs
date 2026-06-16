//! Core completion logic - pure logic, zero terminal dependencies
//!
//! File path completion, command name completion, and Auto variable completion.
//! These have no dependency on reedline or any terminal library.

pub mod auto;
pub mod command;
pub mod file;
pub mod flag;
pub mod help_parser;
pub mod provider;
pub mod spec;
pub mod spec_format;
pub mod types;

pub use provider::{CompletionContext, CompletionProvider};
pub use spec::{
    ArgSpec, CompletionSource, CompletionSpec, FlagSpec, ParseMode, SubcommandSpec, WhenCondition,
};
pub use types::{CompletionArgument, CompletionSignature};

use crate::bookmarks::BookmarkManager;
use types::CompletionSignature as CompSig;

/// Completion kind — determines color and icon in the menu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// Built-in shell command (ls, cd, grep)
    Command,
    /// External command (git, cargo)
    External,
    /// File path
    File,
    /// Directory path
    Directory,
    /// Environment variable ($PATH)
    Variable,
    /// Flag argument (--verbose)
    Flag,
    /// Subcommand (cargo build)
    Subcommand,
    /// AI-suggested completion (future)
    AiSuggested,
}

impl Default for CompletionKind {
    fn default() -> Self {
        Self::Command
    }
}

/// Completion suggestion
#[derive(Debug, Clone, PartialEq)]
pub struct Completion {
    /// Display text shown in the menu
    pub display: String,
    /// Text to insert as replacement
    pub replacement: String,
    /// Optional description (shown in descriptive list mode)
    pub description: Option<String>,
    /// Completion kind (determines color/icon)
    pub kind: CompletionKind,
    /// Whether this is an exact prefix match (vs fuzzy match).
    /// Used to control partial completion behavior.
    pub is_prefix_match: bool,
}

impl Completion {
    /// Create a simple completion with display and replacement
    pub fn new(display: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            display: display.into(),
            replacement: replacement.into(),
            description: None,
            kind: CompletionKind::Command,
            is_prefix_match: true,
        }
    }

    /// Create a completion with kind
    pub fn with_kind(
        display: impl Into<String>,
        replacement: impl Into<String>,
        kind: CompletionKind,
    ) -> Self {
        Self {
            display: display.into(),
            replacement: replacement.into(),
            description: None,
            kind,
            is_prefix_match: true,
        }
    }

    /// Create a completion with description
    pub fn with_description(
        display: impl Into<String>,
        replacement: impl Into<String>,
        description: impl Into<String>,
        kind: CompletionKind,
    ) -> Self {
        Self {
            display: display.into(),
            replacement: replacement.into(),
            description: Some(description.into()),
            kind,
            is_prefix_match: true,
        }
    }

    /// Mark this completion as a fuzzy (non-prefix) match
    pub fn as_fuzzy(mut self) -> Self {
        self.is_prefix_match = false;
        self
    }
}

/// Get completions for the current input (legacy, no registry).
///
/// Delegates to `get_completions_with_context` with an empty signature list,
/// which falls back to hardcoded command names.
pub fn get_completions(input: &str) -> Vec<Completion> {
    get_completions_with_context(input, &[])
}

/// Get completions for the current input, using registry signatures.
///
/// This function intelligently determines which completion type to use
/// based on the input context:
/// - Command names at the start of line or after |
/// - Flag names when typing `-` or `--` after a known command
/// - File paths after command names
/// - Shell variables after $
pub fn get_completions_with_context(
    input: &str,
    signatures: &[CompSig],
) -> Vec<Completion> {
    // Check if input ends with whitespace (user wants file completion after command)
    let ends_with_space = input.ends_with(|c: char| c.is_whitespace());

    let trimmed = input.trim();

    // Empty input: complete all commands
    if trimmed.is_empty() {
        return command::complete_command(trimmed, signatures);
    }

    // Check if we're after a pipe
    if let Some(pipe_idx) = trimmed.rfind('|') {
        // Get the part after the last pipe
        let after_pipe = trimmed[pipe_idx + 1..].trim();

        // If nothing after pipe or just starting a command, complete commands
        if after_pipe.is_empty() || (!after_pipe.contains(' ') && !ends_with_space) {
            return command::complete_command(after_pipe, signatures);
        }
    }

    // Variable completion: input contains $
    if trimmed.contains('$') {
        let var_completions = auto::complete_auto(trimmed);
        if !var_completions.is_empty() {
            return var_completions;
        }
    }

    // Check if we should complete files or commands
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    // If input ends with space or has multiple words, do file/flag completion
    if ends_with_space || parts.len() > 1 {
        // Special handling for 'b' command
        if parts[0] == "b" {
            let is_arg1 =
                (parts.len() == 1 && ends_with_space) || (parts.len() == 2 && !ends_with_space);
            let is_del_arg = parts.len() >= 2
                && parts[1] == "del"
                && ((parts.len() == 2 && ends_with_space)
                    || (parts.len() == 3 && !ends_with_space));

            if is_arg1 {
                let prefix = if parts.len() == 2 { parts[1] } else { "" };
                let mut comps = Vec::new();

                // Subcommands
                for sub in ["add", "del", "list"] {
                    if sub.starts_with(prefix) {
                        comps.push(Completion::with_kind(sub, sub, CompletionKind::Subcommand));
                    }
                }

                // Bookmarks
                let manager = BookmarkManager::new();
                for (name, _) in manager.list() {
                    if name.starts_with(prefix) {
                        comps.push(Completion::with_kind(
                            name.clone(),
                            name.clone(),
                            CompletionKind::Command,
                        ));
                    }
                }
                return comps;
            }

            if is_del_arg {
                let prefix = if parts.len() == 3 { parts[2] } else { "" };
                let manager = BookmarkManager::new();
                let mut comps = Vec::new();
                for (name, _) in manager.list() {
                    if name.starts_with(prefix) {
                        comps.push(Completion::with_kind(
                            name.clone(),
                            name.clone(),
                            CompletionKind::Command,
                        ));
                    }
                }
                return comps;
            }
        }

        // Multiple words: check if last word starts with -
        if let Some(last) = parts.last() {
            if last.starts_with('-') {
                // Flag completion — collect already-set flags from the line
                let already_set = collect_flags(&parts);
                return flag::complete_flags(last, parts[0], signatures, &already_set);
            }
        }
        // Complete file paths - pass original input, not trimmed!
        return file::complete_file(input);
    }

    // First word: complete commands
    if parts.len() == 1 {
        let cmd_completions = command::complete_command(trimmed, signatures);
        if !cmd_completions.is_empty() {
            return cmd_completions;
        }

        // If no command matches, try file completion
        return file::complete_file(input);
    }

    // Fallback to file completion
    file::complete_file(input)
}

/// Collect flag tokens already present on the command line.
fn collect_flags(parts: &[&str]) -> Vec<String> {
    let mut flags = Vec::new();
    for &part in &parts[1..] {
        if part.starts_with("--") {
            // Long flag: extract name after --
            let name = part.trim_start_matches('-').split('=').next().unwrap_or("");
            if !name.is_empty() {
                flags.push(name.to_string());
            }
        } else if part.starts_with('-') && part.len() > 1 && !part[1..].chars().all(|c| c.is_ascii_digit()) {
            // Short flag(s): -a, -al, etc. Extract each letter
            for ch in part[1..].chars() {
                flags.push(ch.to_string());
            }
        }
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_empty() {
        let completions = get_completions_with_context("", &[]);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "ls"));
    }

    #[test]
    fn test_complete_command() {
        let completions = get_completions_with_context("l", &[]);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "ls"));
    }

    #[test]
    fn test_complete_file_after_command() {
        let completions = get_completions_with_context("ls src", &[]);
        let _ = completions;
    }

    #[test]
    fn test_complete_file_partial() {
        let completions = get_completions_with_context("ls s", &[]);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "src/"));
    }

    #[test]
    fn test_complete_file_after_command_with_space() {
        let completions = get_completions_with_context("ls ", &[]);
        let _ = completions;
    }

    #[test]
    fn test_complete_variable() {
        let completions = get_completions_with_context("echo $P", &[]);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "PATH"));
    }

    #[test]
    fn test_complete_after_pipe() {
        let completions = get_completions_with_context("ls | gr", &[]);
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "grep"));
    }

    #[test]
    fn test_complete_no_match() {
        let completions = get_completions_with_context("nonexistent_command xyz", &[]);
        let _ = completions;
    }

    #[test]
    fn test_complete_flags_with_context() {
        use types::{CompletionArgument, CompletionSignature};

        let sigs = vec![CompletionSignature {
            name: "ls".into(),
            description: "List directory contents".into(),
            arguments: vec![
                CompletionArgument {
                    name: "all".into(),
                    description: "Show all files".into(),
                    required: false,
                    is_flag: true,
                    short: Some('a'),
                },
                CompletionArgument {
                    name: "long".into(),
                    description: "Long listing".into(),
                    required: false,
                    is_flag: true,
                    short: Some('l'),
                },
            ],
        }];

        // ls --a<TAB> should suggest --all
        let completions = get_completions_with_context("ls --a", &sigs);
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].replacement, "--all");
        assert_eq!(completions[0].kind, CompletionKind::Flag);
    }
}
