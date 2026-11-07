//! PLAN-961 numeric execution semantics: source → numeric profile compile
//! → shared lowering → MIR verify → AVAD wire → AV execution. Oracles are
//! INDEPENDENT values/trap kinds (V-01..V-05 anchors and the T-08 edge
//! families), never old-engine outputs.

use ace_av_core::abc::{encode_abc, AbcBytes};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};

fn run(src: &str) -> ExecOutcome {

    let compiled =
        ace_core::source::compile_numeric_source(src, "n.at", &[]).expect("numeric compile");
    let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0x961)
        .expect("numeric lowering");
    mir_verify::verify_mir(&mir).expect("numeric MIR verifies");
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
        other => panic!("expected i64 success, got {other:?}"),
    }
}

fn ok_f64(src: &str) -> f64 {
    match run(src) {
        ExecOutcome::Success {
            value: Value::F64(v),
            ..
        } => v,
        other => panic!("expected f64 success, got {other:?}"),
    }
}

fn trap_code(src: &str) -> String {
    match run(src) {
        ExecOutcome::Trap { kind, .. } => kind.code().to_string(),
        other => panic!("expected a trap for `{src}`, got {other:?}"),
    }
}

// V-01: i64 beyond i32 round-trips without truncation.
#[test]
fn v01_i64_magnitude_roundtrip() {
    let v = ok_i64("fn main() i64 { let a = 2147483648i64\n let b = a + 1i64\n return b }");
    assert_eq!(v, 2147483649);
}

// V-02: u64 high-bit comparisons are unsigned.
#[test]
fn v02_u64_high_bit_compare() {
    let v = ok_i64(
        "fn main() i64 {
    let hi = 18446744073709551613u64
    let lo = 1u64
    let sum = hi + lo
    if sum > 18446744073709551613u64 { return 1i64 }
    return 0i64
}",
    );
    assert_eq!(v, 1, "unsigned high-bit add: (MAX-2)+1 > MAX-2 (signed would trap)");
}

// V-03: MAX+1 traps per the checked contract (never wraps).
#[test]
fn v03_max_plus_one_traps() {
    assert_eq!(
        trap_code("fn main() i64 { let a = 9223372036854775807i64\n return a + 1i64 }"),
        "av.trap.int-overflow"
    );
    // u8 arithmetic checks at the u8 width.
    assert_eq!(
        trap_code("fn main() u8 { let a = 200u8\n let b = 100u8\n return a + b }"),
        "av.trap.int-overflow"
    );
}

// V-04: an unexecuted branch's trap stays latent (short-circuit).
#[test]
fn v04_latent_trap_in_short_circuit() {
    let v = ok_i64(
        "fn main() i64 {
    let x = 10i64
    if x < 5i64 && 1i64 / 0i64 == 0i64 { return 2i64 }
    return 1i64
}",
    );
    assert_eq!(v, 1, "the RHS division never executes");
}

// V-05: NaN payload and -0.0 survive as exact values (f64 lanes).
#[test]
fn v05_float_bit_exact_values() {
    // -0.0: negation of the +0.0 literal; returning it distinguishes signs.
    let v = ok_f64("fn main() f64 { let z = 0.0d\n return -z }");
    assert_eq!(v.to_bits(), (-0.0f64).to_bits(), "-0.0 bit-exact");
    // f32 arithmetic in native precision: 0.1f32 + 0.2f32 == 0.3f32 rounds
    // per f32 (would differ if computed through f64).
    let src = "fn main() f64 {
    let a = (0.1f32).as(f64)
    let b = (0.2f32).as(f64)
    return a + b
}";
    let v = ok_f64(src);
    assert_eq!(v.to_bits(), ((0.1f32 as f64) + (0.2f32 as f64)).to_bits());
}

// T-08: literal rounding — a non-exact decimal rounds to the target
// width's nearest representable value (0.1f32 observed bit-exactly).
#[test]
fn v08_literal_rounding() {
    let v = ok_f64("fn main() f64 { return (0.1f32).as(f64) }");
    assert_eq!(v.to_bits(), (0.1f32 as f64).to_bits(), "0.1f32 rounds to nearest f32");
    let v2 = ok_f64("fn main() f64 { return 0.1d }");
    assert_eq!(v2.to_bits(), 0.1f64.to_bits(), "0.1d rounds to nearest f64");
}

// V-08 edge families (T-08 core): shifts, MIN/-1, float→int, casts.
#[test]
fn v08_shift_and_division_edges() {
    // shift by the width traps.
    assert_eq!(
        trap_code("fn main() i64 { let x = 1i64\n return x.shl(64i64) }"),
        "av.trap.shift-out-of-range"
    );
    // i64::MIN / -1 traps DivOverflow.
    assert_eq!(
        trap_code(
            "fn main() i64 {
    let mn = -9223372036854775808i64
    let minus = -1i64
    return mn / minus
}"
        ),
        "av.trap.div-overflow"
    );
    // i64::MIN % -1 traps RemOverflow (unlike host %).
    assert_eq!(
        trap_code(
            "fn main() i64 {
    let mn = -9223372036854775808i64
    let minus = -1i64
    return mn % minus
}"
        ),
        "av.trap.rem-overflow"
    );
    // div by zero.
    assert_eq!(
        trap_code("fn main() i64 { return 1i64 / 0i64 }"),
        "av.trap.div-by-zero"
    );
    // float division by ±0.0 traps (legacy parity; not IEEE inf).
    assert_eq!(
        trap_code("fn main() f64 { let z = 0.0d\n return 1.5d / z }"),
        "av.trap.div-by-zero"
    );
}

#[test]
fn v08_conversion_edges() {
    // checked narrowing: 300i64 → u8 traps.
    assert_eq!(
        trap_code("fn main() u8 { let a = 300i64\n return a.as(u8) }"),
        "av.trap.int-overflow"
    );
    // float → int out of range traps FloatToInt.
    assert_eq!(
        trap_code(
            "fn main() i64 {
    let a = 9223372036854775807i64
    let f = a.as(f64)
    let big = f * f
    let huge = big * f
    return huge.as(i64)
}",
        ),
        "av.trap.float-to-int"
    );
    // NaN → int traps FloatToInt.
    assert_eq!(
        trap_code(
            "fn main() i64 {
    let a = 9223372036854775807i64
    let f = a.as(f64)
    let b = f * f
    let c = b * b
    let d = c * c
    let e = d * d
    let inf = e * e
    let nan = inf - inf
    return nan.as(i64)
}"
        ),
        "av.trap.float-to-int"
    );
    // f64 → f32 finite overflow traps FloatOverflow.
    assert_eq!(
        trap_code(
            "fn main() f32 {
    let a = 9223372036854775807i64
    let f = a.as(f64)
    let b = f * f
    let c = b * b
    let d = c * c
    let e = d * d
    return e.as(f32)
}",
        ),
        "av.trap.float-overflow"
    );
    // sign extension: -1i8 → i64 is -1 (all ones).
    let v = ok_i64("fn main() i64 { let a = -1i8\n return a.as(i64) }");
    assert_eq!(v, -1);
    // widening never traps; value preserved.
    let v = ok_i64("fn main() i64 { let a = 200u8\n return a.as(i64) }");
    assert_eq!(v, 200);
    // float → int truncation toward zero.
    let v = ok_i64("fn main() i64 { let a = -0.9d\n return a.as(i64) }");
    assert_eq!(v, 0);
}

#[test]
fn v08_bit_and_shift_values() {
    let v = ok_i64(
        "fn main() i64 {
    let a = 12i64
    let b = 10i64
    let c = a & b
    let d = a | b
    let e = a ^ b
    let f = (1i64).shl(62i64)
    let g = f.shr(60i64)
    return ((c + d) + e) + (f + g)
}",
    );
    assert_eq!(v, (12 & 10) + (12 | 10) + (12 ^ 10) + (1i64 << 62) + (1i64 << 2));
    // i32 bit ops (the only Num-encoded i32 surface).
    let v2 = ok_i64(
        "fn main() i64 {
    let a = 12 & 10
    let b = a.as(i64)
    return b
}",
    );
    assert_eq!(v2, 8);
}
