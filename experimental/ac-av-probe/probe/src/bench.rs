//! Six-scenario turnaround bench (PLAN-798 §5.5, r2 semantics).
//!
//! Scenario definitions are ENFORCED, not labeled (review F798-R1-04):
//! 1. `fresh` — a fresh store root PER SAMPLE (real cold state).
//! 2. `same-process-hot` — repeat requests in one driver/process.
//! 3. `restart-disk-hot` — a NEW PROCESS per sample (spawned `request`
//!    subcommand) over the warm root; odd samples alternate the project
//!    label for cross-project sharing.
//! 4. `single-body-edit` — every sample requests a CURRENT NEW revision
//!    (`load256::generate_revision(i + 1)`), each with its own independent
//!    expected value; never the same source twice.
//! 5. `signature-conflict` — accepts ONLY the target frontend diagnostic
//!    (`stage=frontend`, code `source.*`); resource/infra errors fail.
//! 6. `gc-concurrent` — a real concurrent GC process (`gc-loop`
//!    subcommand) overlapping the requests; not a serial pre-pass.
//!
//! `validate_and_gate` checks sample counts, per-row success, exact
//! diagnostics and the frozen hard gate; the worker turns failures into a
//! non-zero exit. Raw rows carry per-sample input hashes and counters.

use crate::driver::{Driver, RequestOutcome};
use crate::load256;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct BenchInputs {
    pub load: load256::Load256,
    pub conflict: String,
    pub samples: usize,
    pub gate_ms: f64,
    pub self_exe: PathBuf,
    /// Size budget for per-sample edit revisions (the frozen tier passes
    /// the full 256 KiB; the debug smoke may shrink it — the RELEASE gate
    /// never does).
    pub revision_target: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct SampleRow {
    pub scenario: &'static str,
    pub sample: usize,
    /// Revision index for edit scenarios (0 = unedited base).
    pub revision: u32,
    pub wall_ms: f64,
    pub ok: bool,
    pub value: Option<i32>,
    pub expected: Option<i32>,
    pub diag: Option<String>,
    pub source_hash: String,
    pub cache_hit_abc: u64,
    pub cache_miss_abc: u64,
    pub mir_lower_modules: u64,
    pub mir_units_persisted: u64,
    pub abc_units_emitted: u64,
    pub closure_encodes: u64,
    pub av_execute: u64,
    /// Managed partitions (§3): decode/index staging, hot closure bytes,
    /// AV live slot high-water.
    pub peak_staging_bytes: u64,
    pub peak_hot_bytes: u64,
    pub av_peak_slot_bytes: u64,
}

/// Frozen managed-partition budgets (§3 r2): staging 16 MiB, hot 8 MiB,
/// execution slots 8 MiB — summing to the 32 MiB managed envelope.
pub const BUDGET_STAGING_BYTES: u64 = 16 * 1024 * 1024;
pub const BUDGET_HOT_BYTES: u64 = 8 * 1024 * 1024;
pub const BUDGET_EXEC_SLOT_BYTES: u64 = 8 * 1024 * 1024;
/// r5 two-line worker-memory policy (plan §5.9 / F798-R12-04, user ruling
/// 2026-10-20): the former single ≤128 MiB worker line splits into two
/// attributed budget lines, both measured and both HARD-gated — nothing is
/// deducted or waived:
///   * Line A (ac-core owned): the frontend compile TRANSIENT, measured in
///     isolation on the frozen load (`frontend_peak_bytes`), ceiling
///     224 MiB anchored to the measured ~200-212 MiB (descriptor::bind
///     alone ≈158.9 MiB); expected to TIGHTEN when ac-core converges the
///     atom render→re-bind architecture — it is not a license to grow.
///   * Line B (probe owned): the residual the probe pipeline itself adds
///     (overall sampled peak MINUS Line A), ceiling 128 MiB (number
///     unchanged from §3; semantics now attributed per §5.9).
pub const BUDGET_FRONTEND_TRANSIENT_PEAK_BYTES: u64 = 224 * 1024 * 1024;
pub const BUDGET_WORKER_PRIVATE_PEAK_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Default)]
pub struct ScenarioStat {
    pub samples: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
    pub failures: usize,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct BenchSummary {
    pub fresh: ScenarioStat,
    pub same_process_hot: ScenarioStat,
    pub restart_disk_hot: ScenarioStat,
    pub single_body_edit: ScenarioStat,
    pub signature_conflict: ScenarioStat,
    pub gc_concurrent: ScenarioStat,
    pub gate_ms: f64,
    pub hard_gate_p95_ms: std::collections::BTreeMap<String, f64>,
    /// Worker private-bytes DELTA (informational) and the sampled PEAK
    /// across the bench (worker + max child), which is what the budget
    /// gates on (F798-R2-02). None off Windows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_private_bytes_delta: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_bytes_peak: Option<u64>,
    /// Max private-bytes peak observed by spawned children (request /
    /// gc-loop processes report their own sampled peaks).
    pub child_private_bytes_peak: u64,
    /// Attribution Line A (r5 §5.9, ac-core owned): the LANDED scalar
    /// frontend's own sampled peak on the frozen load, measured once in
    /// isolation. HARD-gated at `BUDGET_FRONTEND_TRANSIENT_PEAK_BYTES`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend_peak_bytes: Option<u64>,
    /// Line A target check (r5): frontend transient ≤ 224 MiB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontend_transient_target_met: Option<bool>,
    /// Line B target check (r5 §5.9, probe owned): overall peak MINUS the
    /// frontend attribution line ≤ 128 MiB — the increment this plan's
    /// pipeline itself adds. Without a frontend attribution measurement
    /// the WHOLE peak counts against this line (conservative).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_private_bytes_target_met: Option<bool>,
}

/// r5 Line B: the probe-owned residual = overall sampled peak minus the
/// frontend attribution line (same sampler family, absolute values). A
/// missing attribution measurement conservatively charges the whole peak
/// to the probe line.
pub fn probe_owned_residual_bytes(summary: &BenchSummary) -> Option<u64> {
    match (summary.private_bytes_peak, summary.frontend_peak_bytes) {
        (Some(overall), Some(frontend)) => Some(overall.saturating_sub(frontend)),
        (Some(overall), None) => Some(overall),
        _ => None,
    }
}

pub const SCENARIOS: [&str; 6] = [
    "fresh",
    "same-process-hot",
    "restart-disk-hot",
    "single-body-edit",
    "signature-conflict",
    "gc-concurrent",
];

pub fn run_six_scenarios(b: &BenchInputs) -> (Vec<SampleRow>, BenchSummary) {
    let mut rows = Vec::with_capacity(b.samples * 6);
    let mut child_peak: u64 = 0;
    let root = std::env::temp_dir().join(format!("av798-bench-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("bench root");
    let entry = "d_main";

    // 1. fresh: per-sample cold store.
    for i in 0..b.samples {
        let fresh_root = root.join(format!("fresh-{i}"));
        let mut d = Driver::with_store(&fresh_root, "bench-fresh").expect("store");
        let r = d.request(&b.load.source, entry, &[]);
        rows.push(row("fresh", i, 0, &r, Some(b.load.expected_main)));
        drop(d);
        let _ = std::fs::remove_dir_all(&fresh_root);
    }

    // 2. same-process-hot: one warm driver.
    {
        let hot_root = root.join("hot");
        let mut d = Driver::with_store(&hot_root, "bench-hot").expect("store");
        d.request(&b.load.source, entry, &[]); // warm
        for i in 0..b.samples {
            let r = d.request(&b.load.source, entry, &[]);
            rows.push(row(
                "same-process-hot",
                i,
                0,
                &r,
                Some(b.load.expected_main),
            ));
        }
    }

    // 3. restart-disk-hot: NEW PROCESS per sample over the warm root.
    {
        let src_file = root.join("load.at");
        std::fs::write(&src_file, &b.load.source).expect("write src");
        for i in 0..b.samples {
            let project = if i % 2 == 1 {
                format!("bench-xproj-{i}")
            } else {
                "bench-restart".to_string()
            };
            let out = std::process::Command::new(&b.self_exe)
                .args([
                    "request",
                    "--store",
                    root.join("hot").to_str().unwrap(),
                    "--src-file",
                    src_file.to_str().unwrap(),
                    "--entry",
                    entry,
                    "--project",
                    &project,
                ])
                .output()
                .expect("spawn request process");
            let stdout = String::from_utf8_lossy(&out.stdout);
            let v: serde_json::Value = serde_json::from_str(stdout.trim())
                .unwrap_or_else(|e| panic!("request proc stdout: {} ({})", e, stdout));
            child_peak = child_peak.max(v["private_bytes_peak"].as_u64().unwrap_or(0));
            rows.push(subprocess_row(
                "restart-disk-hot",
                i,
                &v,
                Some(b.load.expected_main),
            ));
            if !out.status.success() && v["ok"].as_bool() != Some(true) {
                // ok=false already reflected in the row.
            }
        }
    }

    // 4. single-body-edit: a CURRENT NEW revision per sample.
    {
        let edit_root = root.join("edit");
        let mut d = Driver::with_store(&edit_root, "bench-edit").expect("store");
        d.request(&b.load.source, entry, &[]); // warm base
        for i in 0..b.samples {
            let rev = load256::generate_with_sized(i as u32 + 1, b.revision_target);
            let r = d.request(&rev.source, entry, &[]);
            rows.push(row(
                "single-body-edit",
                i,
                i as u32 + 1,
                &r,
                Some(rev.expected_main),
            ));
        }
    }

    // 5. signature-conflict: ONLY the target frontend diagnostic counts.
    {
        let sig_root = root.join("sig");
        let mut d = Driver::with_store(&sig_root, "bench-sig").expect("store");
        d.request(&b.load.source, entry, &[]);
        for i in 0..b.samples {
            let r = d.request(&b.conflict, entry, &[]);
            let diag = r.diag.as_ref().map(|d| format!("{}:{}", d.stage, d.code));
            let target_ok = matches!(
                diag.as_deref(),
                Some(s) if s.starts_with("frontend:source.")
            );
            rows.push(SampleRow {
                scenario: "signature-conflict",
                sample: i,
                revision: 0,
                wall_ms: r.wall_ms,
                ok: target_ok,
                value: None,
                expected: None,
                diag,
                source_hash: r.source_hash,
                cache_hit_abc: *r.counters.get("cache_hit_abc").unwrap_or(&0),
                cache_miss_abc: *r.counters.get("cache_miss_abc").unwrap_or(&0),
                mir_lower_modules: *r.counters.get("mir_lower_modules").unwrap_or(&0),
                mir_units_persisted: *r.counters.get("mir_units_persisted").unwrap_or(&0),
                abc_units_emitted: *r.counters.get("abc_units_emitted").unwrap_or(&0),
                closure_encodes: *r.counters.get("closure_encodes").unwrap_or(&0),
                av_execute: *r.counters.get("av_execute").unwrap_or(&0),
                peak_staging_bytes: *r.counters.get("peak_staging_bytes").unwrap_or(&0),
                peak_hot_bytes: *r.counters.get("peak_hot_bytes").unwrap_or(&0),
                av_peak_slot_bytes: *r.counters.get("av_peak_slot_bytes").unwrap_or(&0),
            });
        }
    }

    // 6. gc-concurrent: a real GC process overlapping the requests.
    {
        let gc_root = root.join("gc");
        let mut d = Driver::with_store(&gc_root, "bench-gc").expect("store");
        d.request(&b.load.source, entry, &[]);
        let stop = root.join("gc-stop.flag");
        let _ = std::fs::remove_file(&stop);
        let child = std::process::Command::new(&b.self_exe)
            .args([
                "gc-loop",
                "--store",
                gc_root.to_str().unwrap(),
                "--stop-file",
                stop.to_str().unwrap(),
                "--interval-ms",
                "20",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn gc-loop");
        // Give the GC loop a moment to enter its first pass.
        std::thread::sleep(std::time::Duration::from_millis(50));
        for i in 0..b.samples {
            let r = d.request(&b.load.source, entry, &[]);
            rows.push(row("gc-concurrent", i, 0, &r, Some(b.load.expected_main)));
        }
        std::fs::write(&stop, b"1").ok();
        let gc_out = child.wait_with_output().expect("join gc-loop");
        let gc_err = String::from_utf8_lossy(&gc_out.stderr);
        for line in gc_err.lines() {
            if let Some(p) = line.strip_prefix("GCPeak=") {
                if let Ok(n) = p.trim().parse::<u64>() {
                    child_peak = child_peak.max(n);
                }
            }
        }
    }

    let mut summary = BenchSummary {
        gate_ms: b.gate_ms,
        child_private_bytes_peak: child_peak,
        ..Default::default()
    };
    for (field, scenario) in [
        (&mut summary.fresh, "fresh"),
        (&mut summary.same_process_hot, "same-process-hot"),
        (&mut summary.restart_disk_hot, "restart-disk-hot"),
        (&mut summary.single_body_edit, "single-body-edit"),
        (&mut summary.signature_conflict, "signature-conflict"),
        (&mut summary.gc_concurrent, "gc-concurrent"),
    ] {
        *field = stat(&rows, scenario);
    }
    summary.hard_gate_p95_ms = [
        (
            "single-body-edit".to_string(),
            summary.single_body_edit.p95_ms,
        ),
        (
            "signature-conflict".to_string(),
            summary.signature_conflict.p95_ms,
        ),
    ]
    .into_iter()
    .collect();
    let _ = std::fs::remove_dir_all(&root);
    (rows, summary)
}

/// Full validation + the frozen hard gate. Returns the failure list; an
/// empty list is the only passing state (the worker exits non-zero
/// otherwise — `--gate-ms 0` fails by construction).
pub fn validate_and_gate(
    rows: &[SampleRow],
    summary: &BenchSummary,
    samples: usize,
) -> Vec<String> {
    let mut failures = Vec::new();
    // Sample counts and per-row success per scenario.
    for scenario in SCENARIOS {
        let rs: Vec<&SampleRow> = rows.iter().filter(|r| r.scenario == scenario).collect();
        if rs.len() != samples {
            failures.push(format!("{}: {} samples != {}", scenario, rs.len(), samples));
        }
        let bad: Vec<usize> = rs.iter().filter(|r| !r.ok).map(|r| r.sample).collect();
        if !bad.is_empty() {
            failures.push(format!("{}: failing samples {:?}", scenario, bad));
        }
    }
    // Edit scenario: every sample must be a distinct revision with a
    // distinct input hash.
    {
        let rs: Vec<&SampleRow> = rows
            .iter()
            .filter(|r| r.scenario == "single-body-edit")
            .collect();
        let mut hashes: Vec<&str> = rs.iter().map(|r| r.source_hash.as_str()).collect();
        hashes.sort();
        hashes.dedup();
        if hashes.len() != rs.len() {
            failures.push(format!(
                "single-body-edit: only {} distinct input hashes across {} samples",
                hashes.len(),
                rs.len()
            ));
        }
    }
    // Hard gate: both gated paths P95 < gate_ms (0 always fails).
    for (name, p95) in &summary.hard_gate_p95_ms {
        if *p95 >= summary.gate_ms {
            failures.push(format!(
                "hard gate: {} P95 {:.1}ms >= {:.1}ms",
                name, p95, summary.gate_ms
            ));
        }
    }
    // r5 two-line worker-memory policy (plan §5.9 / F798-R12-04): both
    // lines are measured and HARD-gated — Line A is the ac-core-owned
    // frontend compile transient (isolated attribution measurement);
    // Line B is the probe-owned residual (overall peak minus Line A, or
    // the whole peak when attribution is missing). Neither line is
    // advisory, neither is waived; together they replace the former
    // single advisory ≤128 MiB worker target.
    if let Some(frontend) = summary.frontend_peak_bytes {
        if frontend > BUDGET_FRONTEND_TRANSIENT_PEAK_BYTES {
            failures.push(format!(
                "frontend transient line: isolated frontend peak {} > {} bytes (ac-core-owned line, r5 §5.9)",
                frontend, BUDGET_FRONTEND_TRANSIENT_PEAK_BYTES
            ));
        }
    }
    if let Some(residual) = probe_owned_residual_bytes(summary) {
        if residual > BUDGET_WORKER_PRIVATE_PEAK_BYTES {
            failures.push(format!(
                "probe-owned residual line: {} > {} bytes (overall minus frontend attribution, r5 §5.9)",
                residual, BUDGET_WORKER_PRIVATE_PEAK_BYTES
            ));
        }
    }
    // Managed partitions per row (§3 r2): staging 16 MiB / hot 8 MiB /
    // execution slots 8 MiB (sum = the 32 MiB managed envelope).
    for r in rows {
        if r.peak_staging_bytes > BUDGET_STAGING_BYTES {
            failures.push(format!(
                "{} sample {}: staging partition {} > 16MiB",
                r.scenario, r.sample, r.peak_staging_bytes
            ));
        }
        if r.peak_hot_bytes > BUDGET_HOT_BYTES {
            failures.push(format!(
                "{} sample {}: hot partition {} > 8MiB",
                r.scenario, r.sample, r.peak_hot_bytes
            ));
        }
        if r.av_peak_slot_bytes > BUDGET_EXEC_SLOT_BYTES {
            failures.push(format!(
                "{} sample {}: exec slot partition {} > 8MiB",
                r.scenario, r.sample, r.av_peak_slot_bytes
            ));
        }
    }
    failures
}

fn stat(rows: &[SampleRow], scenario: &str) -> ScenarioStat {
    use crate::observations::percentile_ms;
    let rs: Vec<&SampleRow> = rows.iter().filter(|r| r.scenario == scenario).collect();
    let walls: Vec<f64> = rs.iter().map(|r| r.wall_ms).collect();
    ScenarioStat {
        samples: rs.len(),
        p50_ms: percentile_ms(&walls, 50.0),
        p95_ms: percentile_ms(&walls, 95.0),
        p99_ms: percentile_ms(&walls, 99.0),
        min_ms: walls.iter().cloned().fold(f64::INFINITY, f64::min),
        max_ms: walls.iter().cloned().fold(0.0, f64::max),
        mean_ms: if walls.is_empty() {
            0.0
        } else {
            walls.iter().sum::<f64>() / walls.len() as f64
        },
        failures: rs.iter().filter(|r| !r.ok).count(),
    }
}

fn row(
    scenario: &'static str,
    sample: usize,
    revision: u32,
    r: &RequestOutcome,
    expected: Option<i32>,
) -> SampleRow {
    SampleRow {
        scenario,
        sample,
        revision,
        wall_ms: r.wall_ms,
        ok: r.ok() && r.value_i32() == expected,
        value: r.value_i32(),
        expected,
        diag: r.diag.as_ref().map(|d| format!("{}:{}", d.stage, d.code)),
        source_hash: r.source_hash.clone(),
        cache_hit_abc: *r.counters.get("cache_hit_abc").unwrap_or(&0),
        cache_miss_abc: *r.counters.get("cache_miss_abc").unwrap_or(&0),
        mir_lower_modules: *r.counters.get("mir_lower_modules").unwrap_or(&0),
        mir_units_persisted: *r.counters.get("mir_units_persisted").unwrap_or(&0),
        abc_units_emitted: *r.counters.get("abc_units_emitted").unwrap_or(&0),
        closure_encodes: *r.counters.get("closure_encodes").unwrap_or(&0),
        av_execute: *r.counters.get("av_execute").unwrap_or(&0),
        peak_staging_bytes: *r.counters.get("peak_staging_bytes").unwrap_or(&0),
        peak_hot_bytes: *r.counters.get("peak_hot_bytes").unwrap_or(&0),
        av_peak_slot_bytes: *r.counters.get("av_peak_slot_bytes").unwrap_or(&0),
    }
}

fn subprocess_row(
    scenario: &'static str,
    sample: usize,
    v: &serde_json::Value,
    expected: Option<i32>,
) -> SampleRow {
    let value = v["value"].as_i64().map(|x| x as i32);
    let ok = v["ok"].as_bool().unwrap_or(false) && value == expected;
    let counters = v["counters"].as_object();
    let c = |k: &str| -> u64 {
        counters
            .and_then(|m| m.get(k))
            .and_then(|x| x.as_u64())
            .unwrap_or(0)
    };
    SampleRow {
        scenario,
        sample,
        revision: 0,
        wall_ms: v["wall_ms"].as_f64().unwrap_or(f64::NAN),
        ok,
        value,
        expected,
        diag: v["diag"].as_str().map(|s| s.to_string()),
        source_hash: v["source_hash"].as_str().unwrap_or("").to_string(),
        cache_hit_abc: c("cache_hit_abc"),
        cache_miss_abc: c("cache_miss_abc"),
        mir_lower_modules: c("mir_lower_modules"),
        mir_units_persisted: c("mir_units_persisted"),
        abc_units_emitted: c("abc_units_emitted"),
        closure_encodes: c("closure_encodes"),
        av_execute: c("av_execute"),
        peak_staging_bytes: c("peak_staging_bytes"),
        peak_hot_bytes: c("peak_hot_bytes"),
        av_peak_slot_bytes: c("av_peak_slot_bytes"),
    }
}

/// The frozen hard gate kept for direct library callers (worker applies it
/// via [`validate_and_gate`]).
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
            failures.push(format!(
                "hard gate: {} P95 {:.1}ms >= {:.1}ms",
                scenario, p95, gate_ms
            ));
        }
        if rows.iter().any(|r| r.scenario == scenario && !r.ok) {
            failures.push(format!("hard gate: {} has failing samples", scenario));
        }
    }
    failures
}

/// Type re-export for the driver counters map.
pub type Counters = std::collections::BTreeMap<String, u64>;

#[allow(unused)]
fn unused_path_helper(_p: &Path) {}
