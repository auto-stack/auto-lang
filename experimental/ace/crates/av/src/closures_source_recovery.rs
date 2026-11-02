//! PLAN-913 T-02: the closures profile's SOURCE-BOUND recovery entry.
//!
//! The 912 function session closes R1/R2/R4 but leaves R3 open: every
//! restart still calls `verify_mir` + `encode_abc` before it can even look
//! up a unit, because the managed-store key's `unit-content` component is a
//! domain hash of the CANONICAL encoded wire (`av-closures-unit/v1`). A
//! restore therefore cannot be a lookup — it has to be a new entry that
//! reads the caller's already-authorized source-origin root and re-verifies
//! the persisted units, producing no frontend stage at all.
//!
//! This module owns:
//!
//! - the bounded, versioned manifest (`ace.closures.source-origin/1`) a
//!   successful production publishes next to its per-unit ABC artifacts;
//! - the atomic publication of that manifest as the project's
//!   `source-recovery-current` root (register the new root first, retire the
//!   superseded one second — a failed publication never loses the old root);
//! - the restore path: resolve the caller's EXPECTED manifest id, unwrap +
//!   digest-check it, bind it to the request's source/config, read every
//!   required unit BY ARTIFACT ID, re-check its envelope, recompute its body
//!   fingerprint, re-decode + deep-admit it (`verify_abc_bytes`), and only
//!   then hand `AdmittedUnit`s to the session (which installs, links and
//!   executes them with ZERO production stages).
//!
//! Trust model: the manifest and every unit artifact are DATA. The parent
//! process — the one that saw the first real production — keeps the
//! manifest's artifact id and its strong digest; a self-consistent rewritten
//! manifest (or index, or payload) is rejected because the request carries
//! the expected digest, never a value re-derived from the store. The store
//! itself is never an authority: only `verify_abc_bytes` admits code.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use ace_artifacts::codec::ChunkKind;
use ace_artifacts::identity::QueryKey;
use ace_artifacts::schema::{LayerCoverage, LayerSpec, Regeneration};
use ace_artifacts::store::{ManagedStore, PutRequest};
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::contract::{fnv1a64, CLOSURES_PRODUCER_ID};
use ace_av_core::function_code::FunctionCode;
use ace_av_core::mir::OwnerScope;
use serde::{Deserialize, Serialize};

use crate::cache::{envelope_unwrap, envelope_wrap, EnvelopeIdentity, EnvelopeKind};
use crate::closures_session::AdmittedUnit;

use ace_core::source::closures_context::{
    restore_context, CaptureModeDump, ContextDump, UnitEvidence,
};
use ace_core::source::closures_fn::ClosuresFunctionFrontend;
use base64::Engine as _;

/// The manifest schema of the source-origin root. Every consumer of the
/// recovery fact reads this id; a different schema is a controlled STALE.
pub const RECOVERY_SCHEMA: &str = "ace.closures.source-origin/1";
/// Root kind registered for a project's CURRENT recovery manifest.
pub const RECOVERY_ROOT_KIND: &str = "source-recovery-current";
/// This domain's private layer seam. It does NOT take a global cache layer
/// number and leaves the A00–A23 obligations untouched (`closures_fixture`
/// keeps A23 for the per-unit ABC wires).
pub const RECOVERY_LAYER: LayerSpec = LayerSpec {
    id: "A23-R",
    name: "closures source-origin recovery manifest (plan913)",
    schema_id: "ac913/a23-closures-source-recovery/v1",
    producer: CLOSURES_PRODUCER_ID,
    consumers: &["closures source recovery entry (re-verifies every referenced unit)"],
    key_inputs: &[
        "producer",
        "owner-module",
        "entry",
        "profile",
        "source-sha256",
        "context-revision",
        "unit-set",
    ],
    ref_kinds: &["provenance"],
    storage: "one manifest artifact per accepted source state (LayoutData carrier)",
    residency: "restored manifests are untrusted data; every referenced unit re-verifies",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::ProbeSeam,
};

/// Bounded sizes (§5.2): the manifest, the source and the recovery
/// namespace's own writes are all capped before materialization.
pub const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_MANIFEST_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_WIRE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_UNITS: usize = 4096;

/// The error of the recovery entry: a NAMED, locatable state — never a
/// silent cold fallback and never a "miss" that rewrites a PASS.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryError {
    pub code: &'static str,
    pub message: String,
}

impl RecoveryError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        RecoveryError {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

/// The recovery measurement axes (§5.6). `p_axis_attempts` MUST be zero for
/// every positive restore: it counts the source-production stages a restore
/// is forbidden to take. `R` is the real deep-verification work.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecoveryCounters {
    pub manifest_reads: usize,
    pub unit_reads: usize,
    pub unit_deep_verifies: usize,
    pub unit_admits: usize,
    pub manifest_bytes: usize,
    pub wire_bytes: usize,
    /// Source-production attempts taken by this restore (parse/resolve/
    /// typecheck/plan/bind/lower/encode). Always zero on the positive path.
    pub p_axis_attempts: usize,
    /// The declaration-context artifact decoded + cross-checked here (R axis:
    /// context decode, evidence build, context rebuild).
    pub context_reads: usize,
}

/// One unit of the published manifest: identity facts plus the address of
/// the persisted ABC wire. Deliberately carries NO guest value, PASS flag,
/// live env/cell or host address.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryUnit {
    /// Wire identity (`d_<name>`).
    pub unit: String,
    /// `owner` or `lambda:<logical_id>:<owner>` — the logical identity the
    /// 912 accepted state exported.
    pub logical: String,
    /// Span-free canonical digest of the unit's verified HIR body.
    pub body_digest: String,
    /// Identity of the closure facts the unit reads.
    pub dependency_digest: String,
    pub env_layout_identity: Option<String>,
    /// The session's spanless submission identity (accepted-state view).
    pub mir_identity: String,
    /// The persisted wire's own fingerprint (`fnv1a64` of the ABC bytes).
    pub body_fp: String,
    pub wire_bytes: usize,
    /// The store artifact id of the unit's canonical ABC wire.
    pub artifact: String,
    pub mint_span: (usize, usize),
    pub current_span: (usize, usize),
    /// PLAN-913 T-05: the unit's SPANLESS identity wire (base64) — the wire
    /// the warm path compares against before it submits. A restore can never
    /// re-encode it (encode is a production stage), so it is required data.
    /// Its `fnv1a64` must equal the accepted state's `mir_identity`.
    pub identity_wire_b64: String,
    pub identity_fnv: String,
}

/// The required declaration-context artifact this manifest closes over.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryContextRef {
    pub schema: String,
    pub artifact: String,
    pub digest: String,
    pub bytes: usize,
}

/// The published, versioned source-origin manifest.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRecoveryManifest {
    pub schema: String,
    pub source_sha256: String,
    pub source_len: usize,
    /// The logical owner (module path).
    pub logical_owner: String,
    /// The MIR owner-module name every unit's wire was lowered against
    /// (`MirModule.owner.module`) — the unit envelopes' owner + the
    /// admission owner. Distinct from the logical source path.
    pub owner_module: String,
    pub owner_schema: String,
    pub entry: String,
    pub profile: String,
    pub capabilities: Vec<String>,
    pub producer_mir_lowering_version: String,
    pub producer_closures_schema: String,
    pub producer_unit_codec: String,
    pub abc_codec: u16,
    pub mir_semantics: u32,
    pub abc_semantics: u32,
    pub av_semantics: u32,
    pub accepted_generation: u64,
    pub commit_point: u64,
    pub context_revision: String,
    /// The accepted-state export this manifest was formed from (a data view,
    /// recorded so a consumer can tell WHICH accepted state it restores).
    pub accepted_state_schema: String,
    /// The required declaration context (dump) that lets the restored process
    /// take ONE more single-body edit without re-producing the module.
    pub context: RecoveryContextRef,
    pub units: Vec<RecoveryUnit>,
}

impl SourceRecoveryManifest {
    /// Every unit's wire identity, sorted (the unit-set key input).
    pub fn unit_names(&self) -> Vec<String> {
        self.units.iter().map(|u| u.unit.clone()).collect()
    }
    fn digest_of_units(&self) -> String {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for u in &self.units {
            for part in [&u.unit, &u.body_fp, &u.body_digest] {
                for b in part.as_bytes() {
                    h ^= *b as u64;
                    h = h.wrapping_mul(0x0000_0100_0000_001b_3);
                }
                h ^= 0xff;
                h = h.wrapping_mul(0x0000_0100_0000_001b_3);
            }
        }
        format!("{h:016x}")
    }
}

/// The ticket a successful publication hands to its caller. The parent keeps
/// `manifest_artifact` + `manifest_digest` and forwards them to a DIFFERENT
/// process; it never pre-seeds a "must pass" ticket.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryTicket {
    pub manifest_artifact: String,
    /// SHA-256 (hex) of the manifest's payload bytes — the strong digest the
    /// restoring process must be given OUT OF BAND by the parent.
    pub manifest_digest: String,
    pub manifest_bytes: usize,
    pub units: usize,
    pub source_sha256: String,
    pub context_revision: String,
    /// The project root this publication registered.
    pub root_project: String,
}

/// One unit's input to publication: the accepted-state view plus the
/// persisted wire's address and its own fingerprint.
#[derive(Clone, Debug)]
pub struct RecoveryUnitInput {
    pub unit: String,
    pub logical: String,
    pub body_digest: String,
    pub dependency_digest: String,
    pub env_layout_identity: Option<String>,
    pub mir_identity: String,
    pub body_fp: String,
    pub wire_bytes: usize,
    pub artifact: String,
    pub mint_span: (usize, usize),
    pub current_span: (usize, usize),
    /// The spanless identity wire (see [`RecoveryUnit::identity_wire_b64`]).
    pub identity_wire: Vec<u8>,
}

/// The publication request: the accepted source state plus the project that
/// owns the recovery root.
pub struct PublicationRequest<'a> {
    pub project: &'a str,
    pub source_sha256: &'a str,
    pub source_len: usize,
    pub logical_owner: &'a str,
    /// The MIR owner-module name the produced units were lowered against
    /// (`MirModule.owner.module`); the unit envelopes bind to it.
    pub owner_module: &'a str,
    /// The owner schema (`MirModule.owner.schema`) the units were lowered
    /// against; the closures profile fixes it to `MIR_CLOSURES_SCHEMA`.
    pub owner_schema: &'a str,
    pub entry: &'a str,
    pub profile: &'a str,
    pub capabilities: &'a [String],
    pub accepted_generation: u64,
    pub commit_point: u64,
    pub context_revision: &'a str,
    pub accepted_state_schema: &'a str,
    /// The serialized declaration context (`ContextDump`, schema
    /// `ace.closures.context-dump/1`) — published as a REQUIRED artifact the
    /// manifest closes over.
    pub context_payload: &'a [u8],
    pub context_schema: &'a str,
    pub units: Vec<RecoveryUnitInput>,
}

fn manifest_envelope_snapshot(source_sha256: &str) -> u64 {
    fnv1a64(source_sha256.as_bytes())
}

fn manifest_identity<'a>(
    owner: &'a str,
    source_sha256: &'a str,
) -> EnvelopeIdentity<'a> {
    EnvelopeIdentity {
        kind: EnvelopeKind::ModuleManifest,
        producer: CLOSURES_PRODUCER_ID,
        schema: RECOVERY_SCHEMA,
        owner_module: owner,
        stage: 2,
        mir_sem: ace_av_core::contract::MIR_CLOSURES_SEMANTICS.0,
        abc_sem: ace_av_core::contract::ABC_CLOSURES_SEMANTICS.0,
        av_sem: ace_av_core::contract::AV_CLOSURES_SEMANTICS.0,
        snapshot: manifest_envelope_snapshot(source_sha256),
        unit: "source-recovery",
        // The manifest envelope's body fingerprint slot carries the
        // manifest's own digest: the writer binds the payload, the reader
        // supplies the expected digest it was given out of band.
        body_fp: "",
    }
}

fn unit_identity<'a>(
    owner: &'a str,
    snapshot: u64,
    unit: &'a str,
    body_fp: &'a str,
) -> EnvelopeIdentity<'a> {
    EnvelopeIdentity {
        kind: EnvelopeKind::AbcUnit,
        producer: CLOSURES_PRODUCER_ID,
        schema: ace_av_core::contract::ABC_CLOSURES_SCHEMA,
        owner_module: owner,
        stage: 2,
        mir_sem: ace_av_core::contract::MIR_CLOSURES_SEMANTICS.0,
        abc_sem: ace_av_core::contract::ABC_CLOSURES_SEMANTICS.0,
        av_sem: ace_av_core::contract::AV_CLOSURES_SEMANTICS.0,
        snapshot,
        unit,
        body_fp,
    }
}

fn manifest_key(m: &SourceRecoveryManifest) -> QueryKey {
    QueryKey::new(
        &RECOVERY_LAYER,
        vec![
            ("producer".into(), CLOSURES_PRODUCER_ID.into()),
            ("owner-module".into(), m.logical_owner.clone()),
            ("entry".into(), m.entry.clone()),
            ("profile".into(), m.profile.clone()),
            ("source-sha256".into(), m.source_sha256.clone()),
            ("context-revision".into(), m.context_revision.clone()),
            ("unit-set".into(), m.digest_of_units()),
        ],
    )
}

/// Publish one accepted source state as the project's CURRENT recovery root.
///
/// Order (§5.3): serialize + cap → put the manifest (leased) → register the
/// NEW root → only then retire the superseded root. A failure at any step
/// leaves the previous valid root intact and publishes no half ticket. The
/// per-unit ABC artifacts are the caller's already-persisted ones; they are
/// leased for the publication window and required roots are (re)registered
/// so a visible root always closes over live artifacts.
pub fn publish_source_recovery(
    store_root: &Path,
    request: PublicationRequest<'_>,
) -> Result<RecoveryTicket, RecoveryError> {
    if request.source_len > MAX_SOURCE_BYTES {
        return Err(RecoveryError::new(
            "recovery.source-too-large",
            format!(
                "source is {} bytes (cap {MAX_SOURCE_BYTES})",
                request.source_len
            ),
        ));
    }
    if request.units.is_empty() || request.units.len() > MAX_UNITS {
        return Err(RecoveryError::new(
            "recovery.unit-count",
            format!(
                "publication requires 1..={MAX_UNITS} units, got {}",
                request.units.len()
            ),
        ));
    }
    let mut units: Vec<RecoveryUnit> = request
        .units
        .iter()
        .map(|u| RecoveryUnit {
            unit: u.unit.clone(),
            logical: u.logical.clone(),
            body_digest: u.body_digest.clone(),
            dependency_digest: u.dependency_digest.clone(),
            env_layout_identity: u.env_layout_identity.clone(),
            mir_identity: u.mir_identity.clone(),
            body_fp: u.body_fp.clone(),
            wire_bytes: u.wire_bytes,
            artifact: u.artifact.clone(),
            mint_span: u.mint_span,
            current_span: u.current_span,
            identity_wire_b64: base64::engine::general_purpose::STANDARD
                .encode(&u.identity_wire),
            identity_fnv: format!("{:016x}", fnv1a64(&u.identity_wire)),
        })
        .collect();
    units.sort_by(|a, b| a.unit.cmp(&b.unit));
    let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
    for u in &units {
        if seen.insert(u.unit.as_str(), ()).is_some() {
            return Err(RecoveryError::new(
                "recovery.duplicate-unit",
                format!("unit `{}` appears twice", u.unit),
            ));
        }
        if u.wire_bytes > MAX_WIRE_BYTES {
            return Err(RecoveryError::new(
                "recovery.unit-too-large",
                format!("unit `{}` is {} wire bytes", u.unit, u.wire_bytes),
            ));
        }
        if u.artifact.is_empty() {
            return Err(RecoveryError::new(
                "recovery.unit-unaddressed",
                format!("unit `{}` has no persisted artifact", u.unit),
            ));
        }
        // The spanless identity wire is required data: its fingerprint must
        // be the accepted state's submission identity.
        if u.identity_wire_b64.is_empty() {
            return Err(RecoveryError::new(
                "recovery.unit-identity",
                format!("unit `{}` has no spanless identity wire", u.unit),
            ));
        }
        if u.identity_fnv != u.mir_identity {
            return Err(RecoveryError::new(
                "recovery.unit-identity",
                format!(
                    "unit `{}` identity wire {} != accepted state's mir_identity {}",
                    u.unit, u.identity_fnv, u.mir_identity
                ),
            ));
        }
    }
    if request.context_payload.is_empty() {
        return Err(RecoveryError::new(
            "recovery.context-missing",
            "publication requires the serialized declaration context",
        ));
    }
    if request.context_payload.len() > MAX_MANIFEST_BYTES {
        return Err(RecoveryError::new(
            "recovery.context-too-large",
            format!(
                "context is {} bytes (cap {MAX_MANIFEST_BYTES})",
                request.context_payload.len()
            ),
        ));
    }
    let store = ManagedStore::open(store_root).map_err(|e| {
        RecoveryError::new("recovery.store-open", format!("open store: {e}"))
    })?;
    // The declaration context rides its own REQUIRED artifact so a manifest
    // stays bounded and the context is digest-addressed independently.
    let context_digest = ace_core::runtime_contract::sha256_hex(request.context_payload);
    let mut context_identity = manifest_identity(
        request.logical_owner,
        request.source_sha256,
    );
    context_identity.unit = "context-dump";
    context_identity.body_fp = &context_digest;
    let context_artifact = store
        .put_artifact(PutRequest {
            layer: RECOVERY_LAYER.id,
            key: QueryKey::new(
                &RECOVERY_LAYER,
                vec![
                    ("producer".into(), CLOSURES_PRODUCER_ID.into()),
                    ("owner-module".into(), request.logical_owner.to_string()),
                    ("context-schema".into(), request.context_schema.to_string()),
                    ("context-digest".into(), context_digest.clone()),
                ],
            ),
            kind: ChunkKind::LayoutData,
            payload: envelope_wrap(request.context_payload, &context_identity),
            edges: Vec::new(),
            project: None,
        })
        .map_err(|e| RecoveryError::new("recovery.put", format!("put context: {e}")))?;
    let _context_lease = store
        .acquire_lease("plan913-recovery-publish", 300, Some(&context_artifact))
        .map_err(|e| {
            RecoveryError::new("recovery.lease", format!("lease context: {e}"))
        })?;
    let manifest = SourceRecoveryManifest {
        schema: RECOVERY_SCHEMA.to_string(),
        source_sha256: request.source_sha256.to_string(),
        source_len: request.source_len,
        logical_owner: request.logical_owner.to_string(),
        owner_module: request.owner_module.to_string(),
        owner_schema: request.owner_schema.to_string(),
        entry: request.entry.to_string(),
        profile: request.profile.to_string(),
        capabilities: request.capabilities.to_vec(),
        producer_mir_lowering_version: ace_lowering::MIR_LOWERING_VERSION.to_string(),
        producer_closures_schema: ace_av_core::contract::MIR_CLOSURES_SCHEMA.to_string(),
        producer_unit_codec: "av-closures-unit/v1".to_string(),
        abc_codec: ace_av_core::contract::ABC_CLOSURES_VERSION,
        mir_semantics: ace_av_core::contract::MIR_CLOSURES_SEMANTICS.0,
        abc_semantics: ace_av_core::contract::ABC_CLOSURES_SEMANTICS.0,
        av_semantics: ace_av_core::contract::AV_CLOSURES_SEMANTICS.0,
        accepted_generation: request.accepted_generation,
        commit_point: request.commit_point,
        context_revision: request.context_revision.to_string(),
        accepted_state_schema: request.accepted_state_schema.to_string(),
        context: RecoveryContextRef {
            schema: request.context_schema.to_string(),
            artifact: context_artifact.clone(),
            digest: context_digest,
            bytes: request.context_payload.len(),
        },
        units,
    };
    let payload = serde_json::to_vec(&manifest).map_err(|e| {
        RecoveryError::new("recovery.encode", format!("encode manifest: {e}"))
    })?;
    if payload.len() > MAX_MANIFEST_BYTES {
        return Err(RecoveryError::new(
            "recovery.manifest-too-large",
            format!(
                "manifest is {} bytes (cap {MAX_MANIFEST_BYTES})",
                payload.len()
            ),
        ));
    }
    let digest = ace_core::runtime_contract::sha256_hex(&payload);
    let mut identity = manifest_identity(&manifest.logical_owner, &manifest.source_sha256);
    identity.body_fp = &digest;
    let framed = envelope_wrap(&payload, &identity);
    // The manifest's required closure: every unit wire AND the declaration
    // context. A visible root therefore always closes over live required data.
    let mut edges: Vec<(String, ace_artifacts::identity::RefKind)> = manifest
        .units
        .iter()
        .map(|u| {
            (
                u.artifact.clone(),
                ace_artifacts::identity::RefKind::RequiredData,
            )
        })
        .collect();
    edges.push((
        context_artifact,
        ace_artifacts::identity::RefKind::RequiredData,
    ));
    let artifact = store
        .put_artifact(PutRequest {
            layer: RECOVERY_LAYER.id,
            key: manifest_key(&manifest),
            kind: ChunkKind::LayoutData,
            payload: framed,
            edges,
            project: None,
        })
        .map_err(|e| RecoveryError::new("recovery.put", format!("put manifest: {e}")))?;
    // Leases for every referenced artifact and the manifest itself for the
    // publication window (an OS-backed lease dies with this process).
    let _manifest_lease = store
        .acquire_lease("plan913-recovery-publish", 300, Some(&artifact))
        .map_err(|e| {
            RecoveryError::new("recovery.lease", format!("lease manifest: {e}"))
        })?;
    let mut unit_leases = Vec::with_capacity(manifest.units.len());
    for u in &manifest.units {
        unit_leases.push(
            store
                .acquire_lease("plan913-recovery-publish", 300, Some(&u.artifact))
                .map_err(|e| {
                    RecoveryError::new(
                        "recovery.lease",
                        format!("lease unit {}: {e}", u.unit),
                    )
                })?,
        );
    }
    // Atomic root update: new root first, superseded roots second.
    let previous = store
        .root_artifacts(request.project, RECOVERY_ROOT_KIND)
        .map_err(|e| RecoveryError::new("recovery.root-read", format!("read roots: {e}")))?;
    store
        .register_root(request.project, &artifact, RECOVERY_ROOT_KIND)
        .map_err(|e| RecoveryError::new("recovery.root", format!("register root: {e}")))?;
    for old in previous {
        if old == artifact {
            continue;
        }
        store
            .remove_root(request.project, RECOVERY_ROOT_KIND, &old)
            .map_err(|e| {
                RecoveryError::new("recovery.root", format!("retire old root {old}: {e}"))
            })?;
    }
    Ok(RecoveryTicket {
        manifest_bytes: payload.len(),
        manifest_digest: digest,
        manifest_artifact: artifact,
        units: manifest.units.len(),
        source_sha256: manifest.source_sha256.clone(),
        context_revision: manifest.context_revision.clone(),
        root_project: request.project.to_string(),
    })
}

/// The restore request: everything the new process may know. Note the
/// absence of any value/oracle field — a restore never carries the expected
/// result, only the parent's expected root binding.
pub struct RestoreRequest<'a> {
    pub expected_manifest: &'a str,
    /// The manifest payload digest the PARENT observed at publication.
    pub expected_digest: &'a str,
    pub source: &'a str,
    pub logical_owner: &'a str,
    pub entry: &'a str,
    pub profile: &'a str,
    pub capabilities: &'a [String],
    /// 0 = read-only store: the restore performs no data/root write.
    pub read_only: bool,
}

/// The result of a restore: the manifest it verified, the admitted units,
/// the execution face of the linked program, and the real work axes.
pub struct RestoreOutcome {
    pub manifest: SourceRecoveryManifest,
    pub units: Vec<AdmittedUnit>,
    /// PLAN-913 T-05: the SAFE frontend context rebuilt from the persisted
    /// declaration context, cross-checked against the decoded units.
    pub frontend: ClosuresFunctionFrontend,
    /// Per-unit spanless identity wires (the warm path's submission identity).
    pub identity_wires: BTreeMap<String, Vec<u8>>,
    pub counters: RecoveryCounters,
    pub accepted_state_schema: String,
    pub accepted_generation: u64,
    pub commit_point: u64,
}

/// Restore one accepted source state from the caller's EXPECTED root.
pub fn restore_source_recovery(
    store_root: &Path,
    request: &RestoreRequest<'_>,
) -> Result<RestoreOutcome, RecoveryError> {
    if request.source.len() > MAX_SOURCE_BYTES {
        return Err(RecoveryError::new(
            "recovery.source-too-large",
            format!(
                "source is {} bytes (cap {MAX_SOURCE_BYTES})",
                request.source.len()
            ),
        ));
    }
    let store = ManagedStore::open(store_root).map_err(|e| {
        RecoveryError::new("recovery.store-open", format!("open store: {e}"))
    })?;
    // 1. Resolve the caller's expected manifest BY ID. A missing artifact is
    //    a locatable MISS — never a silent cold rebuild.
    let found = store
        .get_artifact_by_id(request.expected_manifest)
        .map_err(|e| RecoveryError::new("recovery.read", format!("read manifest: {e}")))?
        .ok_or_else(|| {
            RecoveryError::new(
                "recovery.miss",
                format!(
                    "expected manifest {} is absent from the store",
                    request.expected_manifest
                ),
            )
        })?;
    if found.kind != ChunkKind::LayoutData {
        return Err(RecoveryError::new(
            "recovery.kind",
            format!(
                "expected manifest carries kind {:?}, expected LayoutData",
                found.kind
            ),
        ));
    }
    let mut counters = RecoveryCounters {
        manifest_reads: 1,
        manifest_bytes: found.payload.len(),
        ..Default::default()
    };
    // 2. Unwrap the manifest against the REQUEST's facts (producer/schema/
    //    owner/semantics/snapshot) and the parent's expected digest.
    let snapshot = manifest_envelope_snapshot_of_source(request.source);
    let identity = EnvelopeIdentity {
        kind: EnvelopeKind::ModuleManifest,
        producer: CLOSURES_PRODUCER_ID,
        schema: RECOVERY_SCHEMA,
        owner_module: request.logical_owner,
        stage: 2,
        mir_sem: ace_av_core::contract::MIR_CLOSURES_SEMANTICS.0,
        abc_sem: ace_av_core::contract::ABC_CLOSURES_SEMANTICS.0,
        av_sem: ace_av_core::contract::AV_CLOSURES_SEMANTICS.0,
        snapshot,
        unit: "source-recovery",
        body_fp: request.expected_digest,
    };
    let payload = envelope_unwrap(&found.payload, &identity).map_err(|e| {
        RecoveryError::new(
            "recovery.envelope",
            format!("manifest envelope rejected: {e:?}"),
        )
    })?;
    // 3. Digest check: the caller's expectation is authoritative, never the
    //    store's self-description.
    let digest = ace_core::runtime_contract::sha256_hex(&payload);
    if !constant_time_eq(digest.as_bytes(), request.expected_digest.as_bytes()) {
        return Err(RecoveryError::new(
            "recovery.digest",
            format!(
                "manifest digest mismatch: store says {}, caller expects {}",
                digest, request.expected_digest
            ),
        ));
    }
    let manifest: SourceRecoveryManifest = serde_json::from_slice(&payload).map_err(|e| {
        RecoveryError::new("recovery.decode", format!("decode manifest: {e}"))
    })?;
    // 4. Bind the manifest to THIS request (source/config/producer).
    verify_manifest_binding(&manifest, request)?;
    // 4b. The REQUIRED declaration context: read BY ARTIFACT ID, unwrap
    //     against the manifest's recorded digest, re-digest and decode. The
    //     decoded dump stays untrusted until `restore_context` cross-checks it
    //     against the units this restore admits below.
    let context_bytes = {
        if manifest.context.bytes > MAX_MANIFEST_BYTES {
            return Err(RecoveryError::new(
                "recovery.context-too-large",
                format!(
                    "manifest declares a {} byte context (cap {MAX_MANIFEST_BYTES})",
                    manifest.context.bytes
                ),
            ));
        }
        let found = store
            .get_artifact_by_id(&manifest.context.artifact)
            .map_err(|e| {
                RecoveryError::new("recovery.read", format!("read context: {e}"))
            })?
            .ok_or_else(|| {
                RecoveryError::new(
                    "recovery.miss",
                    format!(
                        "declaration context artifact {} is absent",
                        manifest.context.artifact
                    ),
                )
            })?;
        if found.kind != ChunkKind::LayoutData {
            return Err(RecoveryError::new(
                "recovery.kind",
                format!(
                    "declaration context artifact carries kind {:?}",
                    found.kind
                ),
            ));
        }
        let mut context_identity =
            manifest_identity(&manifest.logical_owner, &manifest.source_sha256);
        context_identity.unit = "context-dump";
        context_identity.body_fp = &manifest.context.digest;
        let bytes = envelope_unwrap(&found.payload, &context_identity).map_err(|e| {
            RecoveryError::new(
                "recovery.envelope",
                format!("declaration context envelope rejected: {e:?}"),
            )
        })?;
        let digest = ace_core::runtime_contract::sha256_hex(&bytes);
        if !constant_time_eq(digest.as_bytes(), manifest.context.digest.as_bytes()) {
            return Err(RecoveryError::new(
                "recovery.digest",
                format!(
                    "declaration context digest mismatch: {} != {}",
                    digest, manifest.context.digest
                ),
            ));
        }
        if bytes.len() != manifest.context.bytes {
            return Err(RecoveryError::new(
                "recovery.context-mismatch",
                format!(
                    "declaration context is {} bytes, manifest says {}",
                    bytes.len(),
                    manifest.context.bytes
                ),
            ));
        }
        counters.context_reads = 1;
        bytes
    };
    // 5. Read + re-verify every required unit BY ARTIFACT ID.
    let owner = OwnerScope {
        schema: manifest.owner_schema.clone(),
        module: manifest.owner_module.clone(),
    };
    let mut units: Vec<AdmittedUnit> = Vec::with_capacity(manifest.units.len());
    for u in &manifest.units {
        let found = store
            .get_artifact_by_id(&u.artifact)
            .map_err(|e| {
                RecoveryError::new(
                    "recovery.read",
                    format!("read unit {}: {e}", u.unit),
                )
            })?
            .ok_or_else(|| {
                RecoveryError::new(
                    "recovery.miss",
                    format!("unit {} artifact {} is absent", u.unit, u.artifact),
                )
            })?;
        counters.unit_reads += 1;
        if found.kind != ChunkKind::LayoutData {
            return Err(RecoveryError::new(
                "recovery.kind",
                format!("unit {} carries kind {:?}", u.unit, found.kind),
            ));
        }
        if found.payload.len() > MAX_WIRE_BYTES {
            return Err(RecoveryError::new(
                "recovery.unit-too-large",
                format!("unit {} payload is {} bytes", u.unit, found.payload.len()),
            ));
        }
        let wire = envelope_unwrap(
            &found.payload,
            &unit_identity(
                &manifest.owner_module,
                // The unit envelope's snapshot is the LOGICAL source origin
                // (`fnv1a64(module_path)`) the production wrote, not the
                // manifest's own source-sha binding.
                fnv1a64(manifest.logical_owner.as_bytes()),
                &u.unit,
                &u.body_fp,
            ),
        )
        .map_err(|e| {
            RecoveryError::new(
                "recovery.envelope",
                format!("unit {} envelope rejected: {e:?}", u.unit),
            )
        })?;
        // The envelope's claimed fingerprint must match its own payload.
        let fp = format!("{:016x}", fnv1a64(&wire));
        if fp != u.body_fp {
            return Err(RecoveryError::new(
                "recovery.unit-fingerprint",
                format!(
                    "unit {} payload fingerprint {fp} != manifest {}",
                    u.unit, u.body_fp
                ),
            ));
        }
        if wire.len() != u.wire_bytes {
            return Err(RecoveryError::new(
                "recovery.unit-fingerprint",
                format!(
                    "unit {} payload is {} bytes, manifest says {}",
                    u.unit,
                    wire.len(),
                    u.wire_bytes
                ),
            ));
        }
        counters.wire_bytes = counters.wire_bytes.saturating_add(wire.len());
        // REAL deep verification: decode + admission of the persisted ABC.
        let loaded = verify_abc_bytes(&wire, &owner).map_err(|e| {
            RecoveryError::new(
                "recovery.verify",
                format!("unit {} deep verification rejected: {e:?}", u.unit),
            )
        })?;
        ace_core::body_work::record_attempt(ace_core::body_work::Stage::RestartDeepVerify,
            &u.unit,
        );
        counters.unit_deep_verifies += 1;
        // Host capability authorization stays a REQUEST fact: the manifest
        // records what the unit needs, the current host decides.
        for required in &loaded.module().capabilities {
            if !request.capabilities.iter().any(|g| g == required) {
                return Err(RecoveryError::new(
                    "recovery.capability",
                    format!(
                        "unit {} requires capability `{required}` not granted by the current host",
                        u.unit
                    ),
                ));
            }
        }
        let code = FunctionCode::admit_unit(loaded, 1).map_err(|e| {
            RecoveryError::new(
                "recovery.admit",
                format!("unit {} unit-admission rejected: {e}", u.unit),
            )
        })?;
        counters.unit_admits += 1;
        units.push(AdmittedUnit {
            name: u.unit.clone(),
            code: Arc::new(code),
            wire: wire.into(),
            body_fp: u.body_fp.clone(),
            artifact: Some(u.artifact.clone()),
        });
    }
    if request.read_only {
        // A read-only restore performs no data/root write: opening the store
        // is a read, and no publication/root API is touched above.
    }
    // 6. The spanless identity wires (required data; their fingerprint is the
    //    accepted state's submission identity).
    let mut identity_wires: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for u in &manifest.units {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(u.identity_wire_b64.as_bytes())
            .map_err(|e| {
                RecoveryError::new(
                    "recovery.identity",
                    format!("unit {} identity wire is not base64: {e}", u.unit),
                )
            })?;
        if bytes.is_empty() {
            return Err(RecoveryError::new(
                "recovery.identity",
                format!("unit {} has an empty identity wire", u.unit),
            ));
        }
        let fp = format!("{:016x}", fnv1a64(&bytes));
        if fp != u.identity_fnv || fp != u.mir_identity {
            return Err(RecoveryError::new(
                "recovery.identity",
                format!(
                    "unit {} identity wire fingerprint {fp} != accepted identity {}",
                    u.unit, u.mir_identity
                ),
            ));
        }
        identity_wires.insert(u.unit.clone(), bytes);
    }
    // 7. Rebuild the SAFE declaration context from the untrusted dump, using
    //    the DECODED unit facts as the cross-check. No parse/resolve/typecheck/
    //    lower/encode runs here.
    let dump: ContextDump = serde_json::from_slice(&context_bytes).map_err(|e| {
        RecoveryError::new(
            "recovery.decode",
            format!("decode declaration context: {e}"),
        )
    })?;
    if dump.schema != manifest.context.schema {
        return Err(RecoveryError::new(
            "recovery.stale",
            format!(
                "declaration context schema {} != manifest {}",
                dump.schema, manifest.context.schema
            ),
        ));
    }
    // The dump is the SAME accepted state the manifest describes.
    if dump.source_sha != manifest.source_sha256
        || dump.module_path != manifest.logical_owner
        || dump.context_revision != manifest.context_revision
        || dump.generation != manifest.accepted_generation
    {
        return Err(RecoveryError::new(
            "recovery.context-mismatch",
            "the declaration context describes a different accepted state than the manifest",
        ));
    }
    let mut evidence: Vec<UnitEvidence> = Vec::with_capacity(units.len());
    for unit in &units {
        evidence.push(unit_evidence(&unit.name, unit.code.unit().module())?);
    }
    let frontend = restore_context(
        dump,
        request.source,
        request.logical_owner,
        request.capabilities,
        &evidence,
    )
    .map_err(|ds| {
        RecoveryError::new(
            "recovery.context",
            format!(
                "the declaration context was rejected: {}",
                ds.iter()
                    .map(|d| format!("{} ({})", d.code, d.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        )
    })?;
    Ok(RestoreOutcome {
        units,
        frontend,
        identity_wires,
        counters,
        accepted_state_schema: manifest.accepted_state_schema.clone(),
        accepted_generation: manifest.accepted_generation,
        commit_point: manifest.commit_point,
        manifest,
    })
}

/// Build the decoded-unit evidence the restored declaration context is
/// cross-checked against: a lambda's MIR environment arity and a creator's
/// `CreateClosure` sites (callee + capture-mode sequence, in body order).
fn unit_evidence(
    unit: &str,
    module: &ace_av_core::mir::MirModule,
) -> Result<ace_core::source::closures_context::UnitEvidence, RecoveryError> {
    use ace_av_core::mir::{CalleeRef, CaptureKind, MirInstr};
    let func = module.funcs.first().ok_or_else(|| {
        RecoveryError::new(
            "recovery.evidence",
            format!("unit {unit} carries no function"),
        )
    })?;
    let mut sites = Vec::new();
    for instr in func.blocks.iter().flat_map(|b| b.instrs.iter()) {
        let MirInstr::CreateClosure { callee, captures, .. } = instr else {
            continue;
        };
        let symbol = match callee {
            CalleeRef::Import(i) => module
                .imports
                .get(*i as usize)
                .map(|im| im.symbol.clone())
                .unwrap_or_default(),
            CalleeRef::Func(f) => module
                .funcs
                .get(*f as usize)
                .map(|f| f.name.clone())
                .unwrap_or_default(),
        };
        let callee_name = symbol
            .strip_prefix("fn:")
            .unwrap_or(symbol.as_str())
            .to_string();
        let mut modes = Vec::with_capacity(captures.len());
        for capture in captures {
            modes.push(match capture.kind {
                CaptureKind::Move => CaptureModeDump::Move,
                CaptureKind::Borrow => CaptureModeDump::Borrow,
                CaptureKind::Shared => CaptureModeDump::Shared,
                other => {
                    return Err(RecoveryError::new(
                        "recovery.evidence",
                        format!(
                            "unit {unit} creates {callee_name} with capture kind {other:?}, \
                             which the closures registry does not declare"
                        ),
                    ))
                }
            });
        }
        sites.push((callee_name, modes));
    }
    Ok(ace_core::source::closures_context::UnitEvidence {
        unit: unit.to_string(),
        env_arity: func.env.as_ref().map(|env| env.len()),
        sites,
    })
}

fn verify_manifest_binding(
    m: &SourceRecoveryManifest,
    request: &RestoreRequest<'_>,
) -> Result<(), RecoveryError> {
    if m.schema != RECOVERY_SCHEMA {
        return Err(RecoveryError::new(
            "recovery.stale",
            format!(
                "manifest schema {} is not {RECOVERY_SCHEMA}",
                m.schema
            ),
        ));
    }
    let source_sha = ace_core::runtime_contract::sha256_hex(request.source.as_bytes());
    if m.source_sha256 != source_sha {
        return Err(RecoveryError::new(
            "recovery.source-mismatch",
            "manifest binds a different source SHA-256 than the request's source".to_string(),
        ));
    }
    if m.source_len != request.source.len() {
        return Err(RecoveryError::new(
            "recovery.source-mismatch",
            format!(
                "manifest source length {} != request {}",
                m.source_len,
                request.source.len()
            ),
        ));
    }
    if m.logical_owner != request.logical_owner
        || m.entry != request.entry
        || m.profile != request.profile
    {
        return Err(RecoveryError::new(
            "recovery.context-mismatch",
            format!(
                "manifest context ({}/{}/{}) != request ({}/{}/{})",
                m.logical_owner, m.entry, m.profile, request.logical_owner, request.entry,
                request.profile
            ),
        ));
    }
    if m.capabilities != request.capabilities {
        return Err(RecoveryError::new(
            "recovery.capability",
            "manifest capabilities differ from the current host grants".to_string(),
        ));
    }
    if m.producer_mir_lowering_version != ace_lowering::MIR_LOWERING_VERSION
        || m.producer_closures_schema != ace_av_core::contract::MIR_CLOSURES_SCHEMA
        || m.owner_schema != ace_av_core::contract::MIR_CLOSURES_SCHEMA
        || m.abc_codec != ace_av_core::contract::ABC_CLOSURES_VERSION
        || m.mir_semantics != ace_av_core::contract::MIR_CLOSURES_SEMANTICS.0
        || m.abc_semantics != ace_av_core::contract::ABC_CLOSURES_SEMANTICS.0
        || m.av_semantics != ace_av_core::contract::AV_CLOSURES_SEMANTICS.0
    {
        return Err(RecoveryError::new(
            "recovery.stale",
            "manifest producer/semantics identity differs from the current toolchain"
                .to_string(),
        ));
    }
    Ok(())
}

/// The manifest envelope binds `snapshot` to the SOURCE SHA-256 so a
/// restoring process can derive it from the source it was given (never from
/// the store).
fn manifest_envelope_snapshot_of_source(source: &str) -> u64 {
    manifest_envelope_snapshot(&ace_core::runtime_contract::sha256_hex(source.as_bytes()))
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// A restored closures FUNCTION session: the accepted units installed and the
/// SAFE declaration context, ready for ONE more single-body edit. This is the
/// entry a restoring process uses; [`restore_source_recovery`] alone is the
/// execute-only face.
pub struct ClosuresRestoredSession {
    pub session: crate::closures_fn_session::ClosuresFunctionSession,
    pub manifest: SourceRecoveryManifest,
    pub counters: RecoveryCounters,
    /// The restored program's entry value (the restore EXECUTED it for real;
    /// the request never carried an expected value).
    pub restore_value: Option<i32>,
}

/// Restore a source-bound accepted state into a live FUNCTION session: install
/// the admitted units, rebuild the declaration context, and hand back a
/// session whose next `apply_source` is a normal 912 warm edit.
pub fn restore_source_session(
    store_root: &Path,
    request: &RestoreRequest<'_>,
) -> Result<ClosuresRestoredSession, RecoveryError> {
    let restored = restore_source_recovery(store_root, request)?;
    let RestoreOutcome {
        manifest,
        units,
        frontend,
        identity_wires,
        counters,
        ..
    } = restored;
    let mut update =
        crate::closures_session::ClosuresUpdateSession::new(None, &manifest.entry);
    update.set_host_grants(request.capabilities.to_vec());
    let outcome = update
        .restore_install(units)
        .map_err(|e| RecoveryError::new("recovery.install", e))?;
    let session = crate::closures_fn_session::ClosuresFunctionSession::from_restored(
        update,
        frontend,
        identity_wires,
    );
    Ok(ClosuresRestoredSession {
        session,
        manifest,
        counters,
        restore_value: outcome.value,
    })
}
