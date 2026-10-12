//! AVM adapter (`avm/run`): executes the materialized entry through
//! `run_with_capture_and_path` — the same production path-carrying API the
//! legacy `run_vm_file_test` uses — and judges observations independently.
//!
//! Stage classification (Plan 768 feasibility §3) is variant-based, never
//! message-guessed for parse/type: `AutoError::Lexer*/Syntax*` → parse,
//! `Type/Name/Comptime` → type (errors are frequently wrapped in
//! `MultipleErrors`; the first error decides), `Msg` → run (the internal
//! `VMError` variant name is the observed code; `auto_gate_` prefix → gate).
//! Panics escaping the VM thread are infrastructure ERRORs, not language
//! errors. miette diagnostic codes back the parse/type oracle channel.

use std::panic::AssertUnwindSafe;
use std::path::Path;

use auto_test_case::compare::{compare_bytes, compare_text_exact, match_message, Mismatch};
use auto_test_case::contract::{CheckSpec, Completion, EntryMode, ExpectSpec, Stage, Syntax};
use auto_test_case::ir::ResolvedCase;
use auto_test_case::report::{ArtifactRef, Status};
use auto_test_case::sha256::{hex, sha256};
use miette::Diagnostic;

use super::{classify, materialize};

/// Adapter verdict: judgement only — the orchestrator binds identity.
pub struct Verdict {
    pub status: Status,
    pub mismatches: Vec<Mismatch>,
    pub detail: Option<String>,
    pub artifacts: Vec<ArtifactRef>,
}

impl Verdict {
    pub fn pass() -> Verdict {
        Verdict {
            status: Status::Pass,
            mismatches: vec![],
            detail: None,
            artifacts: vec![],
        }
    }
    pub fn fail(mismatches: Vec<Mismatch>) -> Verdict {
        Verdict {
            status: Status::Fail,
            mismatches,
            detail: None,
            artifacts: vec![],
        }
    }
    pub fn error(detail: impl Into<String>) -> Verdict {
        Verdict {
            status: Status::Error,
            mismatches: vec![],
            detail: Some(detail.into()),
            artifacts: vec![],
        }
    }
}

/// Run one `avm/run` check; `Err` means infrastructure failure before the
/// backend ran (scratch problems, unsupported entry, non-UTF-8 source).
pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    _case_root: &Path,
    project_dir: &Path,
    artifact_dir: &Path,
) -> Result<Verdict, String> {
    if case.spec.entry == EntryMode::Module {
        return Err("avm/run does not support module entry (declare script/expression/program, or use a2r/emit)".to_string());
    }

    std::fs::create_dir_all(project_dir).map_err(|e| format!("scratch: {}", e))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("scratch: {}", e))?;
    materialize::materialize_project(case, project_dir)?;

    let entry_abs = project_dir.join(case.entry_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    let bytes = case
        .files
        .iter()
        .find(|f| f.virtual_path == case.entry_path)
        .map(|f| f.bytes.clone())
        .ok_or_else(|| {
            format!(
                "entry file {:?} missing from resolved project",
                case.entry_path
            )
        })?;
    let code = String::from_utf8(bytes)
        .map_err(|_| "non-UTF-8 source cannot run on the avm adapter".to_string())?;
    let entry_path_str = entry_abs.display().to_string();

    // For `.as` sources the pipeline lowers internally (path suffix drives
    // it, exactly like the legacy production path); the pre-lowered bytes
    // are hashed here purely as submitted-input evidence.
    let lowered_sha = if case.spec.source.syntax == Syntax::AutoScript {
        crate::trans::auto_s2s::lower_source(&code)
            .ok()
            .map(|low| hex(&sha256(low.as_bytes())))
    } else {
        None
    };

    let execution = std::panic::catch_unwind(AssertUnwindSafe(|| {
        crate::run_with_capture_and_path(&code, &entry_path_str)
    }));

    let mut verdict = match execution {
        Err(_) => {
            Verdict::error("VM thread panicked — infrastructure error, not a language failure")
        }
        Ok(Err(e)) => {
            let failure = classify(&e);
            let _ = std::fs::write(artifact_dir.join("error.txt"), &failure.message);
            match &check.expect.completion {
                Completion::Success => Verdict::fail(vec![Mismatch {
                    observation: "completion".into(),
                    kind: "stage",
                    expected: "success".into(),
                    actual: format!("{} error: {}", failure.stage.as_str(), failure.message),
                    first_diff: None,
                }]),
                Completion::Error {
                    stage,
                    code,
                    message,
                } => {
                    let mut mismatches = Vec::new();
                    if failure.stage != *stage {
                        mismatches.push(Mismatch {
                            observation: "completion.stage".into(),
                            kind: "stage",
                            expected: stage.as_str().to_string(),
                            actual: failure.stage.as_str().to_string(),
                            first_diff: None,
                        });
                    }
                    if let Some(want) = code {
                        let got = failure.code.clone().unwrap_or_default();
                        if got != *want {
                            mismatches.push(Mismatch {
                                observation: "completion.code".into(),
                                kind: "code",
                                expected: want.clone(),
                                actual: got,
                                first_diff: None,
                            });
                        }
                    }
                    if let Some(matcher) = message {
                        if let Err(m) = match_message(matcher, &failure.message) {
                            mismatches.push(m);
                        }
                    }
                    if mismatches.is_empty() {
                        Verdict::pass()
                    } else {
                        Verdict::fail(mismatches)
                    }
                }
            }
        }
        Ok(Ok((result, stdout))) => {
            let _ = std::fs::write(artifact_dir.join("result.txt"), &result);
            let _ = std::fs::write(artifact_dir.join("stdout.txt"), &stdout);
            match &check.expect.completion {
                Completion::Error { stage, .. } => Verdict::fail(vec![Mismatch {
                    observation: "completion".into(),
                    kind: "stage",
                    expected: format!("{} error", stage.as_str()),
                    actual: format!("success (result={:?}, stdout={:?})", result, stdout),
                    first_diff: None,
                }]),
                Completion::Success => {
                    let mut mismatches = Vec::new();
                    if let Some(want) = &expect_result(&check.expect) {
                        if let Some(m) = compare_text_exact(want, &result, "result") {
                            mismatches.push(m);
                        }
                    }
                    if let Some(want) = &expect_stream(&check.expect, "stdout") {
                        if let Some(m) =
                            compare_bytes(want.0.as_bytes(), stdout.as_bytes(), want.1, "stdout")
                        {
                            mismatches.push(m);
                        }
                    }
                    if let Some(want) = &expect_stream(&check.expect, "stderr") {
                        // The AVM capture API exposes no stderr channel; the
                        // observation is empty by construction, so a declared
                        // non-empty expectation fails honestly.
                        if let Some(m) = compare_bytes(want.0.as_bytes(), b"", want.1, "stderr") {
                            mismatches.push(m);
                        }
                    }
                    if mismatches.is_empty() {
                        Verdict::pass()
                    } else {
                        Verdict::fail(mismatches)
                    }
                }
            }
        }
    };
    if let Some(low) = lowered_sha {
        verdict.artifacts.push(ArtifactRef {
            logical_path: "<lowered-input>".into(),
            stored_path: format!("sha256:{}", low),
            sha256: low,
            bytes: None,
        });
    }
    Ok(verdict)
}

fn expect_result(expect: &ExpectSpec) -> Option<String> {
    expect.result.as_ref().map(|r| r.text.clone())
}

fn expect_stream(
    expect: &ExpectSpec,
    which: &str,
) -> Option<(String, auto_test_case::contract::Compare)> {
    match which {
        "stdout" => expect.stdout.as_ref().map(|s| (s.text.clone(), s.compare)),
        _ => expect.stderr.as_ref().map(|s| (s.text.clone(), s.compare)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AutoError;

    #[test]
    fn classify_multiple_errors_recurses_to_first() {
        let e = AutoError::MultipleErrors {
            count: 1,
            plural: "error".into(),
            errors: vec![AutoError::Msg("DivisionByZero at line 3".into())],
        };
        let f = classify(&e);
        assert_eq!(f.stage, Stage::Run);
        assert_eq!(f.code.as_deref(), Some("DivisionByZero"));
    }

    #[test]
    fn classify_runtime_error_variant_name_is_the_code() {
        let e = AutoError::Msg("RuntimeError(\"Array index 10 out of bounds\") at line 2".into());
        let f = classify(&e);
        assert_eq!(f.stage, Stage::Run);
        assert_eq!(f.code.as_deref(), Some("RuntimeError"));
    }

    #[test]
    fn catch_unwind_wraps_vm_panics() {
        let r = std::panic::catch_unwind(AssertUnwindSafe(|| panic!("boom")));
        assert!(
            r.is_err(),
            "panics must be catchable at the adapter boundary"
        );
    }
}
