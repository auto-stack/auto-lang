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
        self.register_form_widgets();
        self.register_display_widgets();
        self.register_navigation_widgets();
        self.register_semantic_widgets();
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

    fn register_form_widgets(&mut self) {
        // Button
        let mut button = WidgetSpec::new("Button", WidgetCategory::Form)
            .with_alias("button");
        button.primary_prop = Some("text".to_string());
        button.backends.insert("ark".to_string(), BackendMapping {
            component: "Button".to_string(),
            import: Some("@kit.ArkUI".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        button.backends.insert("jet".to_string(), BackendMapping {
            component: "Button".to_string(),
            import: Some("androidx.compose.material3.Button".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        button.backends.insert("vue".to_string(), BackendMapping {
            component: "Button".to_string(),
            import: Some("@/components/ui/button/Button".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(button);

        // Input (TextInput in Ark)
        let mut input = WidgetSpec::new("Input", WidgetCategory::Form)
            .with_alias("input");
        input.backends.insert("ark".to_string(), BackendMapping {
            component: "TextInput".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        input.backends.insert("jet".to_string(), BackendMapping {
            component: "OutlinedTextField".to_string(),
            import: Some("androidx.compose.material3.OutlinedTextField".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        input.backends.insert("vue".to_string(), BackendMapping {
            component: "Input".to_string(),
            import: Some("@/components/ui/input/Input".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(input);
    }

    fn register_display_widgets(&mut self) {
        // Text
        let mut text = WidgetSpec::new("Text", WidgetCategory::Display)
            .with_alias("text");
        text.primary_prop = Some("text".to_string());
        text.backends.insert("ark".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        text.backends.insert("jet".to_string(), BackendMapping {
            component: "Text".to_string(),
            import: Some("androidx.compose.material3.Text".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        text.backends.insert("vue".to_string(), BackendMapping {
            component: "span".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(text);

        // Image
        let mut image = WidgetSpec::new("Image", WidgetCategory::Display)
            .with_alias("image");
        image.backends.insert("ark".to_string(), BackendMapping {
            component: "Image".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        image.backends.insert("jet".to_string(), BackendMapping {
            component: "Image".to_string(),
            import: Some("androidx.compose.foundation.Image".to_string()),
            props: HashMap::new(),
            events: HashMap::new(),
        });
        image.backends.insert("vue".to_string(), BackendMapping {
            component: "img".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(image);
    }

    fn register_navigation_widgets(&mut self) {
        // Swiper
        let mut swiper = WidgetSpec::new("Swiper", WidgetCategory::Navigation)
            .with_alias("swiper");
        swiper.has_children = true;
        swiper.backends.insert("ark".to_string(), BackendMapping {
            component: "Swiper".to_string(),
            import: None,
            props: HashMap::new(),
            events: HashMap::new(),
        });
        self.register(swiper);
    }

    fn register_semantic_widgets(&mut self) {
        // Semantic HTML elements map to Column in Ark
        for tag in ["header", "footer", "nav", "main", "aside", "article", "section"] {
            let mut widget = WidgetSpec::new(tag, WidgetCategory::Semantic);
            widget.has_children = true;
            widget.backends.insert("ark".to_string(), BackendMapping {
                component: "Column".to_string(),
                import: None,
                props: HashMap::new(),
                events: HashMap::new(),
            });
            self.register(widget);
        }

        // Heading elements map to Text
        for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
            let mut widget = WidgetSpec::new(tag, WidgetCategory::Display);
            widget.primary_prop = Some("text".to_string());
            widget.backends.insert("ark".to_string(), BackendMapping {
                component: "Text".to_string(),
                import: None,
                props: HashMap::new(),
                events: HashMap::new(),
            });
            self.register(widget);
        }
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

    #[test]
    fn test_default_widgets_button() {
        let registry = WidgetRegistry::with_defaults();
        let button = registry.get("button").unwrap();
        assert_eq!(button.name, "Button");
        assert_eq!(button.category, WidgetCategory::Form);

        let ark_mapping = button.backend("ark").unwrap();
        assert_eq!(ark_mapping.component, "Button");
        assert_eq!(ark_mapping.import, Some("@kit.ArkUI".to_string()));
    }

    #[test]
    fn test_default_widgets_text() {
        let registry = WidgetRegistry::with_defaults();
        let text = registry.get("text").unwrap();
        assert_eq!(text.name, "Text");
        assert_eq!(text.category, WidgetCategory::Display);
    }

    #[test]
    fn test_default_widgets_image() {
        let registry = WidgetRegistry::with_defaults();
        let image = registry.get("image").unwrap();
        assert_eq!(image.name, "Image");

        let ark_mapping = image.backend("ark").unwrap();
        assert_eq!(ark_mapping.component, "Image");
    }

    #[test]
    fn test_semantic_widgets_map_to_column() {
        let registry = WidgetRegistry::with_defaults();
        for tag in ["header", "footer", "nav", "main"] {
            let widget = registry.get(tag).unwrap();
            let ark = widget.backend("ark").unwrap();
            assert_eq!(ark.component, "Column", "{} should map to Column", tag);
        }
    }
}
