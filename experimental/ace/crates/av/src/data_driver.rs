//! PLAN-820 data driver: the `av.data/1` whole-path request seam with 783
//! managed-store persistence and a re-verification restore (the driver's
//! core-i32 A08/A09 per-unit cascade stays untouched; this is the data
//! profile's own minimal layer until T-05's dependency coverage).
//!
//! Path: front door → `compile_record_source` → `lower_checked_data` →
//! `verify_mir` → AVAD encode → (store persist under the data layer) →
//! `verify_abc_bytes` → execute. Restore NEVER trusts the disk: the
//! envelope identity and the full decode/verify admission re-run before
//! any dispatch (contract-decision §9).

use ace_artifacts::codec::ChunkKind;
use ace_artifacts::schema::{LayerCoverage, LayerSpec, Regeneration};
use ace_artifacts::store::{ManagedStore, PutRequest};
use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{
    fnv1a64, AvSemanticsVersion, ExecOutcome, MirSemanticsVersion, ABC_DATA_SCHEMA,
    AV_DATA_PROFILE, DATA_PRODUCER_ID, MIR_DATA_SCHEMA,
};
use ace_av_core::mir_verify::verify_mir;
use std::path::Path;

use crate::cache::{envelope_unwrap, envelope_wrap, EnvelopeIdentity, EnvelopeKind};
use ace_artifacts::identity::QueryKey;
use ace_lowering::lower_checked_data;

/// The data profile's ABC layer (A20): whole-module AVAD artifacts keyed on
/// the content axes that invalidate them (profile, type-table fingerprint,
/// codec/semantic generations).
pub const DATA_ABC_LAYER: LayerSpec = LayerSpec {
    id: "A20",
    name: "backend-object (plan820 data-profile whole-module ABC)",
    schema_id: "ac820/a20-data-abc/v1",
    producer: DATA_PRODUCER_ID,
    consumers: &["av data executor (restore re-verifies)"],
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
    storage: "one whole-module AVAD artifact per source snapshot (LayoutData carrier)",
    residency: "restored artifacts re-verify (decode + deep admission) before execution",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

/// Envelope identity for one persisted data-module artifact.
fn data_envelope<'a>(owner: &'a str, entry: &'a str, snapshot: u64) -> EnvelopeIdentity<'a> {
    EnvelopeIdentity {
        kind: EnvelopeKind::AbcUnit,
        producer: DATA_PRODUCER_ID,
        schema: ABC_DATA_SCHEMA,
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

/// Layer key for one data module: content axes the artifact's validity
/// depends on. A type/layout/profile change produces a DIFFERENT key (the
/// old row misses and rebuilds — it is never read as the new shape).
pub fn data_module_key(owner: &str, entry: &str, mir: &ace_av_core::mir::MirModule) -> QueryKey {
    let types_fp = format!("{:016x}", fnv1a64(format!("{:?}", mir.types).as_bytes()));
    QueryKey::new(
        &DATA_ABC_LAYER,
        vec![
            ("producer".to_string(), DATA_PRODUCER_ID.to_string()),
            ("owner-module".to_string(), owner.to_string()),
            ("entry".to_string(), entry.to_string()),
            ("profile".to_string(), AV_DATA_PROFILE.to_string()),
            ("types-fp".to_string(), types_fp),
            (
                "abc-codec".to_string(),
                format!(
                    "{}/v{}",
                    ABC_DATA_SCHEMA,
                    ace_av_core::contract::ABC_DATA_VERSION
                ),
            ),
            (
                "abc-semantics".to_string(),
                ace_av_core::contract::ABC_DATA_SEMANTICS.0.to_string(),
            ),
            (
                "mir-semantics".to_string(),
                ace_av_core::contract::MIR_DATA_SEMANTICS.0.to_string(),
            ),
            (
                "av-semantics".to_string(),
                ace_av_core::contract::AV_DATA_SEMANTICS.0.to_string(),
            ),
            (
                "source-snapshot".to_string(),
                format!(
                    "{:016x}",
                    mir.funcs.first().map(|f| f.source_snapshot).unwrap_or(0)
                ),
            ),
        ],
    )
}

/// One data-profile whole-path request outcome.
#[derive(Clone, Debug)]
pub struct DataRequestOutcome {
    pub av: Option<ExecOutcome>,
    pub steps: u64,
    /// Artifact id of the persisted AVAD module (`None` without a store).
    pub artifact: Option<String>,
    pub owner: String,
    pub entry: String,
    pub snapshot: u64,
    pub wall_ms: f64,
}

/// Run one data-profile request; persist the AVAD module when `store_root`
/// is given. The AV execution result is independent of persistence.
pub fn request_data(
    store_root: Option<&Path>,
    project: &str,
    src: &str,
    entry: &str,
    caps: &[String],
) -> Result<DataRequestOutcome, String> {
    let started = std::time::Instant::now();
    crate::driver::front_door_admit(src)
        .map_err(|d| format!("front door: {} {}", d.stage, d.code))?;
    let compiled = ace_core::source::compile_record_source(src, "data.at", caps).map_err(|ds| {
        ds.first()
            .map(|d| format!("frontend: {} {}", d.code, d.message))
            .unwrap_or_else(|| "frontend failed".to_string())
    })?;
    let mir =
        lower_checked_data(&compiled.checked, entry, fnv1a64(src.as_bytes())).map_err(|e| {
            format!(
                "lower: {} {}",
                e.code(),
                match e {
                    ace_lowering::AdapterError::Unsupported { message, .. }
                    | ace_lowering::AdapterError::Invalid { message, .. } => message,
                }
            )
        })?;
    verify_mir(&mir).map_err(|e| format!("mir verify: {:?}", e))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode: {}", e.code()))?;
    let flat = wire.to_flat();

    let owner = mir.owner.module.clone();
    let snapshot = fnv1a64(src.as_bytes());
    let mut artifact = None;
    if let Some(root) = store_root {
        let store = ManagedStore::open(root).map_err(|e| format!("open store: {e}"))?;
        let key = data_module_key(&owner, entry, &mir);
        let env = envelope_wrap(&flat, &data_envelope(&owner, entry, snapshot));
        let art = store
            .put_artifact(PutRequest {
                layer: DATA_ABC_LAYER.id,
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
    Ok(DataRequestOutcome {
        av: Some(out),
        steps: trace.execution_seq,
        artifact,
        owner,
        entry: entry.to_string(),
        snapshot,
        wall_ms: started.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Restore a persisted data module from the store and EXECUTE it — the
/// new-worker seam (contract-decision §9): envelope admission, decode and
/// the full deep verification re-run; nothing on disk is trusted.
pub fn restore_data(
    store_root: &Path,
    owner: &str,
    entry: &str,
    snapshot: u64,
    types_fp: &str,
) -> Result<ExecOutcome, String> {
    let store = ManagedStore::open(store_root).map_err(|e| format!("open store: {e}"))?;
    let key = QueryKey::new(
        &DATA_ABC_LAYER,
        vec![
            ("producer".to_string(), DATA_PRODUCER_ID.to_string()),
            ("owner-module".to_string(), owner.to_string()),
            ("entry".to_string(), entry.to_string()),
            ("profile".to_string(), AV_DATA_PROFILE.to_string()),
            ("types-fp".to_string(), types_fp.to_string()),
            (
                "abc-codec".to_string(),
                format!(
                    "{}/v{}",
                    ABC_DATA_SCHEMA,
                    ace_av_core::contract::ABC_DATA_VERSION
                ),
            ),
            (
                "abc-semantics".to_string(),
                ace_av_core::contract::ABC_DATA_SEMANTICS.0.to_string(),
            ),
            (
                "mir-semantics".to_string(),
                ace_av_core::contract::MIR_DATA_SEMANTICS.0.to_string(),
            ),
            (
                "av-semantics".to_string(),
                ace_av_core::contract::AV_DATA_SEMANTICS.0.to_string(),
            ),
            ("source-snapshot".to_string(), format!("{snapshot:016x}")),
        ],
    );
    let loaded_art = store
        .get_artifact(DATA_ABC_LAYER.id, &key)
        .map_err(|e| format!("lookup: {e}"))?
        .ok_or_else(|| "data module artifact not found (clean miss)".to_string())?;
    if loaded_art.kind != ChunkKind::LayoutData {
        return Err(format!(
            "artifact {} carries kind {:?}, expected LayoutData",
            loaded_art.artifact_id, loaded_art.kind
        ));
    }
    let payload = envelope_unwrap(&loaded_art.payload, &data_envelope(owner, entry, snapshot))
        .map_err(|e| format!("envelope: {e}"))?;
    let owner_scope = ace_av_core::mir::OwnerScope {
        schema: MIR_DATA_SCHEMA.to_string(),
        module: owner.to_string(),
    };
    let loaded = verify_abc_bytes(&payload, &owner_scope).map_err(|e| format!("re-verify: {e}"))?;
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    Ok(out)
}

/// The types-fingerprint axis of a data module key (shared by the worker
/// CLI so callers can address the artifact they just persisted).
pub fn types_fp_of(mir: &ace_av_core::mir::MirModule) -> String {
    format!("{:016x}", fnv1a64(format!("{:?}", mir.types).as_bytes()))
}

const _: () = {
    // The data semantic-version constants keep their own axis (compile-time
    // assertion that the tuple types line up with the envelope fields).
    let _: MirSemanticsVersion = ace_av_core::contract::MIR_DATA_SEMANTICS;
    let _: AvSemanticsVersion = ace_av_core::contract::AV_DATA_SEMANTICS;
};
