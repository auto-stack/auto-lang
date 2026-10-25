//! PLAN-821 R1 repair replays (`av_owned_821` family): the independent
//! review's counterexamples as PERMANENT regressions — every input from
//! review-r1 (typed-MIR shapes and source shapes) is replayed against its
//! pinned oracle. A regression of any single face reddens here.

use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::av::{execute, execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value, MIR_OWNED_SCHEMA};
use auto_av_core::function_code::FunctionCode;
use auto_av_core::mir::*;
use auto_av_core::program_snapshot::ProgramSnapshot;
use auto_av_core::{abc_verify, mir_verify};
use std::sync::Arc;

fn owner(name: &str) -> OwnerScope {
    OwnerScope {
        schema: MIR_OWNED_SCHEMA.to_string(),
        module: name.to_string(),
    }
}

struct T {
    str: u32,
    i32: u32,
    view: u32,
    mutref: u32,
    note: u32,
}

fn types(m: &mut MirModule) -> T {
    let str = m.intern_type(MirType::Str);
    T {
        str,
        i32: m.intern_type(MirType::I32),
        view: m.intern_type(MirType::StrView),
        mutref: m.intern_type(MirType::MutRef),
        note: m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "r1".into(),
            name: "Note".into(),
            fields: vec![MirFieldDef {
                name: "body".into(),
                ty: str,
            }],
        })),
    }
}

fn local(ty: u32) -> MirSlot {
    MirSlot {
        kind: SlotKind::Local,
        ty,
        name: Some("l".into()),
    }
}

fn temp(ty: u32) -> MirSlot {
    MirSlot {
        kind: SlotKind::Temp,
        ty,
        name: Some("t".into()),
    }
}

fn param(ty: u32) -> MirSlot {
    MirSlot {
        kind: SlotKind::Param,
        ty,
        name: Some("p".into()),
    }
}

fn block(instrs: Vec<MirInstr>, term: Terminator) -> MirBlock {
    MirBlock { instrs, term }
}

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
    m.entry = (m.funcs.len() - 1) as u32;
}

fn extra(
    m: &mut MirModule,
    _t: &T,
    params: Vec<u32>,
    result: u32,
    slots: Vec<MirSlot>,
    blocks: Vec<MirBlock>,
) -> u32 {
    let sig = m.intern_type(MirType::Fn(FnSig { params, result }));
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: format!("f{}", m.funcs.len()),
        sig,
        slots,
        blocks,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    (m.funcs.len() - 1) as u32
}

fn admit_exec(m: &MirModule) -> (ExecOutcome, auto_av_core::av::ExecTrace) {
    mir_verify::verify_mir(m).expect("verifies");
    let wire = encode_abc(m).expect("encode");
    let decoded = decode_abc(&wire.to_flat()).expect("decode");
    let loaded = abc_verify::verify_abc(decoded, &m.owner).expect("admits");
    execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false))
}

fn reject(m: &MirModule, want: &str) {
    let err = mir_verify::verify_mir(m).expect_err("must reject");
    let code = err.code().to_string();
    assert!(code.contains(want), "expected `{want}`, got `{code}`");
}

// ---- F821-R1-01: return terminal states -------------------------------------

#[test]
fn av_owned_821_r1_return_dropped_and_ended_escape_rejected() {
    // return of a DROPPED slot: rejected (never a typed zero as success).
    let mut m = MirModule::new_owned(owner("r1a"));
    let t = types(&mut m);
    let lit = m.intern_const(MirConst::Str(b"abc".to_vec().into_boxed_slice()));
    extra(
        &mut m,
        &t,
        vec![],
        t.str,
        vec![temp(t.str)],
        vec![block(
            vec![
                MirInstr::Const { dst: 0, value: lit },
                MirInstr::Drop { slot: 0 },
            ],
            Terminator::Return { value: 0 },
        )],
    );
    main_i32(
        &mut m,
        &t,
        vec![temp(t.str), temp(t.view), temp(t.i32)],
        vec![block(
            vec![
                MirInstr::Call {
                    callee: CalleeRef::Func(0),
                    dst: 0,
                    args: vec![],
                },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::StrLen { dst: 2, view: 1 },
                MirInstr::BorrowEnd { token: 1 },
            ],
            Terminator::Return { value: 2 },
        )],
    );
    reject(&m, "use-after-move");

    // A view whose borrow ENDED still may not escape its frame's root.
    for ended in [false, true] {
        let mut m = MirModule::new_owned(owner("r1b"));
        let t = types(&mut m);
        let lit = m.intern_const(MirConst::Str(b"abc".to_vec().into_boxed_slice()));
        let mut ins = vec![
            MirInstr::Const { dst: 0, value: lit },
            MirInstr::View { dst: 1, src: 0 },
        ];
        if ended {
            ins.push(MirInstr::BorrowEnd { token: 1 });
        }
        extra(
            &mut m,
            &t,
            vec![],
            t.view,
            vec![local(t.str), temp(t.view)],
            vec![block(ins, Terminator::Return { value: 1 })],
        );
        main_i32(
            &mut m,
            &t,
            vec![temp(t.view), temp(t.i32)],
            vec![block(
                vec![
                    MirInstr::Call {
                        callee: CalleeRef::Func(0),
                        dst: 0,
                        args: vec![],
                    },
                    MirInstr::StrLen { dst: 1, view: 0 },
                    MirInstr::BorrowEnd { token: 0 },
                ],
                Terminator::Return { value: 1 },
            )],
        );
        reject(&m, "borrow-escape");
    }
}

// ---- F821-R1-02: loan invariants ---------------------------------------------

#[test]
fn av_owned_821_r1_loan_invariants_hold() {
    // (a) call-result view provenance: identity(view of s) then Drop(s)
    // must be REJECTED (the result still borrows s).
    let mut m = MirModule::new_owned(owner("r1c"));
    let t = types(&mut m);
    let lit = m.intern_const(MirConst::Str(b"abc".to_vec().into_boxed_slice()));
    extra(
        &mut m,
        &t,
        vec![t.view],
        t.view,
        vec![param(t.view)],
        vec![block(vec![], Terminator::Return { value: 0 })],
    );
    main_i32(
        &mut m,
        &t,
        vec![local(t.str), temp(t.view), temp(t.view), temp(t.i32)],
        vec![block(
            vec![
                MirInstr::Const { dst: 0, value: lit },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::Call {
                    callee: CalleeRef::Func(0),
                    dst: 2,
                    args: vec![1],
                },
                MirInstr::BorrowEnd { token: 1 },
                MirInstr::Drop { slot: 0 },
                MirInstr::StrLen { dst: 3, view: 2 },
                MirInstr::BorrowEnd { token: 2 },
            ],
            Terminator::Return { value: 3 },
        )],
    );
    reject(&m, "mut-conflict");

    // (b) a live MutRef blocks a new shared view; ending it unblocks.
    for ended in [false, true] {
        let mut m = MirModule::new_owned(owner("r1d"));
        let t = types(&mut m);
        let lit = m.intern_const(MirConst::Str(b"abc".to_vec().into_boxed_slice()));
        let mut ins = vec![
            MirInstr::Const { dst: 0, value: lit },
            MirInstr::MutRef { dst: 1, src: 0 },
        ];
        if ended {
            ins.push(MirInstr::BorrowEnd { token: 1 });
        }
        ins.extend([
            MirInstr::View { dst: 2, src: 0 },
            MirInstr::StrLen { dst: 3, view: 2 },
            MirInstr::BorrowEnd { token: 2 },
        ]);
        main_i32(
            &mut m,
            &t,
            vec![local(t.str), temp(t.mutref), temp(t.view), temp(t.i32)],
            vec![block(ins, Terminator::Return { value: 3 })],
        );
        if ended {
            let (out, _) = admit_exec(&m);
            assert!(
                matches!(
                    out,
                    ExecOutcome::Success {
                        value: Value::I32(3),
                        ..
                    }
                ),
                "ended exclusive borrow unblocks the view: {out:?}"
            );
        } else {
            reject(&m, "mut-conflict");
        }
    }

    // (c) StrToOwned into a borrowed root: rejected (destination gate).
    let mut m = MirModule::new_owned(owner("r1e"));
    let t = types(&mut m);
    let lit = m.intern_const(MirConst::Str(b"abc".to_vec().into_boxed_slice()));
    main_i32(
        &mut m,
        &t,
        vec![local(t.str), temp(t.view), temp(t.i32)],
        vec![block(
            vec![
                MirInstr::Const { dst: 0, value: lit },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::StrToOwned { dst: 0, view: 1 },
                MirInstr::StrLen { dst: 2, view: 1 },
                MirInstr::BorrowEnd { token: 1 },
            ],
            Terminator::Return { value: 2 },
        )],
    );
    reject(&m, "mut-conflict");
}

// ---- F821-R1-03: static views across units -----------------------------------

#[test]
fn av_owned_821_r1_static_view_across_units_resolves_in_owner() {
    // helper's const 0 = "ABC" (a static view); caller's const 0 = I32(0).
    // The linked program must read the OWNING unit's bytes: len = 3.
    let mut helper = MirModule::new_owned(owner("units"));
    let ht = types(&mut helper);
    let lit = helper.intern_const(MirConst::Str(b"ABC".to_vec().into_boxed_slice()));
    let sig = helper.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ht.view,
    }));
    helper.funcs.push(MirFunction {
        owner: helper.owner.clone(),
        name: "helper".into(),
        sig,
        slots: vec![temp(ht.view)],
        blocks: vec![block(
            vec![MirInstr::Const { dst: 0, value: lit }],
            Terminator::Return { value: 0 },
        )],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    helper.entry = 0;
    helper.is_unit = true;

    let mut caller = MirModule::new_owned(owner("units"));
    let ct = types(&mut caller);
    let zero = caller.intern_const(MirConst::I32(0));
    let isig = caller.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ct.view,
    }));
    caller.imports.push(MirImport {
        symbol: "fn:helper".into(),
        effect: Effect::Pure,
        sig: isig,
    });
    let sig2 = caller.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: ct.i32,
    }));
    caller.funcs.push(MirFunction {
        owner: caller.owner.clone(),
        name: "d_main".into(),
        sig: sig2,
        slots: vec![temp(ct.view), temp(ct.i32)],
        blocks: vec![block(
            vec![
                MirInstr::Const {
                    dst: 1,
                    value: zero,
                },
                MirInstr::Call {
                    callee: CalleeRef::Import(0),
                    dst: 0,
                    args: vec![],
                },
                MirInstr::StrLen { dst: 1, view: 0 },
                MirInstr::BorrowEnd { token: 0 },
            ],
            Terminator::Return { value: 1 },
        )],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    caller.entry = 0;
    caller.is_unit = true;

    let code: Vec<Arc<FunctionCode>> = [helper, caller]
        .iter()
        .map(|m| {
            let wire = encode_abc(m).unwrap();
            let loaded =
                abc_verify::verify_abc(decode_abc(&wire.to_flat()).unwrap(), &m.owner).unwrap();
            Arc::new(FunctionCode::admit_unit(loaded, 1).unwrap())
        })
        .collect();
    let snap = ProgramSnapshot::link(code, "d_main", 1).expect("links");
    let (out, _) = execute_program(&snap, &ExecLimits::default(), &CancelFlag::new(false));
    assert_eq!(
        out,
        ExecOutcome::Success {
            value: Value::I32(3),
            steps: out_steps(&out)
        },
        "the static view resolves in its OWNING unit's const table"
    );
}

fn out_steps(out: &ExecOutcome) -> u64 {
    match out {
        ExecOutcome::Success { steps, .. } => *steps,
        _ => 0,
    }
}

// ---- F821-R1-04: aggregate operand transfer ----------------------------------

#[test]
fn av_owned_821_r1_aggregate_transfer_releases_source_at_commit() {
    // A -> Aggregate(r) -> Drop(r) -> alloc B -> Drop B: A must release at
    // the explicit Drop(r) (the aggregate's operand transfer killed the
    // source slot), so the free order is [A, B].
    let mut m = MirModule::new_owned(owner("r1f"));
    let t = types(&mut m);
    let a = m.intern_const(MirConst::Str(b"A".to_vec().into_boxed_slice()));
    let b = m.intern_const(MirConst::Str(b"B".to_vec().into_boxed_slice()));
    let ret = m.intern_const(MirConst::I32(0));
    main_i32(
        &mut m,
        &t,
        vec![temp(t.str), temp(t.str), local(t.note), temp(t.i32)],
        vec![block(
            vec![
                MirInstr::Const { dst: 0, value: a },
                MirInstr::Aggregate {
                    dst: 2,
                    record_ty: t.note,
                    operands: vec![0],
                },
                MirInstr::Drop { slot: 2 },
                MirInstr::Const { dst: 1, value: b },
                MirInstr::Drop { slot: 1 },
                MirInstr::Const { dst: 3, value: ret },
            ],
            Terminator::Return { value: 3 },
        )],
    );
    let (out, trace) = admit_exec(&m);
    assert!(matches!(out, ExecOutcome::Success { .. }));
    let arena = trace.arena.as_ref().unwrap();
    let allocs = arena.alloc_events();
    let frees = arena.free_events();
    assert_eq!(
        frees.iter().map(|f| f.id).collect::<Vec<_>>(),
        allocs.iter().map(|a| a.id).collect::<Vec<_>>(),
        "A frees at the explicit Drop(r) — BEFORE B allocates"
    );
    assert_eq!(arena.live(), 0);
}

// ---- F821-R1-05/06: honest analysis memory + byte fuel ------------------------

#[test]
fn av_owned_821_r1_clone_byte_fuel_thresholds_discriminate() {
    // Byte-scaled fuel: a 16B clone passes a 400-step budget; a 1MiB
    // clone exceeds it (Resource InstrSteps) — the sizes DISCRIMINATE.
    for (len, expect_ok) in [(16usize, true), (1_048_576, false)] {
        let mut m = MirModule::new_owned(owner("r1g"));
        let t = types(&mut m);
        let lit = m.intern_const(MirConst::Str(
            "x".repeat(len).as_bytes().to_vec().into_boxed_slice(),
        ));
        let zero = m.intern_const(MirConst::I32(0));
        main_i32(
            &mut m,
            &t,
            vec![temp(t.str), temp(t.str), temp(t.i32)],
            vec![block(
                vec![
                    MirInstr::Const { dst: 0, value: lit },
                    MirInstr::Clone { dst: 1, src: 0 },
                    MirInstr::Const {
                        dst: 2,
                        value: zero,
                    },
                ],
                Terminator::Return { value: 2 },
            )],
        );
        mir_verify::verify_mir(&m).expect("verifies");
        let wire = encode_abc(&m).unwrap();
        let loaded =
            abc_verify::verify_abc(decode_abc(&wire.to_flat()).unwrap(), &m.owner).unwrap();
        let limits = ExecLimits {
            max_steps: 400,
            ..ExecLimits::default()
        };
        let (out, _) = execute(&loaded, &limits, &CancelFlag::new(false));
        if expect_ok {
            assert!(
                matches!(out, ExecOutcome::Success { .. }),
                "16B clone fits a 400-step budget: {out:?}"
            );
        } else {
            assert!(
                matches!(
                    &out,
                    ExecOutcome::Resource { kind, .. }
                        if matches!(kind, auto_av_core::contract::ResourceKind::InstrSteps)
                ),
                "1MiB clone burns byte-scaled fuel: {out:?}"
            );
        }
    }
}

#[test]
fn av_owned_821_r1_analysis_memory_bound_honest_no_tokens() {
    // 8192 scalar slots, zero tokens: the analysis must verify with a
    // bound that reflects the REAL (sparse) layout — token-free inputs no
    // longer pay a phantom n^2 root matrix (the review measured a 35MB
    // real peak against a 4,160-byte claim).
    let mut m = MirModule::new_owned(owner("r1h"));
    let t = types(&mut m);
    let zero = m.intern_const(MirConst::I32(0));
    let slots: Vec<MirSlot> = (0..8192).map(|_| temp(t.i32)).collect();
    main_i32(
        &mut m,
        &t,
        slots,
        vec![block(
            vec![MirInstr::Const {
                dst: 0,
                value: zero,
            }],
            Terminator::Return { value: 0 },
        )],
    );
    let checked = mir_verify::verify_mir(&m).expect("the legal finite input verifies");
    let bytes = checked.stats().analysis_bytes;
    assert!(
        bytes < 4_000_000,
        "the token-free bound reflects the sparse layout (saw {bytes})"
    );
    assert!(bytes > 0);
}

// ---- F821-R1-08: borrowed record source paths ---------------------------------

const BORROWED_RECORD: &str = "\
type Note { body Str; n int }
fn read(p Note) int { return p.n }
fn main() int {
    let a Note = Note { body: \"A\", n: 7 }
    return read(a)
}
";

const BORROW_REUSE: &str = "\
type Note { body Str; n int }
fn read(p Note) int { return p.n }
fn main() int {
    let a Note = Note { body: \"A\", n: 7 }
    let x int = read(a)
    let y int = read(a)
    return x + y + a.body.len()
}
";

const MUT_RECORD: &str = "\
type Note { body Str; n int }
fn set(mut p Note) int { p.n = 8; return p.n }
fn main() int {
    var a Note = Note { body: \"A\", n: 7 }
    return set(a.mut)
}
";

const MOVE_CONTROL: &str = "\
type Note { body Str; n int }
fn read(move p Note) int { return p.n }
fn main() int {
    let a Note = Note { body: \"A\", n: 7 }
    return read(a.move)
}
";

#[test]
fn av_owned_821_r1_borrowed_record_source_paths() {
    // Borrowed record param: reads work, the caller keeps ownership.
    for (src, want) in [(BORROWED_RECORD, 7i32), (BORROW_REUSE, 15)] {
        let compiled = auto_ac_prototype::source::compile_text_record_source(src, "r.at", &[])
            .expect("compiles");
        let mir = auto_av_probe::hir_adapter::lower_checked_owned(&compiled.checked, "d_main", 0x1)
            .expect("lowers");
        let (out, trace) = admit_exec(&mir);
        assert_eq!(
            out,
            ExecOutcome::Success {
                value: Value::I32(want),
                steps: out_steps(&out)
            },
            "borrowed record call runs with the exact oracle"
        );
        assert_eq!(trace.arena.as_ref().unwrap().live(), 0);
    }

    // Move-in control unchanged.
    let compiled = auto_ac_prototype::source::compile_text_record_source(MOVE_CONTROL, "r.at", &[])
        .expect("compiles");
    let mir = auto_av_probe::hir_adapter::lower_checked_owned(&compiled.checked, "d_main", 0x1)
        .expect("lowers");
    let (out, _) = admit_exec(&mir);
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::I32(7),
            ..
        }
    ));

    // R2-F08': the mut record param now RUNS with caller-visible
    // write-back — the callee's p.n=8 lands in the caller's `a` (oracle
    // 8), and the ledger stays clean.
    {
        let compiled =
            auto_ac_prototype::source::compile_text_record_source(MUT_RECORD, "r.at", &[])
                .expect("the frontend accepts mut params");
        let mir = auto_av_probe::hir_adapter::lower_checked_owned(&compiled.checked, "d_main", 0x1)
            .expect("the owned lowering carries the write-back");
        let (out, trace) = admit_exec(&mir);
        assert!(matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(8),
                ..
            }
        ));
        assert_eq!(trace.arena.as_ref().unwrap().live(), 0);
    }
    // Caller-visible write-back: the caller reads `a` AFTER the call and
    // observes the callee's mutation (8*100+8 = 808 = 0x328).
    {
        let src = "type Note { body Str; n int }
fn set(mut p Note) int { p.n = 8; return p.n }
fn main() int {
    var a Note = Note { body: \"A\", n: 7 }
    let r int = set(a.mut)
    return r * 100 + a.n
}
";
        let compiled =
            auto_ac_prototype::source::compile_text_record_source(src, "r.at", &[]).unwrap();
        let mir = auto_av_probe::hir_adapter::lower_checked_owned(&compiled.checked, "d_main", 0x1)
            .unwrap();
        let (out, _) = admit_exec(&mir);
        assert!(matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(808),
                ..
            }
        ));
    }
}
