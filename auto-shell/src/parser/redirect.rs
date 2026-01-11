//! I/O redirection parsing
//!
//! Handles parsing of redirection operators (>, >>, <).

/// Redirection specification
#[derive(Debug, Clone, PartialEq)]
pub struct Redirect {
    pub input: Option<String>,
    pub output: Option<String>,
    pub append: bool,
}

/// Parse redirection operators from a command
pub fn parse_redirect(input: &str) -> (String, Option<Redirect>) {
    // TODO: Implement redirect parsing in Phase 2
    (input.to_string(), None)
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
}
