//! PLAN-828 T-08 FORMAL MEASUREMENT CAMPAIGN (V-08) — the real judged
//! series over the persistent worker.
//!
//! Everything the plan §5.5/V-08 requires for the formal gate runs here:
//! the frozen 273 575 B / 115-function load, ONE worker process warmed
//! once, then ≥30 REAL body edits at fn_037's frozen multiplier site.
//! The judged window is edit ENQUEUE (parent side, IPC included) → the
//! parent's verification of the published CURRENT result against the
//! revision's independent expectation. The dual gate: P95 < `gate_ms`
//! AND every result correct AND every structural axis green (unchanged
//! functions zero stage work, zero whole-module movement, exactly one
//! install per revision). Dual-line memory: the worker's sampled peak
//! rides every publication; the parent samples its own line around the
//! whole campaign. Initialization is excluded (single-listed), the
//! ready handshake is judged before timing starts, and every raw sample
//! is preserved (`raw_jsonl`).

use crate::formal_worker::{run_formal_session_judge, JudgeConfig, JudgeEdit};
use crate::load256;
use std::path::Path;

/// One formal campaign run over the persistent worker.
pub struct CampaignConfig {
    /// Revisions to measure (>= 30 for the formal gate).
    pub revisions: u32,
    /// The enforced P95 bound in ms (V-08: 1000.0).
    pub gate_ms: f64,
}

pub const FORMAL_GATE_MS: f64 = 1000.0;
pub const FORMAL_MIN_REVISIONS: u32 = 30;

/// Build the judged edit script for the frozen load: revision n's source
/// differs from n-1 exactly at fn_037's first multiplier (the generator
/// guarantees distinct sources and independent expectations).
fn formal_edits(revisions: u32) -> Result<Vec<JudgeEdit>, String> {
    let mut edits = Vec::with_capacity(revisions as usize);
    let mut prev = load256::generate_revision(0);
    for n in 1..=revisions {
        let rev = load256::generate_revision(n);
        let (start, end, insert) = diff_edit(&prev.source, &rev.source)?;
        edits.push(JudgeEdit {
            start,
            end,
            insert: String::from_utf8(insert).map_err(|e| e.to_string())?,
            expected: rev.expected_main,
        });
        prev = rev;
    }
    Ok(edits)
}

fn diff_edit(cur: &str, next: &str) -> Result<(u64, u64, Vec<u8>), String> {
    let (a, b) = (cur.as_bytes(), next.as_bytes());
    let mut start = 0usize;
    while start < a.len() && start < b.len() && a[start] == b[start] {
        start += 1;
    }
    let mut tail = 0usize;
    while tail < a.len() - start
        && tail < b.len() - start
        && a[a.len() - 1 - tail] == b[b.len() - 1 - tail]
    {
        tail += 1;
    }
    if start == a.len() && start == b.len() {
        return Err("identical sources: not a revision".to_string());
    }
    Ok((
        start as u64,
        (a.len() - tail) as u64,
        b[start..b.len() - tail].to_vec(),
    ))
}

/// Run the FORMAL judged campaign (V-08). Returns the judge report; the
/// caller asserts the gates and preserves the raw stream.
pub fn run_formal_campaign(
    worker_exe: &Path,
    cfg: &CampaignConfig,
) -> Result<crate::formal_worker::JudgeReport, String> {
    let base = load256::generate_revision(0);
    let edits = formal_edits(cfg.revisions)?;
    let judge_cfg = JudgeConfig {
        src: base.source,
        entry: "d_main".to_string(),
        caps: vec![],
        expected_initial: base.expected_main,
        edits,
        gate_ms: cfg.gate_ms,
    };
    run_formal_session_judge(worker_exe, &judge_cfg)
}
