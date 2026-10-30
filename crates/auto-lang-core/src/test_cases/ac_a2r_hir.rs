//! PLAN-892 `ac-a2r/hir-scalar-emit-v1` recipe adapter (`probe/run` seam,
//! the 796 registered-recipe contract): the case source runs through the
//! REAL A2X chain — checked core/scalar frontend → target-neutral semantic
//! view → Rust provider → full actual file set — produced by the PREBUILT
//! `a2x-worker` tool, and judged entirely PARENT-SIDE by byte-comparing
//! the actual set against the reviewed golden sidecars.
//!
//! Plan §5.5 daily-channel contracts implemented here:
//! * spawn-zero: this adapter spawns EXACTLY ONE process — the prebuilt
//!   worker (the trusted tool identity; a case can never name an
//!   executable). rustc/cargo/target-Rust programs are NEVER spawned on
//!   the daily path; the worker's self-reported `spawned_rustc` must be 0
//!   and the parent verifies it;
//! * NO lazy toolchain builds: a missing worker is a controlled infra
//!   ERROR carrying the explicit maintenance command — stricter than the
//!   older adapters' first-use cargo build (the maintenance build is an
//!   observable operation owned by the verify script/tests, never the
//!   case timing path);
//! * goldens are NEVER an input to production: the worker receives only
//!   the source/profile/entry/caps/nonce; expected bytes arrive through
//!   the oracle sidecars like every other adapter;
//! * identity: the worker's protocol/recipe echo and manifest hash are
//!   re-verified parent-side from the emitted files (trust but verify).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use auto_test_case::compare::{compare_bytes, Mismatch};
use auto_test_case::contract::{CheckSpec, Completion, ProbeStep, Syntax};
use auto_test_case::ir::{ResolvedCase, ResolvedOracle};
use auto_test_case::report::ArtifactRef;
use auto_test_case::sha256::{hex, sha256};

use auto_test_case::report::Status;
use super::avm::Verdict;

pub const A2R_HIR_RECIPE: &str = "ac-a2r/hir-scalar-emit-v1";
pub const A2X_WORKER_PROTOCOL: &str = "a2xworker/1";
pub const A2X_WORKER_ENV: &str = "AC_A2X_HIR_WORKER";

/// Locate the PREBUILT a2x worker. Env override first (re-read every
/// call); otherwise the conventional release location next to the repo
/// root. NEVER builds — a missing worker is infra, with the maintenance
/// command in the error text.
pub fn worker_binary() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var(A2X_WORKER_ENV) {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!("{A2X_WORKER_ENV} points at a missing file: {env_path}"));
    }
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    while let Some(d) = dir {
        // the conventional prebuilt locations: the repo-root target (the
        // verify script copies there) and the a2x workspace's own release
        // output (where `cargo build --manifest-path .../auto-a2x` puts it)
        for exe in [
            d.join("target").join("release").join(a2x_worker_exe_name()),
            d.join("experimental")
                .join("auto-a2x")
                .join("target")
                .join("release")
                .join(a2x_worker_exe_name()),
        ] {
            if exe.is_file() {
                return Ok(exe);
            }
        }
        dir = d.parent();
    }
    Err(format!(
        "prebuilt a2x worker not found (no {A2X_WORKER_ENV} override, no <repo>/target/release/{}); \
         maintenance build (observable, outside case timing): \
         cargo build --release --manifest-path experimental/auto-a2x/Cargo.toml -p a2x-probe --bin a2x-worker",
        a2x_worker_exe_name()
    ))
}

fn a2x_worker_exe_name() -> &'static str {
    if cfg!(windows) {
        "a2x-worker.exe"
    } else {
        "a2x-worker"
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
    if case.spec.source.syntax != Syntax::Auto {
        return Err(format!(
            "{A2R_HIR_RECIPE} consumes auto source files; syntax {:?} is not accepted",
            case.spec.source.syntax.as_str()
        ));
    }
    if check.options.probe_recipe.as_deref() != Some(A2R_HIR_RECIPE) {
        return Err("probe check with an unregistered recipe (contract violation)".into());
    }
    let binding = check
        .options
        .probe_a2x_hir
        .as_ref()
        .ok_or("probe check without an a2x_hir binding (contract violation)")?;
    let steps = check
        .options
        .probe_steps
        .as_ref()
        .ok_or("probe check without steps (contract violation)")?;
    if steps.len() != 1 || !matches!(steps[0], ProbeStep::Run) {
        return Err(format!(
            "{A2R_HIR_RECIPE} takes exactly one `run` step (the daily emit); other step shapes arrive with their own contract"
        ));
    }
    let source_bytes = case
        .files
        .first()
        .map(|f| f.bytes.clone())
        .ok_or_else(|| "a2x_hir adapter needs the resolved entry input".to_string())?;

    let worker = worker_binary()?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("artifact dir: {e}"))?;
    let source_path = artifact_dir.join("source.at");
    std::fs::write(&source_path, &source_bytes)
        .map_err(|e| format!("stage source: {e}"))?;
    let actual_dir = artifact_dir.join("actual");
    let nonce = hex(&sha256(
        format!("{}|{}|a2x-hir", std::process::id(), started.elapsed().as_nanos()).as_bytes(),
    ));

    let request = serde_json::json!({
        "op": "daily-emit",
        "source_path": source_path,
        "profile": binding.profile,
        "entry": binding.entry,
        "caps": binding.caps,
        "out_dir": actual_dir,
        "nonce": nonce,
    });

    // The ONLY process this adapter ever spawns (plan §5.5 spawn-zero).
    let out = Command::new(&worker)
        .arg(request.to_string())
        .output()
        .map_err(|e| format!("spawn a2x worker {}: {e}", worker.display()))?;
    if !out.status.success() {
        // A worker refusal is INFRA, never a language verdict.
        return Ok(Verdict {
            status: Status::Error,
            detail: Some(format!(
                "a2x worker failed (exit {:?}): {}",
                out.status.code(),
                String::from_utf8_lossy(&out.stdout).trim()
            )),
            ..Verdict::pass()
        });
    }
    let resp: serde_json::Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("worker response not json: {e}"))?;
    if resp["status"] != "ok" {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some(format!("worker controlled failure: {}", resp["detail"])),
            ..Verdict::pass()
        });
    }
    // Identity echo (trust but verify).
    if resp["protocol"].as_str() != Some(A2X_WORKER_PROTOCOL)
        || resp["recipe"].as_str() != Some(A2R_HIR_RECIPE)
    {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some(format!(
                "worker identity mismatch: protocol={:?} recipe={:?}",
                resp["protocol"], resp["recipe"]
            )),
            ..Verdict::pass()
        });
    }
    if resp["spawned_rustc"].as_i64() != Some(0) {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some("worker reported nonzero rustc spawns on the daily path".into()),
            ..Verdict::pass()
        });
    }
    // Parent-side manifest re-hash: the reported digest must match the
    // emitted manifest bytes on disk.
    let manifest_bytes =
        std::fs::read(actual_dir.join("manifest.json")).map_err(|e| format!("read manifest: {e}"))?;
    let manifest_sha = hex(&sha256(&manifest_bytes));
    if resp["manifest_sha256"].as_str() != Some(manifest_sha.as_str()) {
        return Ok(Verdict {
            status: Status::Error,
            detail: Some("worker manifest hash disagrees with the emitted file".into()),
            ..Verdict::pass()
        });
    }

    match &oracle.completion {
        Completion::Success => {
            let mut mismatches = Vec::new();
            let mut refs = Vec::new();
            let mut expected_paths: Vec<String> = Vec::new();
            for want in &oracle.artifacts {
                expected_paths.push(want.path.clone());
                let path = actual_dir.join(&want.path);
                match std::fs::read(&path) {
                    Err(_) => mismatches.push(Mismatch {
                        observation: format!("artifact {}", want.path),
                        kind: "missing",
                        expected: format!("{} bytes", want.bytes.len()),
                        actual: "absent".to_string(),
                        first_diff: None,
                    }),
                    Ok(actual) => {
                        if let Some(m) = compare_bytes(
                            &want.bytes,
                            &actual,
                            want.compare,
                            &format!("artifact {}", want.path),
                        ) {
                            mismatches.push(m);
                        }
                        refs.push(ArtifactRef {
                            logical_path: want.path.clone(),
                            stored_path: path.display().to_string(),
                            sha256: hex(&sha256(&actual)),
                            bytes: Some(actual.len()),
                        });
                    }
                }
            }
            // Undeclared extra products are a mismatch (Design 35 §6.2):
            // the actual set must equal the declared set exactly.
            let extras: Vec<String> = std::fs::read_dir(&actual_dir)
                .map(|rd| {
                    rd.filter_map(|e| e.ok())
                        .filter(|e| e.path().is_file())
                        .filter(|e| {
                            let name = e.file_name().to_string_lossy().to_string();
                            !expected_paths.contains(&name) && name != "source.at"
                        })
                        .map(|e| e.file_name().to_string_lossy().to_string())
                        .collect()
                })
                .unwrap_or_default();
            if !extras.is_empty() {
                mismatches.push(Mismatch {
                    observation: "artifacts".into(),
                    kind: "extra",
                    expected: format!("exactly {:?}", expected_paths),
                    actual: format!("extra files: {:?}", extras),
                    first_diff: None,
                });
            }
            let status = if mismatches.is_empty() {
                Status::Pass
            } else {
                Status::Fail
            };
            let mut v = Verdict {
                status,
                mismatches,
                detail: None,
                artifacts: refs,
                native: None,
            };
            if v.status == Status::Pass {
                v.detail = Some(format!(
                    "a2x-hir daily emit: 4-file actual set byte-identical to the reviewed golden (manifest sha256 {manifest_sha}, worker spawn only)"
                ));
            }
            Ok(v)
        }
        other => Err(format!(
            "{A2R_HIR_RECIPE} daily checks declare success completions only (got {:?}); the trap/diagnostic axes are certification-leg observations",
            other
        )),
    }
}
