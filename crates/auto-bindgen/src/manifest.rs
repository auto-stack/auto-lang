//! JSON manifest types describing C header function signatures.
//!
//! These manifests are pre-generated (or hand-written) and consumed by:
//! - **AutoVM**: `c_ffi.rs` runtime loads manifests to create FFI shims via libloading
//! - **a2c transpiler**: reads manifests to resolve C function names automatically
//!
//! Plan 216 Phase 1.

use serde::{Deserialize, Serialize};

/// Top-level manifest for a single C header.
/// Calling convention annotation (Plan 595 / 004 §3.5): `"c"` = plain C ABI,
/// `"system"` = win32 (`extern "system"`; identical to C on x64, stdcall on x86).
/// Consumed by backend generators; the VM runtime ignores it (its shims are
/// compiled Rust closures with the platform's native convention).
pub const ABI_C: &str = "c";
pub const ABI_SYSTEM: &str = "system";

/// Plan 610 ⑥: lowering form selection for the a2r backend — `"static"` (S
/// form: `#[link(name)]` extern block, link-time import lib) or `"dynamic"`
/// (D form: libloading runtime resolution). Other backends ignore it (VM
/// always loads at runtime; a2c calls directly).
pub const LINK_STATIC: &str = "static";
pub const LINK_DYNAMIC: &str = "dynamic";

fn default_abi() -> String {
    ABI_C.to_string()
}

fn default_link() -> String {
    LINK_STATIC.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CHeaderManifest {
    /// Header file name, e.g. `"string.h"`
    pub header: String,
    /// Platform library name: `"c"` on POSIX, resolved at runtime
    pub library: String,
    /// Calling convention for this header's functions (`"c"` | `"system"`,
    /// serde default `"c"`; win32 headers like windows.h use `"system"`)
    #[serde(default = "default_abi")]
    pub abi: String,
    /// Plan 610 ⑥: a2r lowering form (`"static"` | `"dynamic"`, serde
    /// default `"static"`). Static = `#[link(name)]` extern block; dynamic =
    /// libloading with env → exe-dir → PATH resolution.
    #[serde(default = "default_link")]
    pub link: String,
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
    /// Function pointer / callback (Plan 595 / 004 §3.5). Carries the
    /// callback's own signature. VM runtime rejects it at registration
    /// (callback bridges are "Impossible"-per Plan 267); a2c consumes it
    /// natively (closures transpile to function pointers, Plan 060).
    FnPtr {
        ret: Box<CTypeDesc>,
        params: Vec<CTypeDesc>,
    },
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
            CTypeDesc::FnPtr { .. } => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FnPtr round-trips through the internally-tagged serde representation
    /// (`kind: "FnPtr"`, `inner: {ret, params}`) — the JSON shape is the
    /// cross-backend contract (VM/a2c/a2r all read these manifests).
    #[test]
    fn fnptr_serde_roundtrip() {
        let f = CFunction {
            name: "SetConsoleCtrlHandler".into(),
            params: vec![
                CParam {
                    name: "handler".into(),
                    ty: CTypeDesc::FnPtr {
                        ret: Box::new(CTypeDesc::Int),
                        params: vec![CTypeDesc::UInt],
                    },
                },
                CParam { name: "add".into(), ty: CTypeDesc::Int },
            ],
            return_type: CTypeDesc::Int,
            variadic: false,
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: CFunction = serde_json::from_str(&json).unwrap();
        assert_eq!(back.params[0].ty, f.params[0].ty);
        assert!(json.contains("\"FnPtr\""), "tagged variant name: {json}");
    }

    /// Historical manifests without an `abi` field deserialize to `"c"`
    /// (serde default) — existing c_bindings JSON stays loadable.
    #[test]
    fn abi_defaults_to_c_for_legacy_json() {
        let legacy = r#"{
            "header": "string.h", "library": "c",
            "functions": [{"name":"strlen","params":[{"name":"s","ty":{"kind":"CStr"}}],
                           "return_type":{"kind":"Size"},"variadic":false}]
        }"#;
        let m: CHeaderManifest = serde_json::from_str(legacy).unwrap();
        assert_eq!(m.abi, ABI_C);
    }
}
