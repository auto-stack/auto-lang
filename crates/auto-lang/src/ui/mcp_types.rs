//! # MCP Types for AutoUI Desktop Integration (Plan 278)
//!
//! Defines data structures for the MCP tools that allow AI agents to
//! inspect and manipulate AutoUI desktop interfaces.
//!
//! ## Tools
//!
//! - `autoui_snapshot` — Capture full page structure as AURA text
//! - `autoui_inspect` — Inspect a single element by AuraNodeId
//! - `autoui_action`  — Perform an action on an element

use std::fmt;

use crate::aura::AuraNodeId;
use auto_val::Value;

// ============================================================================
// Snapshot Types
// ============================================================================

/// A complete snapshot of the current UI page.
///
/// Contains the widget name, all state variable values,
/// and the full component tree with properties and actions.
#[derive(Debug, Clone)]
pub struct UiSnapshot {
    /// Widget name (e.g., "TodoApp")
    pub widget_name: String,
    /// State variable values: (name, value, type_hint)
    pub state: Vec<(String, String, String)>,
    /// Root of the component tree
    pub tree: UiNode,
}

/// A node in the UI component tree.
///
/// Each node represents a single visual component (Button, Input, Text, etc.)
/// with its current properties, available actions, and children.
#[derive(Debug, Clone)]
pub struct UiNode {
    /// Stable component ID (AuraNodeId)
    pub id: AuraNodeId,
    /// Component kind (e.g., "Button", "Input", "Text", "Column")
    pub kind: String,
    /// Key-value properties (e.g., "label" -> "Add", "placeholder" -> "Type here")
    pub props: Vec<(String, String)>,
    /// Available actions on this component
    pub actions: Vec<UiAction>,
    /// Child nodes
    pub children: Vec<UiNode>,
}

/// An action available on a UI component.
#[derive(Debug, Clone)]
pub struct UiAction {
    /// Action name (e.g., "press", "type", "toggle", "select", "set_value")
    pub name: String,
    /// Handler that will be triggered (e.g., ".AddTodo")
    pub handler: String,
}

// ============================================================================
// Inspect Types
// ============================================================================

/// Detailed information about a single UI element.
#[derive(Debug, Clone)]
pub struct ElementInfo {
    /// Component ID
    pub id: AuraNodeId,
    /// Component kind
    pub kind: String,
    /// Key-value properties
    pub props: Vec<(String, String)>,
    /// Available actions
    pub actions: Vec<UiAction>,
    /// Source location (if available)
    pub source_location: Option<SourceLocation>,
}

/// Source code location for a UI element.
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub col: usize,
}

// ============================================================================
// Action Types
// ============================================================================

/// Actions that can be performed on UI elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiActionType {
    /// Press a button (triggers onclick handler)
    Press,
    /// Type text into an input/textarea
    TypeText,
    /// Toggle a checkbox
    Toggle,
    /// Select an option from a dropdown/radio
    SelectOption,
    /// Set a slider value
    SetValue,
}

impl fmt::Display for UiActionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UiActionType::Press => write!(f, "press"),
            UiActionType::TypeText => write!(f, "type_text"),
            UiActionType::Toggle => write!(f, "toggle"),
            UiActionType::SelectOption => write!(f, "select_option"),
            UiActionType::SetValue => write!(f, "set_value"),
        }
    }
}

/// Result of performing an action on a UI element.
#[derive(Debug, Clone)]
pub struct ActionResult {
    /// Whether the action succeeded
    pub status: String,
    /// Target element ID
    pub element_id: AuraNodeId,
    /// Action that was performed
    pub action: String,
    /// Handler that was triggered (if any)
    pub handler: Option<String>,
    /// State changes: (field_name, before, after)
    pub state_changes: Vec<(String, String, String)>,
}

// ============================================================================
// AURA Text Formatting
// ============================================================================

impl UiSnapshot {
    /// Format the snapshot as AURA text.
    ///
    /// Returns a human-readable, machine-parseable text representation
    /// of the complete UI state.
    pub fn to_aura_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push("AuraUI Snapshot v1".to_string());
        lines.push(format!("widget: {:?}", self.widget_name));

        // State section
        if !self.state.is_empty() {
            lines.push("state:".to_string());
            for (name, value, type_hint) in &self.state {
                lines.push(format!("  {}: {} ({})", name, value, type_hint));
            }
        }

        // Tree section
        lines.push("".to_string());
        lines.push("tree:".to_string());
        self.tree.format_aura(&mut lines, 1);

        lines.join("\n")
    }
}

impl UiNode {
    /// Format this node and its children as AURA text.
    fn format_aura(&self, lines: &mut Vec<String>, indent: usize) {
        let pad = "  ".repeat(indent);

        // Component header: Kind #aura_N {
        lines.push(format!("{}{} #{} {{", pad, self.kind, self.id));

        let inner_pad = "  ".repeat(indent + 1);

        // Properties
        for (key, value) in &self.props {
            lines.push(format!("{}{}: {:?}", inner_pad, key, value));
        }

        // Actions
        if !self.actions.is_empty() {
            let action_strs: Vec<String> = self.actions
                .iter()
                .map(|a| format!("{} -> {}", a.name, a.handler))
                .collect();
            lines.push(format!("{}actions: [{}]", inner_pad, action_strs.join(", ")));
        }

        // Children
        for child in &self.children {
            child.format_aura(lines, indent + 1);
        }

        // Closing brace
        lines.push(format!("{}}}", pad));
    }
}

impl ElementInfo {
    /// Format as AURA text.
    pub fn to_aura_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Inspect #{}", self.id));
        lines.push(format!("  type: {}", self.kind));
        lines.push("  properties:".to_string());
        for (key, value) in &self.props {
            lines.push(format!("    {}: {:?}", key, value));
        }
        if !self.actions.is_empty() {
            lines.push("  actions:".to_string());
            for action in &self.actions {
                lines.push(format!("    {} -> {}", action.name, action.handler));
            }
        }
        if let Some(loc) = &self.source_location {
            lines.push(format!("  source: line {}, col {}", loc.line, loc.col));
        }

        lines.join("\n")
    }
}

impl ActionResult {
    /// Format as AURA text.
    pub fn to_aura_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push("ActionResult".to_string());
        lines.push(format!("  status: {}", self.status));
        lines.push(format!("  element: #{}", self.element_id));
        lines.push(format!("  action: {}", self.action));
        if let Some(handler) = &self.handler {
            lines.push(format!("  handler: {}", handler));
        }
        if !self.state_changes.is_empty() {
            lines.push("  state_changes:".to_string());
            for (field, before, after) in &self.state_changes {
                lines.push(format!("    {}: {} -> {}", field, before, after));
            }
        }

        lines.join("\n")
    }
}

/// Format a Value for display in AURA output.
pub fn format_value(v: &Value) -> String {
    match v {
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format!("{:.2}", f),
        Value::Bool(b) => b.to_string(),
        Value::Str(s) => format!("{:?}", s), // quoted string
        Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Infer a type hint from a Value.
pub fn type_hint(v: &Value) -> &'static str {
    match v {
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Bool(_) => "bool",
        Value::Str(_) => "str",
        Value::Null => "null",
        Value::Array(_) => "list",
        Value::Obj(_) => "object",
        _ => "unknown",
    }
}
