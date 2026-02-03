//! AutoLang Type Inference and Checking Subsystem
//!
//! # Overview
//!
//! This module provides type inference and type checking for AutoLang.
//!

pub mod constraints;
pub mod context;
pub mod expr;
pub mod functions;
pub mod stmt;
pub mod unification;

// Re-export public API
pub use constraints::TypeConstraint;
pub use context::InferenceContext;
pub use expr::infer_expr;
pub use functions::check_fn;
pub use stmt::check_stmt;

use crate::ast::{Member, Type};
use crate::error::{AutoError, TypeError};

/// Unify two types
pub fn unify(
    ctx: &mut InferenceContext,
    ty1: Type,
    ty2: Type,
) -> Result<Type, crate::error::TypeError> {
    ctx.unify(ty1, ty2)
}

/// Check field type compatibility
pub fn check_field_type(
    member: &Member,
    value_ty: &Type,
    span: miette::SourceSpan,
) -> Result<(), AutoError> {
    let expected_ty = &member.ty;

    if matches!(expected_ty, Type::Unknown) {
        return Ok(());
    }

    if matches!(value_ty, Type::Unknown) {
        return Ok(());
    }

    if !types_are_compatible(expected_ty, value_ty) {
        return Err(TypeError::FieldMismatch {
            span,
            field: member.name.to_string(),
            expected: expected_ty.to_string(),
            found: value_ty.to_string(),
        }
        .into());
    }

    Ok(())
}

fn types_are_compatible(expected: &Type, found: &Type) -> bool {
    match (expected, found) {
        (Type::Unknown, _) | (_, Type::Unknown) => true,
        (Type::Int, Type::Int) | (Type::Int, Type::Uint) => true,
        (Type::Uint, Type::Uint) => true,
        (Type::Float, Type::Float) | (Type::Float, Type::Double) => true,
        (Type::Double, Type::Double) => true,
        (Type::Str(_), Type::Str(_)) => true,
        (Type::Bool, Type::Bool) => true,
        (Type::Char, Type::Char) => true,
        (Type::Array(a), Type::Array(b)) => {
            a.len == b.len && types_are_compatible(&a.elem, &b.elem)
        }
        (Type::Ptr(inner_a), Type::Ptr(inner_b)) => {
            types_are_compatible(&inner_a.of.borrow(), &inner_b.of.borrow())
        }
        (Type::User(a), Type::User(b)) => a.name == b.name,
        (Type::Spec(_), Type::Spec(_)) => true,
        _ => false,
    }
}
