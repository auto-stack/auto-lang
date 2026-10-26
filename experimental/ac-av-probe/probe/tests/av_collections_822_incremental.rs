//! PLAN-822 T-09 incremental tests (V-13) over the REAL update consumer
//! (`collections_session`): two-function revisions through the real
//! verify→encode→abc-admit→link→execute chain with semantics + cleanup
//! observation, unchanged-unit production/IO counters, real
//! `fn:<name>` dependency changes (positive carry + link rejection), and
//! the production structural gate that FAILS on a forced whole rebuild.
//! Prefix `av_collections_822`.

use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::mir::{
    BinOp, ListTypeRef, MirBlock, MirConst, MirFunction, MirImport, MirInstr, MirModule, MirSlot,
    MirType, OwnerScope, SlotKind, Terminator,
};
use auto_av_core::mir_verify::verify_mir;
use auto_av_core::program_snapshot::ProgramSnapshot;
use auto_av_probe::collections_session::CollectionsUpdateSession;
use std::sync::Arc;

fn owner_scope() -> OwnerScope {
    OwnerScope {
        schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
        module: "collections822".to_string(),
    }
}

/// `fn main() i32 { beta() + a_bump }` — a REAL cross-unit caller: the
/// callee is addressed through the `fn:beta` import (the unit model's
/// only legal callee form), resolved and signature-checked at link time.
fn alpha_unit(a_bump: i32) -> MirModule {
    let mut m = MirModule::collections_unit(owner_scope());
    let t_i32 = m.intern_type(MirType::I32);
    let t_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: t_i32,
    }));
    let cb = m.intern_const(MirConst::I32(a_bump)) as u32;
    m.imports = vec![MirImport {
        symbol: "fn:beta".to_string(),
        effect: auto_av_core::mir::Effect::Pure,
        sig: t_sig,
    }];
    m.funcs = vec![MirFunction {
        owner: owner_scope(),
        name: "main".to_string(),
        sig: t_sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: t_i32,
                name: Some("call".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t_i32,
                name: Some("bump".into()),
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Call {
                    callee: auto_av_core::mir::CalleeRef::Import(0),
                    dst: 0,
                    args: vec![],
                },
                MirInstr::Const { dst: 1, value: cb },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 0,
                    lhs: 0,
                    rhs: 1,
                },
            ],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: a_bump as u64,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

/// `fn beta() i32 { list-roundtrip(b_bump) }` — the collections
/// capability under update (a typed list materializes the bump and reads
/// it back); `b_bump` varying = a body-only revision.
fn beta_unit(b_bump: i32) -> MirModule {
    let mut m = MirModule::collections_unit(owner_scope());
    let t_i32 = m.intern_type(MirType::I32);
    let t_list = m.intern_type(MirType::List(ListTypeRef { elem: t_i32 }));
    let t_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: t_i32,
    }));
    let cb = m.intern_const(MirConst::I32(b_bump)) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    m.funcs = vec![MirFunction {
        owner: owner_scope(),
        name: "beta".to_string(),
        sig: t_sig,
        slots: vec![
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
                name: Some("idx".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t_i32,
                name: Some("got".into()),
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 1, value: cb },
                MirInstr::ListNew {
                    dst: 0,
                    list_ty: t_list,
                    cap: 0,
                },
                MirInstr::ListPush { root: 0, value: 1 },
                MirInstr::Const { dst: 2, value: c0 },
                MirInstr::ListGet {
                    dst: 3,
                    base: 0,
                    index: 2,
                },
            ],
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: b_bump as u64,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

/// The dependency-change shape: `fn beta() bool` — a LEGAL unit on its
/// own, but alpha's `fn:beta` import still declares `() -> i32`, so the
/// LINK (the real interface check) must refuse the pair.
fn beta_unit_bool() -> MirModule {
    let mut m = MirModule::collections_unit(owner_scope());
    let t_bool = m.intern_type(MirType::Bool);
    let t_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: t_bool,
    }));
    let ctrue = m.intern_const(MirConst::Bool(true)) as u32;
    m.funcs = vec![MirFunction {
        owner: owner_scope(),
        name: "beta".to_string(),
        sig: t_sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: t_bool,
            name: Some("r".into()),
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const {
                dst: 0,
                value: ctrue,
            }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 999,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

fn run_snapshot(snap: &ProgramSnapshot) -> i32 {
    use auto_av_core::contract::{ExecOutcome, Value};
    let (out, _) = execute_program(snap, &ExecLimits::default(), &CancelFlag::new(false));
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("snapshot execution: {other:?}"),
    }
}

/// V-13 sub-case 2 — the REAL update entry: install alpha(a=10) +
/// beta(b=1), then three beta body revisions. EVERY revision must:
/// execute to the folded truth (a + b), clean up (live=0,
/// allocs == frees), keep alpha's code OBJECT carried by the session,
/// and produce NOTHING for alpha (verified/encoded/admitted/store-writes
/// all == 1 — beta only). The OLD snapshot keeps executing its OLD truth.
#[test]
fn av_collections_822_incremental_body_edit_identity() {
    let root = std::env::temp_dir().join(format!(
        "av822-session-body-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let store = root.join("store");
    let mut s = CollectionsUpdateSession::new(Some(&store), "main");

    let out0 = s
        .apply_revision(&[("main", alpha_unit(10)), ("beta", beta_unit(1))])
        .expect("install");
    assert_eq!(out0.value, Some(11), "x=0: beta(0)=1, alpha=+10");
    assert_eq!(out0.live_after_release, 0, "cleanup: live drains");
    assert_eq!(out0.allocs, out0.frees, "cleanup: ledger balanced");
    assert_eq!(out0.production.verified, 2, "both units produced once");
    assert_eq!(out0.production.store_writes, 2);
    assert!(out0.gate.pass(), "a fresh install has no degraded units");
    let alpha_obj = s.unit("main").expect("alpha installed").clone();
    let snap_old = s.snapshot().expect("old snapshot links");

    for (b, want) in [(2, 12), (3, 13), (4, 14)] {
        let out = s
            .apply_revision(&[("beta", beta_unit(b))])
            .unwrap_or_else(|e| panic!("beta revision b={b}: {e}"));
        assert_eq!(out.value, Some(want), "revision b={b} semantics");
        assert_eq!(out.live_after_release, 0, "b={b}: live drains");
        assert_eq!(out.allocs, out.frees, "b={b}: ledger balanced");
        // Production/IO of the UNCHANGED unit is ZERO across every stage.
        assert_eq!(out.production.verified, 1, "b={b}: only beta verified");
        assert_eq!(out.production.encoded, 1, "b={b}: only beta encoded");
        assert_eq!(out.production.admitted, 1, "b={b}: only beta admitted");
        assert_eq!(out.production.store_writes, 1, "b={b}: only beta persisted");
        assert_eq!(out.gate.rebuilt_units, 1);
        assert_eq!(out.gate.unchanged_degraded, 0, "b={b}: honest revision");
        assert!(out.gate.pass());
        // Identity: the session carried the SAME alpha object.
        assert!(
            Arc::ptr_eq(s.unit("main").unwrap(), &alpha_obj),
            "b={b}: alpha's code object is carried, not rebuilt"
        );
        // The OLD snapshot still executes its OLD truth (beta=1 → 11).
        assert_eq!(
            run_snapshot(&snap_old),
            11,
            "b={b}: old snapshot keeps its truth"
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// V-13 sub-case 3 — real dependency changes: the POSITIVE control (a
/// body-only beta revision keeps alpha carried and executes) and the
/// NEGATIVE control (a legal beta unit whose SIGNATURE changed makes the
/// LINK refuse the pair — the real interface check, not a malformed
/// MIR); after the refusal the session's prior state still updates fine.
#[test]
fn av_collections_822_incremental_signature_change_precise() {
    let mut s = CollectionsUpdateSession::new(None, "main");
    s.apply_revision(&[("main", alpha_unit(5)), ("beta", beta_unit(1))])
        .expect("install");
    let alpha_obj = s.unit("main").unwrap().clone();

    // Positive: a body-only revision carries alpha and executes.
    let ok = s
        .apply_revision(&[("beta", beta_unit(2))])
        .expect("body revision");
    assert_eq!(ok.value, Some(7));
    assert!(Arc::ptr_eq(s.unit("main").unwrap(), &alpha_obj));
    assert!(ok.gate.pass());

    // Negative: beta's SIGNATURE changed (i32 -> bool). The unit itself
    // passes the real gates; the LINK refuses the interface mismatch.
    let err = s
        .apply_revision(&[("beta", beta_unit_bool())])
        .expect_err("the link must refuse the signature change");
    assert!(
        err.contains("signature mismatch") || err.contains("does not resolve"),
        "the real link check speaks: {err}"
    );

    // The refusal left the session's prior state intact — a subsequent
    // honest revision still succeeds on the carried units.
    let after = s
        .apply_revision(&[("beta", beta_unit(3))])
        .expect("the session survives the refused revision");
    assert_eq!(after.value, Some(8));
    assert!(Arc::ptr_eq(s.unit("main").unwrap(), &alpha_obj));
}

/// V-13 sub-case 4 — the forced-whole counter-control on the PRODUCTION
/// gate: re-submitting alpha UNCHANGED (identical content) alongside a
/// real beta edit swaps alpha's object under the same fingerprint — the
/// gate reports it degraded-unchanged (NON-ZERO) and FAILS, while the
/// honest revision's gate passes. The gate lives in the session (the
/// production consumer), not in this test.
#[test]
fn av_collections_822_incremental_forced_degradation_gate_nonzero() {
    let mut s = CollectionsUpdateSession::new(None, "main");
    s.apply_revision(&[("main", alpha_unit(10)), ("beta", beta_unit(1))])
        .expect("install");

    // Honest: submit ONLY beta.
    let honest = s.apply_revision(&[("beta", beta_unit(2))]).expect("honest");
    assert_eq!(honest.gate.unchanged_degraded, 0);
    assert!(
        honest.gate.pass(),
        "the honest revision passes the production gate"
    );

    // Forced whole: re-submit alpha's IDENTICAL module with beta's edit —
    // alpha is re-produced (fresh object, same fingerprint).
    let forced = s
        .apply_revision(&[("main", alpha_unit(10)), ("beta", beta_unit(3))])
        .expect("the forced revision still executes (the gate judges, not the run)");
    assert_eq!(
        forced.value,
        Some(13),
        "semantics stay correct under the forced shape"
    );
    assert_eq!(
        forced.gate.unchanged_degraded, 1,
        "the forced-whole rebuild degrades exactly the unchanged unit"
    );
    assert_eq!(forced.gate.rebuilt_units, 2);
    assert!(
        !forced.gate.pass(),
        "the PRODUCTION gate fails non-zero on the forced whole rebuild"
    );
}

/// V-13 sub-case 1 — the frozen D828 baseline campaign replays GREEN
/// through the shared core AFTER the collections extension (dual gate:
/// all-correct + structural + P95 < 1000 ms). The scalar campaign is the
/// shared-core baseline; sub-cases 2/3/4 above cover the collections
/// capability itself through its real update entry.
#[test]
fn av_collections_822_incremental_baseline_campaign_replay() {
    use auto_av_probe::formal_campaign::{
        run_formal_campaign, CampaignConfig, FORMAL_GATE_MS, FORMAL_MIN_REVISIONS,
    };
    let cfg = CampaignConfig {
        revisions: FORMAL_MIN_REVISIONS,
        gate_ms: FORMAL_GATE_MS,
    };
    let worker = std::path::PathBuf::from(env!("CARGO_BIN_EXE_av_probe_worker"));
    let report = run_formal_campaign(&worker, &cfg).expect("the baseline campaign completes");
    assert!(report.failures.is_empty(), "{:?}", report.failures);
    assert!(
        report.all_correct,
        "every baseline revision matched its independent expectation"
    );
    assert!(report.structural_ok, "the structural gate stays green");
    assert!(report.gate_pass, "P95 < {} ms holds", FORMAL_GATE_MS);
}

/// F822-R2-01: a REJECTED candidate never becomes current — the
/// signature-changed beta passes its own gates but the LINK refuses the
/// pair; the session's CURRENT beta object, identity and snapshot stay
/// untouched and executable (result 7). The reviewer replay shape: query
/// the current object IMMEDIATELY after the refusal, before any other
/// submission.
#[test]
fn av_collections_822_incremental_rejected_revision_preserves_session() {
    let mut s = CollectionsUpdateSession::new(None, "main");
    let out = s
        .apply_revision(&[("main", alpha_unit(5)), ("beta", beta_unit(2))])
        .expect("install");
    assert_eq!(out.value, Some(7));
    let beta_obj = s.unit("beta").expect("beta installed").clone();
    let alpha_obj = s.unit("main").expect("alpha installed").clone();

    // The rejected candidate: a SEPARATELY VALID bool-returning beta.
    let err = s
        .apply_revision(&[("beta", beta_unit_bool())])
        .expect_err("the link must refuse the signature change");
    assert!(
        err.contains("signature mismatch") || err.contains("does not resolve"),
        "{err}"
    );

    // IMMEDIATELY after the refusal: the current beta object is UNCHANGED
    // (not the rejected candidate), the prior revision/identity stand, and
    // the CURRENT snapshot still links and executes to 7.
    assert!(
        Arc::ptr_eq(s.unit("beta").unwrap(), &beta_obj),
        "a rejected candidate must not become the current object"
    );
    assert!(Arc::ptr_eq(s.unit("main").unwrap(), &alpha_obj));
    let snap = s.snapshot().expect("the prior snapshot still links");
    assert_eq!(run_snapshot(&snap), 7);
}

/// F822-R2-01 second seam: an EXECUTION failure after a successful link
/// is also a non-installing candidate — the prior state survives.
#[test]
fn av_collections_822_incremental_execution_failure_preserves_session() {
    let mut s = CollectionsUpdateSession::new(None, "main");
    let out = s
        .apply_revision(&[("main", alpha_unit(5)), ("beta", beta_unit(2))])
        .expect("install");
    assert_eq!(out.value, Some(7));
    let beta_obj = s.unit("beta").unwrap().clone();

    // An executing-but-trapping beta (list index out of range at run
    // time): verify/link pass; the EXECUTION fails; nothing installs.
    let err = s
        .apply_revision(&[("beta", beta_trapping_unit())])
        .expect_err("the trapping revision must not install");
    assert!(err.contains("did not execute"), "{err}");
    assert!(
        Arc::ptr_eq(s.unit("beta").unwrap(), &beta_obj),
        "a run-failed candidate must not become the current object"
    );
    let snap = s.snapshot().expect("prior snapshot still links");
    assert_eq!(run_snapshot(&snap), 7);
}

/// The trapping beta shape: `fn beta() i32 { list[9] }` — legal MIR, real
/// gates pass, the RUNTIME traps (out-of-range read).
fn beta_trapping_unit() -> MirModule {
    let mut m = MirModule::collections_unit(owner_scope());
    let t_i32 = m.intern_type(MirType::I32);
    let t_list = m.intern_type(MirType::List(ListTypeRef { elem: t_i32 }));
    let t_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: t_i32,
    }));
    let c7 = m.intern_const(MirConst::I32(7)) as u32;
    let c9 = m.intern_const(MirConst::I32(9)) as u32;
    m.funcs = vec![MirFunction {
        owner: owner_scope(),
        name: "beta".to_string(),
        sig: t_sig,
        slots: vec![
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
                name: Some("i".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t_i32,
                name: Some("got".into()),
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 1, value: c7 },
                MirInstr::ListNew {
                    dst: 0,
                    list_ty: t_list,
                    cap: 0,
                },
                MirInstr::ListPush { root: 0, value: 1 },
                MirInstr::Const { dst: 2, value: c9 },
                MirInstr::ListGet {
                    dst: 3,
                    base: 0,
                    index: 2,
                }, // traps at runtime
            ],
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: 777,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

// -- F822-R4-01: Place referent type-table validation ---------------------

/// A beta unit carrying ONE extra unused type-table entry
/// Place{elem: <referent>}. The body itself is the legal list roundtrip;
/// the malformed (or legal-control) referent lives in the table only —
/// exactly the reviewer's minimal input shape.
fn beta_with_place_entry(b_bump: i32, referent: u32) -> MirModule {
    let mut m = beta_unit(b_bump);
    let idx = m.intern_type(MirType::Place(ListTypeRef { elem: referent })) as u32;
    let _ = idx;
    m
}

/// The reviewer replay: an out-of-range referent (u32::MAX, an unused
/// entry) must be refused at the MIR/ABC admission gate — the REAL
/// update consumer returns Err (no FunctionCode, no link) and never
/// panics; the legal Place<i32> entry travels the identical path and
/// executes to 7.
#[test]
fn av_collections_822_incremental_place_referent_out_of_range_refused() {
    // The real gates reject the malformed table at verify.
    let bad = beta_with_place_entry(7, u32::MAX);
    match verify_mir(&bad) {
        Err(auto_av_core::mir_verify::MirVerifyError::Invalid { code, .. }) => {
            assert_eq!(code, "mir.type-ref");
        }
        other => panic!("out-of-range referent must be refused: {other:?}"),
    }
    // The REAL consumer surfaces the refusal as an Err — no panic.
    let mut s = CollectionsUpdateSession::new(None, "main");
    s.apply_revision(&[("main", alpha_unit(0)), ("beta", beta_unit(1))])
        .expect("install a legal session first");
    let err = s
        .apply_revision(&[("beta", beta_with_place_entry(7, u32::MAX))])
        .expect_err("the malformed candidate must be refused");
    assert!(err.contains("verify beta"), "{err}");
    // And the prior session still executes.
    let snap = s.snapshot().expect("prior snapshot links");
    assert_eq!(run_snapshot(&snap), 1);

    // The legal control: a Place<i32> entry (referent = the i32 index 0)
    // travels the identical produce/link/execute path and returns 7.
    let legal = beta_with_place_entry(7, 0);
    let mut s2 = CollectionsUpdateSession::new(None, "main");
    let out = s2
        .apply_revision(&[("main", alpha_unit(0)), ("beta", legal)])
        .expect("the legal Place entry admits");
    assert_eq!(out.value, Some(7));
}

/// Boundary and domain siblings: exactly-at-len referent, a container
/// referent (Place over a List) and a self/cyclic referent are all
/// refused deterministically at the type-table pass.
#[test]
fn av_collections_822_incremental_place_referent_boundary_and_domain() {
    // Boundary: elem == the table length AFTER interning points at the
    // Place's own slot (a self reference — the domain check refuses it;
    // one FURTHER past is a hard out-of-range).
    let base = beta_unit(7);
    let len = base.types.len() as u32;
    let mut selfref = base.clone();
    selfref.intern_type(MirType::Place(ListTypeRef { elem: len }));
    match verify_mir(&selfref) {
        Err(auto_av_core::mir_verify::MirVerifyError::Unsupported { code, .. }) => {
            assert_eq!(code, "mir.type.place-elem");
        }
        other => panic!("self-at-boundary referent must be refused: {other:?}"),
    }
    let mut boundary = base.clone();
    boundary.intern_type(MirType::Place(ListTypeRef { elem: len + 1 }));
    match verify_mir(&boundary) {
        Err(auto_av_core::mir_verify::MirVerifyError::Invalid { code, .. }) => {
            assert_eq!(code, "mir.type-ref");
        }
        other => panic!("boundary referent must be refused: {other:?}"),
    }

    // Domain: a Place whose referent is a List type (an unsupported
    // element kind — no list can ever hold lists).
    let mut domain = beta_unit(7);
    let list_idx = domain.intern_type(MirType::List(ListTypeRef { elem: 0 }));
    domain.intern_type(MirType::Place(ListTypeRef { elem: list_idx }));
    match verify_mir(&domain) {
        Err(auto_av_core::mir_verify::MirVerifyError::Unsupported { code, .. }) => {
            assert_eq!(code, "mir.type.place-elem");
        }
        other => panic!("container referent must be refused: {other:?}"),
    }

    // Self/cyclic: the Place's own table slot as its referent (the
    // element domain rejects Place kinds, covering the cycle).
    let mut cyc = beta_unit(7);
    let slot = cyc.types.len() as u32;
    cyc.intern_type(MirType::Place(ListTypeRef { elem: slot }));
    match verify_mir(&cyc) {
        Err(auto_av_core::mir_verify::MirVerifyError::Unsupported { code, .. }) => {
            assert_eq!(code, "mir.type.place-elem");
        }
        other => panic!("self referent must be refused: {other:?}"),
    }
}

// -- F822-R17-01: V-13 sub-case 3 layout & host-grant axes ---------------

/// F822-R17-01 kit: beta variants whose INTERNAL LAYOUT changes while the
/// interface stays `() -> i32` — the list element type moves i32 → Str
/// (a collections layout fact: the unit's type-table/data layout), still
/// executing the same folded truth family.
fn beta_unit_layout(b_bump: i32) -> MirModule {
    let mut m = MirModule::collections_unit(owner_scope());
    let t_i32 = m.intern_type(MirType::I32);
    let t_str = m.intern_type(MirType::Str);
    let t_list = m.intern_type(MirType::List(ListTypeRef { elem: t_str }));
    let t_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: t_i32,
    }));
    let cb = m.intern_const(MirConst::I32(b_bump)) as u32;
    let cs = m.intern_const(MirConst::Str(Box::from(*b"x"))) as u32;
    m.funcs = vec![MirFunction {
        owner: owner_scope(),
        name: "beta".to_string(),
        sig: t_sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Local,
                ty: t_list,
                name: Some("list".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t_str,
                name: Some("v".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t_i32,
                name: Some("len".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t_i32,
                name: Some("r".into()),
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 1, value: cs },
                MirInstr::ListNew {
                    dst: 0,
                    list_ty: t_list,
                    cap: 0,
                },
                MirInstr::ListPush { root: 0, value: 1 },
                MirInstr::ListLen { dst: 2, src: 0 },
                MirInstr::Const { dst: 3, value: cb },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 2,
                    lhs: 2,
                    rhs: 3,
                },
            ],
            term: Terminator::Return { value: 2 },
        }],
        entry: 0,
        source_snapshot: b_bump as u64,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

/// F822-R17-01 kit: a cross-unit pair whose INTERFACE carries a
/// collections LAYOUT fact — alpha passes a `List<i32>` argument through
/// its `fn:beta` import (`(List<i32>) -> i32`).
fn alpha_unit_list() -> MirModule {
    let mut m = MirModule::collections_unit(owner_scope());
    let t_i32 = m.intern_type(MirType::I32);
    let t_list = m.intern_type(MirType::List(ListTypeRef { elem: t_i32 }));
    let t_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![t_list],
        result: t_i32,
    }));
    let c7 = m.intern_const(MirConst::I32(7)) as u32;
    let t_entry = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: t_i32,
    }));
    m.imports = vec![MirImport {
        symbol: "fn:beta".to_string(),
        effect: auto_av_core::mir::Effect::Pure,
        sig: t_sig,
    }];
    m.funcs = vec![MirFunction {
        owner: owner_scope(),
        name: "main".to_string(),
        sig: t_entry,
        slots: vec![
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
                name: Some("got".into()),
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::ListNew {
                    dst: 0,
                    list_ty: t_list,
                    cap: 0,
                },
                MirInstr::Const { dst: 1, value: c7 },
                MirInstr::ListPush { root: 0, value: 1 },
                MirInstr::Call {
                    callee: auto_av_core::mir::CalleeRef::Import(0),
                    dst: 2,
                    args: vec![0],
                },
            ],
            term: Terminator::Return { value: 2 },
        }],
        entry: 0,
        source_snapshot: 11,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

/// F822-R17-01 kit: beta taking a LIST parameter — `(elem) -> i32`
/// returning the list length. `elem_str` selects the element layout:
/// `List<i32>` (the compatible interface twin) or `List<Str>` (a LEGAL
/// unit whose param LAYOUT differs — the link must refuse the pair).
fn beta_unit_param(elem_str: bool) -> MirModule {
    let mut m = MirModule::collections_unit(owner_scope());
    let t_i32 = m.intern_type(MirType::I32);
    let t_elem = if elem_str {
        m.intern_type(MirType::Str)
    } else {
        t_i32
    };
    let t_list = m.intern_type(MirType::List(ListTypeRef { elem: t_elem }));
    let t_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![t_list],
        result: t_i32,
    }));
    m.funcs = vec![MirFunction {
        owner: owner_scope(),
        name: "beta".to_string(),
        sig: t_sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Param,
                ty: t_list,
                name: Some("list".into()),
            },
            MirSlot {
                kind: SlotKind::Local,
                ty: t_i32,
                name: Some("len".into()),
            },
        ],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::ListLen { dst: 1, src: 0 }],
            term: Terminator::Return { value: 1 },
        }],
        entry: 0,
        source_snapshot: if elem_str { 21 } else { 20 },
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

/// F822-R17-01 kit: beta with a DECLARED required capability (the input's
/// required-capabilities fact, separate from the host grants).
fn beta_unit_trace(b_bump: i32) -> MirModule {
    let mut m = beta_unit(b_bump);
    m.capabilities = vec![auto_av_core::contract::TRACE_CAPABILITY.to_string()];
    m
}

/// V-13 sub-case 3 LAYOUT axis, positive control: an INTERNAL layout
/// revision (beta's list element type i32 → Str; same `() -> i32`
/// interface) is a legal honest revision — only beta rebuilds (every
/// production stage == 1, alpha's code object carried, gate honest),
/// the new layout executes its folded truth and the prior snapshot
/// keeps the old one.
#[test]
fn av_collections_822_incremental_layout_internal_revision_precise() {
    let root = std::env::temp_dir().join(format!(
        "av822-session-layout-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let store = root.join("store");
    let mut s = CollectionsUpdateSession::new(Some(&store), "main");
    let out0 = s
        .apply_revision(&[("main", alpha_unit(10)), ("beta", beta_unit(1))])
        .expect("install");
    assert_eq!(out0.value, Some(11));
    assert!(out0.gate.pass());
    let alpha_obj = s.unit("main").expect("alpha installed").clone();
    let snap_old = s.snapshot().expect("old snapshot links");

    // The LAYOUT revision: List<Str> internally, interface unchanged.
    let out = s
        .apply_revision(&[("beta", beta_unit_layout(2))])
        .expect("internal layout revision is legal");
    assert_eq!(out.value, Some(13), "len(1) + b(2), alpha +10");
    assert_eq!(out.live_after_release, 0);
    assert_eq!(out.allocs, out.frees);
    assert_eq!(out.production.verified, 1, "only the layout-revised unit");
    assert_eq!(out.production.encoded, 1);
    assert_eq!(out.production.admitted, 1);
    assert_eq!(out.gate.rebuilt_units, 1);
    assert_eq!(out.gate.unchanged_degraded, 0);
    assert!(
        Arc::ptr_eq(s.unit("main").unwrap(), &alpha_obj),
        "alpha carried"
    );
    assert_eq!(
        run_snapshot(&snap_old),
        11,
        "prior snapshot keeps its truth"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// V-13 sub-case 3 LAYOUT axis, negative control: a collections layout
/// fact flowing through the INTERFACE — beta's param moves
/// `List<i32>` → `List<Str>` (a legal unit alone); alpha's `fn:beta`
/// import still declares `(List<i32>) -> i32`, so the LINK's
/// content-equal signature check refuses the pair; the session keeps
/// its prior revision, units and executable snapshot.
#[test]
fn av_collections_822_incremental_layout_interface_change_rejected() {
    let root = std::env::temp_dir().join(format!(
        "av822-session-layout-iface-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let store = root.join("store");
    let mut s = CollectionsUpdateSession::new(Some(&store), "main");
    let out0 = s
        .apply_revision(&[
            ("main", alpha_unit_list()),
            ("beta", beta_unit_param(false)),
        ])
        .expect("compatible container interface installs");
    assert_eq!(out0.value, Some(1), "len of alpha's one-element list");
    let beta_obj = s.unit("beta").expect("beta installed").clone();
    let snap_old = s.snapshot().expect("old snapshot links");

    let err = s
        .apply_revision(&[("beta", beta_unit_param(true))])
        .expect_err("a param layout change under a stable import must be refused");
    assert!(
        err.contains("signature"),
        "the link's signature check names the refusal: {err}"
    );
    assert!(
        Arc::ptr_eq(s.unit("beta").unwrap(), &beta_obj),
        "beta not swapped"
    );
    assert_eq!(
        run_snapshot(&snap_old),
        1,
        "the prior snapshot still executes its truth"
    );
    // The session remains USABLE after the refusal.
    let out = s
        .apply_revision(&[("beta", beta_unit_param(false))])
        .expect("re-submitting the compatible twin still installs");
    assert_eq!(out.value, Some(1));
    let _ = std::fs::remove_dir_all(&root);
}

/// V-13 sub-case 3 HOST-GRANT axis: the input's REQUIRED capabilities
/// and the CURRENT host grants are separate facts. A capability-requiring
/// beta installs under a live grant; the SAME unit's next revision under
/// a REVOKED grant set is refused before any installation (session state
/// intact); restoring the legal grant makes the revision apply again.
#[test]
fn av_collections_822_incremental_host_grant_lifecycle() {
    let root = std::env::temp_dir().join(format!(
        "av822-session-grant-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let store = root.join("store");
    let mut s = CollectionsUpdateSession::new(Some(&store), "main");
    let grant = vec![auto_av_core::contract::TRACE_CAPABILITY.to_string()];

    // (a) Legal allow: the grant set covers the required capability.
    s.set_host_grants(grant.clone());
    let out0 = s
        .apply_revision(&[("main", alpha_unit(10)), ("beta", beta_unit_trace(1))])
        .expect("granted capability admits");
    assert_eq!(out0.value, Some(11));
    let beta_obj = s.unit("beta").expect("beta installed").clone();
    let snap_old = s.snapshot().expect("old snapshot links");

    // (b) Revoked: the next revision of the SAME capability-requiring
    // unit under an EMPTY host grant set is refused — nothing installs.
    s.set_host_grants(vec![]);
    let err = s
        .apply_revision(&[("beta", beta_unit_trace(2))])
        .expect_err("a revoked grant must refuse the revision");
    assert!(
        err.contains("not granted"),
        "the refusal names the capability gap: {err}"
    );
    assert!(
        Arc::ptr_eq(s.unit("beta").unwrap(), &beta_obj),
        "failure installs nothing"
    );
    assert_eq!(run_snapshot(&snap_old), 11, "prior state still executes");

    // (c) Restoring the legal fact: the same revision applies.
    s.set_host_grants(grant);
    let out2 = s
        .apply_revision(&[("beta", beta_unit_trace(2))])
        .expect("re-granting admits the revision");
    assert_eq!(out2.value, Some(12));
    let _ = std::fs::remove_dir_all(&root);
}

/// F822-R18-01: the authorization preflight covers the COMPLETE candidate
/// closure — CARRIED units included. After a granted install of a
/// capability-requiring beta, a REVOKED grant set refuses any new
/// revision that would still execute the carried beta: a caller-only
/// edit AND an EMPTY submission both refuse with ZERO production, no
/// installation, the prior state intact; under a LIVE grant the same
/// carried shape succeeds (the reuse is legal — the grant is the fact
/// that changed); re-granting admits the previously refused revision.
/// Snapshots published under the earlier live grant keep executing
/// (authorization binds at revision time, not retroactively).
#[test]
fn av_collections_822_incremental_host_grant_carried_closure() {
    let root = std::env::temp_dir().join(format!(
        "av822-session-grant-carried-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let store = root.join("store");
    let mut s = CollectionsUpdateSession::new(Some(&store), "main");
    let grant = vec![auto_av_core::contract::TRACE_CAPABILITY.to_string()];

    s.set_host_grants(grant.clone());
    let out0 = s
        .apply_revision(&[("main", alpha_unit(10)), ("beta", beta_unit_trace(1))])
        .expect("granted install");
    assert_eq!(out0.value, Some(11));
    let beta_obj = s.unit("beta").expect("beta installed").clone();
    let snap_old = s.snapshot().expect("old snapshot links");

    // Revoked: a caller-only revision that would REUSE the carried
    // restricted beta refuses — zero production, nothing installs.
    s.set_host_grants(vec![]);
    for (label, submitted) in [
        ("caller-only", vec![("main", alpha_unit(20))]),
        ("empty", Vec::<(&str, MirModule)>::new()),
    ] {
        let err = s
            .apply_revision(&submitted)
            .err()
            .unwrap_or_else(|| panic!("{label}: the carried restricted unit must refuse"));
        assert!(
            err.contains("not granted"),
            "{label}: the refusal names the gap: {err}"
        );
        let c = s.production_counters();
        assert_eq!(
            (c.verified, c.encoded, c.admitted, c.store_writes),
            (0, 0, 0, 0),
            "{label}: a refused revision produces nothing"
        );
        assert!(
            Arc::ptr_eq(s.unit("beta").unwrap(), &beta_obj),
            "{label}: no swap"
        );
    }
    assert_eq!(
        run_snapshot(&snap_old),
        11,
        "the prior snapshot keeps executing"
    );

    // Live grant: the SAME carried shape succeeds (reuse is legal).
    s.set_host_grants(grant.clone());
    let out1 = s
        .apply_revision(&[("main", alpha_unit(20))])
        .expect("carried restricted unit under a live grant");
    assert_eq!(out1.value, Some(21));
    assert!(
        Arc::ptr_eq(s.unit("beta").unwrap(), &beta_obj),
        "beta carried"
    );
    assert_eq!(out1.live_after_release, 0);

    // Re-grant after a refusal: the previously refused edit applies.
    s.set_host_grants(vec![]);
    assert!(
        s.apply_revision(&[("main", alpha_unit(30))]).is_err(),
        "revoked again refuses"
    );
    s.set_host_grants(grant);
    let out2 = s
        .apply_revision(&[("main", alpha_unit(30))])
        .expect("re-granting admits the refused revision");
    assert_eq!(out2.value, Some(31));
    let _ = std::fs::remove_dir_all(&root);
}
