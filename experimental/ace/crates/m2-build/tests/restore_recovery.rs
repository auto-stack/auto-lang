//! `ace.m2.p978.restore-recovery` — V-ID-RESTORE / V-03 (PLAN-978
//! T-08/T-10): cold recovery over the m2b namespace — the 783 recovery
//! pass, the M978B cache purge, CURRENT re-verification, and the
//! restore-refusal on a missing store.

mod common;

use ace_m2_build::driver::M2Build;
use ace_m2_build::identity::TargetKind;
use common::{b01_inputs, scratch};

/// A missing store root is a NAMED restore refusal (never a silent
/// fresh-create that would pretend recovery happened).
#[test]
fn p978_restore_missing_store_refused() {
    let root = scratch("restore-missing");
    let err = M2Build::new(&root)
        .restore("nope")
        .expect_err("missing store must refuse");
    assert_eq!(err.code, "m2build.restore.missing-store");
}

/// Cold restore after real work: the recovery pass runs clean, every
/// published CURRENT root re-verifies live, and post-restore builds
/// re-populate the purged cache from scratch (fresh identities, real
/// values).
#[test]
fn p978_restore_reverifies_current_and_rebuilds() {
    let root = scratch("restore-ok");
    {
        let mut driver = M2Build::new(&root);
        driver.build(TargetKind::Av, &b01_inputs()).unwrap();
        driver.build(TargetKind::Native, &b01_inputs()).unwrap();
    }
    // A COLD restart (fresh driver instance, same store on disk).
    let report = M2Build::new(&root).restore("b01").unwrap();
    assert!(
        report.iter().any(|l| l.starts_with("recovered:")),
        "recovery receipt present: {report:?}"
    );
    assert!(
        report
            .iter()
            .any(|l| l.contains("current/av/") && l.ends_with("live")),
        "AV current root re-verified live: {report:?}"
    );
    assert!(
        report
            .iter()
            .any(|l| l.contains("current/native/") && l.ends_with("live")),
        "native current root re-verified live: {report:?}"
    );
    // Post-restore: cache was purged → the next build REBUILDS (a fresh
    // real pass, never a blind trust of historical PASS).
    let mut after = M2Build::new(&root);
    let rebuilt = after.build(TargetKind::Av, &b01_inputs()).unwrap();
    assert!(rebuilt.rebuilt, "post-restore build re-produces");
    assert_eq!(rebuilt.run.unwrap().exit_code, Some(14));
}
