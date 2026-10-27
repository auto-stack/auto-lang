//! PLAN-820 T-04 resource-boundary counterexamples (`av_data_820` family,
//! part 3 — V-07). Frozen oracles (contract-decision §12):
//! - within-cap control succeeds; the over-cap program rejects MID-RUN as
//!   `av.resource.slot-bytes` with the real peak on the trace;
//! - one-unit-under budget rejects by exactly the missing byte;
//! - the Nth-allocation fault seam fails as Infra WITHOUT applying the
//!   write: prior trace events survive, the ledger keeps its prior
//!   accounting, and the aborted slot keeps its prior value;
//! - cancellation keeps the peak on the trace (cleanup accounting is not
//!   eaten by fuel exhaustion).
//!
//! The u64 layout-overflow arm (`mir.layout-overflow`) is defense-in-depth
//! and UNREACHABLE inside the frozen table budgets (65536 types × ≤8 KiB
//! per record ≈ 512 MiB logical, far below 2^63) — documented here, not
//! faked with a synthetic overflow.

use auto_av_core::abc::encode_abc;
use auto_av_core::av::{execute, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, ResourceKind, MIR_DATA_SCHEMA};
use auto_av_core::mir::MirConst;
use auto_av_core::mir::*;
use auto_av_core::{abc_verify, mir_verify};

fn owner(m: &str) -> OwnerScope {
    OwnerScope {
        schema: MIR_DATA_SCHEMA.to_string(),
        module: m.to_string(),
    }
}

/// The wide-record budget probe: `n` record locals, each AGGREGATE filling
/// a 1024-field f64 record from one shared f64 const slot. Live data per
/// filled slot ≈ 16 (cell) + 24 (Vec) + 1024×16 = 16400 bytes; the frozen
/// slot-data budget is 8 MiB.
fn wide_record_module(n: u32) -> MirModule {
    assert!(n >= 1);
    let mut m = MirModule::new_data(owner("wide"));
    let tf = m.intern_type(MirType::F64);
    let fields: Vec<MirFieldDef> = (0..1024)
        .map(|i| MirFieldDef {
            name: format!("f{i}"),
            ty: tf,
        })
        .collect();
    let rec = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "wide".into(),
        name: "Wide".into(),
        fields,
    }));
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: tf,
    }));
    let c0 = m.intern_const(MirConst::F64Bits(0));
    let mut slots = vec![MirSlot {
        kind: SlotKind::Local,
        ty: tf,
        name: None,
    }];
    for _ in 0..n {
        slots.push(MirSlot {
            kind: SlotKind::Local,
            ty: rec,
            name: None,
        });
    }
    let mut instrs = vec![MirInstr::Const { dst: 0, value: c0 }];
    for i in 0..n {
        instrs.push(MirInstr::Aggregate {
            dst: 1 + i,
            record_ty: rec,
            // every operand reads the SAME materialized f64 slot — legal
            // (slots are values, reads do not move them)
            operands: vec![0; 1024],
        });
    }
    m.funcs.push(MirFunction {
        owner: owner("wide"),
        name: "main".into(),
        sig,
        slots,
        blocks: vec![MirBlock {
            instrs,
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    m
}

fn admit(m: &MirModule) -> abc_verify::LoadedModule {
    mir_verify::verify_mir(m).expect("verify");
    let wire = encode_abc(m).expect("encode").to_flat();
    abc_verify::verify_abc_bytes(&wire, &m.owner).expect("admit")
}

#[test]
fn av_data_820_slot_budget_within_cap_control_succeeds() {
    // PLAN-821 adaptation (817-precedent): Value grew with the owned
    // variants (StrHandle), so a record field's footprint rose ~16→~24B;
    // re-calibrated from 400 to 195 slots (Value 16->32B, record deep ~32.8 KiB) to keep the same ~6.4 MiB
    // within-cap target (< 8 MiB, peak > 6 MiB).
    let loaded = admit(&wide_record_module(195));
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(out, ExecOutcome::Success { .. }),
        "within-cap control succeeds, got {out:?}"
    );
    assert!(
        trace.peak_slot_bytes > 6 * 1024 * 1024,
        "real record peak recorded"
    );
}

#[test]
fn av_data_820_slot_budget_over_cap_rejects_mid_run() {
    // 520 slots × ~16.0 KiB ≈ 8.3 MiB > 8 MiB: the run rejects MID-WAY as
    // a slot-bytes resource with the true high-water on the trace.
    let loaded = admit(&wide_record_module(520));
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    match out {
        ExecOutcome::Resource {
            kind: ResourceKind::SlotBytes,
            ..
        } => {}
        other => panic!("expected SlotBytes resource, got {other:?}"),
    }
    assert!(
        trace.peak_slot_bytes <= 8 * 1024 * 1024,
        "peak never crosses the budget: {}",
        trace.peak_slot_bytes
    );
    assert!(trace.peak_slot_bytes > 7 * 1024 * 1024);
}

#[test]
fn av_data_820_slot_budget_one_unit_under_rejects() {
    // Exactly one byte below the smallest live footprint: the root frame
    // itself cannot commit. (One f64 slot = 16 bytes of live data.)
    let mut m = MirModule::new_data(owner("one"));
    let tf = m.intern_type(MirType::F64);
    let sig = m.intern_type(MirType::Fn(FnSig {
        params: vec![],
        result: tf,
    }));
    let c = m.intern_const(MirConst::F64Bits(0));
    m.funcs.push(MirFunction {
        owner: owner("one"),
        name: "main".into(),
        sig,
        slots: vec![MirSlot {
            kind: SlotKind::Local,
            ty: tf,
            name: None,
        }],
        blocks: vec![MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: c }],
            term: Terminator::Return { value: 0 },
        }],
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
        env: None,
    });
    m.entry = 0;
    let loaded = admit(&m);
    let limits = ExecLimits {
        max_slot_bytes: 15, // one byte under the 16-byte footprint
        ..ExecLimits::default()
    };
    let (out, trace) = execute(&loaded, &limits, &CancelFlag::new(false));
    assert!(
        matches!(
            out,
            ExecOutcome::Resource {
                kind: ResourceKind::SlotBytes,
                ..
            }
        ),
        "one-unit-under rejects before dispatch, got {out:?}"
    );
    assert_eq!(trace.peak_slot_bytes, 0, "nothing committed");
}

#[test]
fn av_data_820_fault_at_nth_allocation_preserves_prior_state() {
    // Fault at the 3rd value allocation: aggregates 1/2 stay live, the
    // trace keeps events emitted before the fault, the ledger never
    // half-commits, and the outcome is Infra (a seam, never a language
    // verdict). The over-budget attempt BEFORE the fault counter — a
    // budget rejection — never reaches the fault counter (attempt=0).
    let loaded = admit(&wide_record_module(8));
    let limits = ExecLimits {
        fault_after_allocs: Some(3),
        ..ExecLimits::default()
    };
    let (out, trace) = execute(&loaded, &limits, &CancelFlag::new(false));
    match out {
        ExecOutcome::Infra(msg) => assert!(
            msg.contains("fault-injected at value allocation #3"),
            "got {msg:?}"
        ),
        other => panic!("expected fault Infra, got {other:?}"),
    }
    // Two aggregates' worth of record data was live and STAYS accounted;
    // nothing beyond the fault point committed. PLAN-822 merge-stage band
    // refresh: Value gained the Place variant (size_of 32→40B, F-822-R2-02),
    // so a 1024-field record's deep_size account moved from ~25KB to ~41KB
    // (cell × 1025 + Vec header). The band stays SEMANTIC: ≥ two committed
    // aggregates, < three.
    assert!(
        trace.peak_slot_bytes >= 2 * 40000 && trace.peak_slot_bytes < 3 * 42000,
        "two committed aggregates' peak survives the fault: {}",
        trace.peak_slot_bytes
    );
}

#[test]
fn av_data_820_cancellation_keeps_peak_accounting() {
    // Cancel immediately: the outcome is Cancelled (not a resource, not a
    // trap) and the trace still carries the real live-data high-water —
    // cleanup accounting survives fuel exhaustion/cancellation.
    let loaded = admit(&wide_record_module(4));
    let cancel = CancelFlag::new(true);
    let (out, trace) = execute(&loaded, &ExecLimits::default(), &cancel);
    assert!(matches!(out, ExecOutcome::Cancelled), "got {out:?}");
    // The root frame existed before the poll: its footprint is recorded.
    assert!(trace.peak_slot_bytes > 0, "peak survives cancellation");
}

#[test]
fn av_data_820_layout_overflow_real_input_rejected() {
    // F820-R3-01 companion (V-04): nested layout overflow IS reachable
    // within the frozen table budgets — a binary expansion of records
    // (every type two fields of its predecessor, i32 leaf) doubles the
    // logical size per level: level k sizes to 8·2^k logical bytes.
    // Level 60 (2^63) still fits u64 and must derive; level 61 overflows
    // the CHECKED arithmetic and must be a controlled `mir.layout-overflow`
    // rejection at layout AND at full MIR/ABC admission — no panic, no
    // silent acceptance (this replaces the r6 "defense-only" test whose
    // ≤8 KiB-per-record inference wrongly proved nested overflow away).
    fn binary_expansion(levels: usize) -> (MirModule, TypeIdx) {
        let mut m = MirModule::new_data(owner("ovf"));
        let ti = m.intern_type(MirType::I32);
        let mut prev = ti;
        for k in 0..levels {
            let fields = vec![
                MirFieldDef {
                    name: "a".into(),
                    ty: prev,
                },
                MirFieldDef {
                    name: "b".into(),
                    ty: prev,
                },
            ];
            prev = m.intern_type(MirType::Record(RecordTypeRef {
                owner_module: "ovf".into(),
                name: format!("T{k}"),
                fields,
            }));
        }
        // One otherwise-valid function (i32 identity) so the module is a
        // complete admission candidate, not a bare type table.
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![ti],
            result: ti,
        }));
        m.funcs.push(MirFunction {
            owner: owner("ovf"),
            name: "main".into(),
            sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Param,
                    ty: ti,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Local,
                    ty: ti,
                    name: None,
                },
            ],
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::Copy { dst: 1, src: 0 }],
                term: Terminator::Return { value: 1 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.entry = 0;
        (m, prev)
    }

    // Legal control: level 60 derives exactly 2^63 logical bytes.
    let (legal, deepest) = binary_expansion(61); // T0..T60, deepest = level 60
    let d = auto_av_core::layout::compute_layout(&legal, deepest)
        .expect("level 60 (2^63 bytes) still derives");
    assert_eq!(d.size, 9223372036854775808, "8·2^60 = 2^63");
    let checked = mir_verify::verify_mir(&legal).expect("legal deep module verifies");
    assert!(checked.stats().work_steps < 2_000_000);

    // Overflow control: one more level — checked arithmetic rejects at
    // layout derivation AND at full MIR admission; the encoded module is
    // equally refused at the ABC gate (the restore path re-verifies).
    let (overflow, deepest) = binary_expansion(62); // deepest = level 61 → 2^64 > u64
    let err =
        auto_av_core::layout::compute_layout(&overflow, deepest).expect_err("2^64 overflows u64");
    assert_eq!(err.code(), "mir.layout-overflow", "got {err:?}");
    let err = mir_verify::verify_mir(&overflow).expect_err("overflow module rejected");
    match &err {
        mir_verify::MirVerifyError::Invalid { code, .. } => {
            assert!(code.contains("mir.layout-overflow"), "got {err:?}");
        }
        other => panic!("overflow must be an invalid-layout rejection, got {other:?}"),
    }
    let wire = auto_av_core::abc::encode_abc(&overflow)
        .expect("encode")
        .to_flat();
    let err = abc_verify::verify_abc_bytes(&wire, &overflow.owner)
        .expect_err("ABC admission re-verifies and refuses the overflow module");
    assert!(
        format!("{err:?}").contains("mir.layout-overflow"),
        "got {err:?}"
    );
}

#[test]
fn av_data_820_layout_work_shares_the_verify_budget() {
    // F820-R3-01: layout derivation must draw from the ONE verification
    // work-step budget. The r6 shape reset steps=0 and the memo per record,
    // so a 2100-deep chain re-walked ancestors (2,205,050 field visits —
    // already past the 2M ceiling) yet passed every local check. Now ONE
    // memo makes the walk linear (the legal deep graph stays green) and
    // every layout step charges the shared verifier counter — a module
    // whose TOTAL derivation work exceeds the ceiling is a RESOURCE
    // rejection, never a silent accept. (The negative control builds
    // ~2.1M field defs — expect a ~200MB test-process footprint.)
    fn chain_fn(m: &mut MirModule, ti: TypeIdx) {
        let sig = m.intern_type(MirType::Fn(FnSig {
            params: vec![ti],
            result: ti,
        }));
        m.funcs.push(MirFunction {
            owner: owner(m.owner.module.as_str()),
            name: "main".into(),
            sig,
            slots: vec![
                MirSlot {
                    kind: SlotKind::Param,
                    ty: ti,
                    name: None,
                },
                MirSlot {
                    kind: SlotKind::Local,
                    ty: ti,
                    name: None,
                },
            ],
            blocks: vec![MirBlock {
                instrs: vec![MirInstr::Copy { dst: 1, src: 0 }],
                term: Terminator::Return { value: 1 },
            }],
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
            env: None,
        });
        m.entry = 0;
    }

    // Positive: the reviewer's 2100-deep chain (legal within budget).
    let mut m = MirModule::new_data(owner("chain"));
    let ti = m.intern_type(MirType::I32);
    let mut prev = ti;
    for k in 0..2100 {
        prev = m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "chain".into(),
            name: format!("C{k}"),
            fields: vec![MirFieldDef {
                name: "p".into(),
                ty: prev,
            }],
        }));
    }
    chain_fn(&mut m, ti);
    let checked =
        mir_verify::verify_mir(&m).expect("the legal deep graph verifies under the shared budget");
    assert!(
        checked.stats().work_steps < 2_000_000,
        "memoized layout is linear: {} steps",
        checked.stats().work_steps
    );

    // Negative control A (unit, direct): the SHARED counter IS the layout
    // budget — a derivation started with the counter near the ceiling
    // refuses with LayoutError::Budget instead of silently completing.
    // This is the axis the r6 shape lost (each call got a fresh counter).
    let (m_small, deepest_small) = {
        let mut m = MirModule::new_data(owner("near"));
        let ti = m.intern_type(MirType::I32);
        let mut prev = ti;
        for k in 0..8 {
            prev = m.intern_type(MirType::Record(RecordTypeRef {
                owner_module: "near".into(),
                name: format!("N{k}"),
                fields: vec![MirFieldDef {
                    name: "p".into(),
                    ty: prev,
                }],
            }));
        }
        (m, prev)
    };
    let mut budget = auto_av_core::contract::MAX_VERIFY_WORK_STEPS - 3;
    let mut memo = vec![None; m_small.types.len()];
    let mut color = vec![0u8; m_small.types.len()];
    let err = auto_av_core::layout::compute_layout_shared(
        &m_small,
        deepest_small,
        &mut budget,
        &mut memo,
        &mut color,
    )
    .expect_err("a near-ceiling shared budget must refuse further derivation");
    assert!(
        matches!(err, auto_av_core::layout::LayoutError::Budget),
        "got {err:?}"
    );

    // Negative control B (integration): a module whose TOTAL admission
    // work crosses a frozen ceiling is RESOURCE-classified — never
    // accepted, never an invalid/panic. For field-count blowups the
    // decoded-heap gate (16 MiB) owns the rejection before the step
    // ceiling; both are the same resource family, and the layout stage
    // now charges the same shared counter control A proves live.
    let mut m = MirModule::new_data(owner("wide"));
    let ti = m.intern_type(MirType::I32);
    let mut many: Vec<MirFieldDef> = Vec::with_capacity(1024);
    for i in 0..1024 {
        many.push(MirFieldDef {
            name: format!("f{i}"),
            ty: ti,
        });
    }
    for k in 0..2050u32 {
        m.intern_type(MirType::Record(RecordTypeRef {
            owner_module: "wide".into(),
            name: format!("W{k}"),
            fields: many.clone(),
        }));
    }
    chain_fn(&mut m, ti);
    let err = mir_verify::verify_mir(&m).expect_err("over-ceiling work is rejected");
    match &err {
        mir_verify::MirVerifyError::Resource { .. } => {}
        other => panic!("expected a resource-classified rejection, got {other:?}"),
    }
}
