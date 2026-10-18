//! AVM adapter (`avm/run`): executes the materialized entry through the
//! staged twin of `run_with_capture_and_path` — same production
//! path-carrying pipeline the legacy runner uses, plus the typed trap code
//! (Plan 773 F-08) — and judges observations independently.
//!
//! Stage classification is type-derived: parse/type come from the AutoError
//! variant, the run-stage code from the engine's `VMError` variant relayed
//! by the staged API. Infrastructure failures (I/O, panics, synthetic
//! deadlines, unclassified messages) are ERRORs and can NEVER satisfy a
//! language negative. Panics escaping the VM thread are infrastructure
//! ERRORs too.
//!
//! The result/stdout observations are persisted as real artifacts, so a
//! passing (or FAILing-but-executed) run registers usable products for
//! downstream consumers (Plan 773 F-03).

use std::panic::AssertUnwindSafe;
use std::path::Path;

use auto_test_case::compare::{compare_bytes, compare_text_exact, match_message, Mismatch};
use auto_test_case::contract::{Completion, EntryMode, Syntax};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::{ArtifactRef, Status};
use auto_test_case::sha256::{hex, sha256};

use super::{classify_staged, materialize, StageOutcome};

/// Adapter verdict: judgement only — the orchestrator binds identity.
pub struct Verdict {
    pub status: Status,
    pub mismatches: Vec<Mismatch>,
    pub detail: Option<String>,
    pub artifacts: Vec<ArtifactRef>,
    /// PLAN-788 v2 native observation record (None for avm/a2r checks).
    pub native: Option<auto_test_case::report::NativeRecord>,
}

impl Verdict {
    pub fn pass() -> Verdict {
        Verdict {
            status: Status::Pass,
            mismatches: vec![],
            detail: None,
            artifacts: vec![],
            native: None,
        }
    }
    pub fn fail(mismatches: Vec<Mismatch>) -> Verdict {
        Verdict {
            status: Status::Fail,
            mismatches,
            detail: None,
            artifacts: vec![],
            native: None,
        }
    }
    pub fn error(detail: impl Into<String>) -> Verdict {
        Verdict {
            status: Status::Error,
            mismatches: vec![],
            detail: Some(detail.into()),
            artifacts: vec![],
            native: None,
        }
    }
}

/// Run one `avm/run` check; `Err` means infrastructure failure before the
/// backend ran (scratch problems, unsupported entry, non-UTF-8 source).
pub fn run_check(
    case: &ResolvedCase,
    oracle: &ResolvedOracle,
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
    let mut lowered_sha = if case.spec.source.syntax == Syntax::AutoScript {
        crate::trans::auto_s2s::lower_source(&code)
            .ok()
            .map(|low| hex(&sha256(low.as_bytes())))
    } else {
        None
    };

    // Staged twin of the legacy capture API (F-08): identical execution,
    // typed trap code attached to run-stage failures.
    let execution = std::panic::catch_unwind(AssertUnwindSafe(|| {
        crate::run_with_capture_and_path_staged(&code, &entry_path_str)
    }));

    let mut verdict = match execution {
        Err(payload) => {
            let msg = panic_message(&payload);
            Verdict::error(format!(
                "VM thread panicked — infrastructure error, not a language failure: {}",
                msg
            ))
        }
        Ok(Err(staged)) => match classify_staged(&staged.error, staged.run_trap_code) {
            StageOutcome::Infrastructure(message) => Verdict::error(format!(
                "infrastructure failure (never satisfies a language negative): {}",
                message
            )),
            StageOutcome::Language(failure) => {
                let _ = std::fs::write(artifact_dir.join("error.txt"), &failure.message);
                match &oracle.completion {
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
        },
        Ok(Ok((result, stdout))) => {
            // Persist the observations as REAL artifacts (F-03): an executed
            // run registers result/stdout as usable products even when its
            // own oracle FAILs.
            let mut refs = vec![
                file_artifact(artifact_dir, "result.txt", result.as_bytes()),
                file_artifact(artifact_dir, "stdout.txt", stdout.as_bytes()),
            ];
            match &oracle.completion {
                Completion::Error { stage, .. } => {
                    let mut v = Verdict::fail(vec![Mismatch {
                        observation: "completion".into(),
                        kind: "stage",
                        expected: format!("{} error", stage.as_str()),
                        actual: format!("success (result={:?}, stdout={:?})", result, stdout),
                        first_diff: None,
                    }]);
                    v.artifacts = refs;
                    v
                }
                Completion::Success => {
                    let mut mismatches = Vec::new();
                    if let Some(want) = &oracle.result {
                        if let Some(m) = compare_text_exact(&want.text, &result, "result") {
                            mismatches.push(m);
                        }
                    }
                    if let Some(want) = &oracle.stdout {
                        if let Some(m) = compare_bytes(
                            want.text.as_bytes(),
                            stdout.as_bytes(),
                            want.compare,
                            "stdout",
                        ) {
                            mismatches.push(m);
                        }
                    }
                    if let Some(want) = &oracle.stderr {
                        // The AVM capture API exposes no stderr channel; the
                        // observation is empty by construction, so a declared
                        // non-empty expectation fails honestly.
                        if let Some(m) =
                            compare_bytes(want.text.as_bytes(), b"", want.compare, "stderr")
                        {
                            mismatches.push(m);
                        }
                    }
                    let mut v = if mismatches.is_empty() {
                        Verdict::pass()
                    } else {
                        Verdict::fail(mismatches)
                    };
                    if let Some(low) = lowered_sha.take() {
                        refs.push(ArtifactRef {
                            logical_path: "<lowered-input>".into(),
                            stored_path: format!("sha256:{}", low),
                            sha256: low,
                            bytes: None,
                        });
                    }
                    v.artifacts = refs;
                    v
                }
            }
        }
    };
    // `.as` evidence hash rides along on error/failure paths too.
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

/// Persist an observation file under the check's scratch and describe it as
/// a product artifact (real bytes on disk, hashed).
fn file_artifact(dir: &Path, name: &str, bytes: &[u8]) -> ArtifactRef {
    let stored = dir.join(name);
    // Scratch writes are inside the audited out domain; failure here is an
    // infrastructure error surfaced by the caller.
    if let Err(e) = std::fs::write(&stored, bytes) {
        // Keep the artifact listed with its intended identity; the verdict
        // machinery turns missing files into ERRORs at consumption time.
        let _ = e;
    }
    ArtifactRef {
        logical_path: name.to_string(),
        stored_path: stored.display().to_string(),
        sha256: hex(&sha256(bytes)),
        bytes: Some(bytes.len()),
    }
}

/// Extract a display message from a panic payload (shared by the adapter
/// boundary and its controls).
pub fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "<non-string panic payload>".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AutoError;
    use auto_test_case::contract::Stage;

    #[test]
    fn classify_io_is_infrastructure_not_run() {
        // A REAL io::Error from the OS, wrapped the way the pipeline wraps it.
        let io_err =
            std::fs::read("Z:/definitely/not/here/atc-probe.bin").expect_err("read must fail");
        let e = AutoError::Io(io_err.to_string());
        match classify_staged(&e, None) {
            StageOutcome::Infrastructure(_) => {}
            StageOutcome::Language(f) => panic!("I/O must be infrastructure, got {:?}", f),
        }
    }

    #[test]
    fn classify_recurses_nested_multiple_errors() {
        // Two-layer container: the inner leaf must still decide.
        let inner = AutoError::MultipleErrors {
            count: 1,
            plural: "error".into(),
            errors: vec![AutoError::Syntax(
                crate::error::SyntaxError::UnexpectedToken {
                    expected: "expression".into(),
                    found: "}".into(),
                    span: miette::SourceSpan::new(0usize.into(), 1usize.into()),
                },
            )],
        };
        let outer = AutoError::MultipleErrors {
            count: 1,
            plural: "error".into(),
            errors: vec![inner],
        };
        match classify_staged(&outer, None) {
            StageOutcome::Language(f) => assert_eq!(f.stage, Stage::Parse),
            StageOutcome::Infrastructure(m) => panic!("nested syntax must classify, got {}", m),
        }
    }

    #[test]
    fn classify_nested_io_is_infrastructure() {
        let inner = AutoError::MultipleErrors {
            count: 1,
            plural: "error".into(),
            errors: vec![AutoError::Io("os error 2".into())],
        };
        let outer = AutoError::MultipleErrors {
            count: 1,
            plural: "error".into(),
            errors: vec![inner],
        };
        assert!(matches!(
            classify_staged(&outer, None),
            StageOutcome::Infrastructure(_)
        ));
    }

    #[test]
    fn classify_typed_trap_code_wins_for_run_stage() {
        // A trap relayed by the staged API: message text is irrelevant, the
        // typed code is the oracle channel.
        let e = AutoError::Msg("DivisionByZero at line 3 (ip=0x002d)".into());
        match classify_staged(&e, Some("DivisionByZero")) {
            StageOutcome::Language(f) => {
                assert_eq!(f.stage, Stage::Run);
                assert_eq!(f.code.as_deref(), Some("DivisionByZero"));
            }
            StageOutcome::Infrastructure(m) => panic!("trap must classify, got {}", m),
        }
        // The SAME message without a typed trap code is infrastructure —
        // message-text matching is no longer a language channel.
        assert!(matches!(
            classify_staged(&e, None),
            StageOutcome::Infrastructure(_)
        ));
    }

    #[test]
    fn panic_message_extracts_strings() {
        let s: Box<dyn std::any::Any + Send> = Box::new("boom".to_string());
        assert_eq!(panic_message(&s), "boom");
        let n: Box<dyn std::any::Any + Send> = Box::new(42u8);
        assert_eq!(panic_message(&n), "<non-string panic payload>");
    }
}
