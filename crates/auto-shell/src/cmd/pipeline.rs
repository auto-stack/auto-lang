//! Pipeline execution
//!
//! Handles execution of command pipelines with structured data passing.

use miette::Result;
use std::path::Path;

use crate::data::ShellValue;

use super::{builtin, external};

/// Execute a pipeline of commands
///
/// Each command receives the output of the previous command as input.
pub fn execute_pipeline(commands: &[String], current_dir: &Path) -> Result<Option<String>> {
    if commands.is_empty() {
        return Ok(None);
    }

    // Start with no input
    let mut input_data: Option<ShellValue> = None;

    for (i, cmd) in commands.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == commands.len() - 1;

        // Execute the command
        let output = if is_first {
            // First command: no input
            execute_command(cmd, current_dir, None)?
        } else {
            // Subsequent commands: receive input from previous command
            execute_command(cmd, current_dir, input_data.as_ref())?
        };

        // Convert output to ShellValue for next command
        input_data = output.map(|s| ShellValue::String(s));

        // If this is the last command, return the final output
        if is_last {
            return Ok(input_data.map(|v| v.to_string()));
        }
    }

    Ok(None)
}

/// Execute a single command with optional input
fn execute_command(
    cmd: &str,
    current_dir: &Path,
    input: Option<&ShellValue>,
) -> Result<Option<String>> {
    let cmd = cmd.trim();

    // Extract input as string if available
    let input_str = input.as_ref().map(|v| {
        if let ShellValue::String(s) = v {
            s.clone()
        } else {
            v.to_string()
        }
    });

    // Check for built-in commands first
    if let Some(output) = builtin::execute_builtin_with_input(cmd, current_dir, input_str.as_deref())? {
        return Ok(Some(output));
    }

    // For external commands, we'll need to pipe input via stdin (TODO)
    // For now, just execute without pipeline input
    external::execute_external(cmd, current_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_command() {
        let commands = vec!["echo hello".to_string()];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("hello".to_string()));
    }

    #[test]
    fn test_empty_pipeline() {
        let commands: Vec<String> = vec![];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_builtin_pipeline() {
        // pwd | echo would just echo the pwd result
        let commands = vec!["pwd".to_string(), "echo test".to_string()];
        let result = execute_pipeline(&commands, Path::new("/test"));
        assert!(result.is_ok());
        // The echo command should output "test"
        assert_eq!(result.unwrap(), Some("test".to_string()));
    }

    #[test]
    fn test_echo_grep_pipeline() {
        // echo hello world foo | grep hello
        let commands = vec![
            "echo hello world foo".to_string(),
            "grep hello".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("hello world foo".to_string()));
    }

    #[test]
    fn test_echo_grep_no_match() {
        // echo hello world | grep xyz
        let commands = vec![
            "echo hello world".to_string(),
            "grep xyz".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("".to_string()));
    }

    #[test]
    fn test_count_pipeline() {
        // echo hello world foo | count
        // This should count 1 line (echo outputs single space-separated line)
        let commands = vec![
            "echo hello world foo".to_string(),
            "count".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("1".to_string()));
    }

    #[test]
    fn test_count_multiline() {
        // Create a multi-line string by using a custom command
        // For now, test that count works with actual newlines
        let commands = vec![
            "echo line1\nline2\nline3".to_string(),
            "count".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        // echo joins with spaces, so it's one line
        assert_eq!(result.unwrap(), Some("1".to_string()));
    }

    #[test]
    fn test_first_pipeline() {
        // echo hello world | first
        // first should get first "line" which is the whole string
        let commands = vec![
            "echo hello world".to_string(),
            "first".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("hello world".to_string()));
    }

    #[test]
    fn test_last_pipeline() {
        // echo hello world | last
        // last should get last "line" which is the whole string
        let commands = vec![
            "echo hello world".to_string(),
            "last".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("hello world".to_string()));
    }

    #[test]
    fn test_genlines_sort_pipeline() {
        // genlines | sort - should sort lines
        let commands = vec![
            "genlines".to_string(),
            "sort".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        // Sorted: 1, 2, 3
        assert_eq!(result.unwrap(), Some("1\n2\n3".to_string()));
    }

    #[test]
    fn test_genlines_head_pipeline() {
        // genlines | head -n 2
        let commands = vec![
            "genlines".to_string(),
            "head -n 2".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("3\n1".to_string()));
    }

    #[test]
    fn test_genlines_tail_pipeline() {
        // genlines | tail -n 2
        let commands = vec![
            "genlines".to_string(),
            "tail -n 2".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("1\n2".to_string()));
    }

    #[test]
    fn test_three_stage_pipeline() {
        // genlines | sort | head -n 2
        let commands = vec![
            "genlines".to_string(),
            "sort".to_string(),
            "head -n 2".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        // Sort: 1, 2, 3; Head: 1, 2
        assert_eq!(result.unwrap(), Some("1\n2".to_string()));
    }

    #[test]
    fn test_genlines_count_pipeline() {
        // genlines | count
        let commands = vec![
            "genlines".to_string(),
            "count".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("3".to_string()));
    }

    #[test]
    fn test_genlines_first_pipeline() {
        // genlines | first
        let commands = vec![
            "genlines".to_string(),
            "first".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("3".to_string()));
    }

    #[test]
    fn test_genlines_last_pipeline() {
        // genlines | last
        let commands = vec![
            "genlines".to_string(),
            "last".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("2".to_string()));
    }

    #[test]
    fn test_genlines_grep_pipeline() {
        // genlines 1 2 3 | grep 2
        let commands = vec![
            "genlines 1 2 3".to_string(),
            "grep 2".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("2".to_string()));
    }

    #[test]
    fn test_sort_unique_pipeline() {
        // genlines foo bar foo | sort -u
        let commands = vec![
            "genlines foo bar foo".to_string(),
            "sort -u".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        // Unique sorted: bar, foo
        assert_eq!(result.unwrap(), Some("bar\nfoo".to_string()));
    }

    #[test]
    fn test_four_stage_pipeline() {
        // genlines 3 1 2 4 | sort | head -n 3 | tail -n 2
        let commands = vec![
            "genlines 3 1 2 4".to_string(),
            "sort".to_string(),
            "head -n 3".to_string(),
            "tail -n 2".to_string(),
        ];
        let result = execute_pipeline(&commands, Path::new("/"));
        assert!(result.is_ok());
        // genlines: 3,1,2,4; sort: 1,2,3,4; head: 1,2,3; tail: 2,3
        assert_eq!(result.unwrap(), Some("2\n3".to_string()));
    }
}
