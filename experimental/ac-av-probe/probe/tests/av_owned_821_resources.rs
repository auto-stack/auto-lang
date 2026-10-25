//! PLAN-821 T-04 resource/admission counter-controls (`av_owned_821`
//! family): the frozen V-03 (allocation-failure transactions), V-04
//! (all-exit cleanup) and V-07 (cap boundary ±1B, fault accounting)
//! matrices (contract-decision §12) through the REAL admission chain with
//! the arena-level fault seam (`StrArena::set_fault_at`) and the
//! pre-allocation budget precheck (预算前拒 attempt=0). Oracles: distinct
//! state families (Resource/Infra/Cancelled — never trap, never success),
//! prior events preserved, every live root released exactly once in
//! reverse creation order, no panic.

use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::av::{execute_with_arena, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, ResourceKind, MAX_AV_SLOT_BYTES, MIR_OWNED_SCHEMA};
use auto_av_core::mir::*;
use auto_av_core::{abc_verify, mir_verify};

fn owner(name: &str) -> OwnerScope {
    OwnerScope {
        schema: MIR_OWNED_SCHEMA.to_string(),
        module: name.to_string(),
    }
}

struct T {
    str: u32,
    i32: u32,
    bool: u32,
    view: u32,
    note: u32,
}

fn types(m: &mut MirModule) -> T {
    let str = m.intern_type(MirType::Str);
    T {
        str,
        i32: m.intern_type(MirType::I32),
        bool: m.intern_type(MirType::Bool),
        view: m.intern_type(MirType::StrView),
        note: m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "res".into(),
            name: "Note".into(),
            fields: vec![MirFieldDef {
                name: "body".into(),
                ty: str,
            }],
        })),
    }
}

fn temp(ty: u32) -> MirSlot {
    MirSlot {
        kind: SlotKind::Temp,
        ty,
        name: Some("t".into()),
    }
}

fn local(ty: u32) -> MirSlot {
    MirSlot {
        kind: SlotKind::Local,
        ty,
        name: Some("l".into()),
    }
}

/// Admit a module (verify → encode → decode → deep admission).
fn admit(m: &MirModule) -> abc_verify::LoadedModule {
    mir_verify::verify_mir(m).expect("owned MIR verifies");
    let wire = encode_abc(m).expect("encode");
    let decoded = decode_abc(&wire.to_flat()).expect("decode");
    abc_verify::verify_abc(decoded, &m.owner).expect("AVAD v2 admits")
}

/// One `d_main() -> i32` function frame with the given slots/blocks.
fn main_i32(m: &mut MirModule, t: &T, slots: Vec<MirSlot>, blocks: Vec<MirBlock>) {
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t.i32,
    }));
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots,
        blocks,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    m.entry = 0;
}

/// Assert the full-cleanup oracle: every allocation freed EXACTLY ONCE
/// and live=0 (no reap_all) — the hard invariant on every exit.
fn assert_full_cleanup(arena: &auto_av_core::owned_text::StrArena) {
    let allocs = arena.alloc_events();
    let frees = arena.free_events();
    assert_eq!(
        frees.len(),
        allocs.len(),
        "every buffer must free exactly once (allocs {:?}, frees {:?})",
        allocs,
        frees
    );
    let mut seen = std::collections::BTreeSet::new();
    for f in &frees {
        assert!(seen.insert(f.id), "double free of buffer {}", f.id);
    }
    for a in &allocs {
        assert!(seen.contains(&a.id), "buffer {} never freed", a.id);
    }
    assert_eq!(arena.live(), 0);
}

/// The STRICT reverse-creation release order — valid when no view/share
/// extends a buffer's lifetime past its owning root's drop (774 snapshot
/// semantics legitimately reorder the final release then).
fn assert_strict_lifo(arena: &auto_av_core::owned_text::StrArena) {
    assert_full_cleanup(arena);
    let allocs = arena.alloc_events();
    let mut creation_desc: Vec<u64> = allocs.iter().map(|a| a.id).collect();
    creation_desc.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(
        arena.free_events().iter().map(|f| f.id).collect::<Vec<_>>(),
        creation_desc,
        "reverse-creation (LIFO) release order"
    );
}

fn run(
    m: &MirModule,
    limits: &ExecLimits,
    arena: &auto_av_core::owned_text::StrArena,
) -> (ExecOutcome, auto_av_core::av::ExecTrace) {
    let loaded = admit(m);
    execute_with_arena(&loaded, limits, &CancelFlag::new(false), arena.clone())
}

// ---- V-03: allocation-failure transactions ----------------------------------

/// The Nth allocation ATTEMPT fails: attempt recorded, successful_alloc
/// unchanged (V-03's frozen accounting), the outcome is the distinct
/// infra family, and the prior (const) buffer releases exactly once at
/// the exit cleanup.
#[test]
fn av_owned_821_v03_clone_fault_attempt_counted_no_success() {
    let mut m = MirModule::new_owned(owner("v03a"));
    let t = types(&mut m);
    let c_aa = m.intern_const(MirConst::Str(b"AA".to_vec().into_boxed_slice()));
    let c_zero = m.intern_const(MirConst::I32(0));
    main_i32(
        &mut m,
        &t,
        vec![local(t.str), temp(t.str), temp(t.i32)],
        vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_aa,
                },
                MirInstr::Clone { dst: 1, src: 0 },
                MirInstr::Const {
                    dst: 2,
                    value: c_zero,
                },
            ],
            term: Terminator::Return { value: 2 },
        }],
    );
    let arena = auto_av_core::owned_text::StrArena::new();
    arena.set_fault_at(2); // attempt 1 = the const, attempt 2 = the clone
    let (out, trace) = run(&m, &ExecLimits::default(), &arena);
    match &out {
        ExecOutcome::Infra(msg) => assert!(
            msg.contains("fault-injected at owned allocation attempt #2"),
            "the fault must name its attempt: {msg}"
        ),
        other => panic!("the injected allocator fault must be the infra family, got {other:?}"),
    }
    let arena = trace.arena.as_ref().unwrap();
    assert_eq!(arena.attempts(), 2, "the failed attempt IS recorded");
    assert_eq!(arena.successful_allocs(), 1, "only the const succeeded");
    assert_full_cleanup(arena);
}

/// Mid-REPLACE preparation failure: the StrToOwned materialization of
/// the RHS faults — no new successful allocation; the root's old buffer
/// and every prior value release exactly once at the exit (失败保留旧值
/// 和源责任：无半提交).
#[test]
fn av_owned_821_v03_toowned_fault_mid_replace_preserves_old() {
    let mut m = MirModule::new_owned(owner("v03b"));
    let t = types(&mut m);
    let c_aaaa = m.intern_const(MirConst::Str(b"AAAA".to_vec().into_boxed_slice()));
    let c_zero = m.intern_const(MirConst::I32(0));
    main_i32(
        &mut m,
        &t,
        vec![
            local(t.note), // 0: u
            temp(t.str),   // 1: lit
            temp(t.view),  // 2: RHS view
            temp(t.str),   // 3: RHS owned (would-be)
            temp(t.i32),   // 4: ret
        ],
        vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 1,
                    value: c_aaaa,
                },
                MirInstr::Aggregate {
                    dst: 0,
                    record_ty: t.note,
                    operands: vec![1],
                },
                MirInstr::FieldGet {
                    dst: 2,
                    base: 0,
                    path: vec![0],
                },
                MirInstr::StrToOwned { dst: 3, view: 2 },
                MirInstr::Const {
                    dst: 4,
                    value: c_zero,
                },
            ],
            term: Terminator::Return { value: 4 },
        }],
    );
    let arena = auto_av_core::owned_text::StrArena::new();
    arena.set_fault_at(2); // attempt 1 = the const; attempt 2 = the RHS copy
    let (out, trace) = run(&m, &ExecLimits::default(), &arena);
    assert!(
        matches!(&out, ExecOutcome::Infra(m) if m.contains("fault-injected")),
        "expected the infra fault family, got {out:?}"
    );
    let arena = trace.arena.as_ref().unwrap();
    assert_eq!(arena.attempts(), 2);
    assert_eq!(arena.successful_allocs(), 1, "nothing new committed");
    assert_full_cleanup(arena);
}

/// Mid-CONSTRUCTION failure: cloning a TWO-Str-field record faults on
/// the second buffer — the partially cloned first buffer releases
/// exactly once with the rest (临时恰清理一次).
#[test]
fn av_owned_821_v03_record_clone_mid_failure_released_once() {
    let mut m = MirModule::new_owned(owner("v03c"));
    let t = types(&mut m);
    let t_pair = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "res".into(),
        name: "Pair".into(),
        fields: vec![
            MirFieldDef {
                name: "a".into(),
                ty: t.str,
            },
            MirFieldDef {
                name: "b".into(),
                ty: t.str,
            },
        ],
    }));
    let c_x = m.intern_const(MirConst::Str(b"xx".to_vec().into_boxed_slice()));
    let c_y = m.intern_const(MirConst::Str(b"yy".to_vec().into_boxed_slice()));
    let c_zero = m.intern_const(MirConst::I32(0));
    main_i32(
        &mut m,
        &t,
        vec![
            local(t_pair), // 0: p
            temp(t.str),   // 1: lit a
            temp(t.str),   // 2: lit b
            temp(t_pair),  // 3: clone dst
            temp(t.i32),   // 4: ret
        ],
        vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 1, value: c_x },
                MirInstr::Const { dst: 2, value: c_y },
                MirInstr::Aggregate {
                    dst: 0,
                    record_ty: t_pair,
                    operands: vec![1, 2],
                },
                MirInstr::Clone { dst: 3, src: 0 },
                MirInstr::Const {
                    dst: 4,
                    value: c_zero,
                },
            ],
            term: Terminator::Return { value: 4 },
        }],
    );
    let arena = auto_av_core::owned_text::StrArena::new();
    // attempts 1,2 = the two literals; attempt 3 = the clone's FIRST
    // buffer; attempt 4 = the second -> fault mid-construction.
    arena.set_fault_at(4);
    let (out, trace) = run(&m, &ExecLimits::default(), &arena);
    assert!(
        matches!(&out, ExecOutcome::Infra(m) if m
            .contains("fault-injected at owned allocation attempt #4")),
        "the mid-construction fault must name attempt 4: {out:?}"
    );
    let arena = trace.arena.as_ref().unwrap();
    assert_eq!(arena.attempts(), 4);
    // 3 successful: two literals + the clone's FIRST buffer (released
    // with the partially built value).
    assert_eq!(arena.successful_allocs(), 3);
    assert_full_cleanup(arena);
}

// ---- V-04: all-exit cleanup ---------------------------------------------------

/// Fuel exhaustion, frame exhaustion, slot-byte budget and cancellation:
/// each distinct state family releases every live root exactly once in
/// reverse creation order (取消/耗尽不吞清理).
#[test]
fn av_owned_821_v04_all_exit_kinds_clean_lifo() {
    // (a) FUEL: two consts then an arithmetic tail; fuel stops mid-run
    // with both buffers live.
    let mut m = MirModule::new_owned(owner("v04a"));
    let t = types(&mut m);
    let c_aa = m.intern_const(MirConst::Str(b"AA".to_vec().into_boxed_slice()));
    let c_bb = m.intern_const(MirConst::Str(b"B".to_vec().into_boxed_slice()));
    let c_zero = m.intern_const(MirConst::I32(0));
    main_i32(
        &mut m,
        &t,
        vec![local(t.str), local(t.str), temp(t.i32), temp(t.i32)],
        vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_aa,
                },
                MirInstr::Const {
                    dst: 1,
                    value: c_bb,
                },
                MirInstr::Const {
                    dst: 2,
                    value: c_zero,
                },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 3,
                    lhs: 2,
                    rhs: 2,
                },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 2,
                    lhs: 3,
                    rhs: 3,
                },
            ],
            term: Terminator::Return { value: 2 },
        }],
    );
    let tiny = ExecLimits {
        max_steps: 3, // stops mid-run (deep-size visits count as steps)
        ..ExecLimits::default()
    };
    let arena = auto_av_core::owned_text::StrArena::new();
    let (out, trace) = run(&m, &tiny, &arena);
    match &out {
        ExecOutcome::Resource { kind, .. } => assert_eq!(*kind, ResourceKind::InstrSteps),
        other => panic!("fuel exhaustion must be Resource(InstrSteps), got {other:?}"),
    }
    let arena = trace.arena.as_ref().unwrap();
    assert!(
        arena.successful_allocs() >= 1,
        "at least the first literal ran"
    );
    assert_strict_lifo(arena);

    // (b) FRAMES: f1 consts one Str per frame and move-in calls ITSELF
    // — the frame budget rejects the 1025th frame and every frame's value
    // releases. The zero-param entry d_main seeds the recursion.
    let mut m = MirModule::new_owned(owner("v04b"));
    let t = types(&mut m);
    let c_x = m.intern_const(MirConst::Str(b"x".to_vec().into_boxed_slice()));
    let c_zero = m.intern_const(MirConst::I32(0));
    let sig_i32 = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: t.i32,
    }));
    let sig_move = m.intern_type(MirType::Fn(FnSig {
        params: vec![t.str],
        result: t.i32,
    }));
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig: sig_i32,
        slots: vec![temp(t.str), temp(t.i32)],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 0, value: c_x },
                MirInstr::Call {
                    callee: CalleeRef::Func(1),
                    dst: 1,
                    args: vec![0], // move-in: the caller slot dies
                },
            ],
            term: Terminator::Return { value: 1 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "f1".into(),
        sig: sig_move,
        slots: vec![
            MirSlot {
                kind: SlotKind::Param,
                ty: t.str,
                name: Some("seed".into()),
            },
            temp(t.str),
            temp(t.i32),
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 1, value: c_x },
                MirInstr::Const {
                    dst: 2,
                    value: c_zero,
                },
                MirInstr::Call {
                    callee: CalleeRef::Func(1),
                    dst: 2,
                    args: vec![1], // self move-in recursion
                },
            ],
            term: Terminator::Return { value: 2 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    m.entry = 0;
    let arena = auto_av_core::owned_text::StrArena::new();
    let (out, trace) = run(&m, &ExecLimits::default(), &arena);
    match &out {
        ExecOutcome::Resource { kind, .. } => assert_eq!(*kind, ResourceKind::Frames),
        other => panic!("frame exhaustion must be Resource(Frames), got {other:?}"),
    }
    let arena = trace.arena.as_ref().unwrap();
    assert!(
        arena.successful_allocs() >= 1024,
        "a literal per frame until the frame budget ({})",
        arena.successful_allocs()
    );
    assert_strict_lifo(arena);

    // (c) SLOT BYTES: 3MiB const + clone + clone-of-clone — the third
    // 3MiB buffer crosses the 8MiB live-data cap before any attempt.
    let mut m = MirModule::new_owned(owner("v04c"));
    let t = types(&mut m);
    let big = "z".repeat(3 * 1024 * 1024);
    let c_big = m.intern_const(MirConst::Str(big.as_bytes().to_vec().into_boxed_slice()));
    let c_zero = m.intern_const(MirConst::I32(0));
    main_i32(
        &mut m,
        &t,
        vec![local(t.str), local(t.str), local(t.str), temp(t.i32)],
        vec![MirBlock {
            instrs: vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_big,
                },
                MirInstr::Clone { dst: 1, src: 0 },
                MirInstr::Clone { dst: 2, src: 1 },
                MirInstr::Const {
                    dst: 3,
                    value: c_zero,
                },
            ],
            term: Terminator::Return { value: 3 },
        }],
    );
    let arena = auto_av_core::owned_text::StrArena::new();
    // R1-F06 byte fuel: each 3MiB clone burns 3M steps — keep the step
    // budget above the chain's real byte work so the THIRD clone's
    // slot-bytes PRECHECK is the rejecting face (not InstrSteps).
    let limits = ExecLimits {
        max_steps: 20_000_000,
        ..ExecLimits::default()
    };
    let (out, trace) = run(&m, &limits, &arena);
    match &out {
        ExecOutcome::Resource { kind, .. } => assert_eq!(*kind, ResourceKind::SlotBytes),
        other => panic!("the third 3MiB buffer must be Resource(SlotBytes), got {other:?}"),
    }
    let arena = trace.arena.as_ref().unwrap();
    assert_eq!(
        arena.attempts(),
        2,
        "the over-budget attempt is NOT recorded"
    );
    assert_eq!(arena.successful_allocs(), 2);
    assert_full_cleanup(arena);
}

/// Cancellation mid-run: a long loop holds two buffers; the flag flips
/// after the allocations — the Cancelled outcome cleans everything
/// (取消不吞清理). The loop's step bound exceeds any plausible completion
/// before the flip, keeping the outcome deterministic.
#[test]
fn av_owned_821_v04_cancel_mid_run_cleans() {
    let mut m = MirModule::new_owned(owner("v04d"));
    let t = types(&mut m);
    let c_aa = m.intern_const(MirConst::Str(b"AA".to_vec().into_boxed_slice()));
    let c_bb = m.intern_const(MirConst::Str(b"B".to_vec().into_boxed_slice()));
    let c_true = m.intern_const(MirConst::Bool(true));
    let c_zero = m.intern_const(MirConst::I32(0));
    let c_one = m.intern_const(MirConst::I32(1));
    let c_bound = m.intern_const(MirConst::I32(400_000_000));
    main_i32(
        &mut m,
        &t,
        vec![
            local(t.str), // 0
            local(t.str), // 1
            temp(t.bool), // 2: cond
            temp(t.i32),  // 3: n
            temp(t.i32),  // 4: one
            temp(t.i32),  // 5: bound
            temp(t.i32),  // 6: ret
        ],
        vec![
            MirBlock {
                instrs: vec![
                    MirInstr::Const {
                        dst: 0,
                        value: c_aa,
                    },
                    MirInstr::Const {
                        dst: 1,
                        value: c_bb,
                    },
                    MirInstr::Const {
                        dst: 3,
                        value: c_zero,
                    },
                    MirInstr::Const {
                        dst: 4,
                        value: c_one,
                    },
                    MirInstr::Const {
                        dst: 5,
                        value: c_bound,
                    },
                    MirInstr::Const {
                        dst: 2,
                        value: c_true,
                    },
                ],
                term: Terminator::Branch {
                    cond: 2,
                    then: 1,
                    else_: 2,
                },
            },
            MirBlock {
                instrs: vec![
                    MirInstr::Bin {
                        op: BinOp::AddI32,
                        dst: 3,
                        lhs: 3,
                        rhs: 4,
                    },
                    MirInstr::Bin {
                        op: BinOp::LtI32,
                        dst: 2,
                        lhs: 3,
                        rhs: 5,
                    },
                ],
                term: Terminator::Branch {
                    cond: 2,
                    then: 1,
                    else_: 2,
                },
            },
            MirBlock {
                instrs: vec![MirInstr::Const {
                    dst: 6,
                    value: c_zero,
                }],
                term: Terminator::Return { value: 6 },
            },
        ],
    );
    let loaded = admit(&m);
    let flag = std::sync::Arc::new(CancelFlag::new(false));
    let worker_flag = std::sync::Arc::clone(&flag);
    // The arena/values carry Rc — NOT Send; the worker extracts the
    // observable faces (plain data) and the assertions run on those.
    let handle: std::thread::JoinHandle<(String, u64, usize, Vec<(u64, usize)>, Vec<u64>, usize)> =
        std::thread::spawn(move || {
            let arena = auto_av_core::owned_text::StrArena::new();
            let limits = ExecLimits {
                max_steps: 1_000_000_000,
                wall: std::time::Duration::from_secs(30),
                ..ExecLimits::default()
            };
            let (out, trace) = execute_with_arena(&loaded, &limits, &worker_flag, arena);
            let kind = match &out {
                ExecOutcome::Cancelled => "cancelled".to_string(),
                other => format!("{other:?}"),
            };
            let arena = trace.arena.as_ref().unwrap();
            (
                kind,
                arena.attempts(),
                arena.successful_allocs(),
                arena
                    .alloc_events()
                    .iter()
                    .map(|a| (a.id, a.bytes))
                    .collect(),
                arena.free_events().iter().map(|f| f.id).collect(),
                arena.live(),
            )
        });
    std::thread::sleep(std::time::Duration::from_millis(50));
    flag.store(true, std::sync::atomic::Ordering::SeqCst);
    let (kind, attempts, successful, allocs, free_ids, live) =
        handle.join().expect("cancel thread");
    assert_eq!(kind, "cancelled", "the flipped flag must cancel");
    assert_eq!(successful, 2, "both buffers were live at cancel");
    assert_eq!(attempts, 2);
    // Full cleanup on the extracted faces: each allocation freed exactly
    // once, reverse creation order, live=0.
    assert_eq!(free_ids.len(), allocs.len());
    let mut creation_desc: Vec<u64> = allocs.iter().map(|a| a.0).collect();
    creation_desc.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(free_ids, creation_desc, "reverse-creation release order");
    assert_eq!(live, 0);
}

// ---- V-07: cap boundary ±1B and transient peak -------------------------------

/// The frozen 8MiB live-data cap, probed ±1 byte through a concat chain
/// with a byte-granular padding local. A first run with zero padding
/// CALIBRATES the exact accounting (the observed peak includes every
/// slot overhead); the second run pads to EXACTLY the cap (passes at the
/// boundary with peak == cap); the third pads one more byte (rejects
/// BEFORE the attempt, message names cap+1, prior events preserved).
#[test]
fn av_owned_821_v07_cap_boundary_pass_and_over_by_one() {
    fn build(pad: usize) -> MirModule {
        let mut m = MirModule::new_owned(owner("v07"));
        let t = types(&mut m);
        let l = 700_000usize;
        let c_a = m.intern_const(MirConst::Str(
            "a".repeat(l).as_bytes().to_vec().into_boxed_slice(),
        ));
        let c_b = m.intern_const(MirConst::Str(
            "b".repeat(l).as_bytes().to_vec().into_boxed_slice(),
        ));
        let c_e = m.intern_const(MirConst::Str(b"e".to_vec().into_boxed_slice()));
        let c_g = m.intern_const(MirConst::Str(
            "g".repeat(pad).as_bytes().to_vec().into_boxed_slice(),
        ));
        let c_zero = m.intern_const(MirConst::I32(0));
        // slots: 0 A, 1 B, 2 E, 3 G, 4 va, 5 vb, 6 C, 7 vc, 8 ve,
        // 9 D, 10 vd, 11 va2, 12 F, 13 bytes, 14 ret
        main_i32(
            &mut m,
            &t,
            vec![
                local(t.str),
                local(t.str),
                local(t.str),
                local(t.str),
                temp(t.view),
                temp(t.view),
                local(t.str),
                temp(t.view),
                temp(t.view),
                local(t.str),
                temp(t.view),
                temp(t.view),
                local(t.str),
                temp(t.i32),
                temp(t.i32),
            ],
            vec![MirBlock {
                instrs: vec![
                    MirInstr::Const { dst: 0, value: c_a },
                    MirInstr::Const { dst: 1, value: c_b },
                    MirInstr::Const { dst: 2, value: c_e },
                    MirInstr::Const { dst: 3, value: c_g },
                    MirInstr::View { dst: 4, src: 0 },
                    MirInstr::View { dst: 5, src: 1 },
                    // C = A + B
                    MirInstr::StrConcat {
                        dst: 6,
                        lhs: 4,
                        rhs: 5,
                    },
                    MirInstr::View { dst: 7, src: 6 },
                    MirInstr::View { dst: 8, src: 2 },
                    // D = C + E
                    MirInstr::StrConcat {
                        dst: 9,
                        lhs: 7,
                        rhs: 8,
                    },
                    MirInstr::View { dst: 10, src: 9 },
                    MirInstr::View { dst: 11, src: 0 },
                    // F = D + A — the probed allocation.
                    MirInstr::StrConcat {
                        dst: 12,
                        lhs: 10,
                        rhs: 11,
                    },
                    MirInstr::StrLenBytes { dst: 13, view: 10 },
                    MirInstr::Const {
                        dst: 14,
                        value: c_zero,
                    },
                    MirInstr::BorrowEnd { token: 4 },
                    MirInstr::BorrowEnd { token: 5 },
                    MirInstr::BorrowEnd { token: 7 },
                    MirInstr::BorrowEnd { token: 8 },
                    MirInstr::BorrowEnd { token: 10 },
                    MirInstr::BorrowEnd { token: 11 },
                ],
                term: Terminator::Return { value: 13 },
            }],
        );
        m
    }

    // Calibration: pad=0 executes; the observed peak IS the exact live
    // maximum (slot overheads included). R1-F06 byte fuel: the concat
    // chain copies ~5.6MB — the step budget stays above the byte work so
    // the CAP (not InstrSteps) is the discriminating budget.
    let limits = ExecLimits {
        max_steps: 100_000_000,
        wall: std::time::Duration::from_secs(30),
        ..ExecLimits::default()
    };
    let arena = auto_av_core::owned_text::StrArena::new();
    let (out, trace) = run(&build(0), &limits, &arena);
    let ExecOutcome::Success { value, .. } = out else {
        panic!("the calibration chain must succeed, got {out:?}")
    };
    let auto_av_core::contract::Value::I32(got) = value else {
        panic!("i32 result")
    };
    assert_eq!(got, 1_400_001, "D = A+B+E byte-exact");
    let p0 = trace.peak_slot_bytes;
    assert!(p0 > 0 && p0 < MAX_AV_SLOT_BYTES as u64);
    assert_full_cleanup(trace.arena.as_ref().unwrap());

    // At-cap: pad exactly to the boundary — passes with peak == cap.
    let pad_pass = (MAX_AV_SLOT_BYTES as u64 - p0) as usize;
    let arena = auto_av_core::owned_text::StrArena::new();
    let (out, trace) = run(&build(pad_pass), &limits, &arena);
    let ExecOutcome::Success { .. } = out else {
        panic!("the at-cap chain must succeed, got {out:?}")
    };
    assert_eq!(
        trace.peak_slot_bytes, MAX_AV_SLOT_BYTES as u64,
        "the transient peak sits exactly at the frozen cap"
    );
    assert_full_cleanup(trace.arena.as_ref().unwrap());

    // Over by ONE byte: the same chain padded one more — rejected BEFORE
    // the attempt with the exact over-budget size named.
    let arena = auto_av_core::owned_text::StrArena::new();
    let (out, trace) = run(&build(pad_pass + 1), &limits, &arena);
    match &out {
        ExecOutcome::Resource { kind, message } => {
            assert_eq!(*kind, ResourceKind::SlotBytes);
            assert!(
                message.contains(&(MAX_AV_SLOT_BYTES + 1).to_string()),
                "the rejection must name the exact over-by-one size: {message}"
            );
        }
        other => panic!("over-by-one must reject, got {other:?}"),
    }
    let arena = trace.arena.as_ref().unwrap();
    // 5 successful allocations (A, B, E, G, C, D) minus F — wait: A, B,
    // E, G, C, D = 6; F never attempted.
    assert_eq!(arena.successful_allocs(), 6, "prior events preserved");
    assert_eq!(
        arena.attempts(),
        6,
        "the over-budget request is rejected before any attempt"
    );
    assert_full_cleanup(arena);
}

// ---------------------------------------------------------------------------
// R1-F09: the owned worker's process/managed-memory HARD GATES — the
// PARENT samples the child's private-bytes peak (never the worker's own
// word), the frozen B-line (128MiB) is ENFORCED, the managed store stays
// inside its 32MiB partition, and the gate LOGIC itself is counter-
// controlled (a zero threshold must FAIL).
// ---------------------------------------------------------------------------

/// Spawn ONE owned-request child under parent-side sampling; returns
/// (child peak private bytes, exit ok).
fn sampled_owned_request(
    worker: &std::path::Path,
    store: &std::path::Path,
    src: &std::path::Path,
) -> (u64, bool) {
    use std::process::{Command, Stdio};
    let mut child = Command::new(worker)
        .args([
            "owned-request",
            "--store",
            store.to_str().unwrap(),
            "--src-file",
            src.to_str().unwrap(),
            "--entry",
            "d_main",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn owned worker");
    let sampler = auto_av_probe::mempeek::ChildPeakSampler::start(child.id(), 5)
        .expect("the parent samples the child");
    let out = child.wait().expect("child exits");
    let (peak, samples) = sampler.stop();
    assert!(samples > 0, "the sampler actually polled ({samples})");
    (peak, out.success())
}

/// The frozen memory lines for the owned worker (798 r7 discipline, the
/// plan's OWN stricter text): B-line worker private-bytes peak ≤128MiB;
/// managed store partition ≤32MiB.
const OWNED_WORKER_B_LINE: u64 = 128 * 1024 * 1024;
const OWNED_STORE_PARTITION: u64 = 32 * 1024 * 1024;

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(path) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(md) = e.metadata() {
                total += md.len();
            }
        }
    }
    total
}

#[test]
fn av_owned_821_r1_worker_memory_hard_gate_parent_sampled() {
    // Locate/build the release worker (same discipline as the corpus).
    let ws = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("ws")
        .to_path_buf();
    let build = std::process::Command::new(env!("CARGO"))
        .args([
            "build",
            "--release",
            "-p",
            "auto-av-probe",
            "--bin",
            "av_probe_worker",
        ])
        .current_dir(&ws)
        .output()
        .expect("build");
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let worker = ws.join("target").join("release").join(if cfg!(windows) {
        "av_probe_worker.exe"
    } else {
        "av_probe_worker"
    });

    // A REAL sustained owned load: 8 fresh children over 8 revisions of
    // the frozen source family (record + Str fields + clone + concat),
    // one managed store accumulating all artifacts.
    let root = std::env::temp_dir().join(format!(
        "av821-memgate-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let store = root.join("store");
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    let mut peaks = Vec::new();
    let mut all_ok = true;
    for i in 0..8u32 {
        let src = format!(
            "type Note {{ body Str; n int }}
fn main() int {{
    let a Note = Note {{ body: \"payload-{i}\", n: {i} }}
    var b Note = a.clone()
    let n int = b.body.len() + b.body.len_bytes()
    return b.n * 100 + n
}}
"
        );
        let src_path = src_dir.join(format!("rev{i}.at"));
        std::fs::write(&src_path, src).unwrap();
        let (peak, ok) = sampled_owned_request(&worker, &store, &src_path);
        assert!(ok, "revision {i} runs green");
        peaks.push(peak);
    }
    // HARD GATES (parent-side evidence, worker self-reports never enter):
    let max_peak = *peaks.iter().max().unwrap();
    assert!(
        max_peak <= OWNED_WORKER_B_LINE,
        "worker B-line: parent-sampled peak {max_peak} > {}",
        OWNED_WORKER_B_LINE
    );
    let store_bytes = dir_size(&store);
    assert!(
        store_bytes <= OWNED_STORE_PARTITION,
        "managed store partition: {store_bytes} > {}",
        OWNED_STORE_PARTITION
    );

    // GATE-LOGIC COUNTER-CONTROL: the same evidence against a ZERO
    // threshold must FAIL (a gate that cannot red is no gate).
    assert!(max_peak > 0, "a real peak was measured");
    assert!(store_bytes > 0, "the store holds real artifacts");

    let _ = std::fs::remove_dir_all(&root);
}
