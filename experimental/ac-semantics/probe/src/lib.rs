//! PLAN-796 persistence probe (783 ManagedStore consumer).
//!
//! The first persistent semantic probe runs against the unified 783
//! store in a MANAGED ISOLATED NAMESPACE: layer `S796` + an inner
//! versioned sema page schema + producer/schema admission on every
//! read. The 783 generic block/graph API is used as a CONTAINER only —
//! no HIR chunk kind impersonates a semantic value, no Future-layer
//! registration is edited to claim the old producer supports A06, and
//! ZERO lines of 783/787 implementation change.

pub mod budget;
pub mod store;
pub mod turnaround;

pub use budget::{
    publish_streamed, BudgetRejected, HotPages, SessionMeter, StreamError, HOT_BUDGET,
    MANAGED_BUDGET, SCRATCH_BUDGET,
};
pub use turnaround::{
    machine_info, stats_of, write_report, Harness, Sample, ScenarioStats, HARD_GATE,
    REFERENCE_GENERATOR, SAMPLES,
};
pub use store::{
    lookup_only, open_store, publish_infer_result, publish_module_index, publish_record_meta,
    query, run_inference, FetchOutcome, ProbeError, QueryFlow, SEMA_CARRIER, SEMA_LAYER,
};
