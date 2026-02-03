//! Statement-level type checking for AutoLang
//!
//! This module implements type checking for various statement types including
//! variable declarations, assignments, and control flow statements.
//!
//! # Overview
//!
//! Statement type checking validates that:
//! - Variable declarations have compatible types
//! - Assignments maintain type consistency
//! - Control flow conditions are boolean
//! - Return statements match function signatures
//!
//! # Example
//!
//! ```rust
//! use auto_lang::infer::{InferenceContext, check_stmt};
//! use auto_lang::ast::{Stmt, Store, StoreKind, Expr, Type, Name};
//!
//! let mut ctx = InferenceContext::new();
//! let store = Store {
//!     kind: StoreKind::Let,
//!     name: Name::from("x"),
//!     ty: Type::Int,
//!     expr: Expr::Int(42),
//! };
//! let result = check_stmt(&mut ctx, &Stmt::Store(store));
//! assert!(result.is_ok());
//! ```

use crate::ast::{Body, Expr, For, If, Stmt, Store, Type};
use crate::error::{AutoError, TypeError};
use crate::infer::{check_fn, infer_expr, InferenceContext};
use miette::SourceSpan;

/// Check a statement for type correctness
///
/// # Arguments
///
/// * `ctx` - Type inference context
/// * `stmt` - Statement to check
///
/// * Returns the type of the statement (Type::Void for most statements,
/// or the expression type for expression statements)
///
/// # Errors
///
/// Returns a TypeError if:
/// - Variable types don't match their initializers
/// - Assignments have incompatible types
/// - Control flow conditions are not boolean
/// - Return types don't match function signature
///
/// # Example
///
/// ```rust
/// use auto_lang::infer::{InferenceContext, check_stmt};
/// use auto_lang::ast::{Stmt, Expr};
///
/// let mut ctx = InferenceContext::new();
/// let stmt = Stmt::Expr(Expr::Int(42));
/// let result = check_stmt(&mut ctx, &stmt);
/// assert!(result.is_ok());
/// ```
pub fn check_stmt(ctx: &mut InferenceContext, stmt: &Stmt) -> Result<Type, AutoError> {
    match stmt {
        // Variable declaration (let/mut/var)
        Stmt::Store(store) => check_store(ctx, store),

        // Expression statement
        Stmt::Expr(expr) => {
            let ty = infer_expr(ctx, expr);
            Ok(ty)
        }

        // If statement/expression
        Stmt::If(if_stmt) => check_if(ctx, if_stmt),

        // For loop
        Stmt::For(for_stmt) => check_for(ctx, for_stmt),

        // Return statement
        Stmt::Return(expr) => check_return(ctx, expr),

        // Block
        Stmt::Block(body) => check_body(ctx, body),

        // Function declaration
        Stmt::Fn(fn_decl) => check_fn(ctx, fn_decl),

        // Type declarations - no runtime type checking needed
        Stmt::TypeDecl(_)
        | Stmt::EnumDecl(_)
        | Stmt::Union(_)
        | Stmt::Tag(_)
        | Stmt::SpecDecl(_)
        | Stmt::TypeAlias(_)
        | Stmt::Ext(_) => Ok(Type::Void),

        // Other statements
        Stmt::Break => Ok(Type::Void),
        Stmt::Comment(_) | Stmt::EmptyLine(_) => Ok(Type::Void),

        // Use, Node, OnEvents, Alias, Is - no type checking needed
        Stmt::Use(_) | Stmt::Node(_) | Stmt::OnEvents(_) | Stmt::Alias(_) | Stmt::Is(_) => {
            Ok(Type::Void)
        }
    }
}

/// Check a variable declaration (let/mut/var)
///
/// Validates that the initializer expression's type matches the declared type
/// (if specified), or infers the type from the expression.
///
/// # Arguments
///
/// * `ctx` - Type inference context
/// * `store` - Variable declaration to check
///
/// # Returns
///
/// Type::Void (declarations don't produce values)
///
/// # Errors
///
/// Returns TypeError::Mismatch if the declared type and expression type are incompatible
fn check_store(ctx: &mut InferenceContext, store: &Store) -> Result<Type, AutoError> {
    // 1. Infer type from expression
    let expr_ty = infer_expr(ctx, &store.expr);

    // 2. Check against declared type (if specified)
    let final_ty = if !matches!(store.ty, Type::Unknown) {
        // Has type annotation - verify compatibility
        match ctx.unify(store.ty.clone(), expr_ty.clone()) {
            Ok(unified_ty) => unified_ty,
            Err(e) => {
                // Add to error list but continue
                ctx.errors.push(e.into());
                store.ty.clone() // Use declared type
            }
        }
    } else {
        // No annotation - use inferred type
        expr_ty
    };

    // 3. Bind variable in context
    ctx.bind_var(store.name.clone(), final_ty);

    Ok(Type::Void)
}

/// Check an if statement/expression
///
/// Validates that:
/// - All conditions are boolean expressions
/// - Then/else branches have compatible types (for if expressions)
///
/// # Arguments
///
/// * `ctx` - Type inference context
/// * `if_stmt` - If statement to check
///
/// # Returns
///
/// The unified type of the branches (for if expressions), or Type::Void (for if statements)
fn check_if(ctx: &mut InferenceContext, if_stmt: &If) -> Result<Type, AutoError> {
    let mut branch_types = Vec::new();

    // Check each if/elif branch
    for branch in &if_stmt.branches {
        // 1. Check condition is boolean
        let cond_ty = infer_expr(ctx, &branch.cond);
        if !matches!(cond_ty, Type::Bool | Type::Unknown) {
            ctx.errors.push(
                TypeError::Mismatch {
                    expected: "bool".to_string(),
                    found: cond_ty.to_string(),
                    span: SourceSpan::new(0.into(), 0),
                }
                .into(),
            );
        }

        // 2. Check branch body
        let branch_ty = check_body(ctx, &branch.body)?;
        branch_types.push(branch_ty);
    }

    // 3. Check else branch if present
    let else_ty = if let Some(else_body) = &if_stmt.else_ {
        check_body(ctx, else_body)?
    } else {
        Type::Void
    };

    branch_types.push(else_ty);

    // 4. Unify all branch types
    // If all are Void, it's a statement
    if branch_types.iter().all(|ty| matches!(ty, Type::Void)) {
        Ok(Type::Void)
    } else {
        // It's an expression - unify all branch types
        let mut unified = branch_types[0].clone();
        for ty in &branch_types[1..] {
            match ctx.unify(unified.clone(), ty.clone()) {
                Ok(new_unified) => unified = new_unified,
                Err(e) => {
                    ctx.errors.push(e.into());
                    return Ok(Type::Unknown);
                }
            }
        }
        Ok(unified)
    }
}

/// Check a for loop
///
/// Validates that the range expression produces an iterable type.
///
/// # Arguments
///
/// * `ctx` - Type inference context
/// * `for_stmt` - For loop to check
///
/// # Returns
///
/// Type::Void (loops don't produce values)
fn check_for(ctx: &mut InferenceContext, for_stmt: &For) -> Result<Type, AutoError> {
    use crate::ast::Iter;

    // Push new scope for loop variable
    ctx.push_scope();

    // Check based on iterator type
    match &for_stmt.iter {
        Iter::Named(var_name) => {
            // Standard iteration: for x in array
            let range_ty = infer_expr(ctx, &for_stmt.range);

            match range_ty {
                Type::Array(ref arr_ty) => {
                    // Bind loop variable to element type
                    ctx.bind_var(var_name.clone(), (*arr_ty.elem).clone());
                }
                Type::Unknown => {
                    // Unknown iterator - can't check
                    ctx.bind_var(var_name.clone(), Type::Unknown);
                }
                _ => {
                    // Non-iterable type
                    ctx.errors.push(
                        TypeError::InvalidOperation {
                            op: "iterate".to_string(),
                            ty: range_ty.to_string(),
                            span: SourceSpan::new(0.into(), 0),
                        }
                        .into(),
                    );
                    ctx.bind_var(var_name.clone(), Type::Unknown);
                }
            }
        }

        Iter::Indexed(index_var, value_var) => {
            // Indexed iteration: for i, x in array
            let range_ty = infer_expr(ctx, &for_stmt.range);

            // Index is always int
            ctx.bind_var(index_var.clone(), Type::Int);

            match range_ty {
                Type::Array(ref arr_ty) => {
                    ctx.bind_var(value_var.clone(), (*arr_ty.elem).clone());
                }
                Type::Unknown => {
                    ctx.bind_var(value_var.clone(), Type::Unknown);
                }
                _ => {
                    ctx.errors.push(
                        TypeError::InvalidOperation {
                            op: "iterate".to_string(),
                            ty: range_ty.to_string(),
                            span: SourceSpan::new(0.into(), 0),
                        }
                        .into(),
                    );
                    ctx.bind_var(value_var.clone(), Type::Unknown);
                }
            }
        }

        Iter::Cond => {
            // Conditional for loop: for condition { }
            let cond_ty = infer_expr(ctx, &for_stmt.range);
            if !matches!(cond_ty, Type::Bool | Type::Unknown) {
                ctx.errors.push(
                    TypeError::Mismatch {
                        expected: "bool".to_string(),
                        found: cond_ty.to_string(),
                        span: SourceSpan::new(0.into(), 0),
                    }
                    .into(),
                );
            }
        }

        Iter::Call(_) | Iter::Ever => {
            // Call-based or infinite loop - no type checking needed for iterator
            let _ = infer_expr(ctx, &for_stmt.range);
        }
    }

    // Check loop body
    check_body(ctx, &for_stmt.body)?;

    // Pop loop scope
    ctx.pop_scope();

    Ok(Type::Void)
}

/// Check a return statement
///
/// Validates that the return value matches the current function's return type.
///
/// # Arguments
///
/// * `ctx` - Type inference context
/// * `expr` - Expression being returned
///
/// # Returns
///
/// Type::Void (return statements don't produce values)
fn check_return(ctx: &mut InferenceContext, expr: &Expr) -> Result<Type, AutoError> {
    let return_ty = infer_expr(ctx, expr);

    // Check against expected return type
    if let Some(expected_ret) = &ctx.current_ret {
        match ctx.unify(expected_ret.clone(), return_ty.clone()) {
            Ok(_) => {}
            Err(e) => {
                ctx.errors.push(e.into());
            }
        }
    }

    Ok(Type::Void)
}

/// Check a block/body of statements
///
/// Checks each statement in sequence and returns the type of the last expression
/// (or Type::Void if the block doesn't end with an expression).
///
/// # Arguments
///
/// * `ctx` - Type inference context
/// * `body` - Block to check
///
/// # Returns
///
/// The type of the last expression, or Type::Void
pub fn check_body(ctx: &mut InferenceContext, body: &Body) -> Result<Type, AutoError> {
    let mut last_ty = Type::Void;

    for stmt in &body.stmts {
        last_ty = check_stmt(ctx, stmt)?;
    }

    Ok(last_ty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Name, StoreKind};

    #[test]
    fn test_check_let_with_type() {
        let mut ctx = InferenceContext::new();
        let store = Store {
            kind: StoreKind::Let,
            name: Name::from("x"),
            ty: Type::Int,
            expr: Expr::Int(42),
        };

        let result = check_store(&mut ctx, &store);
        assert!(result.is_ok());
        assert!(matches!(ctx.lookup_type(&Name::from("x")), Some(Type::Int)));
    }

    #[test]
    fn test_check_let_inference() {
        let mut ctx = InferenceContext::new();
        let store = Store {
            kind: StoreKind::Let,
            name: Name::from("y"),
            ty: Type::Unknown,
            expr: Expr::Bool(true),
        };

        let result = check_store(&mut ctx, &store);
        assert!(result.is_ok());
        assert!(matches!(
            ctx.lookup_type(&Name::from("y")),
            Some(Type::Bool)
        ));
    }

    #[test]
    fn test_check_let_type_mismatch() {
        let mut ctx = InferenceContext::new();
        let store = Store {
            kind: StoreKind::Let,
            name: Name::from("z"),
            ty: Type::Int,
            expr: Expr::Bool(false),
        };

        let _result = check_store(&mut ctx, &store);
        // Should add error to context
        assert!(!ctx.errors.is_empty());
    }

    #[test]
    fn test_check_expr_stmt() {
        let mut ctx = InferenceContext::new();
        let stmt = Stmt::Expr(Expr::Int(100));

        let result = check_stmt(&mut ctx, &stmt);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Type::Int));
    }

    #[test]
    fn test_check_return() {
        let mut ctx = InferenceContext::new();
        ctx.current_ret = Some(Type::Int);

        let result = check_return(&mut ctx, &Expr::Int(42));
        assert!(result.is_ok());
        assert!(ctx.errors.is_empty());
    }

    #[test]
    fn test_check_return_mismatch() {
        let mut ctx = InferenceContext::new();
        ctx.current_ret = Some(Type::Int);

        let _result = check_return(&mut ctx, &Expr::Bool(true));
        // Should add error to context
        assert!(!ctx.errors.is_empty());
    }

    #[test]
    fn test_check_body_empty() {
        let mut ctx = InferenceContext::new();
        let body = Body::new();

        let result = check_body(&mut ctx, &body);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Type::Void));
    }

    #[test]
    fn test_check_body_with_expr() {
        let mut ctx = InferenceContext::new();
        let mut body = Body::new();
        body.stmts.push(Stmt::Expr(Expr::Int(42)));

        let result = check_body(&mut ctx, &body);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Type::Int));
    }
}
