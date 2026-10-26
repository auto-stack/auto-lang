//! PLAN-822 collections driver: the `av.collections/1` whole-path seam
//! with 783 managed-store persistence and a re-verification restore
//! (mirrors the 821 owned driver; contract-decision §1/§6).
//!
//! Path: programmatic typed fixture (UNTRUSTED builder output) →
//! `verify_mir` (the real gate — the builder is never Checked) → AVAD v3
//! encode → (store persist under the collections layer) →
//! `verify_abc_bytes` → execute. Restore NEVER trusts the disk.
//!
//! First case (contract-decision §5.2): List<Note> with `Note { text: Str,
//! n: i32 }`, a nested-Str field replace via LIST_FIELD_SET, an owned
//! LIST_REMOVE transfer, a Result-style VARIANT_PACK, a guard-then-project
//! read and explicit release — the arena ledger's alloc/free sequence and
//! live-after-release=0 are the frozen oracle axes.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{
    fnv1a64, ExecOutcome, ABC_COLLECTIONS_SCHEMA, AV_COLLECTIONS_PROFILE, COLLECTIONS_PRODUCER_ID,
    MIR_COLLECTIONS_SCHEMA,
};
use auto_av_core::mir::{
    ListTypeRef, MirConst, MirFieldDef, MirFunction, MirInstr, MirModule, MirSlot, MirType,
    OwnerScope, RecordTypeRef, VariantArm, VariantTypeRef,
};
use auto_av_core::mir_verify::verify_mir;
use std::path::Path;

use crate::cache::{envelope_unwrap, envelope_wrap, EnvelopeIdentity, EnvelopeKind};
use ac_artifact_probe::identity::QueryKey;

/// The collections profile's ABC layer (A22): whole-module AVAD v3
/// artifacts keyed on the content axes that invalidate them.
pub const COLLECTIONS_ABC_LAYER: LayerSpec = LayerSpec {
    id: "A22",
    name: "backend-object (plan822 collections-profile whole-module ABC)",
    schema_id: "ac822/a22-collections-abc/v1",
    producer: COLLECTIONS_PRODUCER_ID,
    consumers: &["av collections executor (restore re-verifies)"],
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
    storage: "one whole-module AVAD v3 artifact per fixture revision (LayoutData carrier)",
    residency: "restored artifacts re-verify (decode + deep admission) before execution",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

/// The owner scope every collections fixture module lives under.
pub fn collections_owner_scope() -> OwnerScope {
    OwnerScope {
        schema: MIR_COLLECTIONS_SCHEMA.to_string(),
        module: "collections822".to_string(),
    }
}

pub fn collections_envelope<'a>(
    owner: &'a str,
    _entry: &'a str,
    snapshot: u64,
) -> EnvelopeIdentity<'a> {
    EnvelopeIdentity {
        kind: EnvelopeKind::AbcUnit,
        producer: COLLECTIONS_PRODUCER_ID,
        schema: ABC_COLLECTIONS_SCHEMA,
        owner_module: owner,
        stage: 2,
        mir_sem: auto_av_core::contract::MIR_COLLECTIONS_SEMANTICS.0,
        abc_sem: auto_av_core::contract::ABC_COLLECTIONS_SEMANTICS.0,
        av_sem: auto_av_core::contract::AV_COLLECTIONS_SEMANTICS.0,
        snapshot,
        unit: "",
        body_fp: "",
    }
}

fn collections_key_axes(
    owner: &str,
    entry: &str,
    types_fp: &str,
    snapshot: &str,
) -> Vec<(String, String)> {
    vec![
        ("producer".to_string(), COLLECTIONS_PRODUCER_ID.to_string()),
        ("owner-module".to_string(), owner.to_string()),
        ("entry".to_string(), entry.to_string()),
        ("profile".to_string(), AV_COLLECTIONS_PROFILE.to_string()),
        ("types-fp".to_string(), types_fp.to_string()),
        (
            "abc-codec".to_string(),
            format!(
                "{}/v{}",
                ABC_COLLECTIONS_SCHEMA,
                auto_av_core::contract::ABC_COLLECTIONS_VERSION
            ),
        ),
        (
            "abc-semantics".to_string(),
            auto_av_core::contract::ABC_COLLECTIONS_SEMANTICS
                .0
                .to_string(),
        ),
        (
            "mir-semantics".to_string(),
            auto_av_core::contract::MIR_COLLECTIONS_SEMANTICS
                .0
                .to_string(),
        ),
        (
            "av-semantics".to_string(),
            auto_av_core::contract::AV_COLLECTIONS_SEMANTICS
                .0
                .to_string(),
        ),
        ("source-snapshot".to_string(), snapshot.to_string()),
    ]
}

/// The types-fingerprint axis of a collections module key.
pub fn collections_types_fp_of(mir: &MirModule) -> String {
    format!("{:016x}", fnv1a64(format!("{:?}", mir.types).as_bytes()))
}

/// The store key of a collections module's ABC layer artifact.
pub fn collections_module_key(owner: &str, entry: &str, types_fp: &str, snapshot: u64) -> QueryKey {
    QueryKey::new(
        &COLLECTIONS_ABC_LAYER,
        collections_key_axes(owner, entry, types_fp, &format!("{snapshot:016x}")),
    )
}

/// One frozen collections fixture (programmatic typed input — the builder
/// output is UNTRUSTED and must pass the real verify/ABC gates).
pub struct CollectionsFixture {
    pub id: &'static str,
    pub entry: &'static str,
}

pub const COLLECTIONS_FIXTURES: &[CollectionsFixture] = &[CollectionsFixture {
    id: "first-case",
    entry: "main",
}];

pub fn collections_fixture(id: &str) -> Option<&'static CollectionsFixture> {
    COLLECTIONS_FIXTURES.iter().find(|f| f.id == id)
}

/// The first-case module (contract-decision §5.2):
/// `Note { text: Str, n: i32 }`; `List<Note>`; `Outcome { Ok(Note), Err(Str) }`.
/// Body: aggregate → push → `list[0].text` replace → remove (owned
/// transfer) → `Outcome::Ok(note)` pack → guard → project → drop both.
pub fn first_case_module() -> MirModule {
    let owner = collections_owner_scope();
    // A COMPLETE module (not a unit): the fixture executes as an
    // executable closure through the standard admission.
    let mut m = MirModule::new_collections(owner.clone());
    let t_i32 = 0u32;
    let t_bool = 1u32;
    let t_str = 2u32;
    m.types = vec![
        MirType::I32,
        MirType::Bool,
        MirType::Str,
        MirType::Record(RecordTypeRef {
            owner_module: "collections822".to_string(),
            name: "Note".to_string(),
            fields: vec![
                MirFieldDef {
                    name: "text".to_string(),
                    ty: t_str,
                },
                MirFieldDef {
                    name: "n".to_string(),
                    ty: t_i32,
                },
            ],
        }),
        MirType::List(ListTypeRef { elem: 3 }),
        MirType::Variant(VariantTypeRef {
            owner_module: "collections822".to_string(),
            name: "Outcome".to_string(),
            arms: vec![
                VariantArm {
                    name: "Ok".to_string(),
                    payload: Some(3),
                },
                VariantArm {
                    name: "Err".to_string(),
                    payload: Some(t_str),
                },
            ],
        }),
    ];
    let _ = t_bool;
    let c_first = m.intern_const(MirConst::Str(Box::from(*b"first")));
    let c_one = m.intern_const(MirConst::I32(1));
    let c_replaced = m.intern_const(MirConst::Str(Box::from(*b"replaced")));
    let c_zero = m.intern_const(MirConst::I32(0));

    // slots: 0 s_text, 1 s_n, 2 s_note, 3 s_list, 4 s_idx0, 5 s_rep,
    //        6 s_removed, 7 s_outcome, 8 s_guard, 9 s_got, 10 s_ret
    let slot = |ty: u32, name: &str| MirSlot {
        kind: auto_av_core::mir::SlotKind::Local,
        ty,
        name: Some(name.to_string()),
    };
    let slots = vec![
        slot(t_str, "text"),
        slot(t_i32, "n"),
        slot(3, "note"),
        slot(4, "list"),
        slot(t_i32, "idx0"),
        slot(t_str, "rep"),
        slot(3, "removed"),
        slot(5, "outcome"),
        slot(t_bool, "guard"),
        slot(3, "got"),
        slot(t_i32, "ret"),
    ];
    let f = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![],
            result: t_i32,
        })),
        slots,
        blocks: vec![auto_av_core::mir::MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_first as u32,
                },
                MirInstr::Const {
                    dst: 1,
                    value: c_one as u32,
                },
                MirInstr::Aggregate {
                    dst: 2,
                    record_ty: 3,
                    operands: vec![0, 1],
                },
                MirInstr::ListNew {
                    dst: 3,
                    list_ty: 4,
                    cap: 2,
                },
                MirInstr::ListPush { root: 3, value: 2 },
                MirInstr::Const {
                    dst: 4,
                    value: c_zero as u32,
                },
                MirInstr::Const {
                    dst: 5,
                    value: c_replaced as u32,
                },
                MirInstr::ListFieldSet {
                    root: 3,
                    index: 4,
                    path: vec![0],
                    value: 5,
                },
                MirInstr::ListRemove {
                    dst: 6,
                    root: 3,
                    index: 4,
                },
                MirInstr::VariantPack {
                    dst: 7,
                    variant_ty: 5,
                    disc: 0,
                    payload: Some(6),
                },
                MirInstr::VariantGuard {
                    dst: 8,
                    src: 7,
                    disc: 0,
                },
                MirInstr::VariantProject {
                    dst: 9,
                    root: 7,
                    disc: 0,
                },
                MirInstr::Drop { slot: 9 },
                MirInstr::Drop { slot: 7 },
                MirInstr::Const {
                    dst: 10,
                    value: c_zero as u32,
                },
            ],
            term: auto_av_core::mir::Terminator::Return { value: 10 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };
    m.funcs = vec![f];
    m.entry = 0;
    m
}

/// The module of one fixture by id.
pub fn fixture_module(id: &str) -> Result<MirModule, String> {
    match id {
        "first-case" => Ok(first_case_module()),
        other => Err(format!("unknown collections fixture `{other}`")),
    }
}

/// One collections whole-path request outcome (mirrors the owned face).
#[derive(Clone, Debug)]
pub struct CollectionsRequestOutcome {
    pub av: Option<ExecOutcome>,
    pub steps: u64,
    pub artifact: Option<String>,
    pub owner: String,
    pub entry: String,
    pub snapshot: u64,
    pub wall_ms: f64,
    pub arena: Option<auto_av_core::owned_text::StrArena>,
    pub type_names: std::collections::BTreeMap<u32, String>,
    /// The types-fingerprint dependency axis (the restore seam's key).
    pub types_fp: String,
    /// R3-F09-style managed-memory partition peaks (informational budget
    /// evidence): `hot` = the loaded closure retained through execution,
    /// `staging` = the transient codec/verification working set's peak,
    /// `exec_slot` = the guest live-data peak (V-07's 8MiB line).
    pub peak_hot_bytes: u64,
    pub peak_staging_bytes: u64,
    pub peak_exec_slot_bytes: u64,
}

/// Run one collections fixture through the REAL pipeline; persist the
/// AVAD v3 module when `store_root` is given. The builder output is
/// UNTRUSTED: `verify_mir` is the gate that admits it.
pub fn request_collections(
    store_root: Option<&Path>,
    project: &str,
    fixture_id: &str,
) -> Result<CollectionsRequestOutcome, String> {
    let started = std::time::Instant::now();
    let mut mir = fixture_module(fixture_id)?;
    // The fixture id is the revision identity (no source text exists —
    // programmatic typed input, contract-decision §6).
    let snapshot = fnv1a64(fixture_id.as_bytes());
    mir.funcs[0].source_snapshot = snapshot;
    // The builder output is untrusted input: the REAL gates decide.
    verify_mir(&mir).map_err(|e| format!("mir verify: {:?}", e))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode: {}", e.code()))?;
    let flat = wire.to_flat();

    let owner = mir.owner.module.clone();
    let entry = mir.funcs[mir.entry as usize].name.clone();
    let types_fp = collections_types_fp_of(&mir);
    let type_names = mir
        .types
        .iter()
        .enumerate()
        .filter_map(|(i, t)| match t {
            MirType::Record(r) => Some((i as u32, r.name.clone())),
            MirType::Variant(v) => Some((i as u32, v.name.clone())),
            _ => None,
        })
        .collect();
    let mut artifact = None;
    if let Some(root) = store_root {
        let store = ManagedStore::open(root).map_err(|e| format!("open store: {e}"))?;
        let key = QueryKey::new(
            &COLLECTIONS_ABC_LAYER,
            collections_key_axes(&owner, &entry, &types_fp, &format!("{snapshot:016x}")),
        );
        let env = envelope_wrap(&flat, &collections_envelope(&owner, &entry, snapshot));
        let art = store
            .put_artifact(PutRequest {
                layer: COLLECTIONS_ABC_LAYER.id,
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
    // Managed partition peaks (mirror the owned driver's R3-F09 formula).
    let hot_bytes = flat.len() + mir.heap_estimate() + loaded.module().heap_estimate();
    let staging_peak = (flat.len() + mir.heap_estimate())
        + loaded.stats().analysis_bytes as usize
        + loaded.module().heap_estimate();
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let arena = trace.arena.clone();
    Ok(CollectionsRequestOutcome {
        av: Some(out),
        steps: trace.execution_seq,
        artifact,
        owner,
        entry,
        snapshot,
        wall_ms: started.elapsed().as_secs_f64() * 1000.0,
        arena,
        type_names,
        types_fp,
        peak_hot_bytes: hot_bytes as u64,
        peak_staging_bytes: staging_peak as u64,
        peak_exec_slot_bytes: trace.peak_slot_bytes,
    })
}

/// Restore a persisted collections module from the store and EXECUTE it —
/// the new-worker seam: envelope admission, decode and the FULL deep
/// verification re-run; nothing on disk is trusted.
pub fn restore_collections(
    store_root: &Path,
    owner: &str,
    entry: &str,
    snapshot: u64,
    types_fp: &str,
) -> Result<ExecOutcome, String> {
    let store = ManagedStore::open(store_root).map_err(|e| format!("open store: {e}"))?;
    let key = QueryKey::new(
        &COLLECTIONS_ABC_LAYER,
        collections_key_axes(owner, entry, types_fp, &format!("{snapshot:016x}")),
    );
    let loaded_art = store
        .get_artifact(COLLECTIONS_ABC_LAYER.id, &key)
        .map_err(|e| format!("lookup: {e}"))?
        .ok_or_else(|| "collections module artifact not found (clean miss)".to_string())?;
    if loaded_art.kind != ChunkKind::LayoutData {
        return Err(format!(
            "artifact {} carries kind {:?}, expected LayoutData",
            loaded_art.artifact_id, loaded_art.kind
        ));
    }
    let payload = envelope_unwrap(
        &loaded_art.payload,
        &collections_envelope(owner, entry, snapshot),
    )
    .map_err(|e| format!("envelope: {e}"))?;
    let owner_scope = OwnerScope {
        schema: MIR_COLLECTIONS_SCHEMA.to_string(),
        module: owner.to_string(),
    };
    let loaded = verify_abc_bytes(&payload, &owner_scope).map_err(|e| format!("re-verify: {e}"))?;
    // F822-R14-01: the re-run gates prove the payload SELF-consistent,
    // not that it is the module the caller asked for. The requested
    // `types_fp` is an IDENTITY axis of the store key: a structurally
    // legal foreign type table stored under an expected key (envelope
    // owner/entry/snapshot all matching) is an identity mismatch — the
    // payload's ACTUAL type-table fingerprint must equal the requested
    // one, rejected BEFORE dispatch (V-08: the mismatch never executes).
    let actual_types_fp = collections_types_fp_of(loaded.module());
    if actual_types_fp != types_fp {
        return Err(format!(
            "payload types-fp mismatch: requested {types_fp}, payload carries {actual_types_fp} — refusing to dispatch"
        ));
    }
    // F822-R15-01: bind the payload's ACTUAL entry identity — the envelope
    // never carried the entry (its builder drops the argument) and its
    // snapshot is outer metadata, not decoded content. A legal payload
    // renamed to a foreign entry, or stamped with a different
    // source_snapshot, stored under an expected key is an identity
    // mismatch: refuse BEFORE dispatch (V-08). (The owner axis needs no
    // content check: verify_abc_bytes RE-STAMPS the decoded owner from
    // the envelope — the payload cannot self-assert one.)
    let module = loaded.module();
    let entry_fn = &module.funcs[module.entry as usize];
    if entry_fn.name != entry {
        return Err(format!(
            "payload entry mismatch: requested `{entry}`, payload carries `{}` — refusing to dispatch",
            entry_fn.name
        ));
    }
    if entry_fn.source_snapshot != snapshot {
        return Err(format!(
            "payload source-snapshot mismatch: requested {:016x}, payload carries {:016x} — refusing to dispatch",
            snapshot, entry_fn.source_snapshot
        ));
    }
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    Ok(out)
}

/// Run one frozen fixture and render the judged face:
/// `{ result, live_after_release, allocs: [{id,bytes}], frees: [{id,bytes}] }`
/// (the canonical result uses `canonical_owned`, extended by containers).
pub fn run_collections_fixture(
    fixture_id: &str,
    store_root: Option<&Path>,
    project: &str,
) -> Result<serde_json::Value, String> {
    let mut outcome = request_collections(store_root, project, fixture_id)?;
    let av = outcome.av.take().expect("request_collections sets av");
    let ExecOutcome::Success { value, .. } = av else {
        return Err(format!("fixture did not succeed: {av:?}"));
    };
    let arena = outcome.arena.take().expect("execution attaches the ledger");
    let result = crate::owned_fixture::canonical_owned(&value, &outcome.type_names)
        .unwrap_or(serde_json::json!({ "note": "non-canonical" }));
    // Host-side release oracle: drop the success value, then live==0.
    drop(value);
    let live_after_release = arena.live() as u64;
    let allocs: Vec<serde_json::Value> = arena
        .alloc_events()
        .iter()
        .map(|a| serde_json::json!({ "id": a.id, "bytes": a.bytes }))
        .collect();
    let frees: Vec<serde_json::Value> = arena
        .free_events()
        .iter()
        .map(|f| serde_json::json!({ "id": f.id, "bytes": f.bytes }))
        .collect();
    Ok(serde_json::json!({
        "fixture": fixture_id,
        "result": result,
        "live_after_release": live_after_release,
        "allocs": allocs,
        "frees": frees,
    }))
}
