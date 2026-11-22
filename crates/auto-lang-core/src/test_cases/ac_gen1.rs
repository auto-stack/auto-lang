//! PLAN-1053 `ac-aac-gen1/native-run-v1` recipe adapter (probe/run seam):
//! the Gen1 native AAC compiler as the corpus AC runner.
//!
//! The case's Auto source rides the whole Gen1 chain — the prepared
//! `aac1053-compile` exe (the whole-subject Auto compiler, mir mode) ->
//! ABC -> the 1051 `ace-aac-bridge` machine service -> parent-side link ->
//! the real PE run — inside the bounded `gen1-worker` single-exchange
//! process (`gen1worker/1`). Judgement is entirely parent-side against
//! the corpus row's own oracle (the exit code; the worker never sees the
//! expected outcomes). Pre-backend pipeline refusals (frozen stage/code
//! reports) are observed as named refusals, never silent empty successes.
//!
//! Tool identity: the worker admits the compile exe and the bridge tool
//! from the `AAC1053_GEN1_DIR` preparation manifests (digest-pinned) on
//! every request; the adapter locates/builds the worker binary the same
//! way the mn recipe locates its worker (`AC_GEN1_WORKER` env override
//! first, then the ace workspace build under containment).

use std::path::{Path, PathBuf};
use std::process::Command;

use auto_test_case::compare::Mismatch;
use auto_test_case::contract::{CheckSpec, EntryMode};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};

use super::avm::Verdict;
use super::materialize;

pub const GEN1_RECIPE: &str = "ac-aac-gen1/native-run-v1";

/// Locate (and on first use build) the gen1-worker binary. Env override
/// first (read on EVERY call — the fault-injection seam stays injectable);
/// otherwise find `experimental/ace/tests/aac-gen1` above this crate's
/// source root and build its worker under the contained, bounded build
/// discipline (debug profile — the daily lane; same shape as the av
/// worker build).
pub fn worker_binary() -> Result<PathBuf, String> {
    if std::env::var("AUTO_TEST_DOMAIN").as_deref() == Ok("legacy") {
        return Err(
            "ACE worker requested by a legacy test group; explicitly select the ACE group".into(),
        );
    }
    if let Ok(env_path) = std::env::var("AC_GEN1_WORKER") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!("AC_GEN1_WORKER points at a missing file: {env_path}"));
    }
    static CACHE: std::sync::OnceLock<Result<PathBuf, String>> = std::sync::OnceLock::new();
    CACHE.get_or_init(locate_worker).clone()
}

fn locate_worker() -> Result<PathBuf, String> {
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    let crate_dir = loop {
        let Some(d) = dir else {
            return Err(
                "gen1 worker not located: no experimental/ace/tests/aac-gen1 above the crate and no AC_GEN1_WORKER override".to_string(),
            );
        };
        if d.join("experimental/ace/tests/aac-gen1/Cargo.toml").is_file() {
            break d.join("experimental/ace/tests/aac-gen1");
        }
        dir = d.parent();
    };
    let exe = crate_dir
        .join("../../target/debug/gen1-worker.exe");
    let cargo = std::env::var("AC_GEN1_BUILD_CARGO").unwrap_or_else(|_| env!("CARGO").to_string());
    let mut cmd = Command::new(&cargo);
    cmd.args([
        "build",
        "--offline",
        "-p",
        "ace-aac-gen1-conformance",
        "--bin",
        "gen1-worker",
    ])
    .current_dir(&crate_dir)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(super::sema::containment::CREATE_SUSPENDED);
    }
    let output = cmd
        .output()
        .map_err(|e| format!("spawn cargo for the gen1 worker: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "gen1 worker build failed: {} {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !exe.is_file() {
        return Err(format!("gen1 worker build produced no binary at {exe:?}"));
    }
    Ok(exe)
}

/// Run one `ac-aac-gen1/native-run-v1` check; `Err` means infrastructure
/// failure before the backend ran.
pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
    project_dir: &Path,
    artifact_dir: &Path,
    _run_dir: &Path,
) -> Result<Verdict, String> {
    if check.options.probe_recipe.as_deref() != Some(GEN1_RECIPE) {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    if case.spec.entry == EntryMode::Module {
        return Err("ac-aac-gen1/native-run-v1 supports program entry only".to_string());
    }
    if case.spec.source.syntax != auto_test_case::contract::Syntax::Auto {
        return Err(format!(
            "{} consumes .at sources; syntax {:?} is not accepted",
            GEN1_RECIPE,
            case.spec.source.syntax.as_str()
        ));
    }
    // Single-file closure only in this first lane (project mode is the
    // named follow-up); a multi-file case is an admission error, never a
    // silent reduction.
    if case.files.len() != 1 {
        return Err(format!(
            "{} currently consumes exactly one source file ({} declared)",
            GEN1_RECIPE,
            case.files.len()
        ));
    }
    let prep_dir = std::env::var("AAC1053_GEN1_DIR")
        .map_err(|_| "set AAC1053_GEN1_DIR to the prepared Gen1 tool directory".to_string())?;
    if !Path::new(&prep_dir).is_absolute() {
        return Err("AAC1053_GEN1_DIR must be absolute".to_string());
    }
    materialize::materialize_project(case, project_dir)?;
    let entry = project_dir.join(&case.entry_path);
    let worker = worker_binary()?;
    let request = serde_json::json!({
        "schema": "gen1worker/1",
        "nonce": format!("{}:{}", case.spec.id, check.id),
        "prep_dir": prep_dir,
        "source_path": entry,
        "entry": "d_main",
        "budget_ms": 8000,
    });
    let mut cmd = Command::new(&worker);
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(super::sema::containment::CREATE_SUSPENDED);
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn gen1 worker: {e}"))?;
    #[cfg(windows)]
    let _job = super::sema::containment::JobGuard::for_child(&child)
        .ok_or_else(|| "containment unavailable: refusing an uncontained gen1 worker".to_string())?;
    #[cfg(windows)]
    super::sema::containment::resume_suspended_process(&child)
        .map_err(|e| format!("resume the contained gen1 worker: {e}"))?;
    use std::io::Write;
    let stdin = child
        .stdin
        .as_mut()
        .ok_or("gen1 worker stdin not piped")?;
    stdin
        .write_all(format!("{request}\n").as_bytes())
        .map_err(|e| format!("write gen1 worker request: {e}"))?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("wait gen1 worker: {e}"))?;
    std::fs::create_dir_all(artifact_dir).ok();
    std::fs::write(
        artifact_dir.join("gen1-response.json"),
        &output.stdout,
    )
    .ok();
    if !output.status.success() {
        return Ok(Verdict::error(format!(
            "gen1 worker exited {}: {}{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    let Ok(resp) =
        serde_json::from_slice::<serde_json::Value>(output.stdout.as_slice().trim_ascii())
    else {
        return Ok(Verdict::error(format!(
            "gen1 worker returned a malformed response: {}",
            String::from_utf8_lossy(&output.stdout)
        )));
    };
    let outcome = &resp["outcome"];
    match outcome.get("kind").and_then(|k| k.as_str()) {
        Some("exit") => {
            let observed = outcome.get("code").and_then(|c| c.as_i64()).unwrap_or(i64::MIN) as i32;
            if let Some(want) = oracle.exit {
                if observed == want {
                    Ok(Verdict::pass())
                } else {
                    Ok(Verdict::fail(vec![Mismatch {
                        observation: "exit".into(),
                        kind: "text",
                        expected: want.to_string(),
                        actual: observed.to_string(),
                        first_diff: None,
                    }]))
                }
            } else {
                Ok(Verdict::fail(vec![Mismatch {
                    observation: "exit".into(),
                    kind: "text",
                    expected: "an exit oracle on the check".into(),
                    actual: format!("exit {observed} observed but nothing to judge against"),
                    first_diff: None,
                }]))
            }
        }
        Some("refused") => {
            // A pre-backend pipeline refusal is a named observation; it
            // satisfies an error-completion oracle and fails an exit one.
            let head = outcome
                .get("report_head")
                .and_then(|h| h.as_str())
                .unwrap_or("")
                .to_string();
            if let Some(want) = oracle.exit {
                Ok(Verdict::fail(vec![Mismatch {
                    observation: "exit".into(),
                    kind: "text",
                    expected: format!("exit {want}"),
                    actual: format!("pipeline refused before the backend: {head}"),
                    first_diff: None,
                }]))
            } else {
                // An error-completion oracle is satisfied by the named
                // refusal (detail preserved in the artifact response).
                let _ = head;
                Ok(Verdict::pass())
            }
        }
        Some("failure") => Ok(Verdict::error(format!(
            "gen1 chain failure {}: {}",
            outcome
                .get("code")
                .and_then(|c| c.as_str())
                .unwrap_or("unknown"),
            outcome
                .get("detail")
                .and_then(|d| d.as_str())
                .unwrap_or("")
        ))),
        other => Ok(Verdict::error(format!(
            "gen1 worker returned an unknown outcome: {other:?}"
        ))),
    }
}
