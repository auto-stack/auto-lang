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

use crate::decl::FnDecl;
use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{LoadedArtifact, ManagedStore, PutRequest, StoreError};

/// Probe-owned layer: producer objects (AutoABI-emitted or C-compiled).
pub const LAYER_OBJECT_V01: LayerSpec = LayerSpec {
    id: "ABI-O1",
    name: "autoabi-producer-object",
    schema_id: "auto797/abi-o1-object/v1",
    producer: "auto-abi-probe emit / cl.exe (identity in key)",
    consumers: &["auto-abi-probe link pipeline"],
    key_inputs: &[
        "decl-set",
        "sem-ref",
        "target",
        "abi-profile",
        "toolchain",
        "source-hash",
    ],
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

/// F-797-R4-07: collect the LOCAL include closure of a C source —
/// every `#include "..."` reachable from it, content-hashed with its
/// resolved path (system/MSVC headers are covered by the toolchain
/// fingerprint; only repo-local headers vary independently). An
/// unresolvable local include is an ERROR: while a dependency cannot be
/// pinned, the compile is not a pure function of the source and must
/// not be cached as one.
pub fn c_include_closure(
    src: &std::path::Path,
    include_dir: &std::path::Path,
) -> Result<String, String> {
    let mut seen: Vec<std::path::PathBuf> = Vec::new();
    let mut stack = vec![src.to_path_buf()];
    let mut digest_input = String::new();
    while let Some(path) = stack.pop() {
        if seen.iter().any(|p| p == &path) {
            continue; // include cycles collapse to the first visit
        }
        let content = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let hash = crate::semref::blake3_hex(&content);
        digest_input.push_str(&format!(
            "{}|{}\n",
            path.display().to_string().replace('/', "\\"),
            hash
        ));
        seen.push(path.clone());
        for line in String::from_utf8_lossy(&content).lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("#include \"") {
                if let Some(name) = rest.strip_suffix('"') {
                    // resolve relative to the INCLUDING file first, then
                    // the shared include dir (the probe's convention)
                    let here = path.parent().unwrap_or(include_dir).join(name);
                    let shared = include_dir.join(name);
                    let resolved = if here.is_file() {
                        here
                    } else if shared.is_file() {
                        shared
                    } else {
                        return Err(format!(
                            "local include {name:?} of {} not found (the compile is not a \
                             pure function of the source while a dependency is unresolvable)",
                            path.display()
                        ));
                    };
                    stack.push(resolved);
                }
            }
        }
    }
    Ok(crate::semref::blake3_hex(digest_input.as_bytes()))
}

/// Key for a cl.exe-compiled C source object: the SOURCE + its local
/// INCLUDE CLOSURE (F-797-R4-07: changing a header changes the key —
/// never a stale hit serving old behavior) + resolved cl identity +
/// frozen flags.
pub fn c_object_key(include_closure_hash: &str) -> QueryKey {
    QueryKey::new(
        &LAYER_OBJECT_V01,
        vec![
            ("kind".into(), "cl-compile".into()),
            ("source+includes".into(), include_closure_hash.into()),
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
    // F-797-R4-10: single-artifact budget (contract-decision §6) — a
    // producer emitting >64MiB is a RESOURCE rejection, not a stored
    // object and not a truncated success.
    if obj.len() as u64 > 64 * 1024 * 1024 {
        return Err(StoreError::ResourceExhausted(format!(
            "producer object is {} bytes; the single-artifact budget is 64MiB",
            obj.len()
        )));
    }
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
