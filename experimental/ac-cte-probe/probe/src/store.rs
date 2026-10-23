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

    /// The underlying 783 store (lifecycle tests drive GC/roots/leases).
    pub fn raw(&self) -> &ManagedStore {
        &self.store
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
                    "{}/{}/{}/{}-{}-{}/{}",
                    id.engine.av_profile,
                    id.engine.mir_schema,
                    id.engine.abc_schema,
                    id.engine.mir_semantics,
                    id.engine.abc_semantics,
                    id.engine.av_semantics,
                    id.engine.exec
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

/// Look up one envelope (test/seed convenience — composes the staged
/// API below). `None` = clean miss. A found row that fails identity
/// validation (wrong schema/producer/engine/request identity) is a
/// CONTROLLED REJECTION — never a miss, never a rebuild-in-place.
pub fn lookup_frozen(
    st: &CteStore,
    identity: &CteRequestIdentityV1,
) -> Result<Option<CteExecutedV1>, CteAdmissionError> {
    let Some(payload) = load_frozen_payload(st, identity)? else {
        return Ok(None);
    };
    decode_envelope(&payload, identity).map(Some)
}

/// Stage 1 of the reserved restore (F-817-R6-02): load the payload
/// bytes. The caller must have reserved a covering bound BEFORE this
/// call — the `Vec` allocation happens here. `None` = clean miss.
pub fn load_frozen_payload(
    st: &CteStore,
    identity: &CteRequestIdentityV1,
) -> Result<Option<Vec<u8>>, CteAdmissionError> {
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
    Ok(Some(loaded.payload))
}

/// Stage 2 of the reserved restore (F-817-R6-02): decode + full-field
/// admission. The caller must have reserved a covering Hot bound BEFORE
/// this call — the decoded envelope's allocations happen here. The
/// serialized payload length is a sound upper bound for the decoded
/// structure (JSON text ≥ decoded content).
pub fn decode_envelope(
    payload: &[u8],
    identity: &CteRequestIdentityV1,
) -> Result<CteExecutedV1, CteAdmissionError> {
    let env: CteExecutedV1 = serde_json::from_slice(payload).map_err(|e| {
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
    Ok(env)
}

/// Publish one envelope (test/seed convenience — no ledger; the
/// production request path reserves first, see driver::publish_env).
pub fn publish_frozen(st: &CteStore, env: &CteExecutedV1) -> Result<String, CteAdmissionError> {
    let payload = serialize_envelope(env)?;
    publish_frozen_payload(st, env, payload)
}

/// Serialize once — callers that must reserve ledger bytes first count
/// with [`count_envelope_bytes`] and reserve BEFORE calling this (the
/// allocation happens here).
pub fn serialize_envelope(env: &CteExecutedV1) -> Result<Vec<u8>, CteAdmissionError> {
    serde_json::to_vec(env).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("encode".into()),
            format!("envelope serialization failed: {e}"),
        )
    })
}

/// The reserved encode (F-817-R6-02): COUNT the bytes without
/// allocating, RESERVE the exact staging amount, then serialize — the
/// allocation provably happens only after the reservation (the budget
/// event log records Reserve(ok) BEFORE Alloc("envelope-encode")). A
/// rejected reservation returns Err with NO payload produced.
pub fn serialize_envelope_reserved(
    env: &CteExecutedV1,
    budget: &mut crate::observe::ManagedBudget,
) -> Result<Vec<u8>, CteAdmissionError> {
    let count = count_envelope_bytes(env)?;
    budget
        .reserve(crate::observe::Partition::Staging, count, "envelope encode")
        .map_err(|m| CteAdmissionError::new(CteCode::Resource("managed-staging".into()), m))?;
    let payload = serialize_envelope(env)?;
    budget.record_alloc("envelope-encode", payload.len() as u64);
    Ok(payload)
}

/// Count the serialized envelope bytes WITHOUT allocating (a counting
/// writer; serde's internal scratch is O(1)) — the reservation input
/// for the F-817-R6-02 reserve-before-encode order.
pub fn count_envelope_bytes(env: &CteExecutedV1) -> Result<u64, CteAdmissionError> {
    struct Count(u64);
    impl std::io::Write for Count {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0 += buf.len() as u64;
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut c = Count(0);
    serde_json::to_writer(&mut c, env).map_err(|e| {
        CteAdmissionError::new(
            CteCode::Infra("encode".into()),
            format!("envelope counting serialization failed: {e}"),
        )
    })?;
    Ok(c.0)
}

/// Publish a PRE-SERIALIZED envelope (the caller reserved the ledger
/// bytes before serializing). The payload moves in — the store path
/// adds NO second unreserved copy (F-817-R6-02).
pub fn publish_frozen_payload(
    st: &CteStore,
    env: &CteExecutedV1,
    payload: Vec<u8>,
) -> Result<String, CteAdmissionError> {
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
