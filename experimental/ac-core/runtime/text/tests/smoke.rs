//! PLAN-770 T-03 smoke: the runtime builds, links into a real PE and its
//! invariant channel reports a clean zero state.

#![cfg(windows)]
use ac_text_runtime_tests::{call_shim, run_probe};

#[test]
fn smoke_links_and_reports_zero_state() {
    let out = run_probe("smoke_zero", |cx| {
        cx.set_main(|fb, fx| {
            let inv = call_shim(fb, fx, "ac_rt_check_invariant", &[])[0];
            inv // 0 = allocs==frees, live==0, live_bytes==0, invalid==0
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
    assert!(out
        .stderr
        .contains("SUMMARY attempts=0 allocs=0 frees=0 live=0 live_bytes=0 peak=0"));
}

#[test]
fn smoke_shim_error_code() {
    let out = run_probe("smoke_shim_error", |cx| {
        cx.set_main(|fb, fx| {
            let code = call_shim(fb, fx, "ac_rt_shim_error", &[])[0];
            ac_text_runtime_tests::expect_eq_i32(fb, code, -6)
        });
    });
    assert_eq!(out.code, Some(0), "stderr:\n{}", out.stderr);
}
