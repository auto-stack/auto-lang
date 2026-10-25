//! PLAN-821 owned driver: the `av.owned/1` whole-path request seam with
//! 783 managed-store persistence and a re-verification restore (mirrors
//! the 820 data driver's minimal layer; contract-decision §9/§13).
//!
//! Path: front door → `compile_text_record_source` (782 semantics at the
//! source gate) → `lower_checked_owned` → `verify_mir` (ownership
//! dataflow) → AVAD v2 encode → (store persist under the owned layer) →
//! `verify_abc_bytes` → execute. Restore NEVER trusts the disk.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{
    fnv1a64, ExecOutcome, ABC_OWNED_SCHEMA, AV_OWNED_PROFILE, MIR_OWNED_SCHEMA, OWNED_PRODUCER_ID,
};
use auto_av_core::mir_verify::verify_mir;
use std::path::Path;

use crate::cache::{envelope_unwrap, envelope_wrap, EnvelopeIdentity, EnvelopeKind};
use crate::hir_adapter::lower_checked_owned;
use ac_artifact_probe::identity::QueryKey;

/// The owned profile's ABC layer (A21): whole-module AVAD v2 artifacts
/// keyed on the content axes that invalidate them.
pub const OWNED_ABC_LAYER: LayerSpec = LayerSpec {
    id: "A21",
    name: "backend-object (plan821 owned-profile whole-module ABC)",
    schema_id: "ac821/a21-owned-abc/v1",
    producer: OWNED_PRODUCER_ID,
    consumers: &["av owned executor (restore re-verifies)"],
    key_inputs: &[
        "producer",
        "owner-module",
        "entry",
        "profile",
        "types-fp",
        "abc-codec",
        "abc-semantics",
        "mir-semantics",
        "av-semantics",
        "source-snapshot",
    ],
    ref_kinds: &["provenance"],
    storage: "one whole-module AVAD v2 artifact per source snapshot (LayoutData carrier)",
    residency: "restored artifacts re-verify (decode + deep admission) before execution",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

fn owned_envelope<'a>(owner: &'a str, entry: &'a str, snapshot: u64) -> EnvelopeIdentity<'a> {
    EnvelopeIdentity {
        kind: EnvelopeKind::AbcUnit,
        producer: OWNED_PRODUCER_ID,
        schema: ABC_OWNED_SCHEMA,
        owner_module: owner,
        stage: 2,
        mir_sem: 1,
        abc_sem: 1,
        av_sem: 1,
        snapshot,
        unit: entry,
        body_fp: "",
    }
}

fn owned_key_axes(
    owner: &str,
    entry: &str,
    types_fp: &str,
    snapshot: &str,
) -> Vec<(String, String)> {
    vec![
        ("producer".to_string(), OWNED_PRODUCER_ID.to_string()),
        ("owner-module".to_string(), owner.to_string()),
        ("entry".to_string(), entry.to_string()),
        ("profile".to_string(), AV_OWNED_PROFILE.to_string()),
        ("types-fp".to_string(), types_fp.to_string()),
        (
            "abc-codec".to_string(),
            format!(
                "{}/v{}",
                ABC_OWNED_SCHEMA,
                auto_av_core::contract::ABC_OWNED_VERSION
            ),
        ),
        (
            "abc-semantics".to_string(),
            auto_av_core::contract::ABC_OWNED_SEMANTICS.0.to_string(),
        ),
        (
            "mir-semantics".to_string(),
            auto_av_core::contract::MIR_OWNED_SEMANTICS.0.to_string(),
        ),
        (
            "av-semantics".to_string(),
            auto_av_core::contract::AV_OWNED_SEMANTICS.0.to_string(),
        ),
        ("source-snapshot".to_string(), snapshot.to_string()),
    ]
}

/// One owned-profile whole-path request outcome.
#[derive(Clone, Debug)]
pub struct OwnedRequestOutcome {
    pub av: Option<ExecOutcome>,
    pub steps: u64,
    pub artifact: Option<String>,
    pub owner: String,
    pub entry: String,
    pub snapshot: u64,
    pub wall_ms: f64,
}

/// Run one owned-profile request; persist the AVAD v2 module when
/// `store_root` is given. The AV result is independent of persistence.
pub fn request_owned(
    store_root: Option<&Path>,
    project: &str,
    src: &str,
    entry: &str,
    caps: &[String],
) -> Result<OwnedRequestOutcome, String> {
    let started = std::time::Instant::now();
    crate::driver::front_door_admit(src)
        .map_err(|d| format!("front door: {} {}", d.stage, d.code))?;
    let compiled =
        auto_ac_prototype::source::compile_text_record_source(src, "owned.at", caps).map_err(
            |ds| {
                ds.first()
                    .map(|d| format!("frontend: {} {}", d.code, d.message))
                    .unwrap_or_else(|| "frontend failed".to_string())
            },
        )?;
    let mir = lower_checked_owned(&compiled.checked, entry, fnv1a64(src.as_bytes())).map_err(
        |e| {
            format!(
                "lower: {} {}",
                e.code(),
                match e {
                    crate::hir_adapter::AdapterError::Unsupported { message, .. }
                    | crate::hir_adapter::AdapterError::Invalid { message, .. } => message,
                }
            )
        },
    )?;
    verify_mir(&mir).map_err(|e| format!("mir verify: {:?}", e))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode: {}", e.code()))?;
    let flat = wire.to_flat();

    let owner = mir.owner.module.clone();
    let snapshot = fnv1a64(src.as_bytes());
    let types_fp = types_fp_of(&mir);
    let mut artifact = None;
    if let Some(root) = store_root {
        let store = ManagedStore::open(root).map_err(|e| format!("open store: {e}"))?;
        let key = QueryKey::new(
            &OWNED_ABC_LAYER,
            owned_key_axes(&owner, entry, &types_fp, &format!("{snapshot:016x}")),
        );
        let env = envelope_wrap(&flat, &owned_envelope(&owner, entry, snapshot));
        let art = store
            .put_artifact(PutRequest {
                layer: OWNED_ABC_LAYER.id,
                key,
                kind: ChunkKind::LayoutData,
                payload: env,
                edges: vec![],
                project: Some(project.to_string()),
            })
            .map_err(|e| format!("persist: {e}"))?;
        artifact = Some(art);
    }

    let loaded = verify_abc_bytes(&flat, &mir.owner).map_err(|e| format!("abc admit: {e}"))?;
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    Ok(OwnedRequestOutcome {
        av: Some(out),
        steps: trace.execution_seq,
        artifact,
        owner,
        entry: entry.to_string(),
        snapshot,
        wall_ms: started.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Restore a persisted owned module from the store and EXECUTE it — the
/// new-worker seam: envelope admission, decode and the full deep
/// verification re-run; nothing on disk is trusted.
pub fn restore_owned(
    store_root: &Path,
    owner: &str,
    entry: &str,
    snapshot: u64,
    types_fp: &str,
) -> Result<ExecOutcome, String> {
    let store = ManagedStore::open(store_root).map_err(|e| format!("open store: {e}"))?;
    let key = QueryKey::new(
        &OWNED_ABC_LAYER,
        owned_key_axes(owner, entry, types_fp, &format!("{snapshot:016x}")),
    );
    let loaded_art = store
        .get_artifact(OWNED_ABC_LAYER.id, &key)
        .map_err(|e| format!("lookup: {e}"))?
        .ok_or_else(|| "owned module artifact not found (clean miss)".to_string())?;
    if loaded_art.kind != ChunkKind::LayoutData {
        return Err(format!(
            "artifact {} carries kind {:?}, expected LayoutData",
            loaded_art.artifact_id, loaded_art.kind
        ));
    }
    let payload = envelope_unwrap(&loaded_art.payload, &owned_envelope(owner, entry, snapshot))
        .map_err(|e| format!("envelope: {e}"))?;
    let owner_scope = auto_av_core::mir::OwnerScope {
        schema: MIR_OWNED_SCHEMA.to_string(),
        module: owner.to_string(),
    };
    let loaded = verify_abc_bytes(&payload, &owner_scope).map_err(|e| format!("re-verify: {e}"))?;
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    Ok(out)
}

/// The types-fingerprint axis of an owned module key.
pub fn types_fp_of(mir: &auto_av_core::mir::MirModule) -> String {
    format!("{:016x}", fnv1a64(format!("{:?}", mir.types).as_bytes()))
}
