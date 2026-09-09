//! Type mapping utilities.
//!
//! Provides conversion between the manifest `CTypeDesc` and various
//! representations used by the rest of the toolchain.

use crate::manifest::CTypeDesc;

/// Map a CTypeDesc to the corresponding C type spelling.
///
/// Returns `String` (not `&'static str`) because `FnPtr` composes its
/// signature, e.g. `int (*)(unsigned int)`.
pub fn ctype_to_c_str(ty: &CTypeDesc) -> String {
    match ty {
        CTypeDesc::Void => "void".into(),
        CTypeDesc::Bool => "int".into(),
        CTypeDesc::Char => "char".into(),
        CTypeDesc::CStr => "const char*".into(),
        CTypeDesc::Int => "int".into(),
        CTypeDesc::UInt => "unsigned int".into(),
        CTypeDesc::Long => "long".into(),
        CTypeDesc::ULong => "unsigned long".into(),
        CTypeDesc::Size => "size_t".into(),
        CTypeDesc::Float => "float".into(),
        CTypeDesc::Double => "double".into(),
        CTypeDesc::Ptr => "void*".into(),
        CTypeDesc::PtrMut => "void*".into(),
        CTypeDesc::FnPtr { ret, params } => format!(
            "{} (*)({})",
            ctype_to_c_str(ret),
            params.iter().map(ctype_to_c_str).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// Map a CTypeDesc to a human-readable Rust type name (informational only).
pub fn ctype_to_rust_str(ty: &CTypeDesc) -> String {
    match ty {
        CTypeDesc::Void => "()".into(),
        CTypeDesc::Bool => "bool".into(),
        CTypeDesc::Char => "u8".into(),
        CTypeDesc::CStr => "*const c_char".into(),
        CTypeDesc::Int => "i32".into(),
        CTypeDesc::UInt => "u32".into(),
        CTypeDesc::Long => "i64".into(),
        CTypeDesc::ULong => "u64".into(),
        CTypeDesc::Size => "usize".into(),
        CTypeDesc::Float => "f32".into(),
        CTypeDesc::Double => "f64".into(),
        CTypeDesc::Ptr => "*const c_void".into(),
        CTypeDesc::PtrMut => "*mut c_void".into(),
        CTypeDesc::FnPtr { ret, params } => format!(
            "fn({}) -> {}",
            params.iter().map(ctype_to_rust_str).collect::<Vec<_>>().join(", "),
            ctype_to_rust_str(ret)
        ),
    }
}
