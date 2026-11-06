//! Allocation controls for the actual source publication composition.
#![cfg(windows)]
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::{Cell, RefCell},
};
struct Allocator;
thread_local! {
    static THRESHOLD: Cell<usize> = const { Cell::new(usize::MAX) };
    static LARGE: Cell<usize> = const { Cell::new(0) };
    static TRACK_HEAP: Cell<bool> = const { Cell::new(false) };
    static LIVE: Cell<usize> = const { Cell::new(0) };
    static PEAK: Cell<usize> = const { Cell::new(0) };
    static TRACKED: RefCell<[(usize, usize); 128]> = const { RefCell::new([(0, 0); 128]) };
    static OVERFLOW: Cell<bool> = const { Cell::new(false) };
}
fn observe(size: usize) {
    let _ = THRESHOLD.try_with(|threshold| {
        if size >= threshold.get() {
            LARGE.with(|n| n.set(n.get() + 1));
        }
    });
}
fn allocated(ptr: *mut u8, size: usize, replaced: Option<*mut u8>) {
    let _ = TRACK_HEAP.try_with(|enabled| {
        if enabled.get() && !ptr.is_null() {
            TRACKED.with(|tracked| {
                let mut tracked = tracked.borrow_mut();
                let old = replaced
                    .and_then(|old| tracked.iter().position(|entry| entry.0 == old as usize));
                let slot = old.or_else(|| tracked.iter().position(|entry| entry.0 == 0));
                if let Some(slot) = slot {
                    let old_size = if old.is_some() { tracked[slot].1 } else { 0 };
                    tracked[slot] = (ptr as usize, size);
                    LIVE.with(|live| {
                        let next = live.get().saturating_add(size);
                        // Include old+new coexistence for realloc. If the old
                        // pointer predates this window, count the entire new
                        // buffer even when the allocator kept the same address.
                        live.set(next - old_size);
                        PEAK.with(|peak| peak.set(peak.get().max(next)));
                    });
                } else {
                    OVERFLOW.with(|overflow| overflow.set(true));
                }
            });
        }
    });
}
fn freed(ptr: *mut u8) {
    let _ = TRACK_HEAP.try_with(|enabled| {
        if enabled.get() {
            TRACKED.with(|tracked| {
                let mut tracked = tracked.borrow_mut();
                if let Some(entry) = tracked.iter_mut().find(|entry| entry.0 == ptr as usize) {
                    LIVE.with(|live| live.set(live.get() - entry.1));
                    *entry = (0, 0);
                }
                // An older allocation freed in this window cannot reduce
                // the measured bytes of the new admission allocations.
            });
        }
    });
}
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        observe(layout.size());
        let next = System.alloc(layout);
        allocated(next, layout.size(), None);
        next
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        observe(layout.size());
        let next = System.alloc_zeroed(layout);
        allocated(next, layout.size(), None);
        next
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        observe(size);
        let next = System.realloc(ptr, layout, size);
        if !next.is_null() {
            allocated(next, size, Some(ptr));
        }
        next
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        freed(ptr);
        System.dealloc(ptr, layout);
    }
}
fn measured_peak<T>(f: impl FnOnce() -> T) -> (T, usize) {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            TRACK_HEAP.with(|enabled| enabled.set(false));
        }
    }
    LIVE.with(|live| live.set(0));
    PEAK.with(|peak| peak.set(0));
    TRACKED.with(|tracked| *tracked.borrow_mut() = [(0, 0); 128]);
    OVERFLOW.with(|overflow| overflow.set(false));
    TRACK_HEAP.with(|enabled| enabled.set(true));
    let guard = Guard;
    let result = f();
    drop(guard);
    (
        result,
        if OVERFLOW.with(Cell::get) {
            usize::MAX
        } else {
            PEAK.with(Cell::get)
        },
    )
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
fn measured<T>(f: impl FnOnce() -> T) -> (T, usize) {
    measured_at(1 << 20, f)
}
fn measured_at<T>(threshold: usize, f: impl FnOnce() -> T) -> (T, usize) {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            THRESHOLD.with(|v| v.set(usize::MAX));
        }
    }
    LARGE.with(|v| v.set(0));
    THRESHOLD.with(|v| v.set(threshold));
    let guard = Guard;
    let result = f();
    drop(guard);
    (result, LARGE.with(Cell::get))
}
mod source_foreign {
    use super::*;
    use ace_abi::{
        source_foreign::{CurrentAuthority, Edit, Session, Stored, Target},
        source_provider as provider,
    };
    use ace_artifacts::store::ManagedStore;
    use ace_foreign_contract::manual::SOURCE_CAP;
    use ace_source_query::resources::Charge;
    use std::{path::PathBuf, sync::Arc, time::SystemTime};

    #[derive(serde::Serialize)]
    struct Pointer<'a> {
        stored: &'a Stored,
        payload: &'a str,
    }

    #[test]
    fn allocator_window_counts_new_live_buffers_after_older_frees_and_real_reallocation() {
        let older = vec![1u8; 4096];
        let ((first, second), peak) = measured_peak(|| {
            drop(older);
            let first = vec![2u8; 1024];
            let second = vec![3u8; 2048];
            std::hint::black_box((&first, &second));
            (first, second)
        });
        assert!(peak >= first.capacity() + second.capacity());
        drop((first, second));

        let mut older = vec![4u8; 1024];
        let ((grown, other), peak) = measured_peak(|| {
            older.reserve_exact(4096);
            let other = vec![5u8; 8192];
            std::hint::black_box((&older, &other));
            (older, other)
        });
        assert!(peak >= grown.capacity() + other.capacity());
    }

    #[test]
    fn provider_rebind_keeps_current_facts_without_allocating_whole_declaration_copies() {
        let caps = vec![SOURCE_CAP.into()];
        // This is the neutral frontend port: it consumes reviewed declaration
        // authority, not COFF or a tool. The actual provider machine calls and
        // old/new provider bytes are checked by the existing conformance cases.
        let digest = "ab".repeat(32);
        let environment =
            provider::admit(provider::reviewed_add_manifest(), &digest, 1, &caps).unwrap();
        let threshold = std::mem::size_of_val(&environment.source_declarations_ref()[0]);
        assert!(
            threshold > 64,
            "fixture cannot distinguish retained binding digests from declaration copies"
        );
        let (mut front, _checked) = ace_source_query::frontend::Frontend::cold(
            "fn main() int { return foreign_add(20,22) }",
            "rebind938",
            "p938",
            &caps,
            environment.clone(),
        )
        .unwrap();
        let old_page = front.page("main").unwrap().bytes.clone();
        let before = front.meter();
        let solver = front.query_counters().solver_runs;
        let next = environment
            .rebind(&provider::authority(&digest, 2, &caps).unwrap())
            .unwrap();
        let (result, copies) = measured_at(threshold, || front.rebind_provider(next));
        result.unwrap();
        assert_eq!(
            copies, 0,
            "provider-only frontend rebind copied whole declarations"
        );
        assert_eq!(front.foreign().generation(), 2);
        assert_eq!(front.meter(), before);
        assert_eq!(front.query_counters().solver_runs, solver);
        assert!(Arc::ptr_eq(&old_page, &front.page("main").unwrap().bytes));
        eprintln!("declaration rebind: threshold={threshold} whole_copies={copies}");
    }

    fn fixture(
        tag: &str,
    ) -> (
        PathBuf,
        Arc<Vec<u8>>,
        ace_foreign_contract::manual::AdmittedForeign,
        Vec<String>,
    ) {
        let stamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("{tag}-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("source_foreign938.h"), provider::HEADER).unwrap();
        let source = root.join("provider.c");
        let object = root.join("provider.obj");
        std::fs::write(&source, include_bytes!("../cside/source_foreign938.c")).unwrap();
        ace_abi::ctool::compile_c(&source, &root, &object).unwrap();
        let bytes = Arc::new(std::fs::read(&object).unwrap());
        let caps = vec![SOURCE_CAP.into()];
        let environment = provider::admit(
            provider::reviewed_add_manifest(),
            &provider::hash(&bytes),
            1,
            &caps,
        )
        .unwrap();
        (root, bytes, environment, caps)
    }

    #[test]
    fn c_cache_entry_read_reserves_shared_io_before_loading_the_original_store_frame() {
        use ace_artifacts::store::{ManagedStore, StoreError};
        use ace_source_query::resources::Resources;
        let (root, _old, _environment, _caps) = fixture("c-cache-entry-io");
        let source = root.join("provider.c");
        let header = root.join("source_foreign938.h");
        let object = root.join("provider.obj");
        let original = std::fs::read_to_string(&source).unwrap();
        std::fs::write(
            &source,
            format!("unsigned char cache_padding[2097152] = {{1}};\n{original}"),
        )
        .unwrap();
        ace_abi::ctool::compile_c(&source, &root, &object).unwrap();
        let bytes = std::fs::read(&object).unwrap();
        assert!(
            bytes.len() > 2 << 20,
            "compiler did not retain the large data fixture"
        );
        let artifacts = [object];
        let store = ManagedStore::open(&root.with_extension("cache-store")).unwrap();
        let (_, hit) = ace_abi::cache::produce_c_object(&store, &source, &root, &artifacts, || {
            Ok((bytes.clone(), vec![header]))
        })
        .unwrap();
        assert!(!hit);
        let resources = Resources::default();
        let before = resources.stats();
        let pressure = resources.reserve(Charge::io((8 << 20) - 1024)).unwrap();
        let pressured = resources.stats().current;
        let (rejected, peak) = measured_peak(|| {
            ace_abi::cache::produce_c_object_with_resources(
                &store,
                &source,
                &root,
                &artifacts,
                &resources,
                || panic!("bounded cache read invoked the producer"),
            )
        });
        assert!(
            matches!(rejected, Err(StoreError::ResourceExhausted(ref message)) if message.starts_with("C cache entry read:")),
            "wrong budget classification"
        );
        assert!(
            peak < 64 << 10,
            "cache entry frame was copied before shared I/O admission: {peak}"
        );
        assert_eq!(resources.stats().current, pressured);
        assert_eq!(resources.stats().rejections, before.rejections + 1);
        drop(pressure);
        let (repaired, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || panic!("I/O repair lost the original cached object"),
        )
        .unwrap();
        assert!(hit);
        assert_eq!(repaired, bytes);
        let after = resources.stats();
        assert_eq!(after.current.resident, before.current.resident);
        assert_eq!(after.current.io, before.current.io);
        assert_eq!(after.live_objects, 0);
    }

    #[test]
    fn c_cache_discovery_ids_refuse_before_copy_and_repair_without_calling_the_producer() {
        use ace_artifacts::store::{ManagedStore, StoreError};
        use ace_source_query::resources::Resources;
        let (root, bytes, _environment, _caps) = fixture("c-cache-discovery-index");
        let source = root.join("provider.c");
        let header = root.join("source_foreign938.h");
        let artifacts = [root.join("provider.obj")];
        // The cache is a sibling of the include search domain: its own writes
        // must not change that domain's negative dependency listing.
        let store = ManagedStore::open(&root.with_extension("cache-store")).unwrap();
        let (seed, hit) =
            ace_abi::cache::produce_c_object(&store, &source, &root, &artifacts, || {
                Ok((bytes.as_ref().clone(), vec![header]))
            })
            .unwrap();
        assert!(!hit);
        assert_eq!(seed, *bytes);
        let resources = Resources::default();
        // The separate entry-read port now retains its raw page. Populate
        // that account's ownership-record capacity before isolating ID fees.
        drop(resources.own_payload(Vec::new()).unwrap());
        let baseline = resources.stats().current;
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline.index - 64))
            .unwrap();
        let error = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || panic!("discovery refusal invoked the producer"),
        )
        .err()
        .expect("C cache copied discovery IDs without shared index admission");
        assert!(
            matches!(error, StoreError::ResourceExhausted(ref message) if message.starts_with("C cache discovery index:")),
            "{error}"
        );
        assert_eq!(resources.stats().current.index, (8 << 20) - 64);
        // A corrupt metadata row must not turn the borrowed SQLite value
        // into a giant Rust String before discovering that only 64B remain.
        let good_id: String = store
            .db()
            .query_row(
                "SELECT artifact_id FROM artifacts WHERE layer=?1",
                [ace_abi::cache::LAYER_OBJECT_V01.id],
                |row| row.get(0),
            )
            .unwrap();
        let oversized_id = "z".repeat(2 << 20);
        assert_eq!(
            store
                .db()
                .execute(
                    "UPDATE artifacts SET artifact_id=?1 WHERE artifact_id=?2",
                    [&oversized_id, &good_id]
                )
                .unwrap(),
            1
        );
        let (rejected, heap_peak) = measured_peak(|| {
            ace_abi::cache::produce_c_object_with_resources(
                &store,
                &source,
                &root,
                &artifacts,
                &resources,
                || panic!("oversized discovery metadata invoked the producer"),
            )
        });
        assert_eq!(
            store
                .db()
                .execute(
                    "UPDATE artifacts SET artifact_id=?1 WHERE artifact_id=?2",
                    [&good_id, &oversized_id]
                )
                .unwrap(),
            1
        );
        assert!(
            matches!(rejected, Err(StoreError::ResourceExhausted(ref message)) if message.starts_with("C cache discovery index:"))
        );
        assert!(
            heap_peak < 64 << 10,
            "discovery copied the 2MiB row before shared admission: {heap_peak}"
        );
        drop(pressure);
        let (repaired, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || panic!("repair lost the original cached provider object"),
        )
        .unwrap();
        assert!(hit);
        assert_eq!(repaired, *bytes);
        assert_eq!(
            resources.stats().current,
            baseline,
            "discovery index outlived its rows"
        );
    }

    #[test]
    fn owned_payload_handoff_moves_bytes_and_keeps_aliases_charged_once() {
        use ace_source_query::resources::{Limits, Resources};
        let resources = Resources::default();
        let mut original = Vec::with_capacity(2 << 20);
        original.extend_from_slice(b"provider bytes");
        let address = original.as_ptr();
        let capacity = original.capacity();
        let before = resources.stats().current;
        let (result, peak) = measured_peak(|| resources.own_payload(original));
        let bytes = result.unwrap();
        assert_eq!(bytes.as_ptr(), address, "shared handoff copied the payload");
        assert_eq!(bytes.capacity(), capacity);
        let charged = resources.stats().current;
        assert_eq!(
            resources.stats().current.resident - before.resident,
            capacity + 40
        );
        assert!(
            peak > 0 && peak <= 40 + charged.index - before.index,
            "payload handoff allocated {peak} bytes beyond its Arc header and precharged ownership index"
        );
        resources
            .adopt(&bytes, Charge::resident(capacity + 40))
            .unwrap();
        let alias = bytes.clone();
        drop(bytes);
        assert_eq!(
            resources.stats().current.resident - before.resident,
            capacity + 40
        );
        drop(alias);
        assert_eq!(resources.stats().current.resident, before.resident);
        assert_eq!(resources.stats().current.index, charged.index);
        assert_eq!(resources.stats().live_objects, 0);

        // Rejection consumes and drops the candidate; no live owner or fee
        // escapes. Releasing the pressure permits the same size again.
        let limited = Resources::limited(Limits {
            total: 16 << 10,
            ..Limits::default()
        })
        .unwrap();
        let pressure = limited
            .reserve(Charge::resident(
                (16 << 10) - limited.stats().current.total().unwrap() - 32,
            ))
            .unwrap();
        let pressured = limited.stats().current;
        assert!(limited.own_payload(vec![1; 1024]).is_err());
        assert_eq!(limited.stats().current, pressured);
        drop(pressure);
        let repaired = limited.own_payload(vec![1; 1024]).unwrap();
        assert_eq!(repaired.len(), 1024);
    }

    #[test]
    fn c_cache_source_identity_does_not_copy_the_whole_input_to_hash_it() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("c-cache-hash-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let file = root.join("large-source.c");
        let bytes = vec![b' '; 2 << 20];
        std::fs::write(&file, &bytes).unwrap();
        let expected = format!("large-source.c|{}", provider::hash(&bytes));
        let (identity, peak) = measured_peak(|| ace_abi::cache::c_source_identity(&file));
        assert_eq!(identity.unwrap(), expected);
        assert!(
            peak < 64 << 10,
            "C source hash copied its complete input: {peak}"
        );
        // Empty and final partial blocks preserve the exact existing digest,
        // and an absent source must still fail rather than hash as empty.
        for len in [0, 1, 8191, 8192, 8193] {
            std::fs::write(&file, &bytes[..len]).unwrap();
            assert_eq!(
                ace_abi::cache::c_source_identity(&file).unwrap(),
                format!("large-source.c|{}", provider::hash(&bytes[..len]))
            );
        }
        std::fs::remove_file(&file).unwrap();
        assert!(ace_abi::cache::c_source_identity(&file).is_err());
        eprintln!("C source identity heap peak={peak} for a 2MiB input");
    }

    #[test]
    fn provider_binding_reserves_shared_budget_and_keeps_page_and_metadata_aliases_charged() {
        let resources = ace_source_query::resources::Resources::default();
        let caps = vec![SOURCE_CAP.into()];
        let digest = "ab".repeat(32);
        let environment =
            provider::admit(provider::reviewed_add_manifest(), &digest, 1, &caps).unwrap();
        let authority = provider::authority(&digest, 2, &caps).unwrap();
        // Populate the ownership record capacity before measuring this port.
        let warm = resources.rebind_foreign(&environment, &authority).unwrap();
        drop(warm);
        let before = resources.stats().current;
        let pressure = resources
            .reserve(Charge::index((8 << 20) - before.index - 64))
            .unwrap();
        let error = resources
            .rebind_foreign(&environment, &authority)
            .err()
            .expect("binding production bypassed the shared pre-allocation budget");
        assert!(error.starts_with("foreign.resource.binding:"), "{error}");
        assert_eq!(environment.generation(), 1);
        assert_eq!(resources.stats().current.resident, before.resident);
        drop(pressure);

        let ((next, alias), peak) = measured_peak(|| {
            let next = resources.rebind_foreign(&environment, &authority).unwrap();
            let alias = next.clone();
            (next, alias)
        });
        let charged = resources.stats().current;
        let page_charge = next.pages().binding.capacity() + 40;
        let metadata_charge = digest.len() + 40;
        assert_eq!(charged.resident - before.resident, page_charge);
        assert_eq!(charged.index - before.index, metadata_charge);
        assert!(
            peak > 0 && peak <= page_charge + metadata_charge,
            "binding heap {peak} exceeds precharge {}",
            page_charge + metadata_charge
        );
        assert!(Arc::ptr_eq(
            &environment.pages().logical,
            &next.pages().logical
        ));
        assert!(Arc::ptr_eq(
            environment.allocation_identity(),
            next.allocation_identity()
        ));
        assert!(!Arc::ptr_eq(
            environment.binding_metadata_identity(),
            next.binding_metadata_identity()
        ));
        let page = next.pages().binding.clone();
        drop(next);
        assert_eq!(
            resources.stats().current,
            charged,
            "environment alias lost ownership"
        );
        drop(alias);
        assert_eq!(
            resources.stats().current.index,
            before.index,
            "binding metadata survived its last owner"
        );
        assert_eq!(
            resources.stats().current.resident - before.resident,
            page_charge,
            "required page alias lost its fee"
        );
        resources
            .adopt(&page, Charge::resident(page_charge))
            .unwrap();
        assert_eq!(
            resources.stats().current.resident - before.resident,
            page_charge,
            "snapshot adoption charged a reserved page twice"
        );
        drop(page);
        assert_eq!(resources.stats().current, before);
        eprintln!(
            "binding allocation window: precharged={} actual_heap_peak={peak}",
            page_charge + metadata_charge
        );
    }

    #[test]
    fn actual_provider_rebind_heap_fits_its_precharged_symbol_index_window() {
        let (_root, bytes, environment, caps) = fixture("provider-index-allocation");
        let mut session = Session::cold(
            "fn main() int { return foreign_add(20,22) }",
            "index938",
            "p938",
            &caps,
            environment.clone(),
            bytes.clone(),
            Target::C,
        )
        .unwrap();
        let resources = session.frontend().resources();
        let next = environment
            .rebind(&provider::authority(&provider::hash(&bytes), 2, &caps).unwrap())
            .unwrap();
        let artifact = session.artifact_bytes("main").unwrap().clone();
        let pressure = resources
            .reserve(Charge::index(
                (8 << 20) - resources.stats().current.index - 4096,
            ))
            .unwrap();
        let before = resources.stats();
        assert_eq!(before.peak.index, before.current.index);
        let (result, peak) = measured_peak(|| session.rebind_provider(next, bytes));
        result.unwrap();
        let charged = resources.stats().peak.index - before.current.index;
        assert!(
            peak > 0 && charged > 0,
            "did not observe real indexed admission"
        );
        assert!(
            peak <= charged,
            "actual admission heap {peak} exceeded precharge {charged}"
        );
        assert!(Arc::ptr_eq(
            &artifact,
            session.artifact_bytes("main").unwrap()
        ));
        drop(pressure);
        eprintln!("provider index window: precharged={charged} actual_heap_peak={peak}");
    }

    #[test]
    fn pointer_encoding_reserves_before_output_allocation_and_preserves_current_on_refusal() {
        let (root, bytes, environment, caps) = fixture("pointer-allocation");
        let authority = CurrentAuthority {
            foreign: provider::authority(&provider::hash(&bytes), 1, &caps).unwrap(),
            runtime_digest: ace_abi::source_foreign::certification_runtime_policy(Target::C),
            emitter: Target::C.identity(),
        };
        let mut session = Session::cold(
            "fn main() int { return foreign_add(20,22) }",
            "pointer938",
            "p938",
            &caps,
            environment,
            bytes,
            Target::C,
        )
        .unwrap();
        let resources = session.frontend().resources();
        let store = ManagedStore::open(&root.join("store")).unwrap();
        let pointer = root.join("current.json");
        let mut current = || Ok(authority.clone());
        let ready = session
            .prepare_publication(Edit::Current, &root.join("baseline"), &mut current, None)
            .unwrap();
        assert_eq!(ready.run(&mut current, None).unwrap().exit_code, Some(42));
        let stored = session
            .publish_prepared(
                ready,
                &store,
                &pointer,
                None,
                None,
                |s, encoder| encoder.json(s),
                &mut current,
                None,
            )
            .unwrap();
        let old = std::fs::read(&pointer).unwrap();
        let identity = ace_a2x_core::sha256::sha256_hex(&old);
        let old_snapshot = session.snapshot().unwrap().digest;
        let baseline_staging = store.space().staging;
        let payload = "x".repeat(2 << 20);
        let pressure = resources
            .reserve(Charge::io(resources.available_io() - (128 << 10)))
            .unwrap();
        let ready = session
            .prepare_publication(Edit::Current, &root.join("refused"), &mut current, None)
            .unwrap();
        let mut called = false;
        let (rejected, allocations) = measured(|| {
            session.publish_prepared(
                ready,
                &store,
                &pointer,
                Some(&identity),
                Some(&stored),
                |s, encoder| {
                    called = true;
                    encoder.json(&Pointer {
                        stored: s,
                        payload: &payload,
                    })
                },
                &mut current,
                None,
            )
        });
        assert!(called, "did not reach the actual pointer encoding port");
        assert!(rejected
            .err()
            .expect("pointer was accepted")
            .contains("foreign.resource"));
        assert_eq!(std::fs::read(&pointer).unwrap(), old);
        assert_eq!(session.snapshot().unwrap().digest, old_snapshot);
        eprintln!("pointer allocation refusal: large_allocations={allocations}");
        assert_eq!(allocations, 0, "refused pointer allocated its whole output");
        drop(pressure);

        let ready = session
            .prepare_publication(Edit::Current, &root.join("repaired"), &mut current, None)
            .unwrap();
        let (accepted, allocations) = measured(|| {
            session.publish_prepared(
                ready,
                &store,
                &pointer,
                Some(&identity),
                Some(&stored),
                |s, encoder| {
                    encoder.json(&Pointer {
                        stored: s,
                        payload: &payload,
                    })
                },
                &mut current,
                None,
            )
        });
        accepted.unwrap();
        eprintln!("pointer allocation admitted: large_allocations={allocations}");
        assert_eq!(
            allocations, 1,
            "pointer hashing copied the whole encoded record"
        );
        let current_bytes = std::fs::read(&pointer).unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&current_bytes).unwrap()["payload"],
            payload,
        );
        assert_ne!(current_bytes, old);
        let (observed, allocations) =
            measured(|| ace_abi::source_foreign::read_pointer_identity(&pointer));
        assert_eq!(
            observed.unwrap().unwrap(),
            ace_a2x_core::sha256::sha256_hex(&current_bytes)
        );
        assert_eq!(
            allocations, 0,
            "identity-only pointer validation copied its payload"
        );
        assert_eq!(resources.stats().current.io, 0);
        assert_eq!(store.space().staging, baseline_staging);
        let pressure = resources
            .reserve(Charge::io(resources.available_io() - 64))
            .unwrap();
        let (refused, allocations) =
            measured(|| ace_abi::source_foreign::read_pointer_bytes_reserved(&pointer, &resources));
        assert!(refused
            .err()
            .expect("unreserved pointer read was accepted")
            .contains("foreign.resource"));
        assert_eq!(allocations, 0, "refused read allocated the pointer payload");
        drop(pressure);
        let (read, allocations) =
            measured(|| ace_abi::source_foreign::read_pointer_bytes_reserved(&pointer, &resources));
        let read = read.unwrap().unwrap();
        assert_eq!(allocations, 1);
        assert_eq!(read.bytes(), current_bytes);
        assert_eq!(resources.stats().current.io, current_bytes.len());
        drop(read);
        assert_eq!(resources.stats().current.io, 0);
    }
}
