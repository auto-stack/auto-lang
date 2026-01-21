//! Macro system for AutoLang
//!
//! This module implements text-level macro expansion for AutoLang syntax sugar.
//! Currently supports UI-specific macros like `widget` and `app`.

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
/// - `app Name { ... }` → `type Name is App { ... }`
///
/// # Example
///
/// ```ignore
/// let code = r#"
///     widget Hello {
///         msg str
///     }
///
///     app MyApp {
///         title str
///     }
/// "#;
///
/// let processed = preprocess(code);
/// assert!(processed.contains("type Hello is Widget"));
/// assert!(processed.contains("type MyApp is App"));
/// ```
pub fn preprocess(code: &str) -> String {
    let mut result = code.to_string();

    // Apply all macro expansions in order
    result = ui::expand_widget_macro(&result);
    result = ui::expand_app_macro(&result);

    result
}
