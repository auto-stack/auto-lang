//! Parent-side typed mirror of the `acworker/1` response face (R2 F06).
//!
//! auto-lang cannot link the worker crate (separate workspace, and the
//! zero-Cranelift red line forbids it); this module is a DELIBERATE serde
//! mirror of `experimental/ac-case-worker/src/protocol.rs` used for typed
//! admission: unknown fields, wrong field types and malformed payloads are
//! refused before any judging. Drift between the two sides is caught by
//! the integration suites (a response the worker legally emits must always
//! deserialize here; a shape only one side accepts fails the runner tests).
//!
//! R3 (F08): most mirror fields exist for serde ADMISSION and are never
//! read afterwards — that is the point of a mirror. Per-field dead_code is
//! expected, so every mirror struct below carries a scoped allow; no
//! global suppression.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // admission mirror: fields are validated, not read
pub struct WorkerResponse {
    pub schema: String,
    pub nonce: String,
    pub request_hash: String,
    pub ok: Option<OkOut>,
    pub error: Option<WorkerError>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // admission mirror: fields are validated, not read
pub struct WorkerError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // admission mirror: fields are validated, not read
pub struct OkOut {
    #[serde(default)]
    pub diagnostics: Vec<DiagOut>,
    #[serde(default)]
    pub hir_b64: Option<String>,
    #[serde(default)]
    pub artifacts: Vec<ArtifactOut>,
    #[serde(default)]
    pub run: Option<RunOut>,
    #[serde(default)]
    pub trace: Vec<SpanOut>,
    pub toolchain: ToolchainOut,
    #[serde(default)]
    pub cache: Option<CacheOut>,
    #[serde(default)]
    pub metric_gaps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // admission mirror: fields are validated, not read
pub struct DiagOut {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub span_start: usize,
    pub span_end: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // admission mirror: fields are validated, not read
pub struct ArtifactOut {
    pub kind: String,
    pub logical: String,
    pub path: String,
    pub bytes: usize,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // admission mirror: fields are validated, not read
pub struct RunOut {
    pub exit_code: Option<i32>,
    pub stdout_b64: String,
    pub stderr_b64: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // admission mirror: fields are validated, not read
pub struct SpanOut {
    pub stage: String,
    pub wall_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // admission mirror: fields are validated, not read
pub struct ToolchainOut {
    #[serde(default)]
    pub lld_path: String,
    #[serde(default)]
    pub lld_sha256: String,
    #[serde(default)]
    /// Display-only SDK um/x64 dir path.
    pub sdk_um: String,
    #[serde(default)]
    /// R3 (F05): CONTENT identity of the linked SDK (sha256 of the
    /// kernel32.lib consumed by the link); the path axis is never identity.
    pub sdk_um_sha256: String,
    #[serde(default)]
    pub ac_core_version: String,
    #[serde(default)]
    pub worker_version: String,
    #[serde(default)]
    pub worker_exe_sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)] // admission mirror: fields are validated, not read
pub struct CacheOut {
    pub mode: String,
    #[serde(default)]
    pub hits: Vec<(String, String)>,
    #[serde(default)]
    pub counters: Vec<(String, u32)>,
}
