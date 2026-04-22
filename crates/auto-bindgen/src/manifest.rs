//! JSON manifest types describing C header function signatures.
//!
//! These manifests are pre-generated (or hand-written) and consumed by:
//! - **AutoVM**: `c_ffi.rs` runtime loads manifests to create FFI shims via libloading
//! - **a2c transpiler**: reads manifests to resolve C function names automatically
//!
//! Plan 216 Phase 1.

use serde::{Deserialize, Serialize};

/// Top-level manifest for a single C header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CHeaderManifest {
    /// Header file name, e.g. `"string.h"`
    pub header: String,
    /// Platform library name: `"c"` on POSIX, resolved at runtime
    pub library: String,
    /// Functions exported from this header
    pub functions: Vec<CFunction>,
}

/// A single C function signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFunction {
    /// Function name, e.g. `"strlen"`
    pub name: String,
    /// Parameters
    pub params: Vec<CParam>,
    /// Return type
    pub return_type: CTypeDesc,
    /// Whether this is a variadic function (e.g. printf)
    pub variadic: bool,
}

/// A single function parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CParam {
    /// Parameter name (informational)
    pub name: String,
    /// Parameter type
    pub ty: CTypeDesc,
}

/// Description of a C type, used in manifests.
///
/// Covers the primitive and pointer types commonly found in standard C headers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "inner")]
pub enum CTypeDesc {
    Void,
    Bool,
    /// `char` (single byte)
    Char,
    /// `const char*` — null-terminated C string
    CStr,
    /// `signed int` (32-bit)
    Int,
    /// `unsigned int`
    UInt,
    /// `signed long` (platform-sized)
    Long,
    /// `unsigned long`
    ULong,
    /// `size_t`
    Size,
    /// `float` (32-bit)
    Float,
    /// `double` (64-bit)
    Double,
    /// Opaque pointer (`void*` or typed pointer)
    Ptr,
    /// Mutable pointer to a named type (e.g. `char*`)
    PtrMut,
}

impl CTypeDesc {
    /// How many 32-bit VM slots this type occupies when passed on the stack.
    pub fn slot_count(&self) -> usize {
        match self {
            CTypeDesc::Void => 0,
            CTypeDesc::Bool | CTypeDesc::Char | CTypeDesc::Int | CTypeDesc::UInt => 1,
            CTypeDesc::Long | CTypeDesc::ULong | CTypeDesc::Size => 2,
            CTypeDesc::Float => 1,
            CTypeDesc::Double => 2,
            CTypeDesc::CStr | CTypeDesc::Ptr | CTypeDesc::PtrMut => 2,
        }
    }
}
