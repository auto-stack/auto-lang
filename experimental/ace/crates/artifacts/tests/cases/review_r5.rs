//! Review R5 counterexample regression (F-01): a failed placement leaves
//! a CAS `.tmp`; a LATER ordinary write must still respect the 8 MiB
//! transient cap — leftovers are counted into every reservation, and our
//! own failed tmp is cleaned under the budget lock.

#![cfg(windows)]

#[path = "../common/mod.rs"]
mod common;

use ace_artifacts::codec::{frame_chunk, ChunkKind};
use ace_artifacts::identity::ChunkId;
use ace_artifacts::schema;
use ace_artifacts::store::{ManagedStore, PutRequest, StoreError};
use common::scratch;
use std::os::windows::fs::OpenOptionsExt;
use std::sync::Arc;

fn key(s: &str) -> ace_artifacts::identity::QueryKey {
    ace_artifacts::identity::QueryKey::new(
        schema::layer("A04").unwrap(),
        vec![("case".into(), s.into())],
    )
}

fn blob_path(store: &ManagedStore, cid: &str) -> std::path::PathBuf {
    store
        .root
        .join("blocks")
        .join("blobs")
        .join(&cid[..2])
        .join(cid)
}

/// Real on-disk transient bytes: staging dir + every CAS `.tmp`.
fn transient_on_disk(store: &ManagedStore) -> (u64, u64) {
    let staging = ManagedStore::dir_size_pub(&store.root.join("staging"));
    let mut tmp = 0u64;
    let blobs = store.root.join("blocks").join("blobs");
    if blobs.is_dir() {
        for shard in std::fs::read_dir(&blobs).unwrap().flatten() {
            for f in std::fs::read_dir(shard.path()).unwrap().flatten() {
                if f.file_name().to_string_lossy().ends_with(".tmp") {
                    tmp += f.metadata().map(|m| m.len()).unwrap_or(0);
                }
            }
        }
    }
    (staging, tmp)
}

// Mirror of the reviewer's control: block a CAS DEST with a no-sharing
// handle so all rename retries fail (Io/os5), leaving the tmp behind
// (pre-fix). With the fix, the failure exit removes OUR tmp under the
// budget lock — and any residue still counts into the next reservation.
#[test]
fn r5_f01_failed_tmp_residue_counted_and_cleaned() {
    let root = scratch("r5-f01");
    let store = Arc::new(ManagedStore::open(&root.join("store")).unwrap());

    // --- Step 1: a 3MiB publish whose dest is un-renamable → failure.
    let payload_a = vec![1u8; 3 * 1024 * 1024];
    let framed_a = frame_chunk(ChunkKind::DeclSummary, &payload_a);
    let cid_a = ChunkId::from_parts(&[framed_a.as_slice()]).0;
    let dest_a = blob_path(&store, &cid_a);
    std::fs::create_dir_all(dest_a.parent().unwrap()).unwrap();
    std::fs::write(&dest_a, b"").unwrap();
    let held_a = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0) // FileShare.None — the reviewer's control
        .open(&dest_a)
        .unwrap();

    let err = store
        .put_artifact(PutRequest {
            layer: "A04",
            key: key("a"),
            kind: ChunkKind::DeclSummary,
            payload: payload_a.clone(),
            edges: vec![],
            project: None,
        })
        .map(|_| ())
        .unwrap_err();
    assert!(
        err.to_string().contains("io") || err.to_string().contains("CAS placement"),
        "controlled placement failure: {}",
        err
    );
    // The failure exit removed OUR tmp under the budget lock; staging is
    // cleaned by the outer error path. (If removal is blocked by the OS,
    // the residue is still counted — asserted by the totals below.)
    drop(held_a);
    let (staging, tmp) = transient_on_disk(&store);
    assert!(
        staging + tmp <= 8 * 1024 * 1024,
        "post-failure transient {} + tmp {} within cap",
        staging,
        tmp
    );

    // --- Step 2: a LATER ordinary 3MiB write (dest briefly blocked to
    // keep its tmp resident) must still respect the cap — leftovers from
    // any earlier residue are counted into the reservation.
    let payload_b = vec![2u8; 3 * 1024 * 1024];
    let framed_b = frame_chunk(ChunkKind::DeclSummary, &payload_b);
    let cid_b = ChunkId::from_parts(&[framed_b.as_slice()]).0;
    let dest_b = blob_path(&store, &cid_b);
    std::fs::create_dir_all(dest_b.parent().unwrap()).unwrap();
    std::fs::write(&dest_b, b"").unwrap();
    let held_b = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1) // FILE_SHARE_READ: blocks rename, allows reads
        .open(&dest_b)
        .unwrap();

    let store2 = store.clone();
    let producer = std::thread::spawn(move || {
        store2.put_artifact(PutRequest {
            layer: "A04",
            key: key("b"),
            kind: ChunkKind::DeclSummary,
            payload: payload_b,
            edges: vec![],
            project: None,
        })
    });
    // Sample the REAL disk bytes during the retry window.
    let mut max_total = 0u64;
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(350);
    while std::time::Instant::now() < deadline {
        let (s, t) = transient_on_disk(&store);
        max_total = max_total.max(s + t);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(held_b); // release — the bounded retries then complete
    let result = producer.join().unwrap();

    // THE CONTRACT (R5-01): the reviewer measured 9,437,328 > 8,388,608;
    // with leftovers counted + own-tmp cleanup it can never exceed the cap.
    assert!(
        max_total <= 8 * 1024 * 1024,
        "real transient bytes (staging + ALL CAS .tmp incl. residue) peaked at {} > cap 8388608 (R5-01)",
        max_total
    );
    assert!(
        result.is_ok(),
        "the later ordinary write succeeds: {:?}",
        result.err().map(|e| e.to_string())
    );
}

// Crash-residue sibling: a `.tmp` left in the CAS with NO row (a crash
// mid-window) consumes quota — a subsequent 3MiB reservation that would
// exceed the cap is REFUSED, and a small write still fits.
#[test]
fn r5_f01_crash_tmp_residue_consumes_quota() {
    let root = scratch("r5-f01b");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    // Plant a 6MiB crash-leftover tmp directly in the CAS (no rows).
    let fake = store
        .root
        .join("blocks")
        .join("blobs")
        .join("zz")
        .join("chk-crash-residue.tmp");
    std::fs::create_dir_all(fake.parent().unwrap()).unwrap();
    std::fs::write(&fake, vec![0u8; 6 * 1024 * 1024]).unwrap();

    // A 3MiB publish needs 6MiB of fresh transient on top of 6MiB residue
    // → 12MiB > 8MiB cap: controlled refusal.
    let err = store
        .put_artifact(PutRequest {
            layer: "A04",
            key: key("over"),
            kind: ChunkKind::DeclSummary,
            payload: vec![3u8; 3 * 1024 * 1024],
            edges: vec![],
            project: None,
        })
        .map(|_| ())
        .unwrap_err();
    assert!(
        matches!(err, StoreError::ResourceExhausted(ref m) if m.contains("leftover CAS tmp")),
        "residue must consume quota with a message naming it: {}",
        err
    );
    // A small write (needs ~2×small) still fits.
    store
        .put_artifact(PutRequest {
            layer: "A04",
            key: key("small"),
            kind: ChunkKind::DeclSummary,
            payload: vec![4u8; 1024],
            edges: vec![],
            project: None,
        })
        .map(|_| ())
        .unwrap();
}
