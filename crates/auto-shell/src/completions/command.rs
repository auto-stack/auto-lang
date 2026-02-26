//! Command name completion
//!
//! Provides completion for built-in shell commands.

/// Built-in shell commands that can be completed
const BUILTIN_COMMANDS: &[&str] = &[
    // File system
    "ls", "cd", "pwd", "mkdir", "rm", "mv", "cp",
    // Data manipulation
    "sort", "uniq", "head", "tail", "wc", "grep", "count", "first", "last",
    // Variables
    "set", "export", "unset",
    // Utilities
    "echo", "help", "clear", "exit",
    // Test helpers
    "genlines",
];

/// Complete command names
pub fn complete_command(input: &str) -> Vec<super::Completion> {
    let mut completions = Vec::new();

    // Only complete if we're at the start of the line or after a pipe
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.ends_with('|') {
        // Complete all built-in commands
        for &cmd in BUILTIN_COMMANDS {
            completions.push(super::Completion {
                display: cmd.to_string(),
                replacement: cmd.to_string(),
            });
        }
    } else {
        // Extract the last word and complete it
        let last_word = trimmed.split_whitespace().last().unwrap_or("");
        if !last_word.is_empty() {
            for &cmd in BUILTIN_COMMANDS {
                if cmd.starts_with(last_word) {
                    completions.push(super::Completion {
                        display: cmd.to_string(),
                        replacement: cmd.to_string(),
                    });
                }
            }
        }
    }

    completions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_command_empty() {
        let completions = complete_command("");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "ls"));
    }

    #[test]
    fn test_complete_command_partial() {
        let completions = complete_command("l");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "ls"));
        // Should not include commands that don't start with 'l'
        assert!(!completions.iter().any(|c| c.display == "cd"));
    }

    #[test]
    fn test_complete_command_exact() {
        let completions = complete_command("ls");
        // Should return exact match
        assert!(completions.iter().any(|c| c.display == "ls"));
    }

    #[test]
    fn test_complete_command_after_pipe() {
        let completions = complete_command("echo test |");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "grep"));
    }

    #[test]
    fn test_complete_no_match() {
        let completions = complete_command("xyz");
        assert!(completions.is_empty());
    }
}
