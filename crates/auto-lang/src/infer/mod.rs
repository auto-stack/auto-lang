//! AutoLang Type Inference and Checking Subsystem
//!
//! # Overview
//!
//! This module provides type inference and type checking for AutoLang.
//!

pub mod constraints;
pub mod context;
pub mod errors;
pub mod expr;
pub mod functions;
pub mod registry;
pub mod stmt;
// Plan 125 Phase 3.6: Task type checking
pub mod task_types;
pub mod unification;

// Re-export public API
pub use constraints::TypeConstraint;
pub use context::InferenceContext;
pub use errors::{suggest_primitive_type, suggest_type, suggest_type_mismatch_fix, suggest_variable, should_continue};
pub use expr::infer_expr;
pub use functions::check_fn;
pub use registry::TypeRegistry;
pub use stmt::check_stmt;
// Plan 125 Phase 3.6: Task type checking
pub use task_types::{EnvelopeInfo, TaskTypeChecker, literal_to_type};

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
///
/// # Phase 6 Enhancement
///
/// When types don't match, errors are automatically collected by the parser
/// instead of aborting compilation, allowing multiple errors to be reported
/// in a single pass.
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
        (Type::Str(_), Type::Str(_))
        | (Type::Str(_), Type::String)
        | (Type::String, Type::Str(_))
        | (Type::String, Type::String) => true,
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
