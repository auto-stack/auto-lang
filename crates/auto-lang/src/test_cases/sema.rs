//! `probe` adapter executor (PLAN-796 T-06): the parent-side half of the
//! restricted probe/run seam. auto-lang NEVER links the experimental
//! semantic stack — this module spawns the `sema-probe` worker (env
//! override `SEMA_CASE_WORKER`, else located from the repo layout and
//! built release once per process) speaking `semaworker/1` and JUDGES
//! its structured step reports against the case oracle. The worker's
//! word is never trusted: nonce/request-hash echo, typed admission,
//! cache/compute bindings oracles and stage traces are all compared
//! HERE. A `restart` step ends a session; the remaining steps continue
//! in a FRESH worker process against the same store root (the real
//! cross-process persistence path — same proof shape as the first-case
//! suite, now inside the unified runner).

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use auto_test_case::compare::Mismatch;
use auto_test_case::contract::{CheckSpec, Completion, ProbeStep};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{ArtifactRef, NativeRecord, Status};
use auto_test_case::sha256::{hex, sha256};
use base64::Engine as _;
use serde::Deserialize;

use super::avm::Verdict;

/// Frozen caps (mirror the seam contract): bounded IPC read, stderr
/// drain cap, per-SESSION parent wall deadline.
const READ_CAP: usize = 8 << 20;
const STDERR_CAP: usize = 1 << 20;
const SESSION_WALL: Duration = Duration::from_secs(60);

/// Locate (and on first use build) the sema worker binary. Env override
/// first (trusted host configuration binds the tool identity); otherwise
/// find `experimental/ac-semantics` from this crate's source root and
/// `cargo build --release` the probe package (warm no-op ~1s).
pub fn worker_binary() -> Result<PathBuf, String> {
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(locate_worker).clone()
}

fn locate_worker() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var("SEMA_CASE_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!("SEMA_CASE_WORKER points at a missing file: {}", env_path));
    }
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let manifest = loop {
        let Some(d) = dir else {
            return Err(
                "sema-probe worker not located: no experimental/ac-semantics above the crate and no SEMA_CASE_WORKER override".to_string(),
            );
        };
        let m = d.join("experimental").join("ac-semantics").join("Cargo.toml");
        if m.is_file() {
            break m;
        }
        dir = d.parent();
    };
    let status = Command::new("cargo")
        .args(["build", "--release", "--workspace", "--manifest-path"])
        .arg(&manifest)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("spawn cargo for sema worker build: {}", e))?;
    if !status.status.success() {
        let tail = String::from_utf8_lossy(&status.stderr);
        let tail: String = tail.lines().rev().take(8).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n");
        return Err(format!("sema worker build failed:\n{}", tail));
    }
    let exe = manifest
        .parent()
        .unwrap()
        .join("target")
        .join("release")
        .join("sema-probe.exe");
    if exe.is_file() {
        Ok(exe)
    } else {
        Err(format!("sema worker build produced no binary at {}", exe.display()))
    }
}

// ---- typed response mirror (admission; drift caught by suites) --------

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
    sema_core_version: String,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct StepOut {
    op: String,
    cache: String,
    compute_calls: u32,
    #[serde(default)]
    bindings: Vec<BindingOut>,
    #[serde(default)]
    diagnostics: Vec<DiagMirror>,
    #[serde(default)]
    stages: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct BindingOut {
    name: String,
    ty: String,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct DiagMirror {
    code: String,
    message: String,
}

/// One bounded single-shot worker session (same I/O discipline as the
/// ac adapter: threaded request write, bounded stdout read, concurrent
/// stderr drain, parent wall deadline with kill).
fn call_worker(request_json: &str, deadline: Duration) -> Result<WorkerResponse, String> {
    let exe = worker_binary()?;
    let started = Instant::now();
    let mut child = Command::new(&exe)
        .arg("serve")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn sema worker ({}): {}", exe.display(), e))?;
    {
        let mut stdin = child.stdin.take().expect("piped stdin");
        let req = request_json.to_string();
        std::thread::spawn(move || {
            let _ = writeln!(stdin, r#"{{"op":"begin","schema":"semaworker/1"}}"#);
            let _ = writeln!(stdin, r#"{{"op":"request","request":{}}}"#, req);
            let _ = writeln!(stdin, r#"{{"op":"end"}}"#);
            let _ = stdin.flush();
        });
    }
    let mut stdout_pipe = child.stdout.take().expect("piped stdout");
    let mut stderr_pipe = child.stderr.take().expect("piped stderr");
    let (tx, rx) = std::sync::mpsc::channel::<Result<Vec<u8>, usize>>();
    let out_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 65536];
        loop {
            match stdout_pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    if buf.len() + n > READ_CAP {
                        let _ = tx.send(Err(buf.len() + n));
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.ends_with(b"\n") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        let _ = tx.send(Ok(buf));
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 65536];
        loop {
            match stderr_pipe.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    if buf.len() + n > STDERR_CAP {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }
                Err(_) => break,
            }
        }
        buf
    });
    let received = loop {
        if started.elapsed() > deadline {
            let _ = child.kill();
            return Err("sema worker exceeded the parent wall deadline".to_string());
        }
        match rx.recv_timeout(Duration::from_millis(10)) {
            Ok(v) => break v,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(_) => break Err(0),
        }
    };
    let status = child.wait();
    let stderr_tail = err_reader.join().unwrap_or_default();
    out_reader.join().map_err(|_| "stdout reader panicked".to_string())?;
    let bytes = received.map_err(|n| format!("sema worker response exceeded the read cap ({n} bytes)"))?;
    let _ = status;
    if !stderr_tail.is_empty() {
        let tail = String::from_utf8_lossy(&stderr_tail);
        let tail: String = tail.lines().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(" | ");
        return Err(format!("sema worker wrote to stderr: {}", tail));
    }
    let line = String::from_utf8_lossy(&bytes);
    let line = line.lines().next().unwrap_or("");
    serde_json::from_str(line).map_err(|e| format!("sema worker response malformed: {e}"))
}

// ---- the check ----------------------------------------------------------------

pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    // recipe×source admission: sema/query-v1 consumes `.at` sources;
    // the atom-data kind is registered with the seam for OTHER recipes
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Err(format!(
            "sema/query-v1 consumes .at sources; syntax {:?} is not accepted by this recipe",
            case.spec.source.syntax.as_str()
        ));
    }
    let sema = check
        .options
        .probe_sema
        .as_ref()
        .ok_or("probe check without a sema binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if check.options.probe_recipe.as_deref() != Some("sema/query-v1") {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    let store_root = artifact_dir.join("store");
    std::fs::create_dir_all(&store_root).map_err(|e| format!("probe store root: {}", e))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("probe artifact dir: {}", e))?;

    // Split the declared steps into SESSIONS at `restart` boundaries.
    // Source edits accumulate ACROSS sessions: the parent tracks the
    // current source text and re-sends it (each session re-projects it).
    let mut source = input_bytes.clone();
    let mut sessions: Vec<Vec<&ProbeStep>> = Vec::new();
    let mut current: Vec<&ProbeStep> = Vec::new();
    for st in steps.iter() {
        let is_restart = matches!(st, ProbeStep::Restart);
        current.push(st);
        if is_restart {
            sessions.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        sessions.push(current);
    }

    let mut mismatches: Vec<Mismatch> = Vec::new();
    let mut reported_query_steps: Vec<StepOut> = Vec::new();
    let mut restarts: usize = 0;

    for (si, session) in sessions.iter().enumerate() {
        let steps_json: Vec<serde_json::Value> = session
            .iter()
            .map(|st| match st {
                ProbeStep::Query => serde_json::json!({"op": "query"}),
                ProbeStep::EditSource { from, to } => serde_json::json!({
                    "op": "edit-source", "from": from, "to": to
                }),
                ProbeStep::Restart => serde_json::json!({"op": "restart"}),
            })
            .collect();
        let nonce = hex(&sha256(format!("{}|{:?}|{}", std::process::id(), started, si).as_bytes()));
        let mut request = serde_json::json!({
            "recipe": "sema/query-v1",
            "nonce": nonce,
            "source_b64": b64(&source),
            "package": sema.package,
            "module": sema.module,
            "fn_name": sema.fn_name,
            "store_root": store_root.display().to_string().replace('/', "\\"),
            "steps": steps_json,
        });
        // request_hash binds the request CONTENT (hash fields excluded)
        let request_hash = hex(&sha256(request.to_string().as_bytes()));
        request["request_hash"] = serde_json::json!(request_hash);

        let resp = call_worker(&request.to_string(), SESSION_WALL)?;
        // admission: schema + echo + shape
        if resp.schema != "semaworker/1" {
            return Err(format!("sema worker schema mismatch: {}", resp.schema));
        }
        if resp.nonce != nonce || resp.request_hash != request_hash {
            return Err("sema worker nonce/request_hash echo mismatch".to_string());
        }
        let ok = resp.ok.clone().ok_or_else(|| {
            let e = resp.error.unwrap_or(WorkerError {
                code: "missing".into(),
                message: "neither ok nor error".into(),
            });
            format!("sema worker error {}: {}", e.code, e.message)
        })?;
        if ok.recipe != "sema/query-v1" {
            return Err(format!("sema worker recipe mismatch: {}", ok.recipe));
        }
        // apply this session's edits to the running source (the worker
        // validated the exactly-once match already)
        for st in session.iter() {
            if let ProbeStep::EditSource { from, to } = st {
                source = replace_once(&source, from, to)?;
            }
        }
        if matches!(session.last(), Some(ProbeStep::Restart)) {
            restarts += 1;
            if !ok.restart_pending {
                return Err("restart step did not end the session as declared".into());
            }
        } else if ok.restart_pending {
            return Err("worker reported restart_pending without a restart step".into());
        }
        reported_query_steps.extend(ok.steps.iter().filter(|s| s.op == "query").cloned());
    }
    let _ = restarts;

    // ---- parent-side judgement against the oracle -----------------------
    let mut pass = matches!(oracle.completion, Completion::Success);
    if !pass {
        mismatches.push(Mismatch {
            observation: "completion".into(),
            kind: "text",
            expected: "success".into(),
            actual: "error".into(),
            first_diff: None,
        });
    }
    let expects = oracle.probe.as_deref().unwrap_or(&[]);
    let query_expect_count = expects.len();
    if reported_query_steps.len() != query_expect_count {
        mismatches.push(Mismatch {
            observation: "probe.steps".into(),
            kind: "text",
            expected: format!("{query_expect_count} query steps"),
            actual: format!("{} query steps", reported_query_steps.len()),
            first_diff: None,
        });
    }
    for (i, (got, want)) in reported_query_steps.iter().zip(expects.iter()).enumerate() {
        if let Some(want_cache) = &want.cache {
            // the worker may report published<miss>/published<stale>;
            // the ORACLE vocabulary is the coarse outcome
            let coarse = if got.cache.starts_with("published") { "published" } else { got.cache.as_str() };
            if coarse != want_cache {
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
                    expected: want_calls.to_string(),
                    actual: got.compute_calls.to_string(),
                    first_diff: None,
                });
            }
        }
        for stage in &want.stages_include {
            if !got.stages.iter().any(|s| s == stage) {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].stages"),
                    kind: "text",
                    expected: format!("includes {stage}"),
                    actual: got.stages.join(","),
                    first_diff: None,
                });
            }
        }
        if let (Some(bytes), Some(_sha)) = (&want.bindings_bytes, &want.bindings_sha256) {
            if let Err(diff) = compare_bindings(bytes, &got.bindings) {
                mismatches.push(Mismatch {
                    observation: format!("probe[{i}].bindings"),
                    kind: "text",
                    expected: diff.expected,
                    actual: diff.actual,
                    first_diff: None,
                });
            }
        }
    }

    // persist the structured receipt (audit trail under the artifact dir)
    let receipt = serde_json::json!({
        "recipe": "sema/query-v1",
        "query_steps": reported_query_steps,
        "store_root": store_root,
    });
    let receipt_path = artifact_dir.join("probe-receipt.json");
    std::fs::write(&receipt_path, serde_json::to_string_pretty(&receipt).unwrap())
        .map_err(|e| format!("receipt write: {e}"))?;
    let receipt_sha = hex(&sha256(&std::fs::read(&receipt_path).unwrap_or_default()));

    let artifacts = vec![ArtifactRef {
        logical_path: "probe-receipt.json".into(),
        stored_path: receipt_path.display().to_string(),
        sha256: receipt_sha,
        bytes: None,
    }];
    let native = NativeRecord {
        exit_code: None,
        stage_spans: Vec::new(),
        cache_mode: Some("sema-store".into()),
        cache_hits: reported_query_steps
            .iter()
            .map(|s| (format!("step-{}", s.cache.clone()), s.cache.clone()))
            .collect(),
        counters: vec![
            ("query_steps".into(), reported_query_steps.len() as u32),
            (
                "compute_calls".into(),
                reported_query_steps.iter().map(|s| s.compute_calls).sum(),
            ),
        ],
        toolchain: vec![
            ("sema-protocol".into(), "semaworker/1".into()),
            ("recipe".into(), "sema/query-v1".into()),
        ],
        gaps: vec![],
    };
    let status = if mismatches.is_empty() { Status::Pass } else { Status::Fail };
    Ok(Verdict {
        status,
        mismatches,
        detail: Some(format!(
            "sema/query-v1: {} query step(s) judged; receipt probe-receipt.json",
            reported_query_steps.len()
        )),
        artifacts,
        native: Some(native),
    })
}

fn replace_once(src: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    let s = std::str::from_utf8(src).map_err(|e| format!("source not utf-8: {e}"))?;
    let hits = s.matches(from).count();
    if hits != 1 {
        return Err(format!("edit-source literal matched {hits} times (expected exactly once)"));
    }
    Ok(s.replacen(from, to, 1).into_bytes())
}

struct Diff {
    expected: String,
    actual: String,
}

/// Bindings oracle: JSON {"bindings":[{name,ty},…]} — order-sensitive
/// list compare (program order is the observable).
fn compare_bindings(oracle_bytes: &[u8], got: &[BindingOut]) -> Result<(), Diff> {
    #[derive(Deserialize)]
    struct Oracle {
        bindings: Vec<OracleBinding>,
    }
    #[derive(Deserialize)]
    struct OracleBinding {
        name: String,
        ty: String,
    }
    let want: Oracle = serde_json::from_slice(oracle_bytes).map_err(|e| Diff {
        expected: format!("valid bindings oracle json: {e}"),
        actual: String::new(),
    })?;
    let want_pairs: Vec<(String, String)> =
        want.bindings.into_iter().map(|b| (b.name, b.ty)).collect();
    let got_pairs: Vec<(String, String)> =
        got.iter().map(|b| (b.name.clone(), b.ty.clone())).collect();
    if want_pairs != got_pairs {
        return Err(Diff {
            expected: format!("{want_pairs:?}"),
            actual: format!("{got_pairs:?}"),
        });
    }
    Ok(())
}

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
