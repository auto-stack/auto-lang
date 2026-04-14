//! Error recovery and type suggestion helpers
//!
//! # Overview
//!
//! This module provides error recovery mechanisms and helpful type suggestions
//! for the type inference system. It allows compilation to continue after errors
//! to show multiple issues at once, and provides "did you mean?" suggestions.
//!
//! # Features
//!
//! - **Error Recovery**: Continue compilation after type errors
//! - **Type Suggestions**: Suggest similar types/variables for common mistakes
//! - **Enhanced Messages**: Format errors with helpful context
//!
//! # Example
//!
//! ```rust
//! use auto_lang::infer::errors::{suggest_type, suggest_variable};
//! use auto_lang::ast::{Name, Type};
//!
//! // Suggest similar types
//! let suggestions = suggest_type("int", &["Int".to_string(), "Uint".to_string()]);
//! assert_eq!(suggestions, Some("Int".to_string()));
//!
//! // Suggest similar variables
//! let vars = vec![Name::from("count"), Name::from("index")];
//! let suggestion = suggest_variable(&Name::from("cuont"), &vars);
//! assert_eq!(suggestion, Some(Name::from("count")));
//! ```

use crate::ast::{Name, Type};
use crate::error::{find_best_match, TypeError, Warning};
use std::fmt::Write;

/// Maximum number of type errors to collect before giving up
pub const MAX_TYPE_ERRORS: usize = 50;

/// Maximum number of suggestions to show for a single error
pub const MAX_SUGGESTIONS: usize = 3;

//===========================================================================
// Error Recovery
//===========================================================================

/// Check if we should continue compilation after an error
///
/// Returns true if the error count is below the threshold and compilation
/// should continue to find more errors.
pub fn should_continue(error_count: usize) -> bool {
    error_count < MAX_TYPE_ERRORS
}

/// Format multiple errors for display
///
/// Combines multiple type errors into a single diagnostic message,
/// showing the count and listing each error with context.
pub fn format_multiple_errors(errors: &[TypeError]) -> String {
    let mut msg = String::new();

    writeln!(msg, "found {} type error{}:", errors.len(), if errors.len() != 1 { "s" } else { "" }).ok();

    for (i, error) in errors.iter().enumerate() {
        writeln!(msg, "  {}. {}", i + 1, error).ok();
    }

    msg
}

/// Format error with suggestions
///
/// Appends "did you mean?" suggestions to an error message if available.
pub fn format_error_with_suggestion(error: &TypeError, suggestion: Option<&str>) -> String {
    let mut msg = format!("{}", error);

    if let Some(hint) = suggestion {
        writeln!(msg, "\n  hint: {}", hint).ok();
    }

    msg
}

//===========================================================================
// Type Suggestions
//===========================================================================

/// Suggest a similar type name from a list of candidates
///
/// Uses Levenshtein distance to find the most similar type name.
/// Returns None if no similar type is found (distance too large).
///
/// # Arguments
///
/// * `target` - The misspelled or unknown type name
/// * `candidates` - List of valid type names to search
///
/// # Example
///
/// ```rust
/// use auto_lang::infer::errors::suggest_type;
///
/// let candidates = vec![
///     "Int".to_string(),
///     "Uint".to_string(),
///     "Float".to_string(),
/// ];
///
/// // Suggest correction for typo
/// let suggestion = suggest_type("int", &candidates);
/// assert_eq!(suggestion, Some("Int".to_string()));
///
/// // No suggestion for completely different name
/// let suggestion = suggest_type("Vector", &candidates);
/// assert_eq!(suggestion, None);
/// ```
pub fn suggest_type(target: &str, candidates: &[String]) -> Option<String> {
    find_best_match(target, candidates)
}

/// Suggest a similar variable name from a list of candidates
///
/// Helps users find typos in variable names by suggesting similar names
/// that are in scope.
///
/// # Arguments
///
/// * `target` - The unknown or misspelled variable name
/// * `candidates` - List of valid variable names in scope
///
/// # Example
///
/// ```rust
/// use auto_lang::infer::errors::suggest_variable;
/// use auto_lang::ast::Name;
///
/// let candidates = vec![
///     Name::from("count"),
///     Name::from("index"),
///     Name::from("length"),
/// ];
///
/// // Suggest correction for typo
/// let suggestion = suggest_variable(&Name::from("cuont"), &candidates);
/// assert_eq!(suggestion, Some(Name::from("count")));
/// ```
pub fn suggest_variable(target: &Name, candidates: &[Name]) -> Option<Name> {
    let target_str = target.to_string();
    let candidate_strs: Vec<String> = candidates.iter().map(|n| n.to_string()).collect();

    find_best_match(&target_str, &candidate_strs).map(Name::from)
}

/// Get common primitive type names for suggestions
///
/// Returns a list of standard AutoLang types that are commonly used.
pub fn get_primitive_types() -> Vec<String> {
    vec![
        "int".to_string(),
        "uint".to_string(),
        "i8".to_string(),
        "i16".to_string(),
        "i32".to_string(),
        "i64".to_string(),
        "u8".to_string(),
        "u16".to_string(),
        "u32".to_string(),
        "u64".to_string(),
        "float".to_string(),
        "double".to_string(),
        "bool".to_string(),
        "char".to_string(),
        "str".to_string(),
        "void".to_string(),
        "nil".to_string(),
    ]
}

/// Suggest a similar type from primitive types
///
/// Convenience function that suggests from the list of primitive types.
pub fn suggest_primitive_type(target: &str) -> Option<String> {
    suggest_type(target, &get_primitive_types())
}

//===========================================================================
// Type Mismatch Suggestions
//===========================================================================

/// Generate a helpful message for type mismatch errors
///
/// Analyzes the expected and found types to provide specific suggestions:
/// - Suggests type conversions if applicable
/// - Points out common mistakes (int vs uint, etc.)
/// - Suggests similar type names if it looks like a typo
///
/// # Example
///
/// ```rust
/// use auto_lang::infer::errors::suggest_type_mismatch_fix;
/// use auto_lang::ast::Type;
///
/// let expected = Type::Int;
/// let found = Type::Uint;
/// let hint = suggest_type_mismatch_fix(&expected, &found);
/// assert!(hint.is_some());
/// ```
pub fn suggest_type_mismatch_fix(expected: &Type, found: &Type) -> Option<String> {
    match (expected, found) {
        // Numeric type mismatches
        (Type::Int, Type::Uint) | (Type::Uint, Type::Int) => {
            Some("use explicit conversion: `as int` or `as uint`".to_string())
        }
        (Type::Float, Type::Double) => {
            Some("use explicit conversion: `as float` or `as double`".to_string())
        }
        (Type::Double, Type::Float) => {
            Some("use explicit conversion: `as double` or `as float`".to_string())
        }

        // Pointer/Reference mismatches
        (Type::Ptr(_), Type::User(_)) => {
            Some("use the address-of operator `&` to create a reference".to_string())
        }
        (Type::User(_), Type::Ptr(_)) => {
            Some("use the dereference operator `*` to access the value".to_string())
        }

        // Optional type suggestions
        (Type::User(a), Type::User(b)) => {
            // Check if it might be a typo
            let a_str = a.name.to_string();
            let b_str = b.name.to_string();
            let distance = levenshtein_distance(&a_str, &b_str);

            if distance > 0 && distance <= 3 {
                Some(format!("did you mean `{}`?", a_str))
            } else {
                None
            }
        }

        _ => None,
    }
}

/// Levenshtein distance for type mismatch checking
///
/// Internal function for calculating edit distance between type names.
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();
    let m = chars1.len();
    let n = chars2.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut matrix = vec![vec![0; n + 1]; m + 1];

    for i in 0..=m {
        matrix[i][0] = i;
    }
    for j in 0..=n {
        matrix[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
            matrix[i][j] = [
                matrix[i - 1][j] + 1,
                matrix[i][j - 1] + 1,
                matrix[i - 1][j - 1] + cost,
            ]
            .iter()
            .min()
            .copied()
            .unwrap();
        }
    }

    matrix[m][n]
}

//===========================================================================
// Warning Helpers
//===========================================================================

/// Create a type coercion warning
///
/// Generates a warning when implicit type conversion occurs.
pub fn coercion_warning(from: &Type, to: &Type, span: miette::SourceSpan) -> Warning {
    Warning::ImplicitTypeConversion {
        span,
        from: from.to_string(),
        to: to.to_string(),
    }
}

/// Create an unused variable warning
///
/// Generates a warning when a variable is defined but never used.
pub fn unused_variable_warning(name: &Name, span: miette::SourceSpan) -> Warning {
    Warning::UnusedVariable {
        span,
        name: name.to_string(),
    }
}

/// Create a dead code warning
///
/// Generates a warning when code is unreachable (e.g., after return).
pub fn dead_code_warning(span: miette::SourceSpan) -> Warning {
    Warning::DeadCode {
        span,
    }
}

//===========================================================================
// Tests
//===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use miette::SourceSpan;

    #[test]
    fn test_should_continue_below_threshold() {
        assert!(should_continue(0));
        assert!(should_continue(25));
        assert!(should_continue(49));
    }

    #[test]
    fn test_should_continue_at_threshold() {
        assert!(!should_continue(50));
        assert!(!should_continue(100));
    }

    #[test]
    fn test_suggest_type_exact_match() {
        let candidates = vec!["Int".to_string(), "Uint".to_string()];
        let suggestion = suggest_type("Int", &candidates);
        assert_eq!(suggestion, Some("Int".to_string()));
    }

    #[test]
    fn test_suggest_type_typo() {
        let candidates = vec!["Int".to_string(), "Uint".to_string(), "Float".to_string()];
        let suggestion = suggest_type("int", &candidates);
        assert_eq!(suggestion, Some("Int".to_string()));
    }

    #[test]
    fn test_suggest_type_no_match() {
        let candidates = vec!["Int".to_string(), "Uint".to_string()];
        let suggestion = suggest_type("Vector", &candidates);
        assert_eq!(suggestion, None);
    }

    #[test]
    fn test_suggest_variable_typo() {
        let candidates = vec![
            Name::from("count"),
            Name::from("index"),
            Name::from("length"),
        ];
        let suggestion = suggest_variable(&Name::from("cuont"), &candidates);
        assert_eq!(suggestion, Some(Name::from("count")));
    }

    #[test]
    fn test_suggest_primitive_type() {
        // Test with typo (lowercase vs uppercase)
        let suggestion = suggest_primitive_type("int");
        assert_eq!(suggestion, Some("int".to_string()));

        // Test with non-existent type
        let suggestion = suggest_primitive_type("Vector");
        assert_eq!(suggestion, None);
    }

    #[test]
    fn test_suggest_type_mismatch_fix_int_uint() {
        let expected = Type::Int;
        let found = Type::Uint;
        let hint = suggest_type_mismatch_fix(&expected, &found);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("explicit conversion"));
    }

    #[test]
    fn test_suggest_type_mismatch_fix_user_type() {
        let expected = Type::User(crate::ast::TypeDecl {
            name: Name::from("MyType"),
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(),
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
            attrs: vec![],
            doc: None,
            is_pub: false,
        });
        let found = Type::User(crate::ast::TypeDecl {
            name: Name::from("MyTyp"), // typo
            kind: crate::ast::TypeDeclKind::UserType,
            parent: None,
            has: Vec::new(),
            specs: Vec::new(),
            spec_impls: Vec::new(),
            generic_params: Vec::new(),
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
            attrs: vec![],
            doc: None,
            is_pub: false,
        });
        let hint = suggest_type_mismatch_fix(&expected, &found);
        assert!(hint.is_some());
        assert!(hint.unwrap().contains("did you mean"));
    }

    #[test]
    fn test_coercion_warning() {
        let warning = coercion_warning(&Type::Int, &Type::Uint, SourceSpan::new(0.into(), 1));
        assert!(matches!(warning, Warning::ImplicitTypeConversion { .. }));
    }

    #[test]
    fn test_unused_variable_warning() {
        let warning = unused_variable_warning(&Name::from("x"), SourceSpan::new(0.into(), 1));
        assert!(matches!(warning, Warning::UnusedVariable { .. }));
    }

    #[test]
    fn test_format_error_with_suggestion() {
        let error = TypeError::Mismatch {
            span: SourceSpan::new(0.into(), 1),
            expected: "int".to_string(),
            found: "str".to_string(),
        };
        let formatted = format_error_with_suggestion(&error, Some("try using `as int`"));
        assert!(formatted.contains("try using"));
    }

    #[test]
    fn test_levenshtein_distance() {
        // Same strings
        assert_eq!(levenshtein_distance("hello", "hello"), 0);

        // One edit
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);

        // Two edits
        assert_eq!(levenshtein_distance("hello", "hllo"), 1);

        // Completely different
        assert!(levenshtein_distance("abc", "xyz") > 0);
    }
}
