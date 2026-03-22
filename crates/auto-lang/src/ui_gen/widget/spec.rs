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
    Overlay,
    Feedback,
    Data,
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
    /// Additional components exported from same module (for multi-component imports)
    pub extra_components: Vec<String>,
}

impl BackendMapping {
    /// Create a new backend mapping with a single component
    pub fn new(component: &str, import: Option<&str>) -> Self {
        Self {
            component: component.to_string(),
            import: import.map(|s| s.to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        }
    }

    /// Create a backend mapping with multiple components from the same import
    pub fn with_components(components: &[&str], import: &str) -> Self {
        if components.is_empty() {
            return Self::new("", Some(import));
        }
        Self {
            component: components[0].to_string(),
            import: Some(import.to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: components[1..].iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Add a property mapping
    pub fn with_prop(mut self, aura_prop: &str, backend_prop: &str) -> Self {
        self.props.insert(aura_prop.to_string(), backend_prop.to_string());
        self
    }

    /// Add an event mapping
    pub fn with_event(mut self, aura_event: &str, backend_event: &str) -> Self {
        self.events.insert(aura_event.to_string(), backend_event.to_string());
        self
    }

    /// Add extra components
    pub fn with_extra_components(mut self, components: &[&str]) -> Self {
        self.extra_components.extend(components.iter().map(|s| s.to_string()));
        self
    }

    /// Get all component names (primary + extras)
    pub fn all_components(&self) -> Vec<&str> {
        let mut result = vec![self.component.as_str()];
        result.extend(self.extra_components.iter().map(|s| s.as_str()));
        result
    }

    /// Get primary component name
    pub fn primary_component(&self) -> &str {
        &self.component
    }
}

impl Default for BackendMapping {
    fn default() -> Self {
        Self {
            component: String::new(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
            extra_components: Vec::new(),
        }
    }
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
    /// Alias tag names (e.g., "col" for "Column")
    pub aliases: Vec<String>,
    /// Backend-specific mappings
    pub backends: HashMap<String, BackendMapping>,
    /// Default props from view block (e.g., align: "center", arrange: "center")
    pub default_props: HashMap<String, String>,
}

impl WidgetSpec {
    /// Create a new widget spec
    pub fn new(name: &str, category: WidgetCategory) -> Self {
        Self {
            name: name.to_string(),
            category,
            primary_prop: None,
            has_children: false,
            aliases: Vec::new(),
            backends: HashMap::new(),
            default_props: HashMap::new(),
        }
    }

    /// Add an alias for this widget
    pub fn with_alias(mut self, alias: &str) -> Self {
        self.aliases.push(alias.to_lowercase());
        self
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
            aliases: Vec::new(),
            backends: HashMap::new(),
            default_props: HashMap::new(),
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
        assert!(spec.aliases.is_empty());
        assert!(spec.backends.is_empty());
    }

    #[test]
    fn test_widget_spec_with_alias() {
        let spec = WidgetSpec::new("Column", WidgetCategory::Layout)
            .with_alias("col");
        assert_eq!(spec.name, "Column");
        assert_eq!(spec.aliases, vec!["col"]);
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
            extra_components: Vec::new(),
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
