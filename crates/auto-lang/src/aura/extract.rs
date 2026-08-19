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
use crate::ast::{Expr, Type, Key, ViewPropValue, ViewProp, ViewEvent};
use std::collections::HashMap;

// Plan 367 P2-3: thread-local store for view fragments.
// Populated during module-level extraction, consumed during view tree extraction.
thread_local! {
    static VIEW_FRAGMENTS: std::cell::RefCell<HashMap<String, crate::ast::ui::ViewFragmentDecl>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Register a view fragment for inline expansion (Plan 367 P2-3).
pub fn register_view_fragment(frag: &crate::ast::ui::ViewFragmentDecl) {
    VIEW_FRAGMENTS.with(|cell| {
        cell.borrow_mut().insert(frag.name.as_str().to_string(), frag.clone());
    });
}

/// Clear all registered view fragments (call before processing a new module).
pub fn clear_view_fragments() {
    VIEW_FRAGMENTS.with(|cell| cell.borrow_mut().clear());
}

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
    // Plan 402: strip `.prevent`/`.stop` modifiers (e.g. oncontextmenu.prevent)
    // before checking, so the base event name is recognized.
    let base = key.split('.').next().unwrap_or(key);
    matches!(
        base,
        "onclick" | "onClick" | "on_click"
            | "oninput" | "onInput" | "on_input"
            | "onchange" | "onChange" | "on_change"
            | "onenter" | "onEnter" | "on_enter"
            | "onsubmit"
            | "onkeyup" | "onkeydown" | "onkeypress"
            | "onfocus" | "onblur"
            | "oncontextmenu" | "onContextMenu" | "on_contextmenu"
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
                    // Plan 402: contextmenu (right-click) event. The stored key
                    // keeps modifiers (e.g. oncontextmenu.prevent) so codegen
                    // backends see them; consumers use base-aware lookup
                    // (crate::aura::aura_events_get_base). Only the
                    // `onContextMenu` casing is normalized to `oncontextmenu`.
                    k if k.starts_with("oncontextmenu") || k.starts_with("onContextMenu") => {
                        let handler = extract_event_handler(&pair.value)?;
                        let full_key = k.replacen("onContextMenu", "oncontextmenu", 1);
                        events.insert(full_key, handler);
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
                                    // Keep the full key (including .prevent/.stop
                                    // modifiers) so codegen backends can emit them;
                                    // consumers use base-aware lookup
                                    // (crate::aura::aura_events_get_base).
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
                    // Plan 057 (ash-gui 表格/块头): 深层点链(.output.Table.atom_type、
                    // .block.command、裸基 output.columns)此前直接 UnsupportedExpr
                    // → 文本节点被静默丢弃(表格数据因此整块不显示)。压平成
                    // 点分路径绑定(self/. 根带前导点;裸 Ident 根不带)。
                    if let Some(path) = flatten_dot_path(expr) {
                        Ok(AuraNode::Text(AuraTextContent::Interpolated {
                            template: format!("${{{}}}", path),
                            bindings: vec![path],
                        }))
                    } else {
                        // Fall through to error for now
                        Err(ExtractError::UnsupportedExpr(format!(
                            "Cannot extract view tree from dot expression: {:?}",
                            expr
                        )))
                    }
                }
            }
        }

        _ => Err(ExtractError::UnsupportedExpr(format!(
            "Cannot extract view tree from: {:?}",
            expr
        ))),
    }
}

/// Plan 057 (ash-gui): 压平任意深度的 Dot 链为点分路径。
/// - `.a.b.c`(根为 self/.)→ Some(".a.b.c")(保留前导点)
/// - `var.a.b`(根为裸 Ident,如循环变量/view-fn 参数)→ Some("var.a.b")
/// - 含非 Ident/Dot 环节(方法调用等)→ None(维持原 Unsupported 行为)。
/// 解析侧(resolve_interpolation_with)按前导点区分根形式构造嵌套 Dot。
fn flatten_dot_path(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Ident(name) => {
            let n = name.as_str();
            if n == "." || n == "self" {
                Some(".".to_string())
            } else if !n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || c == '_') {
                Some(n.to_string())
            } else {
                None
            }
        }
        Expr::Dot(obj, field) => {
            let base = flatten_dot_path(obj)?;
            let f = field.as_str();
            if !f.is_empty() && f.chars().all(|c| c.is_alphanumeric() || c == '_') {
                Some(format!("{}.{}", base, f))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract event handler pattern from expression
fn extract_event_handler(expr: &Expr) -> ExtractResult<AuraEvent> {    match expr {
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
            // Plan 043: numeric field (tuple index, e.g. `field.0`) → `field[0]`
            // for valid TypeScript (`.0` is a property access, not a tuple index).
            if field.as_str().chars().all(|c| c.is_ascii_digit()) && !field.as_str().is_empty() {
                format!("{}[{}]", obj_str, field.as_str())
            } else {
                format!("{}.{}", obj_str, field.as_str())
            }
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
    // Plan 028 F9: `on stream sse(url[, "event"])` subscriptions — keep the
    // (url, event) wiring info alongside the handler keyed by its pattern.
    let stream_handlers: Vec<crate::aura::types::AuraStreamHandler> = decl.on.iter()
        .flat_map(|on| on.handlers.iter().filter(|h| h.stream.is_some()))
        .map(|h| {
            let sub = h.stream.as_ref().expect("filtered");
            let handler_key = if h.params.is_empty() {
                h.pattern.clone()
            } else {
                format!("{}({})", h.pattern, h.params.join(", "))
            };
            crate::aura::types::AuraStreamHandler {
                handler_key,
                kind: sub.kind.clone(),
                url: sub.url.clone(),
                event: sub.event.clone(),
            }
        })
        .collect();
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
        stream_endpoints: Vec::new(),
        stream_handlers,
        computed,
        // Plan 012 Batch G (gap 12): store-level watch block, same mapping
        // as the widget-level one.
        watchers: decl.watch.iter()
            .map(|w| crate::aura::types::AuraWatch {
                sources: w.sources.iter().map(|s| s.as_str().to_string()).collect(),
                immediate: w.immediate,
                deep: w.deep,
                payload: LogicPayload::AstStmts(w.body.stmts.clone()),
            })
            .collect(),
        module_fns: Vec::new(),
    })
}

/// Extract a plain module-level function into an `AuraModuleFn` for the vue
/// store composable. Skips `#[api]` handlers (those belong to the HTTP layer,
/// not the frontend) and test functions.
pub fn extract_module_fn(fn_decl: &crate::ast::Fn) -> Option<AuraModuleFn> {
    if fn_decl.api_attrs.is_some() || fn_decl.is_test {
        return None;
    }
    let params: Vec<String> = fn_decl.params.iter()
        .map(|p| p.name.as_str().to_string())
        .collect();
    let ret_ts = match &fn_decl.ret {
        crate::ast::Type::Void => "".to_string(),
        crate::ast::Type::StrSlice | crate::ast::Type::StrOwned
        | crate::ast::Type::StrFixed(_) | crate::ast::Type::CStrLit => "string".to_string(),
        crate::ast::Type::Int | crate::ast::Type::Uint | crate::ast::Type::USize
        | crate::ast::Type::I64 | crate::ast::Type::U64
        | crate::ast::Type::Float | crate::ast::Type::Double => "number".to_string(),
        crate::ast::Type::Bool => "boolean".to_string(),
        _ => "any".to_string(),
    };
    Some(AuraModuleFn {
        name: fn_decl.name.as_str().to_string(),
        params,
        ret_ts,
        body: fn_decl.body.stmts.clone(),
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
        style_css: decl.style.clone(),
        ext_imports: decl.ext_imports.clone(),
        watchers: decl.watch.iter()
            .map(|w| crate::aura::types::AuraWatch {
                sources: w.sources.iter().map(|s| s.as_str().to_string()).collect(),
                immediate: w.immediate,
                deep: w.deep,
                payload: LogicPayload::AstStmts(w.body.stmts.clone()),
            })
            .collect(),
        exposes: decl.expose.iter()
            .map(|n| n.as_str().to_string())
            .collect(),
    }
)
}

/// Plan 408: Map a `component fn`/`view fn` param type hint (raw string from
/// the parser) to an Auto `Type` for `defineProps` synthesis.
///
/// Only primitive hints get a typed mapping; everything else (custom types,
/// composite forms like `[]Note`, empty hints) falls back to `Type::Unknown`,
/// which `prop_to_ts_type` renders as `any` (per Plan 408 §3 risk table:
/// "参数一律 any" degradation). This keeps synthesis conservative and avoids
/// a full string→Type parser.
fn fragment_param_type(type_hint: &str) -> Type {
    match type_hint.trim() {
        "str" => Type::StrSlice,
        "int" | "i64" | "uint" | "usize" => Type::Int,
        "float" | "double" => Type::Float,
        "bool" => Type::Bool,
        _ => Type::Unknown,
    }
}

/// Plan 408: Extract an `AuraWidget` (for independent SFC synthesis) from a
/// `component fn` declaration. Mirrors `extract_widget_from_decl` but a
/// fragment has only params (→ props) + a view body — no model/msg/on/
/// computed/routes/lifecycle/bind/watch/expose/style/ext_imports.
pub fn extract_widget_from_fragment(
    frag: &crate::ast::ui::ViewFragmentDecl,
) -> ExtractResult<AuraWidget> {
    // Fragment params → AuraProp (defineProps source).
    let props: Vec<AuraProp> = frag.params.iter()
        .map(|(pname, type_hint)| AuraProp {
            name: pname.as_str().to_string(),
            type_info: fragment_param_type(type_hint),
            default: None,
        })
        .collect();

    // Plan 408 P3: component fn computed block → AuraComputed (mirrors
    // extract_widget_from_decl's computed handling). view fn has computed=None.
    let computed: Vec<AuraComputed> = if let Some(ref computed_block) = frag.computed {
        computed_block.properties.iter()
            .map(|p| AuraComputed {
                name: p.name.as_str().to_string(),
                expr: p.expr.clone(),
            })
            .collect()
    } else {
        Vec::new()
    };

    // Plan 408 P4: component fn model → state_vars (mirrors
    // extract_widget_from_decl's model handling). view fn has model=None.
    let mut state_vars = if let Some(ref model) = frag.model {
        extract_model_fields(model)?
    } else {
        Vec::new()
    };

    // Plan 408 P4: component fn messages → AuraMessage (mirrors widget). view fn 恒空。
    let messages: Vec<AuraMessage> = frag.messages.iter()
        .map(|m| extract_msg_decl(m))
        .collect();

    // Plan 408 P4: component fn on block → handlers + handler_params (mirrors
    // extract_widget_from_decl's on handling). Includes the .Tick interval
    // extraction and .Init/.Destroy lifecycle lift-out, so a component fn with
    // a timer or lifecycle hooks behaves like a widget. view fn has on=None.
    let (mut handlers, handler_params) = if let Some(ref on) = frag.on {
        extract_on_block(on)?
    } else {
        (HashMap::new(), HashMap::new())
    };
    let tick_interval = if handlers.keys().any(|k| k == ".Tick") {
        let interval_val = state_vars.iter()
            .find(|v| v.name == "interval")
            .and_then(|v| {
                if let Expr::Int(n) = &v.initial { Some(*n as u32) } else { None }
            })
            .or(Some(1000));
        state_vars.retain(|v| v.name != "interval");
        interval_val
    } else {
        None
    };
    let lifecycle_names = [
        crate::aura::types::lifecycle::INIT,
        crate::aura::types::lifecycle::DESTROY,
    ];
    let lifecycle_events: Vec<crate::aura::types::AuraLifecycle> = lifecycle_names.iter()
        .filter_map(|name| {
            handlers.remove(*name).map(|payload| {
                crate::aura::types::AuraLifecycle::new(&name[1..], payload)
            })
        })
        .collect();

    // Fragment body is a single root ViewNode (parse_view_fragment_decl_body_tail
    // parses exactly one). Extract it the same way a view block root is.
    let mut view_tree = extract_view_node(&frag.body)?;
    let span_map = assign_node_ids(&mut view_tree);

    Ok(AuraWidget {
        name: frag.name.as_str().to_string(),
        state_vars,
        computed,
        messages,
        view_tree,
        handlers,
        handler_params,
        props,
        routes: None,
        lifecycle: lifecycle_events,
        tick_interval,
        span_map,
        key_bindings: HashMap::new(),
        api_imports: Vec::new(),
        style_css: frag.style.clone(), // PLAN-026 缺陷②: component fn 的 style 块
        ext_imports: frag.ext_imports.clone(),
        watchers: frag.watch.iter()
            .map(|w| crate::aura::types::AuraWatch {
                sources: w.sources.iter().map(|s| s.as_str().to_string()).collect(),
                immediate: w.immediate,
                deep: w.deep,
                payload: LogicPayload::AstStmts(w.body.stmts.clone()),
            })
            .collect(),
        exposes: Vec::new(),
    })
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

/// Plan 408: Convert a `component fn` call site into an `AuraNode::Component`
/// reference (instead of inline-expanding it). `props` are the call-site view
/// props (already filtered to Expr values); `events` are the call-site events.
/// `name` is the component/tag name. The result references the independently
/// synthesized SFC via the existing `known_sub_widgets` mechanism.
fn fragment_to_component_node(
    name: &str,
    props: &[ViewProp],
    events: &[ViewEvent],
    children: Vec<AuraNode>,
    span: Option<(usize, usize)>,
) -> AuraNode {
    let aura_props: Vec<(String, Expr)> = props.iter()
        .filter_map(|p| match &p.value {
            ViewPropValue::Expr(expr) => Some((p.name.clone(), expr.clone())),
            ViewPropValue::StyleBinding(_) => None,
        })
        .collect();
    let aura_events: HashMap<String, AuraEvent> = events.iter()
        .map(|e| {
            (e.name.clone(), AuraEvent { handler: e.handler.clone(), params: e.params.clone() })
        })
        .collect();
    AuraNode::Component {
        name: name.to_string(),
        props: aura_props,
        events: aura_events,
        children,
        span,
        debug_id: None,
    }
}

/// Extract view node from parsed ViewNode
fn extract_view_node(node: &ViewNode) -> ExtractResult<AuraNode> {
    match node {
        ViewNode::Element { tag, props, events, children, span } => {
            // Plan 367 P2-3: check if this element is a view fragment call.
            // Fragment calls are PascalCase tags whose name matches a registered
            // view fragment. Props are passed as named props matching fragment params.
            // When matched, inline-expand the fragment body with parameter substitution.
            let is_pascal = tag.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            if is_pascal {
                let fragment = VIEW_FRAGMENTS.with(|cell| cell.borrow().get(tag.as_str()).cloned());
                if let Some(frag) = fragment {
                    // Plan 408: `component fn` → independent SFC, emit a
                    // component reference instead of inline-expanding.
                    if frag.is_component {
                        // Plan 408 P11: extract slot children (the call site's
                        // body) so the parent can inject <template #x> content.
                        let aura_children: Vec<AuraNode> = children.iter()
                            .map(|c| extract_view_node(c))
                            .collect::<ExtractResult<_>>()?;
                        return Ok(fragment_to_component_node(tag, props, events, aura_children, *span));
                    }
                    // `view fn` → inline-expand (Plan 367 P2-3, unchanged).
                    // Build substitution map: param_name → call expression
                    let mut subs: HashMap<String, Expr> = HashMap::new();
                    for (pname, _type_hint) in &frag.params {
                        if let Some(prop) = props.iter().find(|p| p.name == *pname) {
                            if let ViewPropValue::Expr(expr) = &prop.value {
                                subs.insert(pname.as_str().to_string(), expr.clone());
                            }
                        }
                    }
                    let expanded = expand_fragment_node(&frag.body, &subs)?;
                    return extract_view_node(&expanded);
                }
            }
            let mut aura_props: HashMap<String, AuraPropValue> = HashMap::new();
            for p in props.iter() {
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
                // Plan 012 P0#13: a second `class:` prop on the same element
                // used to collapse into the HashMap last-wins, silently
                // dropping the first (e.g. `class: "static"` + a dynamic
                // `class:` expr). Merge them into an array binding instead —
                // Vue unions `:class="['static', expr]"` entries, matching
                // the static+dynamic `class`/`:class` coexistence semantics.
                if p.name.as_str() == "class" {
                    if let Some(existing) = aura_props.remove("class") {
                        let merged = match (existing, value) {
                            (AuraPropValue::Expr(Expr::Array(mut elems)), AuraPropValue::Expr(e)) => {
                                elems.push(e);
                                AuraPropValue::Expr(Expr::Array(elems))
                            }
                            (AuraPropValue::Expr(prev), AuraPropValue::Expr(e)) => {
                                AuraPropValue::Expr(Expr::Array(vec![prev, e]))
                            }
                            // Non-expr class values (StyleBinding) can't be
                            // merged into an array — keep the last one.
                            (_, v) => v,
                        };
                        aura_props.insert("class".to_string(), merged);
                        continue;
                    }
                }
                aura_props.insert(p.name.clone(), value);
            }

            let aura_events: HashMap<String, AuraEvent> = events.iter()
                .map(|e| {
                    let event = AuraEvent {
                        handler: e.handler.clone(),
                        params: e.params.clone(),
                    };
                    // Keep the full key (including modifiers); consumers use
                    // base-aware lookup (crate::aura::aura_events_get_base).
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
            // Plan 367 P2-3: check if this is a view fragment call.
            // If so, inline-expand the fragment body with parameter substitution.
            let fragment = VIEW_FRAGMENTS.with(|cell| cell.borrow().get(name.as_str()).cloned());
            if let Some(frag) = fragment {
                // Plan 408: `component fn` → independent SFC reference.
                if frag.is_component {
                    return Ok(fragment_to_component_node(name, props, events, Vec::new(), *span));
                }
                // `view fn` → inline-expand (Plan 367 P2-3, unchanged).
                // Build parameter substitution map: arg0 → call_expr, arg1 → ...
                let mut substitutions: HashMap<String, Expr> = HashMap::new();
                for (i, param) in frag.params.iter().enumerate() {
                    if let Some(call_prop) = props.get(i) {
                        if let ViewPropValue::Expr(expr) = &call_prop.value {
                            substitutions.insert(param.0.as_str().to_string(), expr.clone());
                        }
                    }
                }
                // Clone and expand the fragment body with substitutions
                let expanded = expand_fragment_node(&frag.body, &substitutions)?;
                return extract_view_node(&expanded);
            }

            // Normal component (not a fragment) — extract as-is
            let aura_props: Vec<(String, Expr)> = props.iter()
                .filter_map(|p| {
                    match &p.value {
                        ViewPropValue::Expr(expr) => {
                            Some((p.name.clone(), expr.clone()))
                        }
                        ViewPropValue::StyleBinding(_) => {
                            // Class bindings not supported for component props
                            None
                        }
                    }
                })
                .collect();

            let aura_events: HashMap<String, AuraEvent> = events.iter()
                .map(|e| {
                    let event = AuraEvent {
                        handler: e.handler.clone(),
                        params: e.params.clone(),
                    };
                    // Keep the full key (including modifiers); consumers use
                    // base-aware lookup (crate::aura::aura_events_get_base).
                    (e.name.clone(), event)
                })
                .collect();

            Ok(AuraNode::Component {
                name: name.clone(),
                props: aura_props,
                events: aura_events,
                children: Vec::new(),
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

/// Plan 367 P2-3: Expand a view fragment body with parameter substitution.
///
/// Walks the ViewNode tree and replaces references to fragment parameters
/// with the actual expressions from the call site. This is a deep clone +
/// transform — the original fragment is not modified.
///
/// Substitution targets:
/// - In `text .param_name` → `text <call_expr>`
/// - In `onclick: .Handler` → preserved (parent widget's handler)
/// - In conditions `if .param_name == ...` → `if <call_expr> == ...`
/// - In style strings → preserved (styles don't reference params)
fn expand_fragment_node(
    node: &ViewNode,
    subs: &HashMap<String, Expr>,
) -> ExtractResult<ViewNode> {
    match node {
        ViewNode::Element { tag, props, events, children, span } => {
            // Transform props: substitute param references in expressions
            let new_props: Vec<crate::ast::ui::ViewProp> = props.iter()
                .map(|p| {
                    let new_value = match &p.value {
                        ViewPropValue::Expr(expr) => {
                            ViewPropValue::Expr(substitute_expr(expr, subs))
                        }
                        other => other.clone(),
                    };
                    Ok(crate::ast::ui::ViewProp {
                        name: p.name.clone(),
                        value: new_value,
                    })
                })
                .collect::<ExtractResult<_>>()?;
            // Recursively expand children
            let new_children: Vec<ViewNode> = children.iter()
                .map(|c| expand_fragment_node(c, subs))
                .collect::<ExtractResult<_>>()?;
            Ok(ViewNode::Element {
                tag: tag.clone(),
                props: new_props,
                events: events.clone(),
                children: new_children,
                span: *span,
            })
        }
        ViewNode::Conditional { condition, then_body, else_body, span } => {
            let new_condition = substitute_condition(condition, subs);
            let new_then: Vec<ViewNode> = then_body.iter()
                .map(|c| expand_fragment_node(c, subs))
                .collect::<ExtractResult<_>>()?;
            let new_else: Option<Vec<ViewNode>> = else_body.as_ref()
                .map(|nodes| nodes.iter()
                    .map(|c| expand_fragment_node(c, subs))
                    .collect::<ExtractResult<_>>()
                )
                .transpose()?;
            Ok(ViewNode::Conditional {
                condition: new_condition,
                then_body: new_then,
                else_body: new_else,
                span: *span,
            })
        }
        ViewNode::ForLoop { var, index, iterable, body, span } => {
            let new_body: Vec<ViewNode> = body.iter()
                .map(|c| expand_fragment_node(c, subs))
                .collect::<ExtractResult<_>>()?;
            // Plan 043 M5: the iterable is a STRING (parse_view_for_loop
            // builds it manually), so it can't go through substitute_expr.
            // Substitute param refs (e.g. `for col in output.columns` →
            // `output.Table.columns`) so inlined view fns narrow to the
            // variant sub-type. Previously this stayed untouched and relied
            // on the widget coincidentally exposing a same-named prop.
            Ok(ViewNode::ForLoop {
                var: var.clone(),
                index: index.clone(),
                iterable: substitute_condition(iterable, subs),
                body: new_body,
                span: *span,
            })
        }
        ViewNode::Component { name, props, events, span } => {
            // Nested component call — expand its props too
            let new_props: Vec<crate::ast::ui::ViewProp> = props.iter()
                .map(|p| {
                    let new_value = match &p.value {
                        ViewPropValue::Expr(expr) => ViewPropValue::Expr(substitute_expr(expr, subs)),
                        other => other.clone(),
                    };
                    Ok(crate::ast::ui::ViewProp { name: p.name.clone(), value: new_value })
                })
                .collect::<ExtractResult<_>>()?;
            Ok(ViewNode::Component {
                name: name.clone(),
                props: new_props,
                events: events.clone(),
                span: *span,
            })
        }
        // Pass through other node types unchanged
        _ => Ok(node.clone()),
    }
}

/// Substitute fragment parameter references in an expression.
/// Replaces `.param_name` (Expr::Dot(Ident("self"), "param_name")) with the
/// actual expression from the call site.
fn substitute_expr(expr: &Expr, subs: &HashMap<String, Expr>) -> Expr {
    use crate::ast::Expr;
    match expr {
        // Bare ident parameter reference. View fn bodies use params as bare
        // idents (e.g. `if active`, or the `note` in `note.title`), not just as
        // `.param`/`self.param`. The parser binds view fn params into scope
        // precisely so these bare ident uses parse (see parser.rs view fn).
        // So in addition to the Dot forms below, substitute any bare ident that
        // names a formal parameter.
        Expr::Ident(name) => {
            if let Some(replacement) = subs.get(name.as_str()) {
                return replacement.clone();
            }
            expr.clone()
        }
        // .param_name → substitution
        Expr::Dot(obj, field) if matches!(obj.as_ref(), Expr::Ident(name) if name.as_str() == "." || name.as_str() == "self") => {
            if let Some(replacement) = subs.get(field.as_str()) {
                return replacement.clone();
            }
            expr.clone()
        }
        // Recurse into compound expressions
        Expr::Bina(lhs, op, rhs) => {
            Expr::Bina(
                Box::new(substitute_expr(lhs, subs)),
                op.clone(),
                Box::new(substitute_expr(rhs, subs)),
            )
        }
        Expr::Dot(obj, field) => {
            Expr::Dot(Box::new(substitute_expr(obj, subs)), field.clone())
        }
        Expr::Call(call) => {
            let mut new_call = call.clone();
            // Plan 053 后续: also substitute the callee (`call.name`), not just
            // args. Method chains on a view-fn param live in the callee — e.g.
            // `output.columns.len()` is Call{ name: Dot(Dot(output,columns),len) }.
            // Without this, the callee's `output` was left unsubstituted (only
            // args were), so the generated :style/:class kept `output.columns`
            // instead of `output.Table.columns` → runtime undefined.
            new_call.name = Box::new(substitute_expr(&call.name, subs));
            new_call.args.args = call.args.args.iter()
                .map(|a| match a {
                    crate::ast::Arg::Pos(e) => crate::ast::Arg::Pos(substitute_expr(e, subs)),
                    crate::ast::Arg::Pair(n, e) => crate::ast::Arg::Pair(n.clone(), substitute_expr(e, subs)),
                    other => other.clone(),
                })
                .collect();
            Expr::Call(new_call)
        }
        Expr::Closure(c) => {
            let mut new_c = c.clone();
            new_c.body = Box::new(substitute_expr(&c.body, subs));
            Expr::Closure(new_c)
        }
        // `if` expressions used as prop values, e.g. `style: if active {..} else {..}`.
        // Substitute parameter refs in each branch's condition (where a bare
        // param like `active` lives). Branch/else bodies are string-literal
        // style payloads in this context and don't reference params, so we
        // leave them untouched (avoiding a stmt-level substitution recursion).
        Expr::If(if_expr) => {
            let mut new_if = if_expr.clone();
            for branch in &mut new_if.branches {
                branch.cond = substitute_expr(&branch.cond, subs);
            }
            Expr::If(new_if)
        }
        _ => expr.clone(),
    }
}

/// Substitute fragment parameter references in a condition string.
/// Condition strings like "param_name == value" need param_name replaced.
fn substitute_condition(condition: &str, subs: &HashMap<String, Expr>) -> String {
    let mut result = condition.to_string();
    for (param_name, expr) in subs {
        // Replace bare references to param_name in the condition string.
        // Conditions are strings, not AST, so we do string replacement.
        // Plan 374: Support complex expressions by converting them to .at-style
        // dotted references (e.g., "i == .store.active_id" for the `active` param).
        let replacement = match expr {
            Expr::Ident(name) => format!(".{}", name.as_str()),
            _ => {
                // For complex expressions, convert to a string representation.
                // Use the expr's Display or a manual conversion.
                expr_to_condition_str(expr)
            }
        };
        // Replace ".param_name" with the replacement
        result = result.replace(
            &format!(".{}", param_name),
            &replacement,
        );
        // Also replace bare "param_name" (not preceded by a dot) for conditions
        // like "if active { ... }" where active is used without a dot prefix.
        let bare_pattern = format!("{}", param_name);
        // Only replace if it's a word boundary (not part of a larger identifier)
        let mut new_result = String::new();
        let bytes = result.as_bytes();
        let bare_bytes = bare_pattern.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if i + bare_bytes.len() <= bytes.len() && &bytes[i..i + bare_bytes.len()] == bare_bytes {
                // Check word boundary: preceded by non-ident char
                let prev_is_ident = i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_' || bytes[i - 1] == b'.');
                let next_is_ident = i + bare_bytes.len() < bytes.len() && (bytes[i + bare_bytes.len()].is_ascii_alphanumeric() || bytes[i + bare_bytes.len()] == b'_');
                if !prev_is_ident && !next_is_ident {
                    new_result.push_str(&replacement);
                    i += bare_bytes.len();
                    continue;
                }
            }
            new_result.push(bytes[i] as char);
            i += 1;
        }
        result = new_result;
    }
    result
}

/// Convert an Expr to a condition string suitable for .at condition syntax.
fn expr_to_condition_str(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => format!(".{}", name.as_str()),
        Expr::Int(n) => n.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Str(s) => format!("\"{}\"", s),
        Expr::Dot(obj, field) => {
            // Plan 043 M5: `.output` / `.output.Table` parse as Dot chains
            // rooted at the `self` placeholder (Ident(".") / Ident("self")).
            // The self-dot base must NOT leak as "self.output" — render the
            // whole self-path dot-prefixed (.output / .output.Table) so
            // substituted iterables/conditions stay valid .at field paths.
            if matches!(obj.as_ref(), Expr::Ident(n) if n.as_str() == "." || n.as_str() == "self") {
                return format!(".{}", field.as_str());
            }
            let obj_str = expr_to_condition_str(obj);
            // If obj is ".self" or ".store", keep the dotted path
            if obj_str.starts_with('.') {
                format!("{}.{}", &obj_str[1..], field.as_str())
            } else {
                format!("{}.{}", obj_str, field.as_str())
            }
        }
        Expr::Bina(left, op, right) => {
            let left_str = expr_to_condition_str(left);
            let right_str = expr_to_condition_str(right);
            let op_str = match op {
                auto_val::Op::Eq => "==",
                auto_val::Op::Neq => "!=",
                auto_val::Op::Lt => "<",
                auto_val::Op::Le => "<=",
                auto_val::Op::Gt => ">",
                auto_val::Op::Ge => ">=",
                auto_val::Op::And => "&&",
                auto_val::Op::Or => "||",
                _ => "?",
            };
            format!("{} {} {}", left_str, op_str, right_str)
        }
        _ => "true".to_string(), // Fallback
    }
}
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
        AuraNode::Component { name, props: _, events: _, children, span, debug_id } => {
            *debug_id = Some(id);
            span_map.insert(id, SpanInfo {
                span: *span,
                aura_tag: name.clone(),
                user_id: None,
            });
            // Plan 408 P11: recurse into slot children so they get debug ids too.
            for child in children.iter_mut() {
                assign_node_ids_recursive(child, next_id, span_map);
            }
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
        // Plan 374: Embed parameter names in the pattern key so RustGenerator
        // and other backends can extract them without a separate handler_params lookup.
        // e.g., ".SelectTag(t)" instead of just ".SelectTag".
        let pattern = if handler.params.is_empty() {
            handler.pattern.clone()
        } else {
            format!("{}({})", handler.pattern, handler.params.join(", "))
        };
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

    /// Parse a widget source (real parser pipeline) and extract its AuraWidget.
    fn extract_widget_from_src(src: &str) -> AuraWidget {
        let session = crate::session::CompilerSession::ui();
        let mut parser = crate::Parser::from(src).with_session(session);
        let ast = parser.parse().expect("widget source must parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                crate::ast::Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        extract_widget_from_decl(decl).expect("extract widget")
    }

    /// Event keys keep their modifiers in the extracted events map
    /// (`onclick.self`, `onkeydown.enter.prevent`, …) — codegen backends need
    /// them, and same-base events must not overwrite each other. Consumers
    /// that dispatch by base name go through `aura_events_get_base`.
    #[test]
    fn test_extract_preserves_event_key_modifiers() {
        let widget = extract_widget_from_src(r#"
widget Nav {
    msg Msg { X, A, B, C }
    model { var n int = 0 }
    view {
        col {
            onclick.self: .X,
            onkeydown.enter.prevent: .A,
            onkeydown.down.prevent: .B,
            onkeydown.up.prevent: .C,
            oncontextmenu.prevent: .X
        }
    }
    on {
        .X -> { .n = 1 }
        .A -> { .n = 2 }
        .B -> { .n = 3 }
        .C -> { .n = 4 }
    }
}
"#);
        let events = match &widget.view_tree {
            AuraNode::Element { events, .. } => events,
            other => panic!("expected element view tree, got {:?}", other),
        };
        for key in [
            "onclick.self",
            "onkeydown.enter.prevent",
            "onkeydown.down.prevent",
            "onkeydown.up.prevent",
            "oncontextmenu.prevent",
        ] {
            assert!(
                events.contains_key(key),
                "full event key `{}` preserved; got keys: {:?}",
                key,
                events.keys().collect::<Vec<_>>()
            );
        }
        // Same-base events coexist (no normalization overwrite).
        assert_eq!(events["onkeydown.enter.prevent"].handler, ".A");
        assert_eq!(events["onkeydown.down.prevent"].handler, ".B");
        assert_eq!(events["onkeydown.up.prevent"].handler, ".C");
        // Base-aware lookup still reaches the modifier-carrying keys.
        assert_eq!(
            crate::aura::aura_events_get_base(events, "oncontextmenu")
                .unwrap()
                .handler,
            ".X"
        );
        assert_eq!(
            crate::aura::aura_events_get_base(events, "onclick")
                .unwrap()
                .handler,
            ".X"
        );
    }
}
