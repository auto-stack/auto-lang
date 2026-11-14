//! auto-ffi provider f0_rich/1 target=x86_64-pc-windows-msvc profile=f0-rich.v1
//!
//! PLAN-967 T-11: the frozen RICH Rust provider — mixed view+scalar
//! parameters beyond the F0 slice. The annotations ARE the metadata.

/// Clamp the view's byte length into [lo, hi] (view + two i32 scalars).
// auto-ffi fn rich_clamp symbol=f0.rich.clamp.v1 form=pure ret=i32
pub fn rich_clamp(s: &str, lo: i32, hi: i32) -> i32 {
    let n = s.as_bytes().len() as i32;
    if n < lo { lo } else if n > hi { hi } else { n }
}

/// Is the view longer than n code points? (view + one i32 scalar -> bool)
// auto-ffi fn rich_is_longer symbol=f0.rich.is_longer.v1 form=pure ret=bool
pub fn rich_is_longer(s: &str, n: i32) -> bool {
    s.chars().count() > n as usize
}

/// First byte equal to `b` (1-based index); None -> not-found.
// auto-ffi fn rich_find symbol=f0.rich.find.v1 form=fallible errors=not-found:-1
pub fn rich_find(s: &str, b: i32) -> Option<i32> {
    s.as_bytes()
        .iter()
        .position(|x| *x as i32 == b)
        .map(|i| i as i32 + 1)
}

/// A CONCRETE generic instance (the importable shape — the generic fn
/// itself stays outside; only monomorphic instances cross).
// auto-ffi fn rich_first_i32 symbol=f0.rich.first_i32.v1 form=pure ret=i32
pub fn rich_first_i32(s: &str) -> i32 {
    s.as_bytes().first().map(|b| *b as i32).unwrap_or(-1)
}
