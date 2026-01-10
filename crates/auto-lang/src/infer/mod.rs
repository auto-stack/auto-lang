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

use crate::ast::Type;

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
