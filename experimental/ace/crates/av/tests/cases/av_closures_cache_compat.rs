//! PLAN-823 T-05 cache-compatibility tests: the store identity axes of
//! the closures layer (V-08: a changed axis is a CLEAN miss, never a
//! stale reuse) and the old-consumer isolation (V-09: closures wire never
//! executes under an owned/collections-scoped consumer — the profile
//! identity refuses before dispatch). Prefix `av_closures_823`
//! (contract-decision r2 §5).

use ace_av::closures_fixture::{
    closures_module_key, closures_types_fp_of, first_case_module, request_closures,
    restore_closures,
};
use ace_av_core::abc::encode_abc;
use ace_av_core::abc_verify::verify_abc_bytes;
use ace_av_core::contract::ExecOutcome;
use ace_av_core::mir_verify::verify_mir;

fn tmp_store(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("av823-cache-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch store");
    dir
}

#[test]
fn av_closures_cache_entry_axis_clean_miss() {
    let store = tmp_store("entry");
    let r = request_closures(Some(&store), "cache-probe", "first-case").expect("persisted");
    let owner = r.owner.clone();
    let snapshot = r.snapshot;
    let types_fp = closures_types_fp_of(&first_case_module());

    // The correct entry restores.
    let ok = restore_closures(&store, &owner, "main", snapshot, &types_fp);
    assert!(ok.is_ok(), "the real entry restores: {ok:?}");

    // A FOREIGN entry under the same owner/types/snapshot is a clean miss
    // — the entry axis participates in the store key.
    let miss = restore_closures(&store, &owner, "make", snapshot, &types_fp);
    assert!(miss.is_err(), "a foreign entry must miss");
    let _ = std::fs::remove_dir_all(&store);
}

#[test]
fn av_closures_cache_snapshot_axis_clean_miss() {
    let store = tmp_store("snapshot");
    let r = request_closures(Some(&store), "cache-probe", "first-case").expect("persisted");
    let owner = r.owner.clone();
    let entry = r.entry.clone();
    let types_fp = closures_types_fp_of(&first_case_module());

    // A foreign snapshot never matches the persisted artifact.
    let miss = restore_closures(&store, &owner, &entry, snapshot_of("other"), &types_fp);
    assert!(miss.is_err(), "a foreign snapshot must miss cleanly");
    let _ = std::fs::remove_dir_all(&store);
}

fn snapshot_of(tag: &str) -> u64 {
    ace_av_core::contract::fnv1a64(format!("av823-{tag}").as_bytes())
}

#[test]
fn av_closures_cache_closures_wire_isolated_from_old_consumers() {
    // V-09: an av.closures/1 artifact under an OLD consumer's admission —
    // the owned-scoped deep verification refuses the closures profile
    // (schema-profile identity) BEFORE any dispatch; likewise the
    // collections scope. The closures identity never executes as foreign
    // wire.
    let m = first_case_module();
    verify_mir(&m).expect("verifies under the closures scope");
    let flat = encode_abc(&m).expect("encode").to_flat();

    for (old_schema, label) in [
        (ace_av_core::contract::MIR_OWNED_SCHEMA, "owned consumer"),
        (
            ace_av_core::contract::MIR_COLLECTIONS_SCHEMA,
            "collections consumer",
        ),
    ] {
        let old_owner = ace_av_core::mir::OwnerScope {
            schema: old_schema.to_string(),
            module: "old-consumer".to_string(),
        };
        let err = verify_abc_bytes(&flat, &old_owner)
            .expect_err("old-scoped admission must refuse closures wire");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("schema-profile") || msg.contains("profile"),
            "{label}: {msg}"
        );
    }

    // The control: the SAME wire admits under the closures scope.
    let ok = verify_abc_bytes(&flat, &m.owner).expect("the closures scope admits its own wire");
    drop(ok);
}

#[test]
fn av_closures_cache_restore_reproduces_and_reverifies() {
    // The full V-08 new-worker seam over a FRESH store handle: the
    // persisted CODE artifact re-verifies (decode + deep admission) and
    // reproduces the same Success — the env NEVER persists (it rebuilds
    // during execution; only CODE is cached).
    let store = tmp_store("reverify");
    let r = request_closures(Some(&store), "cache-probe", "first-case").expect("persisted");
    let ExecOutcome::Success { value, .. } = r.av.clone().expect("av set") else {
        panic!("first case must succeed: {:?}", r.av);
    };
    let out = restore_closures(
        &store,
        &r.owner.clone(),
        &r.entry.clone(),
        r.snapshot,
        &r.types_fp.clone(),
    )
    .expect("restore re-verifies and executes");
    match out {
        ExecOutcome::Success { value: v, .. } => assert_eq!(v, value),
        other => panic!("restored run must match: {other:?}"),
    }
    // The artifact is exactly the one under the closures layer key.
    let store2 = ace_artifacts::store::ManagedStore::open(&store).expect("reopen");
    let key = closures_module_key(&r.owner, &r.entry, &r.types_fp, r.snapshot);
    assert!(store2
        .get_artifact(ace_av::closures_fixture::CLOSURES_ABC_LAYER.id, &key)
        .expect("lookup")
        .is_some());
    drop(store2);
    let _ = std::fs::remove_dir_all(&store);
}

// I06: the same admitted content can be published from two project labels
// into the same store, then restored and executed with the original identity.
#[test]
fn av_closures_cache_cross_project_content_identity() {
    let store = tmp_store("cross-project");
    let a = request_closures(Some(&store), "project-a", "first-case").expect("project a");
    let b = request_closures(Some(&store), "project-b", "first-case").expect("project b");
    assert_eq!(
        a.artifact, b.artifact,
        "identical content keeps artifact identity"
    );
    assert_eq!(a.owner, b.owner);
    assert_eq!(a.entry, b.entry);
    assert_eq!(a.snapshot, b.snapshot);
    assert_eq!(a.types_fp, b.types_fp);
    assert!(matches!(
        a.av,
        Some(ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(29),
            ..
        })
    ));
    assert!(matches!(
        b.av,
        Some(ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(29),
            ..
        })
    ));
    let restored = restore_closures(&store, &b.owner, &b.entry, b.snapshot, &b.types_fp)
        .expect("cross-project restore re-verifies");
    assert!(matches!(
        restored,
        ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(29),
            ..
        }
    ));
    let _ = std::fs::remove_dir_all(&store);
}
