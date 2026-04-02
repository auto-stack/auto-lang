//! Plan 125: Phase 3.6 - Task Type Checking
//!
//! This module provides type checking for task message routing.
//!
//! ## Overview
//!
//! When a task defines an `on` block with mixed patterns:
//!
//! ```auto
//! on(ctx) {
//!     "ping" => { ctx.reply("pong") }
//!     msg string => { write_to_disk(msg) }
//!     amount int if amount > 10000 => { ctx.reply("need_approval") }
//! }
//! ```
//!
//! The type checker:
//! 1. Infers the envelope type from patterns
//! 2. Checks send/ask message types against the envelope
//! 3. Infers reply types from ctx.reply() calls

use crate::ast::{Arg, Body, Expr, LiteralValue, Name, TaskMsgPattern, TaskOnBlock, Type, TypeDecl, TypeDeclKind, Union, UnionField};
use crate::error::TypeError;
use std::collections::HashMap;

/// Inferred envelope type information for a task
#[derive(Debug, Clone)]
pub struct EnvelopeInfo {
    /// The task name
    pub task_name: Name,
    /// The inferred envelope type (union of all pattern types)
    pub envelope_type: Type,
    /// Mapping from pattern to handler return type (for ask mode)
    pub reply_types: HashMap<String, Type>,
}

impl EnvelopeInfo {
    /// Create a new EnvelopeInfo for a task
    pub fn new(task_name: &str) -> Self {
        Self {
            task_name: task_name.into(),
            envelope_type: Type::Unknown,
            reply_types: HashMap::new(),
        }
    }
}

/// Task type checker for Phase 3 polymorphic routing
pub struct TaskTypeChecker {
    /// Collected envelope info per task
    pub envelopes: HashMap<Name, EnvelopeInfo>,
}

impl TaskTypeChecker {
    /// Create a new TaskTypeChecker
    pub fn new() -> Self {
        Self {
            envelopes: HashMap::new(),
        }
    }

    /// Infer envelope type from an on block
    ///
    /// # Arguments
    ///
    /// * `task_name` - The task name
    /// * `on_block` - The on block to analyze
    ///
    /// # Returns
    ///
    /// The inferred envelope type (union of all pattern types)
    pub fn infer_envelope_type(&mut self, task_name: &str, on_block: &TaskOnBlock) -> Type {
        let mut variants: Vec<Type> = Vec::new();

        for (pattern, _guard, _body) in &on_block.handlers {
            let pattern_type = self.pattern_to_type(pattern);
            variants.push(pattern_type);
        }

        if variants.is_empty() {
            Type::Void
        } else if variants.len() == 1 {
            variants.into_iter().next().unwrap()
        } else {
            // Create a union type with all variants
            let fields: Vec<UnionField> = variants
                .iter()
                .enumerate()
                .map(|(i, ty)| UnionField {
                    name: format!("Variant{}", i).into(),
                    ty: ty.clone(),
                })
                .collect();

            Type::Union(Union {
                name: format!("{}Envelope", task_name).into(),
                fields,
            })
        }
    }

    /// Convert a pattern to its corresponding type
    fn pattern_to_type(&self, pattern: &TaskMsgPattern) -> Type {
        match pattern {
            TaskMsgPattern::Literal(lit) => literal_to_type(lit),
            TaskMsgPattern::TypeBinding { name: _, type_expr } => type_expr.as_ref().clone(),
            TaskMsgPattern::Simple(variant_name) => {
                // Simple variant is like a unit enum variant
                Type::User(TypeDecl {
                    name: variant_name.clone(),
                    kind: TypeDeclKind::UserType,
                    parent: None,
                    has: Vec::new(),
                    specs: Vec::new(),
                    spec_impls: Vec::new(),
                    generic_params: Vec::new(),
                    members: Vec::new(),
                    delegations: Vec::new(),
                    methods: Vec::new(),
                })
            }
            TaskMsgPattern::WithBindings { variant, bindings: _ } => {
                // Variant with bindings is like a tuple enum variant
                Type::User(TypeDecl {
                    name: variant.clone(),
                    kind: TypeDeclKind::UserType,
                    parent: None,
                    has: Vec::new(),
                    specs: Vec::new(),
                    spec_impls: Vec::new(),
                    generic_params: Vec::new(),
                    members: Vec::new(),
                    delegations: Vec::new(),
                    methods: Vec::new(),
                })
            }
        }
    }

    /// Check if a message type is accepted by a task
    ///
    /// # Arguments
    ///
    /// * `envelope_type` - The task's envelope type
    /// * `message_type` - The message type being sent
    ///
    /// # Returns
    ///
    /// Ok(()) if the message type is compatible, Err otherwise
    pub fn check_send_type(
        &self,
        envelope_type: &Type,
        message_type: &Type,
    ) -> Result<(), TypeError> {
        if self.type_accepts(envelope_type, message_type) {
            Ok(())
        } else {
            Err(TypeError::Mismatch {
                expected: format!("{:?}", envelope_type),
                found: format!("{:?}", message_type),
                span: miette::SourceSpan::new(0_usize.into(), 0_usize.into()),
            })
        }
    }

    /// Check if a type accepts another type
    fn type_accepts(&self, expected: &Type, found: &Type) -> bool {
        match (expected, found) {
            // Unknown accepts anything
            (Type::Unknown, _) | (_, Type::Unknown) => true,

            // Same types
            (Type::Int, Type::Int) => true,
            (Type::Uint, Type::Uint) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Char, Type::Char) => true,
            (Type::Str(_), Type::Str(_))
            | (Type::Str(_), Type::String)
            | (Type::String, Type::Str(_))
            | (Type::String, Type::String) => true,
            (Type::Void, Type::Void) => true,

            // Numeric compatibility
            (Type::Int, Type::I64) | (Type::I64, Type::Int) => true,
            (Type::Uint, Type::U64) | (Type::U64, Type::Uint) => true,

            // Union types: check if any field accepts
            (Type::Union(union_type), found_type) => {
                union_type.fields.iter().any(|f| self.type_accepts(&f.ty, found_type))
            }

            // User types
            (Type::User(a), Type::User(b)) => a.name == b.name,

            // Default: no match
            _ => false,
        }
    }

    /// Infer reply type from handler body
    ///
    /// Looks for ctx.reply(expr) calls and infers the type of expr
    pub fn infer_reply_type(&self, body: &Body) -> Option<Type> {
        for stmt in &body.stmts {
            if let Some(reply_type) = self.extract_reply_type_from_stmt(stmt) {
                return Some(reply_type);
            }
        }
        None
    }

    /// Extract reply type from a statement
    fn extract_reply_type_from_stmt(&self, stmt: &crate::ast::Stmt) -> Option<Type> {
        match stmt {
            crate::ast::Stmt::Expr(expr) => self.extract_reply_type_from_expr(expr),
            _ => None,
        }
    }

    /// Extract reply type from an expression
    fn extract_reply_type_from_expr(&self, expr: &Expr) -> Option<Type> {
        match expr {
            // ctx.reply(value) - Call with Dot callee
            Expr::Call(call) => {
                // Check if this is ctx.reply call
                if let Expr::Dot(obj, method) = call.name.as_ref() {
                    if let Expr::Ident(name) = obj.as_ref() {
                        if name.as_str() == "ctx" && method.as_str() == "reply" {
                            if let Some(arg) = call.args.args.first() {
                                return Some(self.arg_to_type(arg));
                            }
                        }
                    }
                }
                None
            }
            // Block expression
            Expr::Block(body) => self.infer_reply_type(body),
            _ => None,
        }
    }

    /// Convert an argument to its type
    fn arg_to_type(&self, arg: &Arg) -> Type {
        match arg {
            Arg::Pos(expr) => self.expr_to_type(expr),
            Arg::Name(name) => Type::User(TypeDecl {
                name: name.clone(),
                kind: TypeDeclKind::UserType,
                parent: None,
                has: Vec::new(),
                specs: Vec::new(),
                spec_impls: Vec::new(),
                generic_params: Vec::new(),
                members: Vec::new(),
                delegations: Vec::new(),
                methods: Vec::new(),
            }),
            Arg::Pair(_, expr) => self.expr_to_type(expr),
        }
    }

    /// Convert an expression to its inferred type (simplified)
    fn expr_to_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Int(_) => Type::Int,
            Expr::Uint(_) => Type::Uint,
            Expr::I64(_) => Type::I64,
            Expr::U64(_) => Type::U64,
            Expr::Float(_, _) => Type::Float,
            Expr::Double(_, _) => Type::Double,
            Expr::Bool(_) => Type::Bool,
            Expr::Char(_) => Type::Char,
            Expr::Str(s) => Type::Str(s.len()),
            Expr::Ident(name) => Type::User(TypeDecl {
                name: name.clone(),
                kind: TypeDeclKind::UserType,
                parent: None,
                has: Vec::new(),
                specs: Vec::new(),
                spec_impls: Vec::new(),
                generic_params: Vec::new(),
                members: Vec::new(),
                delegations: Vec::new(),
                methods: Vec::new(),
            }),
            _ => Type::Unknown,
        }
    }
}

impl Default for TaskTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a literal value to its corresponding type
pub fn literal_to_type(lit: &LiteralValue) -> Type {
    match lit {
        LiteralValue::String(s) => Type::Str(s.len()),
        LiteralValue::Int(_) => Type::Int,
        LiteralValue::Uint(_) => Type::Uint,
        LiteralValue::Float(_, _) => Type::Float,
        LiteralValue::Bool(_) => Type::Bool,
        LiteralValue::Char(_) => Type::Char,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_to_type() {
        assert!(matches!(literal_to_type(&LiteralValue::Int(42)), Type::Int));
        assert!(matches!(literal_to_type(&LiteralValue::Bool(true)), Type::Bool));
        assert!(matches!(literal_to_type(&LiteralValue::Char('a')), Type::Char));
    }

    #[test]
    fn test_task_type_checker_new() {
        let checker = TaskTypeChecker::new();
        assert!(checker.envelopes.is_empty());
    }

    #[test]
    fn test_check_send_type_matching() {
        let checker = TaskTypeChecker::new();
        let result = checker.check_send_type(&Type::Int, &Type::Int);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_send_type_mismatch() {
        let checker = TaskTypeChecker::new();
        let result = checker.check_send_type(&Type::Int, &Type::Bool);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_send_type_union() {
        let checker = TaskTypeChecker::new();
        let envelope = Type::Union(Union {
            name: "TestEnvelope".into(),
            fields: vec![
                UnionField { name: "IntVariant".into(), ty: Type::Int },
                UnionField { name: "StrVariant".into(), ty: Type::Str(0) },
            ],
        });
        assert!(checker.check_send_type(&envelope, &Type::Int).is_ok());
        assert!(checker.check_send_type(&envelope, &Type::Str(0)).is_ok());
        assert!(checker.check_send_type(&envelope, &Type::Bool).is_err());
    }

    #[test]
    fn test_type_accepts_unknown() {
        let checker = TaskTypeChecker::new();
        assert!(checker.type_accepts(&Type::Unknown, &Type::Int));
        assert!(checker.type_accepts(&Type::Int, &Type::Unknown));
    }

    #[test]
    fn test_infer_envelope_type_empty() {
        use crate::token::Pos;
        let mut checker = TaskTypeChecker::new();
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let on_block = TaskOnBlock::new(pos);
        let result = checker.infer_envelope_type("TestTask", &on_block);
        assert!(matches!(result, Type::Void));
    }

    #[test]
    fn test_infer_envelope_type_single_pattern() {
        use crate::ast::Body;
        use crate::token::Pos;

        let mut checker = TaskTypeChecker::new();
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let mut on_block = TaskOnBlock::new(pos);

        on_block.add_handler_with_guard(
            TaskMsgPattern::TypeBinding {
                name: "msg".into(),
                type_expr: Box::new(Type::Str(0)),
            },
            None,
            Body::new(),
        );

        let result = checker.infer_envelope_type("TestTask", &on_block);
        assert!(matches!(result, Type::Str(_)));
    }

    #[test]
    fn test_infer_envelope_type_multiple_patterns() {
        use crate::ast::Body;
        use crate::token::Pos;

        let mut checker = TaskTypeChecker::new();
        let pos = Pos { line: 1, at: 1, pos: 0, len: 0 };
        let mut on_block = TaskOnBlock::new(pos);

        on_block.add_handler_with_guard(
            TaskMsgPattern::Literal(LiteralValue::String("ping".into())),
            None,
            Body::new(),
        );

        on_block.add_handler_with_guard(
            TaskMsgPattern::TypeBinding {
                name: "msg".into(),
                type_expr: Box::new(Type::Str(0)),
            },
            None,
            Body::new(),
        );

        let result = checker.infer_envelope_type("TestTask", &on_block);
        assert!(matches!(result, Type::Union(_)));
    }
}
