//! Shared UI Generation Infrastructure
//!
//! This module provides shared utilities for all UI generators (Vue, Jet, Tauri).
//! It implements the unified AURA syntax with Tailwind CSS as the primary styling approach.
//!
//! ## Architecture
//!
//! ```text
//! shared/
//! ├── mod.rs           - Module entry (this file)
//! ├── tailwind.rs      - Tailwind class parser and semantic analyzer
//! ├── registry.rs      - Component mapping registry (AURA → Vue/Jet)
//! ├── state.rs         - Model/Computed/Msg state analyzer
//! └── style.rs         - ComputedStyle data structure
//! ```
//!
//! ## Design Principles
//!
//! 1. **Unified AURA Syntax**: One source, multiple targets
//! 2. **Tailwind-First**: Use Tailwind CSS classes as the primary styling mechanism
//! 3. **Generator Responsibility**: Each generator maps Tailwind to its native styling

pub mod tailwind;
pub mod registry;
pub mod state;
pub mod style;

// Re-export main types
pub use tailwind::{TailwindParser, TailwindClass, Color, Shadow, FontWeight, TextAlign};
pub use registry::{ComponentRegistry, ComponentMapping, VueMapping, JetMapping};
pub use state::{StateAnalyzer, ModelProperty, ComputedProperty, MessageDef, EventHandler};
pub use style::{ComputedStyle, Display, FlexDirection, Spacing, Size, Dimension};

/// Shared generation utilities
pub struct SharedGen;

impl SharedGen {
    /// Parse a class attribute string into individual Tailwind classes
    pub fn parse_classes(class_str: &str) -> Vec<String> {
        class_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// Merge multiple class strings, filtering empty ones
    pub fn merge_classes(classes: &[&str]) -> String {
        classes
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_classes() {
        let classes = SharedGen::parse_classes("flex gap-4 p-4");
        assert_eq!(classes, vec!["flex", "gap-4", "p-4"]);
    }

    #[test]
    fn test_merge_classes() {
        let merged = SharedGen::merge_classes(&["flex", "flex-col", ""]);
        assert_eq!(merged, "flex flex-col");
    }
}
