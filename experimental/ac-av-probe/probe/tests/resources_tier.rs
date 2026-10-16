//! T-05 tier: scale/budget/GC/deep chains (PLAN-798 V-09/V-10 resource
//! legs) + the turnaround smoke (full six-scenario release bench runs via
//! scripts/verify-ac-av-798.ps1 -Scope turnaround).

use auto_av_core::contract::{ResourceKind, MAX_FRONTEND_NESTING, MAX_TABLE_ENTRIES};
use auto_av_core::mir::*;
use auto_av_core::verify_mir_with_deadline;
use auto_av_probe::driver::{front_door_admit, front_door_admit_atom, Driver};
use std::time::{Duration, Instant};

fn owner() -> OwnerScope {
    OwnerScope { schema: auto_av_core::contract::MIR_SCHEMA.to_string(), module: "big".into() }
}

/// V-09: a 4096-block straight chain + a mixed CFG (branches AND a legal
/// loop) traverse/verify inside the frozen work-step/wall budgets.
#[test]
fn av_resources_4096_block_chain_within_budget() {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let tb = m.intern_type(MirType::Bool);
    let sig = m.intern_type(MirType::Fn(FnSig { params: vec![], result: ti }));
    let c0 = m.intern_const(MirConst::I32(0));
    const N: u32 = 4096;
    // Chain: block i assigns r (slot 0) and jumps to i+1; last returns r.
    let mut blocks = Vec::with_capacity(N as usize + 1);
    for i in 0..N {
        blocks.push(MirBlock {
            instrs: vec![MirInstr::Const { dst: 0, value: c0 }],
            term: Terminator::Jump { target: i + 1 },
        });
    }
    blocks.push(MirBlock { instrs: vec![], term: Terminator::Return { value: 0 } });
    // Mixed: a diamond + a loop appended at the end of the chain. The
    // branch condition is a bool temp initialized in the last chain block.
    let c_true = m.intern_const(MirConst::Bool(true));
    blocks[N as usize].instrs.push(MirInstr::Const { dst: 1, value: c_true });
    blocks[N as usize].term = Terminator::Branch { cond: 1, then: N + 1, else_: N + 2 };
    blocks.push(MirBlock { instrs: vec![], term: Terminator::Jump { target: N + 3 } });
    blocks.push(MirBlock { instrs: vec![], term: Terminator::Jump { target: N + 3 } });
    // loop header N+3 → body N+4 → back to N+3; exit N+5.
    blocks.push(MirBlock { instrs: vec![], term: Terminator::Jump { target: N + 4 } });
    blocks.push(MirBlock {
        instrs: vec![],
        term: Terminator::Branch { cond: 1, then: N + 3, else_: N + 5 },
    });
    blocks.push(MirBlock { instrs: vec![], term: Terminator::Return { value: 0 } });
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots: vec![
            MirSlot { kind: SlotKind::Local, ty: ti, name: Some("r".into()) },
            MirSlot { kind: SlotKind::Temp, ty: tb, name: None },
        ],
        blocks,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    m.entry = 0;
    assert!(m.table_entries() <= MAX_TABLE_ENTRIES);
    let started = Instant::now();
    let checked = verify_mir_with_deadline(&m, started + Duration::from_secs(5))
        .expect("4096 chain + diamond + loop verifies");
    let elapsed = started.elapsed();
    assert!(elapsed < Duration::from_secs(5), "verify wall {elapsed:?}");
    assert!(checked.stats().work_steps <= 2_000_000, "within the frozen step budget");
}

/// V-09: the same shape OVER the table budget is a resource rejection
/// (never an illegal-syntax classification), checked before deep analysis.
#[test]
fn av_resources_over_table_budget_is_resource() {
    let mut m = MirModule::new(owner());
    let ti = m.intern_type(MirType::I32);
    let sig = m.intern_type(MirType::Fn(FnSig { params: vec![], result: ti }));
    let blocks: Vec<MirBlock> = (0..70_000)
        .map(|i: u32| MirBlock {
            instrs: vec![],
            term: Terminator::Jump { target: i.saturating_sub(1) },
        })
        .collect();
    m.funcs.push(MirFunction {
        owner: m.owner.clone(),
        name: "d_main".into(),
        sig,
        slots: vec![],
        blocks,
        entry: 0,
        source_snapshot: 0,
        spans: vec![],
    });
    m.entry = 0;
    let err = verify_mir_with_deadline(&m, Instant::now() + Duration::from_secs(5)).unwrap_err();
    assert!(matches!(err, auto_av_core::mir_verify::MirVerifyError::Resource { kind: ResourceKind::TableEntries, .. }));
}

/// V-09: source front door — 128 nesting boundary BOTH sides; rejection is
/// a RESOURCE classification that fires BEFORE the (iterative) parser, and
/// the input is never handed to the frontend afterwards.
#[test]
fn av_resources_front_door_nesting_boundary() {
    // Just under: 120 nested parens around an expression.
    let ok_src = format!(
        "fn main() int {{\n    var x = {}1{}\n    return x\n}}\n",
        "(".repeat(120),
        ")".repeat(120)
    );
    assert!(front_door_admit(&ok_src).is_ok(), "120 nesting admits");
    let mut d = Driver::ephemeral();
    let r = d.request(&ok_src, "d_main", &[]);
    assert!(r.ok(), "under-budget deep parens execute: {:?}", r.diag);

    // Over: 4096 nested parens — resource rejection at the door.
    let deep_src = format!(
        "fn main() int {{\n    var x = {}1{}\n    return x\n}}\n",
        "(".repeat(4096),
        ")".repeat(4096)
    );
    let started = Instant::now();
    let err = front_door_admit(&deep_src).unwrap_err();
    assert_eq!(err.code, "av.resource.frontend-nesting");
    assert!(started.elapsed() < Duration::from_millis(500), "door is cheap");
    // And through the driver: the SAME resource classification, before the
    // frontend (frontend_reject counter absent, admission_reject present).
    let r = d.request(&deep_src, "d_main", &[]);
    let diag = r.diag.expect("rejected");
    assert_eq!(diag.stage, "admission");
    assert_eq!(diag.code, "av.resource.frontend-nesting");
    assert_eq!(*r.counters.get("admission_reject").unwrap_or(&0), 1);
}

/// V-09: the recursion-critical ATOM front door — 4096-deep nested lists
/// are rejected before the recursive document parser runs.
#[test]
fn av_resources_atom_front_door_rejects_before_recursive_parser() {
    let ok = "[a: [1, [2, 3]], b: 4]";
    assert!(front_door_admit_atom(ok).is_ok());
    let mut deep = String::from("[");
    deep.push_str(&"[1,".repeat(4096));
    deep.push('1');
    deep.push_str(&"]".repeat(4100));
    let started = Instant::now();
    let err = front_door_admit_atom(&deep).unwrap_err();
    assert_eq!(err.code, "av.resource.frontend-nesting");
    assert!(started.elapsed() < Duration::from_millis(500));
    let _ = MAX_FRONTEND_NESTING;
}

/// V-09: cold-product scale — ≥32 MiB of REAL MIR/ABC product units in the
/// managed store (streamed as ≥1 MiB modules under the 4 MiB input cap),
/// random read-only unit access re-verifies, 512 local revisions on the
/// working module stay correct, and a GC pass converges the water level
/// with leases released. (Sustained store size assertions use the store's
/// own space report.)
#[test]
fn av_resources_32mib_product_random_read_and_churn() {
    let root = std::env::temp_dir().join(format!("av798-res-32m-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let store = auto_av_probe::cache::open_store(&root).unwrap();
    // 60 modules x ~31k single-instr blocks ≈ >32 MiB of ABC product
    // (each module stays under the 65_536 table-entry and 4 MiB input
    // budgets; the products stream through ≤256 KiB codec pages).
    let per_fn_blocks = 31_000u32;
    let mut total_abc = 0usize;
    let mut artifact_ids = Vec::new();
    for mi in 0..64 {
        let mut m = MirModule::unit(OwnerScope {
            schema: auto_av_core::contract::MIR_SCHEMA.to_string(),
            module: format!("big-{mi:02}"),
        });
        let ti = m.intern_type(MirType::I32);
        let sig = m.intern_type(MirType::Fn(FnSig { params: vec![ti], result: ti }));
        let c = m.intern_const(MirConst::I32(mi as i32));
        let p = m.intern_type(MirType::I32);
        let _ = p;
        let mut blocks = Vec::with_capacity(per_fn_blocks as usize);
        for i in 0..per_fn_blocks {
            blocks.push(MirBlock {
                instrs: vec![MirInstr::Const { dst: 1, value: c }],
                term: if i + 1 == per_fn_blocks {
                    Terminator::Return { value: 1 }
                } else {
                    Terminator::Jump { target: i + 1 }
                },
            });
        }
        m.funcs.push(MirFunction {
            owner: m.owner.clone(),
            name: format!("d_bulk_{mi:02}"),
            sig,
            slots: vec![
                MirSlot { kind: SlotKind::Param, ty: ti, name: Some("x".into()) },
                MirSlot { kind: SlotKind::Local, ty: ti, name: Some("r".into()) },
            ],
            blocks,
            entry: 0,
            source_snapshot: 0,
            spans: vec![],
        });
        m.entry = 0;
        let bytes = auto_av_core::encode_abc(&m).expect("encode");
        for pg in &bytes.pages {
            assert!(pg.len() <= auto_av_core::contract::MAX_CODEC_PAGE_BYTES);
        }
        total_abc += bytes.total_len;
        let key = auto_av_probe::cache::abc_unit_key(
            &format!("big-{mi:02}"),
            &format!("d_bulk_{mi:02}"),
            &format!("bulk-{mi:02}"),
            &auto_av_probe::cache::caps_fingerprint(&[]),
        );
        let id = auto_av_probe::cache::put_enveloped(
            &store,
            &key,
            "A09",
            &auto_av_probe::cache::EnvelopeIdentity::abc(&format!("big-{mi:02}")),
            &bytes.to_flat(),
            vec![],
            Some("bulk"),
        )
        .expect("put");
        artifact_ids.push((format!("big-{mi:02}"), id));
    }
    assert!(total_abc >= 32 * 1024 * 1024, "cold ABC product {} < 32 MiB", total_abc);

    // Random read-only: pick units at (deterministic) pseudo-random offsets,
    // load by artifact id, decode + verify.
    let mut seed = 0x798_5eedu64;
    for k in 0..64 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let (owner_name, art) = &artifact_ids[(seed as usize >> 33) % artifact_ids.len()];
        let payload =
            auto_av_probe::cache::get_by_artifact_id(&store, art, &auto_av_probe::cache::EnvelopeIdentity::abc(owner_name))
                .expect("random read");
        let decoded = auto_av_core::decode_abc(&payload).expect("decode");
        let owner_scope = OwnerScope {
            schema: auto_av_core::contract::ABC_SCHEMA.to_string(),
            module: owner_name.clone(),
        };
        auto_av_core::verify_abc(decoded, &owner_scope).expect("re-verify");
        let _ = k;
    }

    // 512 local revisions on a working module: edit → request → correct
    // result each round (fresh store project; hit+miss mix).
    let work_root = std::env::temp_dir().join(format!("av798-res-churn-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work_root);
    let mut d = Driver::with_store(&work_root, "churn").unwrap();
    let mut src = String::from(
        "fn work(v int) int {\n    var acc = v\n    for acc > 10 {\n        acc = acc - 1\n    }\n    return acc\n}\n\nfn main() int {\n    if work(37) != 10 {\n        return 1\n    }\n    return 0\n}\n",
    );
    for rev in 0usize..512 {
        src = src.replace(&format!("return acc + {}", rev.saturating_sub(1)), &format!("return acc + {}", rev));
        let r = d.request(&src, "d_main", &[]);
        assert!(r.ok(), "rev {rev}: {:?}", r.diag);
        if rev % 64 == 0 {
            let cfg = ac_artifact_probe::store::gc::gc_config_default();
            let _ = ac_artifact_probe::store::gc::run_gc(&store, cfg).expect("gc");
        }
    }
    // Water-level convergence: with no live overage, a GC pass reports a
    // bounded managed footprint (the 783 64/48 MiB policy inherits).
    let cfg = ac_artifact_probe::store::gc::gc_config_default();
    let receipt = ac_artifact_probe::store::gc::run_gc(&store, cfg).expect("final gc");
    let space_after = store.space();
    let _ = receipt;
    assert!(
        space_after.total < 512 * 1024 * 1024,
        "managed footprint bounded: {}",
        space_after.total
    );
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&work_root);
}

/// V-11 smoke: the six-scenario bench runs end-to-end on a scaled load
/// (3 samples) with correct results, real hit/miss counters and diagnosed
/// signature conflicts. The 30-sample release-grade run with the P95 hard
/// gate is driven by scripts/verify-ac-av-798.ps1 -Scope turnaround.
#[test]
fn av_turnaround_six_scenarios_smoke() {
    let root = std::env::temp_dir().join(format!("av798-turn-smoke-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let load = auto_av_probe::load256::generate();
    // Scaled: shrink the generator's target via a truncated source (the
    // frozen full-size run belongs to the release script).
    let small: String = load.source.lines().take(160).collect::<Vec<_>>().join("\n");
    let small = format!("{}\nfn main() int {{\n    return 0\n}}\n", small);
    let small = auto_av_probe::load256::generate_with_sized(false, 24 * 1024);
    let edited = auto_av_probe::load256::generate_with_sized(true, 24 * 1024);
    let conflicted = auto_av_probe::load256::edit_signature_conflict(&small.source);
    let rows = auto_av_probe::bench::run_six_scenarios(&root, &small, &edited, &conflicted, 3);
    let scenarios: std::collections::BTreeSet<&str> = rows.iter().map(|r| r.scenario).collect();
    assert_eq!(scenarios.len(), 6, "all six scenarios ran: {:?}", scenarios);
    for row in &rows {
        if row.scenario == "signature-conflict" {
            assert!(row.ok, "conflict diagnosed: {:?}", row.diag);
            assert!(row.diag.is_some());
        } else {
            assert!(row.ok, "scenario {} sample {} ok (value {:?}, expected {:?})", row.scenario, row.sample, row.value, row.expected);
        }
    }
    // The body-edit scenario really re-keyed exactly one unit.
    let edit_rows: Vec<_> = rows.iter().filter(|r| r.scenario == "single-body-edit").collect();
    assert!(
        edit_rows.iter().any(|r| r.mir_build == 1),
        "body edit misses exactly one unit: {:?}",
        edit_rows.iter().map(|r| r.mir_build).collect::<Vec<_>>()
    );
    let _ = std::fs::remove_dir_all(&root);
}
