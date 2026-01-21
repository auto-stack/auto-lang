//! Macro system for AutoLang
//!
//! This module implements text-level macro expansion for AutoLang syntax sugar.
//! Currently supports UI-specific macros like `widget`.

pub mod ui;

/// Preprocess AutoLang code by expanding macros
///
/// This function applies text-level transformations to the input code
/// before it reaches the parser. This enables syntactic sugar without
/// modifying the core parser.
///
/// # Supported Macros
///
/// - `widget Name { ... }` → `type Name is Widget { ... }`
///
/// # Example
///
/// ```ignore
/// let code = r#"
///     widget Hello {
///         msg str
///     }
/// "#;
///
/// let processed = preprocess(code);
/// assert!(processed.contains("type Hello is Widget"));
/// ```
pub fn preprocess(code: &str) -> String {
    ui::expand_widget_macro(code)
}
