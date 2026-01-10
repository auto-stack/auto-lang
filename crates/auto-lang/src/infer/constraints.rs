//! 类型约束表示和求解
//!
//! # 概述
//!
//! 本模块定义了类型约束系统，用于跟踪和求解类型推导过程中产生的约束。
//!
//! # 约束类型
//!
//! - `Equal` - 两个类型必须相等
//! - `Callable` - 类型必须可调用
//! - `Indexable` - 类型必须可索引（数组/字符串）
//! - `Subtype` - 类型必须是另一个类型的子类型
//!
//! # 示例
//!
//! ```rust
//! use auto_lang::infer::TypeConstraint;
//! use auto_lang::ast::Type;
//! use miette::SourceSpan;
//!
//! // 创建相等性约束
//! let constraint = TypeConstraint::Equal(
//!     Type::Int,
//!     Type::Int,
//!     SourceSpan::new(0_usize.into(), 10_usize.into()),
//! );
//! ```

use crate::ast::Type;
use miette::SourceSpan;

/// 类型约束
///
/// 表示类型推导过程中产生的约束条件。
#[derive(Debug, Clone)]
pub enum TypeConstraint {
    /// 两个类型必须相等
    ///
    /// # 字段
    ///
    /// * `ty1` - 第一个类型
    /// * `ty2` - 第二个类型
    /// * `span` - 源代码位置（用于错误报告）
    Equal(Type, Type, SourceSpan),

    /// 类型必须可调用
    ///
    /// # 字段
    ///
    /// * `ty` - 要检查的类型
    /// * `span` - 源代码位置
    Callable(Type, SourceSpan),

    /// 类型必须可索引（数组/字符串）
    ///
    /// # 字段
    ///
    /// * `ty` - 要检查的类型
    /// * `span` - 源代码位置
    Indexable(Type, SourceSpan),

    /// 类型必须是另一个类型的子类型
    ///
    /// # 字段
    ///
    /// * `subtype` - 子类型
    /// * `supertype` - 父类型
    /// * `span` - 源代码位置
    Subtype(Type, Type, SourceSpan),
}

impl TypeConstraint {
    /// 获取约束的源代码位置
    pub fn span(&self) -> SourceSpan {
        match self {
            TypeConstraint::Equal(_, _, span) => *span,
            TypeConstraint::Callable(_, span) => *span,
            TypeConstraint::Indexable(_, span) => *span,
            TypeConstraint::Subtype(_, _, span) => *span,
        }
    }

    /// 判断约束是否是相等性约束
    pub fn is_equality(&self) -> bool {
        matches!(self, TypeConstraint::Equal(_, _, _))
    }

    /// 判断约束是否是可调用性约束
    pub fn is_callable(&self) -> bool {
        matches!(self, TypeConstraint::Callable(_, _))
    }

    /// 判断约束是否是可索引性约束
    pub fn is_indexable(&self) -> bool {
        matches!(self, TypeConstraint::Indexable(_, _))
    }

    /// 判断约束是否是子类型约束
    pub fn is_subtype(&self) -> bool {
        matches!(self, TypeConstraint::Subtype(_, _, _))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_equality() {
        let constraint = TypeConstraint::Equal(
            Type::Int,
            Type::Int,
            SourceSpan::new(0_usize.into(), 10_usize.into()),
        );

        assert!(constraint.is_equality());
        assert!(!constraint.is_callable());
        assert!(!constraint.is_indexable());
        assert!(!constraint.is_subtype());
    }

    #[test]
    fn test_constraint_span() {
        let span = SourceSpan::new(5_usize.into(), 15_usize.into());
        let constraint = TypeConstraint::Equal(Type::Int, Type::Float, span);

        assert_eq!(constraint.span(), span);
    }
}
