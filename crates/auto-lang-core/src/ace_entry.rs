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

    /// PLAN-982（983 文本面）：**受检文本入口**——把请求文本作为**运行时数据**
    /// 交给入口的借用 `str` 形参。默认实现是**具名拒绝**（不是静默回退）：不支持
    /// 文本的宿主不会假装支持。文本在本次执行的 arena 内分配、借用不逃逸；
    /// `ac.runtime.text.v1` 只放行"入口收借用 `str`"这一件事，
    /// [`AceExecutor::execute`] 与 [`AceRequest::args`] 的 argv 契约不变。
    fn execute_text(&self, _request: &AceRequest<'_>, _texts: &[&[u8]]) -> AceRunReport {
        AceRunReport::denied(
            "av",
            "ace.text/unsupported",
            "the installed executor has no checked text entry; the request is refused rather than silently dropped",
        )
    }

    /// PLAN-982 B-CONFIG: the configuration-document face, served by the SAME
    /// installed executor — never a second global registry. An executor that
    /// cannot serve configuration returns
    /// [`AceConfigReport::unsupported`] (a NAMED refusal); the caller must not
    /// retry on the legacy configuration engine.
    fn eval_config(
        &self,
        _request: &crate::ace_config::AceConfigRequest<'_>,
    ) -> crate::ace_config::AceConfigReport {
        crate::ace_config::AceConfigReport::unsupported()
    }
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

/// Whether an ACE executor is currently installed (the host installed the
/// bridge). Environment-dependent engine selection uses this — never a
/// semantic fallback: an installed executor that FAILS still reports the
/// failure.
pub fn ace_executor_installed() -> bool {
    match ACE_EXECUTOR.read() {
        Ok(slot) => slot.is_some(),
        Err(_) => false,
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
/// PLAN-982（983 文本面）：**受检文本入口**的门口——与 [`run_ace`] 同一套
/// 引擎选择/拒绝纪律，只是把 `texts` 作为**运行时文本数据**交给入口的借用
/// `str` 形参。没有安装执行器 / 引擎不支持文本都是具名终态，绝不回退 legacy。
pub fn run_ace_text(
    choice: AceEngineChoice,
    request: &AceRequest<'_>,
    texts: &[&[u8]],
) -> AceRunReport {
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
    let mut report = executor.execute_text(request, texts);
    report.wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    report
}

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

/// Evaluate one project configuration document through the installed ACE
/// executor.
///
/// Same discipline as [`run_ace`]: no executor / poisoned registry / any
/// executor failure is a NAMED terminal state, and the legacy configuration
/// engine (`eval_config_with_vm`) is never dispatched from here — not as a
/// retry, not as a fallback.
pub fn eval_config_with_ace(
    request: &crate::ace_config::AceConfigRequest<'_>,
) -> crate::ace_config::AceConfigReport {
    use crate::ace_config::{AceConfigReport, CONFIG_ENGINE_ID};
    let started = std::time::Instant::now();
    let executor = match ACE_EXECUTOR.read() {
        Ok(slot) => slot.clone(),
        Err(_) => {
            return AceConfigReport::host_failure(
                "ace.host/executor-lock-poisoned",
                "executor lock poisoned",
            )
        }
    };
    let Some(executor) = executor else {
        return AceConfigReport::host_failure(
            "ace.host/executor-unregistered",
            "no ACE executor installed; the host must install one (auto-lang-ace-host)",
        );
    };
    let mut report = executor.eval_config(request);
    report.wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    let _ = CONFIG_ENGINE_ID;
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

    /// PLAN-971 B: compile-only project check — diagnostics (all of them),
    /// the producer's declaration table and its discovered test candidates.
    /// The default is a NAMED refusal so an executor that predates this
    /// face still compiles and never silently pretends the check passed.
    fn check_project(&self, _request: &AceCheckRequest<'_>) -> AceCheckReport {
        AceCheckReport::host_failure(
            "?",
            "ace.host/check-unsupported",
            "the installed project executor does not implement the compile-only check face",
        )
    }

    /// PLAN-971 B: start a source-level debug session. The default is a
    /// NAMED refusal (an executor without the debug face), never a silent
    /// non-debug run.
    fn start_debug(
        &self,
        _request: &AceDebugRequest<'_>,
    ) -> Result<Box<dyn AceDebugSession>, AceDebugStartError> {
        Err(AceDebugStartError {
            code: "ace.debug/unsupported".to_string(),
            message:
                "the installed project executor does not implement the source-level debug face"
                    .to_string(),
            diagnostic: None,
        })
    }
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

// ---------------------------------------------------------------------------
// PLAN-971 B (T-02/T-08): the compile-only project CHECK face.
//
// The same explicit project modules are compiled through the installed
// project executor WITHOUT executing anything, returning (a) EVERY
// diagnostic with the original module identity and module-relative span
// (the LSP analysis backend's multi-file diagnostics), (b) the producer's
// bound declaration table, and (c) the discovered test candidates. The
// test lane in turn gates on discovery: a project with ZERO discovered
// test candidates is a NAMED failure, never a pass (V-03).
//
// This face never runs the entry or any test, and like every other ace_entry
// face it has no path into the legacy AutoVM: an unregistered executor or a
// host that does not implement the check face is a named host failure.
// ---------------------------------------------------------------------------

/// One source-level diagnostic of a compile-only project check. `module`
/// is the ORIGINAL module identity (`pkg::mod`) and `span` is relative to
/// that module's own source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AceDiagnostic {
    pub module: String,
    pub stage: String,
    pub code: String,
    pub message: String,
    pub span: Option<AceSpan>,
}

/// One declaration the producer bound for the project: the producer's
/// declaration table, not a legacy AST walk. Callers (the test lane, the
/// LSP test-discovery surface) consume these facts directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AceDeclaration {
    /// Source-level declaration name (the original name, not the symbol).
    pub name: String,
    /// Owning module identity (`pkg::mod`).
    pub module: String,
    /// The bound symbol the executor resolves the declaration by.
    pub symbol: String,
    /// Declared parameter count.
    pub params: usize,
    /// Rendered declared result type (`int`, `bool`, ...).
    pub ret: String,
    /// Module-relative declaration span.
    pub span: Option<AceSpan>,
}

/// A compile-only project check request: the project's source modules and
/// the binding root (`entry_module` selects the module the package binder
/// roots the graph at; the LSP passes the document's own module).
pub struct AceCheckRequest<'a> {
    pub modules: &'a [AceModuleSpec<'a>],
    /// Binding root module key (`pkg::mod`) — must be one of `modules`.
    pub entry_module: &'a str,
    pub capabilities: &'a [String],
    pub cancel: Option<&'a AtomicBool>,
}

impl<'a> AceCheckRequest<'a> {
    pub fn new(
        modules: &'a [AceModuleSpec<'a>],
        entry_module: &'a str,
        capabilities: &'a [String],
    ) -> Self {
        AceCheckRequest {
            modules,
            entry_module,
            capabilities,
            cancel: None,
        }
    }
}

/// Terminal states of a compile-only project check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AceCheckOutcome {
    /// Compiles with no diagnostics.
    Clean,
    /// The pipeline rejected the project; contains EVERY diagnostic (with
    /// original module identity and span), not just the primary one.
    Diagnostics(Vec<AceDiagnostic>),
    Cancelled,
    ResourceDenied {
        kind: String,
        message: String,
    },
    Denied {
        code: String,
        message: String,
    },
    HostFailure {
        code: String,
        message: String,
    },
}

/// Evidence companion of one compile-only project check.
#[derive(Debug, Clone)]
pub struct AceCheckReport {
    pub engine: &'static str,
    pub outcome: AceCheckOutcome,
    /// The producer's bound declaration table (empty unless the project
    /// compiled: a rejected project has no admitted declarations).
    pub declarations: Vec<AceDeclaration>,
    /// Discovered test candidates: zero-parameter `int`-returning
    /// declarations under the frozen test contract, the conventional entry
    /// (`main`) excluded and module initializers skipped. An empty list is
    /// evidence of zero discovery — the test lane treats it as a NAMED
    /// failure ([`AceTestRunOutcome::NoTestsDiscovered`]), never a pass.
    pub tests: Vec<AceDeclaration>,
    /// Executor-measured wall time (milliseconds; filled by
    /// [`check_ace_project`]).
    pub wall_ms: f64,
}

impl AceCheckReport {
    fn host_failure(engine: &'static str, code: &str, message: impl Into<String>) -> Self {
        AceCheckReport {
            engine,
            outcome: AceCheckOutcome::HostFailure {
                code: code.to_string(),
                message: message.into(),
            },
            declarations: Vec::new(),
            tests: Vec::new(),
            wall_ms: 0.0,
        }
    }

    fn denied(engine: &'static str, code: &str, message: impl Into<String>) -> Self {
        AceCheckReport {
            engine,
            outcome: AceCheckOutcome::Denied {
                code: code.to_string(),
                message: message.into(),
            },
            declarations: Vec::new(),
            tests: Vec::new(),
            wall_ms: 0.0,
        }
    }
}

/// Run one compile-only project check synchronously through the installed
/// project executor. Nothing is executed; no executor / unknown engine /
/// unsupported check face is a NAMED terminal state.
pub fn check_ace_project(choice: AceEngineChoice, request: &AceCheckRequest<'_>) -> AceCheckReport {
    let started = std::time::Instant::now();
    let executor = match ACE_PROJECT_EXECUTOR.read() {
        Ok(slot) => slot.clone(),
        Err(_) => {
            return AceCheckReport::host_failure(
                choice.as_str(),
                "ace.host/project-executor-lock-poisoned",
                "project executor lock poisoned",
            )
        }
    };
    let Some(executor) = executor else {
        return AceCheckReport::host_failure(
            choice.as_str(),
            "ace.host/project-executor-unregistered",
            "no ACE project executor installed; the host must install one (auto-lang-ace-host)",
        );
    };
    let mut report = executor.check_project(request);
    report.wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    report
}

/// Name-driven form of [`check_ace_project`]: an unknown name — including
/// every legacy engine name — is a named refusal with zero dispatch.
pub fn check_ace_project_named(engine: &str, request: &AceCheckRequest<'_>) -> AceCheckReport {
    let choice = match parse_ace_engine(engine) {
        Ok(choice) => choice,
        Err(message) => return AceCheckReport::denied("?", "ace.engine/unknown", message),
    };
    check_ace_project(choice, request)
}

/// A discovered-test run request: the project modules plus the entry the
/// project lane runs after the tests. Test NAMES are not carried — they are
/// discovered from the producer's declaration table (T-02: discovery moved
/// into the producer), and zero discovery is a named failure.
pub struct AceTestRunRequest<'a> {
    pub modules: &'a [AceModuleSpec<'a>],
    pub entry_module: &'a str,
    /// Entry def id (`"d_main"` for the conventional entry).
    pub entry_def_id: &'a str,
    pub capabilities: &'a [String],
    pub wall: Option<Duration>,
    pub cancel: Option<&'a AtomicBool>,
}

impl<'a> AceTestRunRequest<'a> {
    pub fn simple(
        modules: &'a [AceModuleSpec<'a>],
        entry_module: &'a str,
        capabilities: &'a [String],
    ) -> Self {
        AceTestRunRequest {
            modules,
            entry_module,
            entry_def_id: "d_main",
            capabilities,
            wall: None,
            cancel: None,
        }
    }
}

/// Terminal states of a discovered-test run.
#[derive(Debug, Clone, PartialEq)]
pub enum AceTestRunOutcome {
    /// Every discovered test passed AND the project entry run succeeded
    /// (the protocol exit is 0).
    Passed,
    /// At least one discovered test failed, or the entry run did not
    /// succeed. `message` names the first failure.
    Failed {
        message: String,
    },
    /// The producer discovered ZERO test candidates: a test claim without a
    /// test is a NAMED failure, never a pass (V-03).
    NoTestsDiscovered {
        message: String,
    },
    /// The project did not compile; diagnostic carries the original module
    /// identity and module-relative span.
    Diagnostic(AceDiagnostic),
    Denied {
        code: String,
        message: String,
    },
    ResourceDenied {
        kind: String,
        message: String,
    },
    Cancelled,
    HostFailure {
        code: String,
        message: String,
    },
}

/// Evidence companion of one discovered-test run.
#[derive(Debug, Clone)]
pub struct AceTestRunReport {
    pub engine: &'static str,
    pub outcome: AceTestRunOutcome,
    /// The producer's discovered candidates (empty when discovery was zero
    /// or the project did not compile).
    pub discovered: Vec<AceDeclaration>,
    /// Per-test results in discovery order (empty when nothing ran).
    pub tests: Vec<AceTestOutcome>,
    /// `0` iff the outcome is [`AceTestRunOutcome::Passed`]; `1` otherwise
    /// — a failed test, an empty discovery and a compile rejection never
    /// exit zero.
    pub exit_code: i32,
    pub wall_ms: f64,
}

impl AceTestRunReport {
    fn from_check(engine: &'static str, check: AceCheckReport) -> Option<Self> {
        let outcome = match &check.outcome {
            AceCheckOutcome::Clean => return None,
            AceCheckOutcome::Diagnostics(diags) => match diags.first() {
                Some(d) => AceTestRunOutcome::Diagnostic(d.clone()),
                None => AceTestRunOutcome::HostFailure {
                    code: "ace.test/empty-diagnostics".to_string(),
                    message: "the project was rejected without a diagnostic".to_string(),
                },
            },
            AceCheckOutcome::Cancelled => AceTestRunOutcome::Cancelled,
            AceCheckOutcome::ResourceDenied { kind, message } => {
                AceTestRunOutcome::ResourceDenied {
                    kind: kind.clone(),
                    message: message.clone(),
                }
            }
            AceCheckOutcome::Denied { code, message } => AceTestRunOutcome::Denied {
                code: code.clone(),
                message: message.clone(),
            },
            AceCheckOutcome::HostFailure { code, message } => AceTestRunOutcome::HostFailure {
                code: code.clone(),
                message: message.clone(),
            },
        };
        Some(AceTestRunReport {
            engine,
            outcome,
            discovered: Vec::new(),
            tests: Vec::new(),
            exit_code: 1,
            wall_ms: 0.0,
        })
    }
}

/// Run the project's DISCOVERED tests: compile-check the project through the
/// installed project executor, refuse a zero-discovery result, then execute
/// every discovered test on the project lane (each in its own generation,
/// per the frozen A test contract), followed by the entry run. Discovery
/// lives in the producer — this function carries no test names.
pub fn run_ace_project_tests(
    choice: AceEngineChoice,
    request: &AceTestRunRequest<'_>,
) -> AceTestRunReport {
    let started = std::time::Instant::now();
    let check_request = AceCheckRequest {
        modules: request.modules,
        entry_module: request.entry_module,
        capabilities: request.capabilities,
        cancel: request.cancel,
    };
    let check = check_ace_project(choice, &check_request);
    let engine = check.engine;
    if let Some(mut report) = AceTestRunReport::from_check(engine, check.clone()) {
        report.wall_ms = started.elapsed().as_secs_f64() * 1000.0;
        return report;
    }
    let discovered = check.tests;
    if discovered.is_empty() {
        return AceTestRunReport {
            engine,
            outcome: AceTestRunOutcome::NoTestsDiscovered {
                message: "the producer discovered zero test candidates in the project \
                          (frozen test contract: a zero-parameter `int` declaration, entry `main` excluded)"
                    .to_string(),
            },
            discovered,
            tests: Vec::new(),
            exit_code: 1,
            wall_ms: started.elapsed().as_secs_f64() * 1000.0,
        };
    }
    let names: Vec<&str> = discovered.iter().map(|d| d.name.as_str()).collect();
    let project_request = AceProjectRequest {
        modules: request.modules,
        entry_module: request.entry_module,
        entry_def_id: request.entry_def_id,
        tests: &names,
        capabilities: request.capabilities,
        wall: request.wall,
        cancel: request.cancel,
    };
    let project = run_ace_project(choice, &project_request);
    let outcome = match &project.outcome {
        AceProjectOutcome::Success { .. } => {
            // F-971-B-02 (review B2): a `Success` with no test results is NOT
            // a passing test lane. Every discovered name must come back
            // exactly once; a host that drops or renames results is a named
            // contract failure, never a vacuous `all()` pass.
            let mut covered = vec![false; names.len()];
            for test in &project.tests {
                if let Some(slot) = names.iter().position(|name| *name == test.name) {
                    covered[slot] = true;
                }
            }
            let complete = project.tests.len() == names.len() && covered.iter().all(|c| *c);
            if !complete {
                AceTestRunOutcome::HostFailure {
                    code: "ace.test/results-mismatch".to_string(),
                    message: format!(
                        "the discovered test lane returned {} result(s) for {} discovered test(s); \
                         a test claim without a matching result is never a pass",
                        project.tests.len(),
                        names.len()
                    ),
                }
            } else if project.tests.iter().all(|t| t.passed) {
                AceTestRunOutcome::Passed
            } else {
                AceTestRunOutcome::Failed {
                    message: project
                        .tests
                        .iter()
                        .find(|t| !t.passed)
                        .map(|t| format!("{}: {}", t.name, t.message))
                        .unwrap_or_else(|| "a discovered test failed".to_string()),
                }
            }
        }
        AceProjectOutcome::Diagnostic {
            module,
            stage,
            code,
            message,
            span,
        } => AceTestRunOutcome::Diagnostic(AceDiagnostic {
            module: module.clone(),
            stage: stage.clone(),
            code: code.clone(),
            message: message.clone(),
            span: *span,
        }),
        AceProjectOutcome::Denied { code, message } => AceTestRunOutcome::Denied {
            code: code.clone(),
            message: message.clone(),
        },
        AceProjectOutcome::ResourceDenied { kind, message } => AceTestRunOutcome::ResourceDenied {
            kind: kind.clone(),
            message: message.clone(),
        },
        AceProjectOutcome::Cancelled => AceTestRunOutcome::Cancelled,
        AceProjectOutcome::HostFailure { code, message } => AceTestRunOutcome::HostFailure {
            code: code.clone(),
            message: message.clone(),
        },
    };
    let exit_code = if matches!(outcome, AceTestRunOutcome::Passed) {
        0
    } else {
        1
    };
    AceTestRunReport {
        engine,
        outcome,
        discovered,
        tests: project.tests,
        exit_code,
        wall_ms: started.elapsed().as_secs_f64() * 1000.0,
    }
}

/// Name-driven form of [`run_ace_project_tests`].
pub fn run_ace_project_tests_named(
    engine: &str,
    request: &AceTestRunRequest<'_>,
) -> AceTestRunReport {
    let choice = match parse_ace_engine(engine) {
        Ok(choice) => choice,
        Err(message) => {
            return AceTestRunReport {
                engine: "?",
                outcome: AceTestRunOutcome::Denied {
                    code: "ace.engine/unknown".to_string(),
                    message,
                },
                discovered: Vec::new(),
                tests: Vec::new(),
                exit_code: 1,
                wall_ms: 0.0,
            }
        }
    };
    run_ace_project_tests(choice, request)
}

// ---------------------------------------------------------------------------
// PLAN-971 B (T-02): the source-level DEBUG face.
//
// One debug session = one compiled project generation plus a PAUSED AV chain.
// The session's stops are the producer's own per-instruction source spans of
// the ORIGINAL module source (never a legacy AST position), and its stack /
// locals are the live AV frames of that same chain. Commands are the classic
// four: continue (to the next breakpoint), step (one instruction), step-over
// (to the next source position at the caller's depth) and step-out (to the
// caller frame). Closing a session reclaims its scratch artifacts.
// ---------------------------------------------------------------------------

/// One source-level breakpoint: a function (by the bound SYMBOL the check
/// face reports) optionally narrowed to a source span inside that module.
/// `span: None` breaks at the function's first instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AceBreakpoint {
    /// Owning module identity (`pkg::mod`).
    pub module: String,
    /// The bound declaration symbol (from [`AceDeclaration::symbol`]).
    pub symbol: String,
    /// Module-relative span; `None` = the function's entry instruction.
    pub span: Option<AceSpan>,
}

/// One debug command (the classic four).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AceDebugCommand {
    /// Run to the next breakpoint (or the end of the run).
    Continue,
    /// Execute exactly one instruction.
    Step,
    /// Run to the next source position at the caller's frame depth.
    StepOver,
    /// Run until the current frame returns (caller depth).
    StepOut,
}

/// Why a debug session is stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AceDebugStop {
    /// Stopped at the entry's first instruction (session start).
    Entry,
    /// Stopped on a breakpoint (index into the start request's list).
    Breakpoint { index: usize },
    /// Stopped after a single instruction step.
    Step,
    /// Stopped after a step-over.
    StepOver,
    /// Stopped after a step-out.
    StepOut,
    /// The run ended (the outcome is on the snapshot).
    Finished,
}

/// One named local of a stopped frame (the value is the executor's own
/// rendering; handles/aggregates are named, never coerced).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AceDebugLocal {
    pub slot: u32,
    pub name: String,
    pub value: String,
}

/// One frame of a stopped call stack, OUTERMOST first. `function` is the
/// source-level declaration name and `module` its owning module identity;
/// `span` is the ORIGINAL source span of the instruction the frame is
/// stopped before.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AceDebugFrame {
    pub module: String,
    pub function: String,
    pub span: Option<AceSpan>,
    pub locals: Vec<AceDebugLocal>,
}

/// The full state of one stop.
#[derive(Debug, Clone, PartialEq)]
pub struct AceDebugSnapshot {
    pub stop: AceDebugStop,
    /// Call stack, outermost first (empty once the run finished).
    pub frames: Vec<AceDebugFrame>,
    /// Cumulative executed instructions of this chain.
    pub steps: u64,
    /// `Some` iff `stop == Finished`: the terminal run outcome.
    pub outcome: Option<AceProjectOutcome>,
    /// Human-readable note (a named stop reason or failure detail).
    pub message: String,
}

/// A live debug session over one compiled project generation. The session
/// owns a paused executor chain (contiguous values), so it is bound to the
/// thread that created it.
pub trait AceDebugSession {
    /// The current stop (the start stop before any resume).
    fn snapshot(&self) -> AceDebugSnapshot;

    /// Resume from the current stop with `command`; returns the next stop.
    /// Once the run finished, further resumes return the same finished
    /// snapshot (idempotent).
    fn resume(&mut self, command: AceDebugCommand) -> AceDebugSnapshot;

    /// Release the session: drop the paused chain, the compiled generation
    /// and the per-session scratch store. Idempotent.
    fn close(&mut self);

    /// Replace the session's breakpoint set. A debug UI typically starts the
    /// session at the entry stop and sets breakpoints from there, so the set
    /// is mutable across stops; the default keeps the start set.
    fn set_breakpoints(&mut self, _breakpoints: &[AceBreakpoint]) {}

    /// Request cancellation of this session's chain (the debug UI's stop
    /// button). The next resume observes it and ends with the named
    /// `Cancelled` outcome; the start request's flag is only the initial
    /// state, so a live session needs this handle. The default keeps the
    /// chain running: a host that wants a live stop MUST implement it (the
    /// reference host does), it is not an optional nicety.
    fn cancel(&mut self) {}
}

/// A debug start request: the project modules, the entry, the breakpoints
/// and the per-session budgets.
pub struct AceDebugRequest<'a> {
    pub modules: &'a [AceModuleSpec<'a>],
    pub entry_module: &'a str,
    pub entry_def_id: &'a str,
    pub capabilities: &'a [String],
    pub breakpoints: &'a [AceBreakpoint],
    /// Wall budget of the WHOLE session (a stop that would exceed it
    /// finishes the session with a named resource outcome).
    pub wall: Option<Duration>,
    pub cancel: Option<&'a AtomicBool>,
}

impl<'a> AceDebugRequest<'a> {
    pub fn simple(
        modules: &'a [AceModuleSpec<'a>],
        entry_module: &'a str,
        capabilities: &'a [String],
        breakpoints: &'a [AceBreakpoint],
    ) -> Self {
        AceDebugRequest {
            modules,
            entry_module,
            entry_def_id: "d_main",
            capabilities,
            breakpoints,
            wall: None,
            cancel: None,
        }
    }
}

/// A named debug start failure (unregistered/unsupported host, a project
/// that does not compile, or an entry that cannot be paused).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AceDebugStartError {
    pub code: String,
    pub message: String,
    /// Present when the project itself was rejected (original module
    /// identity + span).
    pub diagnostic: Option<AceDiagnostic>,
}

/// Start one debug session through the installed project executor. No
/// executor / unknown engine / unsupported debug face is a NAMED start
/// failure; the legacy AutoVM is never dispatched.
pub fn start_ace_debug(
    choice: AceEngineChoice,
    request: &AceDebugRequest<'_>,
) -> Result<Box<dyn AceDebugSession>, AceDebugStartError> {
    let executor = match ACE_PROJECT_EXECUTOR.read() {
        Ok(slot) => slot.clone(),
        Err(_) => {
            return Err(AceDebugStartError {
                code: "ace.host/project-executor-lock-poisoned".to_string(),
                message: "project executor lock poisoned".to_string(),
                diagnostic: None,
            })
        }
    };
    let Some(executor) = executor else {
        return Err(AceDebugStartError {
            code: "ace.host/project-executor-unregistered".to_string(),
            message:
                "no ACE project executor installed; the host must install one (auto-lang-ace-host)"
                    .to_string(),
            diagnostic: None,
        });
    };
    let _ = choice;
    executor.start_debug(request)
}

/// Name-driven form of [`start_ace_debug`]: an unknown name — including
/// every legacy engine name — is a named refusal with zero dispatch.
pub fn start_ace_debug_named(
    engine: &str,
    request: &AceDebugRequest<'_>,
) -> Result<Box<dyn AceDebugSession>, AceDebugStartError> {
    match parse_ace_engine(engine) {
        Ok(choice) => start_ace_debug(choice, request),
        Err(message) => Err(AceDebugStartError {
            code: "ace.engine/unknown".to_string(),
            message,
            diagnostic: None,
        }),
    }
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

        // PLAN-971 B (T-02/T-08): the compile-only check face and the
        // zero-discovery gate, frozen WITHOUT any ACE dependency.
        let modules = [AceModuleSpec {
            package: "app",
            version: "0",
            id: "main",
            source: "fn main() int { return 0 }",
        }];
        let check_req = AceCheckRequest::new(&modules, "app::main", &caps);
        for bad in ["autovm", "vm", "evaluator", ""] {
            assert!(
                matches!(&check_ace_project_named(bad, &check_req).outcome, AceCheckOutcome::Denied { code, .. } if code == "ace.engine/unknown"),
                "engine `{bad}` must be refused on the check face"
            );
        }
        assert!(matches!(
            check_ace_project(AceEngineChoice::AceAV, &check_req).outcome,
            AceCheckOutcome::HostFailure { ref code, .. } if code == "ace.host/project-executor-unregistered"
        ));
        // An executor that predates the check face refuses it by NAME (never
        // a silently clean check), and the discovered-test run inherits it.
        let legacy_face = Arc::new(Probe(true));
        assert!(install_ace_project_executor(legacy_face).is_ok());
        assert!(matches!(
            check_ace_project(AceEngineChoice::AceAV, &check_req).outcome,
            AceCheckOutcome::HostFailure { ref code, .. } if code == "ace.host/check-unsupported"
        ));
        let test_req = AceTestRunRequest::simple(&modules, "app::main", &caps);
        let run = run_ace_project_tests(AceEngineChoice::AceAV, &test_req);
        assert!(
            matches!(&run.outcome, AceTestRunOutcome::HostFailure { code, .. } if code == "ace.host/check-unsupported"),
            "unsupported check face must not become a test pass: {:?}",
            run.outcome
        );
        assert_eq!(run.exit_code, 1);
        assert!(shutdown_ace_project_executor());

        struct CheckProbe {
            tests: Vec<AceDeclaration>,
            /// F-971-B-02: when true the probe drops every test result while
            /// still reporting a successful project run (the vacuous-pass
            /// hole the gate must refuse).
            drop_results: bool,
        }
        impl AceProjectExecutor for CheckProbe {
            fn execute_project(&self, request: &AceProjectRequest<'_>) -> AceProjectReport {
                AceProjectReport {
                    engine: "av",
                    outcome: AceProjectOutcome::Success {
                        value: AceValue::I32(0),
                        steps: 0,
                    },
                    tests: if self.drop_results {
                        Vec::new()
                    } else {
                        request
                            .tests
                            .iter()
                            .map(|name| AceTestOutcome {
                                name: (*name).to_string(),
                                module: "app::main".to_string(),
                                passed: true,
                                message: String::new(),
                            })
                            .collect()
                    },
                    exit_code: 0,
                    diag_count: 0,
                    wall_ms: 0.0,
                }
            }
            fn check_project(&self, _request: &AceCheckRequest<'_>) -> AceCheckReport {
                AceCheckReport {
                    engine: "av",
                    outcome: AceCheckOutcome::Clean,
                    declarations: self.tests.clone(),
                    tests: self.tests.clone(),
                    wall_ms: 0.0,
                }
            }
        }
        // Zero discovered test candidates: a test claim without a test is a
        // NAMED failure (V-03) — never `Passed`, never exit 0.
        assert!(install_ace_project_executor(Arc::new(CheckProbe {
            tests: vec![],
            drop_results: false
        }))
        .is_ok());
        let run = run_ace_project_tests(AceEngineChoice::AceAV, &test_req);
        assert!(
            matches!(&run.outcome, AceTestRunOutcome::NoTestsDiscovered { .. }),
            "zero discovery must be a named failure, got {:?}",
            run.outcome
        );
        assert_eq!(run.exit_code, 1);
        assert!(shutdown_ace_project_executor());
        // One discovered candidate: the run executes it through the project
        // lane and exits zero only when it passed.
        let discovered = AceDeclaration {
            name: "check_one".to_string(),
            module: "app::main".to_string(),
            symbol: "d_check_one".to_string(),
            params: 0,
            ret: "int".to_string(),
            span: Some(AceSpan { start: 0, end: 12 }),
        };
        assert!(install_ace_project_executor(Arc::new(CheckProbe {
            tests: vec![discovered.clone()],
            drop_results: false
        }))
        .is_ok());
        let run = run_ace_project_tests(AceEngineChoice::AceAV, &test_req);
        assert_eq!(run.outcome, AceTestRunOutcome::Passed);
        assert_eq!(run.exit_code, 0);
        assert_eq!(run.discovered.len(), 1);
        assert_eq!(run.tests[0].name, "check_one");
        assert!(shutdown_ace_project_executor());
        // F-971-B-02: a host that reports a successful run but returns NO test
        // results must not pass vacuously — the discovery gate alone is not
        // enough; the executed result set must cover every discovered name.
        assert!(install_ace_project_executor(Arc::new(CheckProbe {
            tests: vec![discovered],
            drop_results: true
        }))
        .is_ok());
        let run = run_ace_project_tests(AceEngineChoice::AceAV, &test_req);
        assert!(
            matches!(&run.outcome, AceTestRunOutcome::HostFailure { code, .. } if code == "ace.test/results-mismatch"),
            "dropped test results must be a named contract failure, got {:?}",
            run.outcome
        );
        assert_eq!(run.exit_code, 1, "a missing result never exits zero");
        assert!(shutdown_ace_project_executor());
    }
}
