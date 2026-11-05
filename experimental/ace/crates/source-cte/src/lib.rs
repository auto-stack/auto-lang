//! PLAN-936 SQ-C — the ordinary-source compile-time execution service.
//!
//! This crate is the acyclic extraction the plan's §2/§5.1 demand: ONE
//! service that takes a real compile-time demand against an ordinary
//! source program (a checked callee plus literal scalar arguments),
//! executes it on the REAL AV through a minimal checked call entry,
//! freezes the successful result as an executed envelope, and answers
//! later identical demands from the frozen page without any AV.
//!
//! What makes this the "source lane" and not a copy of the 817 probe:
//!
//! * **No deck, no source-text synthesis** — the entry is an AST-level
//!   wrapper appended to the caller's already-parsed program, checked by
//!   the REAL core resolver/verifier. Nothing re-parses user source, and
//!   `wrapper_source`-style text baking does not exist here.
//! * **No AV-frontend / native dependency** — the execution tail is the
//!   shared `lower_checked → verify_mir → ABC roundtrip → verify_abc →
//!   av-core execute` chain (pure `ace-av-core`), never `ace-av`'s
//!   incremental session and never `ace-native`. The old probe's
//!   snapshot lane keeps living beside its own frontend and delegates
//!   here (T-03); it never drags its frontend types into this crate.
//! * **New identity, new namespace** — the executed envelope is
//!   `plan936/source-cte` / `ac936/ct-exec/v1` under store layer `S936`.
//!   It reuses NOTHING from the 817 `ac817/a05-cte-exec/v1` execution
//!   identity: no `source_fp`, no source-level fingerprints, and a strict
//!   value fold (a bool expectation accepts only a `Bool` AV value — the
//!   old nonzero-I32→true projection is gone).
//! * **Execution-key / use-binding separation** — what makes executions
//!   reusable (callee logical id, its signature/body versions, the
//!   transitive callee-closure versions, the canonical arguments, the
//!   engine axis) is the [`identity::ExecKey`]; what makes one OWNER's
//!   consumption of the value admissible (module, owner, expected type)
//!   is the per-request [`identity::UseBinding`] and is never written
//!   into the cached page.
//!
//! T-02 bounded surface (this first landing): single-module scalar
//! programs, direct free-function callees with literal i32/bool
//! arguments, the callee's transitive call closure in the key, executed
//! freeze/hit/restore over the unified managed store, and the real-AV
//! counters (`ct_execute` counts only attempts that actually reached the
//! AV — trap/resource/cancel included; a hit never does). The HashExpr
//! grammar/surface (T-04), the query DAG scheduler (T-05), instance
//! composition (T-08) and the restore/R-window machinery (T-09) build on
//! this seam.

pub mod envelope;
pub mod executor;
pub mod identity;
pub mod schedule;
pub mod service;
pub mod store;

#[cfg(test)]
mod tests;

/// The D-798 per-call AV wall cap (2s) — an upper bound the request
/// deadline narrows per call; never raised (F-817-R6-01 discipline).
pub const MAX_CALL_WALL: std::time::Duration = ace_av_core::contract::MAX_AV_WALL;

pub use envelope::CtExecutedV1;
pub use executor::{
    checked_module_of, execute_call, run_entry_on_av, AvExecFacts, ColdAvExecutor, CtExecError,
    CtExecOutcome, CtExecutor, ExecCtl,
};
pub use identity::{
    transitive_closure_versions, CtEngine, CtValue, ExecKey, ExpectTy, UseBinding, CT_ALGO,
    CT_LANE, CT_PRODUCER, CT_PROFILE, CT_SCHEMA,
};
pub use schedule::{serve_program, MetaAxis, PlanError, RootBudget, ServedRoot};
pub use service::{CtAnswer, CtCounters, CtRequest, CtService};
pub use store::{CtEnvelopeSink, MemoryEnvelopeSink, CT_LAYER, CT_ROOT_KIND};
