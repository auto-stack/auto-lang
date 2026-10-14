//! PLAN-770 trace-marks object: the two `hir.test.trace.v1` intrinsics as a
//! PLAIN COFF object (`rustc --crate-type=lib --emit=obj`), not a Rust
//! staticlib. Both the scalar trace support library and the text runtime
//! are Rust no_std staticlibs and each carries `__rustc::rust_begin_unwind`
//! (the panic entry), so the two staticlibs can never co-link into one PE
//! (rust-lld duplicate symbol). A `--emit=obj` library artifact carries no
//! panic entry at all — linking it next to the text runtime restores the
//! plan's frozen V-07 oracle (mark_a/mark_b wrapped text ops in one PE).
//!
//! Behavior is byte-identical to `test-support/src/lib.rs`: mark_a writes
//! "a" to stderr and returns 1, mark_b writes "b" and returns 2. The
//! scalar suites keep using the original staticlib unchanged (zero
//! regression); only the text x-trace co-link face consumes this object.

#![no_std]

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
            &byte,
            1,
            &mut written as *mut u32,
            core::ptr::null_mut(),
        );
    }
}
