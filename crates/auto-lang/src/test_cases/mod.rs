//! TestCase runner orchestration (Plan 768): suite → resolved cases →
//! adapter dispatch → per-check outcomes with identity binding.
//!
//! Scratch discipline: every run gets a unique directory; every case/check
//! materializes inputs and writes observations/diffs ONLY inside its own
//! subdirectory. Fixtures and oracles are never written to, and the
//! process-global cwd/env are never touched.
//!
//! Product wiring: a passing check's artifacts are registered as products;
//! a check whose `product_from` producer did not pass is BLOCKED (its own
//! body never runs), which fails the gate without inventing a verdict.

use std::path::Path;

use auto_test_case::admission::Limits;
use auto_test_case::contract::{CaseSpec, CheckSpec};
use auto_test_case::discovery::{load_suite, CaseFile, LoadedSuite};
use auto_test_case::ir::{resolve_case, ResolvedCase};
use auto_test_case::report::{CheckOutcome, IdentityHashes, RunReport, Status};
use auto_test_case::sha256::{hex, sha256};
use auto_test_case::suite::apply_defaults;
use miette::Diagnostic;

pub mod a2r;
pub mod avm;
pub mod materialize;

/// Classified failure of one backend execution (stage is variant-derived;
/// code is a miette diagnostic code for parse/type, the internal VMError
/// variant name for run-stage traps).
pub struct Failure {
    pub stage: auto_test_case::contract::Stage,
    pub code: Option<String>,
    pub message: String,
}

/// Variant-based stage classification shared by the adapters (Plan 768
/// feasibility §3): `Lexer*/Syntax*` → parse, `Type/Name/Comptime` → type
/// (both frequently wrapped in `MultipleErrors`; the first error decides),
/// `Msg` → run with the VMError variant name as code (`auto_gate_` prefix →
/// gate, classified parse), `MsgWithSource` → type/link boundary.
pub fn classify(e: &crate::error::AutoError) -> Failure {
    use crate::error::AutoError;
    use auto_test_case::contract::Stage;
    fn first(e: &AutoError) -> &AutoError {
        match e {
            AutoError::MultipleErrors { errors, .. } => errors.first().unwrap_or(e),
            other => other,
        }
    }
    let e = first(e);
    let message = e.to_string();
    match e {
        AutoError::Lexer(_)
        | AutoError::LexerWithSource(_)
        | AutoError::Syntax(_)
        | AutoError::SyntaxWithSource(_) => Failure {
            stage: Stage::Parse,
            code: e.code().map(|c| c.to_string()),
            message,
        },
        AutoError::Type(_) | AutoError::Name(_) | AutoError::Comptime(_) => Failure {
            stage: Stage::Type,
            code: e.code().map(|c| c.to_string()),
            message,
        },
        AutoError::Msg(m) => {
            if m.starts_with("auto_gate_") {
                Failure {
                    stage: Stage::Parse,
                    code: Some(m.split(':').next().unwrap_or("").to_string()),
                    message,
                }
            } else {
                // Internal VMError debug text, e.g. `DivisionByZero at line 3
                // (ip=0x002d)` — the variant name is the stable code.
                let code: String = m
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                Failure {
                    stage: Stage::Run,
                    code: if code.is_empty() { None } else { Some(code) },
                    message,
                }
            }
        }
        AutoError::MsgWithSource(_) => Failure {
            stage: Stage::Type,
            code: e.code().map(|c| c.to_string()),
            message,
        },
        _ => Failure {
            stage: Stage::Run,
            code: None,
            message,
        },
    }
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
                report
                    .coverage_gaps
                    .push((case_id.clone(), req.as_str().to_string()));
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
    // `file:` payload / expected-artifact references resolve against the
    // suite-declared fixture base (imports keep legacy files in place).
    let ref_base = loaded.fixture_base.clone();
    let resolved = match resolve_case(spec.clone(), &case_file.doc, &ref_base, limits) {
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

    // Product registry: logical paths produced by PASSING checks, usable as
    // product_from inputs by later checks of the same case.
    let mut products: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();

    for check in &spec.checks {
        let (check, _origins) = apply_defaults(check, &loaded.suite.defaults);

        let started = std::time::Instant::now();
        let mut outcome = if let Some(skip) = spec.status_skip.as_ref().or(check.skip.as_ref()) {
            let mut o = base_outcome(loaded, spec, &check, Status::Skip, &identity_base(&check));
            o.detail = Some(format!("skip: {} (issue {})", skip.reason, skip.issue));
            o
        } else if let Some(upstream) = &check.product_from {
            match products.get(upstream) {
                None => {
                    let mut o = base_outcome(
                        loaded,
                        spec,
                        &check,
                        Status::Blocked,
                        &identity_base(&check),
                    );
                    o.detail = Some(format!(
                        "upstream check {:?} produced no usable product (not passing) — BLOCKED",
                        upstream
                    ));
                    o
                }
                Some(made) => {
                    let mut o = dispatch_check(
                        loaded,
                        spec,
                        &resolved,
                        &check,
                        &ref_base,
                        run_dir,
                        &identity_base(&check),
                        started,
                    );
                    o.detail = Some(format!("consumed products from {:?}: {:?}", upstream, made));
                    o
                }
            }
        } else {
            dispatch_check(
                loaded,
                spec,
                &resolved,
                &check,
                &ref_base,
                run_dir,
                &identity_base(&check),
                started,
            )
        };
        outcome.duration_ms = outcome.duration_ms.max(started.elapsed().as_millis());
        if outcome.status == Status::Pass {
            products.insert(
                check.id.clone(),
                outcome
                    .artifacts
                    .iter()
                    .map(|a| a.logical_path.clone())
                    .collect(),
            );
        }
        report.results.push(outcome);
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_check(
    loaded: &LoadedSuite,
    spec: &CaseSpec,
    resolved: &ResolvedCase,
    check: &CheckSpec,
    case_root: &Path,
    run_dir: &Path,
    identity: &IdentityHashes,
    started: std::time::Instant,
) -> CheckOutcome {
    let case_dir = run_dir.join(sanitize(&spec.id));
    let project_dir = case_dir.join("project");
    let artifact_dir = case_dir.join(sanitize(&check.id));
    let mut o = base_outcome(loaded, spec, check, Status::Error, identity);
    o.duration_ms = started.elapsed().as_millis();
    let dispatched = match (check.adapter, check.action) {
        (auto_test_case::contract::Adapter::Avm, auto_test_case::contract::Action::Run) => {
            avm::run_check(resolved, check, case_root, &project_dir, &artifact_dir)
        }
        (auto_test_case::contract::Adapter::A2r, auto_test_case::contract::Action::Emit) => {
            a2r::run_check(resolved, check, case_root, &project_dir, &artifact_dir)
        }
        _ => Err("unreachable adapter/action (contract decode enforces the matrix)".to_string()),
    };
    match dispatched {
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
    o
}

fn fingerprint<T: std::fmt::Debug>(v: &T) -> String {
    hex(&sha256(format!("{:?}", v).as_bytes()))
}

fn execution_key(loaded: &LoadedSuite, case_id: &str, check_id: &str) -> String {
    format!(
        "{}:{}:{}:{}",
        loaded.suite.id, loaded.suite_sha256, case_id, check_id
    )
}

fn sanitize(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
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

fn error_outcome(
    loaded: &LoadedSuite,
    spec: &CaseSpec,
    check: &CheckSpec,
    detail: &str,
) -> CheckOutcome {
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
