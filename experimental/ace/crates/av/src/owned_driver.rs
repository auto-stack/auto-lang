//! PLAN-821 owned driver: the `av.owned/1` whole-path request seam with
//! 783 managed-store persistence and a re-verification restore (mirrors
//! the 820 data driver's minimal layer; contract-decision §9/§13).
//!
//! Path: front door → `compile_text_record_source` (782 semantics at the
//! source gate) → `lower_checked_owned` → `verify_mir` (ownership
//! dataflow) → AVAD v2 encode → (store persist under the owned layer) →
//! `verify_abc_bytes` → execute. Restore NEVER trusts the disk.

use ace_artifacts::codec::ChunkKind;
use ace_artifacts::schema::{LayerCoverage, LayerSpec, Regeneration};
use ace_artifacts::store::{ManagedStore, PutRequest};
use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{
    fnv1a64, ExecOutcome, ABC_OWNED_SCHEMA, AV_OWNED_PROFILE, MIR_OWNED_SCHEMA, OWNED_PRODUCER_ID,
};
use ace_av_core::mir_verify::verify_mir;
use std::path::Path;

use crate::cache::{envelope_unwrap, envelope_wrap, EnvelopeIdentity, EnvelopeKind};
use ace_artifacts::identity::QueryKey;
use ace_lowering::lower_checked_owned;

#[inline]
fn observe_test_call<T>(stage: &str, f: impl FnOnce() -> T) -> T {
    #[cfg(feature = "test-case-batch")]
    {
        crate::test_observations::observed(stage, f)
    }
    #[cfg(not(feature = "test-case-batch"))]
    {
        let _ = stage;
        f()
    }
}

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
                ace_av_core::contract::ABC_OWNED_VERSION
            ),
        ),
        (
            "abc-semantics".to_string(),
            ace_av_core::contract::ABC_OWNED_SEMANTICS.0.to_string(),
        ),
        (
            "mir-semantics".to_string(),
            ace_av_core::contract::MIR_OWNED_SEMANTICS.0.to_string(),
        ),
        (
            "av-semantics".to_string(),
            ace_av_core::contract::AV_OWNED_SEMANTICS.0.to_string(),
        ),
        ("source-snapshot".to_string(), snapshot.to_string()),
    ]
}

/// One owned-profile whole-path request outcome. `arena` is the Rc-shared
/// ledger handle of THIS execution (PLAN-821 §5: the host-side release
/// oracle — drop the success value, then `arena.live()` must read 0);
/// `record_names` maps record type-table indices to their nominal names
/// for the canonical result face.
#[derive(Clone, Debug)]
pub struct OwnedRequestOutcome {
    pub av: Option<ExecOutcome>,
    pub steps: u64,
    pub artifact: Option<String>,
    pub owner: String,
    pub entry: String,
    pub snapshot: u64,
    pub wall_ms: f64,
    pub arena: Option<ace_av_core::owned_text::StrArena>,
    pub record_names: std::collections::BTreeMap<u32, String>,
    /// R3-F09: the managed-memory partition peaks, in the data pipeline's
    /// working-set dimension (driver.rs F798-R3-08) — `hot` is the loaded
    /// closure retained through execution (wire + decoded module
    /// structure), `staging` the transient codec/verification working
    /// set's peak, `exec_slot` the guest live-data peak (the AV's own
    /// slot-bytes accounting, V-07's 8MiB line). These are NOT disk
    /// usage and NOT the worker process's private-bytes peak.
    pub peak_hot_bytes: u64,
    pub peak_staging_bytes: u64,
    pub peak_exec_slot_bytes: u64,
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
    let compiled = observe_test_call("compile", || {
        ace_core::source::compile_text_record_source(src, "owned.at", caps)
    })
    .map_err(|ds| {
        ds.first()
            .map(|d| format!("frontend: {} {}", d.code, d.message))
            .unwrap_or_else(|| "frontend failed".to_string())
    })?;
    let mir = observe_test_call("lower", || {
        lower_checked_owned(&compiled.checked, entry, fnv1a64(src.as_bytes()))
    })
    .map_err(|e| {
        format!(
            "lower: {} {}",
            e.code(),
            match e {
                ace_lowering::AdapterError::Unsupported { message, .. }
                | ace_lowering::AdapterError::Invalid { message, .. } => message,
            }
        )
    })?;
    observe_test_call("verify_mir", || verify_mir(&mir))
        .map_err(|e| format!("mir verify: {:?}", e))?;
    let wire = observe_test_call("encode", || encode_abc(&mir))
        .map_err(|e| format!("encode: {}", e.code()))?;
    let flat = wire.to_flat();

    let owner = mir.owner.module.clone();
    let snapshot = fnv1a64(src.as_bytes());
    let types_fp = types_fp_of(&mir);
    let record_names = mir
        .types
        .iter()
        .enumerate()
        .filter_map(|(i, t)| match t {
            ace_av_core::mir::MirType::Record(r) => Some((i as u32, r.name.clone())),
            _ => None,
        })
        .collect();
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

    let loaded = observe_test_call("verify_abc", || verify_abc_bytes(&flat, &mir.owner))
        .map_err(|e| format!("abc admit: {e}"))?;
    // R3-F09: the managed-memory partition peaks, in the data pipeline's
    // working-set dimension — hot = the loaded closure retained through
    // execution (wire + MIR + decoded module), staging = the transient
    // codec/verification working set's peak (encode buffer, MIR
    // structure, decode + analysis), finalized AFTER the deep admit.
    let hot_bytes = flat.len() + mir.heap_estimate() + loaded.module().heap_estimate();
    let staging_peak = (flat.len() + mir.heap_estimate())
        + loaded.stats().analysis_bytes as usize
        + loaded.module().heap_estimate();
    let (out, trace) = observe_test_call("vm", || {
        execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false))
    });
    let arena = trace.arena.clone();
    Ok(OwnedRequestOutcome {
        av: Some(out),
        steps: trace.execution_seq,
        artifact,
        owner,
        entry: entry.to_string(),
        snapshot,
        wall_ms: started.elapsed().as_secs_f64() * 1000.0,
        arena,
        record_names,
        peak_hot_bytes: hot_bytes as u64,
        peak_staging_bytes: staging_peak as u64,
        peak_exec_slot_bytes: trace.peak_slot_bytes,
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
    let owner_scope = ace_av_core::mir::OwnerScope {
        schema: MIR_OWNED_SCHEMA.to_string(),
        module: owner.to_string(),
    };
    let loaded = verify_abc_bytes(&payload, &owner_scope).map_err(|e| format!("re-verify: {e}"))?;
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    Ok(out)
}

/// The types-fingerprint axis of an owned module key.
pub fn types_fp_of(mir: &ace_av_core::mir::MirModule) -> String {
    format!("{:016x}", fnv1a64(format!("{:?}", mir.types).as_bytes()))
}

/// The store key of an owned module's ABC layer artifact (the restore/
/// test seam — mirrors the data driver's `data_module_key`).
pub fn owned_module_key(owner: &str, entry: &str, types_fp: &str, snapshot: u64) -> QueryKey {
    QueryKey::new(
        &OWNED_ABC_LAYER,
        owned_key_axes(owner, entry, types_fp, &format!("{snapshot:016x}")),
    )
}
