//! TestCase runner orchestration (Plan 768): suite → resolved cases →
//! adapter dispatch → per-check outcomes with identity binding.
//!
//! Scratch discipline: every run gets a unique directory; every case/check
//! materializes inputs and writes observations/diffs ONLY inside its own
//! subdirectory. Fixtures and oracles are never written to, and the
//! process-global cwd/env are never touched.

use std::path::{Path, PathBuf};

use auto_test_case::admission::Limits;
use auto_test_case::contract::{CaseSpec, CheckSpec};
use auto_test_case::discovery::{load_suite, CaseFile, LoadedSuite};
use auto_test_case::ir::{resolve_case, ResolvedCase};
use auto_test_case::report::{CheckOutcome, IdentityHashes, RunReport, Status};
use auto_test_case::sha256::{hex, sha256};
use auto_test_case::suite::apply_defaults;

pub mod a2r;
pub mod avm;

/// Everything an adapter needs to execute one check.
pub struct CheckJob<'a> {
    pub case: &'a ResolvedCase,
    pub check: &'a CheckSpec,
    /// Materialized input project root (files already written).
    pub project_dir: PathBuf,
    /// This check's private artifact directory.
    pub artifact_dir: PathBuf,
}

/// Write the case's virtual project into `dir` (creating parents).
pub fn materialize_project(case: &ResolvedCase, dir: &Path) -> Result<(), String> {
    for file in &case.files {
        let target = dir.join(file.virtual_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("scratch mkdir: {}", e))?;
        }
        std::fs::write(&target, &file.bytes)
            .map_err(|e| format!("scratch write {}: {}", target.display(), e))?;
    }
    Ok(())
}

fn fingerprint<T: std::fmt::Debug>(v: &T) -> String {
    hex(&sha256(format!("{:?}", v).as_bytes()))
}

fn execution_key(loaded: &LoadedSuite, case_id: &str, check_id: &str) -> String {
    format!("{}:{}:{}:{}", loaded.suite.id, loaded.suite_sha256, case_id, check_id)
}

/// Run every check of every case of a suite; returns the full report.
///
/// `out_dir` receives a unique run directory (`run-<millis>-<pid>`); the
/// report's artifact references point into it.
pub fn run_suite(suite_path: &Path, out_dir: &Path, limits: &Limits) -> Result<RunReport, String> {
    let loaded = load_suite(suite_path, limits).map_err(|e| format!("suite load: {}", e))?;
    let run_id = format!(
        "run-{}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        std::process::id()
    );
    let run_dir = out_dir.join(&run_id);
    std::fs::create_dir_all(&run_dir).map_err(|e| format!("scratch root: {}", e))?;

    let required: Vec<String> = loaded
        .suite
        .required_adapters
        .iter()
        .map(|a| a.as_str().to_string())
        .collect();
    let mut report = RunReport::new(&loaded.suite.id, &loaded.suite_sha256, &required);
    report.load_errors = loaded.errors.iter().map(|e| e.to_string()).collect();

    for case_file in &loaded.cases {
        for spec in case_file.specs.iter() {
            run_case(&loaded, case_file, spec, &run_dir, limits, &mut report);
        }
    }

    // Required coverage: every selected case must hold a non-skipped check
    // for each required adapter; gaps are contract errors failing the gate.
    let mut case_adapters: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    let mut selected_cases: Vec<String> = Vec::new();
    for r in &report.results {
        if !selected_cases.contains(&r.case_id) {
            selected_cases.push(r.case_id.clone());
        }
        if r.status != Status::Skip {
            case_adapters
                .entry(r.case_id.clone())
                .or_default()
                .insert(r.adapter.clone());
        }
    }
    for case_id in &selected_cases {
        for req in &loaded.suite.required_adapters {
            let declared = case_adapters
                .get(case_id)
                .map(|set| set.contains(req.as_str()))
                .unwrap_or(false);
            if !declared {
                report.coverage_gaps.push((case_id.clone(), req.as_str().to_string()));
            }
        }
    }

    Ok(report)
}

fn run_case(
    loaded: &LoadedSuite,
    case_file: &CaseFile,
    spec: &CaseSpec,
    run_dir: &Path,
    limits: &Limits,
    report: &mut RunReport,
) {
    let case_root = case_file.path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let resolved = match resolve_case(spec.clone(), &case_file.doc, &case_root, limits) {
        Ok(r) => r,
        Err(e) => {
            // Resolution failure is a contract/infrastructure error for every
            // declared check — recorded per check so inventory stays complete.
            for check in &spec.checks {
                report.results.push(error_outcome(
                    loaded,
                    spec,
                    check,
                    &format!("payload resolution failed: {}", e),
                ));
            }
            return;
        }
    };

    let identity_base = |check: &CheckSpec| IdentityHashes {
        suite: loaded.suite_sha256.clone(),
        case_file: case_file.sha256.clone(),
        source: resolved.source_sha256.clone(),
        oracle: fingerprint(&check.expect),
        contract: fingerprint(spec),
    };

    for check in &spec.checks {
        let (check, _origins) = apply_defaults(check, &loaded.suite.defaults);

        let started = std::time::Instant::now();
        let outcome = if let Some(skip) = spec.status_skip.as_ref().or(check.skip.as_ref()) {
            let mut o = base_outcome(loaded, spec, &check, Status::Skip, &identity_base(&check));
            o.detail = Some(format!("skip: {} (issue {})", skip.reason, skip.issue));
            o
        } else {
            let case_dir = run_dir.join(sanitize(&spec.id));
            let project_dir = case_dir.join("project");
            let artifact_dir = case_dir.join(sanitize(&check.id));
            let mut o = base_outcome(loaded, spec, &check, Status::Error, &identity_base(&check));
            o.duration_ms = started.elapsed().as_millis();
            match dispatch(&resolved, &check, &project_dir, &artifact_dir) {
                Ok(verdict) => {
                    o.status = verdict.status;
                    o.mismatches = verdict.mismatches;
                    o.detail = verdict.detail;
                    o.artifacts = verdict.artifacts;
                }
                Err(e) => {
                    o.status = Status::Error;
                    o.detail = Some(e);
                }
            }
            o.duration_ms = o.duration_ms.max(started.elapsed().as_millis());
            o
        };
        report.results.push(outcome);
    }
}

fn dispatch(
    case: &ResolvedCase,
    check: &CheckSpec,
    project_dir: &Path,
    artifact_dir: &Path,
) -> Result<avm::Verdict, String> {
    match (check.adapter, check.action) {
        (auto_test_case::contract::Adapter::Avm, auto_test_case::contract::Action::Run) => {
            avm::run_check(case, check, project_dir, artifact_dir)
        }
        (auto_test_case::contract::Adapter::A2r, auto_test_case::contract::Action::Emit) => {
            a2r::run_check(case, check, project_dir, artifact_dir)
        }
        _ => Err("unreachable adapter/action (contract decode enforces the matrix)".to_string()),
    }
}

fn sanitize(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') { c } else { '_' })
        .collect()
}

fn base_outcome(
    loaded: &LoadedSuite,
    spec: &CaseSpec,
    check: &CheckSpec,
    status: Status,
    identity: &IdentityHashes,
) -> CheckOutcome {
    CheckOutcome {
        suite_id: loaded.suite.id.clone(),
        case_id: spec.id.clone(),
        check_id: check.id.clone(),
        execution_key: execution_key(loaded, &spec.id, &check.id),
        adapter: check.adapter.as_str().to_string(),
        action: check.action.as_str().to_string(),
        tier: check.tier.clone(),
        status,
        mismatches: Vec::new(),
        detail: None,
        identity: identity.clone(),
        artifacts: Vec::new(),
        duration_ms: 0,
    }
}

fn error_outcome(loaded: &LoadedSuite, spec: &CaseSpec, check: &CheckSpec, detail: &str) -> CheckOutcome {
    CheckOutcome {
        suite_id: loaded.suite.id.clone(),
        case_id: spec.id.clone(),
        check_id: check.id.clone(),
        execution_key: execution_key(loaded, &spec.id, &check.id),
        adapter: check.adapter.as_str().to_string(),
        action: check.action.as_str().to_string(),
        tier: check.tier.clone(),
        status: Status::Error,
        mismatches: Vec::new(),
        detail: Some(detail.to_string()),
        identity: IdentityHashes::default(),
        artifacts: Vec::new(),
        duration_ms: 0,
    }
}
