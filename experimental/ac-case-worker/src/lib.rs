//! ac-case-worker — PLAN-788 AC native test-case worker.
//!
//! Owns the experimental compiler stack (ac-core frontend/verifier/lowerer,
//! ac-artifact-probe unified store) behind a bounded, versioned JSON IPC
//! surface so the auto-lang product graph stays zero-Cranelift. The worker
//! never trusts or evaluates `expected` payloads: it returns structured
//! observations (typed diagnostics, real artifact manifests, raw run bytes,
//! stage traces, cache counters) and every comparison stays in the parent
//! runner (auto-lang `test_cases::ac`).
//!
//! T-01 status: whole-path feasibility only (tests/feasibility_t01_.rs).
//! The IPC protocol, budgets and session lifecycle are frozen in
//! docs/plans/attachments/788/contract-decision.md and land with T-03.
