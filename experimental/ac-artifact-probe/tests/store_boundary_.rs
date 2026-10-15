//! V-10 store-boundary tests (§5.4): missing source / missing toolchain are
//! explicit unavailable states, distinct from misses; out-of-root references
//! and reparse jumps are refused; user property (source copies, delivered
//! exes, files near the store) is never in the deletion set.

#![cfg(windows)]

mod common;

use ac_artifact_probe::driver::{vertical_and_run, DriverError, VerticalCfg};
use ac_artifact_probe::store::{run_gc, GcConfig, ManagedStore, StoreError};
use common::scratch;
use std::process::Command;

#[test]
fn store_boundary_missing_source_without_hit_is_explicit_unavailable() {
    let root = scratch("bound-src");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let absent = root.join("no-such.at");
    let err = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: absent.clone(),
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap_err();
    match err {
        DriverError::SourceUnavailable { path, .. } => {
            assert_eq!(path, absent, "the error names the exact missing input");
        }
        other => panic!("expected SourceUnavailable, got {}", other),
    }
}

#[test]
fn store_boundary_missing_toolchain_is_explicit_unavailable_not_fake_hit() {
    let root = scratch("bound-tool");
    let store_root = root.join("store");
    let store = ManagedStore::open(&store_root).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 3 }").unwrap();
    // Prime the store so the next run would be an object hit.
    let out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src.clone(),
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(3));

    // Toolchain (rust-lld) unavailable: the hit exists but cannot be
    // delivered — a controlled link error, never a fake artifact.
    std::env::set_var("AC741_RUST_LLD", root.join("definitely-missing-lld.exe"));
    let err = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src,
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string());
    std::env::remove_var("AC741_RUST_LLD");
    match err.unwrap_err() {
        msg if msg.contains("AC741_RUST_LLD") || msg.contains("missing") => {}
        other => panic!("expected controlled link failure, got {}", other),
    }
}

#[test]
fn store_boundary_not_managed_root_refuses_deleting_operations() {
    let root = scratch("bound-marker");
    // A directory that LOOKS like a store but has no ownership marker.
    let fake = root.join("fake-store");
    std::fs::create_dir_all(fake.join("blocks").join("blobs").join("ab")).unwrap();
    std::fs::write(
        fake.join("blocks")
            .join("blobs")
            .join("ab")
            .join("deadbeef"),
        b"junk",
    )
    .unwrap();
    let err = match ManagedStore::open_existing(&fake) {
        Err(e) => e,
        Ok(_) => panic!("open_existing must refuse an unmanaged directory"),
    };
    assert!(matches!(err, StoreError::NotManaged(_)), "{}", err);
}

#[test]
fn store_boundary_gc_never_leaves_the_managed_namespace() {
    let root = scratch("bound-namespace");
    let store_root = root.join("store");
    let store = ManagedStore::open(&store_root).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 4 }").unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src.clone(),
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(4));

    // User property around the store: the source itself, the delivered
    // exe, and a stray file INSIDE the store root but OUTSIDE the managed
    // namespaces (blocks/staging/trash).
    let exe_copy = root.join("delivered-app.exe");
    std::fs::copy(&out.exe, &exe_copy).unwrap();
    let stray = store_root.join("user-notes.txt");
    std::fs::write(&stray, "user data").unwrap();
    let hash_of = |p: &std::path::Path| -> String {
        blake3::hash(&std::fs::read(p).unwrap())
            .to_hex()
            .to_string()
    };
    let (h_src, h_exe, h_stray) = (hash_of(&src), hash_of(&exe_copy), hash_of(&stray));

    store.remove_project_roots("p").unwrap();
    let receipt = run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    assert!(receipt.deleted_chunks >= 1, "managed content WAS reclaimed");
    assert_eq!(hash_of(&src), h_src, "user source untouched");
    assert_eq!(hash_of(&exe_copy), h_exe, "delivered exe untouched");
    assert_eq!(
        hash_of(&stray),
        h_stray,
        "stray file inside store root untouched (outside managed namespaces)"
    );
}

#[test]
fn store_boundary_reparse_jump_is_not_followed_by_scans() {
    let root = scratch("bound-reparse");
    let store_root = root.join("store");
    let store = ManagedStore::open(&store_root).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 2 }").unwrap();
    let out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src,
            work_dir: root.join("w"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out.exit_code, Some(2));

    // Sentinel directory OUTSIDE the store that a junction would expose.
    let sentinel_dir = root.join("user-precious");
    std::fs::create_dir_all(&sentinel_dir).unwrap();
    let precious = sentinel_dir.join("data.bin");
    std::fs::write(&precious, b"precious-user-bytes").unwrap();
    let h_before = {
        use std::io::Read;
        let mut f = std::fs::File::open(&precious).unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        buf
    };

    // Junction INSIDE the blob namespace pointing at the sentinel dir. The
    // worktree red line forbids junctions in WORKTREES; this is a throwaway
    // temp store used precisely to prove the scanner refuses the jump.
    let shard = store_root.join("blocks").join("blobs").join("zz");
    std::fs::create_dir_all(&shard).unwrap();
    let junction = shard.join("evil-jump");
    let status = Command::new("cmd")
        .args([
            "/C",
            "mklink",
            "/J",
            junction.to_str().unwrap(),
            sentinel_dir.to_str().unwrap(),
        ])
        .output()
        .expect("mklink");
    assert!(
        status.status.success(),
        "junction setup failed: {:?}",
        status
    );
    assert!(
        ac_artifact_probe::store::is_reparse(&junction),
        "junction detected as reparse"
    );

    // Recovery (orphan scan) and GC must NOT delete through the junction.
    let rec = ac_artifact_probe::store::recover(&store, true).unwrap();
    let _ = rec;
    store.remove_project_roots("p").unwrap();
    let _ = run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    let h_after = std::fs::read(&precious).unwrap();
    assert_eq!(h_before, h_after, "sentinel data survived scans");
    // And the junction itself is still there for manual inspection (we do
    // NOT remove it via the store; clean up explicitly below).
    assert!(junction.exists());

    // Cleanup: remove the junction by rmdir (junction removal never
    // touches the target).
    let rm = Command::new("cmd")
        .args(["/C", "rmdir", junction.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(rm.status.success());
    assert!(precious.exists());
}
