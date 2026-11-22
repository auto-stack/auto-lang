//! PLAN-1052 T-02 query host & dependency graph conformance (V-01/V-04):
//! the Auto `query` module's complete query key, positive/negative read
//! sets, body-vs-signature neighbour decisions and the production counters
//! are compared byte-exact against independent Rust references.

use ace_aac_incremental_conformance as lane;

fn obs(func: &str, call: &str) -> String {
    lane::run_str("query", "query", func, call).expect("Auto query run")
}

fn codes_bytes(report: &str) -> Vec<u8> {
    (0..report.len() / 2)
        .map(|i| u8::from_str_radix(&report[2 * i..2 * i + 2], 16).unwrap())
        .collect()
}

/// V-01/V-04: a body-only edit on callee B makes B recompute its body page
/// (neighbour 1 = callee-recompute) while caller A's SIGNATURE read stays
/// warm; a missing-name N flip MUST recompute A (neighbour 2 =
/// caller-recompute). The Auto module's negative-flip report carries the
/// negative key and both decisions.
#[test]
fn v04_negative_flip_and_body_separation() {
    let got = obs("negative_flip_report", "negative_flip_report(\"ac934/dom/v1\", \"logical-N\")");
    let neg_key = lane::axis2("ac1052/neg-read/v1", "ac934/dom/v1", "logical-N");
    // after N appears: sig same, body same, negative flipped -> recompute (2)
    // after B's body-only edit: sig same, body changed, no negative ->
    // callee-recompute (1)
    let want = format!("{neg_key},2,1");
    assert_eq!(got, want, "negative flip + body separation decisions");
}

/// V-01: the positive read set binds callee SIGNATURES — re-derived (same)
/// and a callee signature move (different).
#[test]
fn v01_positive_read_set() {
    let got = obs("pos_read_report", "pos_read_report()");
    let a = lane::axis2(
        "ac1052/pos-set/v1",
        &(lane::axis2("ac1052/pos-read/v1", "logical-B", "sig-v5")
            + &lane::axis2("ac1052/pos-read/v1", "logical-C", "sig-v3")),
        "",
    );
    let b = lane::axis2(
        "ac1052/pos-set/v1",
        &(lane::axis2("ac1052/pos-read/v1", "logical-B", "sig-v5")
            + &lane::axis2("ac1052/pos-read/v1", "logical-C", "sig-v3")),
        "",
    );
    let c = lane::axis2(
        "ac1052/pos-set/v1",
        &(lane::axis2("ac1052/pos-read/v1", "logical-B", "sig-v6")
            + &lane::axis2("ac1052/pos-read/v1", "logical-C", "sig-v3")),
        "",
    );
    let want = format!("{a},{b},{c}");
    assert_eq!(got, want, "positive read set identity");
    assert_ne!(a, c, "a callee signature move must change the read set");
}

/// The complete query key separates kinds, owners, axes and capabilities.
#[test]
fn v01_complete_key() {
    let got = obs("fact_key", "fact_key(\"infer-body\", \"logical-A\", \"sig-v5\", \"body-v7\", \"caps1\")");
    let want = lane::fact_key("infer-body", "logical-A", "sig-v5", "body-v7", "caps1");
    assert_eq!(got, want, "complete query key");
    let body_moved = lane::fact_key("infer-body", "logical-A", "sig-v5", "body-v8", "caps1");
    let caps_moved = lane::fact_key("infer-body", "logical-A", "sig-v5", "body-v7", "caps2");
    assert_ne!(got, body_moved, "body axis is part of the key");
    assert_ne!(got, caps_moved, "capability set is part of the key");
}

/// The decision taxonomy: reuse / key-changed / dep-changed /
/// negative-flipped / candidates, then the neighbour family 1,2,2
/// (callee-recompute, caller-recompute-sig, caller-recompute-negative).
#[test]
fn v01_decision_taxonomy() {
    let got = codes_bytes(&obs("query_codes", "query_codes()"));
    let want: &[u8] = &[0, 1, 2, 3, 4, 1, 2, 2];
    assert_eq!(got, want, "query decision taxonomy");
}

/// Production counters are real numbers, byte-exact.
#[test]
fn v01_counters() {
    let got = obs("counter_report", "counter_report(3, 5, 2, 1)");
    assert_eq!(got, "3,5,2,1", "production counters");
}
