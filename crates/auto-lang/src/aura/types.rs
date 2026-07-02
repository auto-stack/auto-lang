//! AURA Core Types - Data structures for UI intermediate representation
//!
//! This module defines the core data structures for AURA (Auto UI Representation Abstract).
//! These types represent the extracted, structured form of UI components.

use std::collections::HashMap;
use std::fmt;

// Re-export Type from ast for convenience
pub use crate::ast::Type;
use crate::ast::{RouteDef, RoutesBlock};

// ============================================================================
// Stable Node ID (Plan 273)
// ============================================================================

/// 稳定唯一 ID，在 AuraNode 提取时分配，用于 DevTools 源码↔组件双向映射
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AuraNodeId(pub u32);

impl fmt::Display for AuraNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "aura_{}", self.0)
    }
}

/// 源码位置信息，用于 DevTools 双向映射
#[derive(Debug, Clone)]
pub struct SpanInfo {
    /// 源码字节偏移和长度
    pub span: Option<(usize, usize)>,
    /// 原始 AuraNode tag（如 "center", "button"）
    pub aura_tag: String,
    /// 用户指定的 id 属性（如 id: "my-btn"）
    pub user_id: Option<String>,
}

// ============================================================================
// AURA Widget - Core Component Definition
// ============================================================================

/// AURA Widget: The core component definition
///
/// This is the primary structure extracted from a `widget` declaration.
/// It contains:
/// - State variables (model)
/// - View tree (pure layout, no logic)
/// - Event handlers (logic payload)
/// - Routes (for router widgets, Plan 105)
/// - Lifecycle methods (Plan 05-Nav)
#[derive(Debug, Clone)]
pub struct AuraWidget {
    /// Widget name (e.g., "Counter")
    pub name: String,

    /// State variable definitions
    pub state_vars: Vec<AuraStateDef>,

    /// Computed properties
    pub computed: Vec<AuraComputed>,

    /// Message type definitions
    pub messages: Vec<AuraMessage>,

    /// View tree: pure layout and bindings, no logic
    pub view_tree: AuraNode,

    /// Event handlers: mapped by event pattern (e.g., "Msg::Inc")
    pub handlers: HashMap<String, LogicPayload>,

    /// Props for reusable components
    pub props: Vec<AuraProp>,

    /// Routes configuration (for router widgets, Plan 105)
    pub routes: Option<AuraRoutes>,

    /// Lifecycle methods (e.g., aboutToAppear) - Plan 05-Nav
    pub lifecycle: Vec<AuraLifecycle>,

    /// Tick interval in ms — when set, the runtime emits .Tick events at this interval
    pub tick_interval: Option<u32>,

    /// Handler parameter names: maps handler pattern to parameter list
    /// e.g., ".AddItem" -> ["text"] for .AddItem(text) -> { ... }
    pub handler_params: HashMap<String, Vec<String>>,

    /// Span map: AuraNodeId → source info for DevTools (Plan 273)
    pub span_map: HashMap<AuraNodeId, SpanInfo>,

    /// Key bindings: key string → handler pattern (Plan 275)
    /// e.g., "1" → ".Digit1", "Enter" → ".Equals"
    pub key_bindings: HashMap<String, String>,

    /// API function names explicitly imported via `use back.api: ...`
    pub api_imports: Vec<String>,
}


// ============================================================================
// Plan 351 / Design 18: Shared Store (Rung 4)
// ============================================================================

/// An AURA shared store — a view-less widget whose state is shared across
/// widgets/routes via `use store:`. Isomorphic to AuraWidget minus
/// view_tree/routes/props.
#[derive(Debug, Clone)]
pub struct AuraStore {
    /// Store name (e.g., "CounterStore")
    pub name: String,

    /// State variable definitions (→ module-level ref()s)
    pub state_vars: Vec<AuraStateDef>,

    /// Message type definitions (→ action functions)
    pub messages: Vec<AuraMessage>,

    /// Event handlers: pattern → logic (→ action function bodies)
    pub handlers: HashMap<String, LogicPayload>,
}


// ============================================================================
// Router Types (Plan 105)
// ============================================================================

/// AURA route definition
///
/// Represents a single route in the application's routing table.
/// Extracted from `routes { ... }` blocks in AutoUI.
#[derive(Debug, Clone, PartialEq)]
pub struct AuraRoute {
    /// URL path pattern (e.g., "/button" or "/user/:id")
    pub path: String,

    /// Module name to render (e.g., "index", "button", "user")
    /// Maps to `@/pages/{module}.vue` in Vue generator
    pub module: String,

    /// Actual widget name from the source file (e.g., "ListPage" from listpage.at)
    /// This is used for imports and component references
    pub widget_name: String,

    /// Extracted parameters from path (e.g., ["id"] from "/user/:id")
    pub params: Vec<String>,
}

/// AURA routes configuration
///
/// Contains all routes for an application, extracted from `routes { ... }` blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct AuraRoutes {
    /// Collection of route definitions
    pub routes: Vec<AuraRoute>,
}

impl AuraRoutes {
    /// Create a new empty routes configuration
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Create routes configuration with the given routes
    pub fn with_routes(routes: Vec<AuraRoute>) -> Self {
        Self { routes }
    }
}

impl Default for AuraRoutes {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Lifecycle Types (Plan 05-Nav)
// ============================================================================

/// Lifecycle method definition
///
/// Represents lifecycle methods like `aboutToAppear()` in AURA widgets.
/// These are extracted from `lifecycle { ... }` blocks.
#[derive(Debug, Clone)]
pub struct AuraLifecycle {
    /// Method name (e.g., "Init", "Destroy")
    pub name: String,

    /// Method body — stored as LogicPayload so generators can reuse existing body generation
    pub payload: LogicPayload,
}

impl AuraLifecycle {
    /// Create a new lifecycle method
    pub fn new(name: impl Into<String>, payload: LogicPayload) -> Self {
        Self {
            name: name.into(),
            payload,
        }
    }
}

/// Well-known lifecycle event names (dot-prefixed, as they appear in `on {}` blocks)
///
/// These are automatically extracted from the `on {}` block and moved into
/// `AuraWidget.lifecycle` during extraction, so generators can handle them
/// platform-specifically.
///
/// | AutoLang | Vue              | ArkTS              | Jetpack Compose       |
/// |----------|------------------|--------------------|-----------------------|
/// | `.Init`  | `onMounted`      | `aboutToAppear`    | `LaunchedEffect`      |
/// | `.Destroy`| `onUnmounted`   | `aboutToDisappear` | `DisposableEffect`    |
/// | `.Tick`  | `setInterval`    | `setInterval`      | `LaunchedEffect+delay`|
pub mod lifecycle {
    /// Component mounted/initialized — runs once after the component is added to the DOM
    pub const INIT: &str = ".Init";
    /// Component about to be destroyed — runs cleanup before removal
    pub const DESTROY: &str = ".Destroy";
    /// Periodic tick — handled separately via `tick_interval`
    pub const TICK: &str = ".Tick";
}

// ============================================================================
// From Implementations for Route Types
// ============================================================================

impl From<RouteDef> for AuraRoute {
    fn from(route: RouteDef) -> Self {
        // Derive widget_name from module name
        // Use smart capitalization that handles common patterns
        let widget_name = capitalize_module(&route.module);
        AuraRoute {
            path: route.path,
            module: route.module,
            widget_name,
            params: route.params,
        }
    }
}

/// Capitalize module name to widget name using smart word detection
///
/// Handles common patterns:
/// - "listpage" -> "ListPage"
/// - "gridpage" -> "GridPage"
/// - "listitem" -> "ListItem"
/// - "button" -> "Button"
fn capitalize_module(module: &str) -> String {
    // Common word boundaries to detect
    const WORD_BOUNDARIES: &[&str] = &[
        "page", "item", "card", "list", "grid", "box", "text", "input",
        "button", "switch", "slider", "checkbox", "radio", "toggle",
        "image", "icon", "badge", "chip", "tab", "table", "progress",
        "header", "footer", "nav", "menu", "sidebar", "panel", "modal",
        "dialog", "form", "field", "area", "view", "screen", "widget"
    ];

    let lower = module.to_lowercase();

    // Try to find word boundaries
    for word in WORD_BOUNDARIES {
        if lower.ends_with(word) && lower.len() > word.len() {
            let prefix = &lower[..lower.len() - word.len()];
            let capitalized_prefix = capitalize_first(prefix);
            let capitalized_word = capitalize_first(word);
            return format!("{}{}", capitalized_prefix, capitalized_word);
        }
    }

    // Fallback: simple capitalization
    capitalize_first(module)
}

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    let first = chars.next().map(|c| c.to_uppercase().collect::<String>()).unwrap_or_default();
    let rest: String = chars.collect();
    format!("{}{}", first, rest)
}

impl From<RoutesBlock> for AuraRoutes {
    fn from(block: RoutesBlock) -> Self {
        AuraRoutes {
            routes: block.routes.into_iter().map(|r| r.into()).collect(),
        }
    }
}

// ============================================================================
// State Definition
// ============================================================================

/// Decorator for state management (Plan 05-Nav Task 0)
#[derive(Debug, Clone)]
pub struct AuraDecorator {
    /// Decorator name (e.g., "Consume", "Provide")
    pub name: String,

    /// Decorator arguments (e.g., ["pathStack"])
    pub args: Vec<String>,
}

/// State variable definition
#[derive(Debug, Clone)]
pub struct AuraStateDef {
    /// Variable name (e.g., "count")
    pub name: String,

    /// Type information (e.g., Type::Int)
    pub type_info: Type,

    /// Initial value expression
    pub initial: AuraExpr,

    /// Decorators for state management (e.g., @Consume, @Provide)
    pub decorators: Vec<AuraDecorator>,
}

/// Prop definition for reusable components
#[derive(Debug, Clone)]
pub struct AuraProp {
    /// Prop name
    pub name: String,

    /// Type information
    pub type_info: Type,

    /// Default value (if any)
    pub default: Option<AuraExpr>,
}

// ============================================================================
// Computed Property Definition
// ============================================================================

/// Computed property definition
#[derive(Debug, Clone)]
pub struct AuraComputed {
    /// Property name (e.g., "activeCount")
    pub name: String,

    /// Computation expression
    pub expr: AuraExpr,
}

// ============================================================================
// Message Definition
// ============================================================================

/// Message type definition (for MVU pattern)
#[derive(Debug, Clone)]
pub struct AuraMessage {
    /// Message type name (e.g., "Msg")
    pub name: String,

    /// Message variants
    pub variants: Vec<AuraMsgVariant>,
}

/// Message variant
#[derive(Debug, Clone)]
pub struct AuraMsgVariant {
    /// Variant name (e.g., "Inc", "Dec")
    pub name: String,

    /// Optional payload type
    pub payload: Option<Type>,
}

// ============================================================================
// View Tree
// ============================================================================

/// Event handler with optional parameters
#[derive(Debug, Clone)]
pub struct AuraEvent {
    /// Handler pattern (e.g., ".Inc" or "Msg::Inc")
    pub handler: String,

    /// Optional parameters (e.g., ["todo.id"] for .Delete(todo.id))
    pub params: Vec<String>,
}

/// AURA property value - can be an expression or a class binding
#[derive(Debug, Clone)]
pub enum AuraPropValue {
    /// Regular expression value
    Expr(AuraExpr),

    /// Style binding: { completed: todo.done }
    StyleBinding(Vec<AuraStyleBinding>),
}

/// A single style binding entry
#[derive(Debug, Clone)]
pub struct AuraStyleBinding {
    /// Style name (e.g., "completed")
    pub style_name: String,

    /// Condition expression
    pub condition: AuraExpr,
}

/// View node: element or text
#[derive(Debug, Clone)]
pub enum AuraNode {
    /// Element node with tag, props, events, and children
    Element {
        /// Tag name (e.g., "col", "button", "h2")
        tag: String,

        /// Properties (key-value pairs, values can be dynamic or class bindings)
        props: HashMap<String, AuraPropValue>,

        /// Event handlers (event name -> AuraEvent)
        events: HashMap<String, AuraEvent>,

        /// Child nodes
        children: Vec<AuraNode>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,

        /// Stable debug ID assigned during extraction (Plan 273)
        debug_id: Option<AuraNodeId>,
    },

    /// Text node (literal or interpolated)
    Text(AuraTextContent),

    /// For loop: for item in .list { body }
    ForLoop {
        /// Loop variable name
        var: String,

        /// Optional index variable
        index: Option<String>,

        /// Iterable expression
        iterable: String,

        /// Loop body nodes
        body: Vec<AuraNode>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,

        /// Stable debug ID assigned during extraction (Plan 273)
        debug_id: Option<AuraNodeId>,
    },

    /// Conditional: if condition { then_body } else { else_body }
    Conditional {
        /// Condition expression
        condition: String,

        /// Body when condition is true
        then_body: Vec<AuraNode>,

        /// Optional else body
        else_body: Option<Vec<AuraNode>>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,

        /// Stable debug ID assigned during extraction (Plan 273)
        debug_id: Option<AuraNodeId>,
    },

    /// Component instantiation
    Component {
        /// Component name
        name: String,

        /// Properties passed to component
        props: HashMap<String, AuraExpr>,

        /// Event handlers
        events: HashMap<String, AuraEvent>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,

        /// Stable debug ID assigned during extraction (Plan 273)
        debug_id: Option<AuraNodeId>,
    },

    /// Router outlet: renders matched child route (Plan 105)
    Outlet,

    /// Navigation link with routing (Plan 105)
    Link {
        /// Target path for router-link
        to: String,

        /// Text content (optional, for shorthand form)
        text: String,

        /// href for external links (optional)
        href: String,

        /// Child content
        children: Vec<AuraNode>,

        /// Source span: (byte_offset, byte_length) in the .at file
        span: Option<(usize, usize)>,

        /// Stable debug ID assigned during extraction (Plan 273)
        debug_id: Option<AuraNodeId>,
    },
}

/// Text content: can be literal or contain interpolations
#[derive(Debug, Clone)]
pub enum AuraTextContent {
    /// Literal text
    Literal(String),

    /// Interpolated text with state references
    /// Format: "Current Count: ${.count}"
    Interpolated {
        /// Raw text with ${...} placeholders
        template: String,

        /// Extracted state references (e.g., ["count"])
        bindings: Vec<String>,
    },
}

impl AuraNode {
    /// Create a new element node
    pub fn element(tag: impl Into<String>) -> Self {
        AuraNode::Element {
            tag: tag.into(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: Vec::new(),
            span: None,
            debug_id: None,
        }
    }

    /// Create a text node
    pub fn text(content: impl Into<String>) -> Self {
        AuraNode::Text(AuraTextContent::Literal(content.into()))
    }

    /// Add a prop
    pub fn with_prop(mut self, key: impl Into<String>, value: AuraExpr) -> Self {
        if let AuraNode::Element { props, .. } = &mut self {
            props.insert(key.into(), AuraPropValue::Expr(value));
        }
        self
    }

    /// Add an event handler
    pub fn with_event(mut self, event: impl Into<String>, handler: impl Into<String>) -> Self {
        if let AuraNode::Element { events, .. } = &mut self {
            events.insert(event.into(), AuraEvent {
                handler: handler.into(),
                params: Vec::new(),
            });
        }
        self
    }

    /// Add a child node
    pub fn with_child(mut self, child: AuraNode) -> Self {
        if let AuraNode::Element { children, .. } = &mut self {
            children.push(child);
        }
        self
    }
}

// ============================================================================
// Logic Payload
// ============================================================================

/// Logic payload: supports both AOT and dynamic execution
///
/// This allows handlers to be:
/// - Transpiled to target language (React/Compose)
/// - Executed by AutoVM (GPUI dynamic)
#[derive(Debug, Clone)]
pub enum LogicPayload {
    /// AURA IR block (simplified statement types for handlers)
    AstBlock(Vec<AuraStmt>),

    /// Original AutoLang AST statements for a2ts delegation
    AstStmts(Vec<crate::ast::Stmt>),

    /// Bytecode for AutoVM dynamic execution (GPUI)
    /// Pre-compiled bytecode that can be executed at runtime
    Bytecode(Vec<u8>),
}

// ============================================================================
// Expressions
// ============================================================================

/// AURA expression: simplified expression types for UI
#[derive(Debug, Clone)]
pub enum AuraExpr {
    /// Literal string
    Literal(String),

    /// Integer literal
    Int(i64),

    /// Float literal
    Float(f64),

    /// Boolean literal
    Bool(bool),

    /// State reference (e.g., "count" from "${.count}")
    /// The "." prefix indicates it's a state variable reference
    StateRef(String),

    /// Message variant reference (e.g., "Msg::Inc")
    MsgVariant {
        /// Message type name
        msg_type: String,
        /// Variant name
        variant: String,
    },

    /// Binary operation
    Binary {
        left: Box<AuraExpr>,
        op: AuraBinOp,
        right: Box<AuraExpr>,
    },

    /// Unary operation
    Unary {
        op: AuraUnaryOp,
        operand: Box<AuraExpr>,
    },

    /// Method call: object.method(args)
    MethodCall {
        /// Object being called on (e.g., "todos")
        object: Box<AuraExpr>,
        /// Method name (e.g., "push", "filter")
        method: String,
        /// Arguments
        args: Vec<AuraExpr>,
    },

    /// Array literal
    Array(Vec<AuraExpr>),

    /// Object literal: { key: value, ... }
    Object(HashMap<String, AuraExpr>),

    /// Conditional expression: if cond { then } else { else }
    /// Used for conditional values like `style: if x {"a"} else {"b"}`
    If {
        cond: Box<AuraExpr>,
        then_branch: Box<AuraExpr>,
        else_branch: Option<Box<AuraExpr>>,
    },

    /// Lambda expression: |params| body
    Lambda {
        /// Parameter names
        params: Vec<String>,
        /// Body expression
        body: Box<AuraExpr>,
    },

    /// Field access: object.field
    FieldAccess {
        /// Object
        object: Box<AuraExpr>,
        /// Field name
        field: String,
    },

    /// Programmatic navigation (Plan 105): Nav.to("/path", { param: value })
    NavCall {
        /// Target path
        path: String,

        /// Navigation parameters
        params: HashMap<String, AuraExpr>,
    },

    /// Constructor call: TypeName(args)
    /// For example: NavPathStack() -> new NavPathStack()
    Constructor {
        /// Type name
        type_name: String,

        /// Arguments
        args: Vec<AuraExpr>,
    },

    /// Index access: target[index]
    /// For example: notes[active_id]
    Index {
        /// Target collection (array, object)
        target: Box<AuraExpr>,
        /// Index expression
        index: Box<AuraExpr>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuraBinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuraUnaryOp {
    Neg,
    Not,
}

// ============================================================================
// Statements
// ============================================================================

/// AURA statement: simplified statement types for handlers
#[derive(Debug, Clone)]
pub enum AuraStmt {
    /// Assignment: target = value
    Assign {
        target: String,
        value: AuraExpr,
    },

    /// Update with operator: target op= value (e.g., count += 1)
    Update {
        target: String,
        op: AuraUpdateOp,
        value: AuraExpr,
    },

    /// Method call statement: object.method(args)
    MethodCall {
        /// Object being called on
        object: String,
        /// Method name
        method: String,
        /// Arguments
        args: Vec<AuraExpr>,
    },
}

/// Update operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuraUpdateOp {
    AddAssign,  // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
}

impl fmt::Display for AuraUpdateOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuraUpdateOp::AddAssign => write!(f, "+="),
            AuraUpdateOp::SubAssign => write!(f, "-="),
            AuraUpdateOp::MulAssign => write!(f, "*="),
            AuraUpdateOp::DivAssign => write!(f, "/="),
        }
    }
}

// ============================================================================
// AURA Module
// ============================================================================

/// AURA Module: contains multiple widgets and app definition
#[derive(Debug, Clone)]
pub struct AuraModule {
    /// Module name
    pub name: String,

    /// Widgets defined in this module
    pub widgets: Vec<AuraWidget>,

    /// Messages defined at module level
    pub messages: Vec<AuraMessage>,

    /// App definition (entry point)
    pub app: Option<AuraApp>,
}

/// App definition: the entry point for UI applications
#[derive(Debug, Clone)]
pub struct AuraApp {
    /// App name
    pub name: String,

    /// Root widget
    pub root: String,

    /// Window properties
    pub window: AuraWindow,
}

/// Window properties
#[derive(Debug, Clone)]
pub struct AuraWindow {
    /// Window title
    pub title: String,

    /// Window width
    pub width: u32,

    /// Window height
    pub height: u32,
}

impl Default for AuraWindow {
    fn default() -> Self {
        AuraWindow {
            title: "Auto App".to_string(),
            width: 800,
            height: 600,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aura_node_element() {
        let node = AuraNode::element("col")
            .with_prop("gap", AuraExpr::Int(16))
            .with_child(AuraNode::text("Hello"));

        match node {
            AuraNode::Element { tag, props, children, .. } => {
                assert_eq!(tag, "col");
                assert_eq!(props.len(), 1);
                assert_eq!(children.len(), 1);
            }
            _ => panic!("Expected Element node"),
        }
    }

    #[test]
    fn test_aura_node_text() {
        let node = AuraNode::text("Hello World");

        match node {
            AuraNode::Text(AuraTextContent::Literal(s)) => {
                assert_eq!(s, "Hello World");
            }
            _ => panic!("Expected Text node"),
        }
    }

    #[test]
    fn test_aura_state_def() {
        let state = AuraStateDef {
            name: "count".to_string(),
            type_info: Type::Int,
            initial: AuraExpr::Int(0),
            decorators: vec![],
        };

        assert_eq!(state.name, "count");
        assert!(matches!(state.type_info, Type::Int));
    }

    #[test]
    fn test_aura_message() {
        let msg = AuraMessage {
            name: "Msg".to_string(),
            variants: vec![
                AuraMsgVariant {
                    name: "Inc".to_string(),
                    payload: None,
                },
                AuraMsgVariant {
                    name: "Set".to_string(),
                    payload: Some(Type::Int),
                },
            ],
        };

        assert_eq!(msg.variants.len(), 2);
    }

    #[test]
    fn test_aura_expr_state_ref() {
        let expr = AuraExpr::StateRef("count".to_string());
        match expr {
            AuraExpr::StateRef(name) => assert_eq!(name, "count"),
            _ => panic!("Expected StateRef"),
        }
    }

    #[test]
    fn test_aura_stmt_update() {
        let stmt = AuraStmt::Update {
            target: "count".to_string(),
            op: AuraUpdateOp::AddAssign,
            value: AuraExpr::Int(1),
        };

        match stmt {
            AuraStmt::Update { target, op, value } => {
                assert_eq!(target, "count");
                assert_eq!(op, AuraUpdateOp::AddAssign);
                match value {
                    AuraExpr::Int(v) => assert_eq!(v, 1),
                    _ => panic!("Expected Int"),
                }
            }
            _ => panic!("Expected Update"),
        }
    }

    #[test]
    fn test_aura_widget() {
        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            }],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col"),
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
        };

        assert_eq!(widget.name, "Counter");
        assert_eq!(widget.state_vars.len(), 1);
    }

    #[test]
    fn test_logic_payload() {
        let ast_payload = LogicPayload::AstBlock(vec![]);
        let bytecode_payload = LogicPayload::Bytecode(vec![0x01, 0x02, 0x03]);

        assert!(matches!(ast_payload, LogicPayload::AstBlock(_)));
        assert!(matches!(bytecode_payload, LogicPayload::Bytecode(_)));
    }

    #[test]
    fn test_aura_route() {
        let route = AuraRoute {
            path: "/user/:id".to_string(),
            module: "user".to_string(),
            widget_name: "User".to_string(),
            params: vec!["id".to_string()],
        };

        assert_eq!(route.path, "/user/:id");
        assert_eq!(route.module, "user");
        assert_eq!(route.params, vec!["id"]);
    }

    #[test]
    fn test_aura_routes() {
        let routes = AuraRoutes::with_routes(vec![
            AuraRoute {
                path: "/".to_string(),
                module: "index".to_string(),
                widget_name: "Index".to_string(),
                params: vec![],
            },
            AuraRoute {
                path: "/user/:id".to_string(),
                module: "user".to_string(),
                widget_name: "User".to_string(),
                params: vec!["id".to_string()],
            },
        ]);

        assert_eq!(routes.routes.len(), 2);
        assert_eq!(routes.routes[0].path, "/");
        assert_eq!(routes.routes[1].params, vec!["id"]);
    }

    #[test]
    fn test_aura_route_from_route_def() {
        let route_def = RouteDef::new("/user/:id".to_string(), "user".to_string());
        let aura_route: AuraRoute = route_def.into();

        assert_eq!(aura_route.path, "/user/:id");
        assert_eq!(aura_route.module, "user");
        assert_eq!(aura_route.params, vec!["id"]);
    }

    #[test]
    fn test_aura_routes_from_routes_block() {
        let mut block = RoutesBlock::new();
        block.add_route(RouteDef::new("/button".to_string(), "button".to_string()));
        block.add_route(RouteDef::new("/user/:id".to_string(), "user".to_string()));

        let aura_routes: AuraRoutes = block.into();

        assert_eq!(aura_routes.routes.len(), 2);
        assert_eq!(aura_routes.routes[0].path, "/button");
        assert_eq!(aura_routes.routes[1].params, vec!["id"]);
    }

    #[test]
    fn test_aura_expr_nav_call() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), AuraExpr::Int(42));

        let nav_call = AuraExpr::NavCall {
            path: "/user".to_string(),
            params,
        };

        match nav_call {
            AuraExpr::NavCall { path, params } => {
                assert_eq!(path, "/user");
                assert_eq!(params.len(), 1);
                // Check that the params contain "id" key with an Int value
                let id_param = params.get("id");
                assert!(id_param.is_some());
                match id_param.unwrap() {
                    AuraExpr::Int(n) => assert_eq!(*n, 42),
                    _ => panic!("Expected Int"),
                }
            }
            _ => panic!("Expected NavCall"),
        }
    }
}
