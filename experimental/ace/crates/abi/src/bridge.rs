//! PLAN-937 CA-D1 bridge: the host composition that connects the REAL
//! existing stacks through the neutral foreign contract — no new FFI
//! database, no new process runner (937 plan §2/§5.4).
//!
//! Two real chains live here:
//!
//! * **typed peer chain** (V-01/V-03/V-04): typed declarations →
//!   foreign-contract facts → physical plan → descriptor/3 projection →
//!   PRE-CALL admission → real Cranelift machine code + real cl.exe
//!   objects, linked into PEs and EXECUTED (never statically "proven"):
//!   AutoABI-typed caller→C callee, C caller→AutoABI-typed callee, and
//!   the emitted↔emitted self-check — every PE carries its own oracle.
//! * **source export chain** (V-02): real Auto source → the checked
//!   frontdoor (`ace_core::source`) → the REAL C provider's fragments
//!   (per-function, cache-accounted) → the PUBLIC library fileset
//!   (assembled by `ace_a2x_c::abi_scalar`, no process main) →
//!   admission → compile with the full CRT closure → an independent
//!   hand-written C caller EXECUTES the exported surface.
//!
//! Increment/restore (V-05/V-06): every bridge artifact is produced
//! through the managed store under bridge-owned keys with per-key
//! producer counters — a hit never runs its producer, a body edit
//! reproduces exactly the one changed function's fragment, and
//! [`restore_bridge`] decodes persisted artifacts with bounded codecs
//! and re-verifies identity WITHOUT re-running any producer.
//!
//! Admission is pre-call (plan §5.3): a rejected binding produces a
//! NAMED rejection and the callee counter stays 0 — no call is made to
//! a known-incompatible surface.

use crate::ctool;
use crate::link::link_pe;
use crate::run::run_pe;
use crate::typed_emit::{emit_typed_callee, emit_typed_caller, frozen_typed_checks, TypedCheck};
use crate::validate::{admit_descriptor, ConsumerExpectation, Rejection};
use crate::{descriptor, layout, typed_decl};

use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey;
use ace_artifacts::schema::{LayerCoverage, LayerSpec, Regeneration};
use ace_artifacts::store::{LoadedArtifact, ManagedStore, PutRequest, StoreError};
use ace_core::link::{run_deadlined_captured, CaptureCaps};
use ace_foreign_contract::codec::{facts_codec, plan_codec};
use ace_foreign_contract::lane::{LaneKind, PhysLane};
use ace_foreign_contract::plan::{PhysForm, PhysRet, PhysicalPlan};
use ace_foreign_contract::typed::TypedDecls;
use ace_foreign_contract::{ErrorModel, FactsSource, FnFact, FnForm, InterfaceFacts};
use ace_semantics_contract::codec::{encode_typedesc_bytes, TypeDesc};
use ace_semantics_contract::identity::{ContentVersion, DeclKind, DeclRef, PackageId};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Bridge-owned artifact layer (probe-owned like ABI-O1; nothing in the
/// 783/787 layer registry changes).
pub const LAYER_BRIDGE_V01: LayerSpec = LayerSpec {
    id: "ABI-BR1",
    name: "autoabi-bridge-artifact",
    schema_id: "autoabi.bridge-artifact/v1",
    producer: "ace-abi bridge (typed emit / a2x-c provider fragments / abi_scalar assembly)",
    consumers: &["ace-abi bridge restore", "bridge certification"],
    key_inputs: &[
        "kind",
        "facts-digest",
        "plan-digest",
        "sem-fp",
        "provider-inputs",
        "source-sha",
        "exports-digest",
        "toolchain",
    ],
    ref_kinds: &["provenance"],
    storage: "content-addressed blobs (facts/plan JSON, fragment text, library fileset)",
    residency: "hot after first build; reclaimed by GC when unrooted",
    regeneration: Regeneration::FromUpstreamRecipe,
    coverage: LayerCoverage::ProbeSeam,
};

/// The frozen MSVC C11 flags of the bridge's CRT-closed legs (the A2C
/// channel's toolchain closure — full CRT via /MT, stdio-capable
/// runtime; the freestanding probe flags of `ctool` are NOT reused
/// mechanically). A drift test on the a2x-worker side pins these to
/// `crun::C_FLAGS`.
pub const BRIDGE_CRT_FLAGS: [&str; 7] = ["-nologo", "-std:c11", "-TC", "-W4", "-WX", "-O2", "-MT"];

/// Budgets of the bridge legs (inherited from the 797 frozen policy).
pub const BRIDGE_BUILD_DEADLINE: Duration = Duration::from_secs(60);
pub const BRIDGE_RUN_DEADLINE: Duration = Duration::from_secs(5);
pub const BRIDGE_TOTAL_DEADLINE: Duration = Duration::from_secs(70);
/// Stream caps per run (1 MiB, as frozen).
pub const BRIDGE_STREAM_CAP: usize = 1 << 20;

#[derive(Debug)]
pub struct BridgeError(pub String);

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bridge: {}", self.0)
    }
}

impl std::error::Error for BridgeError {}

impl From<String> for BridgeError {
    fn from(e: String) -> Self {
        BridgeError(e)
    }
}

impl From<StoreError> for BridgeError {
    fn from(e: StoreError) -> Self {
        BridgeError(format!("store: {e:?}"))
    }
}

/// Named pre-call admission rejection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionRejection {
    pub stage: String,
    pub code: String,
    pub message: String,
}

impl AdmissionRejection {
    fn of(r: &Rejection) -> AdmissionRejection {
        let dbg = format!("{r:?}");
        AdmissionRejection {
            stage: "admission".into(),
            code: dbg
                .split(" {")
                .next()
                .unwrap_or("Rejection")
                .split('\n')
                .next()
                .unwrap_or("Rejection")
                .to_string(),
            message: dbg,
        }
    }
}

// ---------------------------------------------------------------------------
// Producer / IO counters (per key, observable by tests and receipts)
// ---------------------------------------------------------------------------

/// Per-key producer-call and payload-read counters. Producers bump
/// `producer_calls` on EVERY invocation (misses); hits never run the
/// producer. `store_gets` counts read operations issued by restore.
#[derive(Clone, Default, Debug)]
pub struct BridgeCounters {
    inner: Arc<BridgeCountersInner>,
}

#[derive(Default, Debug)]
struct BridgeCountersInner {
    producer_calls: Mutex<HashMap<String, u64>>,
    store_gets: Mutex<HashMap<String, u64>>,
    bytes_read: AtomicU64,
    bytes_written: AtomicU64,
}

impl BridgeCounters {
    pub fn new() -> BridgeCounters {
        BridgeCounters::default()
    }

    fn bump_producer(&self, key: &str) {
        *self
            .inner
            .producer_calls
            .lock()
            .unwrap()
            .entry(key.to_string())
            .or_insert(0) += 1;
    }

    pub fn producer_calls(&self, key: &str) -> u64 {
        self.inner
            .producer_calls
            .lock()
            .unwrap()
            .get(key)
            .copied()
            .unwrap_or(0)
    }

    /// Total producer invocations across all keys.
    pub fn total_producer_calls(&self) -> u64 {
        self.inner.producer_calls.lock().unwrap().values().sum()
    }

    fn bump_get(&self, key: &str) {
        *self
            .inner
            .store_gets
            .lock()
            .unwrap()
            .entry(key.to_string())
            .or_insert(0) += 1;
    }

    pub fn store_gets(&self, key: &str) -> u64 {
        self.inner
            .store_gets
            .lock()
            .unwrap()
            .get(key)
            .copied()
            .unwrap_or(0)
    }

    pub fn add_bytes_read(&self, n: u64) {
        self.inner.bytes_read.fetch_add(n, Ordering::Relaxed);
    }

    pub fn add_bytes_written(&self, n: u64) {
        self.inner.bytes_written.fetch_add(n, Ordering::Relaxed);
    }

    pub fn bytes_read(&self) -> u64 {
        self.inner.bytes_read.load(Ordering::Relaxed)
    }

    pub fn bytes_written(&self) -> u64 {
        self.inner.bytes_written.load(Ordering::Relaxed)
    }
}

/// Produce through the BRIDGE layer with counters: hit → payload, no
/// producer call; miss → producer (bumps its key) → persisted.
fn produce_counted<F>(
    store: &ManagedStore,
    counters: &BridgeCounters,
    key: &QueryKey,
    kind: ChunkKind,
    produce: F,
) -> Result<(Vec<u8>, bool), BridgeError>
where
    F: FnOnce() -> Result<Vec<u8>, String>,
{
    if let Some(LoadedArtifact { payload, .. }) = store.get_artifact(LAYER_BRIDGE_V01.id, key)? {
        counters.add_bytes_read(payload.len() as u64);
        return Ok((payload, true));
    }
    let canonical = key.canonical();
    counters.bump_producer(&canonical);
    let payload = produce().map_err(|e| BridgeError(format!("produce {canonical}: {e}")))?;
    // single-artifact budget (64 MiB, the frozen probe policy)
    if payload.len() > 64 * 1024 * 1024 {
        return Err(BridgeError(format!(
            "producer emitted {} bytes (>64MiB cap)",
            payload.len()
        )));
    }
    store.put_artifact(PutRequest {
        layer: LAYER_BRIDGE_V01.id,
        key: key.clone(),
        kind,
        payload: payload.clone(),
        edges: vec![],
        project: None,
    })?;
    counters.add_bytes_written(payload.len() as u64);
    Ok((payload, false))
}

/// Test/verification seam for the counted producer (same path the
/// chains use; exposed so resource-cap tests exercise the REAL gate).
#[doc(hidden)]
pub fn produce_counted_for_test<F>(
    store: &ManagedStore,
    counters: &BridgeCounters,
    key: &QueryKey,
    kind: ChunkKind,
    produce: F,
) -> Result<(Vec<u8>, bool), BridgeError>
where
    F: FnOnce() -> Result<Vec<u8>, String>,
{
    produce_counted(store, counters, key, kind, produce)
}

// ---------------------------------------------------------------------------
// Keys (the §5.4 key structure: logical / physical / adapter / object)
// ---------------------------------------------------------------------------

/// Logical key: interface facts (identity + signature + source).
pub fn facts_key(facts_digest: &str, source_identity: &str) -> QueryKey {
    QueryKey::new(
        &LAYER_BRIDGE_V01,
        vec![
            ("kind".into(), "bridge-facts".into()),
            ("facts-digest".into(), facts_digest.into()),
            ("source".into(), source_identity.into()),
        ],
    )
}

/// Physical key: the projected plan, keyed by the PLAN's OWN digest
/// (a pure function of the forms) + domain — an interface-preserving
/// body edit leaves the forms (and thus this key) untouched: the plan
/// page is a pure hit (plan §5.4 "不读body的caller metadata P=0").
pub fn plan_key(plan_digest: &str) -> QueryKey {
    QueryKey::new(
        &LAYER_BRIDGE_V01,
        vec![
            ("kind".into(), "bridge-plan".into()),
            ("plan-digest".into(), plan_digest.into()),
            ("domain".into(), ace_foreign_contract::plan::DOMAIN.into()),
            (
                "domain-version".into(),
                ace_foreign_contract::plan::DOMAIN_VERSION.into(),
            ),
            ("target".into(), ace_foreign_contract::plan::TARGET.into()),
            ("profile".into(), ace_foreign_contract::plan::PROFILE.into()),
        ],
    )
}

/// Adapter key: ONE source function's C fragment (per-function increment
/// unit). Mixes the semantic fingerprint (which CARRIES the function's
/// own body identity) + provider identity — deliberately NOT the whole
/// source hash: an interface-preserving body edit is a NEW sem_fp for
/// exactly the edited function, and every OTHER function's key (its
/// sem_fp unchanged) stays a pure hit (plan §5.4).
pub fn fragment_key(
    source_fn: &str,
    sem_fp: &str,
    provider_inputs: &[(String, String)],
) -> QueryKey {
    let _ = ""; // source identity is NOT a fragment-key input (see doc)
    let mut components = vec![
        ("kind".to_string(), "bridge-c-fragment".to_string()),
        ("fn".to_string(), source_fn.to_string()),
        ("sem-fp".to_string(), sem_fp.to_string()),
    ];
    for (k, v) in provider_inputs {
        components.push((k.clone(), v.clone()));
    }
    QueryKey::new(&LAYER_BRIDGE_V01, components)
}

/// Adapter key: the assembled LIBRARY fileset (what the adapter actually
/// reads: fragment digests + runtime digest + export specs + schema).
pub fn library_fileset_key(
    fragment_digests: &str,
    runtime_sha256: &str,
    exports_digest: &str,
) -> QueryKey {
    QueryKey::new(
        &LAYER_BRIDGE_V01,
        vec![
            ("kind".into(), "bridge-library-fileset".into()),
            ("fragments".into(), fragment_digests.into()),
            ("runtime".into(), runtime_sha256.into()),
            ("exports".into(), exports_digest.into()),
            ("schema".into(), ace_a2x_c::LIBRARY_FILESET_SCHEMA.into()),
        ],
    )
}

/// Object key: the typed Cranelift callee (identity = plan + emitter).
pub fn typed_callee_key(plan_digest: &str) -> QueryKey {
    QueryKey::new(
        &LAYER_BRIDGE_V01,
        vec![
            ("kind".into(), "bridge-typed-callee".into()),
            ("plan-digest".into(), plan_digest.into()),
            ("toolchain".into(), crate::cache::emitter_identity()),
            ("target".into(), layout::TARGET.into()),
        ],
    )
}

// ---------------------------------------------------------------------------
// Semantic identity through the REAL contract
// ---------------------------------------------------------------------------

pub const BRIDGE_PACKAGE: &str = "probe937";
pub const BRIDGE_MODULE: &str = "bridge";

fn lane_typedesc(k: LaneKind) -> TypeDesc {
    match k {
        LaneKind::I32 => TypeDesc::I32,
        LaneKind::U32 => TypeDesc::U32,
        LaneKind::I64 => TypeDesc::I64,
        LaneKind::U64 => TypeDesc::U64,
        LaneKind::F64 => TypeDesc::F64,
        LaneKind::Bool => TypeDesc::Bool,
        LaneKind::Char => TypeDesc::Char,
    }
}

/// Semantic binding hash over an interface's functions (nominal logicals
/// + content versions minted through the real contract; same digest
/// shape as the 797 semref binding).
pub fn interface_binding_hash(facts: &InterfaceFacts) -> Result<String, BridgeError> {
    let package = PackageId::new(BRIDGE_PACKAGE);
    let mut buf = Vec::new();
    for f in &facts.functions {
        let (params, ret) = match &f.form {
            FnForm::Pure { params, ret } => (params.clone(), Some(*ret)),
            FnForm::Fallible {
                ctx: _,
                params,
                outs,
            } => {
                // status form: params + out lanes are the semantic shape;
                // the i32 status is the declared result.
                let mut ps = params.clone();
                ps.extend(outs.iter().copied());
                (ps, Some(LaneKind::I32))
            }
        };
        let td = TypeDesc::Fn {
            params: params.iter().map(|k| lane_typedesc(*k)).collect(),
            ret: Box::new(ret.map(lane_typedesc).unwrap_or(TypeDesc::Unit)),
        };
        let canonical =
            encode_typedesc_bytes(&td).map_err(|e| BridgeError(format!("codec: {e}")))?;
        let decl = DeclRef::new(
            &package,
            BRIDGE_MODULE,
            DeclKind::Fn,
            &f.symbol,
            ContentVersion::of_canonical(&canonical),
        );
        decl.verify()
            .map_err(|e| BridgeError(format!("decl verify: {e:?}")))?;
        buf.extend_from_slice(decl.logical.as_str().as_bytes());
        buf.extend_from_slice(decl.content.as_str().as_bytes());
    }
    Ok(crate::semref::blake3_hex(&buf))
}

// ---------------------------------------------------------------------------
// Descriptor/3 projection + independent expectations
// ---------------------------------------------------------------------------

fn param_kind_of(lane: PhysLane) -> descriptor::ParamKind {
    match lane {
        PhysLane::I32 => descriptor::ParamKind::I32,
        PhysLane::U32 => descriptor::ParamKind::U32,
        PhysLane::I64 => descriptor::ParamKind::I64,
        PhysLane::U64 => descriptor::ParamKind::U64,
        PhysLane::F64 => descriptor::ParamKind::F64,
        PhysLane::BoolU8 => descriptor::ParamKind::Bool,
        PhysLane::CharU32 => descriptor::ParamKind::Char,
    }
}

fn ret_kind_of(r: PhysRet) -> descriptor::RetKind {
    match r {
        PhysRet::RaxI32 => descriptor::RetKind::I32,
        PhysRet::RaxU32 => descriptor::RetKind::U32,
        PhysRet::RaxI64 => descriptor::RetKind::I64,
        PhysRet::RaxU64 => descriptor::RetKind::U64,
        PhysRet::Xmm0F64 => descriptor::RetKind::F64,
        PhysRet::RaxBoolU8 => descriptor::RetKind::Bool,
        PhysRet::RaxCharU32 => descriptor::RetKind::Char,
        PhysRet::Void => descriptor::RetKind::Unit,
    }
}

/// Project the neutral plan into a descriptor/3 module (SAME wire
/// language the 797 consumers speak; descriptor semantics own the ctx
/// lane and out slots, so the physical param list's ctx/out pointer
/// lanes map to the FORM fields, not to params).
pub fn descriptor_from_plan(
    facts: &InterfaceFacts,
    plan: &PhysicalPlan,
) -> Result<descriptor::ModuleDescriptor, BridgeError> {
    let semref_hash = interface_binding_hash(facts)?;
    let symbols = plan
        .functions
        .iter()
        .map(|pf| {
            let form = match &pf.form {
                PhysForm::Pure { params, ret } => descriptor::SymbolForm::Pure {
                    params: params.iter().map(|p| param_kind_of(p.lane)).collect(),
                    ret: ret_kind_of(*ret),
                },
                PhysForm::Fallible { ctx, params, outs } => descriptor::SymbolForm::Fallible {
                    ctx: *ctx,
                    // declared params only: the ctx lane and the trailing
                    // out-pointer lanes are FORM fields in descriptor/3.
                    params: params
                        .iter()
                        .filter(|p| {
                            p.name != "ctx"
                                && !(p.name.starts_with("out") && p.name.ends_with("_ptr"))
                        })
                        .map(|p| param_kind_of(p.lane))
                        .collect(),
                    outs: outs.iter().map(|l| param_kind_of(*l)).collect(),
                },
            };
            descriptor::SymbolEntry {
                name: pf.symbol.clone(),
                form,
            }
        })
        .collect();
    Ok(descriptor::ModuleDescriptor {
        format: descriptor::DESCRIPTOR_FORMAT.into(),
        abi_domain: descriptor::ABI_DOMAIN.into(),
        abi_version: descriptor::ABI_VERSION.into(),
        target: plan.target.clone(),
        profile: plan.profile.clone(),
        packing: plan.packing.clone(),
        interface_revision: 1,
        semref_hash,
        semantic_refs: vec![],
        records: vec![],
        symbols,
        runtime: descriptor::RuntimeReqs { services: vec![] },
    })
}

/// The hand-frozen consumer expectation of the TYPED interface — spelled
/// from the independent cside header (`cside/bridge_typed.h`), never
/// from the produced descriptor.
pub fn typed_expectation(semref_hash: String) -> ConsumerExpectation {
    ConsumerExpectation {
        abi_domain: descriptor::ABI_DOMAIN.into(),
        abi_version: descriptor::ABI_VERSION.into(),
        target: layout::TARGET.into(),
        profile: layout::ABI_PROFILE.into(),
        packing: descriptor::PACKING.into(),
        required_symbols: vec![
            (
                "t937_mix".into(),
                "pure(i32,u32,i64,u64,f64,bool,char,i64)->i64".into(),
            ),
            (
                "t937_div".into(),
                "fallible(ctx:nullable,i64,i64)->status,out i64".into(),
            ),
        ],
        required_records: vec![],
        offered_services: vec![],
        semref_hash,
        semantic_refs: vec![],
    }
}

/// The hand-frozen consumer expectation of the SOURCE export interface
/// (`cside/bridge_source.h` spelling: the fallible public form with the
/// value + reason out lanes).
pub fn source_expectation(
    exports: &[ace_a2x_c::abi_scalar::ExportSpec],
    semref_hash: String,
) -> ConsumerExpectation {
    let mut required = Vec::new();
    for s in exports {
        let params = s
            .params
            .iter()
            .map(|p| p.spell())
            .collect::<Vec<_>>()
            .join(",");
        required.push((
            s.public_symbol.clone(),
            format!(
                "fallible(ctx:nullable,{params})->status,out {},i32",
                s.ret.spell()
            ),
        ));
    }
    ConsumerExpectation {
        abi_domain: descriptor::ABI_DOMAIN.into(),
        abi_version: descriptor::ABI_VERSION.into(),
        target: layout::TARGET.into(),
        profile: layout::ABI_PROFILE.into(),
        packing: descriptor::PACKING.into(),
        required_symbols: required,
        required_records: vec![],
        offered_services: vec![],
        semref_hash,
        semantic_refs: vec![],
    }
}

// ---------------------------------------------------------------------------
// Scratch + compile helpers
// ---------------------------------------------------------------------------

/// Namespaced scratch dir (caller-supplied root + pid + step —
/// concurrent bridge runs never collide).
pub fn bridge_scratch(root: &Path, step: &str) -> PathBuf {
    root.join(format!("bridge-937-{}-{step}", std::process::id()))
}

fn write_all(dir: &Path, files: &[(&str, &str)]) -> Result<(), BridgeError> {
    std::fs::create_dir_all(dir).map_err(|e| BridgeError(format!("mkdir: {e}")))?;
    for (name, text) in files {
        std::fs::write(dir.join(name), text)
            .map_err(|e| BridgeError(format!("write {name}: {e}")))?;
    }
    Ok(())
}

/// Compile+link the CRT-closed bridge C leg under the contained
/// executor (cl via the frozen locator; vcvars64 environment derived
/// exactly like the A2C channel; `/MT` static CRT closure — see
/// `BRIDGE_CRT_FLAGS`).
pub fn compile_bridge_crt(dir: &Path, sources: &[&str], exe: &Path) -> Result<(), BridgeError> {
    let tool = ctool::find_cl().map_err(|e| BridgeError(format!("cl locator: {e:?}")))?;
    let vcvars = match std::env::var("A2C_VCVARS64") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            let vc = tool
                .cl_exe
                .ancestors()
                .nth(7)
                .ok_or_else(|| BridgeError("cannot derive VC dir".into()))?
                .to_path_buf();
            let p = vc.join("Auxiliary").join("Build").join("vcvars64.bat");
            if !p.is_file() {
                return Err(BridgeError(format!(
                    "vcvars64.bat missing: {}",
                    p.display()
                )));
            }
            p
        }
    };
    let mut line = String::new();
    line.push_str(&format!("call \"{}\" >nul && ", vcvars.display()));
    line.push_str(&format!("\"{}\"", tool.cl_exe.display()));
    for f in BRIDGE_CRT_FLAGS {
        line.push(' ');
        line.push_str(f);
    }
    line.push_str(&format!(" -Fe:{}", exe.display()));
    for s in sources {
        line.push(' ');
        line.push_str(s);
    }
    let mut cmd = Command::new("cmd.exe");
    cmd.arg("/D").arg("/C");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.raw_arg(&line);
    }
    #[cfg(not(windows))]
    {
        cmd.arg(&line);
    }
    cmd.current_dir(dir);
    let run = run_deadlined_captured(
        &mut cmd,
        BRIDGE_BUILD_DEADLINE,
        CaptureCaps {
            stdout_max: BRIDGE_STREAM_CAP,
            stderr_max: BRIDGE_STREAM_CAP,
        },
    )
    .map_err(|e| BridgeError(format!("contained cl spawn: {e:?}")))?;
    if run.exit_code != Some(0) {
        return Err(BridgeError(format!(
            "cl exited {:?}:\n{}\n{}",
            run.exit_code,
            String::from_utf8_lossy(&run.stdout).trim(),
            String::from_utf8_lossy(&run.stderr).trim()
        )));
    }
    if !exe.is_file() {
        return Err(BridgeError("cl reported success but exe missing".into()));
    }
    Ok(())
}

/// The cside dir (headers + hand-written C legs).
pub fn cside_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("cside")
}

/// Compile one cside C source through the applicability-cached cl path
/// (freestanding probe closure — exactly the 797 leg-b/c discipline).
fn produce_c_object_cached(
    store: &ManagedStore,
    counters: &BridgeCounters,
    src: &Path,
    obj_out: &Path,
) -> Result<(), BridgeError> {
    let key_tag = format!(
        "cl:{}",
        src.file_name().unwrap_or_default().to_string_lossy()
    );
    let include = cside_dir();
    let obj_path = obj_out.to_path_buf();
    let (bytes, _hit) = crate::cache::produce_c_object(store, src, &include, &[], || {
        counters.bump_producer(&key_tag);
        let deps = ctool::compile_c_with_deps(src, &include, &obj_path)
            .map_err(|e| format!("cl: {e:?}"))?;
        let bytes = std::fs::read(&obj_path).map_err(|e| format!("read obj: {e}"))?;
        Ok((bytes, deps))
    })
    .map_err(|e| BridgeError(format!("c object: {e:?}")))?;
    // ALWAYS materialize the object bytes at obj_out — a store HIT does
    // not write the file, but the link step needs it on disk.
    std::fs::write(obj_out, bytes).map_err(|e| BridgeError(e.to_string()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Typed peer chain (V-01 / V-03 / V-04)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypedLeg {
    /// Emitted caller + emitted callee (baseline self-check).
    SelfCheck,
    /// Emitted caller + C callee (native -> C typed).
    NativeToC,
    /// C caller + emitted callee (C -> native typed).
    CToNative,
}

#[derive(Debug)]
pub struct TypedChainOutcome {
    pub facts_digest: String,
    pub plan_digest: String,
    pub descriptor_fingerprint: String,
    /// (leg name, PE exit code) — 0 expected on every leg.
    pub legs: Vec<(String, i32)>,
}

/// Run the typed chain: facts → plan → descriptor → PRE-CALL admission →
/// produce objects → link+run every leg. Every PE exits with its FAILED
/// check count (0 expected).
pub fn run_typed_chain(
    store: &ManagedStore,
    counters: &BridgeCounters,
    scratch_root: &Path,
    legs: &[TypedLeg],
    total_budget: Duration,
) -> Result<TypedChainOutcome, BridgeError> {
    let started = Instant::now();
    let budget_left = || {
        total_budget
            .checked_sub(started.elapsed())
            .ok_or_else(|| BridgeError("total budget exhausted".into()))
    };

    // 1) facts + plan (deterministic functions of the frozen decls).
    let decls: TypedDecls = typed_decl::typed_decls();
    let facts = decls
        .validate()
        .map_err(|e| BridgeError(format!("typed decls: {e}")))?;
    let plan =
        PhysicalPlan::project(&facts).map_err(|e| BridgeError(format!("plan projection: {e}")))?;
    let facts_digest = facts.digest().map_err(BridgeError)?;
    let plan_digest = plan.digest().map_err(BridgeError)?;

    // 2) persist the facts/plan pages (identity root of the restore
    //    path) BEFORE any object production.
    let source_identity = match &facts.source {
        FactsSource::Typed { id } => format!("typed/{id}"),
        FactsSource::CheckedSource { source_sha256, .. } => source_sha256.clone(),
    };
    let (facts_page, _) = produce_counted(
        store,
        counters,
        &facts_key(&facts_digest, &source_identity),
        ChunkKind::LayoutData,
        || facts_codec::encode(&facts).map_err(|e| e.to_string()),
    )?;
    let facts_bytes = facts_page;
    facts_codec::decode(&facts_bytes).map_err(|e| BridgeError(e.to_string()))?; // self-check
    let plan_digest_value = plan.digest().map_err(BridgeError)?;
    let (_, _) = produce_counted(
        store,
        counters,
        &plan_key(&plan_digest_value),
        ChunkKind::LayoutData,
        || plan_codec::encode(&plan).map_err(|e| e.to_string()),
    )?;

    // 3) descriptor projection + PRE-CALL admission (independent
    //    expectation; a rejection happens BEFORE any callee production).
    let semref = interface_binding_hash(&facts)?;
    let descriptor = descriptor_from_plan(&facts, &plan)?;
    let fingerprint = descriptor
        .fingerprint()
        .map_err(|e| BridgeError(e.to_string()))?;
    let expect = typed_expectation(semref);
    if let Err(rej) = admit_descriptor(&descriptor, &expect) {
        return Err(BridgeError(format!(
            "pre-call admission rejected: {:?}",
            AdmissionRejection::of(&rej)
        )));
    }

    // 3) typed callee object (Cranelift) — cached under the plan key.
    let callee_key = typed_callee_key(&plan_digest);
    let (callee_obj, _) = produce_counted(store, counters, &callee_key, ChunkKind::Object, || {
        emit_typed_callee().map_err(|e| e)
    })?;

    let mut out_legs = Vec::new();
    let scratch = bridge_scratch(scratch_root, "typed");
    let _ = std::fs::remove_dir_all(&scratch);
    for leg in legs {
        budget_left()?;
        match leg {
            TypedLeg::SelfCheck => {
                let dir = scratch.join("self");
                write_all(&dir, &[])?;
                std::fs::write(
                    dir.join("caller.obj"),
                    emit_typed_caller(&frozen_typed_checks())?,
                )
                .map_err(|e| BridgeError(e.to_string()))?;
                std::fs::write(dir.join("callee.obj"), &callee_obj)
                    .map_err(|e| BridgeError(e.to_string()))?;
                let exe = link_pe(
                    &dir,
                    &[dir.join("caller.obj"), dir.join("callee.obj")],
                    "t937_entry",
                )
                .map_err(|e| BridgeError(format!("{e:?}")))?;
                let run = run_pe(&exe).map_err(|e| BridgeError(format!("{e:?}")))?;
                out_legs.push(("self-check".into(), run.code));
            }
            TypedLeg::NativeToC => {
                let dir = scratch.join("n2c");
                write_all(&dir, &[])?;
                let c_obj = dir.join("bridge_typed_callee.obj");
                produce_c_object_cached(
                    store,
                    counters,
                    &cside_dir().join("bridge_typed_callee.c"),
                    &c_obj,
                )?;
                std::fs::write(
                    dir.join("caller.obj"),
                    emit_typed_caller(&frozen_typed_checks())?,
                )
                .map_err(|e| BridgeError(e.to_string()))?;
                let exe = link_pe(&dir, &[dir.join("caller.obj"), c_obj], "t937_entry")
                    .map_err(|e| BridgeError(format!("{e:?}")))?;
                let run = run_pe(&exe).map_err(|e| BridgeError(format!("{e:?}")))?;
                out_legs.push(("native-to-c".into(), run.code));
            }
            TypedLeg::CToNative => {
                let dir = scratch.join("c2n");
                write_all(&dir, &[])?;
                let c_obj = dir.join("bridge_typed_caller.obj");
                produce_c_object_cached(
                    store,
                    counters,
                    &cside_dir().join("bridge_typed_caller.c"),
                    &c_obj,
                )?;
                std::fs::write(dir.join("callee.obj"), &callee_obj)
                    .map_err(|e| BridgeError(e.to_string()))?;
                let exe = link_pe(&dir, &[c_obj, dir.join("callee.obj")], "t937_c_entry")
                    .map_err(|e| BridgeError(format!("{e:?}")))?;
                let run = run_pe(&exe).map_err(|e| BridgeError(format!("{e:?}")))?;
                out_legs.push(("c-to-native".into(), run.code));
            }
        }
    }
    Ok(TypedChainOutcome {
        facts_digest,
        plan_digest,
        descriptor_fingerprint: fingerprint,
        legs: out_legs,
    })
}

/// Run the typed chain with a caller check-set mutation (the negative
/// control hook: an intentionally wrong expected value must FAIL the PE).
pub fn run_typed_caller_with_checks(
    store: &ManagedStore,
    counters: &BridgeCounters,
    scratch_root: &Path,
    checks: &[TypedCheck],
    entry_callee: bool,
) -> Result<i32, BridgeError> {
    let dir = bridge_scratch(scratch_root, "typed-mut");
    let _ = std::fs::remove_dir_all(&dir);
    write_all(&dir, &[])?;
    std::fs::write(dir.join("caller.obj"), emit_typed_caller(checks)?)
        .map_err(|e| BridgeError(e.to_string()))?;
    // callee from the store (same identity as the chain)
    let decls: TypedDecls = typed_decl::typed_decls();
    let facts = decls.validate().map_err(|e| BridgeError(e))?;
    let plan = PhysicalPlan::project(&facts).map_err(|e| BridgeError(e))?;
    let plan_digest = plan.digest().map_err(BridgeError)?;
    let (callee_obj, _) = produce_counted(
        store,
        counters,
        &typed_callee_key(&plan_digest),
        ChunkKind::Object,
        emit_typed_callee,
    )?;
    std::fs::write(dir.join("callee.obj"), &callee_obj).map_err(|e| BridgeError(e.to_string()))?;
    let exe = link_pe(
        &dir,
        &[dir.join("caller.obj"), dir.join("callee.obj")],
        "t937_entry",
    )
    .map_err(|e| BridgeError(format!("{e:?}")))?;
    let _ = entry_callee;
    let run = run_pe(&exe).map_err(|e| BridgeError(format!("{e:?}")))?;
    Ok(run.code)
}

// ---------------------------------------------------------------------------
// Source export chain (V-02 / V-05 / V-06)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct SourceExportRequest {
    /// Real Auto source text (compiled by the checked frontdoor — never
    /// a fixture-name body).
    pub src: String,
    pub module: String,
    /// (source function name, public symbol) pairs to export.
    pub exports: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct SourceExportOutcome {
    pub facts_digest: String,
    pub library_manifest: String,
    pub caller_exit: i32,
    pub stdout: String,
    /// The restorable handle (bridge artifact identity).
    pub handle: BridgeHandle,
}

/// The frozen first-case source (V-02): `sum(20,22)=42`, `div` with the
/// controlled zero-divisor failure; a body+1 edit flips the result.
pub fn source_export_first_case() -> SourceExportRequest {
    SourceExportRequest {
        src: "fn sum(a int, b int) int { return a + b }\nfn div(n int, d int) int { return n / d }\nfn main() int { return 0 }\n".into(),
        module: "t937_source".into(),
        exports: vec![
            ("sum".into(), "t937_sum".into()),
            ("div".into(), "t937_div".into()),
        ],
    }
}

/// The +1 body edit family (V-02 distinguishing oracle: the DELIVERED
/// value must flip when the real body changes).
pub fn source_export_body_edit() -> String {
    "fn sum(a int, b int) int { return a + b + 1 }\nfn div(n int, d int) int { return n / d }\nfn main() int { return 0 }\n".into()
}

use ace_a2x_c::abi_scalar::{assemble_library_fileset, ExportSpec, LibraryFilesetInputs};

/// Map the source channel's compute types to public lanes.
fn lane_of_ty(ty: &ace_a2x_core::Ty) -> Result<LaneKind, BridgeError> {
    match ty {
        ace_a2x_core::Ty::I32 => Ok(LaneKind::I32),
        ace_a2x_core::Ty::Bool => Ok(LaneKind::Bool),
        other => Err(BridgeError(format!(
            "source export serves i32/bool only, not {other:?}"
        ))),
    }
}

fn sha256_hex(text: &str) -> String {
    ace_a2x_core::sha256::sha256_hex(text.as_bytes())
}

/// Run the source export chain: frontdoor → per-function fragments
/// (cached) → library fileset (cached) → admission → CRT-closed compile
/// → the independent C caller EXECUTES the exports.
#[allow(clippy::too_many_lines)]
pub fn run_source_export(
    store: &ManagedStore,
    counters: &BridgeCounters,
    scratch_root: &Path,
    req: &SourceExportRequest,
    total_budget: Duration,
) -> Result<SourceExportOutcome, BridgeError> {
    let started = Instant::now();
    let budget_left = || {
        total_budget
            .checked_sub(started.elapsed())
            .ok_or_else(|| BridgeError("total budget exhausted".into()))
    };

    use ace_a2x_core::target::{Family, TargetProvider};
    use ace_a2x_core::{semantic_fingerprint, SemanticView};

    // 1) REAL checked source through the unified frontdoor.
    let compiled = ace_core::source::compile_scalar_source(&req.src, &req.module, &[])
        .map_err(|e| BridgeError(format!("frontdoor: {e:?}")))?;
    let view = SemanticView::from_checked(&compiled.checked)
        .map_err(|e| BridgeError(format!("semantic view: {e:?}")))?;
    let source_sha = sha256_hex(&req.src);

    // 2) Resolve the exported functions against the REAL view.
    let provider = ace_a2x_c::CTargetProvider::v1();
    let provider_inputs = provider.fragment_key_inputs("");
    let mut specs = Vec::new();
    let mut exported = Vec::new();
    for (src_name, public_symbol) in &req.exports {
        let f = view
            .functions()
            .iter()
            .find(|f| f.source_name == *src_name)
            .ok_or_else(|| {
                BridgeError(format!(
                    "export '{src_name}' not found in the checked source"
                ))
            })?;
        let params: Vec<LaneKind> = f
            .params
            .iter()
            .map(|p| lane_of_ty(&p.ty))
            .collect::<Result<_, _>>()?;
        let ret = lane_of_ty(&f.result)?;
        exported.push(f);
        specs.push(ExportSpec {
            public_symbol: public_symbol.clone(),
            source_fn: ace_a2x_c::c_fn_name(src_name),
            params,
            ret,
        });
    }

    // 3) Interface facts from the REAL checked source (every source
    //    export is FALLIBLE — checked arithmetic; the wrapper's explicit
    //    trap table is the error model, the reason lane keeps the cause).
    let functions: Vec<FnFact> = specs
        .iter()
        .map(|s| FnFact {
            symbol: s.public_symbol.clone(),
            form: FnForm::Fallible {
                ctx: Some(true),
                params: s.params.clone(),
                outs: vec![s.ret, LaneKind::I32], // value + reason lanes
            },
            error_model: Some(ErrorModel {
                kind: "status-i32".into(),
                codes: ace_a2x_c::abi_scalar::public_error_model_rows(),
            }),
            semantic_ref: None,
        })
        .collect();
    let facts = InterfaceFacts {
        format: ace_foreign_contract::FACTS_FORMAT.into(),
        logical: format!("source/{}", req.module),
        content_version: crate::semref::blake3_hex(source_sha.as_bytes()),
        source: FactsSource::CheckedSource {
            module: req.module.clone(),
            source_sha256: source_sha.clone(),
        },
        functions,
        effects: vec![],
        capabilities: vec![],
    };
    facts
        .validate()
        .map_err(|e| BridgeError(format!("source facts: {e}")))?;
    let facts_digest = facts.digest().map_err(BridgeError)?;

    // 3b) persist the facts/plan pages (restore identity root).
    let (_, _) = produce_counted(
        store,
        counters,
        &facts_key(&facts_digest, &source_sha),
        ChunkKind::LayoutData,
        || facts_codec::encode(&facts).map_err(|e| e.to_string()),
    )?;
    let plan =
        PhysicalPlan::project(&facts).map_err(|e| BridgeError(format!("plan projection: {e}")))?;
    let plan_digest_value = plan.digest().map_err(BridgeError)?;
    let plan_key_canonical = plan_key(&plan_digest_value).canonical();
    let (_, _) = produce_counted(
        store,
        counters,
        &plan_key(&plan_digest_value),
        ChunkKind::LayoutData,
        || plan_codec::encode(&plan).map_err(|e| e.to_string()),
    )?;

    // 4) Per-function C fragments through the store (per-function P0:
    //    unchanged function -> hit; edited body -> exactly one miss).
    let runtime = provider
        .runtime_source(&view, Family::Scalar)
        .map_err(|e| BridgeError(format!("runtime: {e:?}")))?;
    let runtime_sha = sha256_hex(&runtime);
    budget_left()?;
    let mut fragments: Vec<String> = Vec::new();
    let mut fragment_key_canonicals: Vec<String> = Vec::new();
    for f in &exported {
        let sem_fp =
            semantic_fingerprint(&view, f).map_err(|e| BridgeError(format!("sem fp: {e:?}")))?;
        let key = fragment_key(&f.source_name, &sem_fp, &provider_inputs);
        fragment_key_canonicals.push(key.canonical());
        let (bytes, _) = produce_counted(store, counters, &key, ChunkKind::SourcePage, || {
            let texts = provider
                .produce_fragments(&view, &[f], Family::Scalar)
                .map_err(|e| format!("fragment: {e:?}"))?;
            Ok(texts[0].clone().into_bytes())
        })?;
        fragments.push(String::from_utf8(bytes).map_err(|e| BridgeError(format!("utf8: {e}")))?);
    }

    // 5) The LIBRARY fileset (explicit assembly — no process main; the
    //    old exe form is untouched by construction).
    let frag_digest = crate::semref::blake3_hex(fragments.join("\n").as_bytes());
    let exports_digest = crate::semref::blake3_hex(
        specs
            .iter()
            .map(|s| format!("{}={}", s.public_symbol, s.source_fn))
            .collect::<Vec<_>>()
            .join(",")
            .as_bytes(),
    );
    let lib_key = library_fileset_key(&frag_digest, &runtime_sha, &exports_digest);
    let inputs = LibraryFilesetInputs {
        entry: "main".into(),
        input_hash: compiled.input_hash.clone(),
        module: req.module.clone(),
    };
    budget_left()?;
    let (lib_json, _) = produce_counted(store, counters, &lib_key, ChunkKind::LayoutData, || {
        let fs = assemble_library_fileset(
            &fragments.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            &runtime,
            &specs,
            &inputs,
        )
        .map_err(|e| e)?;
        serde_json::to_vec(&serde_json::json!({
            "files": fs.files,
            "manifest": fs.manifest,
        }))
        .map_err(|e| e.to_string())
    })?;

    // 6) PRE-CALL admission (independent expectation: the caller's own
    //    header spelling).
    let semref = interface_binding_hash(&facts)?;
    let descriptor = descriptor_from_plan(&facts, &plan)?;
    let expect = source_expectation(&specs, semref);
    if let Err(rej) = admit_descriptor(&descriptor, &expect) {
        return Err(BridgeError(format!(
            "pre-call admission rejected: {:?}",
            AdmissionRejection::of(&rej)
        )));
    }

    // 7) Materialize + compile (CRT closure) + RUN the independent
    //    caller against the real library.
    budget_left()?;
    let lib: serde_json::Value =
        serde_json::from_slice(&lib_json).map_err(|e| BridgeError(format!("lib json: {e}")))?;
    let files: Vec<(String, String)> = lib["files"]
        .as_array()
        .ok_or_else(|| BridgeError("files missing".into()))?
        .iter()
        .map(|v| {
            let arr = v.as_array().unwrap();
            (
                arr[0].as_str().unwrap().to_string(),
                arr[1].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let manifest = lib["manifest"].as_str().unwrap_or_default().to_string();

    let dir = bridge_scratch(scratch_root, "source-export");
    let _ = std::fs::remove_dir_all(&dir);
    write_all(
        &dir,
        &files
            .iter()
            .map(|(p, t)| (p.as_str(), t.as_str()))
            .collect::<Vec<_>>(),
    )?;
    std::fs::copy(
        cside_dir().join("bridge_source_caller.c"),
        dir.join("bridge_source_caller.c"),
    )
    .map_err(|e| BridgeError(format!("caller copy: {e}")))?;
    let exe = dir.join("source_export.exe");
    budget_left()?;
    compile_bridge_crt(
        &dir,
        &["library.c", "generated.c", "bridge_source_caller.c"],
        &exe,
    )?;
    let mut cmd = Command::new(&exe);
    let run = run_deadlined_captured(
        &mut cmd,
        BRIDGE_RUN_DEADLINE,
        CaptureCaps {
            stdout_max: BRIDGE_STREAM_CAP,
            stderr_max: BRIDGE_STREAM_CAP,
        },
    )
    .map_err(|e| BridgeError(format!("contained run: {e:?}")))?;
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let exit = run
        .exit_code
        .ok_or_else(|| BridgeError("source-export caller terminated abnormally".into()))?;

    Ok(SourceExportOutcome {
        facts_digest: facts_digest.clone(),
        library_manifest: manifest,
        caller_exit: exit,
        stdout,
        handle: BridgeHandle {
            facts_digest: facts_digest.clone(),
            plan_key: plan_key_canonical,
            source_sha256: source_sha.clone(),
            fragment_keys: fragment_key_canonicals,
            library_key: lib_key.canonical(),
        },
    })
}

// ---------------------------------------------------------------------------
// Restore (V-06): bounded decode + re-verify, no producers
// ---------------------------------------------------------------------------

/// The persisted bridge identity (what restore re-reads).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BridgeHandle {
    pub facts_digest: String,
    pub plan_key: String,
    pub source_sha256: String,
    pub fragment_keys: Vec<String>,
    pub library_key: String,
}

#[derive(Debug)]
pub struct RestoredBridge {
    pub facts: InterfaceFacts,
    pub plan: PhysicalPlan,
    pub library_manifest: String,
    pub payload_bytes_read: u64,
    pub producers_ran: u64,
}

/// Restore a bridge from persisted artifacts: STRICT decode + identity
/// re-verification only. No source recompile, no type inference, no
/// fragment/object production (observable via the counters).
pub fn restore_bridge(
    store: &ManagedStore,
    counters: &BridgeCounters,
    handle: &BridgeHandle,
) -> Result<RestoredBridge, BridgeError> {
    let read_key = |canonical: &str| -> Result<Vec<u8>, BridgeError> {
        let key = parse_key(canonical)?;
        counters.bump_get(canonical);
        let art = store
            .get_artifact(LAYER_BRIDGE_V01.id, &key)
            .map_err(|e| BridgeError(format!("get: {e:?}")))?
            .ok_or_else(|| BridgeError(format!("bridge page missing: {canonical}")))?;
        counters.add_bytes_read(art.payload.len() as u64);
        Ok(art.payload)
    };

    // The facts page is the identity root; the plan is re-derivable but
    // persisted alongside (physical key identity); fragments and the
    // library page are the adapter closure.
    let facts_bytes =
        read_key(&facts_key(&handle.facts_digest, &handle.source_sha256).canonical())?;
    let plan_bytes = read_key(&handle.plan_key)?;
    for k in &handle.fragment_keys {
        read_key(k)?;
    }
    let lib_bytes = read_key(&handle.library_key)?;

    let facts = facts_codec::decode(&facts_bytes).map_err(|e| BridgeError(e.to_string()))?;
    let plan = plan_codec::decode(&plan_bytes).map_err(|e| BridgeError(e.to_string()))?;
    // digest re-verification: persisted content must hash to the SAME
    // identity the handle names (a tampered/forged page mismatches).
    let fd = facts.digest().map_err(BridgeError)?;
    if fd != handle.facts_digest {
        return Err(BridgeError(format!(
            "restored facts digest mismatch: persisted {fd}, handle {}",
            handle.facts_digest
        )));
    }
    match &facts.source {
        FactsSource::CheckedSource { source_sha256, .. }
            if source_sha256 == &handle.source_sha256 => {}
        other => {
            return Err(BridgeError(format!(
                "restored facts source identity mismatch: {other:?}"
            )))
        }
    }
    let lib: serde_json::Value =
        serde_json::from_slice(&lib_bytes).map_err(|e| BridgeError(format!("lib: {e}")))?;
    let manifest = lib["manifest"].as_str().unwrap_or_default().to_string();
    if manifest.is_empty() {
        return Err(BridgeError(
            "library manifest missing in persisted page".into(),
        ));
    }
    Ok(RestoredBridge {
        facts,
        plan,
        library_manifest: manifest,
        payload_bytes_read: counters.bytes_read(),
        producers_ran: counters.total_producer_calls(),
    })
}

/// Parse a canonical `LAYER[k=v,...]` key back into a QueryKey (for
/// restore reads of persisted handles).
pub fn parse_key(canonical: &str) -> Result<QueryKey, BridgeError> {
    let inner = canonical
        .strip_prefix(&format!("{}[", LAYER_BRIDGE_V01.id))
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| BridgeError(format!("not a bridge key: {canonical}")))?;
    let mut components = Vec::new();
    if !inner.is_empty() {
        for part in inner.split(',') {
            let (k, v) = part
                .split_once('=')
                .ok_or_else(|| BridgeError(format!("bad key component '{part}'")))?;
            components.push((k.to_string(), v.to_string()));
        }
    }
    // keys are constructed sorted; restore re-sorts defensively.
    components.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(QueryKey {
        layer: LAYER_BRIDGE_V01.id,
        components,
    })
}
