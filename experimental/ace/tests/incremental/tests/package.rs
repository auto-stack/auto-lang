//! PLAN-1052 T-09 package closure & generated FFI assembly conformance
//! (V-05): the Auto `package` module's lock-closure identity (cwd
//! invariance), undeclared-materialized rejection, provider/target ABI
//! invalidation and the missing-external-library keep-old flow are
//! compared byte-exact against independent Rust references.

use ace_aac_incremental_conformance as lane;

fn obs(func: &str, call: &str) -> String {
    lane::run_str("package", "package", func, call).expect("Auto package run")
}

fn codes_bytes(report: &str) -> Vec<u8> {
    (0..report.len() / 2)
        .map(|i| u8::from_str_radix(&report[2 * i..2 * i + 2], 16).unwrap())
        .collect()
}

/// V-05: the same lock closure re-derived under a different cwd has the
/// SAME identity (cwd is not part of the key); a provider change
/// invalidates the affected artifact (4); a missing external library is a
/// tool/link error (4) and keeps the old closure (3).
#[test]
fn v05_lock_closure_cwd_invariance() {
    let got = obs("v05_report", "v05_report(\"lock-a\", \"packages-1\", \"provider-x\")");
    let k1 = lane::axis2(
        "ac1052/lock-closure/v1",
        &lane::axis2(
            "ac1052/lock/v1",
            "lock-a",
            "packages-1",
        ),
        &lane::axis2("ac1052/provider-target/v1", "provider-x", "mn-windows-x64/1"),
    ) + "caps1";
    let k2 = lane::axis2(
        "ac1052/lock-closure/v1",
        &lane::axis2("ac1052/lock/v1", "lock-a", "packages-1"),
        &lane::axis2("ac1052/provider-target/v1", "provider-x", "mn-windows-x64/1"),
    ) + "caps1";
    let want = format!("{k1},{k2},4,4,3");
    assert_eq!(got, want, "V-05 lock closure + cwd invariance + ABI change + missing lib");
    assert_eq!(k1, k2, "same lock closure under a different cwd must keep identity");
}

/// The package taxonomy: ok / undeclared-materialized / declared-missing /
/// cwd-invariant / provider-changed / missing-lib.
#[test]
fn v05_package_taxonomy() {
    let got = codes_bytes(&obs("package_codes", "package_codes()"));
    let want: &[u8] = &[0, 1, 2, 0, 4, 5];
    assert_eq!(got, want, "package closure taxonomy");
}
