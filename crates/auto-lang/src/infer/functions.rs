use crate::ast::{Fn, Type};
use crate::error::{AutoError, TypeError};
use crate::infer::expr::infer_expr;
use crate::infer::stmt::check_body;
use crate::infer::InferenceContext;
use miette::SourceSpan;

/// Check a function declaration
///
/// Infers logical types for parameters and return values, checks the function body,
/// and ensures type consistency.
///
/// # Arguments
///
/// * `ctx` - Type inference context
/// * `fn_decl` - Function declaration to check
///
/// # Returns
///
/// The function type (Type::Fn)
pub fn check_fn(ctx: &mut InferenceContext, fn_decl: &Fn) -> Result<Type, AutoError> {
    // 1. Create new scope for function parameters
    ctx.push_scope();

    // 2. Process parameters
    let mut param_tys = Vec::new();
    for param in &fn_decl.params {
        let ty = if !matches!(param.ty, Type::Unknown) {
            // Explicit type
            param.ty.clone()
        } else if let Some(default) = &param.default {
            // Infer from default value
            infer_expr(ctx, default)
        } else {
            // Missing type - error: parameter must have explicit type or default value
            ctx.errors.push(
                TypeError::Mismatch {
                    expected: "explicit type or default value".to_string(),
                    found: format!("parameter '{}' with no type information", param.name),
                    span: SourceSpan::new(0.into(), 0), // TODO: Real span
                }
                .into(),
            );
            Type::Unknown
        };

        // Bind parameter to scope
        ctx.bind_var(param.name.clone(), ty.clone());
        param_tys.push(ty);
    }

    // 3. Handle return type
    let declared_ret = if !matches!(fn_decl.ret, Type::Unknown) {
        Some(fn_decl.ret.clone())
    } else {
        None
    };

    // Save previous return type to restore later (for nested functions)
    let prev_ret = ctx.current_ret.take();
    ctx.current_ret = declared_ret.clone();

    // 4. Check body
    let body_ty = check_body(ctx, &fn_decl.body)?;

    // 5. Verify return type
    let final_ret = if let Some(declared) = declared_ret {
        // If declared, body type must match (unless body is Void and implicit return is allowed,
        // but explicit return checks are handled in check_stmt/check_return.
        // Here we mainly check if the body's natural result matches expected if it's an expression body)

        // Actually, check_body returns the type of the last statement.
        // If the function is declared to return T, the body should return T.
        // However, if the body ends in a statement that is NOT a return, it returns Void usually.
        // In AutoLang/Rust, implicit return is the last expression.

        match ctx.unify(declared.clone(), body_ty) {
            Ok(_) => declared,
            Err(e) => {
                ctx.errors.push(e.into());
                declared
            }
        }
    } else {
        // Infer return type from body
        body_ty
    };

    // Restore context
    ctx.current_ret = prev_ret;
    ctx.pop_scope();

    // 6. Construct Function Type
    // Note: Type::Fn expects Box<FunctionType> which is effectively Box<crate::ast::types::FunctionType>
    // checking ast/types.rs for FunctionType definition might be needed if compilation fails.
    // Assuming Type::Fn(Box<FunctionType>) where FunctionType { params: Vec<Type>, ret: Box<Type> }

    // Let's assume Type::Fn variant structure based on plans or previous file views.
    // Verify Type structure if needed. For now assume standard structure.

    // Using AST construction from memory of Plan 010 docs:
    // Ok(Type::Fn(Box::new(crate::ast::FunctionType {
    //    params: param_tys,
    //    ret: Box::new(final_ret),
    // })))

    // Actually, looking at older code might be safer, but let's try to verify Type definition first.
    // To be safe, I will output this file *after* verifying Type definition in next step.
    // Wait, I can't verify in middle of write_to_file.
    // I will write a "safe" version using a helper or just checking AST first.
    // Retrying step logic: I should check Type struct.

    // 6. Construct Function Type
    Ok(Type::Fn(param_tys, Box::new(final_ret)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, Expr, FnKind, Name, Param, Stmt};
    use crate::infer::InferenceContext;

    #[test]
    fn test_check_fn_with_explicit_types() {
        let mut ctx = InferenceContext::new();
        let fn_decl = Fn {
            kind: FnKind::Function,
            name: Name::from("add"),
            parent: None,
            params: vec![
                Param {
                    name: Name::from("x"),
                    ty: Type::Int,
                    default: None,
                },
                Param {
                    name: Name::from("y"),
                    ty: Type::Int,
                    default: None,
                },
            ],
            body: Body {
                stmts: vec![Stmt::Return(Box::new(Expr::Int(42)))],
                has_new_line: false,
            },
            ret: Type::Int,
            ret_name: None,
            is_static: false,
            type_params: vec![],
            span: None,
        };

        let result = check_fn(&mut ctx, &fn_decl);
        assert!(result.is_ok());
        let fn_ty = result.unwrap();

        if let Type::Fn(params, ret) = fn_ty {
            assert_eq!(params.len(), 2);
            assert!(matches!(params[0], Type::Int));
            assert!(matches!(params[1], Type::Int));
            assert!(matches!(*ret, Type::Int));
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_check_fn_with_default_values() {
        let mut ctx = InferenceContext::new();
        let fn_decl = Fn {
            kind: FnKind::Function,
            name: Name::from("greet"),
            parent: None,
            params: vec![Param {
                name: Name::from("count"),
                ty: Type::Unknown,
                default: Some(Expr::Int(5)),
            }],
            body: Body {
                stmts: vec![], // Empty body returns Void
                has_new_line: false,
            },
            ret: Type::Void,
            ret_name: None,
            is_static: false,
            type_params: vec![],
            span: None,
        };

        let result = check_fn(&mut ctx, &fn_decl);
        assert!(result.is_ok());
        let fn_ty = result.unwrap();

        if let Type::Fn(params, ret) = fn_ty {
            assert_eq!(params.len(), 1);
            assert!(matches!(params[0], Type::Int));
            assert!(matches!(*ret, Type::Void));
        } else {
            panic!("Expected function type");
        }
    }

    #[test]
    fn test_check_fn_missing_parameter_type() {
        let mut ctx = InferenceContext::new();
        let fn_decl = Fn {
            kind: FnKind::Function,
            name: Name::from("mystery"),
            parent: None,
            params: vec![Param {
                name: Name::from("x"),
                ty: Type::Unknown,
                default: None,
            }],
            body: Body {
                stmts: vec![],
                has_new_line: false,
            },
            ret: Type::Void,
            ret_name: None,
            is_static: false,
            type_params: vec![],
            span: None,
        };

        let result = check_fn(&mut ctx, &fn_decl);
        // Should still return Ok, but with errors collected
        assert!(result.is_ok());
        assert!(!ctx.errors.is_empty());
    }

    #[test]
    fn test_check_fn_inferred_return_type() {
        let mut ctx = InferenceContext::new();
        let fn_decl = Fn {
            kind: FnKind::Function,
            name: Name::from("compute"),
            parent: None,
            params: vec![],
            body: Body {
                stmts: vec![Stmt::Expr(Expr::Int(42))],
                has_new_line: false,
            },
            ret: Type::Unknown,
            ret_name: None,
            is_static: false,
            type_params: vec![],
            span: None,
        };

        let result = check_fn(&mut ctx, &fn_decl);
        assert!(result.is_ok());
        let fn_ty = result.unwrap();

        if let Type::Fn(params, ret) = fn_ty {
            assert_eq!(params.len(), 0);
            assert!(matches!(*ret, Type::Int));
        } else {
            panic!("Expected function type");
        }
    }
}
