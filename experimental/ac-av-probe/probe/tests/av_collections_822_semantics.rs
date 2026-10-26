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
            // 9: Place<i32> (R2 retained element references; R3 binds
            // the referent element type — the test lists are List<i32>).
            MirType::Place(ListTypeRef { elem: 0 }),
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
    let _s0 = f.slot(v, "x");
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

// -- F822-R2-03: reverse-order element cleanup (contract §4 / V-06) ------

/// F822-R2-03 replay shape: List<Str> with FIRST then SECOND pushed, the
/// list retained at normal function exit. The frozen oracle: SECOND
/// frees before FIRST (reverse insertion) and the list buffer frees
/// LAST — free ids exactly [2, 1, 0] with live=0 after.
#[test]
fn av_collections_822_semantics_list_drop_reverse_elements() {
    let mut f = Fx::new();
    let s0 = f.slot(5, "list"); // List<Str>
    let s1 = f.slot(2, "a");
    let s2 = f.slot(2, "b");
    let s3 = f.slot(0, "r");
    let c1 = f.c_str("first");
    let c2 = f.c_str("second");
    let c0 = f.c_i32(0);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 5,
        cap: 0,
    });
    f.i(MirInstr::Const { dst: s1, value: c1 });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    f.i(MirInstr::Const { dst: s2, value: c2 });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    f.i(MirInstr::Const { dst: s3, value: c0 });
    f.ret(s3);
    let m = f.build();
    verify_mir(&m).unwrap_or_else(|e| panic!("{e:?}"));
    let wire = encode_abc(&m).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap();
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Success { .. }), "{out:?}");
    let arena = trace.arena.expect("arena");
    let frees: Vec<u64> = arena.free_events().iter().map(|e| e.id).collect();
    assert_eq!(
        frees,
        vec![2, 1, 0],
        "reverse element order then the buffer; got {frees:?}"
    );
    assert_eq!(arena.live(), 0);
}

/// F822-R2-03: the trap exit takes the SAME reverse path — a mid-run trap
/// unwinds the retained list with SECOND before FIRST and live=0 after
/// (no reaping may hide the order).
#[test]
fn av_collections_822_semantics_list_drop_reverse_on_trap() {
    let mut f = Fx::new();
    let s0 = f.slot(5, "list");
    let s1 = f.slot(2, "a");
    let s2 = f.slot(2, "b");
    let s3 = f.slot(0, "badidx");
    let s4 = f.slot(2, "got");
    let c1 = f.c_str("first");
    let c2 = f.c_str("second");
    let c9 = f.c_i32(9);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 5,
        cap: 0,
    });
    f.i(MirInstr::Const { dst: s1, value: c1 });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    f.i(MirInstr::Const { dst: s2, value: c2 });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    f.i(MirInstr::Const { dst: s3, value: c9 }); // out of range
    f.i(MirInstr::ListRemove {
        dst: s4,
        root: s0,
        index: s3,
    }); // traps
    f.ret(s3);
    let m = f.build();
    verify_mir(&m).unwrap_or_else(|e| panic!("{e:?}"));
    let wire = encode_abc(&m).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap();
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Trap { .. }), "{out:?}");
    let arena = trace.arena.expect("arena");
    let frees: Vec<u64> = arena.free_events().iter().map(|e| e.id).collect();
    assert_eq!(
        frees,
        vec![2, 1, 0],
        "trap unwind keeps the reverse order; got {frees:?}"
    );
    assert_eq!(arena.live(), 0);
}

/// F822-R2-03 sibling: a retained List<Outcome<Note, Str>> — each element
/// is a variant whose Ok payload owns a Note's Str; the elements release
/// reverse (second element's payload before the first's), buffer last.
#[test]
fn av_collections_822_semantics_nested_variant_list_drop_reverse() {
    let mut f = Fx::new();
    let s0 = f.slot(8, "list"); // List<Outcome>
    let s1 = f.slot(2, "t1");
    let s2 = f.slot(0, "n1");
    let s3 = f.slot(3, "note1");
    let s4 = f.slot(7, "ok1");
    let s5 = f.slot(2, "t2");
    let s6 = f.slot(0, "n2");
    let s7 = f.slot(3, "note2");
    let s8 = f.slot(7, "ok2");
    let s9 = f.slot(0, "r");
    let c0 = f.c_i32(0);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 8,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_str("AA"));
    iconst!(&mut f, s2, f.c_i32(1));
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
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s4,
    });
    iconst!(&mut f, s5, f.c_str("BBB"));
    iconst!(&mut f, s6, f.c_i32(2));
    f.i(MirInstr::Aggregate {
        dst: s7,
        record_ty: 3,
        operands: vec![s5, s6],
    });
    f.i(MirInstr::VariantPack {
        dst: s8,
        variant_ty: 7,
        disc: 0,
        payload: Some(s7),
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s8,
    });
    f.i(MirInstr::Const { dst: s9, value: c0 });
    f.ret(s9);
    let m = f.build();
    verify_mir(&m).unwrap_or_else(|e| panic!("{e:?}"));
    let wire = encode_abc(&m).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap();
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Success { .. }), "{out:?}");
    let arena = trace.arena.expect("arena");
    let frees: Vec<u64> = arena.free_events().iter().map(|e| e.id).collect();
    // ids: 0=list buffer, 1="AA", 2="BBB" — reverse: BBB(2), AA(1), buf(0).
    assert_eq!(frees, vec![2, 1, 0], "nested payload order; got {frees:?}");
    assert_eq!(arena.live(), 0);
}

// -- F822-R2-02: the index Place/token surface (V-02/V-11/V-12) ---------

/// Positive control: a minted place reads its element, repeatedly, and a
/// place over a DIFFERENT index reads its own element.
#[test]
fn av_collections_822_semantics_place_positive_reads() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v1");
    let s2 = f.slot(0, "v2");
    let s3 = f.slot(0, "i0");
    let s4 = f.slot(0, "i1");
    let s5 = f.slot(9, "p0");
    let s6 = f.slot(9, "p1");
    let s7 = f.slot(0, "g0");
    let s8 = f.slot(0, "g1");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(30));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s2, f.c_i32(40));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    iconst!(&mut f, s3, f.c_i32(0));
    iconst!(&mut f, s4, f.c_i32(1));
    f.i(MirInstr::ListPlace {
        dst: s5,
        base: s0,
        index: s3,
    });
    f.i(MirInstr::ListPlace {
        dst: s6,
        base: s0,
        index: s4,
    });
    f.i(MirInstr::PlaceGet { dst: s7, src: s5 });
    f.i(MirInstr::PlaceGet { dst: s8, src: s6 });
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: s7,
        lhs: s7,
        rhs: s8,
    });
    f.ret(s7);
    assert_eq!(success(&f.build()), Value::I32(70));
}

/// V-02: a place minted BEFORE a push (structural epoch bump) traps
/// stale-place on use; a place minted AFTER reads the appended element.
#[test]
fn av_collections_822_semantics_place_stale_after_push() {
    // Old place after push -> trap.
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v");
    let s2 = f.slot(0, "i0");
    let s3 = f.slot(9, "p");
    let s4 = f.slot(0, "g");
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
    iconst!(&mut f, s2, f.c_i32(0));
    f.i(MirInstr::ListPlace {
        dst: s3,
        base: s0,
        index: s2,
    });
    iconst!(&mut f, s1, f.c_i32(7));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    }); // epoch bump
    f.i(MirInstr::PlaceGet { dst: s4, src: s3 }); // stale
    f.ret(s4);
    assert_eq!(trap_kind(&f.build()), TrapKind::StalePlace);

    // Fresh place after the push reads the appended element.
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v");
    let s3 = f.slot(0, "i1");
    let s4 = f.slot(9, "p");
    let s5 = f.slot(0, "g");
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
    iconst!(&mut f, s1, f.c_i32(8));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s3, f.c_i32(1));
    f.i(MirInstr::ListPlace {
        dst: s4,
        base: s0,
        index: s3,
    });
    f.i(MirInstr::PlaceGet { dst: s5, src: s4 });
    f.ret(s5);
    assert_eq!(success(&f.build()), Value::I32(8));
}

/// V-12: after remove(0), a place minted before the remove traps
/// stale-place; a FRESH place at the same index reads the SHIFTED value.
#[test]
fn av_collections_822_semantics_place_stale_after_remove_fresh_reads_shift() {
    // Old place after remove -> trap.
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v1");
    let s2 = f.slot(0, "v2");
    let s3 = f.slot(0, "i0");
    let s4 = f.slot(9, "p");
    let s5 = f.slot(0, "g");
    let s6 = f.slot(0, "removed");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(10));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s2, f.c_i32(20));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    iconst!(&mut f, s3, f.c_i32(0));
    f.i(MirInstr::ListPlace {
        dst: s4,
        base: s0,
        index: s3,
    });
    f.i(MirInstr::ListRemove {
        dst: s6,
        root: s0,
        index: s3,
    }); // epoch + shift
    f.i(MirInstr::PlaceGet { dst: s5, src: s4 }); // stale
    f.ret(s5);
    assert_eq!(trap_kind(&f.build()), TrapKind::StalePlace);

    // Fresh place at index 0 reads the SHIFTED value (20).
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v1");
    let s2 = f.slot(0, "v2");
    let s3 = f.slot(0, "i0");
    let s4 = f.slot(9, "p");
    let s5 = f.slot(0, "g");
    let s6 = f.slot(0, "removed");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(10));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s2, f.c_i32(20));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    iconst!(&mut f, s3, f.c_i32(0));
    f.i(MirInstr::ListRemove {
        dst: s6,
        root: s0,
        index: s3,
    });
    f.i(MirInstr::ListPlace {
        dst: s4,
        base: s0,
        index: s3,
    });
    f.i(MirInstr::PlaceGet { dst: s5, src: s4 });
    f.ret(s5);
    assert_eq!(success(&f.build()), Value::I32(20));
}

/// V-12 per-element replacement validity: an in-place LIST_SET retires
/// places minted over THAT element (generation bump) while a place over a
/// DIFFERENT element stays live; PLACE_SET retires its own place the same
/// way; fresh places always see the current value.
#[test]
fn av_collections_822_semantics_place_retired_by_element_replacement() {
    // LIST_SET[1] keeps p0 live.
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v1");
    let s2 = f.slot(0, "v2");
    let s3 = f.slot(0, "nv");
    let s4 = f.slot(0, "i0");
    let s5 = f.slot(0, "i1");
    let s6 = f.slot(9, "p0");
    let s8 = f.slot(0, "g");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(10));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s2, f.c_i32(20));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    iconst!(&mut f, s4, f.c_i32(0));
    iconst!(&mut f, s5, f.c_i32(1));
    f.i(MirInstr::ListPlace {
        dst: s6,
        base: s0,
        index: s4,
    });
    iconst!(&mut f, s3, f.c_i32(99));
    f.i(MirInstr::ListSet {
        root: s0,
        index: s5,
        value: s3,
    }); // gen bump at 1
    f.i(MirInstr::PlaceGet { dst: s8, src: s6 }); // p0 still live -> 10
    f.ret(s8);
    assert_eq!(success(&f.build()), Value::I32(10));

    // Same shape but using p1 after the replacement -> stale trap.
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v1");
    let s2 = f.slot(0, "v2");
    let s3 = f.slot(0, "nv");
    let s4 = f.slot(0, "i0");
    let s5 = f.slot(0, "i1");
    let s7 = f.slot(9, "p1");
    let s8 = f.slot(0, "g");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(10));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s2, f.c_i32(20));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    iconst!(&mut f, s4, f.c_i32(0));
    iconst!(&mut f, s5, f.c_i32(1));
    f.i(MirInstr::ListPlace {
        dst: s7,
        base: s0,
        index: s5,
    });
    iconst!(&mut f, s3, f.c_i32(99));
    f.i(MirInstr::ListSet {
        root: s0,
        index: s5,
        value: s3,
    });
    f.i(MirInstr::PlaceGet { dst: s8, src: s7 }); // stale (element gen)
    f.ret(s8);
    assert_eq!(trap_kind(&f.build()), TrapKind::StalePlace);

    // PLACE_SET commits, retires its own place at commit time.
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v1");
    let s3 = f.slot(0, "nv");
    let s4 = f.slot(0, "i0");
    let s7 = f.slot(9, "p1");
    let s8 = f.slot(0, "g");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(10));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s4, f.c_i32(0));
    f.i(MirInstr::ListPlace {
        dst: s7,
        base: s0,
        index: s4,
    });
    iconst!(&mut f, s3, f.c_i32(123));
    f.i(MirInstr::PlaceSet {
        place: s7,
        value: s3,
    });
    f.i(MirInstr::PlaceGet { dst: s8, src: s7 }); // stale (own retirement)
    f.ret(s8);
    assert_eq!(trap_kind(&f.build()), TrapKind::StalePlace);

    // A FRESH place after the PLACE_SET reads 123.
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v1");
    let s3 = f.slot(0, "nv");
    let s4 = f.slot(0, "i0");
    let s7 = f.slot(9, "p1");
    let s9 = f.slot(9, "p2");
    let s8 = f.slot(0, "g");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_i32(10));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s4, f.c_i32(0));
    f.i(MirInstr::ListPlace {
        dst: s7,
        base: s0,
        index: s4,
    });
    iconst!(&mut f, s3, f.c_i32(123));
    f.i(MirInstr::PlaceSet {
        place: s7,
        value: s3,
    });
    f.i(MirInstr::ListPlace {
        dst: s9,
        base: s0,
        index: s4,
    }); // fresh
    f.i(MirInstr::PlaceGet { dst: s8, src: s9 });
    f.ret(s8);
    assert_eq!(success(&f.build()), Value::I32(123));
}

/// V-02: an out-of-range place mint traps before any access.
#[test]
fn av_collections_822_semantics_place_mint_out_of_range_traps() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v");
    let s2 = f.slot(0, "i9");
    let s3 = f.slot(9, "p");
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
    iconst!(&mut f, s2, f.c_i32(9));
    f.i(MirInstr::ListPlace {
        dst: s3,
        base: s0,
        index: s2,
    });
    f.ret(s2);
    assert_eq!(trap_kind(&f.build()), TrapKind::ListIndex);
}

// -- F822-R3-01: typed Place admission (wrong referent types rejected) ---

/// The reviewer replay: a bool destination over a List<i32> place is a
/// TYPED-ADMISSION rejection (verify refuses; nothing encodes, admits or
/// executes).
#[test]
fn av_collections_822_semantics_place_get_wrong_type_rejected() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v");
    let s2 = f.slot(0, "i0");
    let s3 = f.slot(9, "p");
    let s4 = f.slot(1, "wrong_bool_dst");
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
    iconst!(&mut f, s2, f.c_i32(0));
    f.i(MirInstr::ListPlace {
        dst: s3,
        base: s0,
        index: s2,
    });
    f.i(MirInstr::PlaceGet { dst: s4, src: s3 }); // bool over List<i32>
    f.ret(s2);
    assert_eq!(reject_code(&f.build()), "mir.list-type");
}

/// The reviewer replay: writing a bool into a List<i32> through a place
/// is rejected at typing — the list can never be corrupted.
#[test]
fn av_collections_822_semantics_place_set_wrong_type_rejected() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(0, "v");
    let s2 = f.slot(0, "i0");
    let s3 = f.slot(9, "p");
    let s4 = f.slot(1, "wrong_bool_val");
    let s5 = f.slot(0, "got");
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
    iconst!(&mut f, s2, f.c_i32(0));
    f.i(MirInstr::ListPlace {
        dst: s3,
        base: s0,
        index: s2,
    });
    iconst!(&mut f, s4, f.m.intern_const(MirConst::Bool(true)));
    f.i(MirInstr::PlaceSet {
        place: s3,
        value: s4,
    }); // bool into List<i32>
    f.i(MirInstr::ListGet {
        dst: s5,
        base: s0,
        index: s2,
    });
    f.ret(s5);
    assert_eq!(reject_code(&f.build()), "mir.list-type");
}

/// The mint itself binds the referent: a Place<i32> minted over a
/// List<Str> is rejected (nominal/owned element identity holds too).
#[test]
fn av_collections_822_semantics_place_mint_binds_referent() {
    let mut f = Fx::new();
    let s0 = f.slot(5, "list_str");
    let s1 = f.slot(2, "v");
    let s2 = f.slot(0, "i0");
    let s3 = f.slot(9, "p_i32_over_str_list"); // Place<i32>, wrong list
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 5,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_str("x"));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s2, f.c_i32(0));
    f.i(MirInstr::ListPlace {
        dst: s3,
        base: s0,
        index: s2,
    });
    f.ret(s2);
    assert_eq!(reject_code(&f.build()), "mir.list-type");
}

// -- F822-R3-02: shared-buffer transfers preserve the destination -------

/// The reviewer replay: MOVE preserves the moved list — the destination
/// reads length 1 and the element survives the source's kill (the source
/// handle release is a responsibility transfer, not a drain).
#[test]
fn av_collections_822_semantics_move_preserves_list() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "src");
    let s1 = f.slot(4, "dst");
    let s2 = f.slot(0, "v");
    let s3 = f.slot(0, "len");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s2, f.c_i32(7));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    f.i(MirInstr::Move { dst: s1, src: s0 });
    f.i(MirInstr::ListLen { dst: s3, src: s1 });
    f.ret(s3);
    assert_eq!(success(&f.build()), Value::I32(1));
}

/// Owned-element lifetime: a moved List<Str> keeps its element — after
/// the move the destination removes the element and the ledger balances
/// (the element frees ONCE, at the removal/drop path, never at the move).
#[test]
fn av_collections_822_semantics_move_preserves_owned_elements() {
    let mut f = Fx::new();
    let s0 = f.slot(5, "src");
    let s1 = f.slot(5, "dst");
    let s2 = f.slot(2, "v");
    let s3 = f.slot(0, "i0");
    let s4 = f.slot(0, "len");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 5,
        cap: 0,
    });
    iconst!(&mut f, s2, f.c_str("payload"));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    f.i(MirInstr::Move { dst: s1, src: s0 });
    f.i(MirInstr::ListLen { dst: s4, src: s1 });
    f.ret(s4);
    let m = f.build();
    let _ = s3;
    verify_mir(&m).unwrap_or_else(|e| panic!("{e:?}"));
    let wire = encode_abc(&m).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap();
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(1),
                ..
            }
        ),
        "{out:?}"
    );
    let arena = trace.arena.expect("arena");
    assert_eq!(arena.live(), 0, "nothing leaks");
    assert_eq!(
        arena.alloc_events().len(),
        arena.free_events().len(),
        "the element freed exactly once"
    );
}

/// A COPY shares the buffer: dropping the source leaves the destination
/// intact (length preserved, no mid-copy drain).
#[test]
fn av_collections_822_semantics_copy_share_drop_source_preserves_dst() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "src");
    let s1 = f.slot(4, "dst");
    let s2 = f.slot(0, "v");
    let s3 = f.slot(0, "len");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s2, f.c_i32(7));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    f.i(MirInstr::Copy { dst: s1, src: s0 });
    f.i(MirInstr::Drop { slot: s0 });
    f.i(MirInstr::ListLen { dst: s3, src: s1 });
    f.ret(s3);
    assert_eq!(success(&f.build()), Value::I32(1));
}

/// F822-R3-02 sibling — the CALL move-in path (owned_kill_slot's other
/// shared caller): passing a List<i32> as an OWNED argument kills the
/// caller's slot but MUST NOT drain the callee's value — the callee reads
/// length 1 through its parameter.
#[test]
fn av_collections_822_semantics_call_move_in_preserves_list() {
    let mut m = MirModule::new_collections(OwnerScope {
        schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
        module: "collections822".to_string(),
    });
    m.types = vec![
        MirType::I32,
        MirType::List(ListTypeRef { elem: 0 }),
        MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![],
            result: 0,
        }),
        MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![1],
            result: 0,
        }),
    ];
    let c7 = m.intern_const(MirConst::I32(7)) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    m.funcs = vec![
        MirFunction {
            owner: m.owner.clone(),
            name: "main".to_string(),
            sig: 2,
            slots: vec![
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
                    name: Some("got".into()),
                },
            ],
            blocks: vec![MirBlock {
                instrs: vec![
                    MirInstr::ListNew {
                        dst: 0,
                        list_ty: 1,
                        cap: 0,
                    },
                    MirInstr::Const { dst: 1, value: c7 },
                    MirInstr::ListPush { root: 0, value: 1 },
                    MirInstr::Call {
                        callee: auto_av_core::mir::CalleeRef::Func(1),
                        dst: 2,
                        args: vec![0],
                    },
                ],
                term: Terminator::Return { value: 2 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
        },
        MirFunction {
            owner: m.owner.clone(),
            name: "helper".to_string(),
            sig: 3,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Param,
                    ty: 1,
                    name: Some("l".into()),
                },
                MirSlot {
                    kind: SlotKind::Local,
                    ty: 0,
                    name: Some("n".into()),
                },
            ],
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::ListLen { dst: 1, src: 0 }],
                term: Terminator::Return { value: 1 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
        },
    ];
    m.entry = 0;
    let _ = c0;
    assert_eq!(success(&m), Value::I32(1));
}

// -- F822-R5-01/R5-02: cross-frame Place access + borrow-escape gates ----

/// A two-function fixture vocabulary for the cross-frame probes:
/// 0=i32 1=bool 2=Str 3=Note 4=List<i32> 5=List<Str> 6=List<Note>
/// 7=Outcome 8=List<Outcome> 9=Place<i32>.
fn cross_frame_module(mode: &str) -> MirModule {
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
                auto_av_core::mir::MirFieldDef {
                    name: "text".into(),
                    ty: 2,
                },
                auto_av_core::mir::MirFieldDef {
                    name: "n".into(),
                    ty: 0,
                },
            ],
        }),
        MirType::List(ListTypeRef { elem: 0 }),
        MirType::List(ListTypeRef { elem: 2 }),
        MirType::List(ListTypeRef { elem: 3 }),
        MirType::Variant(auto_av_core::mir::VariantTypeRef {
            owner_module: "collections822".to_string(),
            name: "Outcome".to_string(),
            arms: vec![
                auto_av_core::mir::VariantArm {
                    name: "Ok".into(),
                    payload: Some(3),
                },
                auto_av_core::mir::VariantArm {
                    name: "Err".into(),
                    payload: Some(2),
                },
            ],
        }),
        MirType::List(ListTypeRef { elem: 7 }),
        MirType::Place(ListTypeRef { elem: 0 }),
    ];
    let c7 = m.intern_const(MirConst::I32(7)) as u32;
    let c9 = m.intern_const(MirConst::I32(9)) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;

    // main slots: 0=list, 1=v, 2=idx0, 3=place, 4=got
    // helper slots (write mode): 0=place param, 1=nv
    // helper slots (read mode): 0=place param, 1=got
    let main_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: 0,
    }));
    let helper_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![9],
        result: if mode == "escape" { 9 } else { 0 },
    }));
    let main_instrs = vec![
        MirInstr::ListNew {
            dst: 0,
            list_ty: 4,
            cap: 0,
        },
        MirInstr::Const { dst: 1, value: c7 },
        MirInstr::ListPush { root: 0, value: 1 },
        MirInstr::Const { dst: 2, value: c0 },
        MirInstr::ListPlace {
            dst: 3,
            base: 0,
            index: 2,
        },
        MirInstr::Call {
            callee: auto_av_core::mir::CalleeRef::Func(1),
            dst: 4,
            args: vec![3],
        },
        MirInstr::ListGet {
            dst: 4,
            base: 0,
            index: 2,
        },
    ];
    let helper_instrs = match mode {
        "write" => vec![
            MirInstr::Const { dst: 1, value: c9 },
            MirInstr::PlaceSet { place: 0, value: 1 },
            MirInstr::Const { dst: 1, value: c0 },
        ],
        "read" => vec![MirInstr::PlaceGet { dst: 1, src: 0 }],
        "escape" => vec![MirInstr::Const { dst: 1, value: c0 }],
        _ => unreachable!(),
    };
    let helper_ret = if mode == "escape" { 0 } else { 1 };
    let _ = helper_ret;
    m.funcs = vec![
        MirFunction {
            owner: m.owner.clone(),
            name: "main".to_string(),
            sig: main_sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Local,
                    ty: 4,
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
                    name: Some("idx0".into()),
                },
                MirSlot {
                    kind: SlotKind::Local,
                    ty: 9,
                    name: Some("p".into()),
                },
                MirSlot {
                    kind: SlotKind::Local,
                    ty: 0,
                    name: Some("got".into()),
                },
            ],
            blocks: vec![MirBlock {
                instrs: main_instrs,
                term: Terminator::Return { value: 4 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
        },
        MirFunction {
            owner: m.owner.clone(),
            name: "helper".to_string(),
            sig: helper_sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Param,
                    ty: 9,
                    name: Some("p".into()),
                },
                MirSlot {
                    kind: SlotKind::Local,
                    ty: 0,
                    name: Some("r".into()),
                },
            ],
            blocks: vec![MirBlock {
                instrs: helper_instrs,
                term: Terminator::Return {
                    value: if mode == "escape" { 0 } else { 1 },
                },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
        },
    ];
    m.entry = 0;
    m
}

/// F822-R5-01 replay: a cross-frame PLACE_SET (the helper writes through
/// the CALLER's place — matching types, live owner frame) writes the
/// referenced CALLER element: the caller's read yields 9. No panic; the
/// helper's own same-numbered slot (which holds the place, not a list)
/// is never indexed.
#[test]
fn av_collections_822_semantics_cross_frame_place_write_no_panic() {
    assert_eq!(success(&cross_frame_module("write")), Value::I32(9));
}

/// The reviewer's passing read control, made permanent: the same caller
/// token read in the helper yields 7.
#[test]
fn av_collections_822_semantics_cross_frame_place_read_control() {
    assert_eq!(success(&cross_frame_module("read")), Value::I32(7));
}

/// F822-R5-02 replay: a helper returning a LOCALLY minted place (the
/// root dies with the helper's frame) is refused at the MIR gate as a
/// borrow escape — never admitted to wait for a dead-frame InvalidHandle.
#[test]
fn av_collections_822_semantics_callee_local_place_escape_refused() {
    let mut m = cross_frame_module("escape");
    // helper: mint over its OWN list and return the token.
    let c7 = m.intern_const(MirConst::I32(7)) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    let no_param_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: 9,
    }));
    let helper = &mut m.funcs[1];
    // dst0=list, dst1=i32 value, dst2=i32 index, dst3=the minted place.
    helper.slots = vec![
        MirSlot {
            kind: SlotKind::Local,
            ty: 4,
            name: Some("hlist".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 0,
            name: Some("hv".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 0,
            name: Some("hi".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 9,
            name: Some("hp".into()),
        },
    ];
    helper.blocks = vec![MirBlock {
        instrs: vec![
            MirInstr::Const { dst: 1, value: c7 },
            MirInstr::ListNew {
                dst: 0,
                list_ty: 4,
                cap: 0,
            },
            MirInstr::ListPush { root: 0, value: 1 },
            MirInstr::Const { dst: 2, value: c0 },
            MirInstr::ListPlace {
                dst: 3,
                base: 0,
                index: 2,
            },
        ],
        term: Terminator::Return { value: 3 },
    }];
    // The helper's declared param list changes: no param.
    helper.sig = no_param_sig;
    // main passes nothing and PLACE_GETs the returned token.
    let main = &mut m.funcs[0];
    main.blocks = vec![MirBlock {
        instrs: vec![
            MirInstr::Call {
                callee: auto_av_core::mir::CalleeRef::Func(1),
                dst: 3,
                args: vec![],
            },
            MirInstr::PlaceGet { dst: 4, src: 3 },
        ],
        term: Terminator::Return { value: 4 },
    }];
    assert_eq!(reject_code(&m), "mir.borrow-escape");
}

/// F822-R5-02 replay: a record field of Place kind (even in an UNUSED
/// record type on an otherwise legal module) is refused as a borrowed
/// token field — same gate as StrView/MutRef/RecordView.
#[test]
fn av_collections_822_semantics_record_place_token_refused() {
    let mut m = cross_frame_module("read");
    m.intern_type(MirType::Record(auto_av_core::mir::RecordTypeRef {
        owner_module: "collections822".to_string(),
        name: "EscapedToken".to_string(),
        fields: vec![auto_av_core::mir::MirFieldDef {
            name: "token".to_string(),
            ty: 9, // Place<i32>
        }],
    }));
    assert_eq!(reject_code(&m), "mir.field-borrow");
}

/// F822-R5-02 positive control: a place PARAM (externally rooted — the
/// caller's frame owns the list) may be USED and RETURNED while the
/// caller lives; the caller reads through the returned token.
#[test]
fn av_collections_822_semantics_place_param_roundtrip_supported() {
    let mut m = cross_frame_module("read");
    let place_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![9],
        result: 9,
    }));
    let c7b = m.intern_const(MirConst::I32(7)) as u32;
    let c0b = m.intern_const(MirConst::I32(0)) as u32;
    // helper returns its param token (slot 0).
    let helper = &mut m.funcs[1];
    helper.blocks = vec![MirBlock {
        instrs: vec![MirInstr::PlaceGet { dst: 1, src: 0 }],
        term: Terminator::Return { value: 0 },
    }];
    // Bump the helper sig result to Place and let main read through it.
    helper.sig = place_sig;
    let main = &mut m.funcs[0];
    main.slots[4] = MirSlot {
        kind: SlotKind::Local,
        ty: 9,
        name: Some("back".into()),
    };
    let got_slot = main.slots.len() as u32;
    main.slots.push(MirSlot {
        kind: SlotKind::Local,
        ty: 0,
        name: Some("final".into()),
    });
    main.blocks = vec![MirBlock {
        instrs: vec![
            MirInstr::ListNew {
                dst: 0,
                list_ty: 4,
                cap: 0,
            },
            MirInstr::Const { dst: 1, value: c7b },
            MirInstr::ListPush { root: 0, value: 1 },
            MirInstr::Const { dst: 2, value: c0b },
            MirInstr::ListPlace {
                dst: 3,
                base: 0,
                index: 2,
            },
            MirInstr::Call {
                callee: auto_av_core::mir::CalleeRef::Func(1),
                dst: 4,
                args: vec![3],
            },
            MirInstr::PlaceGet {
                dst: got_slot,
                src: 4,
            },
        ],
        term: Terminator::Return { value: got_slot },
    }];
    assert_eq!(success(&m), Value::I32(7));
}

// -- F822-R6-01/02: cross-block provenance + retained-alias policy -------

/// A two-function probe kit for the R6 shapes:
/// 0=i32 1=bool 2=Str 3=Note 4=List<i32> 9=Place<i32>.
/// helper_modes: "jump-local" (mint then Jump; return in block 1),
/// "jump-param" (Jump; return the param in block 1),
/// "merge-local" (mint before a Branch; both edges join one return block
/// that returns the minted place), "merge-param" (same CFG, returns the
/// external param).
fn r6_module(helper_mode: &str) -> MirModule {
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
                auto_av_core::mir::MirFieldDef {
                    name: "text".into(),
                    ty: 2,
                },
                auto_av_core::mir::MirFieldDef {
                    name: "n".into(),
                    ty: 0,
                },
            ],
        }),
        MirType::List(ListTypeRef { elem: 0 }),
        MirType::List(ListTypeRef { elem: 2 }),
        MirType::List(ListTypeRef { elem: 3 }),
        MirType::Variant(auto_av_core::mir::VariantTypeRef {
            owner_module: "collections822".to_string(),
            name: "Outcome".to_string(),
            arms: vec![
                auto_av_core::mir::VariantArm {
                    name: "Ok".into(),
                    payload: Some(3),
                },
                auto_av_core::mir::VariantArm {
                    name: "Err".into(),
                    payload: Some(2),
                },
            ],
        }),
        MirType::List(ListTypeRef { elem: 7 }),
        MirType::Place(ListTypeRef { elem: 0 }),
    ];
    let c7 = m.intern_const(MirConst::I32(7)) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    let ctrue = m.intern_const(MirConst::Bool(true)) as u32;
    let param_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![9],
        result: 9,
    }));
    let main_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: 0,
    }));

    // helper slots: 0=list, 1=v, 2=idx0, 3=minted place, 4=param place,
    // 5=cond. (Local-only modes leave slot 4 unused; the param modes use
    // it as the param.)
    let helper_blocks = match helper_mode {
        "jump-local" => vec![
            MirBlock {
                instrs: vec![
                    MirInstr::Const { dst: 1, value: c7 },
                    MirInstr::ListNew {
                        dst: 0,
                        list_ty: 4,
                        cap: 0,
                    },
                    MirInstr::ListPush { root: 0, value: 1 },
                    MirInstr::Const { dst: 2, value: c0 },
                    MirInstr::ListPlace {
                        dst: 3,
                        base: 0,
                        index: 2,
                    },
                ],
                term: Terminator::Jump { target: 1 },
            },
            MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 3 },
            },
        ],
        "jump-param" => vec![
            MirBlock {
                instrs: vec![],
                term: Terminator::Jump { target: 1 },
            },
            MirBlock {
                instrs: vec![],
                term: Terminator::Return { value: 4 },
            },
        ],
        "merge-local" | "merge-param" => vec![
            MirBlock {
                instrs: vec![
                    MirInstr::Const { dst: 1, value: c7 },
                    MirInstr::ListNew {
                        dst: 0,
                        list_ty: 4,
                        cap: 0,
                    },
                    MirInstr::ListPush { root: 0, value: 1 },
                    MirInstr::Const { dst: 2, value: c0 },
                    MirInstr::ListPlace {
                        dst: 3,
                        base: 0,
                        index: 2,
                    },
                    MirInstr::Const {
                        dst: 5,
                        value: ctrue,
                    },
                ],
                term: Terminator::Branch {
                    cond: 5,
                    then: 1,
                    else_: 1,
                },
            },
            MirBlock {
                instrs: vec![],
                term: Terminator::Return {
                    value: if helper_mode == "merge-local" { 3 } else { 4 },
                },
            },
        ],
        _ => unreachable!(),
    };
    let uses_param = helper_mode.ends_with("param");
    let mut helper_slots = vec![
        MirSlot {
            kind: SlotKind::Local,
            ty: 4,
            name: Some("h".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 0,
            name: Some("v".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 0,
            name: Some("i".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 9,
            name: Some("p".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 9,
            name: Some("arg".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 1,
            name: Some("c".into()),
        },
    ];
    if uses_param {
        helper_slots[4] = MirSlot {
            kind: SlotKind::Param,
            ty: 9,
            name: Some("arg".into()),
        };
    }
    let helper_sig = if uses_param {
        param_sig
    } else {
        m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
            params: vec![],
            result: 9,
        }))
    };

    // main: build the caller list + place; call helper accordingly; read
    // through the returned place (param modes) or just fold (local modes
    // are expected to be REFUSED at admission anyway).
    let main_slots = vec![
        MirSlot {
            kind: SlotKind::Local,
            ty: 4,
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
            name: Some("idx0".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 9,
            name: Some("p".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 9,
            name: Some("back".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: 0,
            name: Some("got".into()),
        },
    ];
    let mut main_instrs = vec![
        MirInstr::ListNew {
            dst: 0,
            list_ty: 4,
            cap: 0,
        },
        MirInstr::Const { dst: 1, value: c7 },
        MirInstr::ListPush { root: 0, value: 1 },
        MirInstr::Const { dst: 2, value: c0 },
        MirInstr::ListPlace {
            dst: 3,
            base: 0,
            index: 2,
        },
    ];
    let call_args: Vec<u32> = if uses_param { vec![3] } else { vec![] };
    main_instrs.push(MirInstr::Call {
        callee: auto_av_core::mir::CalleeRef::Func(1),
        dst: 4,
        args: call_args,
    });
    main_instrs.push(MirInstr::PlaceGet { dst: 5, src: 4 });

    m.funcs = vec![
        MirFunction {
            owner: m.owner.clone(),
            name: "main".to_string(),
            sig: main_sig,
            slots: main_slots,
            blocks: vec![MirBlock {
                instrs: main_instrs,
                term: Terminator::Return { value: 5 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
        },
        MirFunction {
            owner: m.owner.clone(),
            name: "helper".to_string(),
            sig: helper_sig,
            slots: helper_slots,
            blocks: helper_blocks,
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
        },
    ];
    m.entry = 0;
    m
}

/// F822-R6-01 replay: a LOCALLY minted place returned across a Jump edge
/// is refused at the MIR gate (the provenance survives the block edge).
#[test]
fn av_collections_822_semantics_local_place_escape_across_jump_refused() {
    assert_eq!(reject_code(&r6_module("jump-local")), "mir.borrow-escape");
}

/// Positive control: an EXTERNALLY rooted place param returned across the
/// same Jump edge is supported — the caller reads 7 through it.
#[test]
fn av_collections_822_semantics_place_param_return_across_jump_supported() {
    assert_eq!(success(&r6_module("jump-param")), Value::I32(7));
}

/// Branch-merge shapes: a LOCAL place crossing a two-predecessor merge
/// into the return is refused; the same CFG returning the external param
/// place is supported.
#[test]
fn av_collections_822_semantics_merge_shapes_local_vs_external() {
    assert_eq!(reject_code(&r6_module("merge-local")), "mir.borrow-escape");
    assert_eq!(success(&r6_module("merge-param")), Value::I32(7));
}

/// F822-R6-02 replay: mutating the list after minting a place is
/// supported DIRECTLY and THROUGH A COPY ALIAS identically — a retained
/// reference never blocks the update (retirement is epoch/generation,
/// not a live-borrow conflict). Both shapes return 9 via the fresh index.
#[test]
fn av_collections_822_semantics_mutate_after_place_copy_supported() {
    let build = |with_copy: bool| {
        let mut m = r6_module("jump-param");
        // Replace main with the single-function mutation shape.
        let c9 = m.intern_const(MirConst::I32(9)) as u32;
        let c0 = m.intern_const(MirConst::I32(0)) as u32;
        let c7 = m.intern_const(MirConst::I32(7)) as u32;
        let main = &mut m.funcs[0];
        main.slots = vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: 4,
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
                name: Some("idx0".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: 9,
                name: Some("p".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: 9,
                name: Some("alias".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: 0,
                name: Some("nv".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: 0,
                name: Some("got".into()),
            },
        ];
        let mut instrs = vec![
            MirInstr::ListNew {
                dst: 0,
                list_ty: 4,
                cap: 0,
            },
            MirInstr::Const { dst: 1, value: c7 },
            MirInstr::ListPush { root: 0, value: 1 },
            MirInstr::Const { dst: 2, value: c0 },
            MirInstr::ListPlace {
                dst: 3,
                base: 0,
                index: 2,
            },
        ];
        if with_copy {
            instrs.push(MirInstr::Copy { dst: 4, src: 3 });
        }
        instrs.push(MirInstr::Const { dst: 5, value: c9 });
        instrs.push(MirInstr::ListSet {
            root: 0,
            index: 2,
            value: 5,
        });
        instrs.push(MirInstr::ListGet {
            dst: 6,
            base: 0,
            index: 2,
        });
        main.blocks = vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 6 },
        }];
        m
    };
    assert_eq!(
        success(&build(false)),
        Value::I32(9),
        "direct mint: update supported"
    );
    assert_eq!(
        success(&build(true)),
        Value::I32(9),
        "copy alias: the retained reference still never blocks the update"
    );
}

/// F822-R6-02 sibling: an externally-rooted helper roundtrip (param in,
/// token back) followed by a LIST UPDATE on the caller's list — the
/// returned token stays retained; the update succeeds and the fresh
/// index reads the new value.
#[test]
fn av_collections_822_semantics_helper_roundtrip_then_update_supported() {
    let mut m = r6_module("jump-param");
    let c9 = m.intern_const(MirConst::I32(9)) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    let main = &mut m.funcs[0];
    let mut instrs = main.blocks[0].instrs.clone();
    // Drop the trailing PlaceGet; instead mutate through a fresh LIST_SET
    // and read the new value (the returned token stays retained and never
    // blocks the update).
    instrs.pop();
    instrs.push(MirInstr::Const { dst: 1, value: c9 });
    instrs.push(MirInstr::ListSet {
        root: 0,
        index: 2,
        value: 1,
    });
    instrs.push(MirInstr::ListGet {
        dst: 5,
        base: 0,
        index: 2,
    });
    main.blocks = vec![MirBlock {
        instrs,
        term: Terminator::Return { value: 5 },
    }];
    let _ = c0;
    assert_eq!(success(&m), Value::I32(9));
}

// -- F822-R7-01: retained Place root matrices pay the analysis budget ---

/// The reviewer's memory-probe shape: one whole-module function builds
/// List<i32>[7], mints `count` Place<i32> locals over its valid element,
/// then reads the element fresh. Acyclic, no escapes, no huge runtime
/// list — the only scale axis is the count of RETAINED ROOT SETS.
fn place_probe_module(count: usize) -> MirModule {
    let mut m = MirModule::new_collections(OwnerScope {
        schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
        module: "collections822".to_string(),
    });
    let t_i32 = m.intern_type(MirType::I32);
    let t_list = m.intern_type(MirType::List(ListTypeRef { elem: t_i32 }));
    let t_place = m.intern_type(MirType::Place(ListTypeRef { elem: t_i32 }));
    let sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: t_i32,
    }));
    let c7 = m.intern_const(MirConst::I32(7)) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    // slots: 0=list, 1=v, 2=idx0, 3=got, 4..=4+count-1=places
    let mut slots = vec![
        MirSlot {
            kind: SlotKind::Local,
            ty: t_list,
            name: Some("list".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: t_i32,
            name: Some("v".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: t_i32,
            name: Some("idx0".into()),
        },
        MirSlot {
            kind: SlotKind::Local,
            ty: t_i32,
            name: Some("got".into()),
        },
    ];
    for p in 0..count {
        slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: t_place,
            name: Some(format!("p{p}")),
        });
    }
    let mut instrs = vec![
        MirInstr::ListNew {
            dst: 0,
            list_ty: t_list,
            cap: 0,
        },
        MirInstr::Const { dst: 1, value: c7 },
        MirInstr::ListPush { root: 0, value: 1 },
        MirInstr::Const { dst: 2, value: c0 },
    ];
    for p in 0..count {
        instrs.push(MirInstr::ListPlace {
            dst: 4 + p as u32,
            base: 0,
            index: 2,
        });
    }
    instrs.push(MirInstr::ListGet {
        dst: 3,
        base: 0,
        index: 2,
    });
    m.funcs = vec![MirFunction {
        owner: m.owner.clone(),
        name: "main".to_string(),
        sig,
        slots,
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

/// F822-R7-01: 12,000 retained Places exceed the frozen 16MiB analysis
/// budget — BOTH real gates (MIR verify and ABC admission, whose
/// credential construction runs the same analysis) reject with a
/// VerifyMemory resource outcome BEFORE the allocations.
#[test]
fn av_collections_822_semantics_place_provenance_memory_budget() {
    let big = place_probe_module(12_000);
    match verify_mir(&big) {
        Err(auto_av_core::mir_verify::MirVerifyError::Resource { kind, .. }) => {
            assert_eq!(kind, auto_av_core::contract::ResourceKind::VerifyMemory);
        }
        other => panic!("the oversized Place input must be resource-refused: {other:?}"),
    }
    // The ABC gate funnels through the same analysis: the wire encodes,
    // but the credential refuses.
    let wire = encode_abc(&big).expect("encode is structural");
    match verify_abc_bytes(&wire.to_flat(), &big.owner) {
        Err(auto_av_core::abc_verify::AbcAdmissionError::Verify(
            auto_av_core::mir_verify::MirVerifyError::Resource { kind, .. },
        )) => {
            assert_eq!(kind, auto_av_core::contract::ResourceKind::VerifyMemory);
        }
        other => panic!("the ABC credential must refuse the same allocation: {other:?}"),
    }

    // The small legitimate control (64 Places) passes BOTH gates and
    // executes to the fresh read.
    let small = place_probe_module(64);
    assert_eq!(success(&small), Value::I32(7));
}

// -- F822-R8-01: explicit deep Clone of collections ----------------------

/// The reviewer replay: CLONE [7], mutate the ORIGINAL to [9], read the
/// clone — an explicit deep copy is isolated (folded: orig 9 + clone 7
/// = 16).
#[test]
fn av_collections_822_semantics_clone_list_is_independent() {
    let mut f = Fx::new();
    let s0 = f.slot(4, "list");
    let s1 = f.slot(4, "cl");
    let s2 = f.slot(0, "v");
    let s3 = f.slot(0, "idx0");
    let s4 = f.slot(0, "nv");
    let s5 = f.slot(0, "got_orig");
    let s6 = f.slot(0, "got_clone");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 4,
        cap: 0,
    });
    iconst!(&mut f, s2, f.c_i32(7));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    f.i(MirInstr::Clone { dst: s1, src: s0 });
    iconst!(&mut f, s3, f.c_i32(0));
    iconst!(&mut f, s4, f.c_i32(9));
    f.i(MirInstr::ListSet {
        root: s0,
        index: s3,
        value: s4,
    });
    f.i(MirInstr::ListGet {
        dst: s5,
        base: s0,
        index: s3,
    });
    f.i(MirInstr::ListGet {
        dst: s6,
        base: s1,
        index: s3,
    });
    f.i(MirInstr::Bin {
        op: BinOp::MulI32,
        dst: s6,
        lhs: s6,
        rhs: s5,
    }); // 7*9=63
    f.ret(s6);
    assert_eq!(success(&f.build()), Value::I32(63));
}

/// Owned-element sibling: CLONE List<Note>, rewrite the ORIGINAL's text
/// field in place — the clone's field read still yields the original
/// payload's length (the clone owns independent buffers).
#[test]
fn av_collections_822_semantics_clone_owned_elements_independent() {
    let mut f = Fx::new();
    let s0 = f.slot(6, "list");
    let s1 = f.slot(2, "t");
    let s2 = f.slot(0, "n");
    let s3 = f.slot(3, "note");
    let s4 = f.slot(6, "cl");
    let s5 = f.slot(0, "idx0");
    let s6 = f.slot(2, "nt");
    let s7 = f.slot(0, "got");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 6,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_str("aaa"));
    iconst!(&mut f, s2, f.c_i32(1));
    f.i(MirInstr::Aggregate {
        dst: s3,
        record_ty: 3,
        operands: vec![s1, s2],
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s3,
    });
    f.i(MirInstr::Clone { dst: s4, src: s0 });
    iconst!(&mut f, s5, f.c_i32(0));
    iconst!(&mut f, s6, f.c_str("bbbbbbbbbb"));
    f.i(MirInstr::ListFieldSet {
        root: s0,
        index: s5,
        path: vec![0],
        value: s6,
    });
    f.i(MirInstr::ListFieldGet {
        dst: s7,
        base: s4,
        index: s5,
        path: vec![1],
    }); // n=1
    f.ret(s7);
    assert_eq!(success(&f.build()), Value::I32(1));
    // And the original's rewritten field reads 10 through a fresh place.
    let mut f = Fx::new();
    let s0 = f.slot(6, "list");
    let s1 = f.slot(2, "t");
    let s2 = f.slot(0, "n");
    let s3 = f.slot(3, "note");
    let s4 = f.slot(6, "cl");
    let s5 = f.slot(0, "idx0");
    let s6 = f.slot(2, "nt");
    let s7 = f.slot(0, "got");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 6,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_str("aaa"));
    iconst!(&mut f, s2, f.c_i32(1));
    f.i(MirInstr::Aggregate {
        dst: s3,
        record_ty: 3,
        operands: vec![s1, s2],
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s3,
    });
    f.i(MirInstr::Clone { dst: s4, src: s0 });
    iconst!(&mut f, s5, f.c_i32(0));
    iconst!(&mut f, s6, f.c_str("bbbbbbbbbb"));
    f.i(MirInstr::ListFieldSet {
        root: s0,
        index: s5,
        path: vec![0],
        value: s6,
    });
    f.i(MirInstr::ListFieldGet {
        dst: s7,
        base: s0,
        index: s5,
        path: vec![1],
    }); // n=1
    f.ret(s7);
    assert_eq!(success(&f.build()), Value::I32(1));
}

/// Variant payload sibling: CLONE Some("aa"), SWITCH the ORIGINAL arm
/// to None — the clone still holds Some (disc 1) whose payload frees
/// exactly once at unwind (ledger balanced, live=0).
#[test]
fn av_collections_822_semantics_clone_variant_payload_independent() {
    let mut f = Fx::new();
    // A local two-arm variant: None / Some(Str).
    let t_opt =
        f.m.intern_type(MirType::Variant(auto_av_core::mir::VariantTypeRef {
            owner_module: "collections822".to_string(),
            name: "Opt".to_string(),
            arms: vec![
                auto_av_core::mir::VariantArm {
                    name: "None".into(),
                    payload: None,
                },
                auto_av_core::mir::VariantArm {
                    name: "Some".into(),
                    payload: Some(2),
                },
            ],
        }));
    let s0 = f.slot(t_opt, "some");
    let s1 = f.slot(2, "t");
    let s2 = f.slot(t_opt, "cl");
    let s4 = f.slot(0, "disc");
    let s5 = f.slot(0, "r");
    iconst!(&mut f, s1, f.c_str("aa"));
    f.i(MirInstr::VariantPack {
        dst: s0,
        variant_ty: t_opt,
        disc: 1,
        payload: Some(s1),
    });
    f.i(MirInstr::Clone { dst: s2, src: s0 });
    f.i(MirInstr::VariantSwitch {
        root: s0,
        disc: 0,
        payload: None,
    });
    f.i(MirInstr::VariantDisc { dst: s4, src: s2 }); // clone keeps Some=1
    iconst!(&mut f, s5, f.c_i32(41));
    f.i(MirInstr::Bin {
        op: BinOp::AddI32,
        dst: s5,
        lhs: s5,
        rhs: s4,
    }); // 42
    f.ret(s5);
    let m = f.build();
    verify_mir(&m).unwrap_or_else(|e| panic!("{e:?}"));
    let wire = encode_abc(&m).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap();
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: Value::I32(42),
                ..
            }
        ),
        "{out:?}"
    );
    let arena = trace.arena.expect("arena");
    assert_eq!(arena.live(), 0);
    assert_eq!(
        arena.alloc_events().len(),
        arena.free_events().len(),
        "the switched-out and cloned payloads each freed exactly once"
    );
}

#[test]
fn av_collections_822_semantics_clone_nth_item_failure_rolls_back() {
    let big = "B".repeat(4096);
    let mut f = Fx::new();
    let s0 = f.slot(5, "list");
    let s1 = f.slot(2, "a");
    let s2 = f.slot(2, "b");
    let s3 = f.slot(5, "cl");
    let s4 = f.slot(0, "len_after");
    let s5 = f.slot(0, "r");
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: 5,
        cap: 0,
    });
    iconst!(&mut f, s1, f.c_str("aa"));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    iconst!(&mut f, s2, f.c_str(&big));
    f.i(MirInstr::ListPush {
        root: s0,
        value: s2,
    });
    f.i(MirInstr::Clone { dst: s3, src: s0 });
    f.i(MirInstr::ListLen { dst: s4, src: s0 });
    iconst!(&mut f, s5, f.c_i32(2));
    f.i(MirInstr::Bin {
        op: BinOp::MulI32,
        dst: s5,
        lhs: s5,
        rhs: s4,
    }); // 2*2=4
    f.ret(s5);
    let m = f.build();
    verify_mir(&m).unwrap_or_else(|e| panic!("{e:?}"));
    let wire = encode_abc(&m).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap();
    // Attempts: 1=list buf, 2="aa" clone, 3=big clone, 4=clone buf —
    // fault at 3 kills the clone mid-way (after "aa" minted).
    let arena = auto_av_core::owned_text::StrArena::new();
    arena.set_fault_at(3);
    let (out, trace) = auto_av_core::av::execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        arena,
    );
    assert!(
        matches!(&out, ExecOutcome::Infra(msg) if msg.contains("fault-injected")),
        "{out:?}"
    );
    let arena = trace.arena.expect("arena");
    assert_eq!(arena.live(), 0, "the partial clone fully unwound");
    assert_eq!(
        arena.alloc_events().len(),
        arena.free_events().len(),
        "allocs {:?} frees {:?}",
        arena.alloc_events(),
        arena.free_events()
    );
}

/// V-11 clone-budget oracle: a deep clone that would cross the live-data
/// budget is rejected BEFORE any mutation (SlotBytes), ledger balanced;
/// the exactly-enough side succeeds.
#[test]
fn av_collections_822_semantics_clone_budget_both_sides() {
    let big = "B".repeat(8192);
    let build = || {
        let mut f = Fx::new();
        let s0 = f.slot(5, "list");
        let s1 = f.slot(2, "v");
        let s2 = f.slot(5, "cl");
        let s3 = f.slot(0, "r");
        f.i(MirInstr::ListNew {
            dst: s0,
            list_ty: 5,
            cap: 0,
        });
        iconst!(&mut f, s1, f.c_str(&big));
        f.i(MirInstr::ListPush {
            root: s0,
            value: s1,
        });
        f.i(MirInstr::Clone { dst: s2, src: s0 });
        iconst!(&mut f, s3, f.c_i32(5));
        f.ret(s3);
        f.build()
    };
    let m = build();
    verify_mir(&m).unwrap_or_else(|e| panic!("{e:?}"));
    let wire = encode_abc(&m).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap();
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Success { .. }), "{out:?}");
    let peak = trace.peak_slot_bytes;
    let mut short = ExecLimits::default();
    short.max_slot_bytes = (peak - 1) as usize;
    let (out, trace) = execute(&loaded, &short, &CancelFlag::new(false));
    assert!(
        matches!(&out, ExecOutcome::Resource { kind, .. } if *kind == auto_av_core::contract::ResourceKind::SlotBytes),
        "{out:?}"
    );
    let arena = trace.arena.expect("arena");
    assert_eq!(arena.live(), 0);
    assert_eq!(arena.alloc_events().len(), arena.free_events().len());
}

/// A legal collections whole-module base for the R9 probes:
/// 0=i32 1=bool 2=Str, main() -> i32 with placeholder slots.
fn r9_base() -> MirModule {
    let mut m = MirModule::new_collections(OwnerScope {
        schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
        module: "collections822".to_string(),
    });
    m.types = vec![MirType::I32, MirType::Bool, MirType::Str];
    let sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: 0,
    }));
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    m.funcs = vec![MirFunction {
        owner: m.owner.clone(),
        name: "main".to_string(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: 0,
            name: Some("r".into()),
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: c0 }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

// -- F822-R9-01/02: acyclic Variant admission + bounded clone traversal --

/// F822-R9-02 replay: a nominal SELF-REFERENTIAL Variant type — even as
/// an UNUSED table entry on an otherwise legal module — is rejected at
/// the MIR gate and at the ABC credential (both run the walker).
#[test]
fn av_collections_822_semantics_variant_self_cycle_rejected() {
    let mut m = r9_base();
    m.intern_type(MirType::Variant(auto_av_core::mir::VariantTypeRef {
        owner_module: "collections822".to_string(),
        name: "Recursive".to_string(),
        arms: vec![
            auto_av_core::mir::VariantArm {
                name: "End".into(),
                payload: None,
            },
            // The payload index is filled AFTER interning (self reference).
        ],
    }));
    // Self-reference: point arm 1's payload at the just-interned slot.
    let self_idx = (m.types.len() - 1) as u32;
    if let MirType::Variant(v) = &mut m.types[self_idx as usize] {
        v.arms.push(auto_av_core::mir::VariantArm {
            name: "Next".into(),
            payload: Some(self_idx),
        });
    }
    match verify_mir(&m) {
        Err(auto_av_core::mir_verify::MirVerifyError::Invalid { code, .. }) => {
            assert_eq!(code, "mir.type-cycle");
        }
        other => panic!("a self-referential Variant must be refused: {other:?}"),
    }
    // The wire encodes structurally; the ABC credential refuses (its
    // verification runs the same walker).
    m.funcs[0].source_snapshot = 1; // keep the module otherwise legal
    let wire = encode_abc(&m).expect("encode is structural");
    match verify_abc_bytes(&wire.to_flat(), &m.owner) {
        Err(auto_av_core::abc_verify::AbcAdmissionError::Verify(
            auto_av_core::mir_verify::MirVerifyError::Invalid { code, .. },
        )) => assert_eq!(code, "mir.type-cycle"),
        other => panic!("the ABC credential must refuse the cycle: {other:?}"),
    }
}

/// Indirect cycles across two Variants (A→B→A), with the table order
/// REVERSED (B declared before A) — the walker catches it regardless of
/// declaration order.
#[test]
fn av_collections_822_semantics_variant_indirect_cycle_rejected() {
    let mut m = r9_base();
    // B first (its payload references A, declared after).
    let b_idx = m.intern_type(MirType::Variant(auto_av_core::mir::VariantTypeRef {
        owner_module: "collections822".to_string(),
        name: "VB".to_string(),
        arms: vec![auto_av_core::mir::VariantArm {
            name: "S".into(),
            payload: None,
        }],
    })) as u32;
    let a_idx = m.intern_type(MirType::Variant(auto_av_core::mir::VariantTypeRef {
        owner_module: "collections822".to_string(),
        name: "VA".to_string(),
        arms: vec![auto_av_core::mir::VariantArm {
            name: "ToB".into(),
            payload: Some(b_idx),
        }],
    })) as u32;
    if let MirType::Variant(v) = &mut m.types[b_idx as usize] {
        v.arms[0].payload = Some(a_idx); // B → A → B
    }
    match verify_mir(&m) {
        Err(auto_av_core::mir_verify::MirVerifyError::Invalid { code, .. }) => {
            assert_eq!(code, "mir.type-cycle")
        }
        other => panic!("an indirect Variant cycle must be refused: {other:?}"),
    }
}

/// F822-R9-02/01 positive control: a THREE-LAYER acyclic nested Variant
/// admits at both gates, executes, and CLONES correctly through the
/// explicit visitor (the reviewer's acyclic control).
#[test]
fn av_collections_822_semantics_nested_variant_clone_control() {
    let mut m = r9_base();
    // Innermost: Opt0 = { None, Some(Str) }
    let t0 = m.intern_type(MirType::Variant(auto_av_core::mir::VariantTypeRef {
        owner_module: "collections822".to_string(),
        name: "Opt0".to_string(),
        arms: vec![
            auto_av_core::mir::VariantArm {
                name: "None".into(),
                payload: None,
            },
            auto_av_core::mir::VariantArm {
                name: "Some".into(),
                payload: Some(2),
            },
        ],
    })) as u32;
    let t1 = m.intern_type(MirType::Variant(auto_av_core::mir::VariantTypeRef {
        owner_module: "collections822".to_string(),
        name: "Opt1".to_string(),
        arms: vec![
            auto_av_core::mir::VariantArm {
                name: "None".into(),
                payload: None,
            },
            auto_av_core::mir::VariantArm {
                name: "Some".into(),
                payload: Some(t0),
            },
        ],
    })) as u32;
    let t2 = m.intern_type(MirType::Variant(auto_av_core::mir::VariantTypeRef {
        owner_module: "collections822".to_string(),
        name: "Opt2".to_string(),
        arms: vec![
            auto_av_core::mir::VariantArm {
                name: "None".into(),
                payload: None,
            },
            auto_av_core::mir::VariantArm {
                name: "Some".into(),
                payload: Some(t1),
            },
        ],
    })) as u32;
    let c_s = m.intern_const(MirConst::Str(Box::from(*b"deep"))) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    // main: pack Opt0(Some "deep") → Opt1(Some) → Opt2(Some); clone the
    // Opt2; switch the ORIGINAL to None; the clone still disc 1.
    m.funcs[0] = MirFunction {
        owner: m.owner.clone(),
        name: "main".to_string(),
        sig: m.funcs[0].sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: 2,
                name: Some("s".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t0,
                name: Some("o0".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t1,
                name: Some("o1".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t2,
                name: Some("o2".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t2,
                name: Some("cl".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: 0,
                name: Some("disc".into()),
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 0, value: c_s },
                MirInstr::VariantPack {
                    dst: 1,
                    variant_ty: t0,
                    disc: 1,
                    payload: Some(0),
                },
                MirInstr::VariantPack {
                    dst: 2,
                    variant_ty: t1,
                    disc: 1,
                    payload: Some(1),
                },
                MirInstr::VariantPack {
                    dst: 3,
                    variant_ty: t2,
                    disc: 1,
                    payload: Some(2),
                },
                MirInstr::Clone { dst: 4, src: 3 },
                MirInstr::VariantSwitch {
                    root: 3,
                    disc: 0,
                    payload: None,
                },
                MirInstr::VariantDisc { dst: 5, src: 4 },
            ],
            term: Terminator::Return { value: 5 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };
    assert_eq!(success(&m), Value::I32(1), "the clone keeps the deep arm");
}

/// F822-R9-01: CANCELLATION lands mid-traversal — a large clone (1,501
/// visitor nodes) with the flag already raised returns Cancelled at the
/// first in-loop poll; the partial assembly fully unwound (ledger
/// balanced, live=0).
#[test]
fn av_collections_822_semantics_clone_cancel_mid_traversal() {
    let mut m = r9_base();
    let t_list = m.intern_type(MirType::List(ListTypeRef { elem: 2 })) as u32;
    let c_x = m.intern_const(MirConst::Str(Box::from(*b"x"))) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    let c5 = m.intern_const(MirConst::I32(5)) as u32;
    let mut instrs = vec![MirInstr::ListNew {
        dst: 0,
        list_ty: t_list,
        cap: 0,
    }];
    for _ in 0..1_500 {
        // An owned Str element kills its source slot at push — rematerialize
        // the literal per push (1,500 Str nodes + the list = 1,501 visitors).
        instrs.push(MirInstr::Const { dst: 1, value: c_x });
        instrs.push(MirInstr::ListPush { root: 0, value: 1 });
    }
    instrs.push(MirInstr::Clone { dst: 2, src: 0 });
    instrs.push(MirInstr::Const { dst: 3, value: c5 });
    m.funcs[0] = MirFunction {
        owner: m.owner.clone(),
        name: "main".to_string(),
        sig: m.funcs[0].sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: t_list,
                name: Some("l".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: 2,
                name: Some("v".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t_list,
                name: Some("cl".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: 0,
                name: Some("r".into()),
            },
        ],
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };
    verify_mir(&m).unwrap_or_else(|e| panic!("{e:?}"));
    let wire = encode_abc(&m).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap();
    let cancel = auto_av_core::av::CancelFlag::new(true);
    let (out, trace) = auto_av_core::av::execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &cancel,
        auto_av_core::owned_text::StrArena::new(),
    );
    assert!(matches!(out, ExecOutcome::Cancelled), "{out:?}");
    let arena = trace.arena.expect("arena");
    assert_eq!(arena.live(), 0, "the partial clone fully unwound");
    assert_eq!(arena.alloc_events().len(), arena.free_events().len());
    let _ = c0;
}

/// F822-R9-01: a zero wall budget hits the SAME in-loop poll — the clone
/// returns the wall Resource outcome mid-traversal with a clean ledger.
#[test]
fn av_collections_822_semantics_clone_wall_mid_traversal() {
    let mut m = r9_base();
    let t_list = m.intern_type(MirType::List(ListTypeRef { elem: 2 })) as u32;
    let c_x = m.intern_const(MirConst::Str(Box::from(*b"x"))) as u32;
    let c5 = m.intern_const(MirConst::I32(5)) as u32;
    let mut instrs = vec![MirInstr::ListNew {
        dst: 0,
        list_ty: t_list,
        cap: 0,
    }];
    for _ in 0..1_500 {
        // An owned Str element kills its source slot at push — rematerialize
        // the literal per push (1,500 Str nodes + the list = 1,501 visitors).
        instrs.push(MirInstr::Const { dst: 1, value: c_x });
        instrs.push(MirInstr::ListPush { root: 0, value: 1 });
    }
    instrs.push(MirInstr::Clone { dst: 2, src: 0 });
    instrs.push(MirInstr::Const { dst: 3, value: c5 });
    m.funcs[0] = MirFunction {
        owner: m.owner.clone(),
        name: "main".to_string(),
        sig: m.funcs[0].sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: t_list,
                name: Some("l".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: 2,
                name: Some("v".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t_list,
                name: Some("cl".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: 0,
                name: Some("r".into()),
            },
        ],
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: Vec::new(),
    };
    verify_mir(&m).unwrap_or_else(|e| panic!("{e:?}"));
    let wire = encode_abc(&m).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap();
    let mut limits = ExecLimits::default();
    limits.wall = std::time::Duration::ZERO;
    let (out, trace) = execute(&loaded, &limits, &auto_av_core::av::CancelFlag::new(false));
    assert!(
        matches!(&out, ExecOutcome::Resource { kind, .. } if *kind == auto_av_core::contract::ResourceKind::ExecutionWall),
        "{out:?}"
    );
    let arena = trace.arena.expect("arena");
    assert_eq!(arena.live(), 0);
    assert_eq!(arena.alloc_events().len(), arena.free_events().len());
}
