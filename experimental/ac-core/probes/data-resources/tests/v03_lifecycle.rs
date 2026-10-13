//! V-03: lifetime and counting — borrow/own/move/clone across normal exits
//! (probe-cases.md §V-03; decisions.md §D-03/D-06).

#![cfg(windows)]

use ac_data_probe_774 as probe;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::InstBuilder;

/// V03-1: borrowed reads never free the owner.
#[test]
fn v03_1_borrow_keeps_owner() {
    let out = probe::run_probe("v03-1", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let ptr = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let sum = probe::call_shim(fb, fx, "p774_borrow_sum", &[ptr, len]).remove(0);
            let v1 = probe::expect_eq_i32(fb, sum, 232);
            let idx = probe::const_i32(fb, 15);
            let byte = probe::call_shim(fb, fx, "p774_borrow_read", &[ptr, len, idx]).remove(0);
            let v2 = probe::expect_eq_i32(fb, byte, 22);
            probe::call_shim(fb, fx, "p774_own_free", &[ptr]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v3 = probe::acc(fb, v1, v2);
            probe::acc(fb, v3, inv)
        }));
    });
    probe::assert_pass(&out, &["OWN-MAKE seed=7 len=16"]);
    assert_eq!(probe::summary_of(&out), (1, 1, 1, 0, 0, 16));
}

/// V03-2: one move, exactly one cleanup + V03-2n double-cleanup detection
/// control (must FAIL the PE exit and surface frees>allocs).
#[test]
fn v03_2_move_single_free_and_detection_control() {
    let out = probe::run_probe("v03-2", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            // "move": the pointer value travels to a second slot (a Cranelift
            // variable); the source slot is dead by model rule.
            let q_var = fb.declare_var(types::I64);
            fb.def_var(q_var, p);
            let q = fb.use_var(q_var);
            probe::call_shim(fb, fx, "p774_own_free", &[q]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            inv
        }));
    });
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (1, 1, 1, 0, 0, 16));

    // Control: free through BOTH slots — the counters expose frees=2>allocs=1
    // and the PE must exit non-zero (detection capability evidence).
    let bad = probe::run_probe("v03-2n", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let q_var = fb.declare_var(types::I64);
            fb.def_var(q_var, p);
            let q = fb.use_var(q_var);
            probe::call_shim(fb, fx, "p774_own_free", &[q]);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            inv
        }));
    });
    assert_eq!(bad.code, Some(1), "double free must fail the invariant");
    assert_eq!(probe::summary_of(&bad), (1, 1, 2, 0, 0, 16));
}

/// V03-3: clone is an independent allocation (distinct pointer, own free).
#[test]
fn v03_3_clone_independent() {
    let out = probe::run_probe("v03-3", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let q = probe::call_shim(fb, fx, "p774_own_clone", &[p, len, outp]).remove(0);
            // violation when p == q (not independent)
            let same = fb.ins().icmp(IntCC::Equal, p, q);
            let vsame = fb.ins().uextend(types::I32, same);
            let nn = probe::expect_non_null(fb, q);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            probe::call_shim(fb, fx, "p774_own_free", &[q]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v1 = probe::acc(fb, vsame, nn);
            probe::acc(fb, v1, inv)
        }));
    });
    probe::assert_pass(&out, &["CLONE len=16"]);
    assert_eq!(probe::summary_of(&out), (2, 2, 2, 0, 0, 32));
}

/// V03-4: branch early-exit — both the early-return path and the full path
/// end with live=0 (exit 0 on both PEs).
#[test]
fn v03_4_branch_early_exit() {
    const CASES: &[(&str, i64)] = &[("v03-4-early", 1), ("v03-4-full", 0)];
    for &(case, seed_val) in CASES {
        let out = probe::run_probe(case, |cx| {
            cx.data("outlen", &[0; 4], true);
            cx.set_main(Box::new(move |fb, fx| {
                let outp = probe::data_addr(fb, fx, "outlen");
                let seed = probe::const_i32(fb, seed_val);
                let seven = probe::const_i32(fb, 7);
                let len = probe::const_i32(fb, 16);
                let p = probe::call_shim(fb, fx, "p774_own_make", &[seven, len, outp]).remove(0);
                let zero = probe::const_i32(fb, 0);
                let cond = fb.ins().icmp(IntCC::NotEqual, seed, zero);
                let early = fb.create_block();
                let full = fb.create_block();
                fb.ins().brif(cond, early, &[], full, &[]);
                fb.switch_to_block(early);
                fb.seal_block(early);
                fb.seal_block(full);
                probe::call_shim(fb, fx, "p774_own_free", &[p]);
                let inv1 = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
                fb.ins().return_(&[inv1]);
                fb.switch_to_block(full);
                let q = probe::call_shim(fb, fx, "p774_own_clone", &[p, len, outp]).remove(0);
                probe::call_shim(fb, fx, "p774_own_free", &[p]);
                probe::call_shim(fb, fx, "p774_own_free", &[q]);
                let inv2 = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
                inv2
            }));
        });
        probe::assert_pass(&out, &[]);
        let s = probe::summary_of(&out);
        if seed_val == 1 {
            assert_eq!(s, (1, 1, 1, 0, 0, 16));
        } else {
            assert_eq!(s, (2, 2, 2, 0, 0, 32));
        }
    }
}

/// V03-5: loop exits — per-iteration make/free is balanced across 3 rounds.
#[test]
fn v03_5_loop_exit() {
    let out = probe::run_probe("v03-5", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let i_var = fb.declare_var(types::I32);
            let zero = probe::const_i32(fb, 0);
            fb.def_var(i_var, zero);
            let header = fb.create_block();
            let body = fb.create_block();
            let exit = fb.create_block();
            fb.ins().jump(header, &[]);
            fb.switch_to_block(header);
            let i = fb.use_var(i_var);
            let three = probe::const_i32(fb, 3);
            let done = fb.ins().icmp(IntCC::SignedGreaterThanOrEqual, i, three);
            fb.ins().brif(done, exit, &[], body, &[]);
            fb.switch_to_block(body);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let one = probe::const_i32(fb, 1);
            let next = fb.ins().iadd(i, one);
            fb.def_var(i_var, next);
            fb.ins().jump(header, &[]);
            fb.seal_block(header);
            fb.seal_block(body);
            fb.seal_block(exit);
            fb.switch_to_block(exit);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            inv
        }));
    });
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (3, 3, 3, 0, 0, 16));
}
