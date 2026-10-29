//! PLAN-770 text runtime — the CALLEE side of the text-profile internal
//! ABI (774 decisions.md §3 D-05/D-06 as amended by §3.8..3.12, consumed
//! verbatim; docs/reports/770-text/design.md §5).
//!
//! This is the runtime static library the ac-core text lowering links into
//! every text-profile PE via `--support-lib`. It is an isomorphic rename of
//! the PLAN-774 probe shim (same counters, ledger, budget and trace line
//! formats), pruned to the 770 consumption face: no record family (771's
//! surface), no R2 handle family (rejected candidate), no probe-only
//! scalar/register controls. 770 additions: `ac_rt_text_eq/ne` (byte-exact
//! UTF-8 value equality) and the `RT-FAIL code=<n>` stderr trace emitted on
//! EVERY negative status return, so the generated code's explicit
//! ExitProcess(70) path is observable per failure class.
//!
//! no_std on purpose (the PE links against kernel32 only, no CRT); no
//! unwinding may ever cross the ABI (panic=abort at the workspace root).

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
pub const ERR_CANCELLED: i32 = -12;

pub type AllocHook = unsafe extern "system" fn(usize) -> *mut core::ffi::c_void;
pub type FreeHook = unsafe extern "system" fn(*mut core::ffi::c_void) -> i32;
pub type PollHook = unsafe extern "system" fn(u64) -> i32;
static mut ALLOC_HOOK: Option<AllocHook> = None;
static mut FREE_HOOK: Option<FreeHook> = None;
static mut POLL_HOOK: Option<PollHook> = None;

/// Private host seam: install only before any buffer or retained ledger exists.
/// Existing callers retain the original process-heap implementation by default.
#[no_mangle]
pub extern "system" fn ac_rt_install_allocator(
    alloc: Option<AllocHook>,
    free: Option<FreeHook>,
    poll: Option<PollHook>,
) -> i32 {
    if alloc.is_none() != free.is_none() || c().live != 0 || led_state().1 != 0 {
        return ERR_SHIM;
    }
    unsafe {
        ALLOC_HOOK = alloc;
        FREE_HOOK = free;
        POLL_HOOK = poll;
    }
    0
}
unsafe fn memory_alloc(bytes: usize) -> *mut core::ffi::c_void {
    match ALLOC_HOOK {
        Some(f) => f(bytes),
        None => HeapAlloc(process_heap(), 0, bytes),
    }
}
unsafe fn memory_free(ptr: *mut core::ffi::c_void) -> i32 {
    match FREE_HOOK {
        Some(f) => f(ptr),
        None => HeapFree(process_heap(), 0, ptr),
    }
}
#[no_mangle]
pub extern "system" fn ac_rt_metadata_bytes() -> usize {
    led_state().1 * core::mem::size_of::<LedgerEntry>()
}
#[no_mangle]
pub extern "system" fn ac_rt_release_idle_metadata() -> i32 {
    let (base, _, len) = led_state();
    if len != 0 || c().live != 0 {
        return ERR_SHIM;
    }
    unsafe {
        if !base.is_null() && memory_free(base.cast()) == 0 {
            c().invalid_frees += 1;
            return ERR_SHIM;
        }
        LEDGER = core::ptr::null_mut();
        LED_CAP = 0;
    }
    0
}
/// Poll per bounded byte chunk; cancellation owns and rolls back the new buffer.
unsafe fn copy_poll(dst: *mut u8, src: *const u8, len: usize) -> bool {
    let mut start = 0;
    while start < len {
        let end = (start + 4096).min(len);
        if let Some(poll) = POLL_HOOK {
            if poll((end - start) as u64) != 0 {
                owned_free(dst);
                return false;
            }
        }
        for i in start..end {
            write_volatile(dst.add(i), read_volatile(src.add(i)));
        }
        start = end;
    }
    true
}

// ---------------------------------------------------------------------------
// counters + fault injection state (single-threaded PE state)
//
// Review r1 (P774-R1-01/05) counting contract: `frees` counts SUCCESSFUL
// releases only (ledger-validated, HeapFree returned nonzero); free calls on
// non-null pointers are counted in `free_attempts`; pointers absent from the
// ledger (double free / foreign) are counted in `invalid_frees` WITHOUT
// reading any freed memory. `live_bytes` is backed by the complete ledger
// below, so a release always restores the budget allowance.
// ---------------------------------------------------------------------------

struct Counters {
    attempts: u32,   // HeapAlloc invocations (after len/budget checks)
    allocs: u32,     // successful allocations
    frees: u32,      // successful releases (ledger-validated)
    live: u32,       // allocs - frees
    live_bytes: u64, // declared user bytes currently live (ledger-backed,
    // r4: full precision — never saturates)
    peak: u64,            // max live_bytes at alloc/free boundaries (u64)
    budget: Option<u64>, // None = unlimited; Some(0) = a real ZERO-byte budget (r3: full i64 range, no truncation)
    fail_nth: u32,       // 0 = off; nth allocator invocation fails (injected)
    seq: u32,            // allocator invocation ordinal
    free_attempts: u32,  // free calls on non-null pointers
    invalid_frees: u32,  // double frees / foreign pointers (ledger misses)
    next_id: u32,        // monotonic resource id for the FREE trace channel
    fail_ledger_nth: u32, // 0 = off; nth LEDGER allocation attempt fails (r2)
    ledger_seq: u32,     // ledger allocation invocation ordinal (metadata)
}

static mut C: Counters = Counters {
    attempts: 0,
    allocs: 0,
    frees: 0,
    live: 0,
    live_bytes: 0,
    peak: 0,
    budget: None,
    fail_nth: 0,
    seq: 0,
    free_attempts: 0,
    invalid_frees: 0,
    next_id: 0,
    fail_ledger_nth: 0,
    ledger_seq: 0,
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

// ---------------------------------------------------------------------------
// complete live-resource ledger (review r1: no silent capacity loss)
//
// Heap-backed growable array keyed by user pointer; every successful
// allocation inserts (ptr, len, id), every release removes exactly that
// entry. Growth doubles the capacity via the same process heap, so the
// ledger is always complete for ALL live allocations — the budget allowance
// and live_bytes can never diverge from reality. Raw-pointer access
// throughout: no indexing, so no panic paths can drag core::fmt (and the
// CRT) into the no-CRT link.
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
struct LedgerEntry {
    ptr: *mut u8,
    len: u32,
    id: u32,
}

static mut LEDGER: *mut LedgerEntry = core::ptr::null_mut();
static mut LED_CAP: usize = 0;
static mut LED_LEN: usize = 0;

fn led_state() -> (*mut LedgerEntry, usize, usize) {
    unsafe {
        (
            core::ptr::addr_of_mut!(LEDGER).read(),
            core::ptr::addr_of_mut!(LED_CAP).read(),
            core::ptr::addr_of_mut!(LED_LEN).read(),
        )
    }
}

// Counting boundary (r2 P774-R2-03): ledger metadata allocations are
// INTERNAL bookkeeping — they are NOT counted in attempts/allocs/frees,
// which count USER resource requests only. Their failures have a dedicated
// injection knob (`ac_rt_fail_ledger_nth`) and surface as env-oom for the
// user request after a full rollback.
fn ledger_insert(ptr: *mut u8, len: u32, id: u32) -> bool {
    unsafe {
        let (base, cap, len_n) = led_state();
        if len_n == cap {
            let new_cap = if cap == 0 { 16 } else { cap * 2 };
            let nbytes = new_cap * core::mem::size_of::<LedgerEntry>();
            let c0 = c();
            c0.ledger_seq += 1;
            if c0.fail_ledger_nth != 0 && c0.ledger_seq == c0.fail_ledger_nth {
                return false; // injected metadata failure (first or growth)
            }
            let fresh = memory_alloc(nbytes) as *mut LedgerEntry;
            if fresh.is_null() {
                return false; // real metadata allocation failure
            }
            for i in 0..len_n {
                write_volatile(fresh.add(i) as *mut LedgerEntry, read_volatile(base.add(i)));
            }
            if !base.is_null() {
                let freed = memory_free(base as *mut core::ffi::c_void);
                if freed == 0 {
                    // the old table release failed: surface through the
                    // invalid-release counter (wired; not injectable)
                    c().invalid_frees += 1;
                    trace_line(|l| {
                        l.s("FREE-INVALID");
                    });
                }
            }
            core::ptr::addr_of_mut!(LEDGER).write(fresh);
            core::ptr::addr_of_mut!(LED_CAP).write(new_cap);
        }
        let base = core::ptr::addr_of_mut!(LEDGER).read();
        let at = core::ptr::addr_of_mut!(LED_LEN).read();
        write_volatile(
            base.add(at) as *mut LedgerEntry,
            LedgerEntry { ptr, len, id },
        );
        core::ptr::addr_of_mut!(LED_LEN).write(at + 1);
        true
    }
}

/// Remove the entry for `ptr`; returns (len, id) when found.
fn ledger_take(ptr: *mut u8) -> Option<(u32, u32)> {
    unsafe {
        let (base, _cap, len_n) = led_state();
        for i in 0..len_n {
            let e = read_volatile(base.add(i));
            if e.ptr == ptr {
                let last = len_n - 1;
                let tail = read_volatile(base.add(last));
                write_volatile(base.add(i) as *mut LedgerEntry, tail);
                core::ptr::addr_of_mut!(LED_LEN).write(last);
                return Some((e.len, e.id));
            }
        }
        None
    }
}

/// The single allocation entry behind every declared request point
/// (AP-1..AP-4). Counting: len 0 is valid and makes NO allocator call;
/// budget rejection makes NO allocator call; the injected nth failure and
/// real HeapAlloc null both count as attempts and return ERR_ENV_OOM.
fn try_alloc(len: u32) -> Result<(*mut u8, u32), i32> {
    let c = c();
    if len == 0 {
        return Ok((core::ptr::null_mut(), 0));
    }
    if let Some(budget) = c.budget {
        // u64 arithmetic on both sides: the budget keeps its full i64 value
        // and the live-byte sum cannot wrap before the comparison (r3).
        if c.live_bytes + (len as u64) > budget {
            return Err(ERR_BUDGET);
        }
    }
    c.seq += 1;
    c.attempts += 1;
    if c.fail_nth != 0 && c.seq == c.fail_nth {
        return Err(ERR_ENV_OOM);
    }
    let p = unsafe { memory_alloc(len as usize) } as *mut u8;
    if p.is_null() {
        return Err(ERR_ENV_OOM);
    }
    c.next_id += 1;
    let id = c.next_id;
    if !ledger_insert(p, len, id) {
        // ledger failure: roll the user buffer back so accounting stays
        // complete; a refused rollback surfaces through invalid_frees
        let freed = unsafe { memory_free(p as *mut core::ffi::c_void) };
        if freed == 0 {
            c.invalid_frees += 1;
            trace_line(|l| {
                l.s("FREE-INVALID");
            });
        }
        return Err(ERR_ENV_OOM);
    }
    c.allocs += 1;
    c.live += 1;
    // full-precision accumulate (r4): u64 cannot saturate at any budget the
    // i64 setter accepts, so no high bits are ever dropped
    c.live_bytes += len as u64;
    if c.live_bytes > c.peak {
        c.peak = c.live_bytes;
    }
    Ok((p, id))
}

/// free of an owned buffer: null is a no-op (uncounted). The pointer is
/// validated against the EXTERNAL ledger first — a ledger miss is a double
/// free / foreign pointer (counted in invalid_frees, no freed memory is
/// read); a hit is released via HeapFree and only counted as a successful
/// free when the OS call returns nonzero.
fn owned_free(ptr: *mut u8) {
    let c = c();
    if ptr.is_null() {
        return;
    }
    c.free_attempts += 1;
    let Some((len, id)) = ledger_take(ptr) else {
        c.invalid_frees += 1;
        trace_line(|l| {
            l.s("FREE-INVALID");
        });
        return;
    };
    let ok = unsafe { memory_free(ptr as *mut core::ffi::c_void) };
    if ok == 0 {
        // OS refused the release: the entry was already removed; the
        // accounting treats this as an invalid release attempt.
        c.invalid_frees += 1;
        trace_line(|l| {
            l.s("FREE-INVALID");
        });
        return;
    }
    c.frees += 1;
    if c.live > 0 {
        c.live -= 1;
    }
    // exact release: the ledger-backed length is always <= live_bytes;
    // subtract plainly so no discrepancy can hide behind a floor
    c.live_bytes -= len as u64;
    trace_line(|l| {
        l.s("FREE id=");
        l.u(id);
    });
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
    fn uu(&mut self, v: u64) {
        if v == 0 {
            self.b(b'0');
            return;
        }
        let mut buf = [0u8; 20];
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
}

/// PLAN-770: every negative status return writes one `RT-FAIL code=<n>`
/// stderr line BEFORE the value crosses back, so the generated code's
/// explicit ExitProcess(70) path is attributable per failure class (the
/// range trap vs the allocation failure distinction the plan freezes).
fn rt_fail(code: i32) {
    trace_line(|l| {
        l.s("RT-FAIL code=");
        l.i(code);
    });
} // ---------------------------------------------------------------------------
  // ABI-A: scalars by value
  // ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// ABI-B: borrowed buffer (ptr, len) — read-only, never freed by the callee
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// ABI-C: owned buffer out (ptr return + *mut i32 length/status channel)
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn ac_rt_own_make(seed: i32, len: i32, out_len: *mut i32) -> *mut u8 {
    if len < 0 {
        rt_fail(ERR_LEN_NEGATIVE);
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
        Ok((p, _id)) if p.is_null() => {
            // zero length: valid, no allocation
            unsafe { write_volatile(out_len, 0) };
            core::ptr::null_mut()
        }
        Ok((p, id)) => {
            for i in 0..len as usize {
                unsafe { write_volatile(p.add(i), (seed.wrapping_add(i as i32)) as u8) };
            }
            trace_line(|l| {
                l.s("OWN-MAKE id=");
                l.u(id);
            });
            unsafe { write_volatile(out_len, len) };
            p
        }
        Err(code) => {
            rt_fail(code);
            unsafe { write_volatile(out_len, code) };
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn ac_rt_own_free(ptr: *mut u8) {
    owned_free(ptr);
}

#[no_mangle]
pub extern "system" fn ac_rt_own_clone(src: *const u8, len: i32, out_len: *mut i32) -> *mut u8 {
    if len < 0 {
        rt_fail(ERR_LEN_NEGATIVE);
        unsafe { write_volatile(out_len, ERR_LEN_NEGATIVE) };
        return core::ptr::null_mut();
    }
    trace_line(|l| {
        l.s("CLONE len=");
        l.i(len);
    });
    match try_alloc(len as u32) {
        Ok((p, _id)) if p.is_null() => {
            unsafe { write_volatile(out_len, 0) };
            core::ptr::null_mut()
        }
        Ok((p, id)) => {
            if !unsafe { copy_poll(p, src, len as usize) } {
                rt_fail(ERR_CANCELLED);
                unsafe { write_volatile(out_len, ERR_CANCELLED) };
                return core::ptr::null_mut();
            }
            trace_line(|l| {
                l.s("CLONE id=");
                l.u(id);
            });
            unsafe { write_volatile(out_len, len) };
            p
        }
        Err(code) => {
            rt_fail(code);
            unsafe { write_volatile(out_len, code) };
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn ac_rt_own_concat(
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
        rt_fail(ERR_LEN_NEGATIVE);
        unsafe { write_volatile(out_len, ERR_LEN_NEGATIVE) };
        return core::ptr::null_mut();
    }
    // Length arithmetic and the overflow rejection run BEFORE any buffer is
    // read or any allocation is attempted (probe-cases V05-6).
    let total = a_len as u64 + b_len as u64;
    if total > i32::MAX as u64 {
        rt_fail(ERR_LEN_OVERFLOW);
        unsafe { write_volatile(out_len, ERR_LEN_OVERFLOW) };
        return core::ptr::null_mut();
    }
    match try_alloc(total as u32) {
        Ok((p, _id)) if p.is_null() => {
            unsafe { write_volatile(out_len, 0) };
            core::ptr::null_mut()
        }
        Ok((p, id)) => {
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
            trace_line(|l| {
                l.s("CONCAT id=");
                l.u(id);
            });
            unsafe { write_volatile(out_len, total as i32) };
            p
        }
        Err(code) => {
            rt_fail(code);
            unsafe { write_volatile(out_len, code) };
            core::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "system" fn ac_rt_own_from_lit(
    lit_ptr: *const u8,
    lit_len: i32,
    out_len: *mut i32,
) -> *mut u8 {
    if lit_len < 0 {
        rt_fail(ERR_LEN_NEGATIVE);
        unsafe { write_volatile(out_len, ERR_LEN_NEGATIVE) };
        return core::ptr::null_mut();
    }
    trace_line(|l| {
        l.s("LIT len=");
        l.i(lit_len);
    });
    match try_alloc(lit_len as u32) {
        Ok((p, _id)) if p.is_null() => {
            unsafe { write_volatile(out_len, 0) };
            core::ptr::null_mut()
        }
        Ok((p, id)) => {
            if !unsafe { copy_poll(p, lit_ptr, lit_len as usize) } {
                rt_fail(ERR_CANCELLED);
                unsafe { write_volatile(out_len, ERR_CANCELLED) };
                return core::ptr::null_mut();
            }
            trace_line(|l| {
                l.s("LIT id=");
                l.u(id);
            });
            unsafe { write_volatile(out_len, lit_len) };
            p
        }
        Err(code) => {
            rt_fail(code);
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

// Nested record layout (review r1 P774-R1-04; decisions.md §D-05):
// Inner { x: i32 @0, y: i32 @4 } — size 8, align 4.
// Outer { inner: Inner @0 (inline, value semantics), flag: bool @8 } —
// size 12, align 4. Offsets are computed recursively per field alignment in
// declaration order; no pointers, assignment copies values.
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
pub extern "system" fn ac_rt_text_scalars(ptr: *const u8, len: i32) -> i32 {
    if len < 0 {
        rt_fail(ERR_LEN_NEGATIVE);
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
        Err(()) => {
            rt_fail(ERR_INVALID_UTF8);
            ERR_INVALID_UTF8
        }
    }
}

#[no_mangle]
pub extern "system" fn ac_rt_text_bytes(_ptr: *const u8, len: i32) -> i32 {
    if len < 0 {
        rt_fail(ERR_LEN_NEGATIVE);
        return ERR_LEN_NEGATIVE;
    }
    trace_line(|l| {
        l.s("TEXT-BYTES n=");
        l.i(len);
    });
    len
}

#[no_mangle]
pub extern "system" fn ac_rt_char_at(ptr: *const u8, len: i32, idx: i32) -> i32 {
    if idx < 0 {
        rt_fail(ERR_RANGE_OOB);
        return ERR_RANGE_OOB;
    }
    if len < 0 {
        rt_fail(ERR_LEN_NEGATIVE);
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
        Err(code) => {
            rt_fail(code);
            code
        }
    }
}

#[no_mangle]
pub extern "system" fn ac_rt_text_slice(
    ptr: *const u8,
    len: i32,
    start: i32,
    end: i32,
    out_len: *mut i32,
) -> *const u8 {
    if len < 0 {
        rt_fail(ERR_LEN_NEGATIVE);
        unsafe { write_volatile(out_len, ERR_LEN_NEGATIVE) };
        return core::ptr::null();
    }
    if start < 0 || end < 0 {
        rt_fail(ERR_RANGE_OOB);
        unsafe { write_volatile(out_len, ERR_RANGE_OOB) };
        return core::ptr::null();
    }
    if start > end {
        rt_fail(ERR_RANGE_REVERSED);
        unsafe { write_volatile(out_len, ERR_RANGE_REVERSED) };
        return core::ptr::null();
    }
    let scalars = match utf8_check(ptr, len as u32) {
        Ok(n) => n,
        Err(()) => {
            rt_fail(ERR_INVALID_UTF8);
            unsafe { write_volatile(out_len, ERR_INVALID_UTF8) };
            return core::ptr::null();
        }
    };
    if (end as u32) > scalars {
        rt_fail(ERR_RANGE_OOB);
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
pub extern "system" fn ac_rt_char_from_codepoint(cp: i32) -> i32 {
    trace_line(|l| {
        l.s("FROM-CP cp=");
        l.i(cp);
    });
    if cp < 0 || cp > 0x10FFFF || (0xD800..=0xDFFF).contains(&cp) {
        rt_fail(ERR_INVALID_SCALAR);
        ERR_INVALID_SCALAR
    } else {
        cp
    }
}

#[no_mangle]
pub extern "system" fn ac_rt_char_to_codepoint(ch: i32) -> i32 {
    if ch < 0 || ch > 0x10FFFF || (0xD800..=0xDFFF).contains(&ch) {
        rt_fail(ERR_INVALID_SCALAR);
        ERR_INVALID_SCALAR
    } else {
        ch
    }
}

// ---------------------------------------------------------------------------
// length boundary conversions (D-01)
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn ac_rt_len_from_i32(v: i32) -> i32 {
    if v < 0 {
        rt_fail(ERR_LEN_NEGATIVE);
        ERR_LEN_NEGATIVE
    } else {
        v
    }
}

#[no_mangle]
pub extern "system" fn ac_rt_len_to_i32(v: i64) -> i32 {
    if v < 0 || v > i32::MAX as i64 {
        rt_fail(ERR_LEN_TOO_LARGE);
        ERR_LEN_TOO_LARGE
    } else {
        v as i32
    }
}

// ---------------------------------------------------------------------------
// control / fault injection / summary / trap / hang
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn ac_rt_set_budget(bytes: i64) {
    // Frozen ruling (r1 P774-R1-01, amended r3 P774-R3-01): v >= 0 sets a
    // byte budget where 0 is a real ZERO-byte budget; a negative value
    // clears the ceiling (unlimited). The i64 channel is stored as u64 —
    // NEVER silently truncated to u32 (4 GiB budgets must stay 4 GiB).
    c().budget = if bytes < 0 { None } else { Some(bytes as u64) };
}

#[no_mangle]
pub extern "system" fn ac_rt_fail_nth_alloc(n: i32) {
    c().fail_nth = if n < 0 { 0 } else { n as u32 };
}

#[no_mangle]
pub extern "system" fn ac_rt_fail_ledger_nth(n: i32) {
    // r2 P774-R2-03: dedicated injection for the LEDGER's own metadata
    // allocations (first table + growth), independent of user AP-1..4.
    c().fail_ledger_nth = if n < 0 { 0 } else { n as u32 };
}

#[no_mangle]
pub extern "system" fn ac_rt_shim_error() -> i32 {
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
        l.uu(c.live_bytes);
        l.s(" peak=");
        l.uu(c.peak);
    });
    trace_line(|l| {
        l.s("FREESTAT free_attempts=");
        l.u(c.free_attempts);
        l.s(" frees_ok=");
        l.u(c.frees);
        l.s(" invalid=");
        l.u(c.invalid_frees);
    });
}

#[no_mangle]
pub extern "system" fn ac_rt_summary() {
    emit_summary();
}

#[no_mangle]
pub extern "system" fn ac_rt_check_invariant() -> i32 {
    emit_summary();
    let c = c();
    if c.allocs == c.frees && c.live == 0 && c.live_bytes == 0 && c.invalid_frees == 0 {
        0
    } else {
        1
    }
}

#[no_mangle]
pub extern "system" fn ac_rt_trap_after_alloc(len: i32) -> ! {
    let (p, _id) = match try_alloc(len.max(0) as u32) {
        Ok(pair) => pair,
        Err(_) => (core::ptr::null_mut(), 0),
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
pub extern "system" fn ac_rt_hang_after_alloc(len: i32) -> ! {
    let (p, _id) = match try_alloc(len.max(0) as u32) {
        Ok(pair) => pair,
        Err(_) => (core::ptr::null_mut(), 0),
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

// ---------------------------------------------------------------------------
// PLAN-770 additions: byte-exact text value equality (D-02: full UTF-8
// value equality; NUL is not a terminator; no normalization). Result is
// 1/0 on the i32 channel — never a negative status for valid buffers (a
// negative length is still a length-domain failure).
// ---------------------------------------------------------------------------

#[no_mangle]
pub extern "system" fn ac_rt_text_eq(
    a_ptr: *const u8,
    a_len: i32,
    b_ptr: *const u8,
    b_len: i32,
) -> i32 {
    if a_len < 0 || b_len < 0 {
        rt_fail(ERR_LEN_NEGATIVE);
        return ERR_LEN_NEGATIVE;
    }
    if a_len != b_len {
        return 0;
    }
    for i in 0..a_len as usize {
        let x = unsafe { read_volatile(a_ptr.add(i)) };
        let y = unsafe { read_volatile(b_ptr.add(i)) };
        if x != y {
            return 0;
        }
    }
    1
}

#[no_mangle]
pub extern "system" fn ac_rt_text_ne(
    a_ptr: *const u8,
    a_len: i32,
    b_ptr: *const u8,
    b_len: i32,
) -> i32 {
    let eq = ac_rt_text_eq(a_ptr, a_len, b_ptr, b_len);
    if eq < 0 {
        eq
    } else {
        1 - eq
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // The shim never panics by construction; if it ever does, die loudly
    // without unwinding across the ABI.
    loop {
        core::hint::spin_loop();
    }
}
