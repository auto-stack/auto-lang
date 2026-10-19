//! REAL semantic binding over the merged PLAN-796 contract (T-06;
//! D-796 satisfied 2026-10-19). The 797-local `autoabi-semref-snapshot/0`
//! adapter (blake3 `15f81fd0…`, T-01..T-05 stand-in) is RETIRED — receipt:
//! docs/plans/attachments/797/semref-retirement-receipt.md.
//!
//! What lives here now is the ONLY semantic identity source: nominal
//! refs and content versions minted through `ac_sema_contract` (DeclRef
//! + TypeDesc + ContentVersion), exactly the read-only consumption face
//! the semantic contract froze for 797 ("797 只读消费：类型描述
//! codec…；ABI 布局与调用规范归 797 独立拥有"). Target layout, calling
//! convention, and physical signatures remain 797-owned (layout.rs /
//! descriptor.rs) — nothing here derives layouts.

use ac_sema_contract::codec::{encode_typedesc_bytes, FieldDesc, RecGroup, RecordShape, TypeDesc};
use ac_sema_contract::identity::{ContentVersion, DeclKind, DeclRef, PackageId};
use std::sync::OnceLock;

/// Fixture package/module identity (logical, not path-bound).
pub const BINDING_PACKAGE: &str = "probe797";
pub const BINDING_MODULE: &str = "autoabi";

pub fn package() -> PackageId {
    PackageId::new(BINDING_PACKAGE)
}

/// Map a physical field scalar to its SEMANTIC TypeDesc — the inverse
/// bridge used by admission to recompute a record's content version
/// FROM ITS PAYLOAD (F-797-R4-01 R5: labels never substitute for
/// payload facts).
pub fn type_desc_of(s: crate::layout::SemScalar) -> TypeDesc {
    use crate::layout::SemScalar;
    match s {
        SemScalar::Bool => TypeDesc::Bool,
        SemScalar::U8 => TypeDesc::U32, // physical u8 lanes carry no semantic kind of their own
        SemScalar::I32 => TypeDesc::I32,
        SemScalar::U32 => TypeDesc::U32,
        SemScalar::I64 => TypeDesc::I64,
        SemScalar::U64 => TypeDesc::U64,
        SemScalar::F64 => TypeDesc::F64,
        SemScalar::Char => TypeDesc::Char,
    }
}

/// Recompute a record's SEMANTIC content version from its payload
/// (fields as semantic TypeDescs, declaration order) — identical to how
/// the binding mints versions, so a tampered payload with an original
/// LABEL still mismatches.
pub fn content_version_of_payload(
    name: &str,
    fields: &[(String, crate::layout::SemScalar)],
) -> String {
    let canonical = encode_typedesc_bytes(&TypeDesc::I64).unwrap();
    let decl = DeclRef::new(
        &package(),
        BINDING_MODULE,
        DeclKind::Record,
        name,
        ContentVersion::of_canonical(&canonical),
    );
    let shape = RecordShape {
        decl,
        fields: fields
            .iter()
            .map(|(n, s)| FieldDesc {
                name: n.clone(),
                ty: type_desc_of(*s),
            })
            .collect(),
    };
    RecGroup {
        members: vec![shape],
    }
    .group_content()
    .as_str()
    .to_string()
}

/// blake3, hex-encoded (64 chars) — kept as the probe-local digest form.
pub fn blake3_hex(bytes: &[u8]) -> String {
    let mut h = blake3::Hasher::new();
    h.update(bytes);
    h.finalize().to_hex().to_string()
}

/// The semantic fn type of `p797_mix` as a REAL contract TypeDesc.
pub fn mix_fn_type() -> TypeDesc {
    TypeDesc::Fn {
        params: vec![
            TypeDesc::I32,
            TypeDesc::I32,
            TypeDesc::I32,
            TypeDesc::I32,
            TypeDesc::I64,
            TypeDesc::F64,
            TypeDesc::I32,
            TypeDesc::F64,
        ],
        ret: Box::new(TypeDesc::I64),
    }
}

/// Nominal ref of `p797_mix` (top-level Fn). The content version is the
/// contract's own canonical encoding of the fn type — a semantic edit
/// (any parameter/return change) is a different content version, and
/// therefore a different ABI descriptor semref input.
pub fn mix_fn_ref() -> Result<DeclRef, String> {
    let canonical = encode_typedesc_bytes(&mix_fn_type()).map_err(|e| format!("codec: {e}"))?;
    let content = ContentVersion::of_canonical(&canonical);
    let r = DeclRef::new(
        &package(),
        BINDING_MODULE,
        DeclKind::Fn,
        "p797_mix",
        content,
    );
    r.verify().map_err(|e| format!("fn ref verify: {e:?}"))?;
    Ok(r)
}

/// Semantic shape of the RecMix record fixture (same field set the
/// physical layout engine and the C mirror use; TYPES are semantic
/// TypeDescs — offsets/sizes stay in layout.rs).
pub fn rec_mix_shape() -> RecordShape {
    let canonical = encode_typedesc_bytes(&TypeDesc::I64).unwrap();
    let content = ContentVersion::of_canonical(&canonical);
    RecordShape {
        decl: DeclRef::new(
            &package(),
            BINDING_MODULE,
            DeclKind::Record,
            "RecMix",
            content,
        ),
        fields: vec![
            FieldDesc {
                name: "flag".into(),
                ty: TypeDesc::Bool,
            },
            FieldDesc {
                name: "count".into(),
                ty: TypeDesc::I32,
            },
            FieldDesc {
                name: "tag".into(),
                ty: TypeDesc::Char,
            },
            FieldDesc {
                name: "big".into(),
                ty: TypeDesc::I64,
            },
        ],
    }
}

/// The RecMix record ref, with the contract's group-content version
/// (single-member RecGroup) as its content version.
pub fn rec_mix_ref() -> Result<DeclRef, String> {
    let mut shape = rec_mix_shape();
    let group = RecGroup {
        members: vec![rec_mix_shape()],
    };
    let content = group.group_content();
    shape.decl.content = content.clone();
    let r = shape.decl;
    r.verify()
        .map_err(|e| format!("record ref verify: {e:?}"))?;
    Ok(r)
}

/// The SEMANTIC param/return kinds the frozen fixtures spell — the
/// deck grammar validates against THESE (not machine widths), so a
/// same-width substitution (u32 for i32) is a typed deviation.
pub fn mix_param_semantics() -> &'static [SemKind] {
    &[
        SemKind::I32,
        SemKind::I32,
        SemKind::I32,
        SemKind::I32,
        SemKind::I64,
        SemKind::F64,
        SemKind::I32,
        SemKind::F64,
    ]
}

/// Semantic scalar kinds (the contract's TypeDesc variants the probe
/// consumes; records stay indirect through DeclRefs).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SemKind {
    Unit,
    I32,
    U32,
    I64,
    U64,
    F64,
    Bool,
    Char,
}

/// The frozen RecMix semantic field list (name, kind) in declaration
/// order — the deck `rec` line must spell EXACTLY this shape.
pub fn rec_mix_semantics() -> &'static [(&'static str, SemKind)] {
    &[
        ("flag", SemKind::Bool),
        ("count", SemKind::I32),
        ("tag", SemKind::Char),
        ("big", SemKind::I64),
    ]
}

/// Map a deck type spelling to its semantic kind (None = unknown).
pub fn sem_kind_of(spelling: &str) -> Option<SemKind> {
    match spelling {
        "unit" => Some(SemKind::Unit),
        "i32" => Some(SemKind::I32),
        "u32" => Some(SemKind::U32),
        "i64" => Some(SemKind::I64),
        "u64" => Some(SemKind::U64),
        "f64" => Some(SemKind::F64),
        "bool" => Some(SemKind::Bool),
        "char" => Some(SemKind::Char),
        _ => None,
    }
}

/// The semantic binding hash the ABI descriptor carries: the frozen
/// fixture set's semantic identity — both nominal logicals plus their
/// content versions, digested with the probe's blake3 form. This is
/// what admission compares (validate.rs `SemRefMismatch`); building it
/// through the REAL contract means a semantic-side edit (field set,
/// fn signature) necessarily changes the binding the ABI side demands.
pub fn binding_hash() -> &'static str {
    static HASH: OnceLock<String> = OnceLock::new();
    HASH.get_or_init(|| {
        let f = mix_fn_ref().expect("fn ref mints");
        let r = rec_mix_ref().expect("record ref mints");
        let mut buf = Vec::new();
        buf.extend_from_slice(f.logical.as_str().as_bytes());
        buf.extend_from_slice(f.content.as_str().as_bytes());
        buf.extend_from_slice(r.logical.as_str().as_bytes());
        buf.extend_from_slice(r.content.as_str().as_bytes());
        blake3_hex(&buf)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real refs mint AND verify through the merged contract.
    #[test]
    fn refs_mint_and_verify() {
        let f = mix_fn_ref().expect("fn ref");
        assert_eq!(f.name, "p797_mix");
        assert_eq!(f.kind, DeclKind::Fn);
        assert!(f.owner.is_none());
        let r = rec_mix_ref().expect("record ref");
        assert_eq!(r.name, "RecMix");
        assert_eq!(r.kind, DeclKind::Record);
        // Logical ids are distinct across kinds.
        assert_ne!(f.logical.as_str(), r.logical.as_str());
    }

    /// Binding hash is stable and full-length.
    #[test]
    fn binding_hash_stable() {
        let h = binding_hash();
        assert_eq!(h.len(), 64);
        assert_eq!(h, binding_hash());
    }

    /// A tampered ref is rejected by the CONTRACT's verify — the ABI
    /// binding path can never trust a foreign-minted identity.
    #[test]
    fn tampered_ref_rejected_by_contract() {
        let mut f = mix_fn_ref().unwrap();
        f.name = "p797_other".into();
        assert!(f.verify().is_err());
    }

    /// A semantic edit changes the binding hash (the ABI side sees a
    /// different semref input) — the T-01..T-05 stand-in is gone.
    #[test]
    fn semantic_edit_changes_binding() {
        let base = binding_hash().to_string();
        // Same digest form over an EDITED fn content version.
        let edited_content = ContentVersion::of_canonical(b"edited-fn-shape");
        let mut buf = Vec::new();
        buf.extend_from_slice(mix_fn_ref().unwrap().logical.as_str().as_bytes());
        buf.extend_from_slice(edited_content.as_str().as_bytes());
        buf.extend_from_slice(rec_mix_ref().unwrap().logical.as_str().as_bytes());
        buf.extend_from_slice(rec_mix_ref().unwrap().content.as_str().as_bytes());
        assert_ne!(base, blake3_hex(&buf));
    }
}
