//! Component Registry
//!
//! Maps AURA element tags to platform-specific components.
//! This registry provides unified component mapping for Vue, Jet, and Tauri generators.
//!
//! ## Usage
//!
//! ```rust
//! use crate::ui_gen::shared::registry::ComponentRegistry;
//!
//! let registry = ComponentRegistry::new();
//! let mapping = registry.get("button").unwrap();
//!
//! // Vue output
//! println!("Vue component: {}", mapping.vue.component);
//!
//! // Jet output
//! println!("Jet composable: {}", mapping.jet.composable);
//! ```

use std::collections::HashMap;

/// Component registry for mapping AURA tags to platform components
pub struct ComponentRegistry {
    mappings: HashMap<String, ComponentMapping>,
}

/// Mapping for a single AURA component
#[derive(Debug, Clone)]
pub struct ComponentMapping {
    /// AURA tag name
    pub tag: String,
    /// Vue-specific mapping
    pub vue: VueMapping,
    /// Jetpack Compose mapping
    pub jet: JetMapping,
    /// Component category
    pub category: ComponentCategory,
}

/// Vue component mapping
#[derive(Debug, Clone)]
pub struct VueMapping {
    /// Import path (for shadcn-vue components)
    pub import: Option<String>,
    /// Component name
    pub component: String,
    /// Property mappings: AURA prop -> Vue prop
    pub props: HashMap<String, String>,
    /// Event mappings: AURA event -> Vue event
    pub events: HashMap<String, String>,
}

/// Jetpack Compose mapping
#[derive(Debug, Clone)]
pub struct JetMapping {
    /// Import path
    pub import: String,
    /// Composable function name
    pub composable: String,
    /// Property mappings: AURA prop -> Compose parameter
    pub props: HashMap<String, String>,
    /// Properties that should be applied via Modifier
    pub modifier_props: Vec<String>,
    /// Event mappings: AURA event -> Compose callback
    pub events: HashMap<String, String>,
}

/// Component category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentCategory {
    Layout,
    Form,
    Display,
    Navigation,
    Feedback,
    Overlay,
    Data,
}

impl ComponentRegistry {
    /// Create a new component registry with default mappings
    pub fn new() -> Self {
        let mut registry = Self {
            mappings: HashMap::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Register default component mappings
    fn register_defaults(&mut self) {
        // === Layout Components ===
        self.register_layout_components();

        // === Form Components ===
        self.register_form_components();

        // === Display Components ===
        self.register_display_components();

        // === Navigation Components ===
        self.register_navigation_components();

        // === Feedback Components ===
        self.register_feedback_components();

        // === Overlay Components ===
        self.register_overlay_components();

        // === Data Components ===
        self.register_data_components();
    }

    fn register_layout_components(&mut self) {
        // Column (flex-col)
        self.register(ComponentMapping {
            tag: "col".to_string(),
            vue: VueMapping {
                import: None,
                component: "div".to_string(),
                props: vec![("class", "class")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.foundation.layout.Column".to_string(),
                composable: "Column".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Layout,
        });

        // Row (flex-row)
        self.register(ComponentMapping {
            tag: "row".to_string(),
            vue: VueMapping {
                import: None,
                component: "div".to_string(),
                props: vec![("class", "class")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.foundation.layout.Row".to_string(),
                composable: "Row".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Layout,
        });

        // Box/Container
        self.register(ComponentMapping {
            tag: "box".to_string(),
            vue: VueMapping {
                import: None,
                component: "div".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.foundation.layout.Box".to_string(),
                composable: "Box".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Layout,
        });

        // Card
        self.register(ComponentMapping {
            tag: "card".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/card/Card".to_string()),
                component: "Card".to_string(),
                props: vec![("variant", "variant")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.Card".to_string(),
                composable: "Card".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Layout,
        });

        // ScrollArea
        self.register(ComponentMapping {
            tag: "scroll".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/scroll-area/ScrollArea".to_string()),
                component: "ScrollArea".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.foundation.verticalScroll".to_string(),
                composable: "verticalScroll".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Layout,
        });
    }

    fn register_form_components(&mut self) {
        // Button
        self.register(ComponentMapping {
            tag: "button".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/button/Button".to_string()),
                component: "Button".to_string(),
                props: vec![
                    ("text", "text"),
                    ("variant", "variant"),
                    ("size", "size"),
                    ("disabled", "disabled"),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
                events: vec![("onclick", "onClick")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.Button".to_string(),
                composable: "Button".to_string(),
                props: vec![("disabled", "enabled")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec!["class".to_string()],
                events: vec![("onclick", "onClick")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            category: ComponentCategory::Form,
        });

        // Input
        self.register(ComponentMapping {
            tag: "input".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/input/Input".to_string()),
                component: "Input".to_string(),
                props: vec![
                    ("placeholder", "placeholder"),
                    ("type", "type"),
                    ("disabled", "disabled"),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
                events: vec![("oninput", "onInput")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.OutlinedTextField".to_string(),
                composable: "OutlinedTextField".to_string(),
                props: vec![
                    ("placeholder", "placeholder"),
                    ("value", "value"),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
                modifier_props: vec!["class".to_string()],
                events: vec![("oninput", "onValueChange")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            category: ComponentCategory::Form,
        });

        // Textarea
        self.register(ComponentMapping {
            tag: "textarea".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/textarea/Textarea".to_string()),
                component: "Textarea".to_string(),
                props: vec![
                    ("placeholder", "placeholder"),
                    ("rows", "rows"),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.OutlinedTextField".to_string(),
                composable: "OutlinedTextField".to_string(),
                props: vec![
                    ("placeholder", "placeholder"),
                    ("value", "value"),
                ]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
                modifier_props: vec!["class".to_string()],
                events: vec![("oninput", "onValueChange")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            category: ComponentCategory::Form,
        });

        // Checkbox
        self.register(ComponentMapping {
            tag: "checkbox".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/checkbox/Checkbox".to_string()),
                component: "Checkbox".to_string(),
                props: vec![("disabled", "disabled")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.Checkbox".to_string(),
                composable: "Checkbox".to_string(),
                props: vec![("checked", "checked")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec![],
                events: vec![("onchange", "onCheckedChange")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            category: ComponentCategory::Form,
        });

        // Switch/Toggle
        self.register(ComponentMapping {
            tag: "switch".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/switch/Switch".to_string()),
                component: "Switch".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.Switch".to_string(),
                composable: "Switch".to_string(),
                props: vec![("checked", "checked")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec![],
                events: vec![("onchange", "onCheckedChange")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            category: ComponentCategory::Form,
        });

        // Select
        self.register(ComponentMapping {
            tag: "select".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/select/Select".to_string()),
                component: "Select".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.ExposedDropdownMenuBox".to_string(),
                composable: "ExposedDropdownMenuBox".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Form,
        });

        // Slider
        self.register(ComponentMapping {
            tag: "slider".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/slider/Slider".to_string()),
                component: "Slider".to_string(),
                props: vec![("min", "min"), ("max", "max")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.Slider".to_string(),
                composable: "Slider".to_string(),
                props: vec![("value", "value"), ("min", "min"), ("max", "max")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec![],
                events: vec![("onchange", "onValueChange")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            category: ComponentCategory::Form,
        });

        // RadioGroup
        self.register(ComponentMapping {
            tag: "radiogroup".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/radio-group/RadioGroup".to_string()),
                component: "RadioGroup".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.RadioButton".to_string(),
                composable: "RadioButton".to_string(),
                props: vec![("selected", "selected")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec![],
                events: vec![("onchange", "onClick")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            category: ComponentCategory::Form,
        });
    }

    fn register_display_components(&mut self) {
        // Text
        self.register(ComponentMapping {
            tag: "text".to_string(),
            vue: VueMapping {
                import: None,
                component: "span".to_string(),
                props: vec![("text", "text")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.Text".to_string(),
                composable: "Text".to_string(),
                props: vec![("text", "text")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Display,
        });

        // Typography headings
        for (tag, size) in [("h1", 32.0), ("h2", 28.0), ("h3", 24.0), ("h4", 20.0), ("h5", 18.0), ("h6", 16.0)] {
            self.register(ComponentMapping {
                tag: tag.to_string(),
                vue: VueMapping {
                    import: None,
                    component: tag.to_string(),
                    props: vec![("text", "text")]
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                    events: HashMap::new(),
                },
                jet: JetMapping {
                    import: "androidx.compose.material3.Text".to_string(),
                    composable: "Text".to_string(),
                    props: vec![("text", "text")]
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect(),
                    modifier_props: vec!["class".to_string()],
                    events: HashMap::new(),
                },
                category: ComponentCategory::Display,
            });
        }

        // Avatar
        self.register(ComponentMapping {
            tag: "avatar".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/avatar/Avatar".to_string()),
                component: "Avatar".to_string(),
                props: vec![("src", "src"), ("name", "alt")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.foundation.Image".to_string(),
                composable: "Image".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Display,
        });

        // Badge
        self.register(ComponentMapping {
            tag: "badge".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/badge/Badge".to_string()),
                component: "Badge".to_string(),
                props: vec![("variant", "variant")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.Badge".to_string(),
                composable: "Badge".to_string(),
                props: HashMap::new(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Display,
        });

        // Image
        self.register(ComponentMapping {
            tag: "image".to_string(),
            vue: VueMapping {
                import: None,
                component: "img".to_string(),
                props: vec![("src", "src"), ("alt", "alt")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.foundation.Image".to_string(),
                composable: "Image".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Display,
        });

        // Icon
        self.register(ComponentMapping {
            tag: "icon".to_string(),
            vue: VueMapping {
                import: None,
                component: "i".to_string(),
                props: vec![("name", "class")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material.icons.Icons".to_string(),
                composable: "Icon".to_string(),
                props: HashMap::new(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Display,
        });

        // Divider/Separator
        self.register(ComponentMapping {
            tag: "divider".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/separator/Separator".to_string()),
                component: "Separator".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.HorizontalDivider".to_string(),
                composable: "HorizontalDivider".to_string(),
                props: HashMap::new(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Display,
        });
    }

    fn register_navigation_components(&mut self) {
        // Tabs
        self.register(ComponentMapping {
            tag: "tabs".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/tabs/Tabs".to_string()),
                component: "Tabs".to_string(),
                props: vec![("value", "modelValue")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.TabRow".to_string(),
                composable: "TabRow".to_string(),
                props: vec![("selected", "selectedTabIndex")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Navigation,
        });

        // Tab
        self.register(ComponentMapping {
            tag: "tab".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/tabs/TabsTrigger".to_string()),
                component: "TabsTrigger".to_string(),
                props: vec![("value", "value")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.Tab".to_string(),
                composable: "Tab".to_string(),
                props: vec![("selected", "selected")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec![],
                events: vec![("onclick", "onClick")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            category: ComponentCategory::Navigation,
        });
    }

    fn register_feedback_components(&mut self) {
        // Progress
        self.register(ComponentMapping {
            tag: "progress".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/progress/Progress".to_string()),
                component: "Progress".to_string(),
                props: vec![("value", "modelValue")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.LinearProgressIndicator".to_string(),
                composable: "LinearProgressIndicator".to_string(),
                props: vec![("value", "progress")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Feedback,
        });

        // Spinner
        self.register(ComponentMapping {
            tag: "spinner".to_string(),
            vue: VueMapping {
                import: None,
                component: "div".to_string(), // CSS spinner
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.CircularProgressIndicator".to_string(),
                composable: "CircularProgressIndicator".to_string(),
                props: HashMap::new(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Feedback,
        });

        // Toast/Snackbar
        self.register(ComponentMapping {
            tag: "toast".to_string(),
            vue: VueMapping {
                import: Some("sonner".to_string()),
                component: "toast".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.Snackbar".to_string(),
                composable: "Snackbar".to_string(),
                props: HashMap::new(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Feedback,
        });
    }

    fn register_overlay_components(&mut self) {
        // Dialog
        self.register(ComponentMapping {
            tag: "dialog".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/dialog/Dialog".to_string()),
                component: "Dialog".to_string(),
                props: vec![("open", "open")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.AlertDialog".to_string(),
                composable: "AlertDialog".to_string(),
                props: HashMap::new(),
                modifier_props: vec![],
                events: vec![("onclose", "onDismissRequest")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
            },
            category: ComponentCategory::Overlay,
        });

        // Modal (alias for Dialog)
        self.register(ComponentMapping {
            tag: "modal".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/dialog/Dialog".to_string()),
                component: "Dialog".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.AlertDialog".to_string(),
                composable: "AlertDialog".to_string(),
                props: HashMap::new(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Overlay,
        });

        // Tooltip
        self.register(ComponentMapping {
            tag: "tooltip".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/tooltip/Tooltip".to_string()),
                component: "Tooltip".to_string(),
                props: vec![("content", "content")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.TooltipBox".to_string(),
                composable: "TooltipBox".to_string(),
                props: HashMap::new(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Overlay,
        });

        // Popover
        self.register(ComponentMapping {
            tag: "popover".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/popover/Popover".to_string()),
                component: "Popover".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.material3.DropdownMenu".to_string(),
                composable: "DropdownMenu".to_string(),
                props: vec![("open", "expanded")]
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                modifier_props: vec![],
                events: HashMap::new(),
            },
            category: ComponentCategory::Overlay,
        });
    }

    fn register_data_components(&mut self) {
        // Table
        self.register(ComponentMapping {
            tag: "table".to_string(),
            vue: VueMapping {
                import: Some("@/components/ui/table/Table".to_string()),
                component: "Table".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.foundation.lazy.LazyColumn".to_string(),
                composable: "LazyColumn".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Data,
        });

        // List
        self.register(ComponentMapping {
            tag: "list".to_string(),
            vue: VueMapping {
                import: None,
                component: "ul".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
            },
            jet: JetMapping {
                import: "androidx.compose.foundation.lazy.LazyColumn".to_string(),
                composable: "LazyColumn".to_string(),
                props: HashMap::new(),
                modifier_props: vec!["class".to_string()],
                events: HashMap::new(),
            },
            category: ComponentCategory::Data,
        });
    }

    /// Register a component mapping
    pub fn register(&mut self, mapping: ComponentMapping) {
        self.mappings.insert(mapping.tag.clone(), mapping);
    }

    /// Get a component mapping by tag
    pub fn get(&self, tag: &str) -> Option<&ComponentMapping> {
        self.mappings.get(tag)
    }

    /// Check if a tag is supported
    pub fn is_supported(&self, tag: &str) -> bool {
        self.mappings.contains_key(tag)
    }

    /// Get all supported tags
    pub fn supported_tags(&self) -> Vec<&str> {
        self.mappings.keys().map(|s| s.as_str()).collect()
    }

    /// Get tags by category
    pub fn by_category(&self, category: ComponentCategory) -> Vec<&ComponentMapping> {
        self.mappings
            .values()
            .filter(|m| m.category == category)
            .collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_button() {
        let registry = ComponentRegistry::new();
        let mapping = registry.get("button").unwrap();

        assert_eq!(mapping.vue.component, "Button");
        assert_eq!(mapping.jet.composable, "Button");
        assert_eq!(mapping.category, ComponentCategory::Form);
    }

    #[test]
    fn test_registry_input() {
        let registry = ComponentRegistry::new();
        let mapping = registry.get("input").unwrap();

        assert_eq!(mapping.vue.component, "Input");
        assert_eq!(mapping.jet.composable, "OutlinedTextField");
    }

    #[test]
    fn test_registry_layout() {
        let registry = ComponentRegistry::new();

        assert!(registry.get("col").is_some());
        assert!(registry.get("row").is_some());
        assert!(registry.get("box").is_some());
    }

    #[test]
    fn test_is_supported() {
        let registry = ComponentRegistry::new();

        assert!(registry.is_supported("button"));
        assert!(registry.is_supported("input"));
        assert!(!registry.is_supported("nonexistent"));
    }

    #[test]
    fn test_by_category() {
        let registry = ComponentRegistry::new();
        let forms = registry.by_category(ComponentCategory::Form);

        assert!(!forms.is_empty());
        assert!(forms.iter().any(|m| m.tag == "button"));
    }
}
