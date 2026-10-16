//! Frozen, bounded declaration IR feeding the Cranelift emitter (plan §0,
//! §5.2). This is the whole "front end" of the T-01 probe: a typed enum a
//! test can construct, NOT a parser and NOT the Auto language surface.
//! T-02/T-03 generalize it into the descriptor-driven contract; the point
//! frozen here is that every emitted module comes from an explicit,
//! inspectable declaration — never from hand-patched Cranelift snippets
//! duplicated per leg.

/// Scalar kinds legal in the T-01 candidate-profile signatures.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AbiScalar {
    I32,
    I64,
    F64,
}

/// Frozen body families the emitter knows how to lower. `MixV1` is the
/// heterogeneous weighted-sum probe body; its semantics are frozen in
/// contract-decision.md §2 and mirrored — independently — by the Rust
/// oracle ([`oracle_mix_v1`]), the Cranelift body (emit.rs) and the C
/// bodies in `cside/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BodyKind {
    MixV1,
}

/// One public AutoABI function declaration (T-01 shape: single scalar
/// return; status/out forms arrive with the T-02/T-03 contract).
pub struct FnDecl {
    pub symbol: &'static str,
    pub params: &'static [AbiScalar],
    pub ret: AbiScalar,
    pub body: BodyKind,
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
