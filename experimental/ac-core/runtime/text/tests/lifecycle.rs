//! PLAN-770 T-03 lifecycle faces (plan V-04 runtime legs): borrow-not-freed,
//! clone independence with content control, LIFO free order, zero-length
//! no-attempt, and cross-frame owned return. Every PE ends with
//! `ac_rt_check_invariant` so SUMMARY/FREESTAT close the case.

#![cfg(windows)]
use ac_text_runtime_tests::{
    acc, call_shim, const_i32, const_ptr, data_addr, expect_eq_i32, expect_non_null, expect_null,
    load_i32, run_probe, store_i64, summary_of,
};
use cranelift_codegen::ir::InstBuilder;

/// Make an owned buffer from a rodata literal; returns (ptr, len).
fn own_from_lit(
    fb: &mut cranelift_frontend::FunctionBuilder,
    fx: &ac_text_runtime_tests::Fx,
    lit: &str,
    lit_len: i32,
) -> (cranelift_codegen::ir::Value, cranelift_codegen::ir::Value) {
    let p = data_addr(fb, fx, lit);
    let n = const_i32(fb, lit_len as i64);
    let sp = data_addr(fb, fx, "outcell");
    let ptr = call_shim(fb, fx, "ac_rt_own_from_lit", &[p, n, sp])[0];
    let len = load_i32(fb, sp, 0);
    (ptr, len)
}

fn observe(fb: &mut cranelift_frontend::FunctionBuilder, fx: &ac_text_runtime_tests::Fx) {
    call_shim(fb, fx, "ac_rt_check_invariant", &[]);
}

#[test]
fn v04_borrow_view_does_not_free_owner() {
    let out = run_probe("v04_borrow", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("lit_auto", b"Auto", false);
        cx.set_main(|fb, fx| {
            let (ptr, len) = own_from_lit(fb, fx, "lit_auto", 4);
            let null_flag = expect_non_null(fb, ptr);
            // read-only views (scalars/bytes) never free
            let sc = call_shim(fb, fx, "ac_rt_text_scalars", &[ptr, len])[0];
            let sc_flag = expect_eq_i32(fb, sc, 4);
            let inv = call_shim(fb, fx, "ac_rt_check_invariant", &[])[0];
            // owner still live at this point -> invariant must FAIL (live=1)
            let inv_flag = expect_eq_i32(fb, inv, 1);
            call_shim(fb, fx, "ac_rt_own_free", &[ptr]);
            let inv2 = call_shim(fb, fx, "ac_rt_check_invariant", &[])[0];
            let inv2_flag = expect_eq_i32(fb, inv2, 0);
            let f = acc(fb, null_flag, sc_flag);
            let f = acc(fb, f, inv_flag);
            acc(fb, f, inv2_flag)
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert_eq!(summary_of(&out), (1, 1, 1, 0, 0, 4));
}

#[test]
fn v04_clone_independent_content_control() {
    let out = run_probe("v04_clone", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("lit_auto", b"Auto", false);
        cx.set_main(|fb, fx| {
            let (ptr, len) = own_from_lit(fb, fx, "lit_auto", 4);
            // clone into an independent buffer (second cell at +8)
            let sp = data_addr(fb, fx, "outcell");
            let off8 = const_ptr(fb, 8);
            let sp2 = fb.ins().iadd(sp, off8);
            let cptr = call_shim(fb, fx, "ac_rt_own_clone", &[ptr, len, sp2])[0];
            let clen = load_i32(fb, sp2, 0);
            let len_flag = expect_eq_i32(fb, clen, 4);
            // free the ORIGINAL, then verify the clone's content against the
            // still-valid rodata literal: content control, not just pointer
            // inequality (774 V03-10 oracle)
            call_shim(fb, fx, "ac_rt_own_free", &[ptr]);
            let lit = data_addr(fb, fx, "lit_auto");
            let four = const_i32(fb, 4);
            let eq = call_shim(fb, fx, "ac_rt_text_eq", &[cptr, clen, lit, four])[0];
            let eq_flag = expect_eq_i32(fb, eq, 1);
            call_shim(fb, fx, "ac_rt_own_free", &[cptr]);
            let f = acc(fb, len_flag, eq_flag);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    // both buffers are live simultaneously before the original frees: peak 8
    assert_eq!(summary_of(&out), (2, 2, 2, 0, 0, 8));
    // the original freed first: FREE id=1 then FREE id=2
    let idx1 = out.stderr.find("FREE id=1").expect("free 1");
    let idx2 = out.stderr.find("FREE id=2").expect("free 2");
    assert!(idx1 < idx2);
}

#[test]
fn v04_lifo_free_order_control() {
    let out = run_probe("v04_lifo", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("lit_a", b"aaaa", false);
        cx.data("lit_b", b"bbbbbbbb", false);
        cx.set_main(|fb, fx| {
            let (pa, _) = own_from_lit(fb, fx, "lit_a", 4);
            // second request uses the +8 cell so the first length survives
            let sp = data_addr(fb, fx, "outcell");
            let off8 = const_ptr(fb, 8);
            let sp2 = fb.ins().iadd(sp, off8);
            let p = data_addr(fb, fx, "lit_b");
            let n8 = const_i32(fb, 8);
            let pb = call_shim(fb, fx, "ac_rt_own_from_lit", &[p, n8, sp2])[0];
            // LIFO control: free B (id=2) before A (id=1)
            call_shim(fb, fx, "ac_rt_own_free", &[pb]);
            call_shim(fb, fx, "ac_rt_own_free", &[pa]);
            const_i32(fb, 0)
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    let idx2 = out.stderr.find("FREE id=2").expect("free 2");
    let idx1 = out.stderr.find("FREE id=1").expect("free 1");
    assert!(idx2 < idx1, "B must free before A under LIFO control");
}

#[test]
fn v04_zero_length_makes_no_attempt() {
    let out = run_probe("v04_zero", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.set_main(|fb, fx| {
            let sp = data_addr(fb, fx, "outcell");
            let nullp = const_ptr(fb, 0);
            let zero = const_i32(fb, 0);
            let ptr = call_shim(fb, fx, "ac_rt_own_from_lit", &[nullp, zero, sp])[0];
            let null_flag = expect_null(fb, ptr);
            let len = load_i32(fb, sp, 0);
            let len_flag = expect_eq_i32(fb, len, 0);
            call_shim(fb, fx, "ac_rt_own_free", &[ptr]); // free(null): no-op
            let f = acc(fb, null_flag, len_flag);
            observe(fb, fx);
            f
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert_eq!(summary_of(&out), (0, 0, 0, 0, 0, 0));
}

#[test]
fn v04_owned_return_transfers_across_frames() {
    // produce(): owned buffer created in the callee, POINTER parked in a
    // caller-owned cell as a full i64, length returned on the i32 channel;
    // main() frees it. The callee exits WITHOUT cleanup: ownership crossed
    // the frame boundary (774 V03-9 form).
    let out = run_probe("v04_return_transfer", |cx| {
        cx.data("outcell", &[0; 16], true);
        cx.data("lit_auto", b"Auto", false);
        cx.func(
            "produce",
            &[cranelift_codegen::ir::types::I64],
            |fb, fx, args| {
                let lit = data_addr(fb, fx, "lit_auto");
                let four = const_i32(fb, 4);
                let ptr = call_shim(fb, fx, "ac_rt_own_from_lit", &[lit, four, args[0]])[0];
                store_i64(fb, args[0], 0, ptr);
                four
            },
        );
        cx.set_main(|fb, fx| {
            let cell = data_addr(fb, fx, "outcell");
            let len = call_shim_aux(fb, fx, "produce", &[cell]);
            let ptr = ac_text_runtime_tests::load_i64(fb, cell, 0);
            let len_flag = expect_eq_i32(fb, len, 4);
            call_shim(fb, fx, "ac_rt_own_free", &[ptr]);
            let inv = call_shim(fb, fx, "ac_rt_check_invariant", &[])[0];
            let inv_flag = expect_eq_i32(fb, inv, 0);
            acc(fb, len_flag, inv_flag)
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert_eq!(summary_of(&out), (1, 1, 1, 0, 0, 4));
}

/// Call a caller-defined aux function (declared via `cx.func`).
fn call_shim_aux(
    fb: &mut cranelift_frontend::FunctionBuilder,
    fx: &ac_text_runtime_tests::Fx,
    name: &str,
    args: &[cranelift_codegen::ir::Value],
) -> cranelift_codegen::ir::Value {
    let f = *fx
        .funcs
        .get(name)
        .unwrap_or_else(|| panic!("unknown aux function {name}"));
    let inst = fb.ins().call(f, args);
    fb.inst_results(inst)[0]
}
