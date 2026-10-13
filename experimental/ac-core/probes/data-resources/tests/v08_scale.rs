//! V-08: scale and budget — 0/1/4096 lifecycle loops, live-bytes budget on
//! both sides of the boundary (probe-cases.md §V-08; decisions.md §D-06).

#![cfg(windows)]

use ac_data_probe_774 as probe;
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::ir::types;
use cranelift_codegen::ir::InstBuilder;

fn lifecycle_loop(case: &str, rounds: i64) -> ac_data_probe_774::ProbeOutcome {
    probe::run_probe(case, |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(move |fb, fx| {
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
            let n = probe::const_i32(fb, rounds);
            let done = fb.ins().icmp(IntCC::SignedGreaterThanOrEqual, i, n);
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
    })
}

/// V08-1/2/3: 0 / 1 / 4096 lifecycles — exact counters, no live growth. The
/// 4096 figure is a verification scale, not a language limit.
#[test]
fn v08_lifecycle_loops() {
    let zero = lifecycle_loop("v08-n0", 0);
    probe::assert_pass(&zero, &[]);
    assert_eq!(probe::summary_of(&zero), (0, 0, 0, 0, 0, 0));

    let one = lifecycle_loop("v08-n1", 1);
    probe::assert_pass(&one, &[]);
    assert_eq!(probe::summary_of(&one), (1, 1, 1, 0, 0, 16));

    let big = lifecycle_loop("v08-n4096", 4096);
    probe::assert_pass(&big, &[]);
    assert_eq!(probe::summary_of(&big), (4096, 4096, 4096, 0, 0, 16));
}

/// V08-4/5: budget = live bytes; exactly-at passes, one-over is rejected
/// before the allocator, and a free immediately restores the allowance.
#[test]
fn v08_budget_boundary() {
    let out = probe::run_probe("v08-budget", |cx| {
        cx.data("outlen", &[0; 4], true);
        cx.set_main(Box::new(|fb, fx| {
            let outp = probe::data_addr(fb, fx, "outlen");
            let sixteen = probe::const_i32(fb, 16);
            // budget(16): 0+16 <= 16 — the exactly-at request succeeds
            let budget = probe::const_ptr(fb, 16);
            probe::call_shim(fb, fx, "p774_set_budget", &[budget]);
            let seed = probe::const_i32(fb, 7);
            let p = probe::call_shim(fb, fx, "p774_own_make", &[seed, sixteen, outp]).remove(0);
            let v1 = probe::expect_non_null(fb, p);
            // 16+1 > 16 — one over: budget rejection, allocator never called
            let one = probe::const_i32(fb, 1);
            let q = probe::call_shim(fb, fx, "p774_own_make", &[one, one, outp]).remove(0);
            let v2 = probe::expect_null(fb, q);
            let lq = probe::load_i32(fb, outp, 0);
            let v3 = probe::expect_eq_i32(fb, lq, probe::status::ERR_BUDGET);
            // free restores the allowance: the same 16-byte request succeeds
            probe::call_shim(fb, fx, "p774_own_free", &[p]);
            let p2 = probe::call_shim(fb, fx, "p774_own_make", &[seed, sixteen, outp]).remove(0);
            let v4 = probe::expect_non_null(fb, p2);
            probe::call_shim(fb, fx, "p774_own_free", &[p2]);
            let inv = probe::call_shim(fb, fx, "p774_check_invariant", &[]).remove(0);
            let v = probe::acc(fb, v1, v2);
            let v = probe::acc(fb, v, v3);
            let v = probe::acc(fb, v, v4);
            probe::acc(fb, v, inv)
        }));
    });
    probe::assert_pass(&out, &["OWN-MAKE seed=7 len=16"]);
    // attempts: make(16) ok, make(1) rejected pre-allocator, make(16) ok
    assert_eq!(probe::summary_of(&out), (2, 2, 2, 0, 0, 16));
}
