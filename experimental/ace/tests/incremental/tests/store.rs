//! PLAN-1052 T-03 store/restore conformance (V-02 identity binding & the
//! admission taxonomy): the Auto `store` module's six-axis BUILD identity
//! keys, page keys, refs and the controlled restore-admission codes are
//! compared byte-exact against independent Rust references written from the
//! contract text — never copies of the Auto output.

use ace_aac_incremental_conformance as lane;

fn obs(func: &str, call: &str) -> String {
    lane::run_str("store", "store", func, call).expect("Auto store run")
}

fn codes_bytes(report: &str) -> Vec<u8> {
    (0..report.len() / 2)
        .map(|i| u8::from_str_radix(&report[2 * i..2 * i + 2], 16).unwrap())
        .collect()
}

/// V-02 identity binding: same inputs re-derive the SAME six-axis key;
/// stdlib/grammar axis moves change it; the page key separates
/// body-only edits (signature readers keep the page); refs separate
/// domains. One Auto report, split by the independent reference.
#[test]
fn v02_identity_binding() {
    let got = obs("identity_report", "identity_report()");
    let base = lane::six_axis_key("aac-build/1", "mn-windows-x64/1", "stdlib-a/1", "grammar-v2/1", "pass-lower/1", "bridge-ac/1");
    let same = lane::six_axis_key("aac-build/1", "mn-windows-x64/1", "stdlib-a/1", "grammar-v2/1", "pass-lower/1", "bridge-ac/1");
    let stdlib_b = lane::six_axis_key("aac-build/1", "mn-windows-x64/1", "stdlib-b/1", "grammar-v2/1", "pass-lower/1", "bridge-ac/1");
    let gram_v3 = lane::six_axis_key("aac-build/1", "mn-windows-x64/1", "stdlib-a/1", "grammar-v3/1", "pass-lower/1", "bridge-ac/1");
    let p1 = lane::page_key("logical-A", "sig-v5", "body-v7", "caps1");
    let p2 = lane::page_key("logical-A", "sig-v5", "body-v8", "caps1");
    let r1 = lane::ref_key("ac1052/ref/required/v1", &p1, "art0001");
    let r2 = lane::ref_key("ac1052/ref/provenance/v1", &p1, "art0001");
    let want = format!(
        "{base},{same},{stdlib_b},{gram_v3}|{p1},{p2}|{r1},{r2}"
    );
    assert_eq!(got, want, "identity family report");
    assert_ne!(base, stdlib_b, "stdlib axis must change the key");
    assert_ne!(base, gram_v3, "grammar axis must change the key");
    assert_ne!(p1, p2, "body-only edit must change the page key");
    assert_ne!(r1, r2, "same page+artifact under different domains must differ");
}

/// V-02 admission taxonomy: the seven restore-admission codes and the four
/// publish codes are returned as ONE byte each in a fixed order.
#[test]
fn v02_admission_taxonomy() {
    let got = codes_bytes(&obs("store_codes", "store_codes()"));
    // admit: all ok | unknown-envelope | corrupt | stale | dangling-ref |
    // missing-coverage | with-diagnostics | publish: commit-ok |
    // prepare-failed | commit-failed | native-failed
    let want: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 0, 1, 2, 3];
    assert_eq!(got, want, "store taxonomy");
}

/// A same-identity page checksum is stable (determinism proof within ONE
/// program, one Gen0 compile); the expected value is the independent blake3
/// over the exact payload bytes (content NUL is content).
#[test]
fn v02_page_checksum() {
    let got = obs("checksum_report", "checksum_report()");
    let expect = blake3::hash(b"fn main() int { return 42 }").to_hex().to_string();
    let want = format!("{expect},{expect}");
    assert_eq!(got, want, "byte-exact blake3 over the payload bytes, deterministic");
}
