//! AURA Extraction - AST → AURA conversion
//!
//! This module implements the extraction pipeline that converts
//! WidgetDecl AST nodes into AuraWidget structures.
//!
//! ## Key Principles
//!
//! - **1:1 Lossless Mapping**: All semantic information is preserved
//! - **Purity**: View tree contains no logic, only layout and bindings
//! - **Separation**: Handlers are extracted as LogicPayload

use super::types::*;
use crate::ast::{Expr, Type, Key, ViewPropValue};
use std::collections::HashMap;

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert Key to String
fn key_to_string(key: &Key) -> String {
    match key {
        Key::NamedKey(name) => name.as_str().to_string(),
        Key::IntKey(i) => i.to_string(),
        Key::BoolKey(b) => b.to_string(),
        Key::StrKey(s) => s.to_string(),
    }
}

/// Plan 345 (gap K2/N4): classify an `on*` attribute key as a DOM-native
/// event (→ `events`, emitted `@click` etc.) vs a callback prop (→ `props`,
/// emitted `:on_select="Handler"`). Only the common DOM event names are
/// native; anything else starting with `on` is a callback prop.
fn is_native_event_key(key: &str) -> bool {
    matches!(
        key,
        "onclick" | "onClick" | "on_click"
            | "oninput" | "onInput" | "on_input"
            | "onchange" | "onChange" | "on_change"
            | "onenter" | "onEnter" | "on_enter"
            | "onsubmit"
            | "onkeyup" | "onkeydown" | "onkeypress"
            | "onfocus" | "onblur"
    )
}

// ============================================================================
// Extraction Error
// ============================================================================

/// Errors during AURA extraction
#[derive(Debug, Clone)]
pub enum ExtractError {
    /// Unsupported expression type in view
    UnsupportedExpr(String),

    /// Unsupported statement type in handler
    UnsupportedStmt(String),

    /// Invalid state reference
    InvalidStateRef(String),

    /// Missing required field
    MissingField(String),
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtractError::UnsupportedExpr(msg) => {
                write!(f, "Unsupported expression in view: {}", msg)
            }
            ExtractError::UnsupportedStmt(msg) => {
                write!(f, "Unsupported statement in handler: {}", msg)
            }
            ExtractError::InvalidStateRef(msg) => {
                write!(f, "Invalid state reference: {}", msg)
            }
            ExtractError::MissingField(msg) => {
                write!(f, "Missing required field: {}", msg)
            }
        }
    }
}

impl std::error::Error for ExtractError {}

pub type ExtractResult<T> = Result<T, ExtractError>;

// ============================================================================
// Statement Extractor
// ============================================================================

// PR-5: extract_stmt removed — AuraStmt eliminated. Handler bodies now use
// LogicPayload::AstStmts (base crate::ast::Stmt) directly. See
// docs/design/dialect-extension-diagnosis.md §6.4.

// ============================================================================
// View Tree Extractor
// ============================================================================

/// Extract view tree from AST expression
///
/// This handles the special UI view syntax:
/// - `col { ... }` → Element with tag "col"
/// - `button +` → Element with tag "button" and text "+"
/// - `h2 > text` → Element with tag "h2" and text child
/// - `${.state}` → Interpolated text
pub fn extract_view_tree(expr: &Expr) -> ExtractResult<AuraNode> {
    match expr {
        // Object expression: represents a UI element with props/children
        Expr::Object(pairs) => {
            // The first pair's key is typically the tag name
            if pairs.is_empty() {
                return Ok(AuraNode::element("div"));
            }

            let first_pair = &pairs[0];
            let tag = key_to_string(&first_pair.key);

            // Extract props and children from the object
            let mut props = HashMap::new();
            let mut events = HashMap::new();
            let children = Vec::new();

            for pair in pairs.iter().skip(1) {
                let key = key_to_string(&pair.key);
                match key.as_str() {
                    // Event handlers
                    "onclick" | "onClick" | "on_click" => {
                        let handler = extract_event_handler(&pair.value)?;
                        events.insert("onclick".to_string(), handler);
                    }
                    // Regular props
                    _ => {
                        let value = pair.value.as_ref().clone();
                        props.insert(key, AuraPropValue::Expr(value));
                    }
                }
            }

            Ok(AuraNode::Element {
                tag,
                props,
                events,
                children,
                span: None,
                debug_id: None,
            })
        }

        // Call expression: could be a UI element constructor
        Expr::Call(call) => {
            // Extract tag name from call name
            let tag = match call.name.as_ref() {
                Expr::Ident(name) => name.as_str().to_string(),
                _ => "div".to_string(),
            };

            let mut props = HashMap::new();
            let mut events = HashMap::new();
            let mut children = Vec::new();

            // Process arguments as props/children
            for arg in &call.args.args {
                match arg {
                    crate::ast::Arg::Pos(expr) => {
                        // Check if it's an object (props) or another node (child)
                        if let Expr::Object(pairs) = expr {
                            for pair in pairs {
                                let key = key_to_string(&pair.key);
                                // Plan 345 (gap K2/N4): only DOM-native `on*`
                                // keys are events; other `on_*` (e.g. on_select,
                                // on_submit) are callback props passed to child
                                // widgets, so they stay in `props` and emit as
                                // `:on_select="Handler"` (function ref).
                                if is_native_event_key(&key) {
                                    let handler = extract_event_handler(&pair.value)?;
                                    events.insert(key, handler);
                                } else {
                                    let value = pair.value.as_ref().clone();
                                    props.insert(key, AuraPropValue::Expr(value));
                                }
                            }
                        } else {
                            // Treat as child node
                            let child = extract_view_tree(expr)?;
                            children.push(child);
                        }
                    }
                    _ => {}
                }
            }

            Ok(AuraNode::Element {
                tag,
                props,
                events,
                children,
                span: None,
                debug_id: None,
            })
        }

        // String literal: text node
        Expr::Str(s) => Ok(AuraNode::text(s)),

        // F-string: interpolated text
        Expr::FStr(fstr) => {
            let template = fstr.to_string();
            // Extract bindings from the template
            let bindings = extract_fstr_bindings(&template);
            Ok(AuraNode::Text(AuraTextContent::Interpolated {
                template,
                bindings,
            }))
        }

        // Dot expression: .field → property reference (treated as interpolated text)
        // This handles cases like Text .title where .title is passed as an argument
        Expr::Dot(obj, field) => {
            match obj.as_ref() {
                // .field → state reference
                Expr::Ident(name) if name.as_str() == "." || name.as_str() == "self" => {
                    // Create interpolated text with single binding
                    let field_name = field.as_str();
                    Ok(AuraNode::Text(AuraTextContent::Interpolated {
                        template: format!("${{.{}}}", field_name),
                        bindings: vec![field_name.to_string()],
                    }))
                }
                // Other dot expressions: object.field → try to extract as child element
                _ => {
                    // Fall through to error for now
                    Err(ExtractError::UnsupportedExpr(format!(
                        "Cannot extract view tree from dot expression: {:?}",
                        expr
                    )))
                }
            }
        }

        _ => Err(ExtractError::UnsupportedExpr(format!(
            "Cannot extract view tree from: {:?}",
            expr
        ))),
    }
}

/// Extract event handler pattern from expression
fn extract_event_handler(expr: &Expr) -> ExtractResult<AuraEvent> {
    match expr {
        // Identifier: could be ".Inc" or "Msg.Inc"
        Expr::Ident(name) => {
            let name_str = name.as_str();
            if name_str.starts_with('.') {
                // Implicit member: .Inc -> Msg::Inc (need context)
                Ok(AuraEvent {
                    handler: format!("Msg::{}", &name_str[1..]),
                    params: Vec::new(),
                })
            } else {
                Ok(AuraEvent {
                    handler: name_str.to_string(),
                    params: Vec::new(),
                })
            }
        }
        // Dot access: Msg.Inc
        Expr::Dot(obj, field) => {
            let obj_name = match obj.as_ref() {
                Expr::Ident(name) => name.as_str(),
                _ => "Msg",
            };
            let field_name = field.as_str();
            Ok(AuraEvent {
                handler: format!("{}::{}", obj_name, field_name),
                params: Vec::new(),
            })
        }
        // Call expression: could be .Delete(todo.id)
        Expr::Call(call) => {
            let handler = match call.name.as_ref() {
                Expr::Ident(name) => {
                    let name_str = name.as_str();
                    if name_str.starts_with('.') {
                        format!("Msg::{}", &name_str[1..])
                    } else {
                        name_str.to_string()
                    }
                }
                _ => "Unknown".to_string(),
            };
            let params: Vec<String> = call.args.args.iter()
                .filter_map(|arg| {
                    if let crate::ast::Arg::Pos(expr) = arg {
                        Some(expr_to_string(expr))
                    } else {
                        None
                    }
                })
                .collect();
            Ok(AuraEvent { handler, params })
        }
        _ => Err(ExtractError::UnsupportedExpr(format!(
            "Cannot extract event handler from: {:?}",
            expr
        ))),
    }
}

/// Convert expression to a simple string representation
/// For ArkTS, converts self.xxx to this.xxx for state references
fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => {
            let name_str = name.as_str();
            // Convert .xxx to this.xxx for ArkTS (if somehow parsed as ident)
            if name_str.starts_with('.') {
                format!("this.{}", &name_str[1..])
            } else if name_str == "self" {
                // self -> this
                "this".to_string()
            } else {
                name_str.to_string()
            }
        }
        Expr::Int(n) => n.to_string(),
        Expr::Str(s) => format!("\"{}\"", s.as_str()),
        Expr::Dot(obj, field) => {
            // Check if this is self.field (parsed from .field syntax)
            if let Expr::Ident(name) = obj.as_ref() {
                let name_str = name.as_str();
                if name_str == "self" {
                    // self.field -> this.field
                    return format!("this.{}", field.as_str());
                }
            }
            let obj_str = expr_to_string(obj);
            format!("{}.{}", obj_str, field.as_str())
        }
        Expr::Object(pairs) => {
            let parts: Vec<String> = pairs.iter()
                .map(|pair| {
                    let key_str = key_to_string(&pair.key);
                    let value_str = expr_to_string(&pair.value);
                    format!("{}: {}", key_str, value_str)
                })
                .collect();
            format!("{{ {} }}", parts.join(", "))
        }
        _ => format!("{:?}", expr),
    }
}

/// Extract state bindings from f-string template
fn extract_fstr_bindings(template: &str) -> Vec<String> {
    let mut bindings = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(&next) = chars.peek() {
                if next == '{' {
                    chars.next(); // consume '{'
                    let mut var = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '}' {
                            chars.next(); // consume '}'
                            break;
                        }
                        var.push(ch);
                        chars.next();
                    }
                    // Remove leading '.' if present
                    let var = var.trim_start_matches('.');
                    bindings.push(var.to_string());
                } else if next.is_alphabetic() || next == '_' || next == '.' {
                    chars.next(); // consume first char
                    let mut var = String::new();
                    if next != '.' {
                        var.push(next);
                    }
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                            var.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // Remove leading '.' if present
                    let var = var.trim_start_matches('.').to_string();
                    bindings.push(var);
                }
            }
        }
    }

    bindings
}

// ============================================================================
// Type Extractor
// ============================================================================

/// Extract AURA type from AST type
pub fn extract_type(ty: &Type) -> Type {
    ty.clone() // For now, just clone since we're using the same Type enum
}

// ============================================================================
// Widget Declaration Extractor (Plan 096)
// ============================================================================

use crate::ast::{WidgetDecl, StoreDecl, ModelBlock, ViewBlock, OnBlock, BindBlock, MsgDecl, PropDecl, ViewNode, ViewText};

/// Extract AuraStore from parsed StoreDecl (Plan 351 / Design 18).
/// A store is a view-less widget: state + msg + handlers → module-level refs + actions.
pub fn extract_store_from_decl(decl: &StoreDecl) -> ExtractResult<AuraStore> {
    let state_vars = if let Some(model) = &decl.model {
        extract_model_fields(model)?
    } else {
        Vec::new()
    };
    let messages: Vec<AuraMessage> = decl.messages.iter()
        .map(|m| extract_msg_decl(m))
        .collect();
    let (handlers, handler_params) = if let Some(on) = &decl.on {
        extract_on_block(on)?
    } else {
        (HashMap::new(), HashMap::new())
    };
    // Plan 367 P2-2: extract computed properties (same pattern as widget)
    let computed: Vec<AuraComputed> = if let Some(ref computed_block) = decl.computed {
        computed_block.properties.iter()
            .map(|p| {
                Ok(AuraComputed {
                    name: p.name.as_str().to_string(),
                    expr: p.expr.clone(),
                })
            })
            .collect::<ExtractResult<Vec<_>>>()?
    } else {
        Vec::new()
    };
    Ok(AuraStore {
        name: decl.name.as_str().to_string(),
        state_vars,
        messages,
        handlers,
        handler_params,
        api_imports: Vec::new(),
        computed,
    })
}

/// Extract AuraWidget from parsed WidgetDecl
pub fn extract_widget_from_decl(decl: &WidgetDecl) -> ExtractResult<AuraWidget> {
    // Extract state variables from model
    let mut state_vars = if let Some(model) = &decl.model {
        extract_model_fields(model)?
    } else {
        Vec::new()
    };

    // Extract messages
    let messages: Vec<AuraMessage> = decl.messages.iter()
        .map(|m| extract_msg_decl(m))
        .collect();

    // Extract view tree
    let view_tree = if let Some(view) = &decl.view {
        extract_view_block(view)?
    } else {
        AuraNode::element("div")
    };

    // Extract handlers
    let (mut handlers, handler_params) = if let Some(on) = &decl.on {
        extract_on_block(on)?
    } else {
        (HashMap::new(), HashMap::new())
    };

    // Detect .Tick handler and extract interval from model vars
    let tick_interval = if handlers.keys().any(|k| k == ".Tick") {
        // Look for a model var named "interval" (default 1000ms)
        let interval_val = state_vars.iter()
            .find(|v| v.name == "interval")
            .and_then(|v| {
                if let Expr::Int(n) = &v.initial {
                    Some(*n as u32)
                } else {
                    None
                }
            })
            .or(Some(1000));
        // Remove "interval" from state_vars so it doesn't become a ref()
        state_vars.retain(|v| v.name != "interval");
        interval_val
    } else {
        None
    };

    // Extract lifecycle handlers (.Init, .Destroy) from the handlers map
    // and move them into the lifecycle vec. .Tick is handled separately via tick_interval.
    let lifecycle_names = [
        crate::aura::types::lifecycle::INIT,
        crate::aura::types::lifecycle::DESTROY,
    ];
    let lifecycle_events: Vec<crate::aura::types::AuraLifecycle> = lifecycle_names.iter()
        .filter_map(|name| {
            handlers.remove(*name).map(|payload| {
                // name[1..] strips the leading "."
                crate::aura::types::AuraLifecycle::new(&name[1..], payload)
            })
        })
        .collect();

    // Extract props
    let props: Vec<AuraProp> = decl.props.iter()
        .map(|p| extract_prop_decl(p))
        .collect();

    // Extract computed properties
    let computed: Vec<AuraComputed> = if let Some(ref computed_block) = decl.computed {
        computed_block.properties.iter()
            .map(|p| {
                Ok(AuraComputed {
                    name: p.name.as_str().to_string(),
                    expr: p.expr.clone(),
                })
            })
            .collect::<ExtractResult<Vec<_>>>()?
    } else {
        Vec::new()
    };

    // Extract routes (Plan 105)
    let routes = if let Some(ref routes_block) = decl.routes {
        Some(crate::aura::types::AuraRoutes::from(routes_block.clone()))
    } else {
        None
    };

    // Assign stable debug IDs to AuraNode tree (Plan 274)
    let mut view_tree = view_tree;
    let span_map = assign_node_ids(&mut view_tree);

    Ok(AuraWidget {
        name: decl.name.as_str().to_string(),
        state_vars,
        computed,
        messages,
        view_tree,
        handlers,
        handler_params,
        props,
        routes,
        lifecycle: lifecycle_events,
        tick_interval,
        span_map,
        key_bindings: extract_key_bindings(&decl.bind),
        api_imports: Vec::new(),
    }
)
}

/// Extract key bindings from bind block (Plan 275)
fn extract_key_bindings(bind: &Option<BindBlock>) -> HashMap<String, String> {
    match bind {
        Some(block) => block.bindings.iter()
            .map(|kb| (kb.key.clone(), kb.handler.clone()))
            .collect(),
        None => HashMap::new(),
    }
}

/// Extract state variables from model block
fn extract_model_fields(model: &ModelBlock) -> ExtractResult<Vec<AuraStateDef>> {
    model.fields.iter()
        .map(|field| {
            Ok(AuraStateDef {
                name: field.name.as_str().to_string(),
                type_info: field.ty.clone(),
                initial: field.init.clone(),
                decorators: field.decorators.iter()
                    .map(|d| AuraDecorator {
                        name: d.name.as_str().to_string(),
                        args: d.args.clone(),
                    })
                    .collect(),
            })
        })
        .collect()
}

/// Extract message declaration
fn extract_msg_decl(msg: &MsgDecl) -> AuraMessage {
    AuraMessage {
        name: msg.name.as_str().to_string(),
        variants: msg.variants.iter()
            .map(|v| AuraMsgVariant {
                name: v.name.as_str().to_string(),
                payload: v.payload.clone(),
            })
            .collect(),
    }
}

/// Extract view tree from view block
fn extract_view_block(view: &ViewBlock) -> ExtractResult<AuraNode> {
    extract_view_node(&view.root)
}

/// Extract view node from parsed ViewNode
fn extract_view_node(node: &ViewNode) -> ExtractResult<AuraNode> {
    match node {
        ViewNode::Element { tag, props, events, children, span } => {
            let aura_props: HashMap<String, AuraPropValue> = props.iter()
                .map(|p| {
                    let value = match &p.value {
                        ViewPropValue::Expr(expr) => {
                            AuraPropValue::Expr(expr.clone())
                        }
                        ViewPropValue::StyleBinding(bindings) => {
                            let aura_bindings: Vec<AuraStyleBinding> = bindings.iter()
                                .map(|b| {
                                    Ok(AuraStyleBinding {
                                        style_name: b.style_name.clone(),
                                        condition: b.condition.clone(),
                                    })
                                })
                                .collect::<ExtractResult<_>>()?;
                            AuraPropValue::StyleBinding(aura_bindings)
                        }
                    };
                    Ok((p.name.clone(), value))
                })
                .collect::<ExtractResult<_>>()?;

            let aura_events: HashMap<String, AuraEvent> = events.iter()
                .map(|e| {
                    let event = AuraEvent {
                        handler: e.handler.clone(),
                        params: e.params.clone(),
                    };
                    (e.name.clone(), event)
                })
                .collect();

            let aura_children: Vec<AuraNode> = children.iter()
                .map(|c| extract_view_node(c))
                .collect::<ExtractResult<_>>()?;

            Ok(AuraNode::Element {
                tag: tag.clone(),
                props: aura_props,
                events: aura_events,
                children: aura_children,
                span: *span,
                debug_id: None,
            })
        }
        ViewNode::Text(content) => {
            let text_content = match content {
                ViewText::Literal(s) => {
                    AuraTextContent::Literal(s.clone())
                }
                ViewText::Interpolated { template, bindings } => {
                    AuraTextContent::Interpolated {
                        template: template.clone(),
                        bindings: bindings.clone(),
                    }
                }
            };
            Ok(AuraNode::Text(text_content))
        }
        ViewNode::ForLoop { var, index, iterable, body, span } => {
            let aura_body: Vec<AuraNode> = body.iter()
                .map(|c| extract_view_node(c))
                .collect::<ExtractResult<_>>()?;

            Ok(AuraNode::ForLoop {
                var: var.clone(),
                index: index.clone(),
                iterable: iterable.clone(),
                body: aura_body,
                span: *span,
                debug_id: None,
            })
        }
        ViewNode::Conditional { condition, then_body, else_body, span } => {
            let aura_then: Vec<AuraNode> = then_body.iter()
                .map(|c| extract_view_node(c))
                .collect::<ExtractResult<_>>()?;

            let aura_else = if let Some(else_nodes) = else_body {
                let nodes: Vec<AuraNode> = else_nodes.iter()
                    .map(|c| extract_view_node(c))
                    .collect::<ExtractResult<_>>()?;
                Some(nodes)
            } else {
                None
            };

            Ok(AuraNode::Conditional {
                condition: condition.clone(),
                then_body: aura_then,
                else_body: aura_else,
                span: *span,
                debug_id: None,
            })
        }
        ViewNode::Component { name, props, events, span } => {
            let aura_props: HashMap<String, Expr> = props.iter()
                .filter_map(|p| {
                    match &p.value {
                        ViewPropValue::Expr(expr) => {
                            Some(Ok((p.name.clone(), expr.clone())))
                        }
                        ViewPropValue::StyleBinding(_) => {
                            // Class bindings not supported for component props
                            None
                        }
                    }
                })
                .collect::<ExtractResult<_>>()?;

            let aura_events: HashMap<String, AuraEvent> = events.iter()
                .map(|e| {
                    let event = AuraEvent {
                        handler: e.handler.clone(),
                        params: e.params.clone(),
                    };
                    (e.name.clone(), event)
                })
                .collect();

            Ok(AuraNode::Component {
                name: name.clone(),
                props: aura_props,
                events: aura_events,
                span: *span,
                debug_id: None,
            })
        }
        // Plan 105: Router outlet and link
        ViewNode::Outlet => Ok(AuraNode::Outlet),
        ViewNode::Link { to, text, href, children, span } => {
            let aura_children: Vec<AuraNode> = children.iter()
                .map(|c| extract_view_node(c))
                .collect::<ExtractResult<_>>()?;
            Ok(AuraNode::Link {
                to: to.clone(),
                text: text.clone(),
                href: href.clone(),
                children: aura_children,
                span: *span,
                debug_id: None,
            })
        }
    }
}

/// Assign stable AuraNodeIds to the AuraNode tree via DFS traversal.
/// Returns a SpanMap mapping each AuraNodeId to its source info.
/// Called once after extraction, before constructing AuraWidget.
fn assign_node_ids(root: &mut AuraNode) -> std::collections::HashMap<AuraNodeId, SpanInfo> {
    let mut next_id: u32 = 0;
    let mut span_map = std::collections::HashMap::new();
    assign_node_ids_recursive(root, &mut next_id, &mut span_map);
    span_map
}

fn assign_node_ids_recursive(
    node: &mut AuraNode,
    next_id: &mut u32,
    span_map: &mut std::collections::HashMap<AuraNodeId, SpanInfo>,
) {
    let id = AuraNodeId(*next_id);
    *next_id += 1;

    match node {
        AuraNode::Element { tag, props, children, span, debug_id, .. } => {
            *debug_id = Some(id);
            // Extract user_id from props if present
            let user_id = props.get("id").and_then(|v| match v {
                crate::aura::types::AuraPropValue::Expr(crate::ast::Expr::Str(s)) => Some(s.as_str().to_string()),
                _ => None,
            });
            span_map.insert(id, SpanInfo {
                span: *span,
                aura_tag: tag.clone(),
                user_id,
            });
            for child in children.iter_mut() {
                assign_node_ids_recursive(child, next_id, span_map);
            }
        }
        AuraNode::Text(_) => {
            // Text nodes don't get a debug_id — they have no span field
        }
        AuraNode::ForLoop { var: _, index: _, iterable: _, body, span, debug_id } => {
            *debug_id = Some(id);
            span_map.insert(id, SpanInfo {
                span: *span,
                aura_tag: "for".to_string(),
                user_id: None,
            });
            for child in body.iter_mut() {
                assign_node_ids_recursive(child, next_id, span_map);
            }
        }
        AuraNode::Conditional { condition: _, then_body, else_body, span, debug_id } => {
            *debug_id = Some(id);
            span_map.insert(id, SpanInfo {
                span: *span,
                aura_tag: "if".to_string(),
                user_id: None,
            });
            for child in then_body.iter_mut() {
                assign_node_ids_recursive(child, next_id, span_map);
            }
            if let Some(else_children) = else_body {
                for child in else_children.iter_mut() {
                    assign_node_ids_recursive(child, next_id, span_map);
                }
            }
        }
        AuraNode::Component { name, props: _, events: _, span, debug_id } => {
            *debug_id = Some(id);
            span_map.insert(id, SpanInfo {
                span: *span,
                aura_tag: name.clone(),
                user_id: None,
            });
        }
        AuraNode::Outlet => {
            // Outlet doesn't get a debug_id
        }
        AuraNode::Link { to: _, text: _, href: _, children, span, debug_id } => {
            *debug_id = Some(id);
            span_map.insert(id, SpanInfo {
                span: *span,
                aura_tag: "link".to_string(),
                user_id: None,
            });
            for child in children.iter_mut() {
                assign_node_ids_recursive(child, next_id, span_map);
            }
        }
    }
}

/// Extract handlers from on block
fn extract_on_block(on: &OnBlock) -> ExtractResult<(HashMap<String, LogicPayload>, HashMap<String, Vec<String>>)> {
    let mut handlers = HashMap::new();
    let mut handler_params = HashMap::new();

    for handler in &on.handlers {
        let pattern = handler.pattern.clone();
        // Keep original AST stmts for a2ts delegation
        let original_stmts: Vec<crate::ast::Stmt> = handler.body.stmts.clone();
        handlers.insert(pattern.clone(), LogicPayload::AstStmts(original_stmts));
        if !handler.params.is_empty() {
            handler_params.insert(pattern, handler.params.clone());
        }
    }

    Ok((handlers, handler_params))
}

/// Extract prop declaration
fn extract_prop_decl(prop: &PropDecl) -> AuraProp {
    AuraProp {
        name: prop.name.as_str().to_string(),
        type_info: prop.ty.clone(),
        default: prop.default.clone(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::AutoStr;

    #[test]
    fn test_extract_fstr_bindings() {
        let template = "Count: ${.count}";
        let bindings = extract_fstr_bindings(template);
        assert_eq!(bindings, vec!["count"]);

        let template2 = "Name: $name, Age: ${.age}";
        let bindings2 = extract_fstr_bindings(template2);
        assert_eq!(bindings2, vec!["name", "age"]);
    }

    #[test]
    fn test_extract_view_tree_text() {
        let expr = Expr::Str(AutoStr::from("Hello"));
        let node = extract_view_tree(&expr).unwrap();
        match node {
            AuraNode::Text(AuraTextContent::Literal(s)) => assert_eq!(s, "Hello"),
            _ => panic!("Expected Text node"),
        }
    }
}
