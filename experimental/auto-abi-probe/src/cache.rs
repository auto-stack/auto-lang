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

/// F-797-R4-07 (R8/R9): source identity for a cl compile — the file
/// NAME plus the CONTENT hash of the source. This is only the KEY
/// handle; the full applicability of a stored generation (source
/// directory, include dir, target, toolchain/flags, search-listing) is
/// carried in the entry envelope and re-verified by
/// [`produce_c_object`] on every hit (F-797-R9-01: the key identity
/// alone proves nothing about whether a generation applies to the
/// CURRENT request).
pub fn c_source_identity(src: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(src).map_err(|e| format!("{}: {e}", src.display()))?;
    Ok(format!(
        "{}|{}",
        src.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
        crate::semref::blake3_hex(&bytes)
    ))
}

/// The stored form of a cached C object (v2). Besides the object bytes
/// and the OBSERVED dependency closure, an entry records the FULL
/// search-domain facts of the compile that produced it — every one of
/// them is re-verified before a hit serves anything:
/// * `src_dir` / `include_dir`: quoted includes resolve relative to the
///   source's own directory first, then the `-I` dir — same name+content
///   at a different directory is a DIFFERENT compilation
///   (F-797-R9-01's cross-project wrong-object counterexample);
/// * `target` / `toolchain`: the key carries them, but the DISCOVERY
///   path must consume them too (F-797-R9-01's foreign-toolchain
///   positive control);
/// * `src_listing` / `inc_listing`: the NEGATIVE-DEPENDENCY snapshot —
///   the source/header file NAMES visible in the two search dirs. A NEW
///   or REMOVED nearby file can change include resolution (shadowing)
///   without touching any recorded dep's content
///   (F-797-R4-07's shadow counterexample), so a listing change is a
///   miss even when every recorded dep still hashes equal.
#[derive(serde::Serialize, serde::Deserialize)]
struct CObjectEntry {
    v: u32,
    source: String,
    src_dir: String,
    include_dir: String,
    target: String,
    toolchain: String,
    obj_b64: String,
    deps: Vec<(String, String)>,
    src_listing: Vec<String>,
    inc_listing: Vec<String>,
}

/// Snapshot the source/header file names a search dir currently offers
/// (negative dependencies). Compile artifacts (.obj/.exe) are excluded
/// so a build does not invalidate itself.
fn search_listing(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| {
            let l = n.to_ascii_lowercase();
            l.ends_with(".c")
                || l.ends_with(".h")
                || l.ends_with(".hpp")
                || l.ends_with(".hh")
                || l.ends_with(".inc")
                || l.ends_with(".inl")
        })
        .collect();
    names.sort();
    names
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

fn current_toolchain() -> String {
    crate::ctool::toolchain_fingerprint().unwrap_or_else(|_| "ctool-unresolved".into())
}

/// F-797-R4-07 (R8/R9) + F-797-R9-01: produce-through-cache for
/// cl-compiled objects with REAL dependency invalidation and FULL
/// generation applicability.
///
/// Store keys are CONTENT-BOUND (a key re-put with different bytes is a
/// Conflict), and the dependency closure cannot be known before
/// compiling — so one source lives under MULTIPLE GENERATIONS, one per
/// observed closure:
///
/// * miss: `produce` compiles and returns (object bytes, observed dep
///   paths from `-showIncludes`); the closure is content-hashed into the
///   KEY and the object + manifest are stored together — probe-local
///   AND system/MSVC headers alike.
/// * hit: the store's PUBLIC db handle (read-only) enumerates
///   generations for THIS source identity AND the CURRENT target +
///   toolchain fingerprint; the first generation whose envelope matches
///   the current source directory, include dir, and search listings,
///   AND whose recorded deps all still hash equal, serves the object.
///   ANY mismatch — changed dep content, deleted file, new/removed
///   nearby header (shadowing), different directory/toolchain, stale
///   envelope — is a miss (real recompile). A hit NEVER invokes the
///   compiler; superseded generations are unrooted and reclaimed by GC.
pub fn produce_c_object<F>(
    store: &ManagedStore,
    src: &std::path::Path,
    include_dir: &std::path::Path,
    produce: F,
) -> Result<(Vec<u8>, bool), StoreError>
where
    F: FnOnce() -> Result<(Vec<u8>, Vec<std::path::PathBuf>), String>,
{
    use base64::Engine as _;
    let source_identity =
        c_source_identity(src).map_err(|e| StoreError::Internal(format!("{e}")))?;
    let src_dir = src
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let inc_dir_s = include_dir.display().to_string();
    let target = crate::layout::TARGET.to_string();
    let toolchain = current_toolchain();
    // DISCOVERY (+ verification). Canonical keys sort fields
    // alphabetically (closure, kind, source, target, toolchain), so
    // `source=…,target=…,toolchain=…` is one contiguous substring that
    // pre-filters to the CURRENT target + toolchain/flags before any
    // payload is even loaded; the envelope re-check is authoritative.
    let discover = || -> Result<Option<Vec<u8>>, StoreError> {
        let ids = {
            let db = store.db();
            let needle = format!(
                "%source={},target={},toolchain={}%",
                like_escape(&source_identity),
                like_escape(&target),
                like_escape(&toolchain)
            );
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
                    .filter(|e| {
                        e.v == 2
                            && e.source == source_identity
                            && e.src_dir == src_dir
                            && e.include_dir == inc_dir_s
                            && e.target == target
                            && e.toolchain == toolchain
                            && e.src_listing == search_listing(std::path::Path::new(&src_dir))
                            && e.inc_listing == search_listing(include_dir)
                    })
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
        v: 2,
        source: source_identity.clone(),
        src_dir: src_dir.clone(),
        include_dir: inc_dir_s.clone(),
        target: target.clone(),
        toolchain: toolchain.clone(),
        obj_b64: base64::engine::general_purpose::STANDARD.encode(&obj),
        deps: hashed,
        src_listing: search_listing(std::path::Path::new(&src_dir)),
        inc_listing: search_listing(include_dir),
    };
    let payload = serde_json::to_vec(&entry).map_err(|e| StoreError::Internal(e.to_string()))?;
    match store.put_artifact(PutRequest {
        layer: LAYER_OBJECT_V01.id,
        key: c_object_key(&source_identity, &closure_hash),
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
            // it when it still verifies (the producer really ran; the
            // downstream link+run+oracle compare still checks
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
