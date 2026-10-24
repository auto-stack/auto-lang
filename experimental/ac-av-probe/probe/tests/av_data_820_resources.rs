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
use auto_av_core::contract::{ExecOutcome, ResourceKind, Value, MIR_DATA_SCHEMA};
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
    // 400 slots × ~16.0 KiB ≈ 6.6 MiB < 8 MiB: success.
    let loaded = admit(&wide_record_module(400));
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
    // Two aggregates' worth of record data (~32 KiB) was live and STAYS
    // accounted; nothing beyond the fault point committed.
    assert!(
        trace.peak_slot_bytes >= 2 * 16000 && trace.peak_slot_bytes < 3 * 17000,
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
fn av_data_820_layout_overflow_arm_is_defense_only() {
    // Documented boundary (not a synthetic fake): u64 layout arithmetic
    // cannot overflow inside the frozen table budgets — 65 536 types × a
    // ≤1024-field record (≤8 KiB logical each) tops out ≈512 MiB, far
    // below 2^63. The checked arithmetic stays as defense-in-depth; the
    // REACHABLE shape guards (field count, cycles, empty) are pinned by
    // av_data_820_record_field_count_and_path_depth_are_resource and the
    // core layout tests.
    let mut m = MirModule::new_data(owner("of"));
    let ti = m.intern_type(MirType::I32);
    let fields: Vec<MirFieldDef> = (0..1024)
        .map(|i| MirFieldDef {
            name: format!("f{i}"),
            ty: ti,
        })
        .collect();
    let rec = m.intern_type(MirType::Record(RecordTypeRef {
        owner_module: "of".into(),
        name: "Max".into(),
        fields,
    }));
    let d = auto_av_core::layout::compute_layout(&m, rec).expect("max-width layout derives");
    assert_eq!(d.size, 4096, "1024 × i32 = 4096 logical bytes");
}
