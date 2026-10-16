//! Frozen, bounded declaration IR feeding the Cranelift emitter (plan §0,
//! §5.2). This is the whole "front end" of the T-01 probe: a typed enum a
//! test can construct, NOT a parser and NOT the Auto language surface.
//! T-02/T-03 generalize it into the descriptor-driven contract; the point
//! frozen here is that every emitted module comes from an explicit,
//! inspectable declaration — never from hand-patched Cranelift snippets
//! duplicated per leg.

/// Scalar kinds legal in the candidate-profile signatures. `Ptr` is the
/// raw pointer lane (i64): descriptor spells it as the specific view/out
/// role (`const RecMix*`, `out i64*`) — the raw-address validity stays a
/// caller contract (plan §5.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AbiScalar {
    I32,
    I64,
    F64,
    Ptr,
}

/// Frozen body families the emitter knows how to lower (T-03 suite).
/// Each has exactly ONE semantic, frozen in contract-decision.md §2/§7
/// and mirrored independently by the Rust oracles here, the Cranelift
/// bodies in emit.rs, and the hand-written C in `cside/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BodyKind {
    /// T-01 heterogeneous weighted sum.
    MixV1,
    /// Nested-call inner helper: x*3 + 11.
    InnerV1,
    /// f64 return path: a*2.5 + b*1.25 (binary-exact coefficients).
    FAccV1,
    /// Nested call: p797_inner((a as i64)*7 + trunc(b)*100) + a.
    OuterV1,
    /// Fallible form: d==0 -> status -1 and out untouched; else *q=n/d
    /// (trunc), status 0.
    FallibleDivV1,
    /// const RecMix view: flag*1000 + count*3 + tag*5 + big (record
    /// fields read at the FROZEN offsets, so any layout drift shows up
    /// in the result).
    RecSumV1,
    /// Observable counter: module-global i64 += x (wrapping), returns
    /// the new total — missed or extra calls are observable.
    BumpV1,
}

/// One public AutoABI function declaration.
pub struct FnDecl {
    pub symbol: &'static str,
    pub params: &'static [AbiScalar],
    pub ret: AbiScalar,
    pub body: BodyKind,
}

/// Frozen status codes of the fallible form (contract-decision §2).
pub mod status {
    pub const OK: i32 = 0;
    pub const ERR_DIV_ZERO: i32 = -1;
}

pub const MIX_PARAMS: &[AbiScalar] = &[
    AbiScalar::I32, // a
    AbiScalar::I32, // b
    AbiScalar::I32, // c
    AbiScalar::I32, // d
    AbiScalar::I64, // e
    AbiScalar::F64, // f
    AbiScalar::I32, // g
    AbiScalar::F64, // h
];

pub const P797_MIX: FnDecl = FnDecl {
    symbol: "p797_mix",
    params: MIX_PARAMS,
    ret: AbiScalar::I64,
    body: BodyKind::MixV1,
};

pub const P797_INNER: FnDecl = FnDecl {
    symbol: "p797_inner",
    params: &[AbiScalar::I64],
    ret: AbiScalar::I64,
    body: BodyKind::InnerV1,
};

pub const P797_FACC: FnDecl = FnDecl {
    symbol: "p797_facc",
    params: &[AbiScalar::F64, AbiScalar::F64],
    ret: AbiScalar::F64,
    body: BodyKind::FAccV1,
};

pub const P797_OUTER: FnDecl = FnDecl {
    symbol: "p797_outer",
    params: &[AbiScalar::I32, AbiScalar::F64],
    ret: AbiScalar::I64,
    body: BodyKind::OuterV1,
};

pub const P797_DIV: FnDecl = FnDecl {
    symbol: "p797_div",
    params: &[AbiScalar::I64, AbiScalar::I64, AbiScalar::Ptr],
    ret: AbiScalar::I32,
    body: BodyKind::FallibleDivV1,
};

pub const P797_REC_SUM: FnDecl = FnDecl {
    symbol: "p797_rec_sum",
    params: &[AbiScalar::Ptr],
    ret: AbiScalar::I64,
    body: BodyKind::RecSumV1,
};

pub const P797_BUMP: FnDecl = FnDecl {
    symbol: "p797_bump",
    params: &[AbiScalar::I32],
    ret: AbiScalar::I64,
    body: BodyKind::BumpV1,
};

/// Frozen T-01 inputs (mirror of `cside/autoabi_v01.h` P797_* macros; the
/// header is authoritative for the C side, this constant for the Rust
/// side, and the header-vs-descriptor consistency check in T-02 compares
/// the two rather than generating either).
#[derive(Clone, Copy, Debug)]
pub struct MixInputs {
    pub a: i32,
    pub b: i32,
    pub c: i32,
    pub d: i32,
    pub e: i64,
    pub f: f64,
    pub g: i32,
    pub h: f64,
}

pub const FROZEN_INPUTS: MixInputs = MixInputs {
    a: 7,
    b: -3,
    c: 100,
    d: 5,
    e: 1_000_000_007,
    f: 1.5,
    g: 11,
    h: -2.25,
};

/// Independent Rust oracle for the MixV1 body. This is one of THREE
/// independent implementations (Rust oracle here, Cranelift body in
/// emit.rs, hand-written C in `cside/`); the expected values baked into
/// probe callers come from here, so a bug in the emitter body shows up as
/// an exit-code mismatch, never as a self-consistent pass.
///
/// f64 -> i64 mirrors the frozen profile rule: truncation toward zero,
/// inputs frozen in-range (no C UB, no fp->int trap).
pub fn oracle_mix_v1(x: &MixInputs) -> i64 {
    let t = |v: f64| v as i64; // trunc toward zero for in-range values
    x.e.wrapping_mul(2)
        .wrapping_add(x.a as i64)
        .wrapping_add((x.b as i64) * 3)
        .wrapping_add((x.c as i64) * 7)
        .wrapping_add((x.d as i64) * 11)
        .wrapping_add((x.g as i64) * 13)
        .wrapping_add(t(x.f) * 1000)
        .wrapping_add(t(x.h) * 17)
}

/// Hand-computed reference for the frozen inputs, baked into the test as
/// a literal so a broken oracle cannot silently redefine "expected"
/// (derivation: 2000000014 + 7 - 9 + 700 + 55 + 143 + 1000 - 34).
pub const HAND_COMPUTED_FROZEN_EXPECTED: i64 = 2_000_001_876;

// ---------------------------------------------------------------------------
// T-03 suite inputs + independent Rust oracles (mirror: emit.rs bodies,
// cside/interop_*.c bodies — three independent implementations).
// ---------------------------------------------------------------------------

/// f64 fixtures: binary-exact coefficients and inputs, so `==` on the
/// wire is the oracle, not an epsilon policy (plan §5.2).
pub const FROZEN_FACC: (f64, f64) = (1.5, -0.75);

pub fn oracle_facc_v1(a: f64, b: f64) -> f64 {
    a * 2.5 + b * 1.25
}

/// Nested-call inner: x*3 + 11.
pub fn oracle_inner_v1(x: i64) -> i64 {
    x.wrapping_mul(3).wrapping_add(11)
}

pub const FROZEN_OUTER: (i32, f64) = (13, 2.25);

pub fn oracle_outer_v1(a: i32, b: f64) -> i64 {
    let t = b as i64; // trunc toward zero, frozen in-range
    oracle_inner_v1((a as i64).wrapping_mul(7).wrapping_add(t.wrapping_mul(100)))
        .wrapping_add(a as i64)
}

/// Fallible div oracle: d == 0 is the frozen controlled failure (status
/// [`status::ERR_DIV_ZERO`], out untouched); otherwise trunc quotient.
pub fn oracle_div_v1(n: i64, d: i64) -> Result<i64, i32> {
    if d == 0 {
        Err(status::ERR_DIV_ZERO)
    } else {
        Ok(n.wrapping_div(d))
    }
}

/// Frozen RecMix field values (layout: flag@0 u8, count@4 i32, tag@8
/// u32, big@16 i64 — size 24, align 8; proven against the C compiler in
/// T-02).
#[derive(Clone, Copy, Debug)]
pub struct RecValues {
    pub flag: u8,
    pub count: i32,
    pub tag: u32,
    pub big: i64,
}

pub const FROZEN_REC: RecValues = RecValues {
    flag: 1,
    count: -7,
    tag: 1001,
    big: 99_999_999_999,
};

pub fn oracle_rec_sum_v1(r: &RecValues) -> i64 {
    (r.flag as i64)
        .wrapping_mul(1000)
        .wrapping_add((r.count as i64).wrapping_mul(3))
        .wrapping_add((r.tag as i64).wrapping_mul(5))
        .wrapping_add(r.big)
}

/// Encode RecValues into the frozen 24-byte little-endian wire form.
/// `tag_offset` exists ONLY for the wrong-offset negative control: pass
/// 12 to place tag at a WRONG offset and watch the callee's correct
/// offsets diverge from the expectation.
pub fn encode_rec_bytes(r: &RecValues, tag_offset: u32) -> [u8; 24] {
    let mut b = [0u8; 24];
    b[0] = r.flag;
    b[4..8].copy_from_slice(&r.count.to_le_bytes());
    b[tag_offset as usize..tag_offset as usize + 4].copy_from_slice(&r.tag.to_le_bytes());
    b[16..24].copy_from_slice(&r.big.to_le_bytes());
    b
}

/// Observable-counter fixture: five distinct increments.
pub const FROZEN_BUMPS: &[i32] = &[3, -1, 10, 0, 7];

pub fn oracle_bump_total(xs: &[i32]) -> i64 {
    xs.iter().map(|x| *x as i64).fold(0i64, i64::wrapping_add)
}
