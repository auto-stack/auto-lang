//! PLAN-961 T-02c numeric codegen tier: the typed numeric surface through
//! the REAL mn pipeline (numeric source profile → closures-family lowering
//! → Numeric CLIF emission → COFF → PE). Every case's expectation is
//! INDEPENDENT (hand-computed values / trap exit 70 family); AV parity is
//! covered separately in the av tier.

#[path = "../common/mod.rs"]
mod common;

/// Full numeric pipeline via the mn driver (escalation lands on the
/// numeric profile for numeric sources).
fn run_num(src: &str, tag: &str) -> ace_native::driver::MnRequest {
    if !common::tools_available() {
        panic!("infra-blocked: rust-lld/SDK unavailable");
    }
    common::run_mn(src, &[], tag)
}

fn expect_exit(src: &str, want: i32, tag: &str) {
    let r = run_num(src, tag);
    assert!(r.ok, "{:?}: {:?}", tag, r.error);
    assert_eq!(r.exit_code, Some(want), "case {tag}");
}

#[test]
#[cfg(windows)]
fn mn_numeric_i64_arith_and_compare() {
    // V-01 magnitude: 2147483648i64 + 1 → low-32 exit = 1... use mod-2^32
    // observable: build comparisons that yield small ints.
    expect_exit(
        "fn main() int {
    let a = 2147483648i64
    let b = a + 1i64
    if b > 2147483648i64 { return 1 }
    return 0
}",
        1,
        "i64-mag",
    );
    // i64 arithmetic inside i32 range round-trips exactly.
    expect_exit(
        "fn main() int {
    let a = 2000000000i64
    let b = a + 12i64
    let c = b.as(i32)
    return c
}",
        2000000012,
        "i64-sum-obs",
    );
    // checked narrowing: 4000000000i64 -> i32 traps (frozen matrix).
    expect_exit(
        "fn main() int {
    let a = 2000000000i64
    let b = a + a
    let c = b.as(i32)
    let _ = c
    return 0
}",
        70,
        "i64-narrow-trap",
    );
    // MAX+1 traps.
    expect_exit(
        "fn main() int {
    let a = 9223372036854775807i64
    let b = a + 1i64
    let _ = b
    return 0
}",
        70,
        "i64-max-plus-1",
    );
    // i64::MIN / -1 traps; MIN % -1 traps.
    expect_exit(
        "fn main() int {
    let mn = -9223372036854775808i64
    let m1 = -1i64
    return (mn / m1).as(i32)
}",
        70,
        "i64-min-div",
    );
    expect_exit(
        "fn main() int {
    let mn = -9223372036854775808i64
    let m1 = -1i64
    return (mn % m1).as(i32)
}",
        70,
        "i64-min-rem",
    );
    // div by zero traps.
    expect_exit(
        "fn main() int {
    let z = 0i64
    return (5i64 / z).as(i32)
}",
        70,
        "i64-div-zero",
    );
}

#[test]
#[cfg(windows)]
fn mn_numeric_narrow_widths() {
    // u8 arithmetic at the width boundary: 200 + 55 = 255 (fits).
    expect_exit(
        "fn main() int {
    let a = 200u8
    let b = 55u8
    let c = a + b
    return c.as(i32)
}",
        255,
        "u8-sum-255",
    );
    // 200 + 100 = 300 > u8::MAX → trap.
    expect_exit(
        "fn main() int {
    let a = 200u8
    let b = 100u8
    let c = a + b
    let _ = c
    return 0
}",
        70,
        "u8-sum-300",
    );
    // u8 wrapping-free mul: 16 * 16 = 256 > 255 → trap.
    expect_exit(
        "fn main() int {
    let a = 16u8
    let b = a * a
    let _ = b
    return 0
}",
        70,
        "u8-mul-256",
    );
    // i8 MIN negation traps.
    expect_exit(
        "fn main() int {
    let m = -128i8
    let n = -m
    let _ = n
    return 0
}",
        70,
        "i8-neg-min",
    );
    // u32 compare unsigned: 4294967295 > 1.
    expect_exit(
        "fn main() int {
    let hi = 4294967295u32
    let lo = 1u32
    if hi > lo { return 1 }
    return 0
}",
        1,
        "u32-unsigned-cmp",
    );
}

#[test]
#[cfg(windows)]
fn mn_numeric_shift_bit_edges() {
    // shift amount == width traps.
    expect_exit(
        "fn main() int {
    let x = 1i64
    let r = x.shl(64i64)
    let _ = r
    return 0
}",
        70,
        "shl-width-trap",
    );
    // negative amount traps.
    expect_exit(
        "fn main() int {
    let x = 1i64
    let n = -1i64
    let r = x.shl(n)
    let _ = r
    return 0
}",
        70,
        "shl-neg-trap",
    );
    // value bits discarded: (1i64 << 62) >> 60 = 4.
    expect_exit(
        "fn main() int {
    let x = 1i64
    let f = x.shl(62i64)
    let g = f.shr(60i64)
    return g.as(i32)
}",
        4,
        "shl-shr-value",
    );
    // i32 bit ops: 12 & 10 = 8.
    expect_exit(
        "fn main() int {
    let a = 12
    let b = 10
    return a & b
}",
        8,
        "i32-and",
    );
    // i8 sign-extended shift: -1i8 >> 1 = -1 arithmetic.
    expect_exit(
        "fn main() int {
    let m = -1i8
    let r = m.shr(1i8)
    return r.as(i32)
}",
        -1,
        "i8-sshr",
    );
}

#[test]
#[cfg(windows)]
fn mn_numeric_float_and_conv_edges() {
    // float div by zero traps (legacy parity).
    expect_exit(
        "fn main() int {
    let z = 0.0d
    let v = 1.5d / z
    let _ = v
    return 0
}",
        70,
        "f64-div-zero",
    );
    // f64→i64 out of range traps.
    expect_exit(
        "fn main() int {
    let a = 9223372036854775807i64
    let f = a.as(f64)
    let b = f * f
    let c = b * f
    return c.as(i64).as(i32)
}",
        70,
        "f64-to-i64-range",
    );
    // checked narrowing: 300i64 → u8 traps.
    expect_exit(
        "fn main() int {
    let a = 300i64
    return a.as(u8).as(i32)
}",
        70,
        "narrow-u8-trap",
    );
    // sign extension: -1i8 → i64 == -1.
    expect_exit(
        "fn main() int {
    let a = -1i8
    let b = a.as(i64)
    return b.as(i32)
}",
        -1,
        "sext-i8-i64",
    );
    // truncation toward zero: -0.9 → 0.
    expect_exit(
        "fn main() int {
    let a = -0.9d
    return a.as(i64).as(i32)
}",
        0,
        "f64-trunc-zero",
    );
    // int→float exact and back: 2^30 stays in i32 range.
    expect_exit(
        "fn main() int {
    let a = 1073741824i64
    let f = a.as(f64)
    let back = f.as(i64)
    return back.as(i32)
}",
        1073741824,
        "i64-f64-i64",
    );
    // 2^31 through f64 narrows to i32 → trap (checked matrix).
    expect_exit(
        "fn main() int {
    let a = 2147483648i64
    let f = a.as(f64)
    let back = f.as(i64)
    return back.as(i32)
}",
        70,
        "i64-f64-i64-narrow-trap",
    );
}
