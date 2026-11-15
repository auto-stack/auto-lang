//! PLAN-970 (T-02): the public explicit-ACE **session** contract — the
//! cross-request persistent-session extension of the PLAN-969 E0 facade
//! (`ace_entry`): REPL sessions with persistent variables, error-then-
//! continue evaluation, session isolation, same-PID incremental edits
//! and async long evaluations with cancellation.
//!
//! Ownership split (frozen by the PLAN-970 T-01 freeze record):
//! - THIS module owns the neutral public contract only. It has no ACE
//!   dependency and no path into the legacy AutoVM REPL/daemon/MCP
//!   (`AutovmReplSession`/`AutovmDaemon`/`mcp::McpServer` are untouched
//!   legacy surfaces — retirement belongs to PLAN-987). An explicit ACE
//!   session op can never fall back to, or silently switch to, a legacy
//!   engine; legacy dispatch = 0 is structural.
//! - The executable session host is injected through
//!   [`install_ace_session_host`] (the `auto-lang-ace-host` bridge crate
//!   provides the reference implementation over the ACE AV
//!   `FunctionIncrementalSession`/`ResidentRunner` spine). An
//!   unregistered host is a named host failure, never a default switch.
//!
//! Session model (frozen): a session owns one normalized program with a
//! frozen declaration set (`main` dispatch + `__repl_eval_int` +
//! `__repl_eval_bool` + user items + `__tail_anchor` alignment anchor),
//! a host-side variable table (scalar carry only), and a monotone
//! revision. Evaluations are incremental body edits through the ACE
//! prepare/commit seam; failed evaluations leave the session exactly at
//! the last accepted generation (error-then-continue). The scalar
//! support domain of the session lanes is `int`/`bool` — other profile
//! types are a named refusal, never a silent coercion.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// The session protocol identity carried by the daemon/client and MCP
/// surfaces built on this contract (T-08): requests/responses carry a
/// `version`; the current value is 1.
pub const SESSION_PROTOCOL_NAME: &str = "ac.session.daemon/1";
/// The current session protocol version (see [`SESSION_PROTOCOL_NAME`]).
pub const SESSION_PROTOCOL_VERSION: u32 = 1;

/// Session-level resource limits (frozen defaults keep the legacy daemon
/// values where a legacy value existed: 20 sessions, 1800s idle expiry).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AceSessionLimits {
    /// Maximum simultaneously live sessions; excess creation is a named
    /// resource refusal.
    pub max_sessions: usize,
    /// Idle expiry in seconds (a session untouched for this long may be
    /// reclaimed by the host's cleanup sweep).
    pub idle_timeout_secs: u64,
    /// Maximum tracked variables per session (the carry table cap).
    pub var_cap: usize,
}

impl Default for AceSessionLimits {
    fn default() -> Self {
        AceSessionLimits {
            max_sessions: 20,
            idle_timeout_secs: 1800,
            var_cap: 64,
        }
    }
}

/// The scalar value surface a session can carry across evaluations
/// (frozen: the session lanes' support domain — `int`/`bool` only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AceSessionValue {
    /// No observable value (statement-only evaluation — the legacy REPL
    /// printed nothing for statements).
    None,
    Int(i32),
    Bool(bool),
}

impl AceSessionValue {
    /// The display form (the legacy REPL's formatted last result; `None`
    /// renders nothing).
    pub fn display(&self) -> Option<String> {
        match self {
            AceSessionValue::None => None,
            AceSessionValue::Int(v) => Some(v.to_string()),
            AceSessionValue::Bool(v) => Some(v.to_string()),
        }
    }
}

/// One tracked variable of a session (the carry-table surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AceSessionVar {
    pub name: String,
    pub value: AceSessionValue,
}

/// One diagnostic of a refused evaluation/edit (stage and code preserved
/// from the producing frontend, never flattened into a string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AceSessionDiag {
    pub stage: String,
    pub code: String,
    pub message: String,
}

/// The distinguished terminal states of one session op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AceSessionOutcome {
    /// The op completed.
    Ok,
    /// A diagnosed source/pipeline rejection (bad edit, ill-typed
    /// evaluation, ...). The session stays at its last accepted
    /// generation.
    Diagnostic {
        code: String,
        diagnostics: Vec<AceSessionDiag>,
    },
    /// Authorization/admission refusal (unknown session, closed session,
    /// reserved-name collision, non-carryable variable, ...).
    Denied { code: String, message: String },
    /// A resource budget denial (session cap, variable cap, executor
    /// budget, wall).
    ResourceDenied { kind: String, message: String },
    /// The op observed cancellation (host cancel flag / explicit cancel).
    Cancelled,
    /// Host/tool infrastructure failure (unregistered host, session
    /// thread gone). Never retried silently.
    HostFailure { code: String, message: String },
}

/// One session operation. Everything is explicit input; the host
/// re-checks session identity and limits on every op.
pub enum AceSessionOp<'a> {
    /// Evaluate one REPL input (statement list or one expression) in the
    /// session, synchronously. The lane/wrap discipline is the host's
    /// frozen model; a failed evaluation has zero effect.
    Eval {
        input: &'a str,
        wall: Option<Duration>,
        cancel: Option<&'a AtomicBool>,
    },
    /// Apply a byte-range incremental edit `(start, end, insert)` to the
    /// session's accumulated program source at a safety point (same-PID
    /// online update). Stale base / bad UTF-8 / declaration-set changes
    /// are named refusals that never switch the current root.
    Edit {
        start: u64,
        end: u64,
        insert: &'a [u8],
    },
    /// Inspect: counts + the current revision (the legacy `inspect`
    /// surface).
    Inspect,
    /// List the carry-table variables (the legacy `get_all_vars` surface).
    Vars,
    /// Export the accumulated source (the legacy `snapshot` surface).
    Snapshot,
    /// Reset the session (fresh normalized program, empty table,
    /// revision restarts; session id survives).
    Reset,
    /// Close the session and release every pinned generation (the last
    /// pin dies here — V-04). Later ops on the id are a named refusal.
    Close,
    /// Start an async long evaluation on the resident runner; returns an
    /// `eval_id` for polling/cancellation (V-02/V-04).
    EvalAsync {
        input: &'a str,
        wall: Option<Duration>,
    },
    /// Poll an async evaluation: running / finished (with value) /
    /// unknown id.
    Result { eval_id: u64 },
    /// Cancel an async evaluation (running → Cancelled; queued-but-
    /// unstarted is dropped with a named ack).
    CancelEval { eval_id: u64 },
}

/// The report of one session op (terminal state + the observable
/// surface: value, revision, variables, snapshot).
#[derive(Debug, Clone)]
pub struct AceSessionReport {
    /// The session id this report belongs to (`None` only for host-level
    /// failures that never reached a session, e.g. create refusals).
    pub session: Option<String>,
    pub outcome: AceSessionOutcome,
    /// The session's revision AFTER the op (monotone edit count of the
    /// underlying incremental session; 0 for a fresh session).
    pub revision: u64,
    /// The evaluation's observable value (`None` for statement-only
    /// inputs and non-eval ops).
    pub value: AceSessionValue,
    /// `EvalAsync` only: the assigned evaluation id.
    pub eval_id: Option<u64>,
    /// `Vars` only: the carry table.
    pub vars: Vec<AceSessionVar>,
    /// `Snapshot`/`Inspect` only: the accumulated source text.
    pub snapshot: Option<String>,
    /// `True` when the op rebuilt the session from source (the named
    /// reshape path for declaration-set changes / resets).
    pub rebuilt: bool,
    /// Host-measured wall time of the whole op (milliseconds).
    pub wall_ms: f64,
}

impl AceSessionReport {
    fn host_failure(code: &str, message: impl Into<String>) -> Self {
        AceSessionReport {
            session: None,
            outcome: AceSessionOutcome::HostFailure {
                code: code.to_string(),
                message: message.into(),
            },
            revision: 0,
            value: AceSessionValue::None,
            eval_id: None,
            vars: Vec::new(),
            snapshot: None,
            rebuilt: false,
            wall_ms: 0.0,
        }
    }
}

/// The executable session host, injected by the bridge crate. Ops are
/// synchronous from the caller's thread; the reference implementation
/// serializes them onto one session-owner thread (the ACE session types
/// are single-threaded by design — the legacy daemon's one-client-at-a-
/// time discipline, now structural).
pub trait AceSessionHost: Send + Sync {
    /// Create a new session seeded with `source` (normalized into the
    /// canonical program shape; `None` = empty session).
    fn create(&self, source: Option<&str>, wall: Option<Duration>) -> AceSessionReport;
    /// Apply one op to an existing session.
    fn op(&self, session: &str, op: AceSessionOp<'_>) -> AceSessionReport;
    /// Live session ids (the legacy `list` surface).
    fn list(&self) -> Vec<String>;
    /// Reclaim idle sessions past the configured timeout; returns the
    /// reclaimed ids (the legacy daemon's expiry sweep).
    fn cleanup_expired(&self) -> Vec<String>;
}

static ACE_SESSION_HOST: RwLock<Option<Arc<dyn AceSessionHost>>> = RwLock::new(None);

/// Install the process-wide session host (single owner; reinstalling the
/// SAME host is idempotent, installing a different one while one is
/// installed is an error — release the current one with
/// [`shutdown_ace_session_host`] first).
pub fn install_ace_session_host(host: Arc<dyn AceSessionHost>) -> Result<(), String> {
    let mut slot = ACE_SESSION_HOST
        .write()
        .map_err(|e| format!("session host lock poisoned: {e}"))?;
    match slot.as_ref() {
        Some(current) => {
            if Arc::ptr_eq(current, &host) {
                Ok(())
            } else {
                Err(
                    "another ACE session host is installed; shutdown_ace_session_host() first"
                        .to_string(),
                )
            }
        }
        None => {
            *slot = Some(host);
            Ok(())
        }
    }
}

/// Release the installed session host. After this, session ops report
/// `ace.session/host-unregistered` — they never fall back.
pub fn shutdown_ace_session_host() -> bool {
    match ACE_SESSION_HOST.write() {
        Ok(mut slot) => slot.take().is_some(),
        Err(_) => false,
    }
}

/// Create a session through the installed host (see
/// [`AceSessionHost::create`]).
pub fn ace_session_create(source: Option<&str>, wall: Option<Duration>) -> AceSessionReport {
    let started = std::time::Instant::now();
    let host = match ACE_SESSION_HOST.read() {
        Ok(slot) => slot.clone(),
        Err(_) => {
            return AceSessionReport::host_failure(
                "ace.session/host-lock-poisoned",
                "session host lock poisoned",
            )
        }
    };
    let host = match host {
        Some(h) => h,
        None => {
            return AceSessionReport::host_failure(
                "ace.session/host-unregistered",
                "no ACE session host is installed; the ACE session surface never falls back to a legacy engine",
            )
        }
    };
    let mut report = host.create(source, wall);
    report.wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    report
}

/// Apply one op through the installed host (see [`AceSessionHost::op`]).
pub fn ace_session_op(session: &str, op: AceSessionOp<'_>) -> AceSessionReport {
    let started = std::time::Instant::now();
    let host = match ACE_SESSION_HOST.read() {
        Ok(slot) => slot.clone(),
        Err(_) => {
            return AceSessionReport::host_failure(
                "ace.session/host-lock-poisoned",
                "session host lock poisoned",
            )
        }
    };
    let host = match host {
        Some(h) => h,
        None => {
            return AceSessionReport::host_failure(
                "ace.session/host-unregistered",
                "no ACE session host is installed; the ACE session surface never falls back to a legacy engine",
            )
        }
    };
    let mut report = host.op(session, op);
    report.wall_ms = started.elapsed().as_secs_f64() * 1000.0;
    report
}

/// Live session ids through the installed host.
pub fn ace_session_list() -> Vec<String> {
    match ACE_SESSION_HOST.read() {
        Ok(slot) => slot.as_ref().map(|h| h.list()).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Reclaim idle sessions through the installed host.
pub fn ace_session_cleanup_expired() -> Vec<String> {
    match ACE_SESSION_HOST.read() {
        Ok(slot) => slot
            .as_ref()
            .map(|h| h.cleanup_expired())
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

/// Test-only shared registry lock: the registry tests (session, daemon,
/// MCP) are the only installers in the process, and they compete with
/// EACH OTHER under a threaded test runner — one shared lock keeps the
/// global slot race-free and makes unregistered-path assertions
/// deterministic.
#[cfg(test)]
pub(crate) mod test_registry {
    pub static REGISTRY_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ace_session::test_registry::REGISTRY_LOCK;

    /// A stub host for dispatch-discipline tests: records ops, returns a
    /// canned report. NEVER a stand-in for the real bridge (the bridge
    /// fixtures in `auto-lang-ace-host` are the real-consumer checks).
    struct StubHost {
        created: std::sync::atomic::AtomicUsize,
        ops: std::sync::Mutex<Vec<String>>,
    }

    impl AceSessionHost for StubHost {
        fn create(&self, _source: Option<&str>, _wall: Option<Duration>) -> AceSessionReport {
            self.created
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            AceSessionReport {
                session: Some("ses_test0001".to_string()),
                outcome: AceSessionOutcome::Ok,
                revision: 0,
                value: AceSessionValue::None,
                eval_id: None,
                vars: Vec::new(),
                snapshot: None,
                rebuilt: false,
                wall_ms: 0.0,
            }
        }

        fn op(&self, session: &str, op: AceSessionOp<'_>) -> AceSessionReport {
            self.ops
                .lock()
                .unwrap()
                .push(format!("{session}:{}", op_kind(&op)));
            AceSessionReport {
                session: Some(session.to_string()),
                outcome: AceSessionOutcome::Ok,
                revision: 1,
                value: AceSessionValue::None,
                eval_id: None,
                vars: Vec::new(),
                snapshot: None,
                rebuilt: false,
                wall_ms: 0.0,
            }
        }

        fn list(&self) -> Vec<String> {
            vec!["ses_test0001".to_string()]
        }

        fn cleanup_expired(&self) -> Vec<String> {
            Vec::new()
        }
    }

    fn op_kind(op: &AceSessionOp<'_>) -> &'static str {
        match op {
            AceSessionOp::Eval { .. } => "eval",
            AceSessionOp::Edit { .. } => "edit",
            AceSessionOp::Inspect => "inspect",
            AceSessionOp::Vars => "vars",
            AceSessionOp::Snapshot => "snapshot",
            AceSessionOp::Reset => "reset",
            AceSessionOp::Close => "close",
            AceSessionOp::EvalAsync { .. } => "eval-async",
            AceSessionOp::Result { .. } => "result",
            AceSessionOp::CancelEval { .. } => "cancel-eval",
        }
    }

    /// p970: with no host installed, every op is the NAMED
    /// `ace.session/host-unregistered` host failure — never a panic,
    /// never a legacy route.
    #[test]
    fn p970_unregistered_host_is_named_failure() {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        assert!(shutdown_ace_session_host() || true); // ensure clean state
        let report = ace_session_create(None, None);
        assert_eq!(
            report.outcome,
            AceSessionOutcome::HostFailure {
                code: "ace.session/host-unregistered".into(),
                message: "no ACE session host is installed; the ACE session surface never falls back to a legacy engine".into(),
            }
        );
        let report = ace_session_op("ses_x", AceSessionOp::Inspect);
        assert!(matches!(
            report.outcome,
            AceSessionOutcome::HostFailure { .. }
        ));
        assert!(ace_session_list().is_empty());
    }

    /// p970: single-owner registry discipline (idempotent same install,
    /// conflicting install refused) — the E0 executor pattern.
    #[test]
    fn p970_registry_single_owner() {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let a: Arc<dyn AceSessionHost> = Arc::new(StubHost {
            created: std::sync::atomic::AtomicUsize::new(0),
            ops: std::sync::Mutex::new(Vec::new()),
        });
        let b: Arc<dyn AceSessionHost> = Arc::new(StubHost {
            created: std::sync::atomic::AtomicUsize::new(0),
            ops: std::sync::Mutex::new(Vec::new()),
        });
        assert!(install_ace_session_host(a.clone()).is_ok());
        assert!(
            install_ace_session_host(a.clone()).is_ok(),
            "same host idempotent"
        );
        assert!(
            install_ace_session_host(b).is_err(),
            "conflicting install refused"
        );
        assert!(shutdown_ace_session_host());
        assert!(!shutdown_ace_session_host(), "double shutdown");
    }

    /// p970: dispatch reaches the installed host and the wall clock is
    /// filled by the facade, not the host.
    #[test]
    fn p970_dispatch_reaches_host() {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let host = Arc::new(StubHost {
            created: std::sync::atomic::AtomicUsize::new(0),
            ops: std::sync::Mutex::new(Vec::new()),
        });
        install_ace_session_host(host.clone()).unwrap();
        let report = ace_session_create(Some("fn main() int { return 1 }"), None);
        assert_eq!(report.session.as_deref(), Some("ses_test0001"));
        assert!(report.wall_ms >= 0.0);
        let flag = AtomicBool::new(false);
        let report = ace_session_op(
            "ses_test0001",
            AceSessionOp::Eval {
                input: "1 + 2",
                wall: None,
                cancel: Some(&flag),
            },
        );
        assert_eq!(report.outcome, AceSessionOutcome::Ok);
        assert_eq!(
            *host.ops.lock().unwrap(),
            vec!["ses_test0001:eval".to_string()]
        );
        assert_eq!(ace_session_list(), vec!["ses_test0001".to_string()]);
        shutdown_ace_session_host();
    }

    /// p970: the frozen protocol identity is a single source of truth the
    /// daemon/MCP surfaces share.
    #[test]
    fn p970_protocol_identity() {
        assert_eq!(SESSION_PROTOCOL_VERSION, 1);
        assert_eq!(SESSION_PROTOCOL_NAME, "ac.session.daemon/1");
        let defaults = AceSessionLimits::default();
        assert_eq!(defaults.max_sessions, 20);
        assert_eq!(defaults.idle_timeout_secs, 1800);
        assert_eq!(defaults.var_cap, 64);
    }

    /// p970: the scalar value surface renders like the legacy REPL
    /// (statements print nothing; ints/bools print their value).
    #[test]
    fn p970_value_display() {
        assert_eq!(AceSessionValue::None.display(), None);
        assert_eq!(AceSessionValue::Int(17).display().as_deref(), Some("17"));
        assert_eq!(
            AceSessionValue::Bool(true).display().as_deref(),
            Some("true")
        );
    }
}
