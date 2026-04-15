//! Headless style adapter - records style classes without applying them
//!
//! Useful for testing that the correct styles are generated
//! without needing a real rendering backend.

use super::{Style, StyleClass};

/// Headless style - stores parsed classes for inspection
pub struct HeadlessStyle {
    /// The parsed style classes (for assertions)
    pub classes: Vec<StyleClass>,
    /// The original input string (for debugging)
    pub source: Option<String>,
}

impl HeadlessStyle {
    /// Create a headless style from a parsed Style
    pub fn from_style(style: &Style) -> Self {
        Self {
            classes: style.classes.clone(),
            source: None,
        }
    }

    /// Create a headless style from a raw class string
    pub fn parse(input: &str) -> Result<Self, String> {
        let style = Style::parse(input)?;
        Ok(Self {
            classes: style.classes.clone(),
            source: Some(input.to_string()),
        })
    }

    /// Check if a specific style class type is present
    pub fn has_class<F>(&self, predicate: F) -> bool
    where
        F: Fn(&StyleClass) -> bool,
    {
        self.classes.iter().any(predicate)
    }

    /// Number of style classes
    pub fn len(&self) -> usize {
        self.classes.len()
    }

    /// Whether there are no style classes
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headless_parse() {
        let style = HeadlessStyle::parse("p-4 gap-2 bg-white").unwrap();
        assert_eq!(style.len(), 3);
        assert!(!style.is_empty());
    }

    #[test]
    fn test_headless_has_class() {
        let style = HeadlessStyle::parse("flex items-center").unwrap();
        assert!(style.has_class(|c| matches!(c, StyleClass::Flex)));
    }

    #[test]
    fn test_headless_source_preserved() {
        let style = HeadlessStyle::parse("p-4").unwrap();
        assert_eq!(style.source.as_deref(), Some("p-4"));
    }
}
