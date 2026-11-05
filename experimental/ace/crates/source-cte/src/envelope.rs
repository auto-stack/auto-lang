//! The executed envelope of the source-CTE lane (PLAN-936 §5.7 row 3).
//!
//! An envelope exists ONLY for a SUCCESSFUL real AV execution. It carries
//! the portable value, the declared type, the execution identity (the
//! complete [`ExecKey`](crate::identity::ExecKey) canonical), the
//! measured execution facts (steps/wall) and the closure versions that
//! were in force. It deliberately does NOT carry:
//!
//! * any source-level fingerprint (`source_fp` is the 817 shape this
//!   lane replaces — the callee logical + version closure IS the
//!   identity);
//! * any owner/use-binding data (that is per-request, §5.7 row 2);
//! * any consumer verdict (a hit still re-admits against the current
//!   request — the 817 discipline that a cached page never caches the
//!   consumer's own check).
//!
//! Codec: same hand-rolled length-prefixed discipline as the 934 pages —
//! every identity is re-derived and checked on decode; nothing is
//! trusted from the bytes; a foreign/truncated/tampered page is a
//! controlled rejection.

use crate::identity::{
    ClosureVersion, CtEngine, CtValue, ExpectTy, CT_ALGO, CT_PRODUCER, CT_PROFILE, CT_SCHEMA,
};
use crate::store::PAGE_MAGIC;

/// One executed envelope.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CtExecutedV1 {
    pub schema: &'static str,
    pub producer: &'static str,
    pub profile: &'static str,
    pub algo: &'static str,
    /// The complete execution-key canonical string (re-checked against
    /// the caller's current key on every read).
    pub exec_key_canonical: String,
    /// The frozen value — strict tag against `declared_type` on decode.
    pub value: CtValue,
    pub declared_type: ExpectTy,
    pub callee: String,
    pub callee_logical: String,
    /// The closure versions the execution ran under (sorted canonical on
    /// encode; the upgrade seam for the T-08 typed-dependency axes).
    pub closure: Vec<ClosureVersion>,
    /// Measured AV facts (never reconstructed).
    pub exec_steps: u64,
    pub exec_wall_ms: u64,
    pub engine: CtEngine,
}

/// Why an envelope cannot be read/admitted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum EnvelopeError {
    /// Byte-level or identity-level corruption/mismatch.
    Decode(String),
    /// The page belongs to another producer/schema than this lane's.
    Envelope { found: String },
    /// The value's tag does not match the declared type (a tampered or
    /// foreign page — this lane never writes such a page).
    TagMismatch { declared: &'static str },
    /// The page's key is not the caller's current key.
    KeyMismatch { expected: String, found: String },
}

impl std::fmt::Display for EnvelopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnvelopeError::Decode(m) => write!(f, "envelope decode rejected: {m}"),
            EnvelopeError::Envelope { found } => {
                write!(f, "envelope identity `{found}` is foreign to {CT_SCHEMA}")
            }
            EnvelopeError::TagMismatch { declared } => {
                write!(
                    f,
                    "envelope value tag does not match declared type {declared}"
                )
            }
            EnvelopeError::KeyMismatch { expected, found } => {
                write!(f, "envelope key {found} is not the expected {expected}")
            }
        }
    }
}

const MAX_STR: usize = 16 * 1024;
const MAX_CLOSURE: usize = 8192;

struct W {
    out: Vec<u8>,
}

impl W {
    fn u8(&mut self, v: u8) {
        self.out.push(v);
    }
    fn u32(&mut self, v: u32) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }
    fn s(&mut self, v: &str) -> Result<(), EnvelopeError> {
        if v.len() > MAX_STR {
            return Err(EnvelopeError::Decode(format!(
                "string of {} bytes exceeds the envelope budget",
                v.len()
            )));
        }
        self.u32(v.len() as u32);
        self.out.extend_from_slice(v.as_bytes());
        Ok(())
    }
}

struct R<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> R<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], EnvelopeError> {
        let end = self
            .at
            .checked_add(n)
            .ok_or_else(|| EnvelopeError::Decode("length overflow".into()))?;
        if end > self.bytes.len() {
            return Err(EnvelopeError::Decode("truncated envelope".into()));
        }
        let s = &self.bytes[self.at..end];
        self.at = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, EnvelopeError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, EnvelopeError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn u64(&mut self) -> Result<u64, EnvelopeError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
    fn s(&mut self) -> Result<String, EnvelopeError> {
        let n = self.u32()? as usize;
        if n > MAX_STR {
            return Err(EnvelopeError::Decode(format!(
                "string of {n} bytes exceeds the envelope budget"
            )));
        }
        let b = self.take(n)?;
        String::from_utf8(b.to_vec()).map_err(|_| EnvelopeError::Decode("bad UTF-8".into()))
    }
}

/// Encode one envelope payload (the store frames it). Fails closed on
/// budget violations.
pub fn encode_envelope(env: &CtExecutedV1) -> Result<Vec<u8>, EnvelopeError> {
    let mut w = W { out: Vec::new() };
    w.out.extend_from_slice(PAGE_MAGIC);
    w.u8(crate::store::PAGE_CODEC_VERSION);
    w.s(&env.exec_key_canonical)?;
    w.s(env.producer)?;
    w.s(env.schema)?;
    w.s(env.profile)?;
    w.s(env.algo)?;
    // value: tag + payload
    match env.value {
        CtValue::I32(x) => {
            w.u8(1);
            w.u32(x as u32);
        }
        CtValue::Bool(b) => {
            w.u8(2);
            w.u8(b as u8);
        }
    }
    w.u8(match env.declared_type {
        ExpectTy::I32 => 1,
        ExpectTy::Bool => 2,
    });
    w.s(&env.callee)?;
    w.s(&env.callee_logical)?;
    if env.closure.len() > MAX_CLOSURE {
        return Err(EnvelopeError::Decode(format!(
            "closure of {} members exceeds the envelope budget",
            env.closure.len()
        )));
    }
    w.u32(env.closure.len() as u32);
    let mut sorted = env.closure.clone();
    sorted.sort_by(|a, b| a.name.cmp(&b.name).then(a.logical.cmp(&b.logical)));
    for c in &sorted {
        w.s(&c.name)?;
        w.s(&c.logical)?;
        w.s(&c.sig_version)?;
        w.s(&c.body_version)?;
    }
    w.u64(env.exec_steps);
    w.u64(env.exec_wall_ms);
    w.s(&env.engine.av_profile)?;
    w.s(&env.engine.mir_schema)?;
    w.s(&env.engine.abc_schema)?;
    w.s(&env.engine.mir_semantics)?;
    w.s(&env.engine.abc_semantics)?;
    w.s(&env.engine.av_semantics)?;
    w.s(env.engine.lane)?;
    if w.out.len() > crate::store::MAX_PAGE_BYTES {
        return Err(EnvelopeError::Decode(format!(
            "envelope of {} bytes exceeds the byte cap",
            w.out.len()
        )));
    }
    Ok(w.out)
}

/// Decode + admit one envelope payload. Every identity is re-derived and
/// checked; the value's tag is strictly matched against the declared
/// type.
pub fn decode_envelope(bytes: &[u8]) -> Result<(String, CtExecutedV1), EnvelopeError> {
    if bytes.len() > crate::store::MAX_PAGE_BYTES {
        return Err(EnvelopeError::Decode(
            "envelope exceeds the byte cap".into(),
        ));
    }
    let mut r = R { bytes, at: 0 };
    if r.take(PAGE_MAGIC.len())? != PAGE_MAGIC {
        return Err(EnvelopeError::Decode("bad page magic".into()));
    }
    let version = r.u8()?;
    if version != crate::store::PAGE_CODEC_VERSION {
        return Err(EnvelopeError::Decode(format!(
            "codec version {version} is not {}",
            crate::store::PAGE_CODEC_VERSION
        )));
    }
    let exec_key_canonical = r.s()?;
    let producer = r.s()?;
    if producer != CT_PRODUCER {
        return Err(EnvelopeError::Envelope { found: producer });
    }
    let schema = r.s()?;
    if schema != CT_SCHEMA {
        return Err(EnvelopeError::Envelope { found: schema });
    }
    let profile = r.s()?;
    if profile != CT_PROFILE {
        return Err(EnvelopeError::Envelope { found: profile });
    }
    let algo = r.s()?;
    if algo != CT_ALGO {
        return Err(EnvelopeError::Envelope { found: algo });
    }
    let value = match r.u8()? {
        1 => CtValue::I32(r.u32()? as i32),
        2 => CtValue::Bool(r.u8()? != 0),
        t => return Err(EnvelopeError::Decode(format!("unknown value tag {t}"))),
    };
    let declared_type = match r.u8()? {
        1 => ExpectTy::I32,
        2 => ExpectTy::Bool,
        t => return Err(EnvelopeError::Decode(format!("unknown type tag {t}"))),
    };
    // Strict tag: the payload's own declared type must match its value.
    match (value, declared_type) {
        (CtValue::I32(_), ExpectTy::I32) | (CtValue::Bool(_), ExpectTy::Bool) => {}
        (CtValue::Bool(_), ExpectTy::I32) => {
            return Err(EnvelopeError::TagMismatch { declared: "i32" })
        }
        (CtValue::I32(_), ExpectTy::Bool) => {
            return Err(EnvelopeError::TagMismatch { declared: "bool" })
        }
    }
    let callee = r.s()?;
    let callee_logical = r.s()?;
    let n = r.u32()? as usize;
    if n > MAX_CLOSURE {
        return Err(EnvelopeError::Decode(format!(
            "closure count {n} exceeds the envelope budget"
        )));
    }
    let mut closure = Vec::with_capacity(n);
    for _ in 0..n {
        closure.push(ClosureVersion {
            name: r.s()?,
            logical: r.s()?,
            sig_version: r.s()?,
            body_version: r.s()?,
        });
    }
    let exec_steps = r.u64()?;
    let exec_wall_ms = r.u64()?;
    let engine = CtEngine {
        av_profile: r.s()?,
        mir_schema: r.s()?,
        abc_schema: r.s()?,
        mir_semantics: r.s()?,
        abc_semantics: r.s()?,
        av_semantics: r.s()?,
        lane: match r.s()?.as_str() {
            crate::identity::CT_LANE => crate::identity::CT_LANE,
            other => {
                // The engine lane is part of the key; a foreign lane in
                // the payload is a decode-level identity rejection.
                return Err(EnvelopeError::Decode(format!(
                    "engine lane `{other}` is foreign to this lane"
                )));
            }
        },
    };
    if r.at != bytes.len() {
        return Err(EnvelopeError::Decode(
            "trailing bytes after the envelope".into(),
        ));
    }
    Ok((
        exec_key_canonical.clone(),
        CtExecutedV1 {
            schema: CT_SCHEMA,
            producer: CT_PRODUCER,
            profile: CT_PROFILE,
            algo: CT_ALGO,
            exec_key_canonical,
            value,
            declared_type,
            callee,
            callee_logical,
            closure,
            exec_steps,
            exec_wall_ms,
            engine,
        },
    ))
}

#[cfg(test)]
mod envelope_tests {
    use super::*;

    fn sample() -> CtExecutedV1 {
        CtExecutedV1 {
            schema: CT_SCHEMA,
            producer: CT_PRODUCER,
            profile: CT_PROFILE,
            algo: CT_ALGO,
            exec_key_canonical: "key-canonical".into(),
            value: CtValue::I32(42),
            declared_type: ExpectTy::I32,
            callee: "sum".into(),
            callee_logical: "logical-sum".into(),
            closure: vec![ClosureVersion {
                name: "sum".into(),
                logical: "logical-sum".into(),
                sig_version: "aa".into(),
                body_version: "bb".into(),
            }],
            exec_steps: 12,
            exec_wall_ms: 3,
            engine: CtEngine::current(),
        }
    }

    #[test]
    fn round_trip_is_exact() {
        let bytes = encode_envelope(&sample()).unwrap();
        let (key, env) = decode_envelope(&bytes).unwrap();
        assert_eq!(key, "key-canonical");
        assert_eq!(env.value, CtValue::I32(42));
        assert_eq!(env.declared_type, ExpectTy::I32);
        assert_eq!(env.exec_steps, 12);
        assert_eq!(env.engine.lane, crate::identity::CT_LANE);
    }

    #[test]
    fn tampered_tag_is_rejected() {
        let mut bytes = encode_envelope(&sample()).unwrap();
        // value tag byte sits right after magic+version+4 length-prefixed
        // strings; locate the declared-type byte by re-encoding with a
        // known value and flipping the tag check instead: craft directly.
        let bad = CtExecutedV1 {
            schema: CT_SCHEMA,
            producer: CT_PRODUCER,
            profile: CT_PROFILE,
            algo: CT_ALGO,
            exec_key_canonical: "k".into(),
            value: CtValue::Bool(true),
            declared_type: ExpectTy::I32,
            callee: "f".into(),
            callee_logical: "l".into(),
            closure: vec![],
            exec_steps: 1,
            exec_wall_ms: 1,
            engine: CtEngine::current(),
        };
        let enc = encode_envelope(&bad).unwrap();
        assert!(matches!(
            decode_envelope(&enc),
            Err(EnvelopeError::TagMismatch { .. })
        ));
        // and a truncated page never decodes
        bytes.truncate(bytes.len() - 3);
        assert!(matches!(
            decode_envelope(&bytes),
            Err(EnvelopeError::Decode(_))
        ));
    }
}
