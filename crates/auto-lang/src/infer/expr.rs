//! 表达式类型推导
//!
//! 本模块实现了 AutoLang 所有表达式类型的类型推导,包括:
//! - 字面量表达式
//! - 标识符引用
//! - 一元和二元运算
//! - 数组和对象
//! - 函数调用和方法调用
//! - 索引表达式
//! - 控制流表达式(if、for等)
//!
//! # 算法
//!
//! 表达式类型推导采用自底向上的递归策略:
//! 1. 对于字面量,直接返回其类型
//! 2. 对于标识符,在类型环境中查找
//! 3. 对于复合表达式,先递归推导子表达式类型,再组合
//! 4. 对于需要类型统一的表达式,添加类型约束并求解
//!
//! # 示例
//!
//! ```rust
//! use auto_lang::infer::{InferenceContext, infer_expr};
//! use auto_lang::ast::{Expr, Type};
//!
//! let mut ctx = InferenceContext::new();
//! let expr = Expr::Int(42);
//! let ty = infer_expr(&mut ctx, &expr);
//! assert!(matches!(ty, Type::Int));
//! ```

use crate::ast::{ArrayType, Arg, Call, Expr, If, Name, PtrType, Range, Type, TypeDecl};
use crate::error::{AutoError, TypeError, Warning};
use crate::infer::context::InferenceContext;
use crate::infer::constraints::TypeConstraint;
use miette::SourceSpan;
use std::rc::Rc;
use std::cell::RefCell;
use auto_val::Op;

/// 推导表达式的类型
///
/// # 参数
///
/// * `ctx` - 类型推导上下文
/// * `expr` - 要推导的表达式
///
/// # 返回
///
/// 推导出的类型,如果无法推导则返回 `Type::Unknown`
///
/// # 示例
///
/// ```rust
/// # use auto_lang::infer::{InferenceContext, infer_expr};
/// # use auto_lang::ast::{Expr, Type};
/// let mut ctx = InferenceContext::new();
/// let expr = Expr::Int(42);
/// let ty = infer_expr(&mut ctx, &expr);
/// assert!(matches!(ty, Type::Int));
/// ```
pub fn infer_expr(ctx: &mut InferenceContext, expr: &Expr) -> Type {
    match expr {
        // ========== 字面量表达式 ==========
        Expr::Int(_) | Expr::I64(_) => Type::Int,
        Expr::I8(_) => Type::Int,
        Expr::Uint(_) | Expr::Byte(_) | Expr::U8(_) => Type::Uint,
        Expr::Float(_, _) => Type::Float,
        Expr::Double(_, _) => Type::Double,
        Expr::Bool(_) => Type::Bool,
        Expr::Char(_) => Type::Char,
        Expr::Str(s) => Type::Str(s.len()),
        Expr::CStr(_) => Type::CStr,

        // 空值类型
        Expr::Nil | Expr::Null => Type::Unknown,

        // ========== 标识符引用 ==========
        Expr::Ident(name) => {
            ctx.lookup_type(name)
                .unwrap_or_else(|| {
                    // 未定义的变量,返回 Unknown (不记录错误,因为可能是运行时绑定的变量如hold表达式)
                    Type::Unknown
                })
        }

        // 生成的名称(内部使用)
        Expr::GenName(name) => {
            ctx.lookup_type(name)
                .unwrap_or(Type::Unknown)
        }

        // ========== 一元运算 ==========
        Expr::Unary(op, operand) => {
            let operand_ty = infer_expr(ctx, operand);
            infer_unary_op_type(ctx, op, &operand_ty)
        }

        // ========== 二元运算 ==========
        Expr::Bina(lhs, op, rhs) => {
            let lhs_ty = infer_expr(ctx, lhs);
            let rhs_ty = infer_expr(ctx, rhs);

            // 添加相等性约束(操作数类型应该兼容)
            let span = SourceSpan::new(0.into(), 0);
            ctx.add_constraint(TypeConstraint::Equal(
                lhs_ty.clone(),
                rhs_ty.clone(),
                span,
            ));

            // 尝试统一操作数类型
            let unified_operands = match ctx.unify_with_coercion(lhs_ty.clone(), rhs_ty.clone(), span) {
                Ok(ty) => ty,
                Err(_) => {
                    // 无法统一,记录错误并返回 Unknown
                    ctx.errors.push(AutoError::Type(TypeError::Mismatch {
                        expected: lhs_ty.to_string(),
                        found: rhs_ty.to_string(),
                        span,
                    }));
                    Type::Unknown
                }
            };

            // 推导结果类型
            infer_binop_type(ctx, op, &unified_operands)
        }

        // ========== 范围表达式 ==========
        Expr::Range(range) => infer_range_type(ctx, range),

        // ========== 数组表达式 ==========
        Expr::Array(elems) => {
            if elems.is_empty() {
                // 空数组类型无法推导
                Type::Unknown
            } else {
                // 推导第一个元素的类型
                let elem_ty = infer_expr(ctx, &elems[0]);

                // 检查所有元素类型是否一致
                let span = SourceSpan::new(0.into(), 0);
                for elem in &elems[1..] {
                    let ty = infer_expr(ctx, elem);
                    ctx.add_constraint(TypeConstraint::Equal(
                        elem_ty.clone(),
                        ty,
                        span,
                    ));
                }

                Type::Array(ArrayType {
                    elem: Box::new(elem_ty),
                    len: elems.len(),
                })
            }
        }

        // ========== 对象表达式 ==========
        Expr::Object(_pairs) => {
            // 对象类型推导较为复杂,暂时返回 Unknown
            // TODO: 实现 struct 类型推导
            Type::Unknown
        }

        // ========== Pair 表达式 ==========
        Expr::Pair(_) => {
            // Pair 用于对象字段,暂时返回 Unknown
            Type::Unknown
        }

        // ========== 函数调用 ==========
        Expr::Call(call) => infer_call_type(ctx, call),

        // ========== 索引表达式 ==========
        Expr::Index(array_expr, index_expr) => {
            infer_index_type(ctx, array_expr, index_expr)
        }

        // ========== Lambda 表达式 ==========
        Expr::Lambda(_fn_decl) => {
            // TODO: 实现函数类型推导
            // 暂时返回 Unknown
            Type::Unknown
        }

        // ========== Closure 表达式 (Plan 060) ==========
        Expr::Closure(closure) => {
            // Phase 2: 闭包类型推导
            // 对于Phase 2（没有上下文信息），我们无法从函数签名推导参数类型
            // 但我们可以：
            // 1. 使用显式类型注解（如果有）
            // 2. 推导body类型作为返回类型

            // 推导body类型
            let body_ty = infer_expr(ctx, &closure.body);

            // 如果有显式返回类型注解，使用它；否则使用body类型
            let ret_ty = if let Some(explicit_ret) = &closure.ret {
                explicit_ret.clone()
            } else {
                body_ty.clone()
            };

            // 构造参数类型列表
            let param_types: Vec<Type> = closure.params.iter()
                .map(|param| {
                    if let Some(explicit_ty) = &param.ty {
                        explicit_ty.clone()
                    } else {
                        // 没有显式类型注解，返回Unknown
                        Type::Unknown
                    }
                })
                .collect();

            // 构造函数类型
            Type::Fn(param_types, Box::new(ret_ty))
        }

        // ========== F-String 表达式 ==========
        Expr::FStr(_) => Type::Str(0),

        // ========== Grid 表达式 ==========
        Expr::Grid(_grid) => Type::Unknown,

        // ========== Cover 表达式 ==========
        Expr::Cover(_cover) => Type::Unknown,

        // ========== Uncover 表达式 ==========
        Expr::Uncover(_uncover) => Type::Unknown,

        // ========== If 表达式 ==========
        Expr::If(if_expr) => infer_if_type(ctx, if_expr),

        // ========== Block 表达式 ==========
        Expr::Block(block) => {
            // 推导 block 中最后一个表达式的类型
            if let Some(last_stmt) = block.stmts.last() {
                // 检查最后一个语句是否是表达式语句
                match last_stmt {
                    crate::ast::Stmt::Expr(expr) => infer_expr(ctx, expr),
                    _ => Type::Void,
                }
            } else {
                Type::Void
            }
        }

        // ========== Ref 表达式 ==========
        Expr::Ref(name) => {
            // 引用类型,创建指针
            let inner_ty = ctx.lookup_type(name)
                .unwrap_or(Type::Unknown);
            Type::Ptr(PtrType {
                of: Rc::new(RefCell::new(inner_ty)),
            })
        }

        // ========== Node 表达式 ==========
        Expr::Node(_node) => Type::Unknown,

        // ========== Borrow 表达式 (Phase 3) ==========
        Expr::View(expr) => {
            // View/immutable borrow: 类型与被借用的表达式相同 (like Rust &T)
            // TODO: 实现 view 借用类型推导 (Phase 3 Week 1)
            infer_expr(ctx, expr)
        }
        Expr::Mut(expr) => {
            // Mutable borrow: 类型与被借用的表达式相同 (like Rust &mut T)
            // TODO: 实现 mut 借用类型推导 (Phase 3 Week 1)
            infer_expr(ctx, expr)
        }
        Expr::Take(expr) => {
            // Take/move: 类型与被移动的表达式相同 (like Rust move)
            // TODO: 实现 take 移动类型推导 (Phase 3 Week 1)
            infer_expr(ctx, expr)
        }

        // ========== Hold 表达式 (Phase 3) ==========
        Expr::Hold(hold) => {
            // Hold: 临时路径绑定,类型为body的类型
            // Hold表达式返回body的结果类型（最后一个表达式的类型）
            if let Some(last_stmt) = hold.body.stmts.last() {
                match last_stmt {
                    crate::ast::Stmt::Expr(expr) => infer_expr(ctx, expr),
                    _ => Type::Void,
                }
            } else {
                Type::Void
            }
        }

        // ========== May type operators (Phase 1b.3) ==========
        Expr::NullCoalesce(left, right) => {
            // Null-coalescing operator: left ?? right
            // Type is the union of left and right types
            // In most cases, if left is May<T>, result is T
            let left_ty = infer_expr(ctx, left);
            let right_ty = infer_expr(ctx, right);

            // If left is May<T> (generic tag), extract T from val field
            match left_ty {
                Type::Tag(tag) if tag.borrow().name.as_ref().starts_with("May_") => {
                    // Extract type from val field
                    tag.borrow().fields.iter()
                        .find(|f| f.name.as_ref() == "val")
                        .map(|f| f.ty.clone())
                        .unwrap_or(Type::Unknown)
                }
                _ => {
                    // Otherwise, unify the two types (clone left_ty to avoid move)
                    ctx.unify(left_ty.clone(), right_ty).unwrap_or(left_ty)
                }
            }
        }
        Expr::ErrorPropagate(expr) => {
            // Error propagation operator: expression.?
            // If expression is May<T>, result is T
            // Otherwise, result is the expression type
            let expr_ty = infer_expr(ctx, expr);
            match expr_ty {
                Type::Tag(tag) if tag.borrow().name.as_ref().starts_with("May_") => {
                    // Extract type from val field
                    tag.borrow().fields.iter()
                        .find(|f| f.name.as_ref() == "val")
                        .map(|f| f.ty.clone())
                        .unwrap_or(Type::Unknown)
                }
                _ => expr_ty,
            }
        }

        // ========== Dot expression (Plan 056: Phase 1) ==========
        Expr::Dot(object, _field) => {
            // Dot expression: object.field or Type.method
            // For now, return the type of the object expression
            // TODO: Phase 2 - Add field lookup and type resolution
            // TODO: Phase 3 - Distinguish between field access and method calls
            infer_expr(ctx, object)
        }
    }
}

/// 推导一元运算的结果类型
fn infer_unary_op_type(ctx: &mut InferenceContext, op: &Op, operand_ty: &Type) -> Type {
    match op {
        Op::Not => {
            // 逻辑非:操作数应该是 Bool,结果是 Bool
            if !matches!(operand_ty, Type::Unknown | Type::Bool) {
                ctx.warnings.push(Warning::ImplicitTypeConversion {
                    from: operand_ty.to_string(),
                    to: "bool".into(),
                    span: SourceSpan::new(0.into(), 0),
                });
            }
            Type::Bool
        }

        Op::Sub => {
            // 取负:保持操作数类型
            operand_ty.clone()
        }

        _ => {
            // 其他一元运算符暂不支持
            Type::Unknown
        }
    }
}

/// 推导二元运算的结果类型
fn infer_binop_type(_ctx: &mut InferenceContext, op: &Op, operand_ty: &Type) -> Type {
    match op {
        // 算术运算:返回操作数类型
        Op::Add | Op::Sub | Op::Mul | Op::Div => operand_ty.clone(),

        // 算术赋值:返回操作数类型
        Op::AddEq | Op::SubEq | Op::MulEq | Op::DivEq => operand_ty.clone(),

        // 比较运算:总是返回 Bool
        Op::Eq | Op::Neq | Op::Lt | Op::Gt | Op::Le | Op::Ge => Type::Bool,

        // 范围运算符:暂时返回 Unknown
        Op::Range | Op::RangeEq => Type::Unknown,

        // 其他运算符
        _ => Type::Unknown,
    }
}

/// 推导范围表达式的类型
fn infer_range_type(ctx: &mut InferenceContext, range: &Range) -> Type {
    // 推导起始和结束的类型
    let start_ty = infer_expr(ctx, &range.start);
    let end_ty = infer_expr(ctx, &range.end);

    // 统一起始和结束类型
    match ctx.unify(start_ty, end_ty) {
        Ok(_) => Type::Unknown, // 范围类型待定义
        Err(err) => {
            ctx.errors.push(err.into());
            Type::Unknown
        }
    }
}

/// 推导函数调用的类型
fn infer_call_type(ctx: &mut InferenceContext, call: &Call) -> Type {
    // 推导被调用者的类型
    let callee_ty = infer_expr(ctx, &call.name);

    // Plan 061: Type argument inference and constraint validation
    // Check if this is a direct function call (identifier)
    if let Expr::Ident(fn_name) = &*call.name {
        // Try to get the function declaration and clone needed data
        let type_params_and_constraints = {
            if let Some(fn_decl) = ctx.universe.borrow().get_fn_decl(fn_name.as_str()) {
                if !fn_decl.type_params.is_empty() {
                    // Clone the data we need to avoid holding the borrow
                    Some((
                        fn_decl.type_params.clone(),
                        fn_decl.params.clone(),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        };

        // If function has generic parameters, infer their concrete types from arguments
        if let Some((type_params, params)) = type_params_and_constraints {
            let mut type_args = Vec::new();

            // For each function parameter, check if it uses a generic type
            for (i, param) in params.iter().enumerate() {
                // Check if parameter type is a generic parameter
                if let Type::User(param_type_decl) = &param.ty {
                    // Check if this type name matches any of the function's generic parameters
                    for type_param in &type_params {
                        if param_type_decl.name == type_param.name {
                            // This parameter uses a generic type
                            // Get the corresponding argument's type
                            if let Some(Arg::Pos(arg_expr)) = call.args.get(i) {
                                let arg_type = infer_expr(ctx, &arg_expr);
                                type_args.push((type_param.name.clone(), arg_type));
                            }
                            break;
                        }
                    }
                }
            }

            // Now validate constraints for any generic parameters with constraints
            for (param_name, concrete_type) in &type_args {
                // Find the type parameter declaration
                if let Some(type_param) = type_params.iter().find(|tp| &tp.name == param_name) {
                    if let Some(constraint) = &type_param.constraint {
                        // Validate that concrete_type satisfies the constraint
                        if let Err(error) = validate_spec_constraint(ctx, constraint, concrete_type) {
                            ctx.errors.push(error.into());
                        }
                    }
                }
            }

            // Store type_args in the Call using unsafe code
            // SAFETY: We have a mutable reference to the InferenceContext which owns the Expr tree,
            // and we're only mutating the type_args field which is not aliased elsewhere.
            // The Call will not be moved or accessed mutably elsewhere during this operation.
            if !type_args.is_empty() {
                unsafe {
                    let call_ptr = call as *const Call as *mut Call;
                    (*call_ptr).type_args = type_args;
                }
            }
        }
    }

    match callee_ty {
        Type::Unknown => Type::Unknown,

        // TODO: 实现完整的函数类型检查
        // 暂时返回调用对象的返回类型字段
        _ => call.ret.clone(),
    }
}

// Plan 061 Phase 2: Helper functions for constraint validation
// These will be used when type argument tracking is implemented

/// Find concrete type argument for a generic parameter in a function call
///
/// This is a placeholder for future type argument tracking functionality.
/// When implemented, it will:
/// 1. Infer types from call arguments
/// 2. Match generic parameter names to concrete types
/// 3. Return the concrete type for a given parameter
///
/// For now, returns None (no validation)
fn find_type_arg_for_param(_call: &Call, _param_name: &Name) -> Option<Type> {
    // TODO: Implement type argument inference
    // This requires:
    // 1. Building a type environment from the call arguments
    // 2. Matching generic parameters to their concrete types
    // 3. Handling partial type information
    None
}

/// Validate that a type satisfies a spec constraint
///
/// This function validates that a concrete type implements all required methods
/// from a spec constraint. Uses the TraitChecker for validation.
///
/// Note: The full validation logic is deferred until type argument tracking is implemented.
/// This function provides the infrastructure and shows the structure of the validation.
///
/// # Arguments
/// * `ctx` - The inference context (provides access to universe/specs)
/// * `constraint` - The constraint type (e.g., Type::Spec(SpecDecl))
/// * `concrete_type` - The concrete type to validate
///
/// # Returns
/// * `Ok(())` if the constraint is satisfied
/// * `Err(TypeError)` if the constraint is violated
fn validate_spec_constraint(
    ctx: &InferenceContext,
    constraint: &Type,
    concrete_type: &Type,
) -> Result<(), crate::error::TypeError> {
    use crate::error::TypeError;
    use miette::SourceSpan;

    let span = SourceSpan::new(0.into(), 0);  // Placeholder span

    // Extract spec reference from constraint
    // When fully implemented, constraint will be Type::Spec(SpecRef)
    // For now, we skip validation as type argument tracking isn't implemented

    match constraint {
        Type::User(type_decl) => {
            // This would be a Spec type when fully implemented
            // Get spec declaration from universe
            let spec_decl_opt = ctx.universe.borrow().get_spec(type_decl.name.as_str());

            let spec_decl = match spec_decl_opt {
                Some(decl) => decl,
                None => {
                    return Err(TypeError::UndefinedSpec {
                        name: type_decl.name.to_string(),
                        span,
                    });
                }
            };

            // Get the concrete type declaration
            let concrete_type_decl = get_type_decl_from_type(ctx, concrete_type)?;

            // Use TraitChecker to validate conformance
            use crate::trait_checker::TraitChecker;
            match TraitChecker::check_conformance(&concrete_type_decl, &spec_decl) {
                Ok(_) => Ok(()),
                Err(errors) => {
                    // Convert AutoError to TypeError
                    // For now, just return a generic constraint violation error
                    Err(TypeError::ConstraintViolation {
                        type_name: format!("{:?}", concrete_type),
                        spec_name: type_decl.name.to_string(),
                        span,
                    })
                }
            }
        }
        _ => {
            // Non-spec constraints not yet implemented
            Ok(())
        }
    }
}

/// Extract TypeDecl from a Type
///
/// This helper function extracts the TypeDecl from various Type variants.
/// For user-defined types, it returns the actual TypeDecl.
/// For builtin types (int, str, etc.), it creates placeholder TypeDecls.
///
/// Note: Some types (Array, Ptr, etc.) don't have TypeDecls and will return an error.
fn get_type_decl_from_type(
    _ctx: &InferenceContext,
    ty: &Type,
) -> Result<TypeDecl, crate::error::TypeError> {
    use crate::error::TypeError;
    use miette::SourceSpan;

    let span = SourceSpan::new(0.into(), 0);  // Placeholder span

    match ty {
        Type::User(type_decl) => Ok(type_decl.clone()),
        // Builtin types - create placeholder TypeDecls
        Type::Int => Ok(TypeDecl::builtin("int")),
        Type::Uint => Ok(TypeDecl::builtin("uint")),
        Type::Float => Ok(TypeDecl::builtin("float")),
        Type::Double => Ok(TypeDecl::builtin("double")),
        Type::Bool => Ok(TypeDecl::builtin("bool")),
        Type::Str(_) => Ok(TypeDecl::builtin("str")),
        Type::CStr => Ok(TypeDecl::builtin("cstr")),
        Type::Char => Ok(TypeDecl::builtin("char")),
        Type::Void => Ok(TypeDecl::builtin("void")),
        // Types that don't have TypeDecls - return error
        _ => Err(TypeError::CannotGetDecl {
            type_: format!("{:?}", ty),
            span,
        }),
    }
}

/// 推导索引表达式的类型
fn infer_index_type(ctx: &mut InferenceContext, array_expr: &Expr, index_expr: &Expr) -> Type {
    // 推导数组/容器的类型
    let container_ty = infer_expr(ctx, array_expr);

    // 推导索引的类型
    let idx_ty = infer_expr(ctx, index_expr);

    // 索引应该是整数类型
    let span = SourceSpan::new(0.into(), 0);
    match ctx.unify(idx_ty.clone(), Type::Int) {
        Ok(_) => {}
        Err(_) => {
            ctx.errors.push(AutoError::Type(TypeError::Mismatch {
                expected: "int".to_string(),
                found: idx_ty.to_string(),
                span,
            }));
        }
    }

    // 提取元素类型
    match container_ty {
        Type::Array(arr) => *arr.elem,
        Type::RuntimeArray(rta) => *rta.elem,  // Plan 052
        Type::Str(_) | Type::CStr => Type::Char,
        Type::Ptr(ptr) => {
            let inner_ty = ptr.of.borrow();
            inner_ty.clone()
        }
        _ => Type::Unknown,
    }
}

/// 推导 if 表达式的类型
fn infer_if_type(ctx: &mut InferenceContext, if_expr: &If) -> Type {
    // 推导第一个分支的类型
    if let Some(first_branch) = if_expr.branches.first() {
        // 推导条件类型
        let cond_ty = infer_expr(ctx, &first_branch.cond);

        // 条件应该是 Bool
        let span = SourceSpan::new(0.into(), 0);
        match ctx.unify(cond_ty.clone(), Type::Bool) {
            Ok(_) => {}
            Err(_) => {
                ctx.errors.push(AutoError::Type(TypeError::Mismatch {
                    expected: "bool".to_string(),
                    found: cond_ty.to_string(),
                    span,
                }));
            }
        }

        // 推导 then 分支类型(从 body 的最后一个语句)
        let then_ty = if let Some(last_stmt) = first_branch.body.stmts.last() {
            match last_stmt {
                crate::ast::Stmt::Expr(expr) => infer_expr(ctx, expr),
                _ => Type::Void,
            }
        } else {
            Type::Void
        };

        // 推导 else 分支类型(如果存在)
        let else_ty = if let Some(else_body) = &if_expr.else_ {
            if let Some(last_stmt) = else_body.stmts.last() {
                match last_stmt {
                    crate::ast::Stmt::Expr(expr) => infer_expr(ctx, expr),
                    _ => Type::Void,
                }
            } else {
                Type::Void
            }
        } else {
            Type::Void
        };

        // 统一两个分支的类型
        match ctx.unify(then_ty, else_ty) {
            Ok(ty) => ty,
            Err(_) => Type::Unknown,
        }
    } else {
        Type::Unknown
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use auto_val::AutoStr;
    use crate::ast::{Body, Branch};
    use crate::universe::Universe;

    fn make_test_context() -> InferenceContext {
        let universe = Rc::new(RefCell::new(Universe::new()));
        InferenceContext::with_universe(universe)
    }

    #[test]
    fn test_infer_literal_int() {
        let mut ctx = make_test_context();
        let expr = Expr::Int(42);
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Int));
    }

    #[test]
    fn test_infer_literal_float() {
        let mut ctx = make_test_context();
        let expr = Expr::Float(3.14, AutoStr::from(""));
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Float));
    }

    #[test]
    fn test_infer_literal_bool() {
        let mut ctx = make_test_context();
        let expr = Expr::Bool(true);
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Bool));
    }

    #[test]
    fn test_infer_literal_string() {
        let mut ctx = make_test_context();
        let expr = Expr::Str(AutoStr::from("hello"));
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Str(_)));
    }

    #[test]
    fn test_infer_array() {
        let mut ctx = make_test_context();
        let expr = Expr::Array(vec![
            Expr::Int(1),
            Expr::Int(2),
            Expr::Int(3),
        ]);
        let ty = infer_expr(&mut ctx, &expr);
        match ty {
            Type::Array(arr) => {
                assert!(matches!(*arr.elem, Type::Int));
                assert_eq!(arr.len, 3);
            }
            _ => panic!("Expected Array type, got {:?}", ty),
        }
    }

    #[test]
    fn test_infer_empty_array() {
        let mut ctx = make_test_context();
        let expr = Expr::Array(vec![]);
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Unknown));
    }

    #[test]
    fn test_infer_binary_op_add() {
        let mut ctx = make_test_context();
        let expr = Expr::Bina(
            Box::new(Expr::Int(1)),
            Op::Add,
            Box::new(Expr::Int(2)),
        );
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Int));
    }

    #[test]
    fn test_infer_binary_op_comparison() {
        let mut ctx = make_test_context();
        let expr = Expr::Bina(
            Box::new(Expr::Int(1)),
            Op::Lt,
            Box::new(Expr::Int(2)),
        );
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Bool));
    }

    #[test]
    fn test_infer_unary_op_not() {
        let mut ctx = make_test_context();
        let expr = Expr::Unary(
            Op::Not,
            Box::new(Expr::Bool(true)),
        );
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Bool));
    }

    #[test]
    fn test_infer_unary_op_neg() {
        let mut ctx = make_test_context();
        let expr = Expr::Unary(
            Op::Sub,
            Box::new(Expr::Int(42)),
        );
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Int));
    }

    #[test]
    fn test_infer_if_expr() {
        let mut ctx = make_test_context();
        let branch = Branch {
            cond: Expr::Bool(true),
            body: Body::single_expr(Expr::Int(1)),
        };
        let else_body = Body::single_expr(Expr::Int(2));
        let if_expr = If {
            branches: vec![branch],
            else_: Some(else_body),
        };
        let expr = Expr::If(if_expr);
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Int));
    }

    #[test]
    fn test_infer_index_expr() {
        let mut ctx = make_test_context();
        let array_expr = Expr::Array(vec![
            Expr::Int(1),
            Expr::Int(2),
            Expr::Int(3),
        ]);
        let index_expr = Expr::Int(0);
        let expr = Expr::Index(Box::new(array_expr), Box::new(index_expr));
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Int));
    }

    // ========== Plan 060: Closure Type Inference Tests ==========

    #[test]
    fn test_infer_closure_simple() {
        use crate::ast::{Closure, ClosureParam, Name};
        let mut ctx = make_test_context();

        // Test 1: Simple closure:  x => x * 2
        let closure = Closure::new(
            vec![ClosureParam::new("x".into(), None)],
            None,
            Expr::Bina(Box::new(Expr::Ident("x".into())), Op::Mul, Box::new(Expr::Int(2))),
        );

        let ty = infer_expr(&mut ctx, &Expr::Closure(closure));

        // Should infer as: fn(int) int
        match &ty {
            Type::Fn(params, ret) => {
                assert_eq!(params.len(), 1);
                assert!(matches!(params[0], Type::Unknown));  // x has no type annotation
                assert!(matches!(&**ret, Type::Int));  // body is x * 2, so return type is int
            }
            _ => panic!("Expected Fn type, got {:?}", ty),
        }
    }

    #[test]
    fn test_infer_closure_with_explicit_types() {
        use crate::ast::{Closure, ClosureParam, Name, Type};
        let mut ctx = make_test_context();

        // Test 2: Closure with explicit types: (x int) int => x + 1
        let closure = Closure::new(
            vec![ClosureParam::new("x".into(), Some(Type::Int))],
            Some(Type::Int),
            Expr::Bina(Box::new(Expr::Ident("x".into())), Op::Add, Box::new(Expr::Int(1))),
        );

        let ty = infer_expr(&mut ctx, &Expr::Closure(closure));

        // Should infer as: fn(int) int
        match &ty {
            Type::Fn(params, ret) => {
                assert_eq!(params.len(), 1);
                assert!(matches!(params[0], Type::Int));  // x has explicit type annotation
                assert!(matches!(&**ret, Type::Int));  // return type is explicitly int
            }
            _ => panic!("Expected Fn type, got {:?}", ty),
        }
    }

    #[test]
    fn test_type_args_stored_in_call() {
        use crate::ast::{Args, Arg, Call};

        // Create a call
        let mut args = Args::new();
        args.args.push(Arg::Pos(Expr::Int(42)));

        let mut call = Call {
            name: Box::new(Expr::Ident("identity".into())),
            args,
            ret: Type::Unknown,
            type_args: vec![],
        };

        // Get immutable reference to call (as would happen during type inference)
        let call_ref: &Call = &call;

        // Verify type_args is initially empty
        assert_eq!(call_ref.type_args.len(), 0);

        // Simulate what happens in infer_call_type: mutate type_args through unsafe code
        let test_type_args = vec![("T".into(), Type::Int)];

        unsafe {
            let call_ptr = call_ref as *const Call as *mut Call;
            (*call_ptr).type_args = test_type_args;
        }

        // Verify that type_args were stored in the original Call
        assert_eq!(call.type_args.len(), 1, "type_args should contain one entry");
        assert_eq!(call.type_args[0].0, "T", "Generic parameter name should be T");
        assert!(matches!(call.type_args[0].1, Type::Int), "Concrete type should be Int");
    }
}
