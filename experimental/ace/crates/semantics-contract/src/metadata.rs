//! Complete meta category / coverage matrix (PLAN-796 §5.1).
//!
//! The matrix registers every meta category of the v0.6-committed
//! language surface with its owner, schema, consumers, query granularity,
//! invalidation/ref semantics, residency, CURRENT support level and the
//! future owner — so backends consume semantic facts instead of guessing.
//! "Registered" is not "implemented": [`MetaSupport::SchemaOnly`] and
//! [`MetaSupport::Future`] name the gap explicitly; nothing here may be
//! read as full-Auto reflection being delivered.

/// Support level of one meta category in this probe.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MetaSupport {
    /// Real codec + real data produced by the probe, tested.
    Implemented,
    /// Schema + admission frozen; producers are MODEL producers only
    /// (model_compute), no runtime/AVM execution claimed.
    SchemaOnly,
    /// Language face committed for v0.6 but outside this probe's scope;
    /// future owner named.
    Future,
}

impl MetaSupport {
    pub fn as_str(self) -> &'static str {
        match self {
            MetaSupport::Implemented => "implemented",
            MetaSupport::SchemaOnly => "schema-only",
            MetaSupport::Future => "future",
        }
    }
}

/// One row of the coverage matrix.
#[derive(Clone, Copy, Debug)]
pub struct CategorySpec {
    /// Stable category id.
    pub id: &'static str,
    pub support: MetaSupport,
    /// Owning stage of the semantic facts.
    pub owner: &'static str,
    /// Versioned schema that carries the category (contract-owned or the
    /// future owner's).
    pub schema: &'static str,
    pub consumers: &'static [&'static str],
    /// Query granularity: per-declaration / per-body / per-group / per-module.
    pub granularity: &'static str,
    /// How invalidation and references work for this category.
    pub refs: &'static str,
    /// Residency expectations (hot/cold, pageable).
    pub residency: &'static str,
    /// Who implements the Future face (plan-level pointer).
    pub future_owner: &'static str,
}

/// The complete category registry for the v0.6-committed surface. Rows
/// here are the auditable answer to "which meta exists and who owns it"
/// (AC-01); tests assert coverage invariants (no duplicate ids, every
/// SchemaOnly row names a schema, every Future row names an owner).
pub const CATEGORY_MATRIX: &[CategorySpec] = &[
    CategorySpec {
        id: "decl.fn",
        support: MetaSupport::Implemented,
        owner: "sema contract (796)",
        schema: "ac796/sema-page/v2:FnMeta",
        consumers: &["HIR driver", "interface queries", "ABI probe (797)"],
        granularity: "per declaration",
        refs: "DeclRef nominal identity + content version; positive dep fn-sig",
        residency: "hot in index; pageable",
        future_owner: "-",
    },
    CategorySpec {
        id: "decl.record",
        support: MetaSupport::Implemented,
        owner: "sema contract (796)",
        schema: "ac796/sema-page/v2:RecordMeta",
        consumers: &["HIR driver", "reflection", "ABI probe (797)"],
        granularity: "per declaration / per recursive group",
        refs: "DeclRef + group content version; field-set positive dep",
        residency: "hot in index; pageable",
        future_owner: "-",
    },
    CategorySpec {
        id: "decl.scope.import",
        support: MetaSupport::Implemented,
        owner: "sema core (796)",
        schema: "ac796/sema-page/v2:ModuleIndex",
        consumers: &["name lookup", "negative deps", "LSP (future)"],
        granularity: "per module",
        refs: "candidate-set fingerprint (order-free)",
        residency: "small; hot",
        future_owner: "incremental imports (787 follow-ups)",
    },
    CategorySpec {
        id: "type.logical",
        support: MetaSupport::Implemented,
        owner: "sema contract (796)",
        schema: "ac796/sema-page/v2:TypeDesc",
        consumers: &["all semantic stages", "ABI probe (797) read-only"],
        granularity: "per type description",
        refs: "portable description; records indirect by DeclRef",
        residency: "inline in pages",
        future_owner: "-",
    },
    CategorySpec {
        id: "type.fields",
        support: MetaSupport::Implemented,
        owner: "sema contract (796)",
        schema: "ac796/sema-page/v2:RecordMeta.shape",
        consumers: &["reflection", "layout (797)", "HIR driver"],
        granularity: "per record (ordered field list)",
        refs: "field-set fingerprint; new-field invalidates",
        residency: "hot for queried records",
        future_owner: "-",
    },
    CategorySpec {
        id: "fn.body.infer",
        support: MetaSupport::Implemented,
        owner: "sema core (796)",
        schema: "ac796/sema-page/v2:InferFacts",
        consumers: &["HIR checked admission", "IDE"],
        granularity: "per function body",
        refs: "body fingerprint + read signatures; never work state",
        residency: "cached per body; recomputable",
        future_owner: "-",
    },
    CategorySpec {
        id: "generic.params.constraints",
        support: MetaSupport::SchemaOnly,
        owner: "sema contract (796)",
        schema: "ac796/semaref/v1 (InstanceKey/CTRequest freeze)",
        consumers: &[
            "instantiation (future)",
            "ABI monomorphization (797 future)",
        ],
        granularity: "per template declaration",
        refs: "template description + substitution key; model constraints only",
        residency: "schema only",
        future_owner: "Design 42 work-package 2/4",
    },
    CategorySpec {
        id: "generic.instances",
        support: MetaSupport::SchemaOnly,
        owner: "sema contract (796)",
        schema: "ac796/semaref/v1:InstanceKey",
        consumers: &["CT seam", "reflection (projected instances only)"],
        granularity: "per instance key",
        refs: "InstanceKey = (template logical id, substitution fp)",
        residency: "schema only",
        future_owner: "Design 42 work-package 2",
    },
    CategorySpec {
        id: "visibility.attributes",
        support: MetaSupport::SchemaOnly,
        owner: "sema contract (796)",
        schema: "ac796/sema-page/v2 (attribute slots in FnMeta/RecordMeta)",
        consumers: &["reflection admission", "FFI surface (future)"],
        granularity: "per declaration",
        refs: "part of content version",
        residency: "inline",
        future_owner: "language attribute grammar plan",
    },
    CategorySpec {
        id: "source.provenance",
        support: MetaSupport::Implemented,
        owner: "sema core (796)",
        schema: "ac796/sema-page/v2 (spans in facts)",
        consumers: &["diagnostics", "IDE", "reflection doc/source reads"],
        granularity: "per fact",
        refs: "source-map dep; position-only edits reuse value, update provenance",
        residency: "inline",
        future_owner: "-",
    },
    CategorySpec {
        id: "ownership.effect",
        support: MetaSupport::SchemaOnly,
        owner: "sema contract (796)",
        schema: "ac796/sema-page/v2:Mode annotations",
        consumers: &["borrow checking (future)", "ABI ownership (797)"],
        granularity: "per param / per value",
        refs: "mode annotations carried in descriptions",
        residency: "inline",
        future_owner: "ownership checking plan (post-787)",
    },
    CategorySpec {
        id: "ct.request.result",
        support: MetaSupport::SchemaOnly,
        owner: "sema core (796)",
        schema: "ac796/ct-seam/v1 (CTRequest/CTResult)",
        consumers: &["instantiation", "macro pipeline (future A05)"],
        granularity: "per request",
        refs: "portable values + dep manifest; model_compute marking only",
        residency: "cached per full key; never vm_execute claims",
        future_owner: "Design 42 work-package 2 (HIR→ABC)",
    },
    CategorySpec {
        id: "generated.items",
        support: MetaSupport::Future,
        owner: "-",
        schema: "-",
        consumers: &["semantic queries continue"],
        granularity: "per generation",
        refs: "hygiene/scope/stable identity + expansion chain (design frozen only)",
        residency: "-",
        future_owner: "Design 42 work-package 4",
    },
    CategorySpec {
        id: "runtime.projection",
        support: MetaSupport::SchemaOnly,
        owner: "sema core (796)",
        schema: "ac796/ct-seam/v1:RuntimeProjectionPolicy",
        consumers: &["link retention (A09/A10)", "loader (future)"],
        granularity: "per type / per program profile",
        refs: "policy participates in query key; never addresses cached",
        residency: "policy only; adapters future",
        future_owner: "Design 42 work-package 3",
    },
    CategorySpec {
        id: "ffi.interface",
        support: MetaSupport::SchemaOnly,
        owner: "sema contract (796) — logical side only",
        schema: "ac796/sema-page/v2:TypeDesc (logical) + 797 owns physical",
        consumers: &["ABI probe (797)"],
        granularity: "per foreign interface",
        refs: "logical identity ≠ physical layout; conversion explicit",
        residency: "-",
        future_owner: "797 target layout contract",
    },
];

/// Look up one category row.
pub fn category(id: &str) -> Option<&'static CategorySpec> {
    CATEGORY_MATRIX.iter().find(|c| c.id == id)
}

/// Matrix coverage invariants (asserted in `sema_identity_matrix` tests):
/// unique ids, named schemas for non-Future rows, named future owners for
/// Future rows.
pub fn matrix_invariants() -> Result<(), &'static str> {
    let mut seen = std::collections::BTreeSet::new();
    for c in CATEGORY_MATRIX {
        if !seen.insert(c.id) {
            return Err("duplicate category id");
        }
        match c.support {
            MetaSupport::Future => {
                if c.future_owner == "-" {
                    return Err("future category without a future owner");
                }
            }
            _ => {
                if c.schema == "-" {
                    return Err("non-future category without a schema");
                }
            }
        }
        // F-796-R12-03: a category whose carrier is a sema PAGE must name
        // the CURRENT page schema — the codec only accepts that version,
        // so a row advertising an older page schema would direct
        // consumers at bytes the codec rejects. Independent schemas
        // (semaref / CT seam) and future rows ("-") are out of scope.
        if c.schema.starts_with("ac796/sema-page/")
            && !c.schema.starts_with(crate::codec::SEMA_PAGE_SCHEMA_V2)
        {
            return Err("page-carrier category names a non-current page schema");
        }
    }
    Ok(())
}
