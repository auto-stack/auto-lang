//! Turnaround harness (PLAN-796 T-05, §5.5 / contract-decision D11).
//!
//! Stage traces exist from the first case; this module turns them into
//! the six measured scenarios with RAW samples (failures kept), the
//! hard gate (single-fn body edit on the 1 MiB reference module:
//! request → current correct inference/diagnostic published, P95 <
//! 1000 ms), and the report writer. Tool preparation (generating and
//! parsing the reference module, warming the store) is EXCLUDED from
//! the measured window and reported separately — it is never
//! subtracted from a user path that actually waited.

use ac_artifact_probe::store::ManagedStore;
use ac_sema_core::adapter::parse_into_input;
use ac_sema_core::query::QueryKind;
use ac_sema_core::{InferOutcome, SemaInput};
use crate::{lookup_only, open_store, publish_infer_result, run_inference, FetchOutcome};
use serde::Serialize;
use std::time::{Duration, Instant};

/// Reference-load generator identity (D11 freeze).
pub const REFERENCE_GENERATOR: &str = "gen796-1mi/v1";

/// The hard gate threshold (§5.5).
pub const HARD_GATE: Duration = Duration::from_millis(1000);

/// The number of raw samples per scenario (§5.5: 各30样本).
pub const SAMPLES: usize = 30;

// ---------------------------------------------------------------------------
// stage trace
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize)]
pub struct Stage {
    pub name: &'static str,
    pub dur_ns: u128,
    pub detail: String,
}

/// One measured sample: sequential, non-overlapping stages (nesting is
/// never double-counted — each stage covers exactly its own span).
#[derive(Clone, Debug, Serialize)]
pub struct Sample {
    pub scenario: &'static str,
    pub index: usize,
    pub stages: Vec<Stage>,
    pub total_ns: u128,
    pub compute_calls: u32,
    pub cache_hit: bool,
    pub io_bytes: u64,
    pub status: &'static str,
}

impl Sample {
    pub fn total(&self) -> Duration {
        Duration::from_nanos(self.total_ns as u64)
    }
}

fn pct(raw: &[u128], p: f64) -> u128 {
    let mut v = raw.to_vec();
    v.sort_unstable();
    if v.is_empty() {
        return 0;
    }
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[idx.min(v.len() - 1)]
}

#[derive(Clone, Debug, Serialize)]
pub struct ScenarioStats {
    pub scenario: &'static str,
    pub samples: usize,
    pub p50_ns: u128,
    pub p95_ns: u128,
    pub p99_ns: u128,
    pub max_ns: u128,
    pub compute_calls: u64,
    pub cache_hits: u64,
    pub statuses: Vec<(&'static str, usize)>,
}

pub fn stats_of(scenario: &'static str, samples: &[Sample]) -> ScenarioStats {
    let totals: Vec<u128> = samples.iter().map(|s| s.total_ns).collect();
    let mut statuses: Vec<(&'static str, usize)> = Vec::new();
    for s in samples {
        match statuses.iter_mut().find(|(st, _)| *st == s.status) {
            Some(e) => e.1 += 1,
            None => statuses.push((s.status, 1)),
        }
    }
    ScenarioStats {
        scenario,
        samples: samples.len(),
        p50_ns: pct(&totals, 0.50),
        p95_ns: pct(&totals, 0.95),
        p99_ns: pct(&totals, 0.99),
        max_ns: totals.iter().copied().max().unwrap_or(0),
        compute_calls: samples.iter().map(|s| s.compute_calls as u64).sum(),
        cache_hits: samples.iter().filter(|s| s.cache_hit).count() as u64,
        statuses,
    }
}

// ---------------------------------------------------------------------------
// reference load (tool preparation — excluded from measured windows)
// ---------------------------------------------------------------------------

/// Deterministically generate the 1 MiB-equivalent core module: `fns`
/// functions of straight-line arithmetic with per-function constants.
pub fn reference_module(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 1024);
    let mut i = 0u32;
    while out.len() < target_bytes {
        let a = i % 977 + 1;
        let b = i % 131 + 1;
        out.push_str(&format!(
            "fn f{i}(x int) int {{ let y{i} = x * {a} + {b}\nvar z{i} = y{i}\nwhile z{i} < {a}00 {{ z{i} = z{i} + {b} }}\nreturn z{i} + x }}\n"
        ));
        i += 1;
    }
    out
}

/// Parse the reference module (tool prep).
pub fn reference_input(src: &str) -> SemaInput {
    parse_into_input(src, "ref", "reference", auto_ac_prototype::source::SourceProfile::Core)
        .expect("reference module parses")
}

// ---------------------------------------------------------------------------
// the measured query path (one body-edit request → published result)
// ---------------------------------------------------------------------------

/// Apply a body edit to `fn_name` (bump a constant in the body — a
/// real semantic edit, not a whitespace change).
pub fn apply_body_edit(input: &mut SemaInput, fn_name: &str, rev: u32) {
    let f = input
        .fns
        .iter_mut()
        .find(|f| f.name == fn_name)
        .unwrap_or_else(|| panic!("fn {fn_name} present"));
    // replace the first IntLit in the body with rev (deterministic edit)
    let mut edited = false;
    for stmt in &mut f.body.stmts {
        if edit_first_intlit(stmt, &(rev as i32)) {
            edited = true;
            break;
        }
    }
    if !edited {
        // a body with no literal (shouldn't happen for the reference
        // module): append a constant return-adjusting let
        use ac_sema_core::input::{InputExpr, InputExprKind, InputStmt, Span};
        f.body.stmts.insert(
            0,
            InputStmt::Let {
                name: format!("rev{rev}"),
                ann: ac_sema_core::Ann::I32,
                mutable: false,
                value: InputExpr { kind: InputExprKind::IntLit(rev as i32), span: Span::new(0, 0) },
                span: Span::new(0, 0),
            },
        );
    }
}

fn edit_first_intlit(stmt: &mut ac_sema_core::input::InputStmt, v: &i32) -> bool {
    use ac_sema_core::input::{InputExpr, InputExprKind, InputStmt};
    fn edit_expr(e: &mut InputExpr, v: &i32) -> bool {
        match &mut e.kind {
            InputExprKind::IntLit(slot) => {
                *slot = *v;
                true
            }
            InputExprKind::Binary { lhs, rhs, .. } => edit_expr(lhs, v) || edit_expr(rhs, v),
            InputExprKind::Unary { operand, .. } => edit_expr(operand, v),
            InputExprKind::Call { args, .. } => args.iter_mut().any(|a| edit_expr(a, v)),
            _ => false,
        }
    }
    match stmt {
        InputStmt::Let { value, .. } => edit_expr(value, v),
        InputStmt::Assign { value, .. } => edit_expr(value, v),
        InputStmt::Return { value, .. } => edit_expr(value, v),
        InputStmt::If { cond, .. } => edit_expr(cond, v),
        InputStmt::While { cond, .. } => edit_expr(cond, v),
        _ => false,
    }
}

/// The full measured user path for one query on one input:
/// request(key build) → cache read+decode+dependency validation →
/// inference (on miss; the real solver) → publish. Returns the sample
/// with per-stage spans.
pub fn measure_query(
    store: &ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    scenario: &'static str,
    index: usize,
) -> Sample {
    let mut stages: Vec<Stage> = Vec::new();
    let mut compute_calls = 0u32;
    let mut cache_hit = false;
    let mut io_bytes = 0u64;
    let status: &'static str;

    let t_total = Instant::now();

    // stage 1: request — key build over the input
    let t = Instant::now();
    let key = ac_sema_core::SemaQueryKey::build(query, input);
    stages.push(Stage {
        name: "request",
        dur_ns: t.elapsed().as_nanos(),
        detail: key.canonical().chars().take(96).collect(),
    });

    // stage 2: cache read + decode + FULL dependency re-validation
    let t = Instant::now();
    let fetch = lookup_only(store, input, query).expect("lookup succeeds");
    let (stale_reason, hit_now) = match &fetch {
        FetchOutcome::Hit { page, .. } => {
            io_bytes += page_bytes(page);
            (None, true)
        }
        FetchOutcome::Stale { reason } => (Some(reason.clone()), false),
        FetchOutcome::Miss => (None, false),
    };
    stages.push(Stage {
        name: "cache-read-validate",
        dur_ns: t.elapsed().as_nanos(),
        detail: stale_reason.unwrap_or_else(|| if hit_now { "hit".into() } else { "miss".into() }),
    });
    cache_hit = hit_now;

    if !hit_now {
        // stage 3: real inference (the new solver, the edited function)
        let t = Instant::now();
        let fn_name = match query {
            QueryKind::InferBody { fn_name } => fn_name.clone(),
            _ => String::new(),
        };
        let outcome = run_inference(input, &fn_name);
        let infer_ns = t.elapsed().as_nanos();
        compute_calls += 1;
        match outcome {
            InferOutcome::Ok(sum) | InferOutcome::Diags(sum) => {
                // stage 4: publish the fresh result
                let t = Instant::now();
                let art =
                    publish_infer_result(store, input, query, &sum, Some("turnaround")).expect("publish");
                let _ = &art;
                stages.push(Stage {
                    name: "publish",
                    dur_ns: t.elapsed().as_nanos(),
                    detail: "published".into(),
                });
                io_bytes += estimate_summary_bytes(&sum);
                status = "published";
            }
            InferOutcome::ResourceExhausted(_) => status = "resource-exhausted",
            InferOutcome::Cancelled => status = "cancelled",
        }
        stages.insert(
            2,
            Stage { name: "infer", dur_ns: infer_ns, detail: "new-solver".into() },
        );
    } else {
        status = "hit";
    }

    Sample {
        scenario,
        index,
        stages,
        total_ns: t_total.elapsed().as_nanos(),
        compute_calls,
        cache_hit,
        io_bytes,
        status,
    }
}

fn page_bytes(p: &ac_sema_contract::SemaPage) -> u64 {
    ac_sema_contract::encode_page(p).map(|b| b.len() as u64).unwrap_or(0)
}

fn estimate_summary_bytes(s: &ac_sema_core::InferSummary) -> u64 {
    (s.facts.bindings.len() * 48 + s.facts.expr_types.len() * 40) as u64
}

// ---------------------------------------------------------------------------
// the six scenarios (§5.5)
// ---------------------------------------------------------------------------

pub struct Harness {
    pub store_root: std::path::PathBuf,
    pub reference_src: String,
}

impl Harness {
    pub fn new(store_root: std::path::PathBuf) -> Self {
        Harness { store_root, reference_src: reference_module(1024 * 1024) }
    }

    /// Scenario 1 — no-op: identical input, repeated queries → hits.
    pub fn scenario_no_op(&self, store: &ManagedStore) -> Vec<Sample> {
        let input = reference_input(&self.reference_src);
        let q = QueryKind::InferBody { fn_name: "f0".into() };
        // warm (first publish outside measurement is NOT done: first
        // sample computes, the rest hit — samples keep both statuses)
        (0..SAMPLES)
            .map(|i| measure_query(store, &input, &q, "no-op", i))
            .collect()
    }

    /// Scenario 2 — local body edit (THE HARD GATE): each sample edits
    /// f0's body (new revision) and re-queries.
    pub fn scenario_local_body_edit(&self, store: &ManagedStore) -> Vec<Sample> {
        let mut input = reference_input(&self.reference_src);
        let q = QueryKind::InferBody { fn_name: "f0".into() };
        (0..SAMPLES)
            .map(|i| {
                apply_body_edit(&mut input, "f0", i as u32 + 1);
                measure_query(store, &input, &q, "local-body-edit", i)
            })
            .collect()
    }

    /// Scenario 3 — signature/field-set change: f1's SIGNATURE flips a
    /// param annotation; f0 (its caller via a synthetic call) queries.
    pub fn scenario_sig_change(&self, store: &ManagedStore) -> Vec<Sample> {
        let mut input = reference_input(&self.reference_src);
        // make f0 call f1 so f0's manifest reads f1's signature
        make_f0_call_f1(&mut input);
        let q = QueryKind::InferBody { fn_name: "f0".into() };
        (0..SAMPLES)
            .map(|i| {
                // flip f1's param between i32 and bool each revision —
                // f0's key components stay identical (its own body/sig
                // unchanged), the manifest validation must catch it
                flip_f1_param(&mut input, i % 2 == 0);
                measure_query(store, &input, &q, "sig-change", i)
            })
            .collect()
    }

    /// Scenario 4 — restart disk-hot: a FRESH store handle (new SQLite
    /// connection = process-fresh caches; OS page cache warm) per
    /// sample, hitting the pre-published result. Cross-process proof
    /// lives in sema_cache_first_case (real child processes).
    pub fn scenario_restart_disk_hot(&self) -> Vec<Sample> {
        let input = reference_input(&self.reference_src);
        let q = QueryKind::InferBody { fn_name: "f0".into() };
        {
            let store = open_store(&self.store_root).unwrap();
            let _ = measure_query(&store, &input, &q, "warmup", 0); // ensure published
        }
        (0..SAMPLES)
            .map(|i| {
                let store = open_store(&self.store_root).unwrap(); // fresh handle
                measure_query(&store, &input, &q, "restart-disk-hot", i)
            })
            .collect()
    }

    /// Scenario 5 — cross-project: the same logical content queried
    /// under a second project context → shared hits.
    pub fn scenario_cross_project(&self, store: &ManagedStore) -> Vec<Sample> {
        // package identity differs → different key; recompute happens
        // once per package, hits after — measured honestly
        let src2 = self.reference_src.replace("fn f0(", "fn g0(");
        let _ = src2;
        let input = reference_input(&self.reference_src);
        let q = QueryKind::InferBody { fn_name: "f3".into() };
        let proj_b: SemaInput = {
            let mut i2 = input.clone();
            i2.package = "proj-b".into();
            i2
        };
        let q2 = QueryKind::InferBody { fn_name: "f3".into() };
        // warm proj-b once OUTSIDE samples
        let _ = measure_query(store, &proj_b, &q2, "warmup", 0);
        (0..SAMPLES)
            .map(|i| measure_query(store, &proj_b, &q2, "cross-project", i))
            .collect()
    }

    /// Scenario 6 — GC concurrent: queries measured while a GC pass
    /// runs on the same store.
    pub fn scenario_gc_concurrent(&self, store: &ManagedStore) -> Vec<Sample> {
        let input = reference_input(&self.reference_src);
        let q = QueryKind::InferBody { fn_name: "f2".into() };
        let _ = measure_query(store, &input, &q, "warmup", 0);
        (0..SAMPLES)
            .map(|i| {
                let store_ref = &store;
                let input_ref = &input;
                let q_ref = &q;
                std::thread::scope(|s| {
                    let gc = s.spawn(move || {
                        let _ = ac_artifact_probe::store::run_gc(
                            store_ref,
                            ac_artifact_probe::store::GcConfig { high: 1, low: 0 },
                        );
                    });
                    let sample = measure_query(store_ref, input_ref, q_ref, "gc-concurrent", i);
                    gc.join().unwrap();
                    sample
                })
            })
            .collect()
    }
}

impl Harness {
    /// Extra report row (§5.5 另报) — full cold: an EMPTY store, the
    /// first query does the whole path cold (parse/tool prep excluded
    /// per D11; subsequent samples verify warm behaviour).
    pub fn scenario_full_cold(&self) -> Vec<Sample> {
        let cold_root = self
            .store_root
            .parent()
            .map(|p| p.join("store-cold"))
            .unwrap_or_else(|| std::env::temp_dir().join("ac796-cold"));
        let _ = std::fs::remove_dir_all(&cold_root);
        let store = open_store(&cold_root).unwrap();
        let input = reference_input(&self.reference_src);
        let q = QueryKind::InferBody { fn_name: "f0".into() };
        (0..SAMPLES)
            .map(|i| measure_query(&store, &input, &q, "full-cold", i))
            .collect()
    }

    /// Extra report row (§5.5 另报) — wide invalidation: 20 dependents
    /// share one callee; each revision flips the SHARED callee's
    /// signature, so every dependent's key (called-sigs-fp) changes —
    /// every sample recomputes and republishes.
    pub fn scenario_wide_invalidation(&self, store: &ManagedStore) -> Vec<Sample> {
        let mut input = reference_input(&self.reference_src);
        let shared = "f9";
        for i in 0..20u32 {
            let caller = format!("wide{i}");
            make_caller(&mut input, &caller.clone(), shared, "x");
        }
        (0..SAMPLES)
            .map(|i| {
                flip_f1_param_named(&mut input, shared, i % 2 == 0);
                let caller = format!("wide{}", i % 20);
                let q = QueryKind::InferBody { fn_name: caller };
                measure_query(store, &input, &q, "wide-invalidation", i)
            })
            .collect()
    }
}

fn make_caller(input: &mut SemaInput, caller: &str, callee: &str, arg: &str) {
    use ac_sema_core::input::{InputExpr, InputExprKind, InputStmt, Span};
    // ensure the caller exists (clone a template fn under the new name)
    if input.lookup_fn(caller).is_none() {
        let mut tpl = input.fns[0].clone();
        tpl.name = caller.to_string();
        tpl.body.stmts.clear();
        input.fns.push(tpl);
    }
    let f = input.fns.iter_mut().find(|f| f.name == caller).unwrap();
    f.body.stmts.push(InputStmt::Return {
        value: InputExpr {
            kind: InputExprKind::Call {
                callee: callee.to_string(),
                args: vec![InputExpr {
                    kind: InputExprKind::Var(arg.to_string()),
                    span: Span::new(0, 0),
                }],
            },
            span: Span::new(0, 0),
        },
        span: Span::new(0, 0),
    });
}

fn flip_f1_param_named(input: &mut SemaInput, name: &str, to_bool: bool) {
    let f = input.fns.iter_mut().find(|f| f.name == name).unwrap();
    f.params[0].1 = if to_bool { ac_sema_core::Ann::Bool } else { ac_sema_core::Ann::I32 };
}

fn make_f0_call_f1(input: &mut SemaInput) {
    use ac_sema_core::input::{InputExpr, InputExprKind, InputStmt, Span};
    let f0 = input.fns.iter_mut().find(|f| f.name == "f0").unwrap();
    f0.body.stmts.push(InputStmt::Return {
        value: InputExpr {
            kind: InputExprKind::Call {
                callee: "f1".into(),
                args: vec![InputExpr { kind: InputExprKind::Var("x".into()), span: Span::new(0, 0) }],
            },
            span: Span::new(0, 0),
        },
        span: Span::new(0, 0),
    });
}

fn flip_f1_param(input: &mut SemaInput, to_bool: bool) {
    let f1 = input.fns.iter_mut().find(|f| f.name == "f1").unwrap();
    f1.params[0].1 = if to_bool { ac_sema_core::Ann::Bool } else { ac_sema_core::Ann::I32 };
}

// ---------------------------------------------------------------------------
// report
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct MachineInfo {
    pub os: String,
    pub arch: String,
    pub parallelism: usize,
    pub rustc: String,
    pub generator: &'static str,
    pub samples_per_scenario: usize,
    pub hard_gate_ms: u64,
}

pub fn machine_info() -> MachineInfo {
    let rustc = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".into());
    MachineInfo {
        os: format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH),
        arch: std::env::consts::ARCH.into(),
        parallelism: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0),
        rustc,
        generator: REFERENCE_GENERATOR,
        samples_per_scenario: SAMPLES,
        hard_gate_ms: HARD_GATE.as_millis() as u64,
    }
}

/// Write the raw-samples + stats report (JSON) and the human-readable
/// markdown summary. Raw samples are NEVER filtered — all statuses kept.
pub fn write_report(
    dir: &std::path::Path,
    machine: &MachineInfo,
    all: &[(&'static str, Vec<Sample>)],
) -> std::io::Result<std::path::PathBuf> {
    std::fs::create_dir_all(dir)?;
    let stats: Vec<ScenarioStats> =
        all.iter().map(|(name, s)| stats_of(name, s)).collect();
    #[derive(Serialize)]
    struct Report<'a> {
        machine: &'a MachineInfo,
        stats: Vec<ScenarioStats>,
        raw: &'a [(&'static str, Vec<Sample>)],
    }
    let json_path = dir.join("turnaround.json");
    std::fs::write(
        &json_path,
        serde_json::to_string_pretty(&Report { machine, stats: stats.clone(), raw: all }).unwrap(),
    )?;
    let mut md = String::from("# PLAN-796 T-05 周转报告\n\n");
    md.push_str(&format!(
        "- 机器: {} | 并行度 {} | rustc {}\n- 参考负载: {}（1 MiB 等价 core 模块，确定性生成）\n- 每场景原始样本: {}（含失败/超时，不删样本）\n- 硬门: 单函数 body 编辑 请求→正确推导/诊断发布 P95 < {} ms\n\n",
        machine.os,
        machine.parallelism,
        machine.rustc,
        machine.generator,
        machine.samples_per_scenario,
        machine.hard_gate_ms,
    ));
    md.push_str("| 场景 | 样本 | P50 (ms) | P95 (ms) | P99 (ms) | max (ms) | compute | hits | 状态分布 |\n|---|---|---|---|---|---|---|---|---|\n");
    for s in &stats {
        md.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {} | {} | {} |\n",
            s.scenario,
            s.samples,
            s.p50_ns as f64 / 1e6,
            s.p95_ns as f64 / 1e6,
            s.p99_ns as f64 / 1e6,
            s.max_ns as f64 / 1e6,
            s.compute_calls,
            s.cache_hits,
            s.statuses
                .iter()
                .map(|(st, n)| format!("{st}:{n}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let hard = stats
        .iter()
        .find(|s| s.scenario == "local-body-edit")
        .map(|s| s.p95_ns)
        .unwrap_or(u128::MAX);
    md.push_str(&format!(
        "\n硬门判定: local-body-edit P95 = {:.2} ms → {}\n",
        hard as f64 / 1e6,
        if hard < HARD_GATE.as_nanos() { "PASS" } else { "FAIL" }
    ));
    let md_path = dir.join("turnaround.md");
    std::fs::write(&md_path, md)?;
    Ok(md_path)
}
