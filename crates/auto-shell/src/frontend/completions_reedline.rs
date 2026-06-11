//! Reedline Completer integration
//!
//! Provides integration between auto-shell's completion system and reedline's Tab completion.
//! The completer holds:
//! - A snapshot of CommandRegistry signatures for built-in commands
//! - A CompletionProvider for external command specs (git, cargo, etc.)
//! - Shared state (current_dir) updated by the REPL after each command

use crate::completions::{Completion, CompletionSignature};
use ash_core::completions::{CompletionContext, CompletionProvider};
use reedline::{Completer, Suggestion};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Shared completion state, updated by REPL after each command.
#[derive(Debug)]
pub struct CompletionState {
    pub current_dir: PathBuf,
}

impl CompletionState {
    pub fn new(current_dir: PathBuf) -> Self {
        Self { current_dir }
    }
}

/// Reedline completer for auto-shell
pub struct ShellCompleter {
    signatures: Vec<CompletionSignature>,
    provider: CompletionProvider,
    state: Arc<Mutex<CompletionState>>,
}

impl ShellCompleter {
    pub fn new(
        signatures: Vec<CompletionSignature>,
        provider: CompletionProvider,
        state: Arc<Mutex<CompletionState>>,
    ) -> Self {
        Self {
            signatures,
            provider,
            state,
        }
    }

    /// Convert our Completion to reedline Suggestion
    fn completion_to_suggestion(completion: Completion) -> Suggestion {
        let value = completion.replacement.clone();
        let description = completion.display.clone();

        // Only show description if it differs from the value
        let description = if value == description {
            None
        } else {
            Some(description)
        };

        // Pass metadata via extra field:
        //   extra[0] = CompletionKind tag (for AshMenu coloring)
        //   extra[1] = "fuzzy" if non-prefix match
        let mut extra = Vec::new();
        extra.push(kind_tag(completion.kind));
        if !completion.is_prefix_match {
            extra.push("fuzzy".to_string());
        }

        Suggestion {
            value,
            description,
            extra: Some(extra),
            span: reedline::Span {
                start: 0,
                end: completion.replacement.len(),
            },
            append_whitespace: false,
            style: None,
            match_indices: None,
        }
    }

    /// Execute an external command and capture its stdout.
    /// Used as the command_executor closure for CompletionProvider.
    fn execute_command(cmd: &str, cwd: &Path) -> Result<String, String> {
        #[cfg(windows)]
        let output = std::process::Command::new("cmd")
            .args(["/C", cmd])
            .current_dir(cwd)
            .output()
            .map_err(|e| e.to_string())?;

        #[cfg(not(windows))]
        let output = std::process::Command::new("sh")
            .args(["-c", cmd])
            .current_dir(cwd)
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }
}

fn kind_tag(kind: crate::completions::CompletionKind) -> String {
    use crate::completions::CompletionKind;
    match kind {
        CompletionKind::Command => "command",
        CompletionKind::External => "external",
        CompletionKind::File => "file",
        CompletionKind::Directory => "directory",
        CompletionKind::Variable => "variable",
        CompletionKind::Flag => "flag",
        CompletionKind::Subcommand => "subcommand",
        CompletionKind::AiSuggested => "ai",
    }
    .to_string()
}

impl Completer for ShellCompleter {
    /// Complete the input line at the given position
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let trimmed = line[..pos].trim_end();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        // Calculate the span to replace: from the last word boundary to cursor position
        let start = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let end = pos;

        // If we have a first word and it's an external command with a spec,
        // route to the CompletionProvider
        if let Some(&cmd) = parts.first() {
            if self.provider.has_spec(cmd) {
                // Determine cursor part and prefix
                let ends_with_space = line[..pos].ends_with(|c: char| c.is_whitespace());
                let (cursor_part, prefix) = if ends_with_space {
                    // Cursor is after a space — completing a new token
                    (parts.len(), "")
                } else {
                    // Cursor is inside a token
                    let idx = parts.len().saturating_sub(1);
                    (idx, parts.last().copied().unwrap_or(""))
                };

                // Build parts with an empty slot for the cursor if needed
                let resolve_parts: Vec<&str> = if ends_with_space {
                    let mut p = parts.clone();
                    p.push("");
                    p
                } else {
                    parts.clone()
                };

                let current_dir = self
                    .state
                    .lock()
                    .map(|s| s.current_dir.clone())
                    .unwrap_or_else(|_| PathBuf::from("."));

                let ctx = CompletionContext {
                    current_dir: current_dir.clone(),
                    command_executor: Box::new(Self::execute_command),
                };

                let completions = self.provider.resolve(
                    &resolve_parts,
                    cursor_part,
                    prefix,
                    &ctx,
                );

                if !completions.is_empty() {
                    return completions
                        .into_iter()
                        .map(|comp| {
                            let mut suggestion = Self::completion_to_suggestion(comp);
                            suggestion.span = reedline::Span { start, end };
                            suggestion
                        })
                        .collect();
                }

                // Provider found the spec but returned nothing — fall through
                // to file completion below
            }
        }

        // Default: use built-in completion system (registry signatures + file/path completion)
        let completions = crate::completions::get_completions_with_context(
            line,
            &self.signatures,
        );

        completions
            .into_iter()
            .map(|comp| {
                let mut suggestion = Self::completion_to_suggestion(comp);
                suggestion.span = reedline::Span { start, end };
                suggestion
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_signatures() -> Vec<CompletionSignature> {
        use crate::completions::CompletionArgument;
        vec![
            CompletionSignature {
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
            },
            CompletionSignature {
                name: "grep".into(),
                description: "Search for patterns".into(),
                arguments: vec![],
            },
        ]
    }

    fn test_completer() -> ShellCompleter {
        ShellCompleter::new(
            test_signatures(),
            CompletionProvider::new(),
            Arc::new(Mutex::new(CompletionState::new(PathBuf::from(".")))),
        )
    }

    #[test]
    fn test_shell_completer_commands() {
        let mut completer = test_completer();
        let suggestions = completer.complete("l", 1);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "ls"));
    }

    #[test]
    fn test_shell_completer_flags() {
        let mut completer = test_completer();
        let suggestions = completer.complete("ls --", 5);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "--all"));
        assert!(suggestions.iter().any(|s| s.value == "--long"));
    }

    #[test]
    fn test_shell_completer_short_flags() {
        let mut completer = test_completer();
        let suggestions = completer.complete("ls -", 4);
        assert!(!suggestions.is_empty());
        // Should include both -a, -l and --all, --long
        assert!(suggestions.iter().any(|s| s.value == "-a"));
        assert!(suggestions.iter().any(|s| s.value == "-l"));
    }

    #[test]
    fn test_shell_completer_kind_tag_in_extra() {
        let mut completer = test_completer();
        let suggestions = completer.complete("ls --a", 6);
        let flag = suggestions.iter().find(|s| s.value == "--all").unwrap();
        assert_eq!(flag.extra.as_ref().unwrap()[0], "flag");
    }

    #[test]
    fn test_provider_routing_for_external_commands() {
        use ash_core::completions::{CompletionSpec, SubcommandSpec, FlagSpec as CoreFlagSpec};

        let mut provider = CompletionProvider::new();
        provider.register(
            CompletionSpec::new("git")
                .desc("Git version control")
                .subcommand(
                    SubcommandSpec::new("checkout")
                        .desc("Switch branches")
                        .flag(CoreFlagSpec::both("b", "branch").desc("Create new branch")),
                )
        );

        let mut completer = ShellCompleter::new(
            test_signatures(),
            provider,
            Arc::new(Mutex::new(CompletionState::new(PathBuf::from(".")))),
        );

        // "git " should show subcommands
        let suggestions = completer.complete("git ", 4);
        let names: Vec<&str> = suggestions.iter().map(|s| s.value.as_str()).collect();
        assert!(names.contains(&"checkout"));
    }
}
