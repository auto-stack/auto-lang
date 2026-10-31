// AUTO-GENERATED typed resource runtime (PLAN-902 AX-B) — DO NOT EDIT.
// Checked i32 core + owned-Str resource layer with explicit cleanup,
// pre-allocation budget checks and default-off fault/observation hooks.
#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoTrap {
    Overflow,
    DivByZero,
    DivOverflow,
    RemByZero,
    RemOverflow,
    InvalidScalar,
    TextIndex,
    Resource,
}

impl AutoTrap {
    pub fn name(self) -> &'static str {
        match self {
            AutoTrap::Overflow => "Overflow",
            AutoTrap::DivByZero => "DivByZero",
            AutoTrap::DivOverflow => "DivOverflow",
            AutoTrap::RemByZero => "RemByZero",
            AutoTrap::RemOverflow => "RemOverflow",
            AutoTrap::InvalidScalar => "InvalidScalar",
            AutoTrap::TextIndex => "TextIndex",
            AutoTrap::Resource => "Resource",
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

// ---- PLAN-902 owned-Str resource layer --------------------------------
//
// A2xStr owns exactly one buffer. Cleanup is EXPLICIT (a2x_str_free at the
// verifier-scheduled points); the type deliberately has NO Drop, so a
// missed free is an observable leak in the event ledger and a double free
// is impossible. Allocation budgets are checked BEFORE every allocation;
// host failure is the typed Resource trap, never a panic.

use std::sync::atomic::{AtomicI64, Ordering};

static RT_ALLOC_COUNT: AtomicI64 = AtomicI64::new(0);
static RT_FREE_COUNT: AtomicI64 = AtomicI64::new(0);

struct RtConfig {
    /// 1-based ordinal of the allocation to fail (<=0: off).
    fault_alloc: i64,
    /// Mirror alloc/free events to stderr (`rt:alloc:<bytes>`).
    events: bool,
}

fn rt_config() -> &'static RtConfig {
    use std::sync::OnceLock;
    static CFG: OnceLock<RtConfig> = OnceLock::new();
    CFG.get_or_init(|| RtConfig {
        fault_alloc: std::env::var("A2X_RT_FAULT_ALLOC")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0),
        events: std::env::var("A2X_RT_EVENTS").ok().as_deref() == Some("1"),
    })
}

fn rt_alloc(bytes: usize) -> A2xResult<()> {
    let n = RT_ALLOC_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
    let cfg = rt_config();
    // a faulted attempt is NOT an allocation: no buffer exists, no event
    if cfg.fault_alloc == n {
        return Err(AutoTrap::Resource);
    }
    if cfg.events {
        eprintln!("rt:alloc:{}", bytes);
    }
    Ok(())
}

fn rt_free(bytes: usize) {
    let _ = RT_FREE_COUNT.fetch_add(1, Ordering::SeqCst);
    if rt_config().events {
        eprintln!("rt:free:{}", bytes);
    }
}

/// Allocation counters (test observation; the language never reads them).
pub fn a2x_rt_alloc_count() -> i64 { RT_ALLOC_COUNT.load(Ordering::SeqCst) }
pub fn a2x_rt_free_count() -> i64 { RT_FREE_COUNT.load(Ordering::SeqCst) }

pub struct A2xStr {
    buf: Option<String>,
}

impl A2xStr {
    pub fn as_str(&self) -> &str {
        self.buf.as_deref().unwrap_or("")
    }
}

fn alloc_buffer(cap: usize, fill: &str) -> A2xResult<A2xStr> {
    // budget BEFORE allocation: capacity is checked arithmetic upstream;
    // the reservation itself must not abort on failure
    let mut b = String::new();
    if b.try_reserve(cap).is_err() {
        return Err(AutoTrap::Resource);
    }
    rt_alloc(cap)?;
    b.push_str(fill);
    Ok(A2xStr { buf: Some(b) })
}

/// Literal promotion (AP-4): an owned copy of a static literal.
pub fn a2x_str_from_lit(s: &str) -> A2xResult<A2xStr> {
    alloc_buffer(s.len(), s)
}

/// Independent owned copy (AP-2, `s.clone()`).
pub fn a2x_str_clone(s: &str) -> A2xResult<A2xStr> {
    alloc_buffer(s.len(), s)
}

/// Promotion of a borrowed view (`s.to(Str)`).
pub fn a2x_str_to_owned(s: &str) -> A2xResult<A2xStr> {
    alloc_buffer(s.len(), s)
}

/// Concatenation into a fresh owned buffer (AP-3).
pub fn a2x_str_concat(l: &str, r: &str) -> A2xResult<A2xStr> {
    let cap = l.len().checked_add(r.len()).ok_or(AutoTrap::Resource)?;
    let mut b = String::new();
    if b.try_reserve(cap).is_err() {
        return Err(AutoTrap::Resource);
    }
    rt_alloc(cap)?;
    b.push_str(l);
    b.push_str(r);
    Ok(A2xStr { buf: Some(b) })
}

/// Explicit free of an owned buffer — the ONLY release path. Returns the
/// freed byte count to the (test) observer protocol.
pub fn a2x_str_free(s: &mut A2xStr) {
    if let Some(b) = s.buf.take() {
        rt_free(b.len());
    }
}

/// Unicode scalar count.
pub fn a2x_str_len(s: &str) -> A2xResult<i32> {
    i32::try_from(s.chars().count()).map_err(|_| AutoTrap::Resource)
}

/// UTF-8 byte length.
pub fn a2x_str_len_bytes(s: &str) -> A2xResult<i32> {
    i32::try_from(s.len()).map_err(|_| AutoTrap::Resource)
}

/// Full-byte equality (no normalization; NUL included).
pub fn a2x_str_eq(l: &str, r: &str) -> bool { l == r }
pub fn a2x_str_ne(l: &str, r: &str) -> bool { l != r }

/// Scalar-indexed char access; out-of-bounds (incl. negative) is the
/// typed TextIndex trap, never a byte-level panic.
pub fn a2x_str_char_at(s: &str, i: i32) -> A2xResult<char> {
    if i < 0 {
        return Err(AutoTrap::TextIndex);
    }
    s.chars().nth(i as usize).ok_or(AutoTrap::TextIndex)
}

/// Scalar-indexed `[start, end)` slice — a view of the operand (no
/// allocation). Bounds are scalar positions; invalid ranges are TextIndex.
pub fn a2x_str_slice<'b>(s: &'b str, start: i32, end: i32) -> A2xResult<&'b str> {
    if start < 0 || end < start {
        return Err(AutoTrap::TextIndex);
    }
    let start = start as usize;
    let end = end as usize;
    let mut byte_start = None;
    let mut byte_end = None;
    for (i, (b, _)) in s.char_indices().enumerate() {
        if i == start {
            byte_start = Some(b);
        }
        if i == end {
            byte_end = Some(b);
        }
    }
    let byte_start = byte_start.ok_or(AutoTrap::TextIndex)?;
    let byte_end = match byte_end {
        Some(b) => b,
        // end == scalar count: slice to the end of the string
        None if start <= s.chars().count() && end == s.chars().count() => s.len(),
        None => return Err(AutoTrap::TextIndex),
    };
    s.get(byte_start..byte_end).ok_or(AutoTrap::TextIndex)
}

/// char code point -> i32 (pure widening).
pub fn a2x_char_to_codepoint(c: char) -> i32 { c as i32 }

/// i32 -> char; invalid scalars are the typed InvalidScalar trap.
pub fn a2x_char_from_codepoint(n: i32) -> A2xResult<char> {
    char::from_u32(n as u32).ok_or(AutoTrap::InvalidScalar)
}
