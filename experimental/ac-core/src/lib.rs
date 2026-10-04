//! auto-ac-prototype — PLAN-741 AC closed-loop prototype.
//!
//! Pipeline: Atom text → spanned syntax tree → core-descriptor binder →
//! unchecked typed HIR → semantic verifier → checked HIR → native lowering
//! (Cranelift) → COFF object → platform link → Windows PE.
//!
//! This crate is a standalone prototype workspace: it must not depend on any
//! auto-lang / auto-atom / auto-val product crates.

pub mod atom_text;
pub mod descriptor;
pub mod hir;
pub mod link;
pub mod native;
pub mod verify;

/// The versioned core descriptor this build accepts. Any document claiming a
/// different schema/revision/profile identity is rejected at bind time.
pub const DESCRIPTOR_SCHEMA: &str = include_str!("../schema/core-i32.atom");

/// Parse and return the embedded descriptor (validated against itself once at
/// startup).
pub fn descriptor() -> &'static descriptor::Descriptor {
    use std::sync::OnceLock;
    static DESC: OnceLock<descriptor::Descriptor> = OnceLock::new();
    DESC.get_or_init(|| {
        descriptor::Descriptor::parse(DESCRIPTOR_SCHEMA)
            .expect("embedded core-i32 descriptor must parse")
    })
}

/// Convenience: bind a source document with the embedded descriptor.
pub fn bind_source(source: &str) -> Result<hir::Bundle, Vec<atom_text::Diagnostic>> {
    descriptor::bind(descriptor(), source)
}
