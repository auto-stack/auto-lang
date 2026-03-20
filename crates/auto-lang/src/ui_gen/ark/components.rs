//! ArkTS Component Registry
//!
//! Maps AURA tags to ArkTS components.

use std::collections::HashMap;

/// Registry mapping AURA tags to ArkTS component templates
pub struct ArkComponentRegistry {
    /// Map from AURA tag to ArkTS component name
    components: HashMap<String, ArkComponent>,
}

/// ArkTS component definition
#[derive(Debug, Clone)]
pub struct ArkComponent {
    /// Component name in ArkTS (e.g., "Column", "Text")
    pub name: String,
    /// Whether component has children
    pub has_children: bool,
    /// Whether component has content (like Text)
    pub has_content: bool,
}

impl ArkComponentRegistry {
    /// Create a new registry with default components
    pub fn new() -> Self {
        let mut registry = Self {
            components: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Register default ArkTS components
    fn register_defaults(&mut self) {
        // Layout components
        self.register("col", ArkComponent {
            name: "Column".to_string(),
            has_children: true,
            has_content: false,
        });

        self.register("row", ArkComponent {
            name: "Row".to_string(),
            has_children: true,
            has_content: false,
        });

        // Basic components
        self.register("text", ArkComponent {
            name: "Text".to_string(),
            has_children: false,
            has_content: true,
        });

        self.register("button", ArkComponent {
            name: "Button".to_string(),
            has_children: false,
            has_content: true,
        });

        self.register("input", ArkComponent {
            name: "TextInput".to_string(),
            has_children: false,
            has_content: false,
        });

        self.register("checkbox", ArkComponent {
            name: "Checkbox".to_string(),
            has_children: false,
            has_content: false,
        });

        // Additional common components
        self.register("image", ArkComponent {
            name: "Image".to_string(),
            has_children: false,
            has_content: false,
        });

        self.register("list", ArkComponent {
            name: "List".to_string(),
            has_children: true,
            has_content: false,
        });

        self.register("scroll", ArkComponent {
            name: "Scroll".to_string(),
            has_children: true,
            has_content: false,
        });

        self.register("divider", ArkComponent {
            name: "Divider".to_string(),
            has_children: false,
            has_content: false,
        });

        self.register("progress", ArkComponent {
            name: "Progress".to_string(),
            has_children: false,
            has_content: false,
        });

        self.register("slider", ArkComponent {
            name: "Slider".to_string(),
            has_children: false,
            has_content: false,
        });

        self.register("switch", ArkComponent {
            name: "Toggle".to_string(),
            has_children: false,
            has_content: false,
        });

        // Semantic HTML elements (map to Column)
        for tag in ["header", "footer", "nav", "main", "aside", "article", "section"] {
            self.register(tag, ArkComponent {
                name: "Column".to_string(),
                has_children: true,
                has_content: false,
            });
        }

        // Heading elements (map to Text with font size)
        for tag in ["h1", "h2", "h3", "h4", "h5", "h6"] {
            self.register(tag, ArkComponent {
                name: "Text".to_string(),
                has_children: false,
                has_content: true,
            });
        }
    }

    /// Register a component
    pub fn register(&mut self, tag: &str, component: ArkComponent) {
        self.components.insert(tag.to_string(), component);
    }

    /// Look up a component by AURA tag
    pub fn get(&self, tag: &str) -> Option<&ArkComponent> {
        self.components.get(tag)
    }

    /// Check if a tag is a known component
    pub fn is_component(&self, tag: &str) -> bool {
        self.components.contains_key(tag)
    }
}

impl Default for ArkComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_default_components() {
        let registry = ArkComponentRegistry::new();

        assert!(registry.get("col").is_some());
        assert!(registry.get("row").is_some());
        assert!(registry.get("text").is_some());
        assert!(registry.get("button").is_some());
    }

    #[test]
    fn test_column_has_children() {
        let registry = ArkComponentRegistry::new();
        let col = registry.get("col").unwrap();

        assert_eq!(col.name, "Column");
        assert!(col.has_children);
        assert!(!col.has_content);
    }

    #[test]
    fn test_text_has_content() {
        let registry = ArkComponentRegistry::new();
        let text = registry.get("text").unwrap();

        assert_eq!(text.name, "Text");
        assert!(!text.has_children);
        assert!(text.has_content);
    }

    #[test]
    fn test_unknown_tag_returns_none() {
        let registry = ArkComponentRegistry::new();

        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_register_custom_component() {
        let mut registry = ArkComponentRegistry::new();

        registry.register("custom", ArkComponent {
            name: "CustomWidget".to_string(),
            has_children: true,
            has_content: false,
        });

        let custom = registry.get("custom").unwrap();
        assert_eq!(custom.name, "CustomWidget");
    }

    #[test]
    fn test_semantic_elements_map_to_column() {
        let registry = ArkComponentRegistry::new();

        // Semantic HTML elements should map to Column
        for tag in ["header", "footer", "nav", "main", "aside", "article", "section"] {
            let component = registry.get(tag).unwrap();
            assert_eq!(component.name, "Column", "{} should map to Column", tag);
            assert!(component.has_children);
        }
    }
}
