//! PLAN-891 typed pipeline orchestration: verified typed MIR module →
//! per-unit typed COFF (cached) → cached signature-shaped wrapper →
//! one real PE per run, launched with the one-shot typed request frame on
//! stdin and judged from the single response frame on stdout.
//!
//! Counters are PER PRODUCTION POINT (plan §5.4): a same-signature rerun
//! with new arguments/nonce reuses unit objects AND the wrapper verbatim
//! (production 0) while the PE still actually runs — the machine result
//! is never cached as a verdict.

use crate::cache::{self, CoffMeta, LinkManifest, LinkUnitEntry};
use crate::config::*;
use crate::link_driver::{self, ObjectInput};
use crate::typed_entry::{self, TypedArg, TypedOutcome, TypedSigPlan};
use crate::typed_runtime;
use crate::MnError;
use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey;
use ace_artifacts::store::{ManagedStore, PutRequest};
use ace_av_core::mir::{MirModule, MirType};
use std::path::{Path, PathBuf};

/// Per-production-point counters of one typed run (the warm-incremental
/// observability axes; every production claim maps to a real counter).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypedCounters {
    pub verify_mir: u64,
    pub unit_prepare: u64,
    pub unit_fp_render: u64,
    pub native_admit: u64,
    pub emit_units: u64,
    pub persist_units: u64,
    pub cache_hits: u64,
    pub read_only_miss: u64,
    pub wrapper_builds: u64,
    pub wrapper_hits: u64,
    pub link_invocations: u64,
    pub run_invocations: u64,
    pub whole_fallback: u64,
}

/// The observed result of one real typed PE run.
#[derive(Clone, Debug)]
pub struct TypedRun {
    pub exit_code: Option<i32>,
    pub stdout_frame: Vec<u8>,
    pub stderr: Vec<u8>,
    /// The decoded single response frame (value / trap / resource / error
    /// — mutually exclusive by construction) plus its echoed signature id
    /// and nonce, already checked against the plan.
    pub outcome: TypedOutcome,
    pub echoed_sig_id: u32,
    pub echoed_nonce: u64,
    pub sig_plan: TypedSigPlan,
    pub manifest_fp: String,
    pub unit_symbols: Vec<String>,
    pub counters: TypedCounters,
    pub exe_path: PathBuf,
    pub wrapper_built: bool,
    pub runtime_identity: typed_runtime::Identity,
}

/// How one typed run treats the caches (mirrors the driver's modes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedCacheMode {
    Reuse,
    Fresh,
    ReadOnly,
}

/// Full typed build + real run. `module` may be a whole verified module OR
/// already-split units stitched by the caller — it is (re)verified here
/// either way: every input passes the shared `verify_mir` before any
/// admission, and admission re-checks the typed closure per unit.
#[allow(clippy::too_many_arguments)]
pub fn run_typed(
    module: &MirModule,
    entry: &str,
    args: &[TypedArg],
    nonce: u64,
    workdir: &Path,
    store: Option<&ManagedStore>,
    mode: TypedCacheMode,
    forbid_codegen: bool,
    trace_lib: Option<&Path>,
) -> Result<TypedRun, MnError> {
    let built = build_typed(module, entry, store, mode, forbid_codegen, trace_lib)?;
    run_built(&built, args, nonce, workdir, trace_lib)
}

/// The built (not yet run) typed closure: unit objects, wrapper, manifest.
pub struct TypedBuild {
    pub entry_symbol: String,
    pub sig_plan: TypedSigPlan,
    pub objects: Vec<ObjectInput>,
    pub wrapper: Vec<u8>,
    pub wrapper_built: bool,
    pub manifest_fp: String,
    pub manifest: LinkManifest,
    pub unit_symbols: Vec<String>,
    pub counters: TypedCounters,
    pub runtime_identity: typed_runtime::Identity,
    pub runtime_library: PathBuf,
    pub trace_lib_sha: Option<String>,
}

/// Build the typed closure: verify → split → admit → per-unit emit/cache →
/// wrapper cache → link manifest. No PE is produced or run here.
pub fn build_typed(
    module: &MirModule,
    entry: &str,
    store: Option<&ManagedStore>,
    mode: TypedCacheMode,
    forbid_codegen: bool,
    trace_lib: Option<&Path>,
) -> Result<TypedBuild, MnError> {
    let mut counters = TypedCounters::default();

    // 1. Shared verification FIRST — unverified input never reaches
    //    admission, no matter what its profile string claims.
    ace_av_core::mir_verify::verify_mir(module).map_err(|e| {
        MnError::Unsupported(
            "typed-verify",
            format!("typed input failed shared verify_mir: {e:?}"),
        )
    })?;
    counters.verify_mir += 1;

    // 2. Split into single-function units (S-shared helper).
    let units = ace_av::cache::split_units(module)
        .map_err(|e| MnError::Unsupported("typed-split", format!("unit split failed: {e}")))?;
    counters.unit_prepare += units.len() as u64;

    let entry_unit = units
        .iter()
        .find(|u| u.funcs.first().is_some_and(|f| f.name == entry))
        .ok_or_else(|| {
            MnError::Unsupported(
                "typed-entry",
                format!("entry `{entry}` is not a unit of the typed module"),
            )
        })?;

    // 3. Typed admission of every unit (entry unit additionally checks the
    //    six-scalar-only entry signature).
    for unit in &units {
        typed_entry::admit_typed_unit(unit, unit.funcs[0].name == entry)?;
        counters.native_admit += 1;
    }
    // Cross-unit import closure: every fn:<name> import must resolve to a
    // unit of THIS typed closure with the SAME declared signature — a
    // same-name cross-ABI or drifted-signature link is rejected BEFORE
    // any object production.
    validate_import_closure(&units)?;

    // 4. Typed runtime library (content-bound identity).
    let runtime = typed_runtime::prepare(None)?;
    let runtime_identity = runtime.identity.clone();

    // 5. Per-unit: fingerprint → cache hit (admit) | miss (emit + persist).
    let read_cache = mode != TypedCacheMode::Fresh;
    let persist = mode != TypedCacheMode::ReadOnly;
    let mut objects: Vec<ObjectInput> = Vec::with_capacity(units.len());
    let mut entries: Vec<LinkUnitEntry> = Vec::with_capacity(units.len());
    let mut unit_symbols = Vec::with_capacity(units.len());
    for unit in &units {
        counters.unit_fp_render += 1;
        let fp = cache::unit_fp(unit);
        let meta = CoffMeta::for_typed_unit(unit, fp, &runtime.identity)?;
        match store {
            Some(store) if read_cache => {
                if let Some((id, b)) = cache::get_unit_coff_with_id(store, &meta)? {
                    counters.cache_hits += 1;
                    let symbol = meta.symbol.clone();
                    entries.push(LinkUnitEntry {
                        sha256: cache::content_hash(&b),
                        len: b.len(),
                        artifact_id: id,
                        extra_page_ids: vec![],
                        meta,
                    });
                    objects.push(ObjectInput {
                        name: format!("mn9-{}", unit.funcs[0].name),
                        bytes: b,
                    });
                    unit_symbols.push(symbol);
                    continue;
                }
            }
            _ => {}
        }
        if forbid_codegen {
            return Err(MnError::Admit(
                "forbidden-codegen",
                "typed hit path attempted to compile (forbid_codegen seam)".into(),
            ));
        }
        let obj = typed_entry::emit_typed_unit_coff(unit, unit.funcs[0].name == entry)?;
        counters.emit_units += 1;
        if let Some(store) = store.filter(|_| persist) {
            let (id, _extra) = cache::put_unit_coff(store, None, &meta, &obj.bytes)?;
            counters.persist_units += 1;
            entries.push(LinkUnitEntry {
                sha256: cache::content_hash(&obj.bytes),
                len: obj.bytes.len(),
                artifact_id: id,
                extra_page_ids: vec![],
                meta,
            });
        }
        objects.push(ObjectInput {
            name: format!("mn9-{}", obj.symbol),
            bytes: obj.bytes.clone(),
        });
        unit_symbols.push(obj.symbol.clone());
    }

    // 6. The signature-shaped wrapper — cached under the typed wrapper
    //    layer; argument values and nonces NEVER enter its key.
    let plan = typed_entry::entry_plan(entry_unit)?;
    let entry_symbol = crate::symbols::typed_function_symbol(&entry_unit.owner, &entry);
    let (wrapper, wrapper_built) = typed_wrapper_cached(
        &entry_symbol,
        &plan,
        store,
        mode,
        forbid_codegen,
        &runtime.identity,
    )?;
    if wrapper_built {
        counters.wrapper_builds += 1;
    } else {
        counters.wrapper_hits += 1;
    }

    // 7. Link manifest (typed ABI identity; observation + restore key).
    let (lld_sha, sdk_tag, kernel32_sha) = link_driver::link_identities()?;
    let trace_lib_sha = match trace_lib {
        Some(p) => Some(typed_runtime::file_hash(p)?),
        None => None,
    };
    let manifest = LinkManifest {
        owner: module.owner.clone(),
        entry: entry.to_string(),
        entry_symbol: entry_symbol.clone(),
        abi: TYPED_ABI.to_string(),
        target: MN_TARGET.to_string(),
        units: entries,
        lld_sha,
        sdk_tag,
        kernel32_sha,
        trace_lib_sha: trace_lib_sha.clone(),
        text_lib_sha: Some(runtime.identity.content_hash.clone()),
    };
    let manifest_fp = manifest.fingerprint();
    if let Some(store) = store.filter(|_| persist) {
        let _ = cache::put_link_manifest(store, None, &manifest)?;
    }

    Ok(TypedBuild {
        entry_symbol,
        sig_plan: plan,
        objects,
        wrapper,
        wrapper_built,
        manifest_fp,
        manifest,
        unit_symbols,
        counters,
        runtime_identity,
        runtime_library: runtime.library,
        trace_lib_sha,
    })
}

/// Run an already-built typed closure as a REAL PE with the one-shot
/// request frame; decode and identity-check the single response frame.
pub fn run_built(
    built: &TypedBuild,
    args: &[TypedArg],
    nonce: u64,
    workdir: &Path,
    trace_lib: Option<&Path>,
) -> Result<TypedRun, MnError> {
    let mut counters = built.counters.clone();
    let (run, linked, outcome, echoed_sig_id, echoed_nonce) = run_objects(
        &built.objects,
        &built.wrapper,
        &built.sig_plan,
        args,
        nonce,
        workdir,
        trace_lib,
        &built.runtime_library,
        &mut counters,
    )?;
    let _ = &linked;
    if echoed_sig_id != built.sig_plan.sig_id {
        return Err(MnError::Admit(
            "typed-frame",
            format!(
                "response signature id {echoed_sig_id:#x} does not match the plan {:#x}",
                built.sig_plan.sig_id
            ),
        ));
    }
    if echoed_nonce != nonce {
        return Err(MnError::Admit(
            "typed-frame",
            format!("response nonce {echoed_nonce:#x} does not echo the request nonce {nonce:#x}"),
        ));
    }
    Ok(TypedRun {
        exit_code: run.exit_code,
        stdout_frame: run.stdout,
        stderr: run.stderr,
        outcome,
        echoed_sig_id,
        echoed_nonce,
        sig_plan: built.sig_plan.clone(),
        manifest_fp: built.manifest_fp.clone(),
        unit_symbols: built.unit_symbols.clone(),
        counters,
        exe_path: linked.exe_path.clone(),
        wrapper_built: built.wrapper_built,
        runtime_identity: built.runtime_identity.clone(),
    })
}

/// Link the given unit objects + wrapper and run the REAL PE with the
/// one-shot frame; shared by the one-shot pipeline and the resident
/// session. Counters are the caller's (production points stay with their
/// owner; this helper adds only link/run).
#[allow(clippy::too_many_arguments)]
pub fn run_objects(
    objects: &[ObjectInput],
    wrapper: &[u8],
    plan: &TypedSigPlan,
    args: &[TypedArg],
    nonce: u64,
    workdir: &Path,
    trace_lib: Option<&Path>,
    runtime_library: &Path,
    counters: &mut TypedCounters,
) -> Result<
    (
        ace_core::link::CapturedRun,
        link_driver::PeLink,
        TypedOutcome,
        u32,
        u64,
    ),
    MnError,
> {
    let frame = typed_entry::encode_request(plan, nonce, args)?;
    let mut support: Vec<PathBuf> = vec![runtime_library.to_path_buf()];
    if let Some(t) = trace_lib {
        support.push(t.to_path_buf());
    }
    let linked = link_driver::link_only(
        objects,
        wrapper,
        START_SYMBOL,
        workdir,
        &support,
        &format!("plan891-typed;sig={};abi={TYPED_ABI}", plan.sig_id),
    )?;
    counters.link_invocations += 1;
    let mut run_cmd = std::process::Command::new(&linked.exe_path);
    let run = ace_core::link::run_deadlined_typed(
        &mut run_cmd,
        RUN_BUDGET,
        ace_core::link::CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
        frame,
    )
    .map_err(|e| MnError::Infra(format!("typed run {}: {e:?}", linked.exe_path.display())))?;
    counters.run_invocations += 1;
    let (echoed_sig_id, echoed_nonce, outcome) = typed_entry::decode_response(&run.stdout)?;
    if echoed_sig_id != plan.sig_id {
        return Err(MnError::Admit(
            "typed-frame",
            format!(
                "response signature id {echoed_sig_id:#x} does not match the plan {:#x}",
                plan.sig_id
            ),
        ));
    }
    if echoed_nonce != nonce {
        return Err(MnError::Admit(
            "typed-frame",
            format!("response nonce {echoed_nonce:#x} does not echo the request nonce {nonce:#x}"),
        ));
    }
    Ok((run, linked, outcome, echoed_sig_id, echoed_nonce))
}

/// Every `fn:<name>` import of every unit must resolve within THIS typed
/// closure to a unit whose actual function signature equals the import's
/// declared signature — cross-ABI or drifted-signature linkage is a named
/// pre-link rejection (V-11), never a wrong-function execution.
fn validate_import_closure(units: &[MirModule]) -> Result<(), MnError> {
    for unit in units {
        for imp in &unit.imports {
            let Some(callee) = imp.symbol.strip_prefix("fn:") else {
                continue;
            };
            let target = units
                .iter()
                .find(|u| u.funcs.first().is_some_and(|f| f.name == callee))
                .ok_or_else(|| {
                    MnError::Admit(
                        "callee-sig",
                        format!(
                            "typed import `{}` does not resolve inside this typed closure",
                            imp.symbol
                        ),
                    )
                })?;
            let want = unit.sig_of(imp.sig);
            let Some(want_sig) = want else {
                return Err(MnError::Admit(
                    "callee-sig",
                    format!("typed import `{}` lacks a signature", imp.symbol),
                ));
            };
            let target_sig = target.sig_of(target.funcs[0].sig).ok_or_else(|| {
                MnError::Admit(
                    "callee-sig",
                    format!("typed callee `{callee}` lacks a signature"),
                )
            })?;
            if want_sig != target_sig {
                return Err(MnError::Admit(
                    "callee-sig",
                    format!(
                        "typed import `{}` signature {:?} does not match callee `{}` signature {:?}",
                        imp.symbol, want_sig.params, callee, target_sig.params
                    ),
                ));
            }
        }
    }
    Ok(())
}

// ---- typed wrapper cache ---------------------------------------------------

fn wrapper_key(
    entry_symbol: &str,
    plan: &TypedSigPlan,
    runtime: &typed_runtime::Identity,
) -> QueryKey {
    QueryKey::new(
        &TYPED_WRAPPER_LAYER,
        vec![
            ("producer".into(), PRODUCER_ID.into()),
            ("schema".into(), TYPED_WRAPPER_SCHEMA.into()),
            ("abi".into(), TYPED_ABI.into()),
            ("entry-symbol".into(), entry_symbol.into()),
            ("sig-id".into(), format!("{:08x}", plan.sig_id)),
            ("target".into(), MN_TARGET.into()),
            ("flags".into(), MN_FLAGS.into()),
            ("transport".into(), "mn891.typed.frames/v1".into()),
            ("helper-library".into(), runtime.content_hash.clone()),
        ],
    )
}

fn wrapper_header(entry_symbol: &str, plan: &TypedSigPlan, bytes: &[u8]) -> String {
    format!(
        "schema={}\nabi={}\nentry-symbol={}\nsig={}\nsig-id={:08x}\ntarget={}\nflags={}\nlen={}\nsha256={}\n",
        TYPED_WRAPPER_SCHEMA,
        TYPED_ABI,
        entry_symbol,
        plan.sig_text,
        plan.sig_id,
        MN_TARGET,
        MN_FLAGS,
        bytes.len(),
        cache::content_hash(bytes),
    )
}

/// Get (admit) or build+persist the typed wrapper. The restore admission
/// re-checks the envelope, exact length, content hash and structural COFF
/// — a self-consistent wrong wrapper still fails.
/// Session-facing wrapper cache access: identical admission/build
/// discipline to the one-shot pipeline (the session's edits are genuine
/// production work, so no forbid_codegen seam here).
pub fn typed_wrapper_for_session(
    entry_symbol: &str,
    plan: &TypedSigPlan,
    store: Option<&ManagedStore>,
    mode: TypedCacheMode,
    runtime: &typed_runtime::Identity,
) -> Result<(Vec<u8>, bool), MnError> {
    typed_wrapper_cached(entry_symbol, plan, store, mode, false, runtime)
}

fn typed_wrapper_cached(
    entry_symbol: &str,
    plan: &TypedSigPlan,
    store: Option<&ManagedStore>,
    mode: TypedCacheMode,
    forbid_codegen: bool,
    runtime: &typed_runtime::Identity,
) -> Result<(Vec<u8>, bool), MnError> {
    let key = wrapper_key(entry_symbol, plan, runtime);
    if mode != TypedCacheMode::Fresh {
        if let Some(store) = store {
            if let Some(art) = store
                .get_artifact(TYPED_WRAPPER_LAYER.id, &key)
                .map_err(|e| MnError::Infra(format!("store get wrapper: {e}")))?
            {
                let payload = &art.payload;
                // The envelope is a line block closed by an EMPTY line
                // (the unit COFF envelope discipline); the wrapper bytes
                // start right after it.
                let body_start = payload
                    .windows(2)
                    .position(|w| w == b"\n\n")
                    .map(|i| i + 2)
                    .ok_or_else(|| {
                        MnError::Admit("wrapper", "wrapper envelope lacks a terminator".into())
                    })?;
                let header = String::from_utf8_lossy(&payload[..body_start]).to_string();
                let expect = |k: &str, want: &str| -> Result<(), MnError> {
                    let line = format!("{k}={want}\n");
                    if !header.contains(&line) {
                        return Err(MnError::Admit(
                            "wrapper",
                            format!("wrapper envelope `{k}` mismatch"),
                        ));
                    }
                    Ok(())
                };
                expect("schema", TYPED_WRAPPER_SCHEMA)?;
                expect("abi", TYPED_ABI)?;
                expect("entry-symbol", entry_symbol)?;
                expect("sig", &plan.sig_text)?;
                expect("target", MN_TARGET)?;
                expect("flags", MN_FLAGS)?;
                let body = &payload[body_start..];
                let len: usize = header
                    .lines()
                    .find(|l| l.starts_with("len="))
                    .and_then(|l| l[4..].parse().ok())
                    .ok_or_else(|| MnError::Admit("wrapper", "bad wrapper len".into()))?;
                let sha = header
                    .lines()
                    .find(|l| l.starts_with("sha256="))
                    .map(|l| l[7..].to_string())
                    .ok_or_else(|| MnError::Admit("wrapper", "bad wrapper sha".into()))?;
                if body.len() != len
                    || cache::content_hash(body) != sha
                    || body.len() > MAX_COFF_BYTES
                {
                    return Err(MnError::Admit(
                        "wrapper",
                        "restored wrapper fails the length/hash bind".into(),
                    ));
                }
                let body = body.to_vec();
                ace_av::coff_admission::check_coff_relocatable(&body)
                    .map_err(|e| MnError::Admit("wrapper", e.to_string()))?;
                return Ok((body, false));
            }
        }
    }
    if forbid_codegen {
        return Err(MnError::Admit(
            "forbidden-codegen",
            "typed wrapper cache miss attempted to build (forbid_codegen seam)".into(),
        ));
    }
    let bytes = typed_entry::emit_typed_wrapper_coff(entry_symbol, plan)?;
    if let Some(store) = store.filter(|_| mode != TypedCacheMode::ReadOnly) {
        let mut payload = wrapper_header(entry_symbol, plan, &bytes).into_bytes();
        payload.push(b'\n');
        payload.extend_from_slice(&bytes);
        if payload.len() > METADATA_CAP {
            return Err(MnError::Resource(
                "metadata-size",
                "typed wrapper envelope above the metadata cap".into(),
            ));
        }
        store
            .put_artifact(PutRequest {
                layer: TYPED_WRAPPER_LAYER.id,
                key,
                kind: ChunkKind::LayoutData,
                payload,
                edges: vec![],
                project: None,
            })
            .map_err(|e| MnError::Infra(format!("store put wrapper: {e}")))?;
    }
    Ok((bytes, true))
}

/// Type-shape helper for closure validation messages.
pub fn describe_type(m: &MirModule, ty: u32) -> String {
    match m.types.get(ty as usize) {
        Some(MirType::Fn(s)) => format!(
            "({})->{}",
            s.params.len(),
            m.types
                .get(s.result as usize)
                .map(|t| format!("{t:?}"))
                .unwrap_or_else(|| "?".into())
        ),
        other => format!("{other:?}"),
    }
}
