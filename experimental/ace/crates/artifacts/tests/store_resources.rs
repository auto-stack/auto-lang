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
