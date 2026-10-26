//! PLAN-798 T-07: the `ac-av/core-run-v1` probe recipe — formal AutoDown
//! execution of the av probe through the unified probe/run seam landed by
//! PLAN-796 (which itself rides the PLAN-788 TestCase v2 runner).
//!
//! Same contract shape as `sema.rs` (the seam's first recipe): the case
//! declares a recipe + one declarative `run` step + an `av` binding
//! (entry/caps); this adapter spawns the plan's own worker
//! (`av_probe_worker case-serve`, envelope `avworker/1`) under Job
//! containment, admits the envelope/nonce/request-hash/recipe/version
//! echoes, and judges the observed outcome PARENT-SIDE against the
//! declared expectations. The worker never sees the expected outcomes.
//!
//! Recipe semantics (v1): one run = the full front-door → HIR → MIR →
//! ABC → AV pipeline on the case source. Judgement faces:
//! - `completion: "success"` + `exit: <i32>` — AV Success value;
//! - `completion: { stage: "run", message: { contains: <trap code> } }` —
//!   a language trap (an executed-outcome, never an error completion);
//! - `completion: { stage: "parse" | "verify", code: <diag code> }` — a
//!   diagnosed admission/frontend/pipeline rejection.

use crate::test_cases::sema::containment;
use auto_test_case::compare::Mismatch;
use auto_test_case::contract::{CheckSpec, Completion, MessageMatcher, Stage};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{NativeRecord, Status};
use auto_test_case::sha256::{hex, sha256};
use base64::Engine as _;

use super::avm::Verdict;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const READ_CAP: usize = (4 << 20) + (64 << 10);
const STDERR_CAP: usize = 1 << 20;
const REQUEST_CAP: usize = 4 << 20;
const READER_GRACE: Duration = Duration::from_secs(2);
const SESSION_WALL: Duration = Duration::from_secs(60);
const BUILD_WALL: Duration = Duration::from_secs(300);

pub const AV_RECIPE: &str = "ac-av/core-run-v1";
/// PLAN-820: the data-profile recipe — record source → data MIR → AVAD →
/// AV through the same worker/executor contract; judgement stays
/// parent-side. Registered as its own recipe identity (append-only; the
/// core recipe contract is untouched).
pub const AV_DATA_RECIPE: &str = "ac-av/data-run-v1";
pub const EXPECTED_AV_DATA_CORE_VERSION: &str = "av.data/1";
/// PLAN-821: the owned-profile recipe (`av.owned/1`, profile id 3) —
/// text-record source → owned Typed MIR → AVAD v2 → AV with the arena
/// ledger face. Same append-only registration rules; the owned face is
/// judged parent-side against THIS crate's frozen expectations (result +
/// live=0 + alloc/free order), never the worker's word.
pub const AV_OWNED_RECIPE: &str = "ac-av/owned-run-v1";
pub const EXPECTED_AV_OWNED_CORE_VERSION: &str = "av.owned/1";

/// Which registered av recipe a check runs under.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RecipeFamily {
    Core,
    Data,
    Owned,
}

impl RecipeFamily {
    pub fn recipe(&self) -> &'static str {
        match self {
            RecipeFamily::Core => AV_RECIPE,
            RecipeFamily::Data => AV_DATA_RECIPE,
            RecipeFamily::Owned => AV_OWNED_RECIPE,
        }
    }
    pub fn expected_core_version(&self) -> &'static str {
        match self {
            RecipeFamily::Core => EXPECTED_AV_CORE_VERSION,
            RecipeFamily::Data => EXPECTED_AV_DATA_CORE_VERSION,
            RecipeFamily::Owned => EXPECTED_AV_OWNED_CORE_VERSION,
        }
    }
}
pub const AV_WORKER_PROTOCOL: &str = "avworker/1";
/// Mirrored worker identity expectations — admission constants; a worker
/// bump without a mirror update fails loudly.
pub const EXPECTED_AV_WORKER_VERSION: &str = "avworker/1.0";
pub const EXPECTED_AV_CORE_VERSION: &str = "av.core-i32/1";

/// Current av toolchain identity (the axes stale adjudication binds probe
/// rows of THIS recipe to).
pub fn current_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();
    if let Ok(sha) = worker_exe_sha256() {
        m.insert("av-worker-exe-sha256".to_string(), sha);
    }
    m.insert(
        "av-worker-version".to_string(),
        EXPECTED_AV_WORKER_VERSION.to_string(),
    );
    m.insert("av-protocol".to_string(), AV_WORKER_PROTOCOL.to_string());
    m.insert("recipe".to_string(), AV_RECIPE.to_string());
    m
}

fn probe_toolchain_axes(exe_sha: &str, family: RecipeFamily) -> Vec<(String, String)> {
    let mut v = vec![
        ("av-protocol".into(), AV_WORKER_PROTOCOL.into()),
        ("recipe".into(), family.recipe().into()),
        ("av-worker-exe-sha256".into(), exe_sha.to_string()),
        (
            "av-worker-version".into(),
            EXPECTED_AV_WORKER_VERSION.into(),
        ),
        (
            "av-core-version".into(),
            family.expected_core_version().into(),
        ),
    ];
    // PLAN-821 contract-decision §10: the owned tool-identity axis carries
    // the frozen profile id (3) explicitly (core/data receipts unchanged).
    if family == RecipeFamily::Owned {
        v.push(("av-profile".into(), "3".into()));
    }
    v
}

fn worker_exe_sha256() -> Result<String, String> {
    worker_file_sha256(&worker_binary()?)
}

fn worker_file_sha256(exe: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    // Re-read the executable on every admission: same-path replacement
    // must change the tool identity. Use the existing accelerated digest
    // implementation rather than the contract crate's debug-mode scalar
    // reference; the SHA-256 bytes and admission policy stay identical.
    let bytes = std::fs::read(exe).map_err(|e| format!("read worker exe: {e}"))?;
    Ok(hex(&Sha256::digest(&bytes)))
}

#[cfg(test)]
mod worker_identity_tests {
    use super::*;

    #[test]
    fn av_worker_sha256_matches_contract_and_same_path_replacement() {
        let path = std::env::temp_dir().join(format!(
            "av798-hash-{}-{}.bin",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        // Empty, known vector, block/padding boundaries and a multi-block
        // artifact all use the SAME path: no metadata/hash cache may hide
        // a replacement, even when its length remains unchanged.
        let mut inputs = vec![Vec::new(), b"abc".to_vec()];
        for len in [55, 56, 63, 64, 65, (1 << 20) + 17] {
            inputs.push((0..len).map(|i| (i % 251) as u8).collect());
        }
        inputs.push(vec![b'x'; 65]);
        inputs.push(vec![b'y'; 65]);
        for bytes in inputs {
            std::fs::write(&path, &bytes).unwrap();
            let observed = worker_file_sha256(&path).unwrap();
            assert_eq!(observed, hex(&sha256(&bytes)));
            if bytes == b"abc" {
                assert_eq!(
                    observed,
                    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
                );
            }
        }
        std::fs::remove_file(&path).unwrap();
        assert!(worker_file_sha256(&path)
            .unwrap_err()
            .starts_with("read worker exe:"));
    }
}

/// Locate (and on first use build) the av worker binary. Env override
/// first (trusted host configuration binds the tool identity);
/// otherwise find `experimental/ac-av-probe` from this crate's source
/// root and build its worker release (bounded).
pub fn worker_binary() -> Result<PathBuf, String> {
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(locate_worker).clone()
}

fn locate_worker() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var("AC_AV_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "AC_AV_CASE_WORKER points at a missing file: {}",
            env_path
        ));
    }
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "av probe worker not located: no experimental/ac-av-probe above the crate and no AC_AV_CASE_WORKER override".to_string(),
            );
        };
        let m = d
            .join("experimental")
            .join("ac-av-probe")
            .join("Cargo.toml");
        if m.is_file() {
            break m;
        }
        dir = d.parent();
    };
    let ws_root = manifest.parent().expect("workspace root");
    let exe = ws_root
        .join("target")
        .join("release")
        .join(av_worker_exe_name());
    // F798-R3-10: ALWAYS build (a no-op incremental build is sub-second)
    // under the BUILD_WALL deadline — testing a stale binary that happens
    // to exist at the target path silently unbinds evidence from source.
    // F798-R15-05: the build runs under the SAME contained-executor
    // discipline as `call_worker`: suspend→Job→resume (dropping the job
    // reaps the whole tree), BOTH output streams drained by capped reader
    // threads (a full stdout pipe must surface as a stream-cap error, not
    // a backpressure wall timeout), ONE absolute deadline, and on breach
    // the TREE is killed and reaped — never a bare child.kill that leaves
    // linker/compiler descendants behind.
    // F798-R15-05 control seams (test-only, mirroring AV798_NATIVE_*):
    // an injectable cargo launcher + wall, so the stdout-cap/timeout/
    // kill-tree failure shapes are testable without a real build.
    let cargo = std::env::var("AV798_BUILD_CARGO").unwrap_or_else(|_| env!("CARGO").to_string());
    let build_wall = Duration::from_millis(
        std::env::var("AV798_BUILD_WALL_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(BUILD_WALL.as_millis() as u64),
    );
    let build_started = Instant::now();
    let mut cmd = Command::new(&cargo);
    cmd.args([
        "build",
        "--release",
        "-p",
        "auto-av-probe",
        "--bin",
        "av_probe_worker",
    ])
    .current_dir(ws_root)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn cargo for the av worker: {e}"))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "containment unavailable: refusing an uncontained av worker build tree".to_string(),
            );
        }
    };
    #[cfg(windows)]
    if let Err(e) = containment::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("resume cargo build: {e}"));
    }
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;
    // Capped reader threads for BOTH streams — a capped-out stream fails
    // the build as a REPORTED stream error, never a silently blocked pipe.
    fn capped_reader<R: std::io::Read + Send + 'static>(
        mut pipe: R,
        tx: std::sync::mpsc::Sender<Result<Vec<u8>, usize>>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let mut chunk = [0u8; 16384];
            let mut exceeded = false;
            loop {
                match pipe.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.len() > STDERR_CAP {
                            exceeded = true;
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            let _ = tx.send(if exceeded { Err(buf.len()) } else { Ok(buf) });
        })
    }
    let (out_tx, out_rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let (err_tx, err_rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let out_reader = capped_reader(child.stdout.take().expect("stdout"), out_tx);
    let err_reader = capped_reader(child.stderr.take().expect("stderr"), err_tx);
    let deadline_at = build_started + build_wall;
    let mut timed_out = false;
    let mut status: Option<std::process::ExitStatus> = None;
    let mut stdout_res: Option<Result<Vec<u8>, usize>> = None;
    let mut stderr_res: Option<Result<Vec<u8>, usize>> = None;
    fn stream_capped(
        out: &Option<Result<Vec<u8>, usize>>,
        err: &Option<Result<Vec<u8>, usize>>,
    ) -> bool {
        matches!(out, Some(Err(_))) || matches!(err, Some(Err(_)))
    }
    loop {
        if Instant::now() > deadline_at {
            timed_out = true;
            break;
        }
        if stdout_res.is_none() {
            stdout_res = out_rx.try_recv().ok();
        }
        if stderr_res.is_none() {
            stderr_res = err_rx.try_recv().ok();
        }
        match child.try_wait() {
            Ok(st @ Some(_)) => {
                status = st;
                break;
            }
            Ok(None) => {}
            Err(e) => return Err(format!("poll cargo build: {e}")),
        }
        if stream_capped(&stdout_res, &stderr_res) {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    if timed_out || stream_capped(&stdout_res, &stderr_res) || status.is_none() {
        // Kill the whole TREE (closing the job reaps every descendant),
        // reap the direct child, then drain the bounded readers.
        let _ = child.kill();
        drop(job.take());
        let _ = child.wait();
        let _ = out_rx.recv_timeout(READER_GRACE);
        let _ = err_rx.recv_timeout(READER_GRACE);
        let _ = out_reader.join();
        let _ = err_reader.join();
        if timed_out {
            return Err("av worker build exceeded the BUILD_WALL budget".to_string());
        }
        if let Some(Err(n)) = &stdout_res {
            return Err(format!("av worker build stdout exceeded the cap ({n})"));
        }
        if let Some(Err(n)) = &stderr_res {
            return Err(format!("av worker build stderr exceeded the cap ({n})"));
        }
        return Err("av worker build ended without an exit status".to_string());
    }
    // Child exited inside the wall: close the job (reaps descendants
    // holding the inherited pipes), then drain both bounded readers.
    drop(job.take());
    let mut log = match out_rx.recv_timeout(READER_GRACE) {
        Ok(Ok(b)) => b,
        Ok(Err(n)) => return Err(format!("av worker build stdout exceeded the cap ({n})")),
        Err(_) => Vec::new(),
    };
    if let Ok(Ok(e)) = err_rx.recv_timeout(READER_GRACE) {
        log.extend_from_slice(&e);
    } else if let Ok(Err(n)) = err_rx.recv_timeout(READER_GRACE) {
        return Err(format!("av worker build stderr exceeded the cap ({n})"));
    }
    let _ = out_reader.join();
    let _ = err_reader.join();
    if !status.expect("set on exit").success() {
        return Err(format!(
            "av worker build failed:\n{}",
            String::from_utf8_lossy(&log)
        ));
    }
    if exe.is_file() {
        Ok(exe)
    } else {
        Err(format!(
            "av worker build produced no binary at {}",
            exe.display()
        ))
    }
}

#[cfg(windows)]
fn av_worker_exe_name() -> &'static str {
    "av_probe_worker.exe"
}
#[cfg(not(windows))]
fn av_worker_exe_name() -> &'static str {
    "av_probe_worker"
}

// ---- typed response mirror ----------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
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

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkerError {
    code: String,
    message: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct OkOut {
    recipe: String,
    steps: Vec<StepOut>,
    worker_version: String,
    av_core_version: String,
}

/// PLAN-820 F820-R6-01: the typed-result report of a `typed:<id>` data
/// run — canonical kind tag + exact raw bits (lowercase hex; i64/u64/f64
/// 16 digits, f32 8, i32/bool unpadded). The parent compares BOTH against
/// its own frozen expectation; the worker's word is never the oracle.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TypedOut {
    kind: String,
    bits: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StepOut {
    op: String,
    outcome: String,
    #[serde(default)]
    value: Option<i32>,
    #[serde(default)]
    typed: Option<TypedOut>,
    #[serde(default)]
    trap: Option<String>,
    #[serde(default)]
    diag: Option<serde_json::Value>,
    #[serde(default)]
    counters: std::collections::BTreeMap<String, u64>,
    #[serde(default)]
    source_hash: String,
    #[serde(default)]
    wall_ms: f64,
    /// mark trace events ("a"/"b" sequence; observation only).
    #[serde(default)]
    trace: Vec<String>,
    /// Stage spans of the run step (name, own-duration ms) — bounded
    /// observability carried into the receipt.
    #[serde(default)]
    spans: Vec<(String, f64)>,
    /// PLAN-821: the owned face of an owned-recipe run step — the
    /// canonical result plus the post-release arena ledger
    /// ({result, live_after_release, allocs:[{id,bytes}], frees:[{id}]}).
    /// An OWNED-recipe increment only; its presence on any other recipe
    /// is an admission violation.
    #[serde(default)]
    owned: Option<serde_json::Value>,
}

/// Spawn the av worker and run ONE `case-serve` request under the bounded
/// executor contract (F798-R3-05/06 — mirrors the sema executor and
/// ac-core's `run_deadlined_captured` shape):
/// - contained start (suspended → job before the first instruction), so
///   the kill-on-close job reaps the whole tree on any exit path;
/// - the request WRITE runs on its own thread (a worker that never reads
///   stdin cannot wedge the parent);
/// - stdout/stderr stream on dedicated reader threads with hard caps —
///   stderr overflow kills the tree immediately, never a silent partial;
/// - ONE absolute deadline covers spawn → write → read → exit → drain;
///   blocking waits only happen with the deadline already decided (the
///   reader-grace bounded joins after a child exit, when the job close has
///   already reaped descendants).
/// After a normal exit the response must be a SUCCESS exit status and
/// EXACTLY one non-empty stdout line — trailing protocol frames and
/// non-zero exits are Infra, never green (F798-R3-05).
fn call_worker(request_json: &str, deadline: Duration) -> Result<WorkerResponse, String> {
    if request_json.len() > REQUEST_CAP {
        return Err(format!(
            "av worker request exceeds the {} byte metadata cap ({} bytes)",
            REQUEST_CAP,
            request_json.len()
        ));
    }
    let exe = worker_binary()?;
    let started = Instant::now();
    let mut cmd = Command::new(&exe);
    cmd.arg("case-serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn av worker ({}): {}", exe.display(), e))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "containment unavailable: refusing an uncontained av worker tree".to_string(),
            );
        }
    };
    #[cfg(windows)]
    if let Err(e) = containment::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("resume av worker: {e}"));
    }
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;

    // Request writer thread: the write can block on a worker that never
    // reads — the parent never does.
    let (write_tx, write_rx) = std::sync::mpsc::channel::<()>();
    {
        let mut stdin = child.stdin.take().expect("stdin");
        let bytes = request_json.as_bytes().to_vec();
        std::thread::spawn(move || {
            let _ = stdin.write_all(&bytes);
            let _ = stdin.flush();
            drop(stdin);
            let _ = write_tx.send(());
        });
    }
    let mut stdout_pipe = child.stdout.take().expect("stdout");
    let mut stderr_pipe = child.stderr.take().expect("stderr");
    let (out_tx, out_rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let (err_tx, err_rx) = std::sync::mpsc::channel::<(bool, Vec<u8>)>();
    let _out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 65536];
        let mut exceeded = false;
        loop {
            match stdout_pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.len() > READ_CAP {
                        exceeded = true;
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = out_tx.send(if exceeded { Err(buf.len()) } else { Ok(buf) });
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

    // Deadline loop: response AND exit, killing on wall or stderr overflow.
    // The stdout reader reads to EOF (not first-newline): the protocol is
    // exactly one line, and TRAILING frames must be visible to the judge.
    let deadline_at = started + deadline;
    let mut received: Option<Result<Vec<u8>, usize>> = None;
    let mut stderr_overflow = false;
    let mut stderr_result: Option<(bool, Vec<u8>)> = None;
    let mut timed_out = false;
    let mut exit_status: Option<std::process::ExitStatus> = None;
    loop {
        if Instant::now() > deadline_at {
            timed_out = true;
            break;
        }
        if stderr_overflow {
            break;
        }
        if received.is_none() {
            match out_rx.try_recv() {
                Ok(v) => received = Some(v),
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => received = Some(Err(0)),
            }
        }
        if stderr_result.is_none() {
            match err_rx.try_recv() {
                Ok(v) => {
                    stderr_overflow = v.0;
                    stderr_result = Some(v);
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
            }
        }
        if exit_status.is_none() {
            match child.try_wait() {
                Ok(st @ Some(_)) => exit_status = st,
                Ok(None) => {}
                Err(e) => return Err(format!("poll av worker: {e}")),
            }
        }
        if exit_status.is_some() {
            // Child gone: closing the job reaps descendants holding the
            // inherited pipes, then bounded reader joins.
            drop(job.take());
            if received.is_none() {
                received = match out_rx.recv_timeout(READER_GRACE) {
                    Ok(v) => Some(v),
                    Err(_) => Some(Err(0)),
                };
            }
            if stderr_result.is_none() {
                stderr_result = err_rx.recv_timeout(READER_GRACE).ok();
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    if timed_out || stderr_overflow {
        let _ = child.kill();
        drop(job.take());
        let _ = child.wait();
        let _ = write_rx.recv_timeout(READER_GRACE);
        let _ = out_rx.recv_timeout(READER_GRACE);
        let _ = err_rx.recv_timeout(READER_GRACE);
        return Err(if timed_out {
            "av worker exceeded the session wall (single absolute deadline over spawn/write/read/exit/drain)".to_string()
        } else {
            format!(
                "av worker stderr exceeded the {}-byte cap — tree killed",
                STDERR_CAP
            )
        });
    }
    // Normal exit: job already dropped (descendants reaped).
    drop(job.take());
    if write_rx.recv_timeout(READER_GRACE).is_err() {
        return Err("av worker request writer did not finish within the drain grace".into());
    }
    let out_buf = match received {
        Some(Ok(b)) => b,
        Some(Err(n)) => {
            return Err(format!(
                "av worker response exceeded the {}-byte read cap (saw {})",
                READ_CAP, n
            ))
        }
        None => match out_rx.recv_timeout(READER_GRACE) {
            Ok(Ok(b)) => b,
            Ok(Err(n)) => return Err(format!("av worker response exceeded the read cap ({n})")),
            Err(_) => {
                return Err("av worker stdout reader did not finish within the drain grace".into())
            }
        },
    };
    let err_buf = match stderr_result {
        Some((_, b)) => b,
        None => err_rx
            .recv_timeout(READER_GRACE)
            .map(|(_, b)| b)
            .unwrap_or_default(),
    };
    // F798-R3-05: process identity — a non-zero exit is Infra even when a
    // plausible JSON line preceded it.
    let status = exit_status.unwrap_or_else(|| child.wait().unwrap_or_default());
    if !status.success() {
        return Err(format!(
            "av worker exited with {:?} (non-zero worker exits are infra, never a language verdict); stderr: {:.300}",
            status.code(),
            String::from_utf8_lossy(&err_buf)
        ));
    }
    // F798-R3-05: protocol completeness — exactly ONE non-empty line; any
    // trailing frame (or zero lines) is Infra.
    let text = String::from_utf8_lossy(&out_buf);
    let non_empty: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.len() != 1 {
        return Err(format!(
            "av worker response must be exactly one non-empty protocol line, saw {} ({:.200})",
            non_empty.len(),
            text
        ));
    }
    serde_json::from_str(non_empty[0]).map_err(|e| {
        format!(
            "bad av worker response ({e}): {:.200} | stderr: {:.300}",
            non_empty[0],
            String::from_utf8_lossy(&err_buf)
        )
    })
}

// ---- parent-side judgement ----------------------------------------------

/// Map the av pipeline's diagnostic stage onto the contract's language
/// stages: front-door/frontend rejections are parse-stage; MIR/ABC/
/// closure admission rejections are verify-stage.
fn map_diag_stage(stage: &str) -> Stage {
    match stage {
        "admission" | "frontend" => Stage::Parse,
        _ => Stage::Verify,
    }
}

pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Core,
    )
}

/// PLAN-820: the data-recipe entry — same executor/judgement contract,
/// data-profile pipeline (record source front door in the worker).
/// PLAN-820 F820-R6-01: the PARENT's frozen av.data-fixture/1 expectation
/// table — an INDEPENDENT copy of the worker-side fixture truths (tag +
/// exact raw bits). The typed judge compares against THIS, never against
/// anything the worker reports as expected.
const DATA_TYPED_FIXTURES: &[(&str, &str, &str)] = &[
    ("u64-max", "u64", "ffffffffffffffff"),
    ("i64-min", "i64", "8000000000000000"),
    ("fneg-nan", "f64", "fff80000deadbeef"),
    ("neg-zero", "f64", "8000000000000000"),
    ("f32-one", "f32", "3f800000"),
    ("f32-nan-payload", "f32", "7f80ad00"),
    ("bool-true", "bool", "1"),
    ("i32-42", "i32", "2a"),
];

pub fn run_data_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Data,
    )
}

/// PLAN-821 T-02: the PARENT's frozen av.owned-fixture/1 expectation
/// table — an INDEPENDENT hand-audited copy of the frozen fixture truths
/// (contract-decision §11): the canonical result face, the owned
/// allocation byte sizes in CREATION order, and the free ids in RELEASE
/// order (live must reach 0 after the host-side release). The owned judge
/// compares against THIS, never against anything the worker reports as
/// expected.
const OWNED_CASES: &[(&str, &str, &[usize], &[u64])] = &[
    // first-case: Note { body: clone of "Auto", n = len("Auto") = 4 }.
    // Hand-audited: two 4-byte buffers (ctor literal + clone), the ctor
    // buffer frees at the field replace, the clone frees at the host
    // release (creation-order LIFO); live=0 after the release.
    (
        "first-case",
        r#"{"fields":[{"kind":"str","utf8":"Auto"},{"kind":"i32","bits":"4"}],"kind":"record","name":"t_rec_Note"}"#,
        &[4, 4],
        &[0, 1],
    ),
];

pub fn run_owned_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    run_check_impl(
        case,
        check,
        oracle,
        project_dir,
        artifact_dir,
        run_dir,
        RecipeFamily::Owned,
    )
}

fn run_check_impl(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
    family: RecipeFamily,
) -> Result<Verdict, String> {
    let started = Instant::now();
    let recipe = family.recipe();
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Err(format!(
            "{recipe} consumes .at sources; syntax {:?} is not accepted by this recipe",
            case.spec.source.syntax.as_str()
        ));
    }
    let av = check
        .options
        .probe_av
        .as_ref()
        .ok_or("probe check without an av binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if check.options.probe_recipe.as_deref() != Some(recipe) {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    // F798-R3-07 shape: leading edit-source allowed (measured window);
    // the judged last step must be a run.
    if !matches!(steps.last(), Some(auto_test_case::contract::ProbeStep::Run)) {
        return Err(format!(
            "{recipe} step sequences must end with a run step (contract violation)"
        ));
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    // Store root (F798-R3-07): with an explicit SESSION id the store lives
    // under a deterministic shared temp root keyed by that id, so calls of
    // the same session (warm-up + measured edit) share one warm store
    // across worker processes; the PARENT owns cleanup. Without a session
    // the check's own artifact dir hosts a private per-request store.
    let store_root = match &av.session {
        Some(s) => std::env::temp_dir().join(format!("av798-session-{s}")),
        None => artifact_dir.join("store"),
    };
    std::fs::create_dir_all(&store_root).map_err(|e| format!("probe store root: {}", e))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("probe artifact dir: {}", e))?;

    let nonce = hex(&sha256(
        format!("{}|{:?}|av", std::process::id(), started).as_bytes(),
    ));
    let steps_json: Vec<serde_json::Value> = steps
        .iter()
        .map(|st| match st {
            auto_test_case::contract::ProbeStep::Run => {
                serde_json::json!({ "op": "run" })
            }
            auto_test_case::contract::ProbeStep::EditSource { from, to } => {
                serde_json::json!({ "op": "edit-source", "from": from, "to": to })
            }
            _ => serde_json::json!({ "op": "run" }),
        })
        .collect();
    // F820-R6-01: a `typed:<id>` entry (data recipe only) selects a frozen
    // av.data-fixture/1 input; the PARENT validates the id against its OWN
    // frozen table before any request — an unregistered fixture is a check
    // error, never a green run.
    let typed_fixture = if family == RecipeFamily::Data {
        match av.entry.strip_prefix("typed:") {
            None => None,
            Some(id) => match DATA_TYPED_FIXTURES.iter().find(|(fid, _, _)| *fid == id) {
                Some(_) => Some(id.to_string()),
                None => {
                    return Err(format!(
                        "av.data typed fixture `{id}` is not registered (parent table)"
                    ))
                }
            },
        }
    } else {
        None
    };
    // PLAN-821: an `owned:<id>` entry (owned recipe only) selects a frozen
    // av.owned-fixture/1 input; the PARENT validates the id against its OWN
    // frozen table before any request — an unregistered fixture is a check
    // error, never a green run.
    let owned_fixture = if family == RecipeFamily::Owned {
        match av.entry.strip_prefix("owned:") {
            None => None,
            Some(id) => match OWNED_CASES.iter().find(|(fid, ..)| *fid == id) {
                Some(_) => Some(id.to_string()),
                None => {
                    return Err(format!(
                        "av.owned fixture `{id}` is not registered (parent table)"
                    ))
                }
            },
        }
    } else {
        None
    };
    let mut request = serde_json::json!({
        "recipe": recipe,
        "nonce": nonce,
        "source_b64": b64(&input_bytes),
        "entry": av.entry,
        "caps": av.caps,
        "store_root": store_root.display().to_string().replace('/', "\\"),
        "steps": steps_json,
    });
    if let Some(id) = &typed_fixture {
        request["typed_fixture"] = serde_json::json!(id);
    }
    if let Some(id) = &owned_fixture {
        request["owned_fixture"] = serde_json::json!(id);
    }
    if let Some(session) = &av.session {
        request["session"] = serde_json::json!(session);
    }
    let request_hash = hex(&sha256(request.to_string().as_bytes()));
    request["request_hash"] = serde_json::json!(request_hash);

    let worker_started = Instant::now();
    // Session wall (test seam: AV798_SESSION_WALL_MS shortens the absolute
    // deadline for the delay counter-control; default = the frozen 60s).
    let wall = std::env::var("AV798_SESSION_WALL_MS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or(SESSION_WALL);
    let resp_result = call_worker(&request.to_string(), wall);
    let worker_us = worker_started.elapsed().as_micros() as u64;

    let mut mismatches: Vec<Mismatch> = Vec::new();
    let mut infra_detail: Option<String> = None;
    let mut step_out: Option<StepOut> = None;
    let exe_sha = worker_exe_sha256()?;

    'admit: {
        let resp = match resp_result {
            Ok(r) => r,
            Err(e) => {
                infra_detail = Some(e);
                break 'admit;
            }
        };
        if resp.schema != AV_WORKER_PROTOCOL {
            infra_detail = Some(format!("av worker schema mismatch: {}", resp.schema));
            break 'admit;
        }
        if resp.nonce != nonce || resp.request_hash != request_hash {
            infra_detail = Some("av worker nonce/request_hash echo mismatch".to_string());
            break 'admit;
        }
        let ok = match resp.ok.clone() {
            Some(ok) => ok,
            None => {
                let e = resp.error.unwrap_or(WorkerError {
                    code: "missing".into(),
                    message: "neither ok nor error".into(),
                });
                infra_detail = Some(format!("av worker error {}: {}", e.code, e.message));
                break 'admit;
            }
        };
        if ok.recipe != recipe {
            infra_detail = Some(format!("av worker recipe mismatch: {}", ok.recipe));
            break 'admit;
        }
        // F820-R7-01: the typed report is a DATA-recipe protocol
        // increment — its presence on any non-data (core/owned) response
        // is an admission violation, never a pass-shaped extra channel.
        if typed_on_non_data(family, &ok.steps) {
            infra_detail =
                Some("typed report on a non-data recipe (protocol violation)".to_string());
            break 'admit;
        }
        // F820-R8-01: data typed shape is an ORACLE-INDEPENDENT admission
        // rule — no exit/value oracle match can bypass it.
        if let Some(v) = data_malformed_typed(&ok.steps) {
            infra_detail = Some(v);
            break 'admit;
        }
        // PLAN-821: the owned face is an OWNED-recipe increment — its
        // presence on any non-owned response is an admission violation,
        // and every owned report is SHAPE-validated here (oracle-independent:
        // {result: object, live_after_release: u64, allocs: [{id,bytes}],
        // frees: [{id}]}), so no oracle choice can let a malformed ledger
        // face through.
        if owned_on_non_owned(family, &ok.steps) {
            infra_detail =
                Some("owned report on a non-owned recipe (protocol violation)".to_string());
            break 'admit;
        }
        if let Some(v) = malformed_owned(&ok.steps) {
            infra_detail = Some(v);
            break 'admit;
        }
        if ok.worker_version != EXPECTED_AV_WORKER_VERSION {
            infra_detail = Some(format!(
                "av worker version `{}` does not match the admitted `{}`",
                ok.worker_version, EXPECTED_AV_WORKER_VERSION
            ));
            break 'admit;
        }
        if ok.av_core_version != family.expected_core_version() {
            infra_detail = Some(format!(
                "av core version `{}` does not match the admitted `{}`",
                ok.av_core_version,
                family.expected_core_version()
            ));
            break 'admit;
        }
        let last_run = ok.steps.iter().rev().find(|s| s.op == "run").cloned();
        match last_run {
            Some(s) => step_out = Some(s),
            _ => {
                infra_detail = Some("av worker returned no run step".to_string());
                break 'admit;
            }
        }
        // F798-R3-05: the response is bound to THIS request and revision —
        // step count, op echo, edit outcomes, and every run step's
        // source_hash re-derived parent-side over the request's own source
        // state (a relay echoing a valid envelope over somebody else's or a
        // stale revision's result is Infra, never a green judgement).
        if ok.steps.len() != steps.len() {
            infra_detail = Some(format!(
                "av worker returned {} steps for a {}-step request",
                ok.steps.len(),
                steps.len()
            ));
            break 'admit;
        }
        let mut current_source = String::from_utf8_lossy(&input_bytes).to_string();
        for (i, (want, got)) in steps.iter().zip(ok.steps.iter()).enumerate() {
            let want_op = match want {
                auto_test_case::contract::ProbeStep::Run => "run",
                auto_test_case::contract::ProbeStep::EditSource { from, to } => {
                    let hits = current_source.matches(from.as_str()).count();
                    if hits != 1 {
                        infra_detail = Some(format!(
                            "step {i}: parent-side edit literal matched {hits} times"
                        ));
                        break 'admit;
                    }
                    current_source = current_source.replacen(from.as_str(), to.as_str(), 1);
                    if got.outcome != "edited" {
                        infra_detail = Some(format!(
                            "step {i}: edit-source outcome `{}` is not `edited`",
                            got.outcome
                        ));
                        break 'admit;
                    }
                    "edit-source"
                }
                _ => {
                    infra_detail = Some(format!("step {i}: unsupported probe step in av recipe"));
                    break 'admit;
                }
            };
            if got.op != want_op {
                infra_detail = Some(format!(
                    "step {i}: op `{}` does not echo the requested `{}`",
                    got.op, want_op
                ));
                break 'admit;
            }
            if want_op == "run" {
                // A typed/owned fixture run's revision identity is the
                // FIXTURE ENTRY (the frozen input that actually ran); a
                // source run binds the current source revision
                // (F820-R6-01; PLAN-821 same rule for owned).
                let expect_hash = if typed_fixture.is_some() || owned_fixture.is_some() {
                    fnv1a64_hex(av.entry.as_bytes())
                } else {
                    fnv1a64_hex(current_source.as_bytes())
                };
                if got.source_hash != expect_hash {
                    infra_detail = Some(format!(
                        "step {i}: source_hash `{}` does not bind the request's current revision ({})",
                        got.source_hash, expect_hash
                    ));
                    break 'admit;
                }
            }
        }
    }

    if let Some(e) = infra_detail {
        // Infrastructure lifecycle failure — an ERROR outcome, never a
        // language verdict.
        return Ok(Verdict {
            status: Status::Error,
            mismatches: vec![Mismatch {
                observation: "worker".into(),
                kind: "infra".into(),
                expected: "a contained worker session".into(),
                actual: e,
                first_diff: None,
            }],
            detail: Some(format!("{recipe}: infra failure")),
            artifacts: vec![],
            native: None,
        });
    }

    let step = step_out.expect("admitted");
    if let Some(id) = &typed_fixture {
        // The typed judge face: tag + exact raw bits against the parent's
        // frozen expectation — independent of `value` (which cannot carry
        // wide/bool results) and of anything the worker claims.
        let (_, want_kind, want_bits) = DATA_TYPED_FIXTURES
            .iter()
            .find(|(fid, _, _)| fid == id)
            .expect("validated above");
        // Counter-control seam (V-10 wrong-bits / wrong-type-tag): the
        // test-only sabotage mutates the PARENT's expectation copy so a
        // real run of a legal fixture must FAIL. Never set in production.
        let (want_kind, want_bits) =
            match std::env::var("AV820_TYPED_EXPECT_SABOTAGE").ok().as_deref() {
                Some("kind") => (
                    if *want_kind == "i64" { "u64" } else { "i64" },
                    want_bits.to_string(),
                ),
                Some("bits") => {
                    let mut b = want_bits.to_string();
                    let last = b.pop().expect("non-empty bits");
                    b.push(if last == '0' { '1' } else { '0' });
                    (*want_kind, b)
                }
                _ => (*want_kind, want_bits.to_string()),
            };
        if step.outcome != "success" {
            mismatches.push(Mismatch {
                observation: "completion".into(),
                kind: "typed-outcome".into(),
                expected: "success".into(),
                actual: step_summary(&step),
                first_diff: None,
            });
        }
        match &step.typed {
            None => mismatches.push(Mismatch {
                observation: "typed".into(),
                kind: "typed-report".into(),
                expected: format!("{want_kind}:{want_bits}"),
                actual: "no typed result on the step".into(),
                first_diff: None,
            }),
            Some(t) => {
                if t.kind != want_kind {
                    mismatches.push(Mismatch {
                        observation: "typed".into(),
                        kind: "typed-tag".into(),
                        expected: want_kind.to_string(),
                        actual: t.kind.clone(),
                        first_diff: None,
                    });
                }
                if t.bits != want_bits {
                    mismatches.push(Mismatch {
                        observation: "typed".into(),
                        kind: "typed-bits".into(),
                        expected: want_bits.clone(),
                        actual: t.bits.clone(),
                        first_diff: None,
                    });
                }
            }
        }
    } else if let Some(id) = &owned_fixture {
        // The owned-fixture judge face: canonical result + live=0 +
        // alloc/free order against the parent's frozen table — fully
        // independent of `value` and of anything the worker claims.
        judge_owned_fixture(id, &step, &mut mismatches);
    } else {
        judge_expectation(family, &step, oracle, &mut mismatches);
    }

    // Receipt (probe-receipt.json in the artifact dir). `request_wall_ms`
    // is the continuous parent-side window from request build to judged
    // response — spawn, request write, worker session, response read and
    // admission all inside (F798-R3-07: the formal turnaround gate
    // measures THIS, never a subtraction of parent call time).
    let request_wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let receipt = serde_json::json!({
        "recipe": recipe,
        "protocol": AV_WORKER_PROTOCOL,
        "nonce": nonce,
        "request_hash": request_hash,
        "step": {
            "op": step.op,
            "outcome": step.outcome,
            "value": step.value,
            "trap": step.trap,
            "diag": step.diag,
            "owned": step.owned,
            "counters": step.counters,
            "source_hash": step.source_hash,
            "wall_ms": step.wall_ms,
            "spans": step.spans,
            "trace": step.trace,
        },
        "worker_us": worker_us,
        "request_wall_ms": request_wall_ms,
        "toolchain": probe_toolchain_axes(&exe_sha, family),
    });
    std::fs::write(
        artifact_dir.join("probe-receipt.json"),
        serde_json::to_string_pretty(&receipt).unwrap_or_default(),
    )
    .map_err(|e| format!("write receipt: {e}"))?;

    let native = NativeRecord {
        exit_code: match step.outcome.as_str() {
            "success" => step.value,
            "trap" => Some(70),
            _ => None,
        },
        stage_spans: vec![
            ("worker_us".to_string(), worker_us as u64),
            ("wall_ms".to_string(), step.wall_ms as u64),
            ("request_wall_ms".to_string(), request_wall_ms as u64),
        ],
        cache_mode: None,
        cache_hits: vec![],
        counters: step
            .counters
            .iter()
            .map(|(k, v)| (k.clone(), *v as u32))
            .collect(),
        toolchain: probe_toolchain_axes(&exe_sha, family),
        gaps: vec![],
    };
    let status = if mismatches.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    Ok(Verdict {
        status,
        mismatches,
        detail: Some(format!(
            "{recipe}: run step judged (outcome {}); receipt probe-receipt.json",
            step.outcome
        )),
        artifacts: vec![],
        native: Some(native),
    })
}

/// Judge the observed step outcome against the declared expectations.
/// The oracle faces: success+exit (value), error-run+message (trap),
/// error-parse/verify+code (diagnosed rejection).
/// Canonical typed-shape validation (F820-R7-01): a data-family typed
/// report must name one of the six scalar tags and carry the kind's
/// canonical lowercase-hex width — malformed reports are mismatches, not
/// silent passes. Returns the violation message, if any.
fn typed_shape_violation(t: &TypedOut) -> Option<String> {
    let width_ok = match t.kind.as_str() {
        "i64" | "u64" | "f64" => t.bits.len() == 16,
        "f32" => t.bits.len() == 8,
        // Canonical UNPADDED hex (contract-decision §3: bool/i32 不补零) —
        // a leading zero is a non-canonical encoding, and bool is exactly
        // the two value digits 0/1 (F820-R8-01: `2`/`00` must not pass).
        // The single digit `0` is the UNIQUE canonical encoding of ZERO
        // (F820-R9-01): forbidding padding must not forbid the zero value.
        "i32" => t.bits == "0" || ((1..=8).contains(&t.bits.len()) && !t.bits.starts_with('0')),
        "bool" => t.bits == "0" || t.bits == "1",
        other => return Some(format!("unknown typed kind `{other}`")),
    };
    if !width_ok
        || !t
            .bits
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Some(format!(
            "typed bits `{}` are not canonical for kind `{}`",
            t.bits, t.kind
        ));
    }
    None
}

/// Admission rule (F820-R7-01): any typed report on a non-data recipe.
fn typed_on_non_data(family: RecipeFamily, steps: &[StepOut]) -> bool {
    family != RecipeFamily::Data && steps.iter().any(|s| s.typed.is_some())
}

/// Admission rule (PLAN-821): any owned report on a non-owned recipe.
fn owned_on_non_owned(family: RecipeFamily, steps: &[StepOut]) -> bool {
    family != RecipeFamily::Owned && steps.iter().any(|s| s.owned.is_some())
}

/// Admission rule (PLAN-821): EVERY owned report on an owned response is
/// shape-validated HERE — before and independent of any oracle branch.
/// The face must carry {result: <object>, live_after_release: <u64>,
/// allocs: [{id: u64, bytes: u64}], frees: [{id: u64}]}.
fn malformed_owned(steps: &[StepOut]) -> Option<String> {
    steps.iter().find_map(|s| {
        s.owned.as_ref().and_then(|o| {
            let bad = |why: &str| {
                Some(format!(
                    "malformed owned report on step `{}`: {}",
                    s.op, why
                ))
            };
            // Every structural defect is an EXPLICIT rejection — a
            // missing key must never `?`-skip the validator (the merge
            // gate caught an alloc entry without `bytes` slipping
            // through as "not malformed").
            let Some(o) = o.as_object() else {
                return bad("owned face is not an object");
            };
            match o.get("result") {
                Some(v) if v.is_object() => {}
                Some(_) => return bad("result is not an object"),
                None => return bad("missing result"),
            }
            match o.get("live_after_release") {
                Some(v) if v.is_u64() => {}
                Some(_) => return bad("live_after_release is not a u64"),
                None => return bad("missing live_after_release"),
            }
            for key in ["allocs", "frees"] {
                let arr = match o.get(key) {
                    Some(v) => match v.as_array() {
                        Some(a) => a,
                        None => return bad(&format!("{key} is not an array")),
                    },
                    None => return bad(&format!("missing {key}")),
                };
                if arr.len() > 65536 {
                    return bad(&format!("{key} exceeds the table-entry family cap"));
                }
                for ev in arr {
                    let Some(e) = ev.as_object() else {
                        return bad(&format!("{key} entry is not an object"));
                    };
                    match e.get("id") {
                        Some(v) if v.is_u64() => {}
                        Some(_) => return bad(&format!("{key} entry id is not a u64")),
                        None => return bad(&format!("{key} entry missing id")),
                    }
                    if key == "allocs" {
                        match e.get("bytes") {
                            Some(v) if v.is_u64() => {}
                            Some(_) => return bad("allocs entry bytes is not a u64"),
                            None => return bad("allocs entry missing bytes"),
                        }
                    }
                }
            }
            None
        })
    })
}

/// The owned-fixture judge: canonical result face (structural equality),
/// live=0 after the host release, allocation byte sizes in creation order
/// and free ids in release order — all against the PARENT's frozen table.
/// Counter-control seam (V-10 wrong-result / wrong-ledger): the test-only
/// sabotage mutates the PARENT's expectation copy so a legal fixture run
/// must FAIL. Never set in production.
fn judge_owned_fixture(id: &str, step: &StepOut, mismatches: &mut Vec<Mismatch>) {
    let (_, want_result, want_allocs, want_frees) = OWNED_CASES
        .iter()
        .find(|(fid, ..)| *fid == id)
        .expect("validated above");
    let mut want_result: serde_json::Value =
        serde_json::from_str(want_result).expect("frozen table JSON parses");
    let mut want_frees: Vec<u64> = want_frees.to_vec();
    match std::env::var("AV821_OWNED_EXPECT_SABOTAGE").ok().as_deref() {
        Some("result") => {
            // Flip one observable leaf: the record's first str field bytes.
            want_result["fields"][0]["utf8"] = serde_json::json!(format!(
                "{}!",
                want_result["fields"][0]["utf8"].as_str().unwrap_or("")
            ));
        }
        Some("ledger") => {
            want_frees.rotate_left(1);
        }
        _ => {}
    }
    if step.outcome != "success" {
        mismatches.push(Mismatch {
            observation: "completion".into(),
            kind: "owned-outcome".into(),
            expected: "success".into(),
            actual: step_summary(step),
            first_diff: None,
        });
        return;
    }
    let owned = match &step.owned {
        Some(o) => o,
        None => {
            mismatches.push(Mismatch {
                observation: "owned".into(),
                kind: "owned-report".into(),
                expected: "the owned face on the step".into(),
                actual: "no owned report".into(),
                first_diff: None,
            });
            return;
        }
    };
    let got_result = owned
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    if got_result != want_result {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-result".into(),
            expected: want_result.to_string(),
            actual: got_result.to_string(),
            first_diff: None,
        });
    }
    let live = owned
        .get("live_after_release")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::MAX);
    if live != 0 {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-live".into(),
            expected: "0 (host release empties the ledger)".into(),
            actual: format!("{live}"),
            first_diff: None,
        });
    }
    let alloc_bytes: Vec<u64> = owned
        .get("allocs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("bytes").and_then(|b| b.as_u64()))
                .collect()
        })
        .unwrap_or_default();
    let want_allocs: Vec<u64> = want_allocs.iter().map(|b| *b as u64).collect();
    if alloc_bytes != want_allocs {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-allocs".into(),
            expected: format!("{want_allocs:?}"),
            actual: format!("{alloc_bytes:?}"),
            first_diff: None,
        });
    }
    // Creation order = strictly ascending ids (the ledger's monotonic
    // allocation identity).
    let alloc_ids: Vec<u64> = owned
        .get("allocs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("id").and_then(|b| b.as_u64()))
                .collect()
        })
        .unwrap_or_default();
    if !alloc_ids.windows(2).all(|w| w[0] < w[1]) {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-alloc-order".into(),
            expected: "strictly ascending allocation ids".into(),
            actual: format!("{alloc_ids:?}"),
            first_diff: None,
        });
    }
    let free_ids: Vec<u64> = owned
        .get("frees")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("id").and_then(|b| b.as_u64()))
                .collect()
        })
        .unwrap_or_default();
    if free_ids != want_frees {
        mismatches.push(Mismatch {
            observation: "owned".into(),
            kind: "owned-frees".into(),
            expected: format!("{want_frees:?}"),
            actual: format!("{free_ids:?}"),
            first_diff: None,
        });
    }
}

/// Admission rule (F820-R8-01): EVERY typed report on a data response is
/// shape-validated HERE — before and independent of any oracle branch, so
/// a matching exit expectation (or any other oracle choice) can never let
/// an illegal bool encoding or an unknown kind tag through.
fn data_malformed_typed(steps: &[StepOut]) -> Option<String> {
    steps.iter().find_map(|s| {
        s.typed
            .as_ref()
            .and_then(typed_shape_violation)
            .map(|v| format!("malformed typed report on step `{}`: {}", s.op, v))
    })
}

fn judge_expectation(
    family: RecipeFamily,
    step: &StepOut,
    oracle: &ResolvedOracle,
    mismatches: &mut Vec<Mismatch>,
) {
    let expect = oracle;
    match &expect.completion {
        Completion::Success => {
            if step.outcome != "success" {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: "success".into(),
                    actual: format!("{} ({})", step.outcome, step_summary(step)),
                    first_diff: None,
                });
                return;
            }
            if let Some(want) = expect.exit {
                if step.value != Some(want) {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "exit".into(),
                        expected: format!("{want}"),
                        actual: format!("{:?}", step.value),
                        first_diff: None,
                    });
                }
            } else if expect.exit.is_none() && expect.stdout.is_none() && expect.stderr.is_none() {
                // Success without a value oracle still requires a value
                // present — FAMILY-ISOLATED (F820-R7-01): the typed
                // channel is a DATA-recipe increment only. A core/i32
                // success must carry the OLD value channel (a typed
                // report on a core response is an admission violation,
                // checked there); a data success may satisfy presence
                // via either channel, and a data typed report must have
                // the canonical tag/width shape. PLAN-821: an owned
                // success satisfies presence via the owned face (the
                // record/Str results have no i32 form).
                let present = step.value.is_some()
                    || (family == RecipeFamily::Data && step.typed.is_some())
                    || (family == RecipeFamily::Owned && step.owned.is_some());
                if !present {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "missing".into(),
                        expected: "an AV success value".into(),
                        actual: "none".into(),
                        first_diff: None,
                    });
                }
                // (Typed SHAPE is an admission rule — see
                // `data_malformed_typed`; nothing here re-validates it.)
            }
        }
        Completion::Error {
            stage,
            code,
            message,
        } => match step.outcome.as_str() {
            "trap" => {
                if *stage != Stage::Run {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{:?}", stage),
                        actual: "run (trap)".into(),
                        first_diff: None,
                    });
                }
                if let Some(want_code) = code {
                    if step.trap.as_deref() != Some(want_code.as_str()) {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want_code.clone(),
                            actual: step.trap.clone().unwrap_or_default(),
                            first_diff: None,
                        });
                    }
                }
                if let Some(matcher) = message {
                    let text = step.trap.clone().unwrap_or_default();
                    if !match_message(matcher, &text) {
                        mismatches.push(Mismatch {
                            observation: "message".into(),
                            kind: "message".into(),
                            expected: format!("{matcher:?}"),
                            actual: text,
                            first_diff: None,
                        });
                    }
                }
            }
            "diag" => {
                let diag = step.diag.clone().unwrap_or_default();
                let got_stage = diag["stage"].as_str().unwrap_or_default();
                let want_stage = map_diag_stage(got_stage);
                if want_stage != *stage {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{:?}", stage),
                        actual: format!("{got_stage} (→ {want_stage:?})"),
                        first_diff: None,
                    });
                }
                let got_code = diag["code"].as_str().unwrap_or_default().to_string();
                if let Some(want_code) = code {
                    if got_code != *want_code {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want_code.clone(),
                            actual: got_code,
                            first_diff: None,
                        });
                    }
                }
                if let Some(matcher) = message {
                    let text = diag["message"].as_str().unwrap_or_default().to_string();
                    if !match_message(matcher, &text) {
                        mismatches.push(Mismatch {
                            observation: "message".into(),
                            kind: "message".into(),
                            expected: format!("{matcher:?}"),
                            actual: text,
                            first_diff: None,
                        });
                    }
                }
            }
            other => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: "an error completion".into(),
                    actual: format!("{other} ({})", step_summary(step)),
                    first_diff: None,
                });
            }
        },
    }
}

fn match_message(matcher: &MessageMatcher, text: &str) -> bool {
    use MessageMatcher::*;
    match matcher {
        Exact(w) => text == w,
        Contains(w) => text.contains(w.as_str()),
    }
}

fn step_summary(step: &StepOut) -> String {
    match step.outcome.as_str() {
        "trap" => format!("trap {}", step.trap.clone().unwrap_or_default()),
        "diag" => format!(
            "diag {}",
            step.diag
                .as_ref()
                .and_then(|d| d.get("code"))
                .and_then(|c| c.as_str())
                .unwrap_or("?")
        ),
        other => other.to_string(),
    }
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// FNV-1a 64 over the request's source bytes, hex-16 — the SAME
/// fingerprint the worker's driver stamps per run step, re-derived
/// parent-side to bind every reported result to the current revision
/// (F798-R3-05).
fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// Test-only seam for the prep-executor controls (F798-R15-05).
/// tests_seam 可见性（815 T-04/814 T-04 两计划同款修复）：消费测试
/// （auto-lang 侧 ac_av/probe_adapter 探针族）在独立 crate——pub(crate)
/// 在 M814-A 目录迁移后跨 crate 不可达（f6e07acee 起 latent 断点）。
/// 缝整体在 test-cases feature 门内（814 交付语义：测试基建面），
/// pub 化零生产面变化，仅测试定位器。
pub mod tests_seam {
    pub fn locate_worker_for_test() -> Result<std::path::PathBuf, String> {
        super::locate_worker()
    }
}

#[cfg(test)]
mod judge_tests {
    //! F820-R7-01 pins: the typed channel is a DATA-recipe increment —
    //! the shared judge and admission rules are exercised HERE, on the
    //! production functions, with synthetic steps (the reviewer's
    //! fault-worker e2e shapes: core success missing the i32 value, with
    //! and without an attached typed object).

    use super::*;

    fn step(value: Option<i32>, typed: Option<TypedOut>) -> StepOut {
        StepOut {
            op: "run".into(),
            outcome: "success".into(),
            value,
            typed,
            trap: None,
            diag: None,
            counters: Default::default(),
            source_hash: String::new(),
            wall_ms: 0.0,
            trace: Vec::new(),
            spans: Vec::new(),
            owned: None,
        }
    }

    fn typed(kind: &str, bits: &str) -> Option<TypedOut> {
        Some(TypedOut {
            kind: kind.into(),
            bits: bits.into(),
        })
    }

    fn oracle_no_exit() -> ResolvedOracle {
        ResolvedOracle {
            completion: auto_test_case::contract::Completion::Success,
            result: None,
            stdout: None,
            stderr: None,
            artifacts: Vec::new(),
            exit: None,
            perf: None,
            probe: None,
            oracle_sha256: String::new(),
        }
    }

    fn mismatches(family: RecipeFamily, step: &StepOut) -> Vec<Mismatch> {
        let oracle = oracle_no_exit();
        let mut ms = Vec::new();
        judge_expectation(family, step, &oracle, &mut ms);
        ms
    }

    #[test]
    fn core_success_requires_the_old_value_channel() {
        // Without typed: the pre-R6 semantics — a missing value mismatches.
        let ms = mismatches(RecipeFamily::Core, &step(None, None));
        assert!(ms.iter().any(|m| m.kind == "missing"));
        // With a typed object attached (the R7 fault-worker shape): the
        // typed channel must NOT satisfy a core success — still missing.
        let ms = mismatches(RecipeFamily::Core, &step(None, typed("bool", "1")));
        assert!(
            ms.iter().any(|m| m.kind == "missing"),
            "typed must not satisfy a core value presence: {ms:?}"
        );
        // And the admission gate rejects the typed-on-core shape outright
        // (infra, never a pass-shaped extra channel).
        let steps = vec![step(None, typed("bool", "1"))];
        assert!(super::typed_on_non_data(RecipeFamily::Core, &steps));
        // A clean core response carries no typed at all.
        assert!(!super::typed_on_non_data(
            RecipeFamily::Core,
            &[step(Some(42), None)]
        ));
        // Positive: value present, no typed — no presence mismatch.
        assert!(mismatches(RecipeFamily::Core, &step(Some(42), None)).is_empty());
    }

    #[test]
    fn data_success_presence_accepts_the_typed_channel() {
        // bool has no i32 form: presence via typed (the F820-R6-01 fix,
        // now family-isolated to data).
        assert!(mismatches(RecipeFamily::Data, &step(None, typed("bool", "1"))).is_empty());
        assert!(mismatches(RecipeFamily::Data, &step(Some(42), None)).is_empty());
        assert!(mismatches(RecipeFamily::Data, &step(None, None))
            .iter()
            .any(|m| m.kind == "missing"));
    }

    #[test]
    fn data_typed_reports_must_have_the_canonical_shape() {
        // The shape rule is an ORACLE-INDEPENDENT admission check
        // (`data_malformed_typed` takes only the steps): the R8 fault
        // table — illegal bool encodings, unknown tags — is red even
        // when a value is present and an exit oracle would match.
        for (kind, bits) in [
            ("bool", "2"),               // bool is exactly 0/1 (contract §3)
            ("bool", "00"),              // non-canonical padded encoding
            ("i32", "02a"),              // leading zero = non-canonical
            ("weird", "1"),              // unknown kind tag
            ("f32", "3f80"),             // wrong fixed width
            ("u64", "zzzzzzzzzzzzzzzz"), // non-hex
        ] {
            assert!(
                typed_shape_violation(&typed(kind, bits).unwrap()).is_some(),
                "{kind}/{bits} must be rejected"
            );
            // ... and the admission gate reports it with a VALUE present
            // (the exit-oracle bypass shape from R8: value=42 + matching
            // exit would have passed the judge — admission still reddens).
            let steps = [step(Some(42), typed(kind, bits))];
            assert!(
                data_malformed_typed(&steps).is_some(),
                "{kind}/{bits} must be an admission violation regardless of oracle"
            );
        }
        // Legal shapes: bool 0/1 both (success-only asserts no value
        // semantics), the frozen NaN payload, wide fixed widths, i32.
        for (kind, bits) in [
            ("bool", "0"),
            ("bool", "1"),
            ("i32", "0"), // the UNIQUE canonical zero (F820-R9-01)
            ("i32", "2a"),
            ("i32", "ffffffff"), // -1
            ("i32", "80000000"), // i32::MIN
            ("f32", "7f80ad00"),
            ("f32", "3f800000"),
            ("f32", "00000000"), // +0.0 (fixed width: zeros are the width)
            ("f64", "8000000000000000"),
            ("f64", "0000000000000000"), // wide zero (fixed width)
            ("u64", "ffffffffffffffff"),
            ("u64", "0000000000000000"),
        ] {
            assert!(typed_shape_violation(&typed(kind, bits).unwrap()).is_none());
        }
        assert!(data_malformed_typed(&[step(None, typed("bool", "1"))]).is_none());
        assert!(data_malformed_typed(&[step(Some(42), None)]).is_none());
    }

    /// PLAN-821 pins: the owned channel is an OWNED-recipe increment — the
    /// admission rules and the fixture judge are exercised on the
    /// production functions with synthetic faces.
    #[test]
    fn owned_face_is_family_isolated_and_shape_validated() {
        let owned_face = serde_json::json!({
            "result": { "kind": "str", "utf8": "Auto" },
            "live_after_release": 0,
            "allocs": [ { "id": 0, "bytes": 4 } ],
            "frees": [ { "id": 0 } ],
        });
        let mut s = step(None, None);
        s.owned = Some(owned_face.clone());
        // Present on a non-owned recipe → admission violation.
        assert!(owned_on_non_owned(RecipeFamily::Core, &[s.clone()]));
        assert!(owned_on_non_owned(RecipeFamily::Data, &[s.clone()]));
        // Present on the owned recipe → fine, and the shape is canonical.
        assert!(!owned_on_non_owned(RecipeFamily::Owned, &[s.clone()]));
        assert!(malformed_owned(&[s.clone()]).is_none());
        // Typed stays data-only even on owned steps.
        let mut with_typed = step(None, typed("bool", "1"));
        with_typed.owned = Some(owned_face);
        assert!(typed_on_non_data(RecipeFamily::Owned, &[with_typed]));
        // Malformed faces: missing/typed result leaf, non-u64 live, an
        // alloc entry without bytes, a free id that is a string.
        for bad in [
            serde_json::json!({ "result": "str", "live_after_release": 0,
                                "allocs": [], "frees": [] }),
            serde_json::json!({ "result": { "kind": "str", "utf8": "x" },
                                "live_after_release": -1, "allocs": [], "frees": [] }),
            serde_json::json!({ "result": { "kind": "str", "utf8": "x" },
                                "live_after_release": 0,
                                "allocs": [ { "id": 0 } ], "frees": [] }),
            serde_json::json!({ "result": { "kind": "str", "utf8": "x" },
                                "live_after_release": 0,
                                "allocs": [], "frees": [ { "id": "0" } ] }),
            // Missing-key faces (merge-gate repair): a `?`-skip used to
            // let these through as "not malformed".
            serde_json::json!({ "live_after_release": 0,
                                "allocs": [], "frees": [] }),
            serde_json::json!({ "result": { "kind": "str", "utf8": "x" },
                                "live_after_release": 0, "allocs": [] }),
        ] {
            let mut b = step(None, None);
            b.owned = Some(bad);
            assert!(
                malformed_owned(&[b]).is_some(),
                "malformed owned face must be an admission violation"
            );
        }
    }

    #[test]
    fn owned_fixture_judge_distinguishes_every_face() {
        let good = serde_json::json!({
            "result": {
                "kind": "record", "name": "t_rec_Note",
                "fields": [
                    { "kind": "str", "utf8": "Auto" },
                    { "kind": "i32", "bits": "4" },
                ],
            },
            "live_after_release": 0,
            "allocs": [ { "id": 0, "bytes": 4 }, { "id": 1, "bytes": 4 } ],
            "frees": [ { "id": 0 }, { "id": 1 } ],
        });
        fn with(face: serde_json::Value) -> StepOut {
            let mut s = step(None, None);
            s.owned = Some(face);
            s
        }
        let mut ms = Vec::new();
        judge_owned_fixture("first-case", &with(good.clone()), &mut ms);
        assert!(
            ms.is_empty(),
            "the frozen first-case face must be green: {ms:?}"
        );
        // Wrong result bytes / live≠0 / an extra allocation / reversed
        // free order — each mutation alone (from the pristine face) must
        // redden its own axis.
        let pristine = good.clone();
        let mut m_result = pristine.clone();
        m_result["result"]["fields"][0]["utf8"] = serde_json::json!("Autp");
        let mut m_live = pristine.clone();
        m_live["live_after_release"] = serde_json::json!(1);
        let mut m_allocs = pristine.clone();
        m_allocs["allocs"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "id": 2, "bytes": 4
            }));
        let mut m_frees = pristine.clone();
        m_frees["frees"].as_array_mut().unwrap().reverse();
        for (axis, face) in [
            ("result", m_result),
            ("live", m_live),
            ("allocs", m_allocs),
            ("frees", m_frees),
        ] {
            let mut ms = Vec::new();
            judge_owned_fixture("first-case", &with(face), &mut ms);
            assert!(!ms.is_empty(), "{axis} mutation must not pass: {ms:?}");
        }
        // No owned report at all → the report mismatch (not a silent pass).
        let mut ms = Vec::new();
        judge_owned_fixture("first-case", &step(None, None), &mut ms);
        assert!(ms.iter().any(|m| m.kind == "owned-report"));
    }
}
