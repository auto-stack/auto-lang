//! PLAN-823 T-04 resource/fault-state tests: the closure construction
//! seams under the executor's budgets — the value-allocation fault seam
//! (V-07: the attempt reports, NO source is consumed), and the fuel/wall
//! doors of the capture clone walk. The module-level budget-before-attempt
//! law (预算前拒 attempt=0) is pinned by the parallel/823 harness
//! (`av_closures_823_t10_budget_boundaries`); these tests are the
//! executor-side states. Prefix `av_closures_823` (contract-decision
//! r2 §5 / plan V-07 axes).

use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, ResourceKind};
use auto_av_core::mir::{
    CaptureKind, CaptureOperand, FnSig, MirBlock, MirConst, MirFunction, MirInstr, MirModule,
    MirSlot, MirType, SlotKind, Terminator,
};
use auto_av_core::mir_verify::verify_mir;
use auto_av_probe::closures_fixture::closures_owner_scope;
use std::time::Duration;

/// The minimal closure module: main materializes ONE owned Str (value
/// allocation #1) and creates a closure with a CLONE capture (the env is
/// value allocation #2 on the fault seam).
fn minimal_clone_module() -> MirModule {
    let owner = closures_owner_scope();
    let mut m = MirModule::new_closures(owner.clone());
    m.types = vec![
        MirType::I32,
        MirType::Str,
        MirType::StrView,
        MirType::Fn(FnSig {
            params: vec![0],
            result: 0,
        }),
        MirType::Closure(FnSig {
            params: vec![0],
            result: 0,
        }),
    ];
    let c_x = m.intern_const(MirConst::Str(Box::from(*b"x")));
    let c_one = m.intern_const(MirConst::I32(1));
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
    let body = MirFunction {
        owner: owner.clone(),
        name: "body".to_string(),
        sig: 3,
        env: Some(vec![1]), // r8: [Str] (the clone capture)
        slots: vec![
            param(0, "bonus"),
            slot(1, "env_text"),
            slot(2, "view"),
            slot(0, "len"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::EnvGet { dst: 1, field: 0 },
                MirInstr::View { dst: 2, src: 1 },
                MirInstr::StrLen { dst: 3, view: 2 },
            ],
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };
    let main = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        })),
        slots: vec![slot(1, "s"), slot(4, "cl"), slot(0, "arg"), slot(0, "ret")],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_x as u32,
                },
                MirInstr::Const {
                    dst: 2,
                    value: c_one as u32,
                },
                MirInstr::CreateClosure {
                    dst: 1,
                    callee: auto_av_core::mir::CalleeRef::Func(1),
                    captures: vec![CaptureOperand {
                        kind: CaptureKind::Clone,
                        src: 0,
                    }],
                },
                MirInstr::CallClosure {
                    dst: 3,
                    closure: 1,
                    args: vec![2],
                },
                MirInstr::Drop { slot: 1 },
            ],
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };
    m.funcs = vec![main, body];
    m.entry = 0;
    m
}

fn admitted(m: &MirModule) -> auto_av_core::abc_verify::LoadedModule {
    verify_mir(m).expect("verifies");
    let wire = encode_abc(m).expect("encode");
    verify_abc_bytes(&wire.to_flat(), &m.owner).expect("admits")
}

#[test]
fn av_closures_823_res_fault_seam_hits_closure_env() {
    let loaded = admitted(&minimal_clone_module());
    // Value allocation #2 IS the closure env: the fault reports the
    // attempt and consumes NO source (the move-kill loop never runs; the
    // outcome stays Infra — the distinct fault state, never a language
    // trap nor a silent pass).
    let mut limits = ExecLimits::default();
    limits.fault_after_allocs = Some(2);
    let (out, _trace) = execute(&loaded, &limits, &CancelFlag::new(false));
    match out {
        ExecOutcome::Infra(msg) => {
            assert!(msg.contains("closure env"), "got {msg:?}");
            assert!(msg.contains('#'), "carries the attempt number");
        }
        other => panic!("expected the closure-env fault, got {other:?}"),
    }

    // The distinguishing control: value allocation #1 (the Str const
    // materialization) faults with a DIFFERENT message — the seam is
    // per-allocation, not a blanket refusal.
    let mut limits1 = ExecLimits::default();
    limits1.fault_after_allocs = Some(1);
    let (out1, _t1) = execute(&loaded, &limits1, &CancelFlag::new(false));
    match out1 {
        ExecOutcome::Infra(msg) => assert!(
            !msg.contains("closure env"),
            "allocation #1 is the const materialization: {msg:?}"
        ),
        other => panic!("expected the const-materialization fault, got {other:?}"),
    }

    // No fault configured: the whole path succeeds (len("x") = 1).
    let (ok, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(ok, ExecOutcome::Success { .. }), "got {ok:?}");
    let arena = trace.arena.clone().expect("ledger");
    assert_eq!(arena.live(), 0);
}

#[test]
fn av_closures_823_res_clone_walk_fuel_door() {
    let loaded = admitted(&minimal_clone_module());
    // A tiny step budget exhausts DURING the capture clone walk (the walk
    // charges the shared cumulative counter against max_fuel) — a
    // Resource classification with the partial staging unwound.
    let mut limits = ExecLimits::default();
    limits.max_steps = 3;
    let (out, trace) = execute(&loaded, &limits, &CancelFlag::new(false));
    match out {
        ExecOutcome::Resource { kind, .. } => {
            assert_eq!(kind, ResourceKind::InstrSteps, "got {kind:?}")
        }
        other => panic!("expected the fuel door, got {other:?}"),
    }
    let arena = trace.arena.clone().expect("ledger");
    assert_eq!(arena.live(), 0, "the unwound staging freed its buffers");
}

#[test]
fn av_closures_823_res_clone_walk_wall_door() {
    let loaded = admitted(&minimal_clone_module());
    // A ZERO wall allowance rejects at the clone walk's ENTRY poll (a
    // deterministic budget rejection, not a hang).
    let mut limits = ExecLimits::default();
    limits.wall = Duration::ZERO;
    let (out, _trace) = execute(&loaded, &limits, &CancelFlag::new(false));
    match out {
        ExecOutcome::Resource { kind, .. } => {
            assert_eq!(kind, ResourceKind::ExecutionWall, "got {kind:?}")
        }
        other => panic!("expected the wall door, got {other:?}"),
    }
}
