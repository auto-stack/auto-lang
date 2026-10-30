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
    fn mn_coll_frame_open(frame: *mut u64, count: u64) -> *mut Cell;
    fn mn_coll_literal(dst: *mut Cell, bytes: *const u8, len: i32) -> i32;
    fn mn_coll_move(dst: *mut Cell, src: *mut Cell) -> i32;
    fn mn_coll_record_new(dst: *mut Cell, fields: u64) -> i32;
    fn mn_coll_record_put(record: u64, field: u64, src: *mut Cell) -> i32;
    fn mn_coll_field_get(dst: *mut Cell, record: u64, field: u64) -> i32;
    fn mn_coll_list_new(dst: *mut Cell, cap: u64) -> i32;
    fn mn_coll_list_insert(list: u64, index: i32, src: *mut Cell) -> i32;
    fn mn_coll_list_remove(dst: *mut Cell, list: u64, index: i32) -> i32;
    fn mn_coll_list_set(list: u64, index: i32, value: *mut Cell) -> i32;
    fn mn_coll_list_peek(dst: *mut Cell, list: u64, index: i32) -> i32;
    fn mn_coll_list_len(list: u64) -> i32;
    fn mn_coll_variant_pack(dst: *mut Cell, disc: u64, payload: *mut Cell) -> i32;
    fn mn_coll_variant_disc(bits: u64) -> i32;
    fn mn_coll_variant_project(dst: *mut Cell, root: u64, disc: u64, moving: u64) -> i32;
    fn mn_coll_str_len(text: u64) -> i32;
    fn mn_coll_clone(dst: *mut Cell, src: *const Cell) -> i32;
    fn mn_coll_fail_nth(n: u64);
    fn ac_rt_fail_nth_alloc(n: i32);
    fn mn_coll_set_budget(n: u64);
    fn mn_coll_arena_limit(n: u64) -> i32;
    fn mn_coll_arena_committed() -> u64;
    fn mn_coll_cancel_after_work(n: u64);
    fn mn_coll_drop(cell: *mut Cell);
    fn ac_rt_fail_ledger_nth(n: i32);
    fn ac_rt_own_free(bytes: *mut u8);
    fn ac_rt_release_idle_metadata() -> i32;
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
    if cfg!(mn_probe_mutations) {
        let mut frame = [0u64; 4];
        let mut cells = [EMPTY; 10];
        ok(mn_coll_frame_enter(
            frame.as_mut_ptr(),
            cells.as_mut_ptr(),
            10,
        ));
        ok(mn_coll_literal(&mut cells[0], b"first".as_ptr(), 5));
        let first = cells[0].bits;
        ok(mn_coll_list_new(&mut cells[1], 1));
        ok(mn_coll_list_insert(cells[1].bits, 0, &mut cells[0]));
        ok(mn_coll_literal(&mut cells[2], b"second".as_ptr(), 6));
        let second = cells[2].bits;
        let before = mn_coll_live_bytes();
        check(
            mn_coll_list_set(cells[1].bits, 1, &mut cells[2]) == -21,
            153,
        );
        ok(mn_coll_list_peek(&mut cells[3], cells[1].bits, 0));
        check(
            cells[3].bits == first
                && cells[2].bits == second
                && cells[2].owned == 1
                && mn_coll_live_bytes() == before,
            154,
        );
        ok(mn_coll_list_set(cells[1].bits, 0, &mut cells[2]));
        ok(mn_coll_list_peek(&mut cells[3], cells[1].bits, 0));
        check(
            cells[3].bits == second && cells[2].owned == 0 && mn_coll_str_len(cells[3].bits) == 6,
            155,
        );
        ok(mn_coll_literal(&mut cells[4], b"live".as_ptr(), 4));
        ok(mn_coll_variant_pack(&mut cells[5], 0, &mut cells[4]));
        let old = cells[5].bits;
        ok(mn_coll_literal(&mut cells[6], b"new".as_ptr(), 3));
        let incoming = cells[6].bits;
        let before = mn_coll_live_bytes();
        mn_coll_fail_nth(1);
        check(
            mn_coll_variant_pack(&mut cells[5], 1, &mut cells[6]) == -25,
            156,
        );
        check(
            cells[5].bits == old
                && cells[6].bits == incoming
                && cells[6].owned == 1
                && mn_coll_live_bytes() == before,
            157,
        );
        ok(mn_coll_variant_project(&mut cells[7], cells[5].bits, 0, 0));
        check(mn_coll_str_len(cells[7].bits) == 4, 158);
        mn_coll_drop(&mut cells[7]);
        mn_coll_fail_nth(0);
        ok(mn_coll_variant_pack(&mut cells[5], 1, &mut cells[6]));
        check(
            cells[5].bits != old && mn_coll_variant_disc(cells[5].bits) == 1 && cells[6].owned == 0,
            159,
        );
        cells[8].bits = 70;
        check(
            mn_coll_variant_project(&mut cells[8], cells[5].bits, 0, 1) == -24
                && cells[8].bits == 70,
            160,
        );
        ok(mn_coll_variant_project(&mut cells[9], cells[5].bits, 1, 1));
        check(mn_coll_str_len(cells[9].bits) == 3, 161);
        check(
            mn_coll_variant_project(&mut cells[8], cells[5].bits, 1, 1) == -24
                && cells[8].bits == 70,
            162,
        );
        ok(mn_coll_frame_leave(frame.as_mut_ptr()));
        mn_coll_summary();
        ExitProcess(42);
    }
    if cfg!(mn_probe_frames) {
        let mut outer = [0u64; 4];
        let mut inner = [0u64; 4];
        let mut cells = [EMPTY; 1];
        ok(mn_coll_frame_enter(
            outer.as_mut_ptr(),
            cells.as_mut_ptr(),
            1,
        ));
        let before = mn_coll_live_bytes();
        // Requested bytes fit, but32MiB of pages plus real frame headers do not.
        check(
            mn_coll_frame_open(inner.as_mut_ptr(), (32 << 20) / 16 - 6).is_null(),
            145,
        );
        check(
            mn_coll_live_bytes() == before && mn_coll_arena_committed() == 0,
            146,
        );
        mn_coll_fail_nth(1);
        check(mn_coll_frame_open(inner.as_mut_ptr(), 4).is_null(), 147);
        check(mn_coll_live_bytes() == before, 148);
        mn_coll_fail_nth(0);
        let large = mn_coll_frame_open(inner.as_mut_ptr(), ((32 << 20) - 4096) / 16 - 4);
        check(!large.is_null(), 149);
        check(mn_coll_arena_committed() == (32 << 20) - 4096, 150);
        ok(mn_coll_frame_leave(inner.as_mut_ptr()));
        check(
            mn_coll_live_bytes() == before && mn_coll_arena_committed() == 0,
            151,
        );
        let empty = mn_coll_frame_open(inner.as_mut_ptr(), 0);
        check(!empty.is_null(), 152);
        ok(mn_coll_frame_leave(inner.as_mut_ptr()));
        ok(mn_coll_frame_leave(outer.as_mut_ptr()));
        mn_coll_summary();
        ExitProcess(42);
    }
    if cfg!(mn_probe_allocator_switch) {
        let mut cells = [EMPTY; 1];
        let mut frame = [0u64; 4];
        ok(mn_coll_frame_enter(
            frame.as_mut_ptr(),
            cells.as_mut_ptr(),
            1,
        ));
        let mut length = 0;
        let old = ac_rt_own_from_lit(b"old-heap".as_ptr(), 8, &mut length);
        check(!old.is_null() && length == 8, 140);
        check(mn_coll_literal(&mut cells[0], b"x".as_ptr(), 1) == -23, 141);
        ac_rt_own_free(old);
        // Idle retained metadata still belongs to the original allocator.
        check(mn_coll_literal(&mut cells[0], b"x".as_ptr(), 1) == -23, 142);
        ok(ac_rt_release_idle_metadata());
        ok(mn_coll_literal(&mut cells[0], b"x".as_ptr(), 1));
        ok(mn_coll_frame_leave(frame.as_mut_ptr()));
        mn_coll_summary();
        ExitProcess(42);
    }
    if cfg!(mn_probe_guest) {
        static TEXT: [u8; 8 << 20] = [b'x'; 8 << 20];
        let mut cells = [EMPTY; 2];
        let mut frame = [0u64; 4];
        ok(mn_coll_frame_enter(
            frame.as_mut_ptr(),
            cells.as_mut_ptr(),
            2,
        ));
        let before = mn_coll_live_bytes();
        check(
            mn_coll_literal(&mut cells[0], TEXT.as_ptr(), (TEXT.len() + 1) as i32) == -22,
            136,
        );
        check(cells[0].owned == 0 && mn_coll_live_bytes() == before, 137);
        ok(mn_coll_literal(
            &mut cells[0],
            TEXT.as_ptr(),
            TEXT.len() as i32,
        ));
        let before = mn_coll_live_bytes();
        cells[1].bits = 70;
        check(mn_coll_clone(&mut cells[1], &cells[0]) == -22, 138);
        check(
            cells[1].bits == 70 && cells[1].owned == 0 && mn_coll_live_bytes() == before,
            139,
        );
        ok(mn_coll_frame_leave(frame.as_mut_ptr()));
        mn_coll_summary();
        ExitProcess(42);
    }
    if cfg!(mn_probe_arena) {
        let mut cells = [EMPTY; 5];
        let mut frame = [0u64; 4];
        ok(mn_coll_frame_enter(
            frame.as_mut_ptr(),
            cells.as_mut_ptr(),
            5,
        ));
        ok(mn_coll_arena_limit(4096));
        check(mn_coll_arena_limit(8192) == -23, 143); // cannot reopen a tightened ceiling
        ok(mn_coll_list_new(&mut cells[0], 80));
        ok(mn_coll_list_new(&mut cells[1], 80));
        ok(mn_coll_list_new(&mut cells[2], 16));
        check(mn_coll_arena_limit(0) == -23, 144); // cannot erase live committed charge
        let before = mn_coll_live_bytes();
        check(mn_coll_list_new(&mut cells[3], 256) == -22, 120);
        check(
            mn_coll_live_bytes() == before
                && cells[3].owned == 0
                && mn_coll_arena_committed() == 4096,
            121,
        );
        mn_coll_drop(&mut cells[0]);
        ok(mn_coll_list_new(&mut cells[3], 70)); // reuse a non-tail fragment
        mn_coll_drop(&mut cells[1]);
        mn_coll_drop(&mut cells[3]); // coalesce preceding and following blocks
        ok(mn_coll_list_new(&mut cells[4], 150));
        ok(mn_coll_frame_leave(frame.as_mut_ptr()));
        check(mn_coll_arena_committed() == 0, 122);
        mn_coll_summary();
        ExitProcess(42);
    }
    if cfg!(mn_probe_cancel) || cfg!(mn_probe_metadata) || cfg!(mn_probe_limits) {
        let mut cells = [EMPTY; 20];
        let mut frame = [0u64; 4];
        ok(mn_coll_frame_enter(
            frame.as_mut_ptr(),
            cells.as_mut_ptr(),
            20,
        ));
        if cfg!(mn_probe_metadata) {
            let baseline = mn_coll_live_bytes();
            ac_rt_fail_ledger_nth(1);
            check(mn_coll_literal(&mut cells[0], b"x".as_ptr(), 1) == -22, 123);
            check(mn_coll_live_bytes() == baseline && cells[0].owned == 0, 124);
            ac_rt_fail_ledger_nth(3); // initial success is ordinal2, grow is ordinal3
            for i in 0..16 {
                ok(mn_coll_literal(cells.as_mut_ptr().add(i), b"x".as_ptr(), 1));
            }
            let before = mn_coll_live_bytes();
            cells[16].bits = 70;
            check(
                mn_coll_literal(&mut cells[16], b"y".as_ptr(), 1) == -22,
                125,
            );
            check(
                cells[16].bits == 70 && cells[16].owned == 0 && mn_coll_live_bytes() == before,
                126,
            );
            ac_rt_fail_ledger_nth(0);
            ok(mn_coll_literal(&mut cells[16], b"z".as_ptr(), 1));
        } else if cfg!(mn_probe_cancel) {
            static TEXT: [u8; 32768] = [b'x'; 32768];
            ok(mn_coll_literal(
                &mut cells[0],
                TEXT.as_ptr(),
                TEXT.len() as i32,
            ));
            cells[1].bits = 70;
            let original = cells[0].bits;
            let before = mn_coll_live_bytes();
            mn_coll_cancel_after_work(8192);
            check(mn_coll_clone(&mut cells[1], &cells[0]) == -26, 127);
            check(
                cells[0].bits == original
                    && cells[1].bits == 70
                    && cells[1].owned == 0
                    && mn_coll_live_bytes() == before,
                128,
            );
            mn_coll_cancel_after_work(u64::MAX);
            ok(mn_coll_list_new(&mut cells[2], 128));
            for i in 0..128 {
                cells[3].bits = i;
                ok(mn_coll_list_insert(cells[2].bits, i as i32, &mut cells[3]));
            }
            let before = mn_coll_live_bytes();
            mn_coll_cancel_after_work(20);
            check(mn_coll_clone(&mut cells[1], &cells[2]) == -26, 129);
            check(
                cells[1].bits == 70
                    && cells[1].owned == 0
                    && mn_coll_list_len(cells[2].bits) == 128
                    && mn_coll_live_bytes() == before,
                130,
            );
            mn_coll_cancel_after_work(0);
            let value = cells[3].bits;
            check(
                mn_coll_list_insert(cells[2].bits, 0, &mut cells[3]) == -26,
                131,
            );
            check(
                cells[3].bits == value && mn_coll_list_len(cells[2].bits) == 128,
                132,
            );
            mn_coll_cancel_after_work(u64::MAX);
            ok(mn_coll_clone(&mut cells[1], &cells[0]));
        } else {
            ok(mn_coll_list_new(&mut cells[0], 65536));
            check(mn_coll_list_new(&mut cells[1], 65537) == -22, 133);
            for i in 0..65536 {
                cells[2].bits = i;
                ok(mn_coll_list_insert(cells[0].bits, i as i32, &mut cells[2]));
            }
            let before = mn_coll_live_bytes();
            cells[2].bits = 70;
            check(
                mn_coll_list_insert(cells[0].bits, 65536, &mut cells[2]) == -22,
                134,
            );
            check(
                cells[2].bits == 70
                    && mn_coll_live_bytes() == before
                    && mn_coll_list_len(cells[0].bits) == 65536,
                135,
            );
        }
        ok(mn_coll_frame_leave(frame.as_mut_ptr()));
        mn_coll_summary();
        ExitProcess(42);
    }
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
