//! Type mapping utilities.
//!
//! Provides conversion between the manifest `CTypeDesc` and various
//! representations used by the rest of the toolchain.

use crate::manifest::CTypeDesc;

/// Map a CTypeDesc to the corresponding C type spelling.
pub fn ctype_to_c_str(ty: &CTypeDesc) -> &'static str {
    match ty {
        CTypeDesc::Void => "void",
        CTypeDesc::Bool => "int",
        CTypeDesc::Char => "char",
        CTypeDesc::CStr => "const char*",
        CTypeDesc::Int => "int",
        CTypeDesc::UInt => "unsigned int",
        CTypeDesc::Long => "long",
        CTypeDesc::ULong => "unsigned long",
        CTypeDesc::Size => "size_t",
        CTypeDesc::Float => "float",
        CTypeDesc::Double => "double",
        CTypeDesc::Ptr => "void*",
        CTypeDesc::PtrMut => "void*",
    }
}

/// Map a CTypeDesc to a human-readable Rust type name (informational only).
pub fn ctype_to_rust_str(ty: &CTypeDesc) -> &'static str {
    match ty {
        CTypeDesc::Void => "()",
        CTypeDesc::Bool => "bool",
        CTypeDesc::Char => "u8",
        CTypeDesc::CStr => "*const c_char",
        CTypeDesc::Int => "i32",
        CTypeDesc::UInt => "u32",
        CTypeDesc::Long => "i64",
        CTypeDesc::ULong => "u64",
        CTypeDesc::Size => "usize",
        CTypeDesc::Float => "f32",
        CTypeDesc::Double => "f64",
        CTypeDesc::Ptr => "*const c_void",
        CTypeDesc::PtrMut => "*mut c_void",
    }
}
