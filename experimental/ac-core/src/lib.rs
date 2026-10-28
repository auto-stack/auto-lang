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

/// Package version of this ac-core build (PLAN-788 toolchain identity:
/// cache keys and worker reports bind against the compiler that ran, not a
/// caller-supplied constant).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub mod descriptor;
pub mod hir;
pub mod link;
pub mod native;
pub mod source;
pub mod text_resources;
pub mod verify;

/// The versioned core descriptor this build accepts (PLAN-741). Any document
/// claiming a different schema/revision/profile identity is rejected at bind
/// time.
/// PLAN-828 §5.8/D-01 producer observation: per-function BODY-work
/// counts recorded at the REAL resolve/typecheck/plan/bind entries.
/// Recording is opt-in per [`capture_body_work`] window; a structural
/// gate reads the producer's own counts (by function name) instead of
/// reconstructing work from a caller. The interface directory (types /
/// body-less declaration scaffolds) is deliberately NOT counted — only
/// body passes are.
pub mod body_work {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    /// One function's body-work counts within an observation window.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct BodyWork {
        pub resolve: u64,
        pub typecheck: u64,
        pub plan: u64,
        pub bind: u64,
    }

    thread_local! {
        static COUNTS: RefCell<Option<BTreeMap<String, BodyWork>>> =
            RefCell::new(None);
    }

    fn record(kind: u8, name: &str) {
        COUNTS.with(|c| {
            if let Some(m) = c.borrow_mut().as_mut() {
                let e = m.entry(name.to_string()).or_default();
                match kind {
                    0 => e.resolve += 1,
                    1 => e.typecheck += 1,
                    2 => e.plan += 1,
                    _ => e.bind += 1,
                }
            }
        });
    }

    /// Public producer hook (kind 0 = resolve walk of one body).
    pub fn record_resolve(name: &str) {
        record(0, name);
    }
    /// Public producer hook (kind 1 = typecheck walk of one body).
    pub fn record_typecheck(name: &str) {
        record(1, name);
    }
    /// Public producer hook (kind 2 = emit plan of one body).
    pub fn record_plan(name: &str) {
        record(2, name);
    }
    /// Public producer hook (kind 3 = bind of one body document).
    pub fn record_bind(name: &str) {
        record(3, name);
    }

    /// Run `f` inside a body-work observation window and return its
    /// result plus the producer-recorded per-function counts.
    pub fn capture_body_work<T>(f: impl FnOnce() -> T) -> (T, BTreeMap<String, BodyWork>) {
        COUNTS.with(|c| *c.borrow_mut() = Some(BTreeMap::new()));
        let out = f();
        let m = COUNTS.with(|c| c.borrow_mut().take()).unwrap_or_default();
        (out, m)
    }
}

pub const DESCRIPTOR_SCHEMA: &str = include_str!("../schema/core-i32.atom");

/// The versioned scalar descriptor this build accepts (PLAN-767): the
/// core-i32 surface plus exact sub/div/rem, full signed order, polymorphic
/// eq/ne, unary neg/not and lazy boolean and/or.
pub const SCALAR_DESCRIPTOR_SCHEMA: &str = include_str!("../schema/scalar-i32.atom");

/// The versioned text descriptor this build accepts (PLAN-770): the
/// scalar-i32 surface plus str/Str/char under the PLAN-774 data-resource
/// contract.
pub const TEXT_DESCRIPTOR_SCHEMA: &str = include_str!("../schema/text.atom");

/// The versioned record descriptor this build accepts (PLAN-771): the
/// scalar-i32 surface plus nominal records, record construction, nested
/// field paths, view/mut borrows (774 D-04) and type-bound methods.
pub const RECORD_DESCRIPTOR_SCHEMA: &str = include_str!("../schema/record.atom");

/// The versioned text-record descriptor this build accepts (PLAN-782): the
/// union of the text and record surfaces — owned `Str` fields inside
/// nominal records, explicit aggregate move/deep clone, under the PLAN-774
/// data-resource contract.
pub const TEXT_RECORD_DESCRIPTOR_SCHEMA: &str = include_str!("../schema/text-record.atom");

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

/// Parse and return the embedded text descriptor (PLAN-770).
pub fn text_descriptor() -> &'static descriptor::Descriptor {
    use std::sync::OnceLock;
    static DESC: OnceLock<descriptor::Descriptor> = OnceLock::new();
    DESC.get_or_init(|| {
        descriptor::Descriptor::parse(TEXT_DESCRIPTOR_SCHEMA)
            .expect("embedded text-i32 descriptor must parse")
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

/// The versioned entry-unit descriptor this build accepts (PLAN-862): the
/// scalar-i32 surface plus the ENTRY surface — a Unit (oid) function
/// result and the unit value expression — under the frozen entry matrix
/// (docs/plans/attachments/862/work/entry-contract.md).
pub const ENTRY_UNIT_DESCRIPTOR_SCHEMA: &str = include_str!("../schema/entry-unit.atom");

/// Parse and return the embedded text-record descriptor (PLAN-782).
pub fn text_record_descriptor() -> &'static descriptor::Descriptor {
    use std::sync::OnceLock;
    static DESC: OnceLock<descriptor::Descriptor> = OnceLock::new();
    DESC.get_or_init(|| {
        descriptor::Descriptor::parse(TEXT_RECORD_DESCRIPTOR_SCHEMA)
            .expect("embedded text-record-i32 descriptor must parse")
    })
}

/// Parse and return the embedded entry-unit descriptor (PLAN-862).
pub fn entry_unit_descriptor() -> &'static descriptor::Descriptor {
    use std::sync::OnceLock;
    static DESC: OnceLock<descriptor::Descriptor> = OnceLock::new();
    DESC.get_or_init(|| {
        descriptor::Descriptor::parse(ENTRY_UNIT_DESCRIPTOR_SCHEMA)
            .expect("embedded entry-unit descriptor must parse")
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
        (text_descriptor(), "text"),
        (record_descriptor(), "record"),
        (text_record_descriptor(), "text-record"),
        (entry_unit_descriptor(), "entry-unit"),
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
