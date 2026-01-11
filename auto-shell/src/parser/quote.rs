//! Quote-aware argument parsing
//!
//! Properly handles quoted strings with escape sequences.

/// Parse input into arguments, respecting quotes
///
/// Supports:
/// - Double quotes: "hello world"
/// - Single quotes: 'it''s'
/// - Escape sequences: \", \', \, \n, \t, etc.
/// - Mixed quotes and unquoted text
///
/// # Examples
/// ```
/// use auto_shell::parser::quote::parse_args;
/// assert_eq!(parse_args("echo hello world"), vec!["echo", "hello", "world"]);
/// assert_eq!(parse_args("echo \"hello world\""), vec!["echo", "hello world"]);
/// assert_eq!(parse_args("echo 'it\\'s'"), vec!["echo", "it's"]);
/// ```
pub fn parse_args(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;
    let mut quoted = false; // Track if current arg was quoted

    while let Some(c) = chars.next() {
        if escape_next {
            // Handle escaped character
            current.push(match c {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                _ => c, // Unknown escape, just keep the character
            });
            escape_next = false;
        } else if c == '\\' {
            // Start escape sequence
            if in_single_quote {
                // In single quotes, backslash is literal (unless escaping ')
                if let Some(&'\'') = chars.peek() {
                    escape_next = true;
                } else {
                    current.push(c);
                }
            } else {
                escape_next = true;
            }
        } else if in_single_quote {
            match c {
                '\'' => in_single_quote = false,
                _ => current.push(c),
            }
        } else if in_double_quote {
            match c {
                '"' => in_double_quote = false,
                _ => current.push(c),
            }
        } else {
            match c {
                '\'' => {
                    // Check if quote is adjacent to previous text
                    if !current.is_empty() {
                        // Quote is part of the text, not starting a quoted section
                        current.push(c);
                    } else {
                        in_single_quote = true;
                        quoted = true;
                    }
                }
                '"' => {
                    // Check if quote is adjacent to previous text
                    if !current.is_empty() {
                        // Quote is part of the text, not starting a quoted section
                        current.push(c);
                    } else {
                        in_double_quote = true;
                        quoted = true;
                    }
                }
                ' ' | '\t' | '\n' => {
                    // Whitespace ends the current argument
                    if !current.is_empty() || quoted {
                        args.push(current.clone());
                        current.clear();
                        quoted = false;
                    }
                }
                _ => current.push(c),
            }
        }
    }

    // Handle final argument
    if !current.is_empty() || quoted {
        args.push(current);
    }

    // Handle empty input
    if args.is_empty() && input.trim().is_empty() {
        Vec::new()
    } else {
        args
    }
}

/// Parse input into arguments, preserving quotes in the result
///
/// Unlike parse_args, this keeps the quote marks in the output strings.
/// Useful when you need to know which arguments were quoted.
pub fn parse_args_preserve_quotes(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escape_next = false;

    while let Some(c) = chars.next() {
        if escape_next {
            current.push(c);
            escape_next = false;
        } else if c == '\\' {
            escape_next = true;
            current.push(c);
        } else if in_single_quote {
            current.push(c);
            if c == '\'' {
                in_single_quote = false;
            }
        } else if in_double_quote {
            current.push(c);
            if c == '"' {
                in_double_quote = false;
            }
        } else {
            match c {
                '\'' => {
                    in_single_quote = true;
                    current.push(c);
                }
                '"' => {
                    in_double_quote = true;
                    current.push(c);
                }
                ' ' | '\t' | '\n' => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(c),
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_words() {
        assert_eq!(parse_args("echo hello world"), vec!["echo", "hello", "world"]);
    }

    #[test]
    fn test_double_quotes() {
        assert_eq!(parse_args("echo \"hello world\""), vec!["echo", "hello world"]);
    }

    #[test]
    fn test_single_quotes() {
        assert_eq!(parse_args("echo 'hello world'"), vec!["echo", "hello world"]);
    }

    #[test]
    fn test_mixed_quotes() {
        assert_eq!(
            parse_args("echo \"hello\" 'world' test"),
            vec!["echo", "hello", "world", "test"]
        );
    }

    #[test]
    fn test_empty_quotes() {
        assert_eq!(parse_args("echo \"\""), vec!["echo", ""]);
    }

    #[test]
    fn test_quotes_with_spaces() {
        assert_eq!(
            parse_args("cmd \"arg with spaces\" another"),
            vec!["cmd", "arg with spaces", "another"]
        );
    }

    #[test]
    fn test_escaped_double_quote() {
        assert_eq!(parse_args("echo \"test\\\"quote\""), vec!["echo", "test\"quote"]);
    }

    #[test]
    fn test_escaped_single_quote() {
        assert_eq!(parse_args("echo 'test\\'quote'"), vec!["echo", "test'quote"]);
    }

    #[test]
    fn test_escaped_backslash() {
        assert_eq!(parse_args("echo \"test\\\\path\""), vec!["echo", "test\\path"]);
    }

    #[test]
    fn test_escaped_newline() {
        assert_eq!(parse_args("echo \"line1\\nline2\""), vec!["echo", "line1\nline2"]);
    }

    #[test]
    fn test_escaped_tab() {
        assert_eq!(parse_args("echo \"col1\\tcol2\""), vec!["echo", "col1\tcol2"]);
    }

    #[test]
    fn test_multiple_spaces() {
        assert_eq!(parse_args("echo    hello    world"), vec!["echo", "hello", "world"]);
    }

    #[test]
    fn test_leading_trailing_spaces() {
        assert_eq!(parse_args("  echo hello  "), vec!["echo", "hello"]);
    }

    #[test]
    fn test_nested_quotes() {
        // Single quotes inside double quotes
        assert_eq!(parse_args("echo \"it's a test\""), vec!["echo", "it's a test"]);

        // Double quotes inside single quotes
        assert_eq!(parse_args("echo 'say \"hello\"'"), vec!["echo", "say \"hello\""]);
    }

    #[test]
    fn test_preserve_quotes_simple() {
        assert_eq!(
            parse_args_preserve_quotes("echo \"hello\" 'world'"),
            vec!["echo", "\"hello\"", "'world'"]
        );
    }

    #[test]
    fn test_preserve_quotes_complex() {
        assert_eq!(
            parse_args_preserve_quotes("cmd \"arg with spaces\" another"),
            vec!["cmd", "\"arg with spaces\"", "another"]
        );
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(parse_args(""), Vec::<String>::new());
        assert_eq!(parse_args("   "), Vec::<String>::new());
    }

    #[test]
    fn test_only_quotes() {
        assert_eq!(parse_args("\"\""), vec![""]);
        assert_eq!(parse_args("''"), vec![""]);
    }

    #[test]
    fn test_special_chars() {
        assert_eq!(parse_args("echo $HOME | grep test"), vec!["echo", "$HOME", "|", "grep", "test"]);
    }

    #[test]
    fn test_pipe_in_quotes() {
        assert_eq!(parse_args("echo \"hello | world\""), vec!["echo", "hello | world"]);
    }

    #[test]
    fn test_variable_expansion_not_affected() {
        // Parser should preserve $ for later expansion
        assert_eq!(parse_args("echo $name ${test}"), vec!["echo", "$name", "${test}"]);
    }

    #[test]
    fn test_consecutive_quotes() {
        assert_eq!(parse_args("\"\"\"\""), vec![""]);
        assert_eq!(parse_args("''''"), vec![""]);
    }

    #[test]
    fn test_quote_after_text() {
        assert_eq!(parse_args("echo\"test\""), vec!["echo\"test\""]);
        assert_eq!(parse_args("echo'test'"), vec!["echo'test'"]);
    }
}
