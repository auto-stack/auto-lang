//! PLAN-824 T-03 semantics tests: the FULL `av.dynamic/1` support surface
//! over the REAL gates (verify → encode → abc-admit → execute) — the §5
//! ownership/identity/bit-fidelity semantics plus the V-11/V-12 negative
//! controls and the V-14 same-source integration axis. Mirrors the
//! av_collections_822_semantics binary shape. Prefix `av_dynamic_824`
//! (contract-decision §9; V-02..V-05/V-11/V-12/V-14 axes).
//!
//! Scope note (plan §6): V-02..V-05 semantic axes run HERE at the profile
//! level; the budget/fault axes of V-06/V-07 are the resources binary
//! (T-04) and the store-envelope tamper axes of V-08 land in the cache
//! binary (T-05). Module-level wire negatives stay in the parallel/824
//! harness (same-source module axis).

use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, execute_entry, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, TrapKind, Value};
use auto_av_core::dynamic::{DynType, DynValue, TraversalOptions};
use auto_av_core::mir::{
    BinOp, FnSig, ListTypeRef, MirBlock, MirConst, MirFieldDef, MirFunction, MirInstr, MirModule,
    MirSlot, MirType, OwnerScope, RecordTypeRef, SlotKind, Terminator, VariantArm, VariantTypeRef,
};
use auto_av_core::mir_verify::{verify_mir, MirVerifyError};
use auto_av_core::portable::{
    freeze, unfreeze, PortableError, RecordBinding, RestoreContext, VariantBinding,
};
use std::rc::Rc;

/// The (name, current-type) field list of the shared Note vocabulary:
/// position 0 is the Str field, position 1 the I32 one (keeps the
/// wrong-NAME controls differing from the wire only in names).
fn note_fields_typed(fields: &[&'static str]) -> Vec<(&'static str, Rc<DynType>)> {
    fields
        .iter()
        .enumerate()
        .map(|(i, n)| {
            (
                *n,
                Rc::new(if i == 0 { DynType::Str } else { DynType::I32 }),
            )
        })
        .collect()
}

/// Owned decode-domain bindings (RestoreContext borrows slices).
struct Ctx {
    recs: Vec<RecordBinding<'static>>,
    vars: Vec<VariantBinding<'static>>,
}

impl Ctx {
    fn empty() -> Ctx {
        Ctx {
            recs: Vec::new(),
            vars: Vec::new(),
        }
    }
    fn note(ty_index: u32) -> Ctx {
        Ctx {
            recs: vec![RecordBinding {
                owner: "dynamic824",
                name: "Note",
                fields: note_fields_typed(&["text", "n"]),
                ty_index,
            }],
            vars: Vec::new(),
        }
    }
    fn note_fields(ty_index: u32, fields: Vec<&'static str>) -> Ctx {
        Ctx {
            recs: vec![RecordBinding {
                owner: "dynamic824",
                name: "Note",
                fields: note_fields_typed(&fields),
                ty_index,
            }],
            vars: Vec::new(),
        }
    }
    fn other_named(ty_index: u32) -> Ctx {
        Ctx {
            recs: vec![RecordBinding {
                owner: "dynamic824",
                name: "Other",
                fields: note_fields_typed(&["text", "n"]),
                ty_index,
            }],
            vars: Vec::new(),
        }
    }
    fn get(&self) -> RestoreContext<'_> {
        RestoreContext::new(&self.recs, &self.vars)
    }
}

fn owner_scope() -> OwnerScope {
    OwnerScope {
        schema: auto_av_core::contract::MIR_DYNAMIC_SCHEMA.to_string(),
        module: "dynamic824".to_string(),
    }
}

fn field(name: &str, ty: u32) -> MirFieldDef {
    MirFieldDef {
        name: name.to_string(),
        ty,
    }
}

/// Fixture builder over the shared dynamic vocabulary:
/// `[0]=I32 [1]=Bool [2]=Str [3]=Note{text,n} [4]=List<Note> [5]=Opt{none,
/// some(i32)} [6]=Dyn [7]=Alias{text,n} (same-width other name) [8]=F64
/// [9]=I64 [10]=U64 [11]=F32 [12]=Char [13]=List<i32> [14]=Fn
/// [15]=Place<i32>`. (V-12 container-of-category types are interned
/// INSIDE their rejection tests — they must not poison the shared table.)
struct Fx {
    m: MirModule,
    slots: Vec<MirSlot>,
    instrs: Vec<MirInstr>,
    ret: u32,
    params: Vec<u32>,
}

impl Fx {
    fn new() -> Fx {
        let mut m = MirModule::new_dynamic(owner_scope());
        m.types = vec![
            MirType::I32,
            MirType::Bool,
            MirType::Str,
            MirType::Record(auto_av_core::mir::RecordTypeRef {
                owner_module: "dynamic824".to_string(),
                name: "Note".to_string(),
                fields: vec![field("text", 2), field("n", 0)],
            }),
            MirType::List(ListTypeRef { elem: 3 }),
            MirType::Variant(VariantTypeRef {
                owner_module: "dynamic824".to_string(),
                name: "Opt".to_string(),
                arms: vec![
                    VariantArm {
                        name: "none".to_string(),
                        payload: None,
                    },
                    VariantArm {
                        name: "some".to_string(),
                        payload: Some(0),
                    },
                ],
            }),
            MirType::Dyn,
            // 7: same field names/types/order as Note, DIFFERENT name —
            // the sharpest same-width nominal control (V-11).
            MirType::Record(auto_av_core::mir::RecordTypeRef {
                owner_module: "dynamic824".to_string(),
                name: "Alias".to_string(),
                fields: vec![field("text", 2), field("n", 0)],
            }),
            MirType::F64,
            MirType::I64,
            MirType::U64,
            MirType::F32,
            MirType::Char,
            MirType::List(ListTypeRef { elem: 0 }),
            // 14: Fn (the pre-closure function category; a DIRECT slot
            // hits the whitelist, containers hit the type gates).
            MirType::Fn(FnSig {
                params: vec![],
                result: 0,
            }),
            // 15: Place<i32> (the 822 runtime index-token category — a
            // legal slot type whose erasure the whitelist refuses).
            MirType::Place(ListTypeRef { elem: 0 }),
        ];
        Fx {
            m,
            slots: Vec::new(),
            instrs: Vec::new(),
            ret: u32::MAX,
            params: Vec::new(),
        }
    }

    fn slot(&mut self, ty: u32, name: &str) -> u32 {
        self.slots.push(MirSlot {
            kind: SlotKind::Local,
            ty,
            name: Some(name.to_string()),
        });
        (self.slots.len() - 1) as u32
    }

    fn c(&mut self, v: MirConst) -> u32 {
        self.m.intern_const(v)
    }

    fn i(&mut self, instr: MirInstr) {
        self.instrs.push(instr);
    }

    /// Declare an entry PARAM (a Param-kind slot; wide scalars enter the
    /// dynamic profile through this seam — profile consts stay i32/bool).
    fn param(&mut self, ty: u32, name: &str) -> u32 {
        self.slots.push(MirSlot {
            kind: SlotKind::Param,
            ty,
            name: Some(name.to_string()),
        });
        let s = (self.slots.len() - 1) as u32;
        self.params.push(ty);
        s
    }

    /// Mark the return slot (its type becomes the entry result type).
    fn ret(&mut self, s: u32) {
        self.ret = s;
    }

    fn assemble(mut self, first_term: Terminator, tail: Vec<MirBlock>) -> MirModule {
        assert_ne!(self.ret, u32::MAX, "set the return slot");
        let ret_ty = self.slots[self.ret as usize].ty;
        let sig = self.m.intern_type(MirType::Fn(FnSig {
            params: self.params.clone(),
            result: ret_ty,
        }));
        let mut blocks = vec![MirBlock {
            instrs: std::mem::take(&mut self.instrs),
            term: first_term,
        }];
        blocks.extend(tail);
        self.m.funcs = vec![MirFunction {
            env: None,
            owner: self.m.owner.clone(),
            name: "main".to_string(),
            sig,
            slots: self.slots,
            blocks,
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
        }];
        self.m.entry = 0;
        self.m
    }

    /// Single-block module returning `ret`.
    fn build(self) -> MirModule {
        let r = self.ret;
        self.assemble(Terminator::Return { value: r }, Vec::new())
    }
}

/// Execute through the REAL gates: verify (the only admission of the
/// builder output) → encode → abc-admit → execute.
fn run(m: &MirModule) -> ExecOutcome {
    verify_mir(m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(m).unwrap_or_else(|e| panic!("encode: {}", e.code()));
    let loaded =
        verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("admit: {e:?}"));
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    out
}

fn success(m: &MirModule) -> Value {
    match run(m) {
        ExecOutcome::Success { value, .. } => value,
        other => panic!("expected success, got {other:?}"),
    }
}

fn success_dyn(m: &MirModule) -> DynValue {
    match success(m) {
        // The execution result hands over the container's ONLY owner (the
        // frame is gone) — try_unwrap doubles as an ownership assertion.
        Value::Dyn(d) => {
            std::rc::Rc::try_unwrap(d).expect("the result uniquely owns its Dyn container")
        }
        other => panic!("expected a Dyn result, got {other:?}"),
    }
}

/// Execute through the REAL gates with explicit entry args (the host seam
/// that carries wide scalars into the dynamic profile).
fn run_entry(m: &MirModule, args: &[Value]) -> ExecOutcome {
    verify_mir(m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(m).unwrap_or_else(|e| panic!("encode: {}", e.code()));
    let loaded =
        verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("admit: {e:?}"));
    let (out, _) = execute_entry(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        args,
    );
    out
}

fn entry_dyn(m: &MirModule, args: &[Value]) -> DynValue {
    match run_entry(m, args) {
        ExecOutcome::Success {
            value: Value::Dyn(d),
            ..
        } => std::rc::Rc::try_unwrap(d).expect("the result uniquely owns its Dyn container"),
        other => panic!("expected a Dyn result, got {other:?}"),
    }
}

fn trap_kind(m: &MirModule) -> TrapKind {
    match run(m) {
        ExecOutcome::Trap { kind, .. } => kind,
        other => panic!("expected a trap, got {other:?}"),
    }
}

fn reject_code(m: &MirModule) -> String {
    match verify_mir(m) {
        Ok(_) => panic!("expected a verify rejection"),
        Err(MirVerifyError::Invalid { code, .. })
        | Err(MirVerifyError::Unsupported { code, .. }) => code,
        Err(other) => format!("resource/{other:?}"),
    }
}

/// The current nominal dependency set for Note (the dynamic824 home).
fn note_ctx() -> Ctx {
    Ctx::note(3)
}

fn note_dyn(text: &str, n: i32) -> DynValue {
    DynValue::pack(
        Value::Record(std::rc::Rc::new(auto_av_core::contract::RecordValue {
            ty: 3,
            fields: vec![
                Value::Str(auto_av_core::contract::StrHandle::detached(
                    text.as_bytes().to_vec().into_boxed_slice(),
                )),
                Value::I32(n),
            ],
        })),
        DynType::record(
            "dynamic824",
            "Note",
            vec![("text", DynType::Str), ("n", DynType::I32)],
        ),
    )
    .expect("module-level pack")
}

/// Build a module packing Note{`text`,`n`} and returning the Dyn.
fn pack_note_module(text: &[u8], n: i32) -> MirModule {
    let mut f = Fx::new();
    let s_text = f.slot(2, "text");
    let s_n = f.slot(0, "n");
    let note = f.slot(3, "note");
    let d = f.slot(6, "d");
    let c_text = f.c(MirConst::Str(Box::from(text)));
    let c_n = f.c(MirConst::I32(n));
    f.i(MirInstr::Const {
        dst: s_text,
        value: c_text,
    });
    f.i(MirInstr::Const {
        dst: s_n,
        value: c_n,
    });
    f.i(MirInstr::Aggregate {
        dst: note,
        record_ty: 3,
        operands: vec![s_text, s_n],
    });
    f.i(MirInstr::DynPack { dst: d, src: note });
    f.ret(d);
    f.build()
}

// ---------------------------------------------------------------------------
// V-02 — owned behavior at the profile surface
// ---------------------------------------------------------------------------

/// V-02: DYN_PACK is the v1 EXPLICIT-CLONE share (contract §13.2): the
/// source slot stays readable after the pack — the executor never moves
/// out of it. Both the source field and the packed container feed the
/// result.
#[test]
fn av_dynamic_824_semantics_pack_source_stays_readable() {
    let mut f = Fx::new();
    let text = f.slot(2, "text");
    let n = f.slot(0, "n");
    let note = f.slot(3, "note");
    let d = f.slot(6, "d");
    let note2 = f.slot(3, "note2");
    let got = f.slot(0, "got");
    let src_again = f.slot(0, "src_again");
    let ret = f.slot(0, "ret");
    let c_text = f.c(MirConst::Str(Box::from(*b"src")));
    let c_n = f.c(MirConst::I32(7));
    f.i(MirInstr::Const {
        dst: text,
        value: c_text,
    });
    f.i(MirInstr::Const { dst: n, value: c_n });
    f.i(MirInstr::Aggregate {
        dst: note,
        record_ty: 3,
        operands: vec![text, n],
    });
    f.i(MirInstr::DynPack { dst: d, src: note });
    // The SOURCE slot is still readable after the pack (explicit clone,
    // not a move): FieldGet reads it directly.
    f.i(MirInstr::FieldGet {
        dst: src_again,
        base: note,
        path: vec![1],
    });
    f.i(MirInstr::DynTake {
        dst: note2,
        src: d,
        ty: 3,
    });
    f.i(MirInstr::FieldGet {
        dst: got,
        base: note2,
        path: vec![1],
    });
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: ret,
        lhs: got,
        rhs: src_again,
    }); // 7 + 7
    f.ret(ret);
    assert_eq!(success(&f.build()), Value::I32(14));
}

/// V-02 "clone两份": packing the same source twice yields TWO Dyn slots —
/// both take cleanly (releases are per-container Rc last-owner events).
#[test]
fn av_dynamic_824_semantics_two_packs_two_dyns() {
    let mut f = Fx::new();
    let text = f.slot(2, "text");
    let n = f.slot(0, "n");
    let note = f.slot(3, "note");
    let d1 = f.slot(6, "d1");
    let d2 = f.slot(6, "d2");
    let n1 = f.slot(3, "n1");
    let n2 = f.slot(3, "n2");
    let a = f.slot(0, "a");
    let b = f.slot(0, "b");
    let ret = f.slot(0, "ret");
    let c_text = f.c(MirConst::Str(Box::from(*b"dup")));
    let c_n = f.c(MirConst::I32(5));
    f.i(MirInstr::Const {
        dst: text,
        value: c_text,
    });
    f.i(MirInstr::Const { dst: n, value: c_n });
    f.i(MirInstr::Aggregate {
        dst: note,
        record_ty: 3,
        operands: vec![text, n],
    });
    f.i(MirInstr::DynPack { dst: d1, src: note });
    f.i(MirInstr::DynPack { dst: d2, src: note });
    f.i(MirInstr::DynTake {
        dst: n1,
        src: d1,
        ty: 3,
    });
    f.i(MirInstr::DynTake {
        dst: n2,
        src: d2,
        ty: 3,
    });
    f.i(MirInstr::FieldGet {
        dst: a,
        base: n1,
        path: vec![1],
    });
    f.i(MirInstr::FieldGet {
        dst: b,
        base: n2,
        path: vec![1],
    });
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: ret,
        lhs: a,
        rhs: b,
    }); // 5 + 5
    f.ret(ret);
    assert_eq!(success(&f.build()), Value::I32(10));
}

/// V-02/§13.2: a SUCCESSFUL take materializes a share — the Dyn slot is
/// NOT consumed by success: it can be taken again.
#[test]
fn av_dynamic_824_semantics_take_share_dyn_still_takeable() {
    let mut f = Fx::new();
    let text = f.slot(2, "text");
    let n = f.slot(0, "n");
    let note = f.slot(3, "note");
    let d = f.slot(6, "d");
    let t1 = f.slot(3, "t1");
    let t2 = f.slot(3, "t2");
    let a = f.slot(0, "a");
    let b = f.slot(0, "b");
    let ret = f.slot(0, "ret");
    let c_text = f.c(MirConst::Str(Box::from(*b"again")));
    let c_n = f.c(MirConst::I32(3));
    f.i(MirInstr::Const {
        dst: text,
        value: c_text,
    });
    f.i(MirInstr::Const { dst: n, value: c_n });
    f.i(MirInstr::Aggregate {
        dst: note,
        record_ty: 3,
        operands: vec![text, n],
    });
    f.i(MirInstr::DynPack { dst: d, src: note });
    f.i(MirInstr::DynTake {
        dst: t1,
        src: d,
        ty: 3,
    });
    // Second take from the SAME Dyn slot: success (share semantics).
    f.i(MirInstr::DynTake {
        dst: t2,
        src: d,
        ty: 3,
    });
    f.i(MirInstr::FieldGet {
        dst: a,
        base: t1,
        path: vec![1],
    });
    f.i(MirInstr::FieldGet {
        dst: b,
        base: t2,
        path: vec![1],
    });
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: ret,
        lhs: a,
        rhs: b,
    }); // 3 + 3
    f.ret(ret);
    assert_eq!(success(&f.build()), Value::I32(6));
}

/// V-02: DYN_IS is the non-consuming probe — a wrong-type is is `false`
/// and NEVER traps; the Dyn stays usable after the miss (the follow-up
/// take succeeds on the same container).
#[test]
fn av_dynamic_824_semantics_is_false_never_traps_dyn_usable() {
    let mut f = Fx::new();
    let text = f.slot(2, "text");
    let n = f.slot(0, "n");
    let note = f.slot(3, "note");
    let d = f.slot(6, "d");
    let miss = f.slot(1, "miss");
    let c_text = f.c(MirConst::Str(Box::from(*b"probe")));
    let c_n = f.c(MirConst::I32(4));
    f.i(MirInstr::Const {
        dst: text,
        value: c_text,
    });
    f.i(MirInstr::Const { dst: n, value: c_n });
    f.i(MirInstr::Aggregate {
        dst: note,
        record_ty: 3,
        operands: vec![text, n],
    });
    f.i(MirInstr::DynPack { dst: d, src: note });
    // Wrong-type probe: is(Note-dyn, List<Note>) = false, no trap.
    f.i(MirInstr::DynIs {
        dst: miss,
        src: d,
        ty: 4,
    });
    let ret = f.slot(0, "ret");
    let t = f.slot(3, "t");
    let got = f.slot(0, "got");
    let c_zero = f.c(MirConst::I32(0));
    f.i(MirInstr::Const {
        dst: ret,
        value: c_zero,
    });
    f.i(MirInstr::DynTake {
        dst: t,
        src: d,
        ty: 3,
    });
    f.i(MirInstr::FieldGet {
        dst: got,
        base: t,
        path: vec![1],
    });
    // miss → return 100; !miss → return got(=4).
    let c100 = f.c(MirConst::I32(100));
    f.ret(ret);
    let m = f.assemble(
        Terminator::Branch {
            cond: miss,
            then: 1,
            else_: 2,
        },
        vec![
            MirBlock {
                instrs: vec![MirInstr::Const {
                    dst: ret,
                    value: c100,
                }],
                term: Terminator::Return { value: ret },
            },
            MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: got },
            },
        ],
    );
    assert_eq!(success(&m), Value::I32(4));
}

// ---------------------------------------------------------------------------
// V-11 — nominal identity / admission negatives at the profile surface
// ---------------------------------------------------------------------------

/// V-11: same-width different-name records never compare equal — pack a
/// Note, probe with Alias (identical field names/types/order): is=false,
/// take(Alias) TRAPS av.trap.dyn-type, and the source Dyn survives the
/// failed probe for a correct take.
#[test]
fn av_dynamic_824_semantics_same_width_different_name_unequal() {
    // Probe module: is(Note-dyn, Alias) must be false and the container
    // stays takeable afterwards.
    let mut f = Fx::new();
    let text = f.slot(2, "text");
    let n = f.slot(0, "n");
    let note = f.slot(3, "note");
    let d = f.slot(6, "d");
    let miss = f.slot(1, "miss");
    let c_text = f.c(MirConst::Str(Box::from(*b"nominal")));
    let c_n = f.c(MirConst::I32(11));
    f.i(MirInstr::Const {
        dst: text,
        value: c_text,
    });
    f.i(MirInstr::Const { dst: n, value: c_n });
    f.i(MirInstr::Aggregate {
        dst: note,
        record_ty: 3,
        operands: vec![text, n],
    });
    f.i(MirInstr::DynPack { dst: d, src: note });
    f.i(MirInstr::DynIs {
        dst: miss,
        src: d,
        ty: 7, // Alias — same shape, other name
    });
    let ret = f.slot(0, "ret");
    let t = f.slot(3, "t");
    let got = f.slot(0, "got");
    let c_zero = f.c(MirConst::I32(0));
    f.i(MirInstr::Const {
        dst: ret,
        value: c_zero,
    });
    f.i(MirInstr::DynTake {
        dst: t,
        src: d,
        ty: 3,
    });
    f.i(MirInstr::FieldGet {
        dst: got,
        base: t,
        path: vec![1],
    });
    f.ret(ret);
    let c100 = f.c(MirConst::I32(100));
    let probe = f.assemble(
        Terminator::Branch {
            cond: miss,
            then: 1,
            else_: 2,
        },
        vec![
            MirBlock {
                instrs: vec![MirInstr::Const {
                    dst: ret,
                    value: c100,
                }],
                term: Terminator::Return { value: ret },
            },
            MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: got },
            },
        ],
    );
    // miss=false → the else path returns got(=11), NOT 100.
    assert_eq!(success(&probe), Value::I32(11));

    // Trap module: take(Note-dyn as Alias) traps with the source intact
    // (checked runtime failure, §5.5).
    let mut g = Fx::new();
    let text2 = g.slot(2, "text");
    let n2 = g.slot(0, "n");
    let note2 = g.slot(3, "note");
    let d2 = g.slot(6, "d");
    let t2 = g.slot(7, "as_alias");
    let c_text2 = g.c(MirConst::Str(Box::from(*b"nominal")));
    let c_n2 = g.c(MirConst::I32(11));
    g.i(MirInstr::Const {
        dst: text2,
        value: c_text2,
    });
    g.i(MirInstr::Const {
        dst: n2,
        value: c_n2,
    });
    g.i(MirInstr::Aggregate {
        dst: note2,
        record_ty: 3,
        operands: vec![text2, n2],
    });
    g.i(MirInstr::DynPack {
        dst: d2,
        src: note2,
    });
    g.i(MirInstr::DynTake {
        dst: t2,
        src: d2,
        ty: 7, // Alias over a Note
    });
    g.ret(t2);
    let trap = g.build();
    assert_eq!(trap_kind(&trap), TrapKind::DynTypeMismatch);
}

/// V-11: a take/is TYPE OPERAND outside the erasable closure (Fn) is an
/// ADMISSION violation — rejected at verify, nothing executes.
#[test]
fn av_dynamic_824_semantics_unknown_type_operand_rejected() {
    let mut f = Fx::new();
    let text = f.slot(2, "text");
    let n = f.slot(0, "n");
    let note = f.slot(3, "note");
    let d = f.slot(6, "d");
    let out = f.slot(3, "out");
    let c_text = f.c(MirConst::Str(Box::from(*b"t")));
    let c_n = f.c(MirConst::I32(1));
    f.i(MirInstr::Const {
        dst: text,
        value: c_text,
    });
    f.i(MirInstr::Const { dst: n, value: c_n });
    f.i(MirInstr::Aggregate {
        dst: note,
        record_ty: 3,
        operands: vec![text, n],
    });
    f.i(MirInstr::DynPack { dst: d, src: note });
    f.i(MirInstr::DynTake {
        dst: out,
        src: d,
        ty: 14, // Fn — outside the closure
    });
    f.ret(out);
    let m = f.build();
    assert_eq!(reject_code(&m), "mir.dyn.whitelist");
}

/// V-11: a frozen payload carries no authority — the SAME value produced
/// under a trace-capability module and a capability-free module freezes to
/// IDENTICAL bytes (authority lives in module admission, never in data).
#[test]
fn av_dynamic_824_semantics_frozen_payload_carries_no_authority() {
    let build = |caps: Vec<&str>, trace: bool| -> MirModule {
        let mut f = Fx::new();
        let text = f.slot(2, "text");
        let n = f.slot(0, "n");
        let note = f.slot(3, "note");
        let d = f.slot(6, "d");
        let c_text = f.c(MirConst::Str(Box::from(*b"inert")));
        let c_n = f.c(MirConst::I32(2));
        f.i(MirInstr::Const {
            dst: text,
            value: c_text,
        });
        f.i(MirInstr::Const { dst: n, value: c_n });
        f.i(MirInstr::Aggregate {
            dst: note,
            record_ty: 3,
            operands: vec![text, n],
        });
        f.i(MirInstr::DynPack { dst: d, src: note });
        if trace {
            f.i(MirInstr::Trace {
                event: auto_av_core::mir::TraceEvent::MarkA,
            });
        }
        f.ret(d);
        let mut m = f.build();
        m.capabilities = caps.into_iter().map(|s| s.to_string()).collect();
        m
    };
    let privileged = build(vec!["hir.test.trace.v1"], true);
    let plain = build(Vec::new(), false);
    let a = success_dyn(&privileged);
    let b = success_dyn(&plain);
    let fa = freeze(&a, &TraversalOptions::default()).expect("freeze privileged");
    let fb = freeze(&b, &TraversalOptions::default()).expect("freeze plain");
    assert_eq!(
        fa, fb,
        "capability-bearing execution must not alter frozen bytes"
    );
}

// ---------------------------------------------------------------------------
// V-03 — bit fidelity through the whole path
// ---------------------------------------------------------------------------

fn empty_ctx() -> Ctx {
    Ctx::empty()
}

/// V-03: f64 payloads (NaN payload bits, ±0, ±∞) enter through the ENTRY
/// PARAM seam (profile consts stay i32/bool — wide-scalar consts are
/// data-profile-only) and survive pack → execute → freeze → unfreeze →
/// take BIT-EXACTLY; the production freeze equals the module-side freeze
/// of the literal value byte-for-byte.
#[test]
fn av_dynamic_824_semantics_f64_bits_preserved_through_dyn() {
    let cases: [u64; 5] = [
        0x7ff8_0000_0000_0042, // quiet NaN with a payload
        0xfff8_0000_dead_beef, // negative quiet NaN payload
        0x8000_0000_0000_0000, // -0.0
        0x0000_0000_0000_0000, // +0.0
        0x7ff0_0000_0000_0000, // +∞
    ];
    for bits in cases {
        let mut f = Fx::new();
        let v = f.param(8, "v");
        let d = f.slot(6, "d");
        f.i(MirInstr::DynPack { dst: d, src: v });
        f.ret(d);
        let got = entry_dyn(&f.build(), &[Value::F64(f64::from_bits(bits))]);
        assert!(got.is(&DynType::F64), "erased type must be F64");
        // freeze → unfreeze → take, all bit-exact.
        let bytes = freeze(&got, &TraversalOptions::default()).expect("freeze");
        let back =
            unfreeze(&bytes, &empty_ctx().get(), &TraversalOptions::default()).expect("unfreeze");
        match back.take_as(&DynType::F64).expect("take F64") {
            Value::F64(x) => assert_eq!(x.to_bits(), bits, "round trip must be bit-exact"),
            other => panic!("not an F64: {other:?}"),
        }
        // The production bytes equal the module-side freeze of the literal.
        let want = DynValue::pack(Value::F64(f64::from_bits(bits)), DynType::F64).expect("pack");
        assert_eq!(bytes, freeze(&want, &TraversalOptions::default()).unwrap());
    }
}

/// V-03: f32 payload bits survive the same path.
#[test]
fn av_dynamic_824_semantics_f32_bits_preserved_through_dyn() {
    let cases: [u32; 3] = [0x7fc0_0042, 0x8000_0000, 0x7f80_0000];
    for bits in cases {
        let mut f = Fx::new();
        let v = f.param(11, "v");
        let d = f.slot(6, "d");
        f.i(MirInstr::DynPack { dst: d, src: v });
        f.ret(d);
        let got = entry_dyn(&f.build(), &[Value::F32(f32::from_bits(bits))]);
        let bytes = freeze(&got, &TraversalOptions::default()).expect("freeze");
        let back =
            unfreeze(&bytes, &empty_ctx().get(), &TraversalOptions::default()).expect("unfreeze");
        match back.take_as(&DynType::F32).expect("take F32") {
            Value::F32(x) => assert_eq!(x.to_bits(), bits),
            other => panic!("not an F32: {other:?}"),
        }
    }
}

/// V-03: integer extremes (i64::MIN, u64::MAX) and a u64 numerically equal
/// to a LIVE arena allocation id stay integers — no handle/type confusion
/// (the live Str with id 0 coexists with a packed U64(0)).
#[test]
fn av_dynamic_824_semantics_int_extremes_and_id_collision_stay_integers() {
    // i64::MIN / u64::MAX round trips (entry-param seam, like the floats).
    for (ty_idx, ty, arg) in [
        (9u32, DynType::I64, Value::I64(i64::MIN)),
        (10u32, DynType::U64, Value::U64(u64::MAX)),
    ] {
        let mut f = Fx::new();
        let v = f.param(ty_idx, "v");
        let d = f.slot(6, "d");
        f.i(MirInstr::DynPack { dst: d, src: v });
        f.ret(d);
        let got = entry_dyn(&f.build(), &[arg.clone()]);
        let bytes = freeze(&got, &TraversalOptions::default()).expect("freeze");
        let back =
            unfreeze(&bytes, &empty_ctx().get(), &TraversalOptions::default()).expect("unfreeze");
        let taken = back.take_as(&ty).expect("take");
        assert_eq!(taken, arg, "extreme integer must round trip");
    }

    // A live allocation (arena id 0 in a fresh run) and a packed U64(0):
    // the value stays an integer — erased types are never handle-aware.
    let mut f = Fx::new();
    let v = f.param(10, "v"); // Param slots must lead the frame
    let live = f.slot(2, "live");
    let d = f.slot(6, "d");
    let c_live = f.c(MirConst::Str(Box::from(*b"id0")));
    f.i(MirInstr::Const {
        dst: live,
        value: c_live,
    });
    f.i(MirInstr::DynPack { dst: d, src: v });
    f.ret(d);
    let got = entry_dyn(&f.build(), &[Value::U64(0)]);
    assert!(got.is(&DynType::U64));
    match got.borrow_as(&DynType::U64).expect("borrow U64") {
        Value::U64(x) => assert_eq!(*x, 0),
        other => panic!("not a U64: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// V-04 / V-12 — the portable boundary at the profile surface
// ---------------------------------------------------------------------------

/// V-04: restored handles are DETACHED — the wire never carries arena ids
/// or hooks. The in-execution payload Str (the second allocation, id 1)
/// restores with id 0 and binds to the CURRENT type table.
#[test]
fn av_dynamic_824_semantics_restored_handles_detached() {
    let mut f = Fx::new();
    let text1 = f.slot(2, "text1");
    let text2 = f.slot(2, "text2");
    let n = f.slot(0, "n");
    let note = f.slot(3, "note");
    let d = f.slot(6, "d");
    let c_a = f.c(MirConst::Str(Box::from(*b"first")));
    let c_b = f.c(MirConst::Str(Box::from(*b"second")));
    let c_n = f.c(MirConst::I32(1));
    f.i(MirInstr::Const {
        dst: text1,
        value: c_a,
    }); // arena id 0 (kept live)
    f.i(MirInstr::Const {
        dst: text2,
        value: c_b,
    }); // arena id 1 — the payload
    f.i(MirInstr::Const { dst: n, value: c_n });
    f.i(MirInstr::Aggregate {
        dst: note,
        record_ty: 3,
        operands: vec![text2, n],
    });
    f.i(MirInstr::DynPack { dst: d, src: note });
    f.ret(d);
    let got = success_dyn(&f.build());
    // The ORIGINAL is an in-execution arena value: second alloc → id 1.
    match got.as_value() {
        Value::Record(rec) => match &rec.fields[0] {
            Value::Str(h) => {
                assert_eq!(h.rc.id, 1, "in-execution payload str carries its arena id")
            }
            other => panic!("field 0 not a Str: {other:?}"),
        },
        other => panic!("expected a record, got {other:?}"),
    }
    // The restored graph is detached: id 0 everywhere.
    let bytes = freeze(&got, &TraversalOptions::default()).expect("freeze");
    let back = unfreeze(&bytes, &note_ctx().get(), &TraversalOptions::default()).expect("unfreeze");
    match back.as_value() {
        Value::Record(rec) => {
            match &rec.fields[0] {
                Value::Str(h) => assert_eq!(h.rc.id, 0, "restored str is detached (id 0)"),
                other => panic!("restored field 0 not a Str: {other:?}"),
            }
            // And the restored nominal index binds to the CURRENT table.
            assert_eq!(rec.ty, 3);
        }
        other => panic!("restored value not a record: {other:?}"),
    }
}

/// V-04/V-12: a shared DAG (two list elements referencing ONE record Rc)
/// packs fine (container sharing is legal) but the portable boundary
/// rejects it — and the failed freeze does NOT consume the source (the
/// take still works on the same Dyn).
#[test]
fn av_dynamic_824_semantics_shared_dag_pack_ok_freeze_rejects() {
    let mut f = Fx::new();
    let text = f.slot(2, "text");
    let n = f.slot(0, "n");
    let note = f.slot(3, "note");
    let note2 = f.slot(3, "note2");
    let list = f.slot(4, "list");
    let d = f.slot(6, "d");
    let c_text = f.c(MirConst::Str(Box::from(*b"shared")));
    let c_n = f.c(MirConst::I32(6));
    f.i(MirInstr::Const {
        dst: text,
        value: c_text,
    });
    f.i(MirInstr::Const { dst: n, value: c_n });
    f.i(MirInstr::Aggregate {
        dst: note,
        record_ty: 3,
        operands: vec![text, n],
    });
    // Copy shares the record Rc — the list will hold TWO refs to one node.
    f.i(MirInstr::Copy {
        dst: note2,
        src: note,
    });
    f.i(MirInstr::ListNew {
        dst: list,
        list_ty: 4,
        cap: 2,
    });
    f.i(MirInstr::ListPush {
        root: list,
        value: note,
    });
    f.i(MirInstr::ListPush {
        root: list,
        value: note2,
    });
    f.i(MirInstr::DynPack { dst: d, src: list });
    f.ret(d);
    let got = success_dyn(&f.build());
    // The portable boundary rejects the shared graph...
    let err = freeze(&got, &TraversalOptions::default()).expect_err("shared DAG must not freeze");
    assert!(
        matches!(
            err,
            PortableError::Unsupported {
                category: "shared-graph"
            }
        ),
        "unexpected error: {err:?}"
    );
    // ...and the source is unconsumed: the take still works.
    let list_ty = DynType::List(Box::new(DynType::record(
        "dynamic824",
        "Note",
        vec![("text", DynType::Str), ("n", DynType::I32)],
    )));
    match got.take_as(&list_ty).expect("take after failed freeze") {
        Value::List(h) => assert_eq!(h.len(), 2),
        other => panic!("not a list: {other:?}"),
    }
}

/// V-12: `List<Fn>` cannot even be DECLARED — Fn is not an element kind,
/// the type-table gate refuses the category before any instruction exists
/// (a closed surface one gate EARLIER than the whitelist; since the 823
/// integration the Closure category exists at tag 17 but stays outside
/// the dynamic erasable surface — same category refusal).
#[test]
fn av_dynamic_824_semantics_list_of_fn_not_declarable() {
    let mut f = Fx::new();
    let l = f.slot(0, "x");
    let _ = l;
    let ret = f.slot(0, "ret");
    let c_zero = f.c(MirConst::I32(0));
    f.i(MirInstr::Const {
        dst: ret,
        value: c_zero,
    });
    f.ret(ret);
    let mut m = f.build();
    let fn_ty = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: 0,
    }));
    let list_fn = m.intern_type(MirType::List(ListTypeRef { elem: fn_ty }));
    m.funcs[0].slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: list_fn,
        name: Some("fns".to_string()),
    });
    assert_eq!(reject_code(&m), "mir.type.list-elem");
}

/// V-12: a record holding an Fn field cannot even be DECLARED — the type
/// gate refuses the function category inside containers one gate earlier
/// than the whitelist (closed surface, delivered categories only).
#[test]
fn av_dynamic_824_semantics_record_with_fn_not_declarable() {
    let mut f = Fx::new();
    let rec = f.slot(0, "placeholder");
    let d = f.slot(6, "d");
    f.i(MirInstr::DynPack { dst: d, src: rec });
    f.ret(d);
    let mut m = f.build();
    let fn_ty = 14; // the shared Fn entry
    let rec_fn = m.intern_type(MirType::Record(auto_av_core::mir::RecordTypeRef {
        owner_module: "dynamic824".to_string(),
        name: "RecFn".to_string(),
        fields: vec![field("f", fn_ty)],
    }));
    m.funcs[0].slots[rec as usize].ty = rec_fn;
    assert_eq!(reject_code(&m), "mir.record-field-type");
}

/// V-12: a record holding a Place field is refused at the TYPE gate
/// (`mir.field-borrow` — borrowed view/token kinds are not field kinds).
#[test]
fn av_dynamic_824_semantics_record_with_place_not_declarable() {
    let mut f = Fx::new();
    let rec = f.slot(0, "placeholder");
    let d = f.slot(6, "d");
    f.i(MirInstr::DynPack { dst: d, src: rec });
    f.ret(d);
    let mut m = f.build();
    let place_ty = 15;
    let rec_p = m.intern_type(MirType::Record(auto_av_core::mir::RecordTypeRef {
        owner_module: "dynamic824".to_string(),
        name: "RecP".to_string(),
        fields: vec![field("p", place_ty)],
    }));
    m.funcs[0].slots[rec as usize].ty = rec_p;
    assert_eq!(reject_code(&m), "mir.field-borrow");
}

/// V-12: a DIRECT Place slot (a legal slot type — places live in slots)
/// is refused by the recursive erasable-closure whitelist itself: the
/// index-token category has no erasure, at the root or any depth.
#[test]
fn av_dynamic_824_semantics_place_slot_pack_rejected() {
    let mut f = Fx::new();
    let list = f.slot(13, "list");
    let idx = f.slot(0, "idx");
    let p = f.slot(15, "p");
    let d = f.slot(6, "d");
    let c_zero = f.c(MirConst::I32(0));
    f.i(MirInstr::ListNew {
        dst: list,
        list_ty: 13,
        cap: 0,
    });
    f.i(MirInstr::Const {
        dst: idx,
        value: c_zero,
    });
    f.i(MirInstr::ListPlace {
        dst: p,
        base: list,
        index: idx,
    });
    f.i(MirInstr::DynPack { dst: d, src: p });
    f.ret(d);
    let m = f.build();
    assert_eq!(reject_code(&m), "mir.dyn.whitelist");
}

/// V-12: erasure is not recursive — a record nesting Dyn is type-legal but
/// Aggregate refuses the Dyn operand at construction, and a direct
/// Dyn→Dyn pack is refused by the whitelist itself.
#[test]
fn av_dynamic_824_semantics_nested_dyn_rejected() {
    let mut f = Fx::new();
    let v = f.slot(0, "v");
    let inner = f.slot(6, "inner");
    let rec = f.slot(3, "rec"); // placeholder type, swapped below
    let d = f.slot(6, "d");
    let c_one = f.c(MirConst::I32(1));
    f.i(MirInstr::Const {
        dst: v,
        value: c_one,
    });
    f.i(MirInstr::DynPack { dst: inner, src: v });
    f.i(MirInstr::Aggregate {
        dst: rec,
        record_ty: 3, // placeholder, swapped below
        operands: vec![inner],
    });
    f.i(MirInstr::DynPack { dst: d, src: rec });
    f.ret(d);
    let mut m = f.build();
    // Intern RecDyn{d: Dyn} LOCALLY (type-table-legal: Dyn is a record
    // field kind) and swap the aggregate/slot onto it.
    let rec_dyn = m.intern_type(MirType::Record(auto_av_core::mir::RecordTypeRef {
        owner_module: "dynamic824".to_string(),
        name: "RecDyn".to_string(),
        fields: vec![field("d", 6)],
    }));
    if let MirInstr::Aggregate { record_ty, .. } = &mut m.funcs[0].blocks[0].instrs[1] {
        *record_ty = rec_dyn;
    }
    m.funcs[0].slots[rec as usize].ty = rec_dyn;
    // The CONSTRUCTION gate closes first: Aggregate refuses a Dyn operand
    // (a record value cannot even be built around an erased inner).
    assert_eq!(reject_code(&m), "mir.aggregate-type");

    // And the direct erasure-of-erasure: pack a Dyn slot into another Dyn
    // — the whitelist refuses MirType::Dyn itself.
    let mut g = Fx::new();
    let v2 = g.slot(0, "v");
    let inner2 = g.slot(6, "inner");
    let outer = g.slot(6, "outer");
    let c_one = g.c(MirConst::I32(1));
    g.i(MirInstr::Const {
        dst: v2,
        value: c_one,
    });
    g.i(MirInstr::DynPack {
        dst: inner2,
        src: v2,
    });
    g.i(MirInstr::DynPack {
        dst: outer,
        src: inner2,
    });
    g.ret(outer);
    assert_eq!(reject_code(&g.build()), "mir.dyn.whitelist");
}

// ---------------------------------------------------------------------------
// V-05 — new-domain admission (unfreeze seam) at the profile surface
// ---------------------------------------------------------------------------

/// V-05: the same frozen payload rebuilds in TWO fresh decode domains with
/// NEW (detached) handles and equal content; a domain whose bindings do
/// not match (unknown type / changed layout) refuses BEFORE any value is
/// built — never a guess, never a partial install.
#[test]
fn av_dynamic_824_semantics_portable_admission_two_domains_and_refusals() {
    let src = note_dyn("admission", 8);
    let bytes = freeze(&src, &TraversalOptions::default()).expect("freeze");

    // Two fresh domains (same bindings, rebuilt twice — "不同store重建"):
    // new handles, equal content, nominal index bound to the CURRENT table.
    for want_index in [3u32, 77u32] {
        let ctx = Ctx::note(want_index);
        let back = unfreeze(&bytes, &ctx.get(), &TraversalOptions::default()).expect("rebuild");
        // Cross-domain content equality is BYTE equality (Value::eq
        // compares the record's positional index, which by contract binds
        // to each domain's own table).
        assert_eq!(
            freeze(&back, &TraversalOptions::default()).unwrap(),
            bytes,
            "rebuilt value freezes to the same bytes"
        );
        if want_index == 3 {
            assert_eq!(back, src, "same-index domain: full value equality");
        }
        match back.as_value() {
            Value::Record(rec) => assert_eq!(
                rec.ty, want_index,
                "the positional index binds to the current table"
            ),
            other => panic!("not a record: {other:?}"),
        }
    }

    // Unknown type (foreign owner/name): refused at admission.
    let foreign = Ctx::other_named(3);
    match unfreeze(&bytes, &foreign.get(), &TraversalOptions::default()) {
        Err(PortableError::Admission { code }) => assert_eq!(code, "portable.unknown-type"),
        other => panic!("unknown type must be an admission refusal: {other:?}"),
    }

    // Changed layout (a field renamed — a revision change): controlled
    // refusal, never a reuse of stale offsets.
    let renamed = Ctx::note_fields(3, vec!["caption", "n"]);
    match unfreeze(&bytes, &renamed.get(), &TraversalOptions::default()) {
        Err(PortableError::Admission { code }) => assert_eq!(code, "portable.layout-mismatch"),
        other => panic!("layout change must be an admission refusal: {other:?}"),
    }

    // The refusals consumed nothing: the source bytes still rebuild in the
    // honest domain.
    let ctx = note_ctx();
    assert!(unfreeze(&bytes, &ctx.get(), &TraversalOptions::default()).is_ok());
}

/// V-12/V-06 (unfreeze side): a fault at the Nth rebuilt node fails with
/// the RESOURCE classification (the same class as the node budget — an
/// injected fault is a boundary equivalent), and the SAME bytes retry
/// clean — no partial graph survives a failed restore.
#[test]
fn av_dynamic_824_semantics_unfreeze_fault_retry_clean() {
    let src = note_dyn("retry", 9);
    let bytes = freeze(&src, &TraversalOptions::default()).expect("freeze");
    let ctx = note_ctx();

    let mut opts = TraversalOptions::default();
    opts.fail_after_nodes = Some(2);
    match unfreeze(&bytes, &ctx.get(), &opts) {
        Err(PortableError::Budget { code }) => assert_eq!(code, "av.resource.dyn-nodes"),
        other => panic!("injected node fault must be a budget-class failure: {other:?}"),
    }
    // Clean retry on the same input: full success (bytes were not consumed).
    let back = unfreeze(&bytes, &ctx.get(), &TraversalOptions::default()).expect("retry clean");
    assert_eq!(back, src);
}

// ---------------------------------------------------------------------------
// V-14 — the same-source integration axis
// ---------------------------------------------------------------------------

/// V-14: the EXECUTION path and the MODULE path produce byte-identical
/// frozen payloads for equal content — the executor, the DynValue
/// container and the portable codec are ONE source (core's registered
/// modules), not two implementations that could drift.
#[test]
fn av_dynamic_824_semantics_production_same_source_byte_equivalence() {
    // Production: run the real pipeline to a Dyn result (record payload).
    let produced = success_dyn(&pack_note_module(b"portable", 11));

    // Module side: pack the same content directly.
    let direct = note_dyn("portable", 11);
    assert_eq!(produced, direct, "execution and module packs agree");

    // Freeze both through the ONE production codec: byte-identical.
    let a = freeze(&produced, &TraversalOptions::default()).expect("freeze produced");
    let b = freeze(&direct, &TraversalOptions::default()).expect("freeze direct");
    assert_eq!(a, b);

    // Cross-API restore: production bytes through the module seam.
    let back = unfreeze(&a, &note_ctx().get(), &TraversalOptions::default()).expect("unfreeze");
    assert_eq!(back, direct);
    // Determinism: a second freeze of the restored value is stable.
    assert_eq!(freeze(&back, &TraversalOptions::default()).unwrap(), a);
}

/// F-824-R1-02 (adapter axis): the REAL `portable_bindings_of` builder
/// derives CURRENT field types from the module's type table. A same-name
/// record whose field TYPE changed in the current module refuses the
/// restore at admission — through the production adapter, not a hand
/// binding; the honest module still restores bit-exactly.
#[test]
fn av_dynamic_824_semantics_restore_revalidates_current_field_types() {
    fn module_with_bool_field(field_is_bool: bool) -> MirModule {
        let mut m = MirModule::new_dynamic(owner_scope());
        m.types = vec![
            MirType::I32,
            MirType::Bool,
            MirType::Record(RecordTypeRef {
                owner_module: "dynamic824".to_string(),
                name: "R".to_string(),
                fields: vec![MirFieldDef {
                    name: "x".to_string(),
                    ty: if field_is_bool { 1 } else { 0 },
                }],
            }),
        ];
        m
    }
    let wire_ty = DynType::record("dynamic824", "R", vec![("x", DynType::I32)]);
    let d = DynValue::pack(
        Value::Record(Rc::new(auto_av_core::contract::RecordValue {
            ty: 2,
            fields: vec![Value::I32(42)],
        })),
        wire_ty,
    )
    .expect("pack");
    let bytes = freeze(&d, &TraversalOptions::default()).expect("freeze");

    // Current schema says R.x is bool — the adapter's binding CARRIES the
    // type, so the frozen i32 payload refuses (the review's Replay B
    // through the production decode domain).
    let changed_module = module_with_bool_field(true);
    let (recs, vars) = auto_av_probe::dynamic_fixture::portable_bindings_of(&changed_module)
        .expect("binding domain");
    let changed = RestoreContext::new(&recs, &vars);
    match unfreeze(&bytes, &changed, &TraversalOptions::default()) {
        Err(PortableError::Admission { code }) => assert_eq!(code, "portable.layout-mismatch"),
        other => panic!("changed field type must refuse: {other:?}"),
    }

    // The honest module restores; the positional index binds to its table.
    let honest_module = module_with_bool_field(false);
    let (recs, vars) = auto_av_probe::dynamic_fixture::portable_bindings_of(&honest_module)
        .expect("binding domain");
    let honest = RestoreContext::new(&recs, &vars);
    let back = unfreeze(&bytes, &honest, &TraversalOptions::default()).expect("restore");
    match back.as_value() {
        Value::Record(r) => {
            assert_eq!(r.ty, 2);
            assert!(matches!(r.fields[0], Value::I32(42)));
        }
        other => panic!("not a record: {other:?}"),
    }
    assert_eq!(freeze(&back, &TraversalOptions::default()).unwrap(), bytes);
}
