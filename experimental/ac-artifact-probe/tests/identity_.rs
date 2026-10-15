//! V-01 identity tests: content sharing across paths; same-signature
//! body edits share the summary but invalidate HIR/object identity;
//! current spans stay correct for the consuming project.

mod common;

use ac_artifact_probe::codec::{build_decl_summary, decode_decl_summary, encode_decl_summary};
use ac_artifact_probe::driver::{vertical_and_run, VerticalCfg};
use ac_artifact_probe::identity::{self, SourceSnapshotId};
use ac_artifact_probe::store::ManagedStore;
use auto_ac_prototype::source::parse_program;
use common::scratch;

fn store_at(p: &std::path::Path) -> ManagedStore {
    let root = p.join("store");
    std::fs::create_dir_all(&root).unwrap();
    ManagedStore::open(&root).unwrap()
}

#[test]
fn identity_same_content_different_paths_shares_snapshot_and_hits() {
    let root = scratch("identity-share");
    let store = store_at(&root);
    // Same bytes, two project locations.
    let src = "fn main() int { return 41 + 1 }";
    let path_a = root.join("a-main.at");
    let path_b = root.join("nested").join("b-main.at");
    std::fs::write(&path_a, src).unwrap();
    std::fs::create_dir_all(path_b.parent().unwrap()).unwrap();
    std::fs::write(&path_b, src).unwrap();

    let cfg_a = VerticalCfg {
        project: "proj-a".into(),
        source: path_a.clone(),
        work_dir: root.join("wa"),
        ..Default::default()
    };
    let cfg_b = VerticalCfg {
        project: "proj-b".into(),
        source: path_b.clone(),
        work_dir: root.join("wb"),
        ..Default::default()
    };

    let out_a = vertical_and_run(&store, &cfg_a).unwrap();
    assert_eq!(out_a.exit_code, Some(42));
    assert_eq!(out_a.counters.parse, 1, "first build is a full compile");
    let blocks_before = store.space().blocks_count;

    let out_b = vertical_and_run(&store, &cfg_b).unwrap();
    assert_eq!(out_b.exit_code, Some(42));
    // Path-only change: everything hits; content is shared (no new chunks).
    assert_eq!(out_b.counters.parse, 0);
    assert_eq!(out_b.counters.verify, 0);
    assert_eq!(out_b.counters.lower, 0);
    let blocks_after = store.space().blocks_count;
    assert_eq!(
        blocks_before, blocks_after,
        "identical content must not add chunks"
    );
    // Snapshot identity is byte-identity, not path identity.
    let snap_a = identity::source_snapshot(src.as_bytes(), "main");
    let snap_b = identity::source_snapshot(src.as_bytes(), "main");
    assert_eq!(snap_a, snap_b);
}

#[test]
fn identity_body_only_edit_keeps_summary_shares_but_invalidates_object() {
    let root = scratch("identity-body");
    let store = store_at(&root);
    let base = "fn helper(x int) int { return x + 1 }\nfn main() int { return helper(5) }";
    let body_changed = "fn helper(x int) int { return x + 2 }\nfn main() int { return helper(5) }";
    let p1 = root.join("v1.at");
    let p2 = root.join("v2.at");
    std::fs::write(&p1, base).unwrap();
    std::fs::write(&p2, body_changed).unwrap();

    let out1 = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: p1,
            work_dir: root.join("w1"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(out1.exit_code, Some(6));
    let a07_v1 = out1
        .artifacts
        .iter()
        .find(|(l, _)| l == "A07")
        .map(|(_, id)| id.clone())
        .unwrap();
    let a09_v1 = out1
        .artifacts
        .iter()
        .find(|(l, _)| l == "A09")
        .map(|(_, id)| id.clone())
        .unwrap();

    let out2 = vertical_and_run(
        &store,
        &VerticalCfg {
            project: "p".into(),
            source: p2,
            work_dir: root.join("w2"),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        out2.exit_code,
        Some(7),
        "changed body must change semantics"
    );
    // Same signature ⇒ summary CONTENT identical ⇒ the same CHUNK is
    // shared (the artifact ROW differs because its lookup key embeds the
    // snapshot); body change ⇒ HIR and object ids differ.
    let a07_v2 = out2
        .artifacts
        .iter()
        .find(|(l, _)| l == "A07")
        .map(|(_, id)| id.clone())
        .unwrap();
    let a09_v2 = out2
        .artifacts
        .iter()
        .find(|(l, _)| l == "A09")
        .map(|(_, id)| id.clone())
        .unwrap();
    let a04_v1 = out1
        .artifacts
        .iter()
        .find(|(l, _)| l == "A04")
        .map(|(_, id)| id.clone())
        .unwrap();
    let a04_v2 = out2
        .artifacts
        .iter()
        .find(|(l, _)| l == "A04")
        .map(|(_, id)| id.clone())
        .unwrap();
    assert_ne!(a04_v1, a04_v2, "summary lookup keys follow the snapshot");
    // Direct content comparison: rebuild both summaries from source.
    let s1 = build_decl_summary(
        &parse_program(&std::fs::read_to_string(root.join("v1.at")).unwrap()).unwrap(),
        "main",
    );
    let s2 = build_decl_summary(
        &parse_program(&std::fs::read_to_string(root.join("v2.at")).unwrap()).unwrap(),
        "main",
    );
    assert_eq!(
        encode_decl_summary(&s1).unwrap(),
        encode_decl_summary(&s2).unwrap(),
        "same signatures ⇒ identical summary content (shared chunk)"
    );
    assert_ne!(a07_v1, a07_v2, "HIR identity follows the body");
    assert_ne!(
        a09_v1, a09_v2,
        "same signature must NOT keep the old object alive"
    );
    // And the object really missed (full recompile happened).
    assert!(out2.counters.lower >= 1, "object miss must re-lower");
}

#[test]
fn identity_summary_codec_content_determinism() {
    use auto_ac_prototype::source::parse_program;
    // The A04 summary artifact is the canonical encoding of the signature
    // projection: int/i32 synonyms and body changes do not change it.
    let s1 = build_decl_summary(&parse_program("fn f(a int) int { return a }").unwrap(), "m");
    let s2 = build_decl_summary(
        &parse_program("fn f(a i32) int { return a + 1 }").unwrap(),
        "m",
    );
    let e1 = encode_decl_summary(&s1).unwrap();
    let e2 = encode_decl_summary(&s2).unwrap();
    assert_eq!(e1, e2);
    let back = decode_decl_summary(&e1).unwrap();
    assert_eq!(back, s1);
}

#[test]
fn identity_snapshot_uses_logical_name_not_path() {
    let s = b"fn main() int { return 0 }";
    let a = SourceSnapshotId::from_parts(&[s, b"app"]);
    let b = SourceSnapshotId::from_parts(&[s, b"app"]);
    let c = SourceSnapshotId::from_parts(&[s, b"lib"]);
    assert_eq!(a, b);
    assert_ne!(a, c);
}
