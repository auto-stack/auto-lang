//! V-05 isolation tests (§5.3): per-stage expected hit/miss is explicit
//! for profile / target / toolchain / body / path changes; unsupported
//! targets are controlled rejections before anything is produced.

#![cfg(windows)]

mod common;

use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::store::ManagedStore;
use common::scratch;

#[test]
fn invalidation_body_only_change_matrix() {
    let root = scratch("inv-body");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let v1 = root.join("v1.at");
    let v2 = root.join("v2.at");
    std::fs::write(&v1, "fn main() int { return 3 * 9 }").unwrap();
    std::fs::write(&v2, "fn main() int { return 3 * 10 }").unwrap();

    let out1 = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: v1.clone(),
            work_dir: root.join("w1"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out1.exit_code, Some(27));
    let out2 = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: v2.clone(),
            work_dir: root.join("w2"),
            ..Default::default()
        },
    )
    .unwrap();
    // Explicit matrix: snapshot MISS (new bytes), summary MISS+recompute
    // with CONTENT identity preserved (signatures equal; the artifact row
    // key follows the snapshot), HIR MISS, object MISS + re-lower.
    assert_eq!(out2.counters.parse, 1, "body change → full miss");
    assert_eq!(
        out2.counters.lower, 1,
        "object must be invalidated on body change"
    );
    assert_eq!(out2.exit_code, Some(30));
    // Summary content identity across the body edit (rebuilt from source).
    let s1 = ac_artifact_probe::codec::build_decl_summary(
        &auto_ac_prototype::source::parse_program(&std::fs::read_to_string(&v1).unwrap()).unwrap(),
        "main",
    );
    let s2 = ac_artifact_probe::codec::build_decl_summary(
        &auto_ac_prototype::source::parse_program(&std::fs::read_to_string(&v2).unwrap()).unwrap(),
        "main",
    );
    assert_eq!(
        ac_artifact_probe::codec::encode_decl_summary(&s1).unwrap(),
        ac_artifact_probe::codec::encode_decl_summary(&s2).unwrap(),
        "same signatures ⇒ same summary content (shared chunk)"
    );
}

#[test]
fn invalidation_profile_change_misses_all_layers() {
    let root = scratch("inv-profile");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    // Scalar compatibility control (§4): a source on the SHARED core∩scalar
    // surface (scalar replaces `while` with conditional `for`, so the
    // control uses plain arithmetic).
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { var i = 2; i = i * 2; return i }").unwrap();
    let core_out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src.clone(),
            work_dir: root.join("w-core"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(core_out.exit_code, Some(4));
    let blocks_before = store.space().blocks_count;

    // Scalar compatibility control (§4): same source, wider surface,
    // different identity triple ⇒ every layer misses and rebuilds.
    let scalar_out = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src,
            work_dir: root.join("w-scalar"),
            profile: auto_ac_prototype::source::SourceProfile::Scalar,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        scalar_out.exit_code,
        Some(4),
        "scalar rebuild preserves semantics"
    );
    assert_eq!(scalar_out.counters.parse, 1, "profile change → full miss");
    assert_eq!(scalar_out.counters.lower, 1);
    assert!(
        store.space().blocks_count > blocks_before,
        "profile change adds new content"
    );
    // Core artifacts remain independently valid (hit again in-process).
    let core_again = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: root.join("m.at"),
            work_dir: root.join("w-core"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(core_again.counters.parse, 0);
    assert_eq!(core_again.exit_code, Some(4));
}

#[test]
fn invalidation_unsupported_target_rejected_before_production() {
    let root = scratch("inv-target");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 1 }").unwrap();
    let blocks_before = store.space().blocks_count;
    let err = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src,
            work_dir: root.join("w"),
            target: "aarch64-pc-windows-msvc".into(),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("unsupported target"),
        "controlled rejection: {}",
        err
    );
    // No fake products were made.
    assert_eq!(store.space().blocks_count, blocks_before);
}

#[test]
fn invalidation_entry_change_selects_different_object() {
    let root = scratch("inv-entry");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src = root.join("m.at");
    std::fs::write(
        &src,
        "fn main() int { return second() }\nfn second() int { return 11 }",
    )
    .unwrap();
    let a = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src.clone(),
            work_dir: root.join("w1"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(a.exit_code, Some(11));
    // Same object chunk would serve either entry only if the entry were
    // not a key component: it is, so `second` as entry lowers its own
    // wrapper target (a DIFFERENT object key) and exits 11 directly.
    let b = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src,
            work_dir: root.join("w2"),
            entry: "second".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(b.exit_code, Some(11));
    assert_eq!(b.counters.lower, 1, "entry is an object-key component");
    assert_ne!(
        a.artifacts.iter().find(|(l, _)| l == "A09").unwrap().1,
        b.artifacts.iter().find(|(l, _)| l == "A09").unwrap().1,
        "different entry ⇒ different object artifact"
    );
}

#[test]
fn invalidation_toolchain_change_keeps_object_hits_but_relinks() {
    let root = scratch("inv-toolchain");
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let src = root.join("m.at");
    std::fs::write(&src, "fn main() int { return 21 * 2 }").unwrap();
    let a = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src.clone(),
            work_dir: root.join("w"),
            toolchain_fp: "lld-v1".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(a.exit_code, Some(42));
    let a10_v1 = a
        .artifacts
        .iter()
        .find(|(l, _)| l == "A10")
        .unwrap()
        .1
        .clone();

    // Toolchain fingerprint changes the LINK receipt (A10), not the object:
    // the object still hits (no re-lower), a fresh link happens.
    let b = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: src,
            work_dir: root.join("w"),
            toolchain_fp: "lld-v2".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(b.exit_code, Some(42));
    assert_eq!(
        b.counters.lower, 0,
        "object hit: toolchain does not invalidate lowering"
    );
    assert_eq!(b.counters.link, 1, "relink is real");
    assert_ne!(
        a10_v1,
        b.artifacts.iter().find(|(l, _)| l == "A10").unwrap().1,
        "toolchain change ⇒ new link receipt"
    );
}
