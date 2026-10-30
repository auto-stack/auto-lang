//! PLAN-892 shared A2X core (AX-A): the target-neutral semantic view over
//! ALREADY-VERIFIED core/scalar HIR.
//!
//! Role per Design51 §3 / plan §2: the shared core consumes checked facts
//! (`auto_ac_prototype::verify::CheckedModule`) and exposes a read-only,
//! target-neutral view — declared functions with signatures, per-body
//! structure, source-order evaluation plans, short-circuit plans and
//! authorized intrinsic (trace) bindings. It never depends on
//! `auto-mir-lower`, the AV executor, Cranelift, the old AST/Database
//! frontend or the store; targets (Rust first) consume the view only.
//!
//! Admission is explicit: only the `core`/`scalar` HIR domains and only
//! their legal expr/stmt surface pass through. Anything wider is a NAMED
//! rejection (`A2xReject`), never a silent passthrough the verifier happens
//! to allow today.

pub mod prepare;
pub mod reject;
pub mod semantic_view;
pub mod sha256;

pub use prepare::{plan_function, FunctionPlan, MAX_STRUCTURE_DEPTH};
pub use reject::A2xReject;
pub use semantic_view::{ExternalFn, FnFacts, ParamFacts, SemanticView, Ty};
