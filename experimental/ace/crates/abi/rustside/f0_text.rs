//! auto-ffi provider f0_text/1 target=x86_64-pc-windows-msvc profile=f0-text.v1
//!
//! PLAN-967 F0 frozen Rust provider (T-F0-I). The annotations ARE the
//! import metadata: `auto_bindgen::rust_import::import` derives the
//! binding, the C-compatible shim and the shared manual manifest from
//! this file — no per-function manifest is hand-written anywhere.
//!
//! Frozen F0 domain: non-generic public free functions, one borrowed
//! UTF-8 `&str` parameter, `i32`/`bool` pure results or `Option<i32>`
//! fallible (status + value out). Sync, same-thread, noescape.

/// UTF-8 byte length of the borrowed view. `A中😀\0` is 9 bytes, the
/// empty string 0 — the length is by BYTES, never strlen-truncated.
// auto-ffi fn text_byte_len symbol=f0.text.byte_len.v1 form=pure ret=i32
pub fn text_byte_len(s: &str) -> i32 {
    s.as_bytes().len() as i32
}

/// ASCII fast-path probe (bool lane).
// auto-ffi fn text_is_ascii symbol=f0.text.is_ascii.v1 form=pure ret=bool
pub fn text_is_ascii(s: &str) -> bool {
    s.is_ascii()
}

/// First byte as i32; `None` maps to the named `empty-input` status.
// auto-ffi fn text_first_byte symbol=f0.text.first_byte.v1 form=fallible errors=empty-input:-1
pub fn text_first_byte(s: &str) -> Option<i32> {
    s.as_bytes().first().map(|b| *b as i32)
}

/// Panic-boundary probe: the provider panics on the literal `boom`.
/// The generated shim catches it in-unit and raises the exported
/// `f0_panic_flag`; the AV dispatcher converts the flag into an
/// independent host failure. No unwind ever crosses the C boundary.
// auto-ffi fn text_panic_probe symbol=f0.text.panic_probe.v1 form=pure ret=i32
pub fn text_panic_probe(s: &str) -> i32 {
    if s == "boom" {
        panic!("f0_text panic probe");
    }
    s.as_bytes().len() as i32
}
