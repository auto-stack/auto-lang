//! Auto-completion module
//!
//! Provides command, file, and shell variable completion.

pub mod auto;
pub mod command;
pub mod file;
pub mod reedline;

use crate::bookmarks::BookmarkManager;

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
    // Check if input ends with whitespace (user wants file completion after command)
    let ends_with_space = input.ends_with(|c: char| c.is_whitespace());

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
        if after_pipe.is_empty() || (!after_pipe.contains(' ') && !ends_with_space) {
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

    // If input ends with space or has multiple words, do file completion
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
                        comps.push(Completion {
                            display: sub.to_string(),
                            replacement: sub.to_string(),
                        });
                    }
                }

                // Bookmarks
                let manager = BookmarkManager::new();
                for (name, _) in manager.list() {
                    if name.starts_with(prefix) {
                        comps.push(Completion {
                            display: name.clone(),
                            replacement: name.clone(),
                        });
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
                        comps.push(Completion {
                            display: name.clone(),
                            replacement: name.clone(),
                        });
                    }
                }
                return comps;
            }
        }

        // Multiple words: check if last word starts with -
        if let Some(last) = parts.last() {
            if last.starts_with('-') {
                // Flag completion (TODO: not implemented yet)
                return Vec::new();
            }
        }
        // Complete file paths - pass original input, not trimmed!
        return file::complete_file(input);
    }

    // First word: complete commands
    if parts.len() == 1 {
        let cmd_completions = command::complete_command(trimmed);
        if !cmd_completions.is_empty() {
            return cmd_completions;
        }

        // If no command matches, try file completion
        return file::complete_file(input);
    }

    // Fallback to file completion
    file::complete_file(input)
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
    fn test_complete_file_partial() {
        let completions = get_completions("ls s");
        // Should complete files/directories starting with "s"
        // In auto-shell directory, should include "src/"
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "src/"));
    }

    #[test]
    fn test_complete_file_after_command_with_space() {
        let completions = get_completions("ls ");
        // "ls " ends with space, should complete files from current directory
        // We can't assert exact results without knowing directory structure
        // but it should return file completions, not command completions
        let _ = completions;
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
