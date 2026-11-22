//! PLAN-1115 `ac-a2r/hir-language-run-v1` recipe adapter (probe/run seam):
//! the ordinary language case source runs through the REAL A2R chain —
//! language front door (instance + CT hooks) → unified language-channel
//! lowering → fileset → REAL rustc → contained execution — produced by the
//! PREBUILT `a2x-worker` tool (one spawn, exactly like the 892 emit
//! adapter's tool discipline; rustc runs INSIDE the worker under its
//! frozen deadlines). Judged entirely PARENT-SIDE against the corpus
//! row's own oracle (exit/stdout/stderr; the A2R observation protocol
//! prints exactly one `value:<n>`/`trap:<Name>` line on stdout).
//!
//! Negative rows (frontend rejections) surface from the worker as the
//! structured `a2x.source-diagnostic/1` envelope; this adapter compares
//! the primary cause's stage/code against the row's expectation.

use std::path::{Path, PathBuf};
use std::time::Instant;

use auto_test_case::contract::{CheckSpec, Completion};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};

use super::avm::Verdict;
use auto_test_case::report::Status;

/// The ordinary-language RUN recipe (distinct identity from the emit and
/// certify lanes; narrow-channel doctrine).
pub const A2R_LANG_RUN_RECIPE: &str = "ac-a2r/hir-language-run-v1";
/// The admission capability the corpus language rows carry.
pub const LANG_CAP: &str = "ac.source.language.av/v1";

fn fail(detail: String) -> Verdict {
    Verdict {
        status: Status::Error,
        mismatches: vec![],
        detail: Some(detail),
        artifacts: vec![],
        native: None,
    }
}

/// PLAN-1115: run one language case through the prebuilt a2x worker's
/// `run-language` op and judge the observation parent-side.
pub fn run_language_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    _project_dir: &Path,
    artifact_dir: &Path,
    run_dir: &Path,
) -> Result<Verdict, String> {
    let started = Instant::now();
    // single-file program sources only on this lane (multi-module rows
    // ride the package seam later)
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Ok(fail(format!(
            "{} does not accept syntax {:?}",
            A2R_LANG_RUN_RECIPE,
            case.spec.source.syntax.as_str()
        )));
    }
    let worker = super::ac_a2r_hir::worker_binary()?;
    std::fs::create_dir_all(run_dir).map_err(|e| format!("run dir: {e}"))?;
    let source_text = case
        .files
        .iter()
        .find(|f| f.virtual_path == case.entry_path)
        .map(|f| String::from_utf8_lossy(&f.bytes).to_string())
        .ok_or_else(|| format!("entry file {:?} missing", case.entry_path))?;
    let source_path: PathBuf = run_dir.join("main.at");
    std::fs::write(&source_path, &source_text)
        .map_err(|e| format!("materialize source: {e}"))?;

    let caps: Vec<String> = std::iter::once(LANG_CAP.to_string())
        .chain(
            check
                .options
                .probe_a2x_hir
                .as_ref()
                .map(|a| a.caps.clone())
                .unwrap_or_default()
                .into_iter()
                .filter(|c| c != LANG_CAP),
        )
        .collect();
    let request = serde_json::json!({
        "op": "run-language",
        "recipe": A2R_LANG_RUN_RECIPE,
        "source_path": source_path,
        "out_dir": run_dir,
        "profile": "language",
        "entry": "main",
        "caps": caps,
        "nonce": "0".repeat(64),
    });
    let request_json = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    let _ = artifact_dir;

    let output = std::process::Command::new(&worker)
        .arg(&request_json)
        .output()
        .map_err(|e| format!("spawn a2x worker: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_text = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        // the worker failed: either a structured frontend rejection (the
        // corpus negative rows) or an infra error.
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(stdout.trim()) {
            if v["schema"] == "a2x.source-diagnostic/1" {
                let cause = &v["causes"][v["primary"].as_u64().unwrap_or(0) as usize];
                let stage = cause["stage"].as_str().unwrap_or("");
                let code = cause["code"].as_str().unwrap_or("");
                match oracle.completion.clone() {
                    Completion::Error {
                        stage: want_stage,
                        code: want_code,
                        ..
                    } => {
                        let stage_matches =
                            want_stage.as_str() == stage;
                        let code_matches = match &want_code {
                            Some(want) => want == code,
                            None => true,
                        };
                        if stage_matches && code_matches {
                            return Ok(Verdict {
                                status: Status::Pass,
                                mismatches: vec![],
                                detail: Some(format!("diagnostic {stage}/{code} as expected")),
                                artifacts: vec![],
                                native: None,
                            });
                        }
                        return Ok(Verdict {
                            status: Status::Fail,
                            mismatches: vec![auto_test_case::compare::Mismatch {
                                observation: "diagnostic".into(),
                                kind: "text",
                                expected: format!(
                                    "{}/{}",
                                    want_stage.as_str(),
                                    want_code.as_deref().unwrap_or("*")
                                ),
                                actual: format!("{stage}/{code}"),
                                first_diff: None,
                            }],
                            detail: None,
                            artifacts: vec![],
                            native: None,
                        });
                    }
                    Completion::Success => {
                        return Ok(fail(format!(
                            "worker rejected the source ({stage}/{code}) but the row expects a run"
                        )))
                    }
                }
            }
        }
        return Ok(fail(format!(
            "a2x worker exit {:?}: {}{}",
            output.status.code(),
            stdout.trim(),
            stderr_text.trim()
        )));
    }
    let resp: serde_json::Value = serde_json::from_str(stdout.trim())
        .map_err(|e| format!("worker response json: {e}"))?;
    let got_stdout = resp["stdout"].as_str().unwrap_or("");
    let got_stderr = resp["stderr"].as_str().unwrap_or("");
    let mut mismatches: Vec<auto_test_case::compare::Mismatch> = Vec::new();
    match oracle.completion {
        Completion::Success => {
            if resp["status"] != "ok" {
                mismatches.push(auto_test_case::compare::Mismatch {
                    observation: "worker".into(),
                    kind: "text",
                    expected: "ok".into(),
                    actual: resp["status"].to_string(),
                    first_diff: None,
                });
            }
            if let Some(want) = oracle.exit {
                // the language wrapper exits 0 on observed outcomes
                if want != 0 {
                    mismatches.push(auto_test_case::compare::Mismatch {
                observation: "exit".into(),
                kind: "text",
                expected: want.to_string(),
                actual: "0 (value/trap on stdout)".into(),
                first_diff: None,
            });
                }
            }
            if let Some(expect) = &oracle.stdout {
                if let Some(bad) =
                    auto_test_case::compare::compare_text_exact(&expect.text, got_stdout, "stdout")
                {
                    mismatches.push(bad);
                }
            } else if !got_stdout.trim().is_empty() {
                mismatches.push(auto_test_case::compare::Mismatch {
                observation: "stdout".into(),
                kind: "text",
                expected: String::new(),
                actual: got_stdout.to_string(),
                first_diff: None,
            });
            }
            if let Some(expect) = &oracle.stderr {
                if let Some(bad) =
                    auto_test_case::compare::compare_text_exact(&expect.text, got_stderr, "stderr")
                {
                    mismatches.push(bad);
                }
            } else if !got_stderr.trim().is_empty() {
                mismatches.push(auto_test_case::compare::Mismatch {
                observation: "stderr".into(),
                kind: "text",
                expected: String::new(),
                actual: got_stderr.to_string(),
                first_diff: None,
            });
            }
        }
        Completion::Error { .. } => {
            mismatches.push(auto_test_case::compare::Mismatch {
                observation: "diagnostic".into(),
                kind: "text",
                expected: "a structured rejection".into(),
                actual: got_stdout.to_string(),
                first_diff: None,
            });
        }
    }
    let _ = started.elapsed();
    Ok(Verdict {
        status: if mismatches.is_empty() {
            Status::Pass
        } else {
            Status::Fail
        },
        mismatches,
        detail: None,
        artifacts: vec![],
        native: None,
    })
}
