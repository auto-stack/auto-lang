//! Material3 Component Registry
//!
//! Maps AURA element tags to Jetpack Compose Material3 components.

use std::collections::HashMap;

/// Maps AURA element tags to Material3 component imports
pub struct Material3Registry {
    /// Component imports: tag -> (package, component_names)
    components: HashMap<&'static str, (&'static str, Vec<&'static str>)>,
}

impl Material3Registry {
    /// Create registry with all Material3 component mappings
    pub fn new() -> Self {
        let mut components = HashMap::new();

        // === Layout Elements ===
        components.insert("col",
            ("androidx.compose.foundation.layout", vec!["Column", "Arrangement"]));
        components.insert("row",
            ("androidx.compose.foundation.layout", vec!["Row", "Arrangement"]));
        components.insert("box",
            ("androidx.compose.foundation.layout", vec!["Box"]));
        components.insert("container",
            ("androidx.compose.foundation.layout", vec!["Box"]));
        components.insert("grid",
            ("androidx.compose.foundation.lazy.grid", vec!["LazyVerticalGrid", "GridCells"]));
        components.insert("scroll",
            ("androidx.compose.foundation", vec!["verticalScroll", "ScrollState"]));

        // === Content Elements ===
        components.insert("button",
            ("androidx.compose.material3", vec!["Button", "OutlinedButton", "TextButton"]));
        components.insert("input",
            ("androidx.compose.material3", vec!["TextField", "OutlinedTextField"]));
        components.insert("textarea",
            ("androidx.compose.material3", vec!["TextField"]));
        components.insert("checkbox",
            ("androidx.compose.material3", vec!["Checkbox"]));
        components.insert("toggle",
            ("androidx.compose.material3", vec!["Switch"]));
        components.insert("switch",
            ("androidx.compose.material3", vec!["Switch"]));
        components.insert("select",
            ("androidx.compose.material3", vec!["ExposedDropdownMenuBox"]));

        // === Typography Elements ===
        components.insert("text",
            ("androidx.compose.material3", vec!["Text"]));
        components.insert("span",
            ("androidx.compose.material3", vec!["Text"]));
        components.insert("p",
            ("androidx.compose.material3", vec!["Text"]));
        components.insert("h1",
            ("androidx.compose.material3", vec!["Text", "MaterialTheme"]));
        components.insert("h2",
            ("androidx.compose.material3", vec!["Text", "MaterialTheme"]));
        components.insert("h3",
            ("androidx.compose.material3", vec!["Text", "MaterialTheme"]));
        components.insert("h4",
            ("androidx.compose.material3", vec!["Text", "MaterialTheme"]));
        components.insert("h5",
            ("androidx.compose.material3", vec!["Text", "MaterialTheme"]));
        components.insert("h6",
            ("androidx.compose.material3", vec!["Text", "MaterialTheme"]));

        // === Navigation Elements ===
        components.insert("tabs",
            ("androidx.compose.material3", vec!["TabRow", "Tab"]));
        components.insert("tab",
            ("androidx.compose.material3", vec!["Tab", "TabRow"]));

        // === Overlay Elements ===
        components.insert("modal",
            ("androidx.compose.material3", vec!["AlertDialog"]));
        components.insert("tooltip",
            ("androidx.compose.material3", vec!["TooltipBox", "PlainTooltip"]));
        components.insert("dialog",
            ("androidx.compose.material3", vec!["AlertDialog"]));

        // === Feedback Elements ===
        components.insert("progress",
            ("androidx.compose.material3", vec!["LinearProgressIndicator", "CircularProgressIndicator"]));
        components.insert("badge",
            ("androidx.compose.material3", vec!["Badge"]));
        components.insert("spinner",
            ("androidx.compose.material3", vec!["CircularProgressIndicator"]));

        // === Display Elements ===
        components.insert("card",
            ("androidx.compose.material3", vec!["Card", "ElevatedCard", "OutlinedCard"]));
        components.insert("avatar",
            ("androidx.compose.foundation", vec!["Image"]));
        components.insert("image",
            ("androidx.compose.foundation", vec!["Image"]));
        components.insert("icon",
            ("androidx.compose.material.icons", vec!["Icons"]));
        components.insert("divider",
            ("androidx.compose.material3", vec!["HorizontalDivider"]));
        components.insert("separator",
            ("androidx.compose.material3", vec!["HorizontalDivider"]));

        // === Form Elements ===
        components.insert("slider",
            ("androidx.compose.material3", vec!["Slider"]));
        components.insert("radio",
            ("androidx.compose.material3", vec!["RadioButton"]));
        components.insert("radiogroup",
            ("androidx.compose.material3", vec!["RadioButton"]));

        // === List Elements ===
        components.insert("list",
            ("androidx.compose.foundation.lazy", vec!["LazyColumn"]));
        components.insert("list_item",
            ("androidx.compose.material3", vec!["ListItem"]));

        // === Table Elements (using LazyColumn) ===
        components.insert("table",
            ("androidx.compose.foundation.lazy", vec!["LazyColumn"]));

        Self { components }
    }

    /// Get component mapping for a tag
    pub fn get(&self, tag: &str) -> Option<(&'static str, &Vec<&'static str>)> {
        self.components.get_key_value(tag).map(|(_, v)| (v.0, &v.1))
    }

    /// Get the primary component name for a tag
    pub fn primary_component(&self, tag: &str) -> Option<&'static str> {
        self.components.get(tag).and_then(|(_, comps)| comps.first().copied())
    }

    /// Get all component names for a tag
    pub fn all_components(&self, tag: &str) -> Option<&Vec<&'static str>> {
        self.components.get(tag).map(|(_, comps)| comps)
    }

    /// Check if a tag is supported
    pub fn is_supported(&self, tag: &str) -> bool {
        self.components.contains_key(tag)
    }
}

impl Default for Material3Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_button_mapping() {
        let registry = Material3Registry::new();
        let (module, components) = registry.get("button").unwrap();
        assert_eq!(module, "androidx.compose.material3");
        assert!(components.contains(&"Button"));
    }

    #[test]
    fn test_registry_input_mapping() {
        let registry = Material3Registry::new();
        let (module, components) = registry.get("input").unwrap();
        assert_eq!(module, "androidx.compose.material3");
        assert!(components.contains(&"TextField"));
    }

    #[test]
    fn test_registry_col_mapping() {
        let registry = Material3Registry::new();
        let (module, components) = registry.get("col").unwrap();
        assert_eq!(module, "androidx.compose.foundation.layout");
        assert!(components.contains(&"Column"));
    }

    #[test]
    fn test_registry_unsupported() {
        let registry = Material3Registry::new();
        assert!(!registry.is_supported("nonexistent"));
    }
}
