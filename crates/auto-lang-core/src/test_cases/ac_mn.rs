//! PLAN-818: the `ac/mir-native-scalar-v1` probe recipe — formal AutoDown
//! execution of the MIR-native backend through the unified probe/run seam
//! (PLAN-796, riding the PLAN-788 TestCase v2 runner).
//!
//! Same contract shape as `ac_av.rs`: the case declares a recipe + steps +
//! an `mn` binding (entry/caps); this adapter builds/spawns the plan's
//! worker (`mn-worker case-run`, protocol `mnworker/1`) under Job
//! containment, admits the schema/nonce/request-hash/recipe/version echoes,
//! and judges the observed outcome PARENT-SIDE. The worker never sees the
//! expected outcomes.
//!
//! Step semantics (T-06): `restart` splits the sequence into worker
//! SESSIONS — each session is a REAL new worker process (the store on disk
//! is the only carried state); a session's leading `edit-source` steps
//! materialize PARENT-SIDE (exactly-once literal replacement) before the
//! request, so the worker protocol `mnworker/1` stays unchanged (the T-02
//! single-run channel decodes and runs identically). The judged step is
//! the sequence's LAST run; every session's run is structurally admitted
//! (it really linked and really executed) and its manifest identity is
//! admitted parent-side — sessions fed identical sources must echo
//! IDENTICAL manifest fingerprints.
//!
//! Judgement faces:
//! - `completion: "success"` + `exit: <i32>` — the real PE exit code
//!   (a language trap IS exit 70, the frozen experiment contract);
//! - `completion: { stage: "parse"|"verify", code }` — a diagnosed
//!   pipeline rejection.

use crate::test_cases::sema::containment;
use auto_test_case::compare::Mismatch;
use auto_test_case::contract::{CheckSpec, Completion, Stage};
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
const SESSION_WALL: Duration = Duration::from_secs(120);
const BUILD_WALL: Duration = Duration::from_secs(300);

pub const MN_RECIPE: &str = "ac/mir-native-scalar-v1";
pub const MN_WORKER_PROTOCOL: &str = "mnworker/1";
/// Mirrored worker identity expectations — admission constants; a worker
/// bump without a mirror update fails loudly.
pub const EXPECTED_MN_WORKER_VERSION: &str = "mnworker/1.0";

pub fn current_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();
    if let Ok(binary) = worker_binary() {
        if let Ok(sha) = worker_file_sha256(&binary) {
            m.insert("mn-worker-exe-sha256".to_string(), sha);
        }
    }
    m.insert("recipe".to_string(), MN_RECIPE.to_string());
    m
}

fn worker_file_sha256(exe: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    // Memoized per (path, len): the turnaround path hashes the worker exe
    // per receipt; a same-process repeat is the same evidence.
    use std::sync::{Mutex, OnceLock};
    static MEMO: OnceLock<Mutex<std::collections::HashMap<(PathBuf, u64), String>>> =
        OnceLock::new();
    let meta = std::fs::metadata(exe).map_err(|e| format!("stat worker exe: {e}"))?;
    let key = (exe.to_path_buf(), meta.len());
    let memo = MEMO.get_or_init(|| Mutex::new(std::collections::HashMap::new()));
    if let Some(h) = memo.lock().unwrap().get(&key) {
        return Ok(h.clone());
    }
    let bytes = std::fs::read(exe).map_err(|e| format!("read worker exe: {e}"))?;
    let h = hex(&Sha256::digest(&bytes));
    memo.lock().unwrap().insert(key, h.clone());
    Ok(h)
}

/// Locate (and on first use build) the mn worker binary. Env override
/// first (read on EVERY call — the fault-injection test seam must stay
/// injectable after the real binary was memoized); otherwise find
/// `experimental/ac-mir-native` above this crate's source root and build
/// its worker release under the contained, bounded build discipline (same
/// shape as the av worker build).
pub fn worker_binary() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var("AC_MN_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "AC_MN_CASE_WORKER points at a missing file: {}",
            env_path
        ));
    }
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| locate_worker_exn().map_err(|e| e))
        .clone()
}

fn locate_worker_exn() -> Result<PathBuf, String> {
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "mn worker not located: no experimental/ac-mir-native above the crate and no AC_MN_CASE_WORKER override".to_string(),
            );
        };
        let m = d
            .join("experimental")
            .join("ac-mir-native")
            .join("Cargo.toml");
        if m.is_file() {
            break m;
        }
        dir = d.parent();
    };
    build_worker(&manifest)
}

fn build_worker(manifest: &Path) -> Result<PathBuf, String> {
    let ws_root = manifest.parent().expect("workspace root");
    let exe = ws_root.join("target").join("release").join("mn-worker.exe");
    // ALWAYS build (no-op incremental builds are sub-second) under the
    // bounded wall — a stale binary silently unbinds evidence from source.
    let cargo = std::env::var("AC_MN_BUILD_CARGO").unwrap_or_else(|_| env!("CARGO").to_string());
    let mut cmd = Command::new(&cargo);
    cmd.args(["build", "--release", "-p", "ac-mn", "--bin", "mn-worker"])
        .current_dir(ws_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let build_started = Instant::now();
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn cargo for the mn worker: {e}"))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "containment unavailable: refusing an uncontained mn worker build tree".to_string(),
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
    // Drain both pipes on reader threads so a full pipe surfaces instead
    // of deadlocking the build (capped; the tail is diagnostics only).
    fn drain_thread<R: Read + Send + 'static>(
        pipe: Option<R>,
    ) -> Option<std::thread::JoinHandle<()>> {
        pipe.map(|mut s| {
            std::thread::spawn(move || {
                let mut buf = Vec::new();
                let mut chunk = [0u8; 8192];
                loop {
                    match s.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            buf.extend_from_slice(&chunk[..n]);
                            if buf.len() > STDERR_CAP {
                                break;
                            }
                        }
                    }
                }
            })
        })
    }
    let mut out_t = drain_thread(child.stdout.take());
    let mut err_t = drain_thread(child.stderr.take());
    let _ = &mut job;
    loop {
        if build_started.elapsed() > BUILD_WALL {
            let _ = child.kill();
            drop(job.take());
            let _ = child.wait();
            return Err("mn worker build exceeded the bounded wall".to_string());
        }
        match child.try_wait() {
            Ok(Some(st)) => {
                drop(job.take());
                if let Some(t) = out_t.take() {
                    let _ = t.join();
                }
                if let Some(t) = err_t.take() {
                    let _ = t.join();
                }
                if !st.success() {
                    return Err(format!("mn worker build exited {st:?}"));
                }
                break;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(format!("poll mn worker build: {e}")),
        }
    }
    if !exe.is_file() {
        return Err(format!(
            "mn worker binary missing after build: {}",
            exe.display()
        ));
    }
    Ok(exe)
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// One contained worker call: spawn suspended → Job → resume → write the
/// request line → read to EOF (exactly one protocol line) → exit; a
/// single absolute deadline over the whole session, tree-killed on
/// breach. Returns the response line.
fn call_worker(request: &str) -> Result<String, String> {
    let exe = worker_binary()?;
    let mut cmd = Command::new(&exe);
    cmd.arg("case-run")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(containment::CREATE_SUSPENDED);
    }
    let started = Instant::now();
    let mut child = cmd.spawn().map_err(|e| format!("spawn mn worker: {e}"))?;
    #[cfg(windows)]
    let mut job = match containment::JobGuard::for_child(&child) {
        Some(j) => Some(j),
        None => {
            let _ = child.kill();
            let _ = child.wait();
            return Err("containment unavailable: refusing an uncontained mn worker".into());
        }
    };
    #[cfg(windows)]
    if let Err(e) = containment::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(format!("resume mn worker: {e}"));
    }
    #[cfg(not(windows))]
    let mut job: Option<containment::JobGuard> = None;
    // Request writer + capped stdout/stderr readers.
    let mut stdin = child.stdin.take();
    let req = request.to_string();
    let write_t = std::thread::spawn(move || {
        if let Some(s) = stdin.as_mut() {
            let _ = s.write_all(req.as_bytes());
            let _ = s.write_all(b"\n");
            let _ = s.flush();
        }
    });
    fn cap_reader(
        r: impl Read + Send + 'static,
        cap: usize,
    ) -> std::thread::JoinHandle<(bool, Vec<u8>)> {
        std::thread::spawn(move || {
            let mut r = r;
            let mut buf = Vec::new();
            let mut exceeded = false;
            let mut chunk = [0u8; 8192];
            loop {
                match r.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.len() > cap {
                            exceeded = true;
                            break;
                        }
                    }
                }
            }
            (exceeded, buf)
        })
    }
    let out_pipe = child.stdout.take().ok_or("worker stdout pipe")?;
    let err_pipe = child.stderr.take().ok_or("worker stderr pipe")?;
    let out_t = cap_reader(out_pipe, READ_CAP);
    let err_t = cap_reader(err_pipe, STDERR_CAP);

    let deadline = started + SESSION_WALL;
    let mut timed_out = false;
    let mut exit_status = None;
    loop {
        if Instant::now() > deadline {
            timed_out = true;
            break;
        }
        match child.try_wait() {
            Ok(st @ Some(_)) => {
                exit_status = st;
                break;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(2)),
            Err(e) => return Err(format!("poll mn worker: {e}")),
        }
    }
    if timed_out {
        let _ = child.kill();
        drop(job.take());
        let _ = child.wait();
        let _ = write_t.join();
        let _ = out_t.join();
        let _ = err_t.join();
        return Err("mn worker exceeded the session wall (single absolute deadline)".into());
    }
    drop(job.take());
    let _ = write_t.join();
    let (out_over, out_buf) = out_t.join().map_err(|_| "stdout reader panicked")?;
    let (err_over, err_buf) = err_t.join().map_err(|_| "stderr reader panicked")?;
    if out_over {
        return Err(format!("mn worker stdout exceeded the {READ_CAP}-byte cap"));
    }
    if err_over {
        return Err(format!(
            "mn worker stderr exceeded the {STDERR_CAP}-byte cap; stderr: {:.300}",
            String::from_utf8_lossy(&err_buf)
        ));
    }
    let status = exit_status.unwrap_or_else(|| child.wait().unwrap_or_default());
    if !status.success() {
        return Err(format!(
            "mn worker exited {:?} (non-zero worker exits are infra, never a verdict); stderr: {:.300}",
            status.code(),
            String::from_utf8_lossy(&err_buf)
        ));
    }
    let text = String::from_utf8_lossy(&out_buf).to_string();
    let non_empty: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if non_empty.len() != 1 {
        return Err(format!(
            "mn worker response must be exactly one non-empty protocol line, saw {} ({:.200})",
            non_empty.len(),
            text
        ));
    }
    Ok(non_empty[0].to_string())
}

/// Exactly-once literal replacement for an `edit-source` step (the
/// discriminating-edit oracle demands a unique match — ambiguity is a
/// contract error, never a silent first-hit edit).
fn replace_once(src: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    let s = std::str::from_utf8(src).map_err(|e| format!("source not utf-8: {e}"))?;
    let hits = s.matches(from).count();
    if hits != 1 {
        return Err(format!(
            "edit-source literal matched {hits} times (expected exactly once)"
        ));
    }
    Ok(s.replacen(from, to, 1).into_bytes())
}

/// One line-read outcome from a ROUNDS worker's stdout.
enum RoundRead {
    Line(String),
    Closed,
    ReadErr(String),
}

/// The ROUNDS session's absolute wall: the frozen default, overridable to
/// a short budget by the liveness fixtures (AC_MN_ROUNDS_WALL_SECS).
fn rounds_wall() -> Duration {
    std::env::var("AC_MN_ROUNDS_WALL_SECS")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|s| *s >= 1)
        .map(Duration::from_secs)
        .unwrap_or(SESSION_WALL)
}

/// A persistent ROUNDS mn-worker session (F-818-R1-01): the parent writes
/// ONE request line per step and reads ONE response line per step, so the
/// parent observes the CONTINUOUS per-edit interval — its own request
/// preparation (source copy + literal search/insert + payload
/// serialization), IPC both ways, the worker's locate/edit/link/run,
/// response serialization, and its own admission of (and independent
/// comparison for) the round's result.
///
/// Failure liveness and IPC bounds (F-818-R3-01/R3-02): every blocking
/// operation lives on a session-owned thread; the parent only ever
/// (a) posts a line to the writer's mailbox (bounded by the step count —
/// a stuck pipe can never block the parent), (b) receives a response
/// under the ONE absolute session deadline, and (c) reaps via a
/// deadline-governed poll — never a bare blocking `wait`. Stdout is
/// capped DURING accumulation (a newline-free or oversized stream trips
/// the cap before unbounded allocation) and the response queue is
/// bounded (an unsolicited flood trips the overflow flag). A stderr cap
/// breach sets a flag checked on every receive and at finish, so a
/// cap-violating worker can never produce an apparently-valid verdict.
/// Contained (suspended spawn → Job → resume; the guard is held for the
/// whole session); teardown kills the tree first, then joins every
/// session-owned thread — pipes close with the tree, so no handle leaks.
struct RoundsWorker {
    child: std::process::Child,
    job: Option<containment::JobGuard>,
    tx_lines: Option<std::sync::mpsc::Sender<String>>,
    writer: Option<std::thread::JoinHandle<()>>,
    reader: Option<std::thread::JoinHandle<()>>,
    stderr_t: Option<std::thread::JoinHandle<()>>,
    stderr_done: std::sync::mpsc::Receiver<(bool, Vec<u8>)>,
    stdout_over: std::sync::Arc<std::sync::atomic::AtomicBool>,
    stderr_over: std::sync::Arc<std::sync::atomic::AtomicBool>,
    rx: std::sync::mpsc::Receiver<RoundRead>,
    started: Instant,
    deadline: Instant,
}

impl RoundsWorker {
    /// Spawn the contained worker and return the session (RAII held from
    /// BEFORE the first byte is written) plus the warmup response line.
    fn start(request: &str) -> Result<(RoundsWorker, String), String> {
        let exe = worker_binary()?;
        let mut cmd = Command::new(&exe);
        cmd.arg("case-run")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(containment::CREATE_SUSPENDED);
        }
        let started = Instant::now();
        let mut child = cmd.spawn().map_err(|e| format!("spawn mn worker: {e}"))?;
        #[cfg(windows)]
        let job = match containment::JobGuard::for_child(&child) {
            Some(j) => Some(j),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("containment unavailable: refusing an uncontained mn worker".into());
            }
        };
        #[cfg(windows)]
        if let Err(e) = containment::resume_suspended_process(&child) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("resume mn worker: {e}"));
        }
        #[cfg(not(windows))]
        let job: Option<containment::JobGuard> = None;
        // Session-owned threads (joined at teardown — never leaked):
        // - writer: owns stdin; the parent only posts to its mailbox, so a
        //   full pipe can never block the parent.
        // - reader: owns stdout; caps DURING accumulation and posts into a
        //   BOUNDED queue (unsolicited floods trip the overflow flag).
        // - stderr: caps at STDERR_CAP and reports the breach on a flag the
        //   parent checks on every receive and at finish.
        let mut stdin = child.stdin.take().ok_or("worker stdin pipe")?;
        let (tx_lines, rx_lines) = std::sync::mpsc::channel::<String>();
        let writer = std::thread::spawn(move || {
            while let Ok(payload) = rx_lines.recv() {
                let flushed = stdin
                    .write_all(payload.as_bytes())
                    .and_then(|_| stdin.flush());
                if flushed.is_err() {
                    break;
                }
            }
        });
        let out_pipe = child.stdout.take().ok_or("worker stdout pipe")?;
        let (tx, rx) = std::sync::mpsc::sync_channel::<RoundRead>(8);
        let stdout_over = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stdout_flag = std::sync::Arc::clone(&stdout_over);
        let reader = std::thread::spawn(move || {
            let mut r = out_pipe;
            let mut pending: Vec<u8> = Vec::new();
            // The newline scan resumes here (a multi-megabyte valid line
            // must not be rescanned from zero per chunk).
            let mut scanned = 0usize;
            let mut chunk = [0u8; 8192];
            loop {
                match r.read(&mut chunk) {
                    Ok(0) => {
                        let _ = tx.send(RoundRead::Closed);
                        break;
                    }
                    Err(e) => {
                        let _ = tx.send(RoundRead::ReadErr(format!("{e}")));
                        break;
                    }
                    Ok(n) => {
                        pending.extend_from_slice(&chunk[..n]);
                        // The cap applies DURING accumulation: a
                        // newline-free or oversized stream trips it before
                        // the pending buffer can grow past the bound.
                        if pending.len() > READ_CAP {
                            stdout_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            let _ = tx.try_send(RoundRead::Closed);
                            break;
                        }
                        let mut stop = false;
                        while let Some(rel) = pending[scanned..].iter().position(|&b| b == b'\n') {
                            let pos = scanned + rel;
                            let line = String::from_utf8_lossy(&pending[..pos]).into_owned();
                            pending.drain(..=pos);
                            scanned = 0;
                            if line.trim().is_empty() {
                                continue;
                            }
                            // Bounded queue: an unsolicited flood fills the
                            // channel and trips the flag instead of
                            // allocating without bound.
                            if tx.try_send(RoundRead::Line(line)).is_err() {
                                stdout_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                                stop = true;
                                break;
                            }
                        }
                        if stop {
                            break;
                        }
                    }
                }
            }
        });
        let err_pipe = child.stderr.take().ok_or("worker stderr pipe")?;
        let stderr_over = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stderr_flag = std::sync::Arc::clone(&stderr_over);
        let (tx_err, stderr_done) = std::sync::mpsc::channel::<(bool, Vec<u8>)>();
        let stderr_t = std::thread::spawn(move || {
            let mut s = err_pipe;
            let mut buf = Vec::new();
            let mut chunk = [0u8; 8192];
            let mut exceeded = false;
            loop {
                match s.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        buf.extend_from_slice(&chunk[..n]);
                        if buf.len() > STDERR_CAP {
                            exceeded = true;
                            stderr_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            break;
                        }
                    }
                }
            }
            let _ = tx_err.send((exceeded, buf));
        });
        let mut worker = RoundsWorker {
            child,
            job,
            tx_lines: Some(tx_lines),
            writer: Some(writer),
            reader: Some(reader),
            stderr_t: Some(stderr_t),
            stderr_done,
            stdout_over,
            stderr_over,
            rx,
            started,
            deadline: started + rounds_wall(),
        };
        // RAII is held from here on; the initial write goes through the
        // session-owned writer (a blocked write cannot block the parent —
        // the warmup receive under the absolute wall governs it).
        worker.send_line(request)?;
        let warm = worker.read_line()?;
        Ok((worker, warm))
    }

    /// Post one line to the writer's mailbox (never blocks on the pipe;
    /// a stuck writer is caught by the response deadline, not by us).
    fn send_line(&mut self, line: &str) -> Result<(), String> {
        if Instant::now() > self.deadline {
            return Err(
                "mn rounds session exceeded the session wall (single absolute deadline)".into(),
            );
        }
        let Some(tx) = self.tx_lines.as_ref() else {
            return Err("mn rounds writer already stopped".into());
        };
        let mut payload = String::with_capacity(line.len() + 1);
        payload.push_str(line);
        payload.push('\n');
        tx.send(payload)
            .map_err(|_| "mn rounds writer already stopped".to_string())
    }

    fn read_line(&self) -> Result<String, String> {
        let Some(timeout) = self.deadline.checked_duration_since(Instant::now()) else {
            return Err(
                "mn rounds session exceeded the session wall (single absolute deadline)".into(),
            );
        };
        let got = self.rx.recv_timeout(timeout);
        // Cap propagation first: a cap-violating worker never yields an
        // apparently-valid line, even when stdout itself stays well-formed.
        if self.stdout_over.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(format!(
                "mn rounds worker stdout exceeded the {READ_CAP}-byte cap"
            ));
        }
        if self.stderr_over.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(format!(
                "mn rounds worker stderr exceeded the {STDERR_CAP}-byte cap"
            ));
        }
        match got {
            Ok(RoundRead::Line(l)) => Ok(l),
            Ok(RoundRead::Closed) => Err("mn rounds worker closed its output early".into()),
            Ok(RoundRead::ReadErr(e)) => Err(format!("mn rounds worker stdout read: {e}")),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                Err("mn rounds session exceeded the session wall (single absolute deadline)".into())
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                Err("mn rounds worker output ended without a response".into())
            }
        }
    }

    /// One round: post the step line, receive the response line.
    fn round(&mut self, line: &str) -> Result<String, String> {
        self.send_line(line)?;
        self.read_line()
    }

    /// Deadline-governed reap — never a bare blocking `wait` (a worker
    /// that answers every round but refuses to exit still hits the wall).
    fn reap_until_deadline(&mut self) -> Result<std::process::ExitStatus, String> {
        loop {
            if Instant::now() > self.deadline {
                return Err(
                    "mn rounds worker did not exit within the session wall (single absolute deadline)"
                        .into(),
                );
            }
            match self.child.try_wait() {
                Ok(Some(st)) => return Ok(st),
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                Err(e) => return Err(format!("poll mn rounds worker: {e}")),
            }
        }
    }

    /// The graceful close handshake under the absolute wall: close line →
    /// close response → deadline-governed reap → stderr cap verdict.
    fn finish(&mut self) -> Result<(), String> {
        let handshake = self
            .send_line("{\"op\":\"close\"}")
            .and_then(|_| self.read_line().map(|_| ()));
        let reap = self.reap_until_deadline().and_then(|st| {
            if st.success() {
                Ok(())
            } else {
                Err(format!("mn rounds worker exited {st:?}"))
            }
        });
        // The tree is gone (reaped, or teardown below kills it): the
        // pipes are closed, so the stderr report arrives promptly.
        let (err_exceeded, err_tail) = self
            .stderr_done
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or((false, Vec::new()));
        let mut failures = Vec::new();
        if let Err(e) = &handshake {
            failures.push(format!("close handshake: {e}"));
        }
        if let Err(e) = &reap {
            failures.push(format!("reap: {e}"));
        }
        if err_exceeded {
            failures.push(format!(
                "worker stderr exceeded the {STDERR_CAP}-byte cap; tail: {:.200}",
                String::from_utf8_lossy(&err_tail)
            ));
        }
        self.teardown();
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    }

    /// Kill the tree FIRST (closing every pipe), then join all
    /// session-owned threads — bounded because the closed pipes end their
    /// blocking reads/writes. No orphan readers or writers.
    fn teardown(&mut self) {
        self.tx_lines.take();
        let _ = self.child.kill();
        let _ = self.child.wait();
        drop(self.job.take());
        if let Some(t) = self.writer.take() {
            let _ = t.join();
        }
        if let Some(t) = self.reader.take() {
            let _ = t.join();
        }
        if let Some(t) = self.stderr_t.take() {
            let _ = t.join();
        }
    }

    fn worker_us(&self) -> u64 {
        self.started.elapsed().as_micros() as u64
    }
}

impl Drop for RoundsWorker {
    fn drop(&mut self) {
        self.teardown();
    }
}

/// One executed session's parent-side record (receipt evidence).
struct SessionRecord {
    steps: Vec<String>,
    source_sha: String,
    exit_code: Option<i32>,
    manifest_fp: Option<String>,
    counters: Vec<(String, u32)>,
    spans: Vec<(String, u64)>,
    worker_us: u64,
    judged: bool,
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
            "{MN_RECIPE} consumes .at sources; syntax {:?} is not accepted by this recipe",
            case.spec.source.syntax.as_str()
        ));
    }
    let mn = check
        .options
        .probe_mn
        .as_ref()
        .ok_or("probe check without an mn binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if !matches!(steps.last(), Some(auto_test_case::contract::ProbeStep::Run)) {
        return Err(format!(
            "{MN_RECIPE} step sequences must end with a run step (contract violation)"
        ));
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    // Store root (T-07 turnaround shape, the 798 session form): with an
    // explicit SESSION id the store lives under a deterministic shared
    // temp root keyed by that id, so calls of the same session (warm-up +
    // measured edit) share one warm store across worker processes; the
    // PARENT owns cleanup. Without a session, the check's own artifact
    // dir hosts a private per-request store.
    let store_root = match &mn.session {
        Some(id) => {
            // The session scratch's PARENT is the adapter host: the
            // default is the system temp; AC_MN_SESSION_ROOT (a directory)
            // relocates session stores (the turnaround host uses the
            // reference machine's build disk — same evidence, faster
            // store I/O).
            match std::env::var("AC_MN_SESSION_ROOT") {
                Ok(base) if std::path::Path::new(&base).is_dir() => {
                    std::path::PathBuf::from(base).join(format!("mn818-session-{id}"))
                }
                _ => std::env::temp_dir().join(format!("mn818-session-{id}")),
            }
        }
        None => artifact_dir.join("mn-store"),
    };
    std::fs::create_dir_all(&store_root).map_err(|e| format!("mn store root: {e}"))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("mn artifact dir: {e}"))?;
    let workdir = artifact_dir.join("mn-work");

    // ---- T-06 session split: `restart` ends a session (each session is a
    // REAL new worker process; the on-disk store is the carried state);
    // a session's leading edits materialize parent-side before its run.
    let mut sessions: Vec<Vec<&auto_test_case::contract::ProbeStep>> = Vec::new();
    let mut current: Vec<&auto_test_case::contract::ProbeStep> = Vec::new();
    for st in steps.iter() {
        let is_restart = matches!(st, auto_test_case::contract::ProbeStep::Restart);
        current.push(st);
        if is_restart {
            sessions.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        sessions.push(current);
    }
    // F-818-R1-03: warm edit sequences (edits, no restart boundary) run
    // through the D-828 incremental session INSIDE the worker — single
    // warm-up, single-body edit transactions, per-step real link/run.
    // Restart-mixed sequences stay on the whole-request session model
    // (cross-process = the cold scenario; its warm edits are not
    // incremental and are not labeled so).
    let has_edit_step = steps
        .iter()
        .any(|s| matches!(s, auto_test_case::contract::ProbeStep::EditSource { .. }));
    let has_restart = steps
        .iter()
        .any(|s| matches!(s, auto_test_case::contract::ProbeStep::Restart));
    if has_edit_step && !has_restart {
        return run_incremental(case, check, oracle, artifact_dir, mn, steps);
    }
    let total_runs = steps
        .iter()
        .filter(|s| matches!(s, auto_test_case::contract::ProbeStep::Run))
        .count();
    let mut run_index = 0usize; // 0-based index of executed RUN steps

    let mut source = input_bytes.clone();
    let mut mismatches: Vec<Mismatch> = Vec::new();
    let mut infra_detail: Option<String> = None;
    let mut diag: Option<(String, String)> = None;
    let mut exit_code: Option<i32> = None;
    let mut stderr_hex = String::new();
    let mut session_records: Vec<SessionRecord> = Vec::new();
    // Parent-side manifest identity admission: source-hash → manifest fp.
    let mut manifest_by_source: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();

    'sessions: for session in sessions.iter() {
        // Materialize this session's leading edits parent-side.
        let mut step_labels = Vec::new();
        for st in session.iter() {
            match st {
                auto_test_case::contract::ProbeStep::EditSource { from, to } => {
                    source = replace_once(&source, from, to)?;
                    step_labels.push("edit-source".to_string());
                }
                auto_test_case::contract::ProbeStep::Restart => {
                    step_labels.push("restart".to_string());
                }
                auto_test_case::contract::ProbeStep::Run => step_labels.push("run".to_string()),
                auto_test_case::contract::ProbeStep::Query => {
                    return Err(format!(
                        "{MN_RECIPE} does not take query steps (decode contract violation)"
                    ));
                }
            }
        }
        let has_run = session
            .iter()
            .any(|s| matches!(s, auto_test_case::contract::ProbeStep::Run));
        if !has_run {
            // A restart-only prefix: a session boundary without a judged
            // run — nothing to request; the next session opens cold.
            continue;
        }
        let source_sha = hex(&sha256(&source));

        let nonce = hex(&sha256(
            format!("{}|{:?}|mn", std::process::id(), started).as_bytes(),
        ));
        let mut request = serde_json::json!({
            "recipe": MN_RECIPE,
            "nonce": nonce,
            "source_b64": b64(&source),
            "entry": mn.entry,
            "caps": mn.caps,
            "store_root": store_root.display().to_string().replace('/', "\\"),
            "workdir": workdir.display().to_string().replace('/', "\\"),
        });
        let request_hash = hex(&sha256(request.to_string().as_bytes()));
        request["request_hash"] = serde_json::json!(request_hash);

        let worker_started = Instant::now();
        let resp_result = call_worker(&request.to_string());
        let worker_us = worker_started.elapsed().as_micros() as u64;

        let mut counters: Vec<(String, u32)> = Vec::new();
        let mut spans: Vec<(String, u64)> = Vec::new();
        let mut session_exit: Option<i32> = None;
        let mut session_manifest: Option<String> = None;

        'admit: {
            let resp = match resp_result {
                Ok(r) => serde_json::from_str::<serde_json::Value>(&r)
                    .map_err(|e| format!("bad mn worker response ({e}): {r:.200}")),
                Err(e) => Err(e),
            };
            let resp = match resp {
                Ok(r) => r,
                Err(e) => {
                    infra_detail = Some(e);
                    break 'admit;
                }
            };
            if resp.get("schema").and_then(|v| v.as_str()) != Some(MN_WORKER_PROTOCOL) {
                infra_detail = Some("mn worker schema mismatch".to_string());
                break 'admit;
            }
            if resp.get("nonce").and_then(|v| v.as_str()) != Some(nonce.as_str())
                || resp.get("request_hash").and_then(|v| v.as_str()) != Some(request_hash.as_str())
            {
                infra_detail = Some("mn worker nonce/request_hash echo mismatch".to_string());
                break 'admit;
            }
            let Some(ok) = resp.get("ok") else {
                let e = resp.get("error");
                infra_detail = Some(format!(
                    "mn worker error: {}",
                    e.map(|v| v.to_string()).unwrap_or_else(|| "missing".into())
                ));
                break 'admit;
            };
            if ok.get("recipe").and_then(|v| v.as_str()) != Some(MN_RECIPE) {
                infra_detail = Some("mn worker recipe mismatch".to_string());
                break 'admit;
            }
            if ok.get("worker_version").and_then(|v| v.as_str()) != Some(EXPECTED_MN_WORKER_VERSION)
            {
                infra_detail = Some("mn worker version mismatch".to_string());
                break 'admit;
            }
            if ok.get("mn_abi").and_then(|v| v.as_str()) != Some("msvc-x64/priv-i32-bool-v1") {
                infra_detail = Some("mn worker ABI axis mismatch".to_string());
                break 'admit;
            }
            if let Some(c) = ok.get("counters").and_then(|v| v.as_object()) {
                for (k, v) in c {
                    if let Some(n) = v.as_u64() {
                        counters.push((k.clone(), n as u32));
                    }
                }
            }
            if let Some(s) = ok.get("spans").and_then(|v| v.as_object()) {
                for (k, v) in s {
                    if let Some(n) = v.as_u64() {
                        spans.push((k.clone(), n));
                    }
                }
            }
            if let Some(d) = resp.get("diagnostic") {
                // A diagnosed pipeline rejection (mn.* / mir.* code) —
                // only the judged session's diagnosis is a verdict.
                let code = d
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?")
                    .to_string();
                let message = d
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let judged = run_index + 1 == total_runs;
                session_records.push(SessionRecord {
                    steps: step_labels,
                    source_sha,
                    exit_code: None,
                    manifest_fp: None,
                    counters,
                    spans,
                    worker_us,
                    judged,
                });
                if judged {
                    diag = Some((code, message));
                    break 'sessions;
                }
                run_index += 1;
                continue 'sessions;
            }
            // Structural counter admission (successful runs only — a
            // diagnosed rejection never reaches link/run by design): a
            // judged run REALLY linked and REALLY executed; a cached PASS
            // is never a verdict here.
            let link = counters
                .iter()
                .find(|(k, _)| k == "link_invocations")
                .map(|(_, v)| *v)
                .unwrap_or(0);
            let run = counters
                .iter()
                .find(|(k, _)| k == "run_invocations")
                .map(|(_, v)| *v)
                .unwrap_or(0);
            if link < 1 || run < 1 {
                infra_detail = Some(format!(
                    "mn worker counters violate the always-execute contract (link={link}, run={run})"
                ));
                break 'admit;
            }
            session_exit = ok
                .get("exit_code")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);
            let session_stderr = ok
                .get("stderr_hex")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            // Parent-side MANIFEST identity admission: a successful run
            // must echo its manifest fingerprint, and sessions fed
            // IDENTICAL sources must agree on it.
            let fp = ok
                .get("manifest_fp")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if fp.is_empty() {
                infra_detail = Some("mn worker success without a manifest fingerprint".to_string());
                break 'admit;
            }
            if let Some(prev) = manifest_by_source.get(&source_sha) {
                if *prev != fp {
                    infra_detail = Some(format!(
                        "mn worker manifest identity unstable for one source: {prev} vs {fp}"
                    ));
                    break 'admit;
                }
            } else {
                manifest_by_source.insert(source_sha.clone(), fp.clone());
            }
            session_manifest = Some(fp);
            if run_index + 1 == total_runs {
                // The judged session: its raw PE bytes ride the receipt.
                stderr_hex = session_stderr;
            }
        }
        if infra_detail.is_some() {
            break 'sessions;
        }
        let judged = run_index + 1 == total_runs;
        session_records.push(SessionRecord {
            steps: step_labels,
            source_sha,
            exit_code: session_exit,
            manifest_fp: session_manifest,
            counters,
            spans,
            worker_us,
            judged,
        });
        if judged {
            exit_code = session_exit;
        }
        run_index += 1;
    }

    if let Some(e) = infra_detail {
        return Ok(Verdict {
            status: Status::Error,
            mismatches: vec![Mismatch {
                observation: "worker".into(),
                kind: "infra".into(),
                expected: "a contained worker session".into(),
                actual: e,
                first_diff: None,
            }],
            detail: Some(format!("{MN_RECIPE}: infra failure")),
            artifacts: vec![],
            native: None,
        });
    }

    // ---- parent-side judgement -----------------------------------------
    let observed = if let Some((code, message)) = &diag {
        // Map the mn.* / mir.* diagnostic onto the contract stages:
        // frontend rejections parse; adapter/MIR admissions verify.
        let stage = if code.starts_with("mn.unsupported.frontend")
            || code.starts_with("mn.unsupported.input")
        {
            Stage::Parse
        } else {
            Stage::Verify
        };
        match &oracle.completion {
            Completion::Error {
                stage: want_stage,
                code: want_code,
                ..
            } => {
                if want_stage != &stage {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{want_stage:?}"),
                        actual: format!("{stage:?}"),
                        first_diff: None,
                    });
                }
                if let Some(want) = want_code {
                    if want != code {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want.clone(),
                            actual: code.clone(),
                            first_diff: None,
                        });
                    }
                }
            }
            other => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("{other:?}"),
                    actual: format!("diagnosed {code}: {message}"),
                    first_diff: None,
                });
            }
        }
        "diagnosed".to_string()
    } else {
        match &oracle.completion {
            Completion::Success => {
                if let Some(want) = oracle.exit {
                    if exit_code != Some(want) {
                        mismatches.push(Mismatch {
                            observation: "exit".into(),
                            kind: "exit".into(),
                            expected: format!("{want}"),
                            actual: format!("{exit_code:?}"),
                            first_diff: None,
                        });
                    }
                } else if exit_code.is_none() {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "missing".into(),
                        expected: "a real PE exit code".into(),
                        actual: "none".into(),
                        first_diff: None,
                    });
                }
            }
            Completion::Error { stage, .. } => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("{stage:?} rejection"),
                    actual: format!("success exit {exit_code:?}"),
                    first_diff: None,
                });
            }
        }
        "success".to_string()
    };

    // ---- receipt: per-session evidence (T-06) -------------------------
    let request_wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let exe_sha = worker_file_sha256(&worker_binary()?)?;
    let judged_record = session_records.iter().rev().find(|r| r.judged);
    let sessions_json: Vec<serde_json::Value> = session_records
        .iter()
        .map(|r| {
            serde_json::json!({
                "steps": r.steps,
                "source_sha256": r.source_sha,
                "exit_code": r.exit_code,
                "manifest_fp": r.manifest_fp,
                "judged": r.judged,
                "worker_us": r.worker_us,
                "counters": r.counters,
                "spans": r.spans,
            })
        })
        .collect();
    let receipt = serde_json::json!({
        "recipe": MN_RECIPE,
        "protocol": MN_WORKER_PROTOCOL,
        "exit_code": exit_code,
        "stderr_hex": stderr_hex,
        "sessions": sessions_json,
        "request_wall_ms": request_wall_ms,
        "toolchain": {
            "recipe": MN_RECIPE,
            "mn-worker-exe-sha256": exe_sha,
        },
    });
    std::fs::write(
        artifact_dir.join("mn-probe-receipt.json"),
        serde_json::to_string_pretty(&receipt).unwrap_or_default(),
    )
    .map_err(|e| format!("write receipt: {e}"))?;

    // Aggregated observability for the record (the judged session's own
    // counters/spans lead, so single-run shapes are byte-compatible in
    // spirit with the T-02 receipt).
    let mut counters: Vec<(String, u32)> = Vec::new();
    let mut spans: Vec<(String, u64)> = Vec::new();
    let mut worker_us_total: u64 = 0;
    for r in &session_records {
        worker_us_total += r.worker_us;
        for (k, v) in &r.counters {
            if let Some(e) = counters.iter_mut().find(|(ek, _)| ek == k) {
                e.1 += v;
            } else {
                counters.push((k.clone(), *v));
            }
        }
        for (k, v) in &r.spans {
            if let Some(e) = spans.iter_mut().find(|(ek, _)| ek == k) {
                e.1 += v;
            } else {
                spans.push((k.clone(), *v));
            }
        }
    }
    let judged_spans: Vec<(String, u64)> =
        judged_record.map(|r| r.spans.clone()).unwrap_or_default();
    let judged_counters: Vec<(String, u32)> = judged_record
        .map(|r| r.counters.clone())
        .unwrap_or_default();

    let native = NativeRecord {
        exit_code,
        stage_spans: {
            let mut s = vec![
                ("worker_us".to_string(), worker_us_total),
                ("request_wall_ms".to_string(), request_wall_ms as u64),
            ];
            s.extend(judged_spans);
            s
        },
        cache_mode: None,
        cache_hits: judged_counters
            .iter()
            .filter(|(k, _)| k == "cache_hit_coff")
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect(),
        counters,
        toolchain: {
            vec![
                ("recipe".to_string(), MN_RECIPE.to_string()),
                ("mn-worker-exe-sha256".to_string(), exe_sha),
            ]
        },
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
            "{MN_RECIPE}: run judged (outcome {observed}); receipt mn-probe-receipt.json"
        )),
        artifacts: vec![],
        native: Some(native),
    })
}

/// F-818-R1-03: the formal INCREMENTAL path — the worker warms one D-828
/// session on the case source and applies the edit steps as single-body
/// transactions. F-818-R1-01: the parent drives the ROUNDS protocol (one
/// persistent worker process; ONE request/response line pair per step)
/// and measures each edit's CONTINUOUS parent-observed window — the
/// §5.5 interval from the parent's own request preparation (source copy,
/// literal search/insert, payload serialization) through IPC, the
/// worker's locate/edit/link/run, the response, and the parent's
/// admission of the round. Parent-side admission beyond the
/// whole-request path: every edit step must be TRULY incremental (no
/// unaffected-function stage work, no whole-module stage, no recorded
/// fallback, no refusal) and every step really links and runs; the
/// judged value is the LAST step's PE exit. A syntax-fallback edit
/// switches the session to whole-request materialization (F-818-R2-02:
/// the accepted prefix is preserved exactly once); its trailing run is
/// admitted under the whole-request contract and the receipt labels the
/// fallback. The worker-side continuous `wall_ms` rides every record.
fn run_incremental(
    case: &ResolvedCase,
    _check: &CheckSpec,
    oracle: &ResolvedOracle,
    artifact_dir: &Path,
    mn: &auto_test_case::contract::ProbeMnSpec,
    steps: &[auto_test_case::contract::ProbeStep],
) -> Result<Verdict, String> {
    use auto_test_case::contract::{Completion, Stage};

    let started = Instant::now();
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Err(format!(
            "{MN_RECIPE} consumes .at sources; syntax {:?} is not accepted",
            case.spec.source.syntax.as_str()
        ));
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    let store_root = match &mn.session {
        Some(id) => {
            let base = std::env::var("AC_MN_SESSION_ROOT")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| std::env::temp_dir());
            std::fs::create_dir_all(&base).map_err(|e| format!("session root: {e}"))?;
            base.join(format!("mn818-inc-{id}"))
        }
        None => artifact_dir.join("mn-inc-store"),
    };
    std::fs::create_dir_all(&store_root).map_err(|e| format!("inc store root: {e}"))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("mn artifact dir: {e}"))?;
    let workdir = artifact_dir.join("mn-inc-work");

    // F-818-R1-01 residual: the OBSERVER SEAM for independent per-edit
    // expectations. The worker never sees expected outcomes; when the
    // observer (the formal test, holding the independent oracle) supplies
    // them via the environment, the adapter compares each edit round's
    // real PE result INSIDE the round's continuous parent-observed window
    // (preparation → IPC → worker → admission → comparison) — a mismatch
    // is a recorded verdict failure, never a silent admission.
    let expected_exits: Option<Vec<i64>> = match std::env::var("AC_MN_INCREMENTAL_EXPECT_EXITS") {
        Err(_) => None,
        Ok(raw) if raw.trim().is_empty() => None,
        Ok(raw) => {
            let list = raw
                .split(',')
                .map(|t| t.trim().parse::<i64>())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("AC_MN_INCREMENTAL_EXPECT_EXITS: {e}"))?;
            let edits = steps
                .iter()
                .filter(|s| matches!(s, auto_test_case::contract::ProbeStep::EditSource { .. }))
                .count();
            if list.len() != edits {
                return Err(format!(
                    "AC_MN_INCREMENTAL_EXPECT_EXITS carries {} values for {edits} edit steps (observer seam misuse)",
                    list.len()
                ));
            }
            Some(list)
        }
    };

    let nonce = hex(&sha256(
        format!("{}|{:?}|mn-inc", std::process::id(), started).as_bytes(),
    ));
    let mut request = serde_json::json!({
        "recipe": MN_RECIPE,
        "mode": "rounds",
        "nonce": nonce,
        "source_b64": b64(&input_bytes),
        "entry": mn.entry,
        "caps": mn.caps,
        "store_root": store_root.display().to_string().replace('/', "\\"),
        "workdir": workdir.display().to_string().replace('/', "\\"),
    });
    let request_hash = hex(&sha256(request.to_string().as_bytes()));
    request["request_hash"] = serde_json::json!(request_hash);

    let (mut rounds, warm_line) = match RoundsWorker::start(&request.to_string()) {
        Ok(p) => p,
        Err(e) => {
            return Ok(Verdict {
                status: Status::Error,
                mismatches: vec![Mismatch {
                    observation: "worker".into(),
                    kind: "infra".into(),
                    expected: "a contained incremental worker session".into(),
                    actual: e,
                    first_diff: None,
                }],
                detail: Some(format!("{MN_RECIPE}: infra failure (incremental)")),
                artifacts: vec![],
                native: None,
            })
        }
    };

    let mut infra_detail: Option<String> = None;
    let mut diag: Option<(String, String)> = None;
    let mut exit_code: Option<i32> = None;
    let mut stderr_hex = String::new();
    let mut stdout_hex = String::new();
    let mut step_records: Vec<serde_json::Value> = Vec::new();
    let mut parent_windows_ms: Vec<f64> = Vec::new();
    let mut whole_fallback = false;
    let mut resp_manifest_fp = String::new();
    // Seam-supplied independent comparisons that FAILED (verdict-level).
    let mut expect_mismatches: Vec<(usize, i64, Option<i32>)> = Vec::new();

    'admit: {
        let warm: serde_json::Value = match serde_json::from_str(&warm_line) {
            Ok(v) => v,
            Err(e) => {
                infra_detail = Some(format!("bad warmup line ({e}): {warm_line:.200}"));
                break 'admit;
            }
        };
        if warm.get("schema").and_then(|v| v.as_str()) != Some(MN_WORKER_PROTOCOL)
            || warm.get("nonce").and_then(|v| v.as_str()) != Some(nonce.as_str())
            || warm.get("request_hash").and_then(|v| v.as_str()) != Some(request_hash.as_str())
        {
            infra_detail = Some("mn rounds warmup echo mismatch".to_string());
            break 'admit;
        }
        let warm_ok = warm.get("ok").cloned().unwrap_or(serde_json::Value::Null);
        if warm_ok.get("op").and_then(|v| v.as_str()) != Some("warmup")
            || warm_ok
                .get("exit_code")
                .map(|v| v.is_null())
                .unwrap_or(true)
        {
            infra_detail = Some("mn rounds worker did not answer the warmup round".to_string());
            break 'admit;
        }
        if warm_ok.get("recipe").and_then(|v| v.as_str()) != Some(MN_RECIPE)
            || warm_ok.get("worker_version").and_then(|v| v.as_str())
                != Some(EXPECTED_MN_WORKER_VERSION)
            || warm_ok.get("mn_abi").and_then(|v| v.as_str()) != Some("msvc-x64/priv-i32-bool-v1")
        {
            infra_detail = Some("mn rounds worker identity mismatch".to_string());
            break 'admit;
        }
        step_records.push(warm_ok);

        let mut source = input_bytes.clone();
        // The observer-seam cursor: one expected exit per edit step,
        // consumed at PREPARATION time (fallback-materialized rounds keep
        // their slot — the comparison there is null, the case oracle owns
        // the judged value).
        let mut expect_idx = 0usize;
        for (i, st) in steps.iter().enumerate() {
            // The parent-observed window opens at the parent's OWN
            // preparation for this round (F-818-R1-01): the exactly-once
            // literal replacement on the parent's source copy plus the
            // round payload serialization happen INSIDE it.
            let round_started = Instant::now();
            let round_nonce = hex(&sha256(format!("{nonce}|step{i}").as_bytes()));
            let (op, round_line, step_expect) = match st {
                auto_test_case::contract::ProbeStep::EditSource { from, to } => {
                    let next = match replace_once(&source, from, to) {
                        Ok(s) => s,
                        Err(e) => {
                            infra_detail = Some(e);
                            break 'admit;
                        }
                    };
                    source = next;
                    let step_expect = expected_exits.as_ref().map(|xs| xs[expect_idx]);
                    expect_idx += 1;
                    (
                        "edit-source",
                        serde_json::json!({
                            "op": "edit-source",
                            "from": from,
                            "to": to,
                            "nonce": round_nonce,
                        })
                        .to_string(),
                        step_expect,
                    )
                }
                auto_test_case::contract::ProbeStep::Run => (
                    "run",
                    serde_json::json!({"op": "run", "nonce": round_nonce}).to_string(),
                    None,
                ),
                other => {
                    infra_detail = Some(format!(
                        "{MN_RECIPE} incremental path does not take {other:?} steps"
                    ));
                    break 'admit;
                }
            };
            let line = match rounds.round(&round_line) {
                Ok(l) => l,
                Err(e) => {
                    infra_detail = Some(e);
                    break 'admit;
                }
            };
            let resp: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    infra_detail = Some(format!("bad round line ({e}): {line:.200}"));
                    break 'admit;
                }
            };
            if resp.get("schema").and_then(|v| v.as_str()) != Some(MN_WORKER_PROTOCOL) {
                infra_detail = Some("mn worker schema mismatch".to_string());
                break 'admit;
            }
            if resp.get("nonce").and_then(|v| v.as_str()) != Some(round_nonce.as_str()) {
                infra_detail = Some("mn worker round nonce echo mismatch".to_string());
                break 'admit;
            }
            if let Some(err) = resp.get("error") {
                infra_detail = Some(format!(
                    "mn worker error: {}",
                    serde_json::to_string(err).unwrap_or_default()
                ));
                break 'admit;
            }
            let Some(ok) = resp.get("ok").cloned() else {
                infra_detail = Some("mn round response without a record".to_string());
                break 'admit;
            };
            if ok.get("op").and_then(|v| v.as_str()) != Some(op) {
                infra_detail = Some(format!(
                    "mn round answered op {:?}, the parent asked for {op}",
                    ok.get("op").and_then(|v| v.as_str())
                ));
                break 'admit;
            }
            if op == "edit-source" {
                if ok.get("warm_edit_fallback").and_then(|v| v.as_bool()) == Some(true) {
                    // F-818-R2-02: the syntax increment could not converge
                    // on this legal edit; the worker materialized it on the
                    // session's CURRENT source (accepted prefix preserved).
                    // Not an incremental sample; the trailing run executes
                    // one whole-request pipeline admitted below. The seam
                    // slot is kept with a NULL comparison — no per-edit PE
                    // result exists here; the case oracle judges the run.
                    whole_fallback = true;
                    let mut rec = ok;
                    if let Some(want) = step_expect {
                        rec["expected_exit"] = serde_json::json!(want);
                        rec["expected_match"] = serde_json::Value::Null;
                    }
                    rec["parent_window_ms"] =
                        serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                    step_records.push(rec);
                    continue;
                }
                // Parent-side INCREMENTALITY admission: the edit is truly
                // incremental and its PE result is a real, admitted exit.
                if ok.get("refused").map(|v| !v.is_null()) == Some(true) {
                    infra_detail = Some(format!(
                        "warm edit refused by the session: {}",
                        ok.get("refused").and_then(|v| v.as_str()).unwrap_or("")
                    ));
                    break 'admit;
                }
                let unaffected = ok
                    .get("unaffected_stage_delta")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let whole = ok
                    .get("whole_module_stage_delta")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let fallback = ok
                    .get("fallback_whole_lower")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                if unaffected != 0 || whole != 0 || fallback != 0 {
                    infra_detail = Some(format!(
                        "edit round {i} violates the incremental contract (unaffected={unaffected}, whole={whole}, fallback={fallback})"
                    ));
                    break 'admit;
                }
                let round_exit = ok
                    .get("exit_code")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);
                if round_exit.is_none() {
                    infra_detail = Some(format!("edit round {i} without a real PE exit"));
                    break 'admit;
                }
                exit_code = round_exit;
                stderr_hex = ok
                    .get("stderr_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                stdout_hex = ok
                    .get("stdout_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut rec = ok;
                // F-818-R1-01 residual: the INDEPENDENT expected-exit
                // comparison runs INSIDE the window (before it closes) —
                // the round-owned measurement covers preparation through
                // admission AND comparison; a mismatch is recorded and
                // fails the verdict below.
                if let Some(want) = step_expect {
                    let matched = round_exit == Some(want as i32);
                    rec["expected_exit"] = serde_json::json!(want);
                    rec["expected_match"] = serde_json::json!(matched);
                    if !matched {
                        expect_mismatches.push((i, want, round_exit));
                    }
                }
                // The parent-observed window closes here: preparation,
                // IPC, worker work, response, admission, and comparison.
                let window_ms = round_started.elapsed().as_secs_f64() * 1000.0;
                rec["parent_window_ms"] = serde_json::json!(window_ms);
                parent_windows_ms.push(window_ms);
                step_records.push(rec);
            } else {
                // A run round: either the judged incremental re-link/run or
                // the whole-request leg after a syntax fallback.
                if ok.get("warm_edit_fallback").and_then(|v| v.as_bool()) == Some(true) {
                    if let Some(d) = ok.get("diagnostic") {
                        // A diagnosed rejection from the whole-request leg
                        // (a diagnosed rejection never reaches link/run by
                        // design — admitted as a verdict, not infra).
                        whole_fallback = true;
                        diag = Some((
                            d.get("code")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?")
                                .to_string(),
                            d.get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        ));
                        let mut rec = ok;
                        rec["parent_window_ms"] =
                            serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                        step_records.push(rec);
                        continue;
                    }
                    let link = ok
                        .get("counters")
                        .and_then(|c| c.get("link_invocations"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let run = ok
                        .get("counters")
                        .and_then(|c| c.get("run_invocations"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    if link < 1 || run < 1 {
                        infra_detail =
                            Some("fallback run violates the always-execute contract".to_string());
                        break 'admit;
                    }
                    let fp = ok
                        .get("manifest_fp")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if fp.is_empty() {
                        infra_detail =
                            Some("fallback success without a manifest fingerprint".to_string());
                        break 'admit;
                    }
                    resp_manifest_fp = fp;
                    let round_exit = ok
                        .get("exit_code")
                        .and_then(|v| v.as_i64())
                        .map(|v| v as i32);
                    if round_exit.is_none() {
                        infra_detail = Some("no judged exit in the fallback run round".to_string());
                        break 'admit;
                    }
                    exit_code = round_exit;
                    stderr_hex = ok
                        .get("stderr_hex")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    stdout_hex = ok
                        .get("stdout_hex")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    whole_fallback = true;
                    let mut rec = ok;
                    rec["parent_window_ms"] =
                        serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                    step_records.push(rec);
                    continue;
                }
                if ok.get("infra").map(|v| !v.is_null()) == Some(true) {
                    infra_detail = Some(format!("run round {i} infra: {:?}", ok.get("infra")));
                    break 'admit;
                }
                let round_exit = ok
                    .get("exit_code")
                    .and_then(|v| v.as_i64())
                    .map(|v| v as i32);
                if round_exit.is_none() {
                    infra_detail = Some(format!("run round {i} without a real PE exit"));
                    break 'admit;
                }
                exit_code = round_exit;
                stderr_hex = ok
                    .get("stderr_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                stdout_hex = ok
                    .get("stdout_hex")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut rec = ok;
                rec["parent_window_ms"] =
                    serde_json::json!(round_started.elapsed().as_secs_f64() * 1000.0);
                step_records.push(rec);
            }
        }
        if infra_detail.is_none() {
            if let Err(e) = rounds.finish() {
                infra_detail = Some(e);
            }
        }
    }
    let worker_us = rounds.worker_us();
    drop(rounds);

    if let Some(e) = infra_detail {
        return Ok(Verdict {
            status: Status::Error,
            mismatches: vec![Mismatch {
                observation: "worker".into(),
                kind: "infra".into(),
                expected: "a contained incremental worker session".into(),
                actual: e,
                first_diff: None,
            }],
            detail: Some(format!("{MN_RECIPE}: infra failure (incremental)")),
            artifacts: vec![],
            native: None,
        });
    }

    // ---- parent-side judgement (the same faces as the whole-request
    // path; the judged value is the LAST step's PE exit).
    let mut mismatches: Vec<Mismatch> = Vec::new();
    if let Some((code, message)) = &diag {
        let stage = if code.starts_with("mn.unsupported.frontend")
            || code.starts_with("mn.unsupported.input")
        {
            Stage::Parse
        } else {
            Stage::Verify
        };
        match &oracle.completion {
            Completion::Error {
                stage: want_stage,
                code: want_code,
                ..
            } => {
                if want_stage != &stage {
                    mismatches.push(Mismatch {
                        observation: "stage".into(),
                        kind: "stage".into(),
                        expected: format!("{want_stage:?}"),
                        actual: format!("{stage:?}"),
                        first_diff: None,
                    });
                }
                if let Some(want) = want_code {
                    if want != code {
                        mismatches.push(Mismatch {
                            observation: "code".into(),
                            kind: "code".into(),
                            expected: want.clone(),
                            actual: code.clone(),
                            first_diff: None,
                        });
                    }
                }
            }
            other => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("{other:?}"),
                    actual: format!("diagnosed {code}: {message}"),
                    first_diff: None,
                });
            }
        }
    } else {
        match &oracle.completion {
            Completion::Success => {
                if let Some(want) = oracle.exit {
                    if exit_code != Some(want) {
                        mismatches.push(Mismatch {
                            observation: "exit".into(),
                            kind: "exit".into(),
                            expected: format!("{want}"),
                            actual: format!("{exit_code:?}"),
                            first_diff: None,
                        });
                    }
                } else if exit_code.is_none() {
                    mismatches.push(Mismatch {
                        observation: "exit".into(),
                        kind: "missing".into(),
                        expected: "a real PE exit code".into(),
                        actual: "none".into(),
                        first_diff: None,
                    });
                }
            }
            Completion::Error { stage, .. } => {
                mismatches.push(Mismatch {
                    observation: "completion".into(),
                    kind: "outcome".into(),
                    expected: format!("{stage:?} rejection"),
                    actual: format!("success exit {exit_code:?}"),
                    first_diff: None,
                });
            }
        }
    }
    // F-818-R1-01 residual: seam-supplied independent comparisons are
    // verdicts — a mismatched edit round fails the case (never a silent
    // admission); the comparison already ran inside each round's window.
    for (i, want, got) in expect_mismatches.drain(..) {
        mismatches.push(Mismatch {
            observation: format!("edit round {i} independent exit"),
            kind: "exit".into(),
            expected: format!("{want}"),
            actual: format!("{got:?}"),
            first_diff: None,
        });
    }

    let request_wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    // The whole-request fallback's manifest identity rides the receipt
    // when present (the true incremental shape has none by design).
    let manifest_fp = resp_manifest_fp.as_str();
    let exe_sha = worker_file_sha256(&worker_binary()?)?;
    let receipt = serde_json::json!({
        "recipe": MN_RECIPE,
        "protocol": MN_WORKER_PROTOCOL,
        "incremental": !whole_fallback,
        "warm_edit_fallback": whole_fallback,
        "exit_code": exit_code,
        "stderr_hex": stderr_hex,
        "stdout_hex": stdout_hex,
        "manifest_fp": manifest_fp,
        "steps": step_records,
        "parent_windows_ms": parent_windows_ms,
        "worker_us": worker_us,
        "request_wall_ms": request_wall_ms,
        "toolchain": {
            "recipe": MN_RECIPE,
            "mn-worker-exe-sha256": exe_sha,
        },
    });
    std::fs::write(
        artifact_dir.join("mn-probe-receipt.json"),
        serde_json::to_string_pretty(&receipt).unwrap_or_default(),
    )
    .map_err(|e| format!("write receipt: {e}"))?;

    let status = if mismatches.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    Ok(Verdict {
        status,
        mismatches,
        detail: Some(format!(
            "{MN_RECIPE}: incremental run judged (exit {exit_code:?}); receipt mn-probe-receipt.json"
        )),
        artifacts: vec![],
        native: Some(NativeRecord {
            exit_code,
            stage_spans: vec![
                ("worker_us".to_string(), worker_us),
                ("request_wall_ms".to_string(), request_wall_ms as u64),
            ],
            cache_mode: None,
            cache_hits: vec![],
            counters: vec![],
            toolchain: vec![
                ("recipe".to_string(), MN_RECIPE.to_string()),
                ("mn-worker-exe-sha256".to_string(), exe_sha),
            ],
            gaps: vec![],
        }),
    })
}
