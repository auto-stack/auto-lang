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

#[path = "cache_paths.rs"]
mod cache_paths;
use crate::decl::FnDecl;
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey;
use ace_artifacts::schema::{LayerCoverage, LayerSpec, Regeneration};
use ace_artifacts::store::{LoadedArtifact, ManagedStore, PutRequest, StoreError};
use cache_paths::{
    join_lex, metadata_array, metadata_size, metadata_vec, parent_of, rel_spec, reserve_c_metadata,
    resize_c_metadata, CPathList,
};

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
    cache_paths::source_identity(src, None)
        .map(|identity| identity.value)
        .map_err(|e| match e {
            StoreError::Internal(message) => message,
            other => other.to_string(),
        })
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
/// * `dir_listings`: the NEGATIVE-DEPENDENCY snapshot — the RECURSIVE
///   file listing of EVERY directory that participates in include
///   resolution for this compilation: the source's directory, the
///   explicit `-I` directory, AND the parent directory of every
///   RECORDED dependency (the preprocessor also searches the including
///   file's directory — a header pulled in from anywhere puts ITS
///   directory on the search path, F-797-R4-07 R11/R12's transitive
///   shadow counterexamples, INCLUDING sub-path targets inside those
///   dirs). A NEW or REMOVED file in any of these trees can
///   change include resolution (shadowing) without touching any
///   recorded dep's content, so a listing change is a miss even when
///   every recorded dep still hashes equal.
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
    dir_listings: Vec<(String, Vec<String>)>,
}

struct Base64Ref<'a>(&'a [u8]);
impl serde::Serialize for Base64Ref<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&base64::display::Base64Display::new(
            self.0,
            &base64::engine::general_purpose::STANDARD,
        ))
    }
}
#[derive(serde::Serialize)]
struct CObjectEntryRef<'a> {
    v: u32,
    source: &'a str,
    src_dir: &'a str,
    include_dir: &'a str,
    target: &'a str,
    toolchain: &'a str,
    obj_b64: Base64Ref<'a>,
    deps: &'a [(String, String)],
    dir_listings: &'a [(String, Vec<String>)],
}

struct CGenerationHash {
    value: String,
    _allocation: Option<ace_source_query::resources::Reservation>,
}
fn c_object_generation(
    entry: &CObjectEntryRef<'_>,
    artifact_paths: &[String],
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<CGenerationHash, StoreError> {
    // These borrowed pieces spell precisely the old formatted lines. Compare
    // their concatenated bytes, including ambiguous separators, so sorting /
    // deduplication and the digest remain identical to String::sort + join.
    struct Line<'a>([&'a str; 4]);
    impl Line<'_> {
        fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
            self.0.iter().flat_map(|s| s.bytes())
        }
    }
    let overflow = || StoreError::ResourceExhausted("C cache generation size overflow".into());
    let count = entry.dir_listings.iter().try_fold(
        entry
            .deps
            .len()
            .checked_add(5)
            .and_then(|n| n.checked_add(artifact_paths.len()))
            .ok_or_else(overflow)?,
        |n, (_, files)| {
            n.checked_add(1)
                .and_then(|n| n.checked_add(files.len()))
                .ok_or_else(overflow)
        },
    )?;
    let bytes = count
        .checked_mul(std::mem::size_of::<Line<'_>>())
        .and_then(|n| n.checked_add(64))
        .ok_or_else(overflow)?;
    let mut allocation = resources
        .map(|r| r.reserve(ace_source_query::resources::Charge::index(bytes)))
        .transpose()
        .map_err(|e| StoreError::ResourceExhausted(format!("C cache generation: {e}")))?;
    let mut lines = Vec::new();
    lines.try_reserve_exact(count).map_err(|e| {
        StoreError::ResourceExhausted(format!("C cache generation allocation: {e}"))
    })?;
    for (p, h) in entry.deps {
        lines.push(Line(["dep|", p, "|", h]));
    }
    for (label, value) in [
        ("src_dir|", entry.src_dir),
        ("inc_dir|", entry.include_dir),
        ("target|", entry.target),
        ("toolchain|", entry.toolchain),
    ] {
        lines.push(Line([label, value, "", ""]));
    }
    // The envelope version participates in generation identity (F-797-R15-02).
    lines.push(Line(["env|4", "", "", ""]));
    for a in artifact_paths {
        lines.push(Line(["art|", a, "", ""]));
    }
    for (d, files) in entry.dir_listings {
        lines.push(Line(["dir|", d, "", ""]));
        for name in files {
            lines.push(Line(["file|", d, "|", name]));
        }
    }
    // Equal lines are byte-identical, so unstable ordering has the same
    // canonical result and needs no sorting scratch allocation.
    lines.sort_unstable_by(|a, b| a.bytes().cmp(b.bytes()));
    lines.dedup_by(|a, b| a.bytes().eq(b.bytes()));
    let mut hash = blake3::Hasher::new();
    for (i, line) in lines.iter().enumerate() {
        if i != 0 {
            hash.update(b"\n");
        }
        for part in line.0 {
            hash.update(part.as_bytes());
        }
    }
    let value = hash.finalize().to_hex().to_string();
    drop(lines);
    if let Some(allocation) = &mut allocation {
        allocation
            .resize(ace_source_query::resources::Charge::index(64))
            .map_err(|e| {
                StoreError::ResourceExhausted(format!("C cache generation ownership: {e}"))
            })?;
    }
    Ok(CGenerationHash {
        value,
        _allocation: allocation,
    })
}

enum CacheBytes {
    Legacy(Vec<u8>),
    Managed(std::sync::Arc<Vec<u8>>),
}
impl AsRef<[u8]> for CacheBytes {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Legacy(bytes) => bytes,
            Self::Managed(bytes) => bytes,
        }
    }
}
impl std::ops::Deref for CacheBytes {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
fn load_c_object_entry(
    store: &ManagedStore,
    id: &str,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<Option<CacheBytes>, StoreError> {
    match resources {
        Some(resources) => resources
            .load_payload(64 << 20, ChunkKind::Object, |bound| {
                store.get_artifact_by_id_bounded(id, bound)
            })
            .map(|payload| payload.map(CacheBytes::Managed))
            .map_err(|e| {
                if matches!(&e, ace_source_query::resources::ResourceError::Budget(_)) {
                    StoreError::ResourceExhausted(format!("C cache entry read: {e}"))
                } else {
                    StoreError::Internal(format!("C cache entry read: {e}"))
                }
            }),
        None => store
            .get_artifact_by_id(id)
            .map(|loaded| loaded.map(|LoadedArtifact { payload, .. }| CacheBytes::Legacy(payload))),
    }
}

fn decode_c_object_bytes(
    encoded: &str,
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<Option<CacheBytes>, StoreError> {
    use base64::Engine as _;
    let Some(resources) = resources else {
        return Ok(base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .map(CacheBytes::Legacy));
    };
    let capacity = encoded
        .len()
        .checked_add(3)
        .and_then(|n| n.checked_div(4))
        .and_then(|n| n.checked_mul(3))
        .filter(|n| *n <= 64 << 20)
        .ok_or_else(|| StoreError::ResourceExhausted("C cache object decode size".into()))?;
    let allocation = resources
        .reserve(ace_source_query::resources::Charge::resident(capacity + 40))
        .map_err(|e| StoreError::ResourceExhausted(format!("C cache object decode: {e}")))?;
    let mut bytes = vec![0u8; capacity];
    let Ok(length) = base64::engine::general_purpose::STANDARD.decode_slice(encoded, &mut bytes)
    else {
        return Ok(None);
    };
    bytes.truncate(length);
    let bytes = std::sync::Arc::new(bytes);
    allocation
        .retain(&bytes)
        .map_err(|e| StoreError::ResourceExhausted(format!("C cache object decode: {e}")))?;
    Ok(Some(CacheBytes::Managed(bytes)))
}

/// Read a completed compiler object into its shared owner without an
/// uncharged first Vec. The original object and I/O limits both apply.
pub fn read_c_object_file(
    path: &std::path::Path,
    resources: &ace_source_query::resources::Resources,
) -> Result<std::sync::Arc<Vec<u8>>, StoreError> {
    use ace_source_query::resources::Charge;
    use std::io::Read;
    let budget = |e| StoreError::ResourceExhausted(format!("C object read: {e}"));
    let io_error = |e| StoreError::Internal(format!("C object read: {e}"));
    let path_bytes = path
        .as_os_str()
        .len()
        .checked_mul(4)
        .and_then(|n| n.checked_add(512))
        .ok_or_else(|| StoreError::ResourceExhausted("C object read path size".into()))?;
    let _path = resources
        .reserve(Charge::index(path_bytes))
        .map_err(budget)?;
    let _metadata = resources.reserve(Charge::io(256)).map_err(budget)?;
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options
            .share_mode(1)
            .custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let mut file = options.open(path).map_err(io_error)?;
    let metadata = file.metadata().map_err(io_error)?;
    let invalid = !metadata.is_file();
    #[cfg(windows)]
    let invalid = {
        use std::os::windows::fs::MetadataExt;
        invalid
            || metadata.file_attributes()
                & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
                != 0
    };
    if invalid {
        return Err(StoreError::Internal(
            "C object read requires a regular file without reparse points".into(),
        ));
    }
    let size = usize::try_from(metadata.len())
        .ok()
        .filter(|n| *n <= 64 << 20)
        .ok_or_else(|| {
            StoreError::ResourceExhausted("C object read exceeds the 64MiB artifact limit".into())
        })?;
    let mut allocation = resources.reserve(Charge::io(size)).map_err(budget)?;
    let mut bytes = vec![0u8; size];
    file.read_exact(&mut bytes).map_err(io_error)?;
    let mut extra = [0u8; 1];
    loop {
        match file.read(&mut extra) {
            Ok(0) => break,
            Ok(_) => return Err(StoreError::Internal("C object changed during read".into())),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(io_error(e)),
        }
    }
    drop(file);
    allocation
        .resize(Charge::resident(bytes.capacity() + 40))
        .map_err(budget)?;
    let bytes = std::sync::Arc::new(bytes);
    allocation.retain(&bytes).map_err(budget)?;
    Ok(bytes)
}

struct DecodedCObjectEntry {
    value: CObjectEntry,
    // Fields drop in this order: no owning metadata outlives its fee.
    _allocation: Option<ace_source_query::resources::Reservation>,
}
fn decode_c_object_entry(
    bytes: &[u8],
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<Option<DecodedCObjectEntry>, StoreError> {
    match resources {
        Some(resources) => match resources.decode_metadata_json::<CObjectEntry>(bytes) {
            Ok((value, allocation)) => Ok(Some(DecodedCObjectEntry {
                value,
                _allocation: Some(allocation),
            })),
            Err(ace_source_query::resources::ResourceError::Budget(message)) => Err(
                StoreError::ResourceExhausted(format!("C cache entry decode: {message}")),
            ),
            // Malformed cache envelopes remain a miss under the original
            // cache contract. A budget refusal must never trigger a producer.
            Err(ace_source_query::resources::ResourceError::Decode(_)) => Ok(None),
        },
        None => Ok(serde_json::from_slice(bytes)
            .ok()
            .map(|value| DecodedCObjectEntry {
                value,
                _allocation: None,
            })),
    }
}

/// Snapshot EVERY file the search tree currently offers (negative
/// dependencies): a RECURSIVE walk of the directory — C includes may
/// carry sub-paths (`#include "sub/dep.h"`) and need no extension
/// (`#include "settings"`), so a top-level/extension whitelist would
/// silently miss resolution changes (F-797-R4-07 R10). Only known
/// COMPILE/LINK ARTIFACTS are excluded (a build must not invalidate
/// itself); anything else — extension-less files included — is a
/// potential include candidate and part of the environment.
struct SearchListing {
    names: Vec<String>,
    allocation: Option<ace_source_query::resources::Reservation>,
}

struct ListingBudget {
    allocation: Option<ace_source_query::resources::Reservation>,
}
impl ListingBudget {
    fn new(
        resources: Option<&ace_source_query::resources::Resources>,
        bytes: usize,
    ) -> Result<Self, StoreError> {
        Ok(Self {
            allocation: resources
                .map(|r| r.reserve(ace_source_query::resources::Charge::index(bytes)))
                .transpose()
                .map_err(|e| StoreError::ResourceExhausted(format!("C cache listing: {e}")))?,
        })
    }
    fn resize(&mut self, bytes: usize) -> Result<(), StoreError> {
        if let Some(allocation) = &mut self.allocation {
            allocation
                .resize(ace_source_query::resources::Charge::index(bytes))
                .map_err(|e| StoreError::ResourceExhausted(format!("C cache listing: {e}")))?;
        }
        Ok(())
    }
}
fn listing_size(parts: &[usize]) -> Result<usize, StoreError> {
    parts
        .iter()
        .try_fold(0usize, |n, p| n.checked_add(*p))
        .ok_or_else(|| StoreError::ResourceExhausted("C cache listing size overflow".into()))
}
fn listing_capacity<T>(values: &Vec<T>) -> Result<usize, StoreError> {
    if values.len() < values.capacity() {
        Ok(values.capacity())
    } else {
        values
            .capacity()
            .checked_mul(2)
            .map(|n| n.max(4))
            .ok_or_else(|| {
                StoreError::ResourceExhausted("C cache listing capacity overflow".into())
            })
    }
}
fn listing_grow<T>(values: &mut Vec<T>, capacity: usize) -> Result<(), StoreError> {
    if capacity != values.capacity() {
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|e| {
                StoreError::ResourceExhausted(format!("C cache listing allocation: {e}"))
            })?;
    }
    Ok(())
}
fn search_listing(
    dir: &std::path::Path,
    artifact_paths: &[String],
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<SearchListing, StoreError> {
    // ONLY the EXACT files this pipeline creates are excluded — matched
    // by FULL PATH, never by basename: a declared output `value.obj`
    // must not hide a real header `sub/value.obj` elsewhere in the
    // tree (F-797-R4-07 R15) — plus our staged-link temporaries bound
    // to a declared output's location (`<output>.tmp-<txn>`). Headers
    // with ANY name anywhere are candidates.
    fn is_artifact(full: &str, arts: &[String]) -> bool {
        arts.iter().any(|a| {
            full.eq_ignore_ascii_case(a)
                || (full
                    .as_bytes()
                    .get(..a.len())
                    .is_some_and(|p| p.eq_ignore_ascii_case(a.as_bytes()))
                    && full
                        .as_bytes()
                        .get(a.len()..a.len().saturating_add(5))
                        .is_some_and(|p| p.eq_ignore_ascii_case(b".tmp-")))
        })
    }
    struct Frame {
        entries: std::vec::IntoIter<std::fs::DirEntry>,
        prefix: String,
        // The iterator owns its entire original allocation, even as entries
        // are consumed. Drop its shared root and prefix before this fee.
        _allocation: ListingBudget,
    }
    fn frame(
        dir: &std::path::Path,
        prefix: String,
        resources: Option<&ace_source_query::resources::Resources>,
    ) -> Result<Frame, StoreError> {
        // Windows std stores WIN32_FIND_DATAW inline in DirEntry and shares
        // one Arc<PathBuf> root. Account root/star/UTF-16 path conversion,
        // file_name sort keys (at most 260 UTF-16 units each), and the live
        // relative/full/child paths before read_dir or any of those copies.
        // Callers provide canonical absolute roots, including long paths.
        let base = listing_size(&[
            dir.as_os_str().len().checked_mul(16).ok_or_else(|| {
                StoreError::ResourceExhausted("C cache listing path overflow".into())
            })?,
            prefix.capacity(),
            8192,
        ])?;
        let mut allocation = ListingBudget::new(resources, base)?;
        let mut entries = Vec::new();
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let capacity = listing_capacity(&entries)?;
                // Two full capacities cover growth old+new coexistence and
                // the stable sort's auxiliary storage. Nothing is collected
                // into an uncharged temporary Vec first.
                let buffers = capacity
                    .checked_mul(std::mem::size_of::<std::fs::DirEntry>())
                    .and_then(|n| n.checked_mul(2))
                    .ok_or_else(|| {
                        StoreError::ResourceExhausted("C cache listing entries overflow".into())
                    })?;
                if capacity != entries.capacity() {
                    allocation.resize(listing_size(&[base, buffers])?)?;
                }
                listing_grow(&mut entries, capacity)?;
                entries.push(e);
            }
        }
        entries.sort_by_key(|e| e.file_name());
        allocation.resize(listing_size(&[
            base,
            entries.capacity() * std::mem::size_of::<std::fs::DirEntry>(),
        ])?)?;
        Ok(Frame {
            entries: entries.into_iter(),
            prefix,
            _allocation: allocation,
        })
    }
    let root_bytes = dir
        .as_os_str()
        .len()
        .checked_mul(4)
        .and_then(|n| n.checked_add(64))
        .ok_or_else(|| StoreError::ResourceExhausted("C cache listing root overflow".into()))?;
    let _root_allocation = ListingBudget::new(resources, root_bytes)?;
    let dir_s = dir.display().to_string();
    let root_base = dir_s.strip_prefix("\\\\?\\").unwrap_or(&dir_s);
    let mut output = ListingBudget::new(resources, 0)?;
    let mut names = Vec::new();
    let mut text_bytes = 0usize;
    let mut stack_allocation = ListingBudget::new(resources, 0)?;
    let mut stack: Vec<Frame> = Vec::new();
    let first = frame(dir, String::new(), resources)?;
    let capacity = listing_capacity(&stack)?;
    stack_allocation.resize(capacity * std::mem::size_of::<Frame>() * 2)?;
    listing_grow(&mut stack, capacity)?;
    stack.push(first);
    while let Some(top) = stack.last_mut() {
        let Some(e) = top.entries.next() else {
            stack.pop();
            continue;
        };
        let Ok(name) = e.file_name().into_string() else {
            continue;
        };
        let rel = if top.prefix.is_empty() {
            name
        } else {
            format!("{}/{name}", top.prefix)
        };
        if e.file_type().is_ok_and(|t| t.is_dir()) {
            let child = frame(&e.path(), rel, resources)?;
            let capacity = listing_capacity(&stack)?;
            let bytes = capacity
                .checked_mul(std::mem::size_of::<Frame>())
                .and_then(|n| n.checked_mul(2))
                .ok_or_else(|| {
                    StoreError::ResourceExhausted("C cache listing stack overflow".into())
                })?;
            stack_allocation.resize(bytes)?;
            listing_grow(&mut stack, capacity)?;
            stack.push(child);
        } else {
            // Always SCAN ROOT + accumulated relative path, not the
            // current descended directory (F-797-R16-01).
            let full = format!("{root_base}\\{}", rel.replace('/', "\\"));
            if !is_artifact(&full, artifact_paths) {
                let capacity = listing_capacity(&names)?;
                text_bytes = listing_size(&[text_bytes, rel.capacity()])?;
                let steady = listing_size(&[
                    text_bytes,
                    capacity
                        .checked_mul(std::mem::size_of::<String>())
                        .ok_or_else(|| {
                            StoreError::ResourceExhausted("C cache listing output overflow".into())
                        })?,
                ])?;
                let old = if capacity != names.capacity() {
                    names.capacity() * std::mem::size_of::<String>()
                } else {
                    0
                };
                output.resize(listing_size(&[steady, old])?)?;
                listing_grow(&mut names, capacity)?;
                names.push(rel);
                output.resize(steady)?;
            }
        }
    }
    Ok(SearchListing {
        names,
        allocation: output.allocation,
    })
}

fn hash_dep(path: &str) -> Result<String, String> {
    cache_paths::hash_file(std::path::Path::new(path)).map_err(|e| format!("dep {path}: {e}"))
}

fn current_toolchain(
    resources: Option<&ace_source_query::resources::Resources>,
) -> Result<cache_paths::CPathText, StoreError> {
    match crate::ctool::find_cl() {
        Ok(tool) => cache_paths::toolchain_text(
            &tool.cl_exe,
            &tool.version,
            &tool.cl_hash,
            crate::ctool::CL_FLAGS,
            resources,
        ),
        Err(_) => {
            let allocation = reserve_c_metadata(resources, "ctool-unresolved".len())?;
            Ok(cache_paths::CPathText {
                value: "ctool-unresolved".into(),
                _allocation: allocation,
            })
        }
    }
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
    artifact_paths: &[std::path::PathBuf],
    produce: F,
) -> Result<(Vec<u8>, bool), StoreError>
where
    F: FnOnce() -> Result<(Vec<u8>, Vec<std::path::PathBuf>), String>,
{
    let (bytes, hit) =
        produce_c_object_impl(store, src, include_dir, artifact_paths, None, || {
            produce()
                .map(|(bytes, deps)| (CacheBytes::Legacy(bytes), deps))
                .map_err(|e| StoreError::Internal(format!("producer: {e}")))
        })?;
    let CacheBytes::Legacy(bytes) = bytes else {
        unreachable!("legacy cache used managed output")
    };
    Ok((bytes, hit))
}

/// Source discovery IDs, raw/typed entry reads and decoded output ownership
/// share one account. The producer supplies owned immutable bytes; cache keys
/// and the original applicability algorithm remain unchanged.
pub fn produce_c_object_with_resources<F>(
    store: &ManagedStore,
    src: &std::path::Path,
    include_dir: &std::path::Path,
    artifact_paths: &[std::path::PathBuf],
    resources: &ace_source_query::resources::Resources,
    produce: F,
) -> Result<(std::sync::Arc<Vec<u8>>, bool), StoreError>
where
    F: FnOnce() -> Result<(std::sync::Arc<Vec<u8>>, Vec<std::path::PathBuf>), StoreError>,
{
    let (bytes, hit) = produce_c_object_impl(
        store,
        src,
        include_dir,
        artifact_paths,
        Some(resources),
        || {
            let (bytes, deps) = produce()?;
            resources
                .adopt(
                    &bytes,
                    ace_source_query::resources::Charge::resident(bytes.capacity() + 40),
                )
                .map_err(|e| {
                    StoreError::ResourceExhausted(format!("C cache producer output: {e}"))
                })?;
            Ok((CacheBytes::Managed(bytes), deps))
        },
    )?;
    let CacheBytes::Managed(bytes) = bytes else {
        unreachable!("source cache used unmanaged output")
    };
    Ok((bytes, hit))
}

fn produce_c_object_impl<F>(
    store: &ManagedStore,
    src: &std::path::Path,
    include_dir: &std::path::Path,
    artifact_paths: &[std::path::PathBuf],
    resources: Option<&ace_source_query::resources::Resources>,
    produce: F,
) -> Result<(CacheBytes, bool), StoreError>
where
    F: FnOnce() -> Result<(CacheBytes, Vec<std::path::PathBuf>), StoreError>,
{
    use base64::Engine as _;
    let source_identity = cache_paths::source_identity(src, resources)?;
    // F-797-R9-01 (R14): bind the applicability domain to the RESOLVED
    // directories, not the path LITERALS — canonicalize both anchors so
    // relative spellings resolve against the CURRENT working directory
    // and two projects sharing the same literals never alias.
    // F-797-R15-01: `Path("same.c")` has an EMPTY parent — resolve it
    // to the current directory instead of failing
    let src_parent = src
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    let src_dir = std::fs::canonicalize(src_parent)
        .map_err(|e| StoreError::Internal(format!("canonicalize src dir: {e}")))?
        .display()
        .to_string();
    let inc_dir_s = std::fs::canonicalize(include_dir)
        .map_err(|e| StoreError::Internal(format!("canonicalize include dir: {e}")))?
        .display()
        .to_string();
    // F-797-R4-07 (R15): artifact exclusion by FULL PATH — resolved
    // against the (always existing) parent directory so the spelling
    // is stable whether or not the output file exists yet; separators
    // unified; compared case-insensitively against full listing paths
    let artifact_paths: Vec<String> = artifact_paths
        .iter()
        .map(|p| {
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let parent = p
                .parent()
                .filter(|d| !d.as_os_str().is_empty())
                .unwrap_or(std::path::Path::new("."));
            let dir_s = std::fs::canonicalize(parent)
                .map(|c| c.display().to_string())
                .unwrap_or_else(|_| parent.display().to_string());
            let dir_s = {
                let b = dir_s.as_bytes();
                if b.len() > 4 && b.starts_with(b"\\\\?\\") {
                    dir_s[4..].to_string()
                } else {
                    dir_s
                }
            };
            format!("{}\\{}", dir_s.replace('/', "\\"), name)
        })
        .collect();
    let _target_allocation = reserve_c_metadata(resources, crate::layout::TARGET.len())?;
    let target = crate::layout::TARGET.to_string();
    let toolchain = current_toolchain(resources)?;
    // DISCOVERY (+ verification). Canonical keys sort fields
    // alphabetically (closure, kind, source, target, toolchain), so
    // `source=…,target=…,toolchain=…` is one contiguous substring that
    // pre-filters to the CURRENT target + toolchain/flags before any
    // payload is even loaded; the envelope re-check is authoritative.
    let discover = || -> Result<Option<CacheBytes>, StoreError> {
        let ids = {
            use ace_source_query::resources::Charge;
            let mut allocation = resources
                .map(|r| r.reserve(Charge::index(0)))
                .transpose()
                .map_err(|e| {
                    StoreError::ResourceExhausted(format!("C cache discovery index: {e}"))
                })?;
            let db = store.db();
            let needle = cache_paths::discovery_needle(
                &source_identity.value,
                &target,
                &toolchain.value,
                resources,
            )?;
            let sql = "SELECT artifact_id FROM artifacts WHERE layer = ?1 \
                       AND key_canonical LIKE ?2 ESCAPE '\\'";
            let mut stmt = db
                .prepare(sql)
                .map_err(|e| StoreError::Internal(format!("closure lookup: {e}")))?;
            let mut rows = stmt
                .query([LAYER_OBJECT_V01.id, needle.value.as_str()])
                .map_err(|e| StoreError::Internal(format!("closure lookup: {e}")))?;
            let mut ids = Vec::new();
            let mut text_bytes = 0usize;
            while let Some(row) = rows
                .next()
                .map_err(|e| StoreError::Internal(format!("closure lookup: {e}")))?
            {
                // Borrow SQLite's current row until its String allocation and
                // vector growth have been admitted. No row.get::<String>()
                // allocates the candidate before this check.
                let id = row
                    .get_ref(0)
                    .map_err(|e| StoreError::Internal(format!("closure lookup: {e}")))?
                    .as_str()
                    .map_err(|e| StoreError::Internal(format!("closure lookup: {e}")))?;
                let old = ids.capacity();
                let capacity = if ids.len() == old {
                    old.checked_mul(2).map(|n| n.max(4))
                } else {
                    Some(old)
                }
                .ok_or_else(|| {
                    StoreError::ResourceExhausted("C cache discovery index overflow".into())
                })?;
                text_bytes = text_bytes.checked_add(id.len()).ok_or_else(|| {
                    StoreError::ResourceExhausted("C cache discovery index overflow".into())
                })?;
                let steady = capacity
                    .checked_mul(std::mem::size_of::<String>())
                    .and_then(|n| n.checked_add(text_bytes))
                    .ok_or_else(|| {
                        StoreError::ResourceExhausted("C cache discovery index overflow".into())
                    })?;
                let peak = steady
                    .checked_add(if capacity != old {
                        old * std::mem::size_of::<String>()
                    } else {
                        0
                    })
                    .ok_or_else(|| {
                        StoreError::ResourceExhausted("C cache discovery index overflow".into())
                    })?;
                if let Some(allocation) = &mut allocation {
                    allocation.resize(Charge::index(peak)).map_err(|e| {
                        StoreError::ResourceExhausted(format!("C cache discovery index: {e}"))
                    })?;
                }
                if capacity != old {
                    ids.try_reserve_exact(capacity - ids.len()).map_err(|e| {
                        StoreError::ResourceExhausted(format!(
                            "C cache discovery index allocation: {e}"
                        ))
                    })?;
                }
                ids.push(id.to_owned());
                if let Some(allocation) = &mut allocation {
                    allocation.resize(Charge::index(steady)).map_err(|e| {
                        StoreError::ResourceExhausted(format!("C cache discovery index: {e}"))
                    })?;
                }
            }
            // Tuple field order drops the owned IDs before their reservation.
            (ids, allocation)
        };
        for id in &ids.0 {
            if let Some(payload) = load_c_object_entry(store, id, resources)? {
                let Some(decoded) = decode_c_object_entry(payload.as_ref(), resources)? else {
                    continue;
                };
                let e = &decoded.value;
                if !(
                    // F-797-R4-07 (R14): v4 only — legacy envelopes
                    // computed under a NARROWER applicability rule
                    // (fewer derived candidate dirs, suffix-filtered
                    // listings) are never served; they miss and are
                    // re-proven under the current rule
                    e.v == 4
                        && e.source == source_identity.value
                        && e.src_dir == src_dir
                        && e.include_dir == inc_dir_s
                        && e.target == target
                        && e.toolchain == toolchain.value
                    // every RECORDED search dir (source dir,
                    // include dir, and each recorded dep's
                    // parent — the transitive include-chain
                    // dirs) must still offer exactly the
                    // recorded file set
                ) {
                    continue;
                }
                let mut listings_match = true;
                for (d, expected) in &e.dir_listings {
                    let listing =
                        search_listing(std::path::Path::new(d), &artifact_paths, resources)?;
                    if listing.names != *expected {
                        listings_match = false;
                        break;
                    }
                }
                if !listings_match {
                    continue;
                }
                let Some(obj) = decode_c_object_bytes(&e.obj_b64, resources)? else {
                    continue;
                };
                let mut dependencies_match = true;
                for (p, h) in &e.deps {
                    let _digest_allocation = reserve_c_metadata(resources, 64)?;
                    if !hash_dep(p).is_ok_and(|now| now == *h) {
                        dependencies_match = false;
                        break;
                    }
                }
                if dependencies_match {
                    return Ok(Some(obj));
                }
            }
        }
        Ok(None)
    };
    if let Some(obj) = discover()? {
        return Ok((obj, true));
    }
    let (obj, deps) = produce()?;
    // F-797-R4-10: single-artifact budget (contract-decision §6) — a
    // producer emitting >64MiB is a RESOURCE rejection, not a stored
    // object and not a truncated success.
    if obj.len() as u64 > 64 * 1024 * 1024 {
        return Err(StoreError::ResourceExhausted(format!(
            "producer object is {} bytes; the single-artifact budget is 64MiB",
            obj.len()
        )));
    }
    let hashed_array = metadata_array::<(String, String)>(deps.len())?;
    let mut hashed_allocation = reserve_c_metadata(resources, hashed_array)?;
    let mut hashed: Vec<(String, String)> = metadata_vec(deps.len())?;
    let mut hashed_text = 0usize;
    for p in &deps {
        // Windows OsStr uses WTF-8; replacing an unpaired surrogate for
        // display uses the same three bytes. Pre-size the String to that
        // encoded length so formatting never grows an unreserved buffer.
        let capacity = p.as_os_str().len();
        resize_c_metadata(
            &mut hashed_allocation,
            metadata_size(&[hashed_array, hashed_text, capacity, 64])?,
        )?;
        let mut ps = String::with_capacity(capacity);
        use std::fmt::Write as _;
        write!(&mut ps, "{}", p.display())
            .map_err(|e| StoreError::Internal(format!("dep path formatting: {e}")))?;
        let digest = hash_dep(&ps).map_err(StoreError::Internal)?;
        hashed_text = metadata_size(&[hashed_text, ps.capacity(), digest.capacity()])?;
        hashed.push((ps, digest));
    }
    // F-797-R10-01: the GENERATION identity — everything that decides
    // whether a stored envelope APPLIES (observed closure + source dir +
    // include dir + target + toolchain/flags + both search listings) is
    // folded into the key's generation hash. A legal recompile after a
    // listing/directory change with an UNCHANGED closure therefore gets
    // a NEW key (the old generation stays for its own environment and is
    // reclaimed by GC) instead of colliding with the content-bound old
    // entry; same-input repeats hit and never put at all.
    // F-797-R4-07 (R11/R12/R13): the SEARCH-DOMAIN DIRS — the source
    // dir, the include dir, the parent dir of every recorded dependency
    // (the preprocessor searches the including file's directory), AND
    // the CANDIDATE positions for relative specs: the spec that
    // resolved dependency P from its resolution dir D1 is necessarily
    // rel(P, D1), so for EVERY (P, D1, D2) triple the position
    // normalize(D2 + rel(P, D1)) is a place where a NEW file could
    // preempt the resolution — including parent-path escapes
    // (`../shared/dep.h`) whose candidates lie OUTSIDE every recorded
    // dir's subtree (R13's counterexample). Unbounded derivation over
    // all triples is complete for every spec that actually resolved;
    // the set is capped and oversized derivations fail closed (the
    // compile is simply not cached).
    // every chain dir is CANONICALIZED so rel_spec/join_lex see a
    // uniform prefix (showIncludes paths are plain absolute; canonical
    // forms are \?-prefixed — mixing them breaks the lexical
    // common-prefix derivation)
    let mut chain_dirs = CPathList::new(resources)?;
    chain_dirs.push(&src_dir)?;
    chain_dirs.push(&inc_dir_s)?;
    for (p, _) in &hashed {
        if let Some(parent) = std::path::Path::new(p).parent() {
            let Ok(pc) = std::fs::canonicalize(parent) else {
                continue;
            };
            let ps = pc.display().to_string();
            if !chain_dirs.contains(&ps) {
                chain_dirs.push(&ps)?;
            }
        }
    }
    let mut dirs = CPathList::new(resources)?;
    for d in &chain_dirs.values {
        dirs.push(d)?;
    }
    for (p, _) in &hashed {
        for d1 in &chain_dirs.values {
            if let Some(spec) = rel_spec(d1, p, resources)? {
                for d2 in &chain_dirs.values {
                    let cand = join_lex(d2, &spec.value, resources)?;
                    let parent = parent_of(&cand.value);
                    if !dirs.contains(parent) {
                        dirs.push(parent)?;
                    }
                    if dirs.values.len() > 1024 {
                        // unprovable applicability at sane cost: do not
                        // cache this compile (conservative miss)
                        return Ok((obj, false));
                    }
                }
            }
        }
    }
    dirs.sort_dedup()?;
    // Each listing's output fee moves into one ticket without a release /
    // re-acquire gap, and outlives the complete borrowed encoding operation.
    let mut listing_allocation: Option<ace_source_query::resources::Reservation> = None;
    let listing_metadata = dirs.values.iter().try_fold(
        metadata_array::<(String, Vec<String>)>(dirs.values.len())?,
        |n, dir| metadata_size(&[n, dir.len()]),
    )?;
    let _listing_metadata_allocation = reserve_c_metadata(resources, listing_metadata)?;
    let mut dir_listings = metadata_vec(dirs.values.len())?;
    for d in &dirs.values {
        let listing = search_listing(std::path::Path::new(d), &artifact_paths, resources)?;
        if let Some(next) = listing.allocation {
            listing_allocation = Some(match listing_allocation {
                Some(current) => current.merge(next).map_err(|e| {
                    StoreError::ResourceExhausted(format!("C cache listing ownership: {e}"))
                })?,
                None => next,
            });
        }
        dir_listings.push((d.clone(), listing.names));
    }
    let entry = CObjectEntryRef {
        v: 4,
        source: &source_identity.value,
        src_dir: &src_dir,
        include_dir: &inc_dir_s,
        target: &target,
        toolchain: &toolchain.value,
        obj_b64: Base64Ref(obj.as_ref()),
        deps: &hashed,
        dir_listings: &dir_listings,
    };
    let generation_hash = c_object_generation(&entry, &artifact_paths, resources)?;
    let (payload, encoded_allocation) = if let Some(resources) = resources {
        let size = ace_foreign_contract::codec::json_size(&entry, 64 << 20).map_err(|e| {
            StoreError::ResourceExhausted(format!("C cache entry encode size: {e}"))
        })?;
        // The owned JSON payload and the original store's one framed Vec
        // coexist. CAS staging copies are files (the existing store owns its
        // separate atomic disk gate), not another in-memory payload clone.
        let copies = size
            .checked_add(ace_artifacts::codec::CHUNK_HEADER_LEN)
            .and_then(|n| n.checked_mul(2))
            .ok_or_else(|| {
                StoreError::ResourceExhausted("C cache entry encode size overflow".into())
            })?;
        let allocation = resources
            .reserve(ace_source_query::resources::Charge::io(copies))
            .map_err(|e| StoreError::ResourceExhausted(format!("C cache entry encode: {e}")))?;
        let payload = ace_foreign_contract::codec::encode_json_bounded(&entry, 64 << 20)
            .map_err(|e| StoreError::Internal(format!("C cache entry encode: {e}")))?;
        (payload, Some(allocation))
    } else {
        let entry = CObjectEntry {
            v: 4,
            source: source_identity.value.clone(),
            src_dir: src_dir.clone(),
            include_dir: inc_dir_s.clone(),
            target: target.clone(),
            toolchain: toolchain.value.clone(),
            obj_b64: base64::engine::general_purpose::STANDARD.encode(&obj),
            deps: hashed,
            dir_listings,
        };
        (
            serde_json::to_vec(&entry).map_err(|e| StoreError::Internal(e.to_string()))?,
            None,
        )
    };
    let stored = store.put_artifact(PutRequest {
        layer: LAYER_OBJECT_V01.id,
        key: c_object_key(&source_identity.value, &generation_hash.value),
        kind: ChunkKind::Object,
        payload,
        edges: vec![],
        project: None,
    });
    // PutRequest and the store's framed buffer are gone on every outcome.
    // Release their fee before a conflict re-discovers the canonical object.
    drop(encoded_allocation);
    match stored {
        Ok(_) => Ok((obj, false)),
        Err(StoreError::Conflict { .. }) => {
            // Same generation identity, different bytes: cl objects
            // embed a compile timestamp, so this is a CONCURRENT compile
            // race (a legal recompile after an applicability change now
            // hashes to a different key and cannot reach this arm). The
            // STORED generation is the canonical one — re-discover and
            // serve it when it still verifies (the producer really ran;
            // the downstream link+run+oracle compare still checks
            // behavior).
            match discover()? {
                Some(stored) => Ok((stored, false)),
                None => Err(StoreError::Internal(format!(
                    "generation key occupied by an unverifiable entry for source {}",
                    source_identity.value
                ))),
            }
        }
        Err(e) => Err(e),
    }
}

/// Key for a cl.exe-compiled C source object (F-797-R4-07 R8/R10):
/// source identity (name + content hash) + target + resolved cl
/// identity/flags + the GENERATION digest (observed closure AND the
/// full applicability domain: src/include dirs and both search
/// listings — F-797-R10-01). A legal recompile after an applicability
/// change hashes to a different key and never conflicts with the old
/// generation.
pub fn c_object_key(source_identity: &str, generation_hash: &str) -> QueryKey {
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
            ("generation".into(), generation_hash.into()),
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

#[cfg(test)]
mod path_metadata_tests {
    use super::*;
    use ace_source_query::resources::{Charge, Resources};

    // The pre-port owning algorithm is an independent compatibility oracle
    // for the new borrowed segment walks; it never calls their helpers.
    fn old_norm(p: &str) -> Vec<String> {
        p.strip_prefix("\\\\?\\")
            .unwrap_or(p)
            .split(['/', '\\'])
            .filter(|s| !s.is_empty() && *s != ".")
            .map(str::to_owned)
            .collect()
    }
    fn old_rel(from: &str, to: &str) -> Option<String> {
        let (from, to) = (old_norm(from), old_norm(to));
        if from.is_empty()
            || to.is_empty()
            || from[0].len() < 2
            || to[0].len() < 2
            || from[0].to_ascii_uppercase() != to[0].to_ascii_uppercase()
        {
            return None;
        }
        let mut i = 1;
        while i < from.len() && i < to.len() && from[i].eq_ignore_ascii_case(&to[i]) {
            i += 1;
        }
        let mut parts = vec!["..".to_owned(); from.len() - i];
        parts.extend(to[i..].iter().cloned());
        if parts.is_empty() {
            None
        } else {
            Some(parts.join("/"))
        }
    }
    fn old_join(dir: &str, spec: &str) -> String {
        let mut parts = old_norm(dir);
        for s in spec
            .split(['/', '\\'])
            .filter(|s| !s.is_empty() && *s != ".")
        {
            if s == ".." {
                if parts.len() > 1 {
                    parts.pop();
                }
            } else {
                parts.push(s.to_owned());
            }
        }
        parts.join("\\")
    }
    #[test]
    fn borrowed_lexical_paths_match_the_original_owning_algorithms() {
        let paths = [
            "",
            ".",
            "a",
            "ab",
            "D:",
            "D:\\",
            "d:/a/./b",
            "D:/A/B",
            "D:/a/b/c",
            "D:/a/../b",
            "E:/a/b",
            "\\\\?\\D:\\a\\b",
            "\\\\server\\share\\dir",
            "D:/π/😀/header",
            "D:/a/file.h",
            "D:/a/file.h.more",
        ];
        let specs = [
            "",
            ".",
            "..",
            "../header",
            "../../../../escape",
            "sub/settings",
            "sub/./../config",
            "π/😀",
            "../C:/x",
            "sub\\deeper\\..\\header",
        ];
        let resources = Resources::default();
        let baseline = resources.stats().current.index;
        for from in paths {
            for to in paths {
                for managed in [None, Some(&resources)] {
                    let actual = rel_spec(from, to, managed).unwrap();
                    assert_eq!(
                        actual.as_ref().map(|p| p.value.as_str()),
                        old_rel(from, to).as_deref(),
                        "from={from:?}, to={to:?}"
                    );
                }
                assert_eq!(resources.stats().current.index, baseline);
            }
            for spec in specs {
                for managed in [None, Some(&resources)] {
                    let actual = join_lex(from, spec, managed).unwrap();
                    let expected = old_join(from, spec);
                    assert_eq!(actual.value, expected, "dir={from:?}, spec={spec:?}");
                    assert_eq!(
                        parent_of(&actual.value),
                        std::path::Path::new(&expected)
                            .parent()
                            .map(|p| p.display().to_string())
                            .unwrap_or(expected.clone())
                    );
                }
                assert_eq!(resources.stats().current.index, baseline);
            }
        }
    }
    #[test]
    fn lexical_output_and_path_vectors_keep_exact_fees_and_refusal_is_repairable() {
        let resources = Resources::default();
        let baseline = resources.stats().current.index;
        let text = format!("D:/a/{}", "long-path".repeat(1024));
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline - 64))
            .unwrap();
        let before = resources.stats().current;
        assert!(matches!(
            rel_spec("D:/a/b", &text, Some(&resources)),
            Err(StoreError::ResourceExhausted(_))
        ));
        assert!(matches!(
            join_lex("D:/a/b", &text, Some(&resources)),
            Err(StoreError::ResourceExhausted(_))
        ));
        assert_eq!(resources.stats().current, before);
        drop(pressure);
        let relative = rel_spec("D:/a/b", &text, Some(&resources))
            .unwrap()
            .unwrap();
        assert_eq!(
            resources.stats().current.index,
            baseline + relative.value.capacity()
        );
        let joined = join_lex("D:/a/b", &relative.value, Some(&resources)).unwrap();
        assert_eq!(
            resources.stats().current.index,
            baseline + relative.value.capacity() + joined.value.capacity()
        );
        assert_eq!(
            joined.value,
            old_join("D:/a/b", &old_rel("D:/a/b", &text).unwrap())
        );
        drop((relative, joined));
        assert_eq!(resources.stats().current.index, baseline);
        let mut paths = CPathList::new(Some(&resources)).unwrap();
        paths.push("z").unwrap();
        paths.push("z").unwrap();
        paths.push("a").unwrap();
        paths.sort_dedup().unwrap();
        assert_eq!(paths.values, ["a", "z"]);
        let expected = paths.values.capacity() * std::mem::size_of::<String>() + 2;
        assert_eq!(resources.stats().current.index, baseline + expected);
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline - expected - 64))
            .unwrap();
        let before = resources.stats().current;
        assert!(matches!(
            paths.push(&text),
            Err(StoreError::ResourceExhausted(_))
        ));
        assert_eq!(paths.values, ["a", "z"]);
        assert_eq!(resources.stats().current, before);
        drop(pressure);
        paths.push(&text).unwrap();
        assert_eq!(
            resources.stats().current.index,
            baseline + expected + text.len()
        );
        drop(paths);
        assert_eq!(resources.stats().current.index, baseline);
    }
}
