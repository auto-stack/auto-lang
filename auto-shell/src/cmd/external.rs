use miette::{IntoDiagnostic, Result};
use std::path::Path;
use std::process::Command;

/// Execute an external command
pub fn execute_external(input: &str, current_dir: &Path) -> Result<Option<String>> {
    // Parse command and arguments
    let parts = parse_command(input);

    if parts.is_empty() {
        return Ok(None);
    }

    let cmd_name = &parts[0];
    let args = &parts[1..];

    // Execute the command
    let output = Command::new(cmd_name)
        .args(args)
        .current_dir(current_dir)
        .output()
        .into_diagnostic()?;

    // Return stdout if successful
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(Some(stdout.trim().to_string()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        miette::bail!("Command failed: {}", stderr.trim());
    }
}

/// Parse command into parts (respecting quotes)
fn parse_command(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            ' ' | '\t' if !in_single_quote && !in_double_quote => {
                if !current.is_empty() {
                    parts.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let parts = parse_command("echo hello world");
        assert_eq!(parts, vec!["echo", "hello", "world"]);
    }

    #[test]
    fn test_parse_with_quotes() {
        let parts = parse_command("echo \"hello world\" 'foo bar'");
        assert_eq!(parts, vec!["echo", "hello world", "foo bar"]);
    }

    #[test]
    fn test_parse_mixed_quotes() {
        let parts = parse_command("echo \"it's\" 'foo\"bar'");
        assert_eq!(parts, vec!["echo", "it's", "foo\"bar"]);
    }

    #[test]
    fn test_parse_empty() {
        let parts = parse_command("");
        assert!(parts.is_empty());
    }

    #[test]
    fn test_parse_single_word() {
        let parts = parse_command("echo");
        assert_eq!(parts, vec!["echo"]);
    }
}
