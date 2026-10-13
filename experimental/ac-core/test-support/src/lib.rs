//! PLAN-741 minimal native host support library for the `hir.test.trace.v1`
//! capability (T-06).
//!
//! This library only provides the test-only trace intrinsics declared by the
//! 03-call-order document: `mark_a` records "a" and returns 1, `mark_b`
//! records "b" and returns 2. It does NOT implement any Auto function, does
//! not interpret HIR, and is not a production string/I/O/ABI component — it
//! exists so the native call-order probe can observe evaluation order.
//!
//! No_std on purpose: the produced PE links against kernel32 only (no CRT),
//! so the support code talks straight to the Windows API. Marks go to the
//! process stderr as single bytes, preserving call order in the stream.

#![no_std]
#![no_builtins]

const STD_ERROR_HANDLE: i32 = -12;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    // The trace support library never panics; if it ever does, die loudly.
    loop {
        core::hint::spin_loop();
    }
}

#[export_name = "hir.test.mark_a"]
pub extern "system" fn mark_a() -> i32 {
    emit(b'a');
    1
}

#[export_name = "hir.test.mark_b"]
pub extern "system" fn mark_b() -> i32 {
    emit(b'b');
    2
}

fn emit(byte: u8) {
    unsafe {
        let handle = GetStdHandle(STD_ERROR_HANDLE);
        let mut written: u32 = 0;
        WriteFile(
            handle,
            &byte as *const u8,
            1,
            &mut written as *mut u32,
            core::ptr::null_mut(),
        );
    }
}

extern "system" {
    fn GetStdHandle(n_std_handle: i32) -> isize;
    fn WriteFile(
        h_file: isize,
        lp_buffer: *const u8,
        n_number_of_bytes_to_write: u32,
        lp_number_of_bytes_written: *mut u32,
        lp_overlapped: *mut core::ffi::c_void,
    ) -> i32;
}
