//! PLAN-1079 language-parity observations through the ORDINARY Auto source
//! profile (`compile_language_source`) — the same front door the
//! `ac-av/language-source-run-v1` recipe runs. Oracles are independent
//! typed values, bit patterns and frozen diagnostic/trap codes, never a
//! value read back out of the implementation.
//!
//! These are the NAN-ZERO / literal / conversion observations the plan's §6
//! supplement IDs require, at a level where the exact bit pattern of a float
//! is observable (the console oracle can only see bytes, and the frozen
//! display contract deliberately cannot distinguish `-0.0` from `0.0`).

use ace_av_core::abc::{encode_abc, AbcBytes};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};

/// Compile/execute one Auto-profile program end to end (schema/CST → bound →
/// checked HIR → MIR → AVAD → AV), the same chain as the language recipe.
fn run(src: &str) -> ExecOutcome {
    let compiled = ace_core::source::compile_language_source(src, "p1079.at", &[])
        .unwrap_or_else(|d| panic!("Auto-profile compile failed: {d:?}"));
    let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0x1079)
        .expect("auto-profile lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let wire: AbcBytes = encode_abc(&mir).expect("AVAD encode");
    let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &mir.owner).expect("AVAD admits");
    let (out, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    out
}

fn ok_i64(src: &str) -> i64 {
    match run(src) {
        ExecOutcome::Success {
            value: Value::I64(v),
            ..
        } => v,
        other => panic!("expected an i64 success, got {other:?}"),
    }
}

fn ok_f32_bits(src: &str) -> u32 {
    match run(src) {
        ExecOutcome::Success {
            value: Value::F32(v),
            ..
        } => v.to_bits(),
        other => panic!("expected an f32 success, got {other:?}"),
    }
}

fn ok_f64_bits(src: &str) -> u64 {
    match run(src) {
        ExecOutcome::Success {
            value: Value::F64(v),
            ..
        } => v.to_bits(),
        other => panic!("expected an f64 success, got {other:?}"),
    }
}

fn trap_code(src: &str) -> String {
    match run(src) {
        ExecOutcome::Trap { kind, .. } => kind.code().to_string(),
        other => panic!("expected a trap, got {other:?}"),
    }
}

/// First diagnostic of a rejected source, as `(code, message)`.
fn compile_diagnostic(src: &str) -> (String, String) {
    match ace_core::source::compile_language_source(src, "p1079.at", &[]) {
        Ok(_) => panic!("expected a source diagnostic for `{src}`"),
        Err(diags) => {
            let d = diags.first().expect("at least one diagnostic");
            (d.code.clone(), d.message.clone())
        }
    }
}

fn compile_code(src: &str) -> String {
    match ace_core::source::compile_language_source(src, "p1079.at", &[]) {
        Ok(_) => panic!("expected a source diagnostic for `{src}`"),
        Err(diags) => diags
            .first()
            .map(|d| d.code.clone())
            .unwrap_or_else(|| panic!("expected at least one diagnostic")),
    }
}

// NAN-ZERO (NaN half): a float produced at RUN TIME by `%` with a ±0.0
// divisor is a NaN VALUE (the frozen `%` lane has no zero guard), and IEEE
// comparison semantics apply to it: it equals nothing, not even itself, and
// it is neither < nor > 0.0. A trap or a silent 0.0/±inf answer fails here.
#[test]
fn nan_from_zero_modulo_is_a_value_not_a_trap() {
    let v = ok_i64(
        "fn main() i64 {
    let z = 0.0d
    let n = 5.0d % z
    if n != n { return 7i64 }
    return 0i64
}",
    );
    assert_eq!(v, 7, "`x % ±0.0` yields NaN (self-inequality true)");

    let v = ok_i64(
        "fn main() i64 {
    let z = 0.0d
    let n = 5.0d % z
    if n < 0.0d { return 1i64 }
    if n > 0.0d { return 2i64 }
    if n == n { return 3i64 }
    if n == 0.0d { return 4i64 }
    return 5i64
}",
    );
    assert_eq!(v, 5, "NaN is unordered and unequal to every value");
}

// NAN-ZERO (-0 half): the sign of zero survives negation, addition of two
// negative zeros and the narrowing conversion to f32 — observed as the exact
// IEEE bit pattern, which the console display intentionally cannot show.
#[test]
fn negative_zero_bits_survive_arithmetic_and_narrowing() {
    let bits = ok_f64_bits(
        "fn main() f64 {
    let z = 0.0d
    let n = -z
    return n + n
}",
    );
    assert_eq!(
        bits,
        (-0.0f64).to_bits(),
        "(-0.0) + (-0.0) is -0.0 (IEEE sign rule)"
    );

    let bits = ok_f32_bits(
        "fn main() f32 {
    let z = 0.0d
    let n = -z
    return n.as(f32)
}",
    );
    assert_eq!(bits, (-0.0f32).to_bits(), "-0.0 narrows to f32 -0.0");

    let bits = ok_f64_bits(
        "fn main() f64 {
    let z = 0.0d
    return z
}",
    );
    assert_eq!(bits, 0.0f64.to_bits(), "+0.0 stays +0.0");
}

// NAN-ZERO (rounding half): f64 → f32 rounds to nearest-even, including the
// exact halfway case (1.0 + 2^-24 is representable in f64 but exactly halfway
// between the two neighbouring f32 values; ties-to-even picks 1.0).
#[test]
fn float_narrow_rounds_to_even() {
    let half = 5.9604644775390625e-8_f64; // 2^-24, exactly representable
    let want = (1.0f64 + half) as f32; // host IEEE round-to-nearest-even
    assert_eq!(want, 1.0, "the halfway case ties to even on the host too");
    let bits = ok_f32_bits(&format!(
        "fn main() f32 {{
    let one = 1.0d
    let eps = {half:.25}d
    return (one + eps).as(f32)
}}"
    ));
    assert_eq!(bits, want.to_bits(), "halfway value ties to even");

    let step = 1.1920928955078125e-7_f64; // 2^-23 = one f32 ulp at 1.0
    let want = (1.0f64 + step) as f32;
    let bits = ok_f32_bits(&format!(
        "fn main() f32 {{
    let one = 1.0d
    let eps = {step:.25}d
    return (one + eps).as(f32)
}}"
    ));
    assert_eq!(bits, want.to_bits(), "one ulp away converts exactly");
}

// Conversion matrix rails: narrowing that the target width cannot represent is
// a CHECKED trap (`av.trap.int-overflow`), never a silent truncation; mixed
// widths without an explicit conversion are a source rejection, not an
// implicit promotion (the named difference from the legacy matrix).
#[test]
fn conversions_are_checked_and_mixed_widths_are_source_rejections() {
    assert_eq!(
        trap_code(
            "fn main() int {
    let big = 4294967296i64
    return big.to(int)
}"
        ),
        "av.trap.int-overflow",
        "i64 -> int narrowing out of range traps"
    );
    assert_eq!(
        compile_code(
            "fn main() i64 {
    let a = 1i64 + 1
    return a
}"
        ),
        "source.type-mismatch",
        "mixed widths need an explicit conversion"
    );
    assert_eq!(
        compile_code(
            "fn main() int {
    let a = 300u8
    let b = a.as(int)
    return b
}"
        ),
        "source.literal-range",
        "a literal outside its target width is a source diagnostic"
    );
    // Review F-02: a radix literal carrying a fraction keeps its OWN named
    // rejection (not the generic "unknown suffix `.5`"), i.e. the fraction
    // branch in `decode_typed_literal` stays reachable.
    for src in [
        "fn main() int {\n    let a = 0x1F.5\n    return 42\n}",
        "fn main() int {\n    let a = 0b1010.5\n    return 42\n}",
    ] {
        let (code, message) = compile_diagnostic(src);
        assert_eq!(code, "source.literal-range", "radix fraction is rejected");
        assert!(
            message.contains("fraction"),
            "radix fraction must name the fraction, got {message:?}"
        );
    }
}

// T-02 regressions for the literal decoder: radix spellings whose bodies carry
// hex letters or binary digits only, and `_` digit separators, decode to the
// exact magnitudes they spell (pre-1079 the Auto face rejected them).
#[test]
fn radix_and_underscore_literals_decode() {
    let v = ok_i64(
        "fn main() i64 {
    let a = 0xFF
    let b = 0b1010
    let c = 1_000_000
    let d = 0xFFu8
    return a.as(i64) + b.as(i64) + c.as(i64) + d.as(i64)
}",
    );
    assert_eq!(v, 255 + 10 + 1_000_000 + 255);
    // The documented hex/suffix corner: a 16-radix body takes the longest
    // run of hex digits, so `0x1Fb` is 507 and the `b` (byte) suffix is not
    // available after hex digits.
    let corner = ok_i64("fn main() i64 { let x = 0x1Fb\n return x.as(i64) }");
    assert_eq!(corner, 507, "`0x1Fb` reads as the hex number 507");
}

/// The Numeric profile shares the same decoder, so the frozen suffix
/// vocabulary is pinned here at the EXACT magnitude/width — the 961 vocabulary
/// matrix accepts `ok | literal-range | type` and therefore never caught the
/// radix decode defect (`0x1Fi8` was reported as "carries no digits").
///
/// The last row pins the documented corner: a 16-radix body takes the longest
/// run of hex digits, so `0x1Fb` is the hex number 507 and NOT `0x1F` byte —
/// the `b` suffix is therefore unavailable after hex digits (spell a hex byte
/// as `0xFFu8` or convert with `.as(u8)`). The legacy decoder had no reading
/// for the form at all, so this is the rule the batch freezes.
#[test]
fn numeric_profile_suffix_vocabulary_decodes_exactly() {
    for (word, ret_ty, want_ty, want_bits) in [
        ("0x1Fi8", "i8", ace_core::hir::BuiltinType::I8, 31u64),
        ("0b101b", "u8", ace_core::hir::BuiltinType::U8, 5),
        ("1_000_000u32", "u32", ace_core::hir::BuiltinType::U32, 1_000_000),
        ("0xFFu8", "u8", ace_core::hir::BuiltinType::U8, 255),

    ] {
        let src = format!("fn f() int {{ return 0 }}\nfn h() {ret_ty} {{ return {word} }}");
        let parts = ace_core::source::adapter::compile_parts_with(
            &src,
            "n_lit.at",
            &[],
            ace_core::source::SourceProfile::Numeric,
        )
        .unwrap_or_else(|d| panic!("numeric compile failed for `{word}`: {d:?}"));
        let checked =
            ace_core::verify::verify_closures(parts.bundle).expect("numeric bundle verifies");
        // Collect the typed constants of EVERY body — body order is not the
        // source order, and the untyped `0` of the filler function is an
        // ordinary constant (never a NumericConst).
        let consts: Vec<(ace_core::hir::BuiltinType, u64)> = checked
            .bundle()
            .modules
            .iter()
            .flat_map(|m| m.bodies.iter())
            .flat_map(|b| b.exprs.iter())
            .filter_map(|e| match &e.kind {
                ace_core::hir::ExprKind::NumericConst { ty, bits } => Some((*ty, *bits)),
                _ => None,
            })
            .collect();
        assert_eq!(
            consts,
            vec![(want_ty, want_bits)],
            "`{word}` must decode to exactly its magnitude and width"
        );
    }
}

// NUL and UTF-8 view separation on the ordinary Auto face: `len()` counts
// scalars, `len_bytes()` counts UTF-8 bytes, and the embedded NUL is an
// ordinary scalar (a C-string shape would answer 3/8 instead of 4/9).
#[test]
fn nul_carrying_text_keeps_scalar_and_byte_lengths() {
    let v = ok_i64(
        "fn main() i64 {
    let s = \"A中😀\\u{0}\".clone()
    return s.len().as(i64) * 10i64 + s.len_bytes().as(i64)
}",
    );
    assert_eq!(v, 49, "4 scalars * 10 + 9 UTF-8 bytes");

    let v = ok_i64(
        "fn main() i64 {
    let s = \"A中😀\\u{0}\".clone()
    return s.char_at(3).to_codepoint().as(i64)
}",
    );
    assert_eq!(v, 0, "the fourth scalar is the embedded NUL");
}
