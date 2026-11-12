//! PLAN-934 SQ-A — ordinary-source semantic query and dependency
//! invalidation (T-02 bounded feasibility layer).
//!
//! Position in the dependency graph (plan §2): this crate sits BETWEEN
//! `ace-core` (the public AST + binding/verification authority) and the
//! consumers (AV / native / A2X). It depends on the core public AST, the
//! REUSED PLAN-796 neutral solver (`ace-semantics`, feature `core-ast`)
//! and the semantic identity contract (`ace-semantics-contract`). It has
//! NO AV/native/target-emission dependency — the direction is `core →
//! (consumer) → source-query` at consumption sites, never
//! `core → source-query` or `source-query → av`.
//!
//! What this layer owns (and what it does NOT):
//!
//! * **Owns**: one logical owner key + span-free semantic versions per
//!   function (signature axis vs body axis), one IMMUTABLE fact page per
//!   body whose node types are indexed by BODY STRUCTURAL ID (not by
//!   name, absolute span or intern slot), the positive/negative read set
//!   that invalidates it, and the session that answers
//!   `hit` / `miss → solve` under the frozen budget.
//! * **Does not own**: name/scope/signature binding (core), the type
//!   solver algorithm (796 `ac796-bidir-worklist/1`, reused verbatim
//!   through the neutral input), HIR emission or verification (core),
//!   persistence pages/transactions (T-06 wires the unified ManagedStore)
//!   and any target backend.
//!
//! Finite surface of this bounded step: the Core/Scalar i32/bool
//! function surface (literals, locals with let/var + shadowing, direct
//! calls, the operator set, condition/loop/re-assignment). Constructs
//! outside it are a CONTROLLED rejection ([`ProduceError::Unsupported`]),
//! never a silent widening, an `Any` type or a re-run of the legacy
//! synthesis. Named-argument calls (E01) are still rejected by the
//! current 796 projection (T-04 extends it with the source-order +
//! parameter-binding pair); this crate surfaces that as explicit
//! `Unsupported` and never as a mistyped fact.

pub mod apply;
pub mod assembly;
pub mod catalog;
pub mod consumer;
pub mod facts;
pub mod frontend;
pub mod host;
pub mod ident;
pub mod instance_host;
pub mod instances;
pub mod keys;
pub mod produce;
pub mod resources;
pub mod restore;
pub mod session;
pub mod store;
pub mod walk;
pub mod work;

#[cfg(test)]
mod foreign_tests;
#[cfg(test)]
mod tests;

pub use apply::{
    apply_body_facts, apply_facts_to_resolved, apply_fn_sig, bind_with_facts, AppliedFn,
    ApplyError, ApplyReport, BindFactsError, BodyTypeSupply, FactsApplyError,
};

pub use catalog::SignatureCatalog;
pub use consumer::{
    compile_core_product, compile_scalar_product, compile_text_record_product, compile_via_facts,
    CompileLane, ConsumerError, FactCompiled, ProductCompile,
};
pub use facts::{
    BindingFact, FactsError, FnSigFacts, InferBodyFacts, NegativeDep, NodeFact, PortableDiag,
    PortableTy,
};
pub use host::{
    Candidate, CommitError, CurrentRecord, FlightError, MemorySink, PageSink, QueryHost, QueryView,
    TargetHandle,
};
pub use ident::{
    content_version, owner_logical, signature_version, sq_hash, SQ_ALGO, SQ_PRODUCER, SQ_PROFILE,
    SQ_SCHEMA,
};
pub use instance_host::{
    decode_instance_page, encode_instance_page, set_fingerprint, CurrentInstanceSet,
    InstanceCommitError, InstanceHost, InstancePage, InstancePageError, InstanceSetCandidate,
    InstanceSink, ManagedInstanceSink, MemoryInstanceSink, INSTANCE_LAYER, INSTANCE_ROOT_KIND,
};
pub use instances::{
    instantiate_program, instantiate_program_limited, InstanceArg, InstanceRecord,
    InstantiateOutcome, INSTANCES_ALGO, INSTANCES_KEY_DOMAIN, INSTANCES_LOGICAL_DOMAIN,
    INSTANCES_PRODUCER, INSTANCES_SCHEMA,
};
pub use keys::{QueryKey, QueryKind, TARGET_SEMANTICS};
pub use produce::{produce_fn_sig, produce_infer_body, ProduceError, ProduceLimits};
pub use restore::{restore_bind, restore_verify, RestoreError};
pub use session::{Counters, PageAudit, QueryError, QueryOutcome, SourceQuerySession};
pub use store::{decode_page, encode_page, store_key, ManagedStoreSink, PageReadError};
pub use walk::{walk_body, BodyWalk, LetDecl, NodeId, NodeInfo, WalkError};

// PLAN-950 metadata-preserving finite modules, shared by real consumers.
pub mod modules;
