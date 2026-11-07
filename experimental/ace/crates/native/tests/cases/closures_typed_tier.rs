//! PLAN-924 typed closures tiers: hand-built `av.closures/1` modules
//! through the REAL native closures pipeline (shared verify → unit split →
//! native admit → CLIF/COFF → PE link/run), covering the typed-only
//! surfaces source syntax cannot express (mixed Move+Clone capture
//! environments, owned-record captures — V-02) and the native
//! limited-domain NAMED rejections at admission (container captures,
//! direct closure-body calls — V-11). The shared/AV contracts keep their
//! own wider support; these rejections are the native family's own face.

#[path = "../common/mod.rs"]
mod common;

use ace_av_core::contract::MIR_CLOSURES_SCHEMA;
use ace_av_core::mir::*;
use ace_native::collections_lower::WrapperKnobs;

fn closures_owner(tag: &str) -> OwnerScope {
    OwnerScope {
        schema: MIR_CLOSURES_SCHEMA.into(),
        module: format!("closures924-{tag}"),
    }
}

fn slot(ty: TypeIdx, kind: SlotKind) -> MirSlot {
    MirSlot {
        ty,
        kind,
        name: None,
    }
}

fn fn_sig(params: Vec<TypeIdx>, result: TypeIdx) -> FnSig {
    FnSig { params, result }
}

fn run_closures_module(
    m: &MirModule,
    tag: &str,
    knobs: &WrapperKnobs,
) -> ace_native::link_driver::PeRun {
    assert!(
        common::tools_available(),
        "native tools are required; no skip pass"
    );
    ace_av_core::verify_mir(m).unwrap();
    let units = ace_av::cache::split_units(m).unwrap();
    let mut objects = Vec::new();
    for unit in &units {
        let object = ace_native::closures_lower::emit_unit_coff(unit)
            .unwrap_or_else(|e| panic!("{tag}/{}: emit {e}", unit.funcs[0].name));
        objects.push(ace_native::link_driver::ObjectInput {
            name: unit.funcs[0].name.clone(),
            bytes: object.bytes,
        });
    }
    let entry = &m.funcs[m.entry as usize];
    let symbol = ace_native::symbols::function_symbol(&m.owner, &entry.name);
    let wrapper = ace_native::collections_lower::emit_wrapper_coff(&symbol, knobs).unwrap();
    // Scratch home stays OUTSIDE any repo checkout (2026-10-31 ruling).
    let work = std::env::temp_dir().join(format!(
        "plan924-typed-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&work).unwrap();
    let run = ace_native::link_driver::link_and_run(
        &objects,
        &wrapper,
        "ac_start",
        &work.join("pe"),
        common::collections::libraries(),
        ace_native::closures_layout::ABI,
    )
    .expect("real native closures PE");
    let _ = std::fs::remove_dir_all(&work);
    run
}

fn assert_clean(run: &ace_native::link_driver::PeRun) {
    let text = String::from_utf8_lossy(&run.stderr);
    for field in [
        "MN-COLL live=0",
        "MN-COLL bytes=0",
        "MN-COLL invalid=0",
        "MN-COLL arena_live=0",
        "MN-COLL arena_invalid=0",
    ] {
        assert!(
            text.contains(&format!("{field}\n")),
            "missing {field}: {text}"
        );
    }
}

// ---- V-02: the typed-only mixed Move+Clone capture environment ----------

/// `make(move a Str, t Str)` builds a closure capturing `a` by MOVE and
/// `t` by typed-only deep CLONE; the body returns a.len() + t.len() + n.
/// main passes ("hello", "hi!") and n = 34 → 5 + 3 + 34 = 42. The move
/// source dies at the creator's CreateClosure; the clone source keeps its
/// own destructor to the end (the emitter only kills MOVE sources).
#[test]
fn typed_mixed_move_clone_capture_returns_42() {
    let owner = closures_owner("mixed");
    let mut m = MirModule::new_closures(owner.clone());
    m.types = vec![
        MirType::Str,                         // 0
        MirType::I32,                         // 1
        MirType::Closure(fn_sig(vec![1], 1)), // 2
        MirType::Fn(fn_sig(vec![0, 0], 2)),   // 3 make
        MirType::Fn(fn_sig(vec![], 1)),       // 4 main
        MirType::Fn(fn_sig(vec![1], 1)),      // 5 body
        MirType::StrView,                     // 6
    ];
    m.consts = vec![
        MirConst::Str(b"hello".to_vec().into()), // 0
        MirConst::Str(b"hi!".to_vec().into()),   // 1
        MirConst::I32(34),                       // 2
    ];
    // d_make: 0 a(Str) 1 t(Str) 2 closure
    m.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d_make".into(),
        sig: 3,
        slots: vec![
            slot(0, SlotKind::Param),
            slot(0, SlotKind::Param),
            slot(2, SlotKind::Temp),
        ],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::CreateClosure {
                dst: 2,
                callee: CalleeRef::Func(2),
                captures: vec![
                    CaptureOperand {
                        kind: CaptureKind::Move,
                        src: 0,
                    },
                    CaptureOperand {
                        kind: CaptureKind::Clone,
                        src: 1,
                    },
                ],
            }],
            term: Terminator::Return { value: 2 },
        }],
    });
    m.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d_main".into(),
        sig: 4,
        slots: vec![
            slot(0, SlotKind::Local), // 0 a: Str
            slot(2, SlotKind::Local), // 1 f: Closure
            slot(0, SlotKind::Temp),  // 2 t1: Str (a moved out)
            slot(0, SlotKind::Temp),  // 3 t2: Str (hi!)
            slot(2, SlotKind::Temp),  // 4 tf: Closure
            slot(1, SlotKind::Temp),  // 5 n: I32
            slot(1, SlotKind::Temp),  // 6 out: I32
        ],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 2, value: 0 },
                MirInstr::Move { dst: 0, src: 2 },
                MirInstr::Const { dst: 3, value: 1 },
                MirInstr::Move { dst: 2, src: 0 },
                MirInstr::Call {
                    dst: 4,
                    callee: CalleeRef::Func(0),
                    args: vec![2, 3],
                },
                MirInstr::Move { dst: 1, src: 4 },
                MirInstr::Const { dst: 5, value: 2 },
                MirInstr::CallClosure {
                    dst: 6,
                    closure: 1,
                    args: vec![5],
                },
            ],
            term: Terminator::Return { value: 6 },
        }],
    });
    // body: 0 n(i32) 1 a(Str) 2 t(Str) 3/4 views 5..8 scalars
    m.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d___closure_make_0".into(),
        sig: 5,
        slots: vec![
            slot(1, SlotKind::Param), // 0 n
            slot(0, SlotKind::Local), // 1 a
            slot(0, SlotKind::Local), // 2 t
            slot(6, SlotKind::Temp),  // 3 view a
            slot(6, SlotKind::Temp),  // 4 view t
            slot(1, SlotKind::Temp),  // 5 len a
            slot(1, SlotKind::Temp),  // 6 len t
            slot(1, SlotKind::Temp),  // 7 sum lens
            slot(1, SlotKind::Temp),  // 8 + n
        ],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: Some(vec![0, 0]),
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::EnvGet { dst: 1, field: 0 },
                MirInstr::EnvGet { dst: 2, field: 1 },
                MirInstr::View { dst: 3, src: 1 },
                MirInstr::StrLen { dst: 5, view: 3 },
                MirInstr::BorrowEnd { token: 3 },
                MirInstr::View { dst: 4, src: 2 },
                MirInstr::StrLen { dst: 6, view: 4 },
                MirInstr::BorrowEnd { token: 4 },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 7,
                    lhs: 5,
                    rhs: 6,
                },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 8,
                    lhs: 7,
                    rhs: 0,
                },
            ],
            term: Terminator::Return { value: 8 },
        }],
    });
    m.entry = 1;
    let run = run_closures_module(&m, "mixed", &Default::default());
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_clean(&run);
}

// ---- V-02: the owned-record capture -------------------------------------

/// `make(move r {s: Str, n: i32})` captures a non-recursive owned record
/// by move; the body returns s.len() + n + param. main builds
/// {s: "abc", n: 41} and calls with param −2 → 3 + 41 − 2 = 42.
#[test]
fn typed_owned_record_capture_returns_42() {
    let owner = closures_owner("record");
    let record = MirType::Record(RecordTypeRef {
        owner_module: format!("{MIR_CLOSURES_SCHEMA}/rec"),
        name: "Cell924".into(),
        fields: vec![
            MirFieldDef {
                name: "s".into(),
                ty: 0,
            },
            MirFieldDef {
                name: "n".into(),
                ty: 1,
            },
        ],
    });
    let mut m = MirModule::new_closures(owner.clone());
    m.types = vec![
        MirType::Str,                         // 0
        MirType::I32,                         // 1
        record.clone(),                       // 2
        MirType::Closure(fn_sig(vec![1], 1)), // 3
        MirType::Fn(fn_sig(vec![2], 3)),      // 4 make
        MirType::Fn(fn_sig(vec![], 1)),       // 5 main
        MirType::Fn(fn_sig(vec![1], 1)),      // 6 body
        MirType::StrView,                     // 7
    ];
    m.consts = vec![
        MirConst::Str(b"abc".to_vec().into()), // 0
        MirConst::I32(41),                     // 1
        MirConst::I32(-2),                     // 2
    ];
    // d_make: 0 r(Record) 1 closure
    m.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d_make".into(),
        sig: 4,
        slots: vec![slot(2, SlotKind::Param), slot(3, SlotKind::Temp)],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::CreateClosure {
                dst: 1,
                callee: CalleeRef::Func(2),
                captures: vec![CaptureOperand {
                    kind: CaptureKind::Move,
                    src: 0,
                }],
            }],
            term: Terminator::Return { value: 1 },
        }],
    });
    m.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d_main".into(),
        sig: 5,
        slots: vec![
            slot(0, SlotKind::Local), // 0 s: Str
            slot(2, SlotKind::Local), // 1 rec: Record
            slot(0, SlotKind::Temp),  // 2 lit: Str
            slot(0, SlotKind::Temp),  // 3 s out: Str
            slot(2, SlotKind::Temp),  // 4 rec out: Record
            slot(3, SlotKind::Temp),  // 5 tf: Closure
            slot(3, SlotKind::Local), // 6 f: Closure
            slot(1, SlotKind::Temp),  // 7 out: I32
            slot(1, SlotKind::Temp),  // 8 n41: I32
            slot(1, SlotKind::Temp),  // 9 param: I32
        ],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 2, value: 0 },
                MirInstr::Move { dst: 0, src: 2 },
                MirInstr::Const { dst: 8, value: 1 },
                MirInstr::Move { dst: 3, src: 0 },
                MirInstr::Aggregate {
                    dst: 1,
                    record_ty: 2,
                    operands: vec![3, 8],
                },
                MirInstr::Move { dst: 4, src: 1 },
                MirInstr::Call {
                    dst: 5,
                    callee: CalleeRef::Func(0),
                    args: vec![4],
                },
                MirInstr::Move { dst: 6, src: 5 },
                MirInstr::Const { dst: 9, value: 2 },
                MirInstr::CallClosure {
                    dst: 7,
                    closure: 6,
                    args: vec![9],
                },
            ],
            term: Terminator::Return { value: 7 },
        }],
    });
    // body: 0 n 1 r 2 s 3 sv 4 rn 5 slen 6 sum 7 +n
    m.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d___closure_make_0".into(),
        sig: 6,
        slots: vec![
            slot(1, SlotKind::Param), // 0 n
            slot(2, SlotKind::Local), // 1 r
            slot(0, SlotKind::Local), // 2 s
            slot(7, SlotKind::Temp),  // 3 view s
            slot(1, SlotKind::Temp),  // 4 rn
            slot(1, SlotKind::Temp),  // 5 s.len
            slot(1, SlotKind::Temp),  // 6 sum
            slot(1, SlotKind::Temp),  // 7 + n
        ],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: Some(vec![2]),
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::EnvGet { dst: 1, field: 0 },
                MirInstr::FieldGet {
                    dst: 2,
                    base: 1,
                    path: vec![0],
                },
                MirInstr::FieldGet {
                    dst: 4,
                    base: 1,
                    path: vec![1],
                },
                MirInstr::View { dst: 3, src: 2 },
                MirInstr::StrLen { dst: 5, view: 3 },
                MirInstr::BorrowEnd { token: 3 },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 6,
                    lhs: 5,
                    rhs: 4,
                },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 7,
                    lhs: 6,
                    rhs: 0,
                },
            ],
            term: Terminator::Return { value: 7 },
        }],
    });
    m.entry = 1;
    let run = run_closures_module(&m, "record", &Default::default());
    assert_eq!(
        run.exit_code,
        Some(42),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_clean(&run);
}

// ---- V-11: the native limited domain's NAMED admission rejections -------

/// A whole module whose factory moves a `List<int>` into the closure env.
/// Shared verify accepts the move; the NATIVE admission must refuse with
/// the named container face (the same rejection the K06 source ride gets;
/// here the typed unit pins it without the source front).
#[test]
fn typed_container_capture_is_a_named_native_rejection() {
    let owner = closures_owner("listcap");
    let mut m = MirModule::new_closures(owner.clone());
    m.types = vec![
        MirType::I32,                           // 0
        MirType::List(ListTypeRef { elem: 0 }), // 1
        MirType::Closure(fn_sig(vec![0], 0)),   // 2
        MirType::Fn(fn_sig(vec![1], 2)),        // 3 make
        MirType::Fn(fn_sig(vec![], 0)),         // 4 main
        MirType::Fn(fn_sig(vec![0], 0)),        // 5 body
    ];
    m.consts = vec![MirConst::I32(7)];
    // d_make: 0 l(List) 1 closure
    m.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d_make".into(),
        sig: 3,
        slots: vec![slot(1, SlotKind::Param), slot(2, SlotKind::Temp)],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::CreateClosure {
                dst: 1,
                callee: CalleeRef::Func(1),
                captures: vec![CaptureOperand {
                    kind: CaptureKind::Move,
                    src: 0,
                }],
            }],
            term: Terminator::Return { value: 1 },
        }],
    });
    // The body never reads its env (the rejection is at ADMISSION, before
    // codegen): it just returns its parameter.
    m.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d___closure_make_0".into(),
        sig: 5,
        slots: vec![slot(0, SlotKind::Param)],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: Some(vec![1]),
        blocks: vec![MirBlock {
            instrs: vec![],
            term: Terminator::Return { value: 0 },
        }],
    });
    m.funcs.push(MirFunction {
        owner: owner.clone(),
        name: "d_main".into(),
        sig: 4,
        slots: vec![
            slot(1, SlotKind::Local), // 0 l: List
            slot(1, SlotKind::Temp),  // 1 l out: List
            slot(2, SlotKind::Temp),  // 2 f: Closure
            slot(0, SlotKind::Temp),  // 3 param: I32
            slot(0, SlotKind::Temp),  // 4 out: I32
        ],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::ListNew {
                    dst: 0,
                    list_ty: 1,
                    cap: 1,
                },
                MirInstr::Move { dst: 1, src: 0 },
                MirInstr::Call {
                    dst: 2,
                    callee: CalleeRef::Func(0),
                    args: vec![1],
                },
                MirInstr::Const { dst: 3, value: 0 },
                MirInstr::CallClosure {
                    dst: 4,
                    closure: 2,
                    args: vec![3],
                },
            ],
            term: Terminator::Return { value: 4 },
        }],
    });
    m.entry = 2;
    ace_av_core::verify_mir(&m).expect("shared verify admits the container move");
    let units = ace_av::cache::split_units(&m).unwrap();
    let mut refused = None;
    for unit in &units {
        if let Err(e) = ace_native::closures_lower::admit_layouts(unit) {
            refused = Some(e);
        }
    }
    let error = refused.expect("the container capture must be refused natively");
    // The refusal lands on the named TYPE face (the same face the K06
    // source ride meets): the container itself is outside the native
    // closures domain before any capture reasoning runs.
    assert_eq!(error.code(), "mn.unsupported.type");
    assert!(
        error.message().contains("List"),
        "the rejection names the container: {}",
        error.message()
    );
}

/// A unit that CALLS a closure-body import directly (the import carries an
/// env declaration) — the native admission refuses: a body runs through
/// its closure value, never as an ordinary call.
#[test]
fn typed_direct_closure_body_call_is_a_named_native_rejection() {
    let owner = closures_owner("directcall");
    let mut unit = MirModule::closures_unit(owner.clone());
    unit.types = vec![
        MirType::I32,                    // 0
        MirType::Str,                    // 1
        MirType::Fn(fn_sig(vec![0], 0)), // 2 body sig
        MirType::Fn(fn_sig(vec![], 0)),  // 3 this unit's sig
    ];
    unit.consts = vec![MirConst::I32(0)];
    unit.imports = vec![MirImport {
        symbol: "fn:d___closure_make_0".into(),
        effect: Effect::Pure,
        sig: 2,
        env: Some(vec![1]),
    }];
    unit.funcs.push(MirFunction {
        owner,
        name: "d_main".into(),
        sig: 3,
        slots: vec![slot(0, SlotKind::Local), slot(0, SlotKind::Temp)],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 0, value: 0 },
                MirInstr::Copy { dst: 1, src: 0 },
                MirInstr::Call {
                    dst: 1,
                    callee: CalleeRef::Import(0),
                    args: vec![1],
                },
            ],
            term: Terminator::Return { value: 1 },
        }],
    });
    ace_av_core::verify_mir(&unit).expect("shared verify admits the direct call shape");
    let error = ace_native::closures_lower::emit_unit_coff(&unit)
        .expect_err("a direct closure-body call must be refused natively");
    assert_eq!(error.code(), "mn.unsupported.call");
    assert!(
        error.message().contains("closure value"),
        "the rejection names the rule: {}",
        error.message()
    );
}
