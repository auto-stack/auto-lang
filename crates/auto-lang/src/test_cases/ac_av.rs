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
use auto_test_case::report::{ArtifactRef, NativeRecord, Status};
use auto_test_case::sha256::{hex, sha256};
use base64::Engine as _;

use super::avm::Verdict;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use base64::Engine;

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
    let exe = worker_binary()?;
    let bytes = std::fs::read(&exe).map_err(|e| format!("read worker exe: {e}"))?;
    Ok(hex(&sha256(&bytes)))
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
    if !exe.is_file() {
        let out = Command::new(env!("CARGO"))
            .args([
                "build",
                "--release",
                "-p",
                "auto-av-probe",
                "--bin",
                "av_probe_worker",
            ])
            .current_dir(ws_root)
            .output()
            .map_err(|e| format!("spawn cargo for the av worker: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "av worker build failed:\n{}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
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
}

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

    // Bounded request write + response read (mirror of the sema executor).
    let mut stdin = child.stdin.take().expect("stdin");
    stdin
        .write_all(request_json.as_bytes())
        .and_then(|_| stdin.flush())
        .map_err(|e| format!("write request: {e}"))?;
    drop(stdin);
    let mut stdout = child.stdout.take().expect("stdout");
    let mut stderr = child.stderr.take().expect("stderr");
    let mut out_buf = Vec::new();
    let mut err_buf = Vec::new();
    let deadline_at = started + deadline;
    loop {
        if Instant::now() > deadline_at {
            let _ = child.kill();
            return Err("av worker response exceeded the session wall".to_string());
        }
        let mut chunk = [0u8; 16384];
        match stdout.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                out_buf.extend_from_slice(&chunk[..n]);
                if out_buf.len() > READ_CAP {
                    let _ = child.kill();
                    return Err("av worker response exceeded the read cap".to_string());
                }
            }
        }
        if out_buf.last() == Some(&b'\n') {
            break;
        }
    }
    let _ = child.wait();
    drop(job);
    // Bounded stderr drain (overflow kills — never a silent partial).
    let _ = stderr.read_to_end(&mut err_buf);
    if err_buf.len() > STDERR_CAP {
        return Err("av worker stderr exceeded 1MiB".to_string());
    }
    let _ = READER_GRACE;
    let text = String::from_utf8_lossy(&out_buf);
    let line = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let err_tail = String::from_utf8_lossy(&err_buf);
    serde_json::from_str(line).map_err(|e| {
        format!(
            "bad av worker response ({e}): {:.200} | stderr: {:.300}",
            line, err_tail
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
    if !matches!(
        steps.first(),
        Some(auto_test_case::contract::ProbeStep::Run)
    ) {
        return Err(format!(
            "{AV_RECIPE} step sequences must start with a run step (contract violation)"
        ));
    }
    let input_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "probe adapter needs the resolved entry input".to_string())?;
    let store_root = artifact_dir.join("store");
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
    let request_hash = hex(&sha256(request.to_string().as_bytes()));
    request["request_hash"] = serde_json::json!(request_hash);

    let worker_started = Instant::now();
    let resp_result = call_worker(&request.to_string(), SESSION_WALL);
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

    // Receipt (probe-receipt.json in the artifact dir).
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
        },
        "worker_us": worker_us,
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
