//! The turnaround bench (PLAN-817 T-07, V-09): six scenarios x N raw
//! samples through the REAL driver path, with the two EDIT hard gates
//! (helper-body edit and Meta-set edit, warm store, P95 < gate) and a
//! JSONL raw report whose rows bind each sample to the hash of the
//! exact deck text it executed. Native observables are OFF here — the
//! native reference is reported separately (contract-decision §7).

use crate::driver::{CteDriver, StepStatus};
use serde_json::json;
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

fn big_deck(rev: i64) -> String {
    // exec snapshot: the incremental lane is the production path for
    // edit-heavy sessions (T-11/SD-06) — the warm resident session makes
    // local helper-body edits cheap on the specified load.
    let src = baseline_source_bench(115, rev);
    format!(
        "deck cte817/1\nmodule main\nexec snapshot\nsource <<\n{src}\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\nnative off\n"
    )
}

const BASE_SRC: &str = "fn cte_sum(a int, b int) int {\n    return a + b + PAD\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n";
const BASE_DECK: &str = "deck cte817/1\nmodule main\nsource <<\nSRC\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\nnative off\n";
fn sum_deck(pad: i64) -> String {
    BASE_DECK.replace("SRC", &BASE_SRC.replace("PAD", &format!("{pad}")))
}

fn meta_deck(extra_fields: usize) -> String {
    // The CONTRACTED path (F-817-R1-05): field_count feeds a REAL AV
    // call as a live argument; each revision adds a field.
    let mut d = String::new();
    d.push_str("deck cte817/1\nmodule main\ncaps cte.meta.fields/v1\nmeta <<\ntype Pair {\n    left int\n    right bool\n");
    for i in 0..extra_fields {
        d.push_str(&format!("    extra{i} int\n"));
    }
    d.push_str("}\n>>\nsource <<\nfn scale(n int) int {\n    return n * 100\n}\n\nfn main() int {\n    let v = 0\n    return v\n}\n>>\nquery n = field_count Pair\nquery q = call scale(n)\nbind main.v = q expect i32 placeholder 0\nentry main\nnative off\n");
    d
}

struct Row {
    scenario: &'static str,
    sample: usize,
    wall_ms: f64,
    input_hash: String,
    cache: &'static str,
    ct_execute: u64,
    ok: bool,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn measure(
    scenario: &'static str,
    sample: usize,
    deck: &str,
    drv: &mut CteDriver,
    rows: &mut Vec<Row>,
    want: Option<i64>,
    inject_delay_ms: u64,
) {
    let t = Instant::now();
    let o = drv.query_step(deck, None);
    let mut wall = t.elapsed().as_secs_f64() * 1000.0;
    // The injectable bad-sample control (T-07): a deterministic spike
    // that must turn the gate red.
    if scenario == "body-edit" && sample < 2 && inject_delay_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(inject_delay_ms));
        wall += inject_delay_ms as f64;
    }
    let ok = o.status == StepStatus::Ok
        && want
            .map(|w| {
                let want = format!("i32:{w}");
                o.values.get("main.ct").map(|v| v.render()) == Some(want.clone())
                    || o.values.get("main.n").map(|v| v.render()) == Some(want.clone())
                    || o.values.get("main.v").map(|v| v.render()) == Some(want)
            })
            .unwrap_or(true);
    rows.push(Row {
        scenario,
        sample,
        wall_ms: wall,
        input_hash: crate::fnv1a64_hex(deck.as_bytes()),
        cache: o.cache.unwrap_or("-"),
        ct_execute: o.ct_execute,
        ok,
    });
}

/// Run the bench; returns (exit_code, summary). Raw rows (JSONL, one
/// per sample) plus the summary line go to `out` when given.
pub fn run_bench(
    samples: usize,
    gate_ms: f64,
    inject_delay_ms: u64,
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

    // 1) fresh: a brand-new store per sample (cold everything).
    for i in 0..samples {
        let dir = root.join(format!("fresh-{i}"));
        let _ = std::fs::remove_dir_all(&dir);
        let mut d = CteDriver::with_store(&dir, "fresh").unwrap();
        measure("fresh", i, &sum_deck(0), &mut d, &mut rows, Some(42), 0);
    }

    // 2) hot: one warm driver, repeated identical requests (hit path).
    let hot_dir = root.join("hot");
    let _ = std::fs::remove_dir_all(&hot_dir);
    let mut hot = CteDriver::with_store(&hot_dir, "hot").unwrap();
    measure(
        "hot",
        usize::MAX,
        &sum_deck(0),
        &mut hot,
        &mut rows,
        Some(42),
        0,
    );
    for i in 0..samples {
        measure("hot", i, &sum_deck(0), &mut hot, &mut rows, Some(42), 0);
    }

    // 3) restart: a REAL worker process per sample (F-817-R1-03) — the
    //    measured window covers request build -> serialize -> spawn ->
    //    IPC -> response parse + admission echoes, against a shared
    //    store the prior process published into.
    let rs_dir = root.join("restart");
    let _ = std::fs::remove_dir_all(&rs_dir);
    {
        let mut warm = CteDriver::with_store(&rs_dir, "rs").unwrap();
        measure(
            "restart",
            usize::MAX,
            &sum_deck(0),
            &mut warm,
            &mut rows,
            Some(42),
            0,
        );
    }
    spawn_process_samples(
        "restart",
        samples,
        &rs_dir,
        &sum_deck(0),
        None,
        std::env::current_exe().ok().as_deref(),
        &mut rows,
    );

    // 4) body-edit (HARD GATE) on the SPECIFIED LOAD: the 115-function
    //    >=256KiB baseline, warm store; every sample is a real LOCAL
    //    helper-body revision — fresh manifest, real re-execution.
    let be_dir = root.join("body-edit");
    let _ = std::fs::remove_dir_all(&be_dir);
    let mut be = CteDriver::with_store(&be_dir, "be").unwrap();
    let be_want = {
        let probe_out = be.query_step(&big_deck(0), None);
        assert_eq!(probe_out.status, crate::driver::StepStatus::Ok);
        probe_out
            .values
            .get("main.ct")
            .and_then(|v| {
                v.render()
                    .strip_prefix("i32:")
                    .and_then(|x| x.parse::<i64>().ok())
            })
            .expect("closed-form probe")
    };
    for i in 0..samples {
        let deck = big_deck(i as i64);
        let want = be_want + i as i64;
        measure(
            "body-edit",
            i,
            &deck,
            &mut be,
            &mut rows,
            Some(want),
            inject_delay_ms,
        );
    }

    // 5) meta-edit (HARD GATE) on the CONTRACTED path: every sample
    //    adds a field; field_count flips the Meta fact, the frozen value
    //    invalidates and the AV call re-executes with the new argument.
    let me_dir = root.join("meta-edit");
    let _ = std::fs::remove_dir_all(&me_dir);
    let mut me = CteDriver::with_store(&me_dir, "me").unwrap();
    for i in 0..samples {
        let deck = meta_deck(i);
        let want = (2 + i as i64) * 100;
        measure("meta-edit", i, &deck, &mut me, &mut rows, Some(want), 0);
    }

    // 6) gc-concurrent: queries under a live GC loop on the same store
    //    (its OWN store handle — 783 liveness is OS-truth cross-handle).
    let gc_dir = root.join("gc");
    let _ = std::fs::remove_dir_all(&gc_dir);
    let mut gd = CteDriver::with_store(&gc_dir, "gc").unwrap();
    for i in 0..3 {
        measure(
            "gc",
            10_000 + i,
            &sum_deck(i as i64),
            &mut gd,
            &mut rows,
            Some(42 + i as i64),
            0,
        );
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
    for i in 0..samples {
        let deck = sum_deck((i % 3) as i64);
        let want = 42 + (i % 3) as i64;
        measure("gc", i, &deck, &mut gd, &mut rows, Some(want), 0);
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let _ = handle.join();

    // 7) worker-edit: the FULL official window through a REAL worker
    //    process per sample (F-817-R1-03): query -> edit -> query, the
    //    discriminating-edit round trip over IPC.
    let we_dir = root.join("worker-edit");
    let _ = std::fs::remove_dir_all(&we_dir);
    for i in 0..samples {
        spawn_process_samples(
            "worker-edit",
            1,
            &we_dir,
            &sum_deck(0),
            Some(("return a + b + 0", "return a + b + 1")),
            std::env::current_exe().ok().as_deref(),
            &mut rows,
        );
        if let Some(r) = rows.last_mut() {
            r.sample = i;
        }
    }

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
    ];
    let mut summary = serde_json::json!({
        "samples_per_scenario": samples,
        "gate_ms": gate_ms,
        "bench_wall_ms": bench_wall_ms,
        "worker_private_bytes_peak": worker_peak,
    });
    let mut failures: Vec<String> = Vec::new();
    let mut all_ok = true;
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
        all_ok &= ok;
        summary[sc] = json!({
            "p95_ms": p95,
            "max_ms": max,
            "samples": walls.len(),
            "all_values_ok": ok,
        });
        // Strict: P95 must be strictly BELOW the gate (equality fails).
        if (sc == "body-edit" || sc == "meta-edit") && p95 >= gate_ms {
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
    if !all_ok {
        failures.push("a sample value mismatch".into());
    }
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
            });
            writeln!(f, "{line}").ok();
        }
        writeln!(f, "{summary}").ok();
    }

    (if failures.is_empty() { 0 } else { 5 }, summary)
}

/// F-817-R1-03: one REAL worker process per sample. The wall covers
/// request build + serialization, spawn, IPC, response parse and the
/// admission echoes (nonce/request_hash) — the official runner's window.
#[allow(clippy::too_many_arguments)]
fn spawn_process_samples(
    scenario: &'static str,
    samples: usize,
    store: &std::path::Path,
    deck: &str,
    edit: Option<(&str, &str)>,
    exe: Option<&std::path::Path>,
    rows: &mut Vec<Row>,
) {
    use base64::Engine as _;
    use std::io::Write;
    use std::process::{Command, Stdio};
    let exe = exe
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_exe().expect("exe"));
    let steps: Vec<serde_json::Value> = match edit {
        None => vec![serde_json::json!({ "op": "query" })],
        Some((from, to)) => vec![
            serde_json::json!({ "op": "query" }),
            serde_json::json!({ "op": "edit-source", "from": from, "to": to }),
            serde_json::json!({ "op": "query" }),
        ],
    };
    for i in 0..samples {
        let t = Instant::now();
        let req = serde_json::json!({
            "recipe": "cte/scalar-query-v1",
            "nonce": format!("{scenario}-{i}"),
            "deck_b64": base64::engine::general_purpose::STANDARD.encode(deck),
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
        let wall = t.elapsed().as_secs_f64() * 1000.0;
        let text = String::from_utf8_lossy(&out.stdout);
        let v: serde_json::Value =
            serde_json::from_str(text.trim()).unwrap_or(serde_json::Value::Null);
        let ok = out.status.success()
            && v["schema"] == "cteworker/1"
            && v["nonce"] == format!("{scenario}-{i}")
            && v["request_hash"] == format!("rh-{scenario}-{i}")
            && v["ok"]["steps"]
                .as_array()
                .map(|a| a.len() == steps.len())
                .unwrap_or(false);
        rows.push(Row {
            scenario,
            sample: i,
            wall_ms: wall,
            input_hash: crate::fnv1a64_hex(deck.as_bytes()),
            cache: if ok { "-" } else { "proto-err" },
            ct_execute: 0,
            ok,
        });
    }
}
