//! Syntax highlighting

/// Highlight AutoLang code
pub fn highlight_auto(code: &str) -> String {
    // TODO: Implement syntax highlighting in Phase 5
    code.to_string()
}

/// Highlight shell command
pub fn highlight_command(cmd: &str) -> String {
    // TODO: Implement syntax highlighting in Phase 5
    cmd.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_auto() {
        let highlighted = highlight_auto("let x = 1");
        assert_eq!(highlighted, "let x = 1");
    }

    #[test]
    fn test_highlight_command() {
        let highlighted = highlight_command("ls -la");
        assert_eq!(highlighted, "ls -la");
    }
}
