//! PLAN-796 limited semantic core.
//!
//! Owns the neutral semantic input model, the limited bidirectional
//! i32/bool inference profile, queries with dependency manifests
//! (positive + negative), read-only reflection projections and the
//! CT/instance/runtime seam schemas. It has NO product-backend
//! dependency; the ac-core AST adapter is feature-gated (`core-ast`) so
//! the default build provably stays backend-free.
//!
//! Separation rules (plan §2, Design 42 §2):
//! * semantic FACTS are immutable, publishable results;
//! * inference WORK STATE (variables, union-find, worklist) is
//!   task-local and never persisted;
//! * runtime projection is a downstream policy, never an identity input.

pub mod budget;
#[cfg(feature = "core-ast")]
pub mod adapter;
pub mod input;
pub mod infer;
pub mod query;
pub mod reflect;

pub use budget::{Budget, BudgetError};
pub use input::{
    Ann, InputBlock, InputExpr, InputExprKind, InputFn, InputRecord, InputStmt, SemaInput, Span,
};
pub use infer::{InferOutcome, InferSession, InferSummary};
pub use query::{
    current_fingerprints, freshness_of, manifest_from, DependencyFingerprints, Freshness,
    QueryKind, SemaQueryKey,
};
pub use reflect::{
    list_candidates, list_fields, query_field, query_signature, CtAdmission, CtRequest, CtResult,
    InstanceKey, ModelCt, PortableValue, RuntimeProjectionPolicy,
};
