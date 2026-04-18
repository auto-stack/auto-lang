//! AURA Atom Format - Serialization to Atom format
//!
//! This module provides serialization of AURA structures to the Atom format,
//! which is a structured text format used for debugging, cross-language
//! toolchain integration, and AI-assisted generation.

use super::types::*;

// ============================================================================
// Atom Serializer
// ============================================================================

/// Serialize an AuraWidget to Atom format
pub fn to_atom(widget: &AuraWidget) -> String {
    let mut output = String::new();
    serialize_widget(widget, &mut output, 0);
    output
}

/// Serialize an AuraModule to Atom format
pub fn module_to_atom(module: &AuraModule) -> String {
    let mut output = String::new();

    // Module header
    output.push_str(&format!("Module {{\n"));
    output.push_str(&format!("    name: \"{}\",\n", module.name));

    // Widgets
    if !module.widgets.is_empty() {
        output.push_str("    widgets: [\n");
        for widget in &module.widgets {
            serialize_widget(widget, &mut output, 2);
            output.push_str(",\n");
        }
        output.push_str("    ],\n");
    }

    // App
    if let Some(app) = &module.app {
        output.push_str(&format!("    app: {{\n"));
        output.push_str(&format!("        name: \"{}\",\n", app.name));
        output.push_str(&format!("        root: \"{}\",\n", app.root));
        output.push_str(&format!("        window: {{\n"));
        output.push_str(&format!("            title: \"{}\",\n", app.window.title));
        output.push_str(&format!("            width: {},\n", app.window.width));
        output.push_str(&format!("            height: {}\n", app.window.height));
        output.push_str("        }\n");
        output.push_str("    }\n");
    }

    output.push_str("}\n");
    output
}

/// Serialize a widget
fn serialize_widget(widget: &AuraWidget, output: &mut String, indent: usize) {
    let ind = "    ".repeat(indent);

    output.push_str(&format!("{}Widget {{\n", ind));
    output.push_str(&format!("{}    name: \"{}\",\n", ind, widget.name));

    // State vars
    if !widget.state_vars.is_empty() {
        output.push_str(&format!("{}    states: [\n", ind));
        for state in &widget.state_vars {
            output.push_str(&format!("{}        {{ name: \"{}\", type: \"{}\", default: ",
                ind, state.name, type_to_string(&state.type_info)));
            serialize_expr(&state.initial, output);
            output.push_str(" },\n");
        }
        output.push_str(&format!("{}    ],\n", ind));
    }

    // Messages
    if !widget.messages.is_empty() {
        output.push_str(&format!("{}    messages: [\n", ind));
        for msg in &widget.messages {
            output.push_str(&format!("{}        {{ name: \"{}\", variants: [",
                ind, msg.name));
            for (i, variant) in msg.variants.iter().enumerate() {
                if i > 0 {
                    output.push_str(", ");
                }
                output.push_str(&format!("\"{}\"", variant.name));
            }
            output.push_str("] },\n");
        }
        output.push_str(&format!("{}    ],\n", ind));
    }

    // Props
    if !widget.props.is_empty() {
        output.push_str(&format!("{}    props: [\n", ind));
        for prop in &widget.props {
            output.push_str(&format!("{}        {{ name: \"{}\", type: \"{}\"",
                ind, prop.name, type_to_string(&prop.type_info)));
            if let Some(default) = &prop.default {
                output.push_str(", default: ");
                serialize_expr(default, output);
            }
            output.push_str(" },\n");
        }
        output.push_str(&format!("{}    ],\n", ind));
    }

    // View tree
    output.push_str(&format!("{}    view: ", ind));
    serialize_node(&widget.view_tree, output, indent + 1);
    output.push_str(",\n");

    // Handlers
    if !widget.handlers.is_empty() {
        output.push_str(&format!("{}    handlers: {{\n", ind));
        for (pattern, payload) in &widget.handlers {
            output.push_str(&format!("{}        \"{}\": ", ind, pattern));
            serialize_payload(payload, output, indent + 2);
            output.push_str(",\n");
        }
        output.push_str(&format!("{}    }}\n", ind));
    }

    output.push_str(&format!("{}}}", ind));
}

/// Serialize a node
fn serialize_node(node: &AuraNode, output: &mut String, indent: usize) {
    let ind = "    ".repeat(indent);

    match node {
        AuraNode::Element { tag, props, events, children } => {
            output.push_str(&format!("Node {{\n"));
            output.push_str(&format!("{}    tag: \"{}\",\n", ind, tag));

            // Props
            if !props.is_empty() {
                output.push_str(&format!("{}    props: {{\n", ind));
                for (key, value) in props {
                    output.push_str(&format!("{}        \"{}\": ", ind, key));
                    match value {
                        AuraPropValue::Expr(expr) => {
                            serialize_expr(expr, output);
                        }
                        AuraPropValue::StyleBinding(bindings) => {
                            output.push_str("StyleBinding({");
                            for (i, b) in bindings.iter().enumerate() {
                                if i > 0 { output.push_str(", "); }
                                output.push_str(&format!("\"{}\": ", b.style_name));
                                serialize_expr(&b.condition, output);
                            }
                            output.push_str("})");
                        }
                    }
                    output.push_str(",\n");
                }
                output.push_str(&format!("{}    }},\n", ind));
            }

            // Events
            if !events.is_empty() {
                output.push_str(&format!("{}    events: {{\n", ind));
                for (event, aura_event) in events {
                    if aura_event.params.is_empty() {
                        output.push_str(&format!("{}        \"{}\": Dispatch(\"{}\"),\n", ind, event, aura_event.handler));
                    } else {
                        output.push_str(&format!("{}        \"{}\": Dispatch(\"{}\", params: [{}]),\n",
                            ind, event, aura_event.handler, aura_event.params.join(", ")));
                    }
                }
                output.push_str(&format!("{}    }},\n", ind));
            }

            // Children
            if !children.is_empty() {
                output.push_str(&format!("{}    children: [\n", ind));
                for child in children {
                    output.push_str(&format!("{}        ", ind));
                    serialize_node(child, output, indent + 2);
                    output.push_str(",\n");
                }
                output.push_str(&format!("{}    ]\n", ind));
            }

            output.push_str(&format!("{}}}", ind));
        }

        AuraNode::Text(content) => {
            match content {
                AuraTextContent::Literal(s) => {
                    output.push_str(&format!("Text(\"{}\")", escape_string(s)));
                }
                AuraTextContent::Interpolated { template, bindings } => {
                    output.push_str(&format!("Interpolated(\"{}\", bindings: [",
                        escape_string(template)));
                    for (i, b) in bindings.iter().enumerate() {
                        if i > 0 {
                            output.push_str(", ");
                        }
                        output.push_str(&format!("\"{}\"", b));
                    }
                    output.push_str("])");
                }
            }
        }

        AuraNode::ForLoop { var, index, iterable, body } => {
            output.push_str(&format!("ForLoop {{\n"));
            output.push_str(&format!("{}    var: \"{}\",\n", ind, var));
            if let Some(idx) = index {
                output.push_str(&format!("{}    index: Some(\"{}\"),\n", ind, idx));
            } else {
                output.push_str(&format!("{}    index: None,\n", ind));
            }
            output.push_str(&format!("{}    iterable: \"{}\",\n", ind, iterable));
            output.push_str(&format!("{}    body: [\n", ind));
            for child in body {
                output.push_str(&format!("{}        ", ind));
                serialize_node(child, output, indent + 2);
                output.push_str(",\n");
            }
            output.push_str(&format!("{}    ]\n", ind));
            output.push_str(&format!("{}}}", ind));
        }

        AuraNode::Conditional { condition, then_body, else_body } => {
            output.push_str(&format!("Conditional {{\n"));
            output.push_str(&format!("{}    condition: \"{}\",\n", ind, condition));
            output.push_str(&format!("{}    then_body: [\n", ind));
            for child in then_body {
                output.push_str(&format!("{}        ", ind));
                serialize_node(child, output, indent + 2);
                output.push_str(",\n");
            }
            output.push_str(&format!("{}    ],\n", ind));
            if let Some(else_nodes) = else_body {
                output.push_str(&format!("{}    else_body: Some([\n", ind));
                for child in else_nodes {
                    output.push_str(&format!("{}        ", ind));
                    serialize_node(child, output, indent + 2);
                    output.push_str(",\n");
                }
                output.push_str(&format!("{}    ]),\n", ind));
            } else {
                output.push_str(&format!("{}    else_body: None,\n", ind));
            }
            output.push_str(&format!("{}}}", ind));
        }

        AuraNode::Component { name, props, events } => {
            output.push_str(&format!("Component {{\n"));
            output.push_str(&format!("{}    name: \"{}\",\n", ind, name));
            if !props.is_empty() {
                output.push_str(&format!("{}    props: {{\n", ind));
                for (key, value) in props {
                    output.push_str(&format!("{}        \"{}\": ", ind, key));
                    serialize_expr(value, output);
                    output.push_str(",\n");
                }
                output.push_str(&format!("{}    }},\n", ind));
            }
            if !events.is_empty() {
                output.push_str(&format!("{}    events: {{\n", ind));
                for (event, aura_event) in events {
                    if aura_event.params.is_empty() {
                        output.push_str(&format!("{}        \"{}\": \"{}\",\n", ind, event, aura_event.handler));
                    } else {
                        output.push_str(&format!("{}        \"{}\": \"{}({})\",\n",
                            ind, event, aura_event.handler, aura_event.params.join(", ")));
                    }
                }
                output.push_str(&format!("{}    }},\n", ind));
            }
            output.push_str(&format!("{}}}", ind));
        }

        // Plan 105: Router outlet and link
        AuraNode::Outlet => {
            output.push_str(&format!("Outlet"));
        }

        AuraNode::Link { to, text, href, children } => {
            output.push_str(&format!("Link {{\n"));
            output.push_str(&format!("{}    to: \"{}\",\n", ind, to));
            if !text.is_empty() {
                output.push_str(&format!("{}    text: \"{}\",\n", ind, text));
            }
            if !href.is_empty() {
                output.push_str(&format!("{}    href: \"{}\",\n", ind, href));
            }
            if !children.is_empty() {
                output.push_str(&format!("{}    children: [\n", ind));
                for child in children {
                    output.push_str(&format!("{}        ", ind));
                    serialize_node(child, output, indent + 2);
                    output.push_str(",\n");
                }
                output.push_str(&format!("{}    ]\n", ind));
            }
            output.push_str(&format!("{}}}", ind));
        }
    }
}

/// Serialize an expression
fn serialize_expr(expr: &AuraExpr, output: &mut String) {
    match expr {
        AuraExpr::Literal(s) => output.push_str(&format!("\"{}\"", escape_string(s))),
        AuraExpr::Int(n) => output.push_str(&n.to_string()),
        AuraExpr::Float(n) => output.push_str(&n.to_string()),
        AuraExpr::Bool(b) => output.push_str(&b.to_string()),
        AuraExpr::StateRef(name) => output.push_str(&format!("State(\"{}\")", name)),
        AuraExpr::MsgVariant { msg_type, variant } => {
            output.push_str(&format!("Msg(\"{}::{}\")", msg_type, variant));
        }
        AuraExpr::Binary { left, op, right } => {
            output.push_str("Binary(");
            serialize_expr(left, output);
            output.push_str(&format!(", {:?}, ", op));
            serialize_expr(right, output);
            output.push_str(")");
        }
        AuraExpr::Unary { op, operand } => {
            output.push_str(&format!("Unary({:?}, ", op));
            serialize_expr(operand, output);
            output.push_str(")");
        }
        AuraExpr::MethodCall { object, method, args } => {
            output.push_str("MethodCall(");
            serialize_expr(object, output);
            output.push_str(&format!(", \"{}\", [", method));
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { output.push_str(", "); }
                serialize_expr(arg, output);
            }
            output.push_str("])");
        }
        AuraExpr::Array(elems) => {
            output.push_str("Array([");
            for (i, elem) in elems.iter().enumerate() {
                if i > 0 { output.push_str(", "); }
                serialize_expr(elem, output);
            }
            output.push_str("])");
        }
        AuraExpr::Object(fields) => {
            output.push_str("Object({");
            for (i, (key, value)) in fields.iter().enumerate() {
                if i > 0 { output.push_str(", "); }
                output.push_str(&format!("\"{}\": ", key));
                serialize_expr(value, output);
            }
            output.push_str("})");
        }
        AuraExpr::Lambda { params, body } => {
            output.push_str(&format!("Lambda({:?}, ", params));
            serialize_expr(body, output);
            output.push_str(")");
        }
        AuraExpr::FieldAccess { object, field } => {
            output.push_str("FieldAccess(");
            serialize_expr(object, output);
            output.push_str(&format!(", \"{}\")", field));
        }
        AuraExpr::NavCall { path, params } => {
            output.push_str(&format!("NavCall(\"{}\", {{", path));
            for (i, (key, value)) in params.iter().enumerate() {
                if i > 0 { output.push_str(", "); }
                output.push_str(&format!("\"{}\": ", key));
                serialize_expr(value, output);
            }
            output.push_str("})");
        }
        AuraExpr::Constructor { type_name, args } => {
            output.push_str(&format!("Constructor(\"{}\", [", type_name));
            for (i, arg) in args.iter().enumerate() {
                if i > 0 { output.push_str(", "); }
                serialize_expr(arg, output);
            }
            output.push_str("])");
        }
    }
}

/// Serialize a logic payload
fn serialize_payload(payload: &LogicPayload, output: &mut String, _indent: usize) {
    match payload {
        LogicPayload::AstBlock(stmts) => {
            output.push_str(&format!("AstBlock([{} statements])", stmts.len()));
        }
        LogicPayload::AstStmts(stmts) => {
            output.push_str(&format!("AstStmts([{} statements])", stmts.len()));
        }
        LogicPayload::Bytecode(bytes) => {
            output.push_str(&format!("Bytecode({} bytes)", bytes.len()));
        }
    }
}

/// Convert type to string
fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Byte => "byte".to_string(),
        Type::Int => "int".to_string(),
        Type::Uint => "uint".to_string(),
        Type::USize => "usize".to_string(),
        Type::I64 => "i64".to_string(),
        Type::U64 => "u64".to_string(),
        Type::Float => "float".to_string(),
        Type::Double => "double".to_string(),
        Type::Bool => "bool".to_string(),
        Type::Char => "char".to_string(),
        Type::Str(_) | Type::String => "str".to_string(),
        Type::CStr => "cstr".to_string(),
        Type::StrSlice => "str_slice".to_string(),
        Type::Void => "void".to_string(),
        Type::Unknown => "unknown".to_string(),
        _ => format!("{:?}", ty),
    }
}

/// Escape string for Atom format
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_to_atom_simple_widget() {
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
        };

        let atom = to_atom(&widget);

        assert!(atom.contains("Widget"));
        assert!(atom.contains("Counter"));
        assert!(atom.contains("count"));
    }

    #[test]
    fn test_to_atom_with_children() {
        let widget = AuraWidget {
            name: "App".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree: AuraNode::element("col")
                .with_child(AuraNode::text("Hello"))
                .with_child(AuraNode::element("button")),
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
        };

        let atom = to_atom(&widget);

        assert!(atom.contains("children"));
        assert!(atom.contains("Hello"));
        assert!(atom.contains("button"));
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("hello"), "hello");
        assert_eq!(escape_string("hello \"world\""), "hello \\\"world\\\"");
        assert_eq!(escape_string("line1\nline2"), "line1\\nline2");
    }

    #[test]
    fn test_type_to_string() {
        assert_eq!(type_to_string(&Type::Int), "int");
        assert_eq!(type_to_string(&Type::Bool), "bool");
        assert_eq!(type_to_string(&Type::Str(0)), "str");
    }

    #[test]
    fn test_module_to_atom() {
        let module = AuraModule {
            name: "MyApp".to_string(),
            widgets: vec![AuraWidget {
                name: "Main".to_string(),
                state_vars: vec![],
                computed: vec![],
                messages: vec![],
                view_tree: AuraNode::text("Hello"),
                handlers: HashMap::new(),
                props: vec![],
                routes: None,
                lifecycle: vec![],
            tick_interval: None,
            }],
            messages: vec![],
            app: Some(AuraApp {
                name: "MyApp".to_string(),
                root: "Main".to_string(),
                window: AuraWindow {
                    title: "My App".to_string(),
                    width: 800,
                    height: 600,
                },
            }),
        };

        let atom = module_to_atom(&module);

        assert!(atom.contains("Module"));
        assert!(atom.contains("MyApp"));
        assert!(atom.contains("app"));
        assert!(atom.contains("800"));
    }
}
