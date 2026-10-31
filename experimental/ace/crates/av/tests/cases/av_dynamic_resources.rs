//! PLAN-824 T-04 resources tests: the dynamic resource/failure boundaries —
//! live-data budget rejection BEFORE the write, the two allocator fault
//! seams (the arena ledger and the value-allocation hook), prior-event
//! preservation, both sides of the boundary, deep-chain limits, trap/cancel
//! cleanup, and the parent-sampled worker memory hard gate (V-06/V-07
//! axes; the precheck/commit/cleanup seams of the dynamic executor face).
//! Prefix `av_dynamic_824` (contract-decision §9).

use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::av::{execute, execute_with_arena, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, ResourceKind, TrapKind, Value, SLOT_CELL_BYTES};
use ace_av_core::dynamic::{DynType, DynValue, TraversalOptions};
use ace_av_core::mir::{
    FnSig, MirBlock, MirConst, MirFieldDef, MirFunction, MirInstr, MirModule, MirSlot, MirType,
    OwnerScope, SlotKind, Terminator, VariantArm, VariantTypeRef,
};
use ace_av_core::mir_verify::{verify_mir, MirVerifyError};
use ace_av_core::owned_text::StrArena;
use ace_av_core::portable::{freeze, unfreeze, RecordBinding, RestoreContext, VariantBinding};

fn owner_scope() -> OwnerScope {
    OwnerScope {
        schema: ace_av_core::contract::MIR_DYNAMIC_SCHEMA.to_string(),
        module: "dynamic824".to_string(),
    }
}

fn slot(ty: u32, name: &str) -> MirSlot {
    MirSlot {
        kind: SlotKind::Local,
        ty,
        name: Some(name.to_string()),
    }
}

/// The canonical dynamic module: note(text,n) → List<Note>[note] → two
/// packs (note + list) + take/field-get fold. Types:
/// `[0]=I32 [1]=Str [2]=Note{text,n} [3]=List<Note> [4]=Dyn`.
fn dyn_module() -> MirModule {
    let owner = owner_scope();
    let mut m = MirModule::new_dynamic(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Str,
        MirType::Record(ace_av_core::mir::RecordTypeRef {
            owner_module: "dynamic824".to_string(),
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
        MirType::List(ace_av_core::mir::ListTypeRef { elem: 2 }),
        MirType::Dyn,
    ];
    let c_text = m.intern_const(MirConst::Str(Box::from(*b"boundary"))) as u32;
    let c_n = m.intern_const(MirConst::I32(21)) as u32;
    let c_zero = m.intern_const(MirConst::I32(0)) as u32;
    // 0 text 1 n 2 note 3 d_note 4 note2 5 got 6 list 7 d_list 8 list2
    // 9 len 10 ret
    let slots = vec![
        slot(1, "text"),
        slot(0, "n"),
        slot(2, "note"),
        slot(4, "d_note"),
        slot(2, "note2"),
        slot(0, "got"),
        slot(3, "list"),
        slot(4, "d_list"),
        slot(3, "list2"),
        slot(0, "len"),
        slot(0, "ret"),
    ];
    m.funcs = vec![MirFunction {
        env: None,
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(ace_av_core::mir::FnSig {
            params: vec![],
            result: 0,
        })),
        slots,
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_text,
                },
                MirInstr::Const { dst: 1, value: c_n },
                MirInstr::Aggregate {
                    dst: 2,
                    record_ty: 2,
                    operands: vec![0, 1],
                },
                MirInstr::DynPack { dst: 3, src: 2 },
                MirInstr::DynTake {
                    dst: 4,
                    src: 3,
                    ty: 2,
                },
                MirInstr::FieldGet {
                    dst: 5,
                    base: 4,
                    path: vec![1],
                },
                MirInstr::ListNew {
                    dst: 6,
                    list_ty: 3,
                    cap: 1,
                },
                MirInstr::ListPush { root: 6, value: 2 },
                MirInstr::DynPack { dst: 7, src: 6 },
                MirInstr::DynTake {
                    dst: 8,
                    src: 7,
                    ty: 3,
                },
                MirInstr::ListLen { dst: 9, src: 8 },
                MirInstr::Bin {
                    op: ace_av_core::mir::BinOp::AddI32,
                    dst: 10,
                    lhs: 5,
                    rhs: 9,
                }, // 21 + 1
                MirInstr::Const {
                    dst: 10,
                    value: c_zero,
                },
            ],
            term: Terminator::Return { value: 10 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

/// The take-mismatch module (a legal module that TRAPS at runtime): pack a
/// Note, take it as List<Note>.
fn take_mismatch_module() -> MirModule {
    let owner = owner_scope();
    let mut m = MirModule::new_dynamic(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Str,
        MirType::Record(ace_av_core::mir::RecordTypeRef {
            owner_module: "dynamic824".to_string(),
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
        MirType::List(ace_av_core::mir::ListTypeRef { elem: 2 }),
        MirType::Dyn,
    ];
    let c_text = m.intern_const(MirConst::Str(Box::from(*b"mismatch"))) as u32;
    let c_n = m.intern_const(MirConst::I32(5)) as u32;
    let c_zero = m.intern_const(MirConst::I32(0)) as u32;
    m.funcs = vec![MirFunction {
        env: None,
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(ace_av_core::mir::FnSig {
            params: vec![],
            result: 0,
        })),
        slots: vec![
            slot(1, "text"),
            slot(0, "n"),
            slot(2, "note"),
            slot(4, "d"),
            slot(3, "as_list"),
            slot(0, "ret"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_text,
                },
                MirInstr::Const { dst: 1, value: c_n },
                MirInstr::Aggregate {
                    dst: 2,
                    record_ty: 2,
                    operands: vec![0, 1],
                },
                MirInstr::DynPack { dst: 3, src: 2 },
                MirInstr::DynTake {
                    dst: 4,
                    src: 3,
                    ty: 3, // List<Note> over a Note
                },
                MirInstr::Const {
                    dst: 5,
                    value: c_zero,
                },
            ],
            term: Terminator::Return { value: 5 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

fn admit(m: &MirModule) -> ace_av_core::abc_verify::LoadedModule {
    verify_mir(m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(m).unwrap_or_else(|e| panic!("encode: {}", e.code()));
    verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("admit: {e:?}"))
}

fn run_with(m: &MirModule, limits: &ExecLimits) -> (ExecOutcome, StrArena) {
    let loaded = admit(m);
    let (out, trace) = execute(&loaded, limits, &CancelFlag::new(false));
    (
        out,
        trace.arena.clone().expect("execute attaches the ledger"),
    )
}

fn resource_kind(out: &ExecOutcome) -> ResourceKind {
    match out {
        ExecOutcome::Resource { kind, .. } => *kind,
        other => panic!("expected a resource outcome, got {other:?}"),
    }
}

fn assert_clean_ledger(arena: &StrArena) {
    assert_eq!(
        arena.alloc_events().len(),
        arena.free_events().len(),
        "allocs {:?} vs frees {:?}",
        arena.alloc_events(),
        arena.free_events()
    );
    assert_eq!(arena.live(), 0);
}

/// V-07 budget-before-write: with a tight live-data budget the dynamic
/// module is rejected BEFORE any mutation crosses the line — a SlotBytes
/// resource (never a trap, never a silent success) and a CLEAN ledger.
#[test]
fn av_dynamic_resources_slot_bytes_rejects_before_write() {
    let m = dyn_module();
    let mut tight = ExecLimits::default();
    // Room for the text const + the note aggregate, not the list pack.
    tight.max_slot_bytes = 3 * SLOT_CELL_BYTES + "boundary".len();
    let (out, arena) = run_with(&m, &tight);
    assert_eq!(resource_kind(&out), ResourceKind::SlotBytes);
    assert_clean_ledger(&arena);
}

/// V-07 value-allocation fault seam: `fault_after_allocs` fails the Nth
/// value allocation (a slot write with a positive data delta) with an
/// Infra outcome BEFORE the write lands — nothing half-committed, the
/// ledger stays clean, and the SAME module under default limits succeeds
/// (the fault is the only delta).
#[test]
fn av_dynamic_resources_value_alloc_fault_no_half_commit() {
    let m = dyn_module();
    // The dynamic module's positive-delta writes: text const, note
    // aggregate, pack dst, list buffer writes, list pack... every seam k
    // in 1..=6 must fail as Infra with the same clean post-state.
    for k in 1..=6u64 {
        let mut limits = ExecLimits::default();
        limits.fault_after_allocs = Some(k);
        let (out, arena) = run_with(&m, &limits);
        match &out {
            ExecOutcome::Infra(msg) => assert!(
                msg.contains("fault-injected at value allocation"),
                "k={k}: {msg}"
            ),
            other => panic!("k={k}: expected the injected infra fault, got {other:?}"),
        }
        assert_clean_ledger(&arena);
    }
    // Default limits: the same module succeeds (fault is the only delta).
    let (out, arena) = run_with(&m, &ExecLimits::default());
    assert!(matches!(out, ExecOutcome::Success { .. }), "{out:?}");
    assert_clean_ledger(&arena);
}

/// V-07 arena allocator axis: the Nth ARENA allocation attempt fails —
/// the attempt EXISTS (attempts >= 1) with successful_alloc == 0, no alloc
/// events, live == 0 (never a trap, never a silent success).
#[test]
fn av_dynamic_resources_arena_fault_attempt_semantics() {
    let m = dyn_module();
    let loaded = admit(&m);
    let arena = StrArena::new();
    arena.set_fault_at(1);
    let (out, trace) = execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        arena,
    );
    let arena = trace.arena.clone().expect("execute attaches the ledger");
    match &out {
        ExecOutcome::Infra(msg) => assert!(msg.contains("fault-injected"), "{msg}"),
        other => panic!("expected the injected arena fault, got {other:?}"),
    }
    assert!(arena.attempts() >= 1, "the attempt exists");
    assert_eq!(arena.successful_allocs(), 0);
    assert_eq!(arena.alloc_events().len(), 0);
    assert_eq!(arena.live(), 0);
}

/// V-07 prior-event preservation: fault the SECOND arena attempt — the
/// FIRST (legal) allocation's events survive the failure, the failed
/// attempt added nothing, and the unwind freed the survivor (live == 0).
#[test]
fn av_dynamic_resources_arena_fault_preserves_prior_legal_events() {
    let m = dyn_module();
    let loaded = admit(&m);
    let arena = StrArena::new();
    arena.set_fault_at(2); // 1st = the text const materialization
    let (out, trace) = execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        arena,
    );
    let arena = trace.arena.clone().expect("execute attaches the ledger");
    assert!(
        matches!(&out, ExecOutcome::Infra(msg) if msg.contains("fault-injected")),
        "{out:?}"
    );
    assert!(arena.attempts() >= 2);
    assert_eq!(
        arena.successful_allocs(),
        1,
        "exactly the first attempt landed"
    );
    assert_eq!(
        arena.alloc_events().len(),
        1,
        "the prior legal event is preserved"
    );
    assert_eq!(arena.live(), 0, "the unwind freed the survivor");
    assert_eq!(arena.free_events().len(), 1);
}

/// V-07 the budget is the only delta: the SAME module under a tight budget
/// is a SlotBytes rejection with a clean ledger, under the default budget
/// a success.
#[test]
fn av_dynamic_resources_budget_is_the_only_delta() {
    let m = dyn_module();
    let mut tight = ExecLimits::default();
    tight.max_slot_bytes = 2 * SLOT_CELL_BYTES + "boundary".len();
    let (tight_out, tight_arena) = run_with(&m, &tight);
    assert_eq!(resource_kind(&tight_out), ResourceKind::SlotBytes);
    assert_clean_ledger(&tight_arena);

    let (wide_out, _) = run_with(&m, &ExecLimits::default());
    assert!(
        matches!(wide_out, ExecOutcome::Success { .. }),
        "{wide_out:?}"
    );
}

/// V-06/V-07 both sides of the boundary: the wide run learns the true
/// peak; at exactly the peak the module succeeds, one byte below it is a
/// SlotBytes rejection with a clean ledger.
#[test]
fn av_dynamic_resources_boundary_both_sides() {
    let m = dyn_module();
    let loaded = admit(&m);
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Success { .. }));
    let peak = trace.peak_slot_bytes;
    assert!(peak > 0);

    let mut at = ExecLimits::default();
    at.max_slot_bytes = peak as usize;
    let (out, arena) = run_with(&m, &at);
    assert!(
        matches!(out, ExecOutcome::Success { .. }),
        "at the peak: {out:?}"
    );
    assert_clean_ledger(&arena);

    let mut below = ExecLimits::default();
    below.max_slot_bytes = peak as usize - 1;
    let (out, arena) = run_with(&m, &below);
    assert_eq!(resource_kind(&out), ResourceKind::SlotBytes);
    assert_clean_ledger(&arena);
}

// ---------------------------------------------------------------------------
// V-06 — deep-chain limits at the profile surface
// ---------------------------------------------------------------------------

/// A record chain of `depth` levels (Rec_d{v:i32, r:Rec_{d-1}}) as a
/// dynamic module: pack → take → deepest field read. Returns the module.
fn chain_module(depth: u32) -> MirModule {
    let owner = owner_scope();
    let mut m = MirModule::new_dynamic(owner.clone());
    // [0]=I32 [1]=Str kept for uniformity; chain types at 2+d.
    m.types = vec![MirType::I32, MirType::Str];
    for d in 0..depth {
        let field_ty = if d == 0 { 0 } else { 2 + d - 1 };
        m.types
            .push(MirType::Record(ace_av_core::mir::RecordTypeRef {
                owner_module: "dynamic824".to_string(),
                name: format!("Rec{d}"),
                fields: vec![
                    MirFieldDef {
                        name: "v".to_string(),
                        ty: 0,
                    },
                    MirFieldDef {
                        name: "r".to_string(),
                        ty: field_ty,
                    },
                ],
            }));
    }
    m.types.push(MirType::Dyn); // index 2+depth: the erased container
    let dyn_ty = 2 + depth;
    let root_ty = 2 + depth - 1;
    let c_zero = m.intern_const(MirConst::I32(0)) as u32;
    let mut instrs = vec![MirInstr::Const {
        dst: 0,
        value: c_zero,
    }];
    let mut slots = vec![slot(0, "v"), slot(0, "ret")];
    // Build bottom-up: Rec0 at slot 2, Rec_d at slot 2+d.
    for d in 0..depth {
        let dst = 2 + d;
        let ops = if d == 0 {
            vec![0u32, 0]
        } else {
            vec![0, 2 + d - 1]
        };
        slots.push(slot(2 + d, &format!("rec{d}")));
        instrs.push(MirInstr::Aggregate {
            dst,
            record_ty: 2 + d,
            operands: ops,
        });
    }
    let d_slot = 2 + depth; // the Dyn slot
    let back = 2 + depth + 1; // the take-back slot
    slots.push(slot(dyn_ty, "d"));
    slots.push(slot(root_ty, "back"));
    instrs.push(MirInstr::DynPack {
        dst: d_slot,
        src: 2 + depth - 1,
    });
    instrs.push(MirInstr::DynTake {
        dst: back,
        src: d_slot,
        ty: root_ty,
    });
    instrs.push(MirInstr::FieldGet {
        dst: 1,
        base: back,
        path: vec![0],
    });
    m.funcs = vec![MirFunction {
        env: None,
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(ace_av_core::mir::FnSig {
            params: vec![],
            result: 0,
        })),
        slots,
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 1 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

/// V-06 deep chain WITHIN the bound: a depth-255 record chain packs,
/// freezes, unfreezes and takes with byte-identical round trip (the depth
/// budget holds at every layer: type walk, pack, freeze, unfreeze).
#[test]
fn av_dynamic_resources_deep_chain_roundtrip_within_bound() {
    const DEPTH: u32 = 255;
    let m = chain_module(DEPTH);
    let loaded = admit(&m);
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, .. } = out else {
        panic!("deep chain executes: {out:?}")
    };
    assert_eq!(value, Value::I32(0));

    // The chain DynType ladder (module-side) + a matching decode domain
    // (all 255 nominal bindings).
    let mut tys: Vec<DynType> = Vec::new();
    let leaf = DynType::record(
        "dynamic824",
        "Rec0",
        vec![("v", DynType::I32), ("r", DynType::I32)],
    );
    tys.push(leaf);
    for d in 1..DEPTH {
        let prev = tys[d as usize - 1].clone();
        tys.push(DynType::record(
            "dynamic824",
            &format!("Rec{d}"),
            vec![("v", DynType::I32), ("r", prev)],
        ));
    }

    // A Dyn of the full chain: pack the module-side value directly.
    let mut value = Value::I32(0);
    for d in 0..DEPTH {
        value = Value::Record(std::rc::Rc::new(ace_av_core::contract::RecordValue {
            ty: 2 + d,
            fields: vec![Value::I32(0), value],
        }));
    }
    let packed =
        DynValue::pack(value, tys[(DEPTH - 1) as usize].clone()).expect("pack the deep chain");
    let bytes = freeze(&packed, &TraversalOptions::default()).expect("freeze the deep chain");

    let names: Vec<String> = (0..DEPTH).map(|d| format!("Rec{d}")).collect();
    let recs: Vec<RecordBinding> = names
        .iter()
        .enumerate()
        .map(|(d, n)| RecordBinding {
            owner: "dynamic824",
            name: n,
            fields: vec![
                ("v", std::rc::Rc::new(DynType::I32)),
                (
                    "r",
                    if d == 0 {
                        std::rc::Rc::new(DynType::I32)
                    } else {
                        std::rc::Rc::new(tys[d - 1].clone())
                    },
                ),
            ],
            ty_index: 2 + d as u32,
        })
        .collect();
    let vars: Vec<VariantBinding> = Vec::new();
    let ctx = RestoreContext::new(&recs, &vars);
    let back =
        unfreeze(&bytes, &ctx, &TraversalOptions::default()).expect("unfreeze the deep chain");
    assert_eq!(back, packed, "round-tripped chain equals the source");
    // take + deepest-leaf read still works on the restored container.
    let taken = back
        .take_as(&tys[(DEPTH - 1) as usize])
        .expect("take the chain");
    let mut cur = taken;
    for _ in 0..DEPTH - 1 {
        match cur {
            Value::Record(rec) => {
                cur = rec.fields[1].clone();
            }
            other => panic!("not a record level: {other:?}"),
        }
    }
    assert!(matches!(cur, Value::Record(_)), "the leaf record survives");
}

/// V-06 deep chain BEYOND the bound: a depth-258 chain is refused at the
/// TYPE WALK (admission) — the whitelist depth guard fires before any
/// instruction executes.
#[test]
fn av_dynamic_resources_deep_chain_beyond_bound_rejected() {
    let m = chain_module(258);
    match verify_mir(&m) {
        Err(MirVerifyError::Unsupported { code, .. })
        | Err(MirVerifyError::Invalid { code, .. }) => {
            assert_eq!(code, "mir.dyn.whitelist");
        }
        other => panic!("depth+1 chain must be a whitelist rejection: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// V-07 — trap/cancel cleanup states
// ---------------------------------------------------------------------------

/// V-07 cleanup on trap: the wrong-type take traps (a CHECKED runtime
/// failure), the all-exit cleanup releases every frame — the ledger drains
/// to zero with allocs == frees (no leak past a trap).
#[test]
fn av_dynamic_resources_take_trap_drains_ledger() {
    let m = take_mismatch_module();
    let loaded = admit(&m);
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    match &out {
        ExecOutcome::Trap { kind, .. } => assert_eq!(*kind, TrapKind::DynTypeMismatch),
        other => panic!("expected the dyn-type trap, got {other:?}"),
    }
    let arena = trace.arena.expect("execute attaches the ledger");
    assert_clean_ledger(&arena);
}

/// V-07 cancel is its OWN state (not invalid, not a trap): a pre-set
/// cancel flag surfaces as `Cancelled` (the probe fires at the first
/// ≥1024-step poll of a long dynamic module) with a drained ledger.
#[test]
fn av_dynamic_resources_cancel_is_its_own_state() {
    // A chain module (within the type bound) padded past the 1024-step
    // poll boundary with trivial arithmetic.
    let mut m = chain_module(255);
    m.funcs[0].blocks[0].instrs.extend(
        std::iter::repeat(MirInstr::Bin {
            op: ace_av_core::mir::BinOp::AddI32,
            dst: 1,
            lhs: 1,
            rhs: 1,
        })
        .take(900),
    );
    let loaded = admit(&m);
    let flag = CancelFlag::new(true);
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &flag);
    assert!(matches!(out, ExecOutcome::Cancelled), "{out:?}");
    let arena = trace.arena.expect("execute attaches the ledger");
    assert_clean_ledger(&arena);
}

// ---------------------------------------------------------------------------
// V-07 — the worker memory hard gate (parent-sampled)
// ---------------------------------------------------------------------------

struct DynamicMemoryEvidence {
    parent_sampled_private: u64,
    self_reported_private: u64,
    live_after_release: u64,
    portable_roundtrip: bool,
}

fn sampled_dynamic_request(
    worker: &std::path::Path,
    store: &std::path::Path,
    fixture: &str,
) -> (DynamicMemoryEvidence, bool) {
    use std::process::{Command, Stdio};
    let child = Command::new(worker)
        .args([
            "dynamic-request",
            "--store",
            store.to_str().unwrap(),
            "--fixture",
            fixture,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn dynamic worker");
    let sampler = ace_av::mempeek::ChildPeakSampler::start(child.id(), 5)
        .expect("the parent samples the child");
    let out = child.wait_with_output().expect("child exits");
    let (peak, samples) = sampler.stop();
    assert!(samples > 0, "the sampler actually polled ({samples})");
    let line = String::from_utf8_lossy(&out.stdout);
    let payload: serde_json::Value =
        serde_json::from_str(line.trim()).expect("the worker prints one JSON line");
    (
        DynamicMemoryEvidence {
            parent_sampled_private: peak,
            self_reported_private: payload["private_bytes_peak"].as_u64().unwrap_or(0),
            live_after_release: payload["dynamic"]["live_after_release"]
                .as_u64()
                .unwrap_or(u64::MAX),
            portable_roundtrip: payload["dynamic"]["portable_roundtrip"] == serde_json::json!(true),
        },
        out.status.success(),
    )
}

/// V-07 worker hard gate: fresh release workers over the DYNAMIC fixtures
/// (the dyn-out round exercises the portable freeze/restore too) stay
/// under the 798 B-line 128 MiB envelope as sampled BY THE PARENT, agree
/// with their own private-bytes counter, and end drained (live == 0).
#[test]
fn av_dynamic_resources_worker_memory_hard_gate_parent_sampled() {
    let ws = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .expect("ACE workspace root")
        .to_path_buf();
    let mut owned_compiler = std::process::Command::new(env!("CARGO"));
    owned_compiler
        .args(["build", "--release", "-p", "ace-av", "--bin", "av-worker"])
        .current_dir(&ws)
        .env_remove("CARGO_TARGET_DIR")
        .arg("--target-dir")
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let build = auto_tool_execution::run_command_output(owned_compiler).expect("build");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let worker = ws.join("target").join("release").join(if cfg!(windows) {
        "av-worker.exe"
    } else {
        "av-worker"
    });

    let root = std::env::temp_dir().join(format!(
        "av824-memgate-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let store = root.join("store");

    const B_LINE: u64 = 128 * 1024 * 1024;
    let mut evidence = Vec::new();
    for (i, fixture) in ["first-case", "dyn-out", "first-case", "dyn-out"]
        .into_iter()
        .enumerate()
    {
        let (ev, ok) = sampled_dynamic_request(&worker, &store, fixture);
        assert!(ok, "sample {i} ({fixture}) runs green");
        evidence.push((fixture.to_string(), ev));
    }
    for (i, (fixture, e)) in evidence.iter().enumerate() {
        assert!(
            e.parent_sampled_private > 0,
            "sample {i}: a real process peak was measured"
        );
        assert!(
            e.parent_sampled_private <= B_LINE,
            "sample {i} ({fixture}): parent-sampled private-bytes peak {} > {}",
            e.parent_sampled_private,
            B_LINE
        );
        assert!(
            e.self_reported_private <= B_LINE,
            "sample {i} ({fixture}): self-reported private-bytes peak {} > {}",
            e.self_reported_private,
            B_LINE
        );
        assert_eq!(
            e.live_after_release, 0,
            "sample {i} ({fixture}): the ledger drains to zero"
        );
        if fixture == "dyn-out" {
            assert!(
                e.portable_roundtrip,
                "sample {i}: the portable round stays green under the gate"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// F-824-R1-03 replay shape: a compact SHARED type DAG — k nested two-arm
/// variants, both arms referencing the PRECEDING type. The module is a
/// few hundred bytes and well under the 256 depth bound; the owned-tree
/// EXPANSION is 2^(k+1)-1 DynType nodes (k=17 → 262143, the review's
/// exact counterexample shape).
fn dag_module(k: u32) -> MirModule {
    let owner = owner_scope();
    let mut m = MirModule::new_dynamic(owner);
    let i = m.intern_type(MirType::I32);
    let b = m.intern_type(MirType::Bool);
    let d = m.intern_type(MirType::Dyn);
    let mut t = i;
    for j in 0..k {
        t = m.intern_type(MirType::Variant(VariantTypeRef {
            owner_module: "dynamic824".to_string(),
            name: format!("V{j}"),
            arms: vec![
                VariantArm {
                    name: "a".to_string(),
                    payload: Some(t),
                },
                VariantArm {
                    name: "b".to_string(),
                    payload: Some(t),
                },
            ],
        }));
    }
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: b,
    }));
    let c = m.intern_const(MirConst::I32(1)) as u32;
    m.funcs = vec![MirFunction {
        env: None,
        owner: owner_scope(),
        name: "main".to_string(),
        sig,
        slots: vec![slot(i, "v"), slot(d, "dyn"), slot(b, "hit")],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 0, value: c },
                MirInstr::DynPack { dst: 1, src: 0 },
                MirInstr::DynIs {
                    dst: 2,
                    src: 1,
                    ty: t,
                },
            ],
            term: Terminator::Return { value: 2 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    }];
    m.entry = 0;
    m
}

/// F-824-R1-03 close: type-DAG expansion is bounded AND charged. The
/// compact sibling one level UNDER the ceiling (2^16-1 = 65535 nodes)
/// executes with its resolution work visible in `steps`; one level OVER
/// (131071) and the review's 17-level shape (262143, ~20 MiB of DynType
/// storage alone) are refused by the node or tighter byte budget BEFORE the
/// allocation storm — a tight step budget refuses with the step resource,
/// never "Success at 8 steps".
#[test]
fn av_dynamic_resources_type_dag_expansion_bounded_and_charged() {
    // Under the ceiling: 65535 expanded nodes — success, work charged.
    let loaded = admit(&dag_module(15));
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, steps } = out else {
        panic!("the 65535-node sibling must execute: {out:?}")
    };
    assert_eq!(value, Value::Bool(false), "i32 is not the variant type");
    assert!(
        steps >= 65_535,
        "the expansion is charged to the step budget: {steps} steps"
    );

    // One level over: 131071 nodes — resource refusal before
    // materialization (verify still admits: the module is legal).
    let loaded = admit(&dag_module(16));
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert_eq!(resource_kind(&out), ResourceKind::DynTypeNodes);
    assert_eq!(resource_kind(&out).code(), "av.resource.dyn-nodes");

    // The review's exact counterexample shape: a ~700-byte legal module
    // under a tight budget that used to report Success/8 steps/160B peak.
    let m = dag_module(17);
    verify_mir(&m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(&m).unwrap_or_else(|e| panic!("encode: {}", e.code()));
    assert!(
        wire.to_flat().len() < 1024,
        "the module stays compact: {} bytes",
        wire.to_flat().len()
    );
    let loaded =
        verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("admission: {e:?}"));
    let mut lim = ExecLimits::default();
    lim.max_steps = 100;
    lim.max_slot_bytes = 4096;
    let (out, trace) = execute(&loaded, &lim, &CancelFlag::new(false));
    // The byte budget can now stop the expansion before the node limit.
    assert_eq!(resource_kind(&out), ResourceKind::SlotBytes);
    assert!(
        trace.peak_slot_bytes <= 4096,
        "no storm lands in the slots: peak {}",
        trace.peak_slot_bytes
    );

    // The step charge is independently observable: an expansion larger
    // than the step budget (65535 nodes vs 100 steps) refuses with the
    // INSTR-STEPS resource — the old invisible-work path is gone.
    let loaded = admit(&dag_module(15));
    let mut lim = ExecLimits::default();
    lim.max_steps = 100;
    let (out, _) = execute(&loaded, &lim, &CancelFlag::new(false));
    assert_eq!(resource_kind(&out), ResourceKind::InstrSteps);
}

#[test]
fn av_dynamic_resources_r2_long_names_are_accounted_before_expansion() {
    let mut m = dag_module(15);
    for ty in &mut m.types {
        if let MirType::Variant(v) = ty {
            v.name.push_str(&"x".repeat(1024));
        }
    }
    let wire = encode_abc(&m).unwrap().to_flat();
    assert!(wire.len() < 17_000);
    let loaded = verify_abc_bytes(&wire, &m.owner).unwrap();
    let limits = ExecLimits {
        max_steps: 1_000_000,
        max_slot_bytes: 4096,
        ..ExecLimits::default()
    };
    let (out, trace) = execute(&loaded, &limits, &CancelFlag::new(false));
    assert_eq!(resource_kind(&out), ResourceKind::SlotBytes);
    assert!(trace.peak_slot_bytes > 240 && trace.peak_slot_bytes <= 4096);
    let default_limits = ExecLimits {
        max_steps: 1_000_000,
        ..ExecLimits::default()
    };
    let (out, trace) = execute(&loaded, &default_limits, &CancelFlag::new(false));
    assert_eq!(resource_kind(&out), ResourceKind::SlotBytes);
    assert!(
        trace.peak_slot_bytes > 4096
            && trace.peak_slot_bytes <= default_limits.max_slot_bytes as u64
    );
    let control = admit(&dag_module(3));
    let (out, trace) = execute(&control, &limits, &CancelFlag::new(false));
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::Bool(false),
            ..
        }
    ));
    assert!(trace.peak_slot_bytes > 240 && trace.peak_slot_bytes <= 4096);
}

#[test]
fn av_dynamic_resources_r2_binding_long_names_are_bounded() {
    let mut m = dag_module(13);
    for ty in &mut m.types {
        if let MirType::Variant(v) = ty {
            v.name.push_str(&"x".repeat(1024));
        }
    }
    let err = match ace_av::dynamic_fixture::portable_bindings_of(&m) {
        Ok(_) => panic!("expanded binding metadata must be budgeted"),
        Err(err) => err,
    };
    assert!(err.contains("dyn budget"), "{err}");
    assert!(ace_av::dynamic_fixture::portable_bindings_of(&dag_module(3)).is_ok());
}

#[test]
fn av_dynamic_resources_r2_pack_take_type_operands_bounded() {
    for operation in ["pack", "take"] {
        for large in [false, true] {
            let mut m = dag_module(if large { 15 } else { 3 });
            if large {
                for ty in &mut m.types {
                    if let MirType::Variant(v) = ty {
                        v.name.push_str(&"x".repeat(1024));
                    }
                }
            }
            let ty = match m.funcs[0].blocks[0].instrs[2] {
                MirInstr::DynIs { ty, .. } => ty,
                _ => unreachable!(),
            };
            if operation == "pack" {
                let MirType::Variant(v) = &mut m.types[ty as usize] else {
                    unreachable!()
                };
                v.arms.push(VariantArm {
                    name: "empty".into(),
                    payload: None,
                });
            }
            let c_bool = m.intern_const(MirConst::Bool(false)) as u32;
            m.funcs[0].slots.push(slot(ty, "variant"));
            if operation == "pack" {
                m.funcs[0].blocks[0].instrs = vec![
                    MirInstr::VariantPack {
                        dst: 3,
                        variant_ty: ty,
                        disc: 2,
                        payload: None,
                    },
                    MirInstr::DynPack { dst: 1, src: 3 },
                    MirInstr::Const {
                        dst: 2,
                        value: c_bool,
                    },
                ];
            } else {
                m.funcs[0].blocks[0].instrs[2] = MirInstr::Const {
                    dst: 2,
                    value: c_bool,
                };
                m.funcs[0].blocks[0]
                    .instrs
                    .push(MirInstr::DynTake { dst: 3, src: 1, ty });
            }
            let loaded = admit(&m);
            let limits = ExecLimits {
                max_steps: 1_000_000,
                max_slot_bytes: 4096,
                ..ExecLimits::default()
            };
            let (out, trace) = execute(&loaded, &limits, &CancelFlag::new(false));
            if large {
                assert_eq!(resource_kind(&out), ResourceKind::SlotBytes, "{operation}");
                assert!(trace.peak_slot_bytes <= 4096);
            } else if operation == "pack" {
                assert!(
                    matches!(
                        out,
                        ExecOutcome::Success {
                            value: Value::Bool(false),
                            ..
                        }
                    ),
                    "{out:?}"
                );
            } else {
                assert!(
                    matches!(
                        out,
                        ExecOutcome::Trap {
                            kind: TrapKind::DynTypeMismatch,
                            ..
                        }
                    ),
                    "{out:?}"
                );
            }
        }
    }
}

#[test]
fn av_dynamic_resources_r2_cancel_during_type_expansion() {
    let loaded = admit(&dag_module(15));
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(true));
    assert!(matches!(out, ExecOutcome::Cancelled), "{out:?}");
    assert!(trace.peak_slot_bytes > 240);
    assert_clean_ledger(&trace.arena.unwrap());
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::Bool(false),
                ..
            }
        ),
        "{out:?}"
    );
}

#[test]
fn av_dynamic_resources_r2_binding_skips_nonerasable_nominal() {
    let mut m = dag_module(3);
    let fn_ty = m.funcs[0].sig;
    m.intern_type(MirType::Record(ace_av_core::mir::RecordTypeRef {
        owner_module: "dynamic824".into(),
        name: "Unsupported".into(),
        fields: vec![MirFieldDef {
            name: "callback".into(),
            ty: fn_ty,
        }],
    }));
    let (records, variants) = ace_av::dynamic_fixture::portable_bindings_of(&m).unwrap();
    assert!(!records.iter().any(|r| r.name == "Unsupported"));
    assert_eq!(variants.len(), 3);
}

#[test]
fn av_dynamic_resources_r2_pack_admission_consumes_fuel() {
    let mut m = dag_module(1);
    let ty = match m.funcs[0].blocks[0].instrs[2] {
        MirInstr::DynIs { ty, .. } => ty,
        _ => unreachable!(),
    };
    let MirType::Variant(v) = &mut m.types[ty as usize] else {
        unreachable!()
    };
    v.arms = (0..1024)
        .map(|i| VariantArm {
            name: format!("a{i}"),
            payload: None,
        })
        .collect();
    let c_bool = m.intern_const(MirConst::Bool(false)) as u32;
    m.funcs[0].slots.push(slot(ty, "variant"));
    m.funcs[0].blocks[0].instrs = vec![
        MirInstr::VariantPack {
            dst: 3,
            variant_ty: ty,
            disc: 0,
            payload: None,
        },
        MirInstr::DynPack { dst: 1, src: 3 },
        MirInstr::Const {
            dst: 2,
            value: c_bool,
        },
    ];
    let loaded = admit(&m);
    let limits = ExecLimits {
        max_steps: 1500,
        ..ExecLimits::default()
    };
    let (out, trace) = execute(&loaded, &limits, &CancelFlag::new(false));
    assert_eq!(resource_kind(&out), ResourceKind::InstrSteps);
    assert_clean_ledger(&trace.arena.unwrap());
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::Bool(false),
                ..
            }
        ),
        "{out:?}"
    );
}
