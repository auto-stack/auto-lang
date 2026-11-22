//! PLAN-1052 T-04 driver/service/publish conformance (V-01/V-03): the Auto
//! `driver` module's error classification, closure decisions and the
//! V-03 bad-edit/keep-old/repair flow are compared byte-exact against
//! independent Rust references.

use ace_aac_incremental_conformance as lane;

fn obs(func: &str, call: &str) -> String {
    lane::run_str("driver", "driver", func, call).expect("Auto driver run")
}

fn codes_bytes(report: &str) -> Vec<u8> {
    (0..report.len() / 2)
        .map(|i| u8::from_str_radix(&report[2 * i..2 * i + 2], 16).unwrap())
        .collect()
}

/// V-03: a bad edit is a LANGUAGE error (1); the failed update KEEPS the
/// old closure (2); after repair the driver really produces a new root (4).
#[test]
fn v03_bad_edit_keep_old_repair() {
    let got = obs("v03_report", "v03_report()");
    // err_bad=1 (language), kept_old=2, err_repair=0 (success), dec_repair=4
    assert_eq!(got, "1,2,0,4", "V-03 bad edit -> keep old -> repair produce");
}

/// The error-class taxonomy: success/language/resource/os/tool/admission.
#[test]
fn v03_error_classification() {
    let got = codes_bytes(&obs("driver_codes", "driver_codes()"));
    // error classes: 0,1,2,3,4,5 | closure: cold=0, hit=1, kept-old=2,
    // affected-only=3, repair=4
    let want: &[u8] = &[0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4];
    assert_eq!(got, want, "driver taxonomy");
}

/// V-01 stage identity: re-derived (same), source moved (different), a
/// distinct pass (different).
#[test]
fn v01_stage_identity() {
    let got = obs("stage_report", "stage_report()");
    let a = lane::stage_key("build", "lower", "src-v1", "gram-v2");
    let b = lane::stage_key("build", "lower", "src-v1", "gram-v2");
    let c = lane::stage_key("build", "lower", "src-v2", "gram-v2");
    let d = lane::stage_key("build", "emit", "src-v1", "gram-v2");
    let want = format!("{a},{b},{c},{d}");
    assert_eq!(got, want, "stage identity report");
    assert_ne!(a, c, "source version moves the stage identity");
    assert_ne!(a, d, "pass moves the stage identity");
}
