//! PLAN-1052 main conformance: the V-01 cold→hot→restore loop and the
//! V-01/V-04 edit neighbourhoods as ONE real decision sequence through the
//! Auto driver/query modules. Every expected code is an independent
//! reference written from the contract text — never the Auto output.

use ace_aac_incremental_conformance as lane;

fn obs(func: &str, call: &str) -> String {
    lane::run_str("conformance", "driver", func, call).expect("Auto driver run")
}

/// V-01 cold→hot→restore, then body/signature/new-symbol edits. The report
/// concatenates the closure decisions and neighbour decisions in order:
///   cold-produce (0) | cache-hit (1) | kept-old (2) |
///   affected-only (3) | repair-produce (4) |
///   neighbour: callee-recompute (1) | caller-reuse (0) |
///   caller-recompute (2).
#[test]
fn v01_cold_hot_restore_loop() {
    let got = obs("v01_report", "v01_report()");
    let want = "0001020304010002";
    assert_eq!(got, want, "V-01 cold/hot/restore + edit neighbourhood sequence");
}

/// V-01 production counters across the loop: cold solve once, hot reuses,
/// body edit recomputes the callee only, a signature edit recomputes the
/// caller, a negative flip recomputes the owner — reported per phase, then
/// the cumulative totals.
#[test]
fn v01_counters_across_edits() {
    let got = obs("v01_counters", "v01_counters()");
    // per-phase (solver,body_hits,body_misses,rejections): cold, hot,
    // body-edit, sig-edit, neg-flip | cumulative (4,1,4,0)
    let want = "1,0,1,0,0,1,0,0,1,0,1,0,1,0,1,0,1,0,1,0|4,1,4,0";
    assert_eq!(got, want, "V-01 production counters across the edit loop");
}
