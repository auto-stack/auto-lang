//! The frozen execution envelope (`ac817/cte-executed/v1`) and its
//! dependency manifest. The envelope is the ONLY persistent proof that a
//! real AV execution produced a value for one exact request identity;
//! restoring it is data validation + freshness, never deserialized trust
//! (contract-decision §6).

use crate::request::{CteExpectType, CteValue};
use serde::{Deserialize, Serialize};

/// Schema identity of the frozen envelope (frozen, §1).
pub const CTE_EXECUTED_SCHEMA: &str = "ac817/cte-executed/v1";
/// 817's own 783 LayerSpec id — the A05-limited implementation. The 783
/// registry's own A05 registration stays `Future` (SD-04).
pub const CTE_FROZEN_LAYER: &str = "ac817/a05-cte-exec/v1";
/// Producer identity bound to real AV success results.
pub const CTE_PRODUCER: &str = "plan817/cte-probe";
/// Probe/worker version string (worker identity axis).
pub const CTE_PROBE_VERSION: &str = "auto-cte-probe/0.1.0";
/// Query algorithm identity (part of the request key).
pub const CTE_QUERY_ALGO_ID: &str = "ac817-cte-query/1";

/// Engine identity triple actually used to produce the value (D-798
/// frozen versions, re-read at load and compared on restore).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CteEngineId {
    pub av_profile: String,
    pub mir_schema: String,
    pub abc_schema: String,
    pub mir_semantics: u32,
    pub abc_semantics: u32,
    pub av_semantics: u32,
    /// Execution lane (T-11/SD-06): "closure" = the whole-module ABC
    /// roundtrip; "snapshot" = the PLAN-828 incremental session (per-
    /// function units + ProgramSnapshot). A lane change re-keys the
    /// cache (V-12 identity).
    pub exec: String,
}

/// Declared dependency granularity. The first profile is conservative:
/// every body of the callee's transitive call closure participates in the
/// fingerprint — no dynamic-read-set minimization is claimed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CteDepsGranularity {
    ClosureBodies,
}

/// The dependency manifest recorded with every frozen value.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CteDepsManifestV1 {
    pub granularity: CteDepsGranularity,
    /// callee fn name + ordered closure (sorted, transitively closed).
    pub callee: String,
    pub closure: Vec<String>,
    /// (fn name, canonical signature fingerprint, canonical body
    /// fingerprint) for every closure member.
    pub fn_fingerprints: Vec<(String, String, String)>,
    /// Meta read sets (empty unless meta nodes executed): sorted record
    /// names whose FieldCount/HasField participated.
    pub meta_records: Vec<String>,
    /// HasField probes: (record, field) pairs actually consulted — the
    /// negative-dependency set (a probed-absent field APPEARING must
    /// invalidate).
    pub meta_probes: Vec<(String, String)>,
    /// Derived meta facts rendered canonically ("Pair#2", "Pair.zzz=false")
    /// — freshness re-derives them against the CURRENT input; any field
    /// add/remove flips a fact and invalidates the frozen value.
    pub meta_facts: Vec<String>,
    /// Negative-dependency candidate-set fingerprints (fn domain, record
    /// domain) — a candidate-set change invalidates negative deps.
    pub fn_candidates_fp: String,
    pub record_candidates_fp: String,
    /// Granted caps at execution time.
    pub caps: Vec<String>,
}

/// The frozen envelope persisted through the 783 generic carrier.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CteExecutedV1 {
    pub schema: String,
    pub recipe: String,
    /// Full request identity (NOT just a hash — restore compares fields).
    pub request_identity: CteRequestIdentityV1,
    pub value: CteValue,
    pub declared_type: CteExpectType,
    /// Producer + engine identity (must match the current driver's).
    pub producer: String,
    pub engine: CteEngineId,
    /// Real execution evidence from the AV trace.
    pub exec_steps: u64,
    pub exec_wall_ms: u64,
    pub deps: CteDepsManifestV1,
    /// Origin: source fingerprint the value was derived from.
    pub source_fp: String,
}

/// Everything the cache key and restore admission compare. Same name in
/// two packages or a method-owner difference produce different
/// identities (§5.2).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CteRequestIdentityV1 {
    pub module: String,
    pub package: String,
    pub callee: String,
    /// Canonical argument rendering (i32/bool literals only).
    pub args_canonical: String,
    pub query_algo: String,
    pub caps: Vec<String>,
    pub engine: CteEngineId,
    pub manifest_fp: String,
}
