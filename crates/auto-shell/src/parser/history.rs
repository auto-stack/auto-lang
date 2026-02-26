//! History expansion for shell commands
//!
//! Implements bash-style history expansion:
//! - `!!` - Last command
//! - `!n` - Command number n
//! - `!-n` - nth command from the end
//! - `!string` - Most recent command starting with string
//! - `!?string` - Most recent command containing string

use miette::{miette, Result};

/// Trait for history access
pub trait History {
    fn search(&self, query: Option<&str>) -> Vec<String>;
}

/// Expand history references in the input string
pub fn expand_history(input: &str, history: &dyn History) -> Result<String> {
    // If no history expansion markers, return as-is
    if !input.contains('!') {
        return Ok(input.to_string());
    }

    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let history_strings = history.search(None);

    while let Some(c) = chars.next() {
        if c == '!' {
            // Try to expand history reference
            match expand_history_ref(&mut chars, &history_strings) {
                Ok(expanded) => result.push_str(&expanded),
                Err(e) => {
                    // If expansion fails, return the error
                    return Err(e);
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

/// Expand a single history reference (after the !)
fn expand_history_ref(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    history_strings: &[String],
) -> Result<String> {
    let peek_char = chars.peek();

    match peek_char {
        None => Ok("!".to_string()), // Trailing ! is literal

        Some(&'!') => {
            // !! - Last command
            chars.next(); // consume second !
            history_strings.last()
                .cloned()
                .ok_or_else(|| miette!("No previous command"))
        }

        Some(&'-') => {
            // !-n - nth command from end
            chars.next(); // consume -
            let num_str: String = chars.take_while(|c| c.is_ascii_digit()).collect();
            let n: usize = num_str.parse()
                .map_err(|_| miette!("Invalid negative history reference: !-{}", num_str))?;

            if n == 0 || n > history_strings.len() {
                return Err(miette!("History reference out of range: !-{}", n));
            }

            let index = history_strings.len() - n;
            Ok(history_strings[index].clone())
        }

        Some(&'?') => {
            // !?string - Most recent command containing string
            chars.next(); // consume ?
            let search_str: String = chars.take_while(|c| *c != '?' && !c.is_whitespace()).collect();

            // Consume closing ? if present
            if chars.peek() == Some(&'?') {
                chars.next();
            }

            // Search backwards through history
            history_strings.iter()
                .rev()
                .find(|cmd| cmd.contains(&search_str))
                .cloned()
                .ok_or_else(|| miette!("No command found containing: {}", search_str))
        }

        Some(&c) if c.is_ascii_digit() => {
            // !n - Command number n
            chars.next(); // consume the first digit
            let num_str: String = std::iter::once(c)
                .chain(chars.take_while(|c| c.is_ascii_digit()))
                .collect();
            let n: usize = num_str.parse()
                .map_err(|_| miette!("Invalid history reference: !{}", num_str))?;

            if n == 0 || n > history_strings.len() {
                return Err(miette!("History reference out of range: !{}", n));
            }

            Ok(history_strings[n - 1].clone())
        }

        Some(&c) if c.is_alphabetic() || c == '_' => {
            // !string - Most recent command starting with string
            chars.next(); // consume the first character
            let search_str: String = std::iter::once(c)
                .chain(chars.take_while(|c| c.is_alphanumeric() || *c == '_'))
                .collect();

            history_strings.iter()
                .rev()
                .find(|cmd| cmd.starts_with(&search_str))
                .cloned()
                .ok_or_else(|| miette!("No command found starting with: {}", search_str))
        }

        _ => {
            // Unknown history reference, treat ! as literal
            Ok("!".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a mock history for testing
    fn mock_history() -> Vec<String> {
        vec![
            "echo hello".to_string(),
            "ls -la".to_string(),
            "cd /tmp".to_string(),
            "pwd".to_string(),
        ]
    }

    struct MockHistory {
        strings: Vec<String>,
    }

    impl History for MockHistory {
        fn search(&self, _query: Option<&str>) -> Vec<String> {
            self.strings.clone()
        }
    }

    #[test]
    fn test_expand_double_bang() {
        let history = MockHistory { strings: mock_history() };
        let result = expand_history("!!", &history).unwrap();
        assert_eq!(result, "pwd");
    }

    #[test]
    fn test_expand_negative_number() {
        let history = MockHistory { strings: mock_history() };
        let result = expand_history("!-1", &history).unwrap();
        assert_eq!(result, "pwd");

        let result = expand_history("!-2", &history).unwrap();
        assert_eq!(result, "cd /tmp");
    }

    #[test]
    fn test_expand_positive_number() {
        let history = MockHistory { strings: mock_history() };
        let result = expand_history("!1", &history).unwrap();
        assert_eq!(result, "echo hello");

        let result = expand_history("!3", &history).unwrap();
        assert_eq!(result, "cd /tmp");
    }

    #[test]
    fn test_expand_string_search() {
        let history = MockHistory { strings: mock_history() };
        let result = expand_history("!echo", &history).unwrap();
        assert_eq!(result, "echo hello");

        let result = expand_history("!l", &history).unwrap();
        assert_eq!(result, "ls -la");
    }

    #[test]
    fn test_expand_contains_search() {
        let history = MockHistory { strings: mock_history() };
        let result = expand_history("!?cd", &history).unwrap();
        assert_eq!(result, "cd /tmp");

        let result = expand_history("!?tmp", &history).unwrap();
        assert_eq!(result, "cd /tmp");
    }

    #[test]
    fn test_no_expansion() {
        let history = MockHistory { strings: mock_history() };
        let result = expand_history("echo hello", &history).unwrap();
        assert_eq!(result, "echo hello");
    }

    #[test]
    fn test_expansion_in_middle() {
        let history = MockHistory { strings: mock_history() };
        let result = expand_history("echo !!", &history).unwrap();
        assert_eq!(result, "echo pwd");
    }

    #[test]
    fn test_out_of_range() {
        let history = MockHistory { strings: mock_history() };
        let result = expand_history("!99", &history);
        assert!(result.is_err());
    }

    #[test]
    fn test_not_found() {
        let history = MockHistory { strings: mock_history() };
        let result = expand_history("!nonexistent", &history);
        assert!(result.is_err());
    }
}
