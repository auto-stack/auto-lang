//! Frozen-envelope persistence over the PLAN-783 `ManagedStore`
//! (contract-decision §6): 817's own `LayerSpec` (`ac817/a05-cte-exec/v1`,
//! coverage=Codec — the 783 registry itself is untouched), the generic
//! `ChunkKind::LayoutData` carrier, JSON envelope payload, restore =
//! full field comparison + freshness, never deserialized trust.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{LoadedArtifact, ManagedStore, PutRequest};
use auto_cte_contract::{
    CteAdmissionError, CteCode, CteEngineId, CteExecutedV1, CteRequestIdentityV1, CTE_FROZEN_LAYER,
    CTE_PRODUCER,
};

/// 817's own layer spec — the A05-limited implementation. The 783
/// registry's A05 row stays `Future` (SD-04): this spec lives beside it
/// as the plan's declared producer namespace, exactly like the 787/798
/// precedent layers.
pub const CTE_LAYER_SPEC: LayerSpec = LayerSpec {
    id: "A05",
    name: "macro-comptime (plan817 CTE executed values)",
    schema_id: "ac817/a05-cte-exec/v1",
    producer: CTE_PRODUCER,
    consumers: &["cte consumer driver", "cte probe cache"],
    key_inputs: &[
        "producer",
        "owner-module",
        "callee",
        "args",
        "query-algo",
        "caps",
        "engine",
        "manifest-fp",
    ],
    ref_kinds: &["provenance"],
    storage: "one frozen value envelope per request identity (LayoutData carrier)",
    residency: "cold — literal materialization needs no live AV heap",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

/// Store adapter: one root, one project string.
pub struct CteStore {
    store: ManagedStore,
    project: String,
}

/// The per-(store, request) cache verdict.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CacheVerdict {
    Miss,
    Hit,
}

impl CteStore {
    pub fn open(root: &std::path::Path, project: &str) -> Result<Self, String> {
        let store = ManagedStore::open(root).map_err(|e| format!("cte store open: {e}"))?;
        Ok(CteStore {
            store,
            project: project.to_string(),
        })
    }

    pub fn project(&self) -> &str {
        &self.project
    }
}

/// Build the store key for one request identity.
pub fn envelope_key(id: &CteRequestIdentityV1) -> QueryKey {
    QueryKey::new(
        &CTE_LAYER_SPEC,
        vec![
            ("producer".into(), CTE_PRODUCER.into()),
            ("owner".into(), id.module.clone()),
            ("callee".into(), id.callee.clone()),
            ("args".into(), id.args_canonical.clone()),
            ("algo".into(), id.query_algo.clone()),
            ("caps".into(), id.caps.join(",")),
            (
                "engine".into(),
                format!(
                    "{}/{}/{}/{}-{}-{}",
                    id.engine.av_profile,
                    id.engine.mir_schema,
                    id.engine.abc_schema,
                    id.engine.mir_semantics,
                    id.engine.abc_semantics,
                    id.engine.av_semantics
                ),
            ),
            ("manifest-fp".into(), id.manifest_fp.clone()),
        ],
    )
}

/// Canonical argument rendering for the identity.
pub fn args_canonical(args: &[auto_cte_contract::CteValue]) -> String {
    args.iter()
        .map(auto_cte_contract::CteValue::render)
        .collect::<Vec<_>>()
        .join(",")
}

/// Look up one envelope. `None` = clean miss. A found row that fails
/// identity validation (wrong schema/producer/engine/request identity)
/// is a CONTROLLED REJECTION — never a miss, never a rebuild-in-place.
pub fn lookup_frozen(
    st: &CteStore,
    identity: &CteRequestIdentityV1,
) -> Result<Option<CteExecutedV1>, CteAdmissionError> {
    let loaded: Option<LoadedArtifact> = st
        .store
        .get_artifact(CTE_FROZEN_LAYER, &envelope_key(identity))
        .map_err(|e| {
            CteAdmissionError::new(
                CteCode::Infra("store".into()),
                format!("frozen lookup failed: {e}"),
            )
        })?;
    let Some(loaded) = loaded else {
        return Ok(None);
    };
    if loaded.kind != ChunkKind::LayoutData {
        return Err(CteAdmissionError::new(
            CteCode::Infra("store".into()),
            format!(
                "frozen artifact {} carries kind {:?}, expected the LayoutData carrier",
                loaded.artifact_id, loaded.kind
            ),
        ));
    }
    let env: CteExecutedV1 = serde_json::from_slice(&loaded.payload).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("store".into()),
            format!("frozen envelope does not decode: {e}"),
        )
    })?;
    // Full-field restore admission (§6): wrong producer/schema/engine/
    // request identity are controlled rejections.
    if env.schema != auto_cte_contract::CTE_EXECUTED_SCHEMA {
        return Err(CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            format!(
                "frozen envelope schema `{}` is not {}",
                env.schema,
                auto_cte_contract::CTE_EXECUTED_SCHEMA
            ),
        ));
    }
    if env.producer != CTE_PRODUCER {
        return Err(CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            format!(
                "frozen envelope producer `{}` is not the real-execution producer `{CTE_PRODUCER}` — model_compute output cannot mint execution",
                env.producer
            ),
        ));
    }
    if env.request_identity != *identity {
        return Err(CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            "frozen envelope request identity does not match this request (wrong owner/key/address)",
        ));
    }
    if env.engine != identity.engine {
        return Err(CteAdmissionError::new(
            CteCode::Infra("restore".into()),
            "frozen envelope engine identity does not match the current engine",
        ));
    }
    Ok(Some(env))
}

/// Publish one envelope (idempotent for identical content; same-key
/// different-content is the store's Conflict, surfaced as-is).
pub fn publish_frozen(st: &CteStore, env: &CteExecutedV1) -> Result<String, CteAdmissionError> {
    let payload = serde_json::to_vec(env).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("encode".into()),
            format!("envelope serialization failed: {e}"),
        )
    })?;
    st.store
        .put_artifact(PutRequest {
            layer: CTE_FROZEN_LAYER,
            key: envelope_key(&env.request_identity),
            kind: ChunkKind::LayoutData,
            payload,
            edges: Vec::new(),
            project: Some(st.project.clone()),
        })
        .map_err(|e| {
            CteAdmissionError::new(
                CteCode::Infra("store".into()),
                format!("frozen publish failed: {e}"),
            )
        })
}

/// Freshness: the envelope's dependency manifest must still describe the
/// CURRENT input (helper bodies/signatures, candidate sets, caps).
pub fn frozen_is_fresh(
    env: &CteExecutedV1,
    input: &ac_sema_core::SemaInput,
    caps: &[String],
) -> bool {
    auto_cte_core::dependencies::manifest_fresh(&env.deps, input, caps)
}

/// The engine identity embedded in fresh envelopes.
pub fn expected_engine(current: &CteEngineId) -> CteEngineId {
    current.clone()
}
