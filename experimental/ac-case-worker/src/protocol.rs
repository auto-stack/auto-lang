//! Bounded, versioned worker IPC protocol (PLAN-788 T-03; frozen in
//! docs/plans/attachments/788/contract-decision.md §5–6).
//!
//! One JSON value per line: requests flow on stdin, exactly one response
//! line per request flows back on stdout. Byte payloads travel as base64 so
//! invalid UTF-8 survives the wire unchanged. The worker never sees or
//! evaluates `expected` payloads — it returns observations; every PASS
//! judgment stays in the parent runner.

use serde::{Deserialize, Serialize};

pub const WORKER_SCHEMA: &str = "acworker/1";

// Frozen hard caps (contract-decision §5). Requests above these are
// rejected at admission; the worker never widens them at runtime.
pub const MAX_STDOUT_BYTES: usize = 1 << 20; // 1 MiB per stream
pub const MAX_STDERR_BYTES: usize = 1 << 20;
pub const MAX_IPC_METADATA_BYTES: usize = 4 << 20;
pub const MAX_SINGLE_ARTIFACT_BYTES: usize = 64 << 20;
pub const MAX_CHECK_ARTIFACT_TOTAL_BYTES: usize = 128 << 20;
pub const MAX_BUILD_MS: u64 = 60_000;
pub const MAX_RUN_MS: u64 = 5_000;
pub const MAX_TOTAL_MS: u64 = 70_000;
pub const SESSION_MAX_REQUESTS: u32 = 256;
pub const SESSION_MAX_LIFETIME_MS: u64 = 180_000;

/// Session envelope read from stdin, one JSON line each.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum Envelope {
    #[serde(rename = "begin")]
    Begin { schema: String },
    #[serde(rename = "request")]
    Request { request: WorkerRequest },
    #[serde(rename = "end")]
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum InputSpec {
    #[serde(rename = "source")]
    Source { source_b64: String },
    #[serde(rename = "hir")]
    Hir { hir_b64: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReqBudget {
    pub build_ms: u64,
    pub run_ms: u64,
    pub total_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRequest {
    pub schema: String,
    /// "check" | "build" | "run".
    pub kind: String,
    /// Parent-generated unguessable nonce binding request and response.
    pub nonce: String,
    /// Parent-computed request identity hash; echoed verbatim (the parent
    /// compares, binding responses to the exact request it sent — a stale
    /// or mismatched response fails there).
    pub request_hash: String,
    pub input: InputSpec,
    /// "core" | "scalar".
    pub profile: String,
    pub entry: String,
    pub budget: ReqBudget,
    /// Absolute path of the parent-owned scratch dir for this request.
    pub scratch: String,
}

impl WorkerRequest {
    /// Admission validation (never a silent default): schema/kind/profile
    /// exact sets, budget bounds, scratch sanity. `Ok` budgets are the
    /// values the executor clamps nothing further on.
    pub fn validate(&self) -> Result<(), WorkerError> {
        if self.schema != WORKER_SCHEMA {
            return Err(WorkerError::admission(
                "request.schema",
                format!(
                    "unsupported schema {:?} (worker implements {})",
                    self.schema, WORKER_SCHEMA
                ),
            ));
        }
        if !matches!(self.kind.as_str(), "check" | "build" | "run") {
            return Err(WorkerError::admission(
                "request.kind",
                format!(
                    "unknown kind {:?} (supported: check, build, run)",
                    self.kind
                ),
            ));
        }
        if !matches!(self.profile.as_str(), "core" | "scalar") {
            return Err(WorkerError::admission(
                "request.profile",
                format!(
                    "profile {:?} not imported in this batch (supported: core, scalar)",
                    self.profile
                ),
            ));
        }
        if self.entry.is_empty()
            || self.entry.len() > 64
            || !self
                .entry
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(WorkerError::admission(
                "request.entry",
                "entry must be a function-name identifier ([A-Za-z0-9_], non-empty, <=64 chars)",
            ));
        }
        if self.nonce.is_empty() || self.request_hash.is_empty() {
            return Err(WorkerError::admission(
                "request.nonce",
                "nonce and request_hash are required on every request",
            ));
        }
        if self.budget.build_ms == 0
            || self.budget.build_ms > MAX_BUILD_MS
            || self.budget.run_ms == 0
            || self.budget.run_ms > MAX_RUN_MS
            || self.budget.total_ms == 0
            || self.budget.total_ms > MAX_TOTAL_MS
        {
            return Err(WorkerError::admission(
                "request.budget",
                format!(
                    "budget out of bounds (hard caps: build {}ms, run {}ms, total {}ms; got {}/{}/{}ms)",
                    MAX_BUILD_MS, MAX_RUN_MS, MAX_TOTAL_MS,
                    self.budget.build_ms, self.budget.run_ms, self.budget.total_ms
                ),
            ));
        }
        let path = std::path::Path::new(&self.scratch);
        if !path.is_absolute() {
            return Err(WorkerError::admission(
                "request.scratch",
                "scratch must be an absolute parent-owned path",
            ));
        }
        let meta = std::fs::symlink_metadata(path).map_err(|e| {
            WorkerError::admission(
                "request.scratch",
                format!("scratch {:?} unusable: {}", self.scratch, e),
            )
        })?;
        if meta.file_type().is_symlink() {
            return Err(WorkerError::admission(
                "request.scratch",
                "scratch must not be a reparse point",
            ));
        }
        if !meta.is_dir() {
            return Err(WorkerError::admission(
                "request.scratch",
                "scratch must be an existing directory",
            ));
        }
        Ok(())
    }
}

/// One typed diagnostic as observed at a pipeline boundary. `stage` uses
/// the PLAN-788 v2 stage set (contract-decision §4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagOut {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub span_start: usize,
    pub span_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactOut {
    /// "hir" | "obj" | "exe".
    pub kind: String,
    /// Logical name inside the check ("hir.atom", "app.obj", "app.exe").
    pub logical: String,
    /// Absolute path in the request scratch.
    pub path: String,
    pub bytes: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunOut {
    pub exit_code: Option<i32>,
    pub stdout_b64: String,
    pub stderr_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpanOut {
    pub stage: String,
    pub wall_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolchainOut {
    pub lld_path: String,
    pub lld_sha256: String,
    pub sdk_um: String,
    pub ac_core_version: String,
    pub worker_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct OkOut {
    /// Empty on success; the FIRST diagnostic's stage/code is the language
    /// observation the parent compares against its error-completion oracle.
    pub diagnostics: Vec<DiagOut>,
    pub hir_b64: Option<String>,
    pub artifacts: Vec<ArtifactOut>,
    pub run: Option<RunOut>,
    pub trace: Vec<SpanOut>,
    pub toolchain: ToolchainOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerError {
    /// Stable machine code ("request.schema", "budget.run", "output.cap",
    /// "tool.missing", "request.internal", ...).
    pub code: String,
    pub message: String,
}

impl WorkerError {
    pub fn admission(code: &str, message: impl Into<String>) -> Self {
        WorkerError {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

/// One response line (success or infrastructure failure; language
/// diagnostics are SUCCESS-shaped — they are observations, not errors).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerResponse {
    pub schema: String,
    pub nonce: String,
    pub request_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<OkOut>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<WorkerError>,
}

impl WorkerResponse {
    pub fn ok(req: &WorkerRequest, ok: OkOut) -> Self {
        WorkerResponse {
            schema: WORKER_SCHEMA.to_string(),
            nonce: req.nonce.clone(),
            request_hash: req.request_hash.clone(),
            ok: Some(ok),
            error: None,
        }
    }

    pub fn err(req_like: (&str, &str), error: WorkerError) -> Self {
        WorkerResponse {
            schema: WORKER_SCHEMA.to_string(),
            nonce: req_like.0.to_string(),
            request_hash: req_like.1.to_string(),
            ok: None,
            error: Some(error),
        }
    }
}
