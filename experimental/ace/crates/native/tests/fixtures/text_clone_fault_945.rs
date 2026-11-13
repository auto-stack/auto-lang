//! PLAN-945 V-04 native leg: the text-runtime owned-clone allocation-failure
//! contract at the real ABI the packages mn lane lowers to
//! (`ac_rt_own_clone` / `ac_rt_fail_nth_alloc`). Runtime-only probe: NOT
//! evidence of source/MIR codegen (the owned wrapper has no fault knobs —
//! that program-level boundary is named in PLAN-945 §5 T-01 D4).
#![no_std]
#![no_builtins]

extern "system" {
    fn ExitProcess(code: u32) -> !;
    fn ac_rt_own_make(seed: i32, len: i32, out_len: *mut i32) -> *mut u8;
    fn ac_rt_own_clone(src: *const u8, len: i32, out_len: *mut i32) -> *mut u8;
    fn ac_rt_own_free(ptr: *mut u8);
    fn ac_rt_fail_nth_alloc(n: i32);
    fn ac_rt_check_invariant() -> i32;
}

#[no_mangle]
pub extern "system" fn ac_start() -> i32 {
    let mut out_len: i32 = 0;
    let src = unsafe { ac_rt_own_make(1, 8, &mut out_len) };
    if src.is_null() || out_len != 8 {
        unsafe { ExitProcess(1) }
    }
    // Fault the NEXT allocation attempt (the seam counts invocations from
    // runtime start: own-make was #1, so #2 is the clone): the clone must
    // fail with NULL, the source buffer must stay intact (a faulted attempt
    // allocates nothing), and no panic may occur (typed failures only).
    unsafe { ac_rt_fail_nth_alloc(2) };
    let mut clone_len: i32 = 0;
    let bad = unsafe { ac_rt_own_clone(src, 8, &mut clone_len) };
    if !bad.is_null() {
        unsafe { ExitProcess(2) }
    }
    for i in 0..8usize {
        let b = unsafe { core::ptr::read_volatile(src.add(i)) };
        let expected = (1i32.wrapping_add(i as i32)) as u8;
        if b != expected {
            unsafe { ExitProcess(3) }
        }
    }
    // Clearing the seam: the clone succeeds, byte-identical, in an
    // INDEPENDENT buffer; both owners release and the ledger converges.
    unsafe { ac_rt_fail_nth_alloc(0) };
    let ok = unsafe { ac_rt_own_clone(src, 8, &mut clone_len) };
    if ok.is_null() || clone_len != 8 || ok == src {
        unsafe { ExitProcess(4) }
    }
    for i in 0..8usize {
        let a = unsafe { core::ptr::read_volatile(ok.add(i)) };
        let b = unsafe { core::ptr::read_volatile(src.add(i)) };
        if a != b {
            unsafe { ExitProcess(5) }
        }
    }
    unsafe {
        ac_rt_own_free(ok);
        ac_rt_own_free(src);
        if ac_rt_check_invariant() != 0 {
            ExitProcess(6)
        }
        ExitProcess(0)
    }
}
