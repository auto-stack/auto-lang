//! Widget Registry
//!
//! This module provides the registry for looking up widget specifications.
//! The registry stores widget specs and allows case-insensitive lookup by tag name.

use super::spec::{BackendMapping, WidgetCategory, WidgetSpec};
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

    /// Create registry with default widgets
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register default widget specifications
    fn register_defaults(&mut self) {
        self.register_layout_widgets();
        // Other categories will be added in later tasks
    }

    fn register_layout_widgets(&mut self) {
        // Column
        let mut col = WidgetSpec::new("Column", WidgetCategory::Layout)
            .with_alias("col");
        col.has_children = true;
        col.backends.insert("ark".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: None, // Built-in
            props: HashMap::new(),
            events: HashMap::new(),
        });
        col.backends.insert("jet".to_string(), BackendMapping {
            component: "Column".to_string(),
            import: Some("androidx.compose.foundation.layout.Column".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        col.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(col);

        // Row
        let mut row = WidgetSpec::new("Row", WidgetCategory::Layout)
            .with_alias("row");
        row.has_children = true;
        row.backends.insert("ark".to_string(), BackendMapping {
            component: "Row".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        row.backends.insert("jet".to_string(), BackendMapping {
            component: "Row".to_string(),
            import: Some("androidx.compose.foundation.layout.Row".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        row.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(row);

        // Stack
        let mut stack = WidgetSpec::new("Stack", WidgetCategory::Layout)
            .with_alias("stack");
        stack.has_children = true;
        stack.backends.insert("ark".to_string(), BackendMapping {
            component: "Stack".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        stack.backends.insert("jet".to_string(), BackendMapping {
            component: "Box".to_string(),
            import: Some("androidx.compose.foundation.layout.Box".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        stack.backends.insert("vue".to_string(), BackendMapping {
            component: "div".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(stack);

        // Scroll
        let mut scroll = WidgetSpec::new("Scroll", WidgetCategory::Layout)
            .with_alias("scroll");
        scroll.has_children = true;
        scroll.backends.insert("ark".to_string(), BackendMapping {
            component: "Scroll".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(scroll);
    }

    /// Register a widget
    pub fn register(&mut self, spec: WidgetSpec) {
        // Register under the canonical name
        let key = spec.name.to_lowercase();
        let aliases = spec.aliases.clone();
        self.widgets.insert(key.clone(), spec);

        // Register under all aliases (they point to the same spec)
        // Note: We need to clone for each alias
        for alias in aliases {
            if let Some(spec) = self.widgets.get(&key) {
                self.widgets.insert(alias, spec.clone());
            }
        }
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

    #[test]
    fn test_default_widgets_col() {
        let registry = WidgetRegistry::with_defaults();
        let col = registry.get("col").unwrap();
        assert_eq!(col.name, "Column");
        assert_eq!(col.category, WidgetCategory::Layout);
        assert!(col.has_children);
    }
}
