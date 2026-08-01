//! Plan 221: NaN-boxed value representation for AutoVM
//!
//! Packs type tag + payload into a single u64 using IEEE 754 NaN bit patterns.
//! Normal f64 values are stored directly (zero overhead).
//! All other types use the NaN-boxed encoding with a 4-bit tag.

/// A NaN-boxed value — 64 bits that can hold any Auto type.
pub type NanoValue = u64;

// Tag constants (placed at bits 51-48 within NaN-boxed values)
#[allow(dead_code)]
const TAG_F64:    u64 = 0x0000_0000_0000_0000;
const TAG_I32:    u64 = 0x0001_0000_0000_0000;
const TAG_STRING: u64 = 0x0002_0000_0000_0000;
const TAG_BOOL:   u64 = 0x0003_0000_0000_0000;
const TAG_NULL:   u64 = 0x0004_0000_0000_0000;
const TAG_OBJECT: u64 = 0x0005_0000_0000_0000;
const TAG_LIST:   u64 = 0x0006_0000_0000_0000;
const TAG_F32:    u64 = 0x0007_0000_0000_0000;
// Plan 377: 单槽 i64/u64/BigInt 编码（48 位 payload + 堆兜底）
const TAG_I64:    u64 = 0x0008_0000_0000_0000;
const TAG_U64:    u64 = 0x0009_0000_0000_0000;
const TAG_BIGINT: u64 = 0x000A_0000_0000_0000;

// NaN-box base: sign=1, exponent=0x7FF (all 1s), tag=0, payload=0
const NANBOX_BASE: u64 = 0xFFF0_0000_0000_0000;

const TAG_SHIFT: u64 = 48;
const TAG_MASK: u64 = 0xF;
const PAYLOAD_MASK: u64 = 0xFFFF_FFFF;
/// Plan 377: i64/u64 单槽编码用的 48 位 payload 掩码（bit 47-0）。
/// 既有 i32/string/bool/object/list/f32 的 encode 只把数据放低 32 位
/// （高 16 位为 0），故用这个掩码 decode 不影响它们。
pub const PAYLOAD48_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

// ---- Detection ----

#[inline(always)]
pub fn is_nanboxed(v: NanoValue) -> bool {
    (v >> 52) == 0xFFF
}

// ---- Encode ----

#[inline(always)]
pub fn encode_f64(f: f64) -> NanoValue { f.to_bits() }

#[inline(always)]
pub fn encode_i32(i: i32) -> NanoValue { NANBOX_BASE | TAG_I32 | ((i as u32) as u64) }

// String payload stores the NEGATIVE i32 tag (-(idx+1)) so that
// decode_i32() on a string NanoValue returns the same negative value.
// This allows pop_i32/push_i32 round-trips to preserve string identity
// for code paths that move values through ListData<i32> or other
// i32-only containers.
#[inline(always)]
pub fn encode_string(idx: u32) -> NanoValue {
    let neg_tag = (-(idx as i32) - 1i32) as u32;
    NANBOX_BASE | TAG_STRING | (neg_tag as u64)
}

// Bool payload uses sentinel values:
// true  = i32::MIN      = -2147483648
// false = i32::MIN + 1  = -2147483647
#[inline(always)]
pub fn encode_bool(b: bool) -> NanoValue {
    let sentinel = if b { i32::MIN } else { i32::MIN + 1 };
    NANBOX_BASE | TAG_BOOL | (sentinel as u32 as u64)
}

// Null payload uses sentinel value: i32::MIN + 1
#[inline(always)]
pub fn encode_null() -> NanoValue { NANBOX_BASE | TAG_NULL | ((i32::MIN + 1) as u32 as u64) }

#[inline(always)]
pub fn encode_object(id: u32) -> NanoValue { NANBOX_BASE | TAG_OBJECT | (id as u64) }

#[inline(always)]
pub fn encode_list(id: u32) -> NanoValue { NANBOX_BASE | TAG_LIST | (id as u64) }

#[inline(always)]
pub fn encode_f32(f: f32) -> NanoValue { NANBOX_BASE | TAG_F32 | (f.to_bits() as u64) }

// Plan 377: i64/u64 单槽编码（48 位 payload）。值落在 48 位范围内直接编码，
// 否则返回 None（调用方堆装箱兜底）。详见 plan 377 §4.1。

/// i64 有符号 48 位范围：[-2^47, 2^47-1]
#[inline(always)]
pub fn try_encode_i64(i: i64) -> Option<NanoValue> {
    if i >= -(1i64 << 47) && i < (1i64 << 47) {
        Some(NANBOX_BASE | TAG_I64 | ((i as u64) & PAYLOAD48_MASK))
    } else {
        None
    }
}

#[inline(always)]
pub fn decode_i64(v: NanoValue) -> i64 {
    let raw = (v & PAYLOAD48_MASK) as i64;
    // 从 bit 47 符号扩展
    if raw & (1i64 << 47) != 0 { raw | (!(PAYLOAD48_MASK as i64)) } else { raw }
}

/// u64 无符号 48 位范围：[0, 2^48-1]
#[inline(always)]
pub fn try_encode_u64(u: u64) -> Option<NanoValue> {
    if u < (1u64 << 48) {
        Some(NANBOX_BASE | TAG_U64 | (u & PAYLOAD48_MASK))
    } else {
        None
    }
}

#[inline(always)]
pub fn decode_u64(v: NanoValue) -> u64 { v & PAYLOAD48_MASK }

/// BigInt（>2^48 的 i64/u64）堆对象 handle 编码
#[inline(always)]
pub fn encode_bigint(heap_id: u32) -> NanoValue { NANBOX_BASE | TAG_BIGINT | (heap_id as u64) }

#[inline(always)]
pub fn decode_bigint_handle(v: NanoValue) -> u32 { (v & PAYLOAD_MASK) as u32 }

// ---- Decode ----

#[inline(always)]
pub fn decode_f64(v: NanoValue) -> f64 { f64::from_bits(v) }

#[inline(always)]
pub fn decode_i32(v: NanoValue) -> i32 { (v & PAYLOAD_MASK) as i32 }

// Reverse of encode_string: neg_tag -> pool index
// neg_tag = -(idx+1), so idx = -neg_tag - 1
#[inline(always)]
pub fn decode_string(v: NanoValue) -> u32 {
    let neg_tag = (v & PAYLOAD_MASK) as i32;
    (-neg_tag - 1) as u32
}

#[inline(always)]
pub fn decode_bool(v: NanoValue) -> bool {
    let sentinel = (v & PAYLOAD_MASK) as i32;
    sentinel == i32::MIN  // true = i32::MIN
}

#[inline(always)]
pub fn decode_object(v: NanoValue) -> u32 { (v & PAYLOAD_MASK) as u32 }

#[inline(always)]
pub fn decode_list(v: NanoValue) -> u32 { (v & PAYLOAD_MASK) as u32 }

#[inline(always)]
pub fn decode_f32(v: NanoValue) -> f32 { f32::from_bits((v & PAYLOAD_MASK) as u32) }

// ---- Type query ----

#[inline(always)]
pub fn tag_of(v: NanoValue) -> u64 {
    if is_nanboxed(v) { (v >> TAG_SHIFT) & TAG_MASK } else { 0 }
}

#[inline(always)]
pub fn is_f64(v: NanoValue) -> bool { !is_nanboxed(v) }

#[inline(always)]
pub fn is_i32(v: NanoValue) -> bool { tag_of(v) == 1 }

#[inline(always)]
pub fn is_string(v: NanoValue) -> bool { tag_of(v) == 2 }

#[inline(always)]
pub fn is_bool(v: NanoValue) -> bool { tag_of(v) == 3 }

#[inline(always)]
pub fn is_null(v: NanoValue) -> bool { tag_of(v) == 4 }

#[inline(always)]
pub fn is_object(v: NanoValue) -> bool { tag_of(v) == 5 }

#[inline(always)]
pub fn is_list(v: NanoValue) -> bool { tag_of(v) == 6 }

#[inline(always)]
pub fn is_f32(v: NanoValue) -> bool { tag_of(v) == 7 }

// Plan 377: i64/u64/BigInt 类型查询（tag 8/9/A）
#[inline(always)]
pub fn is_i64(v: NanoValue) -> bool { tag_of(v) == 8 }

#[inline(always)]
pub fn is_u64(v: NanoValue) -> bool { tag_of(v) == 9 }

#[inline(always)]
pub fn is_bigint(v: NanoValue) -> bool { tag_of(v) == 0xA }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_f64_roundtrip() {
        let vals = [0.0, -0.0, 1.0, -1.0, 3.14, f64::MAX, f64::MIN, f64::EPSILON];
        for v in vals {
            assert_eq!(decode_f64(encode_f64(v)), v);
        }
        assert!(!is_nanboxed(encode_f64(1.0)));
        assert!(is_f64(encode_f64(1.0)));
    }

    #[test]
    fn test_i32_roundtrip() {
        let vals = [0, 1, -1, i32::MAX, i32::MIN, 42, -100];
        for v in vals {
            assert_eq!(decode_i32(encode_i32(v)), v);
        }
        assert!(is_i32(encode_i32(42)));
    }

    #[test]
    fn test_string_roundtrip() {
        for idx in [0u32, 1, 100, u32::MAX] {
            assert_eq!(decode_string(encode_string(idx)), idx);
        }
        assert!(is_string(encode_string(0)));
    }

    #[test]
    fn test_bool_roundtrip() {
        assert_eq!(decode_bool(encode_bool(true)), true);
        assert_eq!(decode_bool(encode_bool(false)), false);
        assert!(is_bool(encode_bool(true)));
    }

    #[test]
    fn test_null() {
        assert!(is_null(encode_null()));
        assert!(is_nanboxed(encode_null()));
    }

    #[test]
    fn test_object_list_roundtrip() {
        assert_eq!(decode_object(encode_object(42)), 42);
        assert_eq!(decode_list(encode_list(7)), 7);
        assert!(is_object(encode_object(0)));
        assert!(is_list(encode_list(0)));
    }

    #[test]
    fn test_f32_roundtrip() {
        let vals = [0.0f32, 1.0, -1.0, 3.14];
        for v in vals {
            assert_eq!(decode_f32(encode_f32(v)), v);
        }
        assert!(is_f32(encode_f32(1.0)));
    }

    #[test]
    fn test_no_collision_between_types() {
        let values = [
            encode_f64(1.0), encode_i32(1), encode_string(1),
            encode_bool(true), encode_null(), encode_object(1),
            encode_list(1), encode_f32(1.0),
        ];
        for i in 0..values.len() {
            for j in (i+1)..values.len() {
                assert_ne!(values[i], values[j], "Collision between types {} and {}", i, j);
            }
        }
        assert!(is_f64(values[0]));
        assert!(is_i32(values[1]));
        assert!(is_string(values[2]));
        assert!(is_bool(values[3]));
        assert!(is_null(values[4]));
        assert!(is_object(values[5]));
        assert!(is_list(values[6]));
        assert!(is_f32(values[7]));
    }

    // Plan 377: i64/u64/BigInt 单槽编码测试
    #[test]
    fn test_i64_roundtrip_48bit() {
        // 48 位有符号范围内的典型值
        let vals = [
            0i64, 1, -1, 42, -100,
            5_000_000_000i64,      // 5e9（>2^32，u64 常见值）
            -5_000_000_000i64,
            10_000_000_000i64,     // 1e10
            (1i64 << 47) - 1,      // 上界 2^47-1
            -(1i64 << 47),         // 下界 -2^47
        ];
        for v in vals {
            let nv = try_encode_i64(v).unwrap_or_else(|| panic!("i64 {} 应可编码", v));
            assert_eq!(decode_i64(nv), v, "i64 {} round-trip", v);
            assert!(is_i64(nv));
            assert!(is_nanboxed(nv));
        }
    }

    #[test]
    fn test_i64_overflow_returns_none() {
        // 超过 48 位范围 → None（调用方堆装箱）
        assert!(try_encode_i64(1i64 << 47).is_none());     // 2^47（超出上界）
        assert!(try_encode_i64(-(1i64 << 47) - 1).is_none()); // -2^47-1（超出下界）
        assert!(try_encode_i64(i64::MAX).is_none());
        assert!(try_encode_i64(i64::MIN).is_none());
    }

    #[test]
    fn test_u64_roundtrip_48bit() {
        let vals = [
            0u64, 1, 42, 100,
            5_000_000_000u64,      // 5e9
            10_000_000_000u64,     // 1e10
            (1u64 << 48) - 1,      // 上界 2^48-1
        ];
        for v in vals {
            let nv = try_encode_u64(v).unwrap_or_else(|| panic!("u64 {} 应可编码", v));
            assert_eq!(decode_u64(nv), v, "u64 {} round-trip", v);
            assert!(is_u64(nv));
            assert!(is_nanboxed(nv));
        }
    }

    #[test]
    fn test_u64_overflow_returns_none() {
        assert!(try_encode_u64(1u64 << 48).is_none());  // 2^48（超出）
        assert!(try_encode_u64(u64::MAX).is_none());
    }

    #[test]
    fn test_bigint_handle_roundtrip() {
        for id in [0u32, 1, 42, u32::MAX] {
            let nv = encode_bigint(id);
            assert_eq!(decode_bigint_handle(nv), id);
            assert!(is_bigint(nv));
            assert!(is_nanboxed(nv));
        }
    }

    #[test]
    fn test_i64_u64_no_collision() {
        // i64/u64/bigint 与既有类型不冲突
        let i64_nv = try_encode_i64(42).unwrap();
        let u64_nv = try_encode_u64(42).unwrap();
        let big_nv = encode_bigint(42);
        assert_ne!(i64_nv, u64_nv);
        assert_ne!(i64_nv, big_nv);
        assert_ne!(u64_nv, big_nv);
        assert_ne!(i64_nv, encode_i32(42));
        assert!(!is_i32(i64_nv));
        assert!(!is_object(i64_nv));
        assert!(is_i64(i64_nv) && !is_u64(i64_nv));
        assert!(is_u64(u64_nv) && !is_i64(u64_nv));
    }
}
