//! Unified Shell Lexer
//!
//! Tokenizes raw shell input into a stream of [`ShellToken`] values.
//! This replaces the ad-hoc quote-tracking logic that was duplicated across
//! `pipeline.rs`, `redirect.rs`, and `highlight.rs`.
//!
//! # Token types
//!
//! | Token | Example |
//! |-------|---------|
//! | `Word` | `ls`, `-la`, `file.txt` |
//! | `Pipe` | `\|` |
//! | `And` | `&&` |
//! | `Or` | `\|\|` |
//! | `Semicolon` | `;` |
//! | `Background` | `&` (lone) |
//! | `RedirectIn` | `<` |
//! | `RedirectOut` | `>` |
//! | `RedirectAppend` | `>>` |
//! | `StderrRedirectOut` | `2>` |
//! | `StderrRedirectAppend` | `2>>` |
//! | `StderrToStdout` | `2>&1` |
//! | `SingleQuoted` | `'hello'` |
//! | `DoubleQuoted` | `"hello $VAR"` |
//! | `Variable` | `$VAR`, `${VAR}` |
//! | `CommandSubst` | `$(cmd)` or `` `cmd` `` |
//! | `Newline` | `\n` |

/// A single shell token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum ShellToken {
    /// A bare word or flag (e.g. `ls`, `-la`, `--verbose`)
    Word(String),
    /// `|`
    Pipe,
    /// `&&`
    And,
    /// `||`
    Or,
    /// `;`
    Semicolon,
    /// Lone `&` (background)
    Background,
    /// `<`
    RedirectIn,
    /// `>`
    RedirectOut,
    /// `>>`
    RedirectAppend,
    /// `2>`
    StderrRedirectOut,
    /// `2>>`
    StderrRedirectAppend,
    /// `2>&1`
    StderrToStdout,
    /// Single-quoted string: `'...'` — no expansion inside
    SingleQuoted(String),
    /// Double-quoted string: `"..."` — variable/command expansion applies
    DoubleQuoted(String),
    /// Variable reference: `$VAR` or `${VAR}`
    Variable(String),
    /// Command substitution: `$(cmd)` (backticks are converted to this)
    CommandSubst(String),
    /// Literal newline (for multi-line input)
    Newline,
}

/// Byte-index based scanner over a string.
struct Scanner<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Scanner<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn len(&self) -> usize {
        self.input.len()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Current byte (unchecked).
    fn byte(&self) -> u8 {
        self.input.as_bytes()[self.pos]
    }

    /// Byte at offset from current position, or None if out of bounds.
    fn byte_at(&self, offset: usize) -> Option<u8> {
        let idx = self.pos + offset;
        if idx < self.input.len() {
            Some(self.input.as_bytes()[idx])
        } else {
            None
        }
    }

    /// Current char (could be multi-byte UTF-8).
    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    /// Advance by one char.
    fn advance(&mut self) {
        if !self.is_eof() {
            let c = self.current_char().unwrap();
            self.pos += c.len_utf8();
        }
    }

    /// Advance by N bytes.
    fn advance_bytes(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.input.len());
    }

    /// Remaining input from current position.
    fn rest(&self) -> &'a str {
        &self.input[self.pos..]
    }
}

/// Tokenize a shell command line into a sequence of [`ShellToken`] values.
///
/// Handles:
/// - Quote tracking (single/double)
/// - Escape sequences (`\"`, `\'`, `\\`)
/// - Nested `$()` with depth tracking
/// - Backtick conversion to `$()` form
/// - Operator disambiguation (`|` vs `||`, `&` vs `&&`, `>` vs `>>`, etc.)
pub fn tokenize(input: &str) -> Vec<ShellToken> {
    let input = convert_backticks(input);
    let mut s = Scanner::new(&input);
    let mut tokens = Vec::new();

    while !s.is_eof() {
        let c = s.byte();

        // Skip whitespace (but not newline)
        if c == b' ' || c == b'\t' || c == b'\r' {
            s.advance();
            continue;
        }

        // Newline
        if c == b'\n' {
            tokens.push(ShellToken::Newline);
            s.advance();
            continue;
        }

        // Single-quoted string
        if c == b'\'' {
            s.advance(); // consume opening '
            let mut content = String::new();
            while !s.is_eof() {
                if s.byte() == b'\'' {
                    s.advance(); // consume closing '
                    break;
                }
                let ch = s.current_char().unwrap();
                content.push(ch);
                s.advance();
            }
            tokens.push(ShellToken::SingleQuoted(content));
            continue;
        }

        // Double-quoted string
        if c == b'"' {
            s.advance(); // consume opening "
            let mut content = String::new();
            while !s.is_eof() {
                if s.byte() == b'\\' {
                    s.advance(); // consume backslash
                    if !s.is_eof() {
                        let ch = s.current_char().unwrap();
                        content.push(ch);
                        s.advance();
                    }
                    continue;
                }
                if s.byte() == b'"' {
                    s.advance(); // consume closing "
                    break;
                }
                let ch = s.current_char().unwrap();
                content.push(ch);
                s.advance();
            }
            tokens.push(ShellToken::DoubleQuoted(content));
            continue;
        }

        // Variable: $VAR or ${VAR} or $(cmd)
        if c == b'$' {
            let next = s.byte_at(1);

            // $(cmd) — command substitution
            if next == Some(b'(') {
                s.advance_bytes(2); // consume $(
                let mut depth = 1;
                let mut cmd = String::new();
                while !s.is_eof() && depth > 0 {
                    let ch = s.byte();
                    if ch == b'$' && s.byte_at(1) == Some(b'(') {
                        s.advance_bytes(2);
                        depth += 1;
                        cmd.push_str("$(");
                    } else if ch == b')' {
                        s.advance();
                        depth -= 1;
                        if depth > 0 {
                            cmd.push(')');
                        }
                    } else {
                        let c = s.current_char().unwrap();
                        cmd.push(c);
                        s.advance();
                    }
                }
                tokens.push(ShellToken::CommandSubst(cmd));
                continue;
            }

            s.advance(); // consume $

            // ${VAR}
            if !s.is_eof() && s.byte() == b'{' {
                s.advance(); // consume {
                let mut name = String::new();
                while !s.is_eof() {
                    if s.byte() == b'}' {
                        s.advance();
                        break;
                    }
                    let ch = s.current_char().unwrap();
                    name.push(ch);
                    s.advance();
                }
                tokens.push(ShellToken::Variable(name));
                continue;
            }

            // $VAR or $?
            let mut name = String::new();
            while !s.is_eof() {
                let ch = s.byte();
                if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'?' {
                    name.push(ch as char);
                    s.advance();
                } else {
                    break;
                }
            }
            if !name.is_empty() {
                tokens.push(ShellToken::Variable(name));
            }
            continue;
        }

        // Pipe: | or ||
        if c == b'|' {
            s.advance();
            if !s.is_eof() && s.byte() == b'|' {
                s.advance();
                tokens.push(ShellToken::Or);
            } else {
                tokens.push(ShellToken::Pipe);
            }
            continue;
        }

        // & (and) or && or background
        if c == b'&' {
            s.advance();
            if !s.is_eof() && s.byte() == b'&' {
                s.advance();
                tokens.push(ShellToken::And);
            } else {
                tokens.push(ShellToken::Background);
            }
            continue;
        }

        // Semicolon
        if c == b';' {
            s.advance();
            tokens.push(ShellToken::Semicolon);
            continue;
        }

        // Redirects: <, >, >>
        if c == b'<' {
            s.advance();
            tokens.push(ShellToken::RedirectIn);
            continue;
        }
        if c == b'>' {
            s.advance();
            if !s.is_eof() && s.byte() == b'>' {
                s.advance();
                tokens.push(ShellToken::RedirectAppend);
            } else {
                tokens.push(ShellToken::RedirectOut);
            }
            continue;
        }

        // 2>, 2>>, 2>&1 — only if followed by >
        if c == b'2' && s.byte_at(1) == Some(b'>') {
            s.advance_bytes(2); // consume 2>
            if !s.is_eof() && s.byte() == b'>' {
                s.advance();
                tokens.push(ShellToken::StderrRedirectAppend);
            } else if !s.is_eof() && s.byte() == b'&' {
                s.advance(); // consume &
                if !s.is_eof() && s.byte() == b'1' {
                    s.advance(); // consume 1
                    tokens.push(ShellToken::StderrToStdout);
                } else {
                    tokens.push(ShellToken::StderrRedirectOut);
                }
            } else {
                tokens.push(ShellToken::StderrRedirectOut);
            }
            continue;
        }

        // Word: accumulate until we hit whitespace, operator, or quote
        let mut word = String::new();
        while !s.is_eof() {
            let ch = s.byte();
            if ch == b' ' || ch == b'\t' || ch == b'\r' || ch == b'\n' {
                break;
            }
            if ch == b'|' || ch == b';' || ch == b'&' || ch == b'<' || ch == b'>' {
                break;
            }
            if ch == b'\'' || ch == b'"' || ch == b'$' {
                break;
            }
            // Potential 2> redirect at word boundary — only break if word is empty
            if ch == b'2' && s.byte_at(1) == Some(b'>') && word.is_empty() {
                break;
            }
            let c = s.current_char().unwrap();
            word.push(c);
            s.advance();
        }

        if !word.is_empty() {
            tokens.push(ShellToken::Word(word));
        } else if !s.is_eof() {
            // Safety: consume unrecognized byte to prevent infinite loop
            s.advance();
        }
    }

    tokens
}

/// Convert backtick command substitution to `$()` syntax.
/// Backticks inside single quotes are left as-is.
fn convert_backticks(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_backtick = false;

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && !in_single_quote && !in_backtick {
            result.push(c);
            if let Some(next) = chars.next() {
                result.push(next);
            }
            continue;
        }

        if c == '\'' && !in_double_quote && !in_backtick {
            in_single_quote = !in_single_quote;
            result.push(c);
        } else if c == '"' && !in_single_quote && !in_backtick {
            in_double_quote = !in_double_quote;
            result.push(c);
        } else if c == '`' && !in_single_quote {
            if in_backtick {
                result.push(')');
            } else {
                result.push_str("$(");
            }
            in_backtick = !in_backtick;
        } else {
            result.push(c);
        }
    }

    result
}

/// Reconstruct a command string from tokens (lossy — quote style is normalized).
pub fn tokens_to_string(tokens: &[ShellToken]) -> String {
    let mut result = String::new();
    for (i, token) in tokens.iter().enumerate() {
        if i > 0 {
            result.push(' ');
        }
        match token {
            ShellToken::Word(s) => result.push_str(s),
            ShellToken::Pipe => result.push('|'),
            ShellToken::And => result.push_str("&&"),
            ShellToken::Or => result.push_str("||"),
            ShellToken::Semicolon => result.push(';'),
            ShellToken::Background => result.push('&'),
            ShellToken::RedirectIn => result.push('<'),
            ShellToken::RedirectOut => result.push('>'),
            ShellToken::RedirectAppend => result.push_str(">>"),
            ShellToken::StderrRedirectOut => result.push_str("2>"),
            ShellToken::StderrRedirectAppend => result.push_str("2>>"),
            ShellToken::StderrToStdout => result.push_str("2>&1"),
            ShellToken::SingleQuoted(s) => {
                result.push('\'');
                result.push_str(s);
                result.push('\'');
            }
            ShellToken::DoubleQuoted(s) => {
                result.push('"');
                result.push_str(s);
                result.push('"');
            }
            ShellToken::Variable(name) => {
                result.push('$');
                result.push_str(name);
            }
            ShellToken::CommandSubst(cmd) => {
                result.push_str("$(");
                result.push_str(cmd);
                result.push(')');
            }
            ShellToken::Newline => result.push('\n'),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_command() {
        let tokens = tokenize("ls -la");
        assert_eq!(tokens, vec![
            ShellToken::Word("ls".into()),
            ShellToken::Word("-la".into()),
        ]);
    }

    #[test]
    fn test_tokenize_pipe() {
        let tokens = tokenize("ls | grep foo");
        assert_eq!(tokens, vec![
            ShellToken::Word("ls".into()),
            ShellToken::Pipe,
            ShellToken::Word("grep".into()),
            ShellToken::Word("foo".into()),
        ]);
    }

    #[test]
    fn test_tokenize_and_or() {
        let tokens = tokenize("a && b || c");
        assert_eq!(tokens, vec![
            ShellToken::Word("a".into()),
            ShellToken::And,
            ShellToken::Word("b".into()),
            ShellToken::Or,
            ShellToken::Word("c".into()),
        ]);
    }

    #[test]
    fn test_tokenize_redirects() {
        let tokens = tokenize("echo hello > out.txt 2>&1");
        assert_eq!(tokens, vec![
            ShellToken::Word("echo".into()),
            ShellToken::Word("hello".into()),
            ShellToken::RedirectOut,
            ShellToken::Word("out.txt".into()),
            ShellToken::StderrToStdout,
        ]);
    }

    #[test]
    fn test_tokenize_append_and_stderr() {
        let tokens = tokenize("cmd >> log.txt 2> err.txt");
        assert_eq!(tokens, vec![
            ShellToken::Word("cmd".into()),
            ShellToken::RedirectAppend,
            ShellToken::Word("log.txt".into()),
            ShellToken::StderrRedirectOut,
            ShellToken::Word("err.txt".into()),
        ]);
    }

    #[test]
    fn test_tokenize_single_quoted() {
        let tokens = tokenize("echo 'hello world'");
        assert_eq!(tokens, vec![
            ShellToken::Word("echo".into()),
            ShellToken::SingleQuoted("hello world".into()),
        ]);
    }

    #[test]
    fn test_tokenize_double_quoted() {
        let tokens = tokenize("echo \"hello $USER\"");
        assert_eq!(tokens, vec![
            ShellToken::Word("echo".into()),
            ShellToken::DoubleQuoted("hello $USER".into()),
        ]);
    }

    #[test]
    fn test_tokenize_variable() {
        let tokens = tokenize("echo $HOME");
        assert_eq!(tokens, vec![
            ShellToken::Word("echo".into()),
            ShellToken::Variable("HOME".into()),
        ]);
    }

    #[test]
    fn test_tokenize_braced_variable() {
        let tokens = tokenize("echo ${HOME}");
        assert_eq!(tokens, vec![
            ShellToken::Word("echo".into()),
            ShellToken::Variable("HOME".into()),
        ]);
    }

    #[test]
    fn test_tokenize_command_subst() {
        let tokens = tokenize("echo $(pwd)");
        assert_eq!(tokens, vec![
            ShellToken::Word("echo".into()),
            ShellToken::CommandSubst("pwd".into()),
        ]);
    }

    #[test]
    fn test_tokenize_nested_command_subst() {
        let tokens = tokenize("echo $(basename $(pwd))");
        assert_eq!(tokens, vec![
            ShellToken::Word("echo".into()),
            ShellToken::CommandSubst("basename $(pwd)".into()),
        ]);
    }

    #[test]
    fn test_tokenize_backtick() {
        let tokens = tokenize("echo `whoami`");
        assert_eq!(tokens, vec![
            ShellToken::Word("echo".into()),
            ShellToken::CommandSubst("whoami".into()),
        ]);
    }

    #[test]
    fn test_tokenize_background() {
        let tokens = tokenize("sleep 1 &");
        assert_eq!(tokens, vec![
            ShellToken::Word("sleep".into()),
            ShellToken::Word("1".into()),
            ShellToken::Background,
        ]);
    }

    #[test]
    fn test_tokenize_semicolon() {
        let tokens = tokenize("a; b");
        assert_eq!(tokens, vec![
            ShellToken::Word("a".into()),
            ShellToken::Semicolon,
            ShellToken::Word("b".into()),
        ]);
    }

    #[test]
    fn test_tokenize_redirect_in() {
        let tokens = tokenize("sort < input.txt");
        assert_eq!(tokens, vec![
            ShellToken::Word("sort".into()),
            ShellToken::RedirectIn,
            ShellToken::Word("input.txt".into()),
        ]);
    }

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_tokens_to_string_roundtrip() {
        let input = "ls -la | grep foo && echo found";
        let tokens = tokenize(input);
        let reconstructed = tokens_to_string(&tokens);
        assert_eq!(reconstructed, input);
    }

    #[test]
    fn test_convert_backticks_in_single_quotes() {
        let result = convert_backticks("echo '`whoami`'");
        assert_eq!(result, "echo '`whoami`'");
    }

    #[test]
    fn test_tokenize_stderr_redirect_append() {
        let tokens = tokenize("cmd 2>> err.log");
        assert_eq!(tokens, vec![
            ShellToken::Word("cmd".into()),
            ShellToken::StderrRedirectAppend,
            ShellToken::Word("err.log".into()),
        ]);
    }

    #[test]
    fn test_tokenize_complex() {
        let tokens = tokenize("ls -la | grep foo && echo \"found: $count\" > out.txt || echo fail");
        assert_eq!(tokens, vec![
            ShellToken::Word("ls".into()),
            ShellToken::Word("-la".into()),
            ShellToken::Pipe,
            ShellToken::Word("grep".into()),
            ShellToken::Word("foo".into()),
            ShellToken::And,
            ShellToken::Word("echo".into()),
            ShellToken::DoubleQuoted("found: $count".into()),
            ShellToken::RedirectOut,
            ShellToken::Word("out.txt".into()),
            ShellToken::Or,
            ShellToken::Word("echo".into()),
            ShellToken::Word("fail".into()),
        ]);
    }

    #[test]
    fn test_tokenize_number_in_word() {
        // "md5sum" contains "5" but is NOT a redirect
        let tokens = tokenize("md5sum file.txt");
        assert_eq!(tokens, vec![
            ShellToken::Word("md5sum".into()),
            ShellToken::Word("file.txt".into()),
        ]);
    }

    #[test]
    fn test_tokenize_exit_code_variable() {
        let tokens = tokenize("echo $?");
        assert_eq!(tokens, vec![
            ShellToken::Word("echo".into()),
            ShellToken::Variable("?".into()),
        ]);
    }
}
