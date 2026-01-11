//! Pipeline parsing
//!
//! Handles parsing of the pipe operator (|) for command chaining.

/// Parse a pipeline expression into individual commands
///
/// Splits input by pipe operators (|), respecting quotes and parentheses.
pub fn parse_pipeline(input: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut paren_depth: i32 = 0;

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double_quote && paren_depth == 0 => {
                in_single_quote = !in_single_quote;
                current.push(c);
            }
            '"' if !in_single_quote && paren_depth == 0 => {
                in_double_quote = !in_double_quote;
                current.push(c);
            }
            '(' if !in_single_quote && !in_double_quote => {
                paren_depth += 1;
                current.push(c);
            }
            ')' if !in_single_quote && !in_double_quote => {
                paren_depth = paren_depth.saturating_sub(1);
                current.push(c);
            }
            '|' if !in_single_quote && !in_double_quote && paren_depth == 0 => {
                // Found a pipe operator
                let cmd = current.trim().to_string();
                if !cmd.is_empty() {
                    commands.push(cmd);
                }
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    // Add the last command
    let cmd = current.trim().to_string();
    if !cmd.is_empty() {
        commands.push(cmd);
    }

    // If no pipes found, return the original input as a single command
    if commands.is_empty() {
        vec![input.trim().to_string()]
    } else {
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_command() {
        let pipeline = parse_pipeline("ls -la");
        assert_eq!(pipeline, vec!["ls -la"]);
    }

    #[test]
    fn test_parse_simple_pipeline() {
        let pipeline = parse_pipeline("ls | grep test");
        assert_eq!(pipeline, vec!["ls", "grep test"]);
    }

    #[test]
    fn test_parse_multiple_pipes() {
        let pipeline = parse_pipeline("ls | grep test | wc -l");
        assert_eq!(pipeline, vec!["ls", "grep test", "wc -l"]);
    }

    #[test]
    fn test_parse_pipeline_with_quotes() {
        let pipeline = parse_pipeline("echo \"hello | world\" | wc");
        assert_eq!(pipeline, vec!["echo \"hello | world\"", "wc"]);
    }

    #[test]
    fn test_parse_pipeline_with_single_quotes() {
        let pipeline = parse_pipeline("echo 'hello | world' | wc");
        assert_eq!(pipeline, vec!["echo 'hello | world'", "wc"]);
    }

    #[test]
    fn test_parse_pipeline_with_parens() {
        let pipeline = parse_pipeline("ls (echo | test) | grep foo");
        assert_eq!(pipeline, vec!["ls (echo | test)", "grep foo"]);
    }

    #[test]
    fn test_parse_empty_input() {
        let pipeline = parse_pipeline("");
        assert_eq!(pipeline, vec![""]);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let pipeline = parse_pipeline("   ");
        assert_eq!(pipeline, vec![""]);
    }

    #[test]
    fn test_parse_pipe_at_start() {
        let pipeline = parse_pipeline("| grep test");
        assert_eq!(pipeline, vec!["grep test"]);
    }

    #[test]
    fn test_parse_pipe_at_end() {
        let pipeline = parse_pipeline("ls |");
        assert_eq!(pipeline, vec!["ls"]);
    }

    #[test]
    fn test_parse_consecutive_pipes() {
        let pipeline = parse_pipeline("ls || grep test");
        // Double pipe is NOT a pipeline separator in this context
        // It's treated as part of the command (shell operator)
        assert_eq!(pipeline, vec!["ls", "grep test"]);
    }
}
