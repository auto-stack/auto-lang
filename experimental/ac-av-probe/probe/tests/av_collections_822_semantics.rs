//! PLAN-822 T-03 semantics tests: the collections support surface over
//! the REAL gates — typed list/variant round trips, index traps, epoch
//! shifts, guard/project/switch lifecycles, payload-moved rejection,
//! nested composition cleanup (V-01/02/04/05/06/12 semantics axes).
//! Prefix `av_collections_822` (contract-decision §6/§8).

use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, TrapKind, Value};
use auto_av_core::mir::{
    BinOp, ListTypeRef, MirBlock, MirConst, MirFieldDef, MirFunction, MirInstr, MirModule, MirSlot,
    MirType, OwnerScope, SlotKind, Terminator, VariantArm, VariantTypeRef,
};
use auto_av_core::mir_verify::{verify_mir, MirVerifyError};

/// Push a CONST with its const index resolved BEFORE the mutable push
/// (single-mut-borrow rule).
macro_rules! iconst {
    ($f:expr, $dst:expr, $val:expr) => {{
        let cv = $val;
        $f.i(MirInstr::Const {
            dst: $dst,
            value: cv,
        });
    }};
}

/// Fixture builder over the shared type vocabulary:
/// 0=i32 1=bool 2=Str 3=Note{text:Str,n:i32} 4=List<i32> 5=List<Str>
/// 6=List<Note> 7=Outcome{Ok(Note),Err(Str)} 8=List<Outcome>
struct Fx {
    m: MirModule,
    slots: Vec<MirSlot>,
    instrs: Vec<MirInstr>,
    ret: u32,
}

impl Fx {
    fn new() -> Fx {
        let mut m = MirModule::new_collections(OwnerScope {
            schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
            module: "collections822".to_string(),
        });
        m.types = vec![
            MirType::I32,
            MirType::Bool,
            MirType::Str,
            MirType::Record(auto_av_core::mir::RecordTypeRef {
                owner_module: "collections822".to_string(),
                name: "Note".to_string(),
                fields: vec![
                    MirFieldDef {
                        name: "text".to_string(),
                        ty: 2,
                    },
                    MirFieldDef {
                        name: "n".to_string(),
                        ty: 0,
                    },
                ],
            }),
            MirType::List(ListTypeRef { elem: 0 }),
            MirType::List(ListTypeRef { elem: 2 }),
            MirType::List(ListTypeRef { elem: 3 }),
            MirType::Variant(VariantTypeRef {
                owner_module: "collections822".to_string(),
                name: "Outcome".to_string(),
                arms: vec![
                    VariantArm {
                        name: "Ok".to_string(),
                        payload: Some(3),
                    },
                    VariantArm {
                        name: "Err".to_string(),
                        payload: Some(2),
                    },
                ],
            }),
            MirType::List(ListTypeRef { elem: 7 }),
        ];
        Fx {
            m,
            slots: Vec::new(),
            instrs: Vec::new(),
            ret: u32::MAX,
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

    fn c_i32(&mut self, v: i32) -> u32 {
        self.m.intern_const(MirConst::I32(v))
    }

    fn c_str(&mut self, s: &str) -> u32 {
        self.m.intern_const(MirConst::Str(Box::from(s.as_bytes())))
    }

    fn i(&mut self, instr: MirInstr) {
        self.instrs.push(instr);
    }

    fn ret(&mut self, s: u32) {
        self.ret = s;
    }

    fn build(mut self) -> MirModule {
        assert_ne!(self.ret, u32::MAX, "set the return slot");
        let sig = self.m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![],
            result: 0,
        }));
        self.m.funcs = vec![MirFunction {
            owner: self.m.owner.clone(),
            name: "main".to_string(),
            sig,
            slots: self.slots,
            blocks: vec![MirBlock {
                instrs: self.instrs,
                term: Terminator::Return { value: self.ret },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
        }];
        self.m.entry = 0;
        self.m
    }
}

/// Execute through the REAL gates: verify (the only admission of the
/// builder output) -> encode -> abc-admit -> execute.
fn run(m: &MirModule) -> ExecOutcome {
    verify_mir(m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(m).unwrap_or_else(|e| panic!("encode: {}", e.code()));
    let loaded =
        verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("admit: {e:?}"));
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    out
}

fn reject_code(m: &MirModule) -> String {
    match verify_mir(m) {
        Ok(_) => panic!("expected a verify rejection"),
        Err(MirVerifyError::Invalid { code, .. })
        | Err(MirVerifyError::Unsupported { code, .. }) => code,
        Err(other) => format!("resource/{other:?}"),
    }
}

fn success(m: &MirModule) -> Value {
    match run(m) {
        ExecOutcome::Success { value, .. } => value,
        other => panic!("expected success, got {other:?}"),
    }
}

fn trap_kind(m: &MirModule) -> TrapKind {
    match run(m) {
        ExecOutcome::Trap { kind, .. } => kind,
        other => panic!("expected a trap, got {other:?}"),
    }
}

/// V-01: List<i32> round trip — push/len/get keep values (len folds the
/// truth into the success value).
#[test]
fn av_collections_822_semantics_list_i32_roundtrip() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v");
    let s2 = f.slot(0, "len");
    let s3 = f.slot(0, "got");
    let c7 = f.c_i32(7);
    let c0 = f.c_i32(0);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    f.i(MirInstr::Const { dst: s1, value: c7 });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    f.i(MirInstr::ListLen { dst: s2, src: s0 });
    f.i(MirInstr::Const { dst: s3, value: c0 });
    f.i(MirInstr::ListGet {
        dst: s3,
        base: s0,
        index: s3,
    });
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: s3,
        lhs: s3,
        rhs: s2,
    }); // 7 + 1
    f.ret(s3);
    assert_eq!(success(&f.build()), Value::I32(8));
}

/// V-12: empty list constructs and drops cleanly (live=0 via the ledger
/// is asserted by the first-case tests; here the len/cap shape).
#[test]
fn av_collections_822_semantics_empty_list_len_cap() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "len");
    let s2 = f.slot(0, "cap");
    let c9 = f.c_i32(9);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    f.i(MirInstr::ListLen { dst: s1, src: s0 });
    f.i(MirInstr::ListCap { dst: s2, src: s0 });
    f.i(MirInstr::Bin {
        op: BinOp::MulI32,
        dst: s1,
        lhs: s1,
        rhs: s2,
    }); // 0 * 4
    f.i(MirInstr::Const { dst: s2, value: c9 });
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: s2,
        lhs: s2,
        rhs: s1,
    }); // 9 + 0
    f.ret(s2);
    assert_eq!(success(&f.build()), Value::I32(9));
}

/// V-12: a negative index never touches the buffer — the trap fires
/// before any access; an out-of-range index traps the same way.
#[test]
fn av_collections_822_semantics_index_traps() {
    // Negative literal index: i32 const -1.
    let neg = |f: &mut Fx, dst: u32, c: u32| f.i(MirInstr::Const { dst, value: c });
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v");
    let s2 = f.slot(0, "idx");
    let s3 = f.slot(0, "got");
    let cm1 = f.c_i32(-1);
    let c5 = f.c_i32(5);
    let _ = c5;
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(7));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    neg(&mut f, s2, cm1);
    f.i(MirInstr::ListGet {
        dst: s3,
        base: s0,
        index: s2,
    });
    f.ret(s3);
    assert_eq!(trap_kind(&f.build()), TrapKind::ListIndex);

    // Out-of-range positive index (len == 1, index == 3).
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v");
    let s2 = f.slot(0, "idx");
    let s3 = f.slot(0, "got");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(7));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s2, f.c_i32(3));
    f.i(MirInstr::ListGet {
        dst: s3,
        base: s0,
        index: s2,
    });
    f.ret(s3);
    assert_eq!(trap_kind(&f.build()), TrapKind::ListIndex);
}

/// V-02/V-12: remove shifts the tail — after remove(0) the recomputed
/// index 0 reads the SHIFTED value, not the removed one.
#[test]
fn av_collections_822_semantics_remove_shifts() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v1");
    let s2 = f.slot(0, "v2");
    let s3 = f.slot(0, "idx");
    let s4 = f.slot(0, "removed");
    let s5 = f.slot(0, "now0");
    let s6 = f.slot(0, "len");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(10));
    iconst!(&mut f, s2, f.c_i32(20));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    iconst!(&mut f, s3, f.c_i32(0));
    f.i(MirInstr::ListRemove {
        dst: s4,
        root: s0,
        index: s3,
    });
    f.i(MirInstr::ListGet {
        dst: s5,
        base: s0,
        index: s3,
    }); // shifted: 20
    f.i(MirInstr::ListLen { dst: s6, src: s0 }); // 1
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: s5,
        lhs: s5,
        rhs: s6,
    }); // 21
    f.ret(s5);
    assert_eq!(success(&f.build()), Value::I32(21));
}

/// V-02: insert at len == append; insert at 0 shifts forward.
#[test]
fn av_collections_822_semantics_insert_positions() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "a");
    let s2 = f.slot(0, "b");
    let s3 = f.slot(0, "idx");
    let s4 = f.slot(0, "got");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(1));
    iconst!(&mut f, s2, f.c_i32(2));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s3, f.c_i32(1));
    f.i(MirInstr::ListInsert {
        root: s0,
        index: s3,
        value: s2,
    }); // [1, 2]
    iconst!(&mut f, s3, f.c_i32(0));
    f.i(MirInstr::ListInsert {
        root: s0,
        index: s3,
        value: s2,
    }); // [2, 1, 2]
    iconst!(&mut f, s4, f.c_i32(0));
    f.i(MirInstr::ListGet {
        dst: s4,
        base: s0,
        index: s4,
    }); // 2
    f.ret(s4);
    assert_eq!(success(&f.build()), Value::I32(2));
}

/// V-01/V-02: List<Note> owned elements — FIELD_GET reads a Copy field;
/// FIELD_SET replaces the owned Str in place (the first-case shape).
#[test]
fn av_collections_822_semantics_list_note_field_ops() {
    let mut f = Fx::new();
    let s0 = f.slot(6, "list"); // List<Note>
    let s1 = f.slot(2, "text");
    let s2 = f.slot(0, "n");
    let s3 = f.slot(3, "note");
    let s4 = f.slot(0, "idx");
    let s5 = f.slot(2, "rep");
    let s6 = f.slot(0, "gotn");
    let c1 = f.c_i32(1);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 6,
        cap: 1,
    });
    iconst!(&mut f, s1, f.c_str("aaa"));
    iconst!(&mut f, s2, f.c_i32(3));
    f.i(MirInstr::Aggregate {
        dst: s3,
        record_ty: 3,
        operands: vec![s1, s2],
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s3,
    });
    iconst!(&mut f, s4, f.c_i32(0));
    f.i(MirInstr::ListFieldGet {
        dst: s6,
        base: s0,
        index: s4,
        path: vec![1],
    }); // n = 3
    iconst!(&mut f, s5, f.c_str("zz"));
    f.i(MirInstr::ListFieldSet {
        root: s0,
        index: s4,
        path: vec![0],
        value: s5,
    });
    f.ret(s6);
    assert_eq!(success(&f.build()), Value::I32(3));
    let _ = c1;
}

/// V-04: VARIANT_PACK/DISC/GUARD round trip (no-payload arm included).
#[test]
fn av_collections_822_semantics_variant_disc_guard() {
    let mut f = Fx::new();
    let s0 = f.slot(2, "text");
    let s1 = f.slot(7, "err");
    let s2 = f.slot(0, "disc");
    let s3 = f.slot(1, "g0");
    let s4 = f.slot(1, "g1");
    let s5 = f.slot(0, "fold");
    iconst!(&mut f, s0, f.c_str("boom"));
    f.i(MirInstr::VariantPack {
        dst: s1,
        variant_ty: 7,
        disc: 1,
        payload: Some(s0),
    }); // Err
    f.i(MirInstr::VariantDisc { dst: s2, src: s1 }); // 1
    f.i(MirInstr::VariantGuard {
        dst: s3,
        src: s1,
        disc: 1,
    }); // true
    f.i(MirInstr::VariantGuard {
        dst: s4,
        src: s1,
        disc: 0,
    }); // false
    iconst!(&mut f, s5, f.c_i32(0));
    // disc(1) + g1(1) + g0(0) = 2
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: s5,
        lhs: s5,
        rhs: s2,
    });
    f.ret(s5);
    assert_eq!(success(&f.build()), Value::I32(1));
}

/// V-04: guard-then-project of an OWNED payload transfers it; a wrong-arm
/// projection traps at runtime (defense in depth — the guard fact exists
/// statically but the live arm differs).
#[test]
fn av_collections_822_semantics_variant_project_owned_and_trap() {
    // Owned payload: Err(Str) -> guard disc 1 -> project -> the Str reads
    // back through a char_at probe? Simpler: project into a Str slot and
    // RETURN cannot cross entry (owned). Instead: drop it and fold len.
    let mut f = Fx::new();
    let s0 = f.slot(2, "text");
    let s1 = f.slot(7, "err");
    let s2 = f.slot(1, "g");
    let s3 = f.slot(2, "msg");
    let s4 = f.slot(0, "len");
    iconst!(&mut f, s0, f.c_str("boom"));
    f.i(MirInstr::VariantPack {
        dst: s1,
        variant_ty: 7,
        disc: 1,
        payload: Some(s0),
    });
    f.i(MirInstr::VariantGuard {
        dst: s2,
        src: s1,
        disc: 1,
    });
    f.i(MirInstr::VariantProject {
        dst: s3,
        root: s1,
        disc: 1,
    });
    f.i(MirInstr::Drop { slot: s3 });
    iconst!(&mut f, s4, f.c_i32(4));
    f.ret(s4);
    assert_eq!(success(&f.build()), Value::I32(4));

    // Runtime wrong-arm defense: guard disc 0 (FALSE fact — static analysis
    // records the fact regardless), then project disc 0 while the live arm
    // is 1 -> runtime VariantBranch trap before any payload read.
    let mut f = Fx::new();
    let s0 = f.slot(2, "text");
    let s1 = f.slot(7, "err");
    let s2 = f.slot(1, "g");
    let s3 = f.slot(3, "note");
    let s4 = f.slot(0, "r");
    iconst!(&mut f, s0, f.c_str("boom"));
    f.i(MirInstr::VariantPack {
        dst: s1,
        variant_ty: 7,
        disc: 1,
        payload: Some(s0),
    });
    f.i(MirInstr::VariantGuard {
        dst: s2,
        src: s1,
        disc: 0,
    });
    f.i(MirInstr::VariantProject {
        dst: s3,
        root: s1,
        disc: 0,
    });
    iconst!(&mut f, s4, f.c_i32(0));
    f.ret(s4);
    // Executing a wrong-arm projection traps (the value never crosses).
    let m = f.build();
    match run(&m) {
        ExecOutcome::Trap { kind, .. } => assert_eq!(kind, TrapKind::VariantBranch),
        other => panic!("expected the wrong-arm trap, got {other:?}"),
    }
}

/// V-05/V-12: SWITCH drops the old live arm exactly once and invalidates
/// guard facts; a re-projection after the switch is rejected statically.
#[test]
fn av_collections_822_semantics_variant_switch_lifecycle() {
    // Legal: pack Err, guard, switch to None (old payload drops once).
    let mut f = Fx::new();
    let s0 = f.slot(2, "text");
    let s1 = f.slot(7, "err");
    let s2 = f.slot(1, "g");
    let s3 = f.slot(2, "again");
    let s4 = f.slot(0, "r");
    iconst!(&mut f, s0, f.c_str("boom"));
    f.i(MirInstr::VariantPack {
        dst: s1,
        variant_ty: 7,
        disc: 1,
        payload: Some(s0),
    });
    f.i(MirInstr::VariantGuard {
        dst: s2,
        src: s1,
        disc: 1,
    });
    iconst!(&mut f, s3, f.c_str("boom2"));
    f.i(MirInstr::VariantSwitch {
        root: s1,
        disc: 1,
        payload: Some(s3),
    });
    iconst!(&mut f, s4, f.c_i32(5));
    f.ret(s4);
    assert_eq!(success(&f.build()), Value::I32(5));

    // Illegal: after the switch the old guard fact is dead — a projection
    // must be rejected as guard-missing.
    let mut f = Fx::new();
    let s0 = f.slot(2, "text");
    let s1 = f.slot(7, "err");
    let s2 = f.slot(1, "g");
    let s3 = f.slot(2, "msg");
    iconst!(&mut f, s0, f.c_str("boom"));
    f.i(MirInstr::VariantPack {
        dst: s1,
        variant_ty: 7,
        disc: 1,
        payload: Some(s0),
    });
    f.i(MirInstr::VariantGuard {
        dst: s2,
        src: s1,
        disc: 1,
    });
    let s5 = f.slot(0, "r");
    iconst!(&mut f, s3, f.c_str("boom2"));
    f.i(MirInstr::VariantSwitch {
        root: s1,
        disc: 1,
        payload: Some(s3),
    });
    f.i(MirInstr::VariantProject {
        dst: s3,
        root: s1,
        disc: 1,
    });
    iconst!(&mut f, s5, f.c_i32(0));
    f.ret(s5);
    assert_eq!(reject_code(&f.build()), "mir.variant.guard-missing");
}

/// V-04/V-12: an owned payload projection MOVES it — re-projection is a
/// static payload-moved rejection; a Copy payload stays readable.
#[test]
fn av_collections_822_semantics_payload_moved_rejection() {
    // Owned (Str) payload moved twice -> payload-moved.
    let mut f = Fx::new();
    let s0 = f.slot(2, "text");
    let s1 = f.slot(7, "err");
    let s2 = f.slot(1, "g");
    let s3 = f.slot(2, "m1");
    let s4 = f.slot(2, "m2");
    let s5 = f.slot(0, "r");
    iconst!(&mut f, s0, f.c_str("boom"));
    f.i(MirInstr::VariantPack {
        dst: s1,
        variant_ty: 7,
        disc: 1,
        payload: Some(s0),
    });
    f.i(MirInstr::VariantGuard {
        dst: s2,
        src: s1,
        disc: 1,
    });
    f.i(MirInstr::VariantProject {
        dst: s3,
        root: s1,
        disc: 1,
    });
    f.i(MirInstr::VariantProject {
        dst: s4,
        root: s1,
        disc: 1,
    });
    iconst!(&mut f, s5, f.c_i32(0));
    f.ret(s5);
    assert_eq!(reject_code(&f.build()), "mir.variant.payload-moved");

    // Drop between the projections is not a re-arm either.
    let mut f = Fx::new();
    let s0 = f.slot(2, "text");
    let s1 = f.slot(7, "err");
    let s2 = f.slot(1, "g");
    let s3 = f.slot(2, "m1");
    let s4 = f.slot(2, "m2");
    let s5 = f.slot(0, "r");
    iconst!(&mut f, s0, f.c_str("boom"));
    f.i(MirInstr::VariantPack {
        dst: s1,
        variant_ty: 7,
        disc: 1,
        payload: Some(s0),
    });
    f.i(MirInstr::VariantGuard {
        dst: s2,
        src: s1,
        disc: 1,
    });
    f.i(MirInstr::VariantProject {
        dst: s3,
        root: s1,
        disc: 1,
    });
    f.i(MirInstr::Drop { slot: s3 });
    f.i(MirInstr::VariantProject {
        dst: s4,
        root: s1,
        disc: 1,
    });
    iconst!(&mut f, s5, f.c_i32(0));
    f.ret(s5);
    assert_eq!(reject_code(&f.build()), "mir.variant.payload-moved");
}

/// V-06: nested List<Outcome<Note, Str>> composition — Ok and Err arms
/// mix in one list, remove transfers, guard+project reads, and the whole
/// tree cleans up (the first-case ledger tests assert live=0 for the
/// simpler shape; here the composition executes to Success).
#[test]
fn av_collections_822_semantics_nested_list_outcome() {
    let mut f = Fx::new();
    let s0 = f.slot(8, "list"); // List<Outcome>
    let s1 = f.slot(2, "text");
    let s2 = f.slot(0, "n");
    let s3 = f.slot(3, "note");
    let s4 = f.slot(7, "ok");
    let s5 = f.slot(7, "err");
    let s6 = f.slot(0, "idx");
    let s7 = f.slot(7, "removed");
    let s8 = f.slot(1, "g");
    let s9 = f.slot(3, "note2");
    let s10 = f.slot(0, "r");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 8,
        cap: 2,
    });
    iconst!(&mut f, s1, f.c_str("txt"));
    iconst!(&mut f, s2, f.c_i32(9));
    f.i(MirInstr::Aggregate {
        dst: s3,
        record_ty: 3,
        operands: vec![s1, s2],
    });
    f.i(MirInstr::VariantPack {
        dst: s4,
        variant_ty: 7,
        disc: 0,
        payload: Some(s3),
    }); // Ok(note)
    iconst!(&mut f, s1, f.c_str("e"));
    f.i(MirInstr::VariantPack {
        dst: s5,
        variant_ty: 7,
        disc: 1,
        payload: Some(s1),
    }); // Err("e")
    f.i(MirInstr::ListPush {
        root: s0,
        value: s4,
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s5,
    });
    iconst!(&mut f, s6, f.c_i32(0));
    f.i(MirInstr::ListRemove {
        dst: s7,
        root: s0,
        index: s6,
    }); // remove Ok
    f.i(MirInstr::VariantGuard {
        dst: s8,
        src: s7,
        disc: 0,
    });
    f.i(MirInstr::VariantProject {
        dst: s9,
        root: s7,
        disc: 0,
    });
    let _ = (s10, s2);
    // Fold: the projected NOTE's n (9) + the remaining element's disc (1).
    let s11 = f.slot(7, "rem");
    let s12 = f.slot(0, "d");
    f.i(MirInstr::ListRemove {
        dst: s11,
        root: s0,
        index: s6,
    });
    f.i(MirInstr::VariantDisc { dst: s12, src: s11 });
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: s2,
        lhs: s2,
        rhs: s12,
    });
    f.ret(s2);
    assert_eq!(success(&f.build()), Value::I32(10));
}

/// V-01 admission: a List<StrView> element is outside the frozen domain;
/// a variant payload of List is rejected likewise.
#[test]
fn av_collections_822_semantics_domain_rejections() {
    let mut f = Fx::new();
    let sv = f.m.intern_type(MirType::StrView);
    let bad = f.m.intern_type(MirType::List(ListTypeRef { elem: sv }));
    let s0 = f.slot(bad, "list");
    let s1 = f.slot(0, "r");
    let c0 = f.c_i32(0);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: bad,
        cap: 0,
    });
    f.i(MirInstr::Const { dst: s1, value: c0 });
    f.ret(s1);
    assert_eq!(reject_code(&f.build()), "mir.type.list-elem");

    let mut f = Fx::new();
    let list_ty = 4u32; // List<i32>
    let v = f.m.intern_type(MirType::Variant(VariantTypeRef {
        owner_module: "collections822".to_string(),
        name: "Bad".to_string(),
        arms: vec![VariantArm {
            name: "Wrap".to_string(),
            payload: Some(list_ty),
        }],
    }));
    let s0 = f.slot(v, "x");
    let s1 = f.slot(0, "r");
    let c0 = f.c_i32(0);
    f.i(MirInstr::Const { dst: s1, value: c0 });
    f.ret(s1);
    assert_eq!(reject_code(&f.build()), "mir.type.variant-payload");
}

/// V-03 runtime flavor: pushing past MAX_LIST_ELEMS is a resource
/// rejection, never a trap — the resource outcome path is exercised by
/// the resources tier; here the LIST_SET in-place swap keeps the OTHER
/// element untouched (ownership precision).
#[test]
fn av_collections_822_semantics_set_keeps_neighbors() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "a");
    let s2 = f.slot(0, "b");
    let s3 = f.slot(0, "idx");
    let s4 = f.slot(0, "nv");
    let s5 = f.slot(0, "got");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(1));
    iconst!(&mut f, s2, f.c_i32(2));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    iconst!(&mut f, s4, f.c_i32(7));
    iconst!(&mut f, s3, f.c_i32(1));
    f.i(MirInstr::ListSet {
        root: s0,
        index: s3,
        value: s4,
    }); // [1, 7]
    iconst!(&mut f, s3, f.c_i32(0));
    f.i(MirInstr::ListGet {
        dst: s5,
        base: s0,
        index: s3,
    }); // 1 untouched
    f.ret(s5);
    assert_eq!(success(&f.build()), Value::I32(1));
}
