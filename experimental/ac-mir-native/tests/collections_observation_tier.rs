mod common;
use ac_mn::collections_observe::{decode, Outcome};
#[test]
fn real_trap_and_normal70_remain_distinct_with_bounded_negative_controls() {
    let trap = common::collections::run_source(
        "fn main() int { var xs = List<int>.new()\n return xs.get(0) }",
        "observe-index",
    );
    assert_eq!(
        decode(trap.exit_code, &trap.stderr).unwrap().outcome,
        Outcome::Trap(21)
    );
    let normal = common::collections::run_source("fn main() int { return 70 }", "observe-normal70");
    assert_eq!(
        decode(normal.exit_code, &normal.stderr).unwrap().outcome,
        Outcome::Return(70)
    );
    let raw = std::str::from_utf8(&trap.stderr).unwrap();
    let line = "MN-COLL-TRAP code=21\n";
    let peak = raw
        .lines()
        .find(|line| line.starts_with("MN-COLL arena_peak="))
        .unwrap();
    for (name, exit, bytes) in [
        ("mismatched-exit", Some(42), trap.stderr.clone()),
        ("missing-status", None, trap.stderr.clone()),
        (
            "forged-prefix",
            Some(70),
            raw.replace(line, "fake-MN-COLL-TRAP code=21\n")
                .into_bytes(),
        ),
        (
            "duplicate",
            Some(70),
            raw.replace(line, &format!("{line}{line}")).into_bytes(),
        ),
        (
            "truncated-reason",
            Some(70),
            raw.replace(line, "MN-COLL-TRAP code=\n").into_bytes(),
        ),
        (
            "unknown-reason",
            Some(70),
            raw.replace(line, "MN-COLL-TRAP code=99\n").into_bytes(),
        ),
        (
            "noncanonical-reason",
            Some(70),
            raw.replace(line, "MN-COLL-TRAP code=021\n").into_bytes(),
        ),
        ("missing-balances", Some(70), line.as_bytes().to_vec()),
        (
            "truncated-summary",
            Some(70),
            trap.stderr[..trap.stderr.len() - 1].to_vec(),
        ),
        (
            "missing-resource",
            Some(70),
            raw.replace("MN-COLL arena_live=0\n", "").into_bytes(),
        ),
        (
            "duplicate-resource",
            Some(70),
            raw.replace("MN-COLL live=0\n", "MN-COLL live=0\nMN-COLL live=0\n")
                .into_bytes(),
        ),
        (
            "missing-text",
            Some(70),
            raw.lines()
                .filter(|l| !l.starts_with("SUMMARY "))
                .map(|l| format!("{l}\n"))
                .collect::<String>()
                .into_bytes(),
        ),
        (
            "leaked-node",
            Some(70),
            raw.replace("MN-COLL live=0\n", "MN-COLL live=1\n")
                .into_bytes(),
        ),
        (
            "over-budget",
            Some(70),
            raw.replace(peak, "MN-COLL arena_peak=33554433")
                .into_bytes(),
        ),
        (
            "over-cap",
            Some(70),
            vec![b'\n'; ac_mn::config::STREAM_CAP + 1],
        ),
    ] {
        if exit == trap.exit_code {
            assert_ne!(bytes, trap.stderr, "{name}: mutation did not occur");
        }
        assert!(decode(exit, &bytes).is_err(), "{name} was admitted");
    }
    // No reason line is an ordinary return70, so the parent trap oracle rejects it.
    let missing = raw.replace(line, "");
    assert_eq!(
        decode(Some(70), missing.as_bytes()).unwrap().outcome,
        Outcome::Return(70)
    );
    assert_ne!(
        decode(Some(70), missing.as_bytes()).unwrap().outcome,
        Outcome::Trap(21)
    );
}
