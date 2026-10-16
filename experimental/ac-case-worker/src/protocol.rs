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
/// Hard bound on ONE stdin request line (the largest legal request is the
/// bounded source payload; beyond this the line is rejected, not buffered).
pub const SESSION_MAX_LINE_BYTES: usize = MAX_IPC_METADATA_BYTES * 2;

/// Session envelope read from stdin, one JSON line each.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Envelope {
    #[serde(rename = "begin")]
    Begin { schema: String },
    #[serde(rename = "request")]
    Request { request: WorkerRequest },
    #[serde(rename = "end")]
    End,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum InputSpec {
    #[serde(rename = "source")]
    Source { source_b64: String },
    #[serde(rename = "hir")]
    Hir { hir_b64: String },
    /// PLAN-788 R1 (F01): a product consumer's already-built exe — the
    /// worker runs it under the SAME contained, budgeted, capped executor
    /// (never a bare parent-side Command). ac/run only, no store.
    #[serde(rename = "product")]
    Product { exe_path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReqBudget {
    pub build_ms: u64,
    pub run_ms: u64,
    pub total_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// PLAN-788 T-05 cache participation: unified-store root the worker
    /// opens (reuse semantics per `cache`). None = no store, direct path.
    pub store_root: Option<String>,
    /// "fresh" | "reuse" | "read-only" (defaults to reuse with a store).
    pub cache: Option<String>,
    /// Store project namespace (roots/GC identity; cross-project runs use
    /// distinct projects over shared content).
    pub project: Option<String>,
    /// Ask the worker to also produce the canonical HIR bytes (the parent
    /// declares a hir.atom golden); observation-only, never published.
    pub include_hir: Option<bool>,
    /// PLAN-788 T-06: syntax grammar for parse/edit requests
    /// ("auto-core" | "interface-mini-v1").
    pub grammar: Option<String>,
    /// PLAN-788 T-06: ordered single-edit STEPS for edit requests; each
    /// step's coordinates are relative to the CURRENT revision at that
    /// step (sequential application, one edit per revision — the
    /// incremental engine's supported shape).
    pub edits: Option<Vec<EditStep>>,
}

/// One bounded syntax edit step (base = the snapshot the step applies to).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct EditStep {
    pub start: u64,
    pub end: u64,
    pub insert_b64: String,
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
        if !matches!(
            self.kind.as_str(),
            "check" | "build" | "run" | "parse" | "edit"
        ) {
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
        if let Some(g) = &self.grammar {
            if !matches!(g.as_str(), "auto-core" | "interface-mini-v1") {
                return Err(WorkerError::admission(
                    "request.grammar",
                    format!(
                        "unknown grammar {:?} (supported: auto-core, interface-mini-v1)",
                        g
                    ),
                ));
            }
        }
        if self.grammar.is_some() != matches!(self.kind.as_str(), "parse" | "edit") {
            return Err(WorkerError::admission(
                "request.grammar",
                "grammar is required for parse/edit and forbidden otherwise",
            ));
        }
        if let Some(edits) = &self.edits {
            if self.kind != "edit" {
                return Err(WorkerError::admission(
                    "request.edits",
                    "edits are only valid on edit requests",
                ));
            }
            if edits.is_empty() || edits.len() > 64 {
                return Err(WorkerError::admission(
                    "request.edits",
                    format!("edit sequence must be 1..=64 steps, got {}", edits.len()),
                ));
            }
            for (i, e) in edits.iter().enumerate() {
                let n = b64_len(&e.insert_b64);
                if n > 64 * 1024 {
                    return Err(WorkerError::admission(
                        "request.edits",
                        format!("edit[{}] insert {} bytes exceeds the 64KiB step cap", i, n),
                    ));
                }
            }
        } else if self.kind == "edit" {
            return Err(WorkerError::admission(
                "request.edits",
                "edit requests require a non-empty edit sequence",
            ));
        }
        if let Some(c) = &self.cache {
            if !matches!(c.as_str(), "fresh" | "reuse" | "read-only") {
                return Err(WorkerError::admission(
                    "request.cache",
                    format!(
                        "unknown cache mode {:?} (supported: fresh, reuse, read-only)",
                        c
                    ),
                ));
            }
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
        // R1 (F04): the FULL ancestor chain is audited — a reparse point
        // anywhere above the scratch is a controlled refusal.
        let mut cur = path.to_path_buf();
        while cur.pop() {
            if let Ok(md) = std::fs::symlink_metadata(&cur) {
                if md.file_type().is_symlink() {
                    return Err(WorkerError::admission(
                        "request.scratch",
                        format!("scratch ancestor {:?} is a reparse point", cur.display()),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// One typed diagnostic as observed at a pipeline boundary. `stage` uses
/// the PLAN-788 v2 stage set (contract-decision §4).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct ToolchainOut {
    pub lld_path: String,
    pub lld_sha256: String,
    pub sdk_um: String,
    pub ac_core_version: String,
    pub worker_version: String,
    /// R1 (F07): the worker executable's content hash — it statically
    /// embeds the compiler and its resolved dependencies, so this single
    /// axis IS the compiler/worker recipe identity the parent binds into
    /// native records and the stale checker re-verifies.
    #[serde(default)]
    pub worker_exe_sha256: String,
}

/// Cache observation for the report record (T-05): mode, driver hit/miss
/// lines, and the REAL stage counters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct CacheOut {
    pub mode: String,
    pub hits: Vec<(String, String)>,
    pub counters: Vec<(String, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct OkOut {
    /// Empty on success; the FIRST diagnostic's stage/code is the language
    /// observation the parent compares against its error-completion oracle.
    pub diagnostics: Vec<DiagOut>,
    pub hir_b64: Option<String>,
    pub artifacts: Vec<ArtifactOut>,
    pub run: Option<RunOut>,
    pub trace: Vec<SpanOut>,
    pub toolchain: ToolchainOut,
    #[serde(default)]
    pub cache: Option<CacheOut>,
    /// Metrics this build cannot capture (gaps — never 0-filled).
    #[serde(default)]
    pub metric_gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

fn b64_len(s: &str) -> usize {
    s.len()
        .saturating_sub(s.chars().filter(|c| *c == '=').count())
        * 3
        / 4
}
