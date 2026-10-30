//! V-03: lifetime and counting — borrow/own/move/clone across normal exits
//! (probe-cases.md §V-03; decisions.md §D-03/D-06).

#![cfg(windows)]

use ace_data_conformance as probe;
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

    // Control: free through BOTH slots — under the r1 counting contract the
    // first release succeeds (frees=1), the second is a ledger miss counted
    // in invalid_frees; check_invariant fails (invalid_frees != 0) and the
    // PE exits non-zero. No freed memory is read.
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
    assert_eq!(probe::summary_of(&bad), (1, 1, 1, 0, 0, 16));
    assert_eq!(probe::freestat_of(&bad), (2, 1, 1));
    assert!(bad.stderr.lines().any(|l| l == "FREE-INVALID"));
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

// ---- review r1 (P774-R1-03): LIFO order, nested exits, temporaries,
// move-in cleanup / return transfer, clone content independence ----

/// V03-6: LIFO scope exit — B freed before A; the resource-id FREE trace is
/// the order oracle. The FIFO control produces the distinguishable other
/// order, proving the channel detects wrong-order cleanup.
#[test]
fn v03_6_lifo_order_and_detection_control() {
    let lifo = probe::run_probe("v03-6-lifo", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let a = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let b = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            probe::call_shim(fb, fx, "p774_own_free", &[b]);
            probe::call_shim(fb, fx, "p774_own_free", &[a]);
            probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0)
        }));
    });
    probe::assert_pass(&lifo, &["FREE id=2", "FREE id=1"]);

    let fifo = probe::run_probe("v03-6-fifo", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let a = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let b = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            probe::call_shim(fb, fx, "p774_own_free", &[a]);
            probe::call_shim(fb, fx, "p774_own_free", &[b]);
            probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0)
        }));
    });
    // the control runs a DIFFERENT (non-LIFO) order and the trace shows it
    probe::assert_pass(&fifo, &["FREE id=1", "FREE id=2"]);
    assert_eq!(probe::summary_of(&lifo), (2, 2, 2, 0, 0, 32));
    assert_eq!(probe::summary_of(&fifo), (2, 2, 2, 0, 0, 32));
}

/// V03-7: nested loop exits — the continue path (i==1) and the break path
/// (i==2) each clean up their per-iteration allocation before leaving the
/// body; the natural path frees and re-enters. Iterations 0,1,2 run.
#[test]
fn v03_7_nested_break_continue_cleanup() {
    let out = probe::run_probe("v03-7", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let one = probe::const_i32(fb, 1);
            let two = probe::const_i32(fb, 2);
            let four = probe::const_i32(fb, 4);
            let i_var = fb.declare_var(types::I32);
            let zero_i = probe::const_i32(fb, 0);
            fb.def_var(i_var, zero_i);
            let header = fb.create_block();
            let body = fb.create_block();
            let mid = fb.create_block();
            let cont_blk = fb.create_block();
            let brk_body = fb.create_block();
            let nat_tail = fb.create_block();
            let exit = fb.create_block();
            fb.ins().jump(header, &[]);
            fb.switch_to_block(header);
            let i = fb.use_var(i_var);
            let done = fb.ins().icmp(IntCC::SignedGreaterThanOrEqual, i, four);
            fb.ins().brif(done, exit, &[], body, &[]);
            fb.switch_to_block(body);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let is_one = fb.ins().icmp(IntCC::Equal, i, one);
            fb.ins().brif(is_one, cont_blk, &[], mid, &[]);
            fb.switch_to_block(cont_blk);
            // continue path: free, increment, re-enter
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let next_c = fb.ins().iadd(i, one);
            fb.def_var(i_var, next_c);
            fb.ins().jump(header, &[]);
            fb.switch_to_block(mid);
            let is_two = fb.ins().icmp(IntCC::Equal, i, two);
            fb.ins().brif(is_two, brk_body, &[], nat_tail, &[]);
            fb.switch_to_block(brk_body);
            // break path: free, leave the loop
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            fb.ins().jump(exit, &[]);
            fb.switch_to_block(nat_tail);
            // natural path: free, increment, re-enter
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let next_n = fb.ins().iadd(i, one);
            fb.def_var(i_var, next_n);
            fb.ins().jump(header, &[]);
            fb.seal_block(header);
            fb.seal_block(body);
            fb.seal_block(mid);
            fb.seal_block(cont_blk);
            fb.seal_block(brk_body);
            fb.seal_block(nat_tail);
            fb.seal_block(exit);
            fb.switch_to_block(exit);
            probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0)
        }));
    });
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (3, 3, 3, 0, 0, 16));
}

/// V03-8: temporary lifetime — both uses of the temporary precede its
/// statement-end cleanup (trace order is the oracle).
#[test]
fn v03_8_temporary_statement_lifetime() {
    let out = probe::run_probe("v03-8", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let s1 = probe::call_shim(fb, fx, "p774_borrow_sum", &[p, len]).remove(0);
            let v1 = probe::expect_eq_i32(fb, s1, 232);
            let s2 = probe::call_shim(fb, fx, "p774_borrow_sum", &[p, len]).remove(0);
            let v2 = probe::expect_eq_i32(fb, s2, 232);
            // statement end: the temporary dies here, after both uses
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, v1, v2);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["BSUM len=16", "BSUM len=16", "FREE id=1"]);
    assert_eq!(probe::summary_of(&out), (1, 1, 1, 0, 0, 16));
}

/// V03-9: move-in param cleanup is the CALLEE's duty; a produced buffer
/// transfers back to the caller through an i64 cell (return transfer).
/// One PE covers both forms.
#[test]
fn v03_9_callee_cleanup_and_return_transfer() {
    let out = probe::run_probe("v03-9", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.data("pcell", &[0; 8], true);
        cx.func(
            "consume",
            &[cranelift_codegen::ir::types::I64],
            Box::new(|fb, fx, params| {
                probe::call_shim(fb, fx, "p774_own_free", &[params[0]]);
                probe::const_i32(fb, 0)
            }),
        );
        cx.func(
            "produce",
            &[cranelift_codegen::ir::types::I64],
            Box::new(|fb, fx, params| {
                let seed = probe::const_i32(fb, 7);
                let len = probe::const_i32(fb, 16);
                let p =
                    probe::call_shim(fb, fx, "p774_own_make", &[seed, len, params[0]]).remove(0);
                // ownership crosses the frame through the pointer cell
                probe::store_i64(fb, params[0], 0, p);
                probe::const_i32(fb, 0)
            }),
        );
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let cell = probe::data_addr(fb, fx, "pcell");
            let seed = probe::const_i32(fb, 7);
            let len = probe::const_i32(fb, 16);
            // (1) move-in: the callee frees
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, len, outp]).remove(0);
            let r1 = probe::call_shim(fb, fx, "consume", &[p]).remove(0);
            let v1 = probe::expect_eq_i32(fb, r1, 0);
            // (2) return transfer: the callee produces, the caller frees
            let r2 = probe::call_shim(fb, fx, "produce", &[cell]).remove(0);
            let v2 = probe::expect_eq_i32(fb, r2, 0);
            let q = probe::load_i64(fb, cell, 0);
            let vq = probe::expect_non_null(fb, q);
            probe::call_shim(fb, fx, "p774_own_free", &[q]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, v1, v2);
            let v = probe::acc(fb, v, vq);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &[]);
    assert_eq!(probe::summary_of(&out), (2, 2, 2, 0, 0, 16));
}

/// V03-10: clone content independence — after the ORIGINAL is freed, the
/// clone still holds the copied bytes.
#[test]
fn v03_10_clone_content_after_owner_freed() {
    let out = probe::run_probe("v03-10", |cx| {
        cx.data("lit", b"Auto", false);
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let lp = probe::data_addr(fb, fx, "lit");
            let outp = probe::data_addr(fb, fx, "outlen");
            let four = probe::const_i32(fb, 4);
            let p = probe::call_shim(fb, fx, "p774_own_from_lit", &[lp, four, outp]).remove(0);
            let q = probe::call_shim(fb, fx, "p774_own_clone", &[p, four, outp]).remove(0);
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let sum = probe::call_shim(fb, fx, "p774_borrow_sum", &[q, four]).remove(0);
            let vs = probe::expect_eq_i32(fb, sum, 409);
            probe::call_shim(fb, fx, "p774_own_free", &[q]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            probe::acc(fb, vs, inv)
        }));
    });
    probe::assert_pass(&out, &["LIT id=1", "CLONE id=2", "FREE id=1"]);
    assert_eq!(probe::summary_of(&out), (2, 2, 2, 0, 0, 8));
}
