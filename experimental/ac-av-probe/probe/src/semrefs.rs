//! Real PLAN-796 semantic-ref binding (PLAN-798 T-06, §5.6).
//!
//! Read-only consumption of the landed `ac-sema-contract` identity
//! library: the av profile's module/function identities and content
//! versions now ride the SHARED semantic identity domain instead of the
//! plan-local FNV fingerprint snapshot (retired with this task — the HIR
//! adapter itself stays). What is consumed:
//! - `logical_id` / `DeclRef` — nominal, registration-order-independent
//!   function identity (restart-stable by construction);
//! - `ContentVersion::of_canonical` — body+signature content versions
//!   from the SAME canonical dump the old fingerprint used;
//! - `SemaRef::admit` — owner/schema admission for the persisted ref
//!   envelope.
//!
//! NOT consumed (per the parallel contract): 796's inference/CT library;
//! its model producer never substitutes for AV execution. The av cache
//! keys additionally carry the av-specific axes (profile/codec/semantics
//! versions/caps) — 796 identity replaces the *content* axis only.

use ac_sema_contract::{
    logical_id, ContentVersion, DeclKind, DeclRef, PackageId, SemaRef, SEMA_PRODUCER_ID,
    SEMA_REF_SCHEMA_V1,
};

/// The av probe's package identity in the shared logical-id domain. The
/// package component keeps same-named functions of DIFFERENT owners
/// distinct (V-13: 同名异 owner).
pub fn av_package() -> PackageId {
    PackageId::new("plan798.av-probe")
}

/// Nominal logical id of one av function unit (order-independent,
/// restart-stable). Same name under a different package/module mints a
/// DIFFERENT id — no cross-owner collision.
pub fn unit_logical_id(package: &PackageId, module: &str, fn_name: &str) -> String {
    logical_id(package, module, DeclKind::Fn, fn_name)
        .as_str()
        .to_string()
}

/// The full declaration ref for one unit: nominal identity + content
/// version derived from the unit's canonical HIR dump (the same dump the
/// retired FNV fingerprint consumed).
pub fn unit_decl_ref(
    package: &PackageId,
    module: &str,
    fn_name: &str,
    canonical_dump: &[u8],
) -> DeclRef {
    DeclRef::new(
        package,
        module,
        DeclKind::Fn,
        fn_name,
        ContentVersion::of_canonical(canonical_dump),
    )
}

/// The persisted reference envelope for an av unit payload, in the shared
/// sema-ref shape (owner/schema/payload id).
pub fn unit_sema_ref(logical: &str) -> SemaRef {
    SemaRef::sema(logical)
}

/// Admission for a restored av unit ref: the envelope must come from the
/// sema producer under the v1 schema, the DECL ref must re-derive its
/// own logical id (tampered / foreign-minted refs are rejected, never
/// trusted — V-01 semantics in the shared domain), and — F798-R3-02 —
/// the envelope must name the EXACT payload fingerprint being consumed:
/// an envelope minted over a different payload does not admit this one.
pub fn admit_unit_ref(
    envelope: &SemaRef,
    decl: &DeclRef,
    payload_fp: &str,
) -> Result<(), ac_sema_contract::RefError> {
    envelope.admit(SEMA_PRODUCER_ID, SEMA_REF_SCHEMA_V1)?;
    decl.verify()?;
    if envelope.payload_id != payload_fp {
        return Err(ac_sema_contract::RefError::UnknownPayload {
            schema: envelope.schema.to_string(),
            payload_id: envelope.payload_id.clone(),
        });
    }
    Ok(())
}

/// Restore-side association admission (F798-R3-02): the persisted
/// envelope's producer/schema are checked against the REAL shared-domain
/// constants, and its payload id must equal the fingerprint of the bytes
/// this restore is about to decode and execute.
pub fn admit_restored_unit(ser: &crate::cache::SemaRefSer, payload_fp: &str) -> Result<(), String> {
    if ser.producer != SEMA_PRODUCER_ID {
        return Err(format!(
            "sema-ref producer `{}` is not the shared `{}`",
            ser.producer, SEMA_PRODUCER_ID
        ));
    }
    if ser.schema != SEMA_REF_SCHEMA_V1 {
        return Err(format!(
            "sema-ref schema `{}` is not the shared `{}`",
            ser.schema, SEMA_REF_SCHEMA_V1
        ));
    }
    if ser.payload_id != payload_fp {
        return Err(format!(
            "sema-ref payload `{}` does not bind the loaded payload `{}`",
            ser.payload_id, payload_fp
        ));
    }
    Ok(())
}

/// The content-version string for a canonical dump — the cache-key
/// content axis (replaces the retired `unit_body_fingerprint` FNV).
pub fn content_version_of(canonical_dump: &[u8]) -> String {
    ContentVersion::of_canonical(canonical_dump)
        .as_str()
        .to_string()
}
