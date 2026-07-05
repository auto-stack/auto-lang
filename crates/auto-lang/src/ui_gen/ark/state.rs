//! ArkTS State Management
//!
//! Generates @State declarations and dispatch functions.

use crate::aura::{AuraWidget, LogicPayload};
use crate::ast::Type;
use std::collections::HashMap;

/// Extracted interface definition for array of objects
#[derive(Debug, Clone)]
pub struct InterfaceDef {
    /// Interface name (e.g., "Item", "EnablementItem")
    pub name: String,
    /// Fields with their types (e.g., {"title": "string", "count": "number"})
    pub fields: HashMap<String, String>,
}

/// Analyze an array expression and extract interface definition if it contains objects
#[allow(dead_code)]
pub fn extract_interface_from_array(name: &str, expr: &crate::ast::Expr) -> Option<InterfaceDef> {
    if let crate::ast::Expr::Array(elems) = expr {
        // Find first object element to extract structure
        for elem in elems {
            if let crate::ast::Expr::Object(pairs) = elem {
                let interface_name = to_pascal_case(name);
                let mut interface_fields = HashMap::new();

                // Add id field for ForEach key function
                interface_fields.insert("id".to_string(), "string".to_string());

                for p in pairs {
                    let field_type = infer_arkts_type_from_expr(&p.value);
                    interface_fields.insert(p.key.to_astr().to_string(), field_type);
                }

                return Some(InterfaceDef {
                    name: interface_name,
                    fields: interface_fields,
                });
            }
        }
    }
    None
}

/// Extract all interfaces from an array expression, including nested arrays
/// Returns a vector of (interface_name, InterfaceDef) pairs
pub fn extract_all_interfaces_from_array(parent_name: &str, name: &str, expr: &crate::ast::Expr) -> Vec<InterfaceDef> {
    let mut interfaces = Vec::new();

    if let crate::ast::Expr::Array(elems) = expr {
        // Find first object element to extract structure
        for elem in elems {
            if let crate::ast::Expr::Object(pairs) = elem {
                let interface_name = format!("{}{}", parent_name, to_pascal_case(name));
                let mut interface_fields = HashMap::new();

                // Add id field for ForEach key function
                interface_fields.insert("id".to_string(), "string".to_string());

                for p in pairs {
                    let key = p.key.to_astr().to_string();
                    let value = &p.value;
                    // Check if this field is an array of objects (nested)
                    if let crate::ast::Expr::Array(nested_elems) = value.as_ref() {
                        // Check if array contains objects
                        let has_objects = nested_elems.iter().any(|e| matches!(e, crate::ast::Expr::Object(_)));
                        if has_objects {
                            // Generate nested interface name
                            let nested_interface_name = format!("{}Item", interface_name);
                            interface_fields.insert(key.clone(), format!("{}[]", nested_interface_name));

                            // Recursively extract nested interface
                            let nested_interfaces = extract_all_interfaces_from_array(&interface_name, &key, value);
                            interfaces.extend(nested_interfaces);
                        } else {
                            let field_type = infer_arkts_type_from_expr(value);
                            interface_fields.insert(key.clone(), field_type);
                        }
                    } else {
                        let field_type = infer_arkts_type_from_expr(value);
                        interface_fields.insert(key.clone(), field_type);
                    }
                }

                interfaces.push(InterfaceDef {
                    name: interface_name,
                    fields: interface_fields,
                });
                break; // Only need to process first object
            }
        }
    }

    interfaces
}

/// Convert snake_case or camelCase to PascalCase for interface names
pub fn to_pascal_case(name: &str) -> String {
    // Handle common naming patterns
    // "items" -> "Item", "enablementItems" -> "EnablementItem"
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    // Remove trailing 's' if present (plural to singular)
    if result.ends_with('s') && result.len() > 1 {
        result.pop();
    }

    result
}

/// Infer ArkTS type from an expression
fn infer_arkts_type_from_expr(expr: &crate::ast::Expr) -> String {
    match expr {
        crate::ast::Expr::Str(s) | crate::ast::Expr::CStr(s) => {
            // Check if it's a resource reference
            if s.starts_with("$r(") {
                "ResourceStr".to_string()
            } else {
                "string".to_string()
            }
        }
        crate::ast::Expr::Int(_) | crate::ast::Expr::Float(_, _) | crate::ast::Expr::Double(_, _) => "number".to_string(),
        crate::ast::Expr::Bool(_) => "boolean".to_string(),
        crate::ast::Expr::Array(_) => "Object[]".to_string(),
        crate::ast::Expr::Object(_) => "Object".to_string(),
        _ => "Object".to_string(),
    }
}

/// Generate interface definition code
pub fn generate_interface(interface: &InterfaceDef) -> String {
    let mut lines = vec![format!("interface {} {{", interface.name)];

    // Sort fields for consistent output
    let mut fields: Vec<_> = interface.fields.iter().collect();
    fields.sort_by_key(|(k, _)| *k);

    for (key, ty) in fields {
        lines.push(format!("  {}: {}", key, ty));
    }

    lines.push("}".to_string());
    lines.join("\n")
}

/// Generate all interfaces needed for a widget's state
#[allow(dead_code)]
pub fn generate_interfaces(widget: &AuraWidget) -> Vec<InterfaceDef> {
    let mut interfaces = Vec::new();

    for state_var in &widget.state_vars {
        if let Some(interface) = extract_interface_from_array(&state_var.name, &state_var.initial) {
            interfaces.push(interface);
        }
    }

    interfaces
}

/// Generate all interfaces needed for a widget's state with widget name prefix
///
/// This generates interface names like "EnablementViewItem" instead of just "Item"
/// to avoid naming conflicts when multiple widgets have similar state variable names.
/// Also generates nested interfaces for arrays within objects.
pub fn generate_interfaces_with_prefix(widget: &AuraWidget, widget_name: &str) -> Vec<InterfaceDef> {
    let mut interfaces = Vec::new();

    for state_var in &widget.state_vars {
        // Use the new function that extracts all interfaces including nested ones
        let all_interfaces = extract_all_interfaces_from_array(widget_name, &state_var.name, &state_var.initial);
        interfaces.extend(all_interfaces);
    }

    interfaces
}

/// Generate @State declarations from widget state_vars
#[allow(dead_code)]
pub fn generate_state_declarations(widget: &AuraWidget) -> String {
    generate_state_declarations_with_prefix(widget, &widget.name)
}

/// Generate @State declarations from widget state_vars with widget name prefix for interfaces
pub fn generate_state_declarations_with_prefix(widget: &AuraWidget, widget_name: &str) -> String {
    let mut lines = Vec::new();

    // Collect interfaces needed for this widget (with prefix)
    let interfaces = generate_interfaces_with_prefix(widget, widget_name);

    for state_var in &widget.state_vars {
        let name = &state_var.name;

        // Determine the ArkTS type
        // First check if this state var's array has an interface
        let base_interface_name = to_pascal_case(name);
        let prefixed_interface_name = format!("{}{}", widget_name, base_interface_name);

        // Check if we can get type from constructor when type is Unknown
        let type_from_constructor = match &state_var.initial {
            crate::ast::Expr::Call(call) => {
                if let crate::ast::Expr::Ident(type_name) = call.name.as_ref() {
                    Some(type_name.to_string())
                } else {
                    None
                }
            }
            _ => None,
        };

        let arkts_type = if is_image_source_prop(name, &state_var.type_info) {
            "ResourceStr".to_string()
        } else if interfaces.iter().any(|i| i.name == prefixed_interface_name) {
            // Use the prefixed interface type with array
            format!("{}[]", prefixed_interface_name)
        } else if matches!(state_var.type_info, Type::Unknown) {
            // If type is Unknown, try to use type from constructor
            type_from_constructor.unwrap_or_else(|| "Object".to_string())
        } else {
            auto_type_to_arkts(&state_var.type_info, &interfaces)
        };

        // Use actual initial value if it's an Array or Object, otherwise use type default
        let default_value = match &state_var.initial {
            crate::ast::Expr::Array(elems) => {
                // Check if this array has an interface (array of objects)
                let has_interface = interfaces.iter().any(|i| i.name == prefixed_interface_name);
                let elems_code: Vec<String> = elems.iter().enumerate().map(|(idx, e)| {
                    if has_interface {
                        // Add id field to objects
                        if let crate::ast::Expr::Object(pairs) = e {
                            let mut pairs_sorted: Vec<_> = pairs.iter().collect();
                            pairs_sorted.sort_by_key(|p| p.key.to_astr().to_string());
                            let mut pairs_code: Vec<String> = vec![format!("id: '{}'", idx)];
                            for p in pairs_sorted {
                                pairs_code.push(format!("{}: {}", p.key.to_astr(), expr_to_arkts(&p.value)));
                            }
                            format!("{{{}}}", pairs_code.join(", "))
                        } else {
                            expr_to_arkts(e)
                        }
                    } else {
                        expr_to_arkts(e)
                    }
                }).collect();
                format!("[{}]", elems_code.join(", "))
            }
            crate::ast::Expr::Object(pairs) => {
                let pairs: Vec<String> = pairs.iter()
                    .map(|p| format!("{}: {}", p.key.to_astr(), expr_to_arkts(&p.value)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            crate::ast::Expr::Str(s) | crate::ast::Expr::CStr(s) => {
                // Check if it's a resource reference - don't quote it
                if s.starts_with("$r(") {
                    s.to_string()
                } else if s.as_str() == "null" {
                    // null keyword - don't quote it
                    "null".to_string()
                } else {
                    format!("'{}'", s)
                }
            }
            crate::ast::Expr::Int(n) => n.to_string(),
            crate::ast::Expr::Float(f, _) | crate::ast::Expr::Double(f, _) => f.to_string(),
            crate::ast::Expr::Bool(b) => b.to_string(),
            crate::ast::Expr::Call(call) => {
                // Constructor call: TypeName(args) -> new TypeName(args)
                let type_name = call.name.repr().to_string();
                let args_code: Vec<String> = call.args.args.iter().map(|a| expr_to_arkts(&a.get_expr())).collect();
                format!("new {}({})", type_name, args_code.join(", "))
            }
            _ => generate_default_value(&state_var.type_info),
        };

        // Determine decorator based on AURA decorators, props, or default to @State
        // Priority: @Consume > @Provide > @Prop > @State
        let decorator = if let Some(consume_dec) = state_var.decorators.iter().find(|d| d.name == "Consume") {
            // @Consume decorator - consumes value from ancestor
            if let Some(key) = consume_dec.args.first() {
                format!("@Consume(\"{}\")", key)
            } else {
                "@Consume".to_string()
            }
        } else if let Some(provide_dec) = state_var.decorators.iter().find(|d| d.name == "Provide") {
            // @Provide decorator - provides value to descendants
            if let Some(key) = provide_dec.args.first() {
                format!("@Provide(\"{}\")", key)
            } else {
                "@Provide".to_string()
            }
        } else if widget.props.iter().any(|p| p.name.as_str() == name) {
            // @Prop for model properties (passed from parent)
            "@Prop".to_string()
        } else {
            // @State for internal state
            "@State".to_string()
        };

        // For @Consume, don't include initial value (it's provided by ancestor)
        if state_var.decorators.iter().any(|d| d.name == "Consume") {
            lines.push(format!("  {} {}: {}", decorator, name, arkts_type));
        } else {
            lines.push(format!("  {} {}: {} = {}", decorator, name, arkts_type, default_value));
        }
    }

    lines.join("\n")
}

/// Check if a property is likely an image source (used with Image component)
fn is_image_source_prop(name: &str, ty: &Type) -> bool {
    // Common naming patterns for image sources
    let is_image_name = name.ends_with("Src") || name.ends_with("Image") || name == "imageSrc" || name == "src" || name == "image";
    // Must be a string type (would be converted to ResourceStr)
    let is_str_type = matches!(ty, Type::StrFixed(_) | Type::StrOwned);
    is_image_name && is_str_type
}

/// Generate dispatch function from widget messages and handlers
pub fn generate_dispatch_function(widget: &AuraWidget) -> String {
    if widget.messages.is_empty() && widget.tick_interval.is_none() {
        return String::new();
    }

    let mut lines = vec![
        "  private dispatch(msg: Msg): void {".to_string(),
        "    switch (msg) {".to_string(),
    ];

    // Iterate over all message variants
    for msg in &widget.messages {
        for variant in &msg.variants {
            let msg_name = &variant.name;
            lines.push(format!("      case Msg.{}: {{", msg_name));

            // Look up handler for this message (pattern format: ".MsgName")
            let handler_key = format!(".{}", msg_name);
            if let Some(payload) = widget.handlers.get(&handler_key) {
                let body = generate_handler_body(payload);
                for line in body.lines() {
                    lines.push(format!("        {}", line));
                }
            } else {
                // No handler defined - emit placeholder
                lines.push("        // TODO: implement handler".to_string());
            }

            lines.push("        break;".to_string());
            lines.push("      }".to_string());
        }
    }

    // Add Tick case if tick_interval is set
    if widget.tick_interval.is_some() {
        lines.push("      case Msg.Tick: {".to_string());
        if let Some(payload) = widget.handlers.get(".Tick") {
            let body = generate_handler_body(payload);
            for line in body.lines() {
                lines.push(format!("        {}", line));
            }
        } else {
            lines.push("        // tick handler".to_string());
        }
        lines.push("        break;".to_string());
        lines.push("      }".to_string());
    }

    lines.push("    }".to_string());
    lines.push("  }".to_string());

    lines.join("\n")
}

/// Generate Msg enum from widget messages (TypeScript syntax)
pub fn generate_msg_enum(widget: &AuraWidget) -> String {
    if widget.messages.is_empty() {
        return String::new();
    }

    let mut lines = vec!["enum Msg {".to_string()];

    for msg in &widget.messages {
        for variant in &msg.variants {
            // TypeScript enum - simple variant
            lines.push(format!("  {},", variant.name));
        }
    }

    lines.push("}".to_string());
    lines.join("\n")
}

/// Convert Auto type to ArkTS type
fn auto_type_to_arkts(ty: &Type, interfaces: &[InterfaceDef]) -> String {
    match ty {
        Type::Int | Type::I64 => "number".to_string(),
        Type::Uint | Type::U64 => "number".to_string(),
        Type::Float | Type::Double => "number".to_string(),
        Type::Bool => "boolean".to_string(),
        Type::StrFixed(_) | Type::StrOwned => "string".to_string(),
        Type::Array(arr) => format!("{}[]", auto_type_to_arkts(&arr.elem, interfaces)),
        Type::List(elem) => format!("{}[]", auto_type_to_arkts(elem, interfaces)),
        Type::Map(k, v) => format!("HashMap<{}, {}>", auto_type_to_arkts(k, interfaces), auto_type_to_arkts(v, interfaces)),
        Type::User(decl) => decl.name.as_str().to_string(),
        Type::Tag(tag) => tag.borrow().name.as_str().to_string(),
        Type::Enum(enum_decl) => enum_decl.borrow().name.as_str().to_string(),
        Type::Spec(spec_decl) => spec_decl.borrow().name.as_str().to_string(),
        Type::GenericInstance(inst) => inst.base_name.as_str().to_string(),
        Type::Option(inner) => format!("{} | null", auto_type_to_arkts(inner, interfaces)),
        Type::Void => "void".to_string(),
        Type::Unknown => "Object".to_string(),
        _ => "Object".to_string(),
    }
}

/// Generate default value for type
fn generate_default_value(ty: &Type) -> String {
    match ty {
        Type::Int | Type::I64 => "0".to_string(),
        Type::Uint | Type::U64 => "1".to_string(),
        Type::Float | Type::Double => "1.0".to_string(),
        Type::Bool => "false".to_string(),
        Type::StrFixed(_) | Type::StrOwned => "\"\"".to_string(),
        Type::Array(_) => "[]".to_string(),
        Type::List(_) => "[]".to_string(),
        Type::Map(_, _) => "new HashMap()".to_string(),
        Type::Option(_) => "null".to_string(),
        _ => "null".to_string(),
    }
}

/// Generate handler body from logic payload.
///
/// PR-4a: delegates `AstStmts` to `ark_adapter::transpile_handler_body` for
/// real ArkTS translation. State field names are not available here (this is a
/// free function); an empty set is passed — the adapter still detects state
/// refs via `Expr::Dot(self/., field)` patterns (which is how they are parsed
/// in handler bodies). The state-field set is a secondary heuristic for bare
/// identifiers, and can be threaded through from callers that have access to
/// `widget.state_vars` if finer-grained detection is needed later.
pub fn generate_handler_body(payload: &LogicPayload) -> String {
    match payload {
        LogicPayload::AstStmts(stmts) => {
            let ctx = crate::ui_gen::ark_adapter::ArkAdapterCtx::empty();
            crate::ui_gen::ark_adapter::transpile_handler_body(stmts, &ctx)
        }
        LogicPayload::Bytecode(_) => {
            // Bytecode execution not supported in static generation
            "// Bytecode execution not supported".to_string()
        }
    }
}

/// Convert AURA expression to ArkTS code
fn expr_to_arkts(expr: &crate::ast::Expr) -> String {
    use crate::ast::Expr;
    use auto_val::Op;
    match expr {
        Expr::Str(s) | Expr::CStr(s) => {
            // Check if it's a resource reference - don't quote it
            if s.starts_with("$r(") {
                s.to_string()
            } else {
                format!("\"{}\"", s)
            }
        }
        Expr::Int(n) => n.to_string(),
        Expr::Float(n, _) | Expr::Double(n, _) => n.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Ident(name) => {
            let resolved = if name.starts_with('.') { &name[1..] } else { name.as_str() };
            format!("this.{}", resolved)
        }
        Expr::Bina(left, op, right) => {
            let left_code = expr_to_arkts(left);
            let right_code = expr_to_arkts(right);
            let op_str = match op {
                Op::Add => "+",
                Op::Sub => "-",
                Op::Mul => "*",
                Op::Div => "/",
                Op::Mod => "%",
                Op::Eq => "===",
                Op::Neq => "!==",
                Op::Lt => "<",
                Op::Le => "<=",
                Op::Gt => ">",
                Op::Ge => ">=",
                Op::And => "&&",
                Op::Or => "||",
                _ => "+",
            };
            format!("{} {} {}", left_code, op_str, right_code)
        }
        Expr::Unary(op, operand) => {
            let expr_code = expr_to_arkts(operand);
            match op {
                Op::Sub => format!("-{}", expr_code),
                _ => format!("!{}", expr_code),
            }
        }
        Expr::Call(call) => {
            if let Expr::Dot(object, method) = call.name.as_ref() {
                let obj_code = expr_to_arkts(object);
                let args_code: Vec<String> = call.args.args.iter().map(|a| expr_to_arkts(&a.get_expr())).collect();
                format!("{}.{}({})", obj_code, method, args_code.join(", "))
            } else {
                let name_code = expr_to_arkts(&call.name);
                let args_code: Vec<String> = call.args.args.iter().map(|a| expr_to_arkts(&a.get_expr())).collect();
                format!("{}({})", name_code, args_code.join(", "))
            }
        }
        Expr::Array(elems) => {
            let elems_code: Vec<String> = elems.iter().map(|e| expr_to_arkts(e)).collect();
            format!("[{}]", elems_code.join(", "))
        }
        Expr::Object(pairs) => {
            let pairs: Vec<String> = pairs.iter()
                .map(|p| format!("{}: {}", p.key.to_astr(), expr_to_arkts(&p.value)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        Expr::Closure(closure) => {
            let params: Vec<String> = closure.params.iter().map(|p| p.name.to_string()).collect();
            let body_code = expr_to_arkts(&closure.body);
            format!("({}) => {}", params.join(", "), body_code)
        }
        Expr::Dot(object, field) => {
            let obj_code = expr_to_arkts(object);
            format!("{}.{}", obj_code, field)
        }
        Expr::NavCall { path, params } => {
            let path_code = expr_to_arkts(path);
            let params_code: Vec<String> = params
                .iter()
                .map(|p| format!("{}: {}", p.key.to_astr(), expr_to_arkts(&p.value)))
                .collect();
            format!("Nav.to({}, {{ {} }})", path_code, params_code.join(", "))
        }
        Expr::Index(target, index) => {
            format!("{}[{}]", expr_to_arkts(target), expr_to_arkts(index))
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_type_to_arkts() {
        let interfaces = vec![];
        assert_eq!(auto_type_to_arkts(&Type::Int, &interfaces), "number");
        assert_eq!(auto_type_to_arkts(&Type::Bool, &interfaces), "boolean");
        assert_eq!(auto_type_to_arkts(&Type::StrFixed(0), &interfaces), "string");
    }

    #[test]
    fn test_generate_default_value() {
        assert_eq!(generate_default_value(&Type::Int), "0");
        assert_eq!(generate_default_value(&Type::Bool), "false");
        assert_eq!(generate_default_value(&Type::StrFixed(0)), "\"\"");
    }

    #[test]
    fn test_generate_default_value_for_list() {
        assert_eq!(generate_default_value(&Type::List(Box::new(Type::Int))), "[]");
    }

    #[test]
    fn test_generate_msg_enum_produces_typescript_enum() {
        use crate::aura::{AuraMessage, AuraMsgVariant, AuraNode, AuraWidget};
        use std::collections::HashMap;

        let widget = AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![AuraMessage {
                name: "Msg".to_string(),
                variants: vec![
                    AuraMsgVariant { name: "Inc".to_string(), payload: None },
                    AuraMsgVariant { name: "Dec".to_string(), payload: None },
                ],
            }],
            view_tree: AuraNode::Element {
                tag: "col".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
            api_imports: vec![],
        }
;

        let result = generate_msg_enum(&widget);

        // Should produce TypeScript enum, not Kotlin sealed class
        assert!(result.contains("enum Msg {"), "Should use 'enum' keyword");
        assert!(!result.contains("sealed class"), "Should not contain 'sealed class'");
        assert!(!result.contains("object"), "Should not contain 'object' keyword");
        assert!(result.contains("Inc,"), "Should contain 'Inc,' variant");
        assert!(result.contains("Dec,"), "Should contain 'Dec,' variant");
    }

    #[test]
    fn test_extract_interface_from_array() {
        use crate::ast::{Expr, Key, Pair};

        // Test with array of objects
        let array_expr = Expr::Array(vec![
            Expr::Object(vec![
                Pair { key: Key::StrKey("title".into()), value: Box::new(Expr::Str("A".into())) },
                Pair { key: Key::StrKey("count".into()), value: Box::new(Expr::Int(1)) },
            ]),
        ]);

        let interface = extract_interface_from_array("items", &array_expr);
        assert!(interface.is_some());

        let iface = interface.unwrap();
        assert_eq!(iface.name, "Item"); // "items" -> "Item" (singular)
        assert_eq!(iface.fields.get("title"), Some(&"string".to_string()));
        assert_eq!(iface.fields.get("count"), Some(&"number".to_string()));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("items"), "Item");
        assert_eq!(to_pascal_case("users"), "User");
        assert_eq!(to_pascal_case("enablement_items"), "EnablementItem");
    }

    #[test]
    fn test_generate_interface_code() {
        let interface = InterfaceDef {
            name: "Item".to_string(),
            fields: {
                let mut fields = HashMap::new();
                fields.insert("title".to_string(), "string".to_string());
                fields.insert("count".to_string(), "number".to_string());
                fields
            },
        };

        let code = generate_interface(&interface);
        assert!(code.contains("interface Item {"));
        assert!(code.contains("count: number"));
        assert!(code.contains("title: string"));
    }
}
