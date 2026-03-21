//! Widget Specification Types
//!
//! This module defines the core types for widget specifications that can be
//! loaded from .at files and used across different backend generators.

use std::collections::HashMap;

/// Widget category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetCategory {
    Layout,
    Form,
    Display,
    Navigation,
    Semantic,
}

/// Backend-specific component mapping
#[derive(Debug, Clone)]
pub struct BackendMapping {
    /// Component/composable name in target backend
    pub component: String,
    /// Import statement (if required)
    pub import: Option<String>,
    /// Property mappings: AURA prop -> backend prop
    pub props: HashMap<String, String>,
    /// Event mappings: AURA event -> backend event
    pub events: HashMap<String, String>,
}

/// Widget specification loaded from .at files
#[derive(Debug, Clone)]
pub struct WidgetSpec {
    /// Widget name (e.g., "Button", "Text")
    pub name: String,
    /// Widget category
    pub category: WidgetCategory,
    /// Primary prop for shorthand syntax
    pub primary_prop: Option<String>,
    /// Whether widget supports children
    pub has_children: bool,
    /// Backend-specific mappings
    pub backends: HashMap<String, BackendMapping>,
}

impl WidgetSpec {
    /// Create a new widget spec
    pub fn new(name: &str, category: WidgetCategory) -> Self {
        Self {
            name: name.to_string(),
            category,
            primary_prop: None,
            has_children: false,
            backends: HashMap::new(),
        }
    }

    /// Get backend mapping
    pub fn backend(&self, backend: &str) -> Option<&BackendMapping> {
        self.backends.get(backend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_spec_creation() {
        let spec = WidgetSpec {
            name: "Button".to_string(),
            category: WidgetCategory::Form,
            primary_prop: Some("text".to_string()),
            has_children: false,
            backends: HashMap::new(),
        };
        assert_eq!(spec.name, "Button");
    }

    #[test]
    fn test_widget_spec_new() {
        let spec = WidgetSpec::new("Text", WidgetCategory::Display);
        assert_eq!(spec.name, "Text");
        assert_eq!(spec.category, WidgetCategory::Display);
        assert_eq!(spec.primary_prop, None);
        assert_eq!(spec.has_children, false);
        assert!(spec.backends.is_empty());
    }

    #[test]
    fn test_backend_mapping() {
        let mut spec = WidgetSpec::new("Button", WidgetCategory::Form);

        let mut props = HashMap::new();
        props.insert("text".to_string(), "label".to_string());

        let mapping = BackendMapping {
            component: "Button".to_string(),
            import: Some("androidx.compose.material3.Button".to_string()),
            props,
            events: HashMap::new(),
        };

        spec.backends.insert("jet".to_string(), mapping);

        let jet_mapping = spec.backend("jet");
        assert!(jet_mapping.is_some());
        assert_eq!(jet_mapping.unwrap().component, "Button");
    }

    #[test]
    fn test_widget_category_equality() {
        assert_eq!(WidgetCategory::Form, WidgetCategory::Form);
        assert_ne!(WidgetCategory::Form, WidgetCategory::Layout);
    }
}
