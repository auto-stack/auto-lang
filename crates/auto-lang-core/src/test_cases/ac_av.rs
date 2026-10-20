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

fn probe_toolchain_axes(exe_sha: &str) -> Vec<(String, String)> {
    vec![
        ("av-protocol".into(), AV_WORKER_PROTOCOL.into()),
        ("recipe".into(), AV_RECIPE.into()),
        ("av-worker-exe-sha256".into(), exe_sha.to_string()),
        (
            "av-worker-version".into(),
            EXPECTED_AV_WORKER_VERSION.into(),
        ),
    ]
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

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StepOut {
    op: String,
    outcome: String,
    #[serde(default)]
    value: Option<i32>,
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
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Err(format!(
            "{AV_RECIPE} consumes .at sources; syntax {:?} is not accepted by this recipe",
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
    if check.options.probe_recipe.as_deref() != Some(AV_RECIPE) {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    // F798-R3-07 shape: leading edit-source allowed (measured window);
    // the judged last step must be a run.
    if !matches!(steps.last(), Some(auto_test_case::contract::ProbeStep::Run)) {
        return Err(format!(
            "{AV_RECIPE} step sequences must end with a run step (contract violation)"
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
    let mut request = serde_json::json!({
        "recipe": AV_RECIPE,
        "nonce": nonce,
        "source_b64": b64(&input_bytes),
        "entry": av.entry,
        "caps": av.caps,
        "store_root": store_root.display().to_string().replace('/', "\\"),
        "steps": steps_json,
    });
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
        if ok.recipe != AV_RECIPE {
            infra_detail = Some(format!("av worker recipe mismatch: {}", ok.recipe));
            break 'admit;
        }
        if ok.worker_version != EXPECTED_AV_WORKER_VERSION {
            infra_detail = Some(format!(
                "av worker version `{}` does not match the admitted `{}`",
                ok.worker_version, EXPECTED_AV_WORKER_VERSION
            ));
            break 'admit;
        }
        if ok.av_core_version != EXPECTED_AV_CORE_VERSION {
            infra_detail = Some(format!(
                "av core version `{}` does not match the admitted `{}`",
                ok.av_core_version, EXPECTED_AV_CORE_VERSION
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
                let expect_hash = fnv1a64_hex(current_source.as_bytes());
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
            detail: Some(format!("{AV_RECIPE}: infra failure")),
            artifacts: vec![],
            native: None,
        });
    }

    let step = step_out.expect("admitted");
    judge_expectation(&step, oracle, &mut mismatches);

    // Receipt (probe-receipt.json in the artifact dir). `request_wall_ms`
    // is the continuous parent-side window from request build to judged
    // response — spawn, request write, worker session, response read and
    // admission all inside (F798-R3-07: the formal turnaround gate
    // measures THIS, never a subtraction of parent call time).
    let request_wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let receipt = serde_json::json!({
        "recipe": AV_RECIPE,
        "protocol": AV_WORKER_PROTOCOL,
        "nonce": nonce,
        "request_hash": request_hash,
        "step": {
            "op": step.op,
            "outcome": step.outcome,
            "value": step.value,
            "trap": step.trap,
            "diag": step.diag,
            "counters": step.counters,
            "source_hash": step.source_hash,
            "wall_ms": step.wall_ms,
            "spans": step.spans,
            "trace": step.trace,
        },
        "worker_us": worker_us,
        "request_wall_ms": request_wall_ms,
        "toolchain": probe_toolchain_axes(&exe_sha),
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
        toolchain: probe_toolchain_axes(&exe_sha),
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
            "{AV_RECIPE}: run step judged (outcome {}); receipt probe-receipt.json",
            step.outcome
        )),
        artifacts: vec![],
        native: Some(native),
    })
}

/// Judge the observed step outcome against the declared expectations.
/// The oracle faces: success+exit (value), error-run+message (trap),
/// error-parse/verify+code (diagnosed rejection).
fn judge_expectation(step: &StepOut, oracle: &ResolvedOracle, mismatches: &mut Vec<Mismatch>) {
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
                // success without a value oracle: still require a value present
                if step.value.is_none() {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "missing".into(),
                        expected: "an AV success value".into(),
                        actual: "none".into(),
                        first_diff: None,
                    });
                }
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
