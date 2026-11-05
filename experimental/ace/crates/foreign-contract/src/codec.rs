//! Strict persisted-artifact codecs (937 plan §5.4 bridge restore).
//!
//! Bridge artifacts (interface facts, physical plans) round-trip through
//! these codecs ONLY: versioned format ids, `deny_unknown_fields`
//! structs (an unknown field from a future/foreign producer is a
//! rejection, never a silent drop), bounded decode size, and wrong-
//! format rejection. Restore is fail-closed: a truncated, oversized,
//! or wrong-schema payload is a named error and the persisted bridge
//! page is NOT treated as restorable.

use crate::facts::InterfaceFacts;
use crate::plan::PhysicalPlan;
use crate::typed::TypedDecls;

/// Hard decode cap: a persisted facts/plan document larger than this is
/// corrupt (the honest producers emit documents orders of magnitude
/// smaller).
pub const MAX_CODEC_BYTES: usize = 1024 * 1024;

#[derive(Debug)]
pub struct CodecError(pub String);

impl std::fmt::Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "foreign-contract codec: {}", self.0)
    }
}

impl std::error::Error for CodecError {}

fn check_len(bytes: &[u8]) -> Result<(), CodecError> {
    if bytes.len() > MAX_CODEC_BYTES {
        return Err(CodecError(format!(
            "payload {} bytes exceeds the {} byte cap",
            bytes.len(),
            MAX_CODEC_BYTES
        )));
    }
    Ok(())
}

/// Interface facts codec (`autoabi.contract.facts/1`).
pub mod facts_codec {
    use super::*;

    pub fn encode(f: &InterfaceFacts) -> Result<Vec<u8>, CodecError> {
        f.validate().map_err(CodecError)?;
        serde_json::to_vec(f).map_err(|e| CodecError(e.to_string()))
    }

    pub fn decode(bytes: &[u8]) -> Result<InterfaceFacts, CodecError> {
        check_len(bytes)?;
        let f: InterfaceFacts =
            serde_json::from_slice(bytes).map_err(|e| CodecError(format!("decode: {e}")))?;
        f.validate().map_err(CodecError)?;
        Ok(f)
    }
}

/// Physical plan codec (`autoabi.contract.plan/1`).
pub mod plan_codec {
    use super::*;

    pub fn encode(p: &PhysicalPlan) -> Result<Vec<u8>, CodecError> {
        p.validate().map_err(CodecError)?;
        serde_json::to_vec(p).map_err(|e| CodecError(e.to_string()))
    }

    pub fn decode(bytes: &[u8]) -> Result<PhysicalPlan, CodecError> {
        check_len(bytes)?;
        let p: PhysicalPlan =
            serde_json::from_slice(bytes).map_err(|e| CodecError(format!("decode: {e}")))?;
        p.validate().map_err(CodecError)?;
        Ok(p)
    }
}

/// Typed-declarations codec (`autoabi.contract.typed-decls/1`).
pub mod typed_decls_codec {
    use super::*;

    pub fn encode(t: &TypedDecls) -> Result<Vec<u8>, CodecError> {
        serde_json::to_vec(t).map_err(|e| CodecError(e.to_string()))
    }

    pub fn decode(bytes: &[u8]) -> Result<TypedDecls, CodecError> {
        check_len(bytes)?;
        let t: TypedDecls =
            serde_json::from_slice(bytes).map_err(|e| CodecError(format!("decode: {e}")))?;
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::tests_support::checksum_facts;

    #[test]
    fn facts_roundtrip_and_unknown_field_rejected() {
        let f = checksum_facts();
        let bytes = facts_codec::encode(&f).unwrap();
        let back = facts_codec::decode(&bytes).unwrap();
        assert_eq!(f, back);
        // An unknown field (future/foreign producer) is a rejection.
        let mut doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        doc["future_field"] = serde_json::json!(1);
        let bad = serde_json::to_vec(&doc).unwrap();
        assert!(facts_codec::decode(&bad)
            .unwrap_err()
            .to_string()
            .contains("decode"));
    }

    #[test]
    fn wrong_format_rejected() {
        let mut f = checksum_facts();
        let bytes = facts_codec::encode(&f).unwrap();
        f.format = "autoabi.contract.facts/9".into();
        let bad = facts_codec::encode(&f);
        assert!(bad
            .unwrap_err()
            .to_string()
            .contains("unknown facts format"));
        // also on decode: an encoded doc whose format string lies
        let mut doc: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        doc["format"] = serde_json::json!("autoabi.contract.facts/9");
        let bad = serde_json::to_vec(&doc).unwrap();
        assert!(facts_codec::decode(&bad)
            .unwrap_err()
            .to_string()
            .contains("unknown facts format"));
    }

    #[test]
    fn oversized_payload_rejected() {
        let mut bytes = facts_codec::encode(&checksum_facts()).unwrap();
        bytes.resize(MAX_CODEC_BYTES + 1, b' ');
        assert!(facts_codec::decode(&bytes)
            .unwrap_err()
            .to_string()
            .contains("cap"));
    }

    #[test]
    fn plan_roundtrip() {
        let p = PhysicalPlan::project(&checksum_facts()).unwrap();
        let bytes = plan_codec::encode(&p).unwrap();
        let back = plan_codec::decode(&bytes).unwrap();
        assert_eq!(p, back);
        assert_eq!(p.digest().unwrap(), back.digest().unwrap());
    }
}
