//! Reedline Completer integration
//!
//! Provides integration between auto-shell's completion system and reedline's Tab completion.
//! The completer holds a snapshot of CommandRegistry signatures so it can provide
//! flag and command name completions.

use crate::completions::{Completion, CompletionSignature};
use reedline::{Completer, Suggestion};

/// Reedline completer for auto-shell
pub struct ShellCompleter {
    signatures: Vec<CompletionSignature>,
}

impl ShellCompleter {
    pub fn new(signatures: Vec<CompletionSignature>) -> Self {
        Self { signatures }
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
        // Get completions from our completion system with registry context
        let completions = crate::completions::get_completions_with_context(
            line,
            &self.signatures,
        );

        // Calculate the span to replace: from the last word boundary to cursor position
        let start = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let end = pos;

        // Convert to reedline Suggestions
        completions
            .into_iter()
            .map(|comp| {
                let mut suggestion = Self::completion_to_suggestion(comp);
                // Update the span to match the actual word to replace
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

    #[test]
    fn test_shell_completer_commands() {
        let sigs = test_signatures();
        let mut completer = ShellCompleter::new(sigs);
        let suggestions = completer.complete("l", 1);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "ls"));
    }

    #[test]
    fn test_shell_completer_flags() {
        let sigs = test_signatures();
        let mut completer = ShellCompleter::new(sigs);
        let suggestions = completer.complete("ls --", 5);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "--all"));
        assert!(suggestions.iter().any(|s| s.value == "--long"));
    }

    #[test]
    fn test_shell_completer_short_flags() {
        let sigs = test_signatures();
        let mut completer = ShellCompleter::new(sigs);
        let suggestions = completer.complete("ls -", 4);
        assert!(!suggestions.is_empty());
        // Should include both -a, -l and --all, --long
        assert!(suggestions.iter().any(|s| s.value == "-a"));
        assert!(suggestions.iter().any(|s| s.value == "-l"));
    }

    #[test]
    fn test_shell_completer_kind_tag_in_extra() {
        let sigs = test_signatures();
        let mut completer = ShellCompleter::new(sigs);
        let suggestions = completer.complete("ls --a", 6);
        let flag = suggestions.iter().find(|s| s.value == "--all").unwrap();
        assert_eq!(flag.extra.as_ref().unwrap()[0], "flag");
    }
}
