//! PLAN-823 closures driver: the `av.closures/1` whole-path seam with
//! 783 managed-store persistence and a re-verification restore (mirrors
//! the 822 collections driver; contract-decision r2 §5).
//!
//! Path: programmatic typed fixture (UNTRUSTED builder output) →
//! `verify_mir` (the real gate) → AVAD v5 encode → (store persist under
//! the closures layer) → `verify_abc_bytes` → execute. Restore NEVER
//! trusts the disk; the cache persists CODE, never a live env (the env
//! rebuilds during execution — §5.2).
//!
//! First case (plan §5.2): `make` creates a closure capturing a CLONE of
//! an owned Str and a MOVE of a List<Note> (`Note { text: Str, n: i32 }`),
//! RETURNS it (creator frame exits — the env stays legal); `main` calls
//! the closure in a NEW invocation frame; the body reads both env fields
//! and MUTATES the mutable env field by pushing a note through the shared
//! list handle; `main` then DROPS the closure — the arena ledger's
//! alloc/free sequence and live-after-release=0 are the frozen oracle
//! axes.

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::schema::{LayerCoverage, LayerSpec, Regeneration};
use ac_artifact_probe::store::{ManagedStore, PutRequest};
use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{
    fnv1a64, ExecOutcome, ABC_CLOSURES_SCHEMA, AV_CLOSURES_PROFILE, CLOSURES_PRODUCER_ID,
    MIR_CLOSURES_SCHEMA,
};
use auto_av_core::mir::{
    CaptureKind, CaptureOperand, FnSig, ListTypeRef, MirConst, MirFieldDef, MirFunction, MirInstr,
    MirModule, MirSlot, MirType, OwnerScope, RecordTypeRef, SlotKind,
};
use auto_av_core::mir_verify::verify_mir;
use std::path::Path;

use crate::cache::{envelope_unwrap, envelope_wrap, EnvelopeIdentity, EnvelopeKind};
use ac_artifact_probe::identity::QueryKey;

/// The closures profile's ABC layer (A23): whole-module AVAD v5 artifacts
/// keyed on the content axes that invalidate them.
pub const CLOSURES_ABC_LAYER: LayerSpec = LayerSpec {
    id: "A23",
    name: "backend-object (plan823 closures-profile whole-module ABC)",
    schema_id: "ac823/a23-closures-abc/v1",
    producer: CLOSURES_PRODUCER_ID,
    consumers: &["av closures executor (restore re-verifies)"],
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
    storage: "one whole-module AVAD v5 artifact per fixture revision (LayoutData carrier)",
    residency: "restored artifacts re-verify (decode + deep admission) before execution",
    regeneration: Regeneration::FromUserInputs,
    coverage: LayerCoverage::Codec,
};

pub fn closures_owner_scope() -> OwnerScope {
    OwnerScope {
        schema: MIR_CLOSURES_SCHEMA.to_string(),
        module: "closures823".to_string(),
    }
}

pub fn closures_envelope<'a>(
    owner: &'a str,
    _entry: &'a str,
    snapshot: u64,
) -> EnvelopeIdentity<'a> {
    EnvelopeIdentity {
        kind: EnvelopeKind::AbcUnit,
        producer: CLOSURES_PRODUCER_ID,
        schema: ABC_CLOSURES_SCHEMA,
        owner_module: owner,
        stage: 2,
        mir_sem: auto_av_core::contract::MIR_CLOSURES_SEMANTICS.0,
        abc_sem: auto_av_core::contract::ABC_CLOSURES_SEMANTICS.0,
        av_sem: auto_av_core::contract::AV_CLOSURES_SEMANTICS.0,
        snapshot,
        unit: "",
        body_fp: "",
    }
}

fn closures_key_axes(
    owner: &str,
    entry: &str,
    types_fp: &str,
    snapshot: &str,
) -> Vec<(String, String)> {
    vec![
        ("producer".to_string(), CLOSURES_PRODUCER_ID.to_string()),
        ("owner-module".to_string(), owner.to_string()),
        ("entry".to_string(), entry.to_string()),
        ("profile".to_string(), AV_CLOSURES_PROFILE.to_string()),
        ("types-fp".to_string(), types_fp.to_string()),
        (
            "abc-codec".to_string(),
            format!(
                "{}/v{}",
                ABC_CLOSURES_SCHEMA,
                auto_av_core::contract::ABC_CLOSURES_VERSION
            ),
        ),
        (
            "abc-semantics".to_string(),
            auto_av_core::contract::ABC_CLOSURES_SEMANTICS.0.to_string(),
        ),
        (
            "mir-semantics".to_string(),
            auto_av_core::contract::MIR_CLOSURES_SEMANTICS.0.to_string(),
        ),
        (
            "av-semantics".to_string(),
            auto_av_core::contract::AV_CLOSURES_SEMANTICS.0.to_string(),
        ),
        ("source-snapshot".to_string(), snapshot.to_string()),
    ]
}

/// The types-fingerprint axis of a closures module key.
pub fn closures_types_fp_of(mir: &MirModule) -> String {
    format!("{:016x}", fnv1a64(format!("{:?}", mir.types).as_bytes()))
}

/// The store key of a closures module's ABC layer artifact.
pub fn closures_module_key(owner: &str, entry: &str, types_fp: &str, snapshot: u64) -> QueryKey {
    QueryKey::new(
        &CLOSURES_ABC_LAYER,
        closures_key_axes(owner, entry, types_fp, &format!("{snapshot:016x}")),
    )
}

/// The §5.2 first case: creator-returns, new-frame call, mutable env
/// field, drop. Type table: 0 i32, 1 Str, 2 Note{text:Str,n:i32},
/// 3 List<Note>, 4 Fn(i32)->i32, 5 Closure(4), 6 StrView, 7 Fn()->Closure.
pub fn first_case_module() -> MirModule {
    let owner = closures_owner_scope();
    let mut m = MirModule::new_closures(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Str,
        MirType::Record(RecordTypeRef {
            owner_module: "closures823".to_string(),
            name: "Note".to_string(),
            fields: vec![
                MirFieldDef {
                    name: "text".to_string(),
                    ty: 1,
                },
                MirFieldDef {
                    name: "n".to_string(),
                    ty: 0,
                },
            ],
        }),
        MirType::List(ListTypeRef { elem: 2 }),
        MirType::Fn(FnSig {
            params: vec![0],
            result: 0,
        }),
        MirType::Closure(FnSig {
            params: vec![0],
            result: 0,
        }),
        MirType::StrView,
        MirType::Fn(FnSig {
            params: vec![],
            result: 5,
        }),
    ];
    let c_auto = m.intern_const(MirConst::Str(Box::from(*b"auto")));
    let c_one = m.intern_const(MirConst::I32(1));
    let c_pushed = m.intern_const(MirConst::Str(Box::from(*b"pushed")));
    let c_five = m.intern_const(MirConst::I32(5));
    let c_ten = m.intern_const(MirConst::I32(10));

    let slot = |ty: u32, name: &str| MirSlot {
        kind: SlotKind::Local,
        ty,
        name: Some(name.to_string()),
    };
    let param = |ty: u32, name: &str| MirSlot {
        kind: SlotKind::Param,
        ty,
        name: Some(name.to_string()),
    };

    // make (func 0): () -> Closure — creates and RETURNS the closure.
    // slots: 0 text:Str (capture source), 1 n:i32, 2 text2:Str (the note's
    // own field source — AGGREGATE consumes owned operands, so the capture
    // source must be a SEPARATE slot), 3 note:Note, 4 list:List, 5 cl.
    let make = MirFunction {
        owner: owner.clone(),
        name: "make".to_string(),
        sig: 7,
        slots: vec![
            slot(1, "text"),
            slot(0, "n"),
            slot(1, "text2"),
            slot(2, "note"),
            slot(3, "list"),
            slot(5, "cl"),
        ],
        blocks: vec![auto_av_core::mir::MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_auto as u32,
                },
                MirInstr::Const {
                    dst: 1,
                    value: c_one as u32,
                },
                MirInstr::Const {
                    dst: 2,
                    value: c_auto as u32,
                },
                MirInstr::Aggregate {
                    dst: 3,
                    record_ty: 2,
                    operands: vec![2, 1],
                },
                MirInstr::ListNew {
                    dst: 4,
                    list_ty: 3,
                    cap: 2,
                },
                MirInstr::ListPush { root: 4, value: 3 },
                MirInstr::CreateClosure {
                    dst: 5,
                    callee: auto_av_core::mir::CalleeRef::Func(1),
                    captures: vec![
                        CaptureOperand {
                            kind: CaptureKind::Clone,
                            src: 0,
                        },
                        CaptureOperand {
                            kind: CaptureKind::Move,
                            src: 4,
                        },
                    ],
                },
            ],
            term: auto_av_core::mir::Terminator::Return { value: 5 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };

    // body (func 1, the closure body): (i32) -> i32. Reads both env fields,
    // mutates the mutable env field (ListPush through the shared handle),
    // returns len*10 + slen(env text) + bonus.
    // slots: 0 bonus(param), 1 env_text:Str, 2 env_list:List, 3 n:i32,
    //  4 note:Note, 5 text2:Str, 6 len:i32, 7 slen:i32, 8 ten:i32,
    //  9 mul:i32, 10 sum:i32, 11 view:StrView, 12 ret:i32
    let body = MirFunction {
        owner: owner.clone(),
        name: "body".to_string(),
        sig: 4,
        env: Some(vec![1, 3]), // r8 env contract: [Str (clone), List<Note> (move)]

        slots: vec![
            param(0, "bonus"),
            slot(1, "env_text"),
            slot(3, "env_list"),
            slot(0, "n"),
            slot(2, "note"),
            slot(1, "text2"),
            slot(0, "len"),
            slot(0, "slen"),
            slot(0, "ten"),
            slot(0, "mul"),
            slot(0, "sum"),
            slot(6, "view"),
            slot(0, "ret"),
        ],
        blocks: vec![auto_av_core::mir::MirBlock {
            instrs: vec![
                MirInstr::EnvGet { dst: 1, field: 0 },
                MirInstr::EnvGet { dst: 2, field: 1 },
                MirInstr::Const {
                    dst: 3,
                    value: c_one as u32,
                },
                MirInstr::Const {
                    dst: 5,
                    value: c_pushed as u32,
                },
                MirInstr::Aggregate {
                    dst: 4,
                    record_ty: 2,
                    operands: vec![5, 3],
                },
                // THE mutable env field: the shared handle mutates the
                // env's list (len 1 -> 2).
                MirInstr::ListPush { root: 2, value: 4 },
                MirInstr::ListLen { dst: 6, src: 2 },
                MirInstr::Const {
                    dst: 8,
                    value: c_ten as u32,
                },
                MirInstr::View { dst: 11, src: 1 },
                MirInstr::StrLen { dst: 7, view: 11 },
                MirInstr::Bin {
                    op: auto_av_core::mir::BinOp::MulI32,
                    dst: 9,
                    lhs: 6,
                    rhs: 8,
                },
                MirInstr::Bin {
                    op: auto_av_core::mir::BinOp::AddI32,
                    dst: 10,
                    lhs: 9,
                    rhs: 7,
                },
                MirInstr::Bin {
                    op: auto_av_core::mir::BinOp::AddI32,
                    dst: 12,
                    lhs: 10,
                    rhs: 0,
                },
            ],
            term: auto_av_core::mir::Terminator::Return { value: 12 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };

    // main (func 2, entry): () -> i32 — calls make (the closure ESCAPES the
    // creator frame), calls it in a fresh frame, drops it.
    // slots: 0 cl:Closure, 1 arg:i32, 2 ret:i32
    let main = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        })),
        slots: vec![slot(5, "cl"), slot(0, "arg"), slot(0, "ret")],
        blocks: vec![auto_av_core::mir::MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 1,
                    value: c_five as u32,
                },
                MirInstr::Call {
                    callee: auto_av_core::mir::CalleeRef::Func(0),
                    dst: 0,
                    args: vec![],
                },
                MirInstr::CallClosure {
                    dst: 2,
                    closure: 0,
                    args: vec![1],
                },
                MirInstr::Drop { slot: 0 },
            ],
            term: auto_av_core::mir::Terminator::Return { value: 2 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };

    m.funcs = vec![make, body, main];
    m.entry = 2;
    m
}

/// One frozen closures fixture (programmatic typed input — the builder
/// output is UNTRUSTED and must pass the real verify/ABC gates).
pub struct ClosuresFixture {
    pub id: &'static str,
}

/// The frozen fixture table (the worker validates ids; the PARENT judges
/// against its OWN frozen expectations — never the worker's word).
pub const CLOSURES_FIXTURES: &[ClosuresFixture] = &[ClosuresFixture { id: "first-case" }];

pub fn closures_fixture(id: &str) -> Option<&'static ClosuresFixture> {
    CLOSURES_FIXTURES.iter().find(|f| f.id == id)
}

/// The module of one fixture by id.
pub fn fixture_module(id: &str) -> Result<MirModule, String> {
    match id {
        "first-case" => Ok(first_case_module()),
        other => Err(format!("unknown closures fixture `{other}`")),
    }
}

/// One request outcome (the observability face of the whole path).
pub struct ClosuresRequestOutcome {
    pub av: Option<ExecOutcome>,
    pub artifact: Option<String>,
    pub owner: String,
    pub entry: String,
    pub snapshot: u64,
    pub arena: Option<auto_av_core::owned_text::StrArena>,
    pub type_names: std::collections::BTreeMap<u32, String>,
    pub types_fp: String,
    /// Managed-partition peaks (the 822 formula): wire + both module
    /// estimates (hot), plus the analysis bytes (staging), and the
    /// executor's live-slot high-water.
    pub peak_hot_bytes: u64,
    pub peak_staging_bytes: u64,
    pub peak_exec_slot_bytes: u64,
}

/// Produce one closures fixture through the REAL gates (the builder output
/// is untrusted), optionally persist the CODE artifact (never a live env),
/// and execute.
pub fn request_closures(
    store_root: Option<&Path>,
    project: &str,
    fixture_id: &str,
) -> Result<ClosuresRequestOutcome, String> {
    let mut mir = fixture_module(fixture_id)?;
    let snapshot = fnv1a64(fixture_id.as_bytes());
    mir.funcs[mir.entry as usize].source_snapshot = snapshot;
    verify_mir(&mir).map_err(|e| format!("mir verify: {:?}", e))?;
    let wire = encode_abc(&mir).map_err(|e| format!("encode: {}", e.code()))?;
    let flat = wire.to_flat();

    let owner = mir.owner.module.clone();
    let entry = mir.funcs[mir.entry as usize].name.clone();
    let types_fp = closures_types_fp_of(&mir);
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
            &CLOSURES_ABC_LAYER,
            closures_key_axes(&owner, &entry, &types_fp, &format!("{snapshot:016x}")),
        );
        let env = envelope_wrap(&flat, &closures_envelope(&owner, &entry, snapshot));
        let art = store
            .put_artifact(PutRequest {
                layer: CLOSURES_ABC_LAYER.id,
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
    // Managed-partition peaks (mirror the collections driver's formula).
    let hot_bytes = flat.len() + mir.heap_estimate() + loaded.module().heap_estimate();
    let staging_peak = (flat.len() + mir.heap_estimate())
        + loaded.stats().analysis_bytes as usize
        + loaded.module().heap_estimate();
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let arena = trace.arena.clone();
    Ok(ClosuresRequestOutcome {
        av: Some(out),
        artifact,
        owner,
        entry,
        snapshot,
        arena,
        type_names,
        types_fp,
        peak_hot_bytes: hot_bytes as u64,
        peak_staging_bytes: staging_peak as u64,
        peak_exec_slot_bytes: trace.peak_slot_bytes,
    })
}

/// Restore a persisted closures module from the store and EXECUTE it —
/// the new-worker seam: envelope admission, decode and the FULL deep
/// verification re-run; nothing on disk is trusted; the env rebuilds
/// during execution (code cache, never a live env).
pub fn restore_closures(
    store_root: &Path,
    owner: &str,
    entry: &str,
    snapshot: u64,
    types_fp: &str,
) -> Result<ExecOutcome, String> {
    let store = ManagedStore::open(store_root).map_err(|e| format!("open store: {e}"))?;
    let key = QueryKey::new(
        &CLOSURES_ABC_LAYER,
        closures_key_axes(owner, entry, types_fp, &format!("{snapshot:016x}")),
    );
    let loaded_art = store
        .get_artifact(CLOSURES_ABC_LAYER.id, &key)
        .map_err(|e| format!("lookup: {e}"))?
        .ok_or_else(|| "closures module artifact not found (clean miss)".to_string())?;
    if loaded_art.kind != ChunkKind::LayoutData {
        return Err(format!(
            "artifact {} carries kind {:?}, expected LayoutData",
            loaded_art.artifact_id, loaded_art.kind
        ));
    }
    let payload = envelope_unwrap(
        &loaded_art.payload,
        &closures_envelope(owner, entry, snapshot),
    )
    .map_err(|e| format!("envelope: {e}"))?;
    let owner_scope = OwnerScope {
        schema: MIR_CLOSURES_SCHEMA.to_string(),
        module: owner.to_string(),
    };
    // Bind the payload's ACTUAL wire profile (the F822-R16-01 rule): the
    // closures identity never executes foreign wire.
    let decoded = auto_av_core::abc::decode_abc(&payload).map_err(|e| format!("decode: {e:?}"))?;
    if decoded.profile != auto_av_core::contract::ABC_PROFILE_CLOSURES {
        return Err(format!(
            "payload profile mismatch: requested closures profile id {}, payload carries profile id {} — refusing to dispatch",
            auto_av_core::contract::ABC_PROFILE_CLOSURES,
            decoded.profile
        ));
    }
    let loaded = auto_av_core::abc_verify::verify_abc(decoded, &owner_scope)
        .map_err(|e| format!("re-verify: {e}"))?;
    // The requested `types_fp` is an IDENTITY axis of the store key: a
    // structurally legal foreign type table under an expected key is an
    // identity mismatch, rejected BEFORE dispatch (V-08).
    let actual_types_fp = closures_types_fp_of(loaded.module());
    if actual_types_fp != types_fp {
        return Err(format!(
            "payload types-fp mismatch: requested {types_fp}, payload carries {actual_types_fp} — refusing to dispatch"
        ));
    }
    // Bind the payload's ACTUAL entry identity (the F822-R15-01 rule).
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
/// `{ result, live_after_release, allocs: [{id,bytes}], frees: [{id,bytes}] }`.
pub fn run_closures_fixture(
    fixture_id: &str,
    store_root: Option<&Path>,
    project: &str,
) -> Result<serde_json::Value, String> {
    let mut outcome = request_closures(store_root, project, fixture_id)?;
    let av = outcome.av.take().expect("request_closures sets av");
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
