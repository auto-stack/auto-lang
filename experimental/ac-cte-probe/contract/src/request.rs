//! The CTE probe request IR (`ac817/cte-recipe-ir/v1`) — the versioned
//! query/test interface. This is NOT Auto language syntax: consumers are
//! plain Auto source with designated local-binding anchors, and every
//! compile-time-evaluated position is described by the binding table.

use serde::{Deserialize, Serialize};

/// Schema identity of the request IR (frozen, contract-decision §1).
pub const CTE_REQUEST_SCHEMA: &str = "ac817/cte-recipe-ir/v1";
/// The official AutoDown probe recipe this IR rides (frozen, §1/§9).
pub const CTE_RECIPE: &str = "cte/scalar-query-v1";
/// The only capability the first profile may grant (Meta param queries).
pub const CTE_META_CAP: &str = "cte.meta.fields/v1";
/// Fixed sema package domain for probe identities (module comes per deck).
pub const CTE_SEMA_PACKAGE: &str = "cte817";

/// AV execution lane for `call` queries (T-11): `closure` (default) is
/// the whole-module ABC roundtrip; `snapshot` routes through the
/// PLAN-828 function-incremental session (per-function units, link,
/// ProgramSnapshot execution, incremental edits between calls).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum CteExecMode {
    #[default]
    Closure,
    Snapshot,
}

impl CteExecMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "closure" => Some(CteExecMode::Closure),
            "snapshot" => Some(CteExecMode::Snapshot),
            _ => None,
        }
    }
    pub fn axis(self) -> &'static str {
        match self {
            CteExecMode::Closure => "closure",
            CteExecMode::Snapshot => "snapshot",
        }
    }
}

/// Projection profile of the embedded source. `core` (default) is the
/// i32/bool fn surface; `record` additionally allows `type` blocks so
/// Meta queries have a record domain to read (T-03). The AV execution
/// surface stays scalar either way — a record-profile deck may not carry
/// `call` queries (admission rejects the mix).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum CteSourceProfile {
    #[default]
    Core,
    Record,
}

impl CteSourceProfile {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "core" => Some(CteSourceProfile::Core),
            "record" => Some(CteSourceProfile::Record),
            _ => None,
        }
    }
}

/// Request-scoped budget ceilings (contract-decision §7; the AV/verify
/// engine caps themselves are the D-798 frozen constants and are never
/// restated here).
pub const MAX_REQUEST_INPUT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_QUERY_NODES_PER_REQUEST: usize = 128;
pub const MAX_QUERY_REF_DEPTH: usize = 128;

/// The only value shapes the first CTE profile carries.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CteValue {
    I32(i32),
    Bool(bool),
}

impl CteValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            CteValue::I32(_) => "i32",
            CteValue::Bool(_) => "bool",
        }
    }
    pub fn render(&self) -> String {
        match self {
            CteValue::I32(v) => format!("i32:{v}"),
            CteValue::Bool(v) => format!("bool:{v}"),
        }
    }
}

/// A literal query argument (i32 or bool).
pub type CteScalar = CteValue;

/// One query node. `QueryRef` may only reference an ALREADY-declared node
/// (declaration order = DAG topological order at the IR level; the
/// scheduler re-validates and owns cycle/trap classification).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum QueryNodeKind {
    Literal(CteScalar),
    /// Reference to a previously declared query node (by id).
    QueryRef(String),
    /// Real AV call: callee function name + argument nodes.
    Call {
        callee: String,
        args: Vec<QueryNodeKind>,
    },
    /// `FieldCount(record)` — needs `cte.meta.fields/v1`; counts as a
    /// meta_query, never as AV execution.
    FieldCount {
        record: String,
    },
    /// `HasField(record, name)` — needs `cte.meta.fields/v1`; a missing
    /// field of an EXISTING record is `false` plus a recorded negative
    /// dependency. A missing record is a semantic error.
    HasField {
        record: String,
        field: String,
    },
}

/// A named query node declaration (`query <id> = <kind>`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct QueryNodeV1 {
    pub id: String,
    pub kind: QueryNodeKind,
}

/// Expected scalar type spelling used by bindings.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CteExpectType {
    I32,
    Bool,
}

impl CteExpectType {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "i32" => Some(CteExpectType::I32),
            "bool" => Some(CteExpectType::Bool),
            _ => None,
        }
    }
    pub fn matches_value(&self, v: &CteValue) -> bool {
        matches!(
            (self, v),
            (CteExpectType::I32, CteValue::I32(_)) | (CteExpectType::Bool, CteValue::Bool(_))
        )
    }
}

/// One consumer binding: the CT value of `source_query` is backfilled at
/// the local binding `binding_name` of function `owner_fn`, whose source
/// placeholder literal must equal `placeholder` (drift detection), and
/// the derived/inferred type must equal `expect`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CteBindingV1 {
    pub ct_ref: String,
    pub owner_fn: String,
    pub binding_name: String,
    pub placeholder: CteScalar,
    pub expect: CteExpectType,
    pub source_query: String,
}

/// A whole CTE probe request (one deck document).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CteRequestV1 {
    pub schema: &'static str,
    pub recipe: &'static str,
    /// ac-core module path / 796 module identity for the embedded source.
    pub module: String,
    /// Plain Auto source (Core profile surface) containing the callee(s)
    /// and the consumer function(s).
    pub source: String,
    /// Query node declarations in topological order.
    pub queries: Vec<QueryNodeV1>,
    /// Consumer binding table.
    pub bindings: Vec<CteBindingV1>,
    /// Consumer entry function (native observable).
    pub entry: String,
    /// Granted capabilities (subset of {cte.meta.fields/v1}).
    pub caps: Vec<String>,
    /// Whether the native observable runs for query steps (default on;
    /// the turnaround tier turns it off — native is reported separately).
    pub native: bool,
    /// Projection profile of the embedded source (default core).
    pub profile: CteSourceProfile,
    /// AV execution lane (default closure).
    pub exec: CteExecMode,
    /// Record declarations for Meta queries (`meta << ... >>` block;
    /// record-profile source text). Merged into the projection's record
    /// domain: FieldCount/HasField read HERE, their results feed as REAL
    /// AV call arguments — the contracted §5.1 path (F-817-R1-05).
    pub meta_source: Option<String>,
}

impl CteRequestV1 {
    pub fn query(&self, id: &str) -> Option<&QueryNodeV1> {
        self.queries.iter().find(|q| q.id == id)
    }
}
