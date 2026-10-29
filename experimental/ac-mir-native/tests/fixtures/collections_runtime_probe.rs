//! Native runtime-only probe. This is not evidence of source/MIR codegen.
#![no_std]
#![no_builtins]
#[repr(C)]
#[derive(Clone, Copy)]
struct Cell {
    bits: u64,
    owned: u64,
}
const EMPTY: Cell = Cell { bits: 0, owned: 0 };
extern "system" {
    fn ExitProcess(code: u32) -> !;
    fn mn_coll_frame_enter(frame: *mut u64, cells: *mut Cell, count: u64) -> i32;
    fn mn_coll_frame_leave(frame: *mut u64) -> i32;
    fn mn_coll_literal(dst: *mut Cell, bytes: *const u8, len: i32) -> i32;
    fn mn_coll_move(dst: *mut Cell, src: *mut Cell) -> i32;
    fn mn_coll_record_new(dst: *mut Cell, fields: u64) -> i32;
    fn mn_coll_record_put(record: u64, field: u64, src: *mut Cell) -> i32;
    fn mn_coll_field_get(dst: *mut Cell, record: u64, field: u64) -> i32;
    fn mn_coll_list_new(dst: *mut Cell, cap: u64) -> i32;
    fn mn_coll_list_insert(list: u64, index: i32, src: *mut Cell) -> i32;
    fn mn_coll_list_remove(dst: *mut Cell, list: u64, index: i32) -> i32;
    fn mn_coll_list_len(list: u64) -> i32;
    fn mn_coll_variant_pack(dst: *mut Cell, disc: u64, payload: *mut Cell) -> i32;
    fn mn_coll_variant_project(dst: *mut Cell, root: u64, disc: u64, moving: u64) -> i32;
    fn mn_coll_str_len(text: u64) -> i32;
    fn mn_coll_clone(dst: *mut Cell, src: *const Cell) -> i32;
    fn mn_coll_fail_nth(n: u64);
    fn ac_rt_fail_nth_alloc(n: i32);
    fn mn_coll_set_budget(n: u64);
    fn mn_coll_live_nodes() -> u64;
    fn mn_coll_live_bytes() -> u64;
    fn mn_coll_summary();
    fn mn_coll_abort(code: i32) -> !;
    fn ac_rt_own_from_lit(bytes: *const u8, len: i32, out: *mut i32) -> *mut u8;
    fn CreateThread(
        attrs: *mut u8,
        stack: usize,
        entry: unsafe extern "system" fn(*mut u8) -> u32,
        arg: *mut u8,
        flags: u32,
        id: *mut u32,
    ) -> isize;
    fn WaitForSingleObject(handle: isize, timeout: u32) -> u32;
    fn CloseHandle(handle: isize) -> i32;
}
unsafe fn check(ok: bool, code: u32) {
    if !ok {
        ExitProcess(code);
    }
}
unsafe fn ok(status: i32) {
    check(status == 0, 101);
}

unsafe extern "system" fn deep(_: *mut u8) -> u32 {
    let mut cells = [EMPTY; 3];
    let mut frame = [0u64; 4];
    ok(mn_coll_frame_enter(
        frame.as_mut_ptr(),
        cells.as_mut_ptr(),
        3,
    ));
    ok(mn_coll_literal(&mut cells[0], "中".as_ptr(), 3));
    for _ in 0..4096 {
        ok(mn_coll_variant_pack(&mut cells[1], 0, &mut cells[0]));
        ok(mn_coll_move(&mut cells[0], &mut cells[1]));
    }
    cells[2].bits = 70;
    let original = cells[0].bits;
    let live = mn_coll_live_bytes();
    mn_coll_fail_nth(4); // fail after the partial clone has attached a child
    check(mn_coll_clone(&mut cells[2], &cells[0]) == -25, 113);
    check(
        cells[0].bits == original
            && cells[2].bits == 70
            && cells[2].owned == 0
            && mn_coll_live_bytes() == live,
        114,
    );
    mn_coll_fail_nth(0);
    ok(mn_coll_clone(&mut cells[2], &cells[0]));
    check(cells[2].bits != original, 115);
    ok(mn_coll_frame_leave(frame.as_mut_ptr()));
    check(mn_coll_live_nodes() == 0 && mn_coll_live_bytes() == 0, 116);
    mn_coll_summary();
    0
}

#[cfg(mn_probe_trap)]
unsafe fn callee() -> ! {
    let mut cells = [EMPTY; 2];
    let mut frame = [0u64; 4];
    ok(mn_coll_frame_enter(
        frame.as_mut_ptr(),
        cells.as_mut_ptr(),
        2,
    ));
    ok(mn_coll_literal(&mut cells[0], b"callee".as_ptr(), 6));
    mn_coll_abort(-21)
}

#[no_mangle]
pub unsafe extern "system" fn ac_start() -> ! {
    if cfg!(mn_probe_deep) {
        // STACK_SIZE_PARAM_IS_A_RESERVATION: actual 64KiB reservation,
        // not just a commitment inside the default megabyte reservation.
        let thread = CreateThread(
            core::ptr::null_mut(),
            65536,
            deep,
            core::ptr::null_mut(),
            0x10000,
            core::ptr::null_mut(),
        );
        check(thread != 0, 117);
        check(WaitForSingleObject(thread, 4000) == 0, 118);
        CloseHandle(thread);
        ExitProcess(42);
    }
    let mut cells = [EMPTY; 12];
    let mut frame = [0u64; 4];
    ok(mn_coll_frame_enter(
        frame.as_mut_ptr(),
        cells.as_mut_ptr(),
        12,
    ));
    ok(mn_coll_literal(&mut cells[0], "中".as_ptr(), 3));
    ok(mn_coll_record_new(&mut cells[1], 2));
    ok(mn_coll_record_put(cells[1].bits, 0, &mut cells[0]));
    cells[2].bits = 41;
    ok(mn_coll_record_put(cells[1].bits, 1, &mut cells[2]));
    ok(mn_coll_variant_pack(&mut cells[3], 0, &mut cells[1]));
    ok(mn_coll_list_new(&mut cells[4], 1));
    ok(mn_coll_list_insert(cells[4].bits, 0, &mut cells[3]));
    #[cfg(mn_probe_trap)]
    {
        callee();
    }
    #[cfg(not(mn_probe_trap))]
    {
        ok(mn_coll_list_remove(&mut cells[5], cells[4].bits, 0));
        ok(mn_coll_variant_project(&mut cells[6], cells[5].bits, 0, 1));
        ok(mn_coll_field_get(&mut cells[7], cells[6].bits, 0));
        ok(mn_coll_field_get(&mut cells[8], cells[6].bits, 1));
        check(
            mn_coll_str_len(cells[7].bits) + cells[8].bits as i32 == 42,
            102,
        );
        ok(mn_coll_list_insert(cells[4].bits, 0, &mut cells[6]));
        ok(mn_coll_clone(&mut cells[9], &cells[4]));
        ok(mn_coll_list_remove(&mut cells[10], cells[9].bits, 0));
        ok(mn_coll_field_get(&mut cells[11], cells[10].bits, 0));
        check(
            cells[11].bits != cells[7].bits && mn_coll_str_len(cells[11].bits) == 1,
            103,
        );
        ok(mn_coll_clone(&mut cells[3], &cells[10]));
        let source_before = cells[3].bits;
        let bytes_before = mn_coll_live_bytes();
        mn_coll_fail_nth(1);
        check(
            mn_coll_list_insert(cells[4].bits, 1, &mut cells[3]) == -25,
            104,
        );
        check(
            cells[3].bits == source_before
                && mn_coll_list_len(cells[4].bits) == 1
                && mn_coll_live_bytes() == bytes_before,
            105,
        );
        mn_coll_fail_nth(1);
        let dest_before = cells[2].bits;
        check(
            mn_coll_variant_pack(&mut cells[2], 0, &mut cells[3]) == -25,
            106,
        );
        check(
            cells[3].bits == source_before
                && cells[2].bits == dest_before
                && mn_coll_live_bytes() == bytes_before,
            107,
        );
        mn_coll_fail_nth(0);
        // The delivered shim uses an absolute allocation ordinal (no reset):
        // literal + the two preceding text clones have consumed ordinals1..3.
        ac_rt_fail_nth_alloc(4);
        check(mn_coll_clone(&mut cells[2], &cells[7]) == -22, 108);
        check(
            cells[2].bits == dest_before && mn_coll_live_bytes() == bytes_before,
            109,
        );
        ac_rt_fail_nth_alloc(0);
        mn_coll_set_budget(0);
        check(mn_coll_literal(&mut cells[2], b"x".as_ptr(), 1) == -22, 110);
        check(
            cells[2].bits == dest_before && mn_coll_live_bytes() == bytes_before,
            111,
        );
        mn_coll_set_budget(8 << 20);
        if !cfg!(mn_probe_leak) {
            ok(mn_coll_frame_leave(frame.as_mut_ptr()));
            check(mn_coll_live_nodes() == 0 && mn_coll_live_bytes() == 0, 112);
        }
        if cfg!(mn_probe_text_leak) {
            let mut length = 0;
            let leaked = ac_rt_own_from_lit(b"lost-owner".as_ptr(), 10, &mut length);
            check(!leaked.is_null() && length == 10, 119);
        }
        mn_coll_summary();
        #[cfg(mn_probe_normal70)]
        {
            ExitProcess(70);
        }
        #[cfg(not(mn_probe_normal70))]
        {
            ExitProcess(42);
        }
    }
}
