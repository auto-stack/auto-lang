//! Prepared rootless bytes remain live through the original OS-backed lease.
use ace_artifacts::{
    codec::ChunkKind,
    identity::QueryKey,
    schema,
    store::{run_gc, GcConfig, ManagedStore, PutRequest},
};

#[test]
fn leased_put_and_owned_root_retirement_preserve_readers_and_other_projects() {
    let root = std::env::temp_dir().join(format!("leased-publication-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let put = |name: &str, byte| {
        store
            .put_artifact_leased(
                PutRequest {
                    layer: "A04",
                    key: QueryKey::new(
                        schema::layer("A04").unwrap(),
                        vec![("case".into(), name.into())],
                    ),
                    kind: ChunkKind::DeclSummary,
                    payload: vec![byte; 4096],
                    edges: vec![],
                    project: None,
                },
                "prepared-reader",
                1,
            )
            .unwrap()
    };
    let (old, old_reader) = put("old", 1);
    let (new, new_reader) = put("new", 2);
    store
        .register_root("owned-pointer", &old, "old-hash")
        .unwrap();
    store
        .register_root("owned-pointer", &new, "new-hash")
        .unwrap();
    store.register_root("other-user", &old, "hard").unwrap();
    assert_eq!(
        store
            .retain_root_kind("owned-pointer", Some("new-hash"))
            .unwrap(),
        1
    );
    assert_eq!(
        store.root_artifacts("other-user", "hard").unwrap(),
        vec![old.clone()]
    );
    let gc = || run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    assert!(gc().ran);
    assert!(store.get_artifact_by_id(&old).unwrap().is_some());
    assert!(store.get_artifact_by_id(&new).unwrap().is_some());
    assert_eq!(store.remove_root("other-user", "hard", &old).unwrap(), 1);
    assert!(gc().ran);
    assert!(
        store.get_artifact_by_id(&old).unwrap().is_some(),
        "retired root still has a live reader"
    );
    assert!(store.lease_live(&old_reader.lease_id).unwrap());
    drop(old_reader);
    let reclaimed = gc();
    assert!(reclaimed.ran && reclaimed.deleted_chunks > 0 && reclaimed.freed_bytes > 0);
    assert!(store.get_artifact_by_id(&old).unwrap().is_none());
    drop(new_reader);
    assert!(gc().ran);
    assert!(
        store.get_artifact_by_id(&new).unwrap().is_some(),
        "accepted root survives its last reader"
    );
    assert_eq!(store.retain_root_kind("owned-pointer", None).unwrap(), 1);
    assert!(gc().deleted_chunks > 0);
    assert!(store.get_artifact_by_id(&new).unwrap().is_none());
}
