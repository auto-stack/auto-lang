//! auto-ac-prototype — PLAN-741 AC closed-loop prototype, extended by
//! PLAN-745 (source core-i32 profile) and PLAN-767 (scalar-i32 profile).
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
pub mod source;
pub mod verify;

/// The versioned core descriptor this build accepts (PLAN-741). Any document
/// claiming a different schema/revision/profile identity is rejected at bind
/// time.
pub const DESCRIPTOR_SCHEMA: &str = include_str!("../schema/core-i32.atom");

/// The versioned scalar descriptor this build accepts (PLAN-767): the
/// core-i32 surface plus exact sub/div/rem, full signed order, polymorphic
/// eq/ne, unary neg/not and lazy boolean and/or.
pub const SCALAR_DESCRIPTOR_SCHEMA: &str = include_str!("../schema/scalar-i32.atom");

/// The versioned record descriptor this build accepts (PLAN-771): the
/// scalar-i32 surface plus nominal records, record construction, nested
/// field paths, view/mut borrows (774 D-04) and type-bound methods.
pub const RECORD_DESCRIPTOR_SCHEMA: &str = include_str!("../schema/record.atom");

/// Parse and return the embedded core descriptor (validated against itself
/// once at startup).
pub fn descriptor() -> &'static descriptor::Descriptor {
    use std::sync::OnceLock;
    static DESC: OnceLock<descriptor::Descriptor> = OnceLock::new();
    DESC.get_or_init(|| {
        descriptor::Descriptor::parse(DESCRIPTOR_SCHEMA)
            .expect("embedded core-i32 descriptor must parse")
    })
}

/// Parse and return the embedded scalar descriptor (PLAN-767).
pub fn scalar_descriptor() -> &'static descriptor::Descriptor {
    use std::sync::OnceLock;
    static DESC: OnceLock<descriptor::Descriptor> = OnceLock::new();
    DESC.get_or_init(|| {
        descriptor::Descriptor::parse(SCALAR_DESCRIPTOR_SCHEMA)
            .expect("embedded scalar-i32 descriptor must parse")
    })
}

/// Parse and return the embedded record descriptor (PLAN-771).
pub fn record_descriptor() -> &'static descriptor::Descriptor {
    use std::sync::OnceLock;
    static DESC: OnceLock<descriptor::Descriptor> = OnceLock::new();
    DESC.get_or_init(|| {
        descriptor::Descriptor::parse(RECORD_DESCRIPTOR_SCHEMA)
            .expect("embedded record-i32 descriptor must parse")
    })
}

/// Convenience: bind a source document with the embedded core descriptor.
pub fn bind_source(source: &str) -> Result<hir::Bundle, Vec<atom_text::Diagnostic>> {
    descriptor::bind(descriptor(), source)
}

/// Bind a document by its declared identity: the root `hir` header's
/// schema/revision/profile triple selects the registered descriptor
/// (PLAN-767 §5.1). An unregistered triple is a controlled bind diagnostic
/// that names the accepted identities — never a guess, never a fallback.
pub fn bind_document(source: &str) -> Result<hir::Bundle, Vec<atom_text::Diagnostic>> {
    let registered: &[(&'static descriptor::Descriptor, &str)] = &[
        (descriptor(), "core"),
        (scalar_descriptor(), "scalar"),
        (record_descriptor(), "record"),
    ];
    let header = match descriptor::peek_identity(source) {
        Ok(h) => h,
        Err(d) => return Err(vec![d]),
    };
    for (desc, _) in registered {
        if desc.schema == header.schema
            && desc.revision == header.revision
            && desc.profile == header.profile
        {
            return descriptor::bind(desc, source);
        }
    }
    let accepted: Vec<String> = registered
        .iter()
        .map(|(d, _)| format!("{}/{}/{}", d.schema, d.revision, d.profile))
        .collect();
    Err(vec![atom_text::Diagnostic::new(
        atom_text::Stage::Bind,
        "bind.descriptor-unknown",
        format!(
            "no registered descriptor for identity `{}/{}/{} (accepted: {})",
            header.schema,
            header.revision,
            header.profile,
            accepted.join(", ")
        ),
        header.span,
    )])
}
