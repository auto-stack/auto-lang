//! Object-layer cache adapter over the merged 783 managed store
//! (plan §5.4, T-05). Layouts already persist since T-01 (ABI-L0);
//! this module adds the PRODUCER-OBJECT layer (ABI-O1) for both
//! AutoABI-emitted objects and cl.exe-compiled C objects.
//!
//! Contract (frozen in contract-decision §8):
//! * keys cover every enumerable real input: declaration set (symbols +
//!   signature forms + body identities), semantic-ref snapshot, ABI
//!   profile, target, emitter/toolchain identity, and for C sources the
//!   source name+content + cl identity + frozen flags — a body/
//!   toolchain/flag change is a DIFFERENT key (real invalidation),
//!   never a stale hit. C-compile DEPENDENCIES (headers, including
//!   comment-hidden / line-spliced / macro includes and the system
//!   header closure) are not enumerable from text: they are OBSERVED
//!   from `cl -showIncludes` and re-verified on every hit
//!   (F-797-R4-07 R8);
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

/// F-797-R4-07 (R8): source identity for a cl compile — the file NAME
/// (local `#include "x.h"` resolution is relative to the source's own
/// directory, so two same-content sources at different names can bind
/// different headers) plus the CONTENT hash of the source. The
/// dependency closure is deliberately NOT part of the key: it is not
/// enumerable from text (comment-hidden includes, line splices, macro
/// includes all change what the preprocessor reads without changing the
/// source). Instead [`produce_c_object`] captures the REAL closure with
/// `cl -showIncludes` and re-verifies it on every hit.
pub fn c_source_identity(src: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    Ok(format!(
        "{}|{}",
        src.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
        crate::semref::blake3_hex(&bytes)
    ))
}

/// The stored form of a cached C object: the object bytes, the SOURCE
/// identity it belongs to, and the OBSERVED dependency closure (absolute
/// path, content hash) of the compile that produced it.
#[derive(serde::Serialize, serde::Deserialize)]
struct CObjectEntry {
    v: u32,
    source: String,
    obj_b64: String,
    deps: Vec<(String, String)>,
}

fn hash_dep(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("dep {path}: {e}"))?;
    Ok(crate::semref::blake3_hex(&bytes))
}

fn like_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// F-797-R4-07 (R8): produce-through-cache for cl-compiled objects with
/// REAL dependency invalidation.
///
/// Store keys are CONTENT-BOUND (a key re-put with different bytes is a
/// Conflict), and the dependency closure cannot be known before
/// compiling — so one source lives under MULTIPLE GENERATIONS, one per
/// observed closure:
///
/// * miss: `produce` compiles and returns (object bytes, observed dep
///   paths from `-showIncludes`); the closure is content-hashed into the
///   KEY (`c_object_key`) and the object + manifest are stored together
///   — probe-local AND system/MSVC headers alike (the toolchain exe
///   hash is NOT a header content hash).
/// * hit: the store's PUBLIC db handle (read-only) enumerates this
///   source's generations; the first whose recorded closure still
///   hashes equal — every dep re-read and re-hashed — serves the
///   object. ANY mismatch (changed content, deleted file, manifest that
///   no longer decodes) falls through to a REAL recompile under a new
///   key. A hit NEVER invokes the compiler; superseded generations are
///   unrooted and reclaimed by GC.
pub fn produce_c_object<F>(
    store: &ManagedStore,
    source_identity: &str,
    produce: F,
) -> Result<(Vec<u8>, bool), StoreError>
where
    F: FnOnce() -> Result<(Vec<u8>, Vec<std::path::PathBuf>), String>,
{
    use base64::Engine as _;
    // DISCOVERY (+ verification): same-source generations under distinct
    // closure keys; the first whose recorded closure still hashes equal
    // serves the object. Read-only through the store's PUBLIC db
    // handle; no compiler runs on this path.
    let discover = || -> Result<Option<Vec<u8>>, StoreError> {
        let ids = {
            let db = store.db();
            // canonical keys sort their fields alphabetically
            // (closure=, kind=, source=, target=, toolchain=), so match
            // the source field as a substring; the envelope's recorded
            // `source` is re-checked before any payload is trusted.
            let needle = format!("%source={},target=%", like_escape(source_identity));
            let sql = "SELECT artifact_id FROM artifacts WHERE layer = ?1 \
                       AND key_canonical LIKE ?2 ESCAPE '\\'";
            let mut stmt = db
                .prepare(sql)
                .map_err(|e| StoreError::Internal(format!("closure lookup: {e}")))?;
            let rows = stmt
                .query_map([LAYER_OBJECT_V01.id, needle.as_str()], |row| {
                    row.get::<_, String>(0)
                })
                .map_err(|e| StoreError::Internal(format!("closure lookup: {e}")))?;
            let mut ids = Vec::new();
            for r in rows {
                ids.push(r.map_err(|e| StoreError::Internal(format!("closure lookup: {e}")))?);
            }
            ids
        };
        for id in ids {
            if let Some(LoadedArtifact { payload, .. }) = store.get_artifact_by_id(&id)? {
                let verified = serde_json::from_slice::<CObjectEntry>(&payload)
                    .ok()
                    .filter(|e| e.v == 1 && e.source == source_identity)
                    .and_then(|e| {
                        base64::engine::general_purpose::STANDARD
                            .decode(&e.obj_b64)
                            .ok()
                            .map(|obj| (e.deps, obj))
                    })
                    .filter(|(deps, _)| {
                        deps.iter()
                            .all(|(p, h)| hash_dep(p).is_ok_and(|now| now == *h))
                    });
                if let Some((_, obj)) = verified {
                    return Ok(Some(obj));
                }
            }
        }
        Ok(None)
    };
    if let Some(obj) = discover()? {
        return Ok((obj, true));
    }
    let (obj, deps) = produce().map_err(|e| StoreError::Internal(format!("producer: {e}")))?;
    // F-797-R4-10: single-artifact budget (contract-decision §6) — a
    // producer emitting >64MiB is a RESOURCE rejection, not a stored
    // object and not a truncated success.
    if obj.len() as u64 > 64 * 1024 * 1024 {
        return Err(StoreError::ResourceExhausted(format!(
            "producer object is {} bytes; the single-artifact budget is 64MiB",
            obj.len()
        )));
    }
    let mut hashed: Vec<(String, String)> = Vec::with_capacity(deps.len());
    for p in &deps {
        let ps = p.display().to_string();
        hashed.push((ps.clone(), hash_dep(&ps).map_err(StoreError::Internal)?));
    }
    // closure digest: order-independent (sorted path|hash lines)
    let mut lines: Vec<String> = hashed.iter().map(|(p, h)| format!("{p}|{h}")).collect();
    lines.sort();
    let closure_hash = crate::semref::blake3_hex(lines.join("\n").as_bytes());
    let entry = CObjectEntry {
        v: 1,
        source: source_identity.to_string(),
        obj_b64: base64::engine::general_purpose::STANDARD.encode(&obj),
        deps: hashed,
    };
    let payload = serde_json::to_vec(&entry).map_err(|e| StoreError::Internal(e.to_string()))?;
    match store.put_artifact(PutRequest {
        layer: LAYER_OBJECT_V01.id,
        key: c_object_key(source_identity, &closure_hash),
        kind: ChunkKind::Object,
        payload,
        edges: vec![],
        project: None,
    }) {
        Ok(_) => Ok((obj, false)),
        Err(StoreError::Conflict { .. }) => {
            // cl objects embed a compile timestamp: recompiling the
            // SAME inputs yields different bytes, and the store's keys
            // are content-bound. The STORED generation is the canonical
            // one for this closure — re-discover (the snapshot may be
            // stale, e.g. a concurrent compile won the race) and serve
            // it when its closure still verifies (the producer really
            // ran; the downstream link+run+oracle compare still checks
            // behavior).
            match discover()? {
                Some(stored) => Ok((stored, false)),
                None => Err(StoreError::Internal(format!(
                    "closure key occupied by a foreign entry for source {source_identity}"
                ))),
            }
        }
        Err(e) => Err(e),
    }
}

/// Key for a cl.exe-compiled C source object (F-797-R4-07 R8): source
/// identity (name + content hash) + the OBSERVED closure digest +
/// resolved cl identity + frozen flags. Fields are ordered so the
/// kind+source prefix is stable for generation discovery.
pub fn c_object_key(source_identity: &str, closure_hash: &str) -> QueryKey {
    QueryKey::new(
        &LAYER_OBJECT_V01,
        vec![
            ("kind".into(), "cl-compile".into()),
            ("source".into(), source_identity.into()),
            ("target".into(), crate::layout::TARGET.into()),
            (
                "toolchain".into(),
                crate::ctool::toolchain_fingerprint().unwrap_or_else(|_| "ctool-unresolved".into()),
            ),
            ("closure".into(), closure_hash.into()),
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
