//! PLAN-938 whole-namespace commit admission: a put's permanent growth is
//! admitted against the namespace ceiling under an exclusive lock held from
//! the reading THROUGH the metadata commit, so concurrent publishers cannot
//! both pass a stale total and overshoot. The ceiling only tightens.

#![cfg(windows)]

#[path = "../common/mod.rs"]
mod common;

use ace_artifacts::codec::{ChunkKind, CHUNK_HEADER_LEN};
use ace_artifacts::schema;
use ace_artifacts::store::{ManagedStore, PutRequest, StoreError};
use common::scratch;
use std::sync::{Arc, Barrier};

fn key(s: &str) -> ace_artifacts::identity::QueryKey {
    ace_artifacts::identity::QueryKey::new(
        schema::layer("A04").unwrap(),
        vec![("case".into(), s.into())],
    )
}

fn put(store: &ManagedStore, name: &str, byte: u8, size: usize) -> Result<String, StoreError> {
    store.put_artifact(PutRequest {
        layer: "A04",
        key: key(name),
        kind: ChunkKind::DeclSummary,
        payload: vec![byte; size],
        edges: vec![],
        project: None,
    })
}

fn is_namespace_refusal(e: &StoreError) -> bool {
    matches!(e, StoreError::ResourceExhausted(m) if m.contains("namespace"))
}

/// The commit's metadata allowance: fixed and page-granular (WAL frame set +
/// row slack), independent of the incoming block. Mirrors the admission
/// constant in `store/mod.rs` (PLAN-941 single-count fix).
const META_ALLOWANCE: u64 = 48 * 1024;

/// The admission's measurement base: the PERMANENT namespace (total minus
/// the transient WAL/SHM bytes), mirroring `store/mod.rs`.
fn permanent(s: ace_artifacts::store::SpaceReport) -> u64 {
    s.total - (s.db - s.db_main)
}

#[test]
fn namespace_budget_admits_before_writes_and_spans_the_metadata_commit() {
    let cap: u64 = 1 << 20;
    // The frozen ceiling never widens through the host seam.
    let root = scratch("namespace-budget-seq");
    assert!(matches!(
        ManagedStore::open_with_namespace_budget(&root, ManagedStore::NAMESPACE_BUDGET + 1),
        Err(StoreError::ResourceExhausted(_))
    ));
    let store = ManagedStore::open_with_namespace_budget(&root, cap).unwrap();

    // Sequential boundary: 64 KiB payloads commit until the projected
    // growth (block + metadata allowance) would cross the ceiling; the
    // refusal is a named namespace refusal, leaves no staging behind, and
    // the committed blocks stay under the ceiling.
    let size = 64 * 1024;
    let mut committed = 0u64;
    let mut count = 0u8;
    let refusal = loop {
        match put(&store, &format!("seq-{count}"), count, size) {
            Ok(_) => {
                committed += (size + CHUNK_HEADER_LEN) as u64;
                count += 1;
                assert!(count < 32, "the namespace ceiling never engaged");
            }
            Err(e) => break e,
        }
    };
    assert!(is_namespace_refusal(&refusal), "{refusal:?}");
    assert!(
        count >= 4,
        "fewer commits than the ceiling allows ({count})"
    );
    let space = store.space();
    assert!(
        space.blocks <= cap,
        "committed blocks {} overshoot the ceiling",
        space.blocks
    );
    assert!(space.blocks >= committed, "a committed block is missing");
    assert_eq!(space.staging, 0, "a refused put left staging behind");
    // A refusal is sticky only in the sense that the same growth still does
    // not fit; a small put that fits is still admitted.
    put(&store, "seq-small", 0xAA, 512).unwrap();
    assert!(store.space().blocks <= cap);

    // Concurrent: pre-fill until exactly ONE more 100 KB block fits under the
    // projection; two publishers released by a barrier both read the total
    // before either commits. The commit-spanning lock makes the second
    // publisher observe the first commit and refuse; without it, both pass
    // the stale reading and the blocks overshoot the ceiling.
    let root = scratch("namespace-budget-conc");
    let store = Arc::new(ManagedStore::open_with_namespace_budget(&root, cap).unwrap());
    let payload = 300_000usize;
    let framed = (payload + CHUNK_HEADER_LEN) as u64;
    // PLAN-941 single-count window over the PERMANENT base: pre-fill until
    // exactly ONE race block fits (block + fixed metadata allowance) but TWO
    // do not; the trailing 96 KiB slack absorbs one fill step's growth.
    let target_lo = cap - 2 * framed - META_ALLOWANCE + 96 * 1024;
    let mut filler = 0u32;
    while permanent(store.space()) < target_lo {
        put(
            &store,
            &format!("fill-{filler}"),
            (filler % 251) as u8,
            16 * 1024,
        )
        .unwrap();
        filler += 1;
        assert!(filler < 4096, "pre-fill never reached the target");
    }
    let pre = permanent(store.space());
    // PLAN-941 single-count admission: one publisher fits (block + fixed
    // metadata allowance), two do not.
    assert!(
        pre + framed + META_ALLOWANCE <= cap,
        "pre-fill {pre} leaves no room for the first publisher"
    );
    assert!(
        pre + framed + framed + META_ALLOWANCE > cap,
        "pre-fill {pre} leaves room for a second publisher too"
    );
    let barrier = Arc::new(Barrier::new(2));
    let publishers: Vec<_> = (0..2u8)
        .map(|i| {
            let store = store.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                put(&store, &format!("race-{i}"), 0x40 + i, payload)
            })
        })
        .collect();
    let results: Vec<_> = publishers
        .into_iter()
        .map(|p| p.join().expect("publisher thread"))
        .collect();
    let ok = results.iter().filter(|r| r.is_ok()).count();
    let refused = results
        .iter()
        .filter(|r| matches!(r, Err(e) if is_namespace_refusal(e)))
        .count();
    assert_eq!(
        (ok, refused),
        (1, 1),
        "the commit-spanning namespace admission did not serialize the race: {results:?}"
    );
    let space = store.space();
    assert!(
        space.blocks <= cap,
        "concurrent publishers overshot the namespace ceiling: blocks {}",
        space.blocks
    );
    assert_eq!(space.staging, 0);
}

/// PLAN-941 marginal admission (the b8987bb0b double-count bug in
/// miniature): a commit whose REAL post-write total (existing namespace +
/// new block + bounded metadata) fits the ceiling must be admitted — even
/// when the new block itself is the largest so far. The former projection
/// added the whole block a second time as its own metadata allowance and
/// refused exactly these marginal commits (the 64 MiB streaming suite's
/// last block was rejected over 4,718 bytes with the real total at
/// 127.87 MiB ≤ 128 MiB).
#[test]
fn namespace_budget_admits_marginal_commit_whose_real_total_fits() {
    let cap: u64 = 1 << 20;
    let root = scratch("namespace-budget-marginal");
    let store = ManagedStore::open_with_namespace_budget(&root, cap).unwrap();
    let size = 128 * 1024;
    let framed = (size + CHUNK_HEADER_LEN) as u64;
    // Pre-fill until exactly ONE more block fits with the fixed allowance:
    // total + framed + META_ALLOWANCE <= cap, but the double-count
    // projection (total + 2*framed + 4096) would refuse it. Fine-grained
    // 512 B fillers keep the overshoot inside the ~framed-sized window.
    // Stop one fill step below the admission edge (a 512 B filler's real
    // growth is ~29 KiB with its metadata page); the window between the
    // single-count edge and the double-count refusal is ~framed-45 KiB wide.
    let target_lo = cap - framed - META_ALLOWANCE - 32 * 1024;
    let mut filler = 0u32;
    while permanent(store.space()) < target_lo {
        put(&store, &format!("fill-{filler}"), (filler % 251) as u8, 512).unwrap();
        filler += 1;
        assert!(filler < 8192, "pre-fill never reached the target");
    }
    let pre = permanent(store.space());
    assert!(
        pre + framed + META_ALLOWANCE <= cap,
        "pre-fill {pre} left no room for the marginal commit"
    );
    assert!(
        pre + 2 * framed + 4096 > cap,
        "pre-fill {pre} does not reproduce the double-count refusal window"
    );
    // The marginal commit is admitted and the PERMANENT namespace stays
    // under the ceiling (one commit's metadata rides in the fixed allowance;
    // the transient WAL/SHM high-water — bounded by auto-checkpoint and
    // returned by the R2-05 shrink — rides above the ceiling by design).
    put(&store, "marginal", 0x5A, size).unwrap();
    let space = store.space();
    let perm = space.total - (space.db - space.db_main);
    assert!(space.blocks <= cap, "blocks {} overshoot", space.blocks);
    assert!(perm <= cap, "permanent {} overshoot", perm);
    assert_eq!(space.staging, 0);
}
