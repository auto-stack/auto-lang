// AUTO-GENERATED minimal typed runtime (PLAN-892 AX-A) — DO NOT EDIT.
// Checked i32 primitives with named traps + authorized trace marks.
#![allow(dead_code)]

/// Named Auto trap reasons (plan §5.1): add/sub/mul/neg overflow,
/// division/remainder by zero, MIN/-1 division and MIN%-1 remainder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoTrap { Overflow, DivByZero, DivOverflow, RemByZero, RemOverflow, InvalidScalar }

impl AutoTrap {
    pub fn name(self) -> &'static str {
        match self {
            AutoTrap::Overflow => "Overflow",
            AutoTrap::DivByZero => "DivByZero",
            AutoTrap::DivOverflow => "DivOverflow",
            AutoTrap::RemByZero => "RemByZero",
            AutoTrap::RemOverflow => "RemOverflow",

                     AutoTrap::InvalidScalar => "InvalidScalar",
        }
    }
}

pub type A2xResult<T> = Result<T, AutoTrap>;

pub fn a2x_add(a: i32, b: i32) -> A2xResult<i32> { a.checked_add(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_sub(a: i32, b: i32) -> A2xResult<i32> { a.checked_sub(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_mul(a: i32, b: i32) -> A2xResult<i32> { a.checked_mul(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_neg(a: i32) -> A2xResult<i32> { a.checked_neg().ok_or(AutoTrap::Overflow) }
pub fn a2x_div(a: i32, b: i32) -> A2xResult<i32> {
    if b == 0 { Err(AutoTrap::DivByZero) } else { a.checked_div(b).ok_or(AutoTrap::DivOverflow) }
}
pub fn a2x_rem(a: i32, b: i32) -> A2xResult<i32> {
    if b == 0 { Err(AutoTrap::RemByZero) } else { a.checked_rem(b).ok_or(AutoTrap::RemOverflow) }
}
// ---- PLAN-961 numeric runtime (frozen T-01 §2/§3 matrix) ----------------
// Trap mapping: width overflow -> Overflow; /0, MIN/-1 -> the frozen div/rem
// names; shift-out-of-range and float->int domain failures -> InvalidScalar;
// f64->f32 finite overflow -> Overflow (closest frozen class; the transport
// layer keeps distinct text codes via the provider's trap table).

pub fn a2x_add_i8(a: i8, b: i8) -> A2xResult<i8> { a.checked_add(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_sub_i8(a: i8, b: i8) -> A2xResult<i8> { a.checked_sub(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_mul_i8(a: i8, b: i8) -> A2xResult<i8> { a.checked_mul(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_neg_i8(a: i8) -> A2xResult<i8> { a.checked_neg().ok_or(AutoTrap::Overflow) }
pub fn a2x_add_u8(a: u8, b: u8) -> A2xResult<u8> { a.checked_add(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_sub_u8(a: u8, b: u8) -> A2xResult<u8> { a.checked_sub(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_mul_u8(a: u8, b: u8) -> A2xResult<u8> { a.checked_mul(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_add_u32(a: u32, b: u32) -> A2xResult<u32> { a.checked_add(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_sub_u32(a: u32, b: u32) -> A2xResult<u32> { a.checked_sub(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_mul_u32(a: u32, b: u32) -> A2xResult<u32> { a.checked_mul(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_add_i64(a: i64, b: i64) -> A2xResult<i64> { a.checked_add(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_sub_i64(a: i64, b: i64) -> A2xResult<i64> { a.checked_sub(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_mul_i64(a: i64, b: i64) -> A2xResult<i64> { a.checked_mul(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_neg_i64(a: i64) -> A2xResult<i64> { a.checked_neg().ok_or(AutoTrap::Overflow) }
pub fn a2x_add_u64(a: u64, b: u64) -> A2xResult<u64> { a.checked_add(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_sub_u64(a: u64, b: u64) -> A2xResult<u64> { a.checked_sub(b).ok_or(AutoTrap::Overflow) }
pub fn a2x_mul_u64(a: u64, b: u64) -> A2xResult<u64> { a.checked_mul(b).ok_or(AutoTrap::Overflow) }

pub fn a2x_div_i8(a: i8, b: i8) -> A2xResult<i8> {
    if b == 0 { return Err(AutoTrap::DivByZero); }
    if a == i8::MIN && b == -1 { return Err(AutoTrap::DivOverflow); }
    Ok(a.wrapping_div(b))
}
pub fn a2x_rem_i8(a: i8, b: i8) -> A2xResult<i8> {
    if b == 0 { return Err(AutoTrap::DivByZero); }
    if a == i8::MIN && b == -1 { return Err(AutoTrap::RemOverflow); }
    Ok(a.wrapping_rem(b))
}
pub fn a2x_div_i64(a: i64, b: i64) -> A2xResult<i64> {
    if b == 0 { return Err(AutoTrap::DivByZero); }
    if a == i64::MIN && b == -1 { return Err(AutoTrap::DivOverflow); }
    Ok(a.wrapping_div(b))
}
pub fn a2x_rem_i64(a: i64, b: i64) -> A2xResult<i64> {
    if b == 0 { return Err(AutoTrap::DivByZero); }
    if a == i64::MIN && b == -1 { return Err(AutoTrap::RemOverflow); }
    Ok(a.wrapping_rem(b))
}
pub fn a2x_div_u8(a: u8, b: u8) -> A2xResult<u8> { if b == 0 { Err(AutoTrap::DivByZero) } else { Ok(a / b) } }
pub fn a2x_rem_u8(a: u8, b: u8) -> A2xResult<u8> { if b == 0 { Err(AutoTrap::DivByZero) } else { Ok(a % b) } }
pub fn a2x_div_u32(a: u32, b: u32) -> A2xResult<u32> { if b == 0 { Err(AutoTrap::DivByZero) } else { Ok(a / b) } }
pub fn a2x_rem_u32(a: u32, b: u32) -> A2xResult<u32> { if b == 0 { Err(AutoTrap::DivByZero) } else { Ok(a % b) } }
pub fn a2x_div_u64(a: u64, b: u64) -> A2xResult<u64> { if b == 0 { Err(AutoTrap::DivByZero) } else { Ok(a / b) } }
pub fn a2x_rem_u64(a: u64, b: u64) -> A2xResult<u64> { if b == 0 { Err(AutoTrap::DivByZero) } else { Ok(a % b) } }

// Frozen: float divisor == ±0.0 traps (legacy parity); rem is fmod (x % ±0.0
// = NaN, no trap); negation is a sign flip (NaN sign flips).
pub fn a2x_div_f32(a: f32, b: f32) -> A2xResult<f32> {
    if b == 0.0 { return Err(AutoTrap::DivByZero); }
    Ok(a / b)
}
pub fn a2x_div_f64(a: f64, b: f64) -> A2xResult<f64> {
    if b == 0.0 { return Err(AutoTrap::DivByZero); }
    Ok(a / b)
}
pub fn a2x_rem_f32(a: f32, b: f32) -> A2xResult<f32> { Ok(a % b) }
pub fn a2x_rem_f64(a: f64, b: f64) -> A2xResult<f64> { Ok(a % b) }
pub fn a2x_neg_f32(a: f32) -> A2xResult<f32> { Ok(-a) }
pub fn a2x_neg_f64(a: f64) -> A2xResult<f64> { Ok(-a) }

pub fn a2x_shl_i8(a: i8, n: i8) -> A2xResult<i8> {
    if n < 0 || n >= 8 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shl(n as u32))
}
pub fn a2x_shr_i8(a: i8, n: i8) -> A2xResult<i8> {
    if n < 0 || n >= 8 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shr(n as u32))
}
pub fn a2x_shl_u8(a: u8, n: u8) -> A2xResult<u8> {
    if n >= 8 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shl(n as u32))
}
pub fn a2x_shr_u8(a: u8, n: u8) -> A2xResult<u8> {
    if n >= 8 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shr(n as u32))
}
pub fn a2x_shl_i32(a: i32, n: i32) -> A2xResult<i32> {
    if n < 0 || n >= 32 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shl(n as u32))
}
pub fn a2x_shr_i32(a: i32, n: i32) -> A2xResult<i32> {
    if n < 0 || n >= 32 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shr(n as u32))
}
pub fn a2x_shl_u32(a: u32, n: u32) -> A2xResult<u32> {
    if n >= 32 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shl(n as u32))
}
pub fn a2x_shr_u32(a: u32, n: u32) -> A2xResult<u32> {
    if n >= 32 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shr(n as u32))
}
pub fn a2x_shl_i64(a: i64, n: i64) -> A2xResult<i64> {
    if n < 0 || n >= 64 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shl(n as u32))
}
pub fn a2x_shr_i64(a: i64, n: i64) -> A2xResult<i64> {
    if n < 0 || n >= 64 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shr(n as u32))
}
pub fn a2x_shl_u64(a: u64, n: u64) -> A2xResult<u64> {
    if n >= 64 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shl(n as u32))
}
pub fn a2x_shr_u64(a: u64, n: u64) -> A2xResult<u64> {
    if n >= 64 { return Err(AutoTrap::InvalidScalar); }
    Ok(a.wrapping_shr(n as u32))
}

// ---- conversion matrix (frozen T-01 §3) ----
pub fn a2x_c_i8_i32(a: i8) -> A2xResult<i32> { Ok(a as i32) }
pub fn a2x_c_i8_i64(a: i8) -> A2xResult<i64> { Ok(a as i64) }
pub fn a2x_c_i32_i8(a: i32) -> A2xResult<i8> { i8::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i32_i64(a: i32) -> A2xResult<i64> { Ok(a as i64) }
pub fn a2x_c_i32_u8(a: i32) -> A2xResult<u8> { u8::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i32_u32(a: i32) -> A2xResult<u32> { u32::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i32_u64(a: i32) -> A2xResult<u64> { u64::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i64_i8(a: i64) -> A2xResult<i8> { i8::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i64_i32(a: i64) -> A2xResult<i32> { i32::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i64_u8(a: i64) -> A2xResult<u8> { u8::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i64_u32(a: i64) -> A2xResult<u32> { u32::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i64_u64(a: i64) -> A2xResult<u64> { u64::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_u8_i32(a: u8) -> A2xResult<i32> { Ok(a as i32) }
pub fn a2x_c_u8_i64(a: u8) -> A2xResult<i64> { Ok(a as i64) }
pub fn a2x_c_u32_i32(a: u32) -> A2xResult<i32> { i32::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_u32_i64(a: u32) -> A2xResult<i64> { Ok(a as i64) }
pub fn a2x_c_u32_u8(a: u32) -> A2xResult<u8> { u8::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_u64_i64(a: u64) -> A2xResult<i64> { i64::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_u64_u32(a: u64) -> A2xResult<u32> { u32::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_u64_u8(a: u64) -> A2xResult<u8> { u8::try_from(a).map_err(|_| AutoTrap::Overflow) }
pub fn a2x_c_i8_f32(a: i8) -> A2xResult<f32> { Ok(a as f32) }
pub fn a2x_c_i8_f64(a: i8) -> A2xResult<f64> { Ok(a as f64) }
pub fn a2x_c_i32_f32(a: i32) -> A2xResult<f32> { Ok(a as f32) }
pub fn a2x_c_i32_f64(a: i32) -> A2xResult<f64> { Ok(a as f64) }
pub fn a2x_c_i64_f32(a: i64) -> A2xResult<f32> { Ok(a as f32) }
pub fn a2x_c_i64_f64(a: i64) -> A2xResult<f64> { Ok(a as f64) }
pub fn a2x_c_u8_f32(a: u8) -> A2xResult<f32> { Ok(a as f32) }
pub fn a2x_c_u8_f64(a: u8) -> A2xResult<f64> { Ok(a as f64) }
pub fn a2x_c_u32_f32(a: u32) -> A2xResult<f32> { Ok(a as f32) }
pub fn a2x_c_u32_f64(a: u32) -> A2xResult<f64> { Ok(a as f64) }
pub fn a2x_c_u64_f32(a: u64) -> A2xResult<f32> { Ok(a as f32) }
pub fn a2x_c_u64_f64(a: u64) -> A2xResult<f64> { Ok(a as f64) }
pub fn a2x_c_f32_i8(a: f32) -> A2xResult<i8> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < i8::MIN as f32 || t > i8::MAX as f32 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as i8)
}
pub fn a2x_c_f32_i32(a: f32) -> A2xResult<i32> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < i32::MIN as f32 || t > i32::MAX as f32 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as i32)
}
pub fn a2x_c_f32_i64(a: f32) -> A2xResult<i64> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < i64::MIN as f32 || t > i64::MAX as f32 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as i64)
}
pub fn a2x_c_f32_u8(a: f32) -> A2xResult<u8> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < 0.0 || t > u8::MAX as f32 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as u8)
}
pub fn a2x_c_f32_u32(a: f32) -> A2xResult<u32> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < 0.0 || t > u32::MAX as f32 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as u32)
}
pub fn a2x_c_f64_i8(a: f64) -> A2xResult<i8> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < i8::MIN as f64 || t > i8::MAX as f64 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as i8)
}
pub fn a2x_c_f64_i32(a: f64) -> A2xResult<i32> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < i32::MIN as f64 || t > i32::MAX as f64 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as i32)
}
pub fn a2x_c_f64_i64(a: f64) -> A2xResult<i64> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < i64::MIN as f64 || t > i64::MAX as f64 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as i64)
}
pub fn a2x_c_f64_u8(a: f64) -> A2xResult<u8> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < 0.0 || t > u8::MAX as f64 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as u8)
}
pub fn a2x_c_f64_u32(a: f64) -> A2xResult<u32> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < 0.0 || t > u32::MAX as f64 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as u32)
}
pub fn a2x_c_f64_u64(a: f64) -> A2xResult<u64> {
    if a.is_nan() || a.is_infinite() { return Err(AutoTrap::InvalidScalar); }
    let t = a.trunc();
    if t < 0.0 || t >= 18446744073709551616.0 { return Err(AutoTrap::InvalidScalar); }
    Ok(t as u64)
}
pub fn a2x_c_f32_f64(a: f32) -> A2xResult<f64> { Ok(a as f64) }
// f64→f32: round-to-nearest; a FINITE value that overflows to ±Inf traps.
pub fn a2x_c_f64_f32(a: f64) -> A2xResult<f32> {
    let r = a as f32;
    if r.is_infinite() && a.is_finite() { return Err(AutoTrap::Overflow); }
    Ok(r)
}


pub fn a2x_add_f32(a: f32, b: f32) -> A2xResult<f32> { Ok(a + b) }
pub fn a2x_sub_f32(a: f32, b: f32) -> A2xResult<f32> { Ok(a - b) }
pub fn a2x_mul_f32(a: f32, b: f32) -> A2xResult<f32> { Ok(a * b) }
pub fn a2x_add_f64(a: f64, b: f64) -> A2xResult<f64> { Ok(a + b) }
pub fn a2x_sub_f64(a: f64, b: f64) -> A2xResult<f64> { Ok(a - b) }
pub fn a2x_mul_f64(a: f64, b: f64) -> A2xResult<f64> { Ok(a * b) }
macro_rules! bitop {
    ($a:ident, $o:ident, $x:ident, $t:ty) => {
        pub fn $a(l: $t, r: $t) -> A2xResult<$t> { Ok(l & r) }
        pub fn $o(l: $t, r: $t) -> A2xResult<$t> { Ok(l | r) }
        pub fn $x(l: $t, r: $t) -> A2xResult<$t> { Ok(l ^ r) }
    };
}
bitop!(a2x_and_i8, a2x_or_i8, a2x_xor_i8, i8);
bitop!(a2x_and_u8, a2x_or_u8, a2x_xor_u8, u8);
bitop!(a2x_and_i32, a2x_or_i32, a2x_xor_i32, i32);
bitop!(a2x_and_u32, a2x_or_u32, a2x_xor_u32, u32);
bitop!(a2x_and_i64, a2x_or_i64, a2x_xor_i64, i64);
bitop!(a2x_and_u64, a2x_or_u64, a2x_xor_u64, u64);
pub fn a2x_lit_i8(v: i32) -> A2xResult<i8> { Ok(v as i8) }
pub fn a2x_lit_u8(v: i32) -> A2xResult<u8> { Ok(v as u8) }
pub fn a2x_lit_u32(v: i32) -> A2xResult<u32> { Ok(v as u32) }
pub fn a2x_lit_i64(lo: i32, hi: i32) -> A2xResult<i64> {
    Ok(((hi as i64) << 32) | (lo as u32 as i64))
}
pub fn a2x_lit_u64(lo: i32, hi: i32) -> A2xResult<u64> {
    Ok(((hi as u32 as u64) << 32) | (lo as u32 as u64))
}
pub fn a2x_lit_f32(bits: i32) -> A2xResult<f32> { Ok(f32::from_bits(bits as u32)) }
pub fn a2x_lit_f64(lo: i32, hi: i32) -> A2xResult<f64> {
    Ok(f64::from_bits(((hi as u32 as u64) << 32) | (lo as u32 as u64)))
}

