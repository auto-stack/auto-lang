//! AutoLang 类型推导和类型检查子系统
//!
//! # 概述
//!
//! 本模块提供了 AutoLang 的类型推导和类型检查功能，包括：
//! - 混合类型推导策略（局部逐步推导 + 简化 Hindley-Milner）
//! - 静态类型检查
//! - 类型错误恢复
//! - 友好的错误提示
//!
//! # 模块结构
//!
//! - [`context`]：类型推导上下文和环境管理
//! - [`constraints`]：类型约束表示和求解
//! - [`unification`]：类型统一算法
//! - [`expr`]：表达式类型推导
//! - [`stmt`]：语句类型检查
//! - [`functions`]：函数签名推导
//! - [`errors`]：类型相关错误辅助
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

pub mod context;
pub mod constraints;
pub mod unification;
pub mod expr;

// 预留未来的模块
// pub mod stmt;
// pub mod functions;
// pub mod errors;

// 重新导出公共 API
pub use context::InferenceContext;
pub use constraints::TypeConstraint;
pub use expr::infer_expr;

use crate::ast::{Type, Member};
use crate::error::{AutoError, TypeError};

/// 统一两个类型
///
/// # 参数
///
/// * `ctx` - 类型推导上下文
/// * `ty1` - 第一个类型
/// * `ty2` - 第二个类型
///
/// # 返回
///
/// 统一后的类型，如果无法统一则返回错误
///
/// # 错误
///
/// 如果两个类型不兼容，返回 `TypeError::Mismatch`
pub fn unify(
    ctx: &mut InferenceContext,
    ty1: Type,
    ty2: Type,
) -> Result<Type, crate::error::TypeError> {
    ctx.unify(ty1, ty2)
}

/// 检查语句的类型
///
/// # 参数
///
/// * `_ctx` - 类型推导上下文
/// * `_stmt` - 要检查的语句
///
/// # 返回
///
/// 如果类型检查成功返回 `Ok(())`，否则返回类型错误
pub fn check_stmt(
    _ctx: &mut InferenceContext,
    _stmt: &crate::ast::Stmt,
) -> Result<(), crate::error::TypeError> {
    // TODO: 实现完整的语句类型检查（阶段 3）
    Ok(())
}

/// 检查字段值是否匹配成员类型定义
///
/// # 参数
///
/// * `member` - 类型成员定义
/// * `value_ty` - 值表达式的类型
/// * `span` - 值表达式在源代码中的位置
///
/// # 返回
///
/// 如果类型匹配返回 `Ok(())`，否则返回 `TypeError::FieldMismatch`
///
/// # 示例
///
/// ```rust
/// use auto_lang::infer::check_field_type;
/// use auto_lang::ast::{Type, Member, Name};
/// use miette::SourceSpan;
///
/// let member = Member {
///     name: Name::from("x"),
///     ty: Type::Int,
///     value: None,
/// };
/// let result = check_field_type(&member, &Type::Int, SourceSpan::new(0.into(), 0_usize));
/// assert!(result.is_ok());
/// ```
pub fn check_field_type(
    member: &Member,
    value_ty: &Type,
    span: miette::SourceSpan,
) -> Result<(), AutoError> {
    let expected_ty = &member.ty;

    // 如果期望类型是 Unknown，跳过检查
    if matches!(expected_ty, Type::Unknown) {
        return Ok(());
    }

    // 如果值类型是 Unknown，无法在编译时检查
    if matches!(value_ty, Type::Unknown) {
        return Ok(()); // 运行时检查
    }

    // 尝试统一类型
    if !types_are_compatible(expected_ty, value_ty) {
        return Err(TypeError::FieldMismatch {
            span,
            field: member.name.to_string(),
            expected: expected_ty.to_string(),
            found: value_ty.to_string(),
        }.into());
    }

    Ok(())
}

/// 简化的类型兼容性检查
///
/// TODO: 后续可以使用完整的 unify 算法
fn types_are_compatible(expected: &Type, found: &Type) -> bool {
    match (expected, found) {
        // Unknown 类型兼容任何类型
        (Type::Unknown, _) | (_, Type::Unknown) => true,

        // 整数类型兼容性（int 可以接受 uint）
        (Type::Int, Type::Int) | (Type::Int, Type::Uint) => true,
        (Type::Uint, Type::Uint) => true,

        // 浮点类型兼容性（float 可以接受 double）
        (Type::Float, Type::Float) | (Type::Float, Type::Double) => true,
        (Type::Double, Type::Double) => true,

        // 字符串类型
        (Type::Str(_), Type::Str(_)) => true,

        // 布尔类型
        (Type::Bool, Type::Bool) => true,

        // 字符类型
        (Type::Char, Type::Char) => true,

        // 数组类型：检查元素类型和长度
        (Type::Array(a), Type::Array(b)) => {
            a.len == b.len && types_are_compatible(&a.elem, &b.elem)
        }

        // 指针类型
        (Type::Ptr(inner_a), Type::Ptr(inner_b)) => {
            types_are_compatible(&inner_a.of.borrow(), &inner_b.of.borrow())
        }

        // 用户定义类型：按名称比较
        (Type::User(a), Type::User(b)) => a.name == b.name,

        // Spec 类型：按名称比较
        (Type::Spec(_), Type::Spec(_)) => true, // 多态类型，总是兼容

        // 其他情况不兼容
        _ => false,
    }
}
