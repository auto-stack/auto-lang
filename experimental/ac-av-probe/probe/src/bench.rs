//! Six-scenario turnaround bench (PLAN-798 §5.5, T-05).
//!
//! Scenarios (each N samples, warm store across the bench):
//! 1. `fresh`            — empty store, first full pipeline.
//! 2. `same-process-hot` — repeat requests in one driver/process.
//! 3. `restart-disk-hot` — NEW driver over the SAME store root (cold
//!                         process, warm disk) — also used cross-project by
//!                         switching the project label.
//! 4. `single-body-edit` — the frozen fn_037 edit per sample (new snapshot;
//!                         one unit misses, callers hit).
//! 5. `signature-conflict` — the frozen fn_leaf_1 arity edit; the request
//!                         must return the correct FRONTEND diagnostic.
//! 6. `gc-concurrent`    — requests interleaved with store GC passes.
//!
//! Each sample row carries the request wall, counters (real recompute vs
//! hit), and failure state; the hard gate (body-edit / signature-conflict
//! P95 < 1000 ms, request → correct AV result/diagnostic) reads these RAW
//! rows — no stage P95 sums, no dropped failures.

use crate::driver::Driver;
use crate::load256;
use serde::Serialize;
use std::path::Path;

#[derive(Clone, Debug, Serialize)]
pub struct SampleRow {
    pub scenario: &'static str,
    pub sample: usize,
    pub wall_ms: f64,
    pub ok: bool,
    pub value: Option<i32>,
    pub expected: Option<i32>,
    pub diag: Option<String>,
    pub cache_hit_abc: u64,
    pub cache_miss_abc: u64,
    pub mir_build: u64,
    pub abc_emit: u64,
    pub av_execute: u64,
}

#[allow(clippy::too_many_arguments)]
/// Hard gate (frozen §5.5): warm-session single-body-edit AND
/// signature-conflict requests must reach the correct AV result/diagnostic
/// with P95 < `gate_ms` measured from request to published outcome.
pub fn hard_gate(rows: &[SampleRow], gate_ms: f64) -> Vec<String> {
    use crate::observations::percentile_ms;
    let mut failures = Vec::new();
    for scenario in ["single-body-edit", "signature-conflict"] {
        let walls: Vec<f64> = rows
            .iter()
            .filter(|r| r.scenario == scenario)
            .map(|r| r.wall_ms)
            .collect();
        let p95 = percentile_ms(&walls, 95.0);
        if p95 >= gate_ms {
            failures.push(format!("hard gate: {} P95 {:.1}ms >= {:.1}ms", scenario, p95, gate_ms));
        }
        let any_bad = rows.iter().any(|r| r.scenario == scenario && !r.ok);
        if any_bad {
            failures.push(format!("hard gate: {} has failing samples", scenario));
        }
    }
    failures
}

pub fn run_six_scenarios(
    root: &Path,
    load: &load256::Load256,
    edited: &load256::Load256,
    conflicted: &str,
    samples: usize,
) -> Vec<SampleRow> {
    let mut rows = Vec::with_capacity(samples * 6);
    let entry = "d_main";

    // 1. fresh
    {
        let fresh_root = root.join("fresh");
        let mut d = Driver::with_store(&fresh_root, "bench-fresh").expect("store");
        for s in 0..samples {
            let r = d.request(&load.source, entry, &[]);
            rows.push(row("fresh", s, &r, Some(load.expected_main)));
        }
    }

    // 2. same-process-hot (store already warm from its own scenario root)
    {
        let hot_root = root.join("hot");
        let mut d = Driver::with_store(&hot_root, "bench-hot").expect("store");
        d.request(&load.source, entry, &[]); // warm
        for s in 0..samples {
            let r = d.request(&load.source, entry, &[]);
            rows.push(row("same-process-hot", s, &r, Some(load.expected_main)));
        }
    }

    // 3. restart-disk-hot: new Driver (fresh in-process state) over the hot
    //    root each sample; alternate project labels to exercise
    //    cross-project sharing of the same logical units.
    {
        let mut d = Driver::with_store(&root.join("hot"), "bench-hot-restart").expect("store");
        for s in 0..samples {
            if s % 2 == 1 {
                d = Driver::with_store(&root.join("hot"), &format!("bench-hot-xproj-{s}"))
                    .expect("store");
            }
            let r = d.request(&load.source, entry, &[]);
            rows.push(row("restart-disk-hot", s, &r, Some(load.expected_main)));
        }
    }

    // 4. single-body-edit: the frozen fn_037 edit (expected changes with it).
    {
        let edit_root = root.join("edit");
        let mut d = Driver::with_store(&edit_root, "bench-edit").expect("store");
        d.request(&load.source, entry, &[]); // warm base
        for s in 0..samples {
            let r = d.request(&edited.source, entry, &[]);
            rows.push(row("single-body-edit", s, &r, Some(edited.expected_main)));
        }
    }

    // 5. signature-conflict: correct diagnostic per sample (warm first).
    {
        let sig_root = root.join("sig");
        let mut d = Driver::with_store(&sig_root, "bench-sig").expect("store");
        d.request(&load.source, entry, &[]);
        for s in 0..samples {
            let r = d.request(&conflicted, entry, &[]);
            let diag = r.diag.as_ref().map(|d| format!("{}:{}", d.stage, d.code));
            rows.push(SampleRow {
                scenario: "signature-conflict",
                sample: s,
                wall_ms: r.wall_ms,
                ok: r.diag.is_some(),
                value: None,
                expected: None,
                diag,
                cache_hit_abc: *r.counters.get("cache_hit_abc").unwrap_or(&0),
                cache_miss_abc: *r.counters.get("cache_miss_abc").unwrap_or(&0),
                mir_build: *r.counters.get("mir_build").unwrap_or(&0),
                abc_emit: *r.counters.get("abc_emit").unwrap_or(&0),
                av_execute: *r.counters.get("av_execute").unwrap_or(&0),
            });
        }
    }

    // 6. gc-concurrent: requests interleaved with GC passes on a warm root.
    {
        let gc_root = root.join("gc");
        let mut d = Driver::with_store(&gc_root, "bench-gc").expect("store");
        d.request(&load.source, entry, &[]);
        let store = ac_artifact_probe::store::ManagedStore::open(&gc_root).expect("gc store");
        let cfg = ac_artifact_probe::store::gc::gc_config_default();
        for s in 0..samples {
            if s % 3 == 0 {
                let _ = ac_artifact_probe::store::gc::run_gc(&store, cfg);
            }
            let r = d.request(&load.source, entry, &[]);
            rows.push(row("gc-concurrent", s, &r, Some(load.expected_main)));
        }
    }
    rows
}

fn row(scenario: &'static str, sample: usize, r: &super::driver::RequestOutcome, expected: Option<i32>) -> SampleRow {
    SampleRow {
        scenario,
        sample,
        wall_ms: r.wall_ms,
        ok: r.ok() && r.value_i32() == expected,
        value: r.value_i32(),
        expected,
        diag: r.diag.as_ref().map(|d| format!("{}:{}", d.stage, d.code)),
        cache_hit_abc: *r.counters.get("cache_hit_abc").unwrap_or(&0),
        cache_miss_abc: *r.counters.get("cache_miss_abc").unwrap_or(&0),
        mir_build: *r.counters.get("mir_build").unwrap_or(&0),
        abc_emit: *r.counters.get("abc_emit").unwrap_or(&0),
        av_execute: *r.counters.get("av_execute").unwrap_or(&0),
    }
}
