// Unified Styling System for AutoUI
//
// This module provides a Tailwind CSS-inspired utility class system that works across
// multiple backends (GPUI, Iced, etc.) through a unified intermediate representation.

mod class;
mod color;
mod parser;

pub use class::{StyleClass, SizeValue};
pub use color::Color;
pub use parser::StyleParser;

// Backend adapters (only compile when the respective backend is enabled)
#[cfg(feature = "ui-gpui")]
pub mod gpui_adapter;

#[cfg(feature = "ui-gpui")]
pub use gpui_adapter::GpuiStyle; // Re-export for backend adapters

#[cfg(feature = "ui-iced")]
pub mod iced_adapter;

#[cfg(feature = "ui-iced")]
pub use iced_adapter::IcedStyle; // Re-export for backend adapters

/// Parsed style collection ready to be applied to backend-specific components
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub classes: Vec<StyleClass>,
}

impl Style {
    /// Parse a style string into a Style collection
    pub fn parse(input: &str) -> Result<Self, String> {
        let parser = StyleParser::new();
        let classes = parser.parse(input)?;
        Ok(Self { classes })
    }

    /// Create an empty style
    pub fn empty() -> Self {
        Self::default()
    }

    /// Add a style class
    pub fn add(mut self, class: StyleClass) -> Self {
        self.classes.push(class);
        self
    }
}

impl From<&str> for Style {
    fn from(input: &str) -> Self {
        Self::parse(input).expect("Failed to parse style string")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let style = Style::parse("p-4 gap-2 bg-white").unwrap();
        assert_eq!(style.classes.len(), 3);
    }

    #[test]
    fn test_from_str() {
        let style: Style = "flex items-center".into();
        assert_eq!(style.classes.len(), 2);
    }
}
