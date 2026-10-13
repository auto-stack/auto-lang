//! a2r adapter (`a2r/emit`): emits real Rust through the production
//! `transpile_rust_with_source_dir` / `transpile_rust_project` APIs and
//! judges the artifact manifest strictly.
//!
//! Plan 768 feasibility §4 rulings implemented here:
//! - emission runs on a dedicated big-stack thread (the emit path is
//!   stack-hungry in debug builds — it overflowed an 8 MiB main thread in
//!   the T-01 probe);
//! - `use.c` manifests are pre-verified: builtin-embedded OR present in the
//!   materialized project, otherwise the check is a contract ERROR (the
//!   emitter silently ignores missing manifests, which would change output
//!   without failing);
//! - `module_name` is inert for emitted bytes (legacy metadata parity only);
//! - single-file emission yields one logical artifact `<module_name>.rs`;
//!   project emission yields the filename-keyed map including `Cargo.toml`;
//! - actual artifacts are persisted under the check's own scratch dir and
//!   are NEVER taken from the expected sidecars; product consumers read the
//!   persisted actuals (no Rust compilation happens in this pilot).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use auto_test_case::compare::{compare_bytes, match_message, Mismatch};
use auto_test_case::contract::{CheckSpec, Completion, Stage};
use auto_test_case::ir::ResolvedCase;
use auto_test_case::report::ArtifactRef;
use auto_test_case::sha256::{hex, sha256};

use super::avm::Verdict;
use super::materialize;
use super::{classify, Failure};

/// Emit stack size: transpile paths need more than a default thread stack
/// in debug builds (128 MiB mirrors the T-01 probe evidence).
const EMIT_STACK_BYTES: usize = 128 * 1024 * 1024;

pub fn run_check(
    case: &ResolvedCase,
    check: &CheckSpec,
    case_root: &Path,
    project_dir: &Path,
    artifact_dir: &Path,
) -> Result<Verdict, String> {
    std::fs::create_dir_all(project_dir).map_err(|e| format!("scratch: {}", e))?;
    std::fs::create_dir_all(artifact_dir).map_err(|e| format!("scratch: {}", e))?;
    materialize::materialize_project(case, project_dir)?;

    let entry_abs = project_dir.join(case.entry_path.replace('/', std::path::MAIN_SEPARATOR_STR));
    let entry_bytes = case
        .files
        .iter()
        .find(|f| f.virtual_path == case.entry_path)
        .map(|f| f.bytes.clone())
        .ok_or_else(|| format!("entry file {:?} missing", case.entry_path))?;
    let entry_str = String::from_utf8(entry_bytes)
        .map_err(|_| "non-UTF-8 source cannot be emitted by the a2r adapter".to_string())?;

    // `use.c` pre-verification across every project source file.
    for file in &case.files {
        if !(file.virtual_path.ends_with(".at") || file.virtual_path.ends_with(".as")) {
            continue;
        }
        if let Ok(text) = String::from_utf8(file.bytes.clone()) {
            for raw in text.lines().filter(|l| l.trim_start().starts_with("use.c")) {
                let target = raw
                    .trim_start()
                    .trim_start_matches("use.c")
                    .trim()
                    .trim_start_matches('<')
                    .trim_end_matches('>')
                    .trim_matches('"')
                    .to_string();
                if target.is_empty() {
                    continue;
                }
                let builtin = crate::vm::ffi::c_ffi::load_builtin_manifest(&target).is_some();
                let on_disk = !target.contains("..")
                    && project_dir
                        .join(target.replace('/', std::path::MAIN_SEPARATOR_STR))
                        .is_file();
                if !builtin && !on_disk {
                    return Err(format!(
                        "`use.c <{}>` in {}: manifest is neither builtin nor present in the materialized project — the emitter would silently skip it (contract error, not a golden mismatch)",
                        target, file.virtual_path
                    ));
                }
            }
        }
    }

    let module_name = check
        .options
        .module_name
        .clone()
        .unwrap_or_else(|| default_module_name(&case.entry_path));
    let use_project_mode =
        case.spec.files.len() > 1 || case.spec.entry == auto_test_case::contract::EntryMode::Module;

    let emit_handle = std::thread::Builder::new()
        .stack_size(EMIT_STACK_BYTES)
        .spawn({
            let entry_path_str = entry_abs.display().to_string();
            let entry_str = entry_str.clone();
            let module_name_for_file = module_name.clone();
            let module_name = module_name.clone();
            let project_dir = project_dir.to_path_buf();
            move || -> Result<BTreeMap<String, Vec<u8>>, crate::error::AutoError> {
                if use_project_mode {
                    let map = crate::trans::rust::transpile_rust_project(&entry_path_str)?;
                    Ok(map
                        .into_iter()
                        .map(|(k, (bytes, _maps))| (k, bytes))
                        .collect())
                } else {
                    let mut sink = crate::trans::rust::transpile_rust_with_source_dir(
                        &project_dir,
                        module_name,
                        &entry_str,
                    )?;
                    let bytes = sink.done()?.clone();
                    Ok(BTreeMap::from([(
                        format!("{}.rs", module_name_for_file),
                        bytes,
                    )]))
                }
            }
        })
        .map_err(|e| format!("spawn emit thread: {}", e))?;
    let emit = match emit_handle.join() {
        Ok(result) => result,
        Err(_) => {
            return Ok(Verdict::error(
                "emit thread panicked — infrastructure error",
            ))
        }
    };

    let artifacts: BTreeMap<String, Vec<u8>> = match emit {
        Err(e) => {
            // Language failure on the emit path — classified by variant on
            // the caller side of the thread boundary.
            let failure = classify(&e);
            let _ = std::fs::write(artifact_dir.join("emit-error.txt"), &failure.message);
            return Ok(judge_completion_error(
                &check.expect.completion,
                &failure,
                vec![],
            ));
        }
        Ok(artifacts) => artifacts,
    };

    // Persist actuals under the check's own scratch (never the oracle tree).
    let mut refs: Vec<ArtifactRef> = Vec::new();
    for (logical, bytes) in &artifacts {
        let stored = artifact_dir
            .join("actual")
            .join(logical.replace('\\', "/").split('/').collect::<PathBuf>());
        if let Some(parent) = stored.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("actual mkdir: {}", e))?;
        }
        std::fs::write(&stored, bytes).map_err(|e| format!("actual write: {}", e))?;
        refs.push(ArtifactRef {
            logical_path: logical.clone(),
            stored_path: stored.display().to_string(),
            sha256: hex(&sha256(bytes)),
            bytes: Some(bytes.len()),
        });
    }

    match &check.expect.completion {
        Completion::Error { .. } => {
            let failure = Failure {
                stage: Stage::Run,
                code: None,
                message: "emit succeeded".to_string(),
            };
            let mut v = judge_completion_error(&check.expect.completion, &failure, Vec::new());
            v.artifacts = refs;
            Ok(v)
        }
        Completion::Success => {
            let mut mismatches = Vec::new();
            let mut expected_paths: Vec<String> = Vec::new();
            for want in &check.expect.artifacts {
                let expected_bytes = std::fs::read(
                    case_root.join(want.file.replace('/', std::path::MAIN_SEPARATOR_STR)),
                )
                .map_err(|e| format!("expected artifact {}: {}", want.file, e))?;
                expected_paths.push(want.path.clone());
                match artifacts.get(&want.path) {
                    None => mismatches.push(Mismatch {
                        observation: format!("artifact {}", want.path),
                        kind: "missing",
                        expected: format!("{} bytes", expected_bytes.len()),
                        actual: "absent".to_string(),
                        first_diff: None,
                    }),
                    Some(actual) => {
                        if let Some(m) = compare_bytes(
                            &expected_bytes,
                            actual,
                            want.compare,
                            &format!("artifact {}", want.path),
                        ) {
                            mismatches.push(m);
                        }
                    }
                }
            }
            // Reject undeclared extra products by default (Design 35 §6.2):
            // the emitted logical-path set must equal the declared set.
            let extras: Vec<String> = artifacts
                .keys()
                .filter(|k| !expected_paths.contains(k))
                .cloned()
                .collect();
            if !extras.is_empty() {
                mismatches.push(Mismatch {
                    observation: "artifacts".into(),
                    kind: "extra",
                    expected: format!("declared set {:?}", expected_paths),
                    actual: format!("undeclared extras {:?}", extras),
                    first_diff: None,
                });
            }
            let mut v = if mismatches.is_empty() {
                Verdict::pass()
            } else {
                Verdict::fail(mismatches)
            };
            v.artifacts = refs;
            Ok(v)
        }
    }
}

fn default_module_name(entry_path: &str) -> String {
    let stem = entry_path.rsplit('/').next().unwrap_or(entry_path);
    stem.trim_end_matches(".at")
        .trim_end_matches(".as")
        .to_string()
}

fn judge_completion_error(
    completion: &Completion,
    failure: &Failure,
    refs: Vec<ArtifactRef>,
) -> Verdict {
    match completion {
        Completion::Success => {
            let mut v = Verdict::fail(vec![Mismatch {
                observation: "completion".into(),
                kind: "stage",
                expected: "success".into(),
                actual: format!("emit error: {}", failure.message),
                first_diff: None,
            }]);
            v.artifacts = refs;
            v
        }
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
            let mut v = if mismatches.is_empty() {
                Verdict::pass()
            } else {
                Verdict::fail(mismatches)
            };
            v.artifacts = refs;
            v
        }
    }
}
