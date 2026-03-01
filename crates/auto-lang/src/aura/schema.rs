//! AURA Widget Schema Definition
//!
//! This module defines the schema for AURA widgets, including:
//! - Valid element tags and their categories
//! - Props each element supports
//! - Type constraints for props
//! - Widget block requirements
//!
//! Used for:
//! - Validation at parse time
//! - LSP features (completion, hover, diagnostics)
//! - Error messages with helpful suggestions

use std::collections::HashMap;

/// Element category for grouping and documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementCategory {
    /// Layout containers: col, row, grid, stack, scroll, container
    Layout,
    /// Interactive content: button, input, checkbox, toggle, link, textarea, select, option
    Content,
    /// Text display: h1-h6, p, span, code, pre
    Typography,
    /// List structures: list, list_item
    List,
    /// Data structures: table, thead, tbody, tr, th, td, tree, tree_item
    Data,
    /// Navigation elements: tabs, tab
    Navigation,
    /// Overlay elements: modal, tooltip
    Overlay,
    /// Form elements: slider, radio, radiogroup
    Form,
    /// Feedback elements: progress, badge, spinner
    Feedback,
    /// Display elements: card, avatar
    Display,
    /// Media elements: image, icon
    Media,
    /// Utility elements: divider, spacer
    Utility,
}

/// Type constraint for a prop value
#[derive(Debug, Clone, PartialEq)]
pub enum PropType {
    /// String literal: "hello"
    String,
    /// Integer number: 42
    Int,
    /// Floating point: 3.14
    Float,
    /// Boolean: true, false
    Bool,
    /// Color value: "#FF0000", "red"
    Color,
    /// Reference to state: .count
    StateRef,
    /// Reference to message: .Click
    MsgRef,
    /// Any expression: .count + 1
    Expr,
    /// Lambda/closure: |x| x * 2
    Closure,
    /// Class binding object: { active: .isSelected }
    ClassBinding,
    /// Interpolated string: `Hello ${.name}`
    Interpolated,
    /// One of a set of string values (enum-like)
    OneOf(Vec<&'static str>),
    /// Union of multiple types
    Union(Vec<PropType>),
}

impl PropType {
    /// Check if a type matches this constraint
    pub fn matches(&self, other: &PropType) -> bool {
        match self {
            PropType::Union(types) => types.iter().any(|t| t.matches(other)),
            PropType::OneOf(options) => {
                if let PropType::String = other {
                    true // String might match, validated at runtime
                } else {
                    false
                }
            }
            _ => self == other,
        }
    }

    /// Get human-readable name for the type
    pub fn name(&self) -> String {
        match self {
            PropType::String => "string".to_string(),
            PropType::Int => "int".to_string(),
            PropType::Float => "float".to_string(),
            PropType::Bool => "bool".to_string(),
            PropType::Color => "color".to_string(),
            PropType::StateRef => "state_ref".to_string(),
            PropType::MsgRef => "msg_ref".to_string(),
            PropType::Expr => "expr".to_string(),
            PropType::Closure => "closure".to_string(),
            PropType::ClassBinding => "class_binding".to_string(),
            PropType::Interpolated => "interpolated".to_string(),
            PropType::OneOf(options) => format!("one_of({})", options.join(" | ")),
            PropType::Union(types) => {
                let names: Vec<_> = types.iter().map(|t| t.name()).collect();
                names.join(" | ")
            }
        }
    }
}

/// Definition of an element prop
#[derive(Debug, Clone)]
pub struct PropDef {
    /// Prop name (e.g., "onclick", "text")
    pub name: &'static str,
    /// Type constraint
    pub type_: PropType,
    /// Whether this prop is required
    pub required: bool,
    /// Default value if not specified
    pub default: Option<&'static str>,
    /// Documentation for this prop
    pub description: &'static str,
}

/// Definition of a view element
#[derive(Debug, Clone)]
pub struct ElementDef {
    /// Element tag name (e.g., "button", "col")
    pub tag: &'static str,
    /// Element category
    pub category: ElementCategory,
    /// Props this element supports
    pub props: Vec<PropDef>,
    /// Whether this element can have children
    pub allows_children: bool,
    /// Documentation for this element
    pub description: &'static str,
}

impl ElementDef {
    /// Get a prop definition by name
    pub fn get_prop(&self, name: &str) -> Option<&PropDef> {
        self.props.iter().find(|p| p.name == name)
    }

    /// Get list of required props
    pub fn required_props(&self) -> Vec<&PropDef> {
        self.props.iter().filter(|p| p.required).collect()
    }

    /// Get list of optional props
    pub fn optional_props(&self) -> Vec<&PropDef> {
        self.props.iter().filter(|p| !p.required).collect()
    }
}

/// Widget block constraints
#[derive(Debug, Clone)]
pub struct WidgetBlockSchema {
    /// Blocks that must appear exactly once
    pub required: Vec<&'static str>,
    /// Blocks that are optional (0 or 1)
    pub optional: Vec<&'static str>,
}

impl WidgetBlockSchema {
    /// Check if a block name is valid for a widget
    pub fn is_valid_block(&self, name: &str) -> bool {
        self.required.contains(&name) || self.optional.contains(&name)
    }

    /// Check if a block is required
    pub fn is_required(&self, name: &str) -> bool {
        self.required.contains(&name)
    }
}

/// The complete AURA schema
pub struct AuraSchema {
    /// Element definitions by tag name
    pub elements: HashMap<&'static str, ElementDef>,
    /// Widget block constraints
    pub widget_blocks: WidgetBlockSchema,
}

impl AuraSchema {
    /// Create the standard AURA schema
    pub fn new() -> Self {
        let mut elements = HashMap::new();

        // Layout elements
        Self::add_layout_elements(&mut elements);

        // Content elements
        Self::add_content_elements(&mut elements);

        // Typography elements
        Self::add_typography_elements(&mut elements);

        // List elements
        Self::add_list_elements(&mut elements);

        // Media elements
        Self::add_media_elements(&mut elements);

        // Utility elements
        Self::add_utility_elements(&mut elements);

        // Feedback & Overlay elements (shadcn-vue components)
        Self::add_feedback_elements(&mut elements);

        AuraSchema {
            elements,
            widget_blocks: WidgetBlockSchema {
                required: vec!["msg", "model", "view"],
                optional: vec!["computed", "on"],
            },
        }
    }

    /// Get an element definition by tag
    pub fn get_element(&self, tag: &str) -> Option<&ElementDef> {
        self.elements.get(tag)
    }

    /// Check if a tag is a valid element
    pub fn is_valid_element(&self, tag: &str) -> bool {
        self.elements.contains_key(tag)
    }

    /// Get all element tags
    pub fn all_tags(&self) -> Vec<&'static str> {
        self.elements.keys().copied().collect()
    }

    /// Get elements by category
    pub fn elements_by_category(&self, category: ElementCategory) -> Vec<&ElementDef> {
        self.elements
            .values()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Find similar element tags (for error suggestions)
    pub fn suggest_similar(&self, tag: &str) -> Option<&'static str> {
        // Simple Levenshtein-like suggestion
        let mut best_match: Option<&'static str> = None;
        let mut best_score = usize::MAX;

        for known_tag in self.elements.keys() {
            let score = Self::levenshtein_distance(tag, known_tag);
            if score < best_score && score <= 3 {
                best_score = score;
                best_match = Some(*known_tag);
            }
        }

        best_match
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        for (i, row) in matrix.iter_mut().enumerate() {
            row[0] = i;
        }

        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        for (i, a_char) in a_chars.iter().enumerate() {
            for (j, b_char) in b_chars.iter().enumerate() {
                let cost = if a_char == b_char { 0 } else { 1 };
                matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                    .min(matrix[i + 1][j] + 1)
                    .min(matrix[i][j] + cost);
            }
        }

        matrix[a_len][b_len]
    }

    // Element registration helpers

    fn add_layout_elements(elements: &mut HashMap<&'static str, ElementDef>) {
        elements.insert("col", ElementDef {
            tag: "col",
            category: ElementCategory::Layout,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "gap", type_: PropType::Int, required: false, default: Some("0"), description: "Spacing between children" },
                PropDef { name: "padding", type_: PropType::Union(vec![PropType::Int, PropType::String]), required: false, default: Some("0"), description: "Inner padding" },
                PropDef { name: "align", type_: PropType::OneOf(vec!["start", "center", "end", "stretch"]), required: false, default: Some("start"), description: "Cross-axis alignment" },
            ],
            allows_children: true,
            description: "Vertical layout container",
        });

        elements.insert("row", ElementDef {
            tag: "row",
            category: ElementCategory::Layout,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "gap", type_: PropType::Int, required: false, default: Some("0"), description: "Spacing between children" },
                PropDef { name: "padding", type_: PropType::Union(vec![PropType::Int, PropType::String]), required: false, default: Some("0"), description: "Inner padding" },
                PropDef { name: "align", type_: PropType::OneOf(vec!["start", "center", "end", "stretch"]), required: false, default: Some("center"), description: "Cross-axis alignment" },
            ],
            allows_children: true,
            description: "Horizontal layout container",
        });

        elements.insert("grid", ElementDef {
            tag: "grid",
            category: ElementCategory::Layout,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "columns", type_: PropType::Int, required: false, default: Some("1"), description: "Number of columns" },
                PropDef { name: "gap", type_: PropType::Int, required: false, default: Some("0"), description: "Cell spacing" },
            ],
            allows_children: true,
            description: "Grid layout container",
        });

        elements.insert("scroll", ElementDef {
            tag: "scroll",
            category: ElementCategory::Layout,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "direction", type_: PropType::OneOf(vec!["vertical", "horizontal", "both"]), required: false, default: Some("vertical"), description: "Scroll direction" },
            ],
            allows_children: true,
            description: "Scrollable container",
        });

        elements.insert("container", ElementDef {
            tag: "container",
            category: ElementCategory::Layout,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "max_width", type_: PropType::Int, required: false, default: None, description: "Maximum width in pixels" },
                PropDef { name: "padding", type_: PropType::Union(vec![PropType::Int, PropType::String]), required: false, default: None, description: "Inner padding" },
            ],
            allows_children: true,
            description: "Generic container with optional constraints",
        });
    }

    fn add_content_elements(elements: &mut HashMap<&'static str, ElementDef>) {
        elements.insert("button", ElementDef {
            tag: "button",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Button label text" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Message to send when clicked" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "disabled", type_: PropType::Union(vec![PropType::Bool, PropType::StateRef]), required: false, default: Some("false"), description: "Whether button is disabled" },
            ],
            allows_children: false,
            description: "A clickable button element",
        });

        elements.insert("input", ElementDef {
            tag: "input",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Bound value (two-way binding)" },
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: None, description: "Placeholder text" },
                PropDef { name: "type", type_: PropType::OneOf(vec!["text", "password", "email", "number"]), required: false, default: Some("text"), description: "Input type" },
                PropDef { name: "onchange", type_: PropType::MsgRef, required: false, default: None, description: "Message on value change" },
                PropDef { name: "onenter", type_: PropType::MsgRef, required: false, default: None, description: "Message on Enter key" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "disabled", type_: PropType::Union(vec![PropType::Bool, PropType::StateRef]), required: false, default: Some("false"), description: "Whether input is disabled" },
            ],
            allows_children: false,
            description: "Text input field",
        });

        elements.insert("checkbox", ElementDef {
            tag: "checkbox",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "checked", type_: PropType::Union(vec![PropType::Bool, PropType::StateRef]), required: false, default: Some("false"), description: "Checked state" },
                PropDef { name: "onchange", type_: PropType::MsgRef, required: false, default: None, description: "Message on toggle" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "disabled", type_: PropType::Union(vec![PropType::Bool, PropType::StateRef]), required: false, default: Some("false"), description: "Whether checkbox is disabled" },
            ],
            allows_children: false,
            description: "Checkbox control",
        });

        elements.insert("toggle", ElementDef {
            tag: "toggle",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "checked", type_: PropType::Union(vec![PropType::Bool, PropType::StateRef]), required: false, default: Some("false"), description: "Checked state" },
                PropDef { name: "onchange", type_: PropType::MsgRef, required: false, default: None, description: "Message on toggle" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Toggle switch",
        });

        elements.insert("link", ElementDef {
            tag: "link",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "href", type_: PropType::String, required: true, default: None, description: "Link URL" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Link text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Hyperlink",
        });
    }

    fn add_typography_elements(elements: &mut HashMap<&'static str, ElementDef>) {
        // Add h1-h6
        for (tag, level) in [("h1", 1), ("h2", 2), ("h3", 3), ("h4", 4), ("h5", 5), ("h6", 6)] {
            elements.insert(tag, ElementDef {
                tag,
                category: ElementCategory::Typography,
                props: vec![
                    PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Heading text" },
                    PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                ],
                allows_children: false,
                description: Box::leak(format!("Level {} heading", level).into_boxed_str()),
            });
        }

        elements.insert("text", ElementDef {
            tag: "text",
            category: ElementCategory::Typography,
            props: vec![
                // Text content is inline, not a named prop
            ],
            allows_children: false,
            description: "Text content (literal or interpolated)",
        });

        elements.insert("p", ElementDef {
            tag: "p",
            category: ElementCategory::Typography,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Paragraph text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Paragraph text",
        });

        elements.insert("span", ElementDef {
            tag: "span",
            category: ElementCategory::Typography,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Span text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Inline text span",
        });
    }

    fn add_list_elements(elements: &mut HashMap<&'static str, ElementDef>) {
        elements.insert("list", ElementDef {
            tag: "list",
            category: ElementCategory::List,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Generic list container",
        });

        elements.insert("list_item", ElementDef {
            tag: "list_item",
            category: ElementCategory::List,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Message when clicked" },
            ],
            allows_children: true,
            description: "List item",
        });
    }

    fn add_media_elements(elements: &mut HashMap<&'static str, ElementDef>) {
        elements.insert("image", ElementDef {
            tag: "image",
            category: ElementCategory::Media,
            props: vec![
                PropDef { name: "src", type_: PropType::String, required: true, default: None, description: "Image URL" },
                PropDef { name: "alt", type_: PropType::String, required: false, default: Some(""), description: "Alt text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "fit", type_: PropType::OneOf(vec!["cover", "contain", "fill", "none"]), required: false, default: Some("cover"), description: "Object fit mode" },
            ],
            allows_children: false,
            description: "Image display",
        });

        elements.insert("icon", ElementDef {
            tag: "icon",
            category: ElementCategory::Media,
            props: vec![
                PropDef { name: "name", type_: PropType::String, required: true, default: None, description: "Icon name" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "size", type_: PropType::Int, required: false, default: Some("24"), description: "Icon size in pixels" },
            ],
            allows_children: false,
            description: "Icon display",
        });
    }

    fn add_utility_elements(elements: &mut HashMap<&'static str, ElementDef>) {
        elements.insert("divider", ElementDef {
            tag: "divider",
            category: ElementCategory::Utility,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "direction", type_: PropType::OneOf(vec!["horizontal", "vertical"]), required: false, default: Some("horizontal"), description: "Divider direction" },
            ],
            allows_children: false,
            description: "Horizontal or vertical divider line",
        });

        elements.insert("spacer", ElementDef {
            tag: "spacer",
            category: ElementCategory::Utility,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
                PropDef { name: "size", type_: PropType::Int, required: false, default: None, description: "Spacer size in pixels (or flex if omitted)" },
            ],
            allows_children: false,
            description: "Flexible or fixed space",
        });
    }

    fn add_feedback_elements(elements: &mut HashMap<&'static str, ElementDef>) {
        // === Alert ===
        elements.insert("alert", ElementDef {
            tag: "alert",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "variant", type_: PropType::OneOf(vec!["default", "destructive"]), required: false, default: Some("default"), description: "Alert style variant" },
                PropDef { name: "title", type_: PropType::String, required: false, default: None, description: "Alert title" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Alert description text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert message box",
        });

        // === Toast/Toaster ===
        elements.insert("toast", ElementDef {
            tag: "toast",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "position", type_: PropType::OneOf(vec!["top-left", "top-center", "top-right", "bottom-left", "bottom-center", "bottom-right"]), required: false, default: Some("bottom-right"), description: "Toast position" },
                PropDef { name: "rich_colors", type_: PropType::Bool, required: false, default: Some("false"), description: "Use rich colors" },
                PropDef { name: "expand", type_: PropType::Bool, required: false, default: Some("false"), description: "Expand toasts" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Toast notification container (Sonner)",
        });

        elements.insert("toaster", ElementDef {
            tag: "toaster",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "position", type_: PropType::OneOf(vec!["top-left", "top-center", "top-right", "bottom-left", "bottom-center", "bottom-right"]), required: false, default: Some("bottom-right"), description: "Toast position" },
            ],
            allows_children: false,
            description: "Toast notification container (alias)",
        });

        // === Dropdown Menu ===
        elements.insert("dropdown", ElementDef {
            tag: "dropdown",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Dropdown menu container",
        });

        elements.insert("dropdown_trigger", ElementDef {
            tag: "dropdown_trigger",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "as_child", type_: PropType::Bool, required: false, default: Some("false"), description: "Use child as trigger" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Dropdown menu trigger",
        });

        elements.insert("dropdown_content", ElementDef {
            tag: "dropdown_content",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "side", type_: PropType::OneOf(vec!["top", "right", "bottom", "left"]), required: false, default: Some("bottom"), description: "Content position side" },
                PropDef { name: "align", type_: PropType::OneOf(vec!["start", "center", "end"]), required: false, default: Some("center"), description: "Content alignment" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Dropdown menu content",
        });

        elements.insert("dropdown_item", ElementDef {
            tag: "dropdown_item",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "value", type_: PropType::String, required: false, default: None, description: "Item value" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Item text" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Dropdown menu item",
        });

        elements.insert("dropdown_separator", ElementDef {
            tag: "dropdown_separator",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Dropdown menu separator",
        });

        elements.insert("dropdown_label", ElementDef {
            tag: "dropdown_label",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Label text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Dropdown menu label",
        });

        // === Popover ===
        elements.insert("popover", ElementDef {
            tag: "popover",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Popover container",
        });

        elements.insert("popover_trigger", ElementDef {
            tag: "popover_trigger",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "as_child", type_: PropType::Bool, required: false, default: Some("false"), description: "Use child as trigger" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Popover trigger",
        });

        elements.insert("popover_content", ElementDef {
            tag: "popover_content",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "side", type_: PropType::OneOf(vec!["top", "right", "bottom", "left"]), required: false, default: Some("bottom"), description: "Content position side" },
                PropDef { name: "align", type_: PropType::OneOf(vec!["start", "center", "end"]), required: false, default: Some("center"), description: "Content alignment" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Popover content",
        });

        // === Sheet (Side Drawer) ===
        elements.insert("sheet", ElementDef {
            tag: "sheet",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sheet (side drawer) container",
        });

        elements.insert("sheet_trigger", ElementDef {
            tag: "sheet_trigger",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "as_child", type_: PropType::Bool, required: false, default: Some("false"), description: "Use child as trigger" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sheet trigger",
        });

        elements.insert("sheet_content", ElementDef {
            tag: "sheet_content",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "side", type_: PropType::OneOf(vec!["top", "right", "bottom", "left"]), required: false, default: Some("right"), description: "Sheet position side" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sheet content",
        });

        elements.insert("sheet_header", ElementDef {
            tag: "sheet_header",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sheet header",
        });

        elements.insert("sheet_title", ElementDef {
            tag: "sheet_title",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Title text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sheet title",
        });

        elements.insert("sheet_footer", ElementDef {
            tag: "sheet_footer",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sheet footer",
        });

        // === Breadcrumb ===
        elements.insert("breadcrumb", ElementDef {
            tag: "breadcrumb",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Breadcrumb navigation container",
        });

        elements.insert("breadcrumb_list", ElementDef {
            tag: "breadcrumb_list",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Breadcrumb list",
        });

        elements.insert("breadcrumb_item", ElementDef {
            tag: "breadcrumb_item",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Breadcrumb item",
        });

        elements.insert("breadcrumb_link", ElementDef {
            tag: "breadcrumb_link",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "href", type_: PropType::String, required: false, default: None, description: "Link URL" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Link text" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Breadcrumb link",
        });

        elements.insert("breadcrumb_separator", ElementDef {
            tag: "breadcrumb_separator",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Breadcrumb separator",
        });

        elements.insert("breadcrumb_page", ElementDef {
            tag: "breadcrumb_page",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Current page text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Breadcrumb current page",
        });

        // ========================================
        // High Priority Components
        // ========================================

        // === Accordion ===
        elements.insert("accordion", ElementDef {
            tag: "accordion",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "type", type_: PropType::OneOf(vec!["single", "multiple"]), required: false, default: Some("single"), description: "Accordion type" },
                PropDef { name: "collapsible", type_: PropType::Bool, required: false, default: Some("false"), description: "Allow collapsing all items" },
                PropDef { name: "default", type_: PropType::String, required: false, default: None, description: "Default expanded item value" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Accordion container",
        });

        elements.insert("accordion_item", ElementDef {
            tag: "accordion_item",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "value", type_: PropType::String, required: true, default: None, description: "Item value (required)" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Accordion item",
        });

        elements.insert("accordion_trigger", ElementDef {
            tag: "accordion_trigger",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Trigger text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Accordion trigger",
        });

        elements.insert("accordion_content", ElementDef {
            tag: "accordion_content",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Accordion content",
        });

        // === Alert Dialog ===
        elements.insert("alert_dialog", ElementDef {
            tag: "alert_dialog",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert dialog container",
        });

        elements.insert("alert_dialog_trigger", ElementDef {
            tag: "alert_dialog_trigger",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "as_child", type_: PropType::Bool, required: false, default: Some("false"), description: "Use child as trigger" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert dialog trigger",
        });

        elements.insert("alert_dialog_content", ElementDef {
            tag: "alert_dialog_content",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert dialog content",
        });

        elements.insert("alert_dialog_header", ElementDef {
            tag: "alert_dialog_header",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert dialog header",
        });

        elements.insert("alert_dialog_footer", ElementDef {
            tag: "alert_dialog_footer",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert dialog footer",
        });

        elements.insert("alert_dialog_title", ElementDef {
            tag: "alert_dialog_title",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Title text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert dialog title",
        });

        elements.insert("alert_dialog_description", ElementDef {
            tag: "alert_dialog_description",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Description text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert dialog description",
        });

        elements.insert("alert_dialog_action", ElementDef {
            tag: "alert_dialog_action",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Action button text" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert dialog action button",
        });

        elements.insert("alert_dialog_cancel", ElementDef {
            tag: "alert_dialog_cancel",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Cancel button text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Alert dialog cancel button",
        });

        // === Command (Command Palette) ===
        elements.insert("command", ElementDef {
            tag: "command",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "query", type_: PropType::StateRef, required: false, default: None, description: "Search query binding" },
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: Some("Type a command..."), description: "Search placeholder" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Command palette container",
        });

        elements.insert("command_input", ElementDef {
            tag: "command_input",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: Some("Type a command..."), description: "Input placeholder" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Command palette search input",
        });

        elements.insert("command_list", ElementDef {
            tag: "command_list",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Command palette list container",
        });

        elements.insert("command_empty", ElementDef {
            tag: "command_empty",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: Some("No results found."), description: "Empty state text" },
            ],
            allows_children: true,
            description: "Command palette empty state",
        });

        elements.insert("command_group", ElementDef {
            tag: "command_group",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "heading", type_: PropType::String, required: false, default: None, description: "Group heading" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Command palette group",
        });

        elements.insert("command_item", ElementDef {
            tag: "command_item",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "value", type_: PropType::String, required: false, default: None, description: "Item value" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Item text" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Select handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Command palette item",
        });

        elements.insert("command_shortcut", ElementDef {
            tag: "command_shortcut",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Shortcut text (e.g., ⌘K)" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Command palette keyboard shortcut",
        });

        elements.insert("command_separator", ElementDef {
            tag: "command_separator",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Command palette separator",
        });

        // === Form ===
        elements.insert("form", ElementDef {
            tag: "form",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "id", type_: PropType::String, required: false, default: None, description: "Form ID" },
                PropDef { name: "onsubmit", type_: PropType::MsgRef, required: false, default: None, description: "Submit handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Form container",
        });

        elements.insert("form_field", ElementDef {
            tag: "form_field",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "name", type_: PropType::String, required: true, default: None, description: "Field name" },
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Field value binding" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Form field",
        });

        elements.insert("form_item", ElementDef {
            tag: "form_item",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Form item wrapper",
        });

        elements.insert("form_label", ElementDef {
            tag: "form_label",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "for", type_: PropType::String, required: false, default: None, description: "Label for attribute" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Label text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Form label",
        });

        elements.insert("form_control", ElementDef {
            tag: "form_control",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Form control wrapper",
        });

        elements.insert("form_description", ElementDef {
            tag: "form_description",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Description text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Form field description",
        });

        elements.insert("form_message", ElementDef {
            tag: "form_message",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Form validation message",
        });

        // === Navigation Menu ===
        elements.insert("nav_menu", ElementDef {
            tag: "nav_menu",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "orientation", type_: PropType::OneOf(vec!["horizontal", "vertical"]), required: false, default: Some("horizontal"), description: "Menu orientation" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Navigation menu container",
        });

        elements.insert("nav_menu_list", ElementDef {
            tag: "nav_menu_list",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Navigation menu list",
        });

        elements.insert("nav_menu_item", ElementDef {
            tag: "nav_menu_item",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "value", type_: PropType::String, required: false, default: None, description: "Item value" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Navigation menu item",
        });

        elements.insert("nav_menu_link", ElementDef {
            tag: "nav_menu_link",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "href", type_: PropType::String, required: false, default: None, description: "Link URL" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Link text" },
                PropDef { name: "active", type_: PropType::Bool, required: false, default: Some("false"), description: "Active state" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Navigation menu link",
        });

        elements.insert("nav_menu_trigger", ElementDef {
            tag: "nav_menu_trigger",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Trigger text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Navigation menu trigger",
        });

        elements.insert("nav_menu_content", ElementDef {
            tag: "nav_menu_content",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Navigation menu content",
        });

        elements.insert("nav_menu_indicator", ElementDef {
            tag: "nav_menu_indicator",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Navigation menu indicator",
        });

        // === Sidebar ===
        elements.insert("sidebar", ElementDef {
            tag: "sidebar",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "side", type_: PropType::OneOf(vec!["left", "right"]), required: false, default: Some("left"), description: "Sidebar position" },
                PropDef { name: "variant", type_: PropType::OneOf(vec!["sidebar", "floating", "inset"]), required: false, default: Some("sidebar"), description: "Sidebar variant" },
                PropDef { name: "collapsible", type_: PropType::OneOf(vec!["offcanvas", "icon", "none"]), required: false, default: Some("offcanvas"), description: "Collapsible mode" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar container",
        });

        elements.insert("sidebar_header", ElementDef {
            tag: "sidebar_header",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar header",
        });

        elements.insert("sidebar_content", ElementDef {
            tag: "sidebar_content",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar content",
        });

        elements.insert("sidebar_footer", ElementDef {
            tag: "sidebar_footer",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar footer",
        });

        elements.insert("sidebar_group", ElementDef {
            tag: "sidebar_group",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar group",
        });

        elements.insert("sidebar_group_label", ElementDef {
            tag: "sidebar_group_label",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Group label text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar group label",
        });

        elements.insert("sidebar_group_content", ElementDef {
            tag: "sidebar_group_content",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar group content",
        });

        elements.insert("sidebar_menu", ElementDef {
            tag: "sidebar_menu",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar menu",
        });

        elements.insert("sidebar_menu_item", ElementDef {
            tag: "sidebar_menu_item",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar menu item",
        });

        elements.insert("sidebar_menu_button", ElementDef {
            tag: "sidebar_menu_button",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "tooltip", type_: PropType::String, required: false, default: None, description: "Tooltip text" },
                PropDef { name: "active", type_: PropType::Bool, required: false, default: Some("false"), description: "Active state" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Button text" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar menu button",
        });

        elements.insert("sidebar_trigger", ElementDef {
            tag: "sidebar_trigger",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Sidebar trigger button",
        });

        elements.insert("sidebar_provider", ElementDef {
            tag: "sidebar_provider",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Sidebar provider context",
        });

        // === Stepper ===
        elements.insert("stepper", ElementDef {
            tag: "stepper",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Current step binding" },
                PropDef { name: "orientation", type_: PropType::OneOf(vec!["horizontal", "vertical"]), required: false, default: Some("horizontal"), description: "Stepper orientation" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Stepper container",
        });

        elements.insert("stepper_item", ElementDef {
            tag: "stepper_item",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "step", type_: PropType::Int, required: true, default: None, description: "Step number" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Stepper item",
        });

        elements.insert("stepper_trigger", ElementDef {
            tag: "stepper_trigger",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Stepper trigger",
        });

        elements.insert("stepper_indicator", ElementDef {
            tag: "stepper_indicator",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Stepper indicator",
        });

        elements.insert("stepper_title", ElementDef {
            tag: "stepper_title",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Step title text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Stepper title",
        });

        elements.insert("stepper_description", ElementDef {
            tag: "stepper_description",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Step description text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Stepper description",
        });

        elements.insert("stepper_separator", ElementDef {
            tag: "stepper_separator",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Stepper separator",
        });

        // ========================================
        // Medium Priority Components
        // ========================================

        // === Calendar ===
        elements.insert("calendar", ElementDef {
            tag: "calendar",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Selected date binding" },
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: Some("Pick a date"), description: "Placeholder text" },
                PropDef { name: "weekday", type_: PropType::OneOf(vec!["short", "long"]), required: false, default: Some("short"), description: "Weekday format" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Calendar date picker",
        });

        // === Carousel ===
        elements.insert("carousel", ElementDef {
            tag: "carousel",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "align", type_: PropType::OneOf(vec!["start", "center", "end"]), required: false, default: Some("center"), description: "Slide alignment" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Carousel container",
        });

        elements.insert("carousel_content", ElementDef {
            tag: "carousel_content",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Carousel content wrapper",
        });

        elements.insert("carousel_item", ElementDef {
            tag: "carousel_item",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Carousel slide item",
        });

        elements.insert("carousel_prev", ElementDef {
            tag: "carousel_prev",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Carousel previous button",
        });

        elements.insert("carousel_next", ElementDef {
            tag: "carousel_next",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Carousel next button",
        });

        // === Combobox ===
        elements.insert("combobox", ElementDef {
            tag: "combobox",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Selected value binding" },
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Combobox container",
        });

        elements.insert("combobox_input", ElementDef {
            tag: "combobox_input",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: Some("Select..."), description: "Input placeholder" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Combobox search input",
        });

        elements.insert("combobox_trigger", ElementDef {
            tag: "combobox_trigger",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "as_child", type_: PropType::Bool, required: false, default: Some("false"), description: "Use child as trigger" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Combobox trigger button",
        });

        elements.insert("combobox_list", ElementDef {
            tag: "combobox_list",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Combobox options list",
        });

        elements.insert("combobox_empty", ElementDef {
            tag: "combobox_empty",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: Some("No results found."), description: "Empty state text" },
            ],
            allows_children: true,
            description: "Combobox empty state",
        });

        elements.insert("combobox_group", ElementDef {
            tag: "combobox_group",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "heading", type_: PropType::String, required: false, default: None, description: "Group heading" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Combobox option group",
        });

        elements.insert("combobox_item", ElementDef {
            tag: "combobox_item",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::String, required: false, default: None, description: "Item value" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Item text" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Select handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Combobox option item",
        });

        // === Context Menu ===
        elements.insert("context_menu", ElementDef {
            tag: "context_menu",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
            ],
            allows_children: true,
            description: "Context menu container",
        });

        elements.insert("context_menu_trigger", ElementDef {
            tag: "context_menu_trigger",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "as_child", type_: PropType::Bool, required: false, default: Some("false"), description: "Use child as trigger" },
            ],
            allows_children: true,
            description: "Context menu trigger (right-click area)",
        });

        elements.insert("context_menu_content", ElementDef {
            tag: "context_menu_content",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Context menu content",
        });

        elements.insert("context_menu_item", ElementDef {
            tag: "context_menu_item",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Item text" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Context menu item",
        });

        elements.insert("context_menu_separator", ElementDef {
            tag: "context_menu_separator",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Context menu separator",
        });

        elements.insert("context_menu_label", ElementDef {
            tag: "context_menu_label",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Label text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Context menu label",
        });

        // === Drawer (Vaul) ===
        elements.insert("drawer", ElementDef {
            tag: "drawer",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
                PropDef { name: "direction", type_: PropType::OneOf(vec!["left", "right", "top", "bottom"]), required: false, default: Some("bottom"), description: "Drawer direction" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Drawer container (Vaul)",
        });

        elements.insert("drawer_trigger", ElementDef {
            tag: "drawer_trigger",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "as_child", type_: PropType::Bool, required: false, default: Some("false"), description: "Use child as trigger" },
            ],
            allows_children: true,
            description: "Drawer trigger",
        });

        elements.insert("drawer_content", ElementDef {
            tag: "drawer_content",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Drawer content",
        });

        elements.insert("drawer_header", ElementDef {
            tag: "drawer_header",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Drawer header",
        });

        elements.insert("drawer_footer", ElementDef {
            tag: "drawer_footer",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Drawer footer",
        });

        elements.insert("drawer_title", ElementDef {
            tag: "drawer_title",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Title text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Drawer title",
        });

        elements.insert("drawer_description", ElementDef {
            tag: "drawer_description",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Description text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Drawer description",
        });

        elements.insert("drawer_close", ElementDef {
            tag: "drawer_close",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Close button text" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Drawer close button",
        });

        // === Hover Card ===
        elements.insert("hover_card", ElementDef {
            tag: "hover_card",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
                PropDef { name: "open_delay", type_: PropType::Int, required: false, default: Some("0"), description: "Open delay (ms)" },
                PropDef { name: "close_delay", type_: PropType::Int, required: false, default: Some("300"), description: "Close delay (ms)" },
            ],
            allows_children: true,
            description: "Hover card container",
        });

        elements.insert("hover_card_trigger", ElementDef {
            tag: "hover_card_trigger",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "as_child", type_: PropType::Bool, required: false, default: Some("false"), description: "Use child as trigger" },
            ],
            allows_children: true,
            description: "Hover card trigger",
        });

        elements.insert("hover_card_content", ElementDef {
            tag: "hover_card_content",
            category: ElementCategory::Overlay,
            props: vec![
                PropDef { name: "side", type_: PropType::OneOf(vec!["top", "right", "bottom", "left"]), required: false, default: Some("bottom"), description: "Content position" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Hover card content",
        });

        // === Number Field ===
        elements.insert("number_field", ElementDef {
            tag: "number_field",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Number value binding" },
                PropDef { name: "min", type_: PropType::Int, required: false, default: None, description: "Minimum value" },
                PropDef { name: "max", type_: PropType::Int, required: false, default: None, description: "Maximum value" },
                PropDef { name: "step", type_: PropType::Int, required: false, default: Some("1"), description: "Step increment" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Number input field with controls",
        });

        elements.insert("number_field_input", ElementDef {
            tag: "number_field_input",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: None, description: "Placeholder text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Number field input",
        });

        elements.insert("number_field_increment", ElementDef {
            tag: "number_field_increment",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Number field increment button",
        });

        elements.insert("number_field_decrement", ElementDef {
            tag: "number_field_decrement",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Number field decrement button",
        });

        // === Pagination ===
        elements.insert("pagination", ElementDef {
            tag: "pagination",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "page", type_: PropType::StateRef, required: false, default: None, description: "Current page binding" },
                PropDef { name: "total", type_: PropType::Int, required: true, default: None, description: "Total items" },
                PropDef { name: "per_page", type_: PropType::Int, required: false, default: Some("10"), description: "Items per page" },
                PropDef { name: "sibling_count", type_: PropType::Int, required: false, default: Some("1"), description: "Sibling pages to show" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Pagination container",
        });

        elements.insert("pagination_list", ElementDef {
            tag: "pagination_list",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Pagination list",
        });

        elements.insert("pagination_item", ElementDef {
            tag: "pagination_item",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "value", type_: PropType::Int, required: false, default: None, description: "Page number" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Pagination page item",
        });

        elements.insert("pagination_ellipsis", ElementDef {
            tag: "pagination_ellipsis",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Pagination ellipsis",
        });

        elements.insert("pagination_prev", ElementDef {
            tag: "pagination_prev",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Pagination previous button",
        });

        elements.insert("pagination_next", ElementDef {
            tag: "pagination_next",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Pagination next button",
        });

        elements.insert("pagination_first", ElementDef {
            tag: "pagination_first",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Pagination first page button",
        });

        elements.insert("pagination_last", ElementDef {
            tag: "pagination_last",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Pagination last page button",
        });

        // === Pin Input (OTP) ===
        elements.insert("pin_input", ElementDef {
            tag: "pin_input",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "PIN value binding" },
                PropDef { name: "length", type_: PropType::Int, required: false, default: Some("4"), description: "Number of PIN digits" },
                PropDef { name: "type", type_: PropType::OneOf(vec!["text", "password"]), required: false, default: Some("text"), description: "Input type" },
                PropDef { name: "otp", type_: PropType::Bool, required: false, default: Some("true"), description: "Enable OTP autocomplete" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "PIN/OTP input container",
        });

        elements.insert("pin_input_group", ElementDef {
            tag: "pin_input_group",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "PIN input group wrapper",
        });

        elements.insert("pin_input_slot", ElementDef {
            tag: "pin_input_slot",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "index", type_: PropType::Int, required: true, default: None, description: "Slot index (0-based)" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "PIN input slot",
        });

        elements.insert("pin_input_separator", ElementDef {
            tag: "pin_input_separator",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "PIN input separator",
        });

        // === Tags Input ===
        elements.insert("tags_input", ElementDef {
            tag: "tags_input",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Tags array binding" },
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: Some("Add tag..."), description: "Input placeholder" },
                PropDef { name: "max", type_: PropType::Int, required: false, default: None, description: "Maximum tags" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Tags input container",
        });

        elements.insert("tags_input_field", ElementDef {
            tag: "tags_input_field",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: None, description: "Input placeholder" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Tags input text field",
        });

        elements.insert("tags_input_item", ElementDef {
            tag: "tags_input_item",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::String, required: false, default: None, description: "Tag value" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Tags input tag item",
        });

        elements.insert("tags_input_delete", ElementDef {
            tag: "tags_input_delete",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Delete handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Tags input delete button",
        });

        // === Toggle Group ===
        elements.insert("toggle_group", ElementDef {
            tag: "toggle_group",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Selected value binding" },
                PropDef { name: "type", type_: PropType::OneOf(vec!["single", "multiple"]), required: false, default: Some("single"), description: "Selection type" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Toggle group container",
        });

        elements.insert("toggle_group_item", ElementDef {
            tag: "toggle_group_item",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::String, required: true, default: None, description: "Item value" },
                PropDef { name: "label", type_: PropType::String, required: false, default: None, description: "ARIA label" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Toggle group item",
        });

        // ========================================
        // Low Priority Components
        // ========================================

        // === Aspect Ratio ===
        elements.insert("aspect_ratio", ElementDef {
            tag: "aspect_ratio",
            category: ElementCategory::Layout,
            props: vec![
                PropDef { name: "ratio", type_: PropType::Int, required: false, default: Some("16"), description: "Aspect ratio (e.g., 16 for 16/9)" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Aspect ratio container",
        });

        // === Button Group ===
        elements.insert("button_group", ElementDef {
            tag: "button_group",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "orientation", type_: PropType::OneOf(vec!["horizontal", "vertical"]), required: false, default: Some("horizontal"), description: "Button orientation" },
                PropDef { name: "size", type_: PropType::OneOf(vec!["sm", "default", "lg"]), required: false, default: Some("default"), description: "Button size" },
                PropDef { name: "variant", type_: PropType::OneOf(vec!["default", "destructive", "outline", "secondary", "ghost", "link"]), required: false, default: Some("default"), description: "Button variant" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Button group container",
        });

        // === Chart ===
        elements.insert("chart", ElementDef {
            tag: "chart",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "config", type_: PropType::String, required: false, default: None, description: "Chart config object" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Chart container",
        });

        // === Collapsible ===
        elements.insert("collapsible", ElementDef {
            tag: "collapsible",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
                PropDef { name: "default_open", type_: PropType::Bool, required: false, default: Some("false"), description: "Default open state" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Collapsible container",
        });

        elements.insert("collapsible_trigger", ElementDef {
            tag: "collapsible_trigger",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "as_child", type_: PropType::Bool, required: false, default: Some("false"), description: "Use child as trigger" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Collapsible trigger",
        });

        elements.insert("collapsible_content", ElementDef {
            tag: "collapsible_content",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Collapsible content",
        });

        // === Input Group ===
        elements.insert("input_group", ElementDef {
            tag: "input_group",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Input group wrapper",
        });

        // === Input OTP ===
        elements.insert("input_otp", ElementDef {
            tag: "input_otp",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "OTP value binding" },
                PropDef { name: "length", type_: PropType::Int, required: false, default: Some("6"), description: "Number of OTP digits" },
                PropDef { name: "pattern", type_: PropType::String, required: false, default: None, description: "Input pattern" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "OTP input container",
        });

        // === Kbd (Keyboard) ===
        elements.insert("kbd", ElementDef {
            tag: "kbd",
            category: ElementCategory::Content,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Keyboard key text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Keyboard key display",
        });

        // === Menubar ===
        elements.insert("menubar", ElementDef {
            tag: "menubar",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Menubar container",
        });

        elements.insert("menubar_menu", ElementDef {
            tag: "menubar_menu",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "value", type_: PropType::String, required: false, default: None, description: "Menu value" },
            ],
            allows_children: true,
            description: "Menubar menu",
        });

        elements.insert("menubar_trigger", ElementDef {
            tag: "menubar_trigger",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Trigger text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Menubar trigger",
        });

        elements.insert("menubar_content", ElementDef {
            tag: "menubar_content",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "align", type_: PropType::OneOf(vec!["start", "center", "end"]), required: false, default: Some("start"), description: "Content alignment" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Menubar content",
        });

        elements.insert("menubar_item", ElementDef {
            tag: "menubar_item",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Item text" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Click handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Menubar item",
        });

        elements.insert("menubar_separator", ElementDef {
            tag: "menubar_separator",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Menubar separator",
        });

        elements.insert("menubar_label", ElementDef {
            tag: "menubar_label",
            category: ElementCategory::Navigation,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Label text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Menubar label",
        });

        // === Native Select ===
        elements.insert("native_select", ElementDef {
            tag: "native_select",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Selected value binding" },
                PropDef { name: "name", type_: PropType::String, required: false, default: None, description: "Form field name" },
                PropDef { name: "disabled", type_: PropType::Bool, required: false, default: Some("false"), description: "Disabled state" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Native HTML select",
        });

        // === Range Calendar ===
        elements.insert("range_calendar", ElementDef {
            tag: "range_calendar",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Date range binding" },
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: Some("Pick a date range"), description: "Placeholder text" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Date range picker calendar",
        });

        // === Resizable ===
        elements.insert("resizable", ElementDef {
            tag: "resizable",
            category: ElementCategory::Layout,
            props: vec![
                PropDef { name: "direction", type_: PropType::OneOf(vec!["horizontal", "vertical"]), required: false, default: Some("horizontal"), description: "Resize direction" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Resizable panel group",
        });

        elements.insert("resizable_panel", ElementDef {
            tag: "resizable_panel",
            category: ElementCategory::Layout,
            props: vec![
                PropDef { name: "default_size", type_: PropType::Int, required: false, default: None, description: "Default panel size (%)" },
                PropDef { name: "min_size", type_: PropType::Int, required: false, default: None, description: "Minimum panel size (%)" },
                PropDef { name: "max_size", type_: PropType::Int, required: false, default: None, description: "Maximum panel size (%)" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Resizable panel",
        });

        elements.insert("resizable_handle", ElementDef {
            tag: "resizable_handle",
            category: ElementCategory::Layout,
            props: vec![
                PropDef { name: "with_handle", type_: PropType::Bool, required: false, default: Some("false"), description: "Show drag handle" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Resizable panel handle",
        });

        // === Auto Complete ===
        elements.insert("autocomplete", ElementDef {
            tag: "autocomplete",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::StateRef, required: false, default: None, description: "Selected value binding" },
                PropDef { name: "open", type_: PropType::StateRef, required: false, default: None, description: "Open state binding" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Autocomplete container",
        });

        elements.insert("autocomplete_input", ElementDef {
            tag: "autocomplete_input",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "placeholder", type_: PropType::String, required: false, default: Some("Search..."), description: "Input placeholder" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: false,
            description: "Autocomplete input",
        });

        elements.insert("autocomplete_list", ElementDef {
            tag: "autocomplete_list",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Autocomplete options list",
        });

        elements.insert("autocomplete_item", ElementDef {
            tag: "autocomplete_item",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "value", type_: PropType::String, required: false, default: None, description: "Item value" },
                PropDef { name: "text", type_: PropType::String, required: false, default: None, description: "Item text" },
                PropDef { name: "onclick", type_: PropType::MsgRef, required: false, default: None, description: "Select handler" },
                PropDef { name: "class", type_: PropType::Union(vec![PropType::String, PropType::ClassBinding]), required: false, default: None, description: "CSS class(es)" },
            ],
            allows_children: true,
            description: "Autocomplete option item",
        });

        elements.insert("autocomplete_empty", ElementDef {
            tag: "autocomplete_empty",
            category: ElementCategory::Form,
            props: vec![
                PropDef { name: "text", type_: PropType::String, required: false, default: Some("No results found."), description: "Empty state text" },
            ],
            allows_children: true,
            description: "Autocomplete empty state",
        });
    }
}

impl Default for AuraSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let schema = AuraSchema::new();
        assert!(schema.get_element("button").is_some());
        assert!(schema.get_element("col").is_some());
        assert!(schema.get_element("input").is_some());
        assert!(schema.get_element("nonexistent").is_none());
    }

    #[test]
    fn test_element_props() {
        let schema = AuraSchema::new();
        let button = schema.get_element("button").unwrap();

        assert!(button.get_prop("text").is_some());
        assert!(button.get_prop("onclick").is_some());
        assert!(button.get_prop("nonexistent").is_none());
        assert!(!button.allows_children);
    }

    #[test]
    fn test_widget_blocks() {
        let schema = AuraSchema::new();

        assert!(schema.widget_blocks.is_required("msg"));
        assert!(schema.widget_blocks.is_required("model"));
        assert!(schema.widget_blocks.is_required("view"));
        assert!(!schema.widget_blocks.is_required("computed"));
        assert!(!schema.widget_blocks.is_required("on"));

        assert!(schema.widget_blocks.is_valid_block("msg"));
        assert!(schema.widget_blocks.is_valid_block("on"));
        assert!(!schema.widget_blocks.is_valid_block("invalid"));
    }

    #[test]
    fn test_similar_suggestions() {
        let schema = AuraSchema::new();

        assert_eq!(schema.suggest_similar("buton"), Some("button"));
        assert_eq!(schema.suggest_similar("buttn"), Some("button"));
        assert_eq!(schema.suggest_similar("rw"), Some("row"));
        assert_eq!(schema.suggest_similar("cl"), Some("col"));
        // "xyz" is too far from any valid element
        // Note: Levenshtein distance of 3 still matches, so we test something more distant
        assert!(schema.suggest_similar("abcdefgh").is_none());
    }

    #[test]
    fn test_elements_by_category() {
        let schema = AuraSchema::new();

        let layout = schema.elements_by_category(ElementCategory::Layout);
        assert!(layout.iter().any(|e| e.tag == "col"));
        assert!(layout.iter().any(|e| e.tag == "row"));

        let content = schema.elements_by_category(ElementCategory::Content);
        assert!(content.iter().any(|e| e.tag == "button"));
        assert!(content.iter().any(|e| e.tag == "input"));
    }

    #[test]
    fn test_prop_type_names() {
        assert_eq!(PropType::String.name(), "string");
        assert_eq!(PropType::MsgRef.name(), "msg_ref");
        assert_eq!(PropType::OneOf(vec!["a", "b"]).name(), "one_of(a | b)");
    }
}
