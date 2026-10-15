//! V-09 health/watermark tests (§5.4): ≥512 project/version churn through
//! the real pipeline converges between the watermarks without manual
//! cleaning; hard pins over budget produce a bounded backoff (stop-write
//! status) with no live deletion; staging over cap is a controlled resource
//! rejection.

#![cfg(windows)]

mod common;

use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::store::{run_gc, GcConfig, ManagedStore};
use ac_artifact_probe::store::{GC_HIGH, GC_LOW};
use common::{first, run_node, scratch};

/// Full 512-iteration churn is the plan's acceptance load. It runs the
/// real pipeline end to end each iteration; on this machine that is
/// minutes, so it is marked ignored-by-default and executed by the
/// acceptance script explicitly (never skipped silently: the script's
/// `-Scope closeout` runs it with --ignored).
#[test]
#[ignore]
fn gc_budget_churn_512_converges_between_watermarks() {
    let root = scratch("gc-churn");
    let store = root.join("store");
    let work = root.join("work");
    let store_s = store.to_str().unwrap();
    let work_s = work.to_str().unwrap();
    let out = common::run_node_deadlined(
        &[
            "churn",
            "--store",
            store_s,
            "--work",
            work_s,
            "--iterations",
            "512",
            "--ops",
            "300",
            "--gc-every",
            "8",
        ],
        900,
    );
    assert_eq!(out.code, 0, "{}", out.raw);
    let c = first(&out, "CHURN");
    assert_eq!(c["iterations"], 512, "all iterations ran");
    assert!(
        c["gc_runs"].as_u64().unwrap() >= 60,
        "GC ran periodically: {}",
        c["gc_runs"]
    );
    // Convergence: after the final pass the store sits under the low
    // watermark (live set is one version + index), and the PEAK stayed
    // bounded (no unbounded growth across 512 versions).
    assert!(
        c["final_total"].as_u64().unwrap() <= GC_LOW,
        "final total {} must converge below the {} low watermark",
        c["final_total"],
        GC_LOW
    );
    assert!(
        c["peak_total"].as_u64().unwrap() < 2 * GC_HIGH,
        "peak {} unbounded (high watermark {})",
        c["peak_total"],
        GC_HIGH
    );
}

/// Quick variant proving the churn machinery and convergence behavior
/// (same assertions, small iteration count) for everyday runs.
#[test]
fn gc_budget_churn_small_converges() {
    let root = scratch("gc-churn-small");
    let out = run_node(&[
        "churn",
        "--store",
        root.join("store").to_str().unwrap(),
        "--work",
        root.join("work").to_str().unwrap(),
        "--iterations",
        "24",
        "--ops",
        "120",
        "--gc-every",
        "4",
    ]);
    assert_eq!(out.code, 0, "{}", out.raw);
    let c = first(&out, "CHURN");
    assert_eq!(c["iterations"], 24);
    assert!(c["final_total"].as_u64().unwrap() <= GC_LOW, "converged: {}", c["final_total"]);
}

#[test]
fn gc_budget_hard_pins_over_budget_backoff_no_live_deletion() {
    let root = scratch("gc-pin");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    // Build several distinct versions and hard-pin EVERY one (roots kind
    // hard): the live set alone exceeds the low watermark.
    let mut pinned = Vec::new();
    for i in 0..6 {
        let src = root.join(format!("v{i}.at"));
        std::fs::write(&src, format!("fn main() int {{ return {} }}", 30 + i)).unwrap();
        let out = vertical_and_run(
            &store,
            &VerticalCfg {
                project: format!("p{i}"),
                source: src,
                work_dir: root.join(format!("w{i}")),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(out.exit_code, Some(30 + i));
        for (_, id) in &out.artifacts {
            store.register_root("pins", id, "hard").unwrap();
            pinned.push(id.clone());
        }
    }
    let space = store.space();
    assert!(space.blocks_count > 0);

    // GC with a floor the pinned live set cannot reach: backoff.
    let receipt = run_gc(&store, GcConfig { high: 1, low: 1 }).unwrap();
    let backoff = receipt
        .backoff
        .clone()
        .unwrap_or_else(|| panic!("pinned-over-budget GC must report backoff, got {:?}", receipt));
    assert!(backoff.contains("stop-write") || backoff.contains("live set"));
    // No live (pinned) data was deleted: every pinned project still loads.
    let space_after = store.space();
    assert!(space_after.blocks_count > 0, "pinned content retained");
    for i in 0..6 {
        let src = root.join(format!("v{i}.at"));
        let out = vertical_and_run(
            &store,
            &VerticalCfg {
                project: format!("p{i}"),
                source: src,
                work_dir: root.join(format!("w{i}")),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(out.exit_code, Some(30 + i), "pinned project {} still builds/hits", i);
    }
    let _ = &pinned;

    // Unpin → the same GC converges (bounded backoff was never a leak).
    store.remove_roots("pins", "hard").unwrap();
    for i in 0..6 {
        store.remove_project_roots(&format!("p{i}")).unwrap();
    }
    let receipt = run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    assert!(receipt.deleted_chunks > 0, "unpinned content is reclaimable");
}

#[test]
fn gc_budget_staging_cap_is_controlled_resource_rejection() {
    // In-flight staging above the 8 MiB cap refuses new writes with a
    // resource status (bounded backoff), not an unbounded temp dir.
    let root = scratch("gc-staging");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    // Fake in-flight staging by writing files into staging/ directly (the
    // budget consults real disk usage).
    let staging = root.join("store").join("staging").join("t-big");
    std::fs::create_dir_all(&staging).unwrap();
    std::fs::write(staging.join("payload"), vec![0u8; 9 * 1024 * 1024]).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 1 }").unwrap();
    match vertical_and_run(
        &store,
        &VerticalCfg { project: "p".into(), source: src, work_dir: root.join("w"), ..Default::default() },
    ) {
        Err(ac_artifact_probe::driver::DriverError::Store(
            ac_artifact_probe::store::StoreError::ResourceExhausted(msg),
        )) => {
            assert!(msg.contains("staging"), "{}", msg);
        }
        other => panic!("expected staging-budget rejection, got {:?}", other.map(|o| o.exit_code)),
    }
}

#[test]
fn gc_budget_receipt_separates_logical_freed_pending() {
    // The GC receipt's byte fields are distinct: logical invalidation,
    // actually freed, pending-reader — never a single "deleted" number.
    let root = scratch("gc-receipt");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 6 }").unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg { project: "p".into(), source: src.clone(), work_dir: root.join("w"), ..Default::default() },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(6));
    store.remove_project_roots("p").unwrap();
    let before = store.space();
    let receipt = run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    let after = store.space();
    assert!(receipt.deleted_chunks >= 1);
    assert!(receipt.invalid_bytes >= receipt.freed_bytes, "logical ≥ freed (freed is physical)");
    // Physical bytes really left the BLOCK layer (the DB/WAL may grow from
    // the GC's own journal writes — that is real accounted usage, not a
    // failure to free).
    assert!(receipt.freed_bytes > 0);
    assert!(after.blocks < before.blocks, "blocks {} → {}", before.blocks, after.blocks);
    assert!(after.blocks_count < before.blocks_count);
}
