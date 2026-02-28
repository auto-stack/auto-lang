//! UI AST Nodes - First-class AST nodes for UI declarations
//!
//! These nodes are only parsed when the scenario is UI (contextual keywords).
//! They represent widget, msg, model, view, and on blocks as first-class citizens.

use super::{Body, Expr, Name, Type};
use auto_val::AutoStr;

// ============================================================================
// Widget Declaration
// ============================================================================

/// Widget declaration: the core UI component
///
/// ```auto
/// widget Counter {
///     msg Msg { Inc, Dec }
///     model { count int = 0 }
///     computed { doubleCount => .count * 2 }
///     view { ... }
///     on { .Inc => { .count += 1 } }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WidgetDecl {
    /// Widget name (e.g., "Counter")
    pub name: Name,

    /// Message type declarations (msg blocks)
    pub messages: Vec<MsgDecl>,

    /// State/model declaration
    pub model: Option<ModelBlock>,

    /// Computed properties
    pub computed: Option<ComputedBlock>,

    /// View tree declaration
    pub view: Option<ViewBlock>,

    /// Event handlers
    pub on: Option<OnBlock>,

    /// Props for reusable components
    pub props: Vec<PropDecl>,
}

// ============================================================================
// Message Declaration
// ============================================================================

/// Message declaration: defines message types for MVU pattern
///
/// ```auto
/// msg Msg { Inc, Dec, Set(int) }
/// ```
#[derive(Debug, Clone)]
pub struct MsgDecl {
    /// Message type name (e.g., "Msg")
    pub name: Name,

    /// Message variants
    pub variants: Vec<MsgVariant>,
}

/// Message variant
#[derive(Debug, Clone)]
pub struct MsgVariant {
    /// Variant name (e.g., "Inc")
    pub name: Name,

    /// Optional payload type
    pub payload: Option<Type>,
}

// ============================================================================
// Model Block
// ============================================================================

/// Model block: defines state variables
///
/// ```auto
/// model {
///     count int = 0
///     name str = ""
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ModelBlock {
    /// State variable declarations
    pub fields: Vec<ModelField>,
}

/// Model field: a single state variable
#[derive(Debug, Clone)]
pub struct ModelField {
    /// Field name
    pub name: Name,

    /// Field type
    pub ty: Type,

    /// Initial value expression
    pub init: Expr,
}

// ============================================================================
// Computed Block
// ============================================================================

/// Computed block: defines computed/derived properties
///
/// ```auto
/// computed {
///     activeCount => .todos.filter(|t| !t.done).len
///     filteredTodos => match .filter {
///         Filter::All => .todos
///         Filter::Active => .todos.filter(|t| !t.done)
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ComputedBlock {
    /// Computed property declarations
    pub properties: Vec<ComputedProperty>,
}

/// Computed property: a derived value
#[derive(Debug, Clone)]
pub struct ComputedProperty {
    /// Property name
    pub name: Name,

    /// Computation expression (as string for flexibility)
    pub expr: String,
}

// ============================================================================
// View Block
// ============================================================================

/// View block: defines the UI structure
///
/// ```auto
/// view {
///     col {
///         button + { onclick: .Inc }
///         h2 > Count: ${.count}
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ViewBlock {
    /// Root node of the view tree
    pub root: ViewNode,
}

/// View node: element or text in the view tree
#[derive(Debug, Clone)]
pub enum ViewNode {
    /// Element node with tag, props, events, and children
    Element {
        /// Tag name (e.g., "col", "button", "h2")
        tag: String,

        /// Properties (key-value pairs)
        props: Vec<ViewProp>,

        /// Event handlers
        events: Vec<ViewEvent>,

        /// Child nodes
        children: Vec<ViewNode>,
    },

    /// Text node (literal or with interpolations)
    Text(ViewText),

    /// For loop: for item in .list { body }
    ForLoop {
        /// Loop variable name (e.g., "todo")
        var: String,

        /// Optional index variable (e.g., Some("i") for `for i, item in ...`)
        index: Option<String>,

        /// Iterable expression (e.g., ".todos")
        iterable: String,

        /// Loop body nodes
        body: Vec<ViewNode>,
    },

    /// Conditional: if condition { then_body } else { else_body }
    Conditional {
        /// Condition expression as string (e.g., ".todos.len > 0")
        condition: String,

        /// Body when condition is true
        then_body: Vec<ViewNode>,

        /// Optional else body
        else_body: Option<Vec<ViewNode>>,
    },

    /// Component instantiation: TodoItem (todo: .todo, onToggle: .Toggle)
    Component {
        /// Component name (e.g., "TodoItem")
        name: String,

        /// Properties passed to component
        props: Vec<ViewProp>,

        /// Event handlers
        events: Vec<ViewEvent>,
    },
}

/// View property
#[derive(Debug, Clone)]
pub struct ViewProp {
    /// Property name
    pub name: String,

    /// Property value expression
    pub value: Expr,
}

/// View event handler
#[derive(Debug, Clone)]
pub struct ViewEvent {
    /// Event name (e.g., "onclick")
    pub name: String,

    /// Handler pattern (e.g., ".Inc" or "Msg::Inc")
    pub handler: String,

    /// Optional parameters for the handler (e.g., ["todo.id"] for .Delete(todo.id))
    pub params: Vec<String>,
}

/// View text content
#[derive(Debug, Clone)]
pub enum ViewText {
    /// Literal text
    Literal(String),

    /// Interpolated text with ${...} placeholders
    Interpolated {
        /// Template string
        template: String,

        /// Extracted state references
        bindings: Vec<String>,
    },
}

// ============================================================================
// On Block
// ============================================================================

/// On block: defines event handlers
///
/// ```auto
/// on {
///     .Inc => { .count += 1 }
///     .Dec => { .count -= 1 }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OnBlock {
    /// Event handlers
    pub handlers: Vec<OnHandler>,
}

/// Event handler
#[derive(Debug, Clone)]
pub struct OnHandler {
    /// Pattern to match (e.g., ".Inc", "Msg::Dec")
    pub pattern: String,

    /// Handler body
    pub body: Body,
}

// ============================================================================
// Prop Declaration
// ============================================================================

/// Prop declaration for reusable components
///
/// ```auto
/// widget Button(text str, disabled bool = false) { ... }
/// ```
#[derive(Debug, Clone)]
pub struct PropDecl {
    /// Prop name
    pub name: Name,

    /// Prop type
    pub ty: Type,

    /// Default value (if any)
    pub default: Option<Expr>,
}

// ============================================================================
// Helper Implementations
// ============================================================================

impl ViewNode {
    /// Create a new element node
    pub fn element(tag: impl Into<String>) -> Self {
        ViewNode::Element {
            tag: tag.into(),
            props: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Create a text node
    pub fn text(content: impl Into<String>) -> Self {
        ViewNode::Text(ViewText::Literal(content.into()))
    }

    /// Add a property
    pub fn with_prop(mut self, name: impl Into<String>, value: Expr) -> Self {
        if let ViewNode::Element { props, .. } = &mut self {
            props.push(ViewProp {
                name: name.into(),
                value,
            });
        }
        self
    }

    /// Add an event handler
    pub fn with_event(mut self, name: impl Into<String>, handler: impl Into<String>) -> Self {
        if let ViewNode::Element { events, .. } = &mut self {
            events.push(ViewEvent {
                name: name.into(),
                handler: handler.into(),
                params: Vec::new(),
            });
        }
        self
    }

    /// Add an event handler with parameters
    pub fn with_event_params(mut self, name: impl Into<String>, handler: impl Into<String>, params: Vec<String>) -> Self {
        if let ViewNode::Element { events, .. } = &mut self {
            events.push(ViewEvent {
                name: name.into(),
                handler: handler.into(),
                params,
            });
        }
        self
    }

    /// Add a child node
    pub fn with_child(mut self, child: ViewNode) -> Self {
        if let ViewNode::Element { children, .. } = &mut self {
            children.push(child);
        }
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
    fn test_view_node_element() {
        let node = ViewNode::element("col")
            .with_child(ViewNode::text("Hello"));

        match node {
            ViewNode::Element { tag, children, .. } => {
                assert_eq!(tag, "col");
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Element node"),
        }
    }

    #[test]
    fn test_view_node_text() {
        let node = ViewNode::text("Hello World");

        match node {
            ViewNode::Text(ViewText::Literal(s)) => {
                assert_eq!(s, "Hello World");
            }
            _ => panic!("Expected Text node"),
        }
    }

    #[test]
    fn test_widget_decl() {
        let widget = WidgetDecl {
            name: AutoStr::from("Counter"),
            messages: vec![],
            model: None,
            view: None,
            on: None,
            props: vec![],
            computed: None,
        };

        assert_eq!(widget.name.as_str(), "Counter");
    }

    #[test]
    fn test_msg_decl() {
        let msg = MsgDecl {
            name: AutoStr::from("Msg"),
            variants: vec![
                MsgVariant {
                    name: AutoStr::from("Inc"),
                    payload: None,
                },
                MsgVariant {
                    name: AutoStr::from("Set"),
                    payload: Some(Type::Int),
                },
            ],
        };

        assert_eq!(msg.variants.len(), 2);
    }
}
