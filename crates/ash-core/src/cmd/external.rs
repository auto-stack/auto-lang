use miette::{miette, IntoDiagnostic, Result};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::pipeline::ExternalStream;

/// Execute an external command with platform-specific fallbacks.
///
/// When `capture_output` is false (default), the child process inherits
/// the terminal's stdin/stdout/stderr for real-time output — suitable for
/// standalone commands like `cargo build`.
///
/// When `capture_output` is true, stdout is captured and returned as a
/// string — suitable for pipeline usage where output feeds into the next
/// command.
///
/// On Windows: Tries command directly → PowerShell → CMD
/// On Unix: Tries command directly → sh (or bash/zsh if available)
pub fn execute_external(input: &str, current_dir: &Path, capture_output: bool) -> Result<Option<String>> {
    // Parse command and arguments
    let parts = parse_command(input);

    if parts.is_empty() {
        return Ok(None);
    }

    let cmd_name = &parts[0];
    let args = &parts[1..];

    // Try to execute the command directly first
    let direct_result = try_execute_command(cmd_name, args, current_dir, capture_output);

    // If direct execution failed, try platform-specific fallbacks
    if direct_result.is_err() {
        #[cfg(windows)]
        {
            // Windows: Try PowerShell, then CMD
            if let Ok(ps_result) = try_execute_powershell(cmd_name, args, current_dir, capture_output) {
                return Ok(ps_result);
            }
            // Note: We could try CMD here, but most things that work in CMD also work in PowerShell
        }

        #[cfg(unix)]
        {
            // Unix: Try sh, then bash, then zsh
            for shell in &["sh", "bash", "zsh"] {
                if let Ok(shell_result) = try_execute_with_shell(cmd_name, args, current_dir, shell, capture_output)
                {
                    return Ok(shell_result);
                }
            }
        }
    }

    direct_result
}

/// Spawn an external command and return a streaming ExternalStream.
///
/// Unlike `execute_external`, this spawns the process with piped stdout
/// and returns an `ExternalStream` that can be read incrementally.
/// This is the streaming equivalent of `capture_output = true`.
///
/// Stderr is inherited (goes to terminal) so the user can see error
/// messages in real time.
pub fn spawn_external_stream(input: &str, current_dir: &Path) -> Result<ExternalStream> {
    spawn_external_stream_impl(input, current_dir, None)
}

/// Spawn an external command with stdin data, returning a streaming ExternalStream.
///
/// Like `spawn_external_stream`, but pipes the given `stdin_data` to the
/// child process's stdin before returning. The write happens in a background
/// thread so the main thread can immediately start reading stdout.
pub fn spawn_external_stream_with_input(
    input: &str,
    current_dir: &Path,
    stdin_data: &str,
) -> Result<ExternalStream> {
    spawn_external_stream_impl(input, current_dir, Some(stdin_data))
}

/// Internal: shared implementation for both spawn variants.
fn spawn_external_stream_impl(
    input: &str,
    current_dir: &Path,
    stdin_data: Option<&str>,
) -> Result<ExternalStream> {
    let parts = parse_command(input);

    if parts.is_empty() {
        return Err(miette!("empty command"));
    }

    let cmd_name = &parts[0];
    let args = &parts[1..];

    // Try direct spawn first
    let direct_result = try_spawn_command_impl(cmd_name, args, current_dir, stdin_data);

    if direct_result.is_err() {
        #[cfg(windows)]
        {
            if let Ok(ps_result) =
                try_spawn_powershell_impl(cmd_name, args, current_dir, stdin_data)
            {
                return Ok(ps_result);
            }
        }

        #[cfg(unix)]
        {
            for shell in &["sh", "bash", "zsh"] {
                if let Ok(shell_result) =
                    try_spawn_with_shell_impl(cmd_name, args, current_dir, shell, stdin_data)
                {
                    return Ok(shell_result);
                }
            }
        }
    }

    direct_result
}

/// Try to spawn a command directly with piped stdout (and optional stdin).
fn try_spawn_command_impl(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
    stdin_data: Option<&str>,
) -> Result<ExternalStream> {
    let mut cmd = Command::new(cmd_name);
    cmd.args(args)
        .current_dir(current_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    if stdin_data.is_some() {
        cmd.stdin(Stdio::piped());
    }

    restore_sigint_in_child(&mut cmd);
    let child = cmd.spawn().into_diagnostic()?;

    match stdin_data {
        Some(data) => Ok(ExternalStream::new_with_stdin(child, data.to_string())),
        None => Ok(ExternalStream::new(child)),
    }
}

/// Try to spawn a command via PowerShell on Windows (with optional stdin).
#[cfg(windows)]
fn try_spawn_powershell_impl(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
    stdin_data: Option<&str>,
) -> Result<ExternalStream> {
    let ps_cmd = format!(
        "{}{}",
        cmd_name,
        args.iter()
            .map(|arg| format!(" \"{arg}\""))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-Command", &ps_cmd])
        .current_dir(current_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    if stdin_data.is_some() {
        cmd.stdin(Stdio::piped());
    }

    restore_sigint_in_child(&mut cmd);
    let child = cmd.spawn().into_diagnostic()?;

    match stdin_data {
        Some(data) => Ok(ExternalStream::new_with_stdin(child, data.to_string())),
        None => Ok(ExternalStream::new(child)),
    }
}

/// Try to spawn a command via a Unix shell (with optional stdin).
#[cfg(unix)]
fn try_spawn_with_shell_impl(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
    shell: &str,
    stdin_data: Option<&str>,
) -> Result<ExternalStream> {
    let shell_cmd = format!(
        "{} {}",
        cmd_name,
        args.iter()
            .map(|arg| format!("\"{}\"", arg.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let mut cmd = Command::new(shell);
    cmd.arg("-c")
        .arg(&shell_cmd)
        .current_dir(current_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    if stdin_data.is_some() {
        cmd.stdin(Stdio::piped());
    }

    restore_sigint_in_child(&mut cmd);
    let child = cmd.spawn().into_diagnostic()?;

    match stdin_data {
        Some(data) => Ok(ExternalStream::new_with_stdin(child, data.to_string())),
        None => Ok(ExternalStream::new(child)),
    }
}

/// Try to execute a command directly using std::process::Command
///
/// When `capture_output` is false, uses `.status()` with inherited stdio
/// for real-time terminal output (e.g. `cargo build`).
///
/// When `capture_output` is true, uses `.output()` to capture stdout
/// for pipeline consumption.
fn try_execute_command(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
    capture_output: bool,
) -> Result<Option<String>> {
    if capture_output {
        let mut cmd = Command::new(cmd_name);
        cmd.args(args)
            .current_dir(current_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        restore_sigint_in_child(&mut cmd);
        let output = cmd.output().into_diagnostic()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let trimmed = stdout.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(miette!("Command failed: {}", stderr.trim()))
        }
    } else {
        let mut cmd = Command::new(cmd_name);
        cmd.args(args).current_dir(current_dir);
        restore_sigint_in_child(&mut cmd);
        let status = cmd.status().into_diagnostic()?;

        if status.success() {
            Ok(None) // Output already went to terminal
        } else {
            Err(miette!(
                "Command failed with exit code: {}",
                status.code().unwrap_or(-1)
            ))
        }
    }
}

/// Try to execute a command via a Unix shell (sh/bash/zsh)
#[cfg(unix)]
fn try_execute_with_shell(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
    shell: &str,
    capture_output: bool,
) -> Result<Option<String>> {
    // Build shell command: sh -c "cmd arg1 arg2..."
    let shell_cmd = format!(
        "{} {}",
        cmd_name,
        args.iter()
            .map(|arg| format!("\"{}\"", arg.replace('"', "\\\"")))
            .collect::<Vec<_>>()
            .join(" ")
    );

    if capture_output {
        let mut cmd = Command::new(shell);
        cmd.arg("-c")
            .arg(&shell_cmd)
            .current_dir(current_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        restore_sigint_in_child(&mut cmd);
        let output = cmd.output().into_diagnostic()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let trimmed = stdout.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(miette!("{} command failed: {}", shell, stderr.trim()))
        }
    } else {
        let mut cmd = Command::new(shell);
        cmd.arg("-c")
            .arg(&shell_cmd)
            .current_dir(current_dir);
        restore_sigint_in_child(&mut cmd);
        let status = cmd.status().into_diagnostic()?;

        if status.success() {
            Ok(None)
        } else {
            Err(miette!(
                "{} command failed with exit code: {}",
                shell,
                status.code().unwrap_or(-1)
            ))
        }
    }
}

/// Try to execute a command via PowerShell on Windows
#[cfg(windows)]
fn try_execute_powershell(
    cmd_name: &str,
    args: &[String],
    current_dir: &Path,
    capture_output: bool,
) -> Result<Option<String>> {
    // Build PowerShell command
    // Use -Command with encoded arguments
    let ps_cmd = format!(
        "{}{}",
        cmd_name,
        args.iter()
            .map(|arg| format!(" \"{arg}\""))
            .collect::<Vec<_>>()
            .join(" ")
    );

    if capture_output {
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", &ps_cmd])
            .current_dir(current_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        restore_sigint_in_child(&mut cmd);
        let output = cmd.output().into_diagnostic()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let trimmed = stdout.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(miette!("PowerShell command failed: {}", stderr.trim()))
        }
    } else {
        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", &ps_cmd])
            .current_dir(current_dir);
        restore_sigint_in_child(&mut cmd);
        let status = cmd.status().into_diagnostic()?;

        if status.success() {
            Ok(None)
        } else {
            Err(miette!(
                "PowerShell command failed with exit code: {}",
                status.code().unwrap_or(-1)
            ))
        }
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

/// Unix: restore SIGINT to default in the child process.
///
/// The parent shell sets SIGINT to a handler that catches Ctrl+C
/// (so ASH survives). Without this fix, the child would inherit
/// the catch handler and also ignore Ctrl+C. We restore SIG_DFL
/// in the child so it terminates normally on Ctrl+C.
#[cfg(unix)]
fn libc_restore_sigint() {
    const SIGINT: i32 = 2;
    const SIG_DFL: usize = 0;
    extern "C" {
        fn signal(sig: i32, handler: usize) -> usize;
    }
    unsafe {
        signal(SIGINT, SIG_DFL);
    }
}

/// Apply SIGINT restoration pre_exec hook to a Command on Unix.
#[cfg(unix)]
fn restore_sigint_in_child(cmd: &mut std::process::Command) {
    use std::os::unix::process::CommandExt;
    unsafe {
        cmd.pre_exec(|| {
            libc_restore_sigint();
            Ok(())
        });
    }
}

/// No-op on Windows (children handle Ctrl+C via console events).
#[cfg(windows)]
fn restore_sigint_in_child(_cmd: &mut std::process::Command) {}

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
