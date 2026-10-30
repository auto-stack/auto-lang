//! Review R4 counterexample regressions:
//! F-01 the full orphan state matrix reaches GC/recover candidates;
//! F-02 the transient budget covers staging + CAS tmp (real file walk)
//! and the peak records successful reservations only.
//! F-03 is script-level (publish IO propagation) — verified by the
//! reviewer's replay control shape, kept as a manual gate check.

#![cfg(windows)]

#[path = "../common/mod.rs"]
mod common;

use ace_artifacts::codec::{frame_chunk, ChunkKind};
use ace_artifacts::identity::ChunkId;
use ace_artifacts::schema;
use ace_artifacts::store::{recover, run_gc, GcConfig, ManagedStore, PutRequest, StoreError};
use common::scratch;
use std::sync::Arc;

fn key(s: &str) -> ace_artifacts::identity::QueryKey {
    ace_artifacts::identity::QueryKey::new(
        schema::layer("A04").unwrap(),
        vec![("case".into(), s.into())],
    )
}

fn put(s: &ManagedStore, k: &ace_artifacts::identity::QueryKey, bytes: &[u8]) -> String {
    s.put_artifact(PutRequest {
        layer: "A04",
        key: k.clone(),
        kind: ChunkKind::DeclSummary,
        payload: bytes.to_vec(),
        edges: vec![],
        project: None,
    })
    .unwrap()
}

fn blob_path(store: &ManagedStore, cid: &str) -> std::path::PathBuf {
    store
        .root
        .join("blocks")
        .join("blobs")
        .join(&cid[..2])
        .join(cid)
}

// ---------------------------------------------------------------------------
// R4-01 state 1: a `deleted` chunk ROW whose CAS file RE-APPEARED — the
// same-content republication F2 crash (file placed, metadata upsert never
// committed). Both GC and recover must reclaim it.
// ---------------------------------------------------------------------------

#[test]
fn r4_f01_deleted_row_reappeared_blob_reclaimed_by_gc_and_recover() {
    for pass in ["gc", "recover"] {
        let root = scratch(&format!("r4-f01a-{}", pass));
        let store = ManagedStore::open(&root.join("store")).unwrap();
        let k = key("state1");
        // Publish 3MiB, GC-delete it (row goes 'deleted', file gone).
        put(&store, &k, &vec![1u8; 3 * 1024 * 1024]);
        store.remove_project_roots("__none__").ok();
        let r = run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
        assert!(
            r.deleted_chunks >= 1,
            "[{}] first GC removed the chunk",
            pass
        );

        // Restore the same bytes at the same CAS address — the F2-crash
        // state (file present, row still 'deleted').
        let payload = vec![1u8; 3 * 1024 * 1024];
        let framed = frame_chunk(ChunkKind::DeclSummary, &payload);
        let cid = ChunkId::from_parts(&[framed.as_slice()]).0;
        let p = blob_path(&store, &cid);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, &framed).unwrap();
        let before = store.space();
        assert!(before.blocks > 3 * 1024 * 1024, "file re-appeared");

        // Reclaim: whichever pass runs must remove it; the physical total
        // converges below low (512KiB) with NO backoff escape.
        let result = if pass == "gc" {
            let r = run_gc(
                &store,
                GcConfig {
                    high: 1024 * 1024,
                    low: 512 * 1024,
                },
            )
            .unwrap();
            (r.orphans_removed, r.backoff.is_some())
        } else {
            let r = recover(&store, true).unwrap();
            (r.orphan_chunks_removed, false)
        };
        let after = store.space();
        assert!(
            result.0 >= 1,
            "[{}] the deleted-row re-appeared blob IS reclaimed (got {:?})",
            pass,
            result
        );
        assert!(
            after.total <= 512 * 1024,
            "[{}] physical total {} converges below low (R4-01)",
            pass,
            after.total
        );
        assert!(
            !result.1 || pass != "gc",
            "[{}] GC must not hide behind backoff for reclaimable garbage",
            pass
        );
    }
}

// ---------------------------------------------------------------------------
// R4-01 state 2: a `live` chunk ROW with NO artifact (a conflict's
// committed loser). GC must reclaim it directly (recover already could).
// ---------------------------------------------------------------------------

#[test]
fn r4_f01_live_row_without_artifact_reclaimed_by_gc() {
    let root = scratch("r4-f01b");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let k = key("state2");
    let winner_art = put(&store, &k, b"winner");
    // Root the winner so it is a LEGAL KEEP-ALIVE sibling for the GC pass.
    store.register_root("keep", &winner_art, "soft").unwrap();
    // Conflicting 3MiB payload: commits a live chunk row, no artifact.
    store
        .put_artifact(PutRequest {
            layer: "A04",
            key: k.clone(),
            kind: ChunkKind::DeclSummary,
            payload: vec![2u8; 3 * 1024 * 1024],
            edges: vec![],
            project: None,
        })
        .unwrap_err();
    let before = store.space();
    assert!(before.total > 1024 * 1024);

    let r = run_gc(
        &store,
        GcConfig {
            high: 1024 * 1024,
            low: 512 * 1024,
        },
    )
    .unwrap();
    let after = store.space();
    assert!(
        after.total <= 512 * 1024,
        "GC reclaims the conflict-loser chunk too: {} → {} (R4-01)",
        before.total,
        after.total
    );
    assert!(
        r.backoff.is_none(),
        "no backoff for reclaimable garbage: {:?}",
        r.backoff
    );
    // Legal sibling: the winner survives and stays readable.
    let loaded = store.get_artifact("A04", &k).unwrap().unwrap();
    assert_eq!(loaded.payload, b"winner");
}

// ---------------------------------------------------------------------------
// R4-02: the transient budget covers staging + the CAS `.tmp` copy. Two
// concurrent 3MiB publishes whose CAS rename is BLOCKED (a dest handle
// with no sharing, the reviewer's control): the combined transient bytes
// on disk (staging + .tmp) must never exceed the 8MiB cap.
// ---------------------------------------------------------------------------

#[test]
fn r4_f02_transient_budget_covers_staging_plus_cas_tmp() {
    let root = scratch("r4-f02");
    let store = Arc::new(ManagedStore::open(&root.join("store")).unwrap());
    let payloads: Vec<Vec<u8>> = vec![vec![3u8; 3 * 1024 * 1024], vec![4u8; 3 * 1024 * 1024]];
    // Block both CAS destinations: open the dest files with no sharing so
    // BlobStore's rename fails (os32) and its .tmp copies stay on disk.
    let mut held_handles: Vec<std::fs::File> = Vec::new();
    for p in &payloads {
        let framed = frame_chunk(ChunkKind::DeclSummary, p);
        let cid = ChunkId::from_parts(&[framed.as_slice()]).0;
        let dest = blob_path(&store, &cid);
        std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
        std::fs::write(&dest, b"").unwrap();
        // Open with FILE_SHARE_READ only (rename needs DELETE) — the
        // reviewer's FileShare.None shape, via the Windows extension.
        let f = {
            use std::os::windows::fs::OpenOptionsExt;
            std::fs::OpenOptions::new()
                .read(true)
                .share_mode(1) // FILE_SHARE_READ only: blocks rename/delete
                .open(&dest)
                .unwrap()
        };
        held_handles.push(f);
    }

    let barrier = Arc::new(std::sync::Barrier::new(2));
    let mut handles = Vec::new();
    for (i, p) in payloads.iter().enumerate() {
        let store = store.clone();
        let barrier = barrier.clone();
        let payload = p.clone();
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            store.put_artifact(PutRequest {
                layer: "A04",
                key: key(&format!("tmp-{i}")),
                kind: ChunkKind::DeclSummary,
                payload,
                edges: vec![],
                project: None,
            })
        }));
    }
    // While both are in flight (rename blocked → bounded retry), sample the
    // REAL transient bytes on disk: staging dir + CAS `.tmp` files.
    let mut max_transient = 0u64;
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(350);
    while std::time::Instant::now() < deadline {
        let staging = ManagedStore::dir_size_pub(&store.root.join("staging"));
        let mut tmp_bytes = 0u64;
        let blobs = store.root.join("blocks").join("blobs");
        if blobs.is_dir() {
            for shard in std::fs::read_dir(&blobs).unwrap().flatten() {
                for f in std::fs::read_dir(shard.path()).unwrap().flatten() {
                    if f.file_name().to_string_lossy().ends_with(".tmp") {
                        tmp_bytes += f.metadata().map(|m| m.len()).unwrap_or(0);
                    }
                }
            }
        }
        max_transient = max_transient.max(staging + tmp_bytes);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    // Release the blocked destinations; the bounded retries then complete.
    drop(held_handles);
    let mut ok = 0usize;
    let mut refused = 0usize;
    for h in handles {
        match h.join().unwrap() {
            Ok(_) => ok += 1,
            Err(StoreError::ResourceExhausted(_)) => refused += 1,
            Err(e) => panic!("unexpected: {}", e),
        }
    }
    assert_eq!(ok + refused, 2, "both publishers resolve");
    // THE CONTRACT: staging + CAS tmp NEVER exceeds 8MiB on disk (the R4
    // counterexample measured 12,583,104).
    assert!(
        max_transient <= 8 * 1024 * 1024,
        "real transient bytes (staging + CAS .tmp) peaked at {} > cap 8388608 (R4-02)",
        max_transient
    );
}
