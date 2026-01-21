use miette::{IntoDiagnostic, Result, miette};
use std::path::Path;
use std::process::Command;

/// Execute an external command with platform-specific fallbacks
///
/// On Windows: Tries command directly → PowerShell → CMD
/// On Unix: Tries command directly → sh (or bash/zsh if available)
pub fn execute_external(input: &str, current_dir: &Path) -> Result<Option<String>> {
    // Parse command and arguments
    let parts = parse_command(input);

    if parts.is_empty() {
        return Ok(None);
    }

    let cmd_name = &parts[0];
    let args = &parts[1..];

    // Try to execute the command directly first
    let direct_result = try_execute_command(cmd_name, args, current_dir);

    // If direct execution failed, try platform-specific fallbacks
    if direct_result.is_err() {
        #[cfg(windows)]
        {
            // Windows: Try PowerShell, then CMD
            if let Ok(ps_result) = try_execute_powershell(cmd_name, args, current_dir) {
                return Ok(ps_result);
            }
            // Note: We could try CMD here, but most things that work in CMD also work in PowerShell
        }

        #[cfg(unix)]
        {
            // Unix: Try sh, then bash, then zsh
            for shell in &["sh", "bash", "zsh"] {
                if let Ok(shell_result) = try_execute_with_shell(cmd_name, args, current_dir, shell) {
                    return Ok(shell_result);
                }
            }
        }
    }

    direct_result
}

/// Try to execute a command directly using std::process::Command
fn try_execute_command(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
) -> Result<Option<String>> {
    let output = Command::new(cmd_name)
        .args(args)
        .current_dir(current_dir)
        .output()
        .into_diagnostic()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(Some(stdout.trim().to_string()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(miette!("Command failed: {}", stderr.trim()))
    }
}

/// Try to execute a command via a Unix shell (sh/bash/zsh)
#[cfg(unix)]
fn try_execute_with_shell(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
    shell: &str,
) -> Result<Option<String>> {
    // Build shell command: sh -c "cmd arg1 arg2..."
    let shell_cmd = format!("{} {}", cmd_name,
        args.iter()
            .map(|arg| format!("\"{}\"", arg.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" "));

    let output = Command::new(shell)
        .arg("-c")
        .arg(&shell_cmd)
        .current_dir(current_dir)
        .output()
        .into_diagnostic()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(Some(stdout.trim().to_string()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(miette!("{} command failed: {}", shell, stderr.trim()))
    }
}

/// Try to execute a command via PowerShell on Windows
#[cfg(windows)]
fn try_execute_powershell(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
) -> Result<Option<String>> {
    // Build PowerShell command
    // Use -Command with encoded arguments
    let mut ps_cmd = format!("{}{}", cmd_name,
        args.iter()
            .map(|arg| format!(" \"{arg}\""))
            .collect::<Vec<_>>()
            .join(" "));

    // Execute via PowerShell
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &ps_cmd])
        .current_dir(current_dir)
        .output()
        .into_diagnostic()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(Some(stdout.trim().to_string()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(miette!("PowerShell command failed: {}", stderr.trim()))
    }
}

/// Try to execute a command via CMD on Windows (fallback)
#[cfg(windows)]
fn try_execute_cmd(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
) -> Result<Option<String>> {
    // Build CMD command
    let cmd_args = format!("/C {}{}", cmd_name,
        args.iter()
            .map(|arg| format!(" \"{}\"", arg))
            .collect::<Vec<_>>()
            .join(" "));

    let output = Command::new("cmd")
        .args(["/C", &cmd_args])
        .current_dir(current_dir)
        .output()
        .into_diagnostic()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(Some(stdout.trim().to_string()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(miette!("CMD command failed: {}", stderr.trim()))
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
