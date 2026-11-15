//! PLAN-969 E0 (T-E0-I): the public explicit-ACE synchronous execution
//! facade — named request/result/authorization/close contract for driving
//! the ACE AV from a host (UI main thread, service, or test consumer).
//!
//! Ownership split (frozen by T-E0-I):
//! - THIS module owns the neutral public contract only. It has no ACE
//!   dependency and no path into the legacy AutoVM/parser/emitter: an
//!   explicit ACE request can never fall back to, or silently switch to,
//!   a legacy engine (legacy dispatch = 0 is structural — `run_ace` and
//!   every helper below reference no `run_autovm`/`parse` symbol).
//! - The executable engine is injected by the host through
//!   [`install_ace_executor`] (the `auto-lang-ace-host` bridge crate is the
//!   reference implementation over the ACE AV `query_entry`/`ForeignSession`
//!   spine). An unregistered executor is a named host failure, never a
//!   default-engine switch.
//!
//! E0 scope (one synchronous run per request; host-owned provider
//! lifecycle): every run re-checks authorization and identity, and the
//! result distinguishes success / source diagnostics (original stage+span)
//! / authorization denial / resource denial / cancellation / host failure.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Explicit engine identities accepted by the ACE door. Legacy engine
/// names ("autovm", "evaluator", ...) are NOT members: the ACE door
/// refuses them instead of dispatching the legacy machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AceEngineChoice {
    /// The ACE AV (`av.core-i32/1` executor lane).
    AceAV,
}

impl AceEngineChoice {
    /// Canonical public name (what [`parse_ace_engine`] accepts).
    pub fn as_str(&self) -> &'static str {
        match self {
            AceEngineChoice::AceAV => "av",
        }
    }
}

/// Parse an explicit engine name. Unknown names — including every legacy
/// engine name — are a named refusal, never a default-engine switch.
pub fn parse_ace_engine(name: &str) -> Result<AceEngineChoice, String> {
    match name {
        "av" | "ace-av" => Ok(AceEngineChoice::AceAV),
        other => Err(format!(
            "unknown engine `{other}`: the ACE door accepts [av] only; \
             legacy engines are never dispatched from it"
        )),
    }
}

/// A byte span in the original source text (half-open `[start, end)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AceSpan {
    pub start: u64,
    pub end: u64,
}

/// The public scalar value surface of one successful run (opaque values
/// are named, never coerced).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AceValue {
    Unit,
    Bool(bool),
    I8(i8),
    U8(u8),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Char(char),
    /// A non-scalar payload (record/text handle) — named by kind, not
    /// copied through the neutral contract.
    Opaque(&'static str),
}

/// One explicit ACE execution request. Everything is explicit input:
/// source, entry, capabilities, grants, provider binding, deadline and
/// the host-owned cancel flag are re-checked on every run.
pub struct AceRequest<'a> {
    /// The Auto source text (plain scalar source; `use.rs` provider
    /// frontdoor items are part of the source).
    pub source: &'a str,
    /// Entry def id following the pipeline convention (`"d_main"`).
    pub entry_def_id: &'a str,
    /// Explicit capability requests (grammar/domain registration).
    pub capabilities: &'a [String],
    /// Explicit grants; re-checked per run against what the host
    /// admitted. Empty = the host grants only what the (absent) provider
    /// lane itself admits.
    pub grants: &'a [String],
    /// Registered provider name (foreign lane) or `None` (pure Auto
    /// source, no foreign dispatch).
    pub provider: Option<&'a str>,
    /// Wall-clock budget for the run; `None` = the executor's frozen
    /// default budget.
    pub wall: Option<Duration>,
    /// Host-owned cancellation flag (checked by the executor; a set flag
    /// yields [`AceOutcome::Cancelled`]). `None` = not cancellable.
    pub cancel: Option<&'a AtomicBool>,
    /// T-02: explicit CONSOLE lane selection — the source may use
    /// `print`/`eprint` and the run executes under per-run console
    /// grants (see [`AceConsoleReport`]); the captured streams come back
    /// on the report. `false` = the pure scalar lane (no console ops).
    pub console: bool,
    /// T-08: script argv (beyond the program path) — an EXPLICIT input
    /// the request carries so consumers never rely on ambient process
    /// state. The AV lane has no argv surface yet: a non-empty `args`
    /// is a NAMED refusal (`ace.args/unsupported`), never a silent
    /// drop and never a legacy-VM route.
    pub args: &'a [&'a str],
    /// T-08: request-local stdin text (the legacy `Some("")` = immediate
    /// EOF convention applies to the empty string). The AV lane has no
    /// input surface yet: `Some(_)` is a NAMED refusal
    /// (`ace.input/unsupported`), never a silent drop.
    pub input: Option<&'a str>,
}

impl<'a> AceRequest<'a> {
    /// A minimal request for a plain scalar source with the `d_main`
    /// entry, no capabilities/grants/provider/cancel and the default
    /// budget.
    pub fn simple(source: &'a str) -> Self {
        AceRequest {
            source,
            entry_def_id: "d_main",
            capabilities: &[],
            grants: &[],
            provider: None,
            wall: None,
            cancel: None,
            console: false,
            args: &[],
            input: None,
        }
    }
}

/// T-02: the console lane's captured streams of one run (bytes — the
/// program owns its stream encoding; nothing is lossily transcoded).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AceConsoleReport {
    /// Bytes written through granted `console.stdout` ops.
    pub stdout: Vec<u8>,
    /// Bytes written through granted `console.stderr` ops.
    pub stderr: Vec<u8>,
    /// Console host dispatches observed this run (0 on an unauthorized
    /// run — refusal happens before any dispatch).
    pub dispatches: u64,
}

/// The distinguished terminal states of one explicit ACE run.
#[derive(Debug, Clone, PartialEq)]
pub enum AceOutcome {
    /// The run executed to completion and produced a value.
    Success { value: AceValue, steps: u64 },
    /// A diagnosed source/pipeline rejection. `stage` preserves the
    /// ORIGINAL stage of the producing frontend (`text`/`bind`/`verify`/
    /// ... plus facade stages `entry`/`lower`/`wire`), `span` the
    /// original source span when the producer reported one.
    Diagnostic {
        stage: String,
        code: String,
        message: String,
        span: Option<AceSpan>,
    },
    /// Authorization/admission refusal (unknown engine option, revoked or
    /// unadmitted provider, unadmitted grant). Refusals happen BEFORE any
    /// dispatch.
    Denied { code: String, message: String },
    /// A resource budget denial (steps/frames/slots/bytes/wall).
    ResourceDenied { kind: String, message: String },
    /// The run observed the host cancel flag.
    Cancelled,
    /// Host/tool infrastructure failure (unregistered executor, provider
    /// load failure, executor-internal fault). Never retried silently.
    HostFailure { code: String, message: String },
}

/// Evidence companion of one run (identity + counters for consumers that
/// must observe dispatch discipline, not just the terminal value).
#[derive(Debug, Clone)]
pub struct AceRunReport {
    /// The engine identity that served (or was asked to serve) the run.
    pub engine: &'static str,
    /// Terminal state (see [`AceOutcome`]).
    pub outcome: AceOutcome,
    /// Diagnostics count when a compile/pipeline rejection carried more
    /// than the primary diagnostic.
    pub diag_count: usize,
    /// Foreign lane: calls that actually dispatched through the provider
    /// table this run (per-run session counter; `None` for pure runs —
    /// the pure lane has no provider table).
    pub callee_count: Option<u64>,
    /// Foreign lane: pre-dispatch rejections observed this run.
    pub rejected_count: Option<u64>,
    /// Executor-measured wall time of the whole request (milliseconds).
    pub wall_ms: f64,
    /// T-02: the console lane's captured streams (`None` on non-console
    /// runs and on pre-execution refusals).
    pub console: Option<AceConsoleReport>,
}

impl AceRunReport {
    fn host_failure(engine: &'static str, code: &str, message: impl Into<String>) -> Self {
        AceRunReport {
            engine,
            outcome: AceOutcome::HostFailure {
                code: code.to_string(),
                message: message.into(),
            },
            diag_count: 0,
            callee_count: None,
            rejected_count: None,
            wall_ms: 0.0,
            console: None,
        }
    }

    fn denied(engine: &'static str, code: &str, message: impl Into<String>) -> Self {
        AceRunReport {
            engine,
            outcome: AceOutcome::Denied {
                code: code.to_string(),
                message: message.into(),
            },
            diag_count: 0,
            callee_count: None,
            rejected_count: None,
            wall_ms: 0.0,
            console: None,
        }
    }
}

/// The executable ACE engine, injected by the host (the
/// `auto-lang-ace-host` bridge is the reference implementation).
/// Implementations must re-check authorization and identity on every
/// [`AceExecutor::execute`] call and must be callable from the UI main
/// thread (synchronous, no thread spawning, no background waits).
pub trait AceExecutor: Send + Sync {
    fn execute(&self, request: &AceRequest<'_>) -> AceRunReport;
}

static ACE_EXECUTOR: RwLock<Option<Arc<dyn AceExecutor>>> = RwLock::new(None);

/// Install the process-wide ACE executor (single owner; reinstalling the
/// SAME executor is idempotent, installing a different one while one is
/// installed is an error — release the current one with
/// [`shutdown_ace_executor`] first).
pub fn install_ace_executor(executor: Arc<dyn AceExecutor>) -> Result<(), String> {
    let mut slot = ACE_EXECUTOR
        .write()
        .map_err(|e| format!("executor lock poisoned: {e}"))?;
    match slot.as_ref() {
        Some(current) => {
            if Arc::ptr_eq(current, &executor) {
                Ok(())
            } else {
                Err("another ACE executor is installed; shutdown_ace_executor() first".to_string())
            }
        }
        None => {
            *slot = Some(executor);
            Ok(())
        }
    }
}

/// Release the installed executor (host close). Returns whether an
/// executor was installed. After this, explicit ACE requests report
/// `ace.host/executor-unregistered` — they never fall back.
pub fn shutdown_ace_executor() -> bool {
    match ACE_EXECUTOR.write() {
        Ok(mut slot) => slot.take().is_some(),
        Err(_) => false,
    }
}

/// Run one explicit ACE request synchronously through the installed
/// executor. No executor / unknown engine / any failure is a NAMED
/// terminal state — the legacy AutoVM is never dispatched from here.
pub fn run_ace(choice: AceEngineChoice, request: &AceRequest<'_>) -> AceRunReport {
    let started = std::time::Instant::now();
    let executor = match ACE_EXECUTOR.read() {
        Ok(slot) => slot.clone(),
        Err(_) => {
            return AceRunReport::host_failure(
                choice.as_str(),
                "ace.host/executor-lock-poisoned",
                "executor lock poisoned",
            )
        }
    };
    let Some(executor) = executor else {
        return AceRunReport::host_failure(
            choice.as_str(),
            "ace.host/executor-unregistered",
            "no ACE executor installed; the host must install one (auto-lang-ace-host)",
        );
    };
    let mut report = executor.execute(request);
    report.wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    report
}

/// Name-driven form of [`run_ace`] for hosts that take the engine as
/// text (CLI/UI config): an unknown name — including every legacy engine
/// name — is a named refusal with zero dispatch.
pub fn run_ace_named(engine: &str, request: &AceRequest<'_>) -> AceRunReport {
    let choice = match parse_ace_engine(engine) {
        Ok(choice) => choice,
        Err(message) => {
            return AceRunReport::denied("?", "ace.engine/unknown", message);
        }
    };
    run_ace(choice, request)
}

// ---------------------------------------------------------------------------
// PLAN-971 B-PROOF-I (A batch): the PROJECT face — one real multi-file
// project roundtrip through the ACE explicit entry. Same discipline as
// the E0 single-source face above: this module owns the neutral public
// contract only (zero ACE dependency, zero legacy fallback — an explicit
// project request can never dispatch the legacy machine), and the
// executable project engine is injected by the host through
// [`install_ace_project_executor`] (the `auto-lang-ace-host` bridge over
// the 960 packages lane is the reference implementation).
//
// A-batch scope (protocol capability only): values and diagnostics with
// the ORIGINAL module identity (`pkg::mod`) and span, isolated test runs
// with a nonzero protocol exit when a test fails, phase-boundary cancel
// (before compile / before each test / before the entry run; in-run step
// granularity, breakpoints, stack and locals are the B batch — T-02).
// ---------------------------------------------------------------------------

/// One source module of an explicit ACE project request. The identity
/// fields mirror the packages-lane input contract: `package`/`version`
/// name the package, `id` is the module id inside it.
pub struct AceModuleSpec<'a> {
    pub package: &'a str,
    pub version: &'a str,
    pub id: &'a str,
    pub source: &'a str,
}

/// An explicit multi-file project request through the ACE door.
pub struct AceProjectRequest<'a> {
    /// All source modules of the project (the entry module included).
    pub modules: &'a [AceModuleSpec<'a>],
    /// Entry module key `pkg::mod` — the module owning the entry def.
    pub entry_module: &'a str,
    /// Entry def id (`"d_main"` for the conventional entry).
    pub entry_def_id: &'a str,
    /// Explicitly discovered test entry names (bare fn names, unique
    /// across the project). A-phase discovery stays with the existing
    /// test owner (`test_runner`); each listed name executes as one
    /// isolated test run (fresh generation, per-test scratch store).
    pub tests: &'a [&'a str],
    /// Explicit capability requests (grammar/domain registration), e.g.
    /// the packages module capability — re-checked on every run.
    pub capabilities: &'a [String],
    /// Wall-clock budget for the whole project request; `None` = the
    /// executor's frozen default budget.
    pub wall: Option<Duration>,
    /// Host-owned cancellation flag, checked before the compile, before
    /// every test run and before the entry run.
    pub cancel: Option<&'a AtomicBool>,
}

impl<'a> AceProjectRequest<'a> {
    /// A minimal project request: `d_main` entry, no tests and the
    /// default budget. `capabilities` is the caller-bound capability
    /// list — [`packages_capabilities`] builds the standard packages
    /// lane list.
    pub fn simple_project(
        modules: &'a [AceModuleSpec<'a>],
        entry_module: &'a str,
        capabilities: &'a [String],
    ) -> Self {
        AceProjectRequest {
            modules,
            entry_module,
            entry_def_id: "d_main",
            tests: &[],
            capabilities,
            wall: None,
            cancel: None,
        }
    }
}

/// The packages-lane module capability the project face consumes
/// (frozen by the T-01 freeze record, PLAN-971 §5).
pub const PACKAGES_CAPABILITY: &str = "ac.source.modules.packages/v1";

/// The standard capability list for the packages lane (bind once, pass
/// by reference to [`AceProjectRequest::simple_project`]).
pub fn packages_capabilities() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string()]
}

/// One executed test of a project request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AceTestOutcome {
    /// The requested test entry name.
    pub name: String,
    /// Owning module identity of the executed test (`pkg::mod`); empty
    /// when the name was not found in the project.
    pub module: String,
    pub passed: bool,
    /// Failure reason (empty on pass).
    pub message: String,
}

/// The distinguished terminal states of one explicit ACE project run.
/// Mirrors [`AceOutcome`] with the diagnostic carrying the ORIGINAL
/// module identity so a multi-file rejection locates the original file.
#[derive(Debug, Clone, PartialEq)]
pub enum AceProjectOutcome {
    /// The entry run executed to completion and produced a value.
    Success { value: AceValue, steps: u64 },
    /// A diagnosed source/pipeline rejection in one original module;
    /// `span` is the original span inside that module's source.
    Diagnostic {
        module: String,
        stage: String,
        code: String,
        message: String,
        span: Option<AceSpan>,
    },
    /// Authorization/admission refusal (unknown engine option).
    Denied { code: String, message: String },
    /// A resource budget denial (wall/steps/slots/bytes).
    ResourceDenied { kind: String, message: String },
    /// The request observed the host cancel flag.
    Cancelled,
    /// Host/tool infrastructure failure. Never retried silently.
    HostFailure { code: String, message: String },
}

/// Evidence companion of one project request.
#[derive(Debug, Clone)]
pub struct AceProjectReport {
    /// The engine identity that served (or was asked to serve) the run.
    pub engine: &'static str,
    /// Terminal state of the request (see [`AceProjectOutcome`]).
    pub outcome: AceProjectOutcome,
    /// Per-test results, in request order (empty when no tests were
    /// requested; tests already executed survive a later cancel).
    pub tests: Vec<AceTestOutcome>,
    /// Protocol-level exit of the roundtrip: `0` iff the entry run
    /// succeeded AND every requested test passed; `1` otherwise — the
    /// test-failure exit contract (a failing test never exits zero).
    pub exit_code: i32,
    /// Diagnostics count when the pipeline rejection carried more than
    /// the primary diagnostic.
    pub diag_count: usize,
    /// Executor-measured wall time of the whole request (milliseconds).
    pub wall_ms: f64,
}

impl AceProjectReport {
    fn host_failure(engine: &'static str, code: &str, message: impl Into<String>) -> Self {
        AceProjectReport {
            engine,
            outcome: AceProjectOutcome::HostFailure {
                code: code.to_string(),
                message: message.into(),
            },
            tests: Vec::new(),
            exit_code: 1,
            diag_count: 0,
            wall_ms: 0.0,
        }
    }

    fn denied(engine: &'static str, code: &str, message: impl Into<String>) -> Self {
        AceProjectReport {
            engine,
            outcome: AceProjectOutcome::Denied {
                code: code.to_string(),
                message: message.into(),
            },
            tests: Vec::new(),
            exit_code: 1,
            diag_count: 0,
            wall_ms: 0.0,
        }
    }
}

/// The executable project engine, injected by the host (the
/// `auto-lang-ace-host` bridge is the reference implementation).
/// Implementations re-check authorization and identity on every
/// [`AceProjectExecutor::execute_project`] call and stay callable from
/// the UI main thread (synchronous, no thread spawning).
pub trait AceProjectExecutor: Send + Sync {
    fn execute_project(&self, request: &AceProjectRequest<'_>) -> AceProjectReport;
}

static ACE_PROJECT_EXECUTOR: RwLock<Option<Arc<dyn AceProjectExecutor>>> = RwLock::new(None);

/// Install the process-wide project executor (same single-owner
/// discipline as [`install_ace_executor`]: reinstalling the SAME
/// executor is idempotent; a different one while installed is an error).
pub fn install_ace_project_executor(executor: Arc<dyn AceProjectExecutor>) -> Result<(), String> {
    let mut slot = ACE_PROJECT_EXECUTOR
        .write()
        .map_err(|e| format!("project executor lock poisoned: {e}"))?;
    match slot.as_ref() {
        Some(current) => {
            if Arc::ptr_eq(current, &executor) {
                Ok(())
            } else {
                Err("another ACE project executor is installed; shutdown_ace_project_executor() first".to_string())
            }
        }
        None => {
            *slot = Some(executor);
            Ok(())
        }
    }
}

/// Release the installed project executor (host close). Returns whether
/// one was installed. After this, explicit project requests report
/// `ace.host/project-executor-unregistered` — they never fall back.
pub fn shutdown_ace_project_executor() -> bool {
    match ACE_PROJECT_EXECUTOR.write() {
        Ok(mut slot) => slot.take().is_some(),
        Err(_) => false,
    }
}

/// Run one explicit multi-file project request synchronously through the
/// installed project executor. No executor / unknown engine / any
/// failure is a NAMED terminal state — the legacy AutoVM is never
/// dispatched from here.
pub fn run_ace_project(
    choice: AceEngineChoice,
    request: &AceProjectRequest<'_>,
) -> AceProjectReport {
    let started = std::time::Instant::now();
    let executor = match ACE_PROJECT_EXECUTOR.read() {
        Ok(slot) => slot.clone(),
        Err(_) => {
            return AceProjectReport::host_failure(
                choice.as_str(),
                "ace.host/project-executor-lock-poisoned",
                "project executor lock poisoned",
            )
        }
    };
    let Some(executor) = executor else {
        return AceProjectReport::host_failure(
            choice.as_str(),
            "ace.host/project-executor-unregistered",
            "no ACE project executor installed; the host must install one (auto-lang-ace-host)",
        );
    };
    let mut report = executor.execute_project(request);
    report.wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    report
}

/// Name-driven form of [`run_ace_project`]: an unknown name — including
/// every legacy engine name — is a named refusal with zero dispatch.
pub fn run_ace_project_named(engine: &str, request: &AceProjectRequest<'_>) -> AceProjectReport {
    let choice = match parse_ace_engine(engine) {
        Ok(choice) => choice,
        Err(message) => {
            return AceProjectReport::denied("?", "ace.engine/unknown", message);
        }
    };
    run_ace_project(choice, request)
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests freeze the E0 dispatch discipline WITHOUT any ACE
    // dependency (the bridge crate carries the executable cases).

    #[test]
    fn p969_unknown_engine_is_a_named_refusal() {
        let req = AceRequest::simple("fn main() int { return 2 + 3 * 4 }");
        let report = run_ace_named("ace-native", &req);
        assert!(matches!(
            report.outcome,
            AceOutcome::Denied { ref code, .. } if code == "ace.engine/unknown"
        ));
        // The refusal carries the refused name.
        match report.outcome {
            AceOutcome::Denied { message, .. } => {
                assert!(
                    message.contains("ace-native"),
                    "message names the engine: {message}"
                );
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn p969_legacy_engine_names_are_refused_not_dispatched() {
        let req = AceRequest::simple("fn main() int { return 2 + 3 * 4 }");
        for legacy in ["autovm", "vm", "evaluator", "eval", "tree", ""] {
            let report = run_ace_named(legacy, &req);
            assert!(
                matches!(&report.outcome, AceOutcome::Denied { code, .. } if code == "ace.engine/unknown"),
                "legacy engine `{legacy}` must be refused, got {:?}",
                report.outcome
            );
        }
    }

    #[test]
    fn p969_executor_install_is_single_owner() {
        // Start from the unregistered state: a request must report the
        // named host failure (never a legacy-engine success). This test
        // is the ONLY installer in the process, so the global stays
        // race-free under a plain threaded `cargo test` too.
        let probe_req = AceRequest::simple("fn main() int { return 2 + 3 * 4 }");
        assert!(
            matches!(
                run_ace(AceEngineChoice::AceAV, &probe_req).outcome,
                AceOutcome::HostFailure { ref code, .. } if code == "ace.host/executor-unregistered"
            ),
            "unregistered state must be a named host failure"
        );

        struct Probe(bool);
        impl AceExecutor for Probe {
            fn execute(&self, _request: &AceRequest<'_>) -> AceRunReport {
                AceRunReport {
                    engine: "av",
                    outcome: if self.0 {
                        AceOutcome::Success {
                            value: AceValue::I32(1),
                            steps: 0,
                        }
                    } else {
                        AceOutcome::Success {
                            value: AceValue::I32(2),
                            steps: 0,
                        }
                    },
                    diag_count: 0,
                    callee_count: None,
                    rejected_count: None,
                    wall_ms: 0.0,
                    console: None,
                }
            }
        }
        let first = Arc::new(Probe(true));
        assert!(install_ace_executor(first.clone()).is_ok());
        // Same executor: idempotent.
        assert!(install_ace_executor(first.clone()).is_ok());
        // A different executor while installed: refused.
        let second = Arc::new(Probe(false));
        assert!(install_ace_executor(second).is_err());
        // The installed one serves.
        let req = AceRequest::simple("fn main() int { return 1 }");
        assert_eq!(
            run_ace(AceEngineChoice::AceAV, &req).outcome,
            AceOutcome::Success {
                value: AceValue::I32(1),
                steps: 0
            }
        );
        // Shutdown ends the ownership; requests become named failures.
        assert!(shutdown_ace_executor());
        assert!(matches!(
            run_ace(AceEngineChoice::AceAV, &req).outcome,
            AceOutcome::HostFailure { .. }
        ));
        // Second shutdown: nothing was installed.
        assert!(!shutdown_ace_executor());
    }

    #[test]
    fn p969_parse_ace_engine_names() {
        assert_eq!(parse_ace_engine("av"), Ok(AceEngineChoice::AceAV));
        assert_eq!(parse_ace_engine("ace-av"), Ok(AceEngineChoice::AceAV));
        assert!(parse_ace_engine("AV").is_err());
        assert_eq!(AceEngineChoice::AceAV.as_str(), "av");
    }

    // PLAN-971 B-PROOF-I: project-face discipline (pure std, zero ACE
    // dependency — the bridge crate carries the executable cases).

    #[test]
    fn p971_project_face_freezes_engine_and_registry_discipline() {
        let modules = [AceModuleSpec {
            package: "app",
            version: "0",
            id: "main",
            source: "fn main() int { return 0 }",
        }];
        let caps = packages_capabilities();
        let req = AceProjectRequest::simple_project(&modules, "app::main", &caps);
        assert_eq!(req.entry_def_id, "d_main");
        assert_eq!(req.capabilities, [PACKAGES_CAPABILITY.to_string()]);

        // Unknown and legacy engine names: named refusal, zero dispatch.
        for bad in ["autovm", "vm", "evaluator", "ace-native", ""] {
            let report = run_ace_project_named(bad, &req);
            assert!(
                matches!(&report.outcome, AceProjectOutcome::Denied { code, .. } if code == "ace.engine/unknown"),
                "engine `{bad}` must be refused on the project face, got {:?}",
                report.outcome
            );
            assert_eq!(report.exit_code, 1);
        }

        // Unregistered project executor: named host failure (never a
        // legacy-engine success). This test is the ONLY installer in the
        // process, so the global stays race-free under threaded tests.
        assert!(
            matches!(
                run_ace_project(AceEngineChoice::AceAV, &req).outcome,
                AceProjectOutcome::HostFailure { ref code, .. } if code == "ace.host/project-executor-unregistered"
            ),
            "unregistered project executor must be a named host failure"
        );

        struct Probe(bool);
        impl AceProjectExecutor for Probe {
            fn execute_project(&self, _request: &AceProjectRequest<'_>) -> AceProjectReport {
                AceProjectReport {
                    engine: "av",
                    outcome: AceProjectOutcome::Success {
                        value: AceValue::I32(if self.0 { 7 } else { 8 }),
                        steps: 0,
                    },
                    tests: vec![AceTestOutcome {
                        name: "probe".into(),
                        module: "app::main".into(),
                        passed: !self.0,
                        message: String::new(),
                    }],
                    exit_code: 0,
                    diag_count: 0,
                    wall_ms: 0.0,
                }
            }
        }
        let first = Arc::new(Probe(true));
        assert!(install_ace_project_executor(first.clone()).is_ok());
        // Same executor: idempotent. A different one: refused.
        assert!(install_ace_project_executor(first.clone()).is_ok());
        let second = Arc::new(Probe(false));
        assert!(install_ace_project_executor(second).is_err());
        // The installed one serves the project face.
        let served = run_ace_project(AceEngineChoice::AceAV, &req);
        assert!(matches!(
            served.outcome,
            AceProjectOutcome::Success {
                value: AceValue::I32(7),
                ..
            }
        ));
        // Shutdown ends ownership; requests become named failures again.
        assert!(shutdown_ace_project_executor());
        assert!(matches!(
            run_ace_project(AceEngineChoice::AceAV, &req).outcome,
            AceProjectOutcome::HostFailure { .. }
        ));
        assert!(!shutdown_ace_project_executor());
    }
}
