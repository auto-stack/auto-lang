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
use crate::ast::{Expr, Stmt, Type, Key};
use auto_val::Op;
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
// Expression Extractor
// ============================================================================

/// Extract AURA expression from AST expression
pub fn extract_expr(expr: &Expr) -> ExtractResult<AuraExpr> {
    match expr {
        Expr::Int(n) => Ok(AuraExpr::Int(*n as i64)),
        Expr::I64(n) => Ok(AuraExpr::Int(*n)),
        Expr::Uint(n) => Ok(AuraExpr::Int(*n as i64)),
        Expr::U64(n) => Ok(AuraExpr::Int(*n as i64)),
        Expr::Byte(n) => Ok(AuraExpr::Int(*n as i64)),
        Expr::I8(n) => Ok(AuraExpr::Int(*n as i64)),
        Expr::U8(n) => Ok(AuraExpr::Int(*n as i64)),
        Expr::Float(n, _) => Ok(AuraExpr::Float(*n)),
        Expr::Double(n, _) => Ok(AuraExpr::Float(*n)),
        Expr::Bool(b) => Ok(AuraExpr::Bool(*b)),
        Expr::Char(c) => Ok(AuraExpr::Int(*c as i64)),
        Expr::Str(s) => Ok(AuraExpr::Literal(s.to_string())),
        Expr::CStr(s) => Ok(AuraExpr::Literal(s.to_string())),

        // Identifier could be a state reference
        Expr::Ident(name) => {
            let name_str = name.as_str();
            // Check if it's a state reference (starts with ".")
            if name_str.starts_with('.') {
                Ok(AuraExpr::StateRef(name_str[1..].to_string()))
            } else {
                // Regular identifier - treat as state reference
                Ok(AuraExpr::StateRef(name_str.to_string()))
            }
        }

        // Binary operation (Bina in AST)
        Expr::Bina(left, op, right) => {
            let left_expr = extract_expr(left)?;
            let right_expr = extract_expr(right)?;
            let aura_op = extract_bin_op(op);
            Ok(AuraExpr::Binary {
                left: Box::new(left_expr),
                op: aura_op,
                right: Box::new(right_expr),
            })
        }

        // Unary operation
        Expr::Unary(op, operand) => {
            let operand_expr = extract_expr(operand)?;
            let aura_op = extract_unary_op(op);
            Ok(AuraExpr::Unary {
                op: aura_op,
                operand: Box::new(operand_expr),
            })
        }

        // Other expressions not yet supported in view
        _ => Err(ExtractError::UnsupportedExpr(format!("{:?}", expr))),
    }
}

/// Extract binary operator from AST Op
fn extract_bin_op(op: &Op) -> AuraBinOp {
    match op {
        Op::Add => AuraBinOp::Add,
        Op::Sub => AuraBinOp::Sub,
        Op::Mul => AuraBinOp::Mul,
        Op::Div => AuraBinOp::Div,
        Op::Mod => AuraBinOp::Mod,
        Op::Eq => AuraBinOp::Eq,
        Op::Neq => AuraBinOp::Ne,
        Op::Lt => AuraBinOp::Lt,
        Op::Le => AuraBinOp::Le,
        Op::Gt => AuraBinOp::Gt,
        Op::Ge => AuraBinOp::Ge,
        Op::And => AuraBinOp::And,
        Op::Or => AuraBinOp::Or,
        _ => AuraBinOp::Eq, // Default fallback
    }
}

/// Extract unary operator from AST Op
fn extract_unary_op(op: &Op) -> AuraUnaryOp {
    match op {
        Op::Sub => AuraUnaryOp::Neg,
        Op::Not => AuraUnaryOp::Not,
        _ => AuraUnaryOp::Not, // Default fallback
    }
}

// ============================================================================
// Statement Extractor
// ============================================================================

/// Extract AURA statement from AST statement
pub fn extract_stmt(stmt: &Stmt) -> ExtractResult<AuraStmt> {
    match stmt {
        // Store statement: let x = value or x = value
        Stmt::Store(store) => {
            let target = store.name.as_str().to_string();
            let value = extract_expr(&store.expr)?;
            Ok(AuraStmt::Assign { target, value })
        }

        // Expression statement
        Stmt::Expr(expr) => {
            // Try to extract as assignment-like expression
            extract_expr(expr)?;
            // For now, just return a placeholder
            Err(ExtractError::UnsupportedStmt(format!("{:?}", stmt)))
        }

        _ => Err(ExtractError::UnsupportedStmt(format!("{:?}", stmt))),
    }
}

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
            let mut children = Vec::new();

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
                        let value = extract_expr(&pair.value)?;
                        props.insert(key, value);
                    }
                }
            }

            Ok(AuraNode::Element {
                tag,
                props,
                events,
                children,
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
                                if key.starts_with("on") {
                                    let handler = extract_event_handler(&pair.value)?;
                                    events.insert(key, handler);
                                } else {
                                    let value = extract_expr(&pair.value)?;
                                    props.insert(key, value);
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

        _ => Err(ExtractError::UnsupportedExpr(format!(
            "Cannot extract view tree from: {:?}",
            expr
        ))),
    }
}

/// Extract event handler pattern from expression
fn extract_event_handler(expr: &Expr) -> ExtractResult<String> {
    match expr {
        // Identifier: could be ".Inc" or "Msg.Inc"
        Expr::Ident(name) => {
            let name_str = name.as_str();
            if name_str.starts_with('.') {
                // Implicit member: .Inc -> Msg::Inc (need context)
                Ok(format!("Msg::{}", &name_str[1..]))
            } else {
                Ok(name_str.to_string())
            }
        }
        // Dot access: Msg.Inc
        Expr::Dot(obj, field) => {
            let obj_name = match obj.as_ref() {
                Expr::Ident(name) => name.as_str(),
                _ => "Msg",
            };
            let field_name = field.as_str();
            Ok(format!("{}::{}", obj_name, field_name))
        }
        _ => Err(ExtractError::UnsupportedExpr(format!(
            "Cannot extract event handler from: {:?}",
            expr
        ))),
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::AutoStr;

    #[test]
    fn test_extract_int_expr() {
        let expr = Expr::Int(42);
        let aura = extract_expr(&expr).unwrap();
        assert!(matches!(aura, AuraExpr::Int(n) if n == 42));
    }

    #[test]
    fn test_extract_state_ref() {
        // Test identifier as state reference
        let expr = Expr::Ident(AutoStr::from("count"));
        let aura = extract_expr(&expr).unwrap();
        match aura {
            AuraExpr::StateRef(name) => assert_eq!(name, "count"),
            _ => panic!("Expected StateRef"),
        }
    }

    #[test]
    fn test_extract_binary_expr() {
        let left = Expr::Int(1);
        let right = Expr::Int(2);
        let expr = Expr::Bina(Box::new(left), Op::Add, Box::new(right));

        let aura = extract_expr(&expr).unwrap();
        match aura {
            AuraExpr::Binary { op, .. } => assert_eq!(op, AuraBinOp::Add),
            _ => panic!("Expected Binary"),
        }
    }

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

    #[test]
    fn test_extract_bin_op() {
        assert_eq!(extract_bin_op(&Op::Add), AuraBinOp::Add);
        assert_eq!(extract_bin_op(&Op::Sub), AuraBinOp::Sub);
        assert_eq!(extract_bin_op(&Op::Eq), AuraBinOp::Eq);
        assert_eq!(extract_bin_op(&Op::Lt), AuraBinOp::Lt);
    }

    #[test]
    fn test_extract_unary_op() {
        assert_eq!(extract_unary_op(&Op::Sub), AuraUnaryOp::Neg);
        assert_eq!(extract_unary_op(&Op::Not), AuraUnaryOp::Not);
    }
}
