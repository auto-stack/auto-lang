//! Allocation controls for the actual source publication composition.
#![cfg(windows)]
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};
struct Allocator;
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
    static LARGE: Cell<usize> = const { Cell::new(0) };
}
fn observe(size: usize) {
    let _ = MEASURING.try_with(|enabled| {
        if enabled.get() && size >= 1 << 20 {
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
        System.dealloc(ptr, layout);
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
fn measured<T>(f: impl FnOnce() -> T) -> (T, usize) {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            MEASURING.with(|v| v.set(false));
        }
    }
    LARGE.with(|v| v.set(0));
    MEASURING.with(|v| v.set(true));
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
    fn pointer_encoding_reserves_before_output_allocation_and_preserves_current_on_refusal() {
        let stamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("pointer-allocation-{}-{stamp}", std::process::id()));
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
