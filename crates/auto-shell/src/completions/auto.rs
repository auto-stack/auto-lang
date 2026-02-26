//! AutoLang and shell variable completion
//!
//! Provides completion for shell variables ($name, ${name}).

/// Complete shell variables
pub fn complete_auto(input: &str) -> Vec<super::Completion> {
    // This is a simplified implementation
    // A full implementation would need access to Shell state to list actual variables
    // For now, we provide completion for common environment variables

    let mut completions = Vec::new();

    // Only complete if input starts with $
    if !input.contains('$') {
        return completions;
    }

    // Find the last $ to complete
    let last_dollar_idx = input.rfind('$').unwrap_or(0);
    let var_part = &input[last_dollar_idx + 1..];

    // Check if it's braced syntax ${...}
    let is_braced = var_part.starts_with('{');
    let partial = if is_braced {
        var_part.trim_start_matches('{')
    } else {
        var_part
    };

    // Common environment variables to complete
    let common_vars = &[
        "PATH", "HOME", "USER", "SHELL", "PWD", "TERM",
        "EDITOR", "VISUAL", "PAGER", "LANG", "LC_ALL",
    ];

    for &var in common_vars {
        if var.starts_with(partial) {
            let replacement = if is_braced {
                // Build ${VAR} manually
                // "${" + VAR + "}" produces "${VAR}"
                let mut result = "${".to_string();
                result.push_str(var);
                result.push('}');
                result
            } else {
                format!("${}", var)
            };

            completions.push(super::Completion {
                display: var.to_string(),
                replacement,
            });
        }
    }

    completions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_auto_dollar() {
        let completions = complete_auto("$P");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "PATH"));
    }

    #[test]
    fn test_complete_auto_braced() {
        let completions = complete_auto("${HO");
        println!("Completions: {:?}", completions);
        assert!(!completions.is_empty());
        // Check that replacement contains the properly formatted ${HOME}
        assert!(completions.iter().any(|c| c.replacement.contains("${HOME") && c.replacement.ends_with('}')));
    }

    #[test]
    fn test_complete_no_dollar() {
        let completions = complete_auto("PATH");
        assert!(completions.is_empty());
    }

    #[test]
    fn test_complete_partial_match() {
        let completions = complete_auto("$U");
        assert!(!completions.is_empty());
        assert!(completions.iter().any(|c| c.display == "USER"));
    }

    #[test]
    fn test_complete_no_match() {
        let completions = complete_auto("$XYZ");
        assert!(completions.is_empty());
    }
}
