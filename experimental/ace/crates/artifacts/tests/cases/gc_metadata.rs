//! Plan 823: terminal metadata cleanup must not remove recovery state.
use ace_artifacts::store::{run_gc, GcConfig, ManagedStore};

#[test]
fn gc_prunes_only_terminal_metadata_and_preserves_user_data() {
    let root = std::env::temp_dir().join(format!("plan823-gc-metadata-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let sentinel = root.join("user-sentinel");
    std::fs::write(&sentinel, b"keep").unwrap();
    let store = ManagedStore::open(&root.join("store")).unwrap();
    {
        let db = store.db();
        db.execute("INSERT INTO chunks(chunk_id,kind,size,state,created_epoch) VALUES ('chk0000000000000000000000000000000000000000000000000000000000000000',1,1,'deleted',0)", []).unwrap();
        db.execute("INSERT INTO chunks(chunk_id,kind,size,state,created_epoch) VALUES ('pending',1,1,'tombstoned',0)", []).unwrap();
        for phase in ["committed", "staging", "conflict", "aborted"] {
            db.execute(
                "INSERT INTO txns(txn_id,phase,detail,epoch) VALUES (?1,?1,'',0)",
                [phase],
            )
            .unwrap();
        }
    }
    let active = store.acquire_lease("active-control", 300, None).unwrap();
    let released = store.acquire_lease("released-control", 300, None).unwrap();
    released.release();
    assert!(run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap().ran);
    {
        let db = store.db();
        let done: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM chunks WHERE chunk_id='chk0000000000000000000000000000000000000000000000000000000000000000'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let pending: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM chunks WHERE chunk_id='pending' AND state='tombstoned'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let committed: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM txns WHERE phase='committed'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let unfinished: i64 = db
            .query_row(
                "SELECT COUNT(*) FROM txns WHERE phase IN ('staging','conflict','aborted')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!((done, pending, committed, unfinished), (0, 1, 0, 3));
        let active_rows: i64 = db
            .query_row("SELECT COUNT(*) FROM leases WHERE released=0", [], |r| {
                r.get(0)
            })
            .unwrap();
        let released_rows: i64 = db
            .query_row("SELECT COUNT(*) FROM leases WHERE released=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(
            (active_rows, released_rows),
            (1, 0),
            "GC retains a live lease and prunes released rows"
        );
    }
    assert_eq!(std::fs::read(&sentinel).unwrap(), b"keep");
    drop(active);
    drop(store);
    let _ = std::fs::remove_dir_all(&root);
}

#[cfg(windows)]
#[test]
fn gc_retains_busy_gate_and_retries_after_reader_release() {
    use ace_artifacts::store::FileGuard;
    let root = std::env::temp_dir().join(format!("plan823-gc-held-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let store = ManagedStore::open(&root.join("store")).unwrap();
    let id = format!("chk{}", "b".repeat(64));
    store.db().execute("INSERT INTO chunks(chunk_id,kind,size,state,created_epoch) VALUES (?1,1,1,'deleted',0)", [&id]).unwrap();
    let path = store.root.join("locks").join(format!("gate-{id}.lock"));
    let guard = FileGuard::try_lock(&path, false)
        .unwrap()
        .expect("held reader gate");
    run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    assert!(path.exists(), "busy lock name remains reachable");
    let count: i64 = store
        .db()
        .query_row(
            "SELECT COUNT(*) FROM chunks WHERE chunk_id=?1",
            [&id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "busy metadata remains for retry");
    drop(guard);
    run_gc(&store, GcConfig { high: 1, low: 0 }).unwrap();
    assert!(!path.exists(), "quiescent lock is reclaimed");
    let count: i64 = store
        .db()
        .query_row(
            "SELECT COUNT(*) FROM chunks WHERE chunk_id=?1",
            [&id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "terminal metadata is reclaimed after release");
    drop(store);
    let _ = std::fs::remove_dir_all(&root);
}
