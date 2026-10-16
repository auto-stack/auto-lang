//! ac-case-worker — PLAN-788 AC native test-case worker.
//!
//! Owns the experimental compiler stack (ac-core frontend/verifier/lowerer,
//! and from T-05 the PLAN-783 unified store) behind a bounded, versioned
//! JSON-lines IPC surface so the auto-lang product graph stays
//! zero-Cranelift. The worker never trusts or evaluates `expected`
//! payloads: it returns structured observations (typed diagnostics, real
//! artifact manifests, raw run bytes, stage traces) and every comparison
//! stays in the parent runner (auto-lang `test_cases::ac`).
//!
//! Whole-path feasibility: tests/feasibility_t01_.rs (T-01). Protocol and
//! budgets: src/protocol.rs (frozen in
//! docs/plans/attachments/788/contract-decision.md). Cache/trace adapter:
//! T-05. The syntax adapter binds only after PLAN-787 merges (D-787).

pub mod exec;
pub mod protocol;
pub mod syntax;

pub use protocol::{
    ArtifactOut, CacheOut, DiagOut, Envelope, InputSpec, OkOut, ReqBudget, RunOut, SpanOut,
    ToolchainOut, WorkerError, WorkerRequest, WorkerResponse, WORKER_SCHEMA,
};
