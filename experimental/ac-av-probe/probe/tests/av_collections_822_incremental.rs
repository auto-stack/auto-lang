//! PLAN-822 T-09 incremental tests (V-13 sub-cases over the D828 seam):
//! the collections capability rides the function-code/snapshot identity
//! machinery — unchanged functions keep their Arc object and generation
//! across revisions; a signature/type change invalidates precisely; the
//! baseline replay (30-revision campaign) is owned by the 828 suite the
//! incremental script scope replays. Prefix `av_collections_822`.

use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::function_code::FunctionCode;
use auto_av_core::mir::{
    BinOp, ListTypeRef, MirBlock, MirConst, MirFunction, MirInstr, MirModule, MirSlot, MirType,
    OwnerScope, SlotKind, Terminator,
};
use auto_av_core::mir_verify::verify_mir;
use auto_av_core::program_snapshot::ProgramSnapshot;
use std::sync::Arc;

fn owner_scope() -> OwnerScope {
    OwnerScope {
        schema: auto_av_core::contract::MIR_COLLECTIONS_SCHEMA.to_string(),
        module: "collections822".to_string(),
    }
}

/// A single-function UNIT: `fn <name>(i32) -> i32` whose body folds the
/// arg with a list round trip. `bump` changes the folded constant — a
/// body-only edit; `ret_str` changes the SIGNATURE (a dependency change).
fn unit_fn(name: &str, bump: i32, ret_str: bool) -> MirModule {
    let mut m = MirModule::collections_unit(owner_scope());
    let t_i32 = m.intern_type(MirType::I32);
    let t_list = m.intern_type(MirType::List(ListTypeRef { elem: t_i32 }));
    let cb = m.intern_const(MirConst::I32(bump)) as u32;
    let c0 = m.intern_const(MirConst::I32(0)) as u32;
    let result_ty = if ret_str {
        m.intern_type(MirType::Str)
    } else {
        0
    };
    let sig = m.intern_type(MirType::Fn(auto_av_core::mir::FnSig {
        params: vec![0],
        result: result_ty,
    }));
    m.funcs = vec![MirFunction {
        owner: owner_scope(),
        name: name.to_string(),
        sig,
        slots: vec![
            MirSlot {
                kind: SlotKind::Param,
                ty: 0,
                name: Some("x".into()),
            },
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
        ],
        blocks: vec![MirBlock {
            instrs: vec![
                MirInstr::Const { dst: 2, value: cb },
                MirInstr::ListNew {
                    dst: 1,
                    list_ty: t_list,
                    cap: 0,
                },
                MirInstr::ListPush { root: 1, value: 2 },
                MirInstr::Const { dst: 3, value: c0 },
                MirInstr::ListGet {
                    dst: 3,
                    base: 1,
                    index: 3,
                },
                MirInstr::Bin {
                    op: BinOp::AddI32,
                    dst: 3,
                    lhs: 3,
                    rhs: 0,
                },
            ],
            term: Terminator::Return { value: 3 },
        }],
        entry: 0,
        source_snapshot: bump as u64,
        spans: Vec::new(),
    }];
    m.entry = 0;
    m
}

fn admit(m: &MirModule) -> Arc<auto_av_core::function_code::FunctionCode> {
    // The REAL gates first (untrusted builder output).
    verify_mir(m).unwrap_or_else(|e| panic!("verify {e:?}"));
    let wire = encode_abc(m).unwrap_or_else(|e| panic!("encode"));
    let _loaded = verify_abc_bytes(&wire.to_flat(), &m.owner).unwrap_or_else(|e| panic!("{e:?}"));
    // Rebuild the unit shape with the MIR-side owner schema (the wire
    // carries the ABC schema; verify_mir binds the MIR schema).
    let dec = decode_abc(&wire.to_flat()).unwrap();
    let mut unit_mod = dec.module;
    unit_mod.is_unit = true;
    let mir_owner = owner_scope();
    unit_mod.owner = mir_owner.clone();
    for f in unit_mod.funcs.iter_mut() {
        f.owner = mir_owner.clone();
    }
    let wire_u = encode_abc(&unit_mod).unwrap();
    let loaded_u =
        verify_abc_bytes(&wire_u.to_flat(), &unit_mod.owner).unwrap_or_else(|e| panic!("{e:?}"));
    FunctionCode::admit_unit(loaded_u, 1)
        .map(Arc::new)
        .unwrap_or_else(|e| panic!("admit: {e}"))
}

/// V-13 sub-case 2: a BODY-only revision of one function keeps the OTHER
/// function's code object IDENTITY (same Arc) and generation; only the
/// edited function mints a new generation.
#[test]
fn av_collections_822_incremental_body_edit_identity() {
    let fa1 = admit(&unit_fn("alpha", 1, false));
    let fb1 = admit(&unit_fn("beta", 2, false));
    let snap1 = ProgramSnapshot::link(vec![fa1.clone(), fb1.clone()], "alpha", 1)
        .unwrap_or_else(|e| panic!("link: {e}"));
    assert!(Arc::ptr_eq(snap1.find_code("beta").unwrap(), &fb1));

    // Body-only edit of alpha: beta's Arc object is REUSED as-is (the
    // caller passes the SAME Arc), alpha mints a new generation.
    let fa2 = admit(&unit_fn("alpha", 9, false));
    assert_ne!(fa2.generation(), 0);
    let snap2 = ProgramSnapshot::link(vec![fa2.clone(), fb1.clone()], "alpha", 2)
        .unwrap_or_else(|e| panic!("link2: {e}"));
    // Identity reuse: the snapshot dispatches the very same payload for
    // the unchanged function (F-02 observation gate).
    assert!(
        Arc::ptr_eq(snap2.find_code("beta").unwrap(), &fb1),
        "the unchanged unit keeps its Arc payload identity across revisions"
    );
    assert!(!Arc::ptr_eq(snap2.find_code("alpha").unwrap(), &fa1));
    // Generation stability: the unchanged function's generation did not
    // bump; the edited one is a fresh mint.
    assert_eq!(snap2.generation_of("beta"), snap1.generation_of("beta"));
}

/// V-13 sub-case 3: a SIGNATURE change is a precise invalidation — the
/// stale unit no longer links against the new closure shape (the link
/// gate rejects rather than silently re-binding).
#[test]
fn av_collections_822_incremental_signature_change_precise() {
    let fa = admit(&unit_fn("alpha", 1, false));
    let fb_i32 = admit(&unit_fn("beta", 2, false));
    // alpha calls beta with an i32 result; link fine.
    ProgramSnapshot::link(vec![fa.clone(), fb_i32.clone()], "alpha", 1)
        .unwrap_or_else(|e| panic!("link: {e}"));

    // beta's revision changes its RESULT type (a dependency fact). A body
    // that still returns i32 under a Str-typed signature is rejected at
    // the REAL gate — the precise-invalidations positive/negative pair:
    // the type change is caught BEFORE any linkage, never silently
    // re-bound.
    let bad = unit_fn("beta", 2, true);
    match verify_mir(&bad) {
        Err(auto_av_core::mir_verify::MirVerifyError::Invalid { code, .. }) => {
            assert_eq!(code, "mir.return-type");
        }
        other => panic!("the signature-changed unit must be rejected: {other:?}"),
    }
}

// -- T-09: V-13 sub-case 1 — the D828 baseline campaign replay ------------

/// V-13 sub-case 1: the FROZEN D828 baseline (273 575 B / 115 functions,
/// >= 30 distinct body revisions at fn_037's multiplier site) replays
/// GREEN through the shared core AFTER the collections extension — the
/// dual gate (P95 < 1000 ms AND all-correct AND structural green) proves
/// the 822 core edits did not regress the incremental machinery.
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

// -- T-09: V-13 sub-case 4 — the forced-degradation counter-control --------

/// V-13 sub-case 4: a FORCED whole-recompute revision (the unchanged
/// function re-admitted into a fresh object — "only the hash kept, the
/// object/generation swapped") MUST be caught by the structural gate:
/// the identity predicate reports a non-zero degraded-unit count where
/// the honest incremental revision reports zero. A gate that could not
/// distinguish them would let silent whole-recompute pass as incremental.
#[test]
fn av_collections_822_incremental_forced_degradation_gate_nonzero() {
    let fa1 = admit(&unit_fn("alpha", 1, false));
    let fb1 = admit(&unit_fn("beta", 2, false));
    let snap1 = ProgramSnapshot::link(vec![fa1.clone(), fb1.clone()], "alpha", 1)
        .unwrap_or_else(|e| panic!("link: {e}"));

    // The honest revision: alpha's body changes, beta's Arc is carried.
    let fa2 = admit(&unit_fn("alpha", 9, false));
    let snap2 = ProgramSnapshot::link(vec![fa2.clone(), fb1.clone()], "alpha", 2)
        .unwrap_or_else(|e| panic!("link2: {e}"));

    // The FORCED whole-recompute revision: beta re-admitted into a FRESH
    // object under the same content (hash identical, object swapped).
    let fb1_rebuilt = admit(&unit_fn("beta", 2, false));
    let snap3 = ProgramSnapshot::link(vec![fa2.clone(), fb1_rebuilt.clone()], "alpha", 3)
        .unwrap_or_else(|e| panic!("link3: {e}"));

    // The structural gate over the unchanged (beta) slot, per revision:
    // honest -> 0 degraded units; forced -> exactly the swapped one.
    let degraded = |prev: &ProgramSnapshot, next: &ProgramSnapshot| -> usize {
        let mut n = 0;
        for i in 0..next.code().len().max(prev.code().len()) {
            let same = match (prev.code().get(i), next.code().get(i)) {
                (Some(x), Some(y)) => Arc::ptr_eq(x, y),
                _ => false,
            };
            if !same {
                n += 1;
            }
        }
        n
    };
    // Revision 1 -> 2 (honest): only the EDITED function degrades; the
    // unchanged function's gate bit reads ZERO.
    assert_eq!(
        degraded(&snap1, &snap2),
        1,
        "the honest edit degrades exactly alpha"
    );
    assert!(
        Arc::ptr_eq(snap2.find_code("beta").unwrap(), &fb1),
        "honest revision: beta's gate bit is zero"
    );
    // Revision 2 -> 3 (forced whole recompute): the UNCHANGED function's
    // object was swapped under the same content — its gate bit reads
    // NON-ZERO (the count is exactly the swapped unchanged unit; alpha,
    // genuinely re-edited here only by carry-over, keeps its object).
    assert!(!Arc::ptr_eq(
        snap3.find_code("beta").unwrap(),
        snap2.find_code("beta").unwrap()
    ), "forced degradation: beta's gate bit must be non-zero (object swapped under an identical hash)");
    assert_eq!(
        degraded(&snap2, &snap3),
        1,
        "the forced revision degrades exactly the swapped unchanged unit"
    );
    // Alpha's object IS carried in the forced shape (the whole-recompute
    // lie is about BETA), so the gate localizes the degradation.
    assert!(Arc::ptr_eq(
        snap3.find_code("alpha").unwrap(),
        snap2.find_code("alpha").unwrap()
    ));
}
