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

/// Measure the canonical JSON output without materializing a buffer. The
/// caller can reserve its enclosing live/staging budget before allocating.
pub fn json_size<T: serde::Serialize + ?Sized>(
    value: &T,
    limit: usize,
) -> Result<usize, CodecError> {
    struct Count {
        len: usize,
        limit: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.len = self
                .len
                .checked_add(bytes.len())
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| std::io::Error::other("JSON byte budget"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut count = Count { len: 0, limit };
    serde_json::to_writer(&mut count, value).map_err(|e| CodecError(e.to_string()))?;
    Ok(count.len)
}

/// Identical wire bytes to serde_json::to_vec, with an exact preflight and a
/// fallible reservation BEFORE allocating the output. A stateful serializer
/// cannot grow the buffer beyond the size it declared on the counting pass.
pub fn encode_json_bounded<T: serde::Serialize + ?Sized>(
    value: &T,
    limit: usize,
) -> Result<Vec<u8>, CodecError> {
    let size = json_size(value, limit)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(size)
        .map_err(|e| CodecError(format!("JSON allocation: {e}")))?;
    struct Output<'a> {
        bytes: &'a mut Vec<u8>,
        size: usize,
    }
    impl std::io::Write for Output<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self
                .bytes
                .len()
                .checked_add(bytes.len())
                .is_none_or(|n| n > self.size)
            {
                return Err(std::io::Error::other(
                    "JSON serializer changed after preflight",
                ));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(
        Output {
            bytes: &mut bytes,
            size,
        },
        value,
    )
    .map_err(|e| CodecError(e.to_string()))?;
    if bytes.len() != size {
        return Err(CodecError("JSON serializer changed after preflight".into()));
    }
    Ok(bytes)
}

/// Pre-allocation bound for the finite source-session wire mirrors. Their
/// scalar/tuple/enum fields fit the per-field allowance below. Byte arrays and
/// source text are separate from the metadata/index share. This is not a
/// universal bound for arbitrary user-defined Rust Deserialize types.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsonAllocation {
    pub metadata: usize,
    pub bytes: usize,
    pub source: usize,
}
impl JsonAllocation {
    pub fn total(self) -> Option<usize> {
        self.metadata
            .checked_add(self.bytes)?
            .checked_add(self.source)
    }
}
#[derive(Debug)]
pub enum AllocationError {
    Budget(String),
    Decode(String),
}
impl std::fmt::Display for AllocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Budget(s) | Self::Decode(s) => f.write_str(s),
        }
    }
}
impl std::error::Error for AllocationError {}
pub fn source_json_allocation(
    bytes: &[u8],
    limit: usize,
) -> Result<JsonAllocation, AllocationError> {
    use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
    #[derive(Clone, Copy)]
    enum Mode {
        Normal,
        Source,
        Bytes,
        Byte,
    }
    struct Bound {
        result: JsonAllocation,
        limit: usize,
        exhausted: bool,
    }
    impl Bound {
        fn add(&mut self, mode: Mode, n: usize, factor: usize) -> Result<(), String> {
            let n = match n.checked_mul(factor) {
                Some(n) => n,
                None => {
                    self.exhausted = true;
                    return Err("JSON allocation size overflow".into());
                }
            };
            let current = match mode {
                Mode::Normal => &mut self.result.metadata,
                Mode::Source => &mut self.result.source,
                Mode::Bytes | Mode::Byte => &mut self.result.bytes,
            };
            *current = match current.checked_add(n) {
                Some(n) => n,
                None => {
                    self.exhausted = true;
                    return Err("JSON allocation size overflow".into());
                }
            };
            if self.result.total().is_none_or(|n| n > self.limit) {
                self.exhausted = true;
                return Err("JSON allocation budget".into());
            }
            Ok(())
        }
    }
    struct Seed<'a> {
        bound: &'a mut Bound,
        mode: Mode,
        factor: usize,
    }
    impl<'de> DeserializeSeed<'de> for Seed<'_> {
        type Value = ();
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
            d.deserialize_any(self)
        }
    }
    impl Seed<'_> {
        fn scalar<E: Error>(&mut self) -> Result<(), E> {
            if matches!(self.mode, Mode::Byte | Mode::Bytes) {
                return Err(E::custom("expected byte array"));
            }
            self.bound
                .add(Mode::Normal, 32, self.factor)
                .map_err(E::custom)
        }
    }
    impl<'de> Visitor<'de> for Seed<'_> {
        type Value = ();
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("bounded source-session JSON")
        }
        fn visit_unit<E: Error>(mut self) -> Result<(), E> {
            self.scalar()
        }
        fn visit_bool<E: Error>(mut self, _: bool) -> Result<(), E> {
            self.scalar()
        }
        fn visit_i64<E: Error>(mut self, n: i64) -> Result<(), E> {
            if matches!(self.mode, Mode::Byte) {
                if !(0..=255).contains(&n) {
                    return Err(E::custom("invalid byte"));
                }
                return self
                    .bound
                    .add(Mode::Byte, 1, self.factor)
                    .map_err(E::custom);
            }
            self.scalar()
        }
        fn visit_u64<E: Error>(mut self, n: u64) -> Result<(), E> {
            if matches!(self.mode, Mode::Byte) {
                if n > 255 {
                    return Err(E::custom("invalid byte"));
                }
                return self
                    .bound
                    .add(Mode::Byte, 1, self.factor)
                    .map_err(E::custom);
            }
            self.scalar()
        }
        fn visit_f64<E: Error>(mut self, _: f64) -> Result<(), E> {
            self.scalar()
        }
        fn visit_str<E: Error>(self, s: &str) -> Result<(), E> {
            if matches!(self.mode, Mode::Byte | Mode::Bytes) {
                return Err(E::custom("expected byte array"));
            }
            self.bound
                .add(Mode::Normal, std::mem::size_of::<String>(), self.factor)
                .map_err(E::custom)?;
            self.bound
                .add(
                    if matches!(self.mode, Mode::Source) {
                        Mode::Source
                    } else {
                        Mode::Normal
                    },
                    s.len(),
                    self.factor,
                )
                .map_err(E::custom)
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
            if matches!(self.mode, Mode::Byte) {
                return Err(A::Error::custom("invalid byte"));
            }
            let byte_array = matches!(self.mode, Mode::Bytes);
            self.bound
                .add(
                    if byte_array {
                        Mode::Bytes
                    } else {
                        Mode::Normal
                    },
                    32,
                    self.factor,
                )
                .map_err(A::Error::custom)?;
            let factor = self.factor.checked_mul(2).ok_or_else(|| {
                self.bound.exhausted = true;
                A::Error::custom("JSON allocation size overflow")
            })?;
            while let Some(()) = seq.next_element_seed(Seed {
                bound: self.bound,
                mode: if byte_array { Mode::Byte } else { Mode::Normal },
                factor,
            })? {}
            Ok(())
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
            if matches!(self.mode, Mode::Byte | Mode::Bytes) {
                return Err(A::Error::custom("expected byte array"));
            }
            self.bound
                .add(Mode::Normal, 64, self.factor)
                .map_err(A::Error::custom)?;
            struct Key(Mode, usize);
            impl<'de> serde::Deserialize<'de> for Key {
                fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                    struct KeyVisitor;
                    impl Visitor<'_> for KeyVisitor {
                        type Value = Key;
                        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            f.write_str("wire field")
                        }
                        fn visit_str<E: Error>(self, s: &str) -> Result<Key, E> {
                            Ok(Key(
                                match s {
                                    "mir" | "coff" | "facts" | "root" => Mode::Bytes,
                                    "source" | "text" => Mode::Source,
                                    _ => Mode::Normal,
                                },
                                s.len(),
                            ))
                        }
                    }
                    d.deserialize_str(KeyVisitor)
                }
            }
            while let Some(Key(mode, len)) = map.next_key()? {
                self.bound
                    .add(
                        Mode::Normal,
                        128usize
                            .checked_add(len)
                            .ok_or_else(|| A::Error::custom("JSON key size overflow"))?,
                        self.factor,
                    )
                    .map_err(A::Error::custom)?;
                map.next_value_seed(Seed {
                    bound: self.bound,
                    mode,
                    factor: self.factor,
                })?;
            }
            Ok(())
        }
    }
    let mut bound = Bound {
        result: JsonAllocation::default(),
        limit,
        exhausted: false,
    };
    let mut d = serde_json::Deserializer::from_slice(bytes);
    if let Err(e) = (Seed {
        bound: &mut bound,
        mode: Mode::Normal,
        factor: 1,
    })
    .deserialize(&mut d)
    {
        return Err(if bound.exhausted {
            AllocationError::Budget(e.to_string())
        } else {
            AllocationError::Decode(e.to_string())
        });
    }
    d.end()
        .map_err(|e| AllocationError::Decode(e.to_string()))?;
    Ok(bound.result)
}

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
    fn source_json_preflight_splits_payload_from_index_and_bounds_real_wire_representation() {
        #[derive(serde::Deserialize)]
        struct Wire {
            root: Vec<u8>,
            source: String,
            rows: Vec<(u32, String)>,
        }
        let bytes = br#"{"root":[0,1,255],"source":"abc\n\u4f60","rows":[[1,"a"],[2,"longer"]]}"#;
        let bound = source_json_allocation(bytes, 1 << 20).unwrap();
        let decoded: Wire = serde_json::from_slice(bytes).unwrap();
        let actual = std::mem::size_of::<Wire>()
            + decoded.root.capacity()
            + decoded.source.capacity()
            + decoded.rows.capacity() * std::mem::size_of::<(u32, String)>()
            + decoded
                .rows
                .iter()
                .map(|(_, s)| s.capacity())
                .sum::<usize>();
        assert!(bound.total().unwrap() >= actual);
        assert!(bound.bytes >= decoded.root.capacity());
        assert_eq!(bound.source, decoded.source.len());
        assert!(matches!(
            source_json_allocation(bytes, 16),
            Err(AllocationError::Budget(_))
        ));
        assert!(matches!(
            source_json_allocation(br#"{"root":[256]}"#, 10000),
            Err(AllocationError::Decode(_))
        ));
        assert!(matches!(
            source_json_allocation(br#"{} {}"#, 10000),
            Err(AllocationError::Decode(_))
        ));
    }

    #[test]
    fn bounded_json_is_byte_identical_and_rejects_before_output_materialization() {
        use std::cell::Cell;
        let value = serde_json::json!({"escaped":"\n\\\"你好", "data":[0,255,42]});
        let expected = serde_json::to_vec(&value).unwrap();
        assert_eq!(json_size(&value, expected.len()).unwrap(), expected.len());
        assert_eq!(
            encode_json_bounded(&value, expected.len()).unwrap(),
            expected
        );
        assert!(encode_json_bounded(&value, expected.len() - 1).is_err());
        struct Observed<'a>(&'a Cell<usize>);
        impl serde::Serialize for Observed<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.set(self.0.get() + 1);
                serializer.collect_seq((0..1024).map(|_| 255u8))
            }
        }
        let calls = Cell::new(0);
        assert!(encode_json_bounded(&Observed(&calls), 100).is_err());
        assert_eq!(
            calls.get(),
            1,
            "over-budget inputs never enter output serialization"
        );
    }

    #[test]
    fn stateful_json_serializer_cannot_outgrow_its_preflight() {
        use std::cell::Cell;
        struct Changed(Cell<bool>);
        impl serde::Serialize for Changed {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(if self.0.replace(true) {
                    "expanded"
                } else {
                    "a"
                })
            }
        }
        assert!(encode_json_bounded(&Changed(Cell::new(false)), 100)
            .unwrap_err()
            .to_string()
            .contains("changed after preflight"));
    }

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
