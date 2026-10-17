//! PLAN-788 R3 repairs, integration level (review-r3 F01–F06 closures).
//!
//! Everything here drives the REAL production entries:
//! - worker responses are judged through `test_cases::ac::validate_response`
//!   + `admit_and_persist_artifacts` + `compare_stream_field` — the same
//!   admission/judgment entry the runner itself uses (F02: measurement
//!   walls span request construction → IPC → parse → admission → judgment
//!   → persistence; nothing stops the clock "before validation");
//! - staleness goes through the REAL `auto-case` binary in a subprocess,
//!   swapping the worker via `AC_CASE_WORKER` (F05: production stale, not
//!   an injected axes map);
//! - over-cap streams / stderr floods / never-reading workers go through
//!   a MOCK worker binary (F03: the parent's own I/O lifecycle is under
//!   test, so the counterparty must be adversarial);
//! - the revision stream (F01) drives TWO real worker sessions on a fixed
//!   100ms arrival clock with heavy/light lanes, real backlog, superseded
//!   cancellation and a latest-wins publisher that REFUSES out-of-order
//!   publication;
//! - the GC scenario (F02) runs the REAL 783 GC as a separate process
//!   against the live store, with barrier lines proving the overlap
//!   window per sample.
//!
//! Raw samples are appended to a JSONL file PER SAMPLE (before any gate
//! assertion — a later panic never drops earlier samples; failures and
//! refusals are preserved).

#![cfg(all(windows, feature = "test-cases"))]

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use auto_lang::test_cases::ac::{
    admit_and_persist_artifacts, b64, b64d_strict, consume_and_run, validate_response,
    worker_binary, MAX_STREAM_BYTES,
};
use auto_test_case::contract::{Adapter, Action, CheckSpec, Compare, Completion, ExpectSpec,
                               Options, StreamExpect};

const P04: &str = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
const RUNNER: &str = env!("CARGO_BIN_EXE_auto-case");

// ---------------------------------------------------------------------------
// shared helpers
// ---------------------------------------------------------------------------

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("p788r3-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sha_hex(b: &[u8]) -> String {
    // same content hash the runner binds (auto-test-case sha256)
    auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(b))
}

/// Body-edit recipe (§5.5 frozen): the p04 loop bound REALLY alternates
/// 3↔4 (independent exit 3/4 oracle), and every sample carries a unique
/// marker comment so no sample ever resumes a previous one's object.
fn p04_bound_edit(idx: usize) -> (String, i64) {
    let bound = if idx % 2 == 0 { 3 } else { 4 };
    let src = format!(
        "// p788-r3 body-edit unique marker {idx}\nfn main() int {{ var i = 0; while i < {bound} {{ i = i + 1 }}; return i }}"
    );
    (src, bound as i64)
}

// ---------------------------------------------------------------------------
// worker session (raw JSON-lines IPC against the real worker binary)
// ---------------------------------------------------------------------------

struct Sess {
    child: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
    req_no: u32,
}

static DIR_SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl Sess {
    fn spawn(tag: &str) -> (Sess, PathBuf) {
        let exe = worker_binary().expect("worker binary located/built");
        Sess::spawn_exe(&exe, tag, &[])
    }

    /// R5: spawn a session against an arbitrary counterparty binary (the
    /// fault-injection relay below); env pairs travel to that process.
    fn spawn_exe(exe: &Path, tag: &str, env: &[(&str, String)]) -> (Sess, PathBuf) {
        let work = std::env::temp_dir().join(format!(
            "p788r3-w-{}-{}-{}",
            tag,
            std::process::id(),
            DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&work);
        std::fs::create_dir_all(&work).unwrap();
        let mut cmd = Command::new(exe);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        for (k, v) in env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().unwrap();
        let reader = BufReader::new(child.stdout.take().unwrap());
        let mut stdin = child.stdin.take().unwrap();
        writeln!(stdin, r#"{{"op":"begin","schema":"acworker/1"}}"#).unwrap();
        stdin.flush().unwrap();
        (
            Sess {
                child,
                stdin,
                reader,
                req_no: 0,
            },
            work,
        )
    }

    fn next_ids(&mut self) -> (String, String) {
        self.req_no += 1;
        (format!("n-{}", self.req_no), format!("h-{}", self.req_no))
    }

    fn close(mut self) {
        let _ = writeln!(self.stdin, r#"{{"op":"end"}}"#);
        let _ = self.stdin.flush();
        let _ = self.child.wait();
    }
}

fn ac_run_request(source: &str, store: Option<&str>, cache: &str, project: &str, work: &Path, budget_total: u64) -> serde_json::Value {
    serde_json::json!({
        "schema": "acworker/1",
        "kind": "run",
        "nonce": "placeholder",
        "request_hash": "placeholder",
        "input": { "kind": "source", "source_b64": b64(source.as_bytes()) },
        "profile": "core",
        "entry": "main",
        "budget": { "build_ms": 60000, "run_ms": 5000, "total_ms": budget_total },
        "scratch": work.display().to_string(),
        "store_root": store,
        "cache": cache,
        "project": project,
        "include_hir": false,
        "capabilities": [],
        "support_lib": null,
        "grammar": null,
        "edits": null,
    })
}

// The nonce/request_hash travel on the ENVELOPE's request object; patch the
// placeholders after building.
fn patch_ids(req: &mut serde_json::Value, nonce: &str, hash: &str) {
    req["nonce"] = serde_json::json!(nonce);
    req["request_hash"] = serde_json::json!(hash);
}

/// Full judgment of one ac/run session response through the runner's own
/// entries (F02): typed admission → manifest admission + persistence →
/// exit oracle. Returns (exit_ok, spans, counters snapshot fields).
fn judge_ac_run(
    v: &serde_json::Value,
    nonce: &str,
    hash: &str,
    work: &Path,
    art_dir: &Path,
    want_exit: i64,
) -> Result<(bool, Vec<(String, u64)>), String> {
    validate_response(v, nonce, hash, "run")?;
    if v.get("error").map(|e| !e.is_null()).unwrap_or(false) {
        return Err(format!(
            "worker error [{}]: {}",
            v["error"]["code"].as_str().unwrap_or("?"),
            v["error"]["message"].as_str().unwrap_or("")
        ));
    }
    admit_and_persist_artifacts(&v["ok"], Adapter::Ac, work, art_dir)?;
    let exit = v["ok"]["run"]["exit_code"].as_i64();
    let spans = v["ok"]["trace"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .filter_map(|s| Some((s["stage"].as_str()?.to_string(), s["wall_ms"].as_u64()?)))
        .collect();
    Ok((exit == Some(want_exit), spans))
}


/// R4 (F05): a judgment failure is RECORDED before it can fail the gate —
/// every scenario loop turns the Result into (ok, spans, error), pushes the
/// Sample, and only THEN asserts. An admission failure or invalid response
/// can no longer vanish with the panicking sample.
fn judged_fields(
    judged: &Result<(bool, Vec<(String, u64)>), String>,
) -> (bool, Vec<(String, u64)>, Option<String>) {
    match judged {
        Ok((ok, spans)) => (*ok, spans.clone(), None),
        Err(e) => (false, Vec::new(), Some(e.clone())),
    }
}

// ---------------------------------------------------------------------------
// sample recorder: JSONL appended PER SAMPLE, before any gate
// ---------------------------------------------------------------------------

#[derive(serde::Serialize)]
struct Sample {
    scenario: String,
    idx: usize,
    wall_ms: u128,
    queue_ms: Option<u128>,
    exit: Option<i64>,
    revision: Option<u32>,
    parse: Option<u32>,
    lower: Option<u32>,
    link: Option<u32>,
    object_hit: Option<String>,
    input_sha: String,
    ok: bool,
    error: Option<String>,
    stage_spans: Vec<(String, u64)>,
    gc: Option<GcInterval>,
}

#[derive(serde::Serialize)]
struct GcInterval {
    started_before_req: bool,
    finished_after_req_start: bool,
    /// Overlap in MICROSECONDS — millisecond truncation turned sub-ms real
    /// overlaps into 0 in review runs; µs keeps the proof exact.
    overlap_us: u128,
    summary: String,
}

struct Recorder {
    path: PathBuf,
    worker_sha: String,
    count: usize,
}

impl Recorder {
    fn new(name: &str) -> Recorder {
        let exe = worker_binary().expect("worker binary");
        let worker_sha = std::fs::read(&exe).map(|b| sha_hex(&b)).unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "p788r3-raw-{}-{}.jsonl",
            name,
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        // header record: environment identity for the raw file
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        let _ = writeln!(
            f,
            "{}",
            serde_json::json!({
                "kind": "header",
                "worker_exe_sha256": worker_sha,
                "resource_gaps": ["cpu_ms", "io_bytes", "peak_rss"],
                "recorded_per_sample_before_gates": true,
                // R4 (F05): the measurement environment travels with the
                // raw file — attribution of long tails starts here.
                "measured_at_unix_ms": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                "harness": "auto-lang integration test plan788_r3_ (--test-threads=1 expected)",
            })
        );
        Recorder {
            path,
            worker_sha,
            count: 0,
        }
    }

    /// R3 (F02): EVERY sample is durable the moment it exists — a panic in
    /// a later assertion, a decode error or a GC unwrap can no longer drop
    /// the evidence.
    fn push(&mut self, s: Sample) {
        let mut f = std::fs::OpenOptions::new().append(true).open(&self.path).unwrap();
        writeln!(f, "{}", serde_json::to_string(&s).unwrap()).unwrap();
        f.flush().unwrap();
        self.count += 1;
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
    v["ok"]["cache"]["hits"]
        .as_array()
        .and_then(|a| a.iter().find(|h| h[0] == "object").map(|h| h[1].as_str().unwrap().to_string()))
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

// ---------------------------------------------------------------------------
// the real 783 GC as a concurrent PROCESS (F02)
// ---------------------------------------------------------------------------

fn store_gc_binary() -> PathBuf {
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
            let manifest = loop {
                let Some(d) = dir else {
                    return Err("ac-artifact-probe not located above the crate".into());
                };
                let m = d.join("experimental").join("ac-artifact-probe").join("Cargo.toml");
                if m.is_file() {
                    break m;
                }
                dir = d.parent();
            };
            let status = Command::new("cargo")
                .args(["build", "--release", "--manifest-path"])
                .arg(&manifest)
                .arg("--bin")
                .arg("store-gc")
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .output()
                .map_err(|e| format!("spawn cargo for store-gc: {}", e))?;
            if !status.status.success() {
                return Err(format!(
                    "store-gc build failed: {}",
                    String::from_utf8_lossy(&status.stderr)
                ));
            }
            let exe = manifest.parent().unwrap().join("target/release/store-gc.exe");
            exe.is_file()
                .then_some(exe)
                .ok_or_else(|| "store-gc.exe missing after build".to_string())
        })
        .clone()
        .unwrap_or_else(|e| panic!("store-gc binary: {e}"))
}

/// One GC round as a concurrent process with barrier lines. Returns the
/// finish-channel after "gc-started" was observed; the caller checks the
/// pre-request barrier via try_recv on it.
fn spawn_gc_round(store: &Path) -> std::sync::mpsc::Receiver<(String, Instant)> {
    let bin = store_gc_binary();
    let mut child = Command::new(&bin)
        .arg(store)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn store-gc");
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    reader.read_line(&mut line).expect("read gc-started");
    assert_eq!(line.trim(), "gc-started", "store-gc barrier line");
    let (tx, rx) = std::sync::mpsc::channel::<(String, Instant)>();
    std::thread::spawn(move || {
        let mut fin = String::new();
        let _ = reader.read_line(&mut fin);
        let at = Instant::now();
        let _ = child.wait();
        let _ = tx.send((fin.trim().to_string(), at));
    });
    rx
}

// ---------------------------------------------------------------------------
// mock worker (adversarial counterparty for the parent's own lifecycle)
// ---------------------------------------------------------------------------

fn mock_worker() -> PathBuf {
    static CACHE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        let root = temp_root("mock");
        let src = root.join("mock.rs");
        std::fs::write(
            &src,
            r#"
use std::io::{self, BufRead, Write};
fn field(s:&str,k:&str)->String{
    s.split(&format!("\"{}\":\"",k)).nth(1).unwrap().split('"').next().unwrap().to_string()
}
fn main(){
    if std::env::var("P788R3_MODE").unwrap_or_default()=="no-read"{
        std::thread::sleep(std::time::Duration::from_millis(1200));return;
    }
    for l in io::stdin().lock().lines(){
        let l=l.unwrap();
        if !l.contains("\"request\":"){continue;}
        let n=field(&l,"nonce");let h=field(&l,"request_hash");
        let scratch=field(&l,"scratch").replace("\\\\","\\");
        std::fs::write(std::path::Path::new(&scratch).join("app.exe"),b"x").unwrap();
        let mode=std::env::var("P788R3_MODE").unwrap_or_default();
        if mode=="stderr-overcap"||mode=="stderr-overcap-slow"{
            let _=io::stderr().write_all(&vec![b'z';(1<<20)+1]);
        }
        if mode=="stderr-overcap-slow"{
            // linger BEFORE answering — an immediate-kill parent ends the
            // run at the cap; a merely-late parent waits the full linger
            std::thread::sleep(std::time::Duration::from_millis(1200));
        }
        // JSON-escape the substituted path (backslashes) so the printed
        // line stays a legal JSON string.
        let path_sub=format!("{}\\app.exe",scratch).replace('\\',"\\\\");
        let ok=std::fs::read_to_string(std::env::var("P788R3_OK_FILE").unwrap())
            .unwrap().replace("$path",&path_sub);
        println!("{{\"schema\":\"acworker/1\",\"nonce\":\"{}\",\"request_hash\":\"{}\",\"ok\":{}}}",n,h,ok);
        return;
    }
}
"#,
        )
        .unwrap();
        let exe = root.join("mock-worker.exe");
        let out = Command::new("rustc")
            .arg(&src)
            .arg("-o")
            .arg(&exe)
            .output()
            .expect("rustc for mock worker");
        assert!(
            out.status.success(),
            "mock rustc: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        exe
    })
    .clone()
}

/// Run the real auto-case binary over a generated single-case suite and
/// return (exit_code, report_json). `worker_env` overrides AC_CASE_WORKER.
fn run_suite_subprocess(
    root: &Path,
    tag: &str,
    source: &str,
    syntax: &str,
    checks: &str,
    worker_env: Option<&Path>,
    extra_env: &[(&str, String)],
) -> (i32, serde_json::Value) {
    let dir = root.join(tag);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("suite.ad"),
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"r3\", include: [\"case.ad\"] }\n```\n",
    )
    .unwrap();
    let path = if syntax == "hir-atom" { "main.atom" } else { "main.at" };
    std::fs::write(
        dir.join("case.ad"),
        format!(
            "```test-case\ncase {{ schema: \"autolang.case/2\", id: \"r3.case\", source: {{ text: {}, syntax: {}, path: {} }}, entry: {{ mode: \"program\" }}, checks: [ {} ] }}\n```\n",
            serde_json::to_string(source).unwrap(),
            serde_json::to_string(syntax).unwrap(),
            serde_json::to_string(path).unwrap(),
            checks
        ),
    )
    .unwrap();
    let out = root.join(format!("{}-out", tag));
    let report = root.join(format!("{}-report.json", tag));
    let mut cmd = Command::new(RUNNER);
    cmd.args(["run", "--suite"])
        .arg(dir.join("suite.ad"))
        .arg("--out")
        .arg(&out)
        .arg("--json-out")
        .arg(&report);
    if let Some(w) = worker_env {
        cmd.env("AC_CASE_WORKER", w);
    }
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    let res = cmd.output().expect("run auto-case");
    let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap())
        .expect("report json");
    (res.status.code().unwrap_or(-1), json)
}

fn stale_subprocess(
    suite: &Path,
    report: &Path,
    worker_env: Option<&Path>,
) -> (i32, String) {
    let mut cmd = Command::new(RUNNER);
    cmd.args(["stale", "--suite"]).arg(suite).arg("--report").arg(report);
    if let Some(w) = worker_env {
        cmd.env("AC_CASE_WORKER", w);
    }
    let res = cmd.output().expect("stale auto-case");
    (
        res.status.code().unwrap_or(-1),
        format!("{}{}", String::from_utf8_lossy(&res.stdout), String::from_utf8_lossy(&res.stderr)),
    )
}

fn row<'a>(report: &'a serde_json::Value, check: &str) -> &'a serde_json::Value {
    report["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["check"] == check)
        .unwrap_or_else(|| panic!("check {check} in report: {report}"))
}

// ---------------------------------------------------------------------------
// F03: caps + stdin deadline (mock counterparty through the REAL runner)
// ---------------------------------------------------------------------------

#[test]
fn r3_parent_io_lifecycle_overcap_and_deadline() {
    let root = temp_root("io-life");
    let mock = mock_worker();

    // Valid response shape the mock echoes (the manifest points at the
    // app.exe the mock itself writes into the scratch).
    let valid_ok = serde_json::json!({
        "diagnostics": [],
        "artifacts": [{
            "kind": "exe", "logical": "app.exe", "path": "$path",
            "bytes": 1, "sha256": sha_hex(b"x"),
        }],
        "run": { "exit_code": 3, "stdout_b64": "", "stderr_b64": "" },
        "trace": [],
        "toolchain": { "worker_exe_sha256": "abc" },
        "metric_gaps": [],
    })
    .to_string();

    // (a) stdout over cap (1 MiB + 1) with NO declared stream oracle — the
    // decoded cap must refuse regardless of comparison.
    let over_ok = serde_json::json!({
        "diagnostics": [],
        "artifacts": [{
            "kind": "exe", "logical": "app.exe", "path": "$path",
            "bytes": 1, "sha256": sha_hex(b"x"),
        }],
        "run": { "exit_code": 3, "stdout_b64": b64(&vec![b'x'; MAX_STREAM_BYTES + 1]), "stderr_b64": "" },
        "trace": [],
        "toolchain": { "worker_exe_sha256": "abc" },
        "metric_gaps": [],
    })
    .to_string();
    let okfile = root.join("ok-over.json");
    std::fs::write(&okfile, &over_ok).unwrap();
    let (gate, report) = run_suite_subprocess(
        &root,
        "stdout-overcap",
        "fn main() int { return 3 }",
        "auto",
        r#"{ id: "run", adapter: "ac", action: "run", expect: { completion: "success", exit: 3 } }"#,
        Some(&mock),
        &[("P788R3_OK_FILE", okfile.display().to_string())],
    );
    assert_ne!(gate, 0, "an over-cap stream can never be a green run");
    assert_eq!(row(&report, "run")["status"], "ERROR", "{report}");
    assert!(
        row(&report, "run")["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("stream cap"),
        "refusal reason names the cap: {report}"
    );

    // (b) stderr flood (1 MiB + 1) then a VALID response — over-cap stderr
    // is a refusal, never a truncated success.
    let okfile = root.join("ok-valid.json");
    std::fs::write(&okfile, &valid_ok).unwrap();
    let (gate, report) = run_suite_subprocess(
        &root,
        "stderr-overcap",
        "fn main() int { return 3 }",
        "auto",
        r#"{ id: "run", adapter: "ac", action: "run", expect: { completion: "success", exit: 3 } }"#,
        Some(&mock),
        &[
            ("P788R3_OK_FILE", okfile.display().to_string()),
            ("P788R3_MODE", "stderr-overcap".to_string()),
        ],
    );
    assert_ne!(gate, 0);
    assert_eq!(row(&report, "run")["status"], "ERROR", "{report}");
    assert!(
        row(&report, "run")["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("stderr exceeded"),
        "stderr over-cap refusal: {report}"
    );

    // (b2, R4-03) stderr flood then a 1200ms LINGER before answering,
    // total budget 3000ms: the parent must TERMINATE the worker at the cap
    // (well under the linger), not merely refuse its late answer.
    let t0 = Instant::now();
    let (gate, report) = run_suite_subprocess(
        &root,
        "stderr-overcap-slow",
        "fn main() int { return 3 }",
        "auto",
        r#"{ id: "run", adapter: "ac", action: "run", options: { budget: { total_ms: 3000 } }, expect: { completion: "success", exit: 3 } }"#,
        Some(&mock),
        &[
            ("P788R3_OK_FILE", okfile.display().to_string()),
            ("P788R3_MODE", "stderr-overcap-slow".to_string()),
        ],
    );
    let wall = t0.elapsed().as_millis();
    assert_ne!(gate, 0);
    assert_eq!(row(&report, "run")["status"], "ERROR", "{report}");
    let detail = row(&report, "run")["detail"].as_str().unwrap_or_default().to_string();
    assert!(
        detail.contains("stderr exceeded"),
        "refusal names the stderr cap: {report}"
    );
    assert!(
        detail.contains("TERMINATED at the cap"),
        "the refusal must state the immediate termination: {report}"
    );
    assert!(
        wall < 700,
        "the worker must be reaped AT the cap, not after its 1200ms linger (took {wall}ms)"
    );

    // (c) a worker that NEVER READS stdin, 100ms total budget, ~100KB
    // source: the parent's deadline must cover the WRITE too — the old
    // synchronous write blocked ~1.2s on the full pipe.
    let big_source = format!("fn main() int {{ return 3 }}{}", " ".repeat(100_000));
    let t0 = Instant::now();
    let (gate, report) = run_suite_subprocess(
        &root,
        "stdin-deadline",
        &big_source,
        "auto",
        r#"{ id: "run", adapter: "ac", action: "run", options: { budget: { total_ms: 100 } }, expect: { completion: "success", exit: 3 } }"#,
        Some(&mock),
        &[("P788R3_MODE", "no-read".to_string())],
    );
    let wall = t0.elapsed().as_millis();
    assert_ne!(gate, 0);
    assert_eq!(row(&report, "run")["status"], "ERROR", "{report}");
    assert!(
        row(&report, "run")["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("did not answer within"),
        "deadline kill reason: {report}"
    );
    assert!(
        wall < 900,
        "the deadline must bound the whole I/O lifecycle (spawn+write+kill+reap); took {wall}ms (the old blocked-write shape was ~1267ms)"
    );

    // (d) F04 red control: diagnostics + a forged SUCCESS artifact (exe)
    // is refused even though the exe manifest itself is well-formed.
    let forged_ok = serde_json::json!({
        "diagnostics": [{
            "stage": "backend", "code": "entry.not-found",
            "message": "entry absent", "span_start": 0, "span_end": 1,
        }],
        "artifacts": [{
            "kind": "exe", "logical": "app.exe", "path": "$path",
            "bytes": 1, "sha256": sha_hex(b"x"),
        }],
        "trace": [],
        "toolchain": { "worker_exe_sha256": "abc" },
        "metric_gaps": [],
    })
    .to_string();
    let okfile = root.join("ok-forged.json");
    std::fs::write(&okfile, &forged_ok).unwrap();
    let (gate, report) = run_suite_subprocess(
        &root,
        "forged-artifacts",
        "fn main() int { return 3 }",
        "auto",
        r#"{ id: "build", adapter: "ac", action: "build", options: { entry: "absent" }, expect: { completion: { stage: "backend", code: "entry.not-found" } } }"#,
        Some(&mock),
        &[("P788R3_OK_FILE", okfile.display().to_string())],
    );
    assert_ne!(gate, 0);
    assert_eq!(row(&report, "build")["status"], "ERROR", "{report}");
    assert!(
        row(&report, "build")["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("success artifacts"),
        "forged-product refusal: {report}"
    );

    // (e) decode-cap boundary unit (exactly at the cap is legal):
    let mut ok = serde_json::json!({
        "schema": "acworker/1", "nonce": "n", "request_hash": "h",
        "ok": {
            "diagnostics": [],
            "artifacts": [],
            "run": { "exit_code": 3, "stdout_b64": b64(&vec![b'x'; MAX_STREAM_BYTES]), "stderr_b64": "" },
            "trace": [],
            "toolchain": { "worker_exe_sha256": "abc" },
            "metric_gaps": [],
        },
    });
    validate_response(&ok, "n", "h", "run").expect("exactly-at-cap stream is legal");
    ok["ok"]["run"]["stdout_b64"] = serde_json::json!(b64(&vec![b'x'; MAX_STREAM_BYTES + 1]));
    let err = validate_response(&ok, "n", "h", "run").unwrap_err();
    assert!(err.contains("stream cap"), "{err}");
}

// ---------------------------------------------------------------------------
// F04: legal partial products through the REAL worker
// ---------------------------------------------------------------------------

#[test]
fn r3_hir_backend_negative_keeps_legal_partial_hir() {
    let root = temp_root("hir-partial");
    let hir = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experimental/ac-core/fixtures/valid/wrapper.atom"),
    )
    .unwrap();
    let (gate, report) = run_suite_subprocess(
        &root,
        "hir-backend-neg",
        &hir,
        "hir-atom",
        r#"{ id: "build", adapter: "ac", action: "build", options: { entry: "absent" }, expect: { completion: { stage: "backend", code: "entry.not-found" } } }"#,
        None,
        &[],
    );
    let r = row(&report, "build");
    assert_eq!(
        r["status"], "PASS",
        "a typed backend negative with the legal canonical-HIR partial product must PASS: {report}"
    );
    assert_eq!(gate, 0);
    // The partial product is admitted AND persisted (producer-validity:
    // the frontend really produced it before the failing boundary).
    assert!(
        r["artifacts"]
            .as_array()
            .map(|a| a.iter().any(|x| x["path"] == "hir.atom"))
            .unwrap_or(false),
        "hir.atom partial product admitted: {report}"
    );
}

// ---------------------------------------------------------------------------
// F05: production stale paths (subprocess, real auto-case)
// ---------------------------------------------------------------------------

#[test]
fn r3_stale_production_paths_per_adapter_action() {
    let root = temp_root("stale");
    // An "altered" worker: the same bytes + one appended byte → a different
    // content hash; never executed, only hashed by the identity axis.
    let real = worker_binary().expect("worker binary");
    let mut bytes = std::fs::read(&real).unwrap();
    bytes.push(0x78);
    let altered = root.join("worker-altered.exe");
    std::fs::write(&altered, &bytes).unwrap();

    // (a) ac/check unchanged → FRESH (lld/sdk axes NOT applicable to a
    // check row — the R3 bug made this STALE).
    let (gate, report) = run_suite_subprocess(
        &root,
        "check-run",
        "fn main() int { return 3 }",
        "auto",
        r#"{ id: "chk", adapter: "ac", action: "check", expect: { completion: "success" } }"#,
        None,
        &[],
    );
    assert_eq!(gate, 0, "{report}");
    let suite = root.join("check-run").join("suite.ad");
    let report_path = root.join("check-run-report.json");
    let (stale_gate, out) = stale_subprocess(&suite, &report_path, None);
    assert_eq!(
        stale_gate, 0,
        "ac/check row with unchanged worker/tools must be FRESH: {out}"
    );

    // (b) ac/run (a LINKED row) unchanged → FRESH: the lld/sdk CONTENT
    // axes are applicable here and equal.
    let (gate, report) = run_suite_subprocess(
        &root,
        "run-run",
        "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }",
        "auto",
        r#"{ id: "run", adapter: "ac", action: "run", expect: { completion: "success", exit: 3 } }"#,
        None,
        &[],
    );
    assert_eq!(gate, 0, "{report}");
    let suite = root.join("run-run").join("suite.ad");
    let report_path = root.join("run-run-report.json");
    let (stale_gate, out) = stale_subprocess(&suite, &report_path, None);
    assert_eq!(
        stale_gate, 0,
        "a linked ac/run row with unchanged linker/SDK content must be FRESH: {out}"
    );

    // (b2) ac/build (a LINKED row with NO run observation — the R4 bug
    // read the wrong artifact key and flagged it STALE unchanged) → FRESH.
    let (gate, report) = run_suite_subprocess(
        &root,
        "build-run",
        "fn main() int { return 3 }",
        "auto",
        r#"{ id: "build", adapter: "ac", action: "build", expect: { completion: "success" } }"#,
        None,
        &[],
    );
    assert_eq!(gate, 0, "{report}");
    let suite = root.join("build-run").join("suite.ad");
    let report_path = root.join("build-run-report.json");
    let (stale_gate, out) = stale_subprocess(&suite, &report_path, None);
    assert_eq!(
        stale_gate, 0,
        "an unchanged LINKED ac/build row (obj/exe artifacts, lld/sdk axes applicable and equal) must be FRESH: {out}"
    );

    // (b3) the same build row under a REPLACED worker → STALE.
    let (stale_gate, out) = stale_subprocess(&suite, &report_path, Some(&altered));
    assert_eq!(stale_gate, 1, "build row under a different worker: {out}");
    assert!(out.contains("worker-exe-sha256"), "drift names the axis: {out}");

    // (c) syntax parse with a REPLACED worker → STALE naming the axis (the
    // R3 bug left syntax rows FRESH after a worker swap).
    let (gate, report) = run_suite_subprocess(
        &root,
        "syntax-run",
        "fn main() int { return 3 }",
        "auto",
        r#"{ id: "parse", adapter: "syntax", action: "parse", options: { grammar: "auto-core" }, expect: { completion: "success" } }"#,
        None,
        &[],
    );
    assert_eq!(gate, 0, "{report}");
    let suite = root.join("syntax-run").join("suite.ad");
    let report_path = root.join("syntax-run-report.json");
    let (stale_gate, out) = stale_subprocess(&suite, &report_path, Some(&altered));
    assert_eq!(
        stale_gate, 1,
        "a syntax row judged under a DIFFERENT worker binary must be STALE: {out}"
    );
    assert!(
        out.contains("worker-exe-sha256"),
        "the drift must name the axis: {out}"
    );

    // (d) same swap on the ac/check row → STALE as well (both adapters
    // bind the worker identity).
    let suite = root.join("check-run").join("suite.ad");
    let report_path = root.join("check-run-report.json");
    let (stale_gate, out) = stale_subprocess(&suite, &report_path, Some(&altered));
    assert_eq!(stale_gate, 1, "check row under a different worker: {out}");
}

// ---------------------------------------------------------------------------
// V-06 producers through the REAL worker path (F01)
// ---------------------------------------------------------------------------

/// A real PE (rustc-built fixture) whose run writes NUL + CRLF to stdout
/// and INVALID UTF-8 to stderr, exit 7. The producer is synthetic; the
/// PATH is the real one (contained executor → caps → base64 wire →
/// parent-side strict decode → byte oracle).
fn bytes_fixture_exe(root: &Path) -> PathBuf {
    let src = root.join("bytes-prod.rs");
    std::fs::write(
        &src,
        r#"
fn main(){
    let out: &[u8] = b"A\x00B\r\nC";
    use std::io::Write;
    let mut o = std::io::stdout();
    let _ = o.write_all(out);
    let _ = o.flush();
    let err: &[u8] = &[0xff, 0xfe, 0x80, b'x'];
    let mut e = std::io::stderr();
    let _ = e.write_all(err);
    let _ = e.flush();
    std::process::exit(7);
}
"#,
    )
    .unwrap();
    let exe = root.join("bytes-prod.exe");
    let out = Command::new("rustc")
        .arg(&src)
        .arg("-o")
        .arg(&exe)
        .output()
        .expect("rustc bytes fixture");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    exe
}

fn consumer_check(id: &str, exit: i32) -> CheckSpec {
    CheckSpec {
        id: id.into(),
        adapter: Adapter::Ac,
        action: Action::Run,
        tier: None,
        options: Options::default(),
        expect: ExpectSpec {
            completion: Completion::Success,
            result: None,
            stdout: None,
            stderr: None,
            artifacts: vec![],
            exit: Some(exit),
            perf: None,
        },
        product_from: Some("upstream".into()),
        skip: None,
    }
}

#[test]
fn r3_native_bytes_real_worker_path_producers() {
    let root = temp_root("bytes");
    let exe = bytes_fixture_exe(&root);

    // (a) NUL + CRLF compared as EXACT BYTES through consume_and_run (the
    // full runner consumer path: worker executor → wire → parent oracle).
    let check = consumer_check("bytes", 7);
    let verdict = consume_and_run(
        &exe,
        &check,
        Some(7),
        Some(&StreamExpect {
            text: "A\0B\r\nC".to_string(),
            compare: Compare::Bytes,
        }),
        None,
        None,
    )
    .expect("consumer run completes");
    assert_eq!(
        verdict.status,
        auto_test_case::report::Status::Pass,
        "NUL/CRLF must round-trip byte-exact through the real worker path: {:?}",
        verdict.mismatches
    );
    assert_eq!(verdict.native.as_ref().unwrap().exit_code, Some(7));

    // (b) INVALID UTF-8 on the undeclared stderr stream: it survives the
    // wire unchanged (strict decode reproduces the exact bytes) and the
    // response still passes full admission — judged through the runner's
    // own decoder, on a raw session against the real worker.
    let (mut s, work) = Sess::spawn("bytes");
    let (nonce, hash) = s.next_ids();
    let mut req = serde_json::json!({
        "schema": "acworker/1",
        "kind": "run",
        "nonce": nonce,
        "request_hash": hash,
        "input": { "kind": "product", "exe_path": exe.display().to_string() },
        "profile": "core",
        "entry": "main",
        "budget": { "build_ms": 60000, "run_ms": 5000, "total_ms": 70000 },
        "scratch": work.display().to_string(),
    });
    let _ = &mut req;
    let t = Instant::now();
    let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
    writeln!(s.stdin, "{}", line).unwrap();
    s.stdin.flush().unwrap();
    let mut resp = String::new();
    s.reader.read_line(&mut resp).unwrap();
    let v: serde_json::Value = serde_json::from_str(resp.trim()).unwrap();
    let _ = t;
    // R5 (F01): admission runs against the REQUEST-side identity — the
    // response echo may never authorize itself.
    validate_response(&v, &nonce, &hash, "run").expect("invalid-UTF-8 stderr survives admission");
    let stderr_bytes = b64d_strict(v["ok"]["run"]["stderr_b64"].as_str().unwrap()).unwrap();
    assert_eq!(
        stderr_bytes,
        vec![0xff, 0xfe, 0x80, b'x'],
        "invalid UTF-8 bytes preserved exactly through the wire"
    );
    let stdout_bytes = b64d_strict(v["ok"]["run"]["stdout_b64"].as_str().unwrap()).unwrap();
    assert_eq!(stdout_bytes, b"A\x00B\r\nC");
    assert_eq!(v["ok"]["run"]["exit_code"].as_i64(), Some(7));
    s.close();
}


/// R4-01: the INDEPENDENT latest-wins publication oracle. A completion may
/// publish only when it is (a) not superseded, (b) newer than everything
/// already published, (c) at least as new as the newest revision already
/// ENQUEUED (a stale completion must not publish over a newer arrival),
/// (d) judged Ok with the BYTE ORACLE TRUE (an Ok(false) never publishes),
/// and (e) the worker's own revision counter equals the request's step
/// count. Everything else gets an honest refusal category; the categories
/// are unit-tested with negative controls below.
fn decide_publication(
    cancelled: bool,
    k: u32,
    last_published: u32,
    max_enqueued: u32,
    judged: &Result<(bool, Option<u32>), String>,
    steps: u32,
) -> &'static str {
    if cancelled {
        return "cancelled(superseded-late)";
    }
    if k <= last_published {
        return "refused-out-of-order";
    }
    if k < max_enqueued {
        return "refused-stale(newer-already-issued)";
    }
    match judged {
        Err(_) => "refused-judgment",
        Ok((false, _)) => "refused-oracle-mismatch",
        Ok((true, Some(c))) if *c != steps => "refused-revision-counter",
        Ok((true, Some(_))) => "published",
        // a missing counter is an incomplete observation — never a pass
        Ok((true, None)) => "refused-judgment",
    }
}

/// R4-01: negative controls for the publication oracle — wrong bytes
/// (Ok(false)), admission errors, wrong/missing revision counters, a newer
/// already-issued revision and superseded tasks must ALL be refused; only
/// the clean newest completion publishes.
#[test]
fn r4_publication_oracle_negative_controls() {
    let good: Result<(bool, Option<u32>), String> = Ok((true, Some(3)));
    assert_eq!(decide_publication(false, 5, 4, 5, &good, 3), "published");
    assert_eq!(
        decide_publication(false, 5, 4, 6, &good, 3),
        "refused-stale(newer-already-issued)"
    );
    assert_eq!(decide_publication(false, 5, 6, 6, &good, 3), "refused-out-of-order");
    assert_eq!(
        decide_publication(true, 5, 4, 5, &good, 3),
        "cancelled(superseded-late)"
    );
    let mismatch: Result<(bool, Option<u32>), String> = Ok((false, Some(3)));
    assert_eq!(
        decide_publication(false, 5, 4, 5, &mismatch, 3),
        "refused-oracle-mismatch",
        "an Ok(false) byte mismatch must NEVER publish (the R4-01 bug)"
    );
    let err: Result<(bool, Option<u32>), String> = Err("admission refused".into());
    assert_eq!(decide_publication(false, 5, 4, 5, &err, 3), "refused-judgment");
    let wrong_counter: Result<(bool, Option<u32>), String> = Ok((true, Some(2)));
    assert_eq!(
        decide_publication(false, 5, 4, 5, &wrong_counter, 3),
        "refused-revision-counter"
    );
    let no_counter: Result<(bool, Option<u32>), String> = Ok((true, None));
    assert_eq!(decide_publication(false, 5, 4, 5, &no_counter, 3), "refused-judgment");
}



/// R5 (F02): a bare failure sample — used by every exit path that fails
/// BEFORE a judgment exists (read error, early EOF, undecodable JSON).
fn failure_sample(scenario: &str, idx: usize, wall_ms: u128, input_sha: &str, error: String) -> Sample {
    Sample {
        scenario: scenario.into(),
        idx,
        wall_ms,
        queue_ms: None,
        exit: None,
        revision: None,
        parse: None,
        lower: None,
        link: None,
        object_hit: None,
        input_sha: input_sha.into(),
        ok: false,
        error: Some(error),
        stage_spans: vec![],
        gc: None,
    }
}

/// R5 (F02): read + parse ONE response line. ANY failure — read error,
/// early EOF, undecodable JSON — is first RECORDED as a failure sample
/// (with the raw tail for diagnosis), only then panics. No failure exits
/// before its sample is durable.
fn read_response_line(
    reader: &mut BufReader<std::process::ChildStdout>,
    rec: &mut Recorder,
    scenario: &str,
    idx: usize,
    wall_ms: u128,
    input_sha: &str,
) -> serde_json::Value {
    let mut resp = String::new();
    match reader.read_line(&mut resp) {
        Ok(0) => {
            rec.push(failure_sample(scenario, idx, wall_ms, input_sha,
                "early EOF before a response line".into()));
            panic!("{scenario} {idx}: worker closed stdout before answering (failure sample recorded)");
        }
        Ok(_) => {}
        Err(e) => {
            rec.push(failure_sample(scenario, idx, wall_ms, input_sha,
                format!("read response: {e}")));
            panic!("{scenario} {idx}: read response failed: {e} (failure sample recorded)");
        }
    }
    match serde_json::from_str(resp.trim()) {
        Ok(v) => v,
        Err(e) => {
            let tail: String = resp.chars().rev().take(120).collect::<Vec<_>>().into_iter().rev().collect();
            rec.push(failure_sample(scenario, idx, wall_ms, input_sha,
                format!("undecodable response ({e}); raw tail {tail:?}")));
            panic!("{scenario} {idx}: undecodable response: {e} (failure sample recorded)");
        }
    }
}

// ---------------------------------------------------------------------------
// R4-05: failure-sample durability counter-control
// ---------------------------------------------------------------------------

#[test]
fn r4_failure_sample_durability_control() {
    // A response that fails the runner's REAL admission (wrong schema)
    // flows through judged_fields into a Recorder BEFORE any gate — the
    // failing sample must already be on disk, marked not-ok, with the
    // refusal reason verbatim.
    let root = temp_root("r4-durability");
    let mut rec = Recorder::new("durability");
    let bad = serde_json::json!({
        "schema": "acworker/999", "nonce": "n", "request_hash": "h",
        "ok": { "diagnostics": [], "artifacts": [], "trace": [],
                "toolchain": {}, "metric_gaps": [] },
    });
    let judged = judge_ac_run(&bad, "n", "h", &root, &root, 3);
    let (ok, spans, error) = judged_fields(&judged);
    assert!(error.is_some(), "the wrong-schema response must fail admission");
    assert!(!ok);
    rec.push(Sample {
        scenario: "durability-control".into(),
        idx: 0,
        wall_ms: 0,
        queue_ms: None,
        exit: bad["ok"]["run"]["exit_code"].as_i64(),
        revision: None,
        parse: None,
        lower: None,
        link: None,
        object_hit: None,
        input_sha: sha_hex(b"durability-control"),
        ok,
        error: error.clone(),
        stage_spans: spans,
        gc: None,
    });
    let text = std::fs::read_to_string(&rec.path).unwrap();
    let last = text.lines().rev().find(|l| !l.trim().is_empty()).unwrap();
    let v: serde_json::Value = serde_json::from_str(last).unwrap();
    assert_eq!(v["scenario"], "durability-control");
    assert_eq!(v["ok"], false, "the failure sample is recorded as not-ok");
    assert!(
        v["error"].as_str().unwrap_or_default().contains("acworker/1"),
        "the refusal reason is verbatim in the durable record: {v}"
    );
    assert!(v["error"].as_str().is_some());
}


// ---------------------------------------------------------------------------
// R5: fault-injection relay — the REAL worker behind a mutating middleman.
// The relay forwards the session verbatim and rewrites each RESPONSE line
// per P788R5_MUTATION ("hash" rewrites the request_hash value to a forged
// one; "badjson" emits an undecodable line). These are the review's actual
// entry-point controls, made permanent: identity must come from the
// REQUEST side, and read/parse failures must leave durable samples.
// ---------------------------------------------------------------------------

fn relay_binary() -> PathBuf {
    static CACHE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| {
        let root = temp_root("r5-relay");
        let src = root.join("relay.rs");
        std::fs::write(
            &src,
            r#"
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
fn main() {
    let worker = std::env::var("P788R5_WORKER").expect("P788R5_WORKER");
    let mode = std::env::var("P788R5_MUTATION").unwrap_or_default();
    let mut child = Command::new(&worker)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
        .spawn().expect("spawn real worker");
    let mut cin = child.stdin.take().unwrap();
    let mut cout = BufReader::new(child.stdout.take().unwrap());
    // stdin forwarding runs on its OWN thread — responses must flow while
    // the parent has not sent `end` (a synchronous forward would deadlock).
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            if writeln!(cin, "{}", line.unwrap()).is_err() {
                break;
            }
            let _ = cin.flush();
        }
    });
    let mut out = std::io::stdout();
    let mut forwarded: u32 = 0;
    let count: usize = std::env::var("P788R5_COUNT")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    loop {
        let mut resp = String::new();
        match cout.read_line(&mut resp) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        if mode == "utf8-after" {
            // forward the first P788R5_COUNT responses verbatim, then emit
            // a raw invalid-UTF-8 byte + newline (the read error control)
            forwarded += 1;
            if forwarded as usize > count {
                let mut so = std::io::stdout();
                so.write_all(&[0xffu8, b'\n']).unwrap();
                so.flush().unwrap();
                continue;
            }
            writeln!(out, "{}", resp.trim_end()).unwrap();
        } else if mode == "hash" {
            let key = "\"request_hash\":\"";
            if let Some(a) = resp.find(key) {
                let start = a + key.len();
                if let Some(rel) = resp[start..].find('"') {
                    let end = start + rel;
                    let mutated = format!("{}{}{}", &resp[..start], "forged-r5", &resp[end..]);
                    writeln!(out, "{}", mutated.trim_end()).unwrap();
                    out.flush().unwrap();
                    continue;
                }
            }
            writeln!(out, "{}", resp.trim_end()).unwrap();
        } else if mode == "badjson" {
            writeln!(out, "{{not-json").unwrap();
            out.flush().unwrap();
        } else {
            writeln!(out, "{}", resp.trim_end()).unwrap();
        }
        out.flush().unwrap();
    }
    let _ = child.wait();
}
"#,
        )
        .unwrap();
        let exe = root.join("relay.exe");
        let out = Command::new("rustc")
            .arg(&src)
            .arg("-o")
            .arg(&exe)
            .output()
            .expect("rustc for relay");
        assert!(
            out.status.success(),
            "relay rustc: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        exe
    })
    .clone()
}

/// R5-01: the relay rewrites every response's request_hash AFTER the real
/// worker produced it. Judged against the REQUEST-side identity the forged
/// response is REFUSED and the failure sample is durable — this is the
/// review's actual-entry control, permanent.
#[test]
fn r5_wrong_hash_refused_and_recorded() {
    let root = temp_root("r5-hash");
    let real = worker_binary().expect("worker binary");
    let relay = relay_binary();
    let mut rec = Recorder::new("r5-hash");
    let (mut s, work) = Sess::spawn_exe(
        &relay,
        "r5hash",
        &[
            ("P788R5_WORKER", real.display().to_string()),
            ("P788R5_MUTATION", "hash".to_string()),
        ],
    );
    let base = "fn main() int { return 3 }";
    let anchor = base.find('3').unwrap();
    let steps = serde_json::json!([{ "start": anchor, "end": anchor + 1, "insert_b64": b64("4".as_bytes()) }]);
    let (n, h) = s.next_ids();
    let mut req = syntax_edit_request(base, steps, &work);
    patch_ids(&mut req, &n, &h);
    let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
    writeln!(s.stdin, "{}", line).unwrap();
    s.stdin.flush().unwrap();
    let t = Instant::now();
    let v = read_response_line(&mut s.reader, &mut rec, "r5-hash", 0, t.elapsed().as_millis(), &sha_hex(base.as_bytes()));
    // the control really mutated the hash
    assert_ne!(
        v["request_hash"].as_str().unwrap_or(""),
        h,
        "the relay must have forged the hash (control validity)"
    );
    assert_eq!(v["nonce"].as_str().unwrap_or(""), n, "nonce correlation intact");
    // judged against the REQUEST-side identity → refused
    let want = base.replace('3', "4");
    let judged = judge_syntax_edit(&v, &n, &h, &work, &root, &want);
    let err = judged.err().expect("a forged request_hash must be refused");
    assert!(
        err.contains("identity mismatch"),
        "refusal names the identity: {err}"
    );
    // the failure sample is durable BEFORE any gate
    rec.push(failure_sample("r5-hash", 0, t.elapsed().as_millis(), &sha_hex(base.as_bytes()), err.clone()));
    let text = std::fs::read_to_string(&rec.path).unwrap();
    assert!(text.contains("identity mismatch"), "durable: {text}");
    s.close();
}

/// R5-02: the relay emits an undecodable response line. The read/parse
/// path records the failure sample and only THEN fails — catch_unwind
/// proves the ordering (the sample is on disk despite the panic).
#[test]
fn r5_bad_json_failure_recorded() {
    let _root = temp_root("r5-json");
    let real = worker_binary().expect("worker binary");
    let relay = relay_binary();
    let mut rec = Recorder::new("r5-json");
    let raw_path = rec.path.clone();
    let (mut s, work) = Sess::spawn_exe(
        &relay,
        "r5json",
        &[
            ("P788R5_WORKER", real.display().to_string()),
            ("P788R5_MUTATION", "badjson".to_string()),
        ],
    );
    let base = "fn main() int { return 3 }";
    let anchor = base.find('3').unwrap();
    let steps = serde_json::json!([{ "start": anchor, "end": anchor + 1, "insert_b64": b64("4".as_bytes()) }]);
    let (n, h) = s.next_ids();
    let mut req = syntax_edit_request(base, steps, &work);
    patch_ids(&mut req, &n, &h);
    let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
    writeln!(s.stdin, "{}", line).unwrap();
    s.stdin.flush().unwrap();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        read_response_line(&mut s.reader, &mut rec, "r5-json", 0, 0, &sha_hex(base.as_bytes()));
    }));
    assert!(
        result.is_err(),
        "an undecodable response must fail the sample path (after recording)"
    );
    let text = std::fs::read_to_string(&raw_path).unwrap();
    assert!(
        text.contains("undecodable response") && text.contains("not-json"),
        "the failure sample (with the raw tail) is durable despite the panic: {text}"
    );
}


/// R6-01: the relay forwards warmup responses verbatim, then emits a raw
/// invalid-UTF-8 byte for the first real sample. The syntax gate's ACTUAL
/// read path must record the failure sample before failing — this is the
/// review's control, permanent, through the same read entry the gate uses.
#[test]
fn r6_invalid_utf8_read_recorded() {
    let _root = temp_root("r6-utf8");
    let real = worker_binary().expect("worker binary");
    let relay = relay_binary();
    let mut rec = Recorder::new("r6-utf8");
    let raw_path = rec.path.clone();
    let (mut s, work) = Sess::spawn_exe(
        &relay,
        "r6utf8",
        &[
            ("P788R5_WORKER", real.display().to_string()),
            ("P788R5_MUTATION", "utf8-after".to_string()),
            ("P788R5_COUNT", "1".to_string()),
        ],
    );
    let base = "fn main() int { return 3 }";
    let pos3 = base.find('3').unwrap();
    let _ = work;
    // one clean round trip (the "warmup" the relay forwards verbatim)
    let (n, h) = {
        let (n, h) = s.next_ids();
        let steps = serde_json::json!([{ "start": pos3, "end": pos3 + 1, "insert_b64": b64("4".as_bytes()) }]);
        let mut req = syntax_edit_request(base, steps, &work);
        patch_ids(&mut req, &n, &h);
        let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
        writeln!(s.stdin, "{}", line).unwrap();
        s.stdin.flush().unwrap();
        (n, h)
    };
    let _ = read_response_line(&mut s.reader, &mut rec, "r6-utf8-warmup", 0, 0, &sha_hex(base.as_bytes()));
    let _ = (n, h);
    // a SECOND request must be in flight so the worker actually answers —
    // the relay then rewrites THAT response into the invalid-UTF-8 byte
    // (without this the relay blocks on the worker and only the 180s
    // session watchdog would end the pipe).
    {
        let (n2, h2) = s.next_ids();
        let steps = serde_json::json!([{ "start": pos3, "end": pos3 + 1, "insert_b64": b64("5".as_bytes()) }]);
        let mut req = syntax_edit_request(base, steps, &work);
        patch_ids(&mut req, &n2, &h2);
        let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
        writeln!(s.stdin, "{}", line).unwrap();
        s.stdin.flush().unwrap();
    }
    // the next read hits the injected 0xff — recorded, THEN failed
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        read_response_line(&mut s.reader, &mut rec, "syntax-edit", 0, 0, &sha_hex(base.as_bytes()));
    }));
    assert!(
        result.is_err(),
        "an invalid-UTF-8 read error must fail the sample path (after recording)"
    );
    let text = std::fs::read_to_string(&raw_path).unwrap();
    assert!(
        text.contains("read response:") && text.contains("valid UTF-8"),
        "the read-failure sample is durable despite the panic: {text}"
    );
}

// ---------------------------------------------------------------------------
// F02: turnaround — six native scenarios, walls through FULL judgment
// ---------------------------------------------------------------------------

const SAMPLES: usize = 30;
const WARMUP: usize = 3;
const P95_GATE_MS: u128 = 1000;

#[test]
fn turnaround_native_six_scenarios_p95_gates() {
    let root = temp_root("turn-native");
    let mut rec = Recorder::new("native");
    let art = root.join("art");
    std::fs::create_dir_all(&art).unwrap();

    // 1) fresh — a NEW store root per sample, full pipeline every time.
    let mut fresh = Vec::new();
    for i in 0..SAMPLES {
        let store = root.join(format!("fresh-{i}"));
        std::fs::create_dir_all(&store).unwrap();
        let src = format!("// fresh {i}\n{P04}");
        let t0 = Instant::now();
        let (mut s, work) = Sess::spawn("fresh");
        let mut req = ac_run_request(&src, Some(store.to_str().unwrap()), "fresh", "fresh-proj", &work, 70000);
        let (n, h) = s.next_ids();
        patch_ids(&mut req, &n, &h);
        let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
        writeln!(s.stdin, "{}", line).unwrap();
        s.stdin.flush().unwrap();
        let v = read_response_line(&mut s.reader, &mut rec, "fresh", i, t0.elapsed().as_millis(), &sha_hex(src.as_bytes()));
        let judged = judge_ac_run(&v, &n, &h, &work, &art, 3);
        s.close();
        let wall = t0.elapsed().as_millis();
        let (ok, spans, error) = judged_fields(&judged);
        rec.push(Sample {
            scenario: "fresh".into(),
            idx: i,
            wall_ms: wall,
            queue_ms: None,
            exit: v["ok"]["run"]["exit_code"].as_i64(),
            revision: None,
            parse: counter(&v, "parse"),
            lower: counter(&v, "lower"),
            link: counter(&v, "link"),
            object_hit: object_hit(&v),
            input_sha: sha_hex(src.as_bytes()),
            ok,
            error: error.clone(),
            stage_spans: spans,
            gc: None,
        });
        assert!(ok, "fresh {i} failed: {error:?}");
        fresh.push(wall);
    }

    // 2) same-process hot — ONE session, warmup discarded, session prep
    //    timed separately (§8: same-process proof stays session-level).
    let hot_store = root.join("hot");
    std::fs::create_dir_all(&hot_store).unwrap();
    let hot_p95;
    let prep_ms;
    {
        let t_prep = Instant::now();
        let (mut s, work) = Sess::spawn("hot");
        prep_ms = t_prep.elapsed().as_millis();
        for _ in 0..WARMUP {
            let (n, h) = s.next_ids();
            let mut req = ac_run_request(P04, Some(hot_store.to_str().unwrap()), "reuse", "hot", &work, 70000);
            patch_ids(&mut req, &n, &h);
            let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
            writeln!(s.stdin, "{}", line).unwrap();
            s.stdin.flush().unwrap();
            let mut resp = String::new();
            s.reader.read_line(&mut resp).unwrap();
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(resp.trim()).unwrap()["ok"]["run"]["exit_code"],
                3
            );
        }
        let mut samples = Vec::new();
        for i in 0..SAMPLES {
            let t = Instant::now();
            let (n, h) = s.next_ids();
            let mut req = ac_run_request(P04, Some(hot_store.to_str().unwrap()), "reuse", "hot", &work, 70000);
            patch_ids(&mut req, &n, &h);
            let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
            writeln!(s.stdin, "{}", line).unwrap();
            s.stdin.flush().unwrap();
            let v = read_response_line(&mut s.reader, &mut rec, "hot", i, t.elapsed().as_millis(), &sha_hex(P04.as_bytes()));
            let (ok, spans, error) = judged_fields(&judge_ac_run(&v, &n, &h, &work, &art, 3));
            let wall = t.elapsed().as_millis();
            rec.push(Sample {
                scenario: "hot".into(),
                idx: i,
                wall_ms: wall,
                queue_ms: None,
                exit: v["ok"]["run"]["exit_code"].as_i64(),
                revision: None,
                parse: counter(&v, "parse"),
                lower: counter(&v, "lower"),
                link: counter(&v, "link"),
                object_hit: object_hit(&v),
                input_sha: sha_hex(P04.as_bytes()),
                ok,
                error: error.clone(),
                stage_spans: spans,
                gc: None,
            });
            assert!(ok, "hot {i}: {error:?}");
            samples.push(wall);
        }
        s.close();
        hot_p95 = p95(&samples);
        let (p50, _, p99) = stats(&samples);
        println!("hot: n={SAMPLES} p50={p50}ms p95={hot_p95}ms p99={p99}ms (session prep {prep_ms}ms, {WARMUP} warmup discarded; wall = build→IPC→parse→admission→manifest-verify→persist→exit-oracle)");
    }

    // 3) restart disk-hot — a NEW process per sample, SAME store.
    let restart_store = root.join("restart");
    std::fs::create_dir_all(&restart_store).unwrap();
    let mut restart = Vec::new();
    for i in 0..SAMPLES {
        let t0 = Instant::now();
        let (mut s, work) = Sess::spawn("restart");
        let (n, h) = s.next_ids();
        let mut req = ac_run_request(P04, Some(restart_store.to_str().unwrap()), "reuse", "restart", &work, 70000);
        patch_ids(&mut req, &n, &h);
        let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
        writeln!(s.stdin, "{}", line).unwrap();
        s.stdin.flush().unwrap();
        let v = read_response_line(&mut s.reader, &mut rec, "restart-disk-hot", i, t0.elapsed().as_millis(), &sha_hex(P04.as_bytes()));
        let (ok, spans, error) = judged_fields(&judge_ac_run(&v, &n, &h, &work, &art, 3));
        s.close();
        let wall = t0.elapsed().as_millis();
        rec.push(Sample {
            scenario: "restart-disk-hot".into(),
            idx: i,
            wall_ms: wall,
            queue_ms: None,
            exit: v["ok"]["run"]["exit_code"].as_i64(),
            revision: None,
            parse: counter(&v, "parse"),
            lower: counter(&v, "lower"),
            link: counter(&v, "link"),
            object_hit: object_hit(&v),
            input_sha: sha_hex(P04.as_bytes()),
            ok,
            error: error.clone(),
            stage_spans: spans,
            gc: None,
        });
        assert!(ok, "restart {i}: {error:?}");
        restart.push(wall);
    }

    // 4) cross-project — the SAME content under alternating projects.
    let xstore = root.join("cross");
    std::fs::create_dir_all(&xstore).unwrap();
    let mut cross = Vec::new();
    for i in 0..SAMPLES {
        let project = if i % 2 == 0 { "proj-a" } else { "proj-b" };
        let t0 = Instant::now();
        let (mut s, work) = Sess::spawn("cross");
        let (n, h) = s.next_ids();
        let mut req = ac_run_request(P04, Some(xstore.to_str().unwrap()), "reuse", project, &work, 70000);
        patch_ids(&mut req, &n, &h);
        let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
        writeln!(s.stdin, "{}", line).unwrap();
        s.stdin.flush().unwrap();
        let v = read_response_line(&mut s.reader, &mut rec, "cross-project", i, t0.elapsed().as_millis(), &sha_hex(P04.as_bytes()));
        let (ok, spans, error) = judged_fields(&judge_ac_run(&v, &n, &h, &work, &art, 3));
        s.close();
        let wall = t0.elapsed().as_millis();
        rec.push(Sample {
            scenario: "cross-project".into(),
            idx: i,
            wall_ms: wall,
            queue_ms: None,
            exit: v["ok"]["run"]["exit_code"].as_i64(),
            revision: None,
            parse: counter(&v, "parse"),
            lower: counter(&v, "lower"),
            link: counter(&v, "link"),
            object_hit: object_hit(&v),
            input_sha: sha_hex(P04.as_bytes()),
            ok,
            error: error.clone(),
            stage_spans: spans,
            gc: None,
        });
        assert!(ok, "cross {i}: {error:?}");
        cross.push(wall);
    }

    // 5) body-edit — the §5.5 recipe: loop bound 3↔4 REALLY alternates
    //    (exit 3/4 oracle) + unique marker per sample; every sample a real
    //    recompute (parse==1, object=lowered asserted).
    let edit_store = root.join("edit");
    std::fs::create_dir_all(&edit_store).unwrap();
    let edit_p95;
    {
        let (mut s, work) = Sess::spawn("edit");
        let mut samples = Vec::new();
        for i in 0..SAMPLES {
            let (src, want) = p04_bound_edit(i);
            let t = Instant::now();
            let (n, h) = s.next_ids();
            let mut req = ac_run_request(&src, Some(edit_store.to_str().unwrap()), "reuse", "edit", &work, 70000);
            patch_ids(&mut req, &n, &h);
            let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
            writeln!(s.stdin, "{}", line).unwrap();
            s.stdin.flush().unwrap();
            let v = read_response_line(&mut s.reader, &mut rec, "body-edit", i, t.elapsed().as_millis(), &sha_hex(src.as_bytes()));
            let (ok, spans, error) = judged_fields(&judge_ac_run(&v, &n, &h, &work, &art, want));
            let wall = t.elapsed().as_millis();
            let parse = counter(&v, "parse");
            let hit = object_hit(&v);
            rec.push(Sample {
                scenario: "body-edit".into(),
                idx: i,
                wall_ms: wall,
                queue_ms: None,
                exit: v["ok"]["run"]["exit_code"].as_i64(),
                revision: None,
                parse,
                lower: counter(&v, "lower"),
                link: counter(&v, "link"),
                object_hit: hit.clone(),
                input_sha: sha_hex(src.as_bytes()),
                ok,
                error: error.clone(),
                stage_spans: spans,
                gc: None,
            });
            assert!(ok, "body-edit {i}: {error:?}");
            assert_eq!(parse, Some(1), "body-edit {i} must re-parse (real recompute)");
            assert_eq!(hit.as_deref(), Some("lowered"), "body-edit {i} must not resume a cached object");
            samples.push(wall);
        }
        s.close();
        edit_p95 = p95(&samples);
        let (p50, _, p99) = stats(&samples);
        println!("body-edit: n={SAMPLES} p50={p50}ms p95={edit_p95}ms p99={p99}ms (bound 3<->4 alternating + unique marker; every sample parse=1, object=lowered)");
    }

    // 6) gc-concurrent — the REAL 783 GC as a concurrent PROCESS against
    //    the live store, barrier-proven overlap on EVERY sample.
    let gc_store = root.join("gc");
    std::fs::create_dir_all(&gc_store).unwrap();
    {
        let (mut s, work) = Sess::spawn("gc");
        // populate the store once so GC round 0 has real content to scan
        {
            let (n, h) = s.next_ids();
            let mut req = ac_run_request(P04, Some(gc_store.to_str().unwrap()), "reuse", "gc", &work, 70000);
            patch_ids(&mut req, &n, &h);
            let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
            writeln!(s.stdin, "{}", line).unwrap();
            s.stdin.flush().unwrap();
            let mut resp = String::new();
            s.reader.read_line(&mut resp).unwrap();
        }
        let mut samples = Vec::new();
        let mut overlaps = 0usize;
        for i in 0..SAMPLES {
            let fin_rx = spawn_gc_round(&gc_store);
            // Barrier: the GC has STARTED (opened the store) but must NOT
            // have finished before the request begins — checked with a
            // zero-wait probe right before the request.
            assert!(
                fin_rx.try_recv().is_err(),
                "gc {i}: GC finished before the request started — overlap barrier broken"
            );
            let t = Instant::now();
            let (n, h) = s.next_ids();
            let mut req = ac_run_request(P04, Some(gc_store.to_str().unwrap()), "reuse", "gc", &work, 70000);
            patch_ids(&mut req, &n, &h);
            let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
            writeln!(s.stdin, "{}", line).unwrap();
            s.stdin.flush().unwrap();
            let v = read_response_line(&mut s.reader, &mut rec, "gc-concurrent", i, t.elapsed().as_millis(), &sha_hex(P04.as_bytes()));
            let (ok, spans, error) = judged_fields(&judge_ac_run(&v, &n, &h, &work, &art, 3));
            let req_end = Instant::now();
            let wall = t.elapsed().as_millis();
            let (summary, fin_at) = fin_rx
                .recv_timeout(Duration::from_secs(60))
                .expect("gc round completes");
            assert!(summary.starts_with("store-gc-ok"), "gc receipt: {summary}");
            let finished_after_req_start = fin_at > t;
            let overlap_us = if finished_after_req_start {
                fin_at.min(req_end).duration_since(t).as_micros()
            } else {
                0
            };
            if finished_after_req_start && overlap_us > 0 {
                overlaps += 1;
            }
            rec.push(Sample {
                scenario: "gc-concurrent".into(),
                idx: i,
                wall_ms: wall,
                queue_ms: None,
                exit: v["ok"]["run"]["exit_code"].as_i64(),
                revision: None,
                parse: counter(&v, "parse"),
                lower: counter(&v, "lower"),
                link: counter(&v, "link"),
                object_hit: object_hit(&v),
                input_sha: sha_hex(P04.as_bytes()),
                ok,
                error: error.clone(),
                stage_spans: spans,
                gc: Some(GcInterval {
                    started_before_req: true,
                    finished_after_req_start,
                    overlap_us,
                    summary,
                }),
            });
            assert!(ok, "gc {i}: {error:?}");
            samples.push(wall);
        }
        s.close();
        assert_eq!(
            overlaps, SAMPLES,
            "every GC sample must overlap its request (barrier-proven; got {overlaps}/{SAMPLES})"
        );
        let (p50, p95v, p99) = stats(&samples);
        println!("gc-concurrent: n={SAMPLES} p50={p50}ms p95={p95v}ms p99={p99}ms (real GC process per sample, {overlaps} barrier-proven overlaps)");
    }

    let fs_ = stats(&fresh);
    let rs = stats(&restart);
    let cs = stats(&cross);
    println!(
        "fresh: p50={}ms p95={}ms p99={}ms | restart-disk-hot: p50={}ms p95={}ms p99={}ms | cross: p50={}ms p95={}ms p99={}ms",
        fs_.0, fs_.1, fs_.2, rs.0, rs.1, rs.2, cs.0, cs.1, cs.2
    );
    println!("raw samples (JSONL, per-sample before gates): {}", rec.path.display());
    println!("worker exe sha256: {}", rec.worker_sha);

    assert!(hot_p95 < P95_GATE_MS, "HARD GATE hot P95 {hot_p95}ms >= {P95_GATE_MS}ms");
    assert!(edit_p95 < P95_GATE_MS, "HARD GATE body-edit P95 {edit_p95}ms >= {P95_GATE_MS}ms");
}

// ---------------------------------------------------------------------------
// F02/F01: syntax gate + the concurrent revision stream
// ---------------------------------------------------------------------------

fn syntax_edit_request(
    base: &str,
    steps: serde_json::Value,
    work: &Path,
) -> serde_json::Value {
    serde_json::json!({
        "schema": "acworker/1",
        "kind": "edit",
        "nonce": "p",
        "request_hash": "p",
        "input": { "kind": "source", "source_b64": b64(base.as_bytes()) },
        "profile": "core",
        "entry": "main",
        "budget": { "build_ms": 60000, "run_ms": 5000, "total_ms": 70000 },
        "scratch": work.display().to_string(),
        "store_root": null,
        "cache": "reuse",
        "project": "syntax",
        "include_hir": false,
        "capabilities": [],
        "support_lib": null,
        "grammar": "auto-core",
        "edits": steps,
    })
}

/// Judge one syntax edit response through the runner's entries: admission
/// → projection admission/persistence → reconstruction byte oracle.
fn judge_syntax_edit(
    v: &serde_json::Value,
    nonce: &str,
    hash: &str,
    work: &Path,
    art_dir: &Path,
    want_text: &str,
) -> Result<(bool, Option<u32>), String> {
    validate_response(v, nonce, hash, "edit")?;
    if v.get("error").map(|e| !e.is_null()).unwrap_or(false) {
        return Err(format!(
            "worker error [{}]: {}",
            v["error"]["code"].as_str().unwrap_or("?"),
            v["error"]["message"].as_str().unwrap_or("")
        ));
    }
    let (arts, _) = admit_and_persist_artifacts(&v["ok"], Adapter::Syntax, work, art_dir)?;
    let recon = arts
        .iter()
        .find(|a| a.logical_path == "reconstructed.at")
        .map(|a| std::fs::read(&a.stored_path).unwrap_or_default())
        .unwrap_or_default();
    let revision = v["ok"]["cache"]["counters"]
        .as_array()
        .and_then(|a| a.iter().find(|c| c[0] == "revision").map(|c| c[1].as_u64().unwrap() as u32));
    Ok((recon == want_text.as_bytes(), revision))
}

#[derive(serde::Serialize, Clone)]
struct RevRecord {
    revision: u32,
    lane: &'static str,
    enqueued_at_ms: u128,
    written_at_ms: Option<u128>,
    completed_at_ms: Option<u128>,
    steps: u32,
    out_of_order: Option<bool>,
    publish: &'static str, // published | refused-out-of-order | cancelled(superseded-late) | refused-judgment
    recon_ok: Option<bool>,
    counter_revision: Option<u32>,
    /// The judgment error verbatim when admission/oracle refused (R3: the
    /// durable record must name the reason, not just the failure fact).
    error: Option<String>,
}

/// Shared latest-wins publisher state (arbitrated under one lock).
struct Pub {
    last_published: u32,
    in_flight: std::collections::BTreeSet<u32>,
    superseded: std::collections::BTreeSet<u32>,
    records: Vec<RevRecord>,
    completion_order: Vec<u32>,
    /// R5 (F02): read/parse failures on a lane — every one is durable in
    /// the raw JSONL before it counts here, and any nonzero count fails
    /// the stream invariants.
    stream_errors: usize,
}

#[test]
fn turnaround_syntax_gate_and_revision_stream() {
    let root = temp_root("turn-syntax");
    let art = root.join("art");
    std::fs::create_dir_all(&art).unwrap();
    let mut rec = Recorder::new("syntax");

    // ---- syntax local-edit gate: 30 single-edit requests, wall through
    //      admission + projection persistence + byte oracle.
    let syntax_p95;
    {
        let (mut s, work) = Sess::spawn("syn");
        let base = "fn helper() int { return 7 }\nfn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
        let pos3 = base.find('3').unwrap();
        // R6-01: round_trip only WRITES the request and returns the
        // REQUEST-side identity — the response is read exclusively through
        // read_response_line, so read/EOF/UTF-8/parse failures are always
        // recorded before they can fail the run.
        let round_trip = |s: &mut Sess, digit: &str| -> (String, String) {
            let (n, h) = s.next_ids();
            let steps = serde_json::json!([{ "start": pos3, "end": pos3 + 1, "insert_b64": b64(digit.as_bytes()) }]);
            let mut req = syntax_edit_request(base, steps, &work);
            patch_ids(&mut req, &n, &h);
            let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
            writeln!(s.stdin, "{}", line).unwrap();
            s.stdin.flush().unwrap();
            (n, h)
        };
        for k in 0..WARMUP {
            let digit = ((k % 9) + 1).to_string();
            let (n, h) = round_trip(&mut s, &digit);
            let _ = read_response_line(&mut s.reader, &mut rec, "syntax-warmup", k, 0, &sha_hex(base.as_bytes()));
            let _ = (n, h);
        }
        let mut samples = Vec::new();
        for k in 0..SAMPLES {
            let digit = ((k % 9) + 1).to_string();
            let want = base.replace('3', &digit);
            let t = Instant::now();
            let (req_nonce, req_hash) = round_trip(&mut s, &digit);
            let v = read_response_line(&mut s.reader, &mut rec, "syntax-edit", k, t.elapsed().as_millis(), &sha_hex(base.as_bytes()));
            // R5 (F01): the EXPECTED identity is the REQUEST-side one —
            // reading nonce/hash back from the response compared the
            // response with itself and let forged hashes through.
            // R4 (F05): the failure sample is durable BEFORE the assert.
            let judged = judge_syntax_edit(&v, &req_nonce, &req_hash, &work, &art, &want);
            let (ok, revision, syn_error) = match &judged {
                Ok((ok, rev)) => (*ok, *rev, None),
                Err(e) => (false, None, Some(e.clone())),
            };
            let wall = t.elapsed().as_millis();
            rec.push(Sample {
                scenario: "syntax-edit".into(),
                idx: k,
                wall_ms: wall,
                queue_ms: None,
                exit: None,
                revision,
                parse: None,
                lower: None,
                link: None,
                object_hit: None,
                input_sha: sha_hex(base.as_bytes()),
                ok,
                error: syn_error.clone(),
                stage_spans: vec![],
                gc: None,
            });
            assert!(ok, "syntax sample {k}: reconstruction oracle: {syn_error:?}");
            samples.push(wall);
        }
        s.close();
        syntax_p95 = p95(&samples);
        let (p50, _, p99) = stats(&samples);
        println!("syntax-edit: n={SAMPLES} p50={p50}ms p95={syntax_p95}ms p99={p99}ms (wall through admission+persist+byte oracle)");
    }

    // ---- V-13: the concurrent revision stream (F01).
    //
    // ONE shared document streamed to TWO real worker lanes on a fixed
    // 100ms arrival clock (the driver NEVER waits for a response to issue
    // the next request). Every 7th revision is a HEAVY edit (churn pairs
    // of 64KiB insert/delete around the digit run) on lane A; the rest are
    // light on lane B. Heavy old + light new on different lanes produce
    // REAL out-of-order completions; the publisher (latest-wins) refuses
    // any completion that is not the newest, marks superseded revisions
    // cancelled, and only ever publishes a strictly increasing sequence.
    const REQS: u32 = 50;
    const ARRIVAL_MS: u64 = 100;
    const HEAVY_EVERY: u32 = 7;
    const CHURN_PAIRS: usize = 31; // 1 digit-run step + 2*31 churn steps = 63 ≤ 64
    const CHURN_BYTES: usize = 64 * 1024;
    {
        let base = "fn main() int { return  }";
        let anchor = base.find("return ").unwrap() + 7;

        // per-lane raw worker handles (Sess is single-reader; lanes need
        // dedicated writer AND reader threads).
        let exe = worker_binary().expect("worker binary");
        let spawn_lane = |tag: &str| -> (
            std::process::ChildStdin,
            BufReader<std::process::ChildStdout>,
            std::process::Child,
            PathBuf,
        ) {
            let work = std::env::temp_dir().join(format!(
                "p788r3-rev-{}-{}-{}",
                tag,
                std::process::id(),
                DIR_SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            ));
            std::fs::create_dir_all(&work).unwrap();
            let mut child = Command::new(&exe)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let reader = BufReader::new(child.stdout.take().unwrap());
            let mut stdin = child.stdin.take().unwrap();
            writeln!(stdin, r#"{{"op":"begin","schema":"acworker/1"}}"#).unwrap();
            stdin.flush().unwrap();
            (stdin, reader, child, work)
        };
        let (stdin_a, reader_a, mut child_a, work_a_root) = spawn_lane("a");
        let (stdin_b, reader_b, mut child_b, work_b_root) = spawn_lane("b");

        // ONE scratch PER REQUEST (the runner itself gives every request
        // its own scratch; reusing one dir would race the next request's
        // artifact writes against the previous manifest admission).
        let scratch_of: std::sync::Arc<
            std::sync::Mutex<std::collections::BTreeMap<u32, PathBuf>>,
        > = std::sync::Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new()));

        // shared bookkeeping
        let expected: std::sync::Arc<
            std::sync::Mutex<std::collections::BTreeMap<u32, (String, u32)>>,
        > = std::sync::Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new()));
        let written: std::sync::Arc<std::sync::Mutex<std::collections::BTreeMap<u32, u128>>> =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new()));
        let enqueue: std::sync::Arc<std::sync::Mutex<std::collections::BTreeMap<u32, u128>>> =
            std::sync::Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new()));
        let nonce_to_k: std::sync::Arc<
            std::sync::Mutex<std::collections::BTreeMap<String, u32>>,
        > = std::sync::Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new()));
        let pub_state = std::sync::Arc::new(std::sync::Mutex::new(Pub {
            last_published: 0,
            in_flight: std::collections::BTreeSet::new(),
            superseded: std::collections::BTreeSet::new(),
            records: Vec::new(),
            completion_order: Vec::new(),
            stream_errors: 0,
        }));
        // R5 (F01): the EXPECTED identity of each revision — minted on the
        // REQUEST side at arrival; readers correlate via the response
        // nonce but judge against THIS pair, never against the echo.
        let identity_of: std::sync::Arc<
            std::sync::Mutex<std::collections::BTreeMap<u32, (String, String)>>,
        > = std::sync::Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new()));
        // R4-01: the newest revision already HANDED to a lane — publication
        // must not fall behind the arrival front.
        let max_issued: std::sync::Arc<std::sync::atomic::AtomicU32> =
            std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));

        // ---- R5: every request is FULLY PRECOMPUTED (steps, oracle doc,
        // scratch, identity, serialized line) BEFORE the clock starts —
        // the arrival loop only sleeps to the tick and hands the line to
        // the lane writer, so neither heavy construction (~2.7MB lines)
        // nor serialization can push an arrival past its slot.
        let mut prebuilt: std::collections::BTreeMap<
            u32,
            (String, String, u32, bool, String),
        > = std::collections::BTreeMap::new(); // k -> (line, want_doc, steps, heavy, scratch)
        for k in 1..=REQS {
            let heavy = k % HEAVY_EVERY == 3;
            let digits: String = (1..=k)
                .map(|i| (i % 10).to_string())
                .collect::<Vec<_>>()
                .join("");
            let mut steps = vec![serde_json::json!({
                "start": anchor, "end": anchor, "insert_b64": b64(digits.as_bytes())
            })];
            if heavy {
                let churn: String = "x".repeat(CHURN_BYTES);
                let at = anchor + digits.len();
                for _ in 0..CHURN_PAIRS {
                    steps.push(serde_json::json!({
                        "start": at, "end": at, "insert_b64": b64(churn.as_bytes())
                    }));
                }
                for _ in 0..CHURN_PAIRS {
                    steps.push(serde_json::json!({
                        "start": at, "end": at + churn.len(), "insert_b64": b64("".as_bytes())
                    }));
                }
            }
            let want_doc = format!("{}{}{}", &base[..anchor], digits, &base[anchor..]);
            let work = if heavy { &work_a_root } else { &work_b_root }
                .join(format!("r{k}"));
            std::fs::create_dir_all(&work).unwrap();
            let nonce = format!("rev-{k}");
            let hash = format!("revh-{k}");
            let mut req = syntax_edit_request(base, serde_json::json!(steps), &work);
            patch_ids(&mut req, &nonce, &hash);
            let line = serde_json::to_string(&serde_json::json!({"op":"request","request":req})).unwrap();
            prebuilt.insert(k, (line, want_doc, steps.len() as u32, heavy, work.display().to_string()));
        }

        // the clock starts ONLY after all precompute/lane setup is done —
        // arrivals measure from a clean t0.
        let t0 = Instant::now();
        // writer threads: own the lane stdin; never block the arrival clock
        let (tx_a, rx_a) = std::sync::mpsc::channel::<(u32, String)>();
        let (tx_b, rx_b) = std::sync::mpsc::channel::<(u32, String)>();
        let w_a = written.clone();
        let hw = std::thread::spawn(move || lane_writer(stdin_a, rx_a, w_a, t0));
        let w_b = written.clone();
        let lw = std::thread::spawn(move || lane_writer(stdin_b, rx_b, w_b, t0));

        // reader threads: parse → runner admission → latest-wins arbitration
        let art_a = root.join("art-a");
        let art_b = root.join("art-b");
        std::fs::create_dir_all(&art_a).unwrap();
        std::fs::create_dir_all(&art_b).unwrap();
        let r_a = (
            expected.clone(), nonce_to_k.clone(), identity_of.clone(), enqueue.clone(),
            written.clone(), pub_state.clone(),
        );
        let work_a_for_reader = scratch_of.clone();
        let raw_for_a = rec.path.clone();
        let max_a = max_issued.clone();
        let hr_a = std::thread::spawn(move || {
            let (expected, nonce_to_k, identity_of, enqueue, written, pub_state) = r_a;
            lane_reader(
                reader_a, work_a_for_reader, art_a, "heavy", expected, nonce_to_k,
                identity_of, enqueue, written, pub_state, max_a, raw_for_a, t0,
            )
        });
        let r_b = (
            expected.clone(), nonce_to_k.clone(), identity_of.clone(), enqueue.clone(),
            written.clone(), pub_state.clone(),
        );
        let work_b_for_reader = scratch_of.clone();
        let raw_for_b = rec.path.clone();
        let max_b = max_issued.clone();
        let hr_b = std::thread::spawn(move || {
            let (expected, nonce_to_k, identity_of, enqueue, written, pub_state) = r_b;
            lane_reader(
                reader_b, work_b_for_reader, art_b, "light", expected, nonce_to_k,
                identity_of, enqueue, written, pub_state, max_b, raw_for_b, t0,
            )
        });

        // ---- the arrival loop: fixed 100ms clock, never waits on responses
        for k in 1..=REQS {
            let target = t0 + Duration::from_millis(k as u64 * ARRIVAL_MS);
            let now = Instant::now();
            if now < target {
                std::thread::sleep(target - now);
            }
            enqueue.lock().unwrap().insert(k, ms_since(t0));
            max_issued.fetch_max(k, std::sync::atomic::Ordering::SeqCst);
            let (line, want_doc, steps, heavy, scratch) = prebuilt.remove(&k).unwrap();
            expected.lock().unwrap().insert(k, (want_doc, steps));
            let work = PathBuf::from(scratch);
            scratch_of.lock().unwrap().insert(k, work);
            let nonce = format!("rev-{k}");
            let hash = format!("revh-{k}");
            nonce_to_k.lock().unwrap().insert(nonce.clone(), k);
            identity_of.lock().unwrap().insert(k, (nonce, hash));
            pub_state.lock().unwrap().in_flight.insert(k);
            if heavy {
                tx_a.send((k, line)).unwrap();
            } else {
                tx_b.send((k, line)).unwrap();
            }
        }
        drop(tx_a);
        drop(tx_b);
        let _ = hw.join();
        let _ = lw.join();
        let _ = hr_a.join();
        let _ = hr_b.join();
        let _ = child_a.wait();
        let _ = child_b.wait();

        // ---- stream invariants
        let st = pub_state.lock().unwrap();
        let records: std::collections::BTreeMap<u32, &RevRecord> =
            st.records.iter().map(|r| (r.revision, r)).collect();
        assert_eq!(records.len() as u32, REQS, "all {REQS} requests must leave a record (none deleted)");
        for k in 1..=REQS {
            assert!(records.contains_key(&k), "revision {k} missing from the stream records");
        }
        // published sequence strictly increasing by construction; the FINAL
        // published revision is the latest one.
        let published: Vec<u32> = st
            .records
            .iter()
            .filter(|r| r.publish == "published")
            .map(|r| r.revision)
            .collect();
        let mut sorted = published.clone();
        sorted.sort_unstable();
        assert_eq!(published, sorted, "publication order must be strictly increasing");
        assert_eq!(st.last_published, REQS, "the latest revision must be the last published");
        // real out-of-order completions happened AND were refused
        let refused: Vec<&RevRecord> = st
            .records
            .iter()
            .filter(|r| r.publish != "published")
            .collect();
        assert!(
            !refused.is_empty(),
            "the heavy/light lanes must produce at least one real out-of-order completion that the publisher REFUSED"
        );
        // R4-01: every refusal carries a LEGITIMATE category, each verified
        // against its own evidence — out-of-order/superseded completions
        // really had a newer completion first; a stale publication refusal
        // really had a newer revision ISSUED before it completed; oracle
        // mismatches and judgment failures never happen on a healthy
        // stream (asserted below via recon_ok/counter).
        let enq_of = |k: u32| enqueue.lock().unwrap()[&k];
        for r in &refused {
            match r.publish {
                "refused-out-of-order" | "cancelled(superseded-late)" => assert_eq!(
                    r.out_of_order,
                    Some(true),
                    "rev {}: {} must be a genuinely newer-first completion",
                    r.revision,
                    r.publish
                ),
                "refused-stale(newer-already-issued)" => {
                    let newer_issued_before_completion = (r.revision + 1..=REQS).any(|j| {
                        records.contains_key(&j) || enq_of(j) < r.completed_at_ms.unwrap_or(u128::MAX)
                    });
                    assert!(
                        newer_issued_before_completion,
                        "rev {}: stale refusal must rest on a newer revision issued before completion",
                        r.revision
                    );
                }
                other => panic!("unknown refusal category {other:?} on rev {}", r.revision),
            }
        }
        assert!(
            refused
                .iter()
                .all(|r| r.publish != "refused-oracle-mismatch" && r.publish != "refused-judgment"),
            "a healthy stream must not produce byte mismatches or admission failures (found: {:?})",
            refused.iter().map(|r| (r.revision, r.publish)).collect::<Vec<_>>()
        );
        // arrival clock discipline: enqueue times track k*100ms with no
        // cumulative drift (the driver never waited for responses).
        for k in 1..=REQS {
            let enq = enqueue.lock().unwrap()[&k];
            let want = k as u128 * ARRIVAL_MS as u128;
            assert!(
                enq.abs_diff(want) <= 60,
                "arrival {k}: {enq}ms vs clock {want}ms — the arrival clock must not wait for responses"
            );
        }
        // queueing is visible: at some point ≥2 ISSUED requests were still
        // uncompleted (only j ≤ k count — future arrivals don't exist yet).
        let mut max_depth = 0usize;
        for k in 1..=REQS {
            let enq = enqueue.lock().unwrap()[&k];
            let depth = st
                .records
                .iter()
                .filter(|r| r.revision <= k)
                .filter(|r| r.completed_at_ms.map(|c| c > enq).unwrap_or(true))
                .count();
            max_depth = max_depth.max(depth);
        }
        assert!(
            max_depth >= 2,
            "the stream must expose real queue backlog (max in-flight {max_depth})"
        );
        // R5 (F02): no read/parse failure on either lane, and every issued
        // revision received a completion record (nothing vanished).
        assert_eq!(
            st.stream_errors, 0,
            "lane read/parse failures are durable in the raw JSONL and fail the stream here"
        );
        assert!(
            st.in_flight.is_empty(),
            "revisions left without a response: {:?}",
            st.in_flight
        );
        // every completed response passed the runner admission AND the
        // byte oracle (the honest Ok(false) verdict travels — R4-01), and
        // the worker's revision counter equals the request's step count.
        assert!(
            st.records.iter().all(|r| r.recon_ok == Some(true)),
            "every completion must survive full admission + reconstruction oracle: {:?}",
            st.records
                .iter()
                .filter(|r| r.recon_ok != Some(true))
                .map(|r| (r.revision, r.recon_ok, r.error.clone()))
                .collect::<Vec<_>>()
        );
        assert!(
            st.records.iter().all(|r| r.counter_revision == Some(r.steps)),
            "the worker revision counter must equal the request step count: {:?}",
            st.records
                .iter()
                .map(|r| (r.revision, r.counter_revision, r.steps))
                .collect::<Vec<_>>()
        );
        let refused_count = refused.len();
        let superseded = st.superseded.len();
        let published_count = published.len();
        println!(
            "revision-stream: {REQS} requests @100ms arrival clock, two lanes; published {published_count}, refused out-of-order {refused_count}, superseded-cancelled {superseded}; max in-flight depth {max_depth}; final published = {}",
            st.last_published
        );
    }

    let raw_path = rec.path.clone();
    println!("raw samples (JSONL): {}", raw_path.display());
    assert!(
        syntax_p95 < P95_GATE_MS,
        "HARD GATE syntax edit P95 {syntax_p95}ms >= {P95_GATE_MS}ms"
    );
}

fn ms_since(t0: Instant) -> u128 {
    t0.elapsed().as_millis()
}

/// Lane writer: owns the lane's stdin; drains the request channel, so the
/// arrival clock NEVER blocks on a full pipe (backlog shows up as
/// written_at − enqueued_at).
fn lane_writer(
    mut stdin: std::process::ChildStdin,
    rx: std::sync::mpsc::Receiver<(u32, String)>,
    written: std::sync::Arc<
        std::sync::Mutex<std::collections::BTreeMap<u32, u128>>,
    >,
    t0: Instant,
) {
    while let Ok((k, line)) = rx.recv() {
        if writeln!(stdin, "{}", line).is_err() {
            break;
        }
        let _ = stdin.flush();
        written.lock().unwrap().insert(k, ms_since(t0));
    }
    let _ = writeln!(stdin, r#"{{"op":"end"}}"#);
    let _ = stdin.flush();
}

/// Lane reader: parses each response, runs it through the RUNNER's
/// admission/judgment entries (against the request's OWN scratch), then
/// arbitrates latest-wins publication. Every record is appended to the raw
/// JSONL THE MOMENT it exists — a later panic can never hide evidence.
#[allow(clippy::too_many_arguments)]
fn lane_reader(
    mut reader: BufReader<std::process::ChildStdout>,
    scratch_of: std::sync::Arc<
        std::sync::Mutex<std::collections::BTreeMap<u32, PathBuf>>,
    >,
    art: PathBuf,
    lane: &'static str,
    expected: std::sync::Arc<
        std::sync::Mutex<std::collections::BTreeMap<u32, (String, u32)>>,
    >,
    nonce_to_k: std::sync::Arc<
        std::sync::Mutex<std::collections::BTreeMap<String, u32>>,
    >,
    identity_of: std::sync::Arc<
        std::sync::Mutex<std::collections::BTreeMap<u32, (String, String)>>,
    >,
    enqueue: std::sync::Arc<
        std::sync::Mutex<std::collections::BTreeMap<u32, u128>>,
    >,
    written: std::sync::Arc<
        std::sync::Mutex<std::collections::BTreeMap<u32, u128>>,
    >,
    st_arc: std::sync::Arc<std::sync::Mutex<Pub>>,
    max_issued: std::sync::Arc<std::sync::atomic::AtomicU32>,
    raw_path: PathBuf,
    t0: Instant,
) {
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                // R5 (F02): a read failure is DURABLE before anything else.
                let mut st = st_arc.lock().unwrap();
                st.stream_errors += 1;
                drop(st);
                let mut f = std::fs::OpenOptions::new().append(true).open(&raw_path).unwrap();
                let _ = writeln!(f, "{}", serde_json::json!({
                    "kind": "stream-error", "lane": lane,
                    "at_ms": ms_since(t0), "error": format!("read: {e}"),
                }));
                continue;
            }
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                // R5 (F02): an undecodable response is DURABLE, never a
                // silent continue — the raw tail names the damage.
                let tail: String = trimmed.chars().rev().take(120).collect::<Vec<_>>().into_iter().rev().collect();
                let mut st = st_arc.lock().unwrap();
                st.stream_errors += 1;
                drop(st);
                let mut f = std::fs::OpenOptions::new().append(true).open(&raw_path).unwrap();
                let _ = writeln!(f, "{}", serde_json::json!({
                    "kind": "stream-error", "lane": lane,
                    "at_ms": ms_since(t0), "error": format!("undecodable: {e}"),
                    "raw_tail": tail,
                }));
                continue;
            }
        };
        let nonce = v["nonce"].as_str().unwrap_or("").to_string();
        let k = match nonce_to_k.lock().unwrap().get(&nonce) {
            Some(k) => *k,
            None => continue,
        };
        // R5 (F01): judge against the REQUEST-side identity — the response
        // echo is only used to correlate, never to authorize.
        let (want_nonce, want_hash) = match identity_of.lock().unwrap().get(&k) {
            Some(x) => x.clone(),
            None => continue,
        };
        let completed_ms = ms_since(t0);
        let (want_text, steps) = {
            let exp = expected.lock().unwrap();
            match exp.get(&k) {
                Some(x) => x.clone(),
                None => continue,
            }
        };
        let judged = {
            let scratch = scratch_of.lock().unwrap().get(&k).cloned();
            match scratch {
                Some(s) => judge_syntax_edit(&v, &want_nonce, &want_hash, &s, &art, &want_text),
                None => continue,
            }
        };
        let mut st = st_arc.lock().unwrap();
        st.completion_order.push(k);
        let out_of_order = st.completion_order.iter().any(|&j| j > k);
        st.in_flight.remove(&k);
        let enq = enqueue.lock().unwrap().get(&k).copied();
        let wr = written.lock().unwrap().get(&k).copied();
        let cancelled = st.superseded.contains(&k);
        // R4-01: the INDEPENDENT oracle decides — an Ok(false) byte
        // mismatch, a wrong revision counter, or a completion older than
        // the newest ENQUEUED revision all refuse; only the clean newest
        // completion publishes.
        let max_enqueued = max_issued.load(std::sync::atomic::Ordering::SeqCst);
        let publish = decide_publication(
            cancelled,
            k,
            st.last_published,
            max_enqueued,
            &judged,
            steps,
        );
        if publish == "published" {
            // publishing k supersedes every older in-flight task
            let superseded: Vec<u32> = st.in_flight.iter().copied().filter(|&j| j < k).collect();
            for j in superseded {
                st.in_flight.remove(&j);
                st.superseded.insert(j);
            }
            st.last_published = k;
        }
        let record = RevRecord {
            revision: k,
            lane,
            enqueued_at_ms: enq.unwrap_or(0),
            written_at_ms: wr,
            completed_at_ms: Some(completed_ms),
            steps,
            out_of_order: Some(out_of_order),
            publish,
            // R4-01: the BYTE-ORACLE verdict travels — Ok(false) is
            // recorded as false, never granted as correct.
            recon_ok: judged.as_ref().ok().map(|(ok, _)| *ok),
            counter_revision: judged.as_ref().ok().and_then(|(_, r)| *r),
            error: judged.as_ref().err().cloned(),
        };
        // durable the moment it exists (R3 F02: no evidence is ever lost)
        {
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .open(&raw_path)
                .unwrap();
            writeln!(
                f,
                "{}",
                serde_json::json!({"kind": "revision-stream", "record": record})
            )
            .unwrap();
        }
        st.records.push(record);
    }
}
