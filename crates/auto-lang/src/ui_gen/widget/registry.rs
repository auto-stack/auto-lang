//! Widget Registry
//!
//! This module provides the registry for looking up widget specifications.
//! The registry stores widget specs and allows case-insensitive lookup by tag name.

use super::spec::WidgetSpec;
use std::collections::HashMap;

/// Widget registry for looking up widget specifications
pub struct WidgetRegistry {
    widgets: HashMap<String, WidgetSpec>,
}

impl WidgetRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    /// Register a widget
    pub fn register(&mut self, spec: WidgetSpec) {
        self.widgets.insert(spec.name.to_lowercase(), spec);
    }

    /// Look up a widget by tag name (case-insensitive)
    pub fn get(&self, tag: &str) -> Option<&WidgetSpec> {
        self.widgets.get(&tag.to_lowercase())
    }

    /// Check if a widget exists
    pub fn contains(&self, tag: &str) -> bool {
        self.widgets.contains_key(&tag.to_lowercase())
    }
}

impl Default for WidgetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = WidgetRegistry::new();
        assert!(registry.get("button").is_none()); // Empty registry
    }
}
