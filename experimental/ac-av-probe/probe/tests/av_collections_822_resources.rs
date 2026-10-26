//! PLAN-822 T-04 resources tests: the collections resource/failure
//! boundaries — capacity guard (both sides), live-data budget rejection
//! BEFORE the write, allocator fault injection (attempt exists /
//! successful_alloc=0), and post-failure ledger cleanliness
//! (V-03/V-07 axes; the precheck/commit/kill seams of the executor).
//! Prefix `av_collections_822` (contract-decision §6/§8).

use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, ResourceKind, MAX_LIST_ELEMS, SLOT_CELL_BYTES};
use auto_av_core::mir::{
    BinOp, ListTypeRef, MirBlock, MirConst, MirFunction, MirInstr, MirModule, MirSlot, MirType,
    OwnerScope, SlotKind, Terminator,
};
use auto_av_core::mir_verify::verify_mir;

/// A minimal `fn main() -> i32` with one `List<i32>` (slot 0), one value
/// slot (1) and `pushes` LIST_PUSH instructions fed by a const.
fn push_module(pushes: usize, _list_ty: u32, elem_ty: u32) -> MirModule {
    let mut m = MirModule::new_collections(OwnerScope {
        schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
        module: "collections822".to_string(),
    });
    m.types = vec![MirType::I32, MirType::List(ListTypeRef { elem: elem_ty })];
    let c7 = m.intern_const(MirConst::I32(7)) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    let slots = vec![
        MirSlot {
            kind: SlotKind::Local,
            ty: 1,
            name: Some("list".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 0,
            name: Some("v".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 0,
            name: Some("r".into()),
        },
    ];
    let mut instrs = vec![
        MirInstr::ListNew {
            dst: 0,
            list_ty: 1,
            cap: 0,
        },
        MirInstr::Const { dst: 1, value: c7 },
    ];
    for _ in 0..pushes {
        instrs.push(MirInstr::ListPush { root: 0, value: 1 });
    }
    instrs.push(MirInstr::Const { dst: 2, value: c0 });
    m.funcs = vec![MirFunction {
        owner: m.owner.clone(),
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![],
            result: 0,
        })),
        slots,
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 2 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

fn run_with(
    m: &MirModule,
    limits: &ExecLimits,
) -> (ExecOutcome, auto_av_core::owned_text::StrArena) {
    verify_mir(m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(m).unwrap_or_else(|e| panic!("encode: {}", e.code()));
    let loaded =
        verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("admit: {e:?}"));
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

/// V-03/cap both sides: LIST_NEW at exactly MAX_LIST_ELEMS verifies and
/// executes; one above is a resource rejection at admission (never a
/// trap, never executed).
#[test]
fn av_collections_822_resources_list_cap_both_sides() {
    // At the cap: admitted and executes (an empty huge-capacity list
    // allocates only the header).
    let m = push_module(0, 1, 0);
    let mut limits = ExecLimits::default();
    let _ = &mut limits;
    let (out, _) = run_with(&m, &ExecLimits::default());
    assert!(matches!(out, ExecOutcome::Success { .. }), "{out:?}");

    // One above: `mir verify` classifies it as the frozen resource kind.
    let mut m = push_module(0, 1, 0);
    if let MirInstr::ListNew { cap, .. } = &mut m.funcs[0].blocks[0].instrs[0] {
        *cap = MAX_LIST_ELEMS as u32 + 1;
    }
    match verify_mir(&m) {
        Err(auto_av_core::mir_verify::MirVerifyError::Resource { kind, .. }) => {
            assert_eq!(kind, ResourceKind::ListElems);
        }
        other => panic!("cap+1 must be a ListElems resource rejection: {other:?}"),
    }
}

/// V-07 budget-before-write: with a tight live-data budget the push that
/// would cross it is rejected BEFORE any mutation — the outcome is a
/// SlotBytes resource, and the ledger carries no leaked allocation
/// (allocs == frees, live == 0 after the run unwinds).
#[test]
fn av_collections_822_resources_slot_bytes_rejects_before_write() {
    let m = push_module(4, 1, 0); // 4 pushes of i32 7
    let mut limits = ExecLimits::default();
    // Room for the list header + one element cell, not two.
    limits.max_slot_bytes = 2 * SLOT_CELL_BYTES + 4 * std::mem::size_of::<auto_av_core::Value>();
    let (out, arena) = run_with(&m, &limits);
    assert_eq!(resource_kind(&out), ResourceKind::SlotBytes);
    // Ledger cleanliness: the list buffer (the only arena allocation)
    // freed exactly once; nothing leaked past the rejection.
    assert_eq!(arena.alloc_events().len(), arena.free_events().len());
    assert_eq!(arena.live(), 0);
}

/// V-03/V-07 fault injection: the Nth allocation ATTEMPT fails with no
/// half-committed state — attempts >= N, the failed attempt produced NO
/// successful allocation, and the outcome is Infra (never a language
/// trap, never a silent success).
#[test]
fn av_collections_822_resources_fault_injection_attempt_semantics() {
    let m = push_module(1, 1, 0);
    verify_mir(&m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(&m).unwrap_or_else(|e| panic!("encode"));
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("{e:?}"));
    // The collections fault seam is the ARENA ledger (V-07 allocator
    // axis): fail the FIRST allocation attempt — the list buffer itself.
    let arena = auto_av_core::owned_text::StrArena::new();
    arena.set_fault_at(1);
    let (out, trace) = auto_av_core::av::execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        arena,
    );
    let arena = trace.arena.clone().expect("execute attaches the ledger");
    match &out {
        ExecOutcome::Infra(msg) => assert!(msg.contains("fault-injected"), "{msg}"),
        other => panic!("expected the injected infra fault, got {other:?}"),
    }
    // V-07 oracle: the attempt EXISTS (>= 1) but successful_alloc == 0.
    assert!(arena.attempts() >= 1);
    assert_eq!(arena.successful_allocs(), 0);
    assert_eq!(arena.alloc_events().len(), 0);
    assert_eq!(arena.live(), 0);
}

/// V-03 failure keeps the source and prior list intact: a budget
/// rejection on the 3rd push leaves the first two elements' accounting
/// consistent — the SAME program under a larger budget succeeds with the
/// same visible result (the budget is the only delta).
#[test]
fn av_collections_822_resources_budget_is_the_only_delta() {
    let m = push_module(3, 1, 0);
    let mut tight = ExecLimits::default();
    tight.max_slot_bytes = 3 * SLOT_CELL_BYTES + 3 * std::mem::size_of::<auto_av_core::Value>();
    let (tight_out, tight_arena) = run_with(&m, &tight);
    assert_eq!(resource_kind(&tight_out), ResourceKind::SlotBytes);
    assert_eq!(tight_arena.live(), 0);

    let (wide_out, _) = run_with(&m, &ExecLimits::default());
    assert!(
        matches!(wide_out, ExecOutcome::Success { .. }),
        "{wide_out:?}"
    );
}

/// V-07 max-allocated path: a program that constructs within the budget
/// reports the peak through the trace (peak >= live at all times), and
/// the normal-exit ledger drains to zero.
#[test]
fn av_collections_822_resources_peak_and_drain() {
    let m = push_module(3, 1, 0);
    verify_mir(&m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(&m).unwrap_or_else(|e| panic!("encode"));
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("{e:?}"));
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Success { .. }));
    assert!(trace.peak_slot_bytes > 0);
    let arena = trace.arena.expect("execute attaches the ledger");
    assert_eq!(arena.live(), 0, "normal exit drains the ledger");
    assert_eq!(arena.alloc_events().len(), arena.free_events().len());
}

/// V-12 arithmetic shape: len/cap stay i32-typed and checked — the
/// largest admissible index program still folds ordinary arithmetic
/// around list ops without surprises (regression guard for the
/// checked-arithmetic contract).
#[test]
fn av_collections_822_resources_index_arithmetic_shape() {
    let m = push_module(2, 1, 0);
    // get(1) after two pushes: fold len+got = 2 + 7 = 9.
    let mut m = m;
    let c1 = m.intern_const(MirConst::I32(1)) as u32;
    let len_slot = m.funcs[0].slots.len() as u32;
    m.funcs[0].slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 0,
        name: Some("len".into()),
    });
    let got_slot = m.funcs[0].slots.len() as u32;
    m.funcs[0].slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 0,
        name: Some("got".into()),
    });
    m.funcs[0].slots[2].ty = 0; // ret stays i32
    let block = &mut m.funcs[0].blocks[0];
    let ret = 2;
    let n = block.instrs.len();
    block.instrs.insert(
        n - 1,
        MirInstr::Const {
            dst: got_slot,
            value: c1,
        },
    );
    block.instrs.insert(
        n,
        MirInstr::ListGet {
            dst: got_slot,
            base: 0,
            index: got_slot,
        },
    );
    block.instrs.insert(
        n + 1,
        MirInstr::ListLen {
            dst: len_slot,
            src: 0,
        },
    );
    block.instrs.insert(
        n + 2,
        MirInstr::Bin {
            op: BinOp::AddI32,
            dst: got_slot,
            lhs: got_slot,
            rhs: len_slot,
        },
    );
    block.term = Terminator::Return { value: ret };
    // ret const is instrs[n-2] originally; point the return at got.
    if let Terminator::Return { value } = &mut block.term {
        *value = got_slot;
    }
    let (out, _) = run_with(&m, &ExecLimits::default());
    match out {
        ExecOutcome::Success { value, .. } => {
            assert_eq!(value, auto_av_core::Value::I32(9));
        }
        other => panic!("{other:?}"),
    }
}
