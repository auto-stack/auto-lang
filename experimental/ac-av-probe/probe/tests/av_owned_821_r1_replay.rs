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
        env: None,
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
        env: None,
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
        env: None,
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
        env: None,
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
        env: None,
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

// ---- R3-F08: the three mut write-back counterexample shapes + the
// per-field consumption invariants (review-r3, permanent) ---------------

/// The R3 source counterexamples: mixed mut/read-only params (the
/// read-only record must NOT consume a wrapper field), double mut (two
/// sequential field extractions from ONE wrapper), and an owned Str
/// result (f0 transfers ownership, no FieldGet view). Exact hand oracles
/// from the review; the ledger must read live=0 after each.
#[test]
fn av_owned_821_r3_mut_writeback_counterexample_shapes() {
    let cases: &[(&str, i32)] = &[
        // mut + read-only record: a.n=8 (written back), x = 8 + b.n(2)
        // = 10 -> 8*100 + 10 = 810.
        (
            "type Note { body Str; n int }
fn set(mut p Note, q Note) int { p.n = 8; return p.n + q.n }
fn main() int {
    var a Note = Note { body: \"A\", n: 7 }
    let b Note = Note { body: \"B\", n: 2 }
    let x int = set(a.mut, b)
    return a.n * 100 + x
}
",
            810,
        ),
        // double mut: a.n=8, b.n=9, x = 17 -> 8*1000 + 9*100 + 17 = 8917.
        (
            "type Note { body Str; n int }
fn set(mut p Note, mut q Note) int { p.n = 8; q.n = 9; return p.n + q.n }
fn main() int {
    var a Note = Note { body: \"A\", n: 7 }
    var b Note = Note { body: \"B\", n: 2 }
    let x int = set(a.mut, b.mut)
    return a.n * 1000 + b.n * 100 + x
}
",
            8917,
        ),
        // mut + owned Str result: a.n=8, x.len()=1 -> 8*100 + 1 = 801.
        (
            "type Note { body Str; n int }
fn set(mut p Note) Str { p.n = 8; return \"R\" }
fn main() int {
    var a Note = Note { body: \"A\", n: 7 }
    let x Str = set(a.mut)
    return a.n * 100 + x.len()
}
",
            801,
        ),
    ];
    for (src, want) in cases {
        let compiled =
            auto_ac_prototype::source::compile_text_record_source(src, "r3.at", &[]).unwrap();
        let mir = auto_av_probe::hir_adapter::lower_checked_owned(&compiled.checked, "d_main", 0x1)
            .unwrap();
        let (out, trace) = admit_exec(&mir);
        assert!(
            matches!(out, ExecOutcome::Success { value: Value::I32(v), .. } if v == *want),
            "source shape `{src}` must produce {want}, got {out:?}"
        );
        assert_eq!(trace.arena.as_ref().unwrap().live(), 0);
    }
}

/// R3-F08 per-field consumption, analysis face: a REPEATED field-move of
/// the same path is use-after-move; a WHOLE-root use of a
/// partially-consumed root is use-after-move; DISTINCT paths stay legal.
#[test]
fn av_owned_821_r3_field_move_per_field_consumption_invariants() {
    // slots: 0 s1(Str) 1 s2(Str) 2 n1(note) 3 n2(note) 4 wrap(t_MutRet)
    //        5 o1(note) 6 o2(note) 7 r(i32) 8 sink(note)
    let build = |same_path: bool, whole_read_after: bool| {
        let mut m = MirModule::new_owned(owner("r3f"));
        let t = types(&mut m);
        let wrap = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "r3f".into(),
            name: "t_MutRet_x".into(),
            fields: vec![
                MirFieldDef {
                    name: "f0".into(),
                    ty: t.i32,
                },
                MirFieldDef {
                    name: "wb1".into(),
                    ty: t.note,
                },
                MirFieldDef {
                    name: "wb2".into(),
                    ty: t.note,
                },
            ],
        }));
        let s1 = m.intern_const(MirConst::Str(Box::from(&b"A"[..])));
        let s2 = m.intern_const(MirConst::Str(Box::from(&b"B"[..])));
        let z = m.intern_const(MirConst::I32(0));
        let slots = vec![
            temp(t.str),
            temp(t.str),
            temp(t.note),
            temp(t.note),
            temp(wrap),
            temp(t.note),
            temp(t.note),
            temp(t.i32),
            temp(wrap),
        ];
        let mut ins = vec![
            MirInstr::Const { dst: 0, value: s1 },
            MirInstr::Const { dst: 1, value: s2 },
            MirInstr::Const { dst: 7, value: z },
            MirInstr::Aggregate {
                dst: 2,
                record_ty: t.note,
                operands: vec![0],
            },
            MirInstr::Aggregate {
                dst: 3,
                record_ty: t.note,
                operands: vec![1],
            },
            MirInstr::Aggregate {
                dst: 4,
                record_ty: wrap,
                operands: vec![7, 2, 3],
            },
            MirInstr::FieldMove {
                dst: 5,
                root: 4,
                path: vec![1],
            },
            MirInstr::FieldMove {
                dst: 6,
                root: 4,
                path: vec![if same_path { 1 } else { 2 }],
            },
        ];
        if whole_read_after {
            ins.push(MirInstr::Move { dst: 8, src: 4 });
        }
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: t.i32,
        }));
        m.funcs.push(MirFunction {
            owner: m.owner.clone(),
            name: "d_main".into(),
            sig,
            slots,
            blocks: vec![MirBlock {
                instrs: ins,
                term: Terminator::Return { value: 7 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m
    };
    // Distinct paths: legal (the wrapper is consumed field by field).
    assert!(
        mir_verify::verify_mir(&build(false, false)).is_ok(),
        "two DISTINCT field extractions from one wrapper are legal"
    );
    // Same path twice: use-after-move.
    let err = mir_verify::verify_mir(&build(true, false)).unwrap_err();
    assert!(
        matches!(&err, mir_verify::MirVerifyError::Invalid { code, .. } if code.contains("use-after-move")),
        "a repeated field extraction is a use-after-move: {err:?}"
    );
    // Whole-root use after partial consumption: use-after-move.
    let err = mir_verify::verify_mir(&build(false, true)).unwrap_err();
    assert!(
        matches!(&err, mir_verify::MirVerifyError::Invalid { code, .. } if code.contains("use-after-move")),
        "a whole-root use of a partially-consumed root is a use-after-move: {err:?}"
    );
}

// ---- R4-F08/R4-F09: re-initialization, path-overlap, heap-payload
// counterexamples (review-r4, permanent) ----------------------------------

enum FieldSeq {
    ReinitRoot,
    ParentThenChild,
    ChildThenParent,
    LegalSiblings,
}

/// R4-F08: a mut loop — the callee's write-back lands across LOOP
/// iterations. The wrapper temp is re-defined per iteration (each Call),
/// so the per-field consumption state must reset with the new owner;
/// oracle: two increments from 7 = 9.
#[test]
fn av_owned_821_r4_mut_loop_writeback() {
    let src = "type Note { body Str; n int }\nfn set(mut p Note) int { p.n = p.n + 1; return p.n }\nfn main() int { var a Note = Note { body: \"A\", n: 7 }; var i int = 0; loop { if i >= 2 { break }; let n int = set(a.mut); i = i + 1 }; return a.n }\n";
    let compiled =
        auto_ac_prototype::source::compile_text_record_source(src, "r4.at", &[]).unwrap();
    let mir =
        auto_av_probe::hir_adapter::lower_checked_owned(&compiled.checked, "d_main", 0x1).unwrap();
    let (out, trace) = admit_exec(&mir);
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(9),
                ..
            }
        ),
        "two loop iterations of set(a.mut) write back 7 -> 9, got {out:?}"
    );
    assert_eq!(trace.arena.as_ref().unwrap().live(), 0);
}

/// R4-F08 per-field consumption: re-initialization resets the field
/// state; ancestor/descendant overlaps are use-after-move at ADMISSION
/// (both directions); disjoint/other-root extractions stay legal.
#[test]
fn av_owned_821_r4_field_state_reinit_and_overlap() {
    let build = |seq: FieldSeq| {
        let mut m = MirModule::new_owned(owner("r4f"));
        let t = types(&mut m);
        let note = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "r4f".into(),
            name: "Note".into(),
            fields: vec![
                MirFieldDef {
                    name: "body".into(),
                    ty: t.str,
                },
                MirFieldDef {
                    name: "n".into(),
                    ty: t.i32,
                },
            ],
        }));
        let outer = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "r4f".into(),
            name: "Outer".into(),
            fields: vec![MirFieldDef {
                name: "inner".into(),
                ty: note,
            }],
        }));
        let a = m.intern_const(MirConst::Str(Box::from(&b"A"[..])));
        let z = m.intern_const(MirConst::I32(0));
        // 0 s1(Str) 1 note 2 outer 3 o1(note) 4 v(Str) 5 r(i32)
        let slots = vec![
            temp(t.str),
            temp(note),
            temp(outer),
            temp(note),
            temp(t.str),
            temp(t.i32),
        ];
        let mut ins = vec![
            MirInstr::Const { dst: 0, value: a },
            MirInstr::Const { dst: 5, value: z },
            MirInstr::Aggregate {
                dst: 1,
                record_ty: note,
                operands: vec![0, 5],
            },
        ];
        // The OUTER aggregate MOVE-CONSUMES the note (slot 1) — only the
        // overlap faces need it as their root.
        if !matches!(seq, FieldSeq::ReinitRoot) {
            ins.push(MirInstr::Aggregate {
                dst: 2,
                record_ty: outer,
                operands: vec![1],
            });
        }
        match seq {
            FieldSeq::ReinitRoot => {
                // Consume [0] out of the note ROOT, drop it, then REBUILD
                // the whole root (a fresh complete owner) and consume [0]
                // again — legal (R4: the per-field state dies with the old
                // owner).
                ins.extend(vec![
                    MirInstr::FieldMove {
                        dst: 4,
                        root: 1,
                        path: vec![0],
                    },
                    MirInstr::Drop { slot: 4 },
                    MirInstr::Const { dst: 0, value: a },
                    MirInstr::Aggregate {
                        dst: 1,
                        record_ty: note,
                        operands: vec![0, 5],
                    },
                    MirInstr::FieldMove {
                        dst: 4,
                        root: 1,
                        path: vec![0],
                    },
                    MirInstr::Drop { slot: 4 },
                ]);
            }
            FieldSeq::ParentThenChild | FieldSeq::ChildThenParent => {
                // dst 3 is note-typed (the parent field), dst 4 is Str
                // (the leaf) — the destinations follow the FIELD TYPES.
                let moves: Vec<MirInstr> = if matches!(seq, FieldSeq::ParentThenChild) {
                    vec![
                        MirInstr::FieldMove {
                            dst: 3,
                            root: 2,
                            path: vec![0],
                        },
                        MirInstr::FieldMove {
                            dst: 4,
                            root: 2,
                            path: vec![0, 0],
                        },
                    ]
                } else {
                    vec![
                        MirInstr::FieldMove {
                            dst: 4,
                            root: 2,
                            path: vec![0, 0],
                        },
                        MirInstr::FieldMove {
                            dst: 3,
                            root: 2,
                            path: vec![0],
                        },
                    ]
                };
                ins.extend(moves);
            }
            FieldSeq::LegalSiblings => {
                // Non-overlapping consumption on DIFFERENT roots: extract
                // outer's [0] (the note moved into it), then a REBUILT
                // note root's [0] — never overlapping.
                ins.extend(vec![
                    MirInstr::FieldMove {
                        dst: 3,
                        root: 2,
                        path: vec![0],
                    },
                    MirInstr::Const { dst: 0, value: a },
                    MirInstr::Aggregate {
                        dst: 1,
                        record_ty: note,
                        operands: vec![0, 5],
                    },
                    MirInstr::FieldMove {
                        dst: 4,
                        root: 1,
                        path: vec![0],
                    },
                    MirInstr::Drop { slot: 4 },
                ]);
            }
        }
        ins.push(MirInstr::Const { dst: 5, value: z });
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: t.i32,
        }));
        m.funcs.push(MirFunction {
            owner: m.owner.clone(),
            name: "d_main".into(),
            sig,
            slots,
            blocks: vec![MirBlock {
                instrs: ins,
                term: Terminator::Return { value: 5 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m
    };
    // Re-initialization of a consumed root: the SECOND extraction is legal.
    let m = build(FieldSeq::ReinitRoot);
    mir_verify::verify_mir(&m).expect("a fresh complete owner resets the field state");
    let (out, _) = admit_exec(&m);
    assert!(matches!(out, ExecOutcome::Success { .. }), "{out:?}");
    // Ancestor then descendant: the subtree went with the ancestor.
    let err = mir_verify::verify_mir(&build(FieldSeq::ParentThenChild)).unwrap_err();
    assert!(
        matches!(&err, mir_verify::MirVerifyError::Invalid { code, .. } if code.contains("use-after-move")),
        "parent-then-child is a use-after-move at ADMISSION: {err:?}"
    );
    // Descendant then ancestor: the ancestor is no longer complete.
    let err = mir_verify::verify_mir(&build(FieldSeq::ChildThenParent)).unwrap_err();
    assert!(
        matches!(&err, mir_verify::MirVerifyError::Invalid { code, .. } if code.contains("use-after-move")),
        "child-then-parent is a use-after-move at ADMISSION: {err:?}"
    );
    // Disjoint/other-root extractions stay legal.
    mir_verify::verify_mir(&build(FieldSeq::LegalSiblings))
        .expect("non-overlapping consumption stays legal");
}

/// R4-F09: the heap estimate must cover the payloads a RecordView OWNS
/// (owner/name/field-name strings) — with 3000 ~1000B field names the
/// wire fits the frozen 4MiB input budget while flat+MIR+decoded together
/// exceed the 8MiB hot partition; the estimate must not call that green.
#[test]
fn av_owned_821_r4_recordview_heap_payload_counted() {
    let mut m = MirModule::new_owned(owner("r4m"));
    let i = m.intern_type(MirType::I32);
    let fields: Vec<_> = (0..3000)
        .map(|n| MirFieldDef {
            name: format!("f{n}_{}", "x".repeat(1000)),
            ty: i,
        })
        .collect();
    let name_payload: usize = fields.iter().map(|f| f.name.len()).sum();
    m.intern_type(MirType::RecordView(RecordTypeRef {
        owner_module: "r4m".into(),
        name: "BigView".into(),
        fields,
    }));
    let z = m.intern_const(MirConst::I32(0));
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: i,
    }));
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Temp,
            ty: i,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: z }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    // Admission budgets (16MiB decode/verify) still pass...
    let wire = encode_abc(&m).expect("encode");
    let flat = wire.to_flat();
    assert!(
        flat.len() < 4 * 1024 * 1024,
        "the wire fits the frozen input budget"
    );
    let loaded =
        abc_verify::verify_abc_bytes(&flat, &m.owner).expect("decode + deep admission pass");
    // ...but the working-set estimate must cover the OWNED payloads: the
    // same lower bound the review computed (flat + two independent copies
    // of the field-name strings) and, with it, the hot line reddens.
    let hot = flat.len() + m.heap_estimate() + loaded.module().heap_estimate();
    let lower = flat.len() + 2 * name_payload;
    assert!(
        hot >= lower,
        "the estimate covers the simultaneously-live payloads: {hot} < {lower}"
    );
    assert!(
        hot > 8 * 1024 * 1024,
        "an over-hot-budget input must not estimate green (hot={hot})"
    );
}
