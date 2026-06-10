//! Prompt rendering

/// Default prompt string
pub fn default_prompt() -> String {
    // TODO: Implement customizable prompt in Phase 7
    "⟩ ".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prompt() {
        let prompt = default_prompt();
        assert!(!prompt.is_empty());
    }
}
