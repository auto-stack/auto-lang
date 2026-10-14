#![cfg(test)]
use ac_data_probe_774::{freestat_of, summary_of, ProbeOutcome};

fn outcome(stderr: &str) -> ProbeOutcome {
    ProbeOutcome { code: Some(0), stdout: String::new(), stderr: stderr.to_owned() }
}

#[test]
fn actual_formatter_u32_edge_roundtrip() {
    let out = outcome(include_str!("edge.stderr.txt"));
    assert_eq!(summary_of(&out), (11u32, 5u32, 2u32, 3u32, 4294967295u64, 4294967295u64));
    assert_eq!(freestat_of(&out), (2, 2, 0));
}

#[test]
fn actual_formatter_distinct_wide_byte_fields_roundtrip() {
    let out = outcome(include_str!("wide.stderr.txt"));
    assert_eq!(summary_of(&out), (11u32, 5u32, 2u32, 3u32, 4294967298u64, 4294967315u64));
    assert_eq!(freestat_of(&out), (2, 2, 0));
}

#[test]
fn actual_formatter_zero_live_keeps_historical_wide_peak() {
    let out = outcome(include_str!("released.stderr.txt"));
    assert_eq!(summary_of(&out), (11u32, 5u32, 5u32, 0u32, 0u64, 4294967315u64));
    assert_eq!(freestat_of(&out), (5, 5, 0));
}
