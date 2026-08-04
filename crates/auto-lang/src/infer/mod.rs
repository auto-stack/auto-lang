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
        (Type::StrFixed(_), Type::StrFixed(_))
        | (Type::StrFixed(_), Type::StrOwned)
        | (Type::StrOwned, Type::StrFixed(_))
        | (Type::StrOwned, Type::StrOwned)
        // Plan 380 P1: string literals ("x") are StrSlice but assignable to
        // StrOwned/StrFixed fields (codegen auto-inserts .to_string()). Without
        // this, struct literals like StatusOk(status: "ok") inside a call arg
        // (e.g. Json(StatusOk(status:"ok"))) fail field type check — the outer
        // call suppresses the codegen .to_string() path that bare lets get.
        | (Type::StrOwned, Type::StrSlice)
        | (Type::StrSlice, Type::StrOwned)
        | (Type::StrFixed(_), Type::StrSlice)
        | (Type::StrSlice, Type::StrFixed(_))
        | (Type::StrSlice, Type::StrSlice) => true,
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
        // C9 (Plan 018 parity): container types were previously never compared
        // here, so a field like `children Option<List<TreeNode>>` and a variable
        // with the same annotation (both parsed as `Type::GenericInstance`)
        // rendered identically yet failed the field type check. Add elementwise
        // compatibility for the container forms used in .at struct literals.
        (Type::GenericInstance(a), Type::GenericInstance(b)) => {
            a.base_name == b.base_name
                && a.args.len() == b.args.len()
                && a
                    .args
                    .iter()
                    .zip(b.args.iter())
                    .all(|(x, y)| types_are_compatible(x, y))
        }
        // C9b (Plan 018 parity): `Option<T>` written in .at source parses as
        // `GenericInstance("Option", [T])`, but `Some(x)` / `None` and suffix-`?`
        // types infer as `Type::Option(T)`. The two are aliases — a struct
        // literal assigning `Some(x)` to an `Option<T>` field must pass the
        // field check. Same aliasing for List / Result.
        (Type::GenericInstance(a), Type::Option(b))
        | (Type::Option(b), Type::GenericInstance(a))
            if a.base_name == "Option" && a.args.len() == 1 => {
                types_are_compatible(&a.args[0], b)
            }
        (Type::GenericInstance(a), Type::Result(b))
        | (Type::Result(b), Type::GenericInstance(a))
            if a.base_name == "Result" && a.args.len() == 1 => {
                types_are_compatible(&a.args[0], b)
            }
        (Type::GenericInstance(a), Type::List(b))
        | (Type::List(b), Type::GenericInstance(a))
            if a.base_name == "List" && a.args.len() == 1 => {
                types_are_compatible(&a.args[0], b)
            }
        (Type::List(a), Type::List(b)) => types_are_compatible(a, b),
        (Type::Map(k1, v1), Type::Map(k2, v2)) => {
            types_are_compatible(k1, k2) && types_are_compatible(v1, v2)
        }
        (Type::Slice(a), Type::Slice(b)) => types_are_compatible(&a.elem, &b.elem),
        (Type::Reference(a), Type::Reference(b)) => types_are_compatible(a, b),
        (Type::Option(a), Type::Option(b)) => types_are_compatible(a, b),
        (Type::Result(a), Type::Result(b)) => types_are_compatible(a, b),
        (Type::Tuple(a), Type::Tuple(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| types_are_compatible(x, y))
        }
        // Missing primitive self-compat (Int/Uint already covered above).
        (Type::Byte, Type::Byte)
        | (Type::USize, Type::USize)
        | (Type::U64, Type::U64)
        | (Type::I64, Type::I64) => true,
        _ => false,
    }
}
