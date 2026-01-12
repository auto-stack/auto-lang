//! Reedline Completer integration
//!
//! Provides integration between auto-shell's completion system and reedline's Tab completion.

use reedline::{Completer, Suggestion};
use crate::completions::Completion;

/// Reedline completer for auto-shell
pub struct ShellCompleter;

impl ShellCompleter {
    pub fn new() -> Self {
        Self
    }

    /// Convert our Completion to reedline Suggestion
    fn completion_to_suggestion(completion: Completion) -> Suggestion {
        Suggestion {
            value: completion.replacement.clone(),
            description: Some(completion.display.clone()),
            extra: None,
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

impl Completer for ShellCompleter {
    /// Complete the input line at the given position
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        // Get completions from our completion system
        let completions = crate::completions::get_completions(line);

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

impl Default for ShellCompleter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_completer_empty() {
        let mut completer = ShellCompleter::new();
        let suggestions = completer.complete("", 0);
        // Should return all commands
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value.contains("ls")));
    }

    #[test]
    fn test_shell_completer_command() {
        let mut completer = ShellCompleter::new();
        let suggestions = completer.complete("l", 1);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "ls"));
    }

    #[test]
    fn test_shell_completer_after_pipe() {
        let mut completer = ShellCompleter::new();
        let suggestions = completer.complete("echo test | gr", 12);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value == "grep"));
    }

    #[test]
    fn test_shell_completer_variable() {
        let mut completer = ShellCompleter::new();
        let suggestions = completer.complete("echo $P", 7);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.value.contains("PATH")));
    }
}
