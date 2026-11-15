//! PLAN-971 B-PROOF-I (A batch): the reference explicit-ACE PROJECT
//! executor — one real multi-file project roundtrip over the 960
//! packages lane (`compile_package_product` → `PackageAvTarget::
//! cold_with_entry`).
//!
//! A-batch scope is PROTOCOL CAPABILITY ONLY: values and diagnostics
//! with the ORIGINAL module identity (`pkg::mod`) and the original span
//! inside that module's source, isolated test runs with a nonzero
//! protocol exit when a test fails, phase-boundary cancel (before the
//! compile / before each test / before the entry run — in-run step
//! granularity, breakpoints, stack and locals are the B batch, T-02),
//! and a per-request scratch store that is reclaimed when the request
//! ends (stop → reclaim). No editor or playground migration rides here
//! (T-03/T-08); there is no path into the legacy AutoVM.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use ace_av::modules::packages::PackageAvTarget;
use ace_av_core::av::{CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_core::source::cst::FrontBudget;
use ace_core::source::packages::{PackageDiagnostic, PackageModuleInput};
use ace_source_query::modules::packages::{compile_package_product, PackageProduct};

use auto_lang_core::{
    AceModuleSpec, AceProjectExecutor, AceProjectOutcome, AceProjectReport, AceProjectRequest,
    AceSpan, AceTestOutcome,
};

use crate::map_value;

/// Frozen default wall budget of one project request (A batch).
const DEFAULT_WALL: Duration = Duration::from_secs(10);

/// The reference project host (stateless — every request compiles its
/// own generation and owns its scratch store for exactly its lifetime).
pub struct ProjectHost;

static PROJECT_HOST: OnceLock<Arc<ProjectHost>> = OnceLock::new();

/// The process singleton (`crate::install()` registers it under the
/// neutral project-executor registry; same-pointer installs are
/// idempotent).
pub fn project_host() -> Arc<dyn auto_lang_core::AceProjectExecutor> {
    PROJECT_HOST.get_or_init(|| Arc::new(ProjectHost)).clone()
}

impl AceProjectExecutor for ProjectHost {
    fn execute_project(&self, request: &AceProjectRequest<'_>) -> AceProjectReport {
        execute_project(request)
    }
}

fn execute_project(request: &AceProjectRequest<'_>) -> AceProjectReport {
    let started = Instant::now();
    let deadline = started + request.wall.unwrap_or(DEFAULT_WALL);
    let cancelled = || {
        request
            .cancel
            .is_some_and(|flag: &AtomicBool| flag.load(Ordering::Relaxed))
    };

    let finish = |outcome: AceProjectOutcome, tests: Vec<AceTestOutcome>, diag_count: usize| {
        let exit_code = if matches!(outcome, AceProjectOutcome::Success { .. })
            && tests.iter().all(|t| t.passed)
        {
            0
        } else {
            1
        };
        AceProjectReport {
            engine: crate::ENGINE_ID,
            outcome,
            tests,
            exit_code,
            diag_count,
            wall_ms: 0.0, // filled by auto_lang_core::run_ace_project
        }
    };

    if cancelled() {
        return finish(AceProjectOutcome::Cancelled, Vec::new(), 0);
    }

    let inputs: Vec<PackageModuleInput> = request.modules.iter().map(package_input).collect();
    let product = match compile_package_product(
        &inputs,
        request.entry_module,
        request.capabilities,
        FrontBudget::WORK_OWNED_64MIB,
    ) {
        Ok(product) => product,
        Err(diags) => {
            let count = diags.len();
            return finish(package_diagnostic_outcome(&diags), Vec::new(), count);
        }
    };

    // Test lane: each requested name executes as one isolated generation
    // on its own per-test scratch store; the store is reclaimed when the
    // generation ends (Drop), so a stopped/cancelled request leaves no
    // residue behind.
    let mut tests = Vec::with_capacity(request.tests.len());
    for name in request.tests {
        if cancelled() {
            return finish(AceProjectOutcome::Cancelled, tests, 0);
        }
        if Instant::now() >= deadline {
            return finish(
                AceProjectOutcome::ResourceDenied {
                    kind: "wall".to_string(),
                    message: "project wall budget exhausted before a test run".to_string(),
                },
                tests,
                0,
            );
        }
        tests.push(run_one_test(&product, name));
    }

    // Entry leg.
    if cancelled() {
        return finish(AceProjectOutcome::Cancelled, tests, 0);
    }
    if Instant::now() >= deadline {
        return finish(
            AceProjectOutcome::ResourceDenied {
                kind: "wall".to_string(),
                message: "project wall budget exhausted before the entry run".to_string(),
            },
            tests,
            0,
        );
    }
    let outcome = run_entry(&product, request.entry_def_id);
    finish(outcome, tests, 0)
}

fn package_input(spec: &AceModuleSpec<'_>) -> PackageModuleInput {
    PackageModuleInput {
        package: spec.package.to_string(),
        version: spec.version.to_string(),
        id: spec.id.to_string(),
        source: spec.source.to_string(),
        provider: None,
    }
}

/// Map the package-lane rejection to the neutral contract, keeping the
/// ORIGINAL module identity and the module-relative span (the binder
/// already rebased the span into the owning module's source).
fn package_diagnostic_outcome(diags: &[PackageDiagnostic]) -> AceProjectOutcome {
    let Some(primary) = diags.first() else {
        return AceProjectOutcome::HostFailure {
            code: "ace.host/empty-diagnostics".to_string(),
            message: "package compile failed without a diagnostic".to_string(),
        };
    };
    AceProjectOutcome::Diagnostic {
        module: primary.module.clone(),
        stage: primary.diagnostic.stage.as_str().to_string(),
        code: primary.diagnostic.code.clone(),
        message: primary.diagnostic.message.clone(),
        span: Some(AceSpan {
            start: primary.diagnostic.span.start as u64,
            end: primary.diagnostic.span.end as u64,
        }),
    }
}

/// Execute one requested test: resolve the bare name against the bound
/// declaration origins, run it as its own generation, and judge it by
/// the frozen test contract (`fn <name>() int` — passed iff the run
/// completes and returns 0).
fn run_one_test(product: &PackageProduct, name: &str) -> AceTestOutcome {
    let mut matches = product
        .bound
        .origins
        .iter()
        .filter(|(_, (_, _, original))| original.as_str() == name);
    let Some((symbol, (pkg, module, _))) = matches.next() else {
        return AceTestOutcome {
            name: name.to_string(),
            module: String::new(),
            passed: false,
            message: format!("test entry `{name}` not found in the project declaration set"),
        };
    };
    if matches.next().is_some() {
        return AceTestOutcome {
            name: name.to_string(),
            module: String::new(),
            passed: false,
            message: format!(
                "test entry `{name}` is ambiguous across modules; A-batch test names must be unique in the project"
            ),
        };
    }
    let module = format!("{pkg}::{module}");
    let failed = |message: String| AceTestOutcome {
        name: name.to_string(),
        module: module.clone(),
        passed: false,
        message,
    };
    let entry_def = format!("d_{symbol}");
    let scratch = match ScratchStore::new() {
        Ok(scratch) => scratch,
        Err(e) => return failed(format!("scratch store open failed: {e}")),
    };
    let target = match PackageAvTarget::cold_with_entry(&scratch.path, product, &entry_def) {
        Ok(target) => target,
        Err(e) => return failed(format!("test link failed: {e}")),
    };
    match execute_generation(&target) {
        ExecOutcome::Success {
            value: Value::I32(0),
            ..
        } => AceTestOutcome {
            name: name.to_string(),
            module,
            passed: true,
            message: String::new(),
        },
        ExecOutcome::Success { value, .. } => failed(format!(
            "test returned {value:?}; the frozen test contract is `fn {name}() int` returning 0"
        )),
        ExecOutcome::Trap { kind, .. } => failed(format!("language trap: {kind:?}")),
        ExecOutcome::Error { message, .. } => failed(format!("run error: {message}")),
        ExecOutcome::Resource { kind, message } => {
            failed(format!("resource denied: {kind:?}: {message}"))
        }
        ExecOutcome::Cancelled => failed("cancelled".to_string()),
        ExecOutcome::Infra(message) => failed(format!("infra: {message}")),
    }
}

/// Run the project entry on its own generation and map the terminal
/// state to the neutral contract (trap spans survive as diagnostics).
fn run_entry(product: &PackageProduct, entry_def_id: &str) -> AceProjectOutcome {
    let scratch = match ScratchStore::new() {
        Ok(scratch) => scratch,
        Err(e) => {
            return AceProjectOutcome::HostFailure {
                code: "ace.project/scratch-open".to_string(),
                message: format!("scratch store open failed: {e}"),
            }
        }
    };
    let target = match PackageAvTarget::cold_with_entry(&scratch.path, product, entry_def_id) {
        Ok(target) => target,
        Err(e) => {
            return AceProjectOutcome::HostFailure {
                code: "ace.project/link".to_string(),
                message: format!("entry link failed: {e}"),
            }
        }
    };
    match execute_generation(&target) {
        ExecOutcome::Success { value, steps } => AceProjectOutcome::Success {
            value: map_value(value),
            steps,
        },
        ExecOutcome::Trap { kind, span } => AceProjectOutcome::Diagnostic {
            module: String::new(),
            stage: "run".to_string(),
            code: format!("av.trap/{kind:?}"),
            message: format!("language trap: {kind:?}"),
            span: span.as_ref().map(|s| AceSpan {
                start: s.start as u64,
                end: s.end as u64,
            }),
        },
        ExecOutcome::Error { message, span } => AceProjectOutcome::Diagnostic {
            module: String::new(),
            stage: "run".to_string(),
            code: "av.error".to_string(),
            message,
            span: span.as_ref().map(|s| AceSpan {
                start: s.start as u64,
                end: s.end as u64,
            }),
        },
        ExecOutcome::Resource { kind, message } => AceProjectOutcome::ResourceDenied {
            kind: format!("{kind:?}"),
            message,
        },
        ExecOutcome::Cancelled => AceProjectOutcome::Cancelled,
        ExecOutcome::Infra(message) => AceProjectOutcome::HostFailure {
            code: "ace.executor/infra".to_string(),
            message,
        },
    }
}

fn execute_generation(target: &PackageAvTarget) -> ExecOutcome {
    let fallback = CancelFlag::new(false);
    let (outcome, _trace) =
        ace_av_core::execute_program(&target.current, &ExecLimits::default(), &fallback);
    outcome
}

/// Per-run scratch store root: a fresh temp directory created for one
/// generation and removed when the guard drops (stop → reclaim).
struct ScratchStore {
    path: PathBuf,
}

impl ScratchStore {
    fn new() -> Result<Self, String> {
        static NONCE: AtomicU64 = AtomicU64::new(0);
        let nonce = NONCE.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ace-p971-project-{}-{nonce}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
        Ok(ScratchStore { path })
    }
}

impl Drop for ScratchStore {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
