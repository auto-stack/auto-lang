//! PLAN-826: the ABC wire-generation policy for the scalar profile.
//!
//! One place selects between the delivered legacy AVAB core wire and the
//! new compact AVAC wire for A09 unit payloads, closure admission and the
//! restore path. The two generations never cross-admit (magic isolation
//! at decode) and never share a cache namespace (the query key's
//! `abc-codec` dimension is the codec's schema), so a policy switch is a
//! deterministic miss/rebuild against the other generation's rows — never
//! an in-place reinterpretation (PLAN-826 contract-decision §3).

use ace_av_core::mir::MirModule;
use ace_av_core::{
    decode_abc, decode_compact_abc, encode_abc, encode_compact_abc, AbcDecodeError, LoadedModule,
    ABC_SCHEMA, COMPACT_ABC_MAGIC, COMPACT_ABC_SCHEMA,
};

/// The decoded wire payload in codec-neutral terms: the wire version and
/// profile ids AS DECODED (admission re-checks them) plus the unchecked
/// module.
#[derive(Clone)]
pub struct DecodedWire {
    pub version: u16,
    pub profile: u16,
    pub module: MirModule,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WireCodec {
    /// The delivered AVAB core wire (`auto.abc.av-core-i32/2`).
    LegacyAbc,
    /// The compact AVAC wire (`auto.abc.av-compact-core-i32/1`, PLAN-826).
    CompactAbc,
    /// PLAN-862: the entry-unit wire (`auto.abc.av-entry-unit/1`) — a NEW
    /// `AVAU` magic family at wire version 1 / profile id 7. It rides the
    /// same paged codec (the module's profile selects the family at encode)
    /// but has its OWN generation gate and cache namespace: an AVAB/AVAD
    /// payload is a deterministic stale miss, never an in-place
    /// reinterpretation.
    EntryUnitAbc,
}

impl WireCodec {
    /// The schema identity used as the cache-key `abc-codec` dimension and
    /// the manifest's codec routing value.
    pub fn schema(&self) -> &'static str {
        match self {
            WireCodec::LegacyAbc => ABC_SCHEMA,
            WireCodec::CompactAbc => COMPACT_ABC_SCHEMA,
            WireCodec::EntryUnitAbc => ace_av_core::contract::ABC_ENTRY_UNIT_SCHEMA,
        }
    }

    /// The owner-scope schema for modules admitted under this codec.
    pub fn owner_schema(&self) -> &'static str {
        self.schema()
    }

    pub fn from_schema(schema: &str) -> Result<WireCodec, String> {
        match schema {
            ABC_SCHEMA => Ok(WireCodec::LegacyAbc),
            COMPACT_ABC_SCHEMA => Ok(WireCodec::CompactAbc),
            // PLAN-862: the entry-unit generation.
            ace_av_core::contract::ABC_ENTRY_UNIT_SCHEMA => Ok(WireCodec::EntryUnitAbc),
            other => Err(format!(
                "unknown abc wire codec `{other}` (admitted: `{ABC_SCHEMA}`, `{COMPACT_ABC_SCHEMA}`)"
            )),
        }
    }

    /// Encode a module's wire payload under this codec.
    pub fn encode(&self, module: &MirModule) -> Result<Vec<u8>, AbcDecodeError> {
        match self {
            WireCodec::LegacyAbc => encode_abc(module).map(|paged| paged.to_flat()),
            // PLAN-862: the module's profile selects the AVAU family inside
            // ncode_abc; this policy only owns the generation gate.
            WireCodec::EntryUnitAbc => encode_abc(module).map(|paged| paged.to_flat()),
            WireCodec::CompactAbc => encode_compact_abc(module),
        }
    }

    /// Decode a payload under this codec. A payload of the OTHER
    /// generation fails here on its magic — never cross-read.
    pub fn decode(&self, payload: &[u8]) -> Result<DecodedWire, AbcDecodeError> {
        match self {
            WireCodec::LegacyAbc => {
                let decoded = decode_abc(payload)?;
                Ok(DecodedWire {
                    version: decoded.version,
                    profile: decoded.profile,
                    module: decoded.module,
                })
            }
            WireCodec::EntryUnitAbc => {
                let decoded = decode_abc(payload)?;
                Ok(DecodedWire {
                    version: decoded.version,
                    profile: decoded.profile,
                    module: decoded.module,
                })
            }
            WireCodec::CompactAbc => {
                let decoded = decode_compact_abc(payload)?;
                Ok(DecodedWire {
                    version: decoded.version,
                    profile: decoded.profile,
                    module: decoded.module,
                })
            }
        }
    }

    /// Admit raw payload bytes through the codec's REAL admission entry
    /// (owner re-stamp + verify + executable-closure credential).
    pub fn admit_bytes(
        &self,
        payload: &[u8],
        expected_owner: &ace_av_core::mir::OwnerScope,
    ) -> Result<LoadedModule, ace_av_core::AbcAdmissionError> {
        match self {
            WireCodec::LegacyAbc => ace_av_core::verify_abc_bytes(payload, expected_owner),
            WireCodec::CompactAbc => ace_av_core::verify_compact_abc_bytes(payload, expected_owner),
            WireCodec::EntryUnitAbc => ace_av_core::verify_abc_bytes(payload, expected_owner),
        }
    }

    /// Admit an already-decoded payload through the codec's REAL
    /// admission entry (the admission re-checks version/profile on the
    /// public struct — a hand-built DecodedWire cannot smuggle them).
    pub fn admit_decoded(
        &self,
        decoded: DecodedWire,
        expected_owner: &ace_av_core::mir::OwnerScope,
    ) -> Result<LoadedModule, ace_av_core::AbcAdmissionError> {
        match self {
            WireCodec::LegacyAbc => ace_av_core::verify_abc(
                ace_av_core::DecodedAbc {
                    version: decoded.version,
                    profile: decoded.profile,
                    module: decoded.module,
                },
                expected_owner,
            ),
            WireCodec::EntryUnitAbc => ace_av_core::verify_abc(
                ace_av_core::DecodedAbc {
                    version: decoded.version,
                    profile: decoded.profile,
                    module: decoded.module,
                },
                expected_owner,
            ),
            WireCodec::CompactAbc => ace_av_core::verify_compact_abc(
                ace_av_core::DecodedCompactAbc {
                    version: decoded.version,
                    profile: decoded.profile,
                    module: decoded.module,
                },
                expected_owner,
            ),
        }
    }

    /// Re-bind a decoded UNIT payload's function identity from the
    /// ENVELOPE (PLAN-826 contract §3: the compact wire carries no
    /// function names — the function-directory index is the wire identity
    /// and owner/module identity rides the envelope). The legacy wire
    /// carries the name itself; callers keep their name-vs-declaration
    /// checks on that generation.
    pub fn rebind_unit_name(&self, module: &mut MirModule, expected: &str) {
        match self {
            WireCodec::CompactAbc => {
                if let Some(f) = module.funcs.first_mut() {
                    f.name = expected.to_string();
                }
            }
            WireCodec::LegacyAbc => {}
            // the paged generation carries function names itself.
            WireCodec::EntryUnitAbc => {}
        }
    }

    /// O(1) format-generation gate (F798-R12-03 shape): a payload is
    /// current-generation for THIS codec exactly when its magic and wire
    /// version match; the other generation's payload is a deterministic
    /// stale miss.
    pub fn is_current_generation(&self, payload: &[u8]) -> bool {
        match self {
            WireCodec::LegacyAbc => {
                payload.len() >= 6
                    && payload.starts_with(ace_av_core::abc::ABC_MAGIC)
                    && u16::from_le_bytes([payload[4], payload[5]]) == ace_av_core::abc::ABC_VERSION
            }
            // PLAN-862: the entry-unit family is current only when the
            // payload carries ITS magic and wire version.
            WireCodec::EntryUnitAbc => {
                payload.len() >= 6
                    && payload.starts_with(&ace_av_core::contract::ABC_ENTRY_UNIT_MAGIC)
                    && u16::from_le_bytes([payload[4], payload[5]])
                        == ace_av_core::contract::ABC_ENTRY_UNIT_VERSION
            }
            WireCodec::CompactAbc => {
                payload.len() >= 6
                    && payload.starts_with(&COMPACT_ABC_MAGIC)
                    && u16::from_le_bytes([payload[4], payload[5]])
                        == ace_av_core::COMPACT_ABC_VERSION
            }
        }
    }
}
