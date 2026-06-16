//! Syntax highlighting for ASH shell input
//!
//! Implements reedline's `Highlighter` trait to colorize shell commands
//! as the user types, similar to Fish Shell's real-time syntax coloring.

use nu_ansi_term::Style;
use reedline::{Highlighter, StyledText};
use std::collections::HashSet;

use super::color::resolve_fg;

/// One Dark-inspired 24-bit theme for syntax highlighting (Plan 317).
/// These RGB values render in truecolor terminals; resolve_fg downsamples
/// to 256/16-color on terminals that don't support 24-bit.
const THEME_CMD_BUILTIN: (u8, u8, u8) = (86, 182, 194); // cyan
const THEME_CMD_EXTERNAL: (u8, u8, u8) = (152, 195, 121); // green
const THEME_STRING: (u8, u8, u8) = (229, 192, 123); // warm yellow
const THEME_FLAG: (u8, u8, u8) = (97, 175, 239); // blue
const THEME_OP: (u8, u8, u8) = (198, 120, 221); // purple
const THEME_VAR: (u8, u8, u8) = (224, 108, 117); // red
const THEME_REDIRECT: (u8, u8, u8) = (92, 99, 112); // gray

/// Shell syntax highlighter for reedline.
///
/// Colorizes tokens by type:
/// - Built-in command names: Cyan + Bold
/// - Strings (quoted): Yellow
/// - Flags (`-f`, `--flag`): Blue
/// - Operators (`|`, `&&`, `||`, `;`): Magenta
/// - Variables (`$VAR`, `${VAR}`): Red
/// - Redirects (`>`, `>>`, `<`, `2>`): DarkGray
/// - Regular arguments: Default
pub struct AshHighlighter {
    builtins: HashSet<&'static str>,
}

impl AshHighlighter {
    pub fn new() -> Self {
        // All built-in / registered command names
        let builtins: HashSet<&'static str> = [
            // Basic
            "cd", "pwd", "echo", "help", "clear", "exit", "quit", "q",
            "alias", "unalias", "source", ".",
            "set", "export", "unset", "use",
            "jobs", "fg", "bg", "suspend",
            "history",
            // File system
            "ls", "l", "mkdir", "rm", "cp", "mv", "touch", "find", "glob",
            "stat", "du", "file", "tee", "ln",
            "cat", "head", "tail", "sort", "uniq", "wc", "grep",
            "cut", "paste", "tr", "split", "rev", "column", "fmt", "diff",
            // Data formats
            "from_json", "to_json", "from_csv", "to_csv",
            "from_toml", "to_toml", "from_yaml", "to_yaml",
            "from_xml", "to_xml",
            // String
            "str_replace", "str_contains", "str_split", "str_join",
            "str_trim", "str_case", "str_length",
            // Math
            "math_sum", "math_avg", "math_min", "math_max", "math_round",
            // Data pipeline
            "select", "get", "where", "update", "insert", "each",
            "build", "run",
            // HTTP
            "http_get", "http_post", "http_put", "http_delete", "http_head",
            "url_encode",
            // Misc
            "date", "sleep", "which", "version", "ps", "sys",
            // Navigation
            "up", "u", "b",
        ].into_iter().collect();

        Self { builtins }
    }
}

impl Highlighter for AshHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut styled = StyledText::new();
        let mut chars = line.char_indices().peekable();
        let len = line.len();

        // State machine
        let mut first_word = true; // are we at the command position?
        let mut after_pipe_or_op = true; // after |, &&, ||, ; — next word is a command
        let mut i = 0;

        // Styles — 24-bit theme (Plan 317), resolved to the terminal's color depth.
        let cmd_builtin_style = Style::new().fg(resolve_fg(THEME_CMD_BUILTIN.0, THEME_CMD_BUILTIN.1, THEME_CMD_BUILTIN.2)).bold();
        let cmd_external_style = Style::new().fg(resolve_fg(THEME_CMD_EXTERNAL.0, THEME_CMD_EXTERNAL.1, THEME_CMD_EXTERNAL.2));
        let string_style = Style::new().fg(resolve_fg(THEME_STRING.0, THEME_STRING.1, THEME_STRING.2));
        let flag_style = Style::new().fg(resolve_fg(THEME_FLAG.0, THEME_FLAG.1, THEME_FLAG.2));
        let op_style = Style::new().fg(resolve_fg(THEME_OP.0, THEME_OP.1, THEME_OP.2)).bold();
        let var_style = Style::new().fg(resolve_fg(THEME_VAR.0, THEME_VAR.1, THEME_VAR.2));
        let redirect_style = Style::new().fg(resolve_fg(THEME_REDIRECT.0, THEME_REDIRECT.1, THEME_REDIRECT.2));
        let default_style = Style::new();

        while i < len {
            let (_, c) = match chars.peek() {
                Some(&(idx, ch)) => (idx, ch),
                None => break,
            };

            // Skip whitespace — emit as default
            if c == ' ' || c == '\t' {
                let start = i;
                while i < len && (line.as_bytes()[i] == b' ' || line.as_bytes()[i] == b'\t') {
                    i += 1;
                    chars.next();
                }
                styled.push((default_style, line[start..i].to_string()));
                continue;
            }

            // String literals: "..." or '...'
            if c == '"' || c == '\'' {
                let quote = c;
                let start = i;
                chars.next(); // consume opening quote
                i += 1;
                while i < len {
                    match chars.peek() {
                        Some(&(_, '\\')) => {
                            // Escaped char inside string
                            chars.next(); i += 1;
                            chars.next(); i += 1;
                        }
                        Some(&(_, ch)) if ch == quote => {
                            chars.next(); i += 1;
                            break;
                        }
                        _ => {
                            chars.next(); i += 1;
                        }
                    }
                }
                styled.push((string_style, line[start..i].to_string()));
                first_word = false;
                after_pipe_or_op = false;
                continue;
            }

            // Variable: $VAR or ${VAR}
            if c == '$' {
                let start = i;
                chars.next(); i += 1;
                if i < len && line.as_bytes()[i] == b'{' {
                    // ${VAR}
                    chars.next(); i += 1;
                    while i < len && line.as_bytes()[i] != b'}' {
                        chars.next(); i += 1;
                    }
                    if i < len {
                        chars.next(); i += 1; // consume }
                    }
                } else if i < len && line.as_bytes()[i] == b'(' {
                    // $(cmd) — command substitution, highlight $ as variable color
                    // and let the inner content be re-highlighted separately
                    chars.next(); i += 1;
                    let mut depth = 1;
                    while i < len && depth > 0 {
                        if line.as_bytes()[i] == b'(' { depth += 1; }
                        else if line.as_bytes()[i] == b')' { depth -= 1; }
                        chars.next(); i += 1;
                    }
                } else {
                    // $VAR
                    while i < len && (line.as_bytes()[i].is_ascii_alphanumeric() || line.as_bytes()[i] == b'_' || line.as_bytes()[i] == b'?') {
                        chars.next(); i += 1;
                    }
                }
                styled.push((var_style, line[start..i].to_string()));
                first_word = false;
                after_pipe_or_op = false;
                continue;
            }

            // Operators and pipes: |, &&, ||, ;
            if c == '|' || c == ';' {
                let start = i;
                chars.next(); i += 1;
                // Check for ||
                if c == '|' && i < len && line.as_bytes()[i] == b'|' {
                    chars.next(); i += 1;
                }
                styled.push((op_style, line[start..i].to_string()));
                after_pipe_or_op = true;
                continue;
            }
            if c == '&' {
                let start = i;
                chars.next(); i += 1;
                if i < len && line.as_bytes()[i] == b'&' {
                    chars.next(); i += 1;
                    styled.push((op_style, line[start..i].to_string()));
                    after_pipe_or_op = true;
                } else {
                    // Single & = background
                    styled.push((op_style, line[start..i].to_string()));
                    after_pipe_or_op = true;
                }
                continue;
            }

            // Redirects: >, >>, <, 2>, 2>>, 2>&1
            if c == '>' || c == '<' || (c == '2' && i + 1 < len && line.as_bytes()[i + 1] == b'>') {
                let start = i;
                if c == '2' {
                    chars.next(); i += 1; // consume 2
                }
                chars.next(); i += 1; // consume > or <
                if i < len && line.as_bytes()[i] == b'>' {
                    chars.next(); i += 1; // consume second >
                }
                if i < len && line.as_bytes()[i] == b'&' {
                    chars.next(); i += 1; // consume &
                    if i < len && line.as_bytes()[i] == b'1' {
                        chars.next(); i += 1; // consume 1
                    }
                }
                styled.push((redirect_style, line[start..i].to_string()));
                continue;
            }

            // Word (command name, flag, or argument)
            let start = i;
            while i < len {
                let ch = line.as_bytes()[i];
                if ch == b' ' || ch == b'\t' || ch == b'|' || ch == b';' ||
                   ch == b'&' || ch == b'>' || ch == b'<' || ch == b'"' || ch == b'\'' ||
                   ch == b'$'
                {
                    break;
                }
                // Break on 2> only if '2' is followed by '>'
                if ch == b'2' && i + 1 < len && line.as_bytes()[i + 1] == b'>' {
                    break;
                }
                chars.next();
                i += 1;
            }
            let word = &line[start..i];

            if first_word || after_pipe_or_op {
                // This is a command position
                if self.builtins.contains(word) {
                    styled.push((cmd_builtin_style, word.to_string()));
                } else {
                    styled.push((cmd_external_style, word.to_string()));
                }
                first_word = false;
                after_pipe_or_op = false;
            } else if word.starts_with('-') || word.starts_with("--") {
                styled.push((flag_style, word.to_string()));
            } else {
                styled.push((default_style, word.to_string()));
            }
        }

        styled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn highlight_line(line: &str) -> String {
        let h = AshHighlighter::new();
        let styled = h.highlight(line, 0);
        styled.render_simple()
    }

    #[test]
    fn test_highlight_builtin() {
        let result = highlight_line("ls -la");
        // Should contain ANSI escape codes (colored)
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_highlight_empty() {
        let result = highlight_line("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_highlight_variable() {
        let result = highlight_line("echo $HOME");
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_highlight_pipe() {
        let result = highlight_line("ls | grep foo");
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_highlight_string() {
        let result = highlight_line("echo \"hello world\"");
        assert!(result.contains("\x1b["));
    }

    #[test]
    fn test_highlight_and_chain() {
        let result = highlight_line("cd /tmp && ls");
        assert!(result.contains("\x1b["));
    }
}
