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

// NaN-box base: sign=1, exponent=0x7FF (all 1s), tag=0, payload=0
const NANBOX_BASE: u64 = 0xFFF0_0000_0000_0000;

const TAG_SHIFT: u64 = 48;
const TAG_MASK: u64 = 0xF;
const PAYLOAD_MASK: u64 = 0xFFFF_FFFF;

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
// decode_i32() on a string NanoValue returns the same negative value
// that the non-nanbox encoding uses. This allows pop_i32/push_i32
// round-trips to preserve string identity for code paths that
// move values through ListData<i32> or other i32-only containers.
#[inline(always)]
pub fn encode_string(idx: u32) -> NanoValue {
    let neg_tag = (-(idx as i32) - 1i32) as u32;
    NANBOX_BASE | TAG_STRING | (neg_tag as u64)
}

// Bool payload uses the same sentinel values as non-nanbox mode:
// true  = i32::MIN      = -2147483648
// false = i32::MIN + 1  = -2147483647
#[inline(always)]
pub fn encode_bool(b: bool) -> NanoValue {
    let sentinel = if b { i32::MIN } else { i32::MIN + 1 };
    NANBOX_BASE | TAG_BOOL | (sentinel as u32 as u64)
}

// Null payload uses the same sentinel as non-nanbox false: i32::MIN + 1
#[inline(always)]
pub fn encode_null() -> NanoValue { NANBOX_BASE | TAG_NULL | ((i32::MIN + 1) as u32 as u64) }

#[inline(always)]
pub fn encode_object(id: u32) -> NanoValue { NANBOX_BASE | TAG_OBJECT | (id as u64) }

#[inline(always)]
pub fn encode_list(id: u32) -> NanoValue { NANBOX_BASE | TAG_LIST | (id as u64) }

#[inline(always)]
pub fn encode_f32(f: f32) -> NanoValue { NANBOX_BASE | TAG_F32 | (f.to_bits() as u64) }

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
}
