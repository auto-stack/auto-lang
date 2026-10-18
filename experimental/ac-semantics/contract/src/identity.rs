//! Semantic identity: nominal declaration refs, content versions and the
//! persisted reference envelope (PLAN-796 §5.1, contract-decision D2/D3).
//!
//! Design invariants (all tested in `sema_identity_`):
//!
//! * **Nominal, not structural.** A declaration's identity is
//!   (package, module, kind, name). Two records with identical shapes in
//!   different packages are different types and never merge.
//! * **Restart-safe.** The persisted form is a domain-separated content
//!   hash of exactly those logical components — never an intern-table
//!   index, table slot or pointer. Re-starting a process with a different
//!   declaration registration order yields identical refs.
//! * **Content versioning.** The *identity* of a declaration is separate
//!   from the *version of its definition*: `ContentVersion` hashes the
//!   canonical definition content (positions/whitespace excluded — a
//!   source-location-only change keeps the semantic value reusable while
//!   provenance updates separately).
//! * **Envelope admission.** A `SemaRef` carries its owner producer and
//!   schema; a consumer validates both before use — a ref minted by
//!   another producer or another schema revision is a controlled
//!   [`RefError`], never a silent reinterpretation.

use blake3::Hasher;

/// Producer identity of the PLAN-796 semantics probe (contract-decision
/// D3): appears in every persisted ref/page so consumers can reject
/// foreign payloads at admission.
pub const SEMA_PRODUCER_ID: &str = "ac796.sema-probe/1";

/// Schema identity of the persisted reference envelope (contract-decision
/// D3).
pub const SEMA_REF_SCHEMA_V1: &str = "ac796/semaref/v1";

/// Hash domain for logical ids (contract-decision D2).
pub const SEMA_LOGICAL_DOMAIN: &str = "ac796/logical/v1";

/// Domain-separated content hash: `hex(blake3(tag ‖ len₈(part) ‖ part …))`.
/// Length-prefixing keeps part boundaries unambiguous (`(ab,c) ≠ (a,bc)`).
pub fn domain_hash(domain: &str, parts: &[&[u8]]) -> String {
    let mut h = Hasher::new();
    h.update(SEMA_PRODUCER_ID.as_bytes());
    h.update(&[0]);
    h.update(domain.as_bytes());
    h.update(&[0]);
    for p in parts {
        h.update(&(p.len() as u64).to_le_bytes());
        h.update(p);
    }
    h.finalize().to_hex().to_string()
}

/// Logical package identity (a name chosen by the publishing project;
/// same-name declarations in different packages never merge).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PackageId(pub String);

impl PackageId {
    pub fn new(name: impl Into<String>) -> Self {
        PackageId(name.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The declaration kinds the contract models. Records are the only
/// nominal type carriers of the current profiles; functions carry
/// signatures and bodies. Discriminants are the wire tags of the page
/// codec (`DeclRef`).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum DeclKind {
    Fn = 1,
    Record = 2,
}

impl DeclKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DeclKind::Fn => "fn",
            DeclKind::Record => "record",
        }
    }
}

/// Stable logical identity of a declaration: a domain hash over
/// (package, module, kind, name). This is the *nominal* identity — equal
/// ids mean the same declaration, different ids never merge, regardless
/// of structural shape or intern order.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct LogicalId(pub String);

impl LogicalId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Mint the logical id of a declaration. Component order is fixed by the
/// call signature, so the id is independent of registration order,
/// process restarts and hash-map iteration order.
pub fn logical_id(package: &PackageId, module: &str, kind: DeclKind, name: &str) -> LogicalId {
    LogicalId(domain_hash(
        SEMA_LOGICAL_DOMAIN,
        &[
            package.as_str().as_bytes(),
            module.as_bytes(),
            kind.as_str().as_bytes(),
            name.as_bytes(),
        ],
    ))
}

/// Version of a declaration's *definition content* (not its identity):
/// a hash of the canonical semantic content — signature, parameter
/// kinds, field set in declaration order, body recipe — with source
/// positions and whitespace excluded (contract-decision D4). A
/// source-location-only edit keeps the content version; a body/signature
/// edit changes it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ContentVersion(pub String);

impl ContentVersion {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn of_canonical(canonical: &[u8]) -> Self {
        ContentVersion(domain_hash("ac796/content/v1", &[canonical]))
    }
}

/// A nominal declaration reference: the persisted, portable form.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DeclRef {
    pub logical: LogicalId,
    pub package: PackageId,
    pub module: String,
    pub kind: DeclKind,
    pub name: String,
    /// Definition content version at publication time.
    pub content: ContentVersion,
}

impl DeclRef {
    pub fn new(
        package: &PackageId,
        module: &str,
        kind: DeclKind,
        name: &str,
        content: ContentVersion,
    ) -> Self {
        DeclRef {
            logical: logical_id(package, module, kind, name),
            package: package.clone(),
            module: module.to_string(),
            kind,
            name: name.to_string(),
            content,
        }
    }

    /// Re-derive the logical id from components and check it matches the
    /// stored one — a tampered or foreign-minted ref is rejected instead
    /// of trusted (V-01: 坏图不 panic).
    pub fn verify(&self) -> Result<(), RefError> {
        let expect = logical_id(&self.package, &self.module, self.kind, &self.name);
        if expect == self.logical {
            Ok(())
        } else {
            Err(RefError::TamperedLogical {
                stored: self.logical.as_str().to_string(),
                derived: expect.as_str().to_string(),
            })
        }
    }
}

/// The persisted reference envelope (contract-decision D3): owner +
/// schema + payload id. Every consumer validates owner and schema before
/// decoding the payload.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SemaRef {
    pub owner: &'static str,
    pub schema: &'static str,
    pub payload_id: String,
}

impl SemaRef {
    pub fn sema(payload_id: impl Into<String>) -> Self {
        SemaRef {
            owner: SEMA_PRODUCER_ID,
            schema: SEMA_REF_SCHEMA_V1,
            payload_id: payload_id.into(),
        }
    }

    /// Admission check against the expected owner/schema. A mismatch is
    /// a controlled error — never a reinterpretation of foreign bytes.
    pub fn admit(&self, expect_owner: &str, expect_schema: &str) -> Result<(), RefError> {
        if self.owner != expect_owner {
            return Err(RefError::WrongOwner {
                expected: expect_owner.to_string(),
                found: self.owner.to_string(),
            });
        }
        if self.schema != expect_schema {
            return Err(RefError::WrongSchema {
                expected: expect_schema.to_string(),
                found: self.schema.to_string(),
            });
        }
        Ok(())
    }
}

/// Controlled identity errors (malformed refs are rejected, never panic).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefError {
    WrongOwner { expected: String, found: String },
    WrongSchema { expected: String, found: String },
    TamperedLogical { stored: String, derived: String },
    UnknownPayload { schema: String, payload_id: String },
    BadContentVersion(String),
}

impl std::fmt::Display for RefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RefError::WrongOwner { expected, found } => {
                write!(f, "ref owner mismatch: expected {expected}, found {found}")
            }
            RefError::WrongSchema { expected, found } => {
                write!(f, "ref schema mismatch: expected {expected}, found {found}")
            }
            RefError::TamperedLogical { stored, derived } => write!(
                f,
                "tampered logical id: stored {stored} but components derive {derived}"
            ),
            RefError::UnknownPayload { schema, payload_id } => {
                write!(f, "unknown payload {payload_id} for schema {schema}")
            }
            RefError::BadContentVersion(d) => write!(f, "bad content version: {d}"),
        }
    }
}

impl std::error::Error for RefError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn pkg(name: &str) -> PackageId {
        PackageId::new(name)
    }

    #[test]
    fn sema_identity_same_name_different_package_never_merges() {
        let a = logical_id(&pkg("app.a"), "m", DeclKind::Record, "Pair");
        let b = logical_id(&pkg("app.b"), "m", DeclKind::Record, "Pair");
        assert_ne!(a, b, "nominal identity includes the package");
        // Same package+module+kind+name is the same declaration.
        let a2 = logical_id(&pkg("app.a"), "m", DeclKind::Record, "Pair");
        assert_eq!(a, a2);
        // Kind participates: a fn `Pair` and a record `Pair` differ.
        let f = logical_id(&pkg("app.a"), "m", DeclKind::Fn, "Pair");
        assert_ne!(a, f);
    }

    #[test]
    fn sema_identity_restart_order_independent() {
        // Registration order must not leak into ids: minting the same
        // components in a different interleaving yields identical ids.
        let ids_a: Vec<LogicalId> = ["x", "y", "z"]
            .iter()
            .map(|n| logical_id(&pkg("p"), "m", DeclKind::Fn, n))
            .collect();
        let mut ids_b: Vec<LogicalId> = ["z", "x", "y"]
            .iter()
            .map(|n| logical_id(&pkg("p"), "m", DeclKind::Fn, n))
            .collect();
        ids_b.sort();
        let mut ids_a = ids_a;
        ids_a.sort();
        assert_eq!(ids_a, ids_b);
    }

    #[test]
    fn sema_identity_decl_ref_tamper_rejected() {
        let r = DeclRef::new(
            &pkg("p"),
            "m",
            DeclKind::Record,
            "T",
            ContentVersion::of_canonical(b"canonical"),
        );
        assert!(r.verify().is_ok());
        let mut bad = r.clone();
        bad.logical = LogicalId("deadbeef".into());
        assert!(matches!(
            bad.verify(),
            Err(RefError::TamperedLogical { .. })
        ));
    }

    #[test]
    fn sema_identity_envelope_admission() {
        let r = SemaRef::sema("abc");
        assert!(r.admit(SEMA_PRODUCER_ID, SEMA_REF_SCHEMA_V1).is_ok());
        assert!(matches!(
            r.admit("other-producer", SEMA_REF_SCHEMA_V1),
            Err(RefError::WrongOwner { .. })
        ));
        assert!(matches!(
            r.admit(SEMA_PRODUCER_ID, "ac796/other/v9"),
            Err(RefError::WrongSchema { .. })
        ));
    }

    #[test]
    fn sema_identity_domain_separation() {
        // Length-prefixing and domain separation: no cross-family replay.
        let a = domain_hash("d1", &[b"ab", b"c"]);
        let b = domain_hash("d1", &[b"a", b"bc"]);
        assert_ne!(a, b);
        let c = domain_hash("d2", &[b"ab", b"c"]);
        assert_ne!(a, c);
    }
}
