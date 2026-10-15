//! Review R2 counterexample regressions (F-01..06): each control mirrors
//! the R2 reproduction; the asserted contract is the full invariant.

#![cfg(windows)]

mod common;

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::layout;
use ac_artifact_probe::schema;
use ac_artifact_probe::store::{
    is_reparse, recover, run_gc, GcConfig, ManagedStore, PutRequest, StoreError,
};
use common::scratch;
use std::sync::Arc;

fn key(s: &str) -> QueryKey {
    QueryKey::new(
        schema::layer("A04").unwrap(),
        vec![("case".into(), s.into())],
    )
}

fn put(s: &ManagedStore, k: &QueryKey, bytes: &[u8]) -> String {
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

// ---------------------------------------------------------------------------
// R2-01: recovery cannot delete an in-flight publisher's payload. The
// publisher's staging→CAS→commit runs under the shared liveness guard;
// recovery holds it exclusive — a put that returns Ok is READABLE.
// ---------------------------------------------------------------------------

#[test]
fn r2_f01_put_during_recover_stays_readable() {
    let root = scratch("r2-f01");
    let store = Arc::new(ManagedStore::open(&root.join("store")).unwrap());
    // A legal crash-orphan target + 600 ordinary orphan CHUNK ROWS via the
    // public conflict path (each losing payload commits a chunk row with
    // no artifact — exactly the state recovery's orphan-row pass reclaims,
    // mirroring the R2 control's "600 个普通孤儿").
    let target = key("recover-target");
    let mut names = Vec::new();
    for i in 0u32..600 {
        let k = key(&format!("junk-{i}"));
        let mut p = vec![0u8; 4096];
        p[..4].copy_from_slice(&i.to_le_bytes());
        put(&store, &k, &p);
        // Conflicting second payload under the same key → conflict; its
        // chunk row is committed with no artifact (an orphan row).
        let _ = store.put_artifact(PutRequest {
            layer: "A04",
            key: k.clone(),
            kind: ChunkKind::DeclSummary,
            payload: vec![1u8; 4096],
            edges: vec![],
            project: None,
        });
        names.push(k);
    }
    let aid = put(&store, &target, b"target payload");
    // Everything is unrooted: all chunks (target included) are reclaim
    // candidates for recovery's orphan passes.
    let rec_store = store.clone();
    let rec = std::thread::spawn(move || recover(&rec_store, false));
    // Race the recovery with public put API republication (content reuse
    // of the existing CAS entry is exactly the R2 control).
    let mut put_ok = false;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while std::time::Instant::now() < deadline {
        match store.put_artifact(PutRequest {
            layer: "A04",
            key: target.clone(),
            kind: ChunkKind::DeclSummary,
            payload: b"target payload".to_vec(),
            edges: vec![],
            project: Some("live-project".into()),
        }) {
            Ok(_) => {
                put_ok = true;
                break;
            }
            Err(StoreError::Busy(_)) => std::thread::sleep(std::time::Duration::from_millis(5)),
            Err(e) => panic!("unexpected put error during recovery: {}", e),
        }
    }
    let receipt = rec.join().unwrap().unwrap();
    assert!(
        receipt.orphan_chunks_removed + receipt.orphan_rows_removed > 0,
        "recovery actually worked"
    );
    assert!(
        put_ok,
        "the put must eventually succeed under the bounded wait"
    );
    // THE INVARIANT: a successful publication (with a project root) is
    // readable afterwards — recovery may have deleted the OLD copies, but
    // never the payload of a publication that committed.
    let loaded = store.get_artifact("A04", &target).unwrap();
    assert!(
        loaded.is_some(),
        "successful put must be readable after concurrent recovery (R2-01)"
    );
    assert_eq!(loaded.unwrap().payload, b"target payload");
    let _ = aid;
}

// ---------------------------------------------------------------------------
// R2-02: B header is 15 bytes on the wire; 13/14/21/22-byte directory
// entries are controlled rejections (no panic); A u64 offsets beyond the
// manifest are rejected, not truncated.
// ---------------------------------------------------------------------------

#[test]
fn r2_f02_layout_b_header_bounds_and_a_offset_no_truncation() {
    let root = scratch("r2-f02");
    let ls = layout::LayoutStore::new(&root);
    let (payload_page, _) = ls.write_chunk(&vec![7u8; layout::PAGE_SIZE]).unwrap();

    // Data page carries a record with plen=8 (mirrors the R2 control):
    // every short dlen then undercuts either the 15-byte header or the
    // header+len bound (23).
    let mut data8 = vec![0u8; layout::PAGE_SIZE];
    data8[14] = 8;
    let (data8id, _) = ls.write_chunk(&data8).unwrap();
    for dlen in [13u16, 14, 21, 22] {
        let mut dir = vec![0u8; layout::PAGE_SIZE];
        dir[0..4].copy_from_slice(&0u32.to_le_bytes()); // page 0
        dir[4..8].copy_from_slice(&0u32.to_le_bytes()); // offset 0
        dir[8..10].copy_from_slice(&dlen.to_le_bytes());
        let (dirid, _) = ls.write_chunk(&dir).unwrap();
        let m = layout::Manifest {
            layout: "B".into(),
            version: 1,
            record_count: 1,
            record_pages: vec![],
            payload_pages: vec![],
            payload_len: 0,
            data_pages: vec![data8id.clone()],
            dir_pages: vec![dirid],
        };
        let mut v = layout::LayoutView::open(&ls, m).unwrap();
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| v.record(0)));
        assert!(
            matches!(&r, Err(_) | Ok(Err(_))) && !matches!(&r, Ok(Ok(_))),
            "dlen={} must be a controlled rejection, got {:?}",
            dlen,
            r.as_ref().map(|x| x.is_ok())
        );
    }
    // dlen=15 with plen=8: still short (header+len < 15+8) → controlled.
    // dlen=23: legal sibling (15+8).
    for (dlen, expect_ok) in [(15u16, false), (23u16, true)] {
        let mut dir = vec![0u8; layout::PAGE_SIZE];
        dir[0..4].copy_from_slice(&0u32.to_le_bytes());
        dir[4..8].copy_from_slice(&0u32.to_le_bytes());
        dir[8..10].copy_from_slice(&dlen.to_le_bytes());
        let (dirid, _) = ls.write_chunk(&dir).unwrap();
        // Data page carries a record with plen=8 at offset 0.
        let mut data = vec![0u8; layout::PAGE_SIZE];
        data[14] = 8; // plen byte (header is 15 bytes: b[0..15])
        let (dataid, _) = ls.write_chunk(&data).unwrap();
        let m = layout::Manifest {
            layout: "B".into(),
            version: 1,
            record_count: 1,
            record_pages: vec![],
            payload_pages: vec![],
            payload_len: 0,
            data_pages: vec![dataid],
            dir_pages: vec![dirid],
        };
        let mut v = layout::LayoutView::open(&ls, m).unwrap();
        let r = v.record(0);
        assert_eq!(
            r.is_ok(),
            expect_ok,
            "dlen={} expected ok={} got {:?}",
            dlen,
            expect_ok,
            r.as_ref().map_err(|e| e.to_string())
        );
    }

    // A: poff = 2^48 must be REJECTED (manifest has 1 payload page), not
    // truncated onto page 0.
    let mut rec = vec![0u8; layout::PAGE_SIZE];
    rec[14..16].copy_from_slice(&8u16.to_le_bytes());
    rec[16..24].copy_from_slice(&2u64.pow(48).to_le_bytes());
    let (badrec, _) = ls.write_chunk(&rec).unwrap();
    let m = layout::Manifest {
        layout: "A".into(),
        version: 1,
        record_count: 1,
        record_pages: vec![badrec],
        payload_pages: vec![payload_page],
        payload_len: 8,
        data_pages: vec![],
        dir_pages: vec![],
    };
    let mut v = layout::LayoutView::open(&ls, m).unwrap();
    assert!(
        v.record(0).is_err(),
        "2^48 payload offset must be rejected, not truncated (R2-02)"
    );
}

// ---------------------------------------------------------------------------
// R2-03: same-process concurrent publishers with distinct content under
// distinct keys — every successful put must be READABLE (staging identity
// is ms+pid+seq; the budget reservation is under the shared guard).
// ---------------------------------------------------------------------------

#[test]
fn r2_f03_same_process_concurrent_puts_all_readable() {
    let root = scratch("r2-f03");
    let store = Arc::new(ManagedStore::open(&root.join("store")).unwrap());
    let barrier = Arc::new(std::sync::Barrier::new(4));
    let mut handles = Vec::new();
    for t in 0..4u64 {
        let store = store.clone();
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            let payload = vec![t as u8; 64 * 1024];
            let k = key(&format!("thread-{t}"));
            barrier.wait();
            store
                .put_artifact(PutRequest {
                    layer: "A04",
                    key: k.clone(),
                    kind: ChunkKind::DeclSummary,
                    payload,
                    edges: vec![],
                    project: None,
                })
                .map(|_| k)
        }));
    }
    let mut successful_unreadable = 0usize;
    let mut errors = 0usize;
    for h in handles {
        match h.join().unwrap() {
            Ok(k) => {
                let loaded = store.get_artifact("A04", &k).unwrap();
                match loaded {
                    Some(a) if !a.payload.is_empty() => {}
                    _ => successful_unreadable += 1,
                }
            }
            Err(_) => errors += 1,
        }
    }
    assert_eq!(errors, 0, "no publisher may fail");
    assert_eq!(
        successful_unreadable, 0,
        "every successful put must be readable (R2-03)"
    );
}

// ---------------------------------------------------------------------------
// R2-04: a junction at a managed NAMESPACE root (trash) is a hard boundary
// — recovery refuses, and the external sentinel survives.
// ---------------------------------------------------------------------------

#[test]
fn r2_f04_trash_root_junction_refused_and_sentinel_survives() {
    let root = scratch("r2-f04");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    // External sentinel directory (outside the store).
    let sentinel_dir = root.join("user-precious");
    std::fs::create_dir_all(&sentinel_dir).unwrap();
    let sentinel = sentinel_dir.join("data.bin");
    std::fs::write(&sentinel, b"precious").unwrap();
    // Replace the (empty) trash namespace with a junction to the sentinel.
    let trash = store.root.join("trash");
    std::fs::remove_dir(&trash).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            trash.to_str().unwrap(),
            sentinel_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(status.status.success(), "junction setup failed");
    assert!(is_reparse(&trash));
    match recover(&store, true) {
        Err(StoreError::Boundary(msg)) => {
            assert!(msg.contains("trash"), "{}", msg);
        }
        other => panic!(
            "namespace-root junction must be a controlled Boundary refusal, got {:?}",
            other.map(|_| ())
        ),
    }
    assert!(sentinel.is_file(), "external sentinel survived");
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"precious");
    // Cleanup the junction by rmdir (never touches the target).
    let rm = std::process::Command::new("cmd")
        .args(["/C", "rmdir", trash.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(rm.status.success());
    assert!(sentinel.is_file());
}

// ---------------------------------------------------------------------------
// R2-05: the ORIGINAL total contract — after GC the measured TOTAL
// (blocks + DB + WAL + everything) is under the low watermark. SQLite
// space is actually returned (auto_vacuum + checkpoint), not waved away.
// ---------------------------------------------------------------------------

#[test]
fn r2_f05_total_converges_below_low_watermark_with_real_db_shrink() {
    let root = scratch("r2-f05");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let keep = key("keep");
    put(&store, &keep, b"keep");
    store
        .register_root(
            "keep",
            &store
                .get_artifact("A04", &keep)
                .unwrap()
                .unwrap()
                .artifact_id,
            "soft",
        )
        .unwrap();
    // Enough churn that the DB file grows well past the scaled low.
    for i in 0u32..40 {
        let k = key(&format!("dead-{i}"));
        let mut p = vec![0u8; 8192];
        p[..4].copy_from_slice(&i.to_le_bytes());
        put(&store, &k, &p);
    }
    let before = store.space();
    assert!(
        before.total > 200_000,
        "store grew past the low watermark: {}",
        before.total
    );
    let receipt = run_gc(
        &store,
        GcConfig {
            high: 1,
            low: 200_000,
        },
    )
    .unwrap();
    assert!(receipt.deleted_chunks > 0);
    assert!(
        receipt.backoff.is_none(),
        "no backoff: the live floor is tiny — {:?}",
        receipt.backoff
    );
    let after = store.space();
    assert!(
        after.total <= 200_000,
        "TOTAL (blocks+db+wal+all) must converge below the low watermark: {} → {} (R2-05)",
        before.total,
        after.total
    );
    assert!(
        after.db < before.db,
        "the DB itself shrank (checkpoint+incremental_vacuum)"
    );
    assert!(
        store.get_artifact("A04", &keep).unwrap().is_some(),
        "live set intact"
    );
}

#[test]
fn r2_f05_churn_small_total_under_low() {
    // The scaled churn control now asserts the ORIGINAL semantic: the
    // measured final TOTAL is below the scaled low watermark.
    let root = scratch("r2-f05c");
    let out = common::run_node(&[
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
        "--high",
        "800000",
        "--low",
        "600000",
    ]);
    assert_eq!(out.code, 0, "{}", out.raw);
    let c = common::first(&out, "CHURN");
    assert!(c["high_crossings"].as_u64().unwrap() >= 1);
    assert!(c["gc_deleted_chunks"].as_u64().unwrap() > 0);
    assert!(
        c["final_total"].as_u64().unwrap() <= 600000,
        "final TOTAL {} must be under the scaled low watermark 600000 (R2-05)",
        c["final_total"]
    );
    assert!(
        c["peak_staging"].as_u64().unwrap() > 0,
        "peak_staging must observe in-flight reservations (R2-05 evidence)"
    );
}
