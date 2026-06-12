//! I/O redirection parsing
//!
//! Handles parsing of redirection operators: `>`, `>>`, `<`, `2>`, `2>&1`.

/// Stderr redirect target
#[derive(Debug, Clone, PartialEq)]
pub enum StderrRedirect {
    /// `2> file` — overwrite
    File(String),
    /// `2>> file` — append
    Append(String),
    /// `2>&1` — merge stderr into stdout
    ToStdout,
}

/// Redirection specification
#[derive(Debug, Clone, PartialEq)]
pub struct Redirect {
    /// `< file` — redirect stdin from file
    pub stdin: Option<String>,
    /// `> file` or `>> file` — redirect stdout to file
    pub stdout: Option<String>,
    /// true for `>>`, false for `>`
    pub append_stdout: bool,
    /// `2> file`, `2>> file`, or `2>&1`
    pub stderr: Option<StderrRedirect>,
}

impl Redirect {
    pub fn new() -> Self {
        Self {
            stdin: None,
            stdout: None,
            append_stdout: false,
            stderr: None,
        }
    }

    pub fn has_any(&self) -> bool {
        self.stdin.is_some() || self.stdout.is_some() || self.stderr.is_some()
    }
}

/// Parse redirection operators from a command string.
///
/// Returns `(command_without_redirects, Option<Redirect>)`.
///
/// Respects quotes — operators inside `"..."` or `'...'` are ignored.
pub fn parse_redirect(input: &str) -> (String, Option<Redirect>) {
    let mut redirect = Redirect::new();
    let mut clean = String::new(); // command without redirects
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(c) = chars.next() {
        // Track quote state
        match c {
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                clean.push(c);
                continue;
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                clean.push(c);
                continue;
            }
            _ => {}
        }

        // Only process operators outside quotes
        if in_single_quote || in_double_quote {
            clean.push(c);
            continue;
        }

        match c {
            '<' => {
                // Input redirect: < file
                if let Some(file) = consume_redirect_target(&mut chars) {
                    redirect.stdin = Some(file);
                }
            }
            '>' => {
                // Check for >> (append)
                if chars.peek() == Some(&'>') {
                    chars.next(); // consume second >
                    if let Some(file) = consume_redirect_target(&mut chars) {
                        redirect.stdout = Some(file);
                        redirect.append_stdout = true;
                    }
                } else {
                    // Single > (overwrite)
                    if let Some(file) = consume_redirect_target(&mut chars) {
                        redirect.stdout = Some(file);
                        redirect.append_stdout = false;
                    }
                }
            }
            '2' => {
                // Check for 2> or 2>> or 2>&1
                if chars.peek() == Some(&'>') {
                    chars.next(); // consume >
                    if chars.peek() == Some(&'>') {
                        chars.next(); // consume second >
                        if let Some(file) = consume_redirect_target(&mut chars) {
                            redirect.stderr = Some(StderrRedirect::Append(file));
                        }
                    } else if chars.peek() == Some(&'&') {
                        chars.next(); // consume &
                        if chars.peek() == Some(&'1') {
                            chars.next(); // consume 1
                            redirect.stderr = Some(StderrRedirect::ToStdout);
                        }
                    } else {
                        if let Some(file) = consume_redirect_target(&mut chars) {
                            redirect.stderr = Some(StderrRedirect::File(file));
                        }
                    }
                } else {
                    // Just a '2' that's not a redirect
                    clean.push(c);
                }
            }
            _ => {
                clean.push(c);
            }
        }
    }

    let clean = clean.trim().to_string();
    if redirect.has_any() {
        (clean, Some(redirect))
    } else {
        (clean, None)
    }
}

/// Consume a redirect target filename from the character stream.
///
/// Skips leading whitespace, then reads until whitespace or end of string.
/// Handles quoted filenames: `> "my file.txt"` → `my file.txt`.
fn consume_redirect_target(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<String> {
    // Skip whitespace
    while chars.peek().map_or(false, |c| c.is_whitespace()) {
        chars.next();
    }

    if chars.peek().is_none() {
        return None;
    }

    let mut target = String::new();

    // Handle quoted filenames
    if chars.peek() == Some(&'"') {
        chars.next(); // consume opening "
        while let Some(c) = chars.next() {
            if c == '"' {
                break;
            }
            target.push(c);
        }
    } else if chars.peek() == Some(&'\'') {
        chars.next(); // consume opening '
        while let Some(c) = chars.next() {
            if c == '\'' {
                break;
            }
            target.push(c);
        }
    } else {
        // Unquoted: read until whitespace
        while let Some(c) = chars.peek() {
            if c.is_whitespace() {
                break;
            }
            target.push(*c);
            chars.next();
        }
    }

    if target.is_empty() {
        None
    } else {
        Some(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_no_redirect() {
        let (cmd, redirect) = parse_redirect("ls -la");
        assert_eq!(cmd, "ls -la");
        assert!(redirect.is_none());
    }

    #[test]
    fn test_parse_stdout_redirect() {
        let (cmd, redirect) = parse_redirect("echo hello > out.txt");
        assert_eq!(cmd, "echo hello");
        let r = redirect.unwrap();
        assert_eq!(r.stdout, Some("out.txt".to_string()));
        assert!(!r.append_stdout);
    }

    #[test]
    fn test_parse_stdout_append() {
        let (cmd, redirect) = parse_redirect("echo hello >> out.txt");
        assert_eq!(cmd, "echo hello");
        let r = redirect.unwrap();
        assert_eq!(r.stdout, Some("out.txt".to_string()));
        assert!(r.append_stdout);
    }

    #[test]
    fn test_parse_stdin_redirect() {
        let (cmd, redirect) = parse_redirect("sort < input.txt");
        assert_eq!(cmd, "sort");
        let r = redirect.unwrap();
        assert_eq!(r.stdin, Some("input.txt".to_string()));
    }

    #[test]
    fn test_parse_stderr_redirect() {
        let (cmd, redirect) = parse_redirect("cmd 2> err.txt");
        assert_eq!(cmd, "cmd");
        let r = redirect.unwrap();
        assert_eq!(r.stderr, Some(StderrRedirect::File("err.txt".to_string())));
    }

    #[test]
    fn test_parse_stderr_append() {
        let (cmd, redirect) = parse_redirect("cmd 2>> err.txt");
        assert_eq!(cmd, "cmd");
        let r = redirect.unwrap();
        assert_eq!(r.stderr, Some(StderrRedirect::Append("err.txt".to_string())));
    }

    #[test]
    fn test_parse_stderr_to_stdout() {
        let (cmd, redirect) = parse_redirect("cmd 2>&1");
        assert_eq!(cmd, "cmd");
        let r = redirect.unwrap();
        assert_eq!(r.stderr, Some(StderrRedirect::ToStdout));
    }

    #[test]
    fn test_parse_combined_stdout_and_stderr() {
        let (cmd, redirect) = parse_redirect("cmd > out.txt 2>&1");
        assert_eq!(cmd, "cmd");
        let r = redirect.unwrap();
        assert_eq!(r.stdout, Some("out.txt".to_string()));
        assert_eq!(r.stderr, Some(StderrRedirect::ToStdout));
    }

    #[test]
    fn test_parse_redirect_in_quotes_ignored() {
        let (cmd, redirect) = parse_redirect("echo \"hello > world\"");
        assert_eq!(cmd, "echo \"hello > world\"");
        assert!(redirect.is_none());
    }

    #[test]
    fn test_parse_quoted_filename() {
        let (cmd, redirect) = parse_redirect("echo hello > \"my file.txt\"");
        assert_eq!(cmd, "echo hello");
        let r = redirect.unwrap();
        assert_eq!(r.stdout, Some("my file.txt".to_string()));
    }

    #[test]
    fn test_parse_redirect_with_path() {
        let (cmd, redirect) = parse_redirect("cargo build 2> /tmp/build.log");
        assert_eq!(cmd, "cargo build");
        let r = redirect.unwrap();
        assert_eq!(r.stderr, Some(StderrRedirect::File("/tmp/build.log".to_string())));
    }
}
