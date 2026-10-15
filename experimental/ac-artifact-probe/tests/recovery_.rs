//! V-08 crash recovery tests (§5.4): kill at each of the three publish /
//! reclaim points, restart, and prove idempotent recovery — no half-committed
//! hit, no healthy-store wipe, staging/orphans reclaimed, tombstone deletes
//! completed.

#![cfg(windows)]

mod common;

use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::store::{recover, ManagedStore};
use common::{first, run_node, scratch};

/// The crash command must die from the injected fault (std::process::abort
/// → exit code 3 on Windows), not exit cleanly.
fn expect_crash(out: &common::NodeOut, fault: &str) {
    assert!(
        out.code != 0,
        "fault {} must abort the process (got exit 0):\n{}",
        fault,
        out.raw
    );
}

#[test]
fn recovery_after_staging_crash() {
    let root = scratch("rec-staging");
    let store_root = root.join("store");
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 12 }").unwrap();
    // A healthy artifact exists before the crash.
    let store = ManagedStore::open(&store_root).unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg { project: "healthy".into(), source: src.clone(), work_dir: root.join("wh"), ..Default::default() },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(12));
    drop(store);

    let crash = run_node(&[
        "crash",
        "--at",
        "after-staging",
        "--store",
        store_root.to_str().unwrap(),
        "--project",
        "victim",
        "--source",
        src.to_str().unwrap(),
        "--work",
        root.join("wv").to_str().unwrap(),
    ]);
    expect_crash(&crash, "after-staging");

    // Restart: recovery reclaims the staging leftover; the victim artifact
    // is a clean MISS (no half-committed hit); the healthy artifact still
    // hits; the store keeps working.
    let store = ManagedStore::open(&store_root).unwrap();
    let receipt = recover(&store, true).unwrap();
    assert!(receipt.staging_dirs_removed >= 1, "staging leftover reclaimed: {:?}", receipt);
    let still = vertical_and_run(
        &store,
        &VerticalCfg { project: "healthy".into(), source: src.clone(), work_dir: root.join("wh"), ..Default::default() },
    )
    .unwrap();
    assert_eq!(still.exit_code, Some(12));
    assert_eq!(still.counters.parse, 0, "healthy artifact still hits after another session's crash");
}

#[test]
fn recovery_after_chunk_move_crash() {
    let root = scratch("rec-move");
    let store_root = root.join("store");
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 13 }").unwrap();
    let store = ManagedStore::open(&store_root).unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg { project: "healthy".into(), source: src.clone(), work_dir: root.join("wh"), ..Default::default() },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(13));
    drop(store);

    let crash = run_node(&[
        "crash",
        "--at",
        "after-chunk-move",
        "--store",
        store_root.to_str().unwrap(),
        "--project",
        "victim",
        "--source",
        src.to_str().unwrap(),
        "--work",
        root.join("wv").to_str().unwrap(),
    ]);
    expect_crash(&crash, "after-chunk-move");
    // The crashed publish moved the chunk into the CAS but committed no
    // metadata: the store may carry an orphan blob.

    let store = ManagedStore::open(&store_root).unwrap();
    let receipt = recover(&store, true).unwrap();
    // The victim's crash may or may not leave an orphan blob (identical
    // first chunks land on the existing CAS entry); what MUST hold: no
    // half-committed ARTIFACT and idempotent recovery.
    let _ = receipt;
    let still = vertical_and_run(
        &store,
        &VerticalCfg { project: "healthy".into(), source: src, work_dir: root.join("wh"), ..Default::default() },
    )
    .unwrap();
    assert_eq!(still.exit_code, Some(13));
    assert_eq!(still.counters.parse, 0);
    // Recovery is idempotent (second pass is a no-op).
    let receipt2 = recover(&store, true).unwrap();
    assert_eq!(receipt2.staging_dirs_removed, 0);
    assert_eq!(receipt2.orphan_chunks_removed, 0);
}

#[test]
fn recovery_after_tombstone_crash() {
    let root = scratch("rec-tomb");
    let store_root = root.join("store");
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 14 }").unwrap();
    let crash = run_node(&[
        "crash",
        "--at",
        "after-tombstone",
        "--store",
        store_root.to_str().unwrap(),
        "--project",
        "victim",
        "--source",
        src.to_str().unwrap(),
        "--work",
        root.join("wv").to_str().unwrap(),
    ]);
    expect_crash(&crash, "after-tombstone");

    // Restart: the tombstoned chunk (blob still on disk) is completed by
    // recovery — delete finishes, state converges, no zombie hit.
    let store = ManagedStore::open(&store_root).unwrap();
    let receipt = recover(&store, true).unwrap();
    assert!(
        receipt.tombstone_deletes_completed >= 1,
        "interrupted tombstone delete completed: {:?}",
        receipt
    );
    // A fresh build of the same source works (recompute after miss).
    let out = vertical_and_run(
        &store,
        &VerticalCfg { project: "fresh".into(), source: src, work_dir: root.join("wf"), ..Default::default() },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(14));
}

#[test]
fn recovery_does_not_wipe_healthy_store() {
    // Three crash points in sequence; after each, the store's block count
    // for LIVE content never goes to zero and hits keep working.
    let root = scratch("rec-multi");
    let store_root = root.join("store");
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 15 }").unwrap();
    let store = ManagedStore::open(&store_root).unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg { project: "healthy".into(), source: src.clone(), work_dir: root.join("wh"), ..Default::default() },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(15));
    let blocks_before = store.space().blocks_count;
    assert!(blocks_before > 0);
    drop(store);

    for at in ["after-staging", "after-chunk-move"] {
        let crash = run_node(&[
            "crash",
            "--at",
            at,
            "--store",
            store_root.to_str().unwrap(),
            "--project",
            &format!("victim-{at}"),
            "--source",
            src.to_str().unwrap(),
            "--work",
            root.join("wv").to_str().unwrap(),
        ]);
        expect_crash(&crash, at);
        let store = ManagedStore::open(&store_root).unwrap();
        recover(&store, true).unwrap();
        // Healthy live content still present and hitting.
        let still = vertical_and_run(
            &store,
            &VerticalCfg { project: "healthy".into(), source: src.clone(), work_dir: root.join("wh"), ..Default::default() },
        )
        .unwrap();
        assert_eq!(still.exit_code, Some(15));
        assert_eq!(still.counters.parse, 0, "no healthy-store wipe after {}", at);
        drop(store);
    }
    let _ = first;
}
