//! PLAN-823 T-03 semantics tests: the full-support-surface fixtures and
//! negatives over the REAL pipeline (verify → encode → abc-admit →
//! execute). Prefix `av_closures_823` (contract-decision r2 §5/r3 §8).
//!
//! Coverage axes (plan §6):
//! * V-02 borrow captures — the legal synchronous call, the escape front
//!   door (RETURN / wrong consumer), the source pin, and the module-level
//!   borrow-domain rejection;
//! * V-03 nested envs and same-closure frame reuse (env identity survives
//!   frame churn; a body can itself CREATE_CLOSURE);
//! * V-04 indirect-call typing negatives (arity / result type);
//! * V-06 call-time discipline — trap during the call releases the env at
//!   the all-exit cleanup; an explicit closure CLONE owns an independent
//!   env (distinguishing oracle);
//! * V-11 post-capture read of a move-captured source rejects;
//! * V-12 cancellation after commit unwinds with the env released.

use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value, AV_CLOSURES_PROFILE};
use auto_av_core::mir::{
    BinOp, CaptureKind, CaptureOperand, FnSig, ListTypeRef, MirBlock, MirConst, MirFieldDef,
    MirFunction, MirInstr, MirModule, MirSlot, MirType, RecordTypeRef, SlotKind, Terminator,
};
use auto_av_core::mir_verify::verify_mir;
use auto_av_probe::closures_fixture::closures_owner_scope;

fn slot(ty: u32, name: &str) -> MirSlot {
    MirSlot {
        kind: SlotKind::Local,
        ty,
        name: Some(name.to_string()),
    }
}

fn param(ty: u32, name: &str) -> MirSlot {
    MirSlot {
        kind: SlotKind::Param,
        ty,
        name: Some(name.to_string()),
    }
}

fn fn_type(params: Vec<u32>, result: u32) -> MirType {
    MirType::Fn(FnSig { params, result })
}

/// The shared type table of the semantics fixtures:
/// 0 i32, 1 Str, 2 StrView, 3 Note{text,n}, 4 List<Note>, 5 Fn(i32)->i32,
/// 6 Closure(5), 7 Fn()->Closure(6).
fn semantics_types() -> Vec<MirType> {
    vec![
        MirType::I32,
        MirType::Str,
        MirType::StrView,
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
        MirType::List(ListTypeRef { elem: 3 }),
        fn_type(vec![0], 0),
        MirType::Closure(FnSig {
            params: vec![0],
            result: 0,
        }),
        fn_type(vec![], 6),
    ]
}

/// Run a module through the REAL gates and return (outcome, arena).
fn run_pipeline(m: &MirModule) -> (ExecOutcome, auto_av_core::owned_text::StrArena) {
    verify_mir(m).expect("semantics fixture verifies");
    let wire = encode_abc(m).expect("encode");
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).expect("abc admit");
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let arena = trace.arena.clone().expect("execution attaches the ledger");
    (out, arena)
}

// ---------------------------------------------------------------------------
// V-02: borrow captures
// ---------------------------------------------------------------------------

/// `borrow_case()`: main mints an owned Str, borrow-captures it, calls the
/// closure synchronously, drops it, then still USES the source (a VIEW +
/// StrLen after the pin lifted). Hand oracle: env view len 4 + source view
/// len 4 = **8**.
fn borrow_case() -> MirModule {
    let owner = closures_owner_scope();
    let mut m = MirModule::new_closures(owner.clone());
    m.types = semantics_types();
    let c_lend = m.intern_const(MirConst::Str(Box::from(*b"lend")));
    let c_one = m.intern_const(MirConst::I32(1));

    // body (func 1): (i32) -> i32 — reads the borrowed view, returns its
    // scalar length (4).
    // slots: 0 bonus(param), 1 env_view:StrView, 2 len:i32
    let body = MirFunction {
        owner: owner.clone(),
        name: "body".to_string(),
        sig: 5,
        env: Some(vec![2]), // r8: [StrView] (the borrow capture's declared entry)
        slots: vec![param(0, "bonus"), slot(2, "env_view"), slot(0, "len")],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::EnvGet { dst: 1, field: 0 },
                MirInstr::StrLen { dst: 2, view: 1 },
            ],
            term: Terminator::Return { value: 2 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };

    // main (func 0, entry): the synchronous borrow lifecycle.
    // slots: 0 src:Str, 1 cl:Closure, 2 arg:i32, 3 ret:i32, 4 v:StrView,
    //        5 slen:i32
    let main = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(fn_type(vec![], 0)),
        slots: vec![
            slot(1, "src"),
            slot(6, "cl"),
            slot(0, "arg"),
            slot(0, "ret"),
            slot(2, "v"),
            slot(0, "slen"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_lend as u32,
                },
                MirInstr::Const {
                    dst: 2,
                    value: c_one as u32,
                },
                MirInstr::CreateClosure {
                    dst: 1,
                    callee: auto_av_core::mir::CalleeRef::Func(1),
                    captures: vec![CaptureOperand {
                        kind: CaptureKind::Borrow,
                        src: 0,
                    }],
                },
                MirInstr::CallClosure {
                    dst: 3,
                    closure: 1,
                    args: vec![2],
                },
                MirInstr::Drop { slot: 1 },
                // The pin lifted with the drop: the source is still owned
                // and readable by its creator.
                MirInstr::View { dst: 4, src: 0 },
                MirInstr::StrLen { dst: 5, view: 4 },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 3,
                    lhs: 3,
                    rhs: 5,
                },
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

#[test]
fn av_closures_823_sem_borrow_legal_sync_call() {
    let m = borrow_case();
    assert_eq!(m.profile, AV_CLOSURES_PROFILE);
    let (out, arena) = run_pipeline(&m);
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(v, 8, "env view len 4 + source view len 4"),
        other => panic!("borrow case must succeed: {other:?}"),
    }
    assert_eq!(arena.live(), 0, "the borrowed source releases with main");
    assert_eq!(arena.alloc_events().len(), 1);
    assert_eq!(arena.free_events().len(), 1);
}

#[test]
fn av_closures_823_sem_borrow_escape_rejected() {
    // RETURN carries the borrow-bound closure out — the front-door escape.
    let mut m = borrow_case();
    // Replace the tail (Drop + view read) with a plain return of `cl`.
    m.funcs[0].blocks[0].instrs.truncate(4);
    m.funcs[0].blocks[0].term = Terminator::Return { value: 1 };
    let err = verify_mir(&m).expect_err("returning a borrow closure must reject");
    assert!(
        format!("{err:?}").contains("mir.borrow-closure.escape"),
        "{err:?}"
    );

    // Passing the borrow closure as an argument is an escape too.
    let mut m2 = borrow_case();
    m2.funcs[0].blocks[0].instrs[3] = MirInstr::Bin {
        op: BinOp::AddI32,
        dst: 3,
        lhs: 2,
        rhs: 1, // the closure slot as an operand — a non-call consumer
    };
    let err2 = verify_mir(&m2).expect_err("a non-call consumer must reject");
    assert!(
        format!("{err2:?}").contains("mir.borrow-closure.escape")
            || format!("{err2:?}").contains("mir.bin-type"),
        "{err2:?}"
    );
}

#[test]
fn av_closures_823_sem_borrow_source_pinned() {
    // Moving the borrow source before the closure drops violates the pin.
    let mut m = borrow_case();
    let site = m.funcs[0].blocks[0].instrs[2].clone();
    let _ = site;
    m.funcs[0].blocks[0].instrs.insert(
        3,
        MirInstr::Move { dst: 5, src: 0 }, // dst 5 typed i32 — also a type
                                           // error, but the PIN fires first (discipline runs before typing)
    );
    let err = verify_mir(&m).expect_err("moving a pinned borrow source must reject");
    assert!(
        format!("{err:?}").contains("mir.borrow-source-pinned"),
        "{err:?}"
    );
}

#[test]
fn av_closures_823_sem_borrow_domain_rejects_non_str() {
    // A borrow capture of a List-typed source is outside the delivered
    // domain — the typing arm rejects (`mir.capture.borrow-domain`).
    let mut m = borrow_case();
    m.funcs[0].blocks[0].instrs[2] = MirInstr::CreateClosure {
        dst: 1,
        callee: auto_av_core::mir::CalleeRef::Func(1),
        captures: vec![CaptureOperand {
            kind: CaptureKind::Borrow,
            src: 3, // the i32 arg slot
        }],
    };
    let err = verify_mir(&m).expect_err("borrowing a non-Str slot must reject");
    assert!(
        format!("{err:?}").contains("mir.capture.borrow-domain"),
        "{err:?}"
    );
}

// ---------------------------------------------------------------------------
// V-03: nested envs and same-closure frame reuse
// ---------------------------------------------------------------------------

/// `nested_case()`: main creates a closure capturing a List<Note> (MOVE),
/// calls it TWICE (frame reuse: the second call is a fresh frame over the
/// SAME env); each call pushes a note and creates + calls + drops a CHILD
/// closure capturing a CLONE of the env list. Hand oracle:
/// call 1: push → len 2, child sees 2 → 2*2=4, ret 20+4=**24**;
/// call 2: push → len 3, child sees 3 → 6, ret 30+6=**36**; main = **60**.
fn nested_case() -> MirModule {
    let owner = closures_owner_scope();
    let mut m = MirModule::new_closures(owner.clone());
    m.types = semantics_types();
    let c_auto = m.intern_const(MirConst::Str(Box::from(*b"note")));
    let c_one = m.intern_const(MirConst::I32(1));

    // child (func 2): (i32) -> i32 — its env holds a CLONE of the parent
    // env's list at child-creation time; returns len*2.
    let child = MirFunction {
        owner: owner.clone(),
        name: "child".to_string(),
        sig: 5,
        env: Some(vec![4]), // r8: [List<Note>] (clone of the parent env's list)
        slots: vec![
            param(0, "bonus"),
            slot(4, "env_list"),
            slot(0, "len"),
            slot(0, "twice"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::EnvGet { dst: 1, field: 0 },
                MirInstr::ListLen { dst: 2, src: 1 },
                MirInstr::Const {
                    dst: 3,
                    value: c_one as u32,
                },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 3,
                    lhs: 2,
                    rhs: 2,
                },
            ],
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };

    // parent body (func 1): (i32) -> i32 — pushes a note onto the env's
    // list (the mutable env field), creates+calls+drops the child on a
    // clone of it, returns len*10 + child ret.
    // slots: 0 bonus(param), 1 env_list, 2 text, 3 n, 4 note, 5 child_cl,
    //        6 len, 7 child_ret, 8 ten, 9 mul
    let body = MirFunction {
        owner: owner.clone(),
        name: "body".to_string(),
        sig: 5,
        env: Some(vec![4]), // r8: [List<Note>]
        slots: vec![
            param(0, "bonus"),
            slot(4, "env_list"),
            slot(1, "text"),
            slot(0, "n"),
            slot(3, "note"),
            slot(6, "child_cl"),
            slot(0, "len"),
            slot(0, "child_ret"),
            slot(0, "ten"),
            slot(0, "mul"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::EnvGet { dst: 1, field: 0 },
                MirInstr::Const {
                    dst: 2,
                    value: c_auto as u32,
                },
                MirInstr::Const {
                    dst: 3,
                    value: c_one as u32,
                },
                MirInstr::Aggregate {
                    dst: 4,
                    record_ty: 3,
                    operands: vec![2, 3],
                },
                MirInstr::ListPush { root: 1, value: 4 },
                MirInstr::CreateClosure {
                    dst: 5,
                    callee: auto_av_core::mir::CalleeRef::Func(2),
                    captures: vec![CaptureOperand {
                        kind: CaptureKind::Clone,
                        src: 1,
                    }],
                },
                MirInstr::CallClosure {
                    dst: 7,
                    closure: 5,
                    args: vec![0],
                },
                MirInstr::Drop { slot: 5 },
                MirInstr::ListLen { dst: 6, src: 1 },
                MirInstr::Const {
                    dst: 8,
                    value: m.intern_const(MirConst::I32(10)) as u32,
                },
                MirInstr::Bin {
                    op: BinOp::MulI32,
                    dst: 9,
                    lhs: 6,
                    rhs: 8,
                },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 9,
                    lhs: 9,
                    rhs: 7,
                },
            ],
            term: Terminator::Return { value: 9 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };

    // main (func 0): make, call twice, sum. The second call reuses the
    // closure over a FRESH frame with the SAME env (identity survives).
    // slots: 0 text, 1 n, 2 note, 3 list, 4 cl, 5 arg, 6 r1, 7 r2
    let main = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(fn_type(vec![], 0)),
        slots: vec![
            slot(1, "text"),
            slot(0, "n"),
            slot(3, "note"),
            slot(4, "list"),
            slot(6, "cl"),
            slot(0, "arg"),
            slot(0, "r1"),
            slot(0, "r2"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_auto as u32,
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
                    cap: 4,
                },
                MirInstr::ListPush { root: 3, value: 2 },
                MirInstr::Const {
                    dst: 5,
                    value: c_one as u32,
                },
                MirInstr::CreateClosure {
                    dst: 4,
                    callee: auto_av_core::mir::CalleeRef::Func(1),
                    captures: vec![CaptureOperand {
                        kind: CaptureKind::Move,
                        src: 3,
                    }],
                },
                MirInstr::CallClosure {
                    dst: 6,
                    closure: 4,
                    args: vec![5],
                },
                MirInstr::CallClosure {
                    dst: 7,
                    closure: 4,
                    args: vec![5],
                },
                MirInstr::Drop { slot: 4 },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 6,
                    lhs: 6,
                    rhs: 7,
                },
            ],
            term: Terminator::Return { value: 6 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };

    m.funcs = vec![main, body, child];
    m.entry = 0;
    m
}

#[test]
fn av_closures_823_sem_nested_and_frame_reuse() {
    let (out, arena) = run_pipeline(&nested_case());
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(
            v, 60,
            "24 + 36 (nested child sees the env at its own creation)"
        ),
        other => panic!("nested case must succeed: {other:?}"),
    }
    assert_eq!(arena.live(), 0);
}

// ---------------------------------------------------------------------------
// V-04: indirect-call typing negatives
// ---------------------------------------------------------------------------

#[test]
fn av_closures_823_sem_call_typing_negatives() {
    use auto_av_probe::closures_fixture::first_case_module;
    // Wrong arity.
    let mut m = first_case_module();
    m.funcs[2].blocks[0].instrs[2] = MirInstr::CallClosure {
        dst: 2,
        closure: 0,
        args: vec![],
    };
    let err = verify_mir(&m).expect_err("arity mismatch must reject");
    assert!(format!("{err:?}").contains("mir.closure.arity"), "{err:?}");

    // Wrong result type (dst typed Closure where the signature returns
    // i32 — slot 0 is main's Closure slot).
    let mut m2 = first_case_module();
    m2.funcs[2].blocks[0].instrs[2] = MirInstr::CallClosure {
        dst: 0,
        closure: 0,
        args: vec![1],
    };
    let err2 = verify_mir(&m2).expect_err("result type mismatch must reject");
    assert!(
        format!("{err2:?}").contains("mir.closure.result-type")
            || format!("{err2:?}").contains("mir.closure.type"),
        "{err2:?}"
    );

    // CALL_CLOSURE on a non-closure operand.
    let mut m3 = first_case_module();
    m3.funcs[2].blocks[0].instrs[2] = MirInstr::CallClosure {
        dst: 2,
        closure: 1, // the i32 arg slot
        args: vec![1],
    };
    let err3 = verify_mir(&m3).expect_err("non-closure operand must reject");
    assert!(format!("{err3:?}").contains("mir.closure.type"), "{err3:?}");
}

// ---------------------------------------------------------------------------
// V-06 / V-11 / V-12: call-time discipline
// ---------------------------------------------------------------------------

#[test]
fn av_closures_823_sem_trap_during_call_releases_env() {
    // The body traps (div by zero) AFTER touching the env — the trap
    // outcome reports, and the all-exit cleanup releases the closure's
    // env (live=0; the fixture carries no spans — the stripped-debug
    // shape, V-11's axis: semantics never ride the diagnostics).
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    let c_zero = m.intern_const(MirConst::I32(0));
    // In the body (funcs[1]), after ListLen (dst 6): zero into slot 12,
    // then 6 / 0 — the trap fires mid-call with the env live.
    m.funcs[1].blocks[0].instrs.insert(
        7,
        MirInstr::Const {
            dst: 12,
            value: c_zero as u32,
        },
    );
    m.funcs[1].blocks[0].instrs.insert(
        8,
        MirInstr::Bin {
            op: BinOp::DivI32,
            dst: 10,
            lhs: 6,
            rhs: 12,
        },
    );
    verify_mir(&m).expect("trap fixture verifies (traps stay latent)");
    let wire = encode_abc(&m).expect("encode");
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).expect("admit");
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(out, ExecOutcome::Trap { .. }),
        "the body must trap: {out:?}"
    );
    let arena = trace.arena.clone().expect("ledger");
    assert_eq!(arena.live(), 0, "all-exit cleanup released the env");
    let mut with_spans = m.clone();
    for (fi, function) in with_spans.funcs.iter_mut().enumerate() {
        let count: usize = function.blocks.iter().map(|b| b.instrs.len()).sum();
        function.spans = (0..count)
            .map(|i| {
                (
                    i as u32,
                    auto_av_core::contract::SpanInfo {
                        func: fi as u32,
                        start: (i * 4) as u32,
                        end: (i * 4 + 4) as u32,
                    },
                )
            })
            .collect();
    }
    let (spanned, spanned_arena) = run_pipeline(&with_spans);
    let (
        ExecOutcome::Trap {
            kind: stripped_kind,
            ..
        },
        ExecOutcome::Trap {
            kind: spanned_kind, ..
        },
    ) = (out, spanned)
    else {
        panic!("both diagnostic forms must trap");
    };
    assert_eq!(stripped_kind, spanned_kind);
    assert_eq!(arena.alloc_events(), spanned_arena.alloc_events());
    assert_eq!(arena.free_events(), spanned_arena.free_events());
    assert_eq!(spanned_arena.live(), 0);
}

#[test]
fn av_closures_823_sem_closure_clone_independent_env() {
    // CLONE of a closure deep-clones the env: calling the original, then
    // cloning, then calling BOTH again yields independent mutations.
    // Hand oracle: c1 call → len 1→2 → 20; clone (len 2 snapshot); c2
    // call → 3 → 30; c1 again → still 2 → 3 → 30 (a SHARED env would
    // read 3 → 40). Total 20+30+30 = **80** (shared would be 90).
    let owner = closures_owner_scope();
    let mut m = MirModule::new_closures(owner.clone());
    m.types = semantics_types();
    let c_auto = m.intern_const(MirConst::Str(Box::from(*b"n")));
    let c_one = m.intern_const(MirConst::I32(1));
    let c_ten = m.intern_const(MirConst::I32(10));

    // body: (i32)->i32 — push a note onto the env list, return len*10.
    let body = MirFunction {
        owner: owner.clone(),
        name: "body".to_string(),
        sig: 5,
        env: Some(vec![4]), // r8: [List<Note>]
        slots: vec![
            param(0, "bonus"),
            slot(4, "env_list"),
            slot(1, "text"),
            slot(0, "n"),
            slot(3, "note"),
            slot(0, "len"),
            slot(0, "ten"),
            slot(0, "mul"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::EnvGet { dst: 1, field: 0 },
                MirInstr::Const {
                    dst: 2,
                    value: c_auto as u32,
                },
                MirInstr::Const {
                    dst: 3,
                    value: c_one as u32,
                },
                MirInstr::Aggregate {
                    dst: 4,
                    record_ty: 3,
                    operands: vec![2, 3],
                },
                MirInstr::ListPush { root: 1, value: 4 },
                MirInstr::ListLen { dst: 5, src: 1 },
                MirInstr::Const {
                    dst: 6,
                    value: c_ten as u32,
                },
                MirInstr::Bin {
                    op: BinOp::MulI32,
                    dst: 7,
                    lhs: 5,
                    rhs: 6,
                },
            ],
            term: Terminator::Return { value: 7 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };

    // main: make → call → clone → call clone → call original → sum.
    // slots: 0 text, 1 n, 2 note, 3 list, 4 cl, 5 cl2, 6 arg, 7..9 rets
    let main = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(fn_type(vec![], 0)),
        slots: vec![
            slot(1, "text"),
            slot(0, "n"),
            slot(3, "note"),
            slot(4, "list"),
            slot(6, "cl"),
            slot(6, "cl2"),
            slot(0, "arg"),
            slot(0, "r1"),
            slot(0, "r2"),
            slot(0, "r3"),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_auto as u32,
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
                    cap: 4,
                },
                MirInstr::ListPush { root: 3, value: 2 },
                MirInstr::Const {
                    dst: 6,
                    value: c_one as u32,
                },
                MirInstr::CreateClosure {
                    dst: 4,
                    callee: auto_av_core::mir::CalleeRef::Func(1),
                    captures: vec![CaptureOperand {
                        kind: CaptureKind::Move,
                        src: 3,
                    }],
                },
                MirInstr::CallClosure {
                    dst: 7,
                    closure: 4,
                    args: vec![6],
                },
                MirInstr::Clone { dst: 5, src: 4 },
                MirInstr::CallClosure {
                    dst: 8,
                    closure: 5,
                    args: vec![6],
                },
                MirInstr::CallClosure {
                    dst: 9,
                    closure: 4,
                    args: vec![6],
                },
                MirInstr::Drop { slot: 5 },
                MirInstr::Drop { slot: 4 },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 7,
                    lhs: 7,
                    rhs: 8,
                },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 7,
                    lhs: 7,
                    rhs: 9,
                },
            ],
            term: Terminator::Return { value: 7 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };

    m.funcs = vec![main, body];
    m.entry = 0;
    let (out, arena) = run_pipeline(&m);
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(v, 80, "20+30+30 — the clone owns an independent env"),
        other => panic!("clone case must succeed: {other:?}"),
    }
    assert_eq!(arena.live(), 0);
}

#[test]
fn av_closures_823_sem_post_capture_read_rejected() {
    // V-11: reading a MOVE-captured source after the capture is a
    // use-after-move at the ownership gate (before any execution).
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    // Slot 4 (the moved list) read after CreateClosure (index 6):
    m.funcs[0].blocks[0]
        .instrs
        .insert(7, MirInstr::ListLen { dst: 1, src: 4 });
    let err = verify_mir(&m).expect_err("post-capture read must reject");
    assert!(
        format!("{err:?}").contains("mir.use-after-move")
            || format!("{err:?}").contains("uninitialized"),
        "{err:?}"
    );
}

#[test]
fn av_closures_823_sem_cancel_after_commit_releases_env() {
    // V-12 (提交后取消): a pre-raised cancel flag aborts at the first
    // poll INSIDE the call; the all-exit cleanup releases the env.
    let m = auto_av_probe::closures_fixture::first_case_module();
    verify_mir(&m).expect("verifies");
    let wire = encode_abc(&m).expect("encode");
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).expect("admit");
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(true));
    assert!(matches!(out, ExecOutcome::Cancelled), "got {out:?}");
    let arena = trace.arena.clone().expect("ledger");
    assert_eq!(arena.live(), 0, "cancel unwinds with the env released");
}

// ---------------------------------------------------------------------------
// R1-F01 (§5.1.4/§5.5): parent-env FIELD transfer — the dedicated
// MoveFromEnv op owns the responsibility; alias Moves reject; moved
// fields are unreadable and single-cleaned.
// ---------------------------------------------------------------------------

/// The F01 counterexample, made PERMANENT: an ENV_GET share MOVEd into a
/// child closure (the alias-transfer path) must reject at the applicable
/// gate — the share never transfers the parent field's responsibility.
#[test]
fn av_closures_823_sem_env_alias_move_rejected() {
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    let mut child = m.funcs[1].clone();
    child.name = "child".into();
    child.env = Some(vec![3]); // [List<Note>]
    child.blocks[0].instrs = vec![
        MirInstr::EnvGet { dst: 2, field: 0 },
        MirInstr::ListLen { dst: 6, src: 2 },
    ];
    child.blocks[0].term = Terminator::Return { value: 6 };
    m.funcs.push(child);
    let body = &mut m.funcs[1];
    let dst = body.slots.len() as u32;
    body.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 5,
        name: Some("child".into()),
    });
    let pos = body.blocks[0]
        .instrs
        .iter()
        .position(|i| matches!(i, MirInstr::EnvGet { field: 1, .. }))
        .unwrap()
        + 1;
    body.blocks[0].instrs.splice(
        pos..pos,
        vec![
            MirInstr::CreateClosure {
                dst,
                callee: auto_av_core::mir::CalleeRef::Func(3),
                captures: vec![CaptureOperand {
                    kind: CaptureKind::Move,
                    src: 2, // the ENV_GET alias of the parent's List field
                }],
            },
            MirInstr::Drop { slot: dst },
            MirInstr::EnvGet { dst: 2, field: 1 },
        ],
    );
    let err = verify_mir(&m).expect_err("an alias MOVE capture must reject");
    assert!(
        format!("{err:?}").contains("mir.capture.env-alias"),
        "{err:?}"
    );
}

/// The CLONE control of the same shape stays legal (a deep copy is a
/// legitimate read of the shared field).
#[test]
fn av_closures_823_sem_env_alias_clone_control() {
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    let mut child = m.funcs[1].clone();
    child.name = "child".into();
    child.env = Some(vec![3]);
    child.blocks[0].instrs = vec![
        MirInstr::EnvGet { dst: 2, field: 0 },
        MirInstr::ListLen { dst: 6, src: 2 },
    ];
    child.blocks[0].term = Terminator::Return { value: 6 };
    m.funcs.push(child);
    let body = &mut m.funcs[1];
    let dst = body.slots.len() as u32;
    body.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 5,
        name: Some("child".into()),
    });
    let pos = body.blocks[0]
        .instrs
        .iter()
        .position(|i| matches!(i, MirInstr::EnvGet { field: 1, .. }))
        .unwrap()
        + 1;
    body.blocks[0].instrs.insert(
        pos,
        MirInstr::CreateClosure {
            dst,
            callee: auto_av_core::mir::CalleeRef::Func(3),
            captures: vec![CaptureOperand {
                kind: CaptureKind::Clone,
                src: 2, // clone of the alias: legal deep copy
            }],
        },
    );
    body.blocks[0]
        .instrs
        .insert(pos + 1, MirInstr::Drop { slot: dst });
    let (out, arena) = run_pipeline(&m);
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => {
            assert_eq!(v, 29, "clone control keeps the first-case oracle")
        }
        other => panic!("clone control must succeed: {other:?}"),
    }
    assert_eq!(arena.live(), 0);
}

/// The REAL transfer: the parent body MoveFromEnvs its List field into
/// the child and RETURNS the child; main DROPS the parent first (its env
/// releases — the moved field is a placeholder) and only then calls the
/// child, which still reads the transferred List correctly.
/// oracle: child env [List] len 1 → 1*10 = 10; main returns child ret+5.
#[test]
fn av_closures_823_sem_parent_field_transfer_real() {
    use auto_av_probe::closures_fixture::closures_owner_scope;
    let owner = closures_owner_scope();
    let mut m = MirModule::new_closures(owner.clone());
    // 0 i32, 1 Str, 2 StrView, 3 Fn(i32)->i32, 4 Closure(3),
    // 5 Fn(i32)->Closure(4), 6 Closure(5), 7 Fn()->Closure(6)
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
        MirType::Fn(FnSig {
            params: vec![0],
            result: 4,
        }),
        MirType::Closure(FnSig {
            params: vec![0],
            result: 4,
        }),
        MirType::Fn(FnSig {
            params: vec![],
            result: 6,
        }),
    ];
    let c_lend = m.intern_const(MirConst::Str(Box::from(*b"lend")));
    let c_five = m.intern_const(MirConst::I32(5));
    // make (0, sig 7): clone-capture the text, return the parent closure.
    let make = MirFunction {
        owner: owner.clone(),
        name: "make".to_string(),
        sig: 7,
        slots: vec![slot(1, "text"), slot(6, "p")],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_lend as u32,
                },
                MirInstr::CreateClosure {
                    dst: 1,
                    callee: auto_av_core::mir::CalleeRef::Func(1),
                    captures: vec![CaptureOperand {
                        kind: CaptureKind::Clone,
                        src: 0,
                    }],
                },
            ],
            term: Terminator::Return { value: 1 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };
    // body (1, sig 5, env [Str]): TRANSFER field 0 to the child, return it.
    let body = MirFunction {
        owner: owner.clone(),
        name: "body".to_string(),
        sig: 5,
        slots: vec![param(0, "bonus"), slot(4, "c")],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::CreateClosure {
                dst: 1,
                callee: auto_av_core::mir::CalleeRef::Func(2),
                captures: vec![CaptureOperand {
                    kind: CaptureKind::MoveFromEnv,
                    src: 0,
                }],
            }],
            term: Terminator::Return { value: 1 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: Some(vec![1]),
    };
    // child (2, sig 3, env [Str]): reads the transferred field.
    let child = MirFunction {
        owner: owner.clone(),
        name: "child".to_string(),
        sig: 3,
        slots: vec![
            param(0, "bonus"),
            slot(1, "env_text"),
            slot(2, "v"),
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
        env: Some(vec![1]),
    };
    // main (entry): make → p; p(5) → child; DROP p; child(5) → 40; +5.
    let main = MirFunction {
        owner,
        name: "main".to_string(),
        sig: m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: 0,
        })),
        slots: vec![slot(6, "p"), slot(4, "c"), slot(0, "arg"), slot(0, "ret")],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 2,
                    value: c_five as u32,
                },
                MirInstr::Call {
                    callee: auto_av_core::mir::CalleeRef::Func(0),
                    dst: 0,
                    args: vec![],
                },
                MirInstr::CallClosure {
                    dst: 1,
                    closure: 0,
                    args: vec![2],
                },
                MirInstr::Drop { slot: 0 },
                MirInstr::CallClosure {
                    dst: 3,
                    closure: 1,
                    args: vec![2],
                },
                MirInstr::Drop { slot: 1 },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 3,
                    lhs: 3,
                    rhs: 2,
                },
            ],
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
        env: None,
    };
    m.funcs = vec![make, body, child, main];
    m.entry = 3;
    let (out, arena) = run_pipeline(&m);
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => {
            assert_eq!(v, 9, "transferred Str len 4 + bonus 5")
        }
        other => panic!("real transfer must succeed: {other:?}"),
    }
    assert_eq!(arena.live(), 0, "every buffer freed exactly once");
    assert_eq!(
        arena.free_events().len(),
        arena.alloc_events().len(),
        "ledger balanced — the parent placeholder frees nothing, the child owns the field"
    );
}

/// A moved-out field may not be re-read in the same body.
#[test]
fn av_closures_823_sem_env_field_reread_rejected() {
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    let mut child = m.funcs[1].clone();
    child.name = "child".into();
    child.env = Some(vec![3]);
    child.blocks[0].instrs = vec![MirInstr::ListLen { dst: 6, src: 2 }];
    child.blocks[0].term = Terminator::Return { value: 6 };
    m.funcs.push(child);
    let body = &mut m.funcs[1];
    let dst = body.slots.len() as u32;
    body.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 5,
        name: Some("child".into()),
    });
    body.blocks[0].instrs = vec![
        MirInstr::CreateClosure {
            dst,
            callee: auto_av_core::mir::CalleeRef::Func(3),
            captures: vec![CaptureOperand {
                kind: CaptureKind::MoveFromEnv,
                src: 1,
            }],
        },
        MirInstr::Drop { slot: dst },
        // the re-read of the moved field: rejects
        MirInstr::EnvGet { dst: 2, field: 1 },
        MirInstr::ListLen { dst: 6, src: 2 },
    ];
    let err = verify_mir(&m).expect_err("a moved-out env field must not re-read");
    assert!(
        format!("{err:?}").contains("mir.env.field-moved"),
        "{err:?}"
    );
}

/// A field may not be transferred twice.
#[test]
fn av_closures_823_sem_env_field_double_transfer_rejected() {
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    for name in ["childA", "childB"] {
        let mut child = m.funcs[1].clone();
        child.name = name.into();
        child.env = Some(vec![3]);
        child.blocks[0].instrs = vec![MirInstr::ListLen { dst: 6, src: 2 }];
        child.blocks[0].term = Terminator::Return { value: 6 };
        m.funcs.push(child);
    }
    let c_zero = m.intern_const(MirConst::I32(0));
    let body = &mut m.funcs[1];
    let d1 = body.slots.len() as u32;
    body.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 5,
        name: Some("c1".into()),
    });
    let d2 = body.slots.len() as u32;
    body.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 5,
        name: Some("c2".into()),
    });
    let cap = CaptureOperand {
        kind: CaptureKind::MoveFromEnv,
        src: 1,
    };
    body.blocks[0].instrs = vec![
        MirInstr::CreateClosure {
            dst: d1,
            callee: auto_av_core::mir::CalleeRef::Func(3),
            captures: vec![cap.clone()],
        },
        MirInstr::CreateClosure {
            dst: d2,
            callee: auto_av_core::mir::CalleeRef::Func(4),
            captures: vec![cap],
        },
        MirInstr::Drop { slot: d1 },
        MirInstr::Drop { slot: d2 },
        MirInstr::Const {
            dst: 12,
            value: c_zero as u32,
        },
    ];
    body.blocks[0].term = Terminator::Return { value: 12 };
    let err = verify_mir(&m).expect_err("a double transfer must reject");
    assert!(
        format!("{err:?}").contains("mir.env.field-moved"),
        "{err:?}"
    );
}
// ---------------------------------------------------------------------------
// R1-F04: state controls that PROVE the reached state first.
// ---------------------------------------------------------------------------

/// V-12 (提交后取消): step the program DETERMINISTICALLY into the closure
/// body (the callee frame is bound to a published env and the capture
/// clone has a distinct ledger identity), THEN raise the cancel flag and
/// resume: the outcome is Cancelled and the all-exit cleanup releases the
/// env (live=0). The pre-commit variant (cancel before CreateClosure's
/// clone) remains the earlier test; this is the committed-state axis.
#[test]
fn av_closures_823_sem_cancel_after_commit_proves_reached_state() {
    use auto_av_core::av::{execute_program_steps, resume_program, SteppedOutcome};
    let mut previous_ledger = None;
    for with_spans in [false, true] {
        let mut m = auto_av_probe::closures_fixture::first_case_module();
        // Keep the callee alive long enough for the documented cancellation poll.
        let instruction = m.funcs[1].blocks[0]
            .instrs
            .iter()
            .find(|i| matches!(i, MirInstr::Const { dst: 8, .. }))
            .unwrap()
            .clone();
        m.funcs[1].blocks[0]
            .instrs
            .splice(0..0, std::iter::repeat(instruction).take(1100));
        if with_spans {
            for (fi, function) in m.funcs.iter_mut().enumerate() {
                let count: usize = function.blocks.iter().map(|b| b.instrs.len()).sum();
                function.spans = (0..count)
                    .map(|i| {
                        (
                            i as u32,
                            auto_av_core::contract::SpanInfo {
                                func: fi as u32,
                                start: (i * 4) as u32,
                                end: (i * 4 + 4) as u32,
                            },
                        )
                    })
                    .collect();
            }
        }
        verify_mir(&m).expect("verifies");
        // Split into per-function units (the real D828 seam) and link a
        // snapshot — the stepping entry over the unit/snapshot path.
        let units = auto_av_probe::cache::split_units(&m).expect("splits");
        assert_eq!(units.len(), 3, "make/body/main");
        let mut codes = Vec::new();
        for u in &units {
            verify_mir(u).expect("unit verifies standalone (r8)");
            let wire = encode_abc(u).expect("unit encode");
            let loaded = verify_abc_bytes(&wire.to_flat(), &u.owner).expect("unit admit");
            codes.push(std::sync::Arc::new(
                auto_av_core::function_code::FunctionCode::admit_unit(loaded, 1)
                    .expect("unit-admit"),
            ));
        }
        let snap = std::sync::Arc::new(
            auto_av_core::program_snapshot::ProgramSnapshot::link(codes, "main", 1).expect("links"),
        );
        let cancel = auto_av_core::av::CancelFlag::new(false);
        // The counter includes fuel spent inside cloning. Advance until the
        // actual target frame is bound to a published environment.
        let cancel_ref = &cancel;
        let mut stepped = execute_program_steps(&snap, &ExecLimits::default(), cancel_ref, 1);
        let paused = loop {
            let SteppedOutcome::Paused(p) = stepped else {
                panic!("program finished before reaching the closure body");
            };
            if matches!(p.active_closure_frame(), Some(("body", _))) {
                break p;
            }
            assert!(p.step_count() < 1000, "closure body was never entered");
            stepped = resume_program(p, cancel_ref, Some(1));
        };
        let ledger = paused_trace_arena(&paused);
        let allocs = ledger.alloc_events();
        assert!(
            allocs.iter().any(|a| a.id == 3 && a.bytes == 4),
            "capture clone must be a distinct allocation: {allocs:?}"
        );
        // NOW raise the flag and resume — the commit-state cancellation.
        cancel_ref.store(true, std::sync::atomic::Ordering::Relaxed);
        let out = match resume_program(paused, cancel_ref, None) {
            SteppedOutcome::Finished(out, trace) => {
                let arena = trace.arena.clone().expect("ledger");
                assert_eq!(arena.live(), 0, "cancel unwinds with the env released");
                let events = (arena.alloc_events(), arena.free_events());
                if let Some(previous) = &previous_ledger {
                    assert_eq!(
                        previous, &events,
                        "strip must not change cancellation cleanup"
                    );
                } else {
                    previous_ledger = Some(events);
                }
                assert_eq!(
                    arena.free_events().iter().filter(|e| e.id == 3).count(),
                    1,
                    "the capture clone is released exactly once"
                );
                out
            }
            SteppedOutcome::Paused(_) => panic!("a raised cancel must finish, not re-park"),
        };
        assert!(matches!(out, ExecOutcome::Cancelled), "got {out:?}");
    }
}

fn paused_trace_arena(p: &auto_av_core::av::PausedRun) -> auto_av_core::owned_text::StrArena {
    p.arena().clone()
}

/// V-04 (同宽错签名): a CREATE_CLOSURE whose dst Closure type has the
/// SAME WIDTH but a different result type than the callee's signature
/// rejects at typing (`mir.closure.type`).
#[test]
fn av_closures_823_sem_same_width_wrong_signature_rejected() {
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    // Intern a SAME-ARITY different-result Closure type: (i32)->Str.
    let wrong = m.intern_type(MirType::Closure(FnSig {
        params: vec![0],
        result: 1,
    }));
    // make's dst slot 5 becomes the WRONG Closure type.
    m.funcs[0].slots[5].ty = wrong;
    // make's sig (7) result must follow — keep it wrong on purpose; the
    // CreateClosure arm fires first (dst vs callee sig).
    let err = verify_mir(&m).expect_err("a same-width wrong-signature closure must reject");
    assert!(format!("{err:?}").contains("mir.closure.type"), "{err:?}");
}

/// V-04 (generation/store 域, 运行防御): the front-door argument is
/// structural — ClosureRefV values originate ONLY from CreateClosure
/// inside the same execution (the verify gate bans every other producer),
/// env handles are monotonic per execution, and the store dies with it,
/// so a cross-store/stale-generation reference cannot be CONSTRUCTED by
/// an admitted program. The dispatch additionally re-checks the
/// handle-keyed generation invariant; this test pins the STATIC face:
/// CALL_CLOSURE on a slot that no CreateClosure could have produced
/// (an i32 slot) rejects at typing.
#[test]
fn av_closures_823_sem_closure_value_provenance_static() {
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    // A Copy from the i32 arg slot into the closure slot — the Copy's
    // typing (i32 into Closure) rejects: no instruction can forge a
    // closure value except CreateClosure.
    m.funcs[2].blocks[0].instrs.insert(
        1,
        MirInstr::Copy {
            dst: 0,
            src: 1, // i32 arg → Closure slot
        },
    );
    let err = verify_mir(&m).expect_err("forging a closure value via Copy must reject");
    assert!(
        format!("{err:?}").contains("mir.copy-type") || format!("{err:?}").contains("mir.type"),
        "{err:?}"
    );
}

/// V-11 (strip 前后一致): the SAME module with spans populated vs
/// stripped produces identical outcome AND ledger (diagnostics never
/// participate in semantics).
#[test]
fn av_closures_823_sem_strip_invariance() {
    let base = auto_av_probe::closures_fixture::first_case_module();
    let mut with_spans = base.clone();
    for (fi, f) in with_spans.funcs.iter_mut().enumerate() {
        let mut flat = 0u32;
        for b in f.blocks.iter() {
            for _ in b.instrs.iter() {
                f.spans.push((
                    flat,
                    auto_av_core::contract::SpanInfo {
                        func: fi as u32,
                        start: flat * 4,
                        end: flat * 4 + 4,
                    },
                ));
                flat += 1;
            }
        }
    }
    let (out_a, arena_a) = run_pipeline(&with_spans);
    let (out_b, arena_b) = run_pipeline(&base);
    assert_eq!(
        format!("{out_a:?}"),
        format!("{out_b:?}"),
        "spans never change the outcome"
    );
    assert_eq!(arena_a.alloc_events(), arena_b.alloc_events());
    assert_eq!(arena_a.free_events(), arena_b.free_events());
    assert_eq!(arena_a.live(), arena_b.live());
}

#[test]
fn av_closures_823_r2_move_alias_and_clone_control() {
    for (through_move, kind, split_block) in [
        (true, CaptureKind::Move, false),
        (true, CaptureKind::Move, true),
        (false, CaptureKind::Move, false),
        (false, CaptureKind::Clone, false),
        (false, CaptureKind::Clone, true),
    ] {
        let mut m = auto_av_probe::closures_fixture::first_case_module();
        let mut child = m.funcs[1].clone();
        child.name = "child".into();
        child.env = Some(vec![3]);
        child.blocks[0].instrs = vec![
            MirInstr::EnvGet { dst: 2, field: 0 },
            MirInstr::ListLen { dst: 6, src: 2 },
        ];
        child.blocks[0].term = Terminator::Return { value: 6 };
        m.funcs.push(child);
        let body = &mut m.funcs[1];
        let dst = body.slots.len() as u32;
        body.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: 5,
            name: Some("child".into()),
        });
        let alias = body.slots.len() as u32;
        body.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: 3,
            name: Some("alias".into()),
        });
        let alias2 = body.slots.len() as u32;
        body.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: 3,
            name: Some("alias2".into()),
        });
        let pos = body.blocks[0]
            .instrs
            .iter()
            .position(|i| matches!(i, MirInstr::EnvGet { field: 1, .. }))
            .unwrap()
            + 1;
        let mut extra = Vec::new();
        if through_move {
            extra.push(MirInstr::Move { dst: alias, src: 2 });
            if split_block {
                extra.push(MirInstr::Move {
                    dst: alias2,
                    src: alias,
                });
            }
        }
        extra.extend([
            MirInstr::CreateClosure {
                dst,
                callee: auto_av_core::mir::CalleeRef::Func(3),
                captures: vec![CaptureOperand {
                    kind,
                    src: if through_move && split_block {
                        alias2
                    } else if through_move {
                        alias
                    } else {
                        2
                    },
                }],
            },
            MirInstr::Drop { slot: dst },
            MirInstr::EnvGet { dst: 2, field: 1 },
        ]);
        body.blocks[0].instrs.splice(pos..pos, extra);
        if split_block {
            let rest = body.blocks[0].instrs.split_off(pos);
            let term = std::mem::replace(&mut body.blocks[0].term, Terminator::Jump { target: 1 });
            body.blocks.push(MirBlock { instrs: rest, term });
        }
        if kind == CaptureKind::Clone {
            let (out, arena) = run_pipeline(&m);
            assert!(matches!(
                out,
                ExecOutcome::Success {
                    value: Value::I32(29),
                    ..
                }
            ));
            assert_eq!(arena.live(), 0);
        } else {
            let err = verify_mir(&m).expect_err("environment aliases cannot transfer ownership");
            assert!(
                format!("{err:?}").contains("mir.capture.env-alias"),
                "{err:?}"
            );
        }
    }
}

#[test]
fn av_closures_823_r2_untrusted_env_dst_is_rejected_without_panic() {
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    verify_mir(&m).expect("valid control");
    m.funcs[1].blocks[0].instrs[0] = MirInstr::EnvGet {
        dst: u32::MAX,
        field: 0,
    };
    let err = verify_mir(&m).expect_err("bad destination is rejected");
    assert!(format!("{err:?}").contains("mir.slot-ref"), "{err:?}");
    let wire = encode_abc(&m).expect("untrusted ABC fixture encodes");
    assert!(verify_abc_bytes(&wire.to_flat(), &m.owner).is_err());
}

#[test]
fn av_closures_823_r2_field_move_from_env_alias_rejected() {
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    let c_auto = match m.funcs[0].blocks[0].instrs[0] {
        MirInstr::Const { value, .. } => value,
        _ => unreachable!(),
    };
    let make = &mut m.funcs[0];
    let text = make.slots.len() as u32;
    make.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 1,
        name: Some("extra_text".into()),
    });
    let record = make.slots.len() as u32;
    make.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 2,
        name: Some("extra_record".into()),
    });
    let site = make.blocks[0]
        .instrs
        .iter()
        .position(|i| matches!(i, MirInstr::CreateClosure { .. }))
        .unwrap();
    make.blocks[0].instrs.splice(
        site..site,
        [
            MirInstr::Const {
                dst: text,
                value: c_auto,
            },
            MirInstr::Aggregate {
                dst: record,
                record_ty: 2,
                operands: vec![text, 1],
            },
        ],
    );
    let MirInstr::CreateClosure { captures, .. } = &mut make.blocks[0].instrs[site + 2] else {
        unreachable!()
    };
    captures.push(CaptureOperand {
        kind: CaptureKind::Move,
        src: record,
    });
    m.funcs[1].env = Some(vec![1, 3, 2]);
    verify_mir(&m).expect("the added owned record is legal without field move");
    let body = &mut m.funcs[1];
    let rec_alias = body.slots.len() as u32;
    body.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 2,
        name: Some("env_record".into()),
    });
    let field = body.slots.len() as u32;
    body.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 1,
        name: Some("field".into()),
    });
    body.blocks[0].instrs.splice(
        0..0,
        [
            MirInstr::EnvGet {
                dst: rec_alias,
                field: 2,
            },
            MirInstr::FieldMove {
                dst: field,
                root: rec_alias,
                path: vec![0],
            },
        ],
    );
    let err = verify_mir(&m).expect_err("ordinary field move cannot transfer an env field");
    assert!(
        format!("{err:?}").contains("mir.capture.env-alias"),
        "{err:?}"
    );
}

#[test]
fn av_closures_823_r2_aggregate_cannot_consume_environment_alias() {
    for clone_first in [false, true] {
        let mut m = auto_av_probe::closures_fixture::first_case_module();
        let one = m.funcs[1].blocks[0]
            .instrs
            .iter()
            .find_map(|i| match i {
                MirInstr::Const { dst: 3, value } => Some(*value),
                _ => None,
            })
            .unwrap();
        let body = &mut m.funcs[1];
        let cloned = body.slots.len() as u32;
        body.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: 1,
            name: Some("independent_text".into()),
        });
        let record = body.slots.len() as u32;
        body.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: 2,
            name: Some("record".into()),
        });
        let mut add = vec![MirInstr::Const { dst: 3, value: one }];
        if clone_first {
            add.push(MirInstr::Clone {
                dst: cloned,
                src: 1,
            });
        }
        add.extend([
            MirInstr::Aggregate {
                dst: record,
                record_ty: 2,
                operands: vec![if clone_first { cloned } else { 1 }, 3],
            },
            MirInstr::Drop { slot: record },
        ]);
        body.blocks[0].instrs.splice(1..1, add);
        if clone_first {
            let (out, arena) = run_pipeline(&m);
            assert!(
                matches!(
                    out,
                    ExecOutcome::Success {
                        value: Value::I32(29),
                        ..
                    }
                ),
                "{out:?}"
            );
            assert_eq!(arena.live(), 0);
        } else {
            let err = verify_mir(&m).expect_err("aggregate must not consume the parent's alias");
            assert!(
                format!("{err:?}").contains("mir.capture.env-alias"),
                "{err:?}"
            );
        }
    }
}

#[test]
fn av_closures_823_r2_clone_redefinition_has_independent_owner() {
    for split in [false, true] {
        let mut m = auto_av_probe::closures_fixture::first_case_module();
        let mut child = m.funcs[1].clone();
        child.name = "child".into();
        child.env = Some(vec![1]);
        child.blocks[0].instrs = vec![
            MirInstr::EnvGet { dst: 1, field: 0 },
            MirInstr::View { dst: 11, src: 1 },
            MirInstr::StrLen { dst: 7, view: 11 },
        ];
        child.blocks[0].term = Terminator::Return { value: 7 };
        m.funcs.push(child);
        let body = &mut m.funcs[1];
        let child_slot = body.slots.len() as u32;
        body.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: 5,
            name: Some("child".into()),
        });
        body.blocks[0].instrs.splice(
            1..1,
            [
                MirInstr::Clone { dst: 1, src: 1 },
                MirInstr::CreateClosure {
                    dst: child_slot,
                    callee: auto_av_core::mir::CalleeRef::Func(3),
                    captures: vec![CaptureOperand {
                        kind: CaptureKind::Move,
                        src: 1,
                    }],
                },
                MirInstr::Drop { slot: child_slot },
                MirInstr::EnvGet { dst: 1, field: 0 },
            ],
        );
        if split {
            // Clone occurs before the jump; its independently owned result is
            // consumed by a capture in the successor block.
            let body = &mut m.funcs[1];
            let rest = body.blocks[0].instrs.split_off(2);
            let term = std::mem::replace(&mut body.blocks[0].term, Terminator::Jump { target: 1 });
            body.blocks.push(MirBlock { instrs: rest, term });
        }
        let (out, arena) = run_pipeline(&m);
        assert!(
            matches!(
                out,
                ExecOutcome::Success {
                    value: Value::I32(29),
                    ..
                }
            ),
            "{out:?}"
        );
        assert_eq!(arena.live(), 0);
    }
}

#[test]
fn av_closures_823_r2_branch_join_retains_possible_alias() {
    for mixed in [true, false] {
        let mut m = auto_av_probe::closures_fixture::first_case_module();
        let bool_ty = m.intern_type(MirType::Bool);
        let truth = m.intern_const(MirConst::Bool(true));
        let mut child = m.funcs[1].clone();
        child.name = "child".into();
        child.env = Some(vec![3]);
        child.blocks[0].instrs = vec![
            MirInstr::EnvGet { dst: 2, field: 0 },
            MirInstr::ListLen { dst: 6, src: 2 },
        ];
        child.blocks[0].term = Terminator::Return { value: 6 };
        m.funcs.push(child);
        let body = &mut m.funcs[1];
        let child_slot = body.slots.len() as u32;
        body.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: 5,
            name: Some("child".into()),
        });
        let value = body.slots.len() as u32;
        body.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: 3,
            name: Some("selected".into()),
        });
        let cond = body.slots.len() as u32;
        body.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: bool_ty,
            name: Some("choice".into()),
        });
        let split = body.blocks[0]
            .instrs
            .iter()
            .position(|i| matches!(i, MirInstr::EnvGet { field: 1, .. }))
            .unwrap()
            + 1;
        let tail = body.blocks[0].instrs.split_off(split);
        body.blocks[0].instrs.push(MirInstr::Const {
            dst: cond,
            value: truth,
        });
        let original_term = std::mem::replace(
            &mut body.blocks[0].term,
            Terminator::Branch {
                cond,
                then: 1,
                else_: 2,
            },
        );
        body.blocks.push(MirBlock {
            instrs: vec![if mixed {
                MirInstr::Move { dst: value, src: 2 }
            } else {
                MirInstr::Clone { dst: value, src: 2 }
            }],
            term: Terminator::Jump { target: 3 },
        });
        body.blocks.push(MirBlock {
            instrs: vec![MirInstr::Clone { dst: value, src: 2 }],
            term: Terminator::Jump { target: 3 },
        });
        let mut joined = vec![
            MirInstr::CreateClosure {
                dst: child_slot,
                callee: auto_av_core::mir::CalleeRef::Func(3),
                captures: vec![CaptureOperand {
                    kind: CaptureKind::Move,
                    src: value,
                }],
            },
            MirInstr::Drop { slot: child_slot },
            MirInstr::EnvGet { dst: 2, field: 1 },
        ];
        joined.extend(tail);
        body.blocks.push(MirBlock {
            instrs: joined,
            term: original_term,
        });
        if mixed {
            let err =
                verify_mir(&m).expect_err("an alias on either incoming branch must be rejected");
            assert!(
                format!("{err:?}").contains("mir.capture.env-alias"),
                "{err:?}"
            );
        } else {
            let (out, arena) = run_pipeline(&m);
            assert!(
                matches!(
                    out,
                    ExecOutcome::Success {
                        value: Value::I32(29),
                        ..
                    }
                ),
                "{out:?}"
            );
            assert_eq!(arena.live(), 0);
        }
    }
}

#[test]
fn av_closures_823_r2_removed_list_element_has_independent_owner() {
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    let zero = m.intern_const(MirConst::I32(0));
    let one = m.intern_const(MirConst::I32(1));
    let mut child = m.funcs[1].clone();
    child.name = "child".into();
    child.env = Some(vec![2]);
    child.blocks[0].instrs = vec![MirInstr::Const { dst: 6, value: one }];
    child.blocks[0].term = Terminator::Return { value: 6 };
    m.funcs.push(child);
    let body = &mut m.funcs[1];
    let child_slot = body.slots.len() as u32;
    body.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 5,
        name: Some("child".into()),
    });
    let removed = body.slots.len() as u32;
    body.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 2,
        name: Some("removed".into()),
    });
    let pos = body.blocks[0]
        .instrs
        .iter()
        .position(|i| matches!(i, MirInstr::EnvGet { field: 1, .. }))
        .unwrap()
        + 1;
    body.blocks[0].instrs.splice(
        pos..pos,
        [
            MirInstr::Const {
                dst: 3,
                value: zero,
            },
            MirInstr::ListRemove {
                dst: removed,
                root: 2,
                index: 3,
            },
            MirInstr::CreateClosure {
                dst: child_slot,
                callee: auto_av_core::mir::CalleeRef::Func(3),
                captures: vec![CaptureOperand {
                    kind: CaptureKind::Move,
                    src: removed,
                }],
            },
            MirInstr::Drop { slot: child_slot },
        ],
    );
    let (out, arena) = run_pipeline(&m);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(19),
                ..
            }
        ),
        "{out:?}"
    );
    assert_eq!(arena.live(), 0);
}

#[test]
fn av_closures_823_r2_prescan_rejects_other_bad_indices() {
    let invalid = [
        MirInstr::Move {
            dst: u32::MAX,
            src: 1,
        },
        MirInstr::Move {
            dst: 1,
            src: u32::MAX,
        },
        MirInstr::FieldMove {
            dst: u32::MAX,
            root: 1,
            path: vec![0],
        },
        MirInstr::FieldMove {
            dst: 1,
            root: u32::MAX,
            path: vec![0],
        },
        MirInstr::EnvGet {
            dst: 1,
            field: u16::MAX,
        },
    ];
    for instr in invalid {
        let mut m = auto_av_probe::closures_fixture::first_case_module();
        m.funcs[1].blocks[0].instrs[0] = instr;
        assert!(verify_mir(&m).is_err(), "typed input must be rejected");
        let wire = encode_abc(&m).expect("encode untrusted fixture");
        assert!(
            verify_abc_bytes(&wire.to_flat(), &m.owner).is_err(),
            "ABC input must be rejected"
        );
    }
    let mut m = auto_av_probe::closures_fixture::first_case_module();
    let create = m.funcs[0].blocks[0]
        .instrs
        .iter_mut()
        .find(|i| matches!(i, MirInstr::CreateClosure { .. }))
        .unwrap();
    let MirInstr::CreateClosure { captures, .. } = create else {
        unreachable!()
    };
    captures[0] = CaptureOperand {
        kind: CaptureKind::MoveFromEnv,
        src: u32::MAX,
    };
    assert!(
        verify_mir(&m).is_err(),
        "invalid environment field must reject"
    );
    let wire = encode_abc(&m).expect("encode untrusted fixture");
    assert!(verify_abc_bytes(&wire.to_flat(), &m.owner).is_err());
}
