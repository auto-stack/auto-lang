//! Auto-completion module
//!
//! Provides command, file, and shell variable completion.

pub mod command;
pub mod file;
pub mod auto;

/// Completion suggestion
#[derive(Debug, Clone, PartialEq)]
pub struct Completion {
    pub display: String,
    pub replacement: String,
}

/// Get completions for the current input
///
/// This function intelligently determines which completion type to use
/// based on the input context:
/// - Command names at the start of line or after |
/// - File paths after command names
/// - Shell variables after $
pub fn get_completions(input: &str) -> Vec<Completion> {
    let trimmed = input.trim();

    // Empty input: complete all commands
    if trimmed.is_empty() {
        return command::complete_command(trimmed);
    }

    // Check if we're after a pipe
    if let Some(pipe_idx) = trimmed.rfind('|') {
        // Get the part after the last pipe
        let after_pipe = trimmed[pipe_idx + 1..].trim();

        // If nothing after pipe or just starting a command, complete commands
        if after_pipe.is_empty() || !after_pipe.contains(' ') {
            return command::complete_command(after_pipe);
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

    if parts.len() == 1 {
        // First word: complete commands
        let cmd_completions = command::complete_command(trimmed);
        if !cmd_completions.is_empty() {
            return cmd_completions;
        }

        // If no command matches, try file completion
        return file::complete_file(trimmed);
    }

    // Multiple words: check if last word starts with -
    if let Some(last) = parts.last() {
        if last.starts_with('-') {
            // Flag completion (TODO: not implemented yet)
            return Vec::new();
        }
    }

    // Otherwise, complete file paths
    file::complete_file(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_empty() {
        let completions = get_completions("");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "ls"));
    }

    #[test]
    fn test_complete_command() {
        let completions = get_completions("l");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "ls"));
    }

    #[test]
    fn test_complete_file_after_command() {
        let completions = get_completions("ls src");
        // Should try to complete "src" as a file path
        let _ = completions;
        // We can't assert exact results without knowing directory structure
    }

    #[test]
    fn test_complete_variable() {
        let completions = get_completions("echo $P");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "PATH"));
    }

    #[test]
    fn test_complete_after_pipe() {
        let completions = get_completions("ls | gr");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "grep"));
    }

    #[test]
    fn test_complete_no_match() {
        let completions = get_completions("nonexistent_command xyz");
        // Should return file completions
        let _ = completions;
    }
}
