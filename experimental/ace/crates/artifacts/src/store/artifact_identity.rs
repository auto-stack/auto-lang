use super::PutRequest;
use crate::identity::{DomainHasher, RefKind};

fn edge_bytes(edge: &(String, RefKind)) -> impl Iterator<Item = u8> + '_ {
    edge.0
        .bytes()
        .chain(std::iter::once(b':'))
        .chain(edge.1.as_str().bytes())
}

pub(super) fn artifact_id(req: &PutRequest, chunk: &str) -> String {
    let mut h = DomainHasher::new(&["art/v1"]);
    let key_hash = req.key.hash();
    let kind = format!("{:?}", req.kind);
    for part in [
        req.layer.as_bytes(),
        key_hash.as_bytes(),
        chunk.as_bytes(),
        kind.as_bytes(),
    ] {
        h.part(&[part]);
    }
    let mut edges: Vec<_> = req.edges.iter().collect();
    // Sort the exact concatenated spelling, not (id, kind) tuple order.
    // Equal spellings are identical hash parts, so their relative order
    // cannot matter; unstable sorting needs no extra sort allocation.
    edges.sort_unstable_by(|a, b| edge_bytes(a).cmp(edge_bytes(b)));
    for (id, kind) in edges {
        h.part(&[id.as_bytes(), b":", kind.as_str().as_bytes()]);
    }
    h.finish_prefixed("art")
}

#[cfg(test)]
mod tests {
    use crate::identity::RefKind;
    use crate::{
        codec::ChunkKind,
        identity::QueryKey,
        schema,
        store::{ManagedStore, PutRequest},
    };
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        cell::Cell,
    };

    struct Allocator;
    thread_local! {
        static THRESHOLD: Cell<usize> = const { Cell::new(usize::MAX) };
        static COUNT: Cell<usize> = const { Cell::new(0) };
    }
    fn observe(bytes: usize) {
        let _ = THRESHOLD.try_with(|threshold| {
            if bytes >= threshold.get() {
                COUNT.with(|n| n.set(n.get() + 1));
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
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                THRESHOLD.with(|n| n.set(usize::MAX));
            }
        }
        COUNT.with(|n| n.set(0));
        THRESHOLD.with(|n| n.set(threshold));
        let reset = Reset;
        let value = f();
        drop(reset);
        (value, COUNT.with(Cell::get))
    }
    fn original(req: &PutRequest, chunk: &str) -> String {
        let mut parts = vec![
            req.layer.as_bytes().to_vec(),
            req.key.hash().into_bytes(),
            chunk.as_bytes().to_vec(),
            format!("{:?}", req.kind).into_bytes(),
        ];
        let mut edges: Vec<String> = req
            .edges
            .iter()
            .map(|(id, kind)| format!("{id}:{}", kind.as_str()))
            .collect();
        edges.sort();
        parts.extend(edges.into_iter().map(String::into_bytes));
        // Independent old wire loop: do not call the shared hash helper.
        let mut h = blake3::Hasher::new();
        h.update(crate::HASH_ALGO.as_bytes());
        h.update(&[0]);
        h.update(b"art/v1");
        h.update(&[0]);
        for part in parts {
            h.update(&(part.len() as u64).to_le_bytes());
            h.update(&part);
        }
        format!("art{}", h.finalize().to_hex())
    }

    #[test]
    fn artifact_identity_streams_borrowed_edges_with_original_concatenated_sort_order() {
        let mut req = PutRequest {
            layer: "A04",
            key: QueryKey::new(
                schema::layer("A04").unwrap(),
                vec![("source".into(), "π".repeat(1 << 20))],
            ),
            kind: ChunkKind::DeclSummary,
            payload: vec![],
            project: None,
            edges: (0u64..8192)
                .rev()
                .map(|n| {
                    (
                        crate::identity::ArtifactId::from_parts(&[&n.to_le_bytes()]).0,
                        if n % 2 == 0 {
                            RefKind::RequiredData
                        } else {
                            RefKind::Provenance
                        },
                    )
                })
                .collect(),
        };
        let expected = original(&req, "chk938");
        let (actual, large) = measured(64 << 10, || ManagedStore::artifact_id(&req, "chk938"));
        assert_eq!(actual, expected);
        assert_eq!(
            large, 1,
            "artifact identity owns more than its borrowed edge index"
        );
        let (actual, allocations) = measured(64, || ManagedStore::artifact_id(&req, "chk938"));
        assert_eq!(actual, expected);
        assert_eq!(
            allocations, 3,
            "artifact identity copied edge text or part arrays"
        );
        // Prefix/colon/Unicode and duplicate cases distinguish concatenated
        // string ordering from sorting the logical fields as a tuple.
        req.edges = vec![
            ("edge".into(), RefKind::RequiredData),
            ("edge!".into(), RefKind::Provenance),
            ("edge:required".into(), RefKind::Recomputable),
            ("π\0".into(), RefKind::InvalidationFingerprint),
            ("edge".into(), RefKind::RequiredData),
            ("".into(), RefKind::Provenance),
        ];
        let expected = original(&req, "chk938");
        assert_eq!(ManagedStore::artifact_id(&req, "chk938"), expected);
        req.edges.reverse();
        assert_eq!(ManagedStore::artifact_id(&req, "chk938"), expected);
        req.edges.clear();
        let expected = original(&req, "chk938");
        let (actual, allocations) = measured(64, || ManagedStore::artifact_id(&req, "chk938"));
        assert_eq!(actual, expected);
        assert_eq!(
            allocations, 2,
            "empty edges allocated an index or copied identity parts"
        );
    }
}
