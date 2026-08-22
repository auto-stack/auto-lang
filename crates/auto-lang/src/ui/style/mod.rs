// Unified Styling System for AutoUI
//
// This module provides a Tailwind CSS-inspired utility class system that works across
// multiple backends (GPUI, Iced, etc.) through a unified intermediate representation.

mod class;
mod color;
mod layout_extract;
mod parser;

pub use class::{StyleClass, SizeValue, GradientDir};
pub use color::Color;
pub use layout_extract::BoxLayout;
pub use parser::StyleParser;

// Backend adapters (only compile when the respective backend is enabled)
#[cfg(feature = "ui-gpui")]
pub mod gpui_adapter;

/// Backend-neutral theme state + semantic color resolution (Plan 413/418
/// follow-up: extracted from iced_adapter so `code-editor`-only builds
/// compile — iced_adapter re-exports it).
pub mod theme;

#[cfg(feature = "ui-gpui")]
pub use gpui_adapter::GpuiStyle; // Re-export for backend adapters

#[cfg(feature = "ui-iced")]
pub mod iced_adapter;

#[cfg(feature = "ui-iced")]
pub use iced_adapter::IcedStyle; // Re-export for backend adapters

#[cfg(feature = "ui-headless")]
pub mod headless_adapter;

#[cfg(feature = "ui-headless")]
pub use headless_adapter::HeadlessStyle;

/// Parsed style collection ready to be applied to backend-specific components
#[derive(Debug, Clone, Default)]
pub struct Style {
    pub classes: Vec<StyleClass>,
    /// `hover:`-prefixed utilities, parsed into a parallel list. Consumed by
    /// the iced button renderer (hover-status styling); every other consumer
    /// iterates `classes` and ignores these, same as before when they were
    /// silently dropped at parse time.
    pub hover_classes: Vec<StyleClass>,
}

impl Style {
    /// Parse a style string into a Style collection
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut classes = Vec::new();
        let mut hover_classes = Vec::new();
        for token in input.split_whitespace() {
            if let Some(rest) = token.strip_prefix("hover:") {
                if let Ok(c) = StyleClass::parse_single(rest) {
                    hover_classes.push(c);
                }
            } else if let Ok(c) = StyleClass::parse_single(token) {
                classes.push(c);
            }
        }
        Ok(Self { classes, hover_classes })
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

    /// Plan 409 §10 续: display:none 语义——含 Hidden 且无任何 display class 覆盖。
    /// Tailwind 中 `hidden md:flex` 在 md+ 下 display 胜出(特异性更高);VM 桌面
    /// 窗口按 md+ 语义,所以同时有 display(Flex/Grid/Block/Inline/...)时不隐藏。
    pub fn is_hidden(&self) -> bool {
        let has_hidden = self.classes.iter().any(|c| matches!(c, StyleClass::Hidden));
        if !has_hidden {
            return false;
        }
        let has_display = self.classes.iter().any(|c| matches!(
            c,
            StyleClass::Flex
                | StyleClass::Grid
                | StyleClass::Block
                | StyleClass::Inline
                | StyleClass::InlineBlock
                | StyleClass::InlineFlex
        ));
        !has_display
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

    #[test]
    fn test_hover_classes_split_from_base() {
        let s = Style::parse("h-6 w-6 bg-transparent hover:bg-muted/60 hover:text-foreground")
            .unwrap();
        assert_eq!(s.classes.len(), 3, "base classes: {:?}", s.classes);
        assert_eq!(s.hover_classes.len(), 2, "hover classes: {:?}", s.hover_classes);
        assert!(s.hover_classes.iter().any(|c| matches!(c, StyleClass::BackgroundColor(_))));
        assert!(s.hover_classes.iter().any(|c| matches!(c, StyleClass::TextColor(_))));
    }

    #[test]
    fn test_hover_class_with_alpha_parses() {
        let s = Style::parse("hover:bg-red-500/10").unwrap();
        assert!(matches!(
            s.hover_classes.first(),
            Some(StyleClass::BackgroundColor(Color::Rgba { a, .. })) if (*a as f32 / 255.0 - 0.1).abs() < 0.01
        ));
    }
}

#[cfg(test)]
mod plan411_tests {
    use super::Style;

    #[test]
    fn test_md_hidden_is_hidden() {
        let s = Style::parse("md:hidden -ml-2").unwrap();
        assert!(s.is_hidden(), "md:hidden should be hidden: {:?}", s.classes);
    }

    #[test]
    fn test_hidden_sm_inline_not_hidden() {
        let s = Style::parse("font-bold text-lg hidden sm:inline").unwrap();
        assert!(!s.is_hidden());
    }

    #[test]
    fn test_responsive_text_size_ladder() {
        // Plan 411 P0-B: responsive prefixes are stripped (desktop semantics),
        // so this ladder must parse to three size classes with the LAST one
        // (lg:text-7xl) winning in sequential adapter application.
        let s = Style::parse("text-4xl md:text-5xl lg:text-7xl").unwrap();
        let has_4xl = s.classes.iter().any(|c| matches!(c, crate::ui::style::StyleClass::Text4Xl));
        assert!(has_4xl, "ladder parse: {:?}", s.classes);
    }

    #[test]
    fn test_md_hidden_classes_parse() {
        // Plan 411: prefix stripping yields exactly [Hidden]; negative margin
        // utilities are unknown and silently skipped (as with other unknowns).
        let s = Style::parse("md:hidden -ml-2").unwrap();
        assert_eq!(s.classes.len(), 1);
        assert!(matches!(s.classes[0], crate::ui::style::StyleClass::Hidden));
    }

    #[test]
    fn test_text_5xl_9xl_parse() {
        // Plan 411 P0-B: hero ladder needs text-5xl..text-9xl to exist so
        // "text-4xl md:text-5xl lg:text-7xl" resolves to 72px (last wins).
        use crate::ui::style::StyleClass;
        assert!(matches!(StyleClass::parse_single("text-5xl"), Ok(StyleClass::Text5Xl)));
        assert!(matches!(StyleClass::parse_single("lg:text-7xl"), Ok(StyleClass::Text7Xl)));
        assert!(matches!(StyleClass::parse_single("text-9xl"), Ok(StyleClass::Text9Xl)));
    }
}
