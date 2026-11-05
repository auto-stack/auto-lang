//! Logical public scalar lanes and their physical projection (937 plan
//! §5.1 "typed标量桥" row).
//!
//! The PUBLIC lane set is exactly `{i32, u32, i64, u64, f64, bool, char}`.
//! There is deliberately **no f32 lane**: the v0.1 candidate public ABI
//! has no f32 support, and a declaration asking for one is a named
//! rejection at validation — not a silent widening to f64.
//!
//! # Private-vs-public boundary (the conversion the plan demands)
//!
//! * `bool` crosses the public boundary as a **u8 lane carrying 0/1**
//!   (physical `bool`/`uint8_t`). The PRIVATE mn/A2C ABI spells bool as
//!   an I32 lane — that private form is NOT aliased here; adapters
//!   convert explicitly at the boundary (937 plan §4 "实际差额" row).
//! * `char` is a logical Unicode scalar value crossing as a **u32 lane**
//!   carrying the scalar value (physical `char32_t`); lone surrogates
//!   and out-of-range values are rejected BEFORE any call.
//! * `f64` is a REAL double lane. The 891-era wide-typed floats use
//!   integer BIT CARRIERS internally; on the public boundary the lane is
//!   a true float — [`LaneValue::F64Bits`] keeps the exact bit pattern
//!   (including quiet-NaN payloads and -0.0) so no marshalling step can
//!   silently canonicalize a payload the caller sent.

use serde::{Deserialize, Serialize};

/// Logical public scalar lane kinds (the typed-declaration vocabulary).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum LaneKind {
    I32,
    U32,
    I64,
    U64,
    F64,
    Bool,
    Char,
}

impl LaneKind {
    pub const ALL: &'static [LaneKind] = &[
        LaneKind::I32,
        LaneKind::U32,
        LaneKind::I64,
        LaneKind::U64,
        LaneKind::F64,
        LaneKind::Bool,
        LaneKind::Char,
    ];

    /// The deck/spec spelling (same tokens the 797 deck grammar uses).
    pub fn spell(self) -> &'static str {
        match self {
            LaneKind::I32 => "i32",
            LaneKind::U32 => "u32",
            LaneKind::I64 => "i64",
            LaneKind::U64 => "u64",
            LaneKind::F64 => "f64",
            LaneKind::Bool => "bool",
            LaneKind::Char => "char",
        }
    }

    pub fn of_spelling(s: &str) -> Option<LaneKind> {
        LaneKind::ALL.iter().copied().find(|k| k.spell() == s)
    }

    /// Physical lane this logical kind projects to on the public ABI.
    pub fn physical(self) -> PhysLane {
        match self {
            LaneKind::I32 => PhysLane::I32,
            LaneKind::U32 => PhysLane::U32,
            LaneKind::I64 => PhysLane::I64,
            LaneKind::U64 => PhysLane::U64,
            LaneKind::F64 => PhysLane::F64,
            // Public bool = u8 lane (0/1). NOT the private mn I32-bool.
            LaneKind::Bool => PhysLane::BoolU8,
            // Public char = u32 lane carrying the Unicode scalar value.
            LaneKind::Char => PhysLane::CharU32,
        }
    }
}

/// PHYSICAL public lane forms on the candidate ABI (Windows x64).
/// These are the machine-visible forms the C header spells and the
/// Cranelift signature builds; they exist ONLY as projections of
/// [`LaneKind`] — there is no physical f32 and no raw-pointer lane in
/// this bridge's public set (pointers appear only as the fallible ctx
/// lane and out slots, spelled by the form, not by a bare lane).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum PhysLane {
    I32,
    U32,
    I64,
    U64,
    F64,
    /// u8 lane, values exactly 0 or 1 (C `bool`).
    BoolU8,
    /// u32 lane carrying a Unicode scalar value (C `char32_t`).
    CharU32,
}

impl PhysLane {
    /// The C spelling used in generated/hand-written headers. Frozen:
    /// the independent hand-written header and the generated wrapper
    /// must agree on these tokens.
    pub fn c_spell(self) -> &'static str {
        match self {
            PhysLane::I32 => "int32_t",
            PhysLane::U32 => "uint32_t",
            PhysLane::I64 => "int64_t",
            PhysLane::U64 => "uint64_t",
            PhysLane::F64 => "double",
            PhysLane::BoolU8 => "bool",
            PhysLane::CharU32 => "char32_t",
        }
    }

    /// Register bank the lane travels in on Windows x64 (positional
    /// convention: argument position < 4 uses that position's register —
    /// GP for integer-class, XMM for F64 — position >= 4 is the stack).
    pub fn slot_class(self) -> SlotBank {
        match self {
            PhysLane::I32 | PhysLane::U32 | PhysLane::I64 | PhysLane::U64 => SlotBank::Gp,
            PhysLane::BoolU8 | PhysLane::CharU32 => SlotBank::Gp,
            PhysLane::F64 => SlotBank::Fp,
        }
    }
}

/// Argument register bank (Windows x64: 4 GP + 4 FP argument registers).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum SlotBank {
    Gp,
    Fp,
}

/// One lane VALUE at the boundary.
///
/// `f64` values are carried as RAW BITS so NaN payloads and -0.0 survive
/// every Rust-side hop bit-exactly; conversion happens only at the very
/// edge (emitter constant materialization / C literal spelling).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LaneValue {
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    /// Exact f64 bit pattern (payload-preserving).
    F64Bits(u64),
    /// Validated to be 0/1 before any call.
    Bool(bool),
    /// Validated to be a Unicode scalar value before any call.
    CharScalar(u32),
    /// Explicit fallible ctx lane value (an address; nullability is a
    /// property of the declared form, not of the value).
    CtxPtr(usize),
}

impl LaneValue {
    /// Strict per-lane VALUE validation (937 plan §5.3: invalid bool /
    /// non-scalar char are rejected BEFORE any call, callee counter 0).
    pub fn validate(kind: LaneKind, v: &LaneValue) -> Result<(), String> {
        let mismatch = |want: &str| Err(format!(
            "lane value kind mismatch: lane {kind:?} expects {want}, got {v:?}",
            kind = kind
        ));
        match (kind, v) {
            (LaneKind::I32, LaneValue::I32(_)) => Ok(()),
            (LaneKind::U32, LaneValue::U32(_)) => Ok(()),
            (LaneKind::I64, LaneValue::I64(_)) => Ok(()),
            (LaneKind::U64, LaneValue::U64(_)) => Ok(()),
            (LaneKind::F64, LaneValue::F64Bits(_)) => Ok(()),
            (LaneKind::Bool, LaneValue::Bool(b)) => {
                // Rust bool is already 0/1; nothing wider can be built.
                let _ = b;
                Ok(())
            }
            (LaneKind::Char, LaneValue::CharScalar(c)) => {
                if char::from_u32(*c).is_some() {
                    Ok(())
                } else {
                    Err(format!(
                        "char lane value U+{c:04X} is not a Unicode scalar value (rejected before call)"
                    ))
                }
            }
            (LaneKind::I32, _) => mismatch("i32"),
            (LaneKind::U32, _) => mismatch("u32"),
            (LaneKind::I64, _) => mismatch("i64"),
            (LaneKind::U64, _) => mismatch("u64"),
            (LaneKind::F64, _) => mismatch("f64 bits"),
            (LaneKind::Bool, _) => mismatch("bool"),
            (LaneKind::Char, _) => mismatch("char scalar"),
        }
    }

    /// Convenience: an f64 value carried payload-exact.
    pub fn f64(v: f64) -> LaneValue {
        LaneValue::F64Bits(v.to_bits())
    }

    /// Read back as f64 (payload-exact via bits).
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            LaneValue::F64Bits(b) => Some(f64::from_bits(*b)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_f32_lane_exists() {
        // The public vocabulary is exactly the seven lanes; f32 has no
        // spelling and no projection.
        assert_eq!(LaneKind::ALL.len(), 7);
        assert!(LaneKind::of_spelling("f32").is_none());
        for k in LaneKind::ALL {
            assert!(k.physical().c_spell() != "float");
        }
    }

    #[test]
    fn char_scalar_range_enforced() {
        assert!(LaneValue::validate(LaneKind::Char, &LaneValue::CharScalar(0)).is_ok());
        assert!(LaneValue::validate(LaneKind::Char, &LaneValue::CharScalar(0x10FFFF)).is_ok());
        // Lone surrogate halves and out-of-range values are rejected.
        assert!(LaneValue::validate(LaneKind::Char, &LaneValue::CharScalar(0xD800)).is_err());
        assert!(LaneValue::validate(LaneKind::Char, &LaneValue::CharScalar(0xDFFF)).is_err());
        assert!(LaneValue::validate(LaneKind::Char, &LaneValue::CharScalar(0x110000)).is_err());
    }

    #[test]
    fn f64_bits_preserve_nan_payload_and_neg_zero() {
        let qnan = f64::from_bits(0x7FF8_0000_DEAD_BEEF); // quiet NaN w/ payload
        let v = LaneValue::f64(qnan);
        assert_eq!(v.as_f64().unwrap().to_bits(), 0x7FF8_0000_DEAD_BEEF);
        let nz = LaneValue::f64(-0.0);
        assert_eq!(nz.as_f64().unwrap().to_bits(), (-0.0f64).to_bits());
        assert_ne!(nz.as_f64().unwrap().to_bits(), (0.0f64).to_bits());
    }

    #[test]
    fn lane_kind_value_mismatch_is_named_rejection() {
        let e = LaneValue::validate(LaneKind::I32, &LaneValue::U32(1)).unwrap_err();
        assert!(e.contains("kind mismatch"), "{e}");
    }
}
