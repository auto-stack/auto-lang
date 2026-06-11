//! Terminal interface module
//!
//! Handles terminal interaction, prompt rendering, and syntax highlighting.

pub mod highlight;
pub mod prompt;

/// Get terminal width
pub fn terminal_width() -> Option<usize> {
    // TODO: Implement terminal width detection
    Some(80)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_width() {
        let width = terminal_width();
        assert!(width.is_some());
    }
}
