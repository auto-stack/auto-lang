//! ArkTS State Management
//!
//! Generates @State declarations and dispatch functions.

use crate::aura::{AuraBinOp, AuraExpr, AuraStmt, AuraUnaryOp, AuraUpdateOp, AuraWidget, LogicPayload};
use crate::ast::Type;

/// Generate @State declarations from widget state_vars
pub fn generate_state_declarations(widget: &AuraWidget) -> String {
    let mut lines = Vec::new();

    for state_var in &widget.state_vars {
        let name = &state_var.name;
        let arkts_type = auto_type_to_arkts(&state_var.type_info);
        let default_value = generate_default_value(&state_var.type_info);

        lines.push(format!("  @State {}: {} = {}", name, arkts_type, default_value));
    }

    lines.join("\n")
}

/// Generate dispatch function from widget messages and handlers
pub fn generate_dispatch_function(widget: &AuraWidget) -> String {
    if widget.messages.is_empty() {
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
fn auto_type_to_arkts(ty: &Type) -> String {
    match ty {
        Type::Int | Type::I64 => "number".to_string(),
        Type::Uint | Type::U64 => "number".to_string(),
        Type::Float | Type::Double => "number".to_string(),
        Type::Bool => "boolean".to_string(),
        Type::Str(_) => "string".to_string(),
        Type::Array(arr) => format!("{}[]", auto_type_to_arkts(&arr.elem)),
        Type::List(elem) => format!("{}[]", auto_type_to_arkts(elem)),
        Type::User(decl) => decl.name.as_str().to_string(),
        Type::Option(inner) => format!("{} | null", auto_type_to_arkts(inner)),
        _ => "any".to_string(),
    }
}

/// Generate default value for type
fn generate_default_value(ty: &Type) -> String {
    match ty {
        Type::Int | Type::I64 => "0".to_string(),
        Type::Uint | Type::U64 => "1".to_string(),
        Type::Float | Type::Double => "1.0".to_string(),
        Type::Bool => "false".to_string(),
        Type::Str(_) => "\"\"".to_string(),
        Type::Array(_) => "[]".to_string(),
        Type::List(_) => "[]".to_string(),
        Type::Option(_) => "null".to_string(),
        _ => "null".to_string(),
    }
}

/// Generate handler body from logic payload
pub fn generate_handler_body(payload: &LogicPayload) -> String {
    match payload {
        LogicPayload::AstBlock(stmts) => {
            // Convert AST statements to ArkTS code
            let mut lines = Vec::new();
            for stmt in stmts {
                let stmt_code = stmt_to_arkts(stmt);
                lines.push(stmt_code);
            }
            lines.join("\n")
        }
        LogicPayload::Bytecode(_) => {
            // Bytecode execution not supported in static generation
            "// Bytecode execution not supported".to_string()
        }
    }
}

/// Convert AURA statement to ArkTS code
fn stmt_to_arkts(stmt: &AuraStmt) -> String {
    match stmt {
        AuraStmt::Assign { target, value } => {
            let value_code = expr_to_arkts(value);
            format!("this.{} = {}", target, value_code)
        }
        AuraStmt::Update { target, op, value } => {
            let op_str = match op {
                AuraUpdateOp::AddAssign => "+=",
                AuraUpdateOp::SubAssign => "-=",
                AuraUpdateOp::MulAssign => "*=",
                AuraUpdateOp::DivAssign => "/=",
            };
            let value_code = expr_to_arkts(value);
            format!("this.{} {} {}", target, op_str, value_code)
        }
        AuraStmt::MethodCall {
            object,
            method,
            args,
        } => {
            let args_code: Vec<String> = args.iter().map(|a| expr_to_arkts(a)).collect();
            format!("this.{}.{}({})", object, method, args_code.join(", "))
        }
    }
}

/// Convert AURA expression to ArkTS code
fn expr_to_arkts(expr: &AuraExpr) -> String {
    match expr {
        AuraExpr::Literal(s) => format!("\"{}\"", s),
        AuraExpr::Int(n) => n.to_string(),
        AuraExpr::Float(n) => n.to_string(),
        AuraExpr::Bool(b) => b.to_string(),
        AuraExpr::StateRef(name) => format!("this.{}", name),
        AuraExpr::MsgVariant { msg_type, variant } => {
            format!("{}.{}", msg_type, variant)
        }
        AuraExpr::Binary { left, op, right } => {
            let left_code = expr_to_arkts(left);
            let right_code = expr_to_arkts(right);
            let op_str = match op {
                AuraBinOp::Add => "+",
                AuraBinOp::Sub => "-",
                AuraBinOp::Mul => "*",
                AuraBinOp::Div => "/",
                AuraBinOp::Mod => "%",
                AuraBinOp::Eq => "===",
                AuraBinOp::Ne => "!==",
                AuraBinOp::Lt => "<",
                AuraBinOp::Le => "<=",
                AuraBinOp::Gt => ">",
                AuraBinOp::Ge => ">=",
                AuraBinOp::And => "&&",
                AuraBinOp::Or => "||",
            };
            format!("{} {} {}", left_code, op_str, right_code)
        }
        AuraExpr::Unary { op, operand } => {
            let expr_code = expr_to_arkts(operand);
            match op {
                AuraUnaryOp::Neg => format!("-{}", expr_code),
                AuraUnaryOp::Not => format!("!{}", expr_code),
            }
        }
        AuraExpr::MethodCall {
            object,
            method,
            args,
        } => {
            let obj_code = expr_to_arkts(object);
            let args_code: Vec<String> = args.iter().map(|a| expr_to_arkts(a)).collect();
            format!("{}.{}({})", obj_code, method, args_code.join(", "))
        }
        AuraExpr::Array(elems) => {
            let elems_code: Vec<String> = elems.iter().map(|e| expr_to_arkts(e)).collect();
            format!("[{}]", elems_code.join(", "))
        }
        AuraExpr::Lambda { params, body } => {
            let body_code = expr_to_arkts(body);
            format!("({}) => {}", params.join(", "), body_code)
        }
        AuraExpr::FieldAccess { object, field } => {
            let obj_code = expr_to_arkts(object);
            format!("{}.{}", obj_code, field)
        }
        AuraExpr::NavCall { path, params } => {
            let params_code: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}: {}", k, expr_to_arkts(v)))
                .collect();
            format!("Nav.to(\"{}\", {{ {} }})", path, params_code.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_type_to_arkts() {
        assert_eq!(auto_type_to_arkts(&Type::Int), "number");
        assert_eq!(auto_type_to_arkts(&Type::Bool), "boolean");
        assert_eq!(auto_type_to_arkts(&Type::Str(0)), "string");
    }

    #[test]
    fn test_generate_default_value() {
        assert_eq!(generate_default_value(&Type::Int), "0");
        assert_eq!(generate_default_value(&Type::Bool), "false");
        assert_eq!(generate_default_value(&Type::Str(0)), "\"\"");
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
            },
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
        };

        let result = generate_msg_enum(&widget);

        // Should produce TypeScript enum, not Kotlin sealed class
        assert!(result.contains("enum Msg {"), "Should use 'enum' keyword");
        assert!(!result.contains("sealed class"), "Should not contain 'sealed class'");
        assert!(!result.contains("object"), "Should not contain 'object' keyword");
        assert!(result.contains("Inc,"), "Should contain 'Inc,' variant");
        assert!(result.contains("Dec,"), "Should contain 'Dec,' variant");
    }
}
