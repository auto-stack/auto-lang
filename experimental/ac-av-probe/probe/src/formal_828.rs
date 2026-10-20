//! Formal function-incremental measurement tier (PLAN-828 T-08).
//!
//! The frozen 798 load (273 575 B, 115 functions, the edit site fn_037's
//! first multiplier) drives a REAL resident session: one initialization,
//! then one body edit per revision. Each sample times EDIT-ENTRY to
//! verified-current-result publication (the edit + the run that proves the
//! new value — initialization is excluded, single-listed), and captures
//! the structural counters that must be zero for every UNCHANGED function.
//! The dual gate: P95 < `gate_ms` AND every structural gate AND every
//! result correct — a timing pass can never cover a granularity failure,
//! and the gate negatives (V-09) prove each axis can go red on its own.

use crate::load256;
use crate::session::{warmup, FunctionIncrementalSession};
use auto_av_core::av::{CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value};
use std::time::Instant;

/// One measured revision.
#[derive(Clone, Debug)]
pub struct FormalSample {
    pub revision: u32,
    /// edit() call → current-result verified (milliseconds).
    pub edit_to_result_ms: f64,
    /// The independent expected value for this revision.
    pub expected: i32,
    /// The structural gate deltas for this edit.
    pub structural: StructuralSnapshot,
}

/// The zero-gates for one edit (all over the UNCHANGED functions).
#[derive(Clone, Debug, Default)]
pub struct StructuralSnapshot {
    /// Sum of every stage counter delta of every function OTHER than the
    /// edited one — must be 0 on the pure body fast path.
    pub unaffected_stage_total: u64,
    /// whole_module_{frontend,lower,closure_roundtrip} — structurally 0.
    pub whole_module_total: u64,
    /// Exactly one function installed this edit.
    pub install_set_len: usize,
    /// Syntax increment evidence: exactly the edited decl re-parsed.
    pub reparsed_decls: u64,
    /// ...and every other decl grafted.
    pub reused_decls: u64,
}

#[derive(Clone, Debug)]
pub struct FormalReport {
    pub samples: Vec<FormalSample>,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub all_correct: bool,
    /// Peak live-slot bytes across verification runs (execution-partition
    /// readout; the process-level two-line memory budget rides the batch
    /// tier — see contract-decision T-08).
    pub peak_slot_bytes: u64,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

/// Run the formal series: warm up ONCE on the frozen base load, then apply
/// `revisions` distinct body edits at fn_037's frozen multiplier site,
/// timing each edit→verified result and snapshotting the structural gate
/// deltas (counters are cumulative; deltas are taken between edits).
pub fn run_formal_series(revisions: u32) -> Result<FormalReport, String> {
    let base = load256::generate_revision(0);
    let mut session: FunctionIncrementalSession = warmup(&base.source, "d_main", &[])?;
    let cancel = CancelFlag::new(false);
    let limits = ExecLimits::default();
    let mut samples = Vec::with_capacity(revisions as usize);
    let mut peak_slot_bytes = 0u64;
    let mut prev_totals = stage_totals(&session);
    let mut prev_src = base.source.clone();
    for n in 1..=revisions {
        let rev = load256::generate_revision(n);
        // The frozen generator's revision n changes fn_037's first
        // multiplier; derive the edit by diffing the two sources (the
        // multiplier may cross a digit-width boundary).
        let (start, end, insert) = diff_edit(&prev_src, &rev.source)?;
        let started = Instant::now();
        let out = session.edit(start, end, insert)?;
        if !out.ok {
            return Err(format!(
                "revision {n}: fast path refused: {:?}",
                out.non_fast_path_reason
            ));
        }
        if out.install_set.len() != 1 || !out.install_set[0].ends_with("fn_037") {
            return Err(format!(
                "revision {n}: install set {:?} != [d_fn_037]",
                out.install_set
            ));
        }
        // Current-result publication: run the CURRENT snapshot and verify
        // against the INDEPENDENT expected for this revision.
        let (exec, trace) = session.run(&limits, &cancel);
        let elapsed = started.elapsed().as_secs_f64() * 1000.0;
        peak_slot_bytes = peak_slot_bytes.max(trace.peak_slot_bytes);
        let got = match exec {
            ExecOutcome::Success {
                value: Value::I32(v),
                ..
            } => v,
            other => return Err(format!("revision {n}: non-success {other:?}")),
        };
        if got != rev.expected_main {
            return Err(format!(
                "revision {n}: result {got} != independent expected {}",
                rev.expected_main
            ));
        }
        let now_totals = stage_totals(&session);
        let structural = delta_structural(&prev_totals, &now_totals, n);
        prev_totals = now_totals;
        prev_src = rev.source;
        samples.push(FormalSample {
            revision: n,
            edit_to_result_ms: elapsed,
            expected: rev.expected_main,
            structural,
        });
    }
    let mut times: Vec<f64> = samples.iter().map(|s| s.edit_to_result_ms).collect();
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Ok(FormalReport {
        p50_ms: percentile(&times, 0.50),
        p95_ms: percentile(&times, 0.95),
        p99_ms: percentile(&times, 0.99),
        all_correct: true,
        samples,
        peak_slot_bytes,
    })
}

/// Cumulative (function → stage-total, whole-module-total, syntax counters).
struct Totals {
    per_fn: std::collections::BTreeMap<String, u64>,
    whole_module: u64,
    reparsed: u64,
    reused: u64,
}

fn stage_totals(session: &FunctionIncrementalSession) -> Totals {
    let counters = session.counters();
    // The resident function set (names) — the formal load's fixed 115.
    let names = session.function_names();
    let mut per_fn = std::collections::BTreeMap::new();
    for name in names {
        let mut total = 0u64;
        for stage in crate::session::STAGES {
            total += counters.stage_of(stage, &name);
        }
        total += counters.stage_of("reuse_hit", &name);
        per_fn.insert(name, total);
    }
    let whole_module = counters.whole_module().values().sum::<u64>();
    Totals {
        per_fn,
        whole_module,
        reparsed: counters.get("syntax_reparsed_decls"),
        reused: counters.get("syntax_reused_decls"),
    }
}

fn delta_structural(prev: &Totals, now: &Totals, revision: u32) -> StructuralSnapshot {
    let mut unaffected = 0u64;
    for (name, total) in &now.per_fn {
        if name == "fn_037" {
            continue;
        }
        unaffected += total - prev.per_fn.get(name).copied().unwrap_or(0);
    }
    StructuralSnapshot {
        unaffected_stage_total: unaffected,
        whole_module_total: now.whole_module - prev.whole_module,
        install_set_len: 1,
        reparsed_decls: now.reparsed - prev.reparsed,
        reused_decls: now.reused - prev.reused,
    }
    .checked(revision)
}

impl StructuralSnapshot {
    fn checked(self, revision: u32) -> Self {
        // Reused decls on the frozen load: 115 total, 1 re-parsed, 114
        // grafted — recorded for evidence, not gated beyond reparsed==1.
        let _ = revision;
        self
    }
}

/// The structural + timing gate. Every axis can fail independently
/// (V-09 anti-fakes): see `validate_and_gate`.
pub fn validate_and_gate(report: &FormalReport, gate_ms: f64, min_samples: usize) -> Vec<String> {
    let mut failures = Vec::new();
    if gate_ms <= 0.0 {
        // A zero threshold must fail by construction (the 798 formal
        // contract): no honest measured run reports 0 ms.
        failures.push("gate-ms must be > 0 (zero-threshold anti-fake)".to_string());
    }
    if report.samples.len() < min_samples {
        failures.push(format!(
            "insufficient samples: {} < {min_samples}",
            report.samples.len()
        ));
    }
    if !report.all_correct {
        failures.push("not all results correct".to_string());
    }
    if report.p95_ms >= gate_ms {
        failures.push(format!("P95 {:.3}ms >= gate {gate_ms}ms", report.p95_ms));
    }
    for s in &report.samples {
        if s.structural.unaffected_stage_total != 0 {
            failures.push(format!(
                "revision {}: unaffected stage total {} != 0",
                s.revision, s.structural.unaffected_stage_total
            ));
        }
        if s.structural.whole_module_total != 0 {
            failures.push(format!(
                "revision {}: whole-module counter nonzero",
                s.revision
            ));
        }
        if s.structural.install_set_len != 1 {
            failures.push(format!(
                "revision {}: install set len {} != 1",
                s.revision, s.structural.install_set_len
            ));
        }
        if s.structural.reparsed_decls != 1 {
            failures.push(format!(
                "revision {}: reparsed {} decls (expected exactly the edited one)",
                s.revision, s.structural.reparsed_decls
            ));
        }
    }
    failures
}

/// First/last differing region between two sources → an Edit (old-source
/// coordinates). The frozen generator's revisions differ in one localized
/// multiplier; a prefix/suffix diff recovers it generically.
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
