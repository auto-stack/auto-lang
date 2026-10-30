//! PLAN-821 T-03 semantics tests (`av_owned_821` family): the frozen
//! V-01/V-02/V-05/V-06/V-11/V-12 matrices (contract-decision §12) as typed
//! MIR fixtures through the REAL admission chain — verify_mir (ownership
//! dataflow) → AVAD v2 encode → decode → deep admission → AV execute.
//! Positives execute with exact-value oracles; negatives pin their
//! specific diagnostic code (a wrong code or a wrong-side rejection is a
//! red test, not a pass). Source-domain shapes that stay with 782's
//! `verify.*` codes (partial move) are pinned through
//! `compile_text_record_source` so both layers stay bound.

use ace_av_core::abc::{decode_abc, encode_abc};
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, TrapKind, Value, MIR_OWNED_SCHEMA};
use ace_av_core::mir::*;
use ace_av_core::{abc_verify, mir_verify};

// ---- fixture toolkit -------------------------------------------------------

/// Type table: str/i32/bool/char/view/mutref/note indices, then per-fn sigs.
#[derive(Clone, Copy)]
struct T {
    str: u32,
    i32: u32,
    bool: u32,
    char: u32,
    view: u32,
    mutref: u32,
    note: u32,
}

/// One owned-profile module under construction; consts are interned via
/// the helpers BEFORE building blocks that reference them (single mutable
/// borrow at a time).
struct F {
    m: MirModule,
    t: T,
}

impl F {
    fn new(name: &str) -> Self {
        let mut m = MirModule::new_owned(OwnerScope {
            schema: MIR_OWNED_SCHEMA.to_string(),
            module: name.to_string(),
        });
        let str = m.intern_type(MirType::Str);
        let t = T {
            str,
            i32: m.intern_type(MirType::I32),
            bool: m.intern_type(MirType::Bool),
            char: m.intern_type(MirType::Char),
            view: m.intern_type(MirType::StrView),
            mutref: m.intern_type(MirType::MutRef),
            note: m.intern_type(MirType::Record(RecordTypeRef {
                owner_module: name.to_string(),
                name: "Note".into(),
                fields: vec![MirFieldDef {
                    name: "body".into(),
                    ty: str,
                }],
            })),
        };
        F { m, t }
    }

    fn s(&mut self, lit: &str) -> u32 {
        self.m
            .intern_const(MirConst::Str(lit.as_bytes().to_vec().into_boxed_slice()))
    }

    fn i(&mut self, v: i32) -> u32 {
        self.m.intern_const(MirConst::I32(v))
    }

    fn b(&mut self, v: bool) -> u32 {
        self.m.intern_const(MirConst::Bool(v))
    }

    /// Push `d_main() -> i32` and make it the entry.
    fn main(&mut self, slots: Vec<MirSlot>, blocks: Vec<MirBlock>) {
        let sig = self.m.intern_type(MirType::Fn(FnSig {
            params: vec![],
            result: self.t.i32,
        }));
        self.m.funcs.push(MirFunction {
            owner: self.m.owner.clone(),
            name: "d_main".into(),
            sig,
            slots,
            blocks,
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        self.m.entry = (self.m.funcs.len() - 1) as u32;
    }

    /// Push one non-entry function with a custom sig; returns its index.
    fn extra(
        &mut self,
        params: Vec<u32>,
        result: u32,
        slots: Vec<MirSlot>,
        blocks: Vec<MirBlock>,
    ) -> u32 {
        let sig = self.m.intern_type(MirType::Fn(FnSig { params, result }));
        self.m.funcs.push(MirFunction {
            owner: self.m.owner.clone(),
            name: format!("f{}", self.m.funcs.len()),
            sig,
            slots,
            blocks,
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        (self.m.funcs.len() - 1) as u32
    }

    /// ctor pair into slots (1 = Str temp, 0 = Note local); the literal
    /// const must be pre-interned by the caller.
    fn ctor_note(&mut self, lit: u32) -> Vec<MirInstr> {
        let note = self.t.note;
        vec![
            MirInstr::Const { dst: 1, value: lit },
            MirInstr::Aggregate {
                dst: 0,
                record_ty: note,
                operands: vec![1],
            },
        ]
    }
}

fn local(ty: u32, name: &str) -> MirSlot {
    MirSlot {
        kind: SlotKind::Local,
        ty,
        name: Some(name.into()),
    }
}

fn temp(ty: u32) -> MirSlot {
    MirSlot {
        kind: SlotKind::Temp,
        ty,
        name: Some("t".into()),
    }
}

fn param(ty: u32, name: &str) -> MirSlot {
    MirSlot {
        kind: SlotKind::Param,
        ty,
        name: Some(name.into()),
    }
}

fn block(instrs: Vec<MirInstr>, term: Terminator) -> MirBlock {
    MirBlock { instrs, term }
}

/// Admit (verify → encode → decode → deep admission) and execute; returns
/// the Success value + trace (panics on any other outcome).
fn admit_exec(m: &MirModule) -> (Value, ace_av_core::av::ExecTrace) {
    mir_verify::verify_mir(m).expect("owned MIR verifies");
    let wire = encode_abc(m).expect("encode");
    let flat = wire.to_flat();
    let decoded = decode_abc(&flat).expect("decode");
    let loaded = abc_verify::verify_abc(decoded, &m.owner).expect("AVAD v2 admits");
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Success { value, .. } = out else {
        panic!("expected success, got {out:?}")
    };
    (value, trace)
}

/// Admit + execute expecting a language trap; returns its kind.
fn admit_trap(m: &MirModule) -> TrapKind {
    mir_verify::verify_mir(m).expect("owned MIR verifies");
    let wire = encode_abc(m).expect("encode");
    let decoded = decode_abc(&wire.to_flat()).expect("decode");
    let loaded = abc_verify::verify_abc(decoded, &m.owner).expect("AVAD v2 admits");
    let (out, _) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    let ExecOutcome::Trap { kind, .. } = out else {
        panic!("expected a trap, got {out:?}")
    };
    kind
}

/// verify_mir must reject with exactly this code family.
fn verify_reject(m: &MirModule, want: &str) {
    let err = mir_verify::verify_mir(m).expect_err("must be rejected");
    let code = err.code().to_string();
    assert!(
        code.contains(want),
        "expected `{want}`, got `{code}` ({err:?})"
    );
}

// ---- V-01: reservation / RHS reads same root -------------------------------

/// Positive: field REPLACE whose RHS READS the same root through a Str
/// field read (a view) + re-materialization — legal (read-only RHS);
/// executes with the exact oracle len("Auto") = 4 and the ledger shape.
#[test]
fn av_owned_v01_field_replace_rhs_reads_same_root_legal() {
    let mut f = F::new("v01a");
    let t = f.t;
    let c_auto = f.s("Auto");
    f.main(
        vec![
            local(t.note, "u"),
            temp(t.str),
            temp(t.view),
            temp(t.i32),
            temp(t.str),
            temp(t.view),
        ],
        vec![block(
            vec![
                MirInstr::Const {
                    dst: 1,
                    value: c_auto,
                },
                MirInstr::Aggregate {
                    dst: 0,
                    record_ty: t.note,
                    operands: vec![1],
                },
                // RHS: read the same root (a Str field read IS a view)
                // and re-materialize it as the replacement value.
                MirInstr::FieldGet {
                    dst: 2,
                    base: 0,
                    path: vec![0],
                },
                MirInstr::StrLen { dst: 3, view: 2 },
                MirInstr::StrToOwned { dst: 4, view: 2 },
                // The RHS borrow ENDS at the materialization boundary —
                // the adapter's canonical statement shape (root-
                // conservative: no view crosses the commit).
                MirInstr::BorrowEnd { token: 2 },
                MirInstr::FieldSet {
                    root: 0,
                    path: vec![0],
                    value: 4,
                },
                MirInstr::FieldGet {
                    dst: 5,
                    base: 0,
                    path: vec![0],
                },
                MirInstr::StrLen { dst: 3, view: 5 },
                MirInstr::BorrowEnd { token: 5 },
            ],
            Terminator::Return { value: 3 },
        )],
    );
    let (value, trace) = admit_exec(&f.m);
    assert_eq!(value, Value::I32(4));
    let arena = trace.arena.expect("owned execution carries the arena");
    // Two 4-byte buffers: the ctor literal frees at the replace; the
    // re-materialized copy cleans at frame exit.
    assert_eq!(
        arena
            .alloc_events()
            .iter()
            .map(|a| a.bytes)
            .collect::<Vec<_>>(),
        vec![4, 4]
    );
    assert_eq!(arena.live(), 0, "frame exit cleans owned values");
}

/// The reachable MIR reservation shape: the self-move (record types are
/// acyclic, so a Str field's value-def MOVE of the same root degenerates
/// to this; source-shaped conflicts stay with verify.* — see below).
#[test]
fn av_owned_v01_self_move_is_reservation_conflict() {
    let mut f = F::new("v01b");
    let t = f.t;
    let c_auto = f.s("Auto");
    let c_zero = f.i(0);
    let mut instrs = f.ctor_note(c_auto);
    // The reachable reservation shape: the self-move (root==field-type
    // cycles are impossible in acyclic records — §9 reconciliation).
    instrs.push(MirInstr::Move { dst: 0, src: 0 });
    instrs.push(MirInstr::Const {
        dst: 3,
        value: c_zero,
    });
    f.main(
        vec![local(t.note, "u"), temp(t.str), temp(t.note), temp(t.i32)],
        vec![block(instrs, Terminator::Return { value: 3 })],
    );
    verify_reject(&f.m, "reservation-conflict");
}

/// Source-domain shape: a partial field move stays with the 782 source
/// gate's `verify.partial-move` — the MIR mirror never sees it.
#[test]
fn av_owned_v01_source_domain_shapes_keep_verify_codes() {
    let ds = ace_core::source::compile_text_record_source(
        "type Note { body Str; n int }\nfn make() Note { return Note { body: \"x\", n: 1 } }\nfn main() int {\n    let b Note = make().move\n    return b.n\n}\n",
        "p.at",
        &[],
    );
    assert!(ds.is_err(), "partial field move must be rejected");
    let code = ds
        .err()
        .unwrap()
        .first()
        .map(|d| d.code.clone())
        .unwrap_or_default();
    assert!(
        code.contains("partial-move"),
        "expected the 782 partial-move code, got {code}"
    );
}

// ---- V-02: borrow exclusivity / multi-view ---------------------------------

/// Legal multi-view: two VIEWs of one root + a COPY of a view (token
/// handle copy = one more read view, CD §4 COPY row) — all read-only;
/// executes len=4 with an empty residue.
#[test]
fn av_owned_v02_multi_view_and_view_copy_legal() {
    let mut f = F::new("v02a");
    let t = f.t;
    let c_auto = f.s("Auto");
    f.main(
        vec![
            local(t.str, "s"),
            temp(t.view),
            temp(t.view),
            temp(t.view),
            temp(t.i32),
            temp(t.bool),
            temp(t.i32),
        ],
        vec![block(
            vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_auto,
                },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::View { dst: 2, src: 0 },
                MirInstr::Copy { dst: 3, src: 1 },
                MirInstr::StrLen { dst: 4, view: 3 },
                MirInstr::StrCmp {
                    eq: true,
                    dst: 5,
                    lhs: 1,
                    rhs: 2,
                },
                MirInstr::BorrowEnd { token: 1 },
                MirInstr::BorrowEnd { token: 2 },
                MirInstr::BorrowEnd { token: 3 },
                MirInstr::Copy { dst: 6, src: 4 },
            ],
            Terminator::Return { value: 6 },
        )],
    );
    let (value, trace) = admit_exec(&f.m);
    assert_eq!(value, Value::I32(4));
    assert_eq!(trace.arena.as_ref().unwrap().live(), 0);
}

/// Overlapping mut: a live MUTREF blocks FIELD_SET on the same root
/// (`mir.mut-conflict`, root-conservative).
#[test]
fn av_owned_v02_live_mutref_blocks_field_set() {
    let mut f = F::new("v02b");
    let t = f.t;
    let c_auto = f.s("Auto");
    let c_b = f.s("B");
    let c_zero = f.i(0);
    let mut instrs = f.ctor_note(c_auto);
    instrs.push(MirInstr::MutRef { dst: 2, src: 0 });
    instrs.push(MirInstr::Const { dst: 3, value: c_b });
    instrs.push(MirInstr::FieldSet {
        root: 0,
        path: vec![0],
        value: 3,
    });
    instrs.push(MirInstr::Const {
        dst: 4,
        value: c_zero,
    });
    f.main(
        vec![
            local(t.note, "u"),
            temp(t.str),
            temp(t.mutref),
            temp(t.str),
            temp(t.i32),
        ],
        vec![block(instrs, Terminator::Return { value: 4 })],
    );
    verify_reject(&f.m, "mut-conflict");
}

/// BORROW_END releases the exclusion: after ending the MUTREF, the same
/// FIELD_SET is legal and executes with the replaced body.
#[test]
fn av_owned_v02_borrow_end_releases_exclusion() {
    let mut f = F::new("v02c");
    let t = f.t;
    let c_auto = f.s("Auto");
    let c_b = f.s("B");
    let mut instrs = f.ctor_note(c_auto);
    instrs.push(MirInstr::MutRef { dst: 2, src: 0 });
    instrs.push(MirInstr::BorrowEnd { token: 2 });
    instrs.push(MirInstr::Const { dst: 3, value: c_b });
    instrs.push(MirInstr::FieldSet {
        root: 0,
        path: vec![0],
        value: 3,
    });
    instrs.push(MirInstr::FieldGet {
        dst: 4,
        base: 0,
        path: vec![0],
    });
    instrs.push(MirInstr::StrLen { dst: 5, view: 4 });
    instrs.push(MirInstr::BorrowEnd { token: 4 });
    f.main(
        vec![
            local(t.note, "u"),
            temp(t.str),
            temp(t.mutref),
            temp(t.str),
            temp(t.view),
            temp(t.i32),
        ],
        vec![block(instrs, Terminator::Return { value: 5 })],
    );
    let (value, _) = admit_exec(&f.m);
    assert_eq!(value, Value::I32(1), "replaced body is \"B\"");
}

/// COPY of an owned Str is refused (`mir.copy-owned` — ownership never
/// rides a copy); COPY of a MutRef likewise (exclusivity).
#[test]
fn av_owned_v02_copy_owned_and_mutref_refused() {
    let mut f = F::new("v02d");
    let t = f.t;
    let c_x = f.s("x");
    f.main(
        vec![local(t.str, "s"), temp(t.str), temp(t.i32)],
        vec![block(
            vec![
                MirInstr::Const { dst: 0, value: c_x },
                MirInstr::Copy { dst: 1, src: 0 },
                MirInstr::Copy { dst: 2, src: 1 },
            ],
            Terminator::Return { value: 2 },
        )],
    );
    verify_reject(&f.m, "copy-owned");

    let mut f = F::new("v02e");
    let t = f.t;
    let c_auto = f.s("Auto");
    let c_zero = f.i(0);
    let mut instrs = f.ctor_note(c_auto);
    instrs.push(MirInstr::MutRef { dst: 2, src: 0 });
    instrs.push(MirInstr::Copy { dst: 3, src: 2 });
    instrs.push(MirInstr::Const {
        dst: 4,
        value: c_zero,
    });
    f.main(
        vec![
            local(t.note, "u"),
            temp(t.str),
            temp(t.mutref),
            temp(t.mutref),
            temp(t.i32),
        ],
        vec![block(instrs, Terminator::Return { value: 4 })],
    );
    verify_reject(&f.m, "copy-owned");
}

/// Two moves of one source: the second MOVE reads a moved slot
/// (use-after-move family, V-02's double-move).
#[test]
fn av_owned_v02_double_move_rejected() {
    let mut f = F::new("v02f");
    let t = f.t;
    let c_auto = f.s("Auto");
    let c_zero = f.i(0);
    let mut instrs = f.ctor_note(c_auto);
    instrs.push(MirInstr::Move { dst: 2, src: 0 });
    instrs.push(MirInstr::Move { dst: 3, src: 0 });
    instrs.push(MirInstr::Const {
        dst: 4,
        value: c_zero,
    });
    f.main(
        vec![
            local(t.note, "u"),
            temp(t.str),
            temp(t.note),
            temp(t.note),
            temp(t.i32),
        ],
        vec![block(instrs, Terminator::Return { value: 4 })],
    );
    verify_reject(&f.m, "use-after-move");
}

// ---- V-05: handles vs integers ---------------------------------------------

/// An i32 whose VALUE equals a live resource id stays an integer (typed
/// isolation); BIN over it executes exactly.
#[test]
fn av_owned_v05_integer_equal_to_resource_id_stays_integer() {
    let mut f = F::new("v05a");
    let t = f.t;
    let c_x = f.s("");
    f.main(
        vec![local(t.str, "s"), temp(t.view), temp(t.i32), temp(t.i32)],
        vec![block(
            vec![
                MirInstr::Const { dst: 0, value: c_x },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::StrLen { dst: 2, view: 1 },
                // n == 0 == the first allocation's ledger id: still an
                // integer; BIN over it is ordinary arithmetic.
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 3,
                    lhs: 2,
                    rhs: 2,
                },
                MirInstr::BorrowEnd { token: 1 },
            ],
            Terminator::Return { value: 3 },
        )],
    );
    let (value, trace) = admit_exec(&f.m);
    assert_eq!(value, Value::I32(0), "0 + 0 stays exact integer arithmetic");
    assert_eq!(trace.arena.as_ref().unwrap().live(), 0);
}

/// A StrView slotted into an i32 BIN is a VERIFY-side rejection
/// (`mir.bin-type`): handle/integer conflation never reaches runtime.
#[test]
fn av_owned_v05_view_into_int_op_rejected_at_verify() {
    let mut f = F::new("v05b");
    let t = f.t;
    let c_x = f.s("x");
    f.main(
        vec![local(t.str, "s"), temp(t.view), temp(t.i32), temp(t.i32)],
        vec![block(
            vec![
                MirInstr::Const { dst: 0, value: c_x },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 3,
                    lhs: 1, // a view where an i32 is required
                    rhs: 2,
                },
                MirInstr::BorrowEnd { token: 1 },
            ],
            Terminator::Return { value: 3 },
        )],
    );
    verify_reject(&f.m, "bin-type");
}

/// Runtime handle domain: a plain-Str root's borrows end, the root
/// redefines (legal — no live token), and the OLD buffer frees exactly
/// once; a fresh view sees the new bytes. The typed layer keeps integers
/// and handles disjoint (see the two V-05 tests above); the value-layer
/// Share/stale-read oracle stays pinned in core's place.rs (a live view
/// across a root write is structurally unreachable in admitted MIR).
#[test]
fn av_owned_v05_borrow_end_then_redefine_frees_old_exactly_once() {
    let mut f = F::new("v05c");
    let t = f.t;
    let c_aaaa = f.s("AAAA");
    let c_b = f.s("B");
    let c_ten = f.i(10);
    f.main(
        vec![
            local(t.str, "s"),
            temp(t.view),
            temp(t.i32), // old len
            temp(t.i32), // new len
            temp(t.i32), // r = old*10 + new
            temp(t.i32), // ten
        ],
        vec![block(
            vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_aaaa,
                },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::StrLen { dst: 2, view: 1 },
                MirInstr::BorrowEnd { token: 1 },
                // Redefine the root: no live borrow remains — legal.
                MirInstr::Const { dst: 0, value: c_b },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::StrLen { dst: 3, view: 1 },
                MirInstr::BorrowEnd { token: 1 },
                MirInstr::Const {
                    dst: 5,
                    value: c_ten,
                },
                MirInstr::Bin {
                    op: BinOp::MulI32,
                    dst: 4,
                    lhs: 2,
                    rhs: 5,
                },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 4,
                    lhs: 4,
                    rhs: 3,
                },
            ],
            Terminator::Return { value: 4 },
        )],
    );
    let (value, trace) = admit_exec(&f.m);
    assert_eq!(value, Value::I32(41), "old len 4, new len 1");
    let arena = trace.arena.expect("arena");
    assert_eq!(
        arena
            .alloc_events()
            .iter()
            .map(|a| a.bytes)
            .collect::<Vec<_>>(),
        vec![4, 1]
    );
    // The old buffer frees at the redefinition, exactly once.
    let allocs = arena.alloc_events();
    let frees = arena.free_events();
    assert_eq!(frees.len(), 2);
    assert_eq!(
        frees[0].id, allocs[0].id,
        "old buffer freed at the redefine"
    );
    assert_eq!(frees[1].id, allocs[1].id, "new buffer freed at frame exit");
    assert_eq!(arena.live(), 0);
}

// ---- V-06: frozen bit samples / escapes / traps ----------------------------

/// The frozen §3 sample table through the full wire: r = len*1000+bytes.
#[test]
fn av_owned_v06_frozen_bit_samples_observe_exactly() {
    for (lit, want_len, want_bytes) in [
        ("", 0, 0),
        ("A", 1, 1),
        ("\0", 1, 1),
        ("e\u{0301}", 2, 3),
        ("😀", 1, 4),
        ("中", 1, 3),
        ("A中😀", 3, 8),
    ] {
        let mut f = F::new("v06a");
        let t = f.t;
        let c_lit = f.s(lit);
        let c_1000 = f.i(1000);
        f.main(
            vec![
                local(t.str, "s"),
                temp(t.view),
                temp(t.i32),
                temp(t.i32),
                temp(t.i32),
                temp(t.i32),
            ],
            vec![block(
                vec![
                    MirInstr::Const {
                        dst: 0,
                        value: c_lit,
                    },
                    MirInstr::View { dst: 1, src: 0 },
                    MirInstr::StrLen { dst: 2, view: 1 },
                    MirInstr::StrLenBytes { dst: 3, view: 1 },
                    MirInstr::Const {
                        dst: 5,
                        value: c_1000,
                    },
                    MirInstr::Bin {
                        op: BinOp::MulI32,
                        dst: 4,
                        lhs: 2,
                        rhs: 5,
                    },
                    MirInstr::Bin {
                        op: BinOp::AddI32,
                        dst: 4,
                        lhs: 4,
                        rhs: 3,
                    },
                    MirInstr::BorrowEnd { token: 1 },
                ],
                Terminator::Return { value: 4 },
            )],
        );
        let (value, _) = admit_exec(&f.m);
        assert_eq!(
            value,
            Value::I32(want_len * 1000 + want_bytes),
            "sample {lit:?}: len={want_len} bytes={want_bytes}"
        );
    }
}

/// char_at out of range traps `av.trap.str-index`; from_codepoint of a
/// surrogate traps `av.trap.char-invalid`.
#[test]
fn av_owned_v06_oob_char_at_and_invalid_codepoint_trap() {
    let mut f = F::new("v06b");
    let t = f.t;
    let c_a = f.s("A");
    let c_1 = f.i(1);
    f.main(
        vec![local(t.str, "s"), temp(t.view), temp(t.i32), temp(t.char)],
        vec![block(
            vec![
                MirInstr::Const { dst: 0, value: c_a },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::Const { dst: 2, value: c_1 },
                MirInstr::StrCharAt {
                    dst: 3,
                    view: 1,
                    index: 2,
                },
                MirInstr::BorrowEnd { token: 1 },
            ],
            Terminator::Return { value: 2 },
        )],
    );
    assert_eq!(admit_trap(&f.m), TrapKind::StrIndex);

    let mut f = F::new("v06c");
    let t = f.t;
    let c_d800 = f.i(0xD800);
    let c_0 = f.i(0);
    f.main(
        vec![temp(t.i32), temp(t.char), temp(t.i32)],
        vec![block(
            vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_d800,
                },
                MirInstr::CharFromCodepoint { dst: 1, code: 0 },
                MirInstr::Const { dst: 2, value: c_0 },
            ],
            Terminator::Return { value: 2 },
        )],
    );
    assert_eq!(admit_trap(&f.m), TrapKind::CharInvalid);
}

/// char domain round-trip: 0x0 / 0x1F600 / 0x10FFFF ride
/// to_codepoint/from_codepoint exactly.
#[test]
fn av_owned_v06_char_domain_round_trips() {
    for cp in [0x0i32, 0x1F600, 0x10FFFF] {
        let mut f = F::new("v06d");
        let t = f.t;
        let c_cp = f.i(cp);
        let c_0 = f.i(0);
        f.main(
            vec![temp(t.i32), temp(t.char), temp(t.i32), temp(t.i32)],
            vec![block(
                vec![
                    MirInstr::Const {
                        dst: 0,
                        value: c_cp,
                    },
                    MirInstr::CharFromCodepoint { dst: 1, code: 0 },
                    MirInstr::CharToCodepoint { dst: 2, ch: 1 },
                    MirInstr::Const { dst: 3, value: c_0 },
                ],
                Terminator::Return { value: 2 },
            )],
        );
        let (value, _) = admit_exec(&f.m);
        assert_eq!(value, Value::I32(cp), "codepoint {cp:#x} round-trips");
    }
}

/// Borrow escape: a NON-ENTRY function returning a view whose root is a
/// LOCAL is `mir.borrow-escape`; returning a view rooted at a borrowed
/// PARAM is legal (the caller owns the root). (Views never cross the
/// ENTRY — that face is `mir.entry-sig`.)
#[test]
fn av_owned_v06_view_escape_rejected_and_param_view_legal() {
    // (a) local-rooted view returned -> mir.borrow-escape.
    let mut f = F::new("v06e");
    let t = f.t;
    let c_x = f.s("x");
    let c_0 = f.i(0);
    f.extra(
        vec![],
        t.view,
        vec![local(t.str, "s"), temp(t.view)],
        vec![block(
            vec![
                MirInstr::Const { dst: 0, value: c_x },
                MirInstr::View { dst: 1, src: 0 },
            ],
            Terminator::Return { value: 1 },
        )],
    );
    f.main(
        vec![temp(t.i32)],
        vec![block(
            vec![MirInstr::Const { dst: 0, value: c_0 }],
            Terminator::Return { value: 0 },
        )],
    );
    verify_reject(&f.m, "borrow-escape");

    // (b) view param returned (root EXTERNAL — caller-owned): legal.
    let mut f = F::new("v06f");
    let t = f.t;
    let c_0 = f.i(0);
    f.extra(
        vec![t.view],
        t.view,
        vec![param(t.view, "v")],
        vec![block(vec![], Terminator::Return { value: 0 })],
    );
    f.main(
        vec![temp(t.i32)],
        vec![block(
            vec![MirInstr::Const { dst: 0, value: c_0 }],
            Terminator::Return { value: 0 },
        )],
    );
    mir_verify::verify_mir(&f.m).expect("param-rooted view return is legal");
}

// ---- V-11: control-flow merges / loops / budget -----------------------------

/// Branch one-arm MOVE + join read = use-after-move (may-union).
#[test]
fn av_owned_v11_one_arm_move_join_read_rejected() {
    let mut f = F::new("v11a");
    let t = f.t;
    let c_auto = f.s("Auto");
    let c_true = f.b(true);
    let mut b0 = f.ctor_note(c_auto);
    b0.push(MirInstr::Const {
        dst: 2,
        value: c_true,
    });
    f.main(
        vec![
            local(t.note, "u"),
            temp(t.str),
            temp(t.bool),
            temp(t.note),
            temp(t.view),
            temp(t.i32),
        ],
        vec![
            block(
                b0,
                Terminator::Branch {
                    cond: 2,
                    then: 1,
                    else_: 2,
                },
            ),
            block(
                vec![MirInstr::Move { dst: 3, src: 0 }],
                Terminator::Jump { target: 2 },
            ),
            block(
                vec![
                    MirInstr::FieldGet {
                        dst: 4,
                        base: 0,
                        path: vec![0],
                    },
                    MirInstr::StrLen { dst: 5, view: 4 },
                    MirInstr::BorrowEnd { token: 4 },
                ],
                Terminator::Return { value: 5 },
            ),
        ],
    );
    verify_reject(&f.m, "use-after-move");
}

/// Loop back-edge MAY-move: even a zero-iteration-capable loop body that
/// moves the root reddens the after-loop read (conservative union).
#[test]
fn av_owned_v11_loop_back_edge_move_conservative() {
    let mut f = F::new("v11b");
    let t = f.t;
    let c_auto = f.s("Auto");
    let c_true = f.b(true);
    let mut b0 = f.ctor_note(c_auto);
    b0.push(MirInstr::Const {
        dst: 2,
        value: c_true,
    });
    f.main(
        vec![
            local(t.note, "u"),
            temp(t.str),
            temp(t.bool),
            temp(t.note),
            temp(t.view),
            temp(t.i32),
        ],
        vec![
            block(
                b0,
                Terminator::Branch {
                    cond: 2,
                    then: 1,
                    else_: 2,
                },
            ),
            block(
                vec![MirInstr::Move { dst: 3, src: 0 }],
                Terminator::Branch {
                    cond: 2,
                    then: 1,
                    else_: 2,
                },
            ),
            block(
                vec![
                    MirInstr::FieldGet {
                        dst: 4,
                        base: 0,
                        path: vec![0],
                    },
                    MirInstr::StrLen { dst: 5, view: 4 },
                    MirInstr::BorrowEnd { token: 4 },
                ],
                Terminator::Return { value: 5 },
            ),
        ],
    );
    verify_reject(&f.m, "use-after-move");
}

/// CALL arg-shape mismatch: passing a record where the callee wants a Str
/// is a verify-side type rejection (`mir.call-arg-type`).
#[test]
fn av_owned_v11_call_arg_shape_mismatch_rejected() {
    let mut f = F::new("v11c");
    let t = f.t;
    let c_auto = f.s("Auto");
    // callee: fn(Str) -> i32 — len of a view of its owned param.
    let callee = f.extra(
        vec![t.str],
        t.i32,
        vec![param(t.str, "s"), temp(t.view), temp(t.i32)],
        vec![block(
            vec![
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::StrLen { dst: 2, view: 1 },
                MirInstr::BorrowEnd { token: 1 },
            ],
            Terminator::Return { value: 2 },
        )],
    );
    let mut instrs = f.ctor_note(c_auto);
    instrs.push(MirInstr::Call {
        callee: CalleeRef::Func(callee),
        dst: 2,
        args: vec![0], // the Note record where Str is required
    });
    f.main(
        vec![local(t.note, "u"), temp(t.str), temp(t.i32)],
        vec![block(instrs, Terminator::Return { value: 2 })],
    );
    verify_reject(&f.m, "call-arg-type");
}

/// Trap cleanup order (V-04/V-11 runtime slice): a str-index trap mid-run
/// with live owned buffers cleans EVERYTHING exactly once, in reverse
/// creation order (LIFO).
#[test]
fn av_owned_v11_trap_cleans_owned_in_creation_order() {
    let mut f = F::new("v11d");
    let t = f.t;
    let c_aa = f.s("AA");
    let c_b = f.s("B");
    let c_5 = f.i(5);
    f.main(
        vec![
            local(t.str, "s"),
            temp(t.view),
            temp(t.i32),
            temp(t.char),
            local(t.str, "s2"),
        ],
        vec![block(
            vec![
                MirInstr::Const {
                    dst: 0,
                    value: c_aa,
                },
                MirInstr::View { dst: 1, src: 0 },
                MirInstr::Const { dst: 4, value: c_b },
                MirInstr::Const { dst: 2, value: c_5 },
                MirInstr::StrCharAt {
                    dst: 3,
                    view: 1,
                    index: 2,
                },
            ],
            Terminator::Return { value: 2 },
        )],
    );
    mir_verify::verify_mir(&f.m).expect("verifies");
    let wire = encode_abc(&f.m).expect("encode");
    let decoded = decode_abc(&wire.to_flat()).expect("decode");
    let loaded = abc_verify::verify_abc(decoded, &f.m.owner).expect("admits");
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(matches!(out, ExecOutcome::Trap { .. }));
    let arena = trace.arena.expect("arena");
    let allocs = arena.alloc_events();
    assert_eq!(
        allocs.iter().map(|a| a.bytes).collect::<Vec<_>>(),
        vec![2, 1],
        "\"AA\" then \"B\""
    );
    let frees = arena.free_events();
    let mut creation_desc: Vec<u64> = allocs.iter().map(|a| a.id).collect();
    creation_desc.sort_unstable_by(|a, b| b.cmp(a));
    assert_eq!(
        frees.iter().map(|e| e.id).collect::<Vec<_>>(),
        creation_desc,
        "every buffer freed exactly once, last-created first"
    );
    assert_eq!(arena.live(), 0);
}

/// Sparse field paths: a 4-field record whose write touches only field 3
/// verifies and executes (MovePath grows by touched paths only).
#[test]
fn av_owned_v11_sparse_field_paths_execute() {
    let mut f = F::new("v11e");
    let t = f.t;
    let t_wide = f.m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "v11e".into(),
        name: "Wide".into(),
        fields: vec![
            MirFieldDef {
                name: "a".into(),
                ty: t.i32,
            },
            MirFieldDef {
                name: "b".into(),
                ty: t.str,
            },
            MirFieldDef {
                name: "c".into(),
                ty: t.i32,
            },
            MirFieldDef {
                name: "d".into(),
                ty: t.str,
            },
        ],
    }));
    let c_x = f.s("x");
    let c_yy = f.s("yy");
    let c_yy2 = f.s("yy");
    let c_1 = f.i(1);
    let c_2 = f.i(2);
    f.main(
        vec![
            local(t_wide, "w"),
            temp(t.str),
            temp(t.str),
            temp(t.i32),
            temp(t.i32),
            temp(t.view),
            temp(t.i32),
            temp(t.str),
        ],
        vec![block(
            vec![
                MirInstr::Const { dst: 1, value: c_x },
                MirInstr::Const {
                    dst: 2,
                    value: c_yy,
                },
                MirInstr::Const { dst: 3, value: c_1 },
                MirInstr::Const { dst: 4, value: c_2 },
                MirInstr::Aggregate {
                    dst: 0,
                    record_ty: t_wide,
                    operands: vec![3, 1, 4, 2],
                },
                // write ONLY field 3 (d) — the sparse path; the RHS
                // re-materializes first (the ctor consumed the literal).
                MirInstr::Const {
                    dst: 7,
                    value: c_yy2,
                },
                MirInstr::FieldSet {
                    root: 0,
                    path: vec![3],
                    value: 7,
                },
                MirInstr::FieldGet {
                    dst: 5,
                    base: 0,
                    path: vec![3],
                },
                MirInstr::StrLen { dst: 6, view: 5 },
                MirInstr::BorrowEnd { token: 5 },
            ],
            Terminator::Return { value: 6 },
        )],
    );
    let (value, _) = admit_exec(&f.m);
    assert_eq!(value, Value::I32(2), "field d = yy");
}

/// MovePath analysis over the frozen verify-memory budget is a Resource
/// rejection (allocated-before-analyzing), never a panic.
#[test]
fn av_owned_v11_analysis_over_budget_is_resource() {
    let mut f = F::new("v11f");
    let t = f.t;
    // 32k slots x 16k linear jump blocks: blocks x slot-words x 32
    // exceeds the 16MiB matrix bound while slots+blocks stay inside the
    // 65536 per-function table family.
    let n_slots = 32_000usize;
    let n_blocks = 16_000usize;
    let mut slots = Vec::with_capacity(n_slots);
    for _ in 0..n_slots {
        slots.push(temp(t.i32));
    }
    let mut blocks = Vec::with_capacity(n_blocks);
    for i in 0..n_blocks - 1 {
        blocks.push(block(
            vec![],
            Terminator::Jump {
                target: i as u32 + 1,
            },
        ));
    }
    blocks.push(block(vec![], Terminator::Return { value: 0 }));
    f.main(slots, blocks);
    let err = mir_verify::verify_mir(&f.m).expect_err("oversized analysis rejected");
    match err {
        mir_verify::MirVerifyError::Resource { .. } => {}
        other => panic!("expected a Resource rejection, got {other:?}"),
    }
}

// ---- V-12: arm-specific init/borrow merges ---------------------------------

/// Single-arm-initialized unpublished temp: a temp defined in only one arm
/// and NEVER read at the join is legal; READING it at the join is
/// `mir.uninitialized-read` (must-∩ init).
#[test]
fn av_owned_v12_single_arm_temp_read_rules() {
    fn build(read_at_join: bool) -> MirModule {
        let mut f = F::new("v12a");
        let t = f.t;
        let c_7 = f.i(7);
        let c_5 = f.i(5);
        let c_true = f.b(true);
        f.main(
            vec![
                temp(t.bool),
                temp(t.i32), // 1: maybe-init temp
                temp(t.i32), // 2: always-init
                temp(t.i32), // 3: result
            ],
            vec![
                block(
                    vec![
                        MirInstr::Const { dst: 2, value: c_7 },
                        MirInstr::Const {
                            dst: 0,
                            value: c_true,
                        },
                    ],
                    Terminator::Branch {
                        cond: 0,
                        then: 1,
                        else_: 2,
                    },
                ),
                block(
                    vec![MirInstr::Const { dst: 1, value: c_5 }],
                    Terminator::Jump { target: 2 },
                ),
                block(
                    if read_at_join {
                        vec![MirInstr::Copy { dst: 3, src: 1 }]
                    } else {
                        vec![]
                    },
                    Terminator::Return { value: 2 },
                ),
            ],
        );
        f.m
    }
    mir_verify::verify_mir(&build(false)).expect("unpublished single-arm temp is legal");
    verify_reject(&build(true), "uninitialized-read");
}

/// One-arm live VIEW + join FIELD_SET = mut-conflict (may-union of live
/// tokens); the same shape with BORROW_END inside the arm is legal.
#[test]
fn av_owned_v12_one_arm_view_blocks_join_mut_until_ended() {
    fn build(end_in_arm: bool) -> MirModule {
        let mut f = F::new("v12b");
        let t = f.t;
        let c_auto = f.s("Auto");
        let c_true = f.b(true);
        let c_b = f.s("B");
        let mut b0 = f.ctor_note(c_auto);
        b0.push(MirInstr::Const {
            dst: 2,
            value: c_true,
        });
        let mut arm = vec![MirInstr::FieldGet {
            dst: 3,
            base: 0,
            path: vec![0],
        }];
        if end_in_arm {
            arm.push(MirInstr::BorrowEnd { token: 3 });
        }
        f.main(
            vec![
                local(t.note, "u"),
                temp(t.str),
                temp(t.bool),
                temp(t.view),
                temp(t.str),
                temp(t.view),
                temp(t.i32),
            ],
            vec![
                block(
                    b0,
                    Terminator::Branch {
                        cond: 2,
                        then: 1,
                        else_: 2,
                    },
                ),
                block(arm, Terminator::Jump { target: 2 }),
                block(
                    vec![
                        MirInstr::Const { dst: 4, value: c_b },
                        MirInstr::FieldSet {
                            root: 0,
                            path: vec![0],
                            value: 4,
                        },
                        MirInstr::FieldGet {
                            dst: 5,
                            base: 0,
                            path: vec![0],
                        },
                        MirInstr::StrLen { dst: 6, view: 5 },
                        MirInstr::BorrowEnd { token: 5 },
                    ],
                    Terminator::Return { value: 6 },
                ),
            ],
        );
        f.m
    }
    verify_reject(&build(false), "mut-conflict");
    let (value, _) = admit_exec(&build(true));
    assert_eq!(
        value,
        Value::I32(1),
        "join mut after ended borrow: body \"B\""
    );
}

/// Two-arm borrow-then-end + join mut is legal (V-12's positive).
#[test]
fn av_owned_v12_two_arm_end_borrow_then_mut_legal() {
    let mut f = F::new("v12c");
    let t = f.t;
    let c_auto = f.s("Auto");
    let c_true = f.b(true);
    let c_bb = f.s("BB");
    let mut b0 = f.ctor_note(c_auto);
    b0.push(MirInstr::Const {
        dst: 2,
        value: c_true,
    });
    f.main(
        vec![
            local(t.note, "u"),
            temp(t.str),
            temp(t.bool),
            temp(t.view), // 3: arm A view
            temp(t.view), // 4: arm B view
            temp(t.str),  // 5: RHS literal
            temp(t.view), // 6: post view
            temp(t.i32),  // 7: result
        ],
        vec![
            block(
                b0,
                Terminator::Branch {
                    cond: 2,
                    then: 1,
                    else_: 2,
                },
            ),
            block(
                vec![
                    MirInstr::FieldGet {
                        dst: 3,
                        base: 0,
                        path: vec![0],
                    },
                    MirInstr::BorrowEnd { token: 3 },
                ],
                Terminator::Jump { target: 3 },
            ),
            block(
                vec![
                    MirInstr::FieldGet {
                        dst: 4,
                        base: 0,
                        path: vec![0],
                    },
                    MirInstr::BorrowEnd { token: 4 },
                ],
                Terminator::Jump { target: 3 },
            ),
            block(
                vec![
                    MirInstr::Const {
                        dst: 5,
                        value: c_bb,
                    },
                    MirInstr::FieldSet {
                        root: 0,
                        path: vec![0],
                        value: 5,
                    },
                    MirInstr::FieldGet {
                        dst: 6,
                        base: 0,
                        path: vec![0],
                    },
                    MirInstr::StrLen { dst: 7, view: 6 },
                    MirInstr::BorrowEnd { token: 6 },
                ],
                Terminator::Return { value: 7 },
            ),
        ],
    );
    let (value, _) = admit_exec(&f.m);
    assert_eq!(value, Value::I32(2), "both arms ended borrows: join mut ok");
}

/// Zero-or-many loop: a body that VIEWs and ENDs each iteration leaves
/// the join free to mut (legal); the same loop whose body MOVES the root
/// reddens the after-loop read even though zero iterations are possible.
#[test]
fn av_owned_v12_zero_or_many_loop_borrow_and_move_faces() {
    fn build_loop(body_move: bool) -> MirModule {
        let mut f = F::new("v12d");
        let t = f.t;
        let c_auto = f.s("Auto");
        let c_true = f.b(true);
        let c_b = f.s("B");
        let c_zero = f.i(0);
        let c_one = f.i(1);
        let c_three = f.i(3);
        let mut b0 = f.ctor_note(c_auto);
        b0.push(MirInstr::Const {
            dst: 2,
            value: c_true,
        });
        b0.push(MirInstr::Const {
            dst: 8,
            value: c_zero,
        });
        b0.push(MirInstr::Const {
            dst: 9,
            value: c_one,
        });
        b0.push(MirInstr::Const {
            dst: 10,
            value: c_three,
        });
        let mut body = if body_move {
            vec![MirInstr::Move { dst: 4, src: 0 }]
        } else {
            vec![
                MirInstr::FieldGet {
                    dst: 3,
                    base: 0,
                    path: vec![0],
                },
                MirInstr::BorrowEnd { token: 3 },
            ]
        };
        // n = n + 1; keep_going = n < 3 — the loop runs exactly 3 times.
        body.push(MirInstr::Bin {
            op: BinOp::AddI32,
            dst: 8,
            lhs: 8,
            rhs: 9,
        });
        body.push(MirInstr::Bin {
            op: BinOp::LtI32,
            dst: 2,
            lhs: 8,
            rhs: 10,
        });
        f.main(
            vec![
                local(t.note, "u"),
                temp(t.str),
                temp(t.bool), // 2: loop condition
                temp(t.view), // 3: body view
                temp(t.note), // 4: body move dst
                temp(t.str),  // 5: RHS
                temp(t.view), // 6: post view
                temp(t.i32),  // 7: result
                temp(t.i32),  // 8: counter n
                temp(t.i32),  // 9: one
                temp(t.i32),  // 10: three
            ],
            vec![
                block(
                    b0,
                    Terminator::Branch {
                        cond: 2,
                        then: 1,
                        else_: 2,
                    },
                ),
                block(
                    body,
                    Terminator::Branch {
                        cond: 2,
                        then: 1,
                        else_: 2,
                    },
                ),
                block(
                    vec![
                        MirInstr::Const { dst: 5, value: c_b },
                        MirInstr::FieldSet {
                            root: 0,
                            path: vec![0],
                            value: 5,
                        },
                        MirInstr::FieldGet {
                            dst: 6,
                            base: 0,
                            path: vec![0],
                        },
                        MirInstr::StrLen { dst: 7, view: 6 },
                        MirInstr::BorrowEnd { token: 6 },
                    ],
                    Terminator::Return { value: 7 },
                ),
            ],
        );
        f.m
    }
    let (value, _) = admit_exec(&build_loop(false));
    assert_eq!(
        value,
        Value::I32(1),
        "ended borrows per iteration: join mut ok"
    );
    verify_reject(&build_loop(true), "use-after-move");
}
