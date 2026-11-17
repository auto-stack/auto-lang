//! PLAN-1035 (F34) typed-MIR leg: the resource classification reaches the
//! shared MIR, so a List-only record (`type Bag { items List<int> }`) is an
//! OWNED value and `CLONE` is its deep-copy channel (the source profile keeps
//! `List.clone`/aggregate builtin clone rejected — this is the frozen typed
//! entry only).
//!
//! The case builds the typed module by hand (the collections/aggregates
//! builder form the 822/962 suites use), passes the REAL gates
//! (`verify_mir` → `encode_abc` → `verify_abc_bytes` → executor), and pins:
//! * the clone allocated its OWN buffer (2 arena allocations, 2 frees);
//! * mutating the clone's inner list leaves the original untouched
//!   (value oracle 42 + 7·10 = 112);
//! * the frame ledger closes (live 0, allocs == frees).

use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::av::{execute_with_arena, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value, MIR_COLLECTIONS_SCHEMA};
use ace_av_core::mir::{
    FnSig, ListTypeRef, MirBlock, MirConst, MirFieldDef, MirFunction, MirInstr, MirModule, MirSlot,
    MirType, OwnerScope, RecordTypeRef, SlotKind, Terminator,
};
use ace_av_core::mir_verify::verify_mir;
use ace_av_core::owned_text::StrArena;

/// The aggregates admission capability a record-with-container-field module
/// carries (stamped by the aggregates source profile in the real chain).
const AGGREGATES_CAP: &str = "av.aggregates/1";

const TY_I32: u32 = 0;
const TY_LIST_I32: u32 = 1;
const TY_BAG: u32 = 2;
const TY_LIST_BAG: u32 = 3;

struct Builder {
    m: MirModule,
    slots: Vec<MirSlot>,
    instrs: Vec<MirInstr>,
}

impl Builder {
    fn new() -> Self {
        let mut m = MirModule::new_collections(OwnerScope {
            schema: MIR_COLLECTIONS_SCHEMA.to_string(),
            module: "typed1035".to_string(),
        });
        m.capabilities.push(AGGREGATES_CAP.to_string());
        m.types = vec![
            MirType::I32,
            MirType::List(ListTypeRef { elem: TY_I32 }),
            MirType::Record(RecordTypeRef {
                owner_module: "typed1035".to_string(),
                name: "t_rec_Bag".to_string(),
                fields: vec![MirFieldDef {
                    name: "items".to_string(),
                    ty: TY_LIST_I32,
                }],
            }),
            MirType::List(ListTypeRef { elem: TY_BAG }),
        ];
        Builder {
            m,
            slots: Vec::new(),
            instrs: Vec::new(),
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

    fn i(&mut self, instr: MirInstr) {
        self.instrs.push(instr);
    }

    fn const_i32(&mut self, dst: u32, value: i32) {
        let c = self.m.intern_const(MirConst::I32(value)) as u32;
        self.i(MirInstr::Const { dst, value: c });
    }

    fn build(mut self, ret: u32) -> MirModule {
        let sig = self.m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: TY_I32,
        }));
        self.m.funcs = vec![MirFunction {
            owner: self.m.owner.clone(),
            name: "main".to_string(),
            sig,
            slots: self.slots,
            blocks: vec![MirBlock {
                instrs: self.instrs,
                term: Terminator::Return { value: ret },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: Vec::new(),
            env: None,
        }];
        self.m.entry = 0;
        self.m
    }
}

fn admit(m: &MirModule) -> ace_av_core::abc_verify::LoadedModule {
    let owner = m.owner.clone();
    verify_mir(m).expect("typed Bag module verifies");
    let wire = encode_abc(m).expect("encodes");
    verify_abc_bytes(&wire.to_flat(), &owner).expect("ABC admits")
}

/// One run's observable facts: entry value plus the arena ledger.
struct Run {
    value: i32,
    live: usize,
    allocs: usize,
    frees: usize,
    free_ids: Vec<u64>,
}

fn run(m: &MirModule) -> Run {
    let loaded = admit(m);
    let arena = StrArena::new();
    let (out, trace) = execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        arena.clone(),
    );
    let value = match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected an i32 success, got {other:?}"),
    };
    Run {
        value,
        live: arena.live(),
        allocs: arena.successful_allocs(),
        frees: arena.free_events().len(),
        free_ids: arena.free_events().iter().map(|f| f.id).collect(),
    }
}

/// CLONE of a List-only record is a DEEP copy: the clone's inner list is a
/// fresh buffer (mutating it cannot be seen from the original), and both
/// owners release exactly once.
#[test]
fn audit1035_res_01_typed_clone_of_list_only_record_is_deep() {
    let mut b = Builder::new();
    let inner = b.slot(TY_LIST_I32, "inner");
    let v = b.slot(TY_I32, "v");
    let bag = b.slot(TY_BAG, "bag");
    let cl = b.slot(TY_BAG, "cl");
    let idx = b.slot(TY_I32, "idx");
    let nv = b.slot(TY_I32, "nv");
    let l_cl = b.slot(TY_LIST_I32, "l_cl");
    let l_orig = b.slot(TY_LIST_I32, "l_orig");
    let got_orig = b.slot(TY_I32, "got_orig");
    let got_clone = b.slot(TY_I32, "got_clone");

    b.i(MirInstr::ListNew {
        dst: inner,
        list_ty: TY_LIST_I32,
        cap: 0,
    });
    b.const_i32(v, 42);
    b.i(MirInstr::ListPush {
        root: inner,
        value: v,
    });
    b.i(MirInstr::Aggregate {
        dst: bag,
        record_ty: TY_BAG,
        operands: vec![inner],
    });
    // The typed deep clone (the frozen typed entry; the source profile keeps
    // aggregate clone rejected).
    b.i(MirInstr::Clone { dst: cl, src: bag });
    // Rewrite the CLONE's element only.
    b.i(MirInstr::FieldGet {
        dst: l_cl,
        base: cl,
        path: vec![0],
    });
    b.const_i32(idx, 0);
    b.const_i32(nv, 7);
    b.i(MirInstr::ListSet {
        root: l_cl,
        index: idx,
        value: nv,
    });
    // Read the ORIGINAL through its own field handle.
    b.i(MirInstr::FieldGet {
        dst: l_orig,
        base: bag,
        path: vec![0],
    });
    b.i(MirInstr::ListGet {
        dst: got_orig,
        base: l_orig,
        index: idx,
    });
    b.i(MirInstr::ListGet {
        dst: got_clone,
        base: l_cl,
        index: idx,
    });
    let ten = b.slot(TY_I32, "ten");
    b.const_i32(ten, 10);
    b.i(MirInstr::Bin {
        op: ace_av_core::mir::BinOp::MulI32,
        dst: got_clone,
        lhs: got_clone,
        rhs: ten,
    });
    b.i(MirInstr::Bin {
        op: ace_av_core::mir::BinOp::AddI32,
        dst: got_orig,
        lhs: got_orig,
        rhs: got_clone,
    });

    let m = b.build(got_orig);
    let r = run(&m);
    assert_eq!(r.value, 112, "clone holds 7, original still 42");
    assert_eq!(r.live, 0, "no buffer may leak");
    assert_eq!(r.allocs, 2, "the clone allocated its own inner buffer");
    assert_eq!(r.frees, r.allocs, "each buffer releases exactly once");
    let mut ids = r.free_ids.clone();
    ids.sort_unstable();
    let n = ids.len();
    ids.dedup();
    assert_eq!(ids.len(), n, "no buffer may be freed twice: {ids:?}");
}

/// The pure-scalar control on the same builder: such a record stays
/// copyable (no Clone channel needed, no extra buffer).
#[test]
fn audit1035_res_01_typed_scalar_record_stays_copyable() {
    use ace_av_core::layout::compute_layout;
    let mut b = Builder::new();
    b.m.types.push(MirType::Record(RecordTypeRef {
        owner_module: "typed1035".to_string(),
        name: "t_rec_Pair".to_string(),
        fields: vec![
            MirFieldDef {
                name: "a".to_string(),
                ty: TY_I32,
            },
            MirFieldDef {
                name: "b".to_string(),
                ty: TY_I32,
            },
        ],
    }));
    let pair_ty = (b.m.types.len() - 1) as u32;
    let pair = b.slot(pair_ty, "pair");
    let a = b.slot(TY_I32, "a");
    let bv = b.slot(TY_I32, "b");
    let got = b.slot(TY_I32, "got");
    b.const_i32(a, 40);
    b.const_i32(bv, 2);
    b.i(MirInstr::Aggregate {
        dst: pair,
        record_ty: pair_ty,
        operands: vec![a, bv],
    });
    b.i(MirInstr::FieldGet {
        dst: got,
        base: pair,
        path: vec![1],
    });
    let m = b.build(got);
    let layout = compute_layout(&m, pair_ty).expect("pure record lays out");
    assert!(layout.is_copy, "a scalar-only record stays copyable");
    let r = run(&m);
    assert_eq!(r.value, 2);
    assert_eq!(r.live, 0);
    assert_eq!(r.allocs, 0, "a pure value record allocates nothing");
}
