//! `probe` adapter executor for the `abi/interop-v1` recipe (PLAN-797
//! T-06): the parent-side half of the restricted probe/run seam,
//! registered under the SAME contract as `sema/query-v1` — no second
//! runner, no case-carried executable. auto-lang NEVER links the
//! experimental ABI stack: this module spawns the `abi-worker` binary
//! (env override `ABI_CASE_WORKER`, else located from the repo layout
//! and built release once per process) speaking `abiworker/1` (same
//! envelope discipline as semaworker/1: nonce/request_hash echo,
//! deny_unknown_fields, bounded IO, Job containment) and JUDGES the
//! typed step reports against the case oracle. The worker's word is
//! never trusted — outcome admission, cache/compute/stage oracles, and
//! the exit-status/version/exe-sha identity axes are all parent-side.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use auto_test_case::compare::Mismatch;
use auto_test_case::contract::{CheckSpec, Completion, MessageMatcher, ProbeStep};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{NativeRecord, Status};
use auto_test_case::sha256::{hex, sha256};
use base64::Engine as _;
use serde::Deserialize;

use super::avm::Verdict;
use super::sema::containment;

/// Frozen caps (mirrors of the seam's shared discipline).
const READ_CAP: usize = (4 << 20) + (64 << 10);
const STDERR_CAP: usize = 1 << 20;
const REQUEST_CAP: usize = 4 << 20;
const READER_GRACE: Duration = Duration::from_secs(2);
const SESSION_WALL: Duration = Duration::from_secs(60);
const BUILD_WALL: Duration = Duration::from_secs(300);

pub const ABI_WORKER_SCHEMA: &str = "abiworker/1";
pub const ABI_RECIPE: &str = "abi/interop-v1";
/// Mirrored worker identity (drift fails admission loudly).
pub const EXPECTED_ABI_WORKER_VERSION: &str = "abi-worker/0.1.0";

/// Locate (and on first use build) the ABI worker binary: env override
/// first (trusted host configuration binds the tool identity), else
/// find `experimental/auto-abi-probe` from this crate's source root
/// and build the `abi-worker` release binary (bounded).
pub fn worker_binary() -> Result<PathBuf, String> {
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(locate_worker).clone()
}

fn locate_worker() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var("ABI_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "ABI_CASE_WORKER points at a missing file: {}",
            env_path
        ));
    }
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "abi-worker not located: no experimental/auto-abi-probe above the crate and no ABI_CASE_WORKER override".to_string(),
            );
        };
        let m = d
            .join("experimental")
            .join("auto-abi-probe")
            .join("Cargo.toml");
        if m.is_file() {
            break m;
        }
        dir = d.parent();
    };
    let mut child = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--bin",
            "abi-worker",
            "--manifest-path",
        ])
        .arg(&manifest)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn cargo for abi worker build: {}", e))?;
    let started = Instant::now();
    let mut err_pipe = child.stderr.take();
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(p) = err_pipe.as_mut() {
            let mut chunk = [0u8; 16384];
            loop {
                match p.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if buf.len() + n > STDERR_CAP {
                            break;
                        }
                        buf.extend_from_slice(&chunk[..n]);
                    }
                }
            }
        }
        buf
    });
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let err = err_reader.join().unwrap_or_default();
                    return Err(format!(
                        "abi worker build failed: {}",
                        String::from_utf8_lossy(&err)
                    ));
                }
                break;
            }
            Ok(None) => {
                if started.elapsed() > BUILD_WALL {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("abi worker build exceeded the 300s wall".to_string());
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("wait abi worker build: {e}")),
        }
    }
    let exe = manifest
        .parent()
        .unwrap()
        .join("target/release/abi-worker.exe");
    if exe.is_file() {
        Ok(exe)
    } else {
        Err(format!(
            "abi worker build produced no exe at {}",
            exe.display()
        ))
    }
}

/// Current toolchain identity for stale adjudication of ABI probe rows
/// (recomputes the SAME axes the report writes).
pub fn current_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();
    let exe_sha = worker_exe_sha256().unwrap_or_default();
    for (k, v) in abi_toolchain_axes(&exe_sha) {
        m.insert(k, v);
    }
    m
}

/// The report's ABI toolchain axes: the SAME set the stale checker
/// declares applicable for probe rows (recipe family differs).
fn abi_toolchain_axes(exe_sha: &str) -> Vec<(String, String)> {
    vec![
        ("abi-protocol".into(), ABI_WORKER_SCHEMA.into()),
        ("recipe".into(), ABI_RECIPE.into()),
        ("abi-worker-exe-sha256".into(), exe_sha.to_string()),
        (
            "abi-worker-version".into(),
            EXPECTED_ABI_WORKER_VERSION.to_string(),
        ),
    ]
}

/// Content identity of the worker tool actually spawned.
fn worker_exe_sha256() -> Result<String, String> {
    let exe = worker_binary()?;
    let bytes = std::fs::read(&exe).map_err(|e| format!("read worker exe: {e}"))?;
    Ok(hex(&sha256(&bytes)))
}

// ---- typed response mirror ---------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerResponse {
    schema: String,
    #[serde(default)]
    nonce: String,
    #[serde(default)]
    request_hash: String,
    #[serde(default)]
    ok: Option<OkOut>,
    #[serde(default)]
    error: Option<WorkerError>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerError {
    code: String,
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct OkOut {
    recipe: String,
    steps: Vec<StepOut>,
    restart_pending: bool,
    worker_version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct StepOut {
    op: String,
    outcome: String,
    cache: String,
    compute_calls: u32,
    #[serde(default)]
    diagnostics: Vec<DiagMirror>,
    #[serde(default)]
    stages: Vec<StageSpanMirror>,
    #[serde(default)]
    meter: MeterMirror,
    #[serde(default)]
    pe_exit: Option<i32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct MeterMirror {
    /// DISK bytes of the worker's scratch dir (NOT memory — the process
    /// memory peak / I/O come from the parent-side Job stats).
    #[serde(default)]
    scratch_peak_disk_bytes: u64,
    #[serde(default)]
    hot_bytes: u64,
    #[serde(default)]
    rejections: u32,
    #[serde(default)]
    gc_runs: u32,
    #[serde(default)]
    gc_deleted_chunks: u32,
    #[serde(default)]
    gc_freed_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct DiagMirror {
    code: String,
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageSpanMirror {
    name: String,
    wall_ns: u128,
}

// ---- worker lifecycle (contained, bounded; mirrors the seam) -----------

/// TEST-ONLY fault injection (F-797-R4-03 controls), mirroring sema's
/// discipline: child env vars + an optional session-wall override,
/// applied only to spawned worker processes while a test holds the
/// lock. Cases can never reach this — not part of the contract decode.
#[doc(hidden)]
pub struct TestFaults {
    pub child_envs: Vec<(String, String)>,
    pub session_wall_ms: Option<u64>,
}

#[doc(hidden)]
pub static TEST_FAULTS: std::sync::RwLock<Option<TestFaults>> = std::sync::RwLock::new(None);

fn active_faults() -> Option<(Vec<(String, String)>, Option<u64>)> {
    TEST_FAULTS.read().ok().and_then(|g| {
        g.as_ref()
            .map(|f| (f.child_envs.clone(), f.session_wall_ms))
    })
}

fn call_worker(
    request_json: &str,
    deadline: Duration,
) -> Result<(WorkerResponse, Option<containment::JobStats>), String> {
    if request_json.len() > REQUEST_CAP {
        return Err(format!(
            "abi worker request exceeds the {} byte metadata cap ({} bytes)",
            REQUEST_CAP,
            request_json.len()
        ));
    }
    let exe = worker_binary()?;
    let mut cmd = Command::new(&exe);
    cmd.arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let faults = active_faults();
    let deadline = match &faults {
        Some((_, Some(ms))) => Duration::from_millis(*ms),
        _ => deadline,
    };
    if let Some((envs, _)) = &faults {
        for (k, v) in envs {
            cmd.env(k, v);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn abi worker ({}): {}", exe.display(), e))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "containment unavailable: refusing an uncontained abi worker tree".to_string(),
            );
        }
    };
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;
    #[cfg(windows)]
    if let Err(e) = containment::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("containment: {e}"));
    }
    let _ = &mut job;
    {
        let mut stdin = child.stdin.take().expect("piped stdin");
        let req = request_json.to_string();
        std::thread::spawn(move || {
            let _ = writeln!(stdin, r#"{{"op":"begin","schema":"{ABI_WORKER_SCHEMA}"}}"#);
            let _ = writeln!(stdin, r#"{{"op":"request","request":{}}}"#, req);
            let _ = writeln!(stdin, r#"{{"op":"end"}}"#);
            let _ = stdin.flush();
        });
    }
    let mut stdout_pipe = child.stdout.take().expect("piped stdout");
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");
    let (err_tx, err_rx) = std::sync::mpsc::channel::<(bool, Vec<u8>)>();
    let (tx, rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let _out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 65536];
        let mut done = false;
        while !done {
            match stdout_pipe.read(&mut chunk) {
                Ok(0) => done = true,
                Ok(n) => {
                    if buf.len() + n > READ_CAP {
                        let _ = tx.send(Err(buf.len() + n));
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.ends_with(b"\n") {
                        done = true;
                    }
                }
                Err(_) => done = true,
            }
        }
        let _ = tx.send(Ok(buf));
    });
    let _err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 16384];
        let mut overflow = false;
        loop {
            match stderr_pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.len() > STDERR_CAP {
                        overflow = true;
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = err_tx.send((overflow, buf));
    });
    let started = Instant::now();
    // F-797-R4-12 (R8): the loop *produces* the response value (no
    // unread mutable initializer); every break carries it.
    let response_opt: Option<Result<WorkerResponse, String>> = loop {
        match rx.try_recv() {
            Ok(Ok(bytes)) => {
                break Some(
                    serde_json::from_slice(&bytes)
                        .map_err(|e| format!("abi worker response malformed: {e}")),
                );
            }
            Ok(Err(observed)) => {
                break Some(Err(format!(
                    "abi worker response exceeds the {} byte cap ({} observed)",
                    READ_CAP, observed
                )));
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                break Some(Err("abi worker response stream ended early".into()));
            }
        }
        if let Ok((true, _)) = err_rx.try_recv() {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        if started.elapsed() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("abi worker session exceeded the wall".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    if response_opt.is_none() {
        return Err("abi worker stderr overflowed the 1MiB cap — tree killed".into());
    }
    // F-797-R4-03: ONE deadline tiles the WHOLE session — response,
    // exit, and reap. A worker that answered then slept, or that is
    // still exiting, is killed at the SAME wall; there is no unbounded
    // wait after the response.
    // F-797-R4-03 (R5): the EXIT loop also polls the stderr channel —
    // an overflow observed while the worker is still running (the
    // combined fault: echo, then >cap stderr, then keep sleeping)
    // terminates the tree IMMEDIATELY, not after the wall.
    let deadline_at = started + deadline;
    loop {
        if let Ok((true, _)) = err_rx.try_recv() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = job.take();
            return Err(
                "abi worker stderr overflowed the 1MiB cap (still running) — tree killed".into(),
            );
        }
        match child.try_wait() {
            Ok(Some(st)) => {
                if !st.success() {
                    let _ = job.take();
                    return Err(format!("abi worker exited abnormally: {st}"));
                }
                break; // clean exit — fall through to the drain grace
            }
            Ok(None) => {
                if Instant::now() > deadline_at {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = job.take();
                    return Err(format!(
                        "abi worker session exceeded the wall (exit phase; response took {:.0}ms)",
                        (deadline - started.elapsed().min(deadline)).as_millis()
                    ));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let _ = job.take();
                return Err(format!("wait abi worker: {e}"));
            }
        }
    }
    // F-797-R4-10 (R8): the job's peak-memory and I/O counters are read
    // while the handle is still open — the per-request worker resource
    // attribution published with the sample.
    let job_stats = job.as_ref().and_then(|j| j.stats());
    let _ = job.take();

    // F-797-R4-03: the stderr OVERFLOW FLAG is decisive even when it is
    // observed AFTER the response — a late >cap write is an
    // infrastructure error, never a PASS. Bounded drain grace.
    let drain_until = std::cmp::min(deadline_at, Instant::now() + READER_GRACE);
    let mut stderr_overflow: Option<bool> = None;
    loop {
        match err_rx.try_recv() {
            Ok((overflow, _)) => {
                stderr_overflow = Some(overflow);
                break;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                if Instant::now() > drain_until {
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
        }
    }
    if stderr_overflow == Some(true) {
        return Err(
            "abi worker stderr overflowed the 1MiB cap (observed after response) — resource error"
                .into(),
        );
    }

    let response = match response_opt {
        Some(r) => r,
        None => Err("abi worker produced no response".into()),
    };
    // F-797-R4-03: exactly one of `ok` / `error` may be present — a
    // contradictory envelope is a protocol error, never `ok`.
    match response.as_ref() {
        Ok(r) => {
            let both = r.ok.is_some() && r.error.is_some();
            let neither = r.ok.is_none() && r.error.is_none();
            if both || neither {
                return Err(format!(
                    "abi worker envelope must carry exactly one of ok/error (ok={}, error={})",
                    r.ok.is_some(),
                    r.error.is_some()
                ));
            }
        }
        Err(_) => {}
    }
    response.map(|r| (r, job_stats))
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn replace_once(src: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    let hay = String::from_utf8_lossy(src).to_string();
    let Some(first) = hay.find(from) else {
        return Err(format!("edit-source `from` not found: {from:?}"));
    };
    if hay[first + from.len()..].contains(from) {
        return Err(format!(
            "edit-source `from` matches more than once: {from:?}"
        ));
    }
    let mut out = String::with_capacity(hay.len() + to.len());
    out.push_str(&hay[..first]);
    out.push_str(to);
    out.push_str(&hay[first + from.len()..]);
    Ok(out.into_bytes())
}

/// F-797-R4-09 (R9): per-request typed judgement — the SAME rules the
/// post-loop batch judge applied, evaluated IMMEDIATELY when a request
/// returns so the request span closes at the acceptance boundary.
fn judge_query_step(
    completion: &Completion,
    got: &StepOut,
    want: &auto_test_case::ir::ResolvedProbeStep,
    i: usize,
    mismatches: &mut Vec<Mismatch>,
) {
    match (completion, got.outcome.as_str()) {
        (Completion::Success, "ok") => {
            if !got.diagnostics.is_empty() {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].outcome"),
                    kind: "text",
                    expected: "ok with zero diagnostics".into(),
                    actual: format!(
                        "ok carrying {} diagnostic(s) [{}]",
                        got.diagnostics.len(),
                        got.diagnostics
                            .first()
                            .map(|d| d.code.as_str())
                            .unwrap_or("?")
                    ),
                    first_diff: None,
                });
            }
            // a successful query must be a REAL PE pass
            if got.pe_exit != Some(0) {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].pe_exit"),
                    kind: "text",
                    expected: "0".into(),
                    actual: format!("{:?}", got.pe_exit),
                    first_diff: None,
                });
            }
        }
        (
            Completion::Error {
                stage,
                code,
                message,
            },
            "diags",
        ) => {
            let want_stage = stage.as_str();
            let matched = got.diagnostics.iter().any(|d| {
                stage_of(&d.code) == want_stage
                    && code.as_deref().map(|c| c == d.code).unwrap_or(true)
                    && match message {
                        Some(MessageMatcher::Exact(m)) => &d.message == m,
                        Some(MessageMatcher::Contains(m)) => d.message.contains(m.as_str()),
                        None => true,
                    }
            });
            if !matched {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].diagnostics"),
                    kind: "message",
                    expected: format!(
                        "a {}-stage diagnostic matching code={:?} message={:?}",
                        want_stage,
                        code.as_deref().unwrap_or("-"),
                        message
                    ),
                    actual: got
                        .diagnostics
                        .iter()
                        .map(|d| format!("{}: {}", stage_of(&d.code), d.message))
                        .collect::<Vec<_>>()
                        .join(" | "),
                    first_diff: None,
                });
            }
        }
        (Completion::Success, other) => {
            mismatches.push(Mismatch {
                observation: format!("probe[{i}].outcome"),
                kind: "text",
                expected: "success".into(),
                actual: format!("{other} (cache={})", got.cache),
                first_diff: None,
            });
        }
        (Completion::Error { .. }, other) => {
            mismatches.push(Mismatch {
                observation: format!("probe[{i}].outcome"),
                kind: "text",
                expected: "a declared error completion".into(),
                actual: format!("{other} with zero matching diagnostics"),
                first_diff: None,
            });
        }
    }
    if let Some(want_cache) = &want.cache {
        if &got.cache != want_cache {
            mismatches.push(Mismatch {
                observation: format!("probe[{i}].cache"),
                kind: "text",
                expected: want_cache.clone(),
                actual: got.cache.clone(),
                first_diff: None,
            });
        }
    }
    if let Some(want_calls) = want.compute_calls {
        if got.compute_calls != want_calls {
            mismatches.push(Mismatch {
                observation: format!("probe[{i}].compute_calls"),
                kind: "text",
                expected: format!("{want_calls}"),
                actual: format!("{}", got.compute_calls),
                first_diff: None,
            });
        }
    }
    for inc in &want.stages_include {
        if !got.stages.iter().any(|s| &s.name == inc) {
            mismatches.push(Mismatch {
                observation: format!("probe[{i}].stages"),
                kind: "text",
                expected: format!("stage `{inc}` present"),
                actual: got
                    .stages
                    .iter()
                    .map(|s| s.name.clone())
                    .collect::<Vec<_>>()
                    .join(","),
                first_diff: None,
            });
        }
    }
}

/// F-797-R4-09 (R9): append ONE raw request row — persisted BEFORE the
/// next request runs, successful and failed requests alike. The row
/// carries the request's full attribution: edit wall (parent-local),
/// request wall to the acceptance boundary, worker outcome/cache/
/// compute, Job-object peak memory (process AND tree) and I/O, worker
/// meter (incl. GC receipt facts), and the failure reason when any.
fn append_request_row(
    path: &Path,
    seq: usize,
    op: &str,
    judged_ok: bool,
    edit_ms: u64,
    request_ms: u64,
    step: Option<&StepOut>,
    stats: Option<&containment::JobStats>,
    error: Option<&str>,
) -> Result<(), String> {
    use std::io::Write as _;
    let ok = judged_ok && error.is_none();
    let row = serde_json::json!({
        "seq": seq,
        "op": op,
        "ok": ok,
        "edit_ms": edit_ms,
        "request_ms": request_ms,
        "outcome": step.map(|s| s.outcome.clone()),
        "pe_exit": step.and_then(|s| s.pe_exit),
        "cache": step.map(|s| s.cache.clone()),
        "compute_calls": step.map(|s| s.compute_calls),
        "stages": step.map(|s| s
            .stages
            .iter()
            .map(|sp| (sp.name.clone(), sp.wall_ns / 1_000_000))
            .collect::<std::collections::BTreeMap<String, u128>>()),
        "meter": step.map(|s| serde_json::json!({
            "hot_bytes": s.meter.hot_bytes,
            "scratch_disk_bytes": s.meter.scratch_peak_disk_bytes,
            "rejections": s.meter.rejections,
            "gc_runs": s.meter.gc_runs,
            "gc_deleted_chunks": s.meter.gc_deleted_chunks,
            "gc_freed_bytes": s.meter.gc_freed_bytes,
        })),
        "peak_mem_bytes": stats.map(|s| s.peak_process_memory_used),
        "peak_job_mem_bytes": stats.map(|s| s.peak_job_memory_used),
        "io_read_bytes": stats.map(|s| s.read_transfer_bytes),
        "io_write_bytes": stats.map(|s| s.write_transfer_bytes),
        "error": error,
    });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("request-evidence dir {}: {e}", parent.display()))?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("request-evidence {}: {e}", path.display()))?;
    writeln!(f, "{row}").map_err(|e| format!("request-evidence {}: {e}", path.display()))?;
    Ok(())
}

/// Map an ABI diagnostic code to a completion stage (the declared error
/// completion judges against this): `deck.*`/parse → parse, admission →
/// admit, pe-exit → run.
fn stage_of(code: &str) -> &'static str {
    match code {
        "deck" => "parse",
        c if c.starts_with("admission/") => "admit",
        "pe-exit" => "run",
        _ => "infra",
    }
}

/// Run one `abi/interop-v1` check: session-slice at restarts, contained
/// worker calls, parent-side typed admission + step oracle judgement.
pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    // recipe×source admission: abi/interop-v1 consumes DATA decks
    // (`.atomdata`), never an executable source kind.
    if case.spec.source.syntax != auto_test_case::contract::Syntax::AtomData {
        return Err(format!(
            "abi/interop-v1 consumes .atomdata decks; syntax {:?} is not accepted by this recipe",
            case.spec.source.syntax.as_str()
        ));
    }
    let abi = check
        .options
        .probe_abi
        .as_ref()
        .ok_or("probe check without an abi binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if check.options.probe_recipe.as_deref() != Some(ABI_RECIPE) {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    // Run belongs only to the AV recipe. Also reject programmatically
    // constructed checks before creating artifacts or locating a worker.
    if steps.iter().any(|step| matches!(step, ProbeStep::Run)) {
        return Err("abi/interop-v1 does not accept run steps".into());
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    let store_root = artifact_dir.join("store");
    std::fs::create_dir_all(&store_root).map_err(|e| format!("probe store root: {e}"))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("probe artifact dir: {e}"))?;

    let mut deck = input_bytes.clone();

    let mut mismatches: Vec<Mismatch> = Vec::new();
    /// F-797-R4-09 (R8/R9): each reported query step paired with the JOB
    /// STATS of the request that produced it (a query request is a
    /// whole worker process — its peak memory and I/O are the sample's
    /// resource attribution).
    struct QuerySample {
        step: StepOut,
        stats: Option<containment::JobStats>,
    }
    let mut reported: Vec<QuerySample> = Vec::new();
    let mut stage_spans: Vec<(String, u64)> = Vec::new();
    let mut infra_detail: Option<String> = None;
    let exe_sha = worker_exe_sha256()?;
    let expects = oracle.probe.as_deref().unwrap_or(&[]);

    // F-797-R4-09 (R9): RAW PER-REQUEST EVIDENCE — one row per request,
    // appended to the check's artifact dir BEFORE the next request runs
    // (failed/infra requests included, each with its own ok flag). The
    // formal gates aggregate THESE rows; nothing is batch-written after
    // the case returns.
    let samples_path = artifact_dir.join("abi-request-samples.jsonl");
    let mut sample_seq: usize = 0;
    let mut pending_edit_ms: u64 = 0;

    // F-797-R4-09 (R8/R9): every WORKER request is its own fully-published
    // unit — build → IPC → wait → decode → admission → TYPED JUDGEMENT —
    // timed at the parent entry as `parent-request:<op>` and closed only
    // AFTER the request's outcome has been judged against the oracle
    // (the acceptance boundary). The worker's stage spans are recorded
    // after their request span as sub-attribution. (The R5 form bundled
    // 31 steps into one worker session and summed worker-internal stage
    // spans as "request_ms"; the R8 form closed the span before the
    // final judgement and persisted rows in one batch after the case.)
    // edit-source steps are applied PARENT-SIDE only — the product
    // request carries the CURRENT deck (the source) with each query —
    // and their (microsecond) wall is carried into the NEXT query row
    // as `edit_ms` so the sample is the full edit→result pair. The
    // worker protocol still supports edit-source ops (driven directly
    // by the worker tests).
    'steps: for (step_no, st) in steps.iter().enumerate() {
        if let ProbeStep::EditSource { from, to } = st {
            let t_edit = Instant::now();
            deck = replace_once(&deck, from, to)?;
            pending_edit_ms = t_edit.elapsed().as_millis() as u64;
            continue;
        }
        let step_json = match st {
            ProbeStep::Query => serde_json::json!({"op": "query"}),
            ProbeStep::Run => return Err("abi/interop-v1 does not accept run steps".into()),
            // SAFETY: edit-source continues above; restart is the only
            // other valid ABI step kind (AV run was rejected above)
            ProbeStep::EditSource { .. } => unreachable!("edit-source handled above"),
            ProbeStep::Restart => serde_json::json!({"op": "restart"}),
        };
        let step_tag = match st {
            ProbeStep::Query => "query",
            ProbeStep::Run => return Err("abi/interop-v1 does not accept run steps".into()),
            ProbeStep::EditSource { .. } => unreachable!("edit-source handled above"),
            ProbeStep::Restart => "restart",
        };
        let t_step = Instant::now();
        let nonce = hex(&sha256(
            format!(
                "{}|{:?}|{}|{}|abi",
                std::process::id(),
                t_step,
                step_no,
                step_tag
            )
            .as_bytes(),
        ));
        let mut request = serde_json::json!({
            "recipe": ABI_RECIPE,
            "nonce": nonce,
            "deck_b64": b64(&deck),
            "leg": abi.leg,
            "store_root": store_root.display().to_string().replace('/', "\\"),
            "steps": [step_json],
        });
        let request_hash = hex(&sha256(request.to_string().as_bytes()));
        request["request_hash"] = serde_json::json!(request_hash);

        let (resp, job_stats) = match call_worker(&request.to_string(), SESSION_WALL) {
            Ok(r) => r,
            Err(e) => {
                stage_spans.push((
                    format!("parent-request:{step_tag}"),
                    t_step.elapsed().as_millis() as u64,
                ));
                let persist = append_request_row(
                    &samples_path,
                    sample_seq,
                    step_tag,
                    false,
                    pending_edit_ms,
                    t_step.elapsed().as_millis() as u64,
                    None,
                    None,
                    Some(&e),
                );
                infra_detail = Some(match persist {
                    Err(pe) => pe,
                    Ok(()) => e,
                });
                break 'steps;
            }
        };
        let admitted: Result<OkOut, String> = (|| {
            if resp.schema != ABI_WORKER_SCHEMA {
                return Err(format!("abi worker schema mismatch: {}", resp.schema));
            }
            if resp.nonce != nonce || resp.request_hash != request_hash {
                return Err("abi worker nonce/request_hash echo mismatch".into());
            }
            let ok = match resp.ok.clone() {
                Some(ok) => ok,
                None => {
                    let e = resp.error.clone().unwrap_or(WorkerError {
                        code: "missing".into(),
                        message: "neither ok nor error".into(),
                    });
                    return Err(format!("abi worker error {}: {}", e.code, e.message));
                }
            };
            if ok.recipe != ABI_RECIPE {
                return Err(format!("abi worker recipe mismatch: {}", ok.recipe));
            }
            if ok.worker_version != EXPECTED_ABI_WORKER_VERSION {
                return Err(format!(
                    "abi worker version `{}` does not match the admitted `{}`",
                    ok.worker_version, EXPECTED_ABI_WORKER_VERSION
                ));
            }
            Ok(ok)
        })();
        let ok = match admitted {
            Ok(o) => o,
            Err(e) => {
                stage_spans.push((
                    format!("parent-request:{step_tag}"),
                    t_step.elapsed().as_millis() as u64,
                ));
                let persist = append_request_row(
                    &samples_path,
                    sample_seq,
                    step_tag,
                    false,
                    pending_edit_ms,
                    t_step.elapsed().as_millis() as u64,
                    None,
                    None,
                    Some(&e),
                );
                infra_detail = Some(match persist {
                    Err(pe) => pe,
                    Ok(()) => e,
                });
                break 'steps;
            }
        };
        if matches!(st, ProbeStep::Restart) {
            let ok_row = ok.restart_pending;
            stage_spans.push((
                format!("parent-request:{step_tag}"),
                t_step.elapsed().as_millis() as u64,
            ));
            let persist = append_request_row(
                &samples_path,
                sample_seq,
                step_tag,
                ok_row,
                pending_edit_ms,
                t_step.elapsed().as_millis() as u64,
                None,
                job_stats.as_ref(),
                if ok_row {
                    None
                } else {
                    Some("restart step did not end the session as declared")
                },
            );
            sample_seq += 1;
            pending_edit_ms = 0;
            if let Err(pe) = persist {
                infra_detail = Some(pe);
                break 'steps;
            }
            if !ok_row {
                infra_detail = Some("restart step did not end the session as declared".into());
                break 'steps;
            }
            continue;
        }
        // query step: judge IMMEDIATELY so the request span closes at
        // the acceptance boundary (admission + typed judgement + row)
        let qi = reported.len();
        // F-797-R4-09 (R10): the row's ok is THIS request's own judged
        // outcome — a typed/cache/compute/stage acceptance mismatch
        // marks the row failed while later requests stay independently
        // classified
        let m_before = mismatches.len();
        let mut query_step: Option<&StepOut> = None;
        for s in ok.steps.iter().filter(|s| s.op == "query") {
            query_step = Some(s);
            // F-797-R4-03 (R5): worker-reported infra/resource outcomes
            // are INFRASTRUCTURE short-circuits — judged before any
            // language completion, reported as Status::Error, never FAIL.
            if s.outcome == "infra" || s.outcome == "resource" {
                let detail = format!(
                    "abi worker reported {} on query step {qi}: {}",
                    s.outcome,
                    s.diagnostics
                        .first()
                        .map(|d| d.message.clone())
                        .unwrap_or_default()
                );
                stage_spans.push((
                    format!("parent-request:{step_tag}"),
                    t_step.elapsed().as_millis() as u64,
                ));
                let persist = append_request_row(
                    &samples_path,
                    sample_seq,
                    step_tag,
                    false,
                    pending_edit_ms,
                    t_step.elapsed().as_millis() as u64,
                    Some(s),
                    job_stats.as_ref(),
                    Some(&detail),
                );
                infra_detail = Some(match persist {
                    Err(pe) => pe,
                    Ok(()) => detail,
                });
                break 'steps;
            }
            match expects.get(qi) {
                Some(want) => judge_query_step(&oracle.completion, s, want, qi, &mut mismatches),
                None => mismatches.push(Mismatch {
                    observation: "probe.steps".into(),
                    kind: "text",
                    expected: format!("{} query steps", expects.len()),
                    actual: format!("{} query steps", qi + 1),
                    first_diff: None,
                }),
            }
        }
        if let Some(s) = query_step {
            reported.push(QuerySample {
                step: s.clone(),
                stats: job_stats.clone(),
            });
        }
        let judged_ok = mismatches.len() == m_before;
        let judged_error: Option<String> = if judged_ok {
            None
        } else {
            Some(
                mismatches[m_before..]
                    .iter()
                    .map(|m| {
                        format!(
                            "{}: expected {}, actual {}",
                            m.observation, m.expected, m.actual
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        };
        // the row is persisted FIRST (its request_ms is measured at
        // judged time, before the write); the request span then closes
        // AFTER the row write COMPLETED — the acceptance boundary is
        // build + IPC + wait + decode + admission + typed judgement +
        // raw-evidence persistence (F-797-R4-09 R11)
        // evidence persistence failure is an INFRASTRUCTURE error — the
        // raw-row chain must never silently break
        append_request_row(
            &samples_path,
            sample_seq,
            step_tag,
            judged_ok,
            pending_edit_ms,
            t_step.elapsed().as_millis() as u64,
            query_step,
            job_stats.as_ref(),
            judged_error.as_deref(),
        )?;
        stage_spans.push((
            format!("parent-request:{step_tag}"),
            t_step.elapsed().as_millis() as u64,
        ));
        if let Some(s) = query_step {
            for sp in &s.stages {
                stage_spans.push((sp.name.clone(), (sp.wall_ns / 1_000_000) as u64));
            }
        }
        sample_seq += 1;
        pending_edit_ms = 0;
    }
    let reported_query_steps: Vec<StepOut> = reported.iter().map(|q| q.step.clone()).collect();

    // ---- shared sample aggregation (F-797-R4-09/-10 R8) ---------------
    let query_steps_n = reported.len() as u32;
    let compute_total: u32 = reported.iter().map(|q| q.step.compute_calls).sum();
    let hot_max: u64 = reported
        .iter()
        .map(|q| q.step.meter.hot_bytes)
        .max()
        .unwrap_or(0);
    let scratch_disk_max: u64 = reported
        .iter()
        .map(|q| q.step.meter.scratch_peak_disk_bytes)
        .max()
        .unwrap_or(0);
    let rejections_total: u32 = reported.iter().map(|q| q.step.meter.rejections).sum();
    let peak_mem_max: u64 = reported
        .iter()
        .filter_map(|q| q.stats.as_ref().map(|s| s.peak_process_memory_used))
        .max()
        .unwrap_or(0);
    let io_read_total: u64 = reported
        .iter()
        .filter_map(|q| q.stats.as_ref().map(|s| s.read_transfer_bytes))
        .sum();
    let io_write_total: u64 = reported
        .iter()
        .filter_map(|q| q.stats.as_ref().map(|s| s.write_transfer_bytes))
        .sum();
    let u32cap = |v: u64| u32::try_from(v).unwrap_or(u32::MAX);
    let mut sample_counters: Vec<(String, u32)> = vec![
        ("query_steps".into(), query_steps_n),
        ("compute_calls".into(), compute_total),
        ("meter_hot_bytes".into(), u32cap(hot_max)),
        (
            "meter_scratch_peak_disk_bytes".into(),
            u32cap(scratch_disk_max),
        ),
        ("meter_rejections".into(), rejections_total),
        ("worker_peak_mem_bytes".into(), u32cap(peak_mem_max)),
        ("worker_io_read_bytes".into(), u32cap(io_read_total)),
        ("worker_io_write_bytes".into(), u32cap(io_write_total)),
        (
            "worker_peak_job_mem_bytes".into(),
            u32cap(
                reported
                    .iter()
                    .filter_map(|q| q.stats.as_ref().map(|s| s.peak_job_memory_used))
                    .max()
                    .unwrap_or(0),
            ),
        ),
        (
            "meter_gc_deleted_chunks".into(),
            reported
                .iter()
                .map(|q| q.step.meter.gc_deleted_chunks)
                .sum(),
        ),
    ];
    // per-sample observed facts (the formal JSONL consumes these):
    // producer count, cache state, and the request's REAL resource
    // attribution from the worker's Job object.
    let mut cache_hits: Vec<(String, String)> = Vec::new();
    for (i, q) in reported.iter().enumerate() {
        cache_hits.push((format!("q{i}"), q.step.cache.clone()));
        let st = q.stats;
        sample_counters.push((format!("req_q{i}_compute"), q.step.compute_calls));
        sample_counters.push((
            format!("req_q{i}_peak_mem_bytes"),
            u32cap(st.map(|s| s.peak_process_memory_used).unwrap_or(0)),
        ));
        sample_counters.push((
            format!("req_q{i}_peak_job_mem_bytes"),
            u32cap(st.map(|s| s.peak_job_memory_used).unwrap_or(0)),
        ));
        sample_counters.push((
            format!("req_q{i}_io_read_bytes"),
            u32cap(st.map(|s| s.read_transfer_bytes).unwrap_or(0)),
        ));
        sample_counters.push((
            format!("req_q{i}_io_write_bytes"),
            u32cap(st.map(|s| s.write_transfer_bytes).unwrap_or(0)),
        ));
        sample_counters.push((
            format!("req_q{i}_hot_bytes"),
            u32cap(q.step.meter.hot_bytes),
        ));
        sample_counters.push((
            format!("req_q{i}_scratch_disk_bytes"),
            u32cap(q.step.meter.scratch_peak_disk_bytes),
        ));
    }

    // ---- infrastructure short-circuit -----------------------------------
    if let Some(detail) = infra_detail {
        let mut spans = stage_spans;
        spans.push((
            "parent-publish".into(),
            started.elapsed().as_millis() as u64,
        ));
        return Ok(Verdict {
            status: Status::Error,
            mismatches: vec![],
            detail: Some(detail),
            artifacts: vec![],
            native: Some(NativeRecord {
                exit_code: None,
                stage_spans: spans,
                cache_mode: Some("abi-store".into()),
                cache_hits,
                counters: sample_counters,
                toolchain: abi_toolchain_axes(&exe_sha),
                gaps: vec![],
            }),
        });
    }

    // ---- step-count check (typed judgement already ran per request) --
    if reported_query_steps.len() != expects.len() {
        mismatches.push(Mismatch {
            observation: "probe.steps".into(),
            kind: "text",
            expected: format!("{} query steps", expects.len()),
            actual: format!("{} query steps", reported_query_steps.len()),
            first_diff: None,
        });
    }

    let status = if mismatches.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    // F-797-R4-09 (R8): the whole-check parent path — entry through
    // typed judgement to publication.
    let mut spans = stage_spans;
    spans.push((
        "parent-publish".into(),
        started.elapsed().as_millis() as u64,
    ));
    Ok(Verdict {
        status,
        detail: if mismatches.is_empty() {
            None
        } else {
            Some(format!(
                "abi probe: {} mismatch(es) over {} query step(s)",
                mismatches.len(),
                reported_query_steps.len()
            ))
        },
        mismatches,
        artifacts: vec![],
        native: Some(NativeRecord {
            exit_code: reported_query_steps.last().and_then(|s| s.pe_exit),
            stage_spans: spans,
            cache_mode: Some("abi-store".into()),
            cache_hits,
            counters: sample_counters,
            toolchain: abi_toolchain_axes(&exe_sha),
            gaps: vec![],
        }),
    })
}
