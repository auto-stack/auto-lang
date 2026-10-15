//! Review R3 counterexample regressions (F-01..03): atomic staging budget,
//! GC counts orphans in the physical total and reclaims them, GC namespace
//! gates. (F-04 is script-level; F-05 docs; F-06 warnings — covered by
//! build checks.)

#![cfg(windows)]

mod common;

use ac_artifact_probe::codec::{frame_chunk, ChunkKind};
use ac_artifact_probe::identity::ChunkId;
use ac_artifact_probe::schema;
use ac_artifact_probe::store::{
    is_reparse, run_gc, GcConfig, ManagedStore, PutRequest, StoreError,
};
use common::scratch;
use std::sync::Arc;

fn key(s: &str) -> ac_artifact_probe::identity::QueryKey {
    ac_artifact_probe::identity::QueryKey::new(
        schema::layer("A04").unwrap(),
        vec![("case".into(), s.into())],
    )
}

// ---------------------------------------------------------------------------
// R3-01: the staging budget is an ATOMIC cross-process quota. Four threads
// × 3 MiB each under a barrier: the fourth must be REFUSED (12 > 8 MiB),
// and the true concurrent peak is recorded, not a per-call prediction.
// ---------------------------------------------------------------------------

#[test]
fn r3_f01_concurrent_staging_budget_is_atomic() {
    let root = scratch("r3-f01");
    let store = Arc::new(ManagedStore::open(&root.join("store")).unwrap());
    const N: usize = 4;
    const EACH: usize = 3 * 1024 * 1024;
    // Mirror the reviewer's control: PRE-HOLD the chunk lock of every
    // payload so each publisher blocks AFTER its staging write (bytes stay
    // resident on disk) — the exact in-flight state the budget must see.
    let payloads: Vec<Vec<u8>> = (0..N).map(|t| vec![t as u8; EACH]).collect();
    let mut held = Vec::new();
    for p in &payloads {
        let framed = frame_chunk(ChunkKind::DeclSummary, p);
        let cid = ChunkId::from_parts(&[framed.as_slice()]).0;
        let lock_path = store.root.join(format!("locks/chunk-{}.lock", cid));
        held.push(
            ac_artifact_probe::store::FileGuard::try_lock(&lock_path, true)
                .unwrap()
                .expect("lock free"),
        );
    }
    let barrier = Arc::new(std::sync::Barrier::new(N));
    let mut handles = Vec::new();
    for t in 0..N {
        let store = store.clone();
        let barrier = barrier.clone();
        let payload = payloads[t].clone();
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            store.put_artifact(PutRequest {
                layer: "A04",
                key: key(&format!("budget-{t}")),
                kind: ChunkKind::DeclSummary,
                payload,
                edges: vec![],
                project: None,
            })
        }));
    }
    // Reviewer-style timed release: the accepted publishers block on the
    // pre-held chunk locks; a timer thread releases them ~800ms in, so
    // they complete with Ok AFTER the budget window — meanwhile the 3rd
    // and 4th reservations were decided during the hold (the reviewer's
    // "200ms后释放" shape, stretched under the bounded lock wait).
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(800));
        drop(held);
    });
    let mut ok = 0usize;
    let mut refused = 0usize;
    for h in handles {
        match h.join().unwrap() {
            Ok(_) => ok += 1,
            Err(StoreError::ResourceExhausted(_)) => refused += 1,
            Err(e) => panic!("unexpected error: {}", e),
        }
    }
    // With all staging resident: 2×3MiB = 6MiB fits the 8MiB cap; the 3rd
    // and 4th concurrent reservations must be REFUSED. 4 successes (the
    // R3 counterexample: 12MiB written) means the quota is not atomic.
    assert!(
        ok <= 2,
        "at most two 3MiB in-flight reservations fit the 8MiB cap (got {} ok) — quota not atomic (R3-01)",
        ok
    );
    assert!(
        refused >= N - 2,
        "overflowing reservations are controlled refusals (got {})",
        refused
    );
    // The observed peak is the real concurrent total (lock-serialized),
    // not a single-call prediction.
    let peak = ac_artifact_probe::store::PEAK_STAGING.load(std::sync::atomic::Ordering::Relaxed);
    assert!(
        peak > 2 * EACH as u64,
        "PEAK_STAGING must observe the concurrent sum (got {})",
        peak
    );
    // The timer thread has released the locks; a retry succeeds afterwards.
    std::thread::sleep(std::time::Duration::from_millis(300));
    store
        .put_artifact(PutRequest {
            layer: "A04",
            key: key("budget-retry"),
            kind: ChunkKind::DeclSummary,
            payload: vec![9u8; 1024],
            edges: vec![],
            project: None,
        })
        .map(|_| ())
        .unwrap_or_else(|e| panic!("retry after drain must succeed: {}", e));
}

// ---------------------------------------------------------------------------
// R3-02: the GC total counts the PHYSICAL store — a crashed F2 orphan (a
// valid framed CAS blob with no metadata row) participates in the
// convergence decision and is reclaimed by the pass.
// ---------------------------------------------------------------------------

#[test]
fn r3_f02_gc_counts_and_reclaims_crash_orphans() {
    let root = scratch("r3-f02");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    // A legal 3MiB framed CAS blob with NO metadata rows — exactly the F2
    // crash-orphan state, written through the same framing as a real put.
    let payload = vec![7u8; 3 * 1024 * 1024];
    let framed = frame_chunk(ChunkKind::DeclSummary, &payload);
    let cid = ChunkId::from_parts(&[framed.as_slice()]).0;
    let blob = store
        .root
        .join("blocks")
        .join("blobs")
        .join(&cid[..2])
        .join(&cid);
    std::fs::create_dir_all(blob.parent().unwrap()).unwrap();
    std::fs::write(&blob, &framed).unwrap();
    let before = store.space();
    assert!(
        before.total > 1024 * 1024,
        "orphan blob counted: {}",
        before.total
    );

    let receipt = run_gc(
        &store,
        GcConfig {
            high: 1024 * 1024,
            low: 512 * 1024,
        },
    )
    .unwrap();
    assert!(receipt.ran, "physical total is above high");
    assert!(
        receipt.orphans_removed >= 1,
        "the crash orphan is RECLAIMED by the pass: {:?}",
        receipt
    );
    let after = store.space();
    assert!(
        after.total <= 512 * 1024,
        "physical TOTAL (orphan included) converges below low: {} → {} (R3-02)",
        before.total,
        after.total
    );
    assert!(
        receipt.backoff.is_none(),
        "no false 'healthy' pass: {:?}",
        receipt.backoff
    );
}

// ---------------------------------------------------------------------------
// R3-03: GC refuses to operate through a reparse point at a managed
// NAMESPACE ROOT (staging), preserving external sentinels — the same gate
// recover has, now shared.
// ---------------------------------------------------------------------------

#[test]
fn r3_f03_gc_staging_root_junction_refused_and_sentinel_survives() {
    let root = scratch("r3-f03");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    // External directory with a stale staging dir + precious file.
    let external = root.join("user-external");
    let stale = external.join("stale");
    std::fs::create_dir_all(&stale).unwrap();
    let precious = stale.join("precious.txt");
    std::fs::write(&precious, b"user data").unwrap();
    // Make it look grace-expired (mtime 1h ago).
    let one_hour_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    let stale_path = stale.display().to_string();
    let stamp = format!(
        "[IO.File]::SetLastWriteTimeUtc('{}', [DateTime]::UtcNow.AddHours(-1))",
        stale_path
    );
    let _ = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &stamp])
        .output();
    let _ = one_hour_ago;
    // Replace staging with a junction to the external dir.
    let staging = store.root.join("staging");
    std::fs::remove_dir(&staging).unwrap();
    let status = std::process::Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            staging.to_str().unwrap(),
            external.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(status.status.success(), "junction setup failed");
    assert!(is_reparse(&staging));

    // GC (hair-trigger so it would definitely walk staging) must REFUSE.
    match run_gc(&store, GcConfig { high: 1, low: 0 }) {
        Err(StoreError::Boundary(msg)) => {
            assert!(msg.contains("staging"), "{}", msg);
        }
        other => panic!(
            "GC must refuse a staging-root junction (R3-03), got {:?}",
            other.map(|r| r.staging_removed)
        ),
    }
    assert!(precious.is_file(), "external sentinel survived");
    // Cleanup the junction itself (rmdir never touches the target).
    let rm = std::process::Command::new("cmd")
        .args(["/C", "rmdir", staging.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(rm.status.success());
    assert!(precious.is_file());
}

// ---------------------------------------------------------------------------
// R3-01 sibling: after the budget drain, same-key concurrent publishers
// still behave (conflict semantics unaffected by the budget lock).
// ---------------------------------------------------------------------------

#[test]
fn r3_f01_budget_lock_does_not_break_same_key_serialization() {
    let root = scratch("r3-f01b");
    let store = Arc::new(ManagedStore::open(&root.join("store")).unwrap());
    let barrier = Arc::new(std::sync::Barrier::new(2));
    let mut handles = Vec::new();
    for (t, payload) in [(0u8, b"alpha".to_vec()), (1, b"beta".to_vec())] {
        let store = store.clone();
        let barrier = barrier.clone();
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            store
                .put_artifact(PutRequest {
                    layer: "A04",
                    key: key("same-key"),
                    kind: ChunkKind::DeclSummary,
                    payload,
                    edges: vec![],
                    project: None,
                })
                .map(|_| t)
                .map_err(|e| e.to_string())
        }));
    }
    let mut winners = 0usize;
    let mut conflicts = 0usize;
    let mut winner_payload = None;
    for h in handles {
        match h.join().unwrap() {
            Ok(t) => {
                winners += 1;
                winner_payload = Some(t);
            }
            Err(e) if e.contains("conflict") => conflicts += 1,
            Err(e) => panic!("unexpected: {}", e),
        }
    }
    assert_eq!(winners, 1, "exactly one winner");
    assert_eq!(conflicts, 1, "the loser reports the conflict");
    let loaded = store
        .get_artifact("A04", &key("same-key"))
        .unwrap()
        .unwrap();
    let expected: &[u8] = if winner_payload == Some(0) {
        b"alpha"
    } else {
        b"beta"
    };
    assert_eq!(loaded.payload, expected, "winner bytes intact");
}
