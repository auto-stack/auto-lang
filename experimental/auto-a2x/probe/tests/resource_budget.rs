//! T-04 resource budget probe (§5.6): the A2X MANAGEMENT working set at
//! load256 scale (115 functions + main, ≥273,575 source bytes — the frozen
//! baseline's shape; the formal T-06 sampling keeps the ORIGINAL load256
//! bytes) must stay ≤ 48 MiB in the WARM steady state.
//!
//! Measured fact (this bin, both debug and release): the COLD whole-source
//! compile transiently peaks far above that — an ac-core FRONTEND
//! characteristic (~1.2 KiB peak per source byte), upstream of A2X and
//! outside its control. The plan's budget governs the A2X management set:
//! after the cold transient drains, the co-resident session state
//! (retained env + residents + store IO + per-fragment production) is what
//! must fit. The warm path therefore parses ONLY the edited function
//! fragment (§5.4) — a whole-source parse per warm edit would re-trigger
//! the frontend transient and blow the budget by construction.

use std::process::Command;

const MANAGEMENT_BUDGET_BYTES: u64 = 48 * 1024 * 1024;

#[test]
fn warm_steady_state_within_48mib() {
    let bin = env!("CARGO_BIN_EXE_a2x-mem");
    let out = Command::new(bin).output().expect("spawn a2x-mem");
    assert!(
        out.status.success(),
        "a2x-mem failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    let mut fields = std::collections::BTreeMap::new();
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            fields.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    let warm = fields["warm_max_current_working_set_bytes"]
        .parse::<u64>()
        .expect("warm number");
    let source_bytes: u64 = fields["source_bytes"].parse().expect("source size");
    let cold_peak: u64 = fields["cold_peak_working_set_bytes"]
        .parse()
        .expect("cold peak");
    assert!(
        source_bytes >= 273_575,
        "measurement must run at the frozen load256 scale (got {}B)",
        source_bytes
    );
    assert!(
        warm <= MANAGEMENT_BUDGET_BYTES,
        "A2X warm management set {}B exceeds the 48MiB budget",
        warm
    );
    // informational: the upstream cold-compile transient (never asserted —
    // it is ac-core's cost, recorded here for the plan's resource table)
    println!(
        "source={}B cold_peak(upstream)={}B warm_max_current={}B budget={}B",
        source_bytes, cold_peak, warm, MANAGEMENT_BUDGET_BYTES
    );
}
