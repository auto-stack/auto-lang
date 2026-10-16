//! PLAN-788 T-08 turnaround acceptance (V-12, §5.5): six scenarios × 30
//! samples over the REAL worker binary through production-shaped IPC
//! (single-shot spawn for fresh/restart/cross-project/body-edit; one
//! session for the same-process hot scenario). Every sample's functional
//! oracle (exit 3 / exit 4) is re-verified — a fast-but-wrong sample is a
//! failure, never data.
//!
//! HARD GATES (contract-decision §8): hot and body-edit P95 < 1000ms.
//! Raw samples are written to target/turnaround-samples.json for the
//! report (kept, never filtered).

#![cfg(windows)]

use base64::Engine;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const SAMPLES: usize = 30;
const P95_GATE_MS: u128 = 1000;

fn b64(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

fn p04(bound: u32) -> String {
    format!(
        "fn main() int {{ var i = 0; while i < {} {{ i = i + 1 }}; return i }}",
        bound
    )
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("ac788-turn-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

struct Session {
    child: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    req_no: u32,
}

impl Session {
    fn spawn(store: Option<&str>, cache: &str, project: &str) -> (Session, std::path::PathBuf) {
        let _ = (store, cache, project);
        let exe = env!("CARGO_BIN_EXE_ac-case-worker");
        let work = scratch(&format!("w{}", std::process::id()));
        let mut child = Command::new(exe)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let reader = BufReader::new(child.stdout.take().unwrap());
        let mut stdin = child.stdin.take().unwrap();
        writeln!(stdin, r#"{{"op":"begin","schema":"acworker/1"}}"#).unwrap();
        stdin.flush().unwrap();
        let s = Session {
            child,
            stdin,
            reader,
            req_no: 0,
        };
        (s, work)
    }

    /// One request over the open session; wall = write→response-read.
    fn request(
        &mut self,
        work: &std::path::Path,
        store: Option<&str>,
        cache: &str,
        project: &str,
        source: &str,
    ) -> (Duration, serde_json::Value) {
        self.req_no += 1;
        let n = self.req_no;
        let req = serde_json::json!({
            "schema": "acworker/1",
            "kind": "run",
            "nonce": format!("n-{n}"),
            "request_hash": format!("h-{n}"),
            "input": { "kind": "source", "source_b64": b64(source) },
            "profile": "core",
            "entry": "main",
            "budget": { "build_ms": 60000, "run_ms": 5000, "total_ms": 70000 },
            "scratch": work.display().to_string(),
            "store_root": store,
            "cache": cache,
            "project": project,
            "include_hir": false,
        });
        let line = serde_json::to_string(&serde_json::json!({
            "op": "request", "request": req
        }))
        .unwrap();
        let t = Instant::now();
        writeln!(self.stdin, "{}", line).unwrap();
        self.stdin.flush().unwrap();
        let mut resp = String::new();
        self.reader.read_line(&mut resp).unwrap();
        let wall = t.elapsed();
        let v: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
        assert!(v["error"].is_null(), "sample failed: {}", resp);
        (wall, v)
    }

    fn close(mut self) {
        let _ = writeln!(self.stdin, r#"{{"op":"end"}}"#);
        let _ = self.stdin.flush();
        let _ = self.child.wait();
    }
}

fn verify_exit(v: &serde_json::Value, want: i64) {
    assert_eq!(
        v["ok"]["run"]["exit_code"], want,
        "functional oracle re-verified per sample"
    );
}

fn p95(samples: &[u128]) -> u128 {
    let mut s = samples.to_vec();
    s.sort_unstable();
    s[(s.len() * 95 + 99) / 100 - 1]
}

fn record(name: &str, samples: &[u128], all: &mut Vec<serde_json::Value>) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let p50 = sorted[sorted.len() / 2];
    let p95v = p95(samples);
    let p99 = sorted[sorted.len() - 1];
    println!(
        "{name}: n={} p50={}ms p95={}ms p99={}ms",
        samples.len(),
        p50,
        p95v,
        p99
    );
    all.push(serde_json::json!({
        "scenario": name,
        "n": samples.len(),
        "p50_ms": p50,
        "p95_ms": p95v,
        "p99_ms": p99,
        "samples_ms": samples,
    }));
}

/// Single-shot production shape: spawn → begin/request/end → exit.
fn one_shot(store: Option<&str>, cache: &str, project: &str, source: &str, want: i64) -> Duration {
    let (mut s, work) = Session::spawn(store, cache, project);
    let (wall, v) = s.request(&work, store, cache, project, source);
    verify_exit(&v, want);
    s.close();
    wall
}

#[test]
fn turnaround_six_scenarios_p95_gates() {
    let root = scratch("stores");
    let mut all: Vec<serde_json::Value> = Vec::new();

    // 1) fresh: no cache reads (new store root per sample), full pipeline.
    let mut fresh = Vec::new();
    for i in 0..SAMPLES {
        let store = root.join(format!("fresh-{i}"));
        std::fs::create_dir_all(&store).unwrap();
        fresh.push(
            one_shot(
                Some(store.to_str().unwrap()),
                "fresh",
                "fresh-proj",
                &p04(3),
                3,
            )
            .as_millis(),
        );
    }
    record("fresh", &fresh, &mut all);

    // 2) same-process hot: ONE worker session, 30 sequential requests,
    // one store — per-request wall (session setup excluded, first request
    // warms the store).
    let hot_store = root.join("hot");
    std::fs::create_dir_all(&hot_store).unwrap();
    {
        let (mut s, work) = Session::spawn(Some(hot_store.to_str().unwrap()), "reuse", "hot");
        let mut samples = Vec::new();
        for _ in 0..SAMPLES {
            let (wall, v) = s.request(
                &work,
                Some(hot_store.to_str().unwrap()),
                "reuse",
                "hot",
                &p04(3),
            );
            verify_exit(&v, 3);
            samples.push(wall.as_millis());
        }
        s.close();
        record("hot", &samples, &mut all);
        assert!(
            p95(&samples) < P95_GATE_MS,
            "HARD GATE: hot P95 {}ms >= {}ms",
            p95(&samples),
            P95_GATE_MS
        );
    }

    // 3) restart disk-hot: a NEW worker process per sample, SAME store —
    // object hit from disk, link+run real.
    let restart_store = root.join("restart");
    std::fs::create_dir_all(&restart_store).unwrap();
    {
        // Warm once.
        one_shot(
            Some(restart_store.to_str().unwrap()),
            "reuse",
            "restart",
            &p04(3),
            3,
        );
        let mut samples = Vec::new();
        for _ in 0..SAMPLES {
            samples.push(
                one_shot(
                    Some(restart_store.to_str().unwrap()),
                    "reuse",
                    "restart",
                    &p04(3),
                    3,
                )
                .as_millis(),
            );
        }
        record("restart-disk-hot", &samples, &mut all);
    }

    // 4) cross-project same content: alternating project namespaces over
    // one store still hit (content-keyed identity, project only roots).
    let xstore = root.join("cross");
    std::fs::create_dir_all(&xstore).unwrap();
    {
        let mut samples = Vec::new();
        for i in 0..SAMPLES {
            let project = if i % 2 == 0 { "proj-a" } else { "proj-b" };
            samples.push(
                one_shot(Some(xstore.to_str().unwrap()), "reuse", project, &p04(3), 3).as_millis(),
            );
        }
        record("cross-project", &samples, &mut all);
    }

    // 5) local body edit: bound alternates 3↔4 — every request changes the
    // snapshot (full recompute) and the NEW oracle (exit 3 / exit 4) is
    // what actually runs.
    let edit_store = root.join("edit");
    std::fs::create_dir_all(&edit_store).unwrap();
    {
        let mut samples = Vec::new();
        for i in 0..SAMPLES {
            let bound = if i % 2 == 0 { 3 } else { 4 };
            samples.push(
                one_shot(
                    Some(edit_store.to_str().unwrap()),
                    "reuse",
                    "edit",
                    &p04(bound),
                    bound as i64,
                )
                .as_millis(),
            );
        }
        record("body-edit", &samples, &mut all);
        assert!(
            p95(&samples) < P95_GATE_MS,
            "HARD GATE: body-edit P95 {}ms >= {}ms",
            p95(&samples),
            P95_GATE_MS
        );
    }

    // 6) GC concurrent: requests flow over one store while the probe GC
    // sweeps periodically (live roots keep the shared chunks — requests
    // must stay functionally correct; samples are the request walls).
    let gc_store = root.join("gc");
    std::fs::create_dir_all(&gc_store).unwrap();
    {
        use ac_artifact_probe::store::ManagedStore;
        let (mut s, work) = Session::spawn(Some(gc_store.to_str().unwrap()), "reuse", "gc");
        let mut samples = Vec::new();
        for i in 0..SAMPLES {
            if i % 10 == 5 {
                let store = ManagedStore::open(&gc_store).unwrap();
                let _ = ac_artifact_probe::store::gc::run_gc(
                    &store,
                    ac_artifact_probe::store::gc::gc_config_default(),
                );
            }
            let (wall, v) = s.request(
                &work,
                Some(gc_store.to_str().unwrap()),
                "reuse",
                "gc",
                &p04(3),
            );
            verify_exit(&v, 3);
            samples.push(wall.as_millis());
        }
        s.close();
        record("gc-concurrent", &samples, &mut all);
    }

    // Raw samples are preserved in full — no filtering, timeouts included.
    let out = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("turnaround-samples.json");
    std::fs::write(&out, serde_json::to_string_pretty(&all).unwrap()).unwrap();
    println!("raw samples: {}", out.display());
}
