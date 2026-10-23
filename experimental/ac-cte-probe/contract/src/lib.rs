//! PLAN-817 CTE probe contract crate (schema-frozen, dependency-free
//! besides serde). Everything here is versioned wire identity: the
//! recipe request IR (`ac817/cte-recipe-ir/v1`), the frozen execution
//! envelope (`ac817/cte-executed/v1`) and the typed admission codes.
//!
//! The frozen DESIGN behind these schemas lives in `../contract-decision.md`
//! (T-01 r1.1). Changing any value here requires a recorded revision
//! there.

pub mod admission;
pub mod frozen;
pub mod request;

pub use admission::{CteAdmissionError, CteCode};
pub use frozen::{
    CteDepsGranularity, CteDepsManifestV1, CteEngineId, CteExecutedV1, CteRequestIdentityV1,
    CTE_EXECUTED_SCHEMA, CTE_FROZEN_LAYER, CTE_PROBE_VERSION, CTE_PRODUCER, CTE_QUERY_ALGO_ID,
};
pub use request::{
    CteBindingV1, CteExecMode, CteExpectType, CteRequestV1, CteScalar, CteSourceProfile, CteValue,
    QueryNodeKind, QueryNodeV1, CTE_META_CAP, CTE_RECIPE, CTE_REQUEST_SCHEMA, CTE_SEMA_PACKAGE,
    MAX_QUERY_NODES_PER_REQUEST, MAX_QUERY_REF_DEPTH, MAX_REQUEST_INPUT_BYTES,
};
