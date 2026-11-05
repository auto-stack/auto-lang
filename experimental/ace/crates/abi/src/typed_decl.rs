//! PLAN-937 CA-D1 (T-04): the frozen TYPED-lane probe declarations and
//! their independent Rust oracles.
//!
//! The typed lane proof (V-03) runs a heterogeneous 8-lane public
//! signature — `i32, u32, i64, u64, f64, bool, char, i64 -> i64` — plus
//! a fallible typed div. Business bodies stay in the FROZEN family set
//! (plan §2: no native source lowering beyond the shared path): the
//! checksum folds every lane's EXACT BITS into an i64 (f64 lanes by raw
//! bit pattern — quiet-NaN payloads and -0.0 participate bit-exactly;
//! bool as 0/1; char by its Unicode scalar value), and the fallible div
//! is the frozen controlled-failure family.
//!
//! THREE independent implementations pin the semantics: the Rust oracle
//! here, the Cranelift bodies in [`crate::typed_emit`], and the
//! hand-written freestanding C in `cside/bridge_typed*.c/h`. The
//! expected values baked into callers come from the Rust oracle, and
//! HAND-COMPUTED literals below guard against a broken oracle silently
//! redefining "expected".

use ace_foreign_contract::lane::{LaneKind, LaneValue};
use ace_foreign_contract::typed::{TotalityEvidence, TypedBodyFamily, TypedDecl, TypedDecls};

/// Per-position mix constants (distinct 64-bit values; unsigned
/// arithmetic — C computes in uint64_t, Rust in wrapping i64, Cranelift
/// in machine i64: all three agree bitwise).
pub const T937_MIX_C: [u64; 8] = [
    0x9E37_79B9_7F4A_7C15,
    0xC2B2_AE3D_27D4_EB4F,
    0x1656_67B1_9E37_79F9,
    0x27D4_EB2F_1656_67C5,
    0x85EB_CA77_C2B2_AE63,
    0x2545_F491_4F6D_DC1D,
    0x9E37_79B1_85EB_CA87,
    0xF39C_C060_5CED_C834,
];

/// Fallible typed-div status codes (the explicit table the facts carry).
pub mod t937_status {
    pub const OK: i32 = 0;
    pub const ERR_DIV_ZERO: i32 = -1;
}

/// The frozen typed checksum inputs — asymmetric extremes from every
/// lane: negative i32, high-bit u32, near-MIN i64, near-MAX u64, a
/// quiet-NaN f64 with payload, true bool, the LAST Unicode scalar
/// value, and a near-MAX positive i64.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypedInputs {
    pub a: i32,
    pub b: u32,
    pub c: i64,
    pub d: u64,
    /// RAW f64 bit pattern (payload-preserving; NaN stays NaN).
    pub f_bits: u64,
    pub g: bool,
    /// Unicode scalar value of the char lane.
    pub h: u32,
    pub i: i64,
}

pub const FROZEN_TYPED_INPUTS: TypedInputs = TypedInputs {
    a: -1_234_567,
    b: 0xDEAD_BEEF,
    c: i64::MIN + 123,
    d: u64::MAX - 7,
    f_bits: 0x7FF8_0000_DEAD_BEEF, // quiet NaN with payload
    g: true,
    h: 0x0010_FFFF, // U+10FFFF (last scalar value)
    i: 0x7FFF_FFFF_FFFF_F000,
};

/// Hand-computed expected (independent Python fold of the same frozen
/// semantics — guards the Rust oracle):
/// 5791786002463203090 = 0x50608EAEDBE61F12 (Python fold with the
/// CORRECT c = i64::MIN + 123 = -9223372036854775685).
pub const HAND_COMPUTED_TYPED_EXPECTED: i64 = 5_791_786_002_463_203_090;

/// Independent Rust oracle of the frozen checksum body.
pub fn oracle_typed_mix(x: &TypedInputs) -> i64 {
    let m = |v: u64, w: u64| v.wrapping_mul(w);
    let sext_a = x.a as i64 as u64;
    let wa = m(sext_a, T937_MIX_C[0]);
    let wb = m(x.b as u64, T937_MIX_C[1]);
    let wc = m(x.c as u64, T937_MIX_C[2]);
    let wd = m(x.d, T937_MIX_C[3]);
    let wf = m(x.f_bits, T937_MIX_C[4]);
    let wg = m(x.g as u8 as u64, T937_MIX_C[5]);
    let wh = m(x.h as u64, T937_MIX_C[6]);
    let wi = m(x.i as u64, T937_MIX_C[7]);
    let acc = wa ^ wb;
    let acc = acc.wrapping_add(wc);
    let acc = acc ^ wd;
    let acc = acc.wrapping_add(wf);
    let acc = acc ^ wg;
    let acc = acc.wrapping_add(wh);
    (acc ^ wi) as i64
}

/// Fallible typed-div oracle (frozen family: d == 0 is the controlled
/// failure, out untouched; else wrapping trunc quotient).
pub fn oracle_typed_div(n: i64, d: i64) -> Result<i64, i32> {
    if d == 0 {
        Err(t937_status::ERR_DIV_ZERO)
    } else {
        Ok(n.wrapping_div(d))
    }
}

/// The frozen typed-lane facts (the input identity of the typed chain).
pub fn typed_decls() -> TypedDecls {
    TypedDecls {
        format: ace_foreign_contract::typed::TYPED_DECLS_FORMAT.into(),
        id: "t937".into(),
        decls: vec![
            TypedDecl {
                symbol: "t937_mix".into(),
                params: vec![
                    LaneKind::I32,
                    LaneKind::U32,
                    LaneKind::I64,
                    LaneKind::U64,
                    LaneKind::F64,
                    LaneKind::Bool,
                    LaneKind::Char,
                    LaneKind::I64,
                ],
                ret: LaneKind::I64,
                body: TypedBodyFamily::ChecksumV1,
                totality: Some(TotalityEvidence::frozen_family(
                    "bit-fold only: wrapping arithmetic on raw lane bits; no checked ops, no failure path",
                )),
                error_model: None,
                ctx: None,
                outs: vec![],
            },
            TypedDecl {
                symbol: "t937_div".into(),
                params: vec![LaneKind::I64, LaneKind::I64],
                ret: LaneKind::I32,
                body: TypedBodyFamily::FallibleDivV1,
                totality: None,
                error_model: Some(ace_foreign_contract::facts::ErrorModel {
                    kind: "status-i32".into(),
                    codes: vec![(t937_status::ERR_DIV_ZERO, "div-by-zero".into())],
                }),
                ctx: Some(true), // nullable ctx (never dereferenced)
                outs: vec![LaneKind::I64],
            },
        ],
    }
}

/// Lane VALUES of the frozen inputs (for pre-call validation and C
/// caller generation).
pub fn frozen_lane_values() -> [LaneValue; 8] {
    let x = FROZEN_TYPED_INPUTS;
    [
        LaneValue::I32(x.a),
        LaneValue::U32(x.b),
        LaneValue::I64(x.c),
        LaneValue::U64(x.d),
        LaneValue::F64Bits(x.f_bits),
        LaneValue::Bool(x.g),
        LaneValue::CharScalar(x.h),
        LaneValue::I64(x.i),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_matches_hand_computed_frozen_value() {
        assert_eq!(
            oracle_typed_mix(&FROZEN_TYPED_INPUTS),
            HAND_COMPUTED_TYPED_EXPECTED
        );
    }

    #[test]
    fn every_lane_is_observed_by_the_checksum() {
        let base = oracle_typed_mix(&FROZEN_TYPED_INPUTS);
        // flip one bit per lane; the result must change each time
        let mut x = FROZEN_TYPED_INPUTS;
        x.a ^= 1;
        assert_ne!(oracle_typed_mix(&x), base);
        let mut x = FROZEN_TYPED_INPUTS;
        x.b ^= 1 << 31;
        assert_ne!(oracle_typed_mix(&x), base);
        let mut x = FROZEN_TYPED_INPUTS;
        x.c ^= 1;
        assert_ne!(oracle_typed_mix(&x), base);
        let mut x = FROZEN_TYPED_INPUTS;
        x.d ^= 1;
        assert_ne!(oracle_typed_mix(&x), base);
        let mut x = FROZEN_TYPED_INPUTS;
        x.f_bits ^= 1; // NaN PAYLOAD bit — must be observable
        assert_ne!(oracle_typed_mix(&x), base);
        let mut x = FROZEN_TYPED_INPUTS;
        x.g = false;
        assert_ne!(oracle_typed_mix(&x), base);
        let mut x = FROZEN_TYPED_INPUTS;
        x.h ^= 1;
        assert_ne!(oracle_typed_mix(&x), base);
        let mut x = FROZEN_TYPED_INPUTS;
        x.i ^= 1;
        assert_ne!(oracle_typed_mix(&x), base);
    }

    #[test]
    fn nan_payload_and_neg_zero_fold_by_bits() {
        // same exponent/mantissa-class, different payload -> different fold
        let mut x = FROZEN_TYPED_INPUTS;
        x.f_bits = 0x7FF8_0000_DEAD_BEEE;
        assert_ne!(oracle_typed_mix(&x), HAND_COMPUTED_TYPED_EXPECTED);
        // -0.0 vs +0.0 differ by sign bit only
        let nz = TypedInputs {
            f_bits: (-0.0f64).to_bits(),
            ..FROZEN_TYPED_INPUTS
        };
        let pz = TypedInputs {
            f_bits: 0.0f64.to_bits(),
            ..FROZEN_TYPED_INPUTS
        };
        assert_ne!(oracle_typed_mix(&nz), oracle_typed_mix(&pz));
    }

    #[test]
    fn div_oracle_is_the_frozen_family() {
        assert_eq!(oracle_typed_div(20, 2), Ok(10));
        assert_eq!(oracle_typed_div(-7, 3), Ok(-2));
        assert_eq!(oracle_typed_div(i64::MIN, -1), Ok(i64::MIN)); // wrapping
        assert_eq!(oracle_typed_div(1, 0), Err(t937_status::ERR_DIV_ZERO));
    }

    #[test]
    fn typed_decls_project_to_valid_facts_and_plan() {
        let facts = typed_decls().validate().unwrap();
        assert!(facts.function("t937_mix").is_some());
        assert!(facts.function("t937_div").is_some());
        let plan = ace_foreign_contract::PhysicalPlan::project(&facts).unwrap();
        assert!(plan.function("t937_mix").is_some());
        // pre-call validation passes for the frozen values
        assert!(facts
            .validate_call_values("t937_mix", None, &frozen_lane_values())
            .is_ok());
    }
}
