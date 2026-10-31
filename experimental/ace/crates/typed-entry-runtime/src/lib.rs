//! PLAN-891 typed-entry transport helper — the FIXED machine side of the
//! `mn891.typed.frames/v1` one-shot channel. The generated typed wrapper
//! calls in here; this library never knows a fixture id, an expected value,
//! a signature shape or a request payload ahead of time — every binding is
//! read from the actual request frame at runtime.
//!
//! Channel freeze (mirror of `ace_native::typed_entry`):
//! - request frame (child stdin, little-endian):
//!   [0..4) magic "MNT1", [4..8) version 1, [8..12) signature id,
//!   [12..16) arg count, [16..24) nonce u64, [24..28) payload len,
//!   [28..32) reserved 0, payload, [.. +8) fnv1a64 checksum.
//! - response frame (child stdout, exactly 44 bytes, no trailing bytes):
//!   [0..4) magic "MNR1", [4..8) version 1, [8..12) signature id,
//!   [12..16) status (0 value / 1 trap / 2 resource / 3 protocol error),
//!   [16..24) nonce echo, [24..28) kind-or-reason, [28..36) value bits u64,
//!   [36..44) fnv1a64 checksum.
//!
//! No_std on purpose: the linked PE carries kernel32 only (no CRT). Like
//! the other native runtime libraries, this code is POINTER-DISCIPLINED —
//! no slice indexing, no `copy_from_slice`, no operation that can panic —
//! so no panicking core code unit is ever pulled out of the static library
//! (a single such reference would drag in fmt/memcpy/unwind machinery the
//! PE has no provider for). The process is single-threaded and one-shot
//! per request; the static request anchor is written once by
//! `mn_typed_read` before any consumer can run.

#![no_std]
#![no_builtins]

use core::ptr::{read_unaligned, read_volatile, write_unaligned, write_volatile};

const STD_INPUT_HANDLE: i32 = -10;
const STD_OUTPUT_HANDLE: i32 = -11;

/// Request magic "MNT1" (little-endian u32).
pub const REQ_MAGIC: u32 = 0x4D4E_5431;
/// Response magic "MNR1" (little-endian u32).
pub const RES_MAGIC: u32 = 0x4D4E_5231;
pub const VERSION: u32 = 1;
pub const HEADER_BYTES: usize = 32;
pub const HASH_BYTES: usize = 8;
/// Frame cap mirror of `ace_native::config::TYPED_MAX_FRAME_BYTES`.
pub const MAX_FRAME_BYTES: usize = 256 * 1024;

pub const STATUS_VALUE: u32 = 0;
pub const STATUS_TRAP: u32 = 1;
pub const STATUS_RESOURCE: u32 = 2;
pub const STATUS_ERROR: u32 = 3;

/// Protocol error reasons (response kind field under status 3).
pub const ERR_MAGIC: u32 = 1;
pub const ERR_VERSION: u32 = 2;
pub const ERR_LENGTH: u32 = 3;
pub const ERR_HASH: u32 = 4;
pub const ERR_SIG: u32 = 5;
pub const ERR_ARGC: u32 = 6;
pub const ERR_RESERVED: u32 = 7;
pub const ERR_IO: u32 = 8;

extern "system" {
    fn GetStdHandle(which: i32) -> isize;
    fn ReadFile(h: isize, p: *mut u8, n: u32, done: *mut u32, over: *mut u8) -> i32;
    fn WriteFile(h: isize, p: *const u8, n: u32, done: *mut u32, over: *mut u8) -> i32;
    fn ExitProcess(code: u32) -> !;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // The transport helper never panics; if it ever does, die loudly.
    loop {
        core::hint::spin_loop();
    }
}

// The one-shot request anchor, written exactly once by `mn_typed_read`.
static mut REQ_BUF: *mut u8 = core::ptr::null_mut();
static mut REQ_SIG: u32 = 0;
static mut REQ_NONCE: u64 = 0;
static mut REQ_LEN: u32 = 0;

fn anchor(buf: *mut u8, sig: u32, nonce: u64, len: u32) {
    unsafe {
        write_volatile(&mut REQ_BUF, buf);
        write_volatile(&mut REQ_SIG, sig);
        write_volatile(&mut REQ_NONCE, nonce);
        write_volatile(&mut REQ_LEN, len);
    }
}

fn anchored() -> (*mut u8, u32, u64, u32) {
    unsafe {
        (
            read_volatile(&REQ_BUF),
            read_volatile(&REQ_SIG),
            read_volatile(&REQ_NONCE),
            read_volatile(&REQ_LEN),
        )
    }
}

#[allow(static_mut_refs)]
fn read_exact(handle: isize, mut buf: *mut u8, mut n: usize) -> bool {
    while n > 0 {
        let chunk = if n > u32::MAX as usize {
            u32::MAX
        } else {
            n as u32
        };
        let mut done: u32 = 0;
        let ok = unsafe { ReadFile(handle, buf, chunk, &mut done, core::ptr::null_mut()) };
        if ok == 0 || done == 0 {
            return false; // IO failure or premature EOF
        }
        n -= done as usize;
        buf = unsafe { buf.add(done as usize) };
    }
    true
}

fn write_all(handle: isize, mut buf: *const u8, mut n: usize) -> bool {
    while n > 0 {
        let chunk = if n > u32::MAX as usize {
            u32::MAX
        } else {
            n as u32
        };
        let mut done: u32 = 0;
        let ok = unsafe { WriteFile(handle, buf, chunk, &mut done, core::ptr::null_mut()) };
        if ok == 0 || done == 0 {
            return false;
        }
        n -= done as usize;
        buf = unsafe { buf.add(done as usize) };
    }
    true
}

fn stdout_handle() -> isize {
    unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }
}

fn stdin_handle() -> isize {
    unsafe { GetStdHandle(STD_INPUT_HANDLE) }
}

/// Unaligned little-endian loads/stores through raw pointers — never
/// slice indexing (bounds checks pull panicking core code into the PE).
fn le32(buf: *const u8, at: usize) -> u32 {
    unsafe { read_unaligned(buf.add(at) as *const u32) }
}

fn le64(buf: *const u8, at: usize) -> u64 {
    unsafe { read_unaligned(buf.add(at) as *const u64) }
}

fn put32(buf: *mut u8, at: usize, v: u32) {
    unsafe { write_unaligned(buf.add(at) as *mut u32, v) }
}

fn put64(buf: *mut u8, at: usize, v: u64) {
    unsafe { write_unaligned(buf.add(at) as *mut u64, v) }
}

/// FNV-1a 64 over a raw byte range (same constants as
/// `ace_av_core::contract::fnv1a64`).
fn fnv1a64(buf: *const u8, n: usize) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut i = 0usize;
    while i < n {
        h ^= unsafe { *buf.add(i) } as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
        i += 1;
    }
    h
}

/// Emit one complete response frame and exit 0. `bits`/`kind` mean what
/// the status says (value bits / trap-or-error reason). Never returns.
fn respond(status: u32, kind: u32, bits: u64) -> ! {
    let (_, sig, nonce, _) = anchored();
    let mut frame = [0u8; 44];
    let p = frame.as_mut_ptr();
    put32(p, 0, RES_MAGIC);
    put32(p, 4, VERSION);
    put32(p, 8, sig);
    put32(p, 12, status);
    put64(p, 16, nonce);
    put32(p, 24, kind);
    put64(p, 28, bits);
    put64(p, 36, fnv1a64(p, 36));
    let out = stdout_handle();
    if !write_all(out, p, 44) {
        unsafe { ExitProcess(1) };
    }
    unsafe { ExitProcess(0) }
}

/// Report a protocol violation as a status-3 frame (reason codes above).
#[no_mangle]
pub extern "system" fn mn_typed_fail(reason: u32) -> ! {
    respond(STATUS_ERROR, reason, 0)
}

/// Read and validate ONE request frame from stdin into `buf` (`cap`
/// bytes). The frame's own header fixes the total length; the fnv1a64
/// checksum and the fixed fields are verified here. Signature identity,
/// arg count and the exact expected length are the WRAPPER's constants —
/// checked by [`mn_typed_check`]. Returns the frame length; any violation
/// writes a status-3 frame and exits (never returns an error).
#[no_mangle]
pub extern "system" fn mn_typed_read(buf: *mut u8, cap: u32) -> u32 {
    let stdin = stdin_handle();
    // Header first: the frame's length fields drive everything else.
    let mut header = [0u8; HEADER_BYTES];
    if !read_exact(stdin, header.as_mut_ptr(), HEADER_BYTES) {
        mn_typed_fail(ERR_IO);
    }
    let hp = header.as_ptr();
    if le32(hp, 0) != REQ_MAGIC {
        mn_typed_fail(ERR_MAGIC);
    }
    if le32(hp, 4) != VERSION {
        mn_typed_fail(ERR_VERSION);
    }
    if le32(hp, 28) != 0 {
        mn_typed_fail(ERR_RESERVED);
    }
    let payload_len = le32(hp, 24) as usize;
    let total = match HEADER_BYTES.checked_add(payload_len) {
        Some(t) => match t.checked_add(HASH_BYTES) {
            Some(v) => v,
            None => usize::MAX,
        },
        None => usize::MAX,
    };
    if total > MAX_FRAME_BYTES || total > cap as usize {
        mn_typed_fail(ERR_LENGTH);
    }
    if !read_exact(stdin, unsafe { buf.add(HEADER_BYTES) }, payload_len + HASH_BYTES) {
        mn_typed_fail(ERR_IO);
    }
    let mut i = 0usize;
    while i < HEADER_BYTES {
        unsafe { *buf.add(i) = header[i] };
        i += 1;
    }
    let stored = le64(buf, total - HASH_BYTES);
    if fnv1a64(buf, total - HASH_BYTES) != stored {
        mn_typed_fail(ERR_HASH);
    }
    let sig = le32(buf, 8);
    let nonce = le64(buf, 16);
    anchor(buf, sig, nonce, total as u32);
    total as u32
}

/// Wrapper-side identity check: the frame's signature id, arg count and
/// total length must equal the generated wrapper's frozen constants.
#[no_mangle]
pub extern "system" fn mn_typed_check(
    buf: *const u8,
    expect_sig: u32,
    expect_len: u32,
    expect_argc: u32,
) {
    if le32(buf, 8) != expect_sig {
        mn_typed_fail(ERR_SIG);
    }
    if le32(buf, 12) != expect_argc {
        mn_typed_fail(ERR_ARGC);
    }
    if le32(buf, 24) as usize + HEADER_BYTES + HASH_BYTES != expect_len as usize {
        mn_typed_fail(ERR_LENGTH);
    }
    let (_, _, _, len) = anchored();
    if len != expect_len {
        mn_typed_fail(ERR_LENGTH);
    }
}

/// Write the single value response (kind code 1..6 per the freeze) and
/// exit 0. `bits` carries the raw carrier value (I64-width; I32 kinds
/// occupy the low 32 bits).
#[no_mangle]
pub extern "system" fn mn_typed_write_value(kind: u32, bits: u64) -> ! {
    respond(STATUS_VALUE, kind, bits)
}

/// Report a semantic trap (checked-arithmetic failure) with a bounded
/// reason code; no value is delivered.
#[no_mangle]
pub extern "system" fn mn_typed_trap(reason: u32) -> ! {
    respond(STATUS_TRAP, reason, 0)
}

/// Report a resource failure with a bounded reason code; no value.
#[no_mangle]
pub extern "system" fn mn_typed_resource(reason: u32) -> ! {
    respond(STATUS_RESOURCE, reason, 0)
}
