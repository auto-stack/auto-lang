//! The store's staging admission must precede its framed payload allocation.
use ace_artifacts::{
    codec::{ChunkKind, CHUNK_HEADER_LEN},
    identity::QueryKey,
    schema,
    store::{ManagedStore, PutRequest, StoreError},
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

struct Allocator;
thread_local! {
    static THRESHOLD: Cell<usize> = const { Cell::new(usize::MAX) };
    static LARGE: Cell<usize> = const { Cell::new(0) };
}
fn observe(size: usize) {
    let _ = THRESHOLD.try_with(|threshold| {
        if size >= threshold.get() {
            LARGE.with(|n| n.set(n.get() + 1));
        }
    });
}
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        observe(layout.size());
        System.alloc(layout)
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        observe(layout.size());
        System.alloc_zeroed(layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        observe(size);
        System.realloc(ptr, layout, size)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

fn measured<T>(threshold: usize, f: impl FnOnce() -> T) -> (T, usize) {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            THRESHOLD.with(|v| v.set(usize::MAX));
        }
    }
    LARGE.with(|v| v.set(0));
    THRESHOLD.with(|v| v.set(threshold));
    let guard = Guard;
    let value = f();
    drop(guard);
    (value, LARGE.with(Cell::get))
}
fn request(name: &str, len: usize) -> PutRequest {
    PutRequest {
        layer: "A04",
        key: QueryKey::new(
            schema::layer("A04").unwrap(),
            vec![("case".into(), name.into())],
        ),
        kind: ChunkKind::DeclSummary,
        payload: vec![7; len],
        edges: vec![],
        project: None,
    }
}

#[test]
fn query_key_canonical_text_allocates_only_its_exact_final_buffer() {
    let key = QueryKey {
        layer: "A04",
        // canonical() has always preserved the stored component order; only
        // QueryKey::new sorts it. Include a direct unsorted value as well.
        components: vec![
            ("z=,[]\0".into(), "π".repeat(1 << 20)),
            ("a".into(), "first".into()),
            ("a".into(), "second".into()),
            ("".into(), "😀\0=[]".into()),
        ],
    };
    let mut expected = format!("{}[", key.layer);
    for (i, (name, value)) in key.components.iter().enumerate() {
        if i > 0 {
            expected.push(',');
        }
        expected.push_str(&format!("{name}={value}"));
    }
    expected.push(']');
    let (actual, allocations) = measured(64 << 10, || key.canonical());
    assert!(
        actual == expected,
        "canonical key bytes or component order changed"
    );
    assert_eq!(
        allocations, 1,
        "canonical text grew or formatted whole field copies"
    );
    assert_eq!(actual.capacity(), actual.len());
    assert_eq!(key.canonical_len(), Some(expected.len()));
    let empty = QueryKey::new(schema::layer("A04").unwrap(), vec![]);
    assert_eq!(empty.canonical(), "A04[]");
    assert_eq!(empty.canonical_len(), Some(5));
}

#[test]
fn query_key_hash_streams_borrowed_components_with_original_framing() {
    fn original(key: &QueryKey) -> String {
        // Independent pre-change wire algorithm, evaluated outside allocation
        // windows: algorithm NUL family NUL, then LE-u64 length + raw part.
        let family = format!("qk/{}", key.layer);
        let mut hash = blake3::Hasher::new();
        hash.update(ace_artifacts::HASH_ALGO.as_bytes());
        hash.update(&[0]);
        hash.update(family.as_bytes());
        hash.update(&[0]);
        for part in std::iter::once(key.layer.as_bytes()).chain(
            key.components
                .iter()
                .flat_map(|(k, v)| [k.as_bytes(), v.as_bytes()]),
        ) {
            hash.update(&(part.len() as u64).to_le_bytes());
            hash.update(part);
        }
        hash.finalize().to_hex().to_string()
    }
    let key = QueryKey::new(
        schema::layer("A07").unwrap(),
        vec![
            ("z=,[]\0".into(), "π".repeat(1 << 20)),
            ("same".into(), "first".into()),
            ("same".into(), "second".into()),
            ("".into(), "".into()),
            ("a\0b".into(), "😀,=[]".into()),
        ],
    );
    let expected = original(&key);
    let (actual, large_copies) = measured(64 << 10, || key.hash());
    assert_eq!(actual, expected, "query identity bytes changed");
    assert_eq!(
        large_copies, 0,
        "query hash copied whole fingerprint components"
    );
    let (actual, allocations) = measured(64, || key.hash());
    assert_eq!(actual, expected);
    assert_eq!(
        allocations, 1,
        "hash owns more than its final 64-byte hex result"
    );
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::path::PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
        .join(format!("key-hash-stream-{}-{stamp}", std::process::id()));
    let store = ManagedStore::open(&root).unwrap();
    let payload = vec![42; 67];
    let artifact = store
        .put_artifact(PutRequest {
            layer: key.layer,
            key: key.clone(),
            kind: ChunkKind::DeclSummary,
            payload: payload.clone(),
            edges: vec![],
            project: None,
        })
        .unwrap();
    let persisted: String = store
        .db()
        .query_row(
            "SELECT key_hash FROM artifacts WHERE artifact_id=?1",
            [&artifact],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        persisted, expected,
        "published key no longer has its original address"
    );
    let loaded = store.get_artifact(key.layer, &key).unwrap().unwrap();
    assert_eq!(loaded.payload, payload);
    assert_eq!(loaded.artifact_id, artifact);
    drop(store);
    let reopened = ManagedStore::open(&root).unwrap();
    let restored = reopened.get_artifact(key.layer, &key).unwrap().unwrap();
    assert_eq!(restored.payload, payload);
    assert_eq!(restored.artifact_id, artifact);
    for layer in ["A04", "A07"] {
        let a = QueryKey::new(
            schema::layer(layer).unwrap(),
            vec![("a".into(), "bc".into())],
        );
        let b = QueryKey::new(
            schema::layer(layer).unwrap(),
            vec![("ab".into(), "c".into())],
        );
        let empty = QueryKey::new(schema::layer(layer).unwrap(), vec![]);
        for candidate in [&a, &b, &empty] {
            assert_eq!(candidate.hash(), original(candidate));
        }
        assert_ne!(a.hash(), b.hash(), "part-length framing was lost");
    }
}

#[test]
fn rejected_staging_never_allocates_the_framed_payload_and_can_retry_after_drain() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::path::PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests").join(
        format!("store-frame-reserve-{}-{stamp}", std::process::id()),
    );
    let store = ManagedStore::open(&root).unwrap();
    let oversized = request("oversized", ManagedStore::STAGING_BUDGET as usize / 2 + 1);
    let threshold = oversized.payload.len();
    let (result, allocations) = measured(threshold, || store.put_artifact(oversized));
    assert!(matches!(result, Err(StoreError::ResourceExhausted(_))));
    assert_eq!(
        allocations, 0,
        "oversized request allocated its rejected frame"
    );
    assert_eq!(store.space().blocks, 0);

    let residue = root.join("staging").join("owned-pressure");
    std::fs::create_dir_all(&residue).unwrap();
    let pressure = std::fs::File::create(residue.join("payload")).unwrap();
    pressure
        .set_len(ManagedStore::STAGING_BUDGET - 128)
        .unwrap();
    let accepted = request("legal-after-drain", 128 << 10);
    let key = accepted.key.clone();
    let threshold = accepted.payload.len();
    let (result, allocations) = measured(threshold, || store.put_artifact(accepted));
    assert!(matches!(result, Err(StoreError::ResourceExhausted(_))));
    assert_eq!(
        allocations, 0,
        "in-flight staging refusal allocated its rejected frame"
    );
    assert_eq!(store.space().blocks, 0);
    pressure.set_len(0).unwrap();
    drop(pressure);
    std::fs::remove_file(residue.join("payload")).unwrap();
    std::fs::remove_dir(residue).unwrap();

    let accepted = request("legal-after-drain", 128 << 10);
    let (result, allocations) = measured(threshold + CHUNK_HEADER_LEN, || {
        store.put_artifact(accepted)
    });
    result.unwrap();
    assert!(
        allocations > 0,
        "allocator control did not observe actual frame production"
    );
    let loaded = store.get_artifact("A04", &key).unwrap().unwrap();
    assert_eq!(loaded.payload, vec![7; threshold]);
    assert_eq!(store.space().staging, 0);
}
