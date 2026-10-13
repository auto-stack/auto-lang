//! PLAN-774 explicit-ABI resource runtime shim — the CALLEE side of the five
//! frozen internal ABI categories (decisions.md §3 D-05/D-06,
//! probe-cases.md §0).
//!
//! This library exists so the probe's Cranelift-generated callers can make
//! REAL machine calls across the frozen ABI inside a linked PE. It is a
//! probe-only runtime stand-in: it does not implement any Auto function, does
//! not interpret HIR, and is not a production string/I/O/runtime contract.
//!
//! no_std on purpose (the PE links against kernel32 only, no CRT); no
//! unwinding may ever cross the probed ABI (panic=abort at the workspace
//! root; the panic handler never returns). Owned buffers carry a 4-byte
//! length/magic header in front of the user bytes, so `own_free(ptr)` can
//! reclaim without a length parameter; the frozen counters account USER bytes
//! only. A free call on a non-null pointer whose header magic no longer
//! matches is a detected double free: it is counted (exposing the violation
//! to `check_invariant`) but the physical HeapFree is skipped, keeping the
//! probe deterministic.

#![no_std]
#![no_builtins]

use core::ptr::{addr_of_mut, read_volatile, write_volatile};

// ---------------------------------------------------------------------------
// kernel32 surface (the only OS dependency, same as ac-core test-support)
// ---------------------------------------------------------------------------

const STD_ERROR_HANDLE: i32 = -12;

extern "system" {
    fn GetStdHandle(n_std_handle: i32) -> isize;
    fn WriteFile(
        h_file: isize,
        lp_buffer: *const u8,
        n_number_of_bytes_to_write: u32,
        lp_number_of_bytes_written: *mut u32,
        lp_overlapped: *mut core::ffi::c_void,
    ) -> i32;
    fn GetProcessHeap() -> isize;
    fn HeapAlloc(h_heap: isize, dw_flags: u32, dw_bytes: usize) -> *mut core::ffi::c_void;
    fn HeapFree(h_heap: isize, dw_flags: u32, lp_mem: *mut core::ffi::c_void) -> i32;
}

// ---------------------------------------------------------------------------
// frozen status codes (probe-cases.md §0; keep in sync with the probe crate)
// ---------------------------------------------------------------------------

pub const ERR_LEN_OVERFLOW: i32 = -1;
pub const ERR_LEN_NEGATIVE: i32 = -2;
pub const ERR_LEN_TOO_LARGE: i32 = -3;
pub const ERR_BUDGET: i32 = -4;
pub const ERR_ENV_OOM: i32 = -5;
pub const ERR_SHIM: i32 = -6;
pub const ERR_INVALID_UTF8: i32 = -7;
pub const ERR_INVALID_SCALAR: i32 = -8;
pub const ERR_RANGE_REVERSED: i32 = -9;
pub const ERR_RANGE_OOB: i32 = -10;
pub const ERR_INDEX_OOB: i32 = -11;

const OWNED_MAGIC: u32 = 0xAC0_774;

// ---------------------------------------------------------------------------
// counters + fault injection state (single-threaded PE state)
// ---------------------------------------------------------------------------

struct Counters {
    attempts: u32,   // HeapAlloc invocations (after len/budget checks)
    allocs: u32,     // successful allocations
    frees: u32,      // free calls on non-null owned pointers
    live: u32,       // allocs - frees
    live_bytes: u32, // declared user bytes currently live
    peak: u32,       // max live_bytes at alloc/free boundaries
    budget: u32,     // 0 = unlimited
    fail_nth: u32,   // 0 = off; nth allocator invocation fails (injected)
    seq: u32,        // allocator invocation ordinal
}

static mut C: Counters = Counters {
    attempts: 0,
    allocs: 0,
    frees: 0,
    live: 0,
    live_bytes: 0,
    peak: 0,
    budget: 0,
    fail_nth: 0,
    seq: 0,
};

fn c() -> &'static mut Counters {
    unsafe { &mut *addr_of_mut!(C) }
}

fn process_heap() -> isize {
    static mut HEAP: isize = 0;
    unsafe {
        let h = &mut *addr_of_mut!(HEAP);
        if *h == 0 {
            *h = GetProcessHeap();
        }
        *h
    }
}

/// The single allocation entry behind every declared request point
/// (AP-1..AP-4). Counting: len 0 is valid and makes NO allocator call;
/// budget rejection makes NO allocator call; the injected nth failure and
/// real HeapAlloc null both count as attempts and return ERR_ENV_OOM.
fn try_alloc(len: u32) -> Result<*mut u8, i32> {
    let c = c();
    if len == 0 {
        return Ok(core::ptr::null_mut());
    }
    if c.budget != 0 && c.live_bytes.saturating_add(len) > c.budget {
        return Err(ERR_BUDGET);
    }
    c.seq += 1;
    c.attempts += 1;
    if c.fail_nth != 0 && c.seq == c.fail_nth {
        return Err(ERR_ENV_OOM);
    }
    let base = unsafe { HeapAlloc(process_heap(), 0, len as usize + 4) } as *mut u8;
    if base.is_null() {
        return Err(ERR_ENV_OOM);
    }
    unsafe {
        write_volatile(base as *mut u32, OWNED_MAGIC);
        // header second word is not needed: length is recoverable from
        // HeapSize-free bookkeeping via live_bytes; free() needs no len.
    }
    c.allocs += 1;
    c.live += 1;
    c.live_bytes += len;
    if c.live_bytes > c.peak {
        c.peak = c.live_bytes;
    }
    Ok(unsafe { base.add(4) })
}

/// free of an owned buffer: null is a no-op (uncounted); a valid header
/// releases; a stale header counts the call (violation surfaces through
/// check_invariant) without a second physical HeapFree.
fn owned_free(ptr: *mut u8) {
    let c = c();
    if ptr.is_null() {
        return;
    }
    c.frees += 1;
    let base = unsafe { ptr.sub(4) };
    let magic = unsafe { read_volatile(base as *const u32) };
    if magic != OWNED_MAGIC {
        return; // detected double free: counted, physically skipped
    }
    unsafe { write_volatile(base as *mut u32, 0) };
    unsafe { HeapFree(process_heap(), 0, base as *mut core::ffi::c_void) };
    if c.live > 0 {
        c.live -= 1;
    }
    // live_bytes: recover the declared length from the arena ledger below.
    let len = arena_take_len(ptr);
    c.live_bytes = c.live_bytes.saturating_sub(len);
}

// Tiny pointer→length ledger for live allocations. The magic header already
// proves validity; the ledger only carries the declared user length needed
// to keep live_bytes exact on free. Fixed capacity, linear scan — a probe,
// not a production allocator. Raw-pointer access throughout: no indexing,
// so no panic paths can drag core::fmt (and the CRT) into the link.
const ARENA_MAX: usize = 64;
static mut ARENA: [(*mut u8, u32); ARENA_MAX] = [(core::ptr::null_mut(), 0); ARENA_MAX];

fn arena_base() -> *mut (*mut u8, u32) {
    unsafe { addr_of_mut!(ARENA) as *mut (*mut u8, u32) }
}

fn arena_put(ptr: *mut u8, len: u32) {
    let base = arena_base();
    for i in 0..ARENA_MAX {
        let slot = unsafe { &mut *base.add(i) };
        if slot.0.is_null() {
            slot.0 = ptr;
            slot.1 = len;
            return;
        }
    }
    // Table full: the probe never holds >64 live allocations; drop bookkeeping
    // rather than fail the allocation that already succeeded.
}

fn arena_take_len(ptr: *mut u8) -> u32 {
    let base = arena_base();
    for i in 0..ARENA_MAX {
        let slot = unsafe { &mut *base.add(i) };
        if slot.0 == ptr {
            let len = slot.1;
            slot.0 = core::ptr::null_mut();
            slot.1 = 0;
            return len;
        }
    }
    0
}

// ---------------------------------------------------------------------------
// trace (stderr, exact frozen line formats)
// ---------------------------------------------------------------------------

struct Line([u8; 160], usize);

impl Line {
    fn new() -> Line {
        Line([0; 160], 0)
    }
    fn b(&mut self, ch: u8) {
        if self.1 < self.0.len() {
            unsafe { write_volatile(self.0.as_mut_ptr().add(self.1), ch) };
            self.1 += 1;
        }
    }
    fn s(&mut self, s: &str) {
        for &ch in s.as_bytes() {
            self.b(ch);
        }
    }
    fn u(&mut self, v: u32) {
        if v == 0 {
            self.b(b'0');
            return;
        }
        let mut buf = [0u8; 10];
        let mut n = 0;
        let mut v = v;
        while v > 0 {
            let digit = b'0' + (v % 10) as u8;
            unsafe { write_volatile(buf.as_mut_ptr().add(n), digit) };
            v /= 10;
            n += 1;
        }
        while n > 0 {
            n -= 1;
            let digit = unsafe { read_volatile(buf.as_ptr().add(n)) };
            self.b(digit);
        }
    }
    fn i(&mut self, v: i32) {
        if v < 0 {
            self.b(b'-');
            self.u(v.unsigned_abs());
        } else {
            self.u(v as u32);
        }
    }
    fn emit(&self) {
        let Line(buf, len) = self;
        unsafe {
            let handle = GetStdHandle(STD_ERROR_HANDLE);
            let mut written: u32 = 0;
            WriteFile(
                handle,
                buf.as_ptr(),
                *len as u32,
                &mut written as *mut u32,
                core::ptr::null_mut(),
            );
        }
    }
}

// Freestanding mem* for the no-CRT PE link: struct initialization and
// by-value moves in this crate lower to memcpy/memset libcalls. Volatile
// byte loops so the compiler cannot pattern-match them back into a call to
// themselves.
#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        write_volatile(dst.add(i), read_volatile(src.add(i)));
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if src < dst as *const u8 && src.add(n) > dst as *const u8 {
        let mut i = n;
        while i > 0 {
            i -= 1;
            write_volatile(dst.add(i), read_volatile(src.add(i)));
        }
    } else {
        for i in 0..n {
            write_volatile(dst.add(i), read_volatile(src.add(i)));
        }
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, val: i32, n: usize) -> *mut u8 {
    let byte = val as u8;
    for i in 0..n {
        write_volatile(dst.add(i), byte);
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let (x, y) = (read_volatile(a.add(i)), read_volatile(b.add(i)));
        if x != y {
            return x as i32 - y as i32;
        }
    }
    0
}

fn trace_line(f: impl FnOnce(&mut Line)) {
    let mut line = Line::new();
    f(&mut line);
    line.b(b'\n');
    line.emit();
} // ---------------------------------------------------------------------------
  // ABI-A: scalars by value
  // ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn p774_mix(a: i32, b: i32, c: i32, d: i32) -> i32 {
    trace_line(|l| {
        l.s("MIX a=");
        l.i(a);
        l.s(" b=");
        l.i(b);
        l.s(" c=");
        l.u(c as u32);
        l.s(" d=");
        l.i(d);
    });
    a.wrapping_add(b).wrapping_add(c).wrapping_add(d)
}

#[no_mangle]
pub extern "system" fn p774_many(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32) -> i32 {
    trace_line(|l| {
        l.s("MANY a=");
        l.i(a);
        l.s(" b=");
        l.i(b);
        l.s(" c=");
        l.i(c);
        l.s(" d=");
        l.i(d);
        l.s(" e=");
        l.i(e);
        l.s(" f=");
        l.i(f);
        l.s(" g=");
        l.i(g);
    });
    a.wrapping_add(b)
        .wrapping_add(c)
        .wrapping_add(d)
        .wrapping_add(e)
        .wrapping_add(f)
        .wrapping_add(g)
}

// ---------------------------------------------------------------------------
// ABI-B: borrowed buffer (ptr, len) — read-only, never freed by the callee
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn p774_borrow_sum(ptr: *const u8, len: i32) -> i32 {
    if len < 0 {
        return ERR_LEN_NEGATIVE;
    }
    trace_line(|l| {
        l.s("BSUM len=");
        l.i(len);
    });
    let mut sum: i32 = 0;
    for i in 0..len as usize {
        sum = sum.wrapping_add(unsafe { read_volatile(ptr.add(i)) } as i32);
    }
    sum
}

#[no_mangle]
pub extern "system" fn p774_borrow_read(ptr: *const u8, len: i32, idx: i32) -> i32 {
    if idx < 0 || idx >= len {
        return ERR_INDEX_OOB;
    }
    let byte = unsafe { read_volatile(ptr.add(idx as usize)) };
    byte as i32
}

// ---------------------------------------------------------------------------
// ABI-C: owned buffer out (ptr return + *mut i32 length/status channel)
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn p774_own_make(seed: i32, len: i32, out_len: *mut i32) -> *mut u8 {
    if len < 0 {
        unsafe { write_volatile(out_len, ERR_LEN_NEGATIVE) };
        return core::ptr::null_mut();
    }
    trace_line(|l| {
        l.s("OWN-MAKE seed=");
        l.i(seed);
        l.s(" len=");
        l.i(len);
    });
    match try_alloc(len as u32) {
        Ok(p) if p.is_null() => {
            // zero length: valid, no allocation
            unsafe { write_volatile(out_len, 0) };
            core::ptr::null_mut()
        }
        Ok(p) => {
            for i in 0..len as usize {
                unsafe { write_volatile(p.add(i), (seed.wrapping_add(i as i32)) as u8) };
            }
            arena_put(p, len as u32);
            unsafe { write_volatile(out_len, len) };
            p
        }
        Err(code) => {
            unsafe { write_volatile(out_len, code) };
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn p774_own_free(ptr: *mut u8) {
    owned_free(ptr);
}

#[no_mangle]
pub extern "system" fn p774_own_clone(src: *const u8, len: i32, out_len: *mut i32) -> *mut u8 {
    if len < 0 {
        unsafe { write_volatile(out_len, ERR_LEN_NEGATIVE) };
        return core::ptr::null_mut();
    }
    trace_line(|l| {
        l.s("CLONE len=");
        l.i(len);
    });
    match try_alloc(len as u32) {
        Ok(p) if p.is_null() => {
            unsafe { write_volatile(out_len, 0) };
            core::ptr::null_mut()
        }
        Ok(p) => {
            for i in 0..len as usize {
                let byte = unsafe { read_volatile(src.add(i)) };
                unsafe { write_volatile(p.add(i), byte) };
            }
            arena_put(p, len as u32);
            unsafe { write_volatile(out_len, len) };
            p
        }
        Err(code) => {
            unsafe { write_volatile(out_len, code) };
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn p774_own_concat(
    a_ptr: *const u8,
    a_len: i32,
    b_ptr: *const u8,
    b_len: i32,
    out_len: *mut i32,
) -> *mut u8 {
    trace_line(|l| {
        l.s("CONCAT a=");
        l.i(a_len);
        l.s(" b=");
        l.i(b_len);
    });
    if a_len < 0 || b_len < 0 {
        unsafe { write_volatile(out_len, ERR_LEN_NEGATIVE) };
        return core::ptr::null_mut();
    }
    // Length arithmetic and the overflow rejection run BEFORE any buffer is
    // read or any allocation is attempted (probe-cases V05-6).
    let total = a_len as u64 + b_len as u64;
    if total > i32::MAX as u64 {
        unsafe { write_volatile(out_len, ERR_LEN_OVERFLOW) };
        return core::ptr::null_mut();
    }
    match try_alloc(total as u32) {
        Ok(p) if p.is_null() => {
            unsafe { write_volatile(out_len, 0) };
            core::ptr::null_mut()
        }
        Ok(p) => {
            let mut w = 0usize;
            for i in 0..a_len as usize {
                let byte = unsafe { read_volatile(a_ptr.add(i)) };
                unsafe { write_volatile(p.add(w), byte) };
                w += 1;
            }
            for i in 0..b_len as usize {
                let byte = unsafe { read_volatile(b_ptr.add(i)) };
                unsafe { write_volatile(p.add(w), byte) };
                w += 1;
            }
            arena_put(p, total as u32);
            unsafe { write_volatile(out_len, total as i32) };
            p
        }
        Err(code) => {
            unsafe { write_volatile(out_len, code) };
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn p774_own_from_lit(
    lit_ptr: *const u8,
    lit_len: i32,
    out_len: *mut i32,
) -> *mut u8 {
    if lit_len < 0 {
        unsafe { write_volatile(out_len, ERR_LEN_NEGATIVE) };
        return core::ptr::null_mut();
    }
    trace_line(|l| {
        l.s("LIT len=");
        l.i(lit_len);
    });
    match try_alloc(lit_len as u32) {
        Ok(p) if p.is_null() => {
            unsafe { write_volatile(out_len, 0) };
            core::ptr::null_mut()
        }
        Ok(p) => {
            for i in 0..lit_len as usize {
                let byte = unsafe { read_volatile(lit_ptr.add(i)) };
                unsafe { write_volatile(p.add(i), byte) };
            }
            arena_put(p, lit_len as u32);
            unsafe { write_volatile(out_len, lit_len) };
            p
        }
        Err(code) => {
            unsafe { write_volatile(out_len, code) };
            core::ptr::null_mut()
        }
    }
}

// ---------------------------------------------------------------------------
// ABI-D / ABI-E: record view / record mut / record out-return
// Rec1 layout (frozen): flag: bool @0, count: i32 @4, tag: char(u32) @8,
// size 12, align 4, offsets 4*i (probe-cases §V-02).
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn p774_rec_view(p: *const u8) -> i32 {
    let flag = unsafe { read_volatile(p as *const i32) };
    let count = unsafe { read_volatile(p.add(4) as *const i32) };
    let tag = unsafe { read_volatile(p.add(8) as *const u32) };
    trace_line(|l| {
        l.s("REC-VIEW flag=");
        l.i(flag);
        l.s(" count=");
        l.i(count);
        l.s(" tag=");
        l.u(tag);
    });
    flag.wrapping_add(count.wrapping_mul(2))
        .wrapping_add((tag as i32).wrapping_mul(3))
}

#[no_mangle]
pub extern "system" fn p774_rec_mut(p: *mut u8, addend: i32) {
    trace_line(|l| {
        l.s("REC-MUT addend=");
        l.i(addend);
    });
    unsafe {
        let count = read_volatile(p.add(4) as *const i32);
        write_volatile(p.add(4) as *mut i32, count.wrapping_add(addend));
        let flag = read_volatile(p as *const i32);
        write_volatile(p as *mut i32, 1 - flag);
    }
}

#[no_mangle]
pub extern "system" fn p774_rec_out(seed: i32, out: *mut u8) {
    trace_line(|l| {
        l.s("REC-OUT seed=");
        l.i(seed);
    });
    unsafe {
        write_volatile(out as *mut i32, seed & 1);
        write_volatile(out.add(4) as *mut i32, seed);
        write_volatile(out.add(8) as *mut u32, (seed | 0x40) as u32);
    }
}

// ---------------------------------------------------------------------------
// text ops (borrowed UTF-8 buffers; D-02 rules)
// ---------------------------------------------------------------------------

/// Scan `len` bytes: on success return (scalar_count, per-scalar end offsets
/// not materialized); validation is the strict UTF-8 level (rejects
/// overlongs, surrogates, > U+10FFFF, stray continuation bytes).
fn utf8_check(ptr: *const u8, len: u32) -> Result<u32, ()> {
    let mut pos: u32 = 0;
    let mut scalars: u32 = 0;
    while pos < len {
        let b0 = unsafe { read_volatile(ptr.add(pos as usize)) };
        let (seq, ok): (u32, bool) = match b0 {
            0x00..=0x7F => (1, true),
            0xC2..=0xDF => (2, true),
            0xE0..=0xEF => (3, true),
            0xF0..=0xF4 => (4, true),
            _ => (0, false),
        };
        if !ok || pos + seq > len {
            return Err(());
        }
        // Validate continuation bytes and the tight ranges that reject
        // overlong encodings, surrogates and > U+10FFFF.
        let cont = |i: u32| -> bool {
            let b = unsafe { read_volatile(ptr.add((pos + i) as usize)) };
            (0x80..=0xBF).contains(&b)
        };
        let b1_in = |lo: u8, hi: u8| -> bool {
            let b = unsafe { read_volatile(ptr.add((pos + 1) as usize)) };
            (lo..=hi).contains(&b)
        };
        let strict = match b0 {
            0x00..=0x7F => true,
            0xC2..=0xDF => cont(1),
            0xE0 => b1_in(0xA0, 0xBF) && cont(2),
            0xE1..=0xEC => cont(1) && cont(2),
            0xED => b1_in(0x80, 0x9F) && cont(2),
            0xEE..=0xEF => cont(1) && cont(2),
            0xF0 => b1_in(0x90, 0xBF) && cont(2) && cont(3),
            0xF1..=0xF3 => cont(1) && cont(2) && cont(3),
            0xF4 => b1_in(0x80, 0x8F) && cont(2) && cont(3),
            _ => false,
        };
        if !strict {
            return Err(());
        }
        pos += seq;
        scalars += 1;
    }
    Ok(scalars)
}

/// Byte offset and decoded value of scalar index `idx` (0-based).
fn utf8_scalar_at(ptr: *const u8, len: u32, idx: u32) -> Result<(u32, u32), i32> {
    let scalars = utf8_check(ptr, len).map_err(|_| ERR_INVALID_UTF8)?;
    if idx >= scalars {
        return Err(ERR_RANGE_OOB);
    }
    let mut pos: u32 = 0;
    let mut sc: u32 = 0;
    while pos < len {
        let b0 = unsafe { read_volatile(ptr.add(pos as usize)) };
        let seq: u32 = match b0 {
            0x00..=0x7F => 1,
            0xC2..=0xDF => 2,
            0xE0..=0xEF => 3,
            _ => 4,
        };
        if sc == idx {
            let value: u32 = match seq {
                1 => b0 as u32,
                2 => {
                    let b1 = unsafe { read_volatile(ptr.add((pos + 1) as usize)) } as u32;
                    ((b0 as u32 & 0x1F) << 6) | (b1 & 0x3F)
                }
                3 => {
                    let b1 = unsafe { read_volatile(ptr.add((pos + 1) as usize)) } as u32;
                    let b2 = unsafe { read_volatile(ptr.add((pos + 2) as usize)) } as u32;
                    ((b0 as u32 & 0x0F) << 12) | ((b1 & 0x3F) << 6) | (b2 & 0x3F)
                }
                _ => {
                    let b1 = unsafe { read_volatile(ptr.add((pos + 1) as usize)) } as u32;
                    let b2 = unsafe { read_volatile(ptr.add((pos + 2) as usize)) } as u32;
                    let b3 = unsafe { read_volatile(ptr.add((pos + 3) as usize)) } as u32;
                    ((b0 as u32 & 0x07) << 18)
                        | ((b1 & 0x3F) << 12)
                        | ((b2 & 0x3F) << 6)
                        | (b3 & 0x3F)
                }
            };
            return Ok((pos, value));
        }
        pos += seq;
        sc += 1;
    }
    Err(ERR_RANGE_OOB)
}

#[no_mangle]
pub extern "system" fn p774_text_scalars(ptr: *const u8, len: i32) -> i32 {
    if len < 0 {
        return ERR_LEN_NEGATIVE;
    }
    match utf8_check(ptr, len as u32) {
        Ok(n) => {
            trace_line(|l| {
                l.s("TEXT-SCALARS n=");
                l.i(n as i32);
            });
            n as i32
        }
        Err(()) => ERR_INVALID_UTF8,
    }
}

#[no_mangle]
pub extern "system" fn p774_text_bytes(_ptr: *const u8, len: i32) -> i32 {
    if len < 0 {
        return ERR_LEN_NEGATIVE;
    }
    trace_line(|l| {
        l.s("TEXT-BYTES n=");
        l.i(len);
    });
    len
}

#[no_mangle]
pub extern "system" fn p774_char_at(ptr: *const u8, len: i32, idx: i32) -> i32 {
    if idx < 0 {
        return ERR_RANGE_OOB;
    }
    if len < 0 {
        return ERR_LEN_NEGATIVE;
    }
    match utf8_scalar_at(ptr, len as u32, idx as u32) {
        Ok((_, value)) => {
            trace_line(|l| {
                l.s("CHAR-AT i=");
                l.i(idx);
                l.s(" v=");
                l.u(value);
            });
            value as i32
        }
        Err(code) => code,
    }
}

#[no_mangle]
pub extern "system" fn p774_text_slice(
    ptr: *const u8,
    len: i32,
    start: i32,
    end: i32,
    out_len: *mut i32,
) -> *const u8 {
    if len < 0 {
        unsafe { write_volatile(out_len, ERR_LEN_NEGATIVE) };
        return core::ptr::null();
    }
    if start < 0 || end < 0 {
        unsafe { write_volatile(out_len, ERR_RANGE_OOB) };
        return core::ptr::null();
    }
    if start > end {
        unsafe { write_volatile(out_len, ERR_RANGE_REVERSED) };
        return core::ptr::null();
    }
    let scalars = match utf8_check(ptr, len as u32) {
        Ok(n) => n,
        Err(()) => {
            unsafe { write_volatile(out_len, ERR_INVALID_UTF8) };
            return core::ptr::null();
        }
    };
    if (end as u32) > scalars {
        unsafe { write_volatile(out_len, ERR_RANGE_OOB) };
        return core::ptr::null();
    }
    let start_off = if start as u32 == scalars {
        len as u32
    } else {
        utf8_scalar_at(ptr, len as u32, start as u32)
            .map(|(off, _)| off)
            .unwrap_or(len as u32)
    };
    let byte_len = if start == end {
        0
    } else {
        let end_off = if end as u32 == scalars {
            len as u32
        } else {
            utf8_scalar_at(ptr, len as u32, end as u32)
                .map(|(off, _)| off)
                .unwrap_or(len as u32)
        };
        end_off - start_off
    };
    trace_line(|l| {
        l.s("SLICE s=");
        l.i(start);
        l.s(" e=");
        l.i(end);
        l.s(" len=");
        l.i(byte_len as i32);
    });
    unsafe { write_volatile(out_len, byte_len as i32) };
    unsafe { ptr.add(start_off as usize) }
}

#[no_mangle]
pub extern "system" fn p774_char_from_codepoint(cp: i32) -> i32 {
    trace_line(|l| {
        l.s("FROM-CP cp=");
        l.i(cp);
    });
    if cp < 0 || cp > 0x10FFFF || (0xD800..=0xDFFF).contains(&cp) {
        ERR_INVALID_SCALAR
    } else {
        cp
    }
}

#[no_mangle]
pub extern "system" fn p774_char_to_codepoint(ch: i32) -> i32 {
    if ch < 0 || ch > 0x10FFFF || (0xD800..=0xDFFF).contains(&ch) {
        ERR_INVALID_SCALAR
    } else {
        ch
    }
}

// ---------------------------------------------------------------------------
// length boundary conversions (D-01)
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn p774_len_from_i32(v: i32) -> i32 {
    if v < 0 {
        ERR_LEN_NEGATIVE
    } else {
        v
    }
}

#[no_mangle]
pub extern "system" fn p774_len_to_i32(v: i64) -> i32 {
    if v < 0 || v > i32::MAX as i64 {
        ERR_LEN_TOO_LARGE
    } else {
        v as i32
    }
}

// ---------------------------------------------------------------------------
// control / fault injection / summary / trap / hang
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn p774_set_budget(bytes: i64) {
    c().budget = if bytes < 0 { 0 } else { bytes as u32 };
}

#[no_mangle]
pub extern "system" fn p774_fail_nth_alloc(n: i32) {
    c().fail_nth = if n < 0 { 0 } else { n as u32 };
}

#[no_mangle]
pub extern "system" fn p774_shim_error() -> i32 {
    ERR_SHIM
}

fn emit_summary() {
    let c = c();
    trace_line(|l| {
        l.s("SUMMARY attempts=");
        l.u(c.attempts);
        l.s(" allocs=");
        l.u(c.allocs);
        l.s(" frees=");
        l.u(c.frees);
        l.s(" live=");
        l.u(c.live);
        l.s(" live_bytes=");
        l.u(c.live_bytes);
        l.s(" peak=");
        l.u(c.peak);
    });
}

#[no_mangle]
pub extern "system" fn p774_summary() {
    emit_summary();
}

#[no_mangle]
pub extern "system" fn p774_check_invariant() -> i32 {
    emit_summary();
    let c = c();
    if c.allocs == c.frees && c.live == 0 {
        0
    } else {
        1
    }
}

#[no_mangle]
pub extern "system" fn p774_trap_after_alloc(len: i32) -> ! {
    let p = match try_alloc(len.max(0) as u32) {
        Ok(p) => p,
        Err(_) => core::ptr::null_mut(),
    };
    let _ = p; // the allocation stays live on purpose: OS reclaim boundary
    trace_line(|l| {
        l.s("TRAP-PRE live=");
        l.u(c().live);
    });
    // Deliberate access violation: a volatile write to null is emitted as a
    // real store, so the process dies with 0xC0000005 (not a ud2 trap).
    unsafe { write_volatile(core::ptr::null_mut::<u8>(), 1) };
    loop {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "system" fn p774_hang_after_alloc(len: i32) -> ! {
    let p = match try_alloc(len.max(0) as u32) {
        Ok(p) => p,
        Err(_) => core::ptr::null_mut(),
    };
    let _ = p;
    trace_line(|l| {
        l.s("HANG-PRE live=");
        l.u(c().live);
    });
    loop {
        core::hint::spin_loop();
    }
}

// ---------------------------------------------------------------------------
// R2 control family: restricted refcount handles (candidate comparison only;
// rejected for the first batch — decisions.md §D-06)
// ---------------------------------------------------------------------------

const H_SLOTS: usize = 8;

#[derive(Copy, Clone)]
struct HSlot {
    active: bool,
    rc: u32,
    value: u8,
}

static mut HTAB: [HSlot; H_SLOTS] = [HSlot {
    active: false,
    rc: 0,
    value: 0,
}; H_SLOTS];

fn hslot(h: i32) -> Option<*mut HSlot> {
    if h < 1 || h as usize > H_SLOTS {
        None
    } else {
        Some(unsafe { (addr_of_mut!(HTAB) as *mut HSlot).add(h as usize - 1) })
    }
}

#[no_mangle]
pub extern "system" fn p774_h_alloc(pattern: i32) -> i32 {
    let base = unsafe { addr_of_mut!(HTAB) as *mut HSlot };
    for i in 0..H_SLOTS {
        let slot = unsafe { &mut *base.add(i) };
        if !slot.active {
            slot.active = true;
            slot.rc = 1;
            slot.value = pattern as u8;
            trace_line(|l| {
                l.s("H-ALLOC h=");
                l.u(i as u32 + 1);
            });
            return i as i32 + 1;
        }
    }
    ERR_ENV_OOM
}

#[no_mangle]
pub extern "system" fn p774_h_clone(h: i32) -> i32 {
    let Some(slot) = hslot(h).map(|p| unsafe { &mut *p }) else {
        return ERR_INDEX_OOB;
    };
    slot.rc += 1;
    let rc = slot.rc;
    trace_line(|l| {
        l.s("H-CLONE h=");
        l.u(h as u32);
        l.s(" rc=");
        l.u(rc);
    });
    h
}

#[no_mangle]
pub extern "system" fn p774_h_drop(h: i32) {
    let Some(slot) = hslot(h).map(|p| unsafe { &mut *p }) else {
        return;
    };
    if slot.rc > 0 {
        slot.rc -= 1;
    }
    let rc = slot.rc;
    if rc == 0 {
        slot.active = false;
        // value is NOT cleared: a stale handle silently reads recycled state
    }
    trace_line(|l| {
        l.s("H-DROP h=");
        l.u(h as u32);
        l.s(" rc=");
        l.u(rc);
    });
}

#[no_mangle]
pub extern "system" fn p774_h_get(h: i32) -> i32 {
    let Some(slot) = hslot(h).map(|p| unsafe { &*p }) else {
        return ERR_INDEX_OOB;
    };
    let v = slot.value;
    trace_line(|l| {
        l.s("H-GET h=");
        l.u(h as u32);
        l.s(" v=");
        l.u(v as u32);
    });
    v as i32
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // The shim never panics by construction; if it ever does, die loudly
    // without unwinding across the ABI.
    loop {
        core::hint::spin_loop();
    }
}
