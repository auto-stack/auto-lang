//! PLAN-770 T-03 fault faces (plan V-05 runtime legs): AP-3/AP-4
//! request-point failures with a live owner, concat length overflow before
//! any allocation, budget rejection without an allocator touch, zero budget
//! + clear, ledger metadata failure rollback, and the RT-FAIL trace oracle
//! separating failure classes. Every PE ends with `ac_rt_check_invariant`
//! so the SUMMARY/FREESTAT observation channel closes the case.

#![cfg(windows)]
use ace_text_conformance::{
    acc, call_shim, const_i32, const_ptr, data_addr, expect_eq_i32, expect_null, load_i32,
    run_probe, summary_of,
};
use cranelift_codegen::ir::InstBuilder;

fn out_cell(
    fb: &mut cranelift_frontend::FunctionBuilder,
    fx: &ace_text_conformance::Fx,
) -> cranelift_codegen::ir::Value {
    data_addr(fb, fx, "outcell")
}

/// End-of-PE observation: emit SUMMARY/FREESTAT (value ignored where the
/// case does not assert balance through it).
fn observe(fb: &mut cranelift_frontend::FunctionBuilder, fx: &ace_text_conformance::Fx) {
    call_shim(fb, fx, "ac_rt_check_invariant", &[]);
}

#[test]
fn v05_ap4_literal_promotion_failure() {
    let out = run_probe("v05_ap4", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("lit_auto", b"Auto", false);
        cx.set_main(|fb, fx| {
            let one = const_i32(fb, 1);
            let four = const_i32(fb, 4);
            call_shim(fb, fx, "ac_rt_fail_nth_alloc", &[one]);
            let sp = out_cell(fb, fx);
            let lit = data_addr(fb, fx, "lit_auto");
            let ptr = call_shim(fb, fx, "ac_rt_own_from_lit", &[lit, four, sp])[0];
            let null_flag = expect_null(fb, ptr);
            let code = load_i32(fb, sp, 0);
            let code_flag = expect_eq_i32(fb, code, -5);
            let f = acc(fb, null_flag, code_flag);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert_eq!(summary_of(&out), (1, 0, 0, 0, 0, 0));
    assert!(out.stderr.contains("RT-FAIL code=-5"), "{}", out.stderr);
}

#[test]
fn v05_ap3_concat_failure_with_live_owner() {
    let out = run_probe("v05_ap3", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("lit_ab", b"Ab", false);
        cx.set_main(|fb, fx| {
            let two = const_i32(fb, 2);
            call_shim(fb, fx, "ac_rt_fail_nth_alloc", &[two]);
            let sp0 = out_cell(fb, fx);
            let lit = data_addr(fb, fx, "lit_ab");
            let owner = call_shim(fb, fx, "ac_rt_own_from_lit", &[lit, two, sp0])[0];
            let off8 = const_ptr(fb, 8);
            let sp1 = fb.ins().iadd(sp0, off8);
            let cat = call_shim(fb, fx, "ac_rt_own_concat", &[owner, two, owner, two, sp1])[0];
            let null_flag = expect_null(fb, cat);
            let code = load_i32(fb, sp1, 0);
            let code_flag = expect_eq_i32(fb, code, -5);
            // the surviving owner is reclaimed per contract; balance holds
            call_shim(fb, fx, "ac_rt_own_free", &[owner]);
            let inv = call_shim(fb, fx, "ac_rt_check_invariant", &[])[0];
            let inv_flag = expect_eq_i32(fb, inv, 0);
            let f = acc(fb, null_flag, code_flag);
            acc(fb, f, inv_flag)
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert_eq!(summary_of(&out), (2, 1, 1, 0, 0, 2));
    assert!(out.stderr.contains("RT-FAIL code=-5"));
}

#[test]
fn v05_concat_length_overflow_precedes_allocation() {
    let out = run_probe("v05_overflow", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.set_main(|fb, fx| {
            let sp = out_cell(fb, fx);
            let half = const_i32(fb, 0x4000_0000);
            let nullp = const_ptr(fb, 0);
            let cat = call_shim(fb, fx, "ac_rt_own_concat", &[nullp, half, nullp, half, sp])[0];
            let null_flag = expect_null(fb, cat);
            let code = load_i32(fb, sp, 0);
            let code_flag = expect_eq_i32(fb, code, -1);
            let f = acc(fb, null_flag, code_flag);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    // the check ran BEFORE any allocation: zero attempts
    assert_eq!(summary_of(&out), (0, 0, 0, 0, 0, 0));
    assert!(out.stderr.contains("RT-FAIL code=-1"));
}

#[test]
fn v05_budget_rejection_then_recovery() {
    let out = run_probe("v05_budget", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("lit_x", b"x", false);
        cx.set_main(|fb, fx| {
            let sp = out_cell(fb, fx);
            let lit = data_addr(fb, fx, "lit_x");
            let one = const_i32(fb, 1);
            let zero = const_ptr(fb, 0);
            // zero budget: any nonzero request rejects without touching the
            // allocator
            call_shim(fb, fx, "ac_rt_set_budget", &[zero]);
            let p1 = call_shim(fb, fx, "ac_rt_own_from_lit", &[lit, one, sp])[0];
            let n1 = expect_null(fb, p1);
            let c1 = load_i32(fb, sp, 0);
            let c1f = expect_eq_i32(fb, c1, -4);
            // negative clears the ceiling: the same request succeeds
            let negv = const_ptr(fb, -1);
            call_shim(fb, fx, "ac_rt_set_budget", &[negv]);
            let p2 = call_shim(fb, fx, "ac_rt_own_from_lit", &[lit, one, sp])[0];
            let l2 = load_i32(fb, sp, 0);
            let l2f = expect_eq_i32(fb, l2, 1);
            let f = acc(fb, n1, c1f);
            let f = acc(fb, f, l2f);
            call_shim(fb, fx, "ac_rt_own_free", &[p2]);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    // the rejected request made NO attempt; the recovered one did
    assert_eq!(summary_of(&out), (1, 1, 1, 0, 0, 1));
    assert!(out.stderr.contains("RT-FAIL code=-4"));
}

#[test]
fn v05_ledger_failure_rolls_back() {
    let out = run_probe("v05_ledger", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("lit_x", b"x", false);
        cx.set_main(|fb, fx| {
            let sp = out_cell(fb, fx);
            let lit = data_addr(fb, fx, "lit_x");
            let one = const_i32(fb, 1);
            let neg_one = const_i32(fb, -1);
            // first LEDGER table allocation fails: the user request rolls
            // back as env-oom; counters show the attempt but no allocation
            call_shim(fb, fx, "ac_rt_fail_ledger_nth", &[one]);
            let p1 = call_shim(fb, fx, "ac_rt_own_from_lit", &[lit, one, sp])[0];
            let n1 = expect_null(fb, p1);
            let c1 = load_i32(fb, sp, 0);
            let c1f = expect_eq_i32(fb, c1, -5);
            // clear the knob: the same request succeeds
            call_shim(fb, fx, "ac_rt_fail_ledger_nth", &[neg_one]);
            let p2 = call_shim(fb, fx, "ac_rt_own_from_lit", &[lit, one, sp])[0];
            let l2 = load_i32(fb, sp, 0);
            let l2f = expect_eq_i32(fb, l2, 1);
            let f = acc(fb, n1, c1f);
            let f = acc(fb, f, l2f);
            call_shim(fb, fx, "ac_rt_own_free", &[p2]);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert_eq!(summary_of(&out), (2, 1, 1, 0, 0, 1));
    assert!(out.stderr.contains("RT-FAIL code=-5"));
}

#[test]
fn v05_range_failure_classes_are_distinguishable() {
    let out = run_probe("v05_classes", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("lit_t1", b"A\xE4\xB8\xAD\xF0\x9F\x98\x80", false);
        cx.set_main(|fb, fx| {
            let lit = data_addr(fb, fx, "lit_t1");
            let sp = out_cell(fb, fx);
            let n8 = const_i32(fb, 8);
            let i2 = const_i32(fb, 2);
            let i1 = const_i32(fb, 1);
            let i0 = const_i32(fb, 0);
            let i4 = const_i32(fb, 4);
            let sur = const_i32(fb, 0xD800);
            call_shim(fb, fx, "ac_rt_text_slice", &[lit, n8, i2, i1, sp]);
            let rev = load_i32(fb, sp, 0);
            let rev_flag = expect_eq_i32(fb, rev, -9);
            call_shim(fb, fx, "ac_rt_text_slice", &[lit, n8, i0, i4, sp]);
            let oob = load_i32(fb, sp, 0);
            let oob_flag = expect_eq_i32(fb, oob, -10);
            let cp = call_shim(fb, fx, "ac_rt_char_from_codepoint", &[sur])[0];
            let cp_flag = expect_eq_i32(fb, cp, -8);
            let f = acc(fb, rev_flag, oob_flag);
            let f = acc(fb, f, cp_flag);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert!(out.stderr.contains("RT-FAIL code=-9"), "{}", out.stderr);
    assert!(out.stderr.contains("RT-FAIL code=-10"));
    assert!(out.stderr.contains("RT-FAIL code=-8"));
}
