//! TestCase runner orchestration (Plan 768 / Plan 773 repair): suite →
//! resolved cases → adapter dispatch → per-check outcomes with identity
//! binding.
//!
//! Scratch discipline: every run gets a unique directory; every case/check
//! materializes inputs and writes observations/diffs ONLY inside its own
//! subdirectory. Fixtures and oracles are never written to, and the
//! process-global cwd/env are never touched. The output directory is
//! path-domain audited BEFORE execution (Plan 773 F-05): it may not overlap
//! the suite/fixture roots in either direction, and no reparse point may
//! sit on the write path.
//!
//! Selection (Plan 773 F-04): selectors are validated against the DECLARED
//! inventory and applied BEFORE adapter dispatch — an unselected check is
//! recorded NOT_RUN without executing (no scratch dir, no side effects), a
//! typo'd or zero-match selector is a contract error, never a green run.
//!
//! Product wiring (Plan 773 F-03): comparison status and artifact usability
//! are separate axes. A check that EXECUTED and produced artifacts registers
//! them as products even when its own oracle FAILs (a golden mismatch does
//! not cut the handoff); only a real execution failure (ERROR/no artifacts)
//! leaves nothing to consume and BLOCKs downstream consumers. Consumers are
//! driven by the internal registered test consumer over the upstream's
//! ACTUAL bytes/manifest — never by re-executing the source and never by
//! reading expected sidecars. Unimplemented public downstream consumption
//! (result/stdout/stderr observations on a consumer, error-completion
//! consumers) is an explicit ERROR, not a pass.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use auto_test_case::admission::{check_no_reparse_ancestors, validate_output_domain, Limits};
use auto_test_case::contract::{Adapter, CaseSpec, CheckSpec, Stage};
use auto_test_case::discovery::{load_suite, CaseFile, LoadedSuite};
use auto_test_case::ir::{resolve_case, resolve_oracle, ResolvedCase, ResolvedOracle};
use auto_test_case::report::{ArtifactRef, CheckOutcome, IdentityHashes, RunReport, Status};
use auto_test_case::sha256::{hex, sha256};
use auto_test_case::suite::apply_defaults;

pub mod a2r;
pub mod abi;
pub mod ac;
pub mod ac_av;
mod ac_protocol;
pub mod avm;
pub mod materialize;
pub mod sema;
pub mod stale;

/// A language failure observed at a production stage boundary (Plan 773
/// F-08): the stage is derived from the ERROR TYPE (and, for run-stage
/// traps, the typed variant code from the engine), never parsed out of the
/// human message text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub stage: Stage,
    pub code: Option<String>,
    pub message: String,
}

/// Stage classification of a backend failure (Plan 773 F-08). Infrastructure
/// failures (I/O, missing files, panics, synthetic deadlines, unclassified
/// messages) are a separate variant: they can NEVER satisfy a language
/// negative — the check becomes an infrastructure ERROR instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageOutcome {
    Language(Failure),
    Infrastructure(String),
}

/// Classify an error from a production stage boundary. `run_trap_code` is
/// the typed `VMError` variant name relayed by the staged run API (None on
/// non-VM-trap paths). `MultipleErrors` containers are unwrapped fully —
/// the first leaf error decides (nested containers included).
pub fn classify_staged(
    e: &crate::error::AutoError,
    run_trap_code: Option<&'static str>,
) -> StageOutcome {
    use crate::error::AutoError;
    use miette::Diagnostic as _;
    // Full recursion: a container inside a container still reaches its leaf.
    fn first_leaf(e: &AutoError) -> &AutoError {
        match e {
            AutoError::MultipleErrors { errors, .. } => errors
                .first()
                .map(first_leaf)
                .unwrap_or_else(|| unreachable!("MultipleErrors always carries >=1 error")),
            other => other,
        }
    }
    // The trap channel is only meaningful for the error it was relayed with;
    // a typed trap code always wins over leaf-shape guessing.
    if let Some(code) = run_trap_code {
        return StageOutcome::Language(Failure {
            stage: Stage::Run,
            code: Some(code.to_string()),
            message: e.to_string(),
        });
    }
    let e = first_leaf(e);
    let message = e.to_string();
    match e {
        AutoError::Lexer(_)
        | AutoError::LexerWithSource(_)
        | AutoError::Syntax(_)
        | AutoError::SyntaxWithSource(_) => StageOutcome::Language(Failure {
            stage: Stage::Parse,
            code: e.code().map(|c| c.to_string()),
            message,
        }),
        AutoError::Type(_) | AutoError::Name(_) | AutoError::Comptime(_) => {
            StageOutcome::Language(Failure {
                stage: Stage::Type,
                code: e.code().map(|c| c.to_string()),
                message,
            })
        }
        // Linker/source-context errors sit on the type/link boundary.
        AutoError::MsgWithSource(_) => StageOutcome::Language(Failure {
            stage: Stage::Type,
            code: e.code().map(|c| c.to_string()),
            message,
        }),
        // Typed runtime failures (miette diagnostic code is the stable id).
        AutoError::Runtime(re) => StageOutcome::Language(Failure {
            stage: Stage::Run,
            code: re.code().map(|c| c.to_string()),
            message,
        }),
        // Everything else — Io, Warning, plain Msg without a typed trap,
        // future variants — is infrastructure: file system trouble, missing
        // inputs, synthetic deadlines. An I/O message must never be
        // matchable as a language negative.
        _ => StageOutcome::Infrastructure(message),
    }
}

/// Selection applied BEFORE adapter dispatch (Plan 773 F-04). `None` fields
/// select everything; every set field must match. An adapter selector with
/// an unknown name, or a selector combination matching zero declared
/// checks, is a contract error (exit 2), never a silently green run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Selection {
    pub case: Option<String>,
    pub adapter: Option<String>,
    pub check: Option<String>,
}

impl Selection {
    fn describes(&self) -> String {
        let mut parts = Vec::new();
        if let Some(c) = &self.case {
            parts.push(format!("case={}", c));
        }
        if let Some(a) = &self.adapter {
            parts.push(format!("adapter={}", a));
        }
        if let Some(c) = &self.check {
            parts.push(format!("check={}", c));
        }
        if parts.is_empty() {
            "full run".to_string()
        } else {
            parts.join(", ")
        }
    }

    fn matches(&self, case_id: &str, check: &CheckSpec) -> bool {
        self.case.as_deref().map(|c| c == case_id).unwrap_or(true)
            && self
                .adapter
                .as_deref()
                .map(|a| a == check.adapter.as_str())
                .unwrap_or(true)
            && self.check.as_deref().map(|c| c == check.id).unwrap_or(true)
    }
}

/// Run every check of every case of a suite (full selection).
pub fn run_suite(suite_path: &Path, out_dir: &Path, limits: &Limits) -> Result<RunReport, String> {
    run_suite_selected(suite_path, out_dir, limits, &Selection::default())
}

/// Protected input/oracle roots for a loaded suite: the suite file's
/// directory (covers case documents) and the fixture base (covers source
/// sidecars and expected artifacts).
fn protected_roots(loaded: &LoadedSuite) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(parent) = loaded.suite_path.parent() {
        roots.push(parent.to_path_buf());
    }
    roots.push(loaded.fixture_base.clone());
    roots
}

fn fingerprint<T: std::fmt::Debug>(v: &T) -> String {
    hex(&sha256(format!("{:?}", v).as_bytes()))
}

/// Run the suite under a selection (Plan 773 F-04/F-05/F-06/F-07 wiring).
pub fn run_suite_selected(
    suite_path: &Path,
    out_dir: &Path,
    limits: &Limits,
    selection: &Selection,
) -> Result<RunReport, String> {
    // Selector shape validation happens before ANY load or execution: a
    // typo'd adapter name is a contract error, not a full run that exits 1.
    if let Some(a) = &selection.adapter {
        if a != Adapter::Avm.as_str()
            && a != Adapter::A2r.as_str()
            && a != Adapter::Ac.as_str()
            && a != Adapter::Syntax.as_str()
            && a != Adapter::Probe.as_str()
        {
            return Err(format!(
                "bad selector: adapter {:?} is not a declared adapter (avm, a2r, ac, syntax, probe)",
                a
            ));
        }
    }

    let loaded = load_suite(suite_path, limits).map_err(|e| format!("suite load: {}", e))?;

    // Resolve the selection against the DECLARED inventory: zero selected
    // checks means the selector names nothing that exists.
    let mut selected_count = 0usize;
    for case_file in &loaded.cases {
        for spec in &case_file.specs {
            for check in &spec.checks {
                if selection.matches(&spec.id, check) {
                    selected_count += 1;
                }
            }
        }
    }
    if selected_count == 0 {
        return Err(format!(
            "bad selector: {} matches no declared check (typo'd or empty intersection)",
            selection.describes()
        ));
    }

    // Output path-domain audit BEFORE execution (F-05): the out directory
    // must not overlap any protected root in either direction and must not
    // traverse a reparse point. Creating the directory first lets Windows
    // canonicalization resolve the real (verbatim) form.
    let out_existed = out_dir.exists();
    std::fs::create_dir_all(out_dir).map_err(|e| format!("scratch root: {}", e))?;
    let protected = protected_roots(&loaded);
    let mut audit = check_no_reparse_ancestors(out_dir)
        .and_then(|()| validate_output_domain(out_dir, &protected));
    // R1 (F04): a pinned store root (AC_CASE_STORE) is an output domain
    // too — it must be reparse-clean and disjoint from every protected
    // root, exactly like the run scratch.
    if let Ok(store) = std::env::var("AC_CASE_STORE") {
        let store_path = std::path::PathBuf::from(&store);
        std::fs::create_dir_all(&store_path).map_err(|e| format!("store root: {}", e))?;
        audit = audit
            .and_then(|()| check_no_reparse_ancestors(&store_path))
            .and_then(|()| validate_output_domain(&store_path, &protected));
    }
    if let Err(e) = audit {
        // Best effort: remove the directory only if this call created it and
        // it is still empty (a pre-existing directory is the caller's).
        if !out_existed {
            let _ = std::fs::remove_dir(out_dir);
        }
        return Err(format!("output path rejected: {}", e));
    }

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
    // R1 (F06): a v2 suite produces a v2 report — the schema follows the
    // suite version every case of the run decoded under.
    if loaded.suite.schema == auto_test_case::suite::SUITE_SCHEMA_V2 {
        report.schema = auto_test_case::report::REPORT_SCHEMA_V2.to_string();
    }
    report.load_errors = loaded.errors.iter().map(|e| e.to_string()).collect();

    // Toolchain/config identity bound into the report (F-06): core crate,
    // vendored parser snapshot, driver crate, and the EFFECTIVE limits.
    for (k, v) in stale::toolchain_axes(limits) {
        report.toolchain.insert(k, v);
    }

    for case_file in &loaded.cases {
        for spec in case_file.specs.iter() {
            run_case(
                &loaded,
                case_file,
                spec,
                &run_dir,
                limits,
                selection,
                &mut report,
            );
        }
    }

    // Required coverage (F-04): a selector cannot erase required adapters,
    // but only cases that actually have a selected (non-NOT_RUN) check
    // participate — a fully unselected case is NOT_RUN and breaks the gate
    // through the exit reducer, not through phantom coverage gaps.
    let mut case_adapters: BTreeMap<String, std::collections::BTreeSet<String>> = BTreeMap::new();
    let mut selected_cases: Vec<String> = Vec::new();
    for r in &report.results {
        if r.status == Status::NotRun {
            continue;
        }
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

#[allow(clippy::too_many_arguments)]
fn run_case(
    loaded: &LoadedSuite,
    case_file: &CaseFile,
    spec: &CaseSpec,
    run_dir: &Path,
    limits: &Limits,
    selection: &Selection,
    report: &mut RunReport,
) {
    // `file:` payload / expected-artifact references resolve against the
    // suite-declared fixture base (imports keep legacy files in place).
    let ref_base = loaded.fixture_base.clone();

    let any_selected = spec.checks.iter().any(|c| selection.matches(&spec.id, c));

    // Resolve payloads only when some check of this case will run; a
    // partial run must not read (or fail on) fixtures it never touches.
    let resolved: Option<ResolvedCase> = if any_selected {
        match resolve_case(spec.clone(), &case_file.doc, &ref_base, limits) {
            Ok(r) => Some(r),
            Err(e) => {
                // Resolution failure is a contract/infrastructure error for
                // every SELECTED check; unselected checks stay NOT_RUN.
                for check in &spec.checks {
                    if selection.matches(&spec.id, check) {
                        report.results.push(error_outcome(
                            loaded,
                            spec,
                            check,
                            &format!("payload resolution failed: {}", e),
                        ));
                    } else {
                        report.results.push(not_run_outcome(
                            loaded,
                            case_file,
                            spec,
                            check,
                            &selection.describes(),
                        ));
                    }
                }
                return;
            }
        }
    } else {
        None
    };

    // Product registry (F-03): logical products of EXECUTED checks with
    // usable artifacts — registered on PASS *and* FAIL-with-artifacts;
    // ERROR/no-artifacts leaves nothing to consume (consumers BLOCK).
    // Pseudo-artifacts (`<lowered-input>` evidence hashes) are not files
    // and never register.
    let mut products: BTreeMap<String, Vec<ArtifactRef>> = BTreeMap::new();

    for check in &spec.checks {
        let (check, _origins) = apply_defaults(check, &loaded.suite.defaults);

        if !selection.matches(&spec.id, &check) {
            // F-04: unselected checks never execute — no scratch dirs, no
            // adapter calls, no side effects. They stay in the inventory as
            // NOT_RUN so a partial report can never count as a full gate.
            report.results.push(not_run_outcome(
                loaded,
                case_file,
                spec,
                &check,
                &selection.describes(),
            ));
            continue;
        }

        let identity_base = |oracle_hash: &str| IdentityHashes {
            suite: loaded.suite_sha256.clone(),
            case_file: case_file.sha256.clone(),
            source: resolved
                .as_ref()
                .map(|r| r.source_sha256.clone())
                .unwrap_or_default(),
            oracle: oracle_hash.to_string(),
            contract: fingerprint(spec),
        };
        // Declaration-level fallback (skipped checks do not read oracles).
        let decl_oracle = fingerprint(&check.expect);

        let started = std::time::Instant::now();
        let mut outcome = if let Some(skip) = spec.status_skip.as_ref().or(check.skip.as_ref()) {
            let mut o = base_outcome(
                loaded,
                spec,
                &check,
                Status::Skip,
                &identity_base(&decl_oracle),
            );
            o.detail = Some(format!("skip: {} (issue {})", skip.reason, skip.issue));
            o
        } else if let Some(upstream) = &check.product_from {
            // F-03: consumer path — judged over the upstream's actual bytes.
            match products.get(upstream) {
                None => {
                    let mut o = base_outcome(
                        loaded,
                        spec,
                        &check,
                        Status::Blocked,
                        &identity_base(&decl_oracle),
                    );
                    o.detail = Some(format!(
                        "upstream check {:?} produced no usable product (not executed or execution failed) — BLOCKED",
                        upstream
                    ));
                    o
                }
                Some(made) => consume_products(
                    loaded,
                    spec,
                    &check,
                    upstream,
                    made,
                    &ref_base,
                    limits,
                    &identity_base,
                ),
            }
        } else {
            let resolved = resolved
                .as_ref()
                .expect("selected check implies resolved case");
            // F-06/F-09: oracle resolution is full admission — expected
            // sidecars must exist, be readable, bounded, and reparse-clean
            // BEFORE the check can execute.
            match resolve_oracle(&spec.id, &check, &ref_base, limits) {
                Err(e) => error_outcome(
                    loaded,
                    spec,
                    &check,
                    &format!("oracle resolution failed: {}", e),
                ),
                Ok(oracle) => {
                    let identity = identity_base(&oracle.oracle_sha256);
                    dispatch_check(
                        loaded, spec, resolved, &check, &oracle, run_dir, &identity, started,
                    )
                }
            }
        };
        outcome.duration_ms = outcome.duration_ms.max(started.elapsed().as_millis());
        if matches!(outcome.status, Status::Pass | Status::Fail) {
            let usable: Vec<ArtifactRef> = outcome
                .artifacts
                .iter()
                .filter(|a| !a.logical_path.starts_with('<'))
                .cloned()
                .collect();
            if !usable.is_empty() {
                products.insert(check.id.clone(), usable);
            }
        }
        report.results.push(outcome);
    }
}

/// Internal registered test consumer (F-03): drives the consumer judgement
/// over the upstream's ACTUAL artifact bytes — reads each persisted actual,
/// verifies it against the manifest hash, then applies the consumer's own
/// declared artifact oracle to the CONSUMED bytes. The expected side is
/// only ever the consumer's own declared oracle; actuals are never taken
/// from it.
#[allow(clippy::too_many_arguments)]
fn consume_products(
    loaded: &LoadedSuite,
    spec: &CaseSpec,
    check: &CheckSpec,
    upstream: &str,
    made: &[ArtifactRef],
    ref_base: &Path,
    limits: &Limits,
    identity_base: &dyn Fn(&str) -> IdentityHashes,
) -> CheckOutcome {
    use auto_test_case::compare::compare_bytes;

    // Unimplemented public downstream consumption is an explicit ERROR,
    // never a pass (F-03): consumers may only judge consumed artifacts.
    if !matches!(
        check.expect.completion,
        auto_test_case::contract::Completion::Success
    ) {
        let mut o = base_outcome(loaded, spec, check, Status::Error, &identity_base(""));
        o.detail = Some(
            "error-completion product consumers are unimplemented in this pilot — the registered consumer judges consumed actuals only".to_string(),
        );
        return o;
    }
    // R1 (F02): ac/run consumers DO observe the consumed product's run —
    // stdout/stderr are supported there; the avm legacy `result` channel
    // remains unimplemented for every consumer.
    let ac_run_consumer =
        check.adapter == Adapter::Ac && check.action == auto_test_case::contract::Action::Run;
    if check.expect.result.is_some()
        || (!ac_run_consumer && (check.expect.stdout.is_some() || check.expect.stderr.is_some()))
    {
        let mut o = base_outcome(loaded, spec, check, Status::Error, &identity_base(""));
        o.detail = Some(
            "result observations (and streams outside ac/run) on a product consumer are unimplemented in this pilot — declare artifact expectations over the consumed manifest".to_string(),
        );
        return o;
    }

    // Oracle resolution for the consumer's own expectations (its expected
    // sidecars are read through the same full admission as everyone else).
    let oracle = match resolve_oracle(&spec.id, check, ref_base, limits) {
        Ok(o) => o,
        Err(e) => {
            return error_outcome(
                loaded,
                spec,
                check,
                &format!("oracle resolution failed: {}", e),
            )
        }
    };
    let identity = identity_base(&oracle.oracle_sha256);
    let mut o = base_outcome(loaded, spec, check, Status::Error, &identity);

    // Consume: read every actual, verify against the manifest (read
    // failures and integrity breaks are infrastructure ERRORs — they
    // propagate, never silently skip).
    let mut consumed: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for a in made {
        let bytes = std::fs::read(&a.stored_path).map_err(|e| {
            format!(
                "consuming product {} from {}: {}",
                a.logical_path, a.stored_path, e
            )
        });
        let bytes = match bytes {
            Ok(b) => b,
            Err(e) => {
                o.detail = Some(e);
                return o;
            }
        };
        let digest = hex(&sha256(&bytes));
        if digest != a.sha256 {
            o.detail = Some(format!(
                "product integrity broken: {} at {} hashes {} but the manifest says {}",
                a.logical_path, a.stored_path, digest, a.sha256
            ));
            return o;
        }
        consumed.insert(a.logical_path.clone(), bytes);
    }

    // Judge the consumer's declared artifact oracle against consumed bytes.
    let mut mismatches = Vec::new();
    for want in &oracle.artifacts {
        match consumed.get(&want.path) {
            None => mismatches.push(auto_test_case::compare::Mismatch {
                observation: format!("consumed artifact {}", want.path),
                kind: "missing",
                expected: format!("{} bytes", want.bytes.len()),
                actual: "absent from consumed manifest".to_string(),
                first_diff: None,
            }),
            Some(actual) => {
                if let Some(m) = compare_bytes(
                    &want.bytes,
                    actual,
                    want.compare,
                    &format!("consumed artifact {}", want.path),
                ) {
                    mismatches.push(m);
                }
            }
        }
    }
    o.artifacts = made.to_vec();
    o.detail = Some(format!(
        "consumed {} actual artifact(s) from {:?} (verified against manifest): {:?}",
        consumed.len(),
        upstream,
        consumed.keys().collect::<Vec<_>>()
    ));
    // An ac/run consumer RUNS the manifest-verified exe (PLAN-788 §5.2: a
    // consumer never recompiles source to replace a bad product; the
    // consumed bytes ARE the build's real output).
    if check.adapter == Adapter::Ac && check.action == auto_test_case::contract::Action::Run {
        match consumed.get("app.exe") {
            None => mismatches.push(auto_test_case::compare::Mismatch {
                observation: "consumed product app.exe".into(),
                kind: "missing",
                expected: "the upstream build's exe".into(),
                actual: "absent from consumed manifest".into(),
                first_diff: None,
            }),
            Some(exe_bytes) => {
                let exe_path = made
                    .iter()
                    .find(|a| a.logical_path == "app.exe")
                    .map(|a| std::path::PathBuf::from(&a.stored_path))
                    .expect("app.exe present in made (manifest verified above)");
                let _ = exe_bytes; // hash-verified above; the worker re-reads the file
                match ac::consume_and_run(
                    &exe_path,
                    check,
                    oracle.exit,
                    oracle.stdout.as_ref(),
                    oracle.stderr.as_ref(),
                    oracle.perf.and_then(|pf| pf.total_ms_max),
                ) {
                    Ok(v) => {
                        // R1 (F02): the consumer verdict's own status
                        // travels — an execution ERROR is never swallowed
                        // into a PASS.
                        if v.status == Status::Error {
                            o.status = Status::Error;
                            o.detail = v.detail;
                            return o;
                        }
                        mismatches.extend(v.mismatches);
                        o.native = v.native;
                    }
                    Err(e) => {
                        o.status = Status::Error;
                        o.detail = Some(format!(
                            "{}; ac consumer run: {}",
                            o.detail.unwrap_or_default(),
                            e
                        ));
                        return o;
                    }
                }
            }
        }
    }
    if mismatches.is_empty() {
        o.status = Status::Pass;
    } else {
        o.status = Status::Fail;
        o.mismatches = mismatches;
        o.detail = Some(format!(
            "{}; consumer oracle mismatched the consumed bytes",
            o.detail.unwrap_or_default()
        ));
    }
    o
}

#[allow(clippy::too_many_arguments)]
fn dispatch_check(
    loaded: &LoadedSuite,
    spec: &CaseSpec,
    resolved: &ResolvedCase,
    check: &CheckSpec,
    oracle: &ResolvedOracle,
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
        (Adapter::Avm, auto_test_case::contract::Action::Run) => {
            avm::run_check(resolved, oracle, &project_dir, &artifact_dir)
        }
        (Adapter::A2r, auto_test_case::contract::Action::Emit) => {
            a2r::run_check(resolved, check, oracle, &project_dir, &artifact_dir)
        }
        (Adapter::Ac, auto_test_case::contract::Action::Check)
        | (Adapter::Ac, auto_test_case::contract::Action::Build)
        | (Adapter::Ac, auto_test_case::contract::Action::Run) => ac::run_check(
            resolved,
            check,
            oracle,
            &project_dir,
            &artifact_dir,
            run_dir,
        ),
        // PLAN-796 T-06 restricted probe/run seam — REGISTERED recipes
        // dispatch to their recipe module; judgement is entirely
        // parent-side. PLAN-797 T-06 appends abi/interop-v1 under the
        // same contract (never a second runner).
        (Adapter::Probe, auto_test_case::contract::Action::Run)
            if check.options.probe_recipe.as_deref() == Some(ac_av::AV_RECIPE) =>
        {
            ac_av::run_check(
                resolved,
                check,
                oracle,
                &project_dir,
                &artifact_dir,
                run_dir,
            )
        }
        (Adapter::Probe, auto_test_case::contract::Action::Run)
            if check.options.probe_recipe.as_deref() == Some(abi::ABI_RECIPE) =>
        {
            abi::run_check(
                resolved,
                check,
                oracle,
                &project_dir,
                &artifact_dir,
                run_dir,
            )
        }
        (Adapter::Probe, auto_test_case::contract::Action::Run) => sema::run_check(
            resolved,
            check,
            oracle,
            &project_dir,
            &artifact_dir,
            run_dir,
        ),
        // T-06: syntax parse/edit bound against the MERGED 787 runtime
        // (D-787 satisfied — delivery b52188c58); the same worker
        // executor carries the request.
        (Adapter::Syntax, auto_test_case::contract::Action::Parse)
        | (Adapter::Syntax, auto_test_case::contract::Action::Edit) => ac::run_check(
            resolved,
            check,
            oracle,
            &project_dir,
            &artifact_dir,
            run_dir,
        ),
        _ => Err("unreachable adapter/action (contract decode enforces the matrix)".to_string()),
    };
    match dispatched {
        Ok(verdict) => {
            o.status = verdict.status;
            o.mismatches = verdict.mismatches;
            o.detail = verdict.detail;
            o.artifacts = verdict.artifacts;
            o.native = verdict.native;
        }
        Err(e) => {
            o.status = Status::Error;
            o.detail = Some(e);
        }
    }
    o
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
        native: None,
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
        native: None,
    }
}

/// NOT_RUN without execution (F-04): identity carries the declaration-level
/// hashes only (nothing was resolved or observed); the detail names the
/// selector so partial reports explain themselves.
fn not_run_outcome(
    loaded: &LoadedSuite,
    case_file: &CaseFile,
    spec: &CaseSpec,
    check: &CheckSpec,
    selector: &str,
) -> CheckOutcome {
    let mut o = error_outcome(loaded, spec, check, &format!("not selected ({})", selector));
    o.status = Status::NotRun;
    o.identity = IdentityHashes {
        suite: loaded.suite_sha256.clone(),
        case_file: case_file.sha256.clone(),
        source: String::new(),
        oracle: fingerprint(&check.expect),
        contract: fingerprint(spec),
    };
    o
}
