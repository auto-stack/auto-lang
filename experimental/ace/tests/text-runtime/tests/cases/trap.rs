//! PLAN-770 T-03 trap faces: fatal trap is OS reclaim (no SUMMARY, no
//! claimed unwinding) and the hang control classifies as a deadline with a
//! surviving predecessor trace.

#![cfg(windows)]
use ace_text_conformance::{call_shim, const_i32, run_probe};

#[test]
fn v06_trap_after_alloc_is_os_reclaim() {
    let out = run_probe("v06_trap", |cx| {
        cx.set_main(|fb, fx| {
            let h61 = const_i32(fb, 16);
            let n = h61;
            call_shim(fb, fx, "ac_rt_trap_after_alloc", &[n]);
            let h62 = const_i32(fb, 0);
            h62 // unreachable; keeps the body well-typed
        });
    });
    assert_eq!(
        out.code,
        Some(0xC000_0005u32 as i32),
        "stderr:\n{}",
        out.stderr
    );
    assert!(out.stderr.contains("TRAP-PRE live=1"), "{}", out.stderr);
    // no cleanup ran and none is claimed: no SUMMARY line after the trap
    assert!(
        !out.stderr.lines().any(|l| l.starts_with("SUMMARY ")),
        "stderr:\n{}",
        out.stderr
    );
}

#[test]
fn v06_hang_classifies_as_deadline_with_trace() {
    let (kind, trace) = ace_text_conformance::run_expect_deadline_traced("v06_hang", |cx| {
        cx.set_main(|fb, fx| {
            let h63 = const_i32(fb, 16);
            let n = h63;
            call_shim(fb, fx, "ac_rt_hang_after_alloc", &[n]);
            let h64 = const_i32(fb, 0);
            h64
        });
    });
    assert_eq!(kind, "deadline");
    assert!(trace.contains("HANG-PRE live=1"), "trace:\n{}", trace);
}
