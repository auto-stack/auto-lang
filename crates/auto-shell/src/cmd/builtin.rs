use miette::Result;
use std::path::Path;

use super::{data, fs};
use crate::parser::quote::parse_args;

/// Check whether a command name is handled by the legacy builtin dispatcher.
///
/// Used by the pipeline to decide whether to buffer `ExternalStream` data
/// into text (for builtins) or chain it directly via OS pipe (for externals).
pub fn is_legacy_builtin(name: &str) -> bool {
    matches!(
        name,
        // execute_builtin commands
        "pwd" | "echo" | "help" | "clear" | "ls" | "l" | "mkdir" | "rm" | "mv" | "cp"
        // execute_builtin_with_input commands
        | "sort" | "uniq" | "head" | "tail" | "wc" | "grep"
        | "count" | "first" | "last" | "genlines"
    )
}

/// Built-in command execution
///
/// Returns Some(output) if command was handled, None if not a built-in
pub fn execute_builtin(input: &str, current_dir: &Path) -> Result<Option<String>> {
    let parts: Vec<String> = parse_args(input);

    if parts.is_empty() {
        return Ok(None);
    }

    match parts[0].as_str() {
        // Basic commands
        "pwd" => Ok(Some(pwd_command(current_dir))),
        "echo" => Ok(Some(echo_command(&parts[1..]))),
        "help" => Ok(Some(help_command())),
        "clear" => Ok(Some(clear_command())),

        // File system commands
        "ls" | "l" => fs::ls_command(
            parse_path_arg(&parts, 1),
            current_dir,
            false,  // all
            false,  // long
            false,  // human
            false,  // time_sort
            false,  // reverse
            false,  // recursive
        ).map(Some),
        // Note: cd is handled by Shell::execute to update state
        "mkdir" => {
            let parents = parts.iter().any(|p| p == "-p" || p == "--parents");
            fs::mkdir_command(parse_path_arg(&parts, 1), current_dir, parents).map(Some)
        }
        "rm" => {
            let recursive = parts
                .iter()
                .any(|p| p == "-r" || p == "-R" || p == "--recursive");
            fs::rm_command(parse_path_arg(&parts, 1), current_dir, recursive).map(Some)
        }
        "mv" => fs::mv_command(
            parse_path_arg(&parts, 1),
            parse_path_arg(&parts, 2),
            current_dir,
        )
        .map(Some),
        "cp" => {
            let recursive = parts
                .iter()
                .any(|p| p == "-r" || p == "-R" || p == "--recursive");
            fs::cp_command(
                parse_path_arg(&parts, 1),
                parse_path_arg(&parts, 2),
                current_dir,
                recursive,
            )
            .map(Some)
        }

        // Data manipulation commands
        "sort" => {
            let reverse = parts.iter().any(|p| p == "-r" || p == "--reverse");
            let unique = parts.iter().any(|p| p == "-u" || p == "--unique");
            Ok(Some(data::sort_command("", reverse, unique))) // TODO: read from pipeline
        }
        "uniq" => {
            let count = parts.iter().any(|p| p == "-c" || p == "--count");
            Ok(Some(data::uniq_command("", count, false))) // TODO: read from pipeline
        }
        "head" => {
            let n = parse_number_arg(&parts, "-n").unwrap_or(10);
            Ok(Some(data::head_command("", n))) // TODO: read from pipeline
        }
        "tail" => {
            let n = parse_number_arg(&parts, "-n").unwrap_or(10);
            Ok(Some(data::tail_command("", n))) // TODO: read from pipeline
        }
        "wc" => Ok(Some(data::wc_command(""))), // TODO: read from pipeline
        "grep" => {
            let pattern = parts.get(1).map(|s| s.as_str()).unwrap_or("");
            Ok(Some(data::grep_command("", pattern, false))) // TODO: read from pipeline
        }

        // Count/first/last (for pipeline compatibility)
        "count" => Ok(Some(count_command(&parts[1..]))),
        "first" => Ok(Some(first_command(&parts[1..]))),
        "last" => Ok(Some(last_command(&parts[1..]))),

        // Test helper command (for testing pipelines)
        "genlines" => Ok(Some(genlines_command(&parts[1..]))),

        _ => Ok(None), // Not a built-in command
    }
}

/// Built-in command execution with pipeline input support
///
/// This version accepts optional pipeline input from previous commands
pub fn execute_builtin_with_input(
    input: &str,
    current_dir: &Path,
    pipeline_input: Option<&str>,
) -> Result<Option<String>> {
    let parts: Vec<String> = parse_args(input);

    if parts.is_empty() {
        return Ok(None);
    }

    match parts[0].as_str() {
        // Data processing commands that work with pipeline input
        "sort" => {
            let reverse = parts.iter().any(|p| p == "-r" || p == "--reverse");
            let unique = parts.iter().any(|p| p == "-u" || p == "--unique");
            let data = pipeline_input.unwrap_or("");
            Ok(Some(data::sort_command(data, reverse, unique)))
        }
        "uniq" => {
            let count = parts.iter().any(|p| p == "-c" || p == "--count");
            let data = pipeline_input.unwrap_or("");
            Ok(Some(data::uniq_command(data, count, false)))
        }
        "head" => {
            let n = parse_number_arg(&parts, "-n").unwrap_or(10);
            let data = pipeline_input.unwrap_or("");
            Ok(Some(data::head_command(data, n)))
        }
        "tail" => {
            let n = parse_number_arg(&parts, "-n").unwrap_or(10);
            let data = pipeline_input.unwrap_or("");
            Ok(Some(data::tail_command(data, n)))
        }
        "wc" => {
            let data = pipeline_input.unwrap_or("");
            Ok(Some(data::wc_command(data)))
        }
        "grep" => {
            let pattern = parts.get(1).map(|s| s.as_str()).unwrap_or("");
            let data = pipeline_input.unwrap_or("");
            Ok(Some(data::grep_command(data, pattern, false)))
        }
        "count" => {
            // Count lines in pipeline input
            let data = pipeline_input.unwrap_or("");
            Ok(Some(data.lines().count().to_string()))
        }
        "first" => {
            // Get first line of pipeline input
            let data = pipeline_input.unwrap_or("");
            Ok(Some(data.lines().next().unwrap_or("").to_string()))
        }
        "last" => {
            // Get last line of pipeline input
            let data = pipeline_input.unwrap_or("");
            Ok(Some(data.lines().last().unwrap_or("").to_string()))
        }

        // Test helper command (same behavior in both)
        "genlines" => Ok(Some(genlines_command(&parts[1..]))),

        // For other commands, use the standard execution
        _ => execute_builtin(input, current_dir),
    }
}

/// Parse a path argument from parts, handling empty/missing cases
fn parse_path_arg(parts: &[String], index: usize) -> &Path {
    parts.get(index).map(|s| s.as_str()).unwrap_or(".").as_ref()
}

/// Parse a number argument (e.g., -n 10)
fn parse_number_arg(parts: &[String], flag: &str) -> Option<usize> {
    for (i, part) in parts.iter().enumerate() {
        if part == flag {
            if let Some(next) = parts.get(i + 1) {
                return next.parse().ok();
            }
        }
        // Also handle -n10 format
        if part.starts_with(flag) && part.len() > flag.len() {
            return part[flag.len()..].parse().ok();
        }
    }
    None
}

/// Print working directory
fn pwd_command(current_dir: &Path) -> String {
    let mut path_str = current_dir.display().to_string();

    // 1. Remove UNC prefix on Windows
    if path_str.starts_with(r"\\?\") {
        path_str = path_str[4..].to_string();
    }

    // 2. Unify separators to forward slash
    path_str.replace('\\', "/")
}

/// Echo arguments
fn echo_command(args: &[String]) -> String {
    args.join(" ")
}

/// Clear screen (platform-specific)
fn clear_command() -> String {
    // ANSI escape code to clear screen
    "\x1b[2J\x1b[H".to_string()
}

/// Count lines in input (for pipeline use)
fn count_command(_args: &[String]) -> String {
    // TODO: In Phase 2, this will count pipeline input
    "0".to_string()
}

/// Get first line of input (for pipeline use)
fn first_command(_args: &[String]) -> String {
    // TODO: In Phase 2, this will extract first line from pipeline input
    "".to_string()
}

/// Get last line of input (for pipeline use)
fn last_command(_args: &[String]) -> String {
    // TODO: In Phase 2, this will extract last line from pipeline input
    "".to_string()
}

/// Show help message
fn help_command() -> String {
    r#"AutoShell v0.1.0

File System Commands:
  ls [path]              List directory contents
  cd <path>              Change directory (cd - for previous)
  mkdir [-p] <path>      Create directory
  rm [-r] <path>         Remove file/directory
  mv <src> <dst>         Move/rename file
  cp [-r] <src> <dst>    Copy file
  pushd <dir>            Push directory onto stack and cd
  popd                   Pop directory from stack
  dirs                   Show directory stack

Data Manipulation:
  sort [-r] [-u]         Sort lines
  uniq [-c]              Remove duplicate lines
  head [-n N]            Show first N lines
  tail [-n N]            Show last N lines
  wc                     Count lines, words, bytes
  grep <pattern>         Search for pattern

Variable Commands:
  set <name=value>       Set a local shell variable
  export <name=value>    Set an environment variable
  unset <name>           Remove a variable
  $name / ${name}        Variable expansion

Shell Features:
  alias name=command     Define command alias
  unalias <name>         Remove alias
  abbr -a <n> <exp>      Define abbreviation (Fish-style)
  source <file> / . <f>  Execute script file
  def name { body }      Define shell function
  hook <event> <func>    Register event hook

Configuration:
  config list            Show all settings
  config set <key> <val> Set configuration value
  config get <key>       Get configuration value

Job Control:
  cmd &                  Run in background
  jobs                   List background jobs
  fg [%n]                Bring job to foreground
  bg [%n]                Resume job in background

Special Variables:
  $?     Exit code of last command
  $_     Last command line
  $@     All arguments of last command
  $#     Number of arguments
  $PWD   Current directory
  $OLDPWD  Previous directory

History Expansion:
  !!      Last command
  !n      Command by number
  !string Last command starting with string
  Ctrl+R  Reverse history search

Pipelines & Chains:
  cmd1 | cmd2            Pipe output
  cmd1 && cmd2           Run cmd2 if cmd1 succeeds
  cmd1 || cmd2           Run cmd2 if cmd1 fails
  cmd > file             Redirect output to file
  cmd >> file            Append output to file
  cmd < file             Redirect input from file
  cmd 2> file            Redirect stderr
  cmd <<EOF ... EOF      Heredoc

AutoLang Integration:
  1 + 2                  Evaluate arithmetic
  let x = 1              Define immutable variable
  var x = 1              Define mutable variable
  fn f() int { ... }     Define Auto function
  for i in 0..10 { }     Range loop
"#
    .to_string()
}

/// Generate test lines (for pipeline testing)
fn genlines_command(args: &[String]) -> String {
    // Parse lines from arguments, default to test data
    if args.is_empty() {
        "3\n1\n2".to_string()
    } else {
        args.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pwd_command() {
        let path = Path::new("/test/path");
        let output = pwd_command(path);
        assert_eq!(output, "/test/path");
    }

    #[test]
    fn test_echo_command() {
        let output = echo_command(&["hello".to_string(), "world".to_string()]);
        assert_eq!(output, "hello world");
    }

    #[test]
    fn test_echo_empty() {
        let output = echo_command(&[]);
        assert_eq!(output, "");
    }

    #[test]
    fn test_parse_path_arg() {
        let parts = vec!["ls".to_string(), "/test/path".to_string()];
        let path = parse_path_arg(&parts, 1);
        assert_eq!(path, Path::new("/test/path"));
    }

    #[test]
    fn test_parse_path_arg_default() {
        let parts = vec!["ls".to_string()];
        let path = parse_path_arg(&parts, 1);
        assert_eq!(path, Path::new("."));
    }

    #[test]
    fn test_parse_number_arg() {
        let parts = vec!["head".to_string(), "-n".to_string(), "5".to_string()];
        let n = parse_number_arg(&parts, "-n");
        assert_eq!(n, Some(5));
    }

    #[test]
    fn test_parse_number_arg_combined() {
        let parts = vec!["head".to_string(), "-n10".to_string()];
        let n = parse_number_arg(&parts, "-n");
        assert_eq!(n, Some(10));
    }

    #[test]
    fn test_builtin_recognition() {
        let path = Path::new("/test");

        // Basic commands should return Some
        assert!(execute_builtin("pwd", path).unwrap().is_some());
        assert!(execute_builtin("echo hello", path).unwrap().is_some());
        assert!(execute_builtin("help", path).unwrap().is_some());

        // ls will fail on non-existent path, but that's expected
        // Just check that it's recognized as a built-in (returns Some)
        // but may return an error
        match execute_builtin("ls", path) {
            Ok(Some(_)) => {} // Good - it executed
            Ok(None) => panic!("ls should be recognized as built-in"),
            Err(_) => {} // Also ok - failed to execute but was recognized
        }

        // Non-built-in commands should return None
        assert!(execute_builtin("git", path).unwrap().is_none());
        assert!(execute_builtin("cargo build", path).unwrap().is_none());
    }
}
