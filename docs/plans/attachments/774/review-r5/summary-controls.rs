//! Protocol controls against the committed SUMMARY parser, not a large allocation test.
#![cfg(test)]
use ac_data_probe_774::{summary_of, ProbeOutcome};

fn outcome(peak: u64) -> ProbeOutcome {
    ProbeOutcome {
        code: Some(0), stdout: String::new(),
        stderr: format!("SUMMARY attempts=3 allocs=3 frees=0 live=3 live_bytes={peak} peak={peak}\n"),
    }
}

#[test]
fn u32_peak_control() {
    let result = summary_of(&outcome(u32::MAX as u64));
    assert_eq!(result.0, 3);
    assert_eq!(result.4 as u64, u32::MAX as u64);
    assert_eq!(result.5 as u64, u32::MAX as u64);
}

#[test]
fn peak_above_u32_keeps_all_bits() {
    let result = summary_of(&outcome(4294967298));
    assert_eq!(result.4 as u64, 4294967298, "live_bytes precision");
    assert_eq!(result.5 as u64, 4294967298, "peak precision silently lost");
}
