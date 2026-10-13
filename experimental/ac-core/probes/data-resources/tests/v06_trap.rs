//! V-06: fatal trap after allocation and the executor's timeout
//! classification (probe-cases.md §V-06; decisions.md §D-06). The trap case
//! records OS reclaim only — no destructor execution is claimed (the absent
//! SUMMARY line is the evidence), and no unwinding is promised.

#![cfg(windows)]

use ac_data_probe_774 as probe;

/// Windows access-violation exit code (STATUS_ACCESS_VIOLATION).
const AV_EXIT: i32 = 0xC000_0005u32 as i32;

/// V06-1: allocate, trace, then deliberately fault. The process dies with
/// 0xC0000005; the allocation stays live (OS reclaims it with the process);
/// the cleanup summary is NEVER emitted.
#[test]
fn v06_1_trap_after_alloc() {
    let out = probe::run_probe("v06-1", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let sixteen = probe::const_i32(fb, 16);
            // diverging shim call; the return value is never produced
            probe::call_shim(fb, fx, "p774_trap_after_alloc", &[sixteen]);
            probe::const_i32(fb, 0)
        }));
    });
    assert_eq!(
        out.code,
        Some(AV_EXIT),
        "trap exit code; stderr:\n{}",
        out.stderr
    );
    // predecessor trace observed before the fault
    assert!(
        out.stderr.lines().any(|l| l == "TRAP-PRE live=1"),
        "missing TRAP-PRE; stderr:\n{}",
        out.stderr
    );
    // NO cleanup ran: no SUMMARY line — the OS reclaimed the process, the
    // language did not execute destructors (and none are claimed).
    assert!(
        !out.stderr.lines().any(|l| l.starts_with("SUMMARY ")),
        "cleanup summary must be absent after a fatal trap; stderr:\n{}",
        out.stderr
    );
}

/// V06-2: hang classification — the contained executor reports Deadline for
/// a process that never exits (controlled timeout, not success/crash). The
/// predecessor trace is OBSERVED through a file-redirected wrapper inside
/// the same contained process tree (the deadline branch drops piped output
/// by design).
#[test]
fn v06_2_hang_deadline_classification() {
    let (kind, trace) = probe::run_expect_deadline_traced("v06-2", |cx| {
        cx.set_main(Box::new(|fb, fx| {
            let sixteen = probe::const_i32(fb, 16);
            probe::call_shim(fb, fx, "p774_hang_after_alloc", &[sixteen]);
            probe::const_i32(fb, 0)
        }));
    });
    assert_eq!(kind, "deadline");
    assert!(
        trace.lines().any(|l| l == "HANG-PRE live=1"),
        "missing HANG-PRE in redirected trace; got:\n{trace}"
    );
}
