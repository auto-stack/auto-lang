//! Object-layer cache adapter over the merged 783 managed store
//! (plan §5.4, T-05). Layouts already persist since T-01 (ABI-L0);
//! this module adds the PRODUCER-OBJECT layer (ABI-O1) for both
//! AutoABI-emitted objects and cl.exe-compiled C objects.
//!
//! Contract (frozen in contract-decision §8):
//! * keys cover every real input: declaration set (symbols + signature
//!   forms + body identities), semantic-ref snapshot, ABI profile,
//!   target, emitter/toolchain identity, and for C sources the source
//!   content hash + cl identity + frozen flags — a body/toolchain/flag
//!   change is a DIFFERENT key (real invalidation), never a stale hit;
//! * a hit never invokes the producer and never skips verification: the
//!   consumer still admits the descriptor, and every request still
//!   links and runs the PE and re-compares the oracle — PASS results
//!   are never cached;
//! * store internals (registry, GC) are the 783/787 code paths — this
//!   adapter only reads and writes through the public store API.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{LoadedArtifact, ManagedStore, PutRequest, StoreError};
use crate::decl::FnDecl;

/// Probe-owned layer: producer objects (AutoABI-emitted or C-compiled).
pub const LAYER_OBJECT_V01: LayerSpec = LayerSpec {
    id: "ABI-O1",
    name: "autoabi-producer-object",
    schema_id: "auto797/abi-o1-object/v1",
    producer: "auto-abi-probe emit / cl.exe (identity in key)",
    consumers: &["auto-abi-probe link pipeline"],
    key_inputs: &["decl-set", "sem-ref", "target", "abi-profile", "toolchain", "source-hash"],
    ref_kinds: &["provenance"],
    storage: "object bytes as content-addressed blobs",
    residency: "hot after first build; reclaimed by GC when unrooted",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::ProbeSeam,
};

/// Emitter identity for AutoABI objects (probe version + Cranelift
/// stack); changes to either are a different key.
pub fn emitter_identity() -> String {
    format!(
        "auto-abi-probe/{}+cranelift/{}",
        env!("CARGO_PKG_VERSION"),
        cranelift_codegen::VERSION
    )
}

/// Canonical spelling of a declaration set: symbols sorted, each with
/// its signature form AND body identity — an internal body change is a
/// different producer input even when the public signature is
/// unchanged.
pub fn canonical_decl_set(defs: &[FnDecl]) -> String {
    let mut parts: Vec<String> = defs
        .iter()
        .map(|d| {
            format!(
                "{}({})->{}[{:?}]",
                d.symbol,
                d.params
                    .iter()
                    .map(|p| format!("{p:?}"))
                    .collect::<Vec<_>>()
                    .join(","),
                format!("{:?}", d.ret),
                d.body
            )
        })
        .collect();
    parts.sort();
    parts.join(";")
}

/// Key for an AutoABI-emitted callee object.
pub fn autoabi_object_key(defs: &[FnDecl], import_inner: bool) -> QueryKey {
    QueryKey::new(
        &LAYER_OBJECT_V01,
        vec![
            ("kind".into(), "autoabi-emit".into()),
            ("decl-set".into(), canonical_decl_set(defs)),
            ("import-inner".into(), import_inner.to_string()),
            ("sem-ref".into(), crate::semref::binding_hash().into()),
            ("target".into(), crate::layout::TARGET.into()),
            ("abi-profile".into(), crate::layout::ABI_PROFILE.into()),
            ("toolchain".into(), emitter_identity()),
        ],
    )
}

/// Key for a cl.exe-compiled C source object: content hash + resolved
/// cl identity + frozen flags (header closure enters with the recipe
/// work; probe C sources are self-contained).
pub fn c_object_key(src_content_hash: &str) -> QueryKey {
    QueryKey::new(
        &LAYER_OBJECT_V01,
        vec![
            ("kind".into(), "cl-compile".into()),
            ("source".into(), src_content_hash.into()),
            ("target".into(), crate::layout::TARGET.into()),
            (
                "toolchain".into(),
                crate::ctool::toolchain_fingerprint().unwrap_or_else(|_| "ctool-unresolved".into()),
            ),
        ],
    )
}

/// Produce-through-cache: on miss, run `produce`, persist, return
/// (bytes, false); on hit, return (bytes, true) WITHOUT calling
/// `produce` — the no-producer-on-hit rule is structural (the closure
/// is simply not invoked).
pub fn produce_with_cache<F>(
    store: &ManagedStore,
    key: &QueryKey,
    produce: F,
) -> Result<(Vec<u8>, bool), StoreError>
where
    F: FnOnce() -> Result<Vec<u8>, String>,
{
    if let Some(LoadedArtifact { payload, .. }) = store.get_artifact(LAYER_OBJECT_V01.id, key)? {
        return Ok((payload, true));
    }
    let obj = produce().map_err(|e| StoreError::Internal(format!("producer: {e}")))?;
    store.put_artifact(PutRequest {
        layer: LAYER_OBJECT_V01.id,
        key: key.clone(),
        kind: ChunkKind::Object,
        payload: obj.clone(),
        edges: vec![],
        project: None,
    })?;
    Ok((obj, false))
}
