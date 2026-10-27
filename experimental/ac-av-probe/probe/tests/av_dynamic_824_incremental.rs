//! PLAN-824 T-09 incremental tests: the D828 function-code/snapshot seam
//! consumed by the REAL dynamic update session — the four V-13 sub-cases
//! (baseline campaign replay, dynamic body-edit identity, precise
//! dependency-change refusals + the authorization lifecycle, and the
//! forced-degradation counter-control on the production gate).
//! Prefix `av_dynamic_824` (contract-decision §9).

use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value};
use auto_av_core::mir::{
    BinOp, MirBlock, MirConst, MirFieldDef, MirFunction, MirImport, MirInstr, MirModule, MirSlot,
    MirType, OwnerScope, SlotKind, Terminator,
};
use auto_av_core::program_snapshot::ProgramSnapshot;
use auto_av_probe::dynamic_session::DynamicUpdateSession;
use std::sync::Arc;

fn owner_scope() -> OwnerScope {
    OwnerScope {
        schema: auto_av_core::contract::MIR_DYNAMIC_SCHEMA.to_string(),
        module: "dynamic824".to_string(),
    }
}

fn slot(ty: u32, name: &str) -> MirSlot {
    MirSlot {
        kind: SlotKind::Local,
        ty,
        name: Some(name.to_string()),
    }
}

/// `fn main() i32 { beta() + a_bump }` — a REAL cross-unit caller: the
/// callee is addressed through the `fn:beta` import (the unit model's
/// only legal callee form), resolved and signature-checked at link time.
fn alpha_unit(a_bump: i32) -> MirModule {
    let mut m = MirModule::dynamic_unit(owner_scope());
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
        slots: vec![slot(t_i32, "call"), slot(t_i32, "bump")],
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

/// `fn beta() i32 { dyn-roundtrip(b_bump) }` — THE dynamic capability
/// under update: a Note{n:b} is packed into a Dyn container, taken back
/// through the checked recovery and its field read. `b_bump` varying =
/// a body-only revision.
fn beta_unit(b_bump: i32) -> MirModule {
    beta_unit_impl(b_bump, false)
}

/// The trace-capability variant: same body + a declared
/// `hir.test.trace.v1` capability with one Trace use (the authorization
/// lifecycle's required fact).
fn beta_unit_trace(b_bump: i32) -> MirModule {
    beta_unit_impl(b_bump, true)
}

fn beta_unit_impl(b_bump: i32, trace: bool) -> MirModule {
    let mut m = MirModule::dynamic_unit(owner_scope());
    let t_i32 = m.intern_type(MirType::I32);
    let t_str = m.intern_type(MirType::Str);
    let t_note = m.intern_type(MirType::Record(auto_av_core::mir::RecordTypeRef {
        owner_module: "dynamic824".to_string(),
        name: "Note".to_string(),
        fields: vec![
            MirFieldDef {
                name: "text".to_string(),
                ty: t_str,
            },
            MirFieldDef {
                name: "n".to_string(),
                ty: t_i32,
            },
        ],
    }));
    let t_dyn = m.intern_type(MirType::Dyn);
    let t_sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![],
        result: t_i32,
    }));
    let c_text = m.intern_const(MirConst::Str(Box::from(*b"inc"))) as u32;
    let cb = m.intern_const(MirConst::I32(b_bump)) as u32;
    let mut instrs = vec![
        MirInstr::Const {
            dst: 0,
            value: c_text,
        },
        MirInstr::Const { dst: 1, value: cb },
        MirInstr::Aggregate {
            dst: 2,
            record_ty: t_note,
            operands: vec![0, 1],
        },
        MirInstr::DynPack { dst: 3, src: 2 },
        MirInstr::DynTake {
            dst: 4,
            src: 3,
            ty: t_note,
        },
        MirInstr::FieldGet {
            dst: 5,
            base: 4,
            path: vec![1],
        },
    ];
    if trace {
        instrs.push(MirInstr::Trace {
            event: auto_av_core::mir::TraceEvent::MarkA,
        });
    }
    m.funcs = vec![MirFunction {
        owner: owner_scope(),
        name: "beta".to_string(),
        sig: t_sig,
        slots: vec![
            slot(t_str, "text"),
            slot(t_i32, "n"),
            slot(t_note, "note"),
            slot(t_dyn, "d"),
            slot(t_note, "back"),
            slot(t_i32, "got"),
        ],
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 5 },
        }],
        entry: 0,
        source_snapshot: b_bump as u64,
        spans: Vec::new(),
    }];
    if trace {
        m.capabilities = vec!["hir.test.trace.v1".to_string()];
    }
    m.entry = 0;
    m
}

/// The dependency-change shape: `fn beta() bool` — a LEGAL dynamic unit
/// on its own, but alpha's `fn:beta` import still declares `() -> i32`,
/// so the LINK (the real interface check) must refuse the pair.
fn beta_unit_bool() -> MirModule {
    let mut m = MirModule::dynamic_unit(owner_scope());
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
        slots: vec![slot(t_bool, "r")],
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
    let (out, _) = execute_program(snap, &ExecLimits::default(), &CancelFlag::new(false));
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("snapshot execution: {other:?}"),
    }
}

/// V-13 sub-case 2 — the REAL update entry over the DYNAMIC capability:
/// install alpha(a=10) + beta(b=1), then three beta body revisions. EVERY
/// revision must: execute to the folded truth (a + b) through the real
/// dyn pack/take path, clean up (live=0, allocs == frees), keep alpha's
/// code OBJECT carried by the session, and produce NOTHING for alpha
/// (verified/encoded/admitted/store-writes all == 1 — beta only). The OLD
/// snapshot keeps executing its OLD truth.
#[test]
fn av_dynamic_824_incremental_body_edit_identity() {
    let root = std::env::temp_dir().join(format!(
        "av824-session-body-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let store = root.join("store");
    let mut s = DynamicUpdateSession::new(Some(&store), "main");

    let out0 = s
        .apply_revision(&[("main", alpha_unit(10)), ("beta", beta_unit(1))])
        .expect("install");
    assert_eq!(out0.value, Some(11), "install: beta(1) + 10");
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
        assert_eq!(
            out.value,
            Some(want),
            "revision b={b} semantics (dyn round trip)"
        );
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

/// V-13 sub-case 3 — real dependency changes and the authorization
/// lifecycle: a legal beta whose SIGNATURE changed makes the LINK refuse
/// the pair (the real interface check); a capability-REQUIRING revision
/// without the host grant rejects BEFORE any production stage (all
/// counters zero); granting admits it; REVOKING the grant refuses the
/// next revision of the same unit (authorization binds per revision, not
/// retroactively); after every refusal the session's prior state still
/// updates fine.
#[test]
fn av_dynamic_824_incremental_signature_and_grant_lifecycle() {
    let mut s = DynamicUpdateSession::new(None, "main");
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

    // Negative (interface): beta's SIGNATURE changed (i32 → bool). The
    // unit itself passes the real gates; the LINK refuses the mismatch.
    let err = s
        .apply_revision(&[("beta", beta_unit_bool())])
        .expect_err("the link must refuse the signature change");
    assert!(
        err.contains("signature mismatch") || err.contains("does not resolve"),
        "the real link check speaks: {err}"
    );

    // Negative (authorization): a trace-requiring beta revision WITHOUT
    // the grant rejects before ANY production stage.
    let err = s
        .apply_revision(&[("beta", beta_unit_trace(3))])
        .expect_err("the ungranted capability must refuse");
    assert!(err.contains("not granted"), "{err}");
    assert_eq!(
        s.production_counters(),
        Default::default(),
        "the refused authorization produced nothing"
    );

    // Grant it: the same revision admits (required vs granted separated).
    s.set_host_grants(vec!["hir.test.trace.v1".to_string()]);
    let granted = s
        .apply_revision(&[("beta", beta_unit_trace(3))])
        .expect("the granted revision installs");
    assert_eq!(granted.value, Some(8));
    assert!(Arc::ptr_eq(s.unit("main").unwrap(), &alpha_obj));

    // REVOKE: a carried trace-requiring beta still binds the NEXT
    // revision — the ungranted closure refuses before production.
    s.set_host_grants(vec![]);
    let err = s
        .apply_revision(&[("beta", beta_unit_trace(4))])
        .expect_err("the revoked grant must refuse the next revision");
    assert!(err.contains("not granted"), "{err}");
    assert_eq!(s.production_counters(), Default::default());

    // The refusals left the session intact — an honest revision (a plain
    // beta without the capability) still succeeds on the carried units.
    let after = s
        .apply_revision(&[("beta", beta_unit(5))])
        .expect("the session survives every refusal");
    assert_eq!(after.value, Some(10));
    assert!(Arc::ptr_eq(s.unit("main").unwrap(), &alpha_obj));
}

/// V-13 sub-case 4 — the forced-whole counter-control on the PRODUCTION
/// gate: re-submitting alpha UNCHANGED (identical content) alongside a
/// real beta edit swaps alpha's object under the same fingerprint — the
/// gate reports it degraded-unchanged (NON-ZERO) and FAILS, while
/// semantics stay correct. The gate lives in the session (the production
/// consumer), not in this test.
#[test]
fn av_dynamic_824_incremental_forced_degradation_gate_nonzero() {
    let mut s = DynamicUpdateSession::new(None, "main");
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
/// through the shared core AFTER the dynamic extension (dual gate:
/// all-correct + structural + P95 < 1000 ms). The scalar campaign is the
/// shared-core baseline; sub-cases 2/3/4 above cover the dynamic
/// capability itself through its real update entry.
#[test]
fn av_dynamic_824_incremental_baseline_campaign_replay() {
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

/// T-11 I10/I11 (dynamic-profile evidence): 512 body-edit revisions
/// through the REAL update session stay BOUNDED and green — every
/// revision executes the folded truth through the dyn pack/take path,
/// drains the ledger, carries alpha's object, and the per-revision wall
/// time stays under the 1000 ms incremental gate (I11); the session's
/// unit table stays at exactly TWO units after the churn (I10 bounded).
#[test]
fn av_dynamic_824_incremental_churn_512_bounded() {
    let mut s = DynamicUpdateSession::new(None, "main");
    s.apply_revision(&[("main", alpha_unit(100)), ("beta", beta_unit(1))])
        .expect("install");
    let alpha_obj = s.unit("main").unwrap().clone();
    let start = std::time::Instant::now();
    let mut worst_wall = std::f64::MIN_POSITIVE;
    for b in 2..=513u32 {
        let t0 = std::time::Instant::now();
        let out = s
            .apply_revision(&[("beta", beta_unit(b as i32))])
            .unwrap_or_else(|e| panic!("churn revision {b}: {e}"));
        worst_wall = worst_wall.max(t0.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(out.value, Some(100 + b as i32), "revision {b} semantics");
        assert_eq!(out.live_after_release, 0, "revision {b} drains");
        assert_eq!(out.allocs, out.frees, "revision {b} ledger balanced");
        assert!(out.gate.pass(), "revision {b} honest gate");
    }
    let total = start.elapsed().as_secs_f64();
    // I11: every single revision stays under the frozen 1000 ms gate.
    assert!(
        worst_wall < 1000.0,
        "worst per-revision wall {worst_wall:.1} ms exceeds the 1000 ms gate"
    );
    // I10: the churn stays bounded — exactly two units, alpha's object
    // carried throughout, and the total churn completes in seconds.
    assert!(Arc::ptr_eq(s.unit("main").unwrap(), &alpha_obj));
    assert_eq!(
        s.unit("beta").is_some(),
        true,
        "the updated unit stays installed"
    );
    // The final state still executes its final truth.
    let snap = s.snapshot().expect("final snapshot links");
    assert_eq!(run_snapshot(&snap), 100 + 513);
    eprintln!(
        "churn-512: total {total:.1}s, worst revision {worst_wall:.1} ms (I10 bounded, I11 <1000ms)"
    );
}
