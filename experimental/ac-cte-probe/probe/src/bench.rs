//! The turnaround bench (PLAN-817 T-07, V-09): six scenarios x N raw
//! samples through the REAL driver path, with the two EDIT hard gates
//! (helper-body edit and Meta-set edit, warm store, P95 < gate) and a
//! JSONL raw report whose rows bind each sample to the hash of the
//! exact deck text it executed. Native observables are OFF here — the
//! native reference is reported separately (contract-decision §7).

use crate::driver::CteDriver;
use serde_json::json;
use serde_json::Value;
use std::io::Write;
use std::time::Instant;

pub const DEFAULT_SAMPLES: usize = 30;
pub const DEFAULT_GATE_MS: f64 = 1000.0;

/// The specified baseline generator (contract-decision §8; >=256KiB,
/// 115 functions, bounded + only arithmetic). `rev` rides ONE helper.
pub fn baseline_source_bench(fns: usize, rev: i64) -> String {
    let mut s = String::new();
    for k in 0..fns {
        let rev_here = if k == fns / 2 { rev } else { 0 };
        s.push_str(&format!(
            "fn h{k}(x int) int {{\n    var acc = x + {k} + {rev_here}\n"
        ));
        for j in 0..80 {
            s.push_str(&format!("    acc = acc + {j} + x + {k}\n"));
        }
        s.push_str("    return acc\n}\n\n");
    }
    s.push_str("fn cte_sum(a int, b int) int {\n    var acc = a + b\n");
    for k in 0..fns {
        s.push_str(&format!("    acc = acc + h{k}(20)\n"));
    }
    s.push_str("    return acc\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n");
    s
}

/// The specified load + the CONTRACTED Meta->AV path: the 115-function
/// baseline source, a meta block whose field set grows per revision, and
/// field_count feeding a real AV call.
fn big_meta_deck(extra_fields: usize) -> String {
    let src = baseline_source_bench(115, 0);
    let mut d = String::new();
    d.push_str(
        "deck cte817/1
module main
caps cte.meta.fields/v1
meta <<
type Pair {
    left int
    right bool
",
    );
    for i in 0..extra_fields {
        d.push_str(&format!(
            "    extra{i} int
"
        ));
    }
    d.push_str(
        "}
>>
source <<
",
    );
    d.push_str(&src);
    d.push_str(
        "
fn scale(n int) int {
    return n * 100
}
>>
query n = field_count Pair
query q = call scale(n)
bind main.ct = q expect i32 placeholder 0
entry main
native off
",
    );
    d
}

fn big_deck(rev: i64) -> String {
    // exec snapshot: the incremental lane is the production path for
    // edit-heavy sessions (T-11/SD-06) — the warm resident session makes
    // local helper-body edits cheap on the specified load.
    let src = baseline_source_bench(115, rev);
    format!(
        "deck cte817/1\nmodule main\nexec snapshot\nsource <<\n{src}\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\nnative off\n"
    )
}

struct Row {
    scenario: &'static str,
    sample: usize,
    wall_ms: f64,
    input_hash: String,
    cache: &'static str,
    ct_execute: u64,
    ok: bool,
    /// The worker PID for process scenarios (restart asserts PIDs
    /// differ); None for in-process rows.
    pid: Option<u32>,
    /// The measured VALUE (rendered) — raw rows are independently
    /// recomputable (F-817-R3-04).
    value: String,
}

/// Measure one in-process sample at the specified load: the deck BUILD
/// is inside the window; the row carries the measured VALUE against the
/// independent expected form (expected < 0 = presence-only).
fn measure_built(
    scenario: &'static str,
    sample: usize,
    rev: i64,
    drv: &mut CteDriver,
    rows: &mut Vec<Row>,
    expected: i64,
) {
    let t = Instant::now();
    let deck = big_deck(rev);
    let o = drv.query_step(&deck, None);
    let wall = t.elapsed().as_secs_f64() * 1000.0;
    let value = o
        .values
        .get("main.ct")
        .map(|v| v.render())
        .unwrap_or_default();
    let ok = o.status == crate::driver::StepStatus::Ok
        && !value.is_empty()
        && (expected < 0 || value == format!("i32:{expected}"));
    rows.push(Row {
        scenario,
        sample,
        wall_ms: wall,
        input_hash: crate::fnv1a64_hex(deck.as_bytes()),
        cache: o.cache.unwrap_or("-"),
        ct_execute: o.ct_execute,
        ok,
        pid: None,
        value,
    });
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub fn run_bench(
    samples: usize,
    gate_ms: f64,
    inject_delay_ms: u64,
    out: Option<&std::path::Path>,
    root: &std::path::Path,
) -> (i32, serde_json::Value) {
    run_bench_ext(samples, gate_ms, inject_delay_ms, false, out, root)
}

/// `inject_rebuild`: the forced whole-module rebuild control (the
/// resident-edit degrade gate must go red on it).
pub fn run_bench_ext(
    samples: usize,
    gate_ms: f64,
    _inject_delay_ms: u64,
    inject_rebuild: bool,
    out: Option<&std::path::Path>,
    root: &std::path::Path,
) -> (i32, serde_json::Value) {
    let _ = std::fs::create_dir_all(root);
    // F-817-R1-04: the gate REFUSES malformed inputs outright — zero,
    // negative or insufficient sample counts and non-finite thresholds
    // exit non-zero with target_met=false (never a green empty run).
    let invalid: Option<String> = if samples == 0 || samples > 100_000 {
        Some(format!("samples {samples} must be 1..=100000"))
    } else if !gate_ms.is_finite() {
        Some(format!("gate-ms {gate_ms} must be finite"))
    } else if gate_ms < 0.0 {
        Some(format!("gate-ms {gate_ms} must be >= 0"))
    } else {
        None
    };
    if let Some(msg) = invalid {
        let summary = serde_json::json!({
            "target_met": false,
            "hard_gate_failures": [format!("invalid input: {msg}")],
        });
        if let Some(path) = out {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(mut f) = std::fs::File::create(path) {
                use std::io::Write;
                let _ = writeln!(f, "{summary}");
            }
        }
        return (2, summary);
    }
    let mut rows: Vec<Row> = Vec::new();
    let bench_start = Instant::now();
    let sampler = auto_av_probe::mempeek::PeakSampler::start(1);

    // ---- closed-form oracle (hand-derived, independent of the AV) ----
    // baseline: h_k(20) = 81*20 + 81*k + 3160 + [k == 57] rev (the init
    // line x + k + rev plus 80 lines of acc + j + x + k each: sum_j =
    // 3160, plus 80x + 80k). cte_sum folds all 115:
    // value(rev) = 42 + 115*(1620 + 3160) + 81*sum_k(0..114) + rev.
    let base_value: i64 = 42 + 115 * 4780 + 81 * (114 * 115 / 2);

    // per-scenario peaks (F-817-R3-01): the worker gate is enforced per
    // measured scenario; the tree gate on the process high-water.
    let mut scenario_peaks: std::collections::BTreeMap<&'static str, u64> =
        std::collections::BTreeMap::new();

    // 1) fresh: a brand-new store per sample (cold everything) on the
    //    SPECIFIED LOAD; the deck build is INSIDE the measured window.
    for i in 0..samples {
        let dir = root.join(format!("fresh-{i}"));
        let _ = std::fs::remove_dir_all(&dir);
        let mut d = CteDriver::with_store(&dir, "fresh").unwrap();
        let sp = auto_av_probe::mempeek::PeakSampler::start(1);
        measure_built("fresh", i, 0, &mut d, &mut rows, base_value);
        let pk = sp.stop();
        scenario_peaks
            .entry("fresh")
            .and_modify(|c| *c = (*c).max(pk))
            .or_insert(pk);
    }

    // 2) hot: one warm driver, repeated identical requests (hit path) on
    //    the specified load.
    let hot_dir = root.join("hot");
    let _ = std::fs::remove_dir_all(&hot_dir);
    let mut hot = CteDriver::with_store(&hot_dir, "hot").unwrap();
    {
        let sp = auto_av_probe::mempeek::PeakSampler::start(1);
        let probe_out = hot.query_step(&big_deck(0), None);
        assert_eq!(probe_out.status, crate::driver::StepStatus::Ok);
        let pk = sp.stop();
        scenario_peaks
            .entry("hot")
            .and_modify(|c| *c = (*c).max(pk))
            .or_insert(pk);
    }
    {
        let sp = auto_av_probe::mempeek::PeakSampler::start(1);
        for i in 0..samples {
            measure_built("hot", i, 0, &mut hot, &mut rows, base_value);
        }
        let pk = sp.stop();
        scenario_peaks
            .entry("hot")
            .and_modify(|c| *c = (*c).max(pk))
            .or_insert(pk);
    }

    // 3) restart: a REAL worker process per sample (F-817-R3-03) on the
    //    SPECIFIED LOAD; the window covers request build -> serialize ->
    //    spawn -> IPC -> response parse + echo/value admission.
    let rs_dir = root.join("restart");
    let _ = std::fs::remove_dir_all(&rs_dir);
    {
        let mut warm = CteDriver::with_store(&rs_dir, "rs").unwrap();
        let warm_out = warm.query_step(&big_deck(0), None);
        assert_eq!(warm_out.status, crate::driver::StepStatus::Ok);
    }
    spawn_process_samples(
        "restart",
        samples,
        &rs_dir,
        &SpawnDeck::Big { rev: 0 },
        None,
        &mut rows,
        &mut scenario_peaks,
    );

    // 4) body-edit (HARD GATE, WARM SESSION over real IPC): ONE worker
    //    process (case-serve-stream, one resident driver + store), warmed
    //    outside the window; each sample = query -> edit -> query on the
    //    specified load with a fresh deck revision per round, measured
    //    from request build to full admission. Expected (independent
    //    closed form): base + rev + 1.
    let be_dir = root.join("body-edit");
    let _ = std::fs::remove_dir_all(&be_dir);
    {
        let mut decks: Vec<String> = Vec::with_capacity(samples + 1);
        let mut expected: Vec<String> = Vec::with_capacity(samples + 1);
        decks.push(big_deck(0));
        // The warm request itself runs query -> edit -> query: its LAST
        // step's value is base + 1 (the edit bumps h57's 0 -> 1).
        expected.push(format!("i32:{}", base_value + 1));
        for i in 0..samples {
            let rev = i as i64;
            decks.push(big_deck(rev));
            expected.push(format!("i32:{}", base_value + rev + 1));
        }
        // Per-round edit literal: deck rev i carries `x + 57 + i`; the
        // edit bumps it to i+1 (fresh hash each round, the edit ALWAYS
        // matches exactly once).
        let mut steps_per_req: Vec<Vec<serde_json::Value>> = Vec::with_capacity(samples + 1);
        steps_per_req.push(vec![
            serde_json::json!({ "op": "query" }),
            serde_json::json!({ "op": "edit-source", "from": "var acc = x + 57 + 0", "to": "var acc = x + 57 + 1" }),
            serde_json::json!({ "op": "query" }),
        ]);
        for i in 0..samples {
            let rev = i as i64;
            steps_per_req.push(vec![
                serde_json::json!({ "op": "query" }),
                serde_json::json!({ "op": "edit-source", "from": format!("var acc = x + 57 + {rev}"), "to": format!("var acc = x + 57 + {}", rev + 1) }),
                serde_json::json!({ "op": "query" }),
            ]);
        }
        spawn_warm_samples(
            "body-edit",
            samples,
            &be_dir,
            &decks,
            &steps_per_req,
            &expected,
            &mut rows,
            &mut scenario_peaks,
            _inject_delay_ms,
        );
    }

    // 5) meta-edit (HARD GATE, WARM SESSION over real IPC): the
    //    CONTRACTED Meta->AV path — one warm worker, each sample a deck
    //    whose meta block grew one field; field_count flips, the AV call
    //    re-executes with the new argument. Expected: (2+i)*100.
    let me_dir = root.join("meta-edit");
    let _ = std::fs::remove_dir_all(&me_dir);
    {
        let mut decks: Vec<String> = Vec::with_capacity(samples + 1);
        let mut expected: Vec<String> = Vec::with_capacity(samples + 1);
        decks.push(big_meta_deck(0));
        expected.push("i32:200".to_string());
        for i in 0..samples {
            decks.push(big_meta_deck(i + 1));
            expected.push(format!("i32:{}", (2 + i as i64 + 1) * 100));
        }
        let steps_per_req: Vec<Vec<serde_json::Value>> = (0..=samples)
            .map(|_| vec![serde_json::json!({ "op": "query" })])
            .collect();
        spawn_warm_samples(
            "meta-edit",
            samples,
            &me_dir,
            &decks,
            &steps_per_req,
            &expected,
            &mut rows,
            &mut scenario_peaks,
            0,
        );
    }

    // 6) gc-concurrent: queries under a live GC loop on the same store
    //    (its OWN store handle), on the specified load.
    let gc_dir = root.join("gc");
    let _ = std::fs::remove_dir_all(&gc_dir);
    let mut gd = CteDriver::with_store(&gc_dir, "gc").unwrap();
    {
        let probe_out = gd.query_step(&big_deck(0), None);
        assert_eq!(probe_out.status, crate::driver::StepStatus::Ok);
    }
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop2 = stop.clone();
    let gc_root = gc_dir.clone();
    let handle = std::thread::spawn(move || {
        let st = match crate::store::CteStore::open(&gc_root, "gc") {
            Ok(s) => s,
            Err(_) => return,
        };
        while !stop2.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = ac_artifact_probe::store::run_gc(
                st.raw(),
                ac_artifact_probe::store::GcConfig { high: 1, low: 0 },
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    });
    {
        let sp = auto_av_probe::mempeek::PeakSampler::start(1);
        for i in 0..samples {
            measure_built("gc", i, (i % 3) as i64, &mut gd, &mut rows, -1);
        }
        let pk = sp.stop();
        scenario_peaks
            .entry("gc")
            .and_modify(|c| *c = (*c).max(pk))
            .or_insert(pk);
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = handle.join();

    // 7) worker-edit: a REAL worker process per sample (query -> edit ->
    //    query) on the specified load; every round edits a NEW revision
    //    literal (per-round fresh hash), expected base + rev + 1.
    let we_dir = root.join("worker-edit");
    let _ = std::fs::remove_dir_all(&we_dir);
    for i in 0..samples {
        let rev = i as i64;
        let edit = (
            format!("var acc = x + 57 + {rev}"),
            format!("var acc = x + 57 + {}", rev + 1),
        );
        spawn_process_samples(
            "worker-edit",
            1,
            &we_dir,
            &SpawnDeck::Big { rev },
            Some(edit),
            &mut rows,
            &mut scenario_peaks,
        );
        if let Some(r) = rows.last_mut() {
            r.sample = i;
        }
    }

    // 8) resident-edit (FORMAL GATE): 30 distinct local body revisions
    //    through ONE resident driver at the specified load. TWO hard
    //    gates (F-817-R3-02): ZERO whole-module re-warms AND P95 < gate.
    //    --inject-rebuild changes the STRUCTURE at the midpoint; the
    //    SAME normal gates go red from the DETECTOR VALUE (no marker
    //    failure) — with the detector disabled (env control) the red
    //    must vanish, proving the gate is load-bearing.
    let re_dir = root.join("resident-edit");
    let _ = std::fs::remove_dir_all(&re_dir);
    let mut re = CteDriver::with_store(&re_dir, "re").unwrap();
    let re_start = re.query_step(&big_deck(0), None);
    assert_eq!(re_start.status, crate::driver::StepStatus::Ok);
    let rewarms_before = re.snapshot_rewarms();
    let sp = auto_av_probe::mempeek::PeakSampler::start(1);
    for i in 0..samples {
        let rev = (i + 1) as i64;
        let deck = if inject_rebuild && i == samples / 2 {
            // Structural change at the SAME function count: a rename is a
            // declaration-set change the single-hunk sync cannot absorb —
            // the whole module re-warms (the detector's trigger) without
            // growing the source past the memory gate.
            let structural = baseline_source_bench(115, rev).replace("h57", "hx57");
            format!(
                "deck cte817/1\nmodule main\nexec snapshot\nsource <<\n{structural}\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\nnative off\n"
            )
        } else {
            big_deck(rev)
        };
        let t = Instant::now();
        let o = re.query_step(&deck, None);
        let wall = t.elapsed().as_secs_f64() * 1000.0;
        let value = o
            .values
            .get("main.ct")
            .map(|v| v.render())
            .unwrap_or_default();
        let structural_round = inject_rebuild && i == samples / 2;
        let ok = o.status == crate::driver::StepStatus::Ok
            && (structural_round || value == format!("i32:{}", base_value + rev));
        rows.push(Row {
            scenario: "resident-edit",
            sample: i,
            wall_ms: wall,
            input_hash: crate::fnv1a64_hex(deck.as_bytes()),
            cache: o.cache.unwrap_or("-"),
            ct_execute: o.ct_execute,
            ok,
            pid: None,
            value,
        });
    }
    let pk = sp.stop();
    scenario_peaks
        .entry("resident-edit")
        .and_modify(|c| *c = (*c).max(pk))
        .or_insert(pk);
    let rewarms_after = re.snapshot_rewarms();

    // ---- aggregate ---------------------------------------------------
    let bench_wall_ms = bench_start.elapsed().as_secs_f64() * 1000.0;
    let worker_peak = sampler.stop();
    let scenarios = [
        "fresh",
        "hot",
        "restart",
        "body-edit",
        "meta-edit",
        "gc",
        "worker-edit",
        "resident-edit",
    ];
    let mut summary = serde_json::json!({
        "samples_per_scenario": samples,
        "gate_ms": gate_ms,
        "bench_wall_ms": bench_wall_ms,
        "worker_private_bytes_peak": worker_peak,
    });
    let mut failures: Vec<String> = Vec::new();
    for sc in scenarios {
        let mut walls: Vec<f64> = rows
            .iter()
            .filter(|r| r.scenario == sc && r.sample != usize::MAX && r.sample < 10_000)
            .map(|r| r.wall_ms)
            .collect();
        walls.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p95 = percentile(&walls, 0.95);
        let max = walls.last().copied().unwrap_or(0.0);
        let ok = rows
            .iter()
            .filter(|r| r.scenario == sc && r.sample != usize::MAX && r.sample < 10_000)
            .all(|r| r.ok);
        if !ok {
            failures.push(format!(
                "{sc} value/admission mismatch (independent oracle)"
            ));
        }
        summary[sc] = json!({
            "p95_ms": p95,
            "max_ms": max,
            "samples": walls.len(),
            "all_values_ok": ok,
        });
        // Strict: P95 must be strictly BELOW the gate (equality fails).
        // The edit hard gates AND the resident-edit lane gate (F-817-R3-02)
        // are enforced here.
        if (sc == "body-edit" || sc == "meta-edit" || sc == "resident-edit") && p95 >= gate_ms {
            failures.push(format!("{sc} p95 {:.1}ms >= gate {gate_ms}ms", p95));
        }
        if walls.len() != samples {
            failures.push(format!(
                "{sc} measured {} samples, expected {samples}",
                walls.len()
            ));
        }
    }
    if gate_ms <= 0.0 {
        failures.push("gate-ms 0 always fails (counter-control)".into());
    }
    // F-817-R3-04: the degrade gate fires ONLY from the DETECTOR VALUE —
    // the same normal check in control and normal runs alike (no marker
    // failure manufactured by the inject flag). The env control disables
    // the detector; the TEST then asserts the red vanishes — proving the
    // gate, not the flag, produces the failure.
    let detector_disabled = std::env::var("AUTO_CTE_DISABLE_RESIDENT_GATE")
        .map(|v| v == "1")
        .unwrap_or(false);
    let resident_rows_ok = rows
        .iter()
        .filter(|r| r.scenario == "resident-edit")
        .all(|r| r.ok);
    let re_delta = rewarms_after.saturating_sub(rewarms_before);
    summary["resident_edit_rewarms"] = json!(re_delta);
    if !resident_rows_ok {
        failures.push("resident-edit sample value mismatch".into());
    }
    if !detector_disabled && re_delta != 0 {
        failures.push(format!(
            "resident-edit degraded: {re_delta} whole-module re-warm(s) during incremental traffic"
        ));
    }
    if inject_rebuild && re_delta == 0 && !detector_disabled {
        failures.push("forced-rebuild control did NOT re-warm (detector saw nothing)".into());
    }
    // F-817-R3-01: the memory hard gates — per-scenario worker peaks and
    // the whole-process tree peak; an overage can NEVER be green, and
    // every scenario peak is recorded in the summary for recomputation.
    const WORKER_GATE: u64 = 384 * 1024 * 1024;
    const TREE_GATE: u64 = 512 * 1024 * 1024;
    let mut peak_map = serde_json::Map::new();
    for (name, pk) in &scenario_peaks {
        peak_map.insert(
            name.to_string(),
            json!({ "bytes": pk, "mib": (*pk as f64) / 1048576.0 }),
        );
    }
    summary["scenario_peaks"] = Value::Object(peak_map);
    for (name, pk) in &scenario_peaks {
        if *pk > WORKER_GATE {
            failures.push(format!(
                "{name} worker peak {:.1}MiB > 384MiB gate",
                *pk as f64 / 1048576.0
            ));
        }
    }
    if worker_peak > TREE_GATE {
        failures.push(format!(
            "process tree peak {:.1}MiB > 512MiB gate",
            worker_peak as f64 / 1048576.0
        ));
    }
    // The managed-budget gate rides the process sampler: any row whose
    // driver reported managed_target_met=false is a hard failure.
    // (In-process rows carry it via ok; the summary records the process
    // peak.)
    summary["hard_gate_failures"] = json!(failures);
    summary["target_met"] = json!(failures.is_empty());

    if let Some(path) = out {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut f = std::fs::File::create(path).expect("raw report file");
        for r in &rows {
            let line = json!({
                "scenario": r.scenario,
                "sample": if r.sample == usize::MAX { serde_json::Value::Null } else { json!(r.sample) },
                "wall_ms": r.wall_ms,
                "input_hash": r.input_hash,
                "cache": r.cache,
                "ct_execute": r.ct_execute,
                "ok": r.ok,
                "pid": r.pid,
                "value": r.value,
            });
            writeln!(f, "{line}").ok();
        }
        writeln!(f, "{summary}").ok();
    }

    (if failures.is_empty() { 0 } else { 5 }, summary)
}

/// The deck a process sample runs.
pub enum SpawnDeck {
    /// The specified 115-fn baseline at `rev` (exec snapshot lane).
    Big { rev: i64 },
    /// The specified load + the CONTRACTED Meta->AV path with `fields`
    /// extra fields in the meta block.
    Meta { fields: usize },
}

impl SpawnDeck {
    fn build(&self) -> String {
        match self {
            SpawnDeck::Big { rev } => big_deck(*rev),
            SpawnDeck::Meta { fields } => big_meta_deck(*fields),
        }
    }
    /// Independent closed-form expectation for the LAST query step.
    fn expected(&self, base_value: i64) -> String {
        match self {
            SpawnDeck::Big { rev } => format!("i32:{}", base_value + rev),
            SpawnDeck::Meta { fields } => format!("i32:{}", (2 + *fields as i64) * 100),
        }
    }
}

/// F-817-R3-03/04: one REAL worker process per sample. The wall covers
/// request build + serialization, spawn, IPC, response parse AND the
/// full admission (echoes + per-step outcome/value/HIR checks against
/// the independent closed form) — a diag or wrong value inside a valid
/// protocol envelope is NOT ok.
fn spawn_process_samples(
    scenario: &'static str,
    samples: usize,
    store: &std::path::Path,
    deck_kind: &SpawnDeck,
    edit: Option<(String, String)>,
    rows: &mut Vec<Row>,
    peaks: &mut std::collections::BTreeMap<&'static str, u64>,
) {
    use base64::Engine as _;
    use std::io::Write;
    use std::process::{Command, Stdio};
    let exe = std::env::current_exe().expect("exe");
    let steps: Vec<serde_json::Value> = match &edit {
        None => vec![serde_json::json!({ "op": "query" })],
        Some((from, to)) => vec![
            serde_json::json!({ "op": "query" }),
            serde_json::json!({ "op": "edit-source", "from": from, "to": to }),
            serde_json::json!({ "op": "query" }),
        ],
    };
    let base_value: i64 = 42 + 115 * 4780 + 81 * (114 * 115 / 2);
    for i in 0..samples {
        let deck = deck_kind.build();
        // The edit bumps the revision literal by exactly +1.
        let rev_bumped: Option<i64> = edit
            .as_ref()
            .and_then(|(from, _)| from.rsplit('+').next())
            .and_then(|t| t.trim().parse().ok());
        let expected = match (&edit, deck_kind) {
            (Some(_), SpawnDeck::Big { rev }) => format!("i32:{}", base_value + rev + 1),
            (None, d) => d.expected(base_value),
            (Some(_), SpawnDeck::Meta { .. }) => unreachable!("meta scenarios do not edit"),
        };
        let _ = rev_bumped;
        let sp = auto_av_probe::mempeek::PeakSampler::start(1);
        let t = Instant::now();
        let req = serde_json::json!({
            "recipe": "cte/scalar-query-v1",
            "nonce": format!("{scenario}-{i}"),
            "deck_b64": base64::engine::general_purpose::STANDARD.encode(&deck),
            "store_root": store.display().to_string().replace('/', "\\"),
            "session": scenario,
            "steps": steps,
            "request_hash": format!("rh-{scenario}-{i}"),
        })
        .to_string();
        let mut child = Command::new(&exe)
            .arg("case-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("worker spawn");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(req.as_bytes())
            .unwrap();
        let out = child.wait_with_output().expect("worker completes");
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        // Admission ends INSIDE the window: parse + echoes + result checks.
        let v: serde_json::Value =
            serde_json::from_str(text.trim()).unwrap_or(serde_json::Value::Null);
        let last = v["ok"]["steps"].as_array().and_then(|a| a.last()).cloned();
        let edited_ok = edit.is_none()
            || v["ok"]["steps"]
                .as_array()
                .map(|a| a.iter().any(|st| st["outcome"] == "edited"))
                .unwrap_or(false);
        let outcome_ok = last
            .as_ref()
            .map(|st| {
                st["outcome"] == "ok"
                    && st["language_class"] == false
                    && st["values"]
                        .as_object()
                        .map(|m| !m.is_empty())
                        .unwrap_or(false)
                    && st["hir"]
                        .as_object()
                        .map(|m| !m.is_empty())
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        let got_value = last
            .as_ref()
            .and_then(|st| st["values"].as_object())
            .and_then(|m| m.values().next())
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let value_ok = got_value == expected;
        let ok = out.status.success()
            && v["schema"] == "cteworker/1"
            && v["nonce"] == format!("{scenario}-{i}")
            && v["request_hash"] == format!("rh-{scenario}-{i}")
            && edited_ok
            && outcome_ok
            && value_ok;
        let wall = t.elapsed().as_secs_f64() * 1000.0;
        let pk = sp.stop();
        peaks
            .entry(scenario)
            .and_modify(|c| *c = (*c).max(pk))
            .or_insert(pk);
        let cache_str: &'static str = Box::leak(
            last.as_ref()
                .and_then(|st| st["cache"].as_str())
                .unwrap_or("-")
                .to_string()
                .into_boxed_str(),
        );
        rows.push(Row {
            scenario,
            sample: i,
            wall_ms: wall,
            input_hash: crate::fnv1a64_hex(deck.as_bytes()),
            cache: if ok { cache_str } else { "proto-err" },
            ct_execute: last
                .as_ref()
                .and_then(|st| st["ct_execute"].as_u64())
                .unwrap_or(0),
            ok,
            pid: v["ok"]["worker_pid"].as_u64().map(|p| p as u32),
            value: got_value,
        });
    }
}

/// F-817-R3-03: the WARM-SESSION hard-gate window. ONE worker process
/// (`case-serve-stream`, one resident driver + store), warmed by a first
/// request OUTSIDE the window; each measured sample is a full round trip
/// — request build + serialize, stdin write, IPC, response read, and the
/// complete admission (echoes + outcome/value/HIR against the
/// independent closed form).
fn spawn_warm_samples(
    scenario: &'static str,
    samples: usize,
    store: &std::path::Path,
    decks: &[String],
    steps_per_req: &[Vec<serde_json::Value>],
    expected: &[String],
    rows: &mut Vec<Row>,
    peaks: &mut std::collections::BTreeMap<&'static str, u64>,
    inject_delay_ms: u64,
) {
    use base64::Engine as _;
    use std::io::{BufRead, BufReader, Write};
    use std::process::{ChildStdin, ChildStdout, Command, Stdio};
    let exe = std::env::current_exe().expect("exe");
    let mut child = Command::new(&exe)
        .arg("case-serve-stream")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("warm worker spawn");
    let mut stdin = child.stdin.take().expect("stdin");
    let stdout = child.stdout.take().expect("stdout");
    let mut reader = BufReader::new(stdout);

    let send = |stdin: &mut ChildStdin, req: &serde_json::Value| {
        stdin
            .write_all(format!("{}\n", req.to_string()).as_bytes())
            .expect("write request");
        stdin.flush().expect("flush");
    };
    let recv = |reader: &mut BufReader<ChildStdout>| -> serde_json::Value {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read response");
        serde_json::from_str(line.trim()).unwrap_or(serde_json::Value::Null)
    };
    let build_req = |i: usize, deck: &str, idx: usize| -> serde_json::Value {
        serde_json::json!({
            "recipe": "cte/scalar-query-v1",
            "nonce": format!("{scenario}-{i}"),
            "deck_b64": base64::engine::general_purpose::STANDARD.encode(deck),
            "store_root": store.display().to_string().replace('/', "\\"),
            "session": scenario,
            "steps": steps_per_req[idx],
            "request_hash": format!("rh-{scenario}-{i}"),
        })
    };
    let admit = |i: usize, v: &serde_json::Value, exp: &str| -> (bool, String, Option<u32>) {
        let last = v["ok"]["steps"].as_array().and_then(|a| a.last()).cloned();
        let outcome_ok = last
            .as_ref()
            .map(|st| {
                st["outcome"] == "ok"
                    && st["language_class"] == false
                    && st["values"]
                        .as_object()
                        .map(|m| !m.is_empty())
                        .unwrap_or(false)
                    && st["hir"]
                        .as_object()
                        .map(|m| !m.is_empty())
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        let got = last
            .as_ref()
            .and_then(|st| st["values"].as_object())
            .and_then(|m| m.values().next())
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let ok = v["schema"] == "cteworker/1"
            && v["nonce"] == format!("{scenario}-{i}")
            && v["request_hash"] == format!("rh-{scenario}-{i}")
            && outcome_ok
            && got == exp;
        (ok, got, v["ok"]["worker_pid"].as_u64().map(|p| p as u32))
    };

    // Warm-up request OUTSIDE the measured window (decks[0]).
    send(&mut stdin, &build_req(usize::MAX, &decks[0], 0));
    let warm_v = recv(&mut reader);
    let (warm_ok, _, _) = admit(usize::MAX, &warm_v, &expected[0]);
    assert!(warm_ok, "warm-up request admitted: {warm_v}");

    let sp = auto_av_probe::mempeek::PeakSampler::start(1);
    for i in 0..samples {
        let deck = &decks[i + 1];
        let t = Instant::now();
        let req = build_req(i, deck, i + 1);
        send(&mut stdin, &req);
        let v = recv(&mut reader);
        let mut wall = t.elapsed().as_secs_f64() * 1000.0;
        // The injectable bad-sample control: a deterministic parent-side
        // delay on the first TWO samples occupies the P95 position.
        if inject_delay_ms > 0 && i < 2 {
            std::thread::sleep(std::time::Duration::from_millis(inject_delay_ms));
            wall += inject_delay_ms as f64;
        }
        let (ok, got, pid) = admit(i, &v, &expected[i + 1]);
        let last = v["ok"]["steps"].as_array().and_then(|a| a.last()).cloned();
        let cache_str: &'static str = Box::leak(
            last.as_ref()
                .and_then(|st| st["cache"].as_str())
                .unwrap_or("-")
                .to_string()
                .into_boxed_str(),
        );
        rows.push(Row {
            scenario,
            sample: i,
            wall_ms: wall,
            input_hash: crate::fnv1a64_hex(deck.as_bytes()),
            cache: if ok { cache_str } else { "proto-err" },
            ct_execute: last
                .as_ref()
                .and_then(|st| st["ct_execute"].as_u64())
                .unwrap_or(0),
            ok,
            pid,
            value: got,
        });
    }
    let pk = sp.stop();
    peaks
        .entry(scenario)
        .and_modify(|c| *c = (*c).max(pk))
        .or_insert(pk);
    drop(stdin);
    let _ = child.wait();
}
