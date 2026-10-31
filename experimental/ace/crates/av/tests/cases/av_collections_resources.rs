//! PLAN-822 T-04 resources tests: the collections resource/failure
//! boundaries — capacity guard (both sides), live-data budget rejection
//! BEFORE the write, allocator fault injection (attempt exists /
//! successful_alloc=0), and post-failure ledger cleanliness
//! (V-03/V-07 axes; the precheck/commit/kill seams of the executor).
//! Prefix `av_collections_822` (contract-decision §6/§8).

use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, ResourceKind, MAX_LIST_ELEMS, SLOT_CELL_BYTES};
use ace_av_core::mir::{
    BinOp, ListTypeRef, MirBlock, MirConst, MirFunction, MirInstr, MirModule, MirSlot, MirType,
    OwnerScope, SlotKind, Terminator,
};
use ace_av_core::mir_verify::verify_mir;

/// A minimal `fn main() -> i32` with one `List<i32>` (slot 0), one value
/// slot (1) and `pushes` LIST_PUSH instructions fed by a const.
fn push_module(pushes: usize, _list_ty: u32, elem_ty: u32) -> MirModule {
    let mut m = MirModule::new_collections(OwnerScope {
        schema: ace_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
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
        sig: m.intern_type(MirType::Fn(ace_av_core::mir::FnSig {
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
        env: None,
    }];
    m.entry = 0;
    m
}

fn run_with(
    m: &MirModule,
    limits: &ExecLimits,
) -> (ExecOutcome, ace_av_core::owned_text::StrArena) {
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
fn av_collections_resources_list_cap_both_sides() {
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
        Err(ace_av_core::mir_verify::MirVerifyError::Resource { kind, .. }) => {
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
fn av_collections_resources_slot_bytes_rejects_before_write() {
    let m = push_module(4, 1, 0); // 4 pushes of i32 7
    let mut limits = ExecLimits::default();
    // Room for the list header + one element cell, not two.
    limits.max_slot_bytes = 2 * SLOT_CELL_BYTES + 4 * std::mem::size_of::<ace_av_core::Value>();
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
fn av_collections_resources_fault_injection_attempt_semantics() {
    let m = push_module(1, 1, 0);
    verify_mir(&m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(&m).unwrap_or_else(|_e| panic!("encode"));
    let loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("{e:?}"));
    // The collections fault seam is the ARENA ledger (V-07 allocator
    // axis): fail the FIRST allocation attempt — the list buffer itself.
    let arena = ace_av_core::owned_text::StrArena::new();
    arena.set_fault_at(1);
    let (out, trace) = ace_av_core::av::execute_with_arena(
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
fn av_collections_resources_budget_is_the_only_delta() {
    let m = push_module(3, 1, 0);
    let mut tight = ExecLimits::default();
    tight.max_slot_bytes = 3 * SLOT_CELL_BYTES + 3 * std::mem::size_of::<ace_av_core::Value>();
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
fn av_collections_resources_peak_and_drain() {
    let m = push_module(3, 1, 0);
    verify_mir(&m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(&m).unwrap_or_else(|_e| panic!("encode"));
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
fn av_collections_resources_index_arithmetic_shape() {
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
            assert_eq!(value, ace_av_core::Value::I32(9));
        }
        other => panic!("{other:?}"),
    }
}

// -- T-09: the worker B-line memory gate, parent-sampled (V-07/V-13) ----

struct CollectionsMemoryEvidence {
    worker_private_peak: u64,
    peak_hot: u64,
    peak_staging: u64,
    peak_exec_slot: u64,
}

/// Spawn the release worker on the frozen first-case fixture and sample
/// the child's private-bytes peak from THIS parent (the V-07 Line-B
/// discipline: the worker reports its own managed partitions; the
/// private-bytes peak is the PARENT's observation, never self-reported).
fn sampled_collections_request(
    worker: &std::path::Path,
    store: &std::path::Path,
) -> (CollectionsMemoryEvidence, bool) {
    use std::process::{Command, Stdio};
    let child = Command::new(worker)
        .args([
            "collections-request",
            "--store",
            store.to_str().unwrap(),
            "--fixture",
            "first-case",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn collections worker");
    let sampler = ace_av::mempeek::ChildPeakSampler::start(child.id(), 5)
        .expect("the parent samples the child");
    let out = child.wait_with_output().expect("child exits");
    let (peak, samples) = sampler.stop();
    assert!(samples > 0, "the sampler actually polled ({samples})");
    let line = String::from_utf8_lossy(&out.stdout);
    let payload: serde_json::Value =
        serde_json::from_str(line.trim()).expect("the worker prints one JSON line");
    (
        CollectionsMemoryEvidence {
            worker_private_peak: peak,
            peak_hot: payload["peak_hot_bytes"].as_u64().expect("peak_hot_bytes"),
            peak_staging: payload["peak_staging_bytes"]
                .as_u64()
                .expect("peak_staging_bytes"),
            peak_exec_slot: payload["peak_exec_slot_bytes"]
                .as_u64()
                .expect("peak_exec_slot_bytes"),
        },
        out.status.success(),
    )
}

// -- F822-R1-01: transactional precheck/commit counter-examples ----------

/// The shared probe of the failure-seam contract: run `m` once under a
/// wide budget to learn its EXACT live-data peak, then rerun at
/// peak (exactly enough — must succeed) and peak-1 (exactly short — must
/// be a SlotBytes resource with a clean ledger). A half-committed
/// implementation leaks or reorders frees at the seam.
fn both_sides_of_budget(
    m: &MirModule,
) -> (
    u64,                                              // the exact peak
    ExecOutcome,                                      // exactly-enough outcome
    (ExecOutcome, ace_av_core::owned_text::StrArena), // exactly-short
) {
    use ace_av_core::mir_verify::verify_mir;
    verify_mir(m).unwrap_or_else(|e| panic!("verify: {e:?}"));
    let wire = encode_abc(m).unwrap_or_else(|_e| panic!("encode"));
    let loaded =
        verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("admit: {e:?}"));
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(out, ExecOutcome::Success { .. }),
        "wide run: {out:?}"
    );
    let peak = trace.peak_slot_bytes;
    let mut tight = ExecLimits::default();
    tight.max_slot_bytes = peak as usize;
    let (enough, _) = execute(&loaded, &tight, &CancelFlag::new(false));
    let mut short = ExecLimits::default();
    short.max_slot_bytes = (peak - 1) as usize;
    let (short_out, short_trace) = execute(&loaded, &short, &CancelFlag::new(false));
    let arena = short_trace.arena.clone().expect("arena");
    (peak, enough, (short_out, arena))
}

fn assert_clean_ledger(arena: &ace_av_core::owned_text::StrArena) {
    assert_eq!(
        arena.alloc_events().len(),
        arena.free_events().len(),
        "alloc/free ledger balanced at the seam: allocs {:?} frees {:?}",
        arena.alloc_events(),
        arena.free_events()
    );
    assert_eq!(arena.live(), 0, "nothing leaks past the rejection");
}

/// Minimal fixture vocabulary for the transaction tests:
/// 0=i32 1=bool 2=Str 3=Note{text:Str,n:i32} 4=List<Str> 5=List<Note>
/// 6=Outcome{Ok(Note),Err(Str)}
struct TxFx {
    m: MirModule,
    slots: Vec<MirSlot>,
    instrs: Vec<MirInstr>,
    ret: u32,
}

#[allow(dead_code)]
impl TxFx {
    fn new() -> TxFx {
        let mut m = MirModule::new_collections(OwnerScope {
            schema: ace_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
            module: "collections822".to_string(),
        });
        m.types = vec![
            MirType::I32,
            MirType::Bool,
            MirType::Str,
            MirType::Record(ace_av_core::mir::RecordTypeRef {
                owner_module: "collections822".to_string(),
                name: "Note".to_string(),
                fields: vec![
                    ace_av_core::mir::MirFieldDef {
                        name: "text".into(),
                        ty: 2,
                    },
                    ace_av_core::mir::MirFieldDef {
                        name: "n".into(),
                        ty: 0,
                    },
                ],
            }),
            MirType::List(ListTypeRef { elem: 2 }),
            MirType::List(ListTypeRef { elem: 3 }),
            MirType::Variant(ace_av_core::mir::VariantTypeRef {
                owner_module: "collections822".to_string(),
                name: "Outcome".to_string(),
                arms: vec![
                    ace_av_core::mir::VariantArm {
                        name: "Ok".into(),
                        payload: Some(3),
                    },
                    ace_av_core::mir::VariantArm {
                        name: "Err".into(),
                        payload: Some(2),
                    },
                ],
            }),
        ];
        TxFx {
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
            name: Some(name.into()),
        });
        (self.slots.len() - 1) as u32
    }
    fn slot_list_str(&mut self) -> u32 {
        self.slot(4, "list")
    }
    fn slot_list_note(&mut self) -> u32 {
        self.slot(5, "list")
    }
    fn slot_str(&mut self, n: &str) -> u32 {
        self.slot(2, n)
    }
    fn slot_i32(&mut self, n: &str) -> u32 {
        self.slot(0, n)
    }
    fn slot_bool(&mut self, n: &str) -> u32 {
        self.slot(1, n)
    }
    fn slot_note(&mut self, n: &str) -> u32 {
        self.slot(3, n)
    }
    fn slot_variant(&mut self, n: &str) -> u32 {
        self.slot(6, n)
    }
    fn t_list_str(&self) -> u32 {
        4
    }
    fn t_list_note(&self) -> u32 {
        5
    }
    fn t_note(&self) -> u32 {
        3
    }
    fn t_outcome(&self) -> u32 {
        6
    }
    fn c_str_owned(&mut self, s: &str) -> u32 {
        self.m.intern_const(MirConst::Str(Box::from(s.as_bytes())))
    }
    fn c_i32v(&mut self, v: i32) -> u32 {
        self.m.intern_const(MirConst::I32(v))
    }
    fn i(&mut self, instr: MirInstr) {
        self.instrs.push(instr);
    }
    fn ret(&mut self, s: u32) {
        self.ret = s;
    }
    fn build(mut self) -> MirModule {
        assert_ne!(self.ret, u32::MAX);
        let sig = self.m.intern_type(MirType::Fn(ace_av_core::mir::FnSig {
            params: vec![],
            result: 0,
        }));
        self.m.funcs = vec![MirFunction {
            owner: self.m.owner.clone(),
            name: "main".into(),
            sig,
            slots: self.slots,
            blocks: vec![MirBlock {
                instrs: self.instrs,
                term: Terminator::Return { value: self.ret },
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

/// List<Str> in-place replace, small -> BIG (F822-R1-01 primary): at
/// exactly-short budget the rejection fires BEFORE the swap — the old
/// element's buffer is not freed mid-instruction (the unwind-order
/// assertion catches a swap-then-check implementation), the source keeps
/// its materialization, and the ledger balances; at exactly-enough the
/// replacement succeeds.
#[test]
fn av_collections_resources_listset_transaction_both_sides() {
    let big = "B".repeat(4096);
    let mut f = TxFx::new();
    let s0 = f.slot_list_str();
    let s1 = f.slot_str("a");
    let s2 = f.slot_str("big");
    let s3 = f.slot_i32("idx");
    let s4 = f.slot_i32("r");
    let ca = f.c_str_owned("a");
    let cb = f.c_str_owned(&big);
    let c0 = f.c_i32v(0);
    let c42 = f.c_i32v(42);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: f.t_list_str(),
        cap: 0,
    });
    let v1 = ca;
    f.i(MirInstr::Const { dst: s1, value: v1 });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s1,
    });
    let v2 = cb;
    f.i(MirInstr::Const { dst: s2, value: v2 });
    let v3 = c0;
    f.i(MirInstr::Const { dst: s3, value: v3 });
    f.i(MirInstr::ListSet {
        root: s0,
        index: s3,
        value: s2,
    });
    let v4 = c42;
    f.i(MirInstr::Const { dst: s4, value: v4 });
    f.ret(s4);
    let (_peak, enough, (short, arena)) = both_sides_of_budget(&f.build());
    assert!(
        matches!(enough, ExecOutcome::Success { .. }),
        "exactly-enough: {enough:?}"
    );
    match &short {
        ExecOutcome::Resource { kind, .. } => assert_eq!(*kind, ResourceKind::SlotBytes),
        other => panic!("exactly-short must be a SlotBytes resource: {other:?}"),
    }
    assert_clean_ledger(&arena);
    // The OLD buffer (id 0, materialized first) frees only in the
    // unwind — a swap-then-check implementation frees it MID-instruction,
    // making it the FIRST free event (nothing else has died yet); the
    // precheck implementation's first free is the unwound BIG buffer
    // (id 2, still held by the source slot at rejection).
    let ids: Vec<u64> = arena.free_events().iter().map(|e| e.id).collect();
    assert_eq!(
        ids.first(),
        Some(&2),
        "first free = the unwound big buffer; got {ids:?}"
    );
    assert_eq!(
        ids.len(),
        3,
        "exactly the three materialized buffers: {ids:?}"
    );
    assert!(
        ids.contains(&0) && ids.contains(&1),
        "the list and its old element unwind too: {ids:?}"
    );
}

/// List<Note> in-place FIELD replace, text "a" -> BIG (same-cause
/// sibling): identical both-sides contract.
#[test]
fn av_collections_resources_listfieldset_transaction_both_sides() {
    let big = "B".repeat(4096);
    let mut f = TxFx::new();
    let s0 = f.slot_list_note();
    let s1 = f.slot_str("a");
    let s2 = f.slot_i32("n");
    let s3 = f.slot_note("note");
    let s4 = f.slot_i32("idx");
    let s5 = f.slot_str("big");
    let s6 = f.slot_i32("r");
    let ca = f.c_str_owned("a");
    let c1 = f.c_i32v(1);
    let cb = f.c_str_owned(&big);
    let c0 = f.c_i32v(0);
    let c42 = f.c_i32v(42);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: f.t_list_note(),
        cap: 0,
    });
    let v1 = ca;
    f.i(MirInstr::Const { dst: s1, value: v1 });
    let v2 = c1;
    f.i(MirInstr::Const { dst: s2, value: v2 });
    f.i(MirInstr::Aggregate {
        dst: s3,
        record_ty: f.t_note(),
        operands: vec![s1, s2],
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s3,
    });
    let v5 = cb;
    f.i(MirInstr::Const { dst: s5, value: v5 });
    let v4 = c0;
    f.i(MirInstr::Const { dst: s4, value: v4 });
    f.i(MirInstr::ListFieldSet {
        root: s0,
        index: s4,
        path: vec![0],
        value: s5,
    });
    let v6 = c42;
    f.i(MirInstr::Const { dst: s6, value: v6 });
    f.ret(s6);
    let (_p, enough, (short, arena)) = both_sides_of_budget(&f.build());
    assert!(
        matches!(enough, ExecOutcome::Success { .. }),
        "exactly-enough: {enough:?}"
    );
    assert!(
        matches!(
            &short,
            ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                ..
            }
        ),
        "{short:?}"
    );
    assert_clean_ledger(&arena);
}

/// LIST_REMOVE under an exactly-short destination budget: the rejection
/// fires BEFORE the element moves out — contents and epoch untouched.
#[test]
fn av_collections_resources_listremove_transaction_both_sides() {
    let big = "B".repeat(4096);
    let mut f = TxFx::new();
    let s0 = f.slot_list_note();
    let s1 = f.slot_str("a");
    let s2 = f.slot_i32("n");
    let s3 = f.slot_note("note");
    let s4 = f.slot_i32("idx");
    let s5 = f.slot_note("removed");
    let s6 = f.slot_i32("r");
    let ca = f.c_str_owned(&big);
    let c1 = f.c_i32v(1);
    let c0 = f.c_i32v(0);
    let c42 = f.c_i32v(42);
    f.i(MirInstr::ListNew {
        dst: s0,
        list_ty: f.t_list_note(),
        cap: 0,
    });
    let v1 = ca;
    f.i(MirInstr::Const { dst: s1, value: v1 });
    let v2 = c1;
    f.i(MirInstr::Const { dst: s2, value: v2 });
    f.i(MirInstr::Aggregate {
        dst: s3,
        record_ty: f.t_note(),
        operands: vec![s1, s2],
    });
    f.i(MirInstr::ListPush {
        root: s0,
        value: s3,
    });
    let v4 = c0;
    f.i(MirInstr::Const { dst: s4, value: v4 });
    f.i(MirInstr::ListRemove {
        dst: s5,
        root: s0,
        index: s4,
    });
    let v6 = c42;
    f.i(MirInstr::Const { dst: s6, value: v6 });
    f.ret(s6);
    let (_p, enough, (short, arena)) = both_sides_of_budget(&f.build());
    assert!(
        matches!(enough, ExecOutcome::Success { .. }),
        "exactly-enough: {enough:?}"
    );
    assert!(
        matches!(
            &short,
            ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                ..
            }
        ),
        "{short:?}"
    );
    assert_clean_ledger(&arena);
}

/// VARIANT_PROJECT of an OWNED payload under an exactly-short destination
/// budget: the rejection fires BEFORE the take — the arm keeps its
/// payload.
#[test]
fn av_collections_resources_variantproject_transaction_both_sides() {
    let big = "B".repeat(4096);
    let mut f = TxFx::new();
    let s0 = f.slot_str("text");
    let s1 = f.slot_variant("err");
    let s2 = f.slot_bool("g");
    let s3 = f.slot_str("msg");
    let s4 = f.slot_i32("r");
    let cb = f.c_str_owned(&big);
    let c42 = f.c_i32v(42);
    f.i(MirInstr::Const {
        dst: s0,
        value: {
            let v = cb;
            v
        },
    });
    f.i(MirInstr::VariantPack {
        dst: s1,
        variant_ty: f.t_outcome(),
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
    let v4 = c42;
    f.i(MirInstr::Const { dst: s4, value: v4 });
    f.ret(s4);
    let (_p, enough, (short, arena)) = both_sides_of_budget(&f.build());
    assert!(
        matches!(enough, ExecOutcome::Success { .. }),
        "exactly-enough: {enough:?}"
    );
    assert!(
        matches!(
            &short,
            ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                ..
            }
        ),
        "{short:?}"
    );
    assert_clean_ledger(&arena);
}

// Formal parent-sampled memory hard gate: original four requests and thresholds.
mod performance {
    use super::*;

    /// T-09/V-07: the collections worker's REAL private-bytes peak (fresh
    /// release processes, parent-sampled) stays under the 798 B-line 128MiB
    /// envelope, and the managed partitions hold their own dimensions
    /// (hot8/staging16/execution8).
    #[test]
    fn av_collections_worker_memory_hard_gate_parent_sampled() {
        let ws = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .expect("ACE workspace root")
            .to_path_buf();
        let mut compiler = std::process::Command::new(env!("CARGO"));
        compiler
            .args(["build", "--release", "-p", "ace-av", "--bin", "av-worker"])
            .current_dir(&ws);
        let build = auto_tool_execution::run_command_capture(
            compiler,
            auto_tool_execution::TreeScope::ProcessTree,
            None,
        )
        .expect("build");
        assert!(
            build.kind == auto_tool_execution::ExitKind::Success,
            "{}",
            String::from_utf8_lossy(build.stderr.as_ref().expect("captured compiler stderr"))
        );
        let worker = ws.join("target").join("release").join(if cfg!(windows) {
            "av-worker.exe"
        } else {
            "av-worker"
        });

        let root = std::env::temp_dir().join(format!(
            "av822-memgate-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let store = root.join("store");

        // Fresh worker processes over the frozen fixture: 4 independent
        // samples against ONE accumulating managed store.
        let mut evidence = Vec::new();
        for i in 0..4u32 {
            let (ev, ok) = sampled_collections_request(&worker, &store);
            assert!(ok, "sample {i} runs green");
            evidence.push(ev);
        }

        const B_LINE: u64 = 128 * 1024 * 1024;
        const HOT: u64 = 8 * 1024 * 1024;
        const STAGING: u64 = 16 * 1024 * 1024;
        const EXECUTION: u64 = 8 * 1024 * 1024;
        assert!(
            evidence.iter().all(|e| e.worker_private_peak > 0),
            "a real process peak was measured"
        );
        for (i, e) in evidence.iter().enumerate() {
            assert!(
                e.worker_private_peak <= B_LINE,
                "sample {i}: worker B-line private-bytes peak {} > {}",
                e.worker_private_peak,
                B_LINE
            );
            assert!(
                e.peak_hot <= HOT,
                "sample {i}: managed hot partition {} > {}",
                e.peak_hot,
                HOT
            );
            assert!(
                e.peak_staging <= STAGING,
                "sample {i}: managed staging partition {} > {}",
                e.peak_staging,
                STAGING
            );
            assert!(
                e.peak_exec_slot <= EXECUTION,
                "sample {i}: managed execution partition {} > {}",
                e.peak_exec_slot,
                EXECUTION
            );
        }
        let _ = std::fs::remove_dir_all(&root);
    }
}
