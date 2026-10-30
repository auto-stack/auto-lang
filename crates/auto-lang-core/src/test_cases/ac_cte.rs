//! cte/scalar-query-v1 runner adapter (PLAN-817 T-02 minimal
//! registration): routes the deck to the `cteworker/1` worker with the
//! contained lifecycle and judges the LAST query step parent-side
//! (cache coarse vocabulary + native exit). `restart` steps are consumed
//! HERE — the step sequence is chunked at restart boundaries and each
//! chunk runs in a FRESH worker process against the same store root
//! (the real cross-process persistence path).

use super::avm::Verdict;
use crate::test_cases::sema::containment;
use auto_test_case::compare::Mismatch;
use auto_test_case::contract::{CheckSpec, ProbeStep, Syntax};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::Status;
use auto_test_case::sha256::{hex, sha256};
use serde::Deserialize;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const CTE_RECIPE: &str = "cte/scalar-query-v1";
pub const CTE_WORKER_PROTOCOL: &str = "cteworker/1";
pub const EXPECTED_CTE_WORKER_VERSION: &str = "auto-cte-probe/0.1.0";
/// Bounded release build wall for the worker (same discipline as the
/// av/sema workers).
pub const BUILD_WALL: Duration = Duration::from_secs(300);
/// One worker call's absolute wall (spawn→response→exit).
pub const SESSION_WALL: Duration = Duration::from_secs(60);
/// Metadata IPC cap (contract-decision §7).
pub const REQUEST_CAP: usize = 4 * 1024 * 1024;

fn b64(data: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(data)
}

fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    format!("{h:016x}")
}

// ---------------------------------------------------------------------------
// worker location + identity
// ---------------------------------------------------------------------------

fn cte_worker_exe_name() -> &'static str {
    if cfg!(windows) {
        "cte_probe_worker.exe"
    } else {
        "cte_probe_worker"
    }
}

fn worker_file_sha256(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read worker exe: {e}"))?;
    Ok(hex(&sha256(&bytes)))
}

/// Locate (and on first use build, bounded) the cte worker binary. Env
/// override first; otherwise build `experimental/ac-cte-probe`
/// `-p auto-cte-probe --bin cte_probe_worker` (release) from the repo
/// root above this crate.
pub fn worker_binary() -> Result<PathBuf, String> {
    if std::env::var("AUTO_TEST_DOMAIN").as_deref() == Ok("legacy") {
        return Err(
            "ACE worker requested by a legacy test group; explicitly select the ACE group".into(),
        );
    }
    // The env override is re-read EVERY call (never cached): a test that
    // pins a broken tool identity must not poison the process-wide
    // built-binary cache for later checks.
    if let Ok(env_path) = std::env::var("AC_CTE_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "AC_CTE_CASE_WORKER points at a missing file: {env_path}"
        ));
    }
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(locate_worker_without_env).clone()
}

fn locate_worker_without_env() -> Result<PathBuf, String> {
    locate_worker()
}

fn locate_worker() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var("AC_CTE_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "AC_CTE_CASE_WORKER points at a missing file: {env_path}"
        ));
    }
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "cte probe worker not located: no experimental/ac-cte-probe above the crate and no AC_CTE_CASE_WORKER override"
                    .to_string(),
            );
        };
        let m = d
            .join("experimental")
            .join("ac-cte-probe")
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
        .join(cte_worker_exe_name());
    // ALWAYS build (no-op incremental is sub-second) under BUILD_WALL —
    // a stale binary silently unbinds evidence from source (same
    // discipline as the av worker).
    let mut cmd = Command::new(env!("CARGO"));
    cmd.args([
        "build",
        "--release",
        "-p",
        "auto-cte-probe",
        "--bin",
        "cte_probe_worker",
        "--manifest-path",
    ])
    .arg(&manifest)
    .current_dir(ws_root);
    let out = cmd
        .output()
        .map_err(|e| format!("cte worker build spawn: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cte worker build failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    if !exe.is_file() {
        return Err(format!(
            "cte worker build produced no exe at {}",
            exe.display()
        ));
    }
    Ok(exe)
}

/// PLAN-868: admission mirror of `auto_mir_lower::MIR_LOWERING_VERSION` —
/// the cte av driver lowers HIR→MIR through the shared crate.
pub const EXPECTED_MIR_LOWERING_VERSION: &str = "4";

pub fn current_toolchain_identity() -> std::collections::BTreeMap<String, String> {
    let mut m = std::collections::BTreeMap::new();
    if let Ok(sha) = worker_binary().and_then(|p| worker_file_sha256(&p)) {
        m.insert("cte-worker-exe-sha256".to_string(), sha);
    }
    m.insert(
        "cte-worker-version".to_string(),
        EXPECTED_CTE_WORKER_VERSION.to_string(),
    );
    m.insert("cte-protocol".to_string(), CTE_WORKER_PROTOCOL.to_string());
    m.insert("recipe".to_string(), CTE_RECIPE.to_string());
    m.insert(
        "mir-lowering".to_string(),
        EXPECTED_MIR_LOWERING_VERSION.to_string(),
    );
    m
}

pub fn probe_toolchain_axes(exe_sha: &str) -> Vec<(String, String)> {
    vec![
        ("cte-protocol".into(), CTE_WORKER_PROTOCOL.into()),
        ("recipe".into(), CTE_RECIPE.into()),
        ("cte-worker-exe-sha256".into(), exe_sha.to_string()),
        (
            "cte-worker-version".into(),
            EXPECTED_CTE_WORKER_VERSION.into(),
        ),
        // PLAN-868: mandatory lowering-contract axis (see ac_av).
        ("mir-lowering".into(), EXPECTED_MIR_LOWERING_VERSION.into()),
    ]
}

fn worker_exe_sha256() -> Result<String, String> {
    worker_binary().and_then(|p| worker_file_sha256(&p))
}

/// Map a cte.* code to the runner's supported stage vocabulary (the
/// same projection the sema recipe uses — no unknown stages).
fn stage_of(code: &str) -> &'static str {
    if code.starts_with("cte.request.") || code.starts_with("cte.callee.") {
        "parse"
    } else if code.starts_with("cte.consumer.")
        || code.starts_with("cte.query.")
        || code.starts_with("cte.schedule.")
    {
        "verify"
    } else if code.starts_with("cte.capability.") || code.starts_with("cte.meta.") {
        "capability"
    } else {
        "run"
    }
}

// ---------------------------------------------------------------------------
// protocol
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct WorkerStepOut {
    op: String,
    outcome: String,
    #[serde(default)]
    source_hash: Option<String>,
    #[serde(default)]
    cache: Option<String>,
    #[serde(default)]
    ct_execute: Option<u64>,
    #[serde(default)]
    meta_query: Option<u64>,
    #[serde(default)]
    exit: Option<i64>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    language_class: bool,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct WorkerOk {
    #[allow(dead_code)]
    recipe: String,
    worker_version: String,
    #[serde(default)]
    engine: Option<String>,
    steps: Vec<WorkerStepOut>,
}

#[derive(Debug, Deserialize)]
struct WorkerResponse {
    schema: String,
    #[serde(default)]
    nonce: String,
    #[serde(default)]
    request_hash: String,
    ok: Option<WorkerOk>,
    error: Option<WorkerErrorWire>,
}

#[derive(Debug, Deserialize)]
struct WorkerErrorWire {
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
}

/// One contained worker call (single request to EOF, capped readers, one
/// absolute wall, Job-contained tree).
fn call_worker(request_json: &str, deadline: Duration) -> Result<WorkerResponse, String> {
    if request_json.len() > REQUEST_CAP {
        return Err(format!(
            "cte worker request exceeds the {REQUEST_CAP} byte metadata cap ({} bytes)",
            request_json.len()
        ));
    }
    let exe = worker_binary()?;
    let started = Instant::now();
    #[cfg(windows)]
    let mut cmd = {
        let mut c = Command::new(&exe);
        use std::os::windows::process::CommandExt;
        c.arg("case-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        c.creation_flags(containment::CREATE_SUSPENDED);
        c
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = Command::new(&exe);
        c.arg("case-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        c
    };
    let mut child = cmd.spawn().map_err(|e| format!("cte worker spawn: {e}"))?;
    let _job = containment::JobGuard::for_child(&child);
    #[cfg(windows)]
    containment::resume_suspended_process(&child).map_err(|e| format!("cte worker resume: {e}"))?;

    // Request writer thread (dropped handle closes stdin → EOF).
    let mut stdin = child.stdin.take().expect("piped stdin");
    let payload = request_json.as_bytes().to_vec();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&payload);
        let _ = stdin.flush();
    });

    // Bounded concurrent readers.
    let stdout_cap = 4 * 1024 * 1024usize;
    let stderr_cap = 1024 * 1024usize;
    let mut out_pipe = child.stdout.take().expect("piped stdout");
    let mut err_pipe = child.stderr.take().expect("piped stderr");
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 16384];
        loop {
            match out_pipe.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if buf.len() + n > stdout_cap {
                        return Err("cte worker stdout exceeded the 4MiB cap".to_string());
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }
            }
        }
        Ok(buf)
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 16384];
        loop {
            match err_pipe.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    if buf.len() + n > stderr_cap {
                        return Err("cte worker stderr exceeded the 1MiB cap".to_string());
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }
            }
        }
        Ok(buf)
    });

    // Wait under the deadline.
    let status = loop {
        if started.elapsed() > deadline {
            let _ = child.kill();
            return Err("cte worker exceeded the session wall (tree killed)".into());
        }
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(format!("cte worker wait: {e}")),
        }
    };
    let stdout = reader
        .join()
        .map_err(|_| "cte worker stdout reader panicked".to_string())?
        .map_err(|e| e)?;
    let stderr = err_reader
        .join()
        .map_err(|_| "cte worker stderr reader panicked".to_string())?
        .map_err(|e| e)?;
    let _ = writer.join();
    if !status.success() {
        return Err(format!(
            "cte worker exited with {status}: {}",
            String::from_utf8_lossy(&stderr)
        ));
    }
    if !stderr.is_empty() {
        return Err(format!(
            "cte worker wrote to stderr: {}",
            String::from_utf8_lossy(&stderr)
        ));
    }
    let text = String::from_utf8_lossy(&stdout);
    let line = text
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    serde_json::from_str(line.trim())
        .map_err(|e| format!("cte worker response does not decode: {e}"))
}

// ---------------------------------------------------------------------------
// run_check
// ---------------------------------------------------------------------------

pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    if case.spec.source.syntax != Syntax::AtomData {
        return Err(format!(
            "{CTE_RECIPE} consumes atom-data decks; syntax {:?} is not accepted by this recipe",
            case.spec.source.syntax.as_str()
        ));
    }
    let cte = check
        .options
        .probe_cte
        .as_ref()
        .ok_or("probe check without a cte binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if check.options.probe_recipe.as_deref() != Some(CTE_RECIPE) {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    if !matches!(steps.last(), Some(ProbeStep::Query)) {
        return Err(format!(
            "{CTE_RECIPE} step sequences must end with a query step (contract violation)"
        ));
    }
    let deck_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    let store_root = match &cte.session {
        Some(s) => std::env::temp_dir().join(format!("cte817-session-{s}")),
        None => artifact_dir.join("store"),
    };
    std::fs::create_dir_all(&store_root).map_err(|e| format!("probe store root: {e}"))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("probe artifact dir: {e}"))?;

    // Chunk the steps at restart boundaries: each chunk runs in a FRESH
    // worker process; edits carry over parent-side into the deck text
    // sent to the next chunk.
    let mut current_deck = String::from_utf8_lossy(&deck_bytes).to_string();
    let mut all_steps: Vec<WorkerStepOut> = Vec::new();
    let mut chunk: Vec<&ProbeStep> = Vec::new();
    let mut consumed_restart = 0usize;
    let total = steps.len();

    let flush = |chunk: &[&ProbeStep],
                 deck: &str,
                 all_steps: &mut Vec<WorkerStepOut>,
                 consumed_restart: &mut usize| {
        if chunk.is_empty() {
            return Ok(());
        }
        let nonce = hex(&sha256(
            format!("{}|{:?}|cte", std::process::id(), started).as_bytes(),
        ));
        let steps_json: Vec<serde_json::Value> = chunk
            .iter()
            .map(|st| match st {
                ProbeStep::Query => serde_json::json!({ "op": "query" }),
                ProbeStep::EditSource { from, to } => {
                    serde_json::json!({ "op": "edit-source", "from": from, "to": to })
                }
                _ => serde_json::json!({ "op": "query" }),
            })
            .collect();
        let mut request = serde_json::json!({
            "recipe": CTE_RECIPE,
            "nonce": nonce,
            "deck_b64": b64(deck.as_bytes()),
            "store_root": store_root.display().to_string().replace('/', "\\"),
            "steps": steps_json,
        });
        if let Some(sess) = &cte.session {
            request["session"] = serde_json::json!(sess);
        }
        let request_hash = fnv1a64_hex(request.to_string().as_bytes());
        request["request_hash"] = serde_json::json!(request_hash);

        let resp = call_worker(&request.to_string(), SESSION_WALL)?;
        // Admission echoes.
        if resp.schema != CTE_WORKER_PROTOCOL {
            return Err(format!("cte worker schema mismatch: {}", resp.schema));
        }
        if resp.nonce != nonce || resp.request_hash != request_hash {
            return Err("cte worker nonce/request_hash echo mismatch".to_string());
        }
        let ok = resp.ok.clone().ok_or_else(|| {
            format!(
                "cte worker error: {} {}",
                resp.error.as_ref().map(|e| e.code.as_str()).unwrap_or("?"),
                resp.error
                    .as_ref()
                    .map(|e| e.message.as_str())
                    .unwrap_or("")
            )
        })?;
        if ok.worker_version != EXPECTED_CTE_WORKER_VERSION {
            return Err(format!(
                "cte worker version `{}` does not match the admitted `{EXPECTED_CTE_WORKER_VERSION}`",
                ok.worker_version
            ));
        }
        if ok.steps.len() != chunk.len() {
            return Err(format!(
                "cte worker returned {} steps for a {}-step chunk",
                ok.steps.len(),
                chunk.len()
            ));
        }
        // Parent-side step verification (op echo + edit exactly-once +
        // per-step deck hash binding).
        let mut deck_now = deck.to_string();
        for (i, (want, got)) in chunk.iter().zip(ok.steps.iter()).enumerate() {
            match want {
                ProbeStep::Query => {
                    if got.op != "query" {
                        return Err(format!("step {i}: op `{}` is not `query`", got.op));
                    }
                    let expect_hash = fnv1a64_hex(deck_now.as_bytes());
                    if got.source_hash.as_deref() != Some(expect_hash.as_str()) {
                        return Err(format!(
                            "step {i}: source_hash `{}` does not bind the request's current deck",
                            got.source_hash.as_deref().unwrap_or("?")
                        ));
                    }
                }
                ProbeStep::EditSource { from, to } => {
                    let hits = deck_now.matches(from.as_str()).count();
                    if hits != 1 {
                        return Err(format!(
                            "step {i}: parent-side edit literal matched {hits} times"
                        ));
                    }
                    deck_now = deck_now.replacen(from.as_str(), to.as_str(), 1);
                    if got.outcome != "edited" {
                        return Err(format!(
                            "step {i}: edit-source outcome `{}` is not `edited`",
                            got.outcome
                        ));
                    }
                }
                _ => unreachable!("restart consumed by the chunker"),
            }
        }
        all_steps.extend(ok.steps);
        Ok(())
    };

    let mut result: Result<(), String> = Ok(());
    for (i, step) in steps.iter().enumerate() {
        match step {
            ProbeStep::Restart => {
                result = flush(&chunk, &current_deck, &mut all_steps, &mut consumed_restart);
                if result.is_err() {
                    break;
                }
                // Fold THIS chunk's edits into the base the next chunk
                // (a fresh worker process) starts from.
                for st in chunk.iter() {
                    if let ProbeStep::EditSource { from, to } = st {
                        let hits = current_deck.matches(from.as_str()).count();
                        if hits == 1 {
                            current_deck = current_deck.replacen(from.as_str(), to.as_str(), 1);
                        }
                    }
                }
                chunk.clear();
                consumed_restart += 1;
                let _ = consumed_restart;
            }
            ProbeStep::EditSource { from, to } => {
                // NO pre-application here: the worker applies edits to
                // the chunk's BASE deck itself. The base only advances
                // at restart boundaries (below), where this chunk's
                // edits fold into the next chunk's starting deck.
                let _ = (from, to);
                chunk.push(step);
            }
            _ => chunk.push(step),
        }
        let _ = i;
        let _ = total;
    }
    if result.is_ok() {
        result = flush(&chunk, &current_deck, &mut all_steps, &mut consumed_restart);
    }
    if let Err(e) = result {
        return Ok(Verdict {
            status: Status::Error,
            mismatches: vec![Mismatch {
                observation: "worker".into(),
                kind: "infra".into(),
                expected: "a contained cte worker session".into(),
                actual: e,
                first_diff: None,
            }],
            detail: Some(format!("{CTE_RECIPE}: infra failure")),
            artifacts: vec![],
            native: None,
        });
    }

    // ---- judge the LAST query step parent-side ----------------------
    let judged = all_steps
        .iter()
        .rev()
        .find(|s| s.op == "query")
        .cloned()
        .ok_or("cte worker returned no query step")?;
    let exe_sha = worker_exe_sha256()?;
    let mut mismatches: Vec<Mismatch> = Vec::new();
    // The judged step's declared expectations come from the LAST probe
    // step expect row (same face as the sema/av recipes).
    let last_expect = oracle.probe.as_deref().unwrap_or(&[]).last().cloned();
    let cache_expect: Option<String> = last_expect.as_ref().and_then(|e| e.cache.clone());
    if let Some(want) = cache_expect {
        let coarse = match judged.cache.as_deref() {
            Some("miss") | Some("stale") => "published",
            Some("hit") => "hit",
            other => other.unwrap_or("?"),
        };
        if coarse != want {
            mismatches.push(Mismatch {
                observation: "cache".into(),
                kind: "cache".into(),
                expected: want,
                actual: coarse.to_string(),
                first_diff: None,
            });
        }
    }
    if let Some(want_exit) = oracle.exit {
        match judged.exit {
            Some(actual) if actual == want_exit as i64 => {}
            other => mismatches.push(Mismatch {
                observation: "exit".into(),
                kind: "exit".into(),
                expected: format!("{want_exit}"),
                actual: other
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "none".into()),
                first_diff: None,
            }),
        }
    }
    // Outcome admission FIRST (the sema discipline): an isolated
    // resource/cancel/infra state is an ERROR row — never a language
    // verdict; a language-class diag is judged against the declared
    // completion (stage + message contain), defaulting to FAIL when the
    // case expected success.
    if judged.outcome == "diag" {
        let detail = format!(
            "{} {}",
            judged.code.as_deref().unwrap_or("?"),
            judged.message.as_deref().unwrap_or("")
        );
        if !judged.language_class {
            return Ok(Verdict {
                status: Status::Error,
                mismatches: vec![Mismatch {
                    observation: "query".into(),
                    kind: "infra".into(),
                    expected: "a completed query (isolated state)".into(),
                    actual: detail,
                    first_diff: None,
                }],
                detail: Some(format!("{CTE_RECIPE}: isolated state on the judged step")),
                artifacts: vec![],
                native: None,
            });
        }
        match &oracle.completion {
            auto_test_case::contract::Completion::Error { stage, message, .. } => {
                let got_stage = stage_of(judged.code.as_deref().unwrap_or(""));
                let want_stage = stage.as_str();
                if got_stage != want_stage {
                    mismatches.push(Mismatch {
                        observation: "completion.stage".into(),
                        kind: "text".into(),
                        expected: want_stage.to_string(),
                        actual: got_stage.to_string(),
                        first_diff: None,
                    });
                }
                if let Some(matcher) = message {
                    let got_msg = judged.message.as_deref().unwrap_or("");
                    let matched = match matcher {
                        auto_test_case::contract::MessageMatcher::Contains(sub) => {
                            got_msg.contains(sub.as_str())
                        }
                        auto_test_case::contract::MessageMatcher::Exact(sub) => got_msg == sub,
                    };
                    if !matched {
                        mismatches.push(Mismatch {
                            observation: "completion.message".into(),
                            kind: "text".into(),
                            expected: format!("{matcher:?}"),
                            actual: got_msg.to_string(),
                            first_diff: None,
                        });
                    }
                }
            }
            auto_test_case::contract::Completion::Success => {
                mismatches.push(Mismatch {
                    observation: "query".into(),
                    kind: "diag".into(),
                    expected: "a completed query".into(),
                    actual: detail,
                    first_diff: None,
                });
            }
        }
    }

    // Receipt (evidence file; never judged).
    let receipt = serde_json::json!({
        "recipe": CTE_RECIPE,
        "steps": all_steps.iter().map(|s| serde_json::json!({
            "op": s.op,
            "outcome": s.outcome,
            "cache": s.cache,
            "ct_execute": s.ct_execute,
            "meta_query": s.meta_query,
            "exit": s.exit,
            "source_hash": s.source_hash,
        })).collect::<Vec<_>>(),
        "toolchain": probe_toolchain_axes(&exe_sha),
        "worker_version": EXPECTED_CTE_WORKER_VERSION,
    });
    let _ = std::fs::write(
        artifact_dir.join("probe-receipt.json"),
        serde_json::to_string_pretty(&receipt).unwrap_or_default(),
    );

    let status = if mismatches.is_empty() {
        Status::Pass
    } else {
        Status::Fail
    };
    Ok(Verdict {
        status,
        mismatches,
        detail: Some(format!(
            "{CTE_RECIPE}: {} steps, last query cache={:?} ct_execute={:?} exit={:?}",
            all_steps.len(),
            judged.cache,
            judged.ct_execute,
            judged.exit
        )),
        artifacts: vec![],
        native: None,
    })
}
