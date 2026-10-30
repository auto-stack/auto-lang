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
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey;
use ace_artifacts::schema::{LayerCoverage, LayerSpec, Regeneration};
use ace_artifacts::store::{LoadedArtifact, ManagedStore, PutRequest, StoreError};

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

/// Snapshot EVERY file the search tree currently offers (negative
/// dependencies): a RECURSIVE walk of the directory — C includes may
/// carry sub-paths (`#include "sub/dep.h"`) and need no extension
/// (`#include "settings"`), so a top-level/extension whitelist would
/// silently miss resolution changes (F-797-R4-07 R10). Only known
/// COMPILE/LINK ARTIFACTS are excluded (a build must not invalidate
/// itself); anything else — extension-less files included — is a
/// potential include candidate and part of the environment.
fn search_listing(dir: &std::path::Path, artifact_paths: &[String]) -> Vec<String> {
    // ONLY the EXACT files this pipeline creates are excluded — matched
    // by FULL PATH, never by basename: a declared output `value.obj`
    // must not hide a real header `sub/value.obj` elsewhere in the
    // tree (F-797-R4-07 R15) — plus our staged-link temporaries bound
    // to a declared output's location (`<output>.tmp-<txn>`). Headers
    // with ANY name anywhere are candidates.
    fn is_artifact(full: &str, arts: &[String]) -> bool {
        arts.iter().any(|a| {
            full.eq_ignore_ascii_case(a)
                || full
                    .to_ascii_lowercase()
                    .starts_with(&format!("{a}.tmp-").to_ascii_lowercase())
        })
    }
    fn walk(
        dir: &std::path::Path,
        root_base: &str,
        prefix: &str,
        out: &mut Vec<String>,
        arts: &[String],
    ) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let Ok(name) = e.file_name().into_string() else {
                continue;
            };
            let rel = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}/{name}")
            };
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                walk(&e.path(), root_base, &rel, out, arts);
            } else {
                // full path = SCAN ROOT + accumulated relative path —
                // the current dir already contains the descended prefix
                // (F-797-R16-01: using the current dir here doubled the
                // relative part, so a declared nested output never
                // matched and its cleanup caused a spurious miss)
                let full = format!("{root_base}\\{}", rel.replace('/', "\\"));
                if !is_artifact(&full, arts) {
                    out.push(rel);
                }
            }
        }
    }
    let dir_s = dir.display().to_string();
    let root_base = {
        let b = dir_s.as_bytes();
        if b.len() > 4 && b.starts_with(b"\\\\?\\") {
            dir_s[4..].to_string()
        } else {
            dir_s
        }
    };
    let mut names = Vec::new();
    walk(dir, &root_base, "", &mut names, artifact_paths);
    names
}

fn hash_dep(path: &str) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("dep {path}: {e}"))?;
    Ok(crate::semref::blake3_hex(&bytes))
}

/// Lexical relative path FROM a directory TO a file (leading `..`
/// segments allowed; same-drive only). `None` when not derivable.
fn rel_spec(from_dir: &str, to_file: &str) -> Option<String> {
    let norm_from = split_norm(from_dir);
    let norm_to = split_norm(to_file);
    if norm_from.is_empty() || norm_to.is_empty() {
        return None;
    }
    if norm_from[0].len() < 2 || norm_to[0].len() < 2 {
        return None; // not absolute (or no drive letter)
    }
    if norm_from[0].to_ascii_uppercase() != norm_to[0].to_ascii_uppercase() {
        return None; // cross-drive
    }
    let mut i = 1;
    while i < norm_from.len() && i < norm_to.len() && eq_ci(&norm_from[i], &norm_to[i]) {
        i += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in i..norm_from.len() {
        parts.push("..".into());
    }
    for seg in &norm_to[i..] {
        parts.push(seg.clone());
    }
    if parts.is_empty() {
        None // the file IS the dir anchor itself
    } else {
        Some(parts.join("/"))
    }
}

/// Lexically join a relative spec onto a directory and normalize `.`
/// and `..` segments WITHOUT touching the filesystem.
fn join_lex(dir: &str, spec: &str) -> String {
    let mut out: Vec<String> = split_norm(dir);
    for seg in spec
        .split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".")
    {
        if seg == ".." {
            // never pop the drive root
            if out.len() > 1 {
                out.pop();
            }
        } else {
            out.push(seg.to_string());
        }
    }
    out.join("\\")
}

fn parent_of(path: &str) -> String {
    match std::path::Path::new(path).parent() {
        Some(p) => p.display().to_string(),
        None => path.to_string(),
    }
}

fn split_norm(p: &str) -> Vec<String> {
    // strip the canonical-form prefix so `\\?\D:\...` and `D:\...`
    // compare and join uniformly (canonicalized anchors must mix with
    // plain showIncludes paths in the derivation)
    let stripped;
    let p = if let Some(rest) = p.strip_prefix("\\\\?\\") {
        stripped = rest;
        &stripped
    } else {
        p
    };
    p.split(['/', '\\'])
        .filter(|s| !s.is_empty() && *s != ".")
        .map(|s| s.to_string())
        .collect()
}

fn eq_ci(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
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
    artifact_paths: &[std::path::PathBuf],
    produce: F,
) -> Result<(Vec<u8>, bool), StoreError>
where
    F: FnOnce() -> Result<(Vec<u8>, Vec<std::path::PathBuf>), String>,
{
    use base64::Engine as _;
    let source_identity =
        c_source_identity(src).map_err(|e| StoreError::Internal(format!("{e}")))?;
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
                        // F-797-R4-07 (R14): v4 only — legacy envelopes
                        // computed under a NARROWER applicability rule
                        // (fewer derived candidate dirs, suffix-filtered
                        // listings) are never served; they miss and are
                        // re-proven under the current rule
                        e.v == 4
                            && e.source == source_identity
                            && e.src_dir == src_dir
                            && e.include_dir == inc_dir_s
                            && e.target == target
                            && e.toolchain == toolchain
                            // every RECORDED search dir (source dir,
                            // include dir, and each recorded dep's
                            // parent — the transitive include-chain
                            // dirs) must still offer exactly the
                            // recorded file set
                            && e.dir_listings.iter().all(|(d, l)| {
                                search_listing(std::path::Path::new(d), &artifact_paths) == *l
                            })
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
    let mut chain_dirs: Vec<String> = vec![src_dir.clone(), inc_dir_s.clone()];
    for (p, _) in &hashed {
        if let Some(parent) = std::path::Path::new(p).parent() {
            let Ok(pc) = std::fs::canonicalize(parent) else {
                continue;
            };
            let ps = pc.display().to_string();
            if !chain_dirs.contains(&ps) {
                chain_dirs.push(ps);
            }
        }
    }
    let mut dirs: Vec<String> = chain_dirs.clone();
    for (p, _) in &hashed {
        for d1 in &chain_dirs {
            if let Some(spec) = rel_spec(d1, p) {
                for d2 in &chain_dirs {
                    let cand = join_lex(d2, &spec);
                    let parent = parent_of(&cand);
                    if !dirs.contains(&parent) {
                        dirs.push(parent);
                    }
                    if dirs.len() > 1024 {
                        // unprovable applicability at sane cost: do not
                        // cache this compile (conservative miss)
                        return Ok((obj, false));
                    }
                }
            }
        }
    }
    dirs.sort();
    dirs.dedup();
    let dir_listings: Vec<(String, Vec<String>)> = dirs
        .iter()
        .map(|d| {
            (
                d.clone(),
                search_listing(std::path::Path::new(d), &artifact_paths),
            )
        })
        .collect();
    let mut gen_lines: Vec<String> = hashed.iter().map(|(p, h)| format!("dep|{p}|{h}")).collect();
    gen_lines.push(format!("src_dir|{src_dir}"));
    gen_lines.push(format!("inc_dir|{inc_dir_s}"));
    gen_lines.push(format!("target|{target}"));
    gen_lines.push(format!("toolchain|{toolchain}"));
    // F-797-R15-02: the ENVELOPE ALGORITHM VERSION is part of the
    // generation identity — a legacy v3 entry (even one whose domain
    // equals the current one) can never occupy a v4 key, so an upgrade
    // recompile always publishes instead of dead-ending in Conflict
    gen_lines.push("env|4".to_string());
    for a in &artifact_paths {
        gen_lines.push(format!("art|{a}"));
    }
    for (d, l) in &dir_listings {
        gen_lines.push(format!("dir|{d}"));
        for n in l {
            gen_lines.push(format!("file|{d}|{n}"));
        }
    }
    gen_lines.sort();
    gen_lines.dedup();
    let generation_hash = crate::semref::blake3_hex(gen_lines.join("\n").as_bytes());
    let entry = CObjectEntry {
        v: 4,
        source: source_identity.clone(),
        src_dir: src_dir.clone(),
        include_dir: inc_dir_s.clone(),
        target: target.clone(),
        toolchain: toolchain.clone(),
        obj_b64: base64::engine::general_purpose::STANDARD.encode(&obj),
        deps: hashed,
        dir_listings,
    };
    let payload = serde_json::to_vec(&entry).map_err(|e| StoreError::Internal(e.to_string()))?;
    match store.put_artifact(PutRequest {
        layer: LAYER_OBJECT_V01.id,
        key: c_object_key(&source_identity, &generation_hash),
        kind: ChunkKind::Object,
        payload,
        edges: vec![],
        project: None,
    }) {
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
                    "generation key occupied by an unverifiable entry for source {source_identity}"
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
