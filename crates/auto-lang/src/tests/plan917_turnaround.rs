//! PLAN-917 V-06: emit turnaround probe for the R07 daily axis — four states
//! (fresh / hit / restart / body-edit), 30 samples each, P90 ≤ 2000 ms per
//! state (plan §5.3 gate). The legacy producer is stateless, so fresh and hit
//! are expected to be statistically identical; the probe records the honest
//! numbers rather than inventing a cache. Explicit manual entry (ignored),
//! mirroring the a2r_compile_smoke precedent:
//!   cargo test -p auto-lang --lib plan917_emit_turnaround_v06 -- --ignored

use std::path::PathBuf;
use std::time::Instant;

const SAMPLES: usize = 30;
const P90_BUDGET_MS: u128 = 2000;
const EMIT_STACK_BYTES: usize = 128 * 1024 * 1024;

fn r07_source() -> (PathBuf, String) {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = d.join("test/cases/ace-m1/src");
    let src = std::fs::read_to_string(dir.join("R07.at")).unwrap();
    (dir, src)
}

/// One production-entry emission on a dedicated big-stack thread (mirrors the
/// a2r adapter). Returns wall millis.
fn emit_once(dir: &PathBuf, src: &str) -> u128 {
    let dir = dir.clone();
    let src = src.to_string();
    let start = Instant::now();
    let handle = std::thread::Builder::new()
        .stack_size(EMIT_STACK_BYTES)
        .spawn(move || {
            let mut sink =
                crate::trans::rust::transpile_rust_with_source_dir(&dir, "R07", &src).unwrap();
            sink.done().unwrap().clone()
        })
        .unwrap();
    let bytes = handle.join().unwrap();
    let _ = bytes;
    start.elapsed().as_millis()
}

fn p90(samples: &mut Vec<u128>) -> u128 {
    samples.sort_unstable();
    let idx = ((samples.len() as f64) * 0.9).ceil() as usize - 1;
    samples[idx]
}

#[test]
#[ignore = "Plan 917 V-06 emit turnaround probe; explicit entry, 4 states x 30 samples"]
fn plan917_emit_turnaround_v06() {
    // Child mode: one emit, print millis, exit (the "restart" state samples a
    // cold process per measurement).
    if std::env::var("PLAN917_EMIT_ONCE").is_ok() {
        let (dir, src) = r07_source();
        println!("{}", emit_once(&dir, &src));
        return;
    }

    let (dir, src) = r07_source();
    let mut report: Vec<(&str, u128, u128)> = Vec::new();

    // fresh: re-read the source per sample (cold read + emit).
    let mut fresh = Vec::new();
    for _ in 0..SAMPLES {
        let (_, s) = r07_source();
        fresh.push(emit_once(&dir, &s));
    }
    report.push(("fresh", p90(&mut fresh), *fresh.iter().max().unwrap()));

    // hit: same-process re-emit of the in-memory source (warm path).
    let mut hit = Vec::new();
    for _ in 0..SAMPLES {
        hit.push(emit_once(&dir, &src));
    }
    report.push(("hit", p90(&mut hit), *hit.iter().max().unwrap()));

    // body-edit: a semantic no-op comment change per sample, then emit.
    let mut body_edit = Vec::new();
    for i in 0..SAMPLES {
        let edited = format!("{}\n// body-edit sample {}\n", src, i);
        body_edit.push(emit_once(&dir, &edited));
    }
    report.push((
        "body-edit",
        p90(&mut body_edit),
        *body_edit.iter().max().unwrap(),
    ));

    // restart: a cold child process per sample (emit inside the child).
    let exe = std::env::current_exe().unwrap();
    let mut restart = Vec::new();
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let out = std::process::Command::new(&exe)
            .arg("plan917_emit_turnaround_v06")
            .arg("--exact")
            .arg("--nocapture")
            .env("PLAN917_EMIT_ONCE", "1")
            .output()
            .unwrap();
        assert!(out.status.success(), "restart child failed: {out:?}");
        restart.push(start.elapsed().as_millis());
    }
    report.push(("restart", p90(&mut restart), *restart.iter().max().unwrap()));

    for (state, p90_ms, max_ms) in &report {
        println!("plan917 V-06 {state}: P90={p90_ms}ms max={max_ms}ms (n={SAMPLES})");
        assert!(
            *p90_ms <= P90_BUDGET_MS,
            "{state} P90 {p90_ms}ms exceeds the {P90_BUDGET_MS}ms budget"
        );
    }
}
