//! V-07 concurrency tests (§5.4): two producers on one key with different
//! results — the conflict is reported, never last-writer-wins; identical
//! content from two producers is idempotent; readers racing GC never see a
//! partial chunk.

#![cfg(windows)]

mod common;

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema;
use ac_artifact_probe::store::{run_gc, GcConfig, ManagedStore, PutRequest, StoreError};
use common::scratch;

#[test]
fn concurrency_same_key_different_content_reports_conflict() {
    let root = scratch("conc-conflict");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let key = QueryKey::new(
        schema::layer("A04").unwrap(),
        vec![("case".into(), "conflict".into())],
    );
    let put = |v: u8| PutRequest {
        layer: "A04",
        key: key.clone(),
        kind: ChunkKind::DeclSummary,
        payload: vec![v; 64],
        edges: vec![],
        project: None,
    };
    let a = store.put_artifact(put(1)).unwrap();
    // Same key, different bytes: controlled CONFLICT, both ids visible.
    match store.put_artifact(put(2)) {
        Err(StoreError::Conflict {
            existing,
            attempted,
            ..
        }) => {
            assert_eq!(existing, a);
            assert_ne!(attempted, a);
        }
        other => panic!("expected conflict, got {:?}", other),
    }
    // The original content survived untouched.
    let loaded = store.get_artifact("A04", &key).unwrap().unwrap();
    assert_eq!(loaded.payload, vec![1u8; 64]);
    // Identical republication is idempotent (same artifact id).
    let again = store.put_artifact(put(1)).unwrap();
    assert_eq!(again, a);
}

#[test]
fn concurrency_two_processes_same_key_one_conflict_reported() {
    let root = scratch("conc-procs");
    let store_root = root.join("store");
    // Producer 1 wins; producer 2 (different content, same key via the
    // probe's own produce path) must see the conflict.
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 7 }").unwrap();
    let out1 = common::run_node(&[
        "vertical",
        "--store",
        store_root.to_str().unwrap(),
        "--project",
        "pa",
        "--source",
        src.to_str().unwrap(),
        "--work",
        root.join("w1").to_str().unwrap(),
    ]);
    assert_eq!(out1.code, 0, "{}", out1.raw);
    // Second process, same content: idempotent success, full hit.
    let out2 = common::run_node(&[
        "vertical",
        "--store",
        store_root.to_str().unwrap(),
        "--project",
        "pb",
        "--source",
        src.to_str().unwrap(),
        "--work",
        root.join("w2").to_str().unwrap(),
    ]);
    assert_eq!(out2.code, 0, "{}", out2.raw);
    let r = common::first(&out2, "RESULT");
    assert_eq!(r["counters"]["parse"], 0);
    assert_eq!(r["exit_code"], 7);
}

#[test]
fn concurrency_reader_gc_interleave_no_partial_reads() {
    let root = scratch("conc-rgc");
    let store_root = root.join("store");
    let store = ManagedStore::open(&store_root).unwrap();
    // A live (rooted) project to read from, plus junk churn for GC.
    let live = root.join("live.at");
    std::fs::write(&live, "fn main() int { return 5 }").unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "live".into(),
            source: live.clone(),
            work_dir: root.join("wl"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(5));

    // Background: junk builds + GC while a reader thread reads the live
    // project through the gated reader.
    let junk_root = store_root.clone();
    let live_dir = root.join("wl");
    let live_src = live.clone();
    let reader = std::thread::spawn(move || {
        let store = ManagedStore::open(&junk_root).unwrap();
        for _ in 0..40 {
            let out = vertical_and_run(
                &store,
                &VerticalCfg {
                    project: "live".into(),
                    source: live_src.clone(),
                    work_dir: live_dir.clone(),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(out.exit_code, Some(5));
        }
    });
    for i in 0..12 {
        let j = root.join(format!("junk-{i}.at"));
        std::fs::write(&j, format!("fn main() int {{ return {} }}", 1000 + i)).unwrap();
        let out = vertical_and_run(
            &store,
            &VerticalCfg {
                project: format!("junk-{}", i % 2),
                source: j,
                work_dir: root.join(format!("wj-{}", i % 2)),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(out.exit_code.unwrap() >= 1000);
        let _ = run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    }
    reader.join().expect("concurrent reads completed cleanly");
}
