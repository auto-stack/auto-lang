//! 类型统一算法
//!
//! # 概述
//!
//! 本模块实现了 Robinson 类型统一算法，这是 Hindley-Milner 类型推导的核心。
//! 统一算法尝试将两个类型通过替换使其相等。
//!
//! # 算法
//!
//! Robinson 算法的基本思想是：
//! 1. 如果两个类型相同，则统一成功
//! 2. 如果其中一个类型是类型变量（Unknown），则用另一个类型替换它
//! 3. 如果是复合类型（如数组、函数），递归统一其组成部分
//! 4. 否则统一失败
//!
//! # Occurs Check
//!
//! 防止出现无限类型（如 `α = List<α>`），在替换前检查变量是否出现在类型中。
//!
//! # 示例
//!
//! ```rust
//! use auto_lang::infer::unification::unify;
//! use auto_lang::ast::{Type, ArrayType};
//! use miette::SourceSpan;
//!
//! let span = SourceSpan::new(0.into(), 0);
//!
//! // 统一相同的基础类型
//! assert!(matches!(unify(&Type::Int, &Type::Int, span), Ok(Type::Int)));
//!
//! // 统一 Unknown 与任何类型
//! assert!(matches!(unify(&Type::Unknown, &Type::Int, span), Ok(Type::Int)));
//!
//! // 统一数组
//! let arr1 = Type::Array(ArrayType {
//!     elem: Box::new(Type::Int),
//!     len: 3,
//! });
//! let arr2 = Type::Array(ArrayType {
//!     elem: Box::new(Type::Int),
//!     len: 3,
//! });
//! assert!(matches!(unify(&arr1, &arr2, span), Ok(Type::Array(..))));
//! ```

use crate::ast::{ArrayType, PtrType, Type};
use crate::error::{TypeError, Warning};
use miette::SourceSpan;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

/// 类型统一错误
#[derive(Debug, Clone)]
pub enum UnificationError {
    /// 类型不匹配
    Mismatch { expected: Type, found: Type },

    /// Occurs check 失败（无限类型）
    OccursFailed { var: String, ty: Type },

    /// 数组长度不匹配
    ArrayLengthMismatch { len1: usize, len2: usize },

    /// 用户类型名称不匹配
    UserTypeNameMismatch { name1: String, name2: String },
}

impl fmt::Display for UnificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnificationError::Mismatch { expected, found } => {
                write!(f, "type mismatch: expected {}, found {}", expected, found)
            }
            UnificationError::OccursFailed { var, ty } => {
                write!(f, "infinite type: {} = {}", var, ty)
            }
            UnificationError::ArrayLengthMismatch { len1, len2 } => {
                write!(f, "array length mismatch: {} != {}", len1, len2)
            }
            UnificationError::UserTypeNameMismatch { name1, name2 } => {
                write!(f, "user type name mismatch: {} != {}", name1, name2)
            }
        }
    }
}

impl From<UnificationError> for TypeError {
    fn from(err: UnificationError) -> Self {
        match err {
            UnificationError::Mismatch { expected, found } => TypeError::Mismatch {
                expected: expected.to_string(),
                found: found.to_string(),
                span: SourceSpan::new(0.into(), 0),
            },
            UnificationError::OccursFailed { .. } => TypeError::Mismatch {
                expected: "finite type".to_string(),
                found: "infinite type".to_string(),
                span: SourceSpan::new(0.into(), 0),
            },
            UnificationError::ArrayLengthMismatch { .. } => TypeError::Mismatch {
                expected: err.to_string(),
                found: "array".to_string(),
                span: SourceSpan::new(0.into(), 0),
            },
            UnificationError::UserTypeNameMismatch { .. } => TypeError::Mismatch {
                expected: err.to_string(),
                found: err.to_string(),
                span: SourceSpan::new(0.into(), 0),
            },
        }
    }
}

/// 检查类型变量是否出现在类型中（occurs check）
///
/// # 参数
///
/// * `var_name` - 类型变量名称
/// * `ty` - 要检查的类型
///
/// # 返回
///
/// 如果类型变量出现在类型中返回 `true`，否则返回 `false`
pub fn occurs_in(var_name: &str, ty: &Type) -> bool {
    match ty {
        // 基础类型不包含类型变量
        Type::Byte | Type::Int | Type::Uint | Type::USize |
        Type::Float | Type::Double | Type::Bool | Type::Char |
        Type::Void => false,

        // 字符串类型
        Type::Str(_) | Type::CStr | Type::StrSlice => false,

        // Unknown 类型
        Type::Unknown => false,

        // 复合类型：递归检查
        Type::Array(arr) => occurs_in(var_name, &arr.elem),

        Type::List(elem) => occurs_in(var_name, elem),

        Type::Ptr(ptr) => {
            let inner_ty = ptr.of.borrow();
            occurs_in(var_name, &inner_ty)
        }

        // May 类型：递归检查内部类型
        Type::May(inner) => occurs_in(var_name, inner),

        // Linear 类型：递归检查内部类型
        Type::Linear(inner) => occurs_in(var_name, inner),

        // 用户类型：暂时假设不包含类型变量
        Type::User(_) | Type::CStruct(_) => false,

        // Spec 类型：暂时假设不包含类型变量
        Type::Spec(_) => false,

        // 其他类型
        Type::Union(_) | Type::Tag(_) | Type::Enum(_) => false,
    }
}

/// 统一两个类型
///
/// # 参数
///
/// * `ty1` - 第一个类型
/// * `ty2` - 第二个类型
/// * `span` - 源代码位置（用于错误报告）
///
/// # 返回
///
/// 统一后的类型，如果无法统一则返回错误
pub fn unify(ty1: &Type, ty2: &Type, span: SourceSpan) -> Result<Type, UnificationError> {
    match (ty1, ty2) {
        // 1. Unknown 类型是通配符
        (Type::Unknown, ty) => Ok(ty.clone()),
        (ty, Type::Unknown) => Ok(ty.clone()),

        // 2. 基础类型必须完全匹配
        (Type::Byte, Type::Byte) => Ok(Type::Byte),
        (Type::Int, Type::Int) => Ok(Type::Int),
        (Type::Uint, Type::Uint) => Ok(Type::Uint),
        (Type::USize, Type::USize) => Ok(Type::USize),
        (Type::Float, Type::Float) => Ok(Type::Float),
        (Type::Double, Type::Double) => Ok(Type::Double),
        (Type::Bool, Type::Bool) => Ok(Type::Bool),
        (Type::Char, Type::Char) => Ok(Type::Char),

        // 3. 字符串类型
        (Type::Str(_), Type::Str(_)) => Ok(ty1.clone()),
        (Type::CStr, Type::CStr) => Ok(ty1.clone()),

        // 4. Void 类型
        (Type::Void, Type::Void) => Ok(Type::Void),

        // 5. 数组类型：递归统一元素类型和长度
        (Type::Array(arr1), Type::Array(arr2)) => {
            // 统一元素类型
            let elem_ty = unify(&arr1.elem, &arr2.elem, span)?;

            // 检查长度
            if arr1.len != arr2.len {
                return Err(UnificationError::ArrayLengthMismatch {
                    len1: arr1.len,
                    len2: arr2.len,
                });
            }

            Ok(Type::Array(ArrayType {
                elem: Box::new(elem_ty),
                len: arr1.len,
            }))
        }

        // 7. 指针类型：统一目标类型
        (Type::Ptr(ptr1), Type::Ptr(ptr2)) => {
            let inner1 = ptr1.of.borrow();
            let inner2 = ptr2.of.borrow();
            let target_ty = unify(&inner1, &inner2, span)?;

            Ok(Type::Ptr(PtrType {
                of: Rc::new(RefCell::new(target_ty)),
            }))
        }

        // 8. 用户类型：必须名称匹配
        (Type::User(decl1), Type::User(decl2)) => {
            if decl1.name == decl2.name {
                Ok(ty1.clone())
            } else {
                Err(UnificationError::UserTypeNameMismatch {
                    name1: decl1.name.to_string(),
                    name2: decl2.name.to_string(),
                })
            }
        }

        // 9. CStruct 类型：必须名称匹配
        (Type::CStruct(decl1), Type::CStruct(decl2)) => {
            if decl1.name == decl2.name {
                Ok(ty1.clone())
            } else {
                Err(UnificationError::UserTypeNameMismatch {
                    name1: decl1.name.to_string(),
                    name2: decl2.name.to_string(),
                })
            }
        }

        // 10. 其他类型不匹配
        (ty1, ty2) => Err(UnificationError::Mismatch {
            expected: ty1.clone(),
            found: ty2.clone(),
        }),
    }
}

/// 尝试统一两个类型，带类型强制转换支持
///
/// 某些类型之间可以自动转换（如 int <-> uint, float <-> double）。
///
/// # 参数
///
/// * `ty1` - 第一个类型
/// * `ty2` - 第二个类型
/// * `warnings` - 警告累加器（用于记录隐式类型转换）
///
/// # 返回
///
/// 统一后的类型，如果无法统一则返回错误
pub fn unify_with_coercion(
    ty1: &Type,
    ty2: &Type,
    warnings: &mut Vec<Warning>,
    span: SourceSpan,
) -> Result<Type, UnificationError> {
    match (ty1, ty2) {
        // int <-> uint 强制转换（带警告）
        (Type::Int, Type::Uint) => {
            warnings.push(Warning::ImplicitTypeConversion {
                from: "int".into(),
                to: "uint".into(),
                span,
            });
            Ok(Type::Uint)
        }
        (Type::Uint, Type::Int) => {
            warnings.push(Warning::ImplicitTypeConversion {
                from: "uint".into(),
                to: "int".into(),
                span,
            });
            Ok(Type::Int)
        }

        // float <-> double 强制转换（带警告）
        (Type::Float, Type::Double) => {
            warnings.push(Warning::ImplicitTypeConversion {
                from: "float".into(),
                to: "double".into(),
                span,
            });
            Ok(Type::Double)
        }
        (Type::Double, Type::Float) => {
            warnings.push(Warning::ImplicitTypeConversion {
                from: "double".into(),
                to: "float".into(),
                span,
            });
            Ok(Type::Float)
        }

        // 其他情况使用标准统一
        _ => unify(ty1, ty2, span),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_same_types() {
        let span = SourceSpan::new(0.into(), 10);

        assert!(matches!(
            unify(&Type::Int, &Type::Int, span),
            Ok(Type::Int)
        ));
        assert!(matches!(
            unify(&Type::Float, &Type::Float, span),
            Ok(Type::Float)
        ));
        assert!(matches!(
            unify(&Type::Bool, &Type::Bool, span),
            Ok(Type::Bool)
        ));
    }

    #[test]
    fn test_unify_different_types_fails() {
        let span = SourceSpan::new(0.into(), 10);

        assert!(matches!(
            unify(&Type::Int, &Type::Bool, span),
            Err(UnificationError::Mismatch { .. })
        ));
        assert!(matches!(
            unify(&Type::Float, &Type::Int, span),
            Err(UnificationError::Mismatch { .. })
        ));
    }

    #[test]
    fn test_unify_with_unknown() {
        let span = SourceSpan::new(0.into(), 10);

        assert!(matches!(
            unify(&Type::Unknown, &Type::Int, span),
            Ok(Type::Int)
        ));
        assert!(matches!(
            unify(&Type::Int, &Type::Unknown, span),
            Ok(Type::Int)
        ));
        assert!(matches!(
            unify(&Type::Unknown, &Type::Unknown, span),
            Ok(Type::Unknown)
        ));
    }

    #[test]
    fn test_unify_arrays() {
        let span = SourceSpan::new(0.into(), 10);
        let arr1 = Type::Array(ArrayType {
            elem: Box::new(Type::Int),
            len: 3,
        });
        let arr2 = Type::Array(ArrayType {
            elem: Box::new(Type::Int),
            len: 3,
        });

        assert!(matches!(
            unify(&arr1, &arr2, span),
            Ok(Type::Array(..))
        ));
    }

    #[test]
    fn test_unify_arrays_different_length_fails() {
        let span = SourceSpan::new(0.into(), 10);
        let arr1 = Type::Array(ArrayType {
            elem: Box::new(Type::Int),
            len: 3,
        });
        let arr2 = Type::Array(ArrayType {
            elem: Box::new(Type::Int),
            len: 4,
        });

        assert!(matches!(
            unify(&arr1, &arr2, span),
            Err(UnificationError::ArrayLengthMismatch { .. })
        ));
    }

    #[test]
    fn test_unify_arrays_different_elem_type_fails() {
        let span = SourceSpan::new(0.into(), 10);
        let arr1 = Type::Array(ArrayType {
            elem: Box::new(Type::Int),
            len: 3,
        });
        let arr2 = Type::Array(ArrayType {
            elem: Box::new(Type::Float),
            len: 3,
        });

        assert!(matches!(
            unify(&arr1, &arr2, span),
            Err(UnificationError::Mismatch { .. })
        ));
    }

    #[test]
    fn test_unify_with_coercion() {
        let span = SourceSpan::new(0.into(), 10);
        let mut warnings = Vec::new();

        // int <-> uint
        assert!(matches!(
            unify_with_coercion(&Type::Int, &Type::Uint, &mut warnings, span),
            Ok(Type::Uint)
        ));
        assert_eq!(warnings.len(), 1);

        warnings.clear();

        // float <-> double
        assert!(matches!(
            unify_with_coercion(&Type::Float, &Type::Double, &mut warnings, span),
            Ok(Type::Double)
        ));
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_occurs_in() {
        // 基础类型不包含类型变量
        assert!(!occurs_in("T", &Type::Int));
        assert!(!occurs_in("T", &Type::Bool));

        // Unknown 类型不是类型变量，不包含任何命名变量
        assert!(!occurs_in("T", &Type::Unknown));

        // 数组：递归检查元素类型
        let arr = Type::Array(ArrayType {
            elem: Box::new(Type::Unknown),
            len: 3,
        });
        assert!(!occurs_in("T", &arr));

        let arr = Type::Array(ArrayType {
            elem: Box::new(Type::Int),
            len: 3,
        });
        assert!(!occurs_in("T", &arr));
    }
}
