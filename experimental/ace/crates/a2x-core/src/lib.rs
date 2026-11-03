//! PLAN-892 shared A2X core (AX-A) + PLAN-902 widening (AX-B): the
//! target-neutral semantic view over ALREADY-VERIFIED HIR.
//!
//! Role per Design51 §3 / plan §2: the shared core consumes checked facts
//! (`ace_core::verify::CheckedModule`) and exposes a read-only,
//! target-neutral view — declared functions with signatures+modes,
//! per-body structure, source-order evaluation plans, short-circuit plans,
//! nominal record shapes, method ownership and the verifier-computed text
//! resource plan. It never depends on `auto-mir-lower`, the AV executor,
//! Cranelift, the old AST/Database frontend or the store; targets (Rust
//! first) consume the view only.
//!
//! Admission is explicit: the `core`/`scalar` (PLAN-892) and
//! `record`/`text`/`text-record` (PLAN-902) HIR domains and only their
//! legal expr/stmt surface pass through. Anything wider is a NAMED
//! rejection (`A2xReject`), never a silent passthrough the verifier happens
//! to allow today.

pub mod prepare;
pub mod reject;
pub mod semantic_view;
pub mod sha256;
/// PLAN-921: the shared emit-target contract (providers live in the
/// target crates; the store consumes [`target::TargetProvider`] only).
pub mod target;

pub use prepare::{
    canonical_signature, plan_function, semantic_fingerprint, FunctionPlan, MAX_STRUCTURE_DEPTH,
};
pub use reject::A2xReject;
pub use semantic_view::{
    ExternalFn, FieldFacts, FnFacts, FnKind, ParamFacts, RecordFacts, SemanticView, Ty,
    VariantArmFacts, VariantFacts,
};
pub use target::{Family, FileSet, FilesetRequest, FragmentText, TargetDescriptor, TargetProvider};
