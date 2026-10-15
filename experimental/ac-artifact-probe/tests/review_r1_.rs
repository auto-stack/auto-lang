//! Review R1 counterexample regressions (F-01/02/03/04/05/06/08/11):
//! each control mirrors the finding's reproduction; the asserted contract
//! is the invariant, not just the original example.

#![cfg(windows)]

mod common;

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema;
use ac_artifact_probe::store::{
    is_reparse, recover, run_gc, GcConfig, ManagedStore, PutRequest, StoreError,
};
use ac_artifact_probe::{codec, layout};
use common::{run_node, scratch};
use std::path::Path;

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
// F-783-R1-01: a lease/hard root granted while GC runs is NEVER granted
// for a payload GC is deleting (either it was counted before the snapshot,
// or the grant is refused afterwards).
// ---------------------------------------------------------------------------

#[test]
fn r1_f01_grants_during_gc_never_cover_deleted_targets() {
    let root = scratch("r1-f01");
    let store_root = root.join("store");
    let s = ManagedStore::open(&store_root).unwrap();
    // 150 dead chunks + one late target (all unrooted).
    let mut names = Vec::new();
    for i in 0u32..150 {
        let k = key(&format!("junk-{i}"));
        let mut p = vec![0u8; 65536];
        p[..4].copy_from_slice(&i.to_le_bytes());
        put(&s, &k, &p);
        names.push(k);
    }
    let target = key("lease-target");
    let aid = put(&s, &target, b"lease target survives");
    drop(s);

    let gc_root = store_root.clone();
    let worker = std::thread::spawn(move || {
        run_gc(
            &ManagedStore::open(&gc_root).unwrap(),
            GcConfig { high: 1, low: 0 },
        )
        .unwrap()
    });
    // Race window: acquire leases/roots for the still-listed target while
    // the GC thread is deleting the 150 candidates.
    let s2 = ManagedStore::open(&store_root).unwrap();
    let mut granted = 0usize;
    let mut refused = 0usize;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while granted == 0 && refused == 0 && std::time::Instant::now() < deadline {
        match s2.acquire_lease("during-gc", 60, Some(&aid)) {
            Ok(lease) => {
                // INVARIANT: a granted lease's target must be readable.
                let readable = s2.get_artifact("A04", &target).unwrap().is_some();
                let live = s2.lease_live(&lease.lease_id).unwrap();
                assert!(
                    readable && live,
                    "granted lease covers a deleted/invalid target (R1 F-01)"
                );
                lease.release();
                granted += 1;
            }
            Err(StoreError::TargetMissing(_)) | Err(StoreError::Busy(_)) => refused += 1,
            Err(e) => panic!("unexpected error racing GC: {}", e),
        }
        match s2.register_root("new-live-project", &aid, "hard") {
            Ok(()) => {
                let readable = s2.get_artifact("A04", &target).unwrap().is_some();
                assert!(readable, "granted hard root covers a deleted target");
                let _ = s2.remove_roots("new-live-project", "hard");
            }
            Err(StoreError::TargetMissing(_)) | Err(StoreError::Busy(_)) => {}
            Err(e) => panic!("unexpected root error racing GC: {}", e),
        }
    }
    let receipt = worker.join().unwrap();
    assert!(receipt.deleted_chunks > 0, "GC actually deleted candidates");
    assert!(
        granted + refused > 0,
        "the race window was never exercised (test too fast)"
    );
}

// ---------------------------------------------------------------------------
// F-783-R1-02: perf never erases a directory it does not own.
// ---------------------------------------------------------------------------

#[test]
fn r1_f02_perf_root_never_deletes_unowned_content() {
    let root = scratch("r1-f02");
    // An EXISTING directory with a sentinel and NO marker.
    let victim = root.join("existing-dir");
    std::fs::create_dir_all(&victim).unwrap();
    let sentinel = victim.join("external-sentinel.txt");
    std::fs::write(&sentinel, b"user data").unwrap();
    let report = ac_artifact_probe::perf::run_scenarios(&victim, "p04-while-retest.at", 2);
    assert_eq!(report.scenarios.len(), 6);
    assert!(
        sentinel.is_file(),
        "sentinel inside the given root must survive"
    );
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"user data");
    // All probe scratch went into a fresh marker-carrying subdirectory.
    let runs: Vec<_> = std::fs::read_dir(&victim)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .collect();
    let run_dirs: Vec<_> = runs.iter().filter(|p| p.is_dir()).collect();
    assert!(!run_dirs.is_empty(), "perf scratch subdirectory exists");
    for d in &run_dirs {
        assert!(
            d.join("AC783-PERF-ROOT.marker").is_file(),
            "probe subdirectory must carry the ownership marker: {}",
            d.display()
        );
    }
    // A path that is not a reparse point reports false (helper sanity).
    assert!(!is_reparse(Path::new("definitely-not-a-link")));
}

// ---------------------------------------------------------------------------
// F-783-R1-03: republication after GC deletion is readable again.
// ---------------------------------------------------------------------------

#[test]
fn r1_f03_republish_after_gc_delete_restores_readability() {
    let root = scratch("r1-f03");
    let s = ManagedStore::open(&root.join("store")).unwrap();
    let k = key("republish");
    put(&s, &k, b"original");
    assert!(s.get_artifact("A04", &k).unwrap().is_some());
    let gc = run_gc(&s, GcConfig { high: 1, low: 0 }).unwrap();
    assert_eq!(gc.deleted_chunks, 1);
    assert!(
        s.get_artifact("A04", &k).unwrap().is_none(),
        "deleted = miss"
    );

    // Identical republication: chunk state transitions back to live.
    put(&s, &k, b"original");
    assert!(
        s.get_artifact("A04", &k).unwrap().is_some(),
        "successful republication must be READABLE again (R1 F-03)"
    );

    // Repeated put/GC/get cycles converge.
    for round in 0..3 {
        let receipt = run_gc(&s, GcConfig { high: 1, low: 0 }).unwrap();
        assert!(receipt.deleted_chunks >= 1, "round {}", round);
        assert!(s.get_artifact("A04", &k).unwrap().is_none());
        put(&s, &k, b"original");
        let loaded = s.get_artifact("A04", &k).unwrap().expect("readable");
        assert_eq!(loaded.payload, b"original");
    }

    // A second key sharing identical content re-puts cleanly after GC.
    let k2 = key("republish-2");
    run_gc(&s, GcConfig { high: 1, low: 0 }).unwrap();
    put(&s, &k2, b"original");
    put(&s, &k, b"original");
    assert!(s.get_artifact("A04", &k).unwrap().is_some());
    assert!(s.get_artifact("A04", &k2).unwrap().is_some());
}

// ---------------------------------------------------------------------------
// F-783-R1-04: the CAS address is validated on every read — a substituted
// but self-consistent chunk is refused, for raw reads AND both consumers.
// ---------------------------------------------------------------------------

#[test]
fn r1_f04_cas_substitution_rejected_at_admission() {
    let root = scratch("r1-f04");
    let s = ManagedStore::open(&root.join("store")).unwrap();
    let k1 = key("one");
    let k2 = key("two");
    put(&s, &k1, b"first");
    put(&s, &k2, b"other");
    let a = s.get_artifact("A04", &k1).unwrap().unwrap();
    let b = s.get_artifact("A04", &k2).unwrap().unwrap();
    let path = |cid: &str| s.root.join("blocks/blobs").join(&cid[..2]).join(cid);
    // Substitute a DIFFERENT valid same-kind chunk at the first address.
    std::fs::copy(path(&b.chunk_id), path(&a.chunk_id)).unwrap();
    match s.get_artifact("A04", &k1) {
        Err(StoreError::ChunkAddressMismatch { requested, actual }) => {
            assert_eq!(requested, a.chunk_id);
            assert_eq!(actual, b.chunk_id);
        }
        other => panic!(
            "substituted chunk must be refused at admission, got {:?}",
            other.map(|o| o.map(|l| l.payload.len()))
        ),
    }
}

#[test]
fn r1_f04_object_consumer_substitution_is_controlled() {
    // Through the real driver: the object blob is swapped for another
    // valid object chunk; the object-hit path must fail controlled BEFORE
    // linking, never link foreign bytes.
    let root = scratch("r1-f04-obj");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src1 = root.join("a.at");
    let src2 = root.join("b.at");
    std::fs::write(&src1, "fn main() int { return 11 }").unwrap();
    std::fs::write(&src2, "fn main() int { return 22 }").unwrap();
    let o1 = ac_artifact_probe::driver::vertical_and_run(
        &store,
        &ac_artifact_probe::driver::VerticalCfg {
            project: "p1".into(),
            source: src1,
            work_dir: root.join("w1"),
            ..Default::default()
        },
    )
    .unwrap();
    let o2 = ac_artifact_probe::driver::vertical_and_run(
        &store,
        &ac_artifact_probe::driver::VerticalCfg {
            project: "p2".into(),
            source: src2,
            work_dir: root.join("w2"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(o1.exit_code, Some(11));
    assert_eq!(o2.exit_code, Some(22));
    let c1 = o1
        .artifacts
        .iter()
        .find(|(l, _)| l == "A09")
        .unwrap()
        .1
        .clone();
    let _ = c1;
    let loaded1 = store
        .get_artifact(
            "A09",
            &ac_artifact_probe::driver::object_key_by_hash(
                &ac_artifact_probe::identity::hir_bundle_key(
                    "auto.source.core-i32.draft",
                    "core-i32-draft",
                    &[],
                    &ac_artifact_probe::identity::SourceSnapshotId(o1.snapshot_id.clone()),
                    codec::HIR_CODEC_VERSION,
                )
                .hash(),
                "x86_64-pc-windows-msvc",
                ac_artifact_probe::identity::BACKEND_ID,
                "main",
            ),
        )
        .unwrap()
        .unwrap();
    let loaded2 = store
        .get_artifact(
            "A09",
            &ac_artifact_probe::driver::object_key_by_hash(
                &ac_artifact_probe::identity::hir_bundle_key(
                    "auto.source.core-i32.draft",
                    "core-i32-draft",
                    &[],
                    &ac_artifact_probe::identity::SourceSnapshotId(o2.snapshot_id.clone()),
                    codec::HIR_CODEC_VERSION,
                )
                .hash(),
                "x86_64-pc-windows-msvc",
                ac_artifact_probe::identity::BACKEND_ID,
                "main",
            ),
        )
        .unwrap()
        .unwrap();
    let path = |cid: &str| store.root.join("blocks/blobs").join(&cid[..2]).join(cid);
    std::fs::copy(path(&loaded2.chunk_id), path(&loaded1.chunk_id)).unwrap();
    // The next object-hit run must refuse the substituted bytes.
    let err = ac_artifact_probe::driver::vertical_and_run(
        &store,
        &ac_artifact_probe::driver::VerticalCfg {
            project: "p1".into(),
            source: root.join("a.at"),
            work_dir: root.join("w1"),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())
    .unwrap_err();
    assert!(
        err.contains("address mismatch") || err.contains("CAS"),
        "substituted object must be refused before linking: {}",
        err
    );
}

// ---------------------------------------------------------------------------
// F-783-R1-05: the layout READER validates framing, address and field
// bounds — corrupt/short/out-of-range inputs are controlled rejections.
// ---------------------------------------------------------------------------

fn r1_manifest(record: String, payload: String) -> layout::Manifest {
    layout::Manifest {
        layout: "A".into(),
        version: 1,
        record_count: 1,
        record_pages: vec![record],
        payload_pages: vec![payload],
        payload_len: 8,
        data_pages: vec![],
        dir_pages: vec![],
    }
}

#[test]
fn r1_f05_layout_reader_rejects_corrupt_substituted_and_malformed() {
    let root = scratch("r1-f05");
    let ls = layout::LayoutStore::new(&root);
    // A minimal legal record page: kind=0, parent=none, spans 0, plen=8,
    // offset=0; payload page of 8 payload bytes at offset 0.
    let mut rec = vec![0u8; layout::PAGE_SIZE];
    rec[14..16].copy_from_slice(&8u16.to_le_bytes());
    let (rid, _) = ls.write_chunk(&rec).unwrap();
    let (pid, _) = ls.write_chunk(&vec![7u8; layout::PAGE_SIZE]).unwrap();

    // Legal sibling: reads fine.
    let mut v = layout::LayoutView::open(&ls, r1_manifest(rid.clone(), pid.clone())).unwrap();
    assert_eq!(v.record(0).unwrap().payload, vec![7u8; 8]);

    // Corrupt frame: checksum-valid failure in the ACTUAL reader.
    let mut bytes = std::fs::read(ls.page_path(&rid)).unwrap();
    bytes[codec::CHUNK_HEADER_LEN] = 99;
    std::fs::write(ls.page_path(&rid), &bytes).unwrap();
    let mut v = layout::LayoutView::open(&ls, r1_manifest(rid.clone(), pid.clone())).unwrap();
    assert!(
        v.record(0).is_err(),
        "corrupt page must be rejected by the reader"
    );

    // Checksum-VALID but structurally invalid offset: offset=PAGE_SIZE-1,
    // len=8 — controlled rejection, never a panic.
    let mut bad = vec![0u8; layout::PAGE_SIZE];
    bad[14..16].copy_from_slice(&8u16.to_le_bytes());
    bad[16..24].copy_from_slice(&((layout::PAGE_SIZE - 1) as u64).to_le_bytes());
    let (badid, _) = ls.write_chunk(&bad).unwrap();
    let mut v = layout::LayoutView::open(&ls, r1_manifest(badid.clone(), pid.clone())).unwrap();
    let result = v.record(0);
    assert!(
        result.is_err(),
        "crossing offset must be Err, got {:?}",
        result.map(|r| r.payload.len())
    );

    // Oversized payload length field.
    let mut big = vec![0u8; layout::PAGE_SIZE];
    big[14..16].copy_from_slice(&999u16.to_le_bytes());
    big[16..24].copy_from_slice(&0u64.to_le_bytes());
    let (bigid, _) = ls.write_chunk(&big).unwrap();
    let mut v = layout::LayoutView::open(&ls, r1_manifest(bigid, pid.clone())).unwrap();
    assert!(
        v.record(0).is_err(),
        "payload length over the population bound must be Err"
    );

    // Substituted page (address mismatch): a DIFFERENT valid page at the
    // manifest's address.
    let (other, _) = ls.write_chunk(&vec![9u8; layout::PAGE_SIZE]).unwrap();
    let mut v = layout::LayoutView::open(&ls, r1_manifest(other, pid)).unwrap();
    assert!(v.record(0).is_err(), "unvalidated substitution must be Err");
}

#[test]
fn r1_f05_layout_b_short_record_and_unknown_layout_rejected() {
    let root = scratch("r1-f05b");
    let ls = layout::LayoutStore::new(&root);
    // B with a short data record: directory says len < B_HEADER.
    let mut dir = vec![0u8; layout::PAGE_SIZE];
    dir[0..4].copy_from_slice(&0u32.to_le_bytes()); // page 0
    dir[4..8].copy_from_slice(&0u32.to_le_bytes()); // offset 0
    dir[8..10].copy_from_slice(&5u16.to_le_bytes()); // len 5 < B_HEADER
    let (dirid, _) = ls.write_chunk(&dir).unwrap();
    let (dataid, _) = ls.write_chunk(&vec![1u8; layout::PAGE_SIZE]).unwrap();
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
    assert!(
        v.record(0).is_err(),
        "short B record must be a controlled rejection"
    );

    // Unknown layout name is refused at open.
    let mut m2 = r1_manifest("x".into(), "y".into());
    m2.layout = "C".into();
    assert!(
        layout::LayoutView::open(&ls, m2).is_err(),
        "unknown layout must be refused at admission"
    );
}

// ---------------------------------------------------------------------------
// F-783-R1-06: recovery reconciles committed orphan rows, trash residue
// and tombstone cascades.
// ---------------------------------------------------------------------------

#[test]
fn r1_f06_recovery_clears_orphan_rows_trash_and_is_idempotent() {
    let root = scratch("r1-f06");
    let s = ManagedStore::open(&root.join("store")).unwrap();
    // Conflict path commits the loser's chunk row with NO artifact.
    let k = key("conflict");
    put(&s, &k, b"winner");
    assert!(s
        .put_artifact(PutRequest {
            layer: "A04",
            key: k.clone(),
            kind: ChunkKind::DeclSummary,
            payload: vec![9u8; 1024],
            edges: vec![],
            project: None,
        })
        .is_err());
    // R4-01: the GC pass itself reclaims the conflict loser (orphan
    // candidate); recovery afterwards is an idempotent no-op for it.
    let rec = recover(&s, true).unwrap();
    let gc = run_gc(&s, GcConfig { high: 1, low: 0 }).unwrap();
    assert!(
        gc.orphans_removed as u64
            + rec.orphan_rows_removed as u64
            + rec.orphan_chunks_removed as u64
            >= 1,
        "conflict-loser chunk reclaimed (GC {} / recover {})",
        gc.orphans_removed,
        rec.orphan_rows_removed
    );
    // Nothing unreachable remains in the block layer.
    assert_eq!(
        s.space().blocks_count,
        0,
        "no unreachable blocks remain: the unrooted winner via GC, the conflict chunk via recovery"
    );

    // Trash residue (a blocked delete's leftover) is retried and cleared.
    std::fs::write(s.root.join("trash/pending-delete"), vec![0u8; 1000]).unwrap();
    let rec = recover(&s, true).unwrap();
    assert_eq!(rec.trash_files_removed, 1, "trash reclaimed");
    assert_eq!(s.space().trash, 0);

    // Idempotence: a second pass is a no-op.
    let rec2 = recover(&s, true).unwrap();
    assert_eq!(rec2.staging_dirs_removed, 0);
    assert_eq!(rec2.orphan_rows_removed, 0);
    assert_eq!(rec2.trash_files_removed, 0);
    assert_eq!(rec2.tombstone_deletes_completed, 0);
}

#[test]
fn r1_f06_tombstone_recovery_cascades_rows() {
    // after-tombstone crash → recovery completes the delete AND converges
    // the metadata rows (artifact/edge/root) like a live GC pass.
    let root = scratch("r1-f06t");
    let store_root = root.join("store");
    let out = run_node(&[
        "crash",
        "--at",
        "after-tombstone",
        "--store",
        store_root.to_str().unwrap(),
        "--project",
        "victim",
        "--source",
        common::fixture_path("p04-while-retest.at")
            .to_str()
            .unwrap(),
        "--work",
        root.join("w").to_str().unwrap(),
    ]);
    assert_ne!(out.code, 0, "fault must abort");
    let s = ManagedStore::open(&store_root).unwrap();
    let rec = recover(&s, true).unwrap();
    assert!(rec.tombstone_deletes_completed >= 1);
    // No zombie rows remain for the reclaimed chunk: republication works.
    let k = key("post-recovery");
    put(&s, &k, b"re-published");
    assert!(s.get_artifact("A04", &k).unwrap().is_some());
}

// ---------------------------------------------------------------------------
// F-783-R1-08: same-key producers are SERIALIZED with a bounded wait (and
// a failed publish cleans its staging); real overlapping processes race
// both identical and conflicting content.
// ---------------------------------------------------------------------------

#[test]
fn r1_f08_busy_chunk_waits_bounded_then_reports_and_cleans_staging() {
    let root = scratch("r1-f08");
    let store_root = root.join("store");
    // Holder keeps the chunk lock for 12s; the contender's bounded wait is
    // 10s → controlled Busy, staging reclaimed.
    let mut holder = std::process::Command::new(common::NODE_EXE);
    holder.args([
        "hold-chunk",
        "--store",
        store_root.to_str().unwrap(),
        "--payload",
        "same chunk",
        "--hold-ms",
        "12000",
    ]);
    holder.stdin(std::process::Stdio::null());
    holder.stdout(std::process::Stdio::null());
    let mut child = holder.spawn().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(500)); // let it lock
    let out = run_node(&[
        "put",
        "--store",
        store_root.to_str().unwrap(),
        "--key",
        "busy",
        "--payload",
        "same chunk",
    ]);
    let err_line = out
        .lines
        .iter()
        .find(|(k, _)| k == "PUT-ERROR")
        .map(|(_, v)| v["error"].as_str().unwrap().to_string());
    let staging: Vec<_> = std::fs::read_dir(store_root.join("staging"))
        .unwrap()
        .flatten()
        .collect();
    let _ = child.wait();
    match err_line {
        Some(e) => assert!(e.contains("busy"), "bounded wait then Busy: {}", e),
        None => {
            // The holder may have released first on a slow machine; then
            // the put must have SUCCEEDED readably (serialization works).
            let put = out
                .lines
                .iter()
                .find(|(k, _)| k == "PUT")
                .map(|(_, v)| v["readable"].as_bool().unwrap())
                .expect("either PUT or PUT-ERROR");
            assert!(put, "serialized put must be readable");
        }
    }
    assert!(
        staging.is_empty(),
        "a failed/finished publish never leaves staging"
    );
}

#[test]
fn r1_f08_overlapping_processes_identical_and_conflicting() {
    let root = scratch("r1-f08c");
    let store_root = root.join("store");
    std::fs::create_dir_all(&store_root).unwrap();

    // Two processes produce the SAME content under one key simultaneously.
    let s1 = store_root.display().to_string();
    let s2 = store_root.display().to_string();
    let h1 = std::thread::spawn(move || {
        run_node(&[
            "put",
            "--store",
            &s1,
            "--key",
            "race",
            "--payload",
            "identical-content",
        ])
    });
    let h2 = std::thread::spawn(move || {
        run_node(&[
            "put",
            "--store",
            &s2,
            "--key",
            "race",
            "--payload",
            "identical-content",
        ])
    });
    let a = h1.join().unwrap();
    let b = h2.join().unwrap();
    let oks = [&a, &b]
        .iter()
        .filter(|o| o.lines.iter().any(|(k, _)| k == "PUT"))
        .count();
    assert_eq!(
        oks, 2,
        "identical content from two live producers: both succeed\n{}\n{}",
        a.raw, b.raw
    );

    // Conflicting content under one key: exactly one winner; the loser
    // reports the conflict naming both content ids — never overwrite.
    let s1 = store_root.display().to_string();
    let s2 = store_root.display().to_string();
    let h1 = std::thread::spawn(move || {
        run_node(&[
            "put",
            "--store",
            &s1,
            "--key",
            "conflict-race",
            "--payload",
            "conflict-A",
        ])
    });
    let h2 = std::thread::spawn(move || {
        run_node(&[
            "put",
            "--store",
            &s2,
            "--key",
            "conflict-race",
            "--payload",
            "conflict-B",
        ])
    });
    let a = h1.join().unwrap();
    let b = h2.join().unwrap();
    let wins = [&a, &b]
        .iter()
        .filter(|o| o.lines.iter().any(|(k, _)| k == "PUT"))
        .count();
    let conflicts = [&a, &b]
        .iter()
        .filter(|o| {
            o.lines.iter().any(|(k, v)| {
                k == "PUT-ERROR" && v["error"].as_str().unwrap_or("").contains("conflict")
            })
        })
        .count();
    // R2-03 tightening: DIFFERENT payloads under one key admit exactly ONE
    // winner (wins==2 would mean both committed different bytes — broken).
    assert_eq!(
        wins, 1,
        "exactly one winner for conflicting payloads\n{}\n{}",
        a.raw, b.raw
    );
    assert_eq!(
        conflicts, 1,
        "the loser reports the conflict naming both ids\n{}\n{}",
        a.raw, b.raw
    );
    // The winner is readable in-process and carries ITS payload bytes.
    let k = key("conflict-race");
    let s = ManagedStore::open(&store_root).unwrap();
    let loaded = s.get_artifact("A04", &k).unwrap().expect("winner readable");
    assert!(
        loaded.payload == b"conflict-A" || loaded.payload == b"conflict-B",
        "winner bytes intact: {:?}",
        String::from_utf8_lossy(&loaded.payload)
    );
}

// ---------------------------------------------------------------------------
// F-783-R1-11: diagnostics bind to the CURRENT project's source path.
// ---------------------------------------------------------------------------

#[test]
fn r1_f11_diagnostics_name_the_current_project_path() {
    let root = scratch("r1-f11");
    let store_root = root.join("store");
    for name in ["project-alpha.at", "nested/project-beta.at"] {
        let p = root.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "fn main() int { return true }").unwrap();
        let out = run_node(&[
            "vertical",
            "--store",
            store_root.to_str().unwrap(),
            "--project",
            "p",
            "--source",
            p.to_str().unwrap(),
            "--work",
            root.join("w").to_str().unwrap(),
        ]);
        let err = out
            .lines
            .iter()
            .find(|(k, _)| k == "ERROR")
            .map(|(_, v)| v["error"].as_str().unwrap().to_string())
            .expect("bad source must error");
        assert!(
            err.contains(p.file_name().unwrap().to_string_lossy().as_ref()),
            "diagnostic must name the consuming project's source ({}), got: {}",
            p.display(),
            err
        );
    }
}
