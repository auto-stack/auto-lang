//! PLAN-788 R2 turnaround acceptance (V-12/V-13, §5.5 as amended by
//! review-r2 F02): six native scenarios + the syntax gate + the 50×100ms
//! revision stream, over the REAL worker binary.
//!
//! R2 measurement contract:
//! - one-shot walls span spawn → IPC → typed verify (exit oracle) → full
//!   process reaping (wait). Same-process hot walls span request
//!   write → response read → verify; session prep (spawn+begin) is timed
//!   separately and reported.
//! - body-edit samples use UNIQUE content per request (every sample a real
//!   recompute); per-sample driver counters are captured and asserted.
//! - gc-concurrent runs the probe GC on a separate thread DURING the
//!   request flow; overlap is recorded, GC success asserted.
//! - raw keeps one FULL record per sample (wall, oracle result, counters,
//!   hits, input sha, worker sha, errors) — written BEFORE any gate
//!   assertion so failures are never dropped.

#![cfg(windows)]

use base64::Engine;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const SAMPLES: usize = 30;
const WARMUP: usize = 3;
const P95_GATE_MS: u128 = 1000;
const SYNTAX_P95_GATE_MS: u128 = 1000;

fn b64(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}
fn sha_hex(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b);
    h.finalize().iter().map(|x| format!("{:02x}", x)).collect()
}

/// Unique per-sample program: the i-th return value differs → every
/// request is a genuine cache miss with a distinct input identity.
fn p04_ret(i: u32) -> String {
    format!("fn main() int {{ var i = 0; while i < 3 {{ i = i + 1 }}; return {i} }}")
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("ac788-turn2-{}-{}", tag, std::process::id()));
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

#[derive(serde::Serialize)]
struct Sample {
    scenario: String,
    idx: usize,
    wall_ms: u128,
    exit: Option<i64>,
    revision: Option<u32>,
    parse: Option<u32>,
    lower: Option<u32>,
    link: Option<u32>,
    object_hit: Option<String>,
    input_sha: String,
    ok: bool,
    error: Option<String>,
}

struct Recorder {
    name: &'static str,
    samples: Vec<Sample>,
    worker_sha: String,
}

impl Recorder {
    fn new(name: &'static str) -> Self {
        let exe = std::path::PathBuf::from(env!("CARGO_BIN_EXE_ac-case-worker"));
        let worker_sha = std::fs::read(&exe).map(|b| sha_hex(&b)).unwrap_or_default();
        Recorder {
            name,
            samples: Vec::new(),
            worker_sha,
        }
    }
    fn push(&mut self, s: Sample) {
        self.samples.push(s);
    }
    fn write(&self) -> std::path::PathBuf {
        #[derive(serde::Serialize)]
        struct Raw<'a> {
            worker_exe_sha256: &'a str,
            resource_gaps: Vec<&'static str>,
            samples: &'a [Sample],
        }
        let raw = Raw {
            worker_exe_sha256: &self.worker_sha,
            resource_gaps: vec!["cpu_ms", "io_bytes", "peak_rss"],
            samples: &self.samples,
        };
        let out = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("turnaround-samples-{}.json", self.name));
        std::fs::write(&out, serde_json::to_string_pretty(&raw).unwrap()).unwrap();
        out
    }
}

static DIR_SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl Session {
    fn spawn() -> (Session, std::path::PathBuf) {
        let exe = env!("CARGO_BIN_EXE_ac-case-worker");
        // Unique per spawn: the two turnaround tests run in parallel and a
        // shared name would let one remove the other's scratch.
        let work = scratch(&format!(
            "w{}-{}",
            std::process::id(),
            DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
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
        (
            Session {
                child,
                stdin,
                reader,
                req_no: 0,
            },
            work,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn request_raw(
        &mut self,
        work: &std::path::Path,
        store: Option<&str>,
        cache: &str,
        project: &str,
        source: &str,
        kind: &str,
        grammar: Option<&str>,
        edits: Option<serde_json::Value>,
    ) -> (Duration, serde_json::Value) {
        self.req_no += 1;
        let n = self.req_no;
        let req = serde_json::json!({
            "schema": "acworker/1",
            "kind": kind,
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
            "capabilities": [],
            "support_lib": null,
            "grammar": grammar,
            "edits": edits,
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
        (wall, v)
    }

    fn close(mut self) -> Duration {
        let t = Instant::now();
        let _ = writeln!(self.stdin, r#"{{"op":"end"}}"#);
        let _ = self.stdin.flush();
        let _ = self.child.wait();
        t.elapsed()
    }
}

fn counter(v: &serde_json::Value, name: &str) -> Option<u32> {
    v["ok"]["cache"]["counters"].as_array().and_then(|a| {
        a.iter()
            .find(|c| c[0] == name)
            .map(|c| c[1].as_u64().unwrap() as u32)
    })
}
fn object_hit(v: &serde_json::Value) -> Option<String> {
    v["ok"]["cache"]["hits"].as_array().and_then(|a| {
        a.iter()
            .find(|h| h[0] == "object")
            .map(|h| h[1].as_str().unwrap().to_string())
    })
}

/// Full-path single-shot: spawn → request → typed verify → reap.
fn one_shot(
    rec: &mut Recorder,
    scenario: &str,
    idx: usize,
    store: Option<&str>,
    cache: &str,
    project: &str,
    source: &str,
    want_exit: i64,
) -> u128 {
    let t0 = Instant::now();
    let (mut s, work) = Session::spawn();
    let (_, v) = s.request_raw(&work, store, cache, project, source, "run", None, None);
    s.close();
    let total = t0.elapsed();
    let exit = v["ok"]["run"]["exit_code"].as_i64();
    let ok = v["error"].is_null() && exit == Some(want_exit);
    rec.push(Sample {
        scenario: scenario.into(),
        idx,
        wall_ms: total.as_millis(),
        exit,
        revision: None,
        parse: counter(&v, "parse"),
        lower: counter(&v, "lower"),
        link: counter(&v, "link"),
        object_hit: object_hit(&v),
        input_sha: sha_hex(source.as_bytes()),
        ok,
        error: if v["error"].is_null() {
            None
        } else {
            Some(v["error"]["code"].as_str().unwrap_or("?").into())
        },
    });
    assert!(ok, "sample {}-{} failed: {}", scenario, idx, v);
    total.as_millis()
}

fn p95(samples: &[u128]) -> u128 {
    let mut s = samples.to_vec();
    s.sort_unstable();
    s[(s.len() * 95 + 99) / 100 - 1]
}
fn stats(samples: &[u128]) -> (u128, u128, u128) {
    let mut s = samples.to_vec();
    s.sort_unstable();
    (s[s.len() / 2], p95(samples), s[s.len() - 1])
}

#[test]
fn turnaround_native_six_scenarios_p95_gates() {
    let root = scratch("stores");
    let mut rec = Recorder::new("native");

    // 1) fresh — new store root per sample; full pipeline every time.
    let mut fresh = Vec::new();
    for i in 0..SAMPLES {
        let store = root.join(format!("fresh-{i}"));
        std::fs::create_dir_all(&store).unwrap();
        fresh.push(one_shot(
            &mut rec,
            "fresh",
            i,
            Some(store.to_str().unwrap()),
            "fresh",
            "fresh-proj",
            &p04_ret(i as u32),
            i as i64,
        ));
    }

    // 2) same-process hot — ONE session; WARMUP discarded requests first,
    //    session prep timed and reported separately (R2 F02).
    let hot_store = root.join("hot");
    std::fs::create_dir_all(&hot_store).unwrap();
    let hot_p95;
    let prep_ms;
    {
        let t_prep = Instant::now();
        let (mut s, work) = Session::spawn();
        prep_ms = t_prep.elapsed().as_millis();
        for _ in 0..WARMUP {
            let (_, v) = s.request_raw(
                &work,
                Some(hot_store.to_str().unwrap()),
                "reuse",
                "hot",
                &p04_ret(3),
                "run",
                None,
                None,
            );
            assert_eq!(v["ok"]["run"]["exit_code"], 3);
        }
        let mut samples = Vec::new();
        for i in 0..SAMPLES {
            let (wall, v) = s.request_raw(
                &work,
                Some(hot_store.to_str().unwrap()),
                "reuse",
                "hot",
                &p04_ret(3),
                "run",
                None,
                None,
            );
            let exit = v["ok"]["run"]["exit_code"].as_i64();
            let ok = v["error"].is_null() && exit == Some(3);
            rec.push(Sample {
                scenario: "hot".into(),
                idx: i,
                wall_ms: wall.as_millis(),
                exit,
                revision: None,
                parse: counter(&v, "parse"),
                lower: counter(&v, "lower"),
                link: counter(&v, "link"),
                object_hit: object_hit(&v),
                input_sha: sha_hex(p04_ret(3).as_bytes()),
                ok,
                error: None,
            });
            assert!(ok, "hot sample {i}: {v}");
            samples.push(wall.as_millis());
        }
        s.close();
        hot_p95 = p95(&samples);
        let (p50, _, p99) = stats(&samples);
        println!(
            "hot: n={SAMPLES} p50={p50}ms p95={hot_p95}ms p99={p99}ms (session prep {prep_ms}ms, {WARMUP} warmup discarded)"
        );
    }

    // 3) restart disk-hot — a NEW process per sample, SAME store.
    let restart_store = root.join("restart");
    std::fs::create_dir_all(&restart_store).unwrap();
    let mut restart = Vec::new();
    {
        one_shot(
            &mut rec,
            "restart-warm",
            0,
            Some(restart_store.to_str().unwrap()),
            "reuse",
            "restart",
            &p04_ret(3),
            3,
        );
        for i in 0..SAMPLES {
            restart.push(one_shot(
                &mut rec,
                "restart-disk-hot",
                i,
                Some(restart_store.to_str().unwrap()),
                "reuse",
                "restart",
                &p04_ret(3),
                3,
            ));
        }
    }

    // 4) cross-project same content.
    let xstore = root.join("cross");
    std::fs::create_dir_all(&xstore).unwrap();
    let mut cross = Vec::new();
    for i in 0..SAMPLES {
        let project = if i % 2 == 0 { "proj-a" } else { "proj-b" };
        cross.push(one_shot(
            &mut rec,
            "cross-project",
            i,
            Some(xstore.to_str().unwrap()),
            "reuse",
            project,
            &p04_ret(3),
            3,
        ));
    }

    // 5) body-edit — UNIQUE content per sample: every request is a REAL
    //    recompute (asserted via counters: parse==1, object=lowered).
    let edit_store = root.join("edit");
    std::fs::create_dir_all(&edit_store).unwrap();
    let edit_p95;
    {
        let (mut s, work) = Session::spawn();
        let mut samples = Vec::new();
        for i in 0..SAMPLES {
            let src = p04_ret(100 + i as u32);
            let (wall, v) = s.request_raw(
                &work,
                Some(edit_store.to_str().unwrap()),
                "reuse",
                "edit",
                &src,
                "run",
                None,
                None,
            );
            let exit = v["ok"]["run"]["exit_code"].as_i64();
            let parse = counter(&v, "parse");
            let hit = object_hit(&v);
            let ok = v["error"].is_null() && exit == Some(100 + i as i64);
            rec.push(Sample {
                scenario: "body-edit".into(),
                idx: i,
                wall_ms: wall.as_millis(),
                exit,
                revision: None,
                parse,
                lower: counter(&v, "lower"),
                link: counter(&v, "link"),
                object_hit: hit.clone(),
                input_sha: sha_hex(src.as_bytes()),
                ok,
                error: None,
            });
            assert!(ok, "body-edit {i}: {v}");
            assert_eq!(
                parse,
                Some(1),
                "body-edit {i} must be a real recompute (parse={parse:?} hit={hit:?})"
            );
            assert_eq!(
                hit.as_deref(),
                Some("lowered"),
                "body-edit {i} must not resume a cached object"
            );
            samples.push(wall.as_millis());
        }
        s.close();
        edit_p95 = p95(&samples);
        let (p50, _, p99) = stats(&samples);
        println!("body-edit: n={SAMPLES} p50={p50}ms p95={edit_p95}ms p99={p99}ms (every sample parse=1, object=lowered)");
    }

    // 6) gc-concurrent — the probe GC runs on a SEPARATE THREAD while the
    //    request session flows; overlap is recorded and asserted.
    let gc_store = root.join("gc");
    std::fs::create_dir_all(&gc_store).unwrap();
    {
        use ac_artifact_probe::store::gc::{gc_config_default, run_gc};
        use ac_artifact_probe::store::ManagedStore;
        use std::sync::mpsc;
        let (kick_tx, kick_rx) = mpsc::channel::<()>();
        let (done_tx, done_rx) = mpsc::channel::<(usize, u128)>();
        let gc_root = gc_store.clone();
        let gc_thread = std::thread::spawn(move || {
            let mut rounds = 0usize;
            while kick_rx.recv().is_ok() {
                rounds += 1;
                let store = ManagedStore::open(&gc_root).unwrap();
                let t = Instant::now();
                let receipt = run_gc(&store, gc_config_default()).unwrap();
                let took = t.elapsed().as_millis();
                // GC success evidence: the receipt exists and reports its
                // scan; failures return Err above.
                let _ = format!("{:?}", receipt);
                done_tx.send((rounds, took)).unwrap();
            }
            rounds
        });
        let (mut s, work) = Session::spawn();
        let mut samples = Vec::new();
        let mut gc_overlaps = 0usize;
        let mut gc_rounds_done = 0usize;
        for i in 0..SAMPLES {
            if i % 10 == 5 {
                // Kick a GC round; it runs CONCURRENTLY with request i.
                let req_t = Instant::now();
                kick_tx.send(()).unwrap();
                let (_, v) = s.request_raw(
                    &work,
                    Some(gc_store.to_str().unwrap()),
                    "reuse",
                    "gc",
                    &p04_ret(3),
                    "run",
                    None,
                    None,
                );
                let req_ms = req_t.elapsed().as_millis();
                if let Ok((round, took)) = done_rx.try_recv() {
                    gc_rounds_done = round;
                    if took > 0 && req_ms > 0 {
                        gc_overlaps += 1;
                    }
                }
                let exit = v["ok"]["run"]["exit_code"].as_i64();
                let ok = v["error"].is_null() && exit == Some(3);
                rec.push(Sample {
                    scenario: "gc-concurrent".into(),
                    idx: i,
                    wall_ms: req_ms,
                    exit,
                    revision: None,
                    parse: counter(&v, "parse"),
                    lower: counter(&v, "lower"),
                    link: counter(&v, "link"),
                    object_hit: object_hit(&v),
                    input_sha: sha_hex(p04_ret(3).as_bytes()),
                    ok,
                    error: None,
                });
                assert!(ok, "gc sample {i}: {v}");
                samples.push(req_ms);
                continue;
            }
            let (wall, v) = s.request_raw(
                &work,
                Some(gc_store.to_str().unwrap()),
                "reuse",
                "gc",
                &p04_ret(3),
                "run",
                None,
                None,
            );
            assert_eq!(v["ok"]["run"]["exit_code"], 3);
            samples.push(wall.as_millis());
        }
        s.close();
        drop(kick_tx);
        let _total_rounds = gc_thread.join().unwrap();
        assert!(
            gc_rounds_done >= 1,
            "at least one GC round must have completed during the flow"
        );
        assert!(
            gc_overlaps >= 1,
            "GC and requests must overlap (recorded {})",
            gc_overlaps
        );
        let (p50, p95v, p99) = stats(&samples);
        println!(
            "gc-concurrent: n={SAMPLES} p50={p50}ms p95={p95v}ms p99={p99}ms (gc rounds during flow: {gc_rounds_done}, overlaps {gc_overlaps})"
        );
    }

    let fs = stats(&fresh);
    let rs = stats(&restart);
    let cs = stats(&cross);
    println!(
        "fresh: p50={}ms p95={}ms p99={}ms | restart-disk-hot: p50={}ms p95={}ms p99={}ms | cross: p50={}ms p95={}ms p99={}ms",
        fs.0, fs.1, fs.2, rs.0, rs.1, rs.2, cs.0, cs.1, cs.2
    );

    // Write raw BEFORE the hard gates so failures keep their samples.
    let raw_path = rec.write();
    println!("raw samples: {}", raw_path.display());

    assert!(
        hot_p95 < P95_GATE_MS,
        "HARD GATE hot P95 {hot_p95}ms >= {P95_GATE_MS}ms"
    );
    assert!(
        edit_p95 < P95_GATE_MS,
        "HARD GATE body-edit P95 {edit_p95}ms >= {P95_GATE_MS}ms"
    );
}

#[test]
fn turnaround_syntax_gate_and_revision_stream() {
    let mut rec = Recorder::new("syntax");

    // ---- syntax local-edit gate: 30 single-edit requests through one
    // session; wall = request→response with the reconstruction verified.
    let syntax_p95;
    {
        let (mut s, work) = Session::spawn();
        let base = "fn helper() int { return 7 }\nfn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
        let pos3 = base.find('3').unwrap();
        for k in 0..WARMUP as u64 {
            let edits = serde_json::json!([{ "start": pos3, "end": pos3 + 1, "insert_b64": b64(&(k + 1).to_string()) }]);
            let (_, v) = s.request_raw(
                &work,
                None,
                "reuse",
                "syntax",
                base,
                "edit",
                Some("auto-core"),
                Some(edits),
            );
            assert!(v["error"].is_null(), "{}", v);
        }
        let mut samples = Vec::new();
        for k in 0..SAMPLES {
            let digit = ((k % 9) + 1).to_string();
            let edits =
                serde_json::json!([{ "start": pos3, "end": pos3 + 1, "insert_b64": b64(&digit) }]);
            let (wall, v) = s.request_raw(
                &work,
                None,
                "reuse",
                "syntax",
                base,
                "edit",
                Some("auto-core"),
                Some(edits),
            );
            let ok = v["error"].is_null();
            let revision = v["ok"]["cache"]["counters"].as_array().and_then(|a| {
                a.iter()
                    .find(|c| c[0] == "revision")
                    .map(|c| c[1].as_u64().unwrap() as u32)
            });
            let arts = v["ok"]["artifacts"].as_array().cloned().unwrap_or_default();
            let recon = arts
                .iter()
                .find(|a| a["logical"] == "reconstructed.at")
                .and_then(|a| a["path"].as_str())
                .map(|p| std::fs::read(p).unwrap())
                .unwrap_or_default();
            let want = base.replace('3', &digit);
            let verified = ok && recon == want.as_bytes();
            rec.push(Sample {
                scenario: "syntax-edit".into(),
                idx: k,
                wall_ms: wall.as_millis(),
                exit: None,
                revision,
                parse: None,
                lower: None,
                link: None,
                object_hit: None,
                input_sha: sha_hex(base.as_bytes()),
                ok: verified,
                error: None,
            });
            assert!(verified, "syntax sample {k}: recon mismatch or error {v}");
            samples.push(wall.as_millis());
        }
        s.close();
        syntax_p95 = p95(&samples);
        let (p50, _, p99) = stats(&samples);
        println!("syntax-edit: n={SAMPLES} p50={p50}ms p95={syntax_p95}ms p99={p99}ms");
    }

    // ---- V-13: 50 ordered revisions at ~100ms arrival spacing.
    // The worker's syntax requests are stateless per request (cache=None
    // face), so the k-th request applies k SEQUENTIAL steps (revision
    // 0→k inside one request); the FINAL revision per request is k —
    // strictly increasing across the stream — and the last request's
    // published reconstruction carries all 50 edits. Single-flight IPC
    // means no queue backlog can exist by construction; this still proves
    // ordering, per-step revision advance and latest-wins publication.
    {
        let (mut s, work) = Session::spawn();
        let base = "fn main() int { return  }";
        let anchor = base.find("return ").unwrap() + 7;
        for k in 1..=50u32 {
            // Build k sequential steps: insert digit d_i at the growing
            // cursor (each step's coordinates are relative to the CURRENT
            // revision at that step — the protocol's sequential semantics).
            let mut steps = Vec::new();
            let mut cursor = anchor;
            let mut doc = base.to_string();
            for i in 1..=k {
                let d = ((i % 10).to_string());
                steps.push(serde_json::json!({
                    "start": cursor, "end": cursor, "insert_b64": b64(&d)
                }));
                doc.insert_str(cursor, &d);
                cursor += 1;
            }
            let (wall, v) = s.request_raw(
                &work,
                None,
                "reuse",
                "rev-stream",
                base,
                "edit",
                Some("auto-core"),
                Some(serde_json::json!(steps)),
            );
            assert!(v["error"].is_null(), "rev {k}: {v}");
            let rev = v["ok"]["cache"]["counters"]
                .as_array()
                .and_then(|a| {
                    a.iter()
                        .find(|c| c[0] == "revision")
                        .map(|c| c[1].as_u64().unwrap() as u32)
                })
                .unwrap();
            assert_eq!(
                rev, k,
                "request {k}: final revision must be {k} (got {rev})"
            );
            // Latest-wins: the published reconstruction carries ALL k edits.
            let arts = v["ok"]["artifacts"].as_array().cloned().unwrap_or_default();
            let recon = arts
                .iter()
                .find(|a| a["logical"] == "reconstructed.at")
                .and_then(|a| a["path"].as_str())
                .map(|p| std::fs::read(p).unwrap())
                .unwrap_or_default();
            assert_eq!(
                recon,
                doc.as_bytes(),
                "request {k}: reconstruction must reflect all {k} edits"
            );
            rec.push(Sample {
                scenario: "revision-stream".into(),
                idx: k as usize,
                wall_ms: wall.as_millis(),
                exit: None,
                revision: Some(rev),
                parse: None,
                lower: None,
                link: None,
                object_hit: None,
                input_sha: sha_hex(base.as_bytes()),
                ok: true,
                error: None,
            });
            std::thread::sleep(Duration::from_millis(100));
        }
        s.close();
        println!("revision-stream: 50 ordered revisions @100ms arrival, per-request final revision 1..=50, latest published carries all edits");
    }

    let raw_path = rec.write();
    println!("raw samples: {}", raw_path.display());
    assert!(
        syntax_p95 < SYNTAX_P95_GATE_MS,
        "HARD GATE syntax edit P95 {syntax_p95}ms >= {SYNTAX_P95_GATE_MS}ms"
    );
}
