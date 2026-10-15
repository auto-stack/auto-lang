//! Unified identity and reference model (PLAN-783 §5.1 / Design 40 §4).
//!
//! Lookup keys, content identities and business identities are separate
//! types; content hashes are domain-separated so a hash from one family can
//! never be replayed as another (`hash domain` includes the family, an
//! algorithm identity and a version). Content identity never embeds the
//! source location; project registration is a separate root-table concern.

use crate::schema;
use blake3::Hasher;
use std::fmt;

/// Domain-separated content hash: `hex(blake3(tag ‖ payload…))`.
/// The tag is `ac783/<family>/v<n>` — never a bare byte hash.
pub fn domain_hash(family: &str, parts: &[&[u8]]) -> String {
    let mut h = Hasher::new();
    h.update(crate::HASH_ALGO.as_bytes());
    h.update(&[0]);
    h.update(family.as_bytes());
    h.update(&[0]);
    for p in parts {
        h.update(&(p.len() as u64).to_le_bytes());
        h.update(p);
    }
    h.finalize().to_hex().to_string()
}

macro_rules! id_type {
    ($(#[$m:meta])* $name:ident, $prefix:literal) => {
        $(#[$m])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub struct $name(pub String);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl $name {
            /// Content identity from already-hashed domain parts.
            pub fn from_parts(parts: &[&[u8]]) -> Self {
                Self(format!(concat!($prefix, "{}"), domain_hash(concat!($prefix, "/v1"), parts)))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

id_type!(
    /// Content identity of a raw source snapshot (A00): the bytes plus the
    /// logical module name — never the filesystem path.
    SourceSnapshotId,
    "snap"
);
id_type!(
    /// Published-artifact content identity (manifest digest).
    ArtifactId,
    "art"
);
id_type!(
    /// Immutable chunk content identity (physical bytes).
    ChunkId,
    "chk"
);

/// A staged lookup key: stage (layer) + named fingerprint components.
/// Components are sorted by name before hashing so key order is canonical.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QueryKey {
    pub layer: &'static str,
    pub components: Vec<(String, String)>,
}

impl QueryKey {
    pub fn new(layer_spec: &schema::LayerSpec, mut components: Vec<(String, String)>) -> Self {
        components.sort_by(|a, b| a.0.cmp(&b.0));
        QueryKey { layer: layer_spec.id, components }
    }

    /// Canonical textual form (stable, diagnostic-friendly).
    pub fn canonical(&self) -> String {
        let mut s = format!("{}[", self.layer);
        for (i, (k, v)) in self.components.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!("{}={}", k, v));
        }
        s.push(']');
        s
    }

    /// Hash form for index lookups (domain: query-key of the layer).
    pub fn hash(&self) -> String {
        let parts: Vec<Vec<u8>> = std::iter::once(self.layer.as_bytes().to_vec())
            .chain(self.components.iter().flat_map(|(k, v)| vec![k.clone().into_bytes(), v.clone().into_bytes()]))
            .collect();
        let refs: Vec<&[u8]> = parts.iter().map(|v| v.as_slice()).collect();
        domain_hash(&format!("qk/{}", self.layer), &refs)
    }

    pub fn component(&self, name: &str) -> Option<&str> {
        self.components.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }
}

/// Reference kind between artifacts (Design 40 §4.2). Only `RequiredData`
/// keeps targets alive; provenance never pins upstream history.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefKind {
    RequiredData,
    Recomputable,
    InvalidationFingerprint,
    Provenance,
}

impl RefKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RefKind::RequiredData => "required",
            RefKind::Recomputable => "recomputable",
            RefKind::InvalidationFingerprint => "invalfp",
            RefKind::Provenance => "provenance",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "required" => Some(RefKind::RequiredData),
            "recomputable" => Some(RefKind::Recomputable),
            "invalfp" => Some(RefKind::InvalidationFingerprint),
            "provenance" => Some(RefKind::Provenance),
            _ => None,
        }
    }
}

/// Table category of an ID inside an artifact (owner-scoped).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefCategory {
    Type,
    Def,
    Body,
    Local,
    Expr,
    Place,
    Block,
    Loop,
}

impl RefCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            RefCategory::Type => "type",
            RefCategory::Def => "def",
            RefCategory::Body => "body",
            RefCategory::Local => "local",
            RefCategory::Expr => "expr",
            RefCategory::Place => "place",
            RefCategory::Block => "block",
            RefCategory::Loop => "loop",
        }
    }
}

/// An ID reference scoped to its owning artifact, with the bounds it must
/// be checked against by any reader.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LocalRef {
    pub owner: ArtifactId,
    pub category: RefCategory,
    pub index: u32,
}

/// A cross-artifact reference: target artifact + expected schema + the
/// owner-scoped table slot it names. Readers must reject wrong owner,
/// wrong category and out-of-bounds indices (tested in `admission_`).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExternalRef {
    pub artifact: ArtifactId,
    pub expected_schema: &'static str,
    pub category: RefCategory,
    pub index: u32,
}

/// Controlled identity errors (never a panic on malformed input).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentityError {
    BadRef { what: &'static str, detail: String },
    UnknownLayer(String),
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentityError::BadRef { what, detail } => write!(f, "bad reference ({}): {}", what, detail),
            IdentityError::UnknownLayer(l) => write!(f, "unknown layer: {}", l),
        }
    }
}

impl std::error::Error for IdentityError {}

/// Validate an external ref against a loaded artifact's table bounds.
pub fn validate_external_ref(
    r: &ExternalRef,
    actual_schema: &str,
    bounds: &[(RefCategory, u32)],
) -> Result<(), IdentityError> {
    if r.expected_schema != actual_schema {
        return Err(IdentityError::BadRef {
            what: "schema",
            detail: format!("expected {} got {}", r.expected_schema, actual_schema),
        });
    }
    let len = bounds
        .iter()
        .find(|(c, _)| *c == r.category)
        .map(|(_, n)| *n)
        .unwrap_or(0);
    if r.index >= len {
        return Err(IdentityError::BadRef {
            what: "index",
            detail: format!("{} index {} out of bounds (len {})", r.category.as_str(), r.index, len),
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Concrete key builders for the vertical line (all domain-separated)
// ---------------------------------------------------------------------------

/// A00 source snapshot: content + logical module name (no path).
pub fn source_snapshot(source_bytes: &[u8], logical_name: &str) -> SourceSnapshotId {
    SourceSnapshotId::from_parts(&[source_bytes, logical_name.as_bytes()])
}

/// A04 declaration-summary lookup key (per module, per profile).
pub fn decl_summary_key(profile: &str, snapshot: &SourceSnapshotId) -> QueryKey {
    QueryKey::new(
        schema::layer("A04").unwrap(),
        vec![
            ("profile".into(), profile.into()),
            ("schema".into(), schema::layer("A04").unwrap().schema_id.into()),
            ("snapshot".into(), snapshot.as_str().into()),
            ("frontend-version".into(), "ac-core-parse-resolve-typecheck".into()),
        ],
    )
}

/// A07 HIR bundle lookup key.
pub fn hir_bundle_key(
    profile: &str,
    hir_domain: &str,
    capabilities: &[String],
    snapshot: &SourceSnapshotId,
    codec_version: u16,
) -> QueryKey {
    let mut caps: Vec<String> = capabilities.to_vec();
    caps.sort();
    QueryKey::new(
        schema::layer("A07").unwrap(),
        vec![
            ("profile".into(), profile.into()),
            ("hir-domain".into(), hir_domain.into()),
            ("capabilities".into(), caps.join(",")),
            ("snapshot".into(), snapshot.as_str().into()),
            ("codec-version".into(), codec_version.to_string()),
            ("binder-version".into(), "ac-core-descriptor-bind".into()),
        ],
    )
}

/// A09 object lookup key. `hir_artifact` is the full downstream input:
/// a body-only edit produces a different HIR artifact id and therefore a
/// different object key, even with identical signatures (V-01/V-05).
pub fn object_key(
    hir_artifact: &ArtifactId,
    target: &str,
    backend_id: &str,
    entry: &str,
) -> QueryKey {
    QueryKey::new(
        schema::layer("A09").unwrap(),
        vec![
            ("hir-artifact".into(), hir_artifact.as_str().into()),
            ("target".into(), target.into()),
            ("backend".into(), backend_id.into()),
            ("entry".into(), entry.into()),
        ],
    )
}

/// The only target this probe can lower/link: the host triple ac-core's
/// Cranelift lowering actually builds (Windows x64 / MSVC / rust-lld).
pub const SUPPORTED_TARGET: &str = "x86_64-pc-windows-msvc";

/// Backend identity of the ac-core lowering path.
pub const BACKEND_ID: &str = "cranelift-0.126/lower_object";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_domain_separation() {
        // Same payload, different families → different ids; family bytes
        // are not a suffix attack surface (length-prefixed).
        let a = domain_hash("alpha", &[b"xx"]);
        let b = domain_hash("beta", &[b"xx"]);
        assert_ne!(a, b);
        // Length-prefixing: (ab, c) ≠ (a, bc)
        let c = domain_hash("f", &[b"ab", b"c"]);
        let d = domain_hash("f", &[b"a", b"bc"]);
        assert_ne!(c, d);
    }

    #[test]
    fn snapshot_identity_ignores_path() {
        let s1 = source_snapshot(b"fn main() int { return 0 }", "app");
        let s2 = source_snapshot(b"fn main() int { return 0 }", "app");
        assert_eq!(s1, s2);
        let s3 = source_snapshot(b"fn main() int { return 1 }", "app");
        assert_ne!(s1, s3);
        let s4 = source_snapshot(b"fn main() int { return 0 }", "other");
        assert_ne!(s1, s4, "logical module name is part of identity");
    }

    #[test]
    fn query_key_canonical_order_independent() {
        let layer = schema::layer("A07").unwrap();
        let k1 = QueryKey::new(layer, vec![("a".into(), "1".into()), ("b".into(), "2".into())]);
        let k2 = QueryKey::new(layer, vec![("b".into(), "2".into()), ("a".into(), "1".into())]);
        assert_eq!(k1.canonical(), k2.canonical());
        assert_eq!(k1.hash(), k2.hash());
        // Different layers never collide on the same components.
        let layer4 = schema::layer("A04").unwrap();
        let k3 = QueryKey::new(layer4, k1.components.clone());
        assert_ne!(k1.hash(), k3.hash());
    }

    #[test]
    fn external_ref_validation_rejects_bad_owner_and_bounds() {
        let art = ArtifactId::from_parts(&[b"x"]);
        let r = ExternalRef {
            artifact: art.clone(),
            expected_schema: "ac783/a07-hir/v1",
            category: RefCategory::Expr,
            index: 3,
        };
        assert!(validate_external_ref(&r, "ac783/a07-hir/v1", &[(RefCategory::Expr, 7)]).is_ok());
        assert!(validate_external_ref(&r, "ac783/a09-object/v1", &[(RefCategory::Expr, 7)]).is_err());
        assert!(validate_external_ref(&r, "ac783/a07-hir/v1", &[(RefCategory::Expr, 3)]).is_err());
        assert!(validate_external_ref(&r, "ac783/a07-hir/v1", &[(RefCategory::Def, 9)]).is_err());
    }
}
