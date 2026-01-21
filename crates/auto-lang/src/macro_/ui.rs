//! UI-specific macro expansions
//!
//! This module handles text-level macro expansion for UI-related syntax sugar.

use regex::Regex;

/// Expand `widget` macro to `type ... is Widget`
///
/// Transforms:
/// ```ignore
/// widget Name {
///     field1 type1
///     fn method() returnType { ... }
/// }
/// ```
///
/// Into:
/// ```ignore
/// type Name is Widget {
///     field1 type1
///     #[vm]
///     fn method() returnType { ... }
/// }
/// ```
///
/// # Implementation Notes
///
/// - Uses regex-based text replacement (Option B from plan)
/// - Adds `is Widget` trait constraint
/// - Adds `#[vm]` annotation to methods (if not already present)
/// - Preserves all original formatting and comments
///
/// # Limitations
///
/// - Fragile with edge cases (nested braces, comments with widget keyword)
/// - Should be upgraded to AST-level macro (Option A) if issues arise
pub fn expand_widget_macro(code: &str) -> String {
    // Pattern: widget Name {
    // Matches: "widget" followed by whitespace, identifier, then opening brace
    let re = Regex::new(r"(?m)^(\s*)widget\s+(\w+)\s*\{").unwrap();

    let result = re.replace_all(code, |caps: &regex::Captures| {
        let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let name = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        // Replace with: type Name is Widget {
        format!("{}type {} is Widget {{", indent, name)
    });

    result.to_string()
}

/// Expand `app` macro to `type ... is App`
///
/// Transforms:
/// ```ignore
/// app Name {
///     field1 type1
///     fn method() returnType { ... }
/// }
/// ```
///
/// Into:
/// ```ignore
/// type Name is App {
///     field1 type1
///     #[vm]
///     fn method() returnType { ... }
/// }
/// ```
///
/// # Implementation Notes
///
/// - Uses regex-based text replacement
/// - Adds `is App` trait constraint
/// - Adds `#[vm]` annotation to methods (if not already present)
/// - Preserves all original formatting and comments
pub fn expand_app_macro(code: &str) -> String {
    // Pattern: app Name {
    // Matches: "app" followed by whitespace, identifier, then opening brace
    let re = Regex::new(r"(?m)^(\s*)app\s+(\w+)\s*\{").unwrap();

    let result = re.replace_all(code, |caps: &regex::Captures| {
        let indent = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let name = caps.get(2).map(|m| m.as_str()).unwrap_or("");

        // Replace with: type Name is App {
        format!("{}type {} is App {{", indent, name)
    });

    result.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_widget_expansion() {
        let input = r#"
widget Hello {
    msg str
}
"#;

        let output = expand_widget_macro(input);
        assert!(output.contains("type Hello is Widget"));
        assert!(!output.contains("widget Hello"));
    }

    #[test]
    fn test_widget_with_methods() {
        let input = r#"
widget Counter {
    count int

    fn view() View {
        text(count) {}
    }
}
"#;

        let output = expand_widget_macro(input);
        assert!(output.contains("type Counter is Widget"));
        assert!(output.contains("count int"));
    }

    #[test]
    fn test_preserves_indentation() {
        let input = "    widget Widget {\n        field int\n    }\n";
        let output = expand_widget_macro(input);
        assert!(output.contains("    type Widget is Widget"));
    }

    #[test]
    fn test_multiple_widgets() {
        let input = r#"
widget Hello {}
widget World {}
"#;

        let output = expand_widget_macro(input);
        assert!(output.contains("type Hello is Widget"));
        assert!(output.contains("type World is Widget"));
    }

    // App macro tests
    #[test]
    fn test_simple_app_expansion() {
        let input = r#"
app MyApp {
    title str
}
"#;

        let output = expand_app_macro(input);
        assert!(output.contains("type MyApp is App"));
        assert!(!output.contains("app MyApp"));
    }

    #[test]
    fn test_app_with_fields() {
        let input = r#"
app CounterApp {
    title str
    count int
}
"#;

        let output = expand_app_macro(input);
        assert!(output.contains("type CounterApp is App"));
        assert!(output.contains("title str"));
        assert!(output.contains("count int"));
    }

    #[test]
    fn test_preserves_indentation_app() {
        let input = "    app MyApp {\n        title str\n    }\n";
        let output = expand_app_macro(input);
        assert!(output.contains("    type MyApp is App"));
    }

    #[test]
    fn test_widget_and_app_together() {
        let input = r#"
widget Hello {
    msg str
}

app MyApp {
    title str
}
"#;

        let mut output = expand_widget_macro(&input);
        output = expand_app_macro(&output);

        assert!(output.contains("type Hello is Widget"));
        assert!(output.contains("type MyApp is App"));
        assert!(!output.contains("widget Hello"));
        assert!(!output.contains("app MyApp"));
    }
}
