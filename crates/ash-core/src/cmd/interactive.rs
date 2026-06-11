//! Interactive command detection for ASH shell.
//!
//! Provides heuristics to detect commands that require full terminal control
//! (raw mode off, stdin/stdout inherited). These commands take over the
//! terminal and need the shell to suspend its line editor (reedline).

use std::path::Path;

/// Commands known to require interactive terminal control.
const INTERACTIVE_COMMANDS: &[&str] = &[
    // Text editors
    "vim",
    "vi",
    "nano",
    "emacs",
    "micro",
    "helix",
    "hx",
    "kak",
    "kakoune",
    // Pagers
    "less",
    "more",
    "bat",
    // System monitors
    "top",
    "htop",
    "btop",
    "glances",
    // Documentation
    "man",
    "info",
    // Remote access
    "ssh",
    "telnet",
    "mosh",
    // Terminal multiplexers
    "screen",
    "tmux",
    // Debuggers
    "gdb",
    "lldb",
    // REPLs (common ones)
    "python",
    "ipython",
    "node",
    "irb",
    // Database clients
    "psql",
    "mysql",
    "sqlite3",
];

/// Check if a command string refers to an interactive command.
///
/// Extracts the command name (first token), resolves any path to just
/// the filename, and checks against the known interactive command list.
///
/// # Examples
/// ```ignore
/// assert!(is_interactive_command("vim file.txt"));
/// assert!(is_interactive_command("/usr/bin/htop"));
/// assert!(!is_interactive_command("ls -la"));
/// assert!(!is_interactive_command("cargo build"));
/// ```
pub fn is_interactive_command(input: &str) -> bool {
    let name = input.split_whitespace().next().unwrap_or("");
    if name.is_empty() {
        return false;
    }

    // Extract just the binary name (strip path prefix)
    let name = Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(name);

    // On Windows, strip .exe suffix
    let name = name.strip_suffix(".exe").unwrap_or(name);

    INTERACTIVE_COMMANDS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interactive_editors() {
        assert!(is_interactive_command("vim"));
        assert!(is_interactive_command("vi file.txt"));
        assert!(is_interactive_command("nano /tmp/test.txt"));
        assert!(is_interactive_command("emacs"));
    }

    #[test]
    fn test_interactive_monitors() {
        assert!(is_interactive_command("top"));
        assert!(is_interactive_command("htop"));
        assert!(is_interactive_command("btop"));
    }

    #[test]
    fn test_interactive_ssh() {
        assert!(is_interactive_command("ssh user@host"));
        assert!(is_interactive_command("mosh user@host"));
    }

    #[test]
    fn test_interactive_multiplexers() {
        assert!(is_interactive_command("tmux"));
        assert!(is_interactive_command("tmux new -s mysession"));
        assert!(is_interactive_command("screen"));
    }

    #[test]
    fn test_interactive_pagers() {
        assert!(is_interactive_command("less README.md"));
        assert!(is_interactive_command("more file.txt"));
    }

    #[test]
    fn test_interactive_with_path() {
        assert!(is_interactive_command("/usr/bin/vim"));
        assert!(is_interactive_command("vim.exe"));
        assert!(is_interactive_command("C:\\Users\\test\\vim.exe"));
    }

    #[test]
    fn test_non_interactive() {
        assert!(!is_interactive_command("ls -la"));
        assert!(!is_interactive_command("cargo build"));
        assert!(!is_interactive_command("git status"));
        assert!(!is_interactive_command("echo hello"));
        assert!(!is_interactive_command("grep pattern file"));
        assert!(!is_interactive_command("cat file.txt"));
        assert!(!is_interactive_command("find . -name '*.rs'"));
    }

    #[test]
    fn test_empty_command() {
        assert!(!is_interactive_command(""));
        assert!(!is_interactive_command("  "));
    }

    #[test]
    fn test_interactive_debuggers() {
        assert!(is_interactive_command("gdb ./myapp"));
        assert!(is_interactive_command("lldb ./myapp"));
    }

    #[test]
    fn test_interactive_repls() {
        assert!(is_interactive_command("python"));
        assert!(is_interactive_command("ipython"));
        assert!(is_interactive_command("node"));
    }
}
