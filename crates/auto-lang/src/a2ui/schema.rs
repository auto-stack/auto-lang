//! A2UI v0.8 JSON Schema Types
//!
//! Rust representations of Google's A2UI (Agent-to-User Interface) protocol.
//! All types derive Serialize and Deserialize for JSON round-tripping.

use serde::{Deserialize, Serialize};

// ============================================================================
// Top-Level Message
// ============================================================================

/// The root message envelope for A2UI protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum A2UIMessage {
    #[serde(rename = "surfaceUpdate")]
    SurfaceUpdate(A2UISurfaceUpdate),
}

// ============================================================================
// Surface Update
// ============================================================================

/// A surface update replaces the content of a named surface with a new component tree.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UISurfaceUpdate {
    #[serde(rename = "surfaceId")]
    pub surface_id: String,
    pub components: Vec<A2UIComponent>,
}

// ============================================================================
// Component
// ============================================================================

/// A single component in the A2UI tree, identified by a unique id.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UIComponent {
    pub id: String,
    #[serde(flatten)]
    pub body: A2UIComponentBody,
}

/// The body (type + properties) of an A2UI component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "component")]
pub enum A2UIComponentBody {
    // --- Layout ---
    #[serde(rename = "Container")]
    Container {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<A2UIComponent>,
    },
    #[serde(rename = "Row")]
    Row {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<A2UIComponent>,
    },
    #[serde(rename = "Column")]
    Column {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        children: Vec<A2UIComponent>,
    },
    #[serde(rename = "ScrollView")]
    ScrollView {
        #[serde(skip_serializing_if = "Option::is_none")]
        child: Option<Box<A2UIComponentBody>>,
    },

    // --- Form ---
    #[serde(rename = "TextInput")]
    TextInput {
        value: A2UIValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<A2UIValue>,
    },
    #[serde(rename = "NumberInput")]
    NumberInput {
        value: A2UIValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
    },
    #[serde(rename = "DateTimeInput")]
    DateTimeInput { value: A2UIValue },
    #[serde(rename = "Button")]
    Button {
        child: A2UIValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<A2UIAction>,
    },
    #[serde(rename = "Checkbox")]
    Checkbox {
        value: A2UIValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<A2UIValue>,
    },
    #[serde(rename = "Radio")]
    Radio {
        value: A2UIValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<A2UIValue>,
    },
    #[serde(rename = "Select")]
    Select {
        value: A2UIValue,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        options: Vec<A2UISelectOption>,
    },
    #[serde(rename = "Slider")]
    Slider {
        value: A2UIValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
    },

    // --- Display ---
    #[serde(rename = "Text")]
    Text { text: A2UIValue },
    #[serde(rename = "Image")]
    Image { src: A2UIValue },
    #[serde(rename = "Icon")]
    Icon { name: A2UIValue },
    #[serde(rename = "Divider")]
    Divider {},
    #[serde(rename = "Spacer")]
    Spacer {},

    // --- Data ---
    #[serde(rename = "List")]
    List {
        items: A2UIValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        template: Option<Box<A2UIComponentBody>>,
    },
    #[serde(rename = "Table")]
    Table {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        columns: Vec<A2UITableColumn>,
        items: A2UIValue,
    },

    // --- Navigation ---
    #[serde(rename = "Tabs")]
    Tabs {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tabs: Vec<A2UITab>,
    },
    #[serde(rename = "Navigation")]
    Navigation {
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        items: Vec<A2UINavItem>,
    },
}

// ============================================================================
// Value (Data Binding)
// ============================================================================

/// A value in A2UI can be either a path binding or a literal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum A2UIValue {
    Path { path: String },
    LiteralString {
        #[serde(rename = "literalString")]
        literal_string: String,
    },
    LiteralNumber {
        #[serde(rename = "literalNumber")]
        literal_number: f64,
    },
    LiteralBool {
        #[serde(rename = "literalBool")]
        literal_bool: bool,
    },
}

impl A2UIValue {
    /// Create a path value.
    pub fn path(p: impl Into<String>) -> Self {
        A2UIValue::Path { path: p.into() }
    }

    /// Create a string literal value.
    pub fn string(s: impl Into<String>) -> Self {
        A2UIValue::LiteralString {
            literal_string: s.into(),
        }
    }

    /// Create a number literal value.
    pub fn number(n: f64) -> Self {
        A2UIValue::LiteralNumber { literal_number: n }
    }

    /// Create a bool literal value.
    pub fn bool(b: bool) -> Self {
        A2UIValue::LiteralBool { literal_bool: b }
    }

    /// Returns true if this is a path binding.
    pub fn is_path(&self) -> bool {
        matches!(self, A2UIValue::Path { .. })
    }

    /// Returns the path if this is a path binding.
    pub fn as_path(&self) -> Option<&str> {
        match self {
            A2UIValue::Path { path } => Some(path),
            _ => None,
        }
    }
}

// ============================================================================
// Action
// ============================================================================

/// An action that is triggered by user interaction (e.g., button click).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UIAction {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context: Vec<A2UIContextBinding>,
}

/// A single context binding passed with an action.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UIContextBinding {
    pub name: String,
    pub path: String,
}

// ============================================================================
// Sub-component Types
// ============================================================================

/// An option in a Select component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UISelectOption {
    pub value: String,
    pub label: A2UIValue,
}

/// A column definition in a Table component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UITableColumn {
    pub key: String,
    pub label: A2UIValue,
}

/// A single tab in a Tabs component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UITab {
    pub label: A2UIValue,
    pub child: Box<A2UIComponentBody>,
}

/// A navigation item in a Navigation component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct A2UINavItem {
    pub label: A2UIValue,
    pub path: String,
}

// ============================================================================
// Convenience Constructors
// ============================================================================

impl A2UISurfaceUpdate {
    pub fn new(surface_id: impl Into<String>) -> Self {
        Self {
            surface_id: surface_id.into(),
            components: Vec::new(),
        }
    }

    pub fn with_components(mut self, components: Vec<A2UIComponent>) -> Self {
        self.components = components;
        self
    }
}

impl A2UIComponent {
    pub fn new(id: impl Into<String>, body: A2UIComponentBody) -> Self {
        Self {
            id: id.into(),
            body,
        }
    }
}

impl A2UIAction {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            context: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: Vec<A2UIContextBinding>) -> Self {
        self.context = context;
        self
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_path() {
        let v = A2UIValue::path("/booking/date");
        assert!(v.is_path());
        assert_eq!(v.as_path(), Some("/booking/date"));
    }

    #[test]
    fn test_value_string() {
        let v = A2UIValue::string("Hello");
        assert!(!v.is_path());
        assert_eq!(v.as_path(), None);
    }

    #[test]
    fn test_serialize_surface_update() {
        let msg = A2UIMessage::SurfaceUpdate(
            A2UISurfaceUpdate::new("booking").with_components(vec![
                A2UIComponent::new(
                    "title",
                    A2UIComponentBody::Text {
                        text: A2UIValue::string("Book Your Table"),
                    },
                ),
                A2UIComponent::new(
                    "datetime",
                    A2UIComponentBody::DateTimeInput {
                        value: A2UIValue::path("/booking/date"),
                    },
                ),
                A2UIComponent::new(
                    "submit-btn",
                    A2UIComponentBody::Button {
                        child: A2UIValue::string("Confirm"),
                        action: Some(A2UIAction::new("confirm_booking")),
                    },
                ),
            ]),
        );

        let json = serde_json::to_string_pretty(&msg).unwrap();
        assert!(json.contains("\"type\": \"surfaceUpdate\""));
        assert!(json.contains("\"surfaceId\": \"booking\""));
        assert!(json.contains("\"component\": \"Text\""));
        assert!(json.contains("\"literalString\": \"Book Your Table\""));
        assert!(json.contains("\"path\": \"/booking/date\""));
    }

    #[test]
    fn test_deserialize_surface_update() {
        let json = r#"{
            "type": "surfaceUpdate",
            "surfaceId": "demo",
            "components": [
                {
                    "id": "greeting",
                    "component": "Text",
                    "text": { "literalString": "Hello" }
                }
            ]
        }"#;

        let msg: A2UIMessage = serde_json::from_str(json).unwrap();
        match msg {
            A2UIMessage::SurfaceUpdate(update) => {
                assert_eq!(update.surface_id, "demo");
                assert_eq!(update.components.len(), 1);
                assert_eq!(update.components[0].id, "greeting");
                match &update.components[0].body {
                    A2UIComponentBody::Text { text } => {
                        assert_eq!(text, &A2UIValue::string("Hello"));
                    }
                    _ => panic!("Expected Text component"),
                }
            }
        }
    }

    #[test]
    fn test_roundtrip_complex() {
        let original = A2UIMessage::SurfaceUpdate(
            A2UISurfaceUpdate::new("test").with_components(vec![
                A2UIComponent::new(
                    "col1",
                    A2UIComponentBody::Column {
                        children: vec![
                            A2UIComponent::new(
                                "btn1",
                                A2UIComponentBody::Button {
                                    child: A2UIValue::string("Click"),
                                    action: Some(A2UIAction::new("click_me")),
                                },
                            ),
                        ],
                    },
                ),
            ]),
        );

        let json = serde_json::to_string(&original).unwrap();
        let restored: A2UIMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }
}
