//! The frozen v1 Rust target profile (plan §5.3). The target is not a
//! cross-platform certified promise — it pins exactly what the provider
//! emits and the certification toolchain binds against.

/// Static target-side identity of the Rust provider. Per-compilation
/// identities (source profile, HIR domain, input hash, entry) travel in the
/// file-set manifest, not here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TargetProfile {
    /// Target language.
    pub lang: &'static str,
    /// Rust edition of the emitted sources.
    pub edition: &'static str,
    /// Backend identity (this crate).
    pub backend: &'static str,
    /// Backend version — bumped when emitted semantics or file-set shape
    /// change; part of every manifest and cache key.
    pub backend_version: &'static str,
}

impl TargetProfile {
    /// The frozen v1 profile: Rust, edition 2021, Windows x64 host toolchain
    /// (the certified rustc identity is bound per-certification, not here).
    pub const fn v1() -> Self {
        TargetProfile {
            lang: "rust",
            edition: "2021",
            backend: "auto-a2x-rust",
            backend_version: env!("CARGO_PKG_VERSION"),
        }
    }
}

impl Default for TargetProfile {
    fn default() -> Self {
        Self::v1()
    }
}
