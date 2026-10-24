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
const READER_GRACE: Duration = Duration::from_secs(2);
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
    CACHE.get_or_init(|| locate_worker_exn().map_err(|e| e)).clone()
}

fn locate_worker_exn() -> Result<PathBuf, String> {
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "mn worker not located: no experimental/ac-mir-native above the crate and no AC_MN_CASE_WORKER override".to_string(),
            );
        };
        let m = d.join("experimental").join("ac-mir-native").join("Cargo.toml");
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
                || resp.get("request_hash").and_then(|v| v.as_str())
                    != Some(request_hash.as_str())
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
                infra_detail =
                    Some("mn worker success without a manifest fingerprint".to_string());
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
    let judged_spans: Vec<(String, u64)> = judged_record
        .map(|r| r.spans.clone())
        .unwrap_or_default();
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
