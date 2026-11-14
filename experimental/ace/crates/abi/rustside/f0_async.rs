//! PLAN-967 T-08: the CONTROLLED REAL async provider (a hand-written
//! PROVIDER fixture — bindings and manifests are never hand-written; this
//! file IS the callee). One entry, one completion-callback lane, one
//! real OS thread: `f0_async_double` computes `byte_len * 2` off-thread
//! and completes through the `done` callback; the literal `cancel-me`
//! input declines to complete so the caller can exercise cancellation.
use std::ffi::c_void;

pub type F0Done = extern "C" fn(ctx: *mut c_void, status: i32, value: i32);

#[no_mangle]
pub extern "C" fn f0_async_double(
    data: *const u8,
    len: u64,
    done: F0Done,
    ctx: *mut c_void,
) -> i32 {
    let bytes = unsafe { std::slice::from_raw_parts(data, len as usize) }.to_vec();
    if bytes == b"cancel-me" {
        return -1;
    }
    // The pin contract carries the host context/callback across the worker
    // thread; usize tokens are plain Send values re-materialized inside.
    let raw = ctx as usize;
    let cb_token = done as usize;
    std::thread::spawn(move || {
        let cb: F0Done = unsafe { std::mem::transmute(cb_token) };
        std::thread::sleep(std::time::Duration::from_millis(10));
        cb(raw as *mut c_void, 0, bytes.len() as i32 * 2);
    });
    0
}
