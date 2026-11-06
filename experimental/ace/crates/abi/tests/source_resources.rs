//! Allocation controls for the actual source publication composition.
#![cfg(windows)]
use ace_abi::semref;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::{Cell, RefCell},
};
struct Allocator;
struct LayoutPressure {
    size: usize,
    remaining: usize,
    resources: ace_source_query::resources::Resources,
    active: bool,
    triggered: bool,
    failed: bool,
    allocation: Option<ace_source_query::resources::Reservation>,
}
// Compile the private production implementation under this target's allocator;
// no public testing hook or alternate path algorithm is introduced.
#[path = "../src/cache_paths.rs"]
mod cache_paths;
#[path = "../src/ctool_capture.rs"]
mod ctool_capture;
#[path = "../src/ctool_dependencies.rs"]
mod ctool_dependencies;
#[path = "../src/ctool_text.rs"]
mod ctool_text;
thread_local! {
    static LAYOUT_PRESSURE: RefCell<Option<LayoutPressure>> = const { RefCell::new(None) };
    static THRESHOLD: Cell<usize> = const { Cell::new(usize::MAX) };
    static LARGE: Cell<usize> = const { Cell::new(0) };
    static TRACK_HEAP: Cell<bool> = const { Cell::new(false) };
    static LIVE: Cell<usize> = const { Cell::new(0) };
    static PEAK: Cell<usize> = const { Cell::new(0) };
    static TRACKED: RefCell<[(usize, usize); 128]> = const { RefCell::new([(0, 0); 128]) };
    static OVERFLOW: Cell<bool> = const { Cell::new(false) };
    static RESIDENT_PROBE: RefCell<Option<(usize, ace_source_query::resources::Resources, ace_source_query::resources::Charge)>> = const { RefCell::new(None) };
    static RESIDENT_HITS: Cell<usize> = const { Cell::new(0) };
    static RESIDENT_ADMITTED: Cell<bool> = const { Cell::new(true) };
}
fn apply_layout_pressure(size: usize) {
    let _ = LAYOUT_PRESSURE.try_with(|slot| {
        // Release this borrow before touching Resources: allocator callbacks
        // can nest. Mark first so that none can apply the pressure twice.
        let action = slot.try_borrow_mut().ok().and_then(|mut state| {
            let state = state.as_mut()?;
            if !state.active || state.triggered || state.size != size {
                return None;
            }
            state.triggered = true;
            Some((state.resources.clone(), state.remaining))
        });
        if let Some((resources, remaining)) = action {
            let current = resources.stats().current;
            let available = (8usize << 20)
                .saturating_sub(current.index)
                .min((48usize << 20).saturating_sub(current.total().unwrap_or(usize::MAX)));
            let allocation = available.checked_sub(remaining).and_then(|bytes| {
                resources
                    .reserve(ace_source_query::resources::Charge::index(bytes))
                    .ok()
            });
            if let Ok(mut state) = slot.try_borrow_mut() {
                if let Some(state) = state.as_mut() {
                    state.failed = allocation.is_none();
                    state.allocation = allocation;
                }
            }
        }
    });
}
fn with_layout_pressure<T>(
    size: usize,
    remaining: usize,
    resources: &ace_source_query::resources::Resources,
    f: impl FnOnce() -> T,
) -> (T, bool) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            LAYOUT_PRESSURE.with(|state| {
                state.borrow_mut().take();
            });
        }
    }
    LAYOUT_PRESSURE.with(|state| {
        assert!(state.borrow().is_none());
        *state.borrow_mut() = Some(LayoutPressure {
            size,
            remaining,
            resources: resources.clone(),
            active: false,
            triggered: false,
            failed: false,
            allocation: None,
        });
    });
    let reset = Reset;
    let value = f();
    let applied = LAYOUT_PRESSURE.with(|state| {
        let state = state.borrow();
        let state = state.as_ref().unwrap();
        state.triggered && !state.failed
    });
    drop(reset);
    (value, applied)
}
fn observe(size: usize) {
    let _ = THRESHOLD.try_with(|threshold| {
        if size >= threshold.get() {
            LARGE.with(|n| n.set(n.get() + 1));
        }
    });
}
fn allocated(ptr: *mut u8, size: usize, replaced: Option<*mut u8>) {
    if !ptr.is_null() {
        apply_layout_pressure(size);
        let _ = RESIDENT_PROBE.try_with(|probe| {
            let probe = probe.borrow();
            if let Some((expected_size, resources, minimum)) = &*probe {
                if size == *expected_size {
                    RESIDENT_HITS.with(|hits| hits.set(hits.get() + 1));
                    let current = resources.stats().current;
                    if current.resident < minimum.resident
                        || current.io < minimum.io
                        || current.index < minimum.index
                        || current.staging < minimum.staging
                        || current.mir < minimum.mir
                    {
                        RESIDENT_ADMITTED.with(|admitted| admitted.set(false));
                    }
                }
            }
        });
    }
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
fn with_resident_probe<T>(
    size: usize,
    resources: &ace_source_query::resources::Resources,
    minimum: usize,
    f: impl FnOnce() -> T,
) -> (T, usize, bool) {
    with_charge_probe(
        size,
        resources,
        ace_source_query::resources::Charge::resident(minimum),
        f,
    )
}
fn with_charge_probe<T>(
    size: usize,
    resources: &ace_source_query::resources::Resources,
    minimum: ace_source_query::resources::Charge,
    f: impl FnOnce() -> T,
) -> (T, usize, bool) {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            RESIDENT_PROBE.with(|probe| {
                probe.borrow_mut().take();
            });
        }
    }
    RESIDENT_HITS.with(|hits| hits.set(0));
    RESIDENT_ADMITTED.with(|admitted| admitted.set(true));
    RESIDENT_PROBE.with(|probe| {
        assert!(probe.borrow().is_none());
        *probe.borrow_mut() = Some((size, resources.clone(), minimum));
    });
    let reset = Reset;
    let result = f();
    drop(reset);
    (
        result,
        RESIDENT_HITS.with(Cell::get),
        RESIDENT_ADMITTED.with(Cell::get),
    )
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
    fn lexical_path_output_is_admitted_at_its_real_allocation_before_copying() {
        use ace_source_query::resources::Resources;
        let resources = Resources::default();
        let baseline = resources.stats().current.index;
        let suffix = "path".repeat(65536);
        let target = format!("D:/a/{suffix}");
        let expected_relative = format!("../{suffix}");
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline - 64))
            .unwrap();
        let (rejected, copies) = measured_at(128 << 10, || {
            cache_paths::rel_spec("D:/a/b", &target, Some(&resources))
        });
        assert!(rejected.is_err());
        assert_eq!(copies, 0, "relative path copied before refusal");
        drop(pressure);
        let (relative, hits, admitted) = with_charge_probe(
            expected_relative.len(),
            &resources,
            Charge::index(baseline + expected_relative.len()),
            || cache_paths::rel_spec("D:/a/b", &target, Some(&resources)),
        );
        let relative = relative.unwrap().unwrap();
        assert_eq!(relative.value, expected_relative);
        assert_eq!(hits, 1);
        assert!(admitted, "relative path copied before reservation");
        let expected_joined = format!("D:\\a\\{suffix}");
        let segments = 3 + 2; // D:/a/b and ../<suffix>
        let (joined, hits, admitted) = with_charge_probe(
            expected_joined.len(),
            &resources,
            Charge::index(
                baseline
                    + relative.value.capacity()
                    + segments * std::mem::size_of::<&str>()
                    + expected_joined.len(),
            ),
            || cache_paths::join_lex("D:/a/b", &relative.value, Some(&resources)),
        );
        let joined = joined.unwrap();
        assert_eq!(joined.value, expected_joined);
        assert_eq!(hits, 1);
        assert!(
            admitted,
            "joined path copied before array/output coexistence reservation"
        );
        assert_eq!(
            resources.stats().current.index,
            baseline + relative.value.capacity() + joined.value.capacity()
        );
        drop((relative, joined));
        assert_eq!(resources.stats().current.index, baseline);
    }

    #[test]
    fn candidate_directory_growth_and_path_copies_are_precharged_and_released_after_dedup() {
        use ace_source_query::resources::Resources;
        let resources = Resources::default();
        let baseline = resources.stats().current.index;
        let mut paths = cache_paths::CPathList::new(Some(&resources)).unwrap();
        for value in ["a", "b", "c", "d"] {
            paths.push(value).unwrap();
        }
        let text = "directory".repeat(32768);
        let before = resources.stats().current.index;
        let pressure = resources
            .reserve(Charge::index((8 << 20) - before - 64))
            .unwrap();
        let (rejected, copies) = measured_at(128 << 10, || paths.push(&text));
        assert!(rejected.is_err());
        assert_eq!(copies, 0);
        assert_eq!(paths.values, ["a", "b", "c", "d"]);
        drop(pressure);
        let array = 8 * std::mem::size_of::<String>();
        let old_array = 4 * std::mem::size_of::<String>();
        let (result, hits, admitted) = with_charge_probe(
            array,
            &resources,
            Charge::index(baseline + 4 + text.len() + array + old_array),
            || paths.push(&text),
        );
        result.unwrap();
        assert_eq!(hits, 1);
        assert!(admitted, "directory array grew before old/new reservation");
        assert!(paths.contains(&text));
        let (result, hits, admitted) = with_charge_probe(
            text.len(),
            &resources,
            Charge::index(baseline + array + 4 + text.len() * 2),
            || paths.push(&text),
        );
        result.unwrap();
        assert_eq!(hits, 1);
        assert!(admitted, "directory String copied before reservation");
        paths.sort_dedup().unwrap();
        assert_eq!(paths.values.len(), 5);
        assert_eq!(
            resources.stats().current.index,
            baseline + array + 4 + text.len()
        );
        drop(paths);
        assert_eq!(resources.stats().current.index, baseline);
        // Exercise the shared module's borrowed parent and split utilities too.
        assert_eq!(cache_paths::parent_of("D:\\a\\file"), "D:\\a");
        assert_eq!(cache_paths::split_norm("D:/a").count(), 2);
    }

    #[test]
    fn source_identity_reserves_digest_and_final_text_before_their_actual_allocations() {
        use ace_artifacts::store::StoreError;
        use ace_source_query::resources::Resources;
        let (root, _, _, _) = fixture("source-identity-budget");
        let source = root.join("provider.c");
        let expected = ace_abi::cache::c_source_identity(&source).unwrap();
        let resources = Resources::default();
        let baseline = resources.stats().current.index;
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline - 64))
            .unwrap();
        let (rejected, large) = measured_at(1 << 20, || {
            cache_paths::source_identity(&source, Some(&resources))
        });
        assert!(matches!(rejected, Err(StoreError::ResourceExhausted(_))));
        assert_eq!(large, 0);
        drop(pressure);
        assert!(source.as_os_str().len() < 247);
        let native = (source.as_os_str().len() + 1) * std::mem::size_of::<u16>();
        for (layout, minimum) in [
            (64, baseline + expected.len() + 64),
            (expected.len(), baseline + expected.len() + 64),
            (native, baseline + expected.len() + 64 + native),
        ] {
            let (identity, hits, admitted) =
                with_charge_probe(layout, &resources, Charge::index(minimum), || {
                    cache_paths::source_identity(&source, Some(&resources))
                });
            let identity = identity.unwrap();
            assert_eq!(identity.value, expected);
            assert_eq!(hits, 1, "did not observe identity buffer {layout}");
            assert!(
                admitted,
                "identity allocation preceded digest/output reservation"
            );
            assert_eq!(
                resources.stats().current.index,
                baseline + identity.value.capacity()
            );
            drop(identity);
            assert_eq!(resources.stats().current.index, baseline);
        }
        let missing = root.join("missing.c");
        assert!(
            matches!(cache_paths::source_identity(&missing, Some(&resources)), Err(StoreError::Internal(ref message)) if message.starts_with(&missing.display().to_string()))
        );
        assert_eq!(resources.stats().current.index, baseline);
    }

    #[test]
    fn canonical_and_artifact_paths_precharge_real_long_junction_targets_and_release_temporaries() {
        use ace_artifacts::store::StoreError;
        use ace_source_query::resources::Resources;
        use std::os::windows::{ffi::OsStrExt, process::CommandExt};
        let (root, _, _, _) = fixture("canonical-budget");
        let mut target = root.join("long-target");
        for i in 0..8 {
            target.push(format!("segment-{i}-{}", "x".repeat(96)));
        }
        target.push("unicode-π-😀");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("keep"), b"target must survive link cleanup").unwrap();
        let link = root.join("short-link");
        let created = std::process::Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(link.display().to_string().replace('/', "\\"))
            .arg(target.display().to_string().replace('/', "\\"))
            .creation_flags(0x08000000)
            .output()
            .unwrap();
        assert!(
            created.status.success(),
            "junction fixture failed: {}",
            String::from_utf8_lossy(&created.stderr)
        );
        struct Unlink(PathBuf);
        impl Drop for Unlink {
            fn drop(&mut self) {
                std::fs::remove_dir(&self.0).expect("remove only fixture junction");
            }
        }
        let unlink = Unlink(link.clone());
        let reference = std::fs::canonicalize(&link).unwrap();
        let expected = reference.display().to_string();
        let wide = (reference.as_os_str().encode_wide().count() + 1) * std::mem::size_of::<u16>();
        assert!(wide > 1024);
        assert!(link.as_os_str().len() < 247);
        let native = (link.as_os_str().len() + 1) * std::mem::size_of::<u16>();
        let resources = Resources::default();
        let baseline = resources.stats().current.index;
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline - 1024))
            .unwrap();
        let before = resources.stats().current;
        let (rejected, copies) = measured_at(wide, || {
            cache_paths::canonical_text(&link, Some(&resources), "fixture canonical")
        });
        assert!(matches!(rejected, Err(StoreError::ResourceExhausted(_))));
        assert_eq!(copies, 0, "long resolved path allocated before admission");
        assert_eq!(resources.stats().current, before);
        let rejected_artifact =
            cache_paths::artifact_path(&link.join("output.obj"), Some(&resources));
        assert!(
            matches!(rejected_artifact, Err(StoreError::ResourceExhausted(_))),
            "artifact normalization treated budget refusal as an I/O fallback"
        );
        assert_eq!(resources.stats().current, before);
        drop(pressure);
        for (layout, minimum) in [
            (native, baseline + native),
            (wide, baseline + wide),
            (expected.len(), baseline + wide + expected.len()),
        ] {
            let (result, hits, admitted) =
                with_charge_probe(layout, &resources, Charge::index(minimum), || {
                    cache_paths::canonical_text(&link, Some(&resources), "fixture canonical")
                });
            let result = result.unwrap();
            assert_eq!(result.value, expected);
            assert_eq!(hits, 1, "did not observe canonical buffer {layout}");
            assert!(admitted, "canonical buffer preceded its reservation");
            assert_eq!(
                resources.stats().current.index,
                baseline + result.value.capacity()
            );
            drop(result);
            assert_eq!(resources.stats().current.index, baseline);
        }
        // Fresh accounts keep historical pressure peaks from hiding an
        // under-reservation inside std's long/relative native conversion.
        let long_resources = Resources::default();
        let long_baseline = long_resources.stats().current.index;
        let (long_result, long_peak) = measured_peak(|| {
            cache_paths::canonical_text(&target, Some(&long_resources), "long canonical")
        });
        let long_result = long_result.unwrap();
        assert_eq!(long_result.value, expected);
        assert!(long_peak > 0 && long_peak <= long_resources.stats().peak.index - long_baseline);
        drop(long_result);
        assert_eq!(long_resources.stats().current.index, long_baseline);
        struct RestoreCwd(PathBuf);
        impl Drop for RestoreCwd {
            fn drop(&mut self) {
                std::env::set_current_dir(&self.0).expect("restore fixture CWD");
            }
        }
        let cwd = RestoreCwd(std::env::current_dir().unwrap());
        std::env::set_current_dir(&link).unwrap();
        let cwd_spelling = std::env::current_dir().unwrap();
        let relative_native =
            (cwd_spelling.as_os_str().encode_wide().count() + 4 + 1) * std::mem::size_of::<u16>();
        let relative_resources = Resources::default();
        let relative_baseline = relative_resources.stats().current.index;
        let ((relative_result, relative_peak), hits, admitted) = with_charge_probe(
            relative_native,
            &relative_resources,
            Charge::index(relative_baseline + relative_native),
            || {
                measured_peak(|| {
                    cache_paths::canonical_text(
                        std::path::Path::new("."),
                        Some(&relative_resources),
                        "relative canonical",
                    )
                })
            },
        );
        let relative_result = relative_result.unwrap();
        assert_eq!(relative_result.value, expected);
        assert_eq!(hits, 1, "did not observe relative native path expansion");
        assert!(
            admitted,
            "relative path expanded before native scratch admission"
        );
        assert!(
            relative_peak > 0
                && relative_peak <= relative_resources.stats().peak.index - relative_baseline
        );
        drop(relative_result);
        assert_eq!(relative_resources.stats().current.index, relative_baseline);
        drop(cwd);
        eprintln!(
            "canonical buffers: utf16={wide} utf8={} long_heap={long_peak} long_reserved={} relative_heap={relative_peak} relative_reserved={}",
            expected.len(),
            long_resources.stats().peak.index - long_baseline,
            relative_resources.stats().peak.index - relative_baseline,
        );
        for path in [
            &link,
            &target,
            &target.join("keep"),
            &root.join("../canonical-missing-parent/output.obj"),
        ] {
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(std::path::Path::new("."));
            let dir = std::fs::canonicalize(parent)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| parent.display().to_string());
            let original = format!(
                "{}\\{}",
                dir.strip_prefix("\\\\?\\")
                    .unwrap_or(&dir)
                    .replace('/', "\\"),
                path.file_name().unwrap_or_default().to_string_lossy()
            );
            let actual = cache_paths::artifact_path(path, Some(&resources)).unwrap();
            assert_eq!(actual.value, original);
        }
        assert_eq!(resources.stats().current.index, baseline);
        let missing = root.join("not-present");
        assert!(
            matches!(cache_paths::canonical_text(&missing, Some(&resources), "fixture canonical"), Err(StoreError::Internal(ref message)) if message.starts_with("fixture canonical: "))
        );
        assert_eq!(resources.stats().current.index, baseline);
        let raw = cache_paths::display_path(&missing, Some(&resources)).unwrap();
        assert_eq!(raw.value, missing.display().to_string());
        drop(raw);
        drop(unlink);
        assert!(!link.exists());
        assert_eq!(
            std::fs::read(target.join("keep")).unwrap(),
            b"target must survive link cleanup"
        );
        assert_eq!(resources.stats().current.index, baseline);
    }

    #[test]
    fn toolchain_identity_and_sql_needle_precharge_streamed_text_and_preserve_original_escaping() {
        use ace_source_query::resources::Resources;
        use std::os::windows::ffi::OsStringExt;
        let path = PathBuf::from(std::ffi::OsString::from_wide(&[
            b'D' as u16,
            b':' as u16,
            b'\\' as u16,
            0xd800,
        ]));
        let version = "π\\_%".repeat(32768);
        let digest = "a".repeat(64);
        let flags = ["-c", "flag_%\\", "π"];
        let expected = format!(
            "cl:{}|ver:{version}|hash:{digest}|flags:{}",
            path.display(),
            flags.join(" ")
        );
        let resources = Resources::default();
        let baseline = resources.stats().current.index;
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline - 64))
            .unwrap();
        let (rejected, large) = measured_at(128 << 10, || {
            cache_paths::toolchain_text(&path, &version, &digest, &flags, Some(&resources))
        });
        assert!(rejected.is_err());
        assert_eq!(large, 0, "toolchain copied before admission");
        drop(pressure);
        let (toolchain, hits, admitted) = with_charge_probe(
            expected.len(),
            &resources,
            Charge::index(baseline + expected.len()),
            || cache_paths::toolchain_text(&path, &version, &digest, &flags, Some(&resources)),
        );
        let toolchain = toolchain.unwrap();
        assert_eq!(toolchain.value, expected);
        assert_eq!(hits, 1);
        assert!(admitted, "toolchain text allocated before reservation");
        let source = "source\\_%π";
        let target = "target_%";
        let escape = |s: &str| {
            s.replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        };
        let expected = format!(
            "%source={},target={},toolchain={}%",
            escape(source),
            escape(target),
            escape(&toolchain.value)
        );
        let input_fee = resources.stats().current.index;
        let pressure = resources
            .reserve(Charge::index((8 << 20) - input_fee - 64))
            .unwrap();
        let (rejected, large) = measured_at(128 << 10, || {
            cache_paths::discovery_needle(source, target, &toolchain.value, Some(&resources))
        });
        assert!(rejected.is_err());
        assert_eq!(large, 0, "query copied before admission");
        drop(pressure);
        let (needle, hits, admitted) = with_charge_probe(
            expected.len(),
            &resources,
            Charge::index(input_fee + expected.len()),
            || cache_paths::discovery_needle(source, target, &toolchain.value, Some(&resources)),
        );
        let needle = needle.unwrap();
        assert_eq!(needle.value, expected);
        assert_eq!(hits, 1);
        assert!(admitted, "query text allocated before reservation");
        drop(needle);
        assert_eq!(resources.stats().current.index, input_fee);
        drop(toolchain);
        assert_eq!(resources.stats().current.index, baseline);
    }

    #[test]
    fn cold_toolchain_discovery_does_not_copy_the_compiler_image_to_hash_it() {
        // Nextest gives this case a fresh process, so the real OnceLock
        // resolver (including the compiler's independent banner query) runs.
        let (tool, copies) = measured_at(64 << 10, ace_abi::ctool::find_cl);
        let tool = tool.unwrap();
        let size = std::fs::metadata(&tool.cl_exe).unwrap().len();
        assert!(
            size > 64 << 10,
            "compiler fixture no longer distinguishes whole-file reads"
        );
        assert_eq!(copies, 0, "cold discovery copied the compiler image");
        let original = std::fs::read(&tool.cl_exe).unwrap();
        assert_eq!(tool.cl_hash, blake3::hash(&original).to_hex().to_string());
        eprintln!("cold compiler identity: file_bytes={size} large_copies={copies}");
    }

    #[test]
    fn tool_discovery_caps_both_streams_before_accepting_a_compiler_identity() {
        const CHILD: &str = "S938_TOOL_DISCOVERY_CAP_CHILD";
        const STREAM: &str = "S938_TOOL_DISCOVERY_CAP_STREAM";
        if std::env::var_os(CHILD).is_some() {
            let stream = std::env::var(STREAM).unwrap();
            let error = ace_abi::ctool::find_cl()
                .expect_err("tool discovery accepted a banner plus oversized raw output");
            assert!(
                error.0.contains("OutputCapExceeded") && error.0.contains(&stream),
                "{error}"
            );
            return;
        }
        // A fresh child isolates the real OnceLock and its test-only tool
        // override from every other case, including bare libtest runs.
        let stamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
            .join(format!("tool-discovery-cap-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let fake = root.join("fake-cl.cmd");
        std::fs::write(&fake, concat!(
            "@echo off\r\n",
            "powershell.exe -NoLogo -NoProfile -NonInteractive -Command \"",
            "$w=if($env:S938_TOOL_DISCOVERY_CAP_STREAM -eq 'stderr'){[Console]::Error}else{[Console]::Out}; ",
            "$w.WriteLine('Microsoft C Compiler Version 19.44.35222'); ",
            "$w.Write(('x'*2097152))\"\r\n"
        )).unwrap();
        for stream in ["stdout", "stderr"] {
            let mut child = std::process::Command::new(std::env::current_exe().unwrap());
            child.args(["--exact", "source_foreign::tool_discovery_caps_both_streams_before_accepting_a_compiler_identity", "--nocapture"])
                .env(CHILD, "1").env(STREAM, stream).env("AUTOABI_MSVC_CL", &fake);
            let run = ace_core::link::run_deadlined_captured(
                &mut child,
                std::time::Duration::from_secs(30),
                ace_core::link::CaptureCaps {
                    stdout_max: 1 << 20,
                    stderr_max: 1 << 20,
                },
            )
            .unwrap();
            assert_eq!(
                run.exit_code,
                Some(0),
                "{stream} discovery cap child failed:\n{}\n{}",
                String::from_utf8_lossy(&run.stdout),
                String::from_utf8_lossy(&run.stderr)
            );
        }
    }

    #[test]
    fn native_lower_emission_reserves_instruction_and_span_capacity_before_allocation() {
        use ace_av_core::{
            contract::SpanInfo,
            mir::{MirBlock, MirInstr, MirSlot},
        };
        use ace_lowering::hir_adapter::{
            lower_function_unit, lower_function_unit_with_emission_reservation,
        };
        use ace_source_query::resources::Resources;
        let source = format!(
            "fn main() int {{ var x = 0; {} return x }}",
            "x = x + 1; ".repeat(40)
        );
        let compiled =
            ace_core::source::compile_scalar_source(&source, "emission.at", &[]).unwrap();
        let bundle = compiled.checked.bundle();
        let original = lower_function_unit(bundle, "d_main", &[], 17).unwrap();
        let original_wire = ace_av_core::abc::encode_abc(&original).unwrap().to_flat();
        let instruction = std::mem::size_of::<MirInstr>();
        let span = std::mem::size_of::<(u32, SpanInfo)>();
        // At instruction 33 this fixture has one named base slot, two
        // one-element slot indices, 32 temp slots, four block cells and
        // one x64 traversal task (48B). These are simultaneous owners.
        let scaffold =
            33 * std::mem::size_of::<MirSlot>() + 1 + 8 + 4 * std::mem::size_of::<MirBlock>() + 48;
        let allowed = scaffold + 96 * instruction + 32 * span - 1;
        eprintln!("emission layout: instruction={instruction} span={span} next_array={} allowance={allowed}", 64 * instruction);
        eprintln!(
            "other initial layouts: block={} type={}",
            std::mem::size_of::<ace_av_core::mir::MirBlock>(),
            std::mem::size_of::<ace_av_core::mir::MirType>()
        );
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let mut allocation = resources.reserve(Charge::mir(0)).unwrap();
        let pressure = resources
            .reserve(Charge::mir((16 << 20) - allowed))
            .unwrap();
        let pressured = resources.stats().current;
        let (denied, forbidden, _) =
            with_charge_probe(64 * instruction, &resources, Charge::default(), || {
                lower_function_unit_with_emission_reservation(
                    bundle,
                    "d_main",
                    &[],
                    17,
                    &mut |bytes| {
                        allocation
                            .resize(Charge::mir(bytes))
                            .map_err(|e| e.to_string())
                    },
                )
            });
        assert!(matches!(
            denied,
            Err(ace_lowering::AdapterError::Resource { .. })
        ));
        assert_eq!(
            forbidden, 0,
            "instruction array allocated before its reservation"
        );
        assert_eq!(
            resources.stats().current.mir,
            pressured.mir + scaffold + 32 * instruction + 32 * span
        );
        // The caller owns the ticket on an error path as well: partial arrays
        // have dropped inside lowering, then its ticket may be released.
        drop(allocation);
        assert_eq!(resources.stats().current, pressured);
        drop(pressure);
        let mut allocation = resources.reserve(Charge::mir(0)).unwrap();
        let (mir, hits, admitted) = with_charge_probe(
            64 * instruction,
            &resources,
            Charge::mir(scaffold + 96 * instruction + 32 * span),
            || {
                lower_function_unit_with_emission_reservation(
                    bundle,
                    "d_main",
                    &[],
                    17,
                    &mut |bytes| {
                        allocation
                            .resize(Charge::mir(bytes))
                            .map_err(|e| e.to_string())
                    },
                )
            },
        );
        let mir = mir.unwrap();
        assert_eq!(hits, 1, "did not observe the real instruction array growth");
        assert!(
            admitted,
            "old/new instruction arrays or live spans were not reserved together"
        );
        let retained: usize = mir
            .funcs
            .iter()
            .map(|f| {
                f.blocks
                    .iter()
                    .map(|b| b.instrs.capacity() * instruction)
                    .sum::<usize>()
                    + f.spans.capacity() * span
                    + f.blocks.capacity() * std::mem::size_of::<MirBlock>()
                    + f.slots.capacity() * std::mem::size_of::<MirSlot>()
                    + f.slots
                        .iter()
                        .filter_map(|s| s.name.as_ref())
                        .map(|s| s.capacity())
                        .sum::<usize>()
            })
            .sum();
        assert_eq!(resources.stats().current.mir, retained);
        assert_eq!(
            ace_av_core::abc::encode_abc(&mir).unwrap().to_flat(),
            original_wire
        );
        ace_av_core::mir_verify::validate_mir(&mir).unwrap();
        drop(mir);
        drop(allocation);
        assert_eq!(resources.stats().current, baseline);
        eprintln!("native emission: instruction={instruction} span={span} retained={retained}");

        let compiled = ace_core::source::compile_scalar_source(
            "fn main() int { return 1 + 2 }",
            "span-emission.at",
            &[],
        )
        .unwrap();
        let scaffold =
            4 * std::mem::size_of::<MirBlock>() + 48 + 4 * std::mem::size_of::<MirSlot>();
        let allowed = scaffold + 4 * instruction + 4 * span - 1;
        let pressure = resources
            .reserve(Charge::mir((16 << 20) - allowed))
            .unwrap();
        let mut allocation = resources.reserve(Charge::mir(0)).unwrap();
        let (denied, span_arrays, _) =
            with_charge_probe(4 * span, &resources, Charge::default(), || {
                lower_function_unit_with_emission_reservation(
                    compiled.checked.bundle(),
                    "d_main",
                    &[],
                    17,
                    &mut |bytes| {
                        allocation
                            .resize(Charge::mir(bytes))
                            .map_err(|e| e.to_string())
                    },
                )
            });
        assert!(matches!(
            denied,
            Err(ace_lowering::AdapterError::Resource { .. })
        ));
        assert_eq!(span_arrays, 0, "span array allocated before reservation");
        assert_eq!(
            resources.stats().current.mir,
            (16 << 20) - allowed + scaffold + 4 * instruction
        );
        drop(allocation);
        drop(pressure);
        assert_eq!(resources.stats().current, baseline);
    }

    #[test]
    fn native_lower_container_admission_covers_blocks_stack_slots_and_final_copy() {
        use ace_av_core::mir::{MirBlock, MirInstr, MirSlot};
        use ace_lowering::hir_adapter::{
            lower_function_unit, lower_function_unit_with_emission_reservation,
        };
        use ace_source_query::resources::Resources;

        fn reject_before_array(
            bundle: &ace_core::hir::Bundle,
            available: usize,
            array: usize,
            retained_on_error: usize,
        ) {
            let resources = Resources::default();
            let baseline = resources.stats().current;
            let pressure = resources
                .reserve(Charge::mir((16 << 20) - available))
                .unwrap();
            let pressured = resources.stats().current;
            let mut allocation = resources.reserve(Charge::mir(0)).unwrap();
            let (result, forbidden, _) =
                with_charge_probe(array, &resources, Charge::default(), || {
                    lower_function_unit_with_emission_reservation(
                        bundle,
                        "d_main",
                        &[],
                        17,
                        &mut |bytes| {
                            allocation
                                .resize(Charge::mir(bytes))
                                .map_err(|e| e.to_string())
                        },
                    )
                });
            assert!(
                matches!(result, Err(ace_lowering::AdapterError::Resource { .. })),
                "array {array}: expected a typed resource refusal"
            );
            assert_eq!(forbidden, 0, "array {array} allocated before admission");
            assert_eq!(
                resources.stats().current.mir,
                pressured.mir + retained_on_error,
                "array {array}: wrong lifetime for the caller's error ticket"
            );
            drop(allocation);
            assert_eq!(resources.stats().current, pressured);
            drop(pressure);
            assert_eq!(resources.stats().current, baseline);
        }

        let block_array = 4 * std::mem::size_of::<MirBlock>();
        // The Windows-x64 traversal task owns an HIR block id, a statement
        // index, a tagged start and a tagged optional exit: 48 bytes.
        let task = 48;
        let basic = ace_core::source::compile_scalar_source(
            "fn main() int { return 1 }",
            "container.at",
            &[],
        )
        .unwrap();
        reject_before_array(basic.checked.bundle(), block_array - 1, block_array, 0);
        reject_before_array(
            basic.checked.bundle(),
            block_array + task - 1,
            task,
            block_array,
        );

        // A real if/else queues continuation + else + then. The second
        // pending task grows the original one-element stack to four cells.
        let branched = ace_core::source::compile_scalar_source(
            "fn main() int { if 1 < 2 { return 1 } else { return 2 } }",
            "task-growth.at",
            &[],
        )
        .unwrap();
        let before_stack_growth = block_array
            + task
            + 4 * std::mem::size_of::<MirSlot>()
            + 4 * std::mem::size_of::<MirInstr>()
            + 4 * std::mem::size_of::<(u32, ace_av_core::contract::SpanInfo)>()
            + 4 * std::mem::size_of::<Option<u32>>();
        reject_before_array(
            branched.checked.bundle(),
            before_stack_growth + 4 * task - 1,
            4 * task,
            before_stack_growth,
        );
        let resources = Resources::default();
        let mut allocation = resources.reserve(Charge::mir(0)).unwrap();
        let (mir, hits, admitted) = with_charge_probe(
            4 * task,
            &resources,
            Charge::mir(before_stack_growth + 4 * task),
            || {
                lower_function_unit_with_emission_reservation(
                    branched.checked.bundle(),
                    "d_main",
                    &[],
                    17,
                    &mut |bytes| {
                        allocation
                            .resize(Charge::mir(bytes))
                            .map_err(|e| e.to_string())
                    },
                )
            },
        );
        let mir = mir.unwrap();
        assert_eq!(hits, 1);
        assert!(
            admitted,
            "old and new traversal stacks must coexist in the reservation"
        );
        assert_eq!(
            ace_av_core::abc::encode_abc(&mir).unwrap().to_flat(),
            ace_av_core::abc::encode_abc(
                &lower_function_unit(branched.checked.bundle(), "d_main", &[], 17).unwrap()
            )
            .unwrap()
            .to_flat()
        );
        drop(mir);
        drop(allocation);
        assert_eq!(resources.stats().current.mir, 0);

        let source = format!(
            "fn main() int {{ {} return x249 }}",
            (0..250)
                .map(|n| format!("let x{n:03} = 1; "))
                .collect::<String>()
        );
        let many = ace_core::source::compile_scalar_source(&source, "base-slots.at", &[]).unwrap();
        let base_array = 250 * std::mem::size_of::<MirSlot>();
        reject_before_array(many.checked.bundle(), base_array - 1, base_array, 0);

        let source = format!(
            "fn main() int {{ var x = 0; {} return x }}",
            "x = x + 1; ".repeat(40)
        );
        let compiled =
            ace_core::source::compile_scalar_source(&source, "final-slots.at", &[]).unwrap();
        let original = lower_function_unit(compiled.checked.bundle(), "d_main", &[], 17).unwrap();
        let f = &original.funcs[0];
        assert_eq!(f.slots.len(), 82);
        let names: usize = f
            .slots
            .iter()
            .filter_map(|s| s.name.as_ref())
            .map(|s| s.capacity())
            .sum();
        let emitted = f
            .blocks
            .iter()
            .map(|b| b.instrs.capacity() * std::mem::size_of::<MirInstr>())
            .sum::<usize>()
            + f.spans.capacity() * std::mem::size_of::<(u32, ace_av_core::contract::SpanInfo)>()
            + block_array;
        let final_array = f.slots.len() * std::mem::size_of::<MirSlot>();
        // Original base, the 128-capacity temporary slot array, both base
        // indices and the stack still coexist with the new final array.
        let before_final = emitted + names + 129 * std::mem::size_of::<MirSlot>() + 8 + task;
        reject_before_array(
            compiled.checked.bundle(),
            before_final + final_array - 1,
            final_array,
            before_final,
        );

        let resources = Resources::default();
        let mut allocation = resources.reserve(Charge::mir(0)).unwrap();
        let (accepted, hits, admitted) = with_charge_probe(
            final_array,
            &resources,
            Charge::mir(before_final + final_array),
            || {
                lower_function_unit_with_emission_reservation(
                    compiled.checked.bundle(),
                    "d_main",
                    &[],
                    17,
                    &mut |bytes| {
                        allocation
                            .resize(Charge::mir(bytes))
                            .map_err(|e| e.to_string())
                    },
                )
            },
        );
        let accepted = accepted.unwrap();
        assert_eq!(hits, 1);
        assert!(
            admitted,
            "final copy did not reserve all coexisting slot arrays and traversal indices"
        );
        assert_eq!(
            resources.stats().current.mir,
            emitted + names + final_array,
            "traversal and intermediate slot buffers must be released after their owners drop"
        );
        assert_eq!(
            ace_av_core::abc::encode_abc(&accepted).unwrap().to_flat(),
            ace_av_core::abc::encode_abc(&original).unwrap().to_flat()
        );
        ace_av_core::mir_verify::validate_mir(&accepted).unwrap();
        drop(accepted);
        drop(allocation);
        assert_eq!(resources.stats().current.mir, 0);
        eprintln!("container admission: block={block_array}, task={task}, base={base_array}, final={final_array}, final temporary={}", before_final + final_array);
    }

    #[test]
    fn native_source_emission_quota_rejects_before_codegen_and_repairs_current_target() {
        let (root, provider_bytes, environment, caps) = fixture("native-emission-quota");
        let source = "fn main() int { return foreign_add(20,22) }";
        let mut session = Session::cold(
            source,
            "emission938",
            "p938",
            &caps,
            environment,
            provider_bytes.clone(),
            Target::Native,
        )
        .unwrap();
        let resources = session.frontend().resources();
        let old = session.artifact_bytes("main").unwrap().clone();
        let ledger = session.frontend().work();
        let lower_before = ledger.totals("main", "mir-lower");
        let native_before = ledger.totals("main", "native-emit");
        let baseline = resources.stats().current;
        let pressure = resources
            .reserve(Charge::mir((16 << 20) - baseline.mir - 1))
            .unwrap();
        let changed = "fn main() int { return foreign_add(21,22) }";
        let denied = session.prepare("main", changed);
        assert!(
            matches!(denied, Err(ref message) if message.starts_with("foreign.resource:")),
            "emission quota was not a source resource rejection"
        );
        let lower = ledger.totals("main", "mir-lower");
        assert_eq!(lower.attempted, lower_before.attempted + 1);
        assert_eq!(lower.produced, lower_before.produced);
        assert_eq!(
            ledger.totals("main", "native-emit").attempted,
            native_before.attempted,
            "quota failed only after native codegen ran"
        );
        assert_eq!(session.frontend().source(), source);
        assert!(Arc::ptr_eq(&old, session.artifact_bytes("main").unwrap()));
        drop(pressure);
        assert_eq!(resources.stats().current.mir, baseline.mir);
        let candidate = session.prepare("main", changed).unwrap();
        assert_eq!(resources.stats().current.mir, baseline.mir);
        let authority = CurrentAuthority {
            foreign: provider::authority(&provider::hash(&provider_bytes), 1, &caps).unwrap(),
            runtime_digest: ace_abi::source_foreign::certification_runtime_policy(Target::Native),
            emitter: Target::Native.identity(),
        };
        let mut current = || Ok(authority.clone());
        let ready = session
            .prepare_publication(
                Edit::Body(candidate),
                &root.join("repaired"),
                &mut current,
                None,
            )
            .unwrap();
        assert_eq!(ready.run(&mut current, None).unwrap().exit_code, Some(43));
        let store = ManagedStore::open(&root.join("store")).unwrap();
        session
            .publish_prepared(
                ready,
                &store,
                &root.join("current.json"),
                None,
                None,
                |stored, encoder| encoder.json(stored),
                &mut current,
                None,
            )
            .unwrap();
        assert_eq!(session.frontend().source(), changed);
        assert!(!Arc::ptr_eq(&old, session.artifact_bytes("main").unwrap()));
    }

    #[test]
    fn c_cache_canonical_text_admission_precedes_store_publication_and_repairs() {
        use ace_artifacts::store::{ManagedStore, StoreError};
        use ace_source_query::resources::Resources;
        let (root, bytes, _, _) = fixture("c-canonical-admission");
        let source = root.join("provider.c");
        let artifacts = [root.join("provider.obj")];
        let header = root.join("source_foreign938.h");
        let store = ManagedStore::open(&root.with_extension("store")).unwrap();
        let source_identity = ace_abi::cache::c_source_identity(&source).unwrap();
        // Generation is always the original 64-byte digest; its content does
        // not affect the canonical output length. Compute the expected layout
        // via the public legacy key, outside the observed production window.
        let standin = ace_abi::cache::c_object_key(&source_identity, &"00".repeat(32));
        let canonical = standin.canonical_len().unwrap();
        let resources = Resources::default();
        let produced_bytes = bytes.clone();
        let deps = vec![header.clone()];
        let ((denied, text_allocations, _), applied) = with_layout_pressure(
            5 * std::mem::size_of::<(String, String)>(),
            canonical - 1,
            &resources,
            || {
                with_charge_probe(canonical, &resources, Charge::default(), || {
                    ace_abi::cache::produce_c_object_with_resources(
                        &store,
                        &source,
                        &root,
                        &artifacts,
                        &resources,
                        || {
                            // Apply pressure at the next fixed key-array
                            // allocation, after its entire key fee was paid.
                            LAYOUT_PRESSURE
                                .with(|state| state.borrow_mut().as_mut().unwrap().active = true);
                            Ok((produced_bytes, deps))
                        },
                    )
                })
            },
        );
        assert!(applied, "did not reach the admitted key allocation");
        let published = store
            .db()
            .query_row("SELECT COUNT(*) FROM artifacts", [], |r| r.get::<_, i64>(0))
            .unwrap();
        eprintln!("canonical boundary: accepted={} text_allocations={text_allocations} published={published} expected_text_bytes={canonical}", denied.is_ok());
        assert!(
            matches!(denied, Err(StoreError::ResourceExhausted(ref message)) if message.contains("canonical key")),
            "canonical output was not rejected at its admission boundary"
        );
        assert_eq!(
            text_allocations, 0,
            "store allocated canonical text before admission"
        );
        assert_eq!(published, 0, "resource refusal published an artifact");
        assert_eq!(store.space().blocks, 0);
        assert_eq!(resources.stats().current.io, 0);
        let mut produced = 0;
        let ((repaired, text_allocations, admitted), applied) = with_layout_pressure(
            5 * std::mem::size_of::<(String, String)>(),
            canonical,
            &resources,
            || {
                with_charge_probe(canonical, &resources, Charge::index(8 << 20), || {
                    ace_abi::cache::produce_c_object_with_resources(
                        &store,
                        &source,
                        &root,
                        &artifacts,
                        &resources,
                        || {
                            produced += 1;
                            LAYOUT_PRESSURE
                                .with(|state| state.borrow_mut().as_mut().unwrap().active = true);
                            Ok((bytes.clone(), vec![header]))
                        },
                    )
                })
            },
        );
        let (repaired, hit) = repaired.unwrap();
        assert!(applied);
        assert_eq!(
            text_allocations, 1,
            "did not observe the store's canonical text allocation"
        );
        assert!(
            admitted,
            "store text allocated after its reservation was released"
        );
        assert!(!hit);
        assert_eq!(produced, 1);
        assert_eq!(repaired, bytes);
        let (cached, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || panic!("repaired canonical publication was not reusable"),
        )
        .unwrap();
        assert!(hit);
        assert_eq!(cached, bytes);
    }

    #[test]
    fn c_generation_key_reserves_its_array_and_all_owned_fields_until_consumed() {
        use ace_source_query::resources::Resources;
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let source = "provider.c|938";
        let generation = "b7".repeat(32);
        let target = ace_abi::layout::TARGET;
        let toolchain = format!("cl:{}", "x".repeat(2 << 20));
        let inputs = resources
            .reserve(Charge::index(
                source.len() + generation.len() + target.len() + toolchain.len(),
            ))
            .unwrap();
        let input_baseline = resources.stats().current;
        let original = ace_artifacts::identity::QueryKey::new(
            &ace_abi::cache::LAYER_OBJECT_V01,
            vec![
                ("kind".into(), "cl-compile".into()),
                ("source".into(), source.into()),
                ("target".into(), target.into()),
                ("toolchain".into(), toolchain.clone()),
                ("generation".into(), generation.clone()),
            ],
        );
        let array = 5 * std::mem::size_of::<(String, String)>();
        let owned = array
            + original
                .components
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>();
        let pressure = resources
            .reserve(Charge::index((8 << 20) - input_baseline.index - owned + 1))
            .unwrap();
        let pressured = resources.stats().current;
        let (denied, copies) = measured_at(toolchain.len(), || {
            cache_paths::c_object_key(
                &ace_abi::cache::LAYER_OBJECT_V01,
                source,
                &generation,
                target,
                &toolchain,
                Some(&resources),
            )
        });
        assert!(matches!(
            denied,
            Err(ace_artifacts::store::StoreError::ResourceExhausted(_))
        ));
        assert_eq!(copies, 0, "key copied a value before budget refusal");
        assert_eq!(resources.stats().current, pressured);
        drop(pressure);
        for layout in [array, toolchain.len()] {
            let (key, hits, admitted) = with_charge_probe(
                layout,
                &resources,
                Charge::index(input_baseline.index + owned),
                || {
                    cache_paths::c_object_key(
                        &ace_abi::cache::LAYER_OBJECT_V01,
                        source,
                        &generation,
                        target,
                        &toolchain,
                        Some(&resources),
                    )
                },
            );
            let key = key.unwrap();
            assert_eq!(hits, 1);
            assert!(admitted, "key storage allocated before reservation");
            assert!(
                key.value == original,
                "key fields or canonical order changed"
            );
            assert_eq!(key.value.hash(), original.hash());
            assert!(
                key.value.canonical() == original.canonical(),
                "canonical key text changed"
            );
            let retained = key.value.components.capacity()
                * std::mem::size_of::<(String, String)>()
                + key
                    .value
                    .components
                    .iter()
                    .map(|(k, v)| k.capacity() + v.capacity())
                    .sum::<usize>();
            assert_eq!(retained, owned);
            assert_eq!(
                resources.stats().current.index,
                input_baseline.index + retained
            );
            // The shared hash consumer must see the retained key's full fee.
            let (_, hashes, retained) = with_charge_probe(
                64,
                &resources,
                Charge::index(input_baseline.index + owned),
                || key.value.hash(),
            );
            assert_eq!(hashes, 1);
            assert!(retained);
            drop(key);
            assert_eq!(resources.stats().current, input_baseline);
        }
        drop(inputs);
        assert_eq!(resources.stats().current, baseline);
    }

    #[test]
    fn compiler_output_text_borrows_valid_bytes_and_precharges_exact_lossy_storage() {
        use ace_source_query::resources::Resources;
        use std::borrow::Cow;
        let resources = Resources::default();
        let empty = resources.stats().current;
        let raw = resources.reserve(Charge::io(2 << 20)).unwrap();
        let raw_baseline = resources.stats().current;
        // Independent oracle is the original standard-library display. These
        // include incomplete, overlong, surrogate and disjoint invalid spans.
        let cases: &[&[u8]] = &[
            b"",
            b"a\0\r\nb",
            b"\xff\xfea\xc3(b",
            b"\xf0\x90",
            b"\xed\xa0\x80\xc0\xaf",
            b"ok\xe2\x82\xac\xf4\x90\x80\x80\0end",
            "π😀\r\n".as_bytes(),
        ];
        for input in cases {
            let expected = String::from_utf8_lossy(input);
            let shown = ctool_text::display(input, Some(&resources)).unwrap();
            assert_eq!(shown.value, expected);
            match &shown.value {
                Cow::Borrowed(s) => {
                    assert_eq!(s.as_ptr(), input.as_ptr());
                    assert_eq!(resources.stats().current, raw_baseline);
                }
                Cow::Owned(s) => {
                    assert_eq!(s.capacity(), expected.len());
                    assert_eq!(resources.stats().current.io, raw_baseline.io + s.capacity());
                }
            }
            drop(shown);
            assert_eq!(resources.stats().current, raw_baseline);
        }
        let invalid = vec![0xff; 1 << 20];
        let expected = String::from_utf8_lossy(&invalid).into_owned();
        let text_bytes = expected.len();
        assert_eq!(text_bytes, 3 << 20);
        let pressure = resources
            .reserve(Charge::io((8 << 20) - raw_baseline.io - text_bytes + 1))
            .unwrap();
        let pressured = resources.stats().current;
        let (denied, copies) = measured_at(text_bytes, || {
            ctool_text::display(&invalid, Some(&resources))
        });
        assert!(matches!(denied, Err(ref message) if message.starts_with("foreign.resource:")));
        assert_eq!(copies, 0, "lossy output allocated before quota refusal");
        assert_eq!(resources.stats().current, pressured);
        drop(pressure);
        let (shown, hits, admitted) = with_charge_probe(
            text_bytes,
            &resources,
            Charge::io(raw_baseline.io + text_bytes),
            || ctool_text::display(&invalid, Some(&resources)),
        );
        let shown = shown.unwrap();
        assert_eq!(hits, 1, "lossy output grew or was copied");
        assert!(admitted, "lossy buffer allocation precedes reservation");
        assert_eq!(shown.value, expected);
        assert_eq!(shown.value.len(), text_bytes);
        assert_eq!(resources.stats().current.io, raw_baseline.io + text_bytes);
        drop(shown);
        assert_eq!(resources.stats().current, raw_baseline);
        let valid = "π".repeat(1 << 19);
        let pressure = resources
            .reserve(Charge::io((8 << 20) - raw_baseline.io))
            .unwrap();
        let full = resources.stats().current;
        let (borrowed, copies) = measured_at(valid.len(), || {
            ctool_text::display(valid.as_bytes(), Some(&resources))
        });
        let borrowed = borrowed.unwrap();
        assert!(matches!(borrowed.value, Cow::Borrowed(_)));
        assert_eq!(borrowed.value.as_ptr(), valid.as_ptr());
        assert_eq!(copies, 0, "valid output made a whole-text copy");
        assert_eq!(resources.stats().current, full);
        drop(borrowed);
        drop(pressure);
        drop(raw);
        assert_eq!(resources.stats().current, empty);
    }

    #[test]
    fn compiler_raw_capture_reserves_both_buffers_before_allocating_or_spawning() {
        use ace_core::link::CaptureCaps;
        use ace_source_query::resources::Resources;
        use std::{process::Command, time::Duration};
        let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests").join(format!(
            "raw-capture-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let marker = root.join("child-started.txt");
        let caps = CaptureCaps {
            stdout_max: 8191,
            stderr_max: 4093,
        };
        let bytes = caps.stdout_max + caps.stderr_max;
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let pressure = resources
            .reserve(Charge::io((8 << 20) - bytes + 1))
            .unwrap();
        let pressured = resources.stats().current;
        let mut cmd = Command::new("cmd");
        cmd.args([
            "/d",
            "/c",
            &format!("echo child > \"{}\"", marker.display()),
        ]);
        let (denied, allocations) = measured_at(caps.stderr_max, || {
            ctool_capture::run(&mut cmd, Duration::from_secs(10), caps, Some(&resources))
        });
        assert!(
            matches!(denied, Err(ctool_capture::CaptureError::Resource(ref message)) if message.contains("foreign.resource"))
        );
        assert_eq!(allocations, 0, "raw buffers allocated before quota refusal");
        assert!(!marker.exists(), "budget denial started the child");
        assert_eq!(resources.stats().current, pressured);
        drop(pressure);
        for layout in [caps.stdout_max, caps.stderr_max] {
            let mut cmd = Command::new("cmd");
            cmd.args(["/d", "/c", "(echo raw-output)&(echo raw-error 1>&2)"]);
            let (captured, hits, admitted) =
                with_charge_probe(layout, &resources, Charge::io(bytes), || {
                    ctool_capture::run(&mut cmd, Duration::from_secs(10), caps, Some(&resources))
                });
            let captured = captured.unwrap();
            assert_eq!(hits, 1);
            assert!(admitted, "raw capture allocation precedes reservation");
            assert_eq!(captured.value.exit_code, Some(0));
            assert_eq!(captured.value.stdout, b"raw-output\r\n");
            assert_eq!(captured.value.stderr, b"raw-error \r\n");
            assert_eq!(captured.value.stdout.capacity(), caps.stdout_max);
            assert_eq!(captured.value.stderr.capacity(), caps.stderr_max);
            assert_eq!(resources.stats().current.io, bytes);
            drop(captured);
            assert_eq!(resources.stats().current, baseline);
        }
        let mut missing = Command::new(root.join("missing-tool.exe"));
        let failure = ctool_capture::run(
            &mut missing,
            Duration::from_secs(10),
            caps,
            Some(&resources),
        );
        assert!(matches!(
            failure,
            Err(ctool_capture::CaptureError::Subprocess(
                ace_core::link::SubprocError::Spawn(_)
            ))
        ));
        assert_eq!(resources.stats().current, baseline);
    }

    #[test]
    fn compiler_dependency_paths_reserve_native_probes_array_growth_and_unique_owned_text() {
        use ace_source_query::resources::Resources;
        let (root, _, _, _) = fixture("dependency-parser-budget");
        let source = root.join("provider.c");
        let paths: Vec<PathBuf> = (0..257)
            .map(|i| {
                let name = if i == 256 {
                    format!("unique-final-{}.h", "q".repeat(64))
                } else {
                    format!("header-{i:04}.h")
                };
                let path = root.join(name);
                std::fs::write(&path, b"header").unwrap();
                path
            })
            .collect();
        let mut output = format!(
            "Note: including file: {}\nNote: including file: {}\n",
            source.display(),
            root.join("missing.h").display()
        );
        for path in &paths {
            output.push_str(&format!("Note: including file: {}\n", path.display()));
        }
        output.push_str(&format!("注意: 包含文件: \"{}\"\n", paths[0].display()));
        let resources = Resources::default();
        let baseline = resources.stats().current;
        let old_array = 128 * std::mem::size_of::<PathBuf>();
        let rejected_array = 256 * std::mem::size_of::<PathBuf>();
        let text_before: usize = paths[..128].iter().map(|p| p.as_os_str().len()).sum();
        let allowance =
            old_array + text_before + cache_paths::native_path_size(&paths[128]).unwrap() + 16;
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline.index - allowance))
            .unwrap();
        let pressured = resources.stats().current;
        let mut admission = resources.reserve(Charge::index(0)).unwrap();
        let (rejected, copies) = measured_at(rejected_array, || {
            ctool_dependencies::parse(&output, &source, |bytes| {
                admission
                    .resize(Charge::index(bytes))
                    .map_err(|e| e.to_string())
            })
        });
        assert!(rejected.is_err());
        assert_eq!(copies, 0, "dependency array grew before refusal");
        drop(admission);
        assert_eq!(resources.stats().current, pressured);
        drop(pressure);
        let array = 512 * std::mem::size_of::<PathBuf>();
        let text: usize = paths.iter().map(|p| p.as_os_str().len()).sum();
        let final_text = paths.last().unwrap().as_os_str().len();
        for (layout, minimum) in [
            (
                array,
                baseline.index + text + array + 256 * std::mem::size_of::<PathBuf>(),
            ),
            (final_text, baseline.index + text + array),
        ] {
            let mut admission = resources.reserve(Charge::index(0)).unwrap();
            let (parsed, hits, admitted) =
                with_charge_probe(layout, &resources, Charge::index(minimum), || {
                    ctool_dependencies::parse(&output, &source, |bytes| {
                        admission
                            .resize(Charge::index(bytes))
                            .map_err(|e| e.to_string())
                    })
                });
            let parsed = parsed.unwrap();
            assert_eq!(
                parsed, paths,
                "source/missing/duplicate handling or encounter order changed"
            );
            assert_eq!(hits, 1, "did not observe dependency allocation {layout}");
            assert!(admitted, "dependency storage allocated before admission");
            let retained = parsed.capacity() * std::mem::size_of::<PathBuf>()
                + parsed.iter().map(PathBuf::capacity).sum::<usize>();
            assert_eq!(resources.stats().current.index, baseline.index + retained);
            drop(parsed);
            drop(admission);
            assert_eq!(resources.stats().current, baseline);
        }
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

    fn large_cache_fixture(
        tag: &str,
    ) -> (
        PathBuf,
        ace_artifacts::store::ManagedStore,
        Vec<u8>,
        [PathBuf; 1],
    ) {
        use ace_artifacts::store::ManagedStore;
        let (root, _old, _environment, _caps) = fixture(tag);
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
        (root, store, bytes, artifacts)
    }

    #[test]
    fn c_cache_recursive_listing_precharges_directory_growth_and_keeps_original_applicability() {
        use ace_artifacts::store::{ManagedStore, StoreError};
        use ace_source_query::resources::Resources;
        let (root, bytes, _, _) = fixture("c-cache-listing");
        let source = root.join("provider.c");
        let header = root.join("source_foreign938.h");
        let object = root.join("provider.obj");
        let artifacts = [object.clone()];
        let sub = root.join("sub");
        std::fs::create_dir_all(sub.join("deep")).unwrap();
        for i in 0..300 {
            std::fs::write(root.join(format!("candidate-{i:04}")), b"header").unwrap();
        }
        std::fs::write(sub.join("deep/settings"), b"extensionless").unwrap();
        // A matching basename elsewhere is a real negative dependency.
        std::fs::write(sub.join("provider.obj"), b"header").unwrap();
        let staged = object.with_file_name("PROVIDER.OBJ.TMP-controlled");
        std::fs::write(&staged, b"owned output").unwrap();
        let legacy = ManagedStore::open(&root.with_extension("legacy-store")).unwrap();
        ace_abi::cache::produce_c_object(&legacy, &source, &root, &artifacts, || {
            Ok((bytes.as_ref().clone(), vec![header.clone()]))
        })
        .unwrap();
        let old_id: String = legacy
            .db()
            .query_row("SELECT artifact_id FROM artifacts", [], |r| r.get(0))
            .unwrap();
        let expected = legacy.get_artifact_by_id(&old_id).unwrap().unwrap().payload;
        let listing: serde_json::Value = serde_json::from_slice(&expected).unwrap();
        // Original candidate derivation retains both NT-prefixed and plain
        // spellings where applicable; exercise every recorded scan.
        let listing_count = listing["dir_listings"].as_array().unwrap().len();
        let names = listing["dir_listings"][0][1].as_array().unwrap();
        assert!(names.iter().any(|n| n == "sub/deep/settings"));
        assert!(names.iter().any(|n| n == "sub/provider.obj"));
        assert!(!names
            .iter()
            .any(|n| n == "provider.obj" || n == "PROVIDER.OBJ.TMP-controlled"));
        let store = ManagedStore::open(&root.with_extension("resource-store")).unwrap();
        let resources = Resources::default();
        resources
            .adopt(&bytes, Charge::resident(bytes.capacity() + 40))
            .unwrap();
        let baseline = resources.stats().current;
        // Leave enough for initial metadata but not a collected directory.
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline.index - (64 << 10)))
            .unwrap();
        let (rejected, large) = measured_at(128 << 10, || {
            ace_abi::cache::produce_c_object_with_resources(
                &store,
                &source,
                &root,
                &artifacts,
                &resources,
                || Ok((bytes.clone(), vec![header.clone()])),
            )
        });
        assert!(
            matches!(rejected, Err(StoreError::ResourceExhausted(ref message)) if message.starts_with("C cache listing:")),
            "{rejected:?}"
        );
        assert_eq!(large, 0, "collected the directory before its reservation");
        assert_eq!(resources.stats().current.index, (8 << 20) - (64 << 10));
        assert_eq!(resources.stats().current.resident, baseline.resident);
        assert_eq!(
            store
                .db()
                .query_row("SELECT count(*) FROM artifacts", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        drop(pressure);
        let layout = 512 * std::mem::size_of::<std::fs::DirEntry>();
        for expected_hit in [false, true] {
            let (result, hits, admitted) =
                with_charge_probe(layout, &resources, Charge::index(layout * 2), || {
                    ace_abi::cache::produce_c_object_with_resources(
                        &store,
                        &source,
                        &root,
                        &artifacts,
                        &resources,
                        || {
                            assert!(!expected_hit, "hit called the compiler");
                            Ok((bytes.clone(), vec![header.clone()]))
                        },
                    )
                });
            let (output, hit) = result.unwrap();
            assert_eq!(hit, expected_hit);
            assert_eq!(output.as_slice(), bytes.as_slice());
            assert_eq!(
                hits, listing_count,
                "did not observe every directory vector growth"
            );
            assert!(
                admitted,
                "directory grew before reserving old/new and sort storage"
            );
            drop(output);
            // The input Arc remains live, so its resident fee remains live.
            assert_eq!(resources.stats().current.resident, bytes.capacity() + 40);
            std::fs::remove_file(&staged).ok();
        }
        let id: String = store
            .db()
            .query_row("SELECT artifact_id FROM artifacts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            store.get_artifact_by_id(&id).unwrap().unwrap().payload,
            expected
        );
        let output_fee: usize = listing["dir_listings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                let names = entry[1].as_array().unwrap();
                names.len().next_power_of_two().max(4) * std::mem::size_of::<String>()
                    + names
                        .iter()
                        .map(|name| name.as_str().unwrap().len())
                        .sum::<usize>()
            })
            .sum();
        let other = ManagedStore::open(&root.with_extension("ownership-store")).unwrap();
        let (written, hits, admitted) = with_charge_probe(
            expected.len(),
            &resources,
            Charge::index(output_fee),
            || {
                ace_abi::cache::produce_c_object_with_resources(
                    &other,
                    &source,
                    &root,
                    &artifacts,
                    &resources,
                    || Ok((bytes.clone(), vec![header.clone()])),
                )
            },
        );
        drop(written.unwrap());
        assert_eq!(
            hits, 1,
            "did not observe JSON emission after all scan frames closed"
        );
        assert!(
            admitted,
            "listing ownership fee was released before its last consumer"
        );
        std::fs::write(sub.join("deep/new-header"), b"new negative dependency").unwrap();
        let mut produced = 0;
        let (output, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || {
                produced += 1;
                Ok((bytes.clone(), vec![header]))
            },
        )
        .unwrap();
        assert!(!hit);
        assert_eq!(produced, 1);
        drop((output, bytes));
        assert_eq!(resources.stats().current.resident, 0);
        assert_eq!(resources.stats().current.io, 0);
    }

    #[test]
    fn c_cache_generation_precharges_borrowed_lines_and_preserves_the_original_digest() {
        use ace_artifacts::store::{ManagedStore, StoreError};
        use ace_source_query::resources::Resources;
        let (root, bytes, _, _) = fixture("c-cache-generation");
        let source = root.join("provider.c");
        let header = root.join("source_foreign938.h");
        let extra_header = root.join("source_foreign938.h.extra");
        std::fs::write(&extra_header, b"a conservatively retained dependency").unwrap();
        std::fs::write(root.join("candidate-π-😀"), b"unicode candidate").unwrap();
        let artifacts = vec![root.join("provider.obj"); 4096];
        let legacy = ManagedStore::open(&root.with_extension("legacy-store")).unwrap();
        ace_abi::cache::produce_c_object(&legacy, &source, &root, &artifacts, || {
            Ok((
                bytes.as_ref().clone(),
                vec![header.clone(), extra_header.clone()],
            ))
        })
        .unwrap();
        let (id, expected_key): (String, String) = legacy
            .db()
            .query_row("SELECT artifact_id,key_canonical FROM artifacts", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        let wire = legacy.get_artifact_by_id(&id).unwrap().unwrap().payload;
        let entry: serde_json::Value = serde_json::from_slice(&wire).unwrap();
        // Independent copy of the original owning-format/sort/dedup/join
        // oracle, including the repeated artifact rows. Production uses none
        // of these owned lines or the joined buffer.
        let deps = entry["deps"].as_array().unwrap();
        let listings = entry["dir_listings"].as_array().unwrap();
        let mut lines: Vec<String> = deps
            .iter()
            .map(|d| format!("dep|{}|{}", d[0].as_str().unwrap(), d[1].as_str().unwrap()))
            .collect();
        for (label, field) in [
            ("src_dir", "src_dir"),
            ("inc_dir", "include_dir"),
            ("target", "target"),
            ("toolchain", "toolchain"),
        ] {
            lines.push(format!("{label}|{}", entry[field].as_str().unwrap()));
        }
        lines.push("env|4".into());
        let normalized = std::fs::canonicalize(&root).unwrap().display().to_string();
        let artifact = format!(
            "{}\\provider.obj",
            normalized
                .strip_prefix("\\\\?\\")
                .unwrap_or(&normalized)
                .replace('/', "\\")
        );
        for _ in &artifacts {
            lines.push(format!("art|{artifact}"));
        }
        for listing in listings {
            let dir = listing[0].as_str().unwrap();
            lines.push(format!("dir|{dir}"));
            for file in listing[1].as_array().unwrap() {
                lines.push(format!("file|{dir}|{}", file.as_str().unwrap()));
            }
        }
        let count = lines.len();
        lines.sort();
        lines.dedup();
        let digest = ace_abi::semref::blake3_hex(lines.join("\n").as_bytes());
        let source_id = ace_abi::cache::c_source_identity(&source).unwrap();
        assert_eq!(
            expected_key,
            ace_abi::cache::c_object_key(&source_id, &digest).canonical()
        );
        let resources = Resources::default();
        resources
            .adopt(&bytes, Charge::resident(bytes.capacity() + 40))
            .unwrap();
        let baseline = resources.stats().current;
        let store = ManagedStore::open(&root.with_extension("resource-store")).unwrap();
        let mut pressure = None;
        let mut pressure_bytes = 0;

        let (rejected, large) = measured_at(128 << 10, || {
            ace_abi::cache::produce_c_object_with_resources(
                &store,
                &source,
                &root,
                &artifacts,
                &resources,
                || {
                    // The now-budgeted artifact-path collection must finish
                    // before isolating the later generation-vector refusal.
                    pressure_bytes = (8 << 20) - resources.stats().current.index - (64 << 10);
                    pressure = Some(
                        resources
                            .reserve(Charge::index(pressure_bytes))
                            .map_err(|e| StoreError::ResourceExhausted(e.to_string()))?,
                    );
                    Ok((bytes.clone(), vec![header.clone(), extra_header.clone()]))
                },
            )
        });
        assert!(
            matches!(rejected, Err(StoreError::ResourceExhausted(ref message)) if message.starts_with("C cache generation:")),
            "{rejected:?}"
        );
        assert_eq!(large, 0, "allocated generation storage before admission");
        assert_eq!(
            resources.stats().current.index,
            baseline.index + pressure_bytes
        );
        assert_eq!(
            store
                .db()
                .query_row("SELECT count(*) FROM artifacts", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        drop(pressure);
        let layout = count * (4 * std::mem::size_of::<&str>());
        let artifact_fee = artifacts.len().next_power_of_two().max(4)
            * std::mem::size_of::<String>()
            + artifacts.len() * artifact.len();
        let (result, hits, admitted) = with_charge_probe(
            layout,
            &resources,
            Charge::index(baseline.index + artifact_fee + layout + 64),
            || {
                ace_abi::cache::produce_c_object_with_resources(
                    &store,
                    &source,
                    &root,
                    &artifacts,
                    &resources,
                    || Ok((bytes.clone(), vec![header, extra_header])),
                )
            },
        );
        let (output, hit) = result.unwrap();
        assert!(!hit);
        assert_eq!(hits, 1, "did not observe borrowed-line vector allocation");
        assert!(admitted, "generation allocated before its reservation");
        assert_eq!(output.as_slice(), bytes.as_slice());
        let (id, key): (String, String) = store
            .db()
            .query_row("SELECT artifact_id,key_canonical FROM artifacts", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(key, expected_key);
        assert_eq!(
            store.get_artifact_by_id(&id).unwrap().unwrap().payload,
            wire
        );
        drop((output, bytes));
        assert_eq!(resources.stats().current.resident, 0);
        assert_eq!(resources.stats().current.io, 0);
    }

    #[test]
    fn c_cache_dependency_metadata_precharges_its_array_and_retains_paths_through_encoding() {
        use ace_artifacts::store::{ManagedStore, StoreError};
        use ace_source_query::resources::Resources;
        let (root, bytes, _, _) = fixture("c-cache-dependency-metadata");
        let source = root.join("provider.c");
        let header = root.join("source_foreign938.h");
        let deps = vec![header; 4096];
        let artifacts = [root.join("provider.obj")];
        let legacy = ManagedStore::open(&root.with_extension("legacy-store")).unwrap();
        ace_abi::cache::produce_c_object(&legacy, &source, &root, &artifacts, || {
            Ok((bytes.as_ref().clone(), deps.clone()))
        })
        .unwrap();
        let (id, key): (String, String) = legacy
            .db()
            .query_row("SELECT artifact_id,key_canonical FROM artifacts", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        let wire = legacy.get_artifact_by_id(&id).unwrap().unwrap().payload;
        let resources = Resources::default();
        resources
            .adopt(&bytes, Charge::resident(bytes.capacity() + 40))
            .unwrap();
        let baseline = resources.stats().current;
        let store = ManagedStore::open(&root.with_extension("resource-store")).unwrap();
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline.index - (64 << 10)))
            .unwrap();
        let cold_deps = deps.clone();
        let (rejected, large) = measured_at(128 << 10, || {
            ace_abi::cache::produce_c_object_with_resources(
                &store,
                &source,
                &root,
                &artifacts,
                &resources,
                || Ok((bytes.clone(), cold_deps)),
            )
        });
        assert!(
            matches!(rejected, Err(StoreError::ResourceExhausted(ref message)) if message.starts_with("C cache metadata:")),
            "{rejected:?}"
        );
        assert_eq!(large, 0, "allocated dependency index before admission");
        assert_eq!(resources.stats().current.index, (8 << 20) - (64 << 10));
        assert_eq!(
            store
                .db()
                .query_row("SELECT count(*) FROM artifacts", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        drop(pressure);
        let layout = deps.len() * std::mem::size_of::<(String, String)>();
        let (result, hits, admitted) =
            with_charge_probe(layout, &resources, Charge::index(layout), || {
                ace_abi::cache::produce_c_object_with_resources(
                    &store,
                    &source,
                    &root,
                    &artifacts,
                    &resources,
                    || Ok((bytes.clone(), deps.clone())),
                )
            });
        let (output, hit) = result.unwrap();
        assert!(!hit);
        assert_eq!(hits, 1, "did not observe dependency array allocation");
        assert!(
            admitted,
            "dependency array allocated before its reservation"
        );
        assert_eq!(output.as_slice(), bytes.as_slice());
        drop(output);
        let (id, actual_key): (String, String) = store
            .db()
            .query_row("SELECT artifact_id,key_canonical FROM artifacts", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(actual_key, key);
        assert_eq!(
            store.get_artifact_by_id(&id).unwrap().unwrap().payload,
            wire
        );
        let other = ManagedStore::open(&root.with_extension("ownership-store")).unwrap();
        let full_fee = layout + deps.iter().map(|p| p.as_os_str().len() + 64).sum::<usize>();
        let (result, hits, admitted) =
            with_charge_probe(wire.len(), &resources, Charge::index(full_fee), || {
                ace_abi::cache::produce_c_object_with_resources(
                    &other,
                    &source,
                    &root,
                    &artifacts,
                    &resources,
                    || Ok((bytes.clone(), deps.clone())),
                )
            });
        drop(result.unwrap());
        assert_eq!(hits, 1, "did not observe final metadata consumer");
        assert!(admitted, "dependency paths lost their fee before encoding");
        let failed = ManagedStore::open(&root.with_extension("failed-store")).unwrap();
        let missing = root.join("missing-dependency");
        let rejected = ace_abi::cache::produce_c_object_with_resources(
            &failed,
            &source,
            &root,
            &artifacts,
            &resources,
            || Ok((bytes.clone(), vec![missing.clone()])),
        );
        assert!(
            matches!(rejected, Err(StoreError::Internal(ref message)) if message.starts_with("dep "))
        );
        assert_eq!(
            failed
                .db()
                .query_row("SELECT count(*) FROM artifacts", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(resources.stats().current.index, baseline.index);
        std::fs::write(&missing, b"repaired conservative dependency").unwrap();
        let (repaired, hit) = ace_abi::cache::produce_c_object_with_resources(
            &failed,
            &source,
            &root,
            &artifacts,
            &resources,
            || Ok((bytes.clone(), vec![missing])),
        )
        .unwrap();
        assert!(!hit);
        assert_eq!(repaired.as_slice(), bytes.as_slice());
        drop(repaired);
        assert_eq!(resources.stats().current.index, baseline.index);
        drop(bytes);
        assert_eq!(resources.stats().current.resident, 0);
        assert_eq!(resources.stats().current.io, 0);
    }

    #[test]
    fn c_cache_entry_read_reserves_shared_io_before_loading_the_original_store_frame() {
        use ace_artifacts::store::StoreError;
        use ace_source_query::resources::Resources;
        let (root, store, bytes, artifacts) = large_cache_fixture("c-cache-entry-io");
        let source = root.join("provider.c");
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
        assert_eq!(repaired.as_slice(), bytes.as_slice());
        assert_eq!(
            resources.stats().current.resident - before.current.resident,
            repaired.capacity() + 40
        );
        let alias = repaired.clone();
        drop(repaired);
        assert!(resources.stats().current.resident > before.current.resident);
        drop(alias);
        let after = resources.stats();
        assert_eq!(after.current.resident, before.current.resident);
        assert_eq!(after.current.io, before.current.io);
        assert_eq!(after.live_objects, 0);
    }

    #[test]
    fn c_cache_encoded_payload_precharges_coexistence_and_preserves_legacy_wire_bytes() {
        use ace_artifacts::{
            codec::CHUNK_HEADER_LEN,
            store::{ManagedStore, StoreError},
        };
        use ace_source_query::resources::Resources;
        let (root, legacy, bytes, artifacts) = large_cache_fixture("c-cache-encoded-write");
        let source = root.join("provider.c");
        let header = root.join("source_foreign938.h");
        let object = root.join("provider.obj");
        let legacy_id: String = legacy
            .db()
            .query_row(
                "SELECT artifact_id FROM artifacts WHERE layer=?1",
                [ace_abi::cache::LAYER_OBJECT_V01.id],
                |row| row.get(0),
            )
            .unwrap();
        let expected = legacy
            .get_artifact_by_id(&legacy_id)
            .unwrap()
            .unwrap()
            .payload;
        let store = ManagedStore::open(&root.with_extension("resource-store")).unwrap();
        let resources = Resources::default();
        let pressure = resources
            .reserve(Charge::io((8 << 20) - (3 << 20)))
            .unwrap();
        let (rejected, copies) = measured_at(1 << 20, || {
            ace_abi::cache::produce_c_object_with_resources(
                &store,
                &source,
                &root,
                &artifacts,
                &resources,
                || {
                    Ok((
                        ace_abi::cache::read_c_object_file(&object, &resources)?,
                        vec![header.clone()],
                    ))
                },
            )
        });
        assert!(
            matches!(rejected, Err(StoreError::ResourceExhausted(ref message)) if message.starts_with("C cache entry encode:"))
        );
        assert_eq!(
            copies, 1,
            "rejected write allocated more than the admitted object-file input"
        );
        assert_eq!(
            store
                .db()
                .query_row("SELECT count(*) FROM artifacts", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(resources.stats().current.resident, 0);
        assert_eq!(resources.stats().current.io, (8 << 20) - (3 << 20));
        drop(pressure);
        let bound = (expected.len() + CHUNK_HEADER_LEN) * 2;
        let (result, hits, admitted) =
            with_charge_probe(expected.len(), &resources, Charge::io(bound), || {
                ace_abi::cache::produce_c_object_with_resources(
                    &store,
                    &source,
                    &root,
                    &artifacts,
                    &resources,
                    || {
                        Ok((
                            ace_abi::cache::read_c_object_file(&object, &resources)?,
                            vec![header.clone()],
                        ))
                    },
                )
            });
        let (output, hit) = result.unwrap();
        assert!(!hit);
        assert_eq!(
            hits, 1,
            "did not observe the exact-sized payload allocation"
        );
        assert!(
            admitted,
            "encoded payload was allocated before coexistence admission"
        );
        assert_eq!(output.as_slice(), bytes.as_slice());
        assert_eq!(resources.stats().current.io, 0);
        assert_eq!(resources.stats().current.resident, output.capacity() + 40);
        let id: String = store
            .db()
            .query_row(
                "SELECT artifact_id FROM artifacts WHERE layer=?1",
                [ace_abi::cache::LAYER_OBJECT_V01.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            store.get_artifact_by_id(&id).unwrap().unwrap().payload,
            expected,
            "streamed base64/borrowed fields changed the original v4 wire"
        );
        drop(output);
        let frame_store = ManagedStore::open(&root.with_extension("frame-store")).unwrap();
        let (framed, hits, admitted) = with_charge_probe(
            expected.len() + CHUNK_HEADER_LEN,
            &resources,
            Charge::io(bound),
            || {
                ace_abi::cache::produce_c_object_with_resources(
                    &frame_store,
                    &source,
                    &root,
                    &artifacts,
                    &resources,
                    || {
                        Ok((
                            ace_abi::cache::read_c_object_file(&object, &resources)?,
                            vec![header.clone()],
                        ))
                    },
                )
            },
        );
        let (framed, hit) = framed.unwrap();
        assert!(!hit);
        assert_eq!(
            hits, 1,
            "did not observe the original store's framed allocation"
        );
        assert!(
            admitted,
            "encoded payload fee was released before the store framed it"
        );
        assert_eq!(framed.as_slice(), bytes.as_slice());
        drop(framed);
        let (cached, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || panic!("encoded object did not hit the original cache"),
        )
        .unwrap();
        assert!(hit);
        assert_eq!(cached.as_slice(), bytes.as_slice());
        drop(cached);
        assert_eq!(resources.stats().current.resident, 0);
        assert_eq!(resources.stats().current.io, 0);
    }

    #[test]
    fn c_cache_conflict_releases_finished_encoding_before_rediscovering_the_canonical_object() {
        use ace_artifacts::store::ManagedStore;
        use ace_source_query::resources::Resources;
        let (root, _legacy, bytes, artifacts) = large_cache_fixture("c-cache-encode-conflict");
        let source = root.join("provider.c");
        let header = root.join("source_foreign938.h");
        let store = ManagedStore::open(&root.with_extension("conflict-store")).unwrap();
        let resources = Resources::default();
        let mut produced = 0;
        let (canonical, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || {
                produced += 1;
                // Controlled interleaving: another caller has published the
                // valid generation after our discovery but before our put.
                let (_, already) =
                    ace_abi::cache::produce_c_object(&store, &source, &root, &artifacts, || {
                        Ok((bytes.clone(), vec![header.clone()]))
                    })?;
                assert!(!already);
                // A COFF timestamp difference reproduces the original legal
                // same-generation race without changing source or closure.
                let mut contender = bytes.clone();
                contender[4] ^= 1;
                let contender = resources.own_payload(contender).map_err(|e| {
                    ace_artifacts::store::StoreError::ResourceExhausted(e.to_string())
                })?;
                Ok((contender, vec![header.clone()]))
            },
        )
        .unwrap();
        assert!(
            !hit,
            "the contender actually ran; this is not a producer-free hit"
        );
        assert_eq!(produced, 1);
        assert_eq!(
            canonical.as_slice(),
            bytes.as_slice(),
            "did not use the verified canonical winner"
        );
        assert_eq!(resources.stats().current.io, 0);
        assert_eq!(
            resources.stats().current.resident,
            canonical.capacity() + 40,
            "discarded contender retained its output or encoded fee"
        );
        drop(canonical);
        assert_eq!(resources.stats().current.resident, 0);
    }

    #[test]
    fn compiler_object_file_read_reserves_before_allocation_and_retains_only_the_shared_output() {
        use ace_artifacts::store::StoreError;
        use ace_source_query::resources::Resources;
        use std::os::windows::fs::OpenOptionsExt;
        let (root, _store, bytes, _artifacts) = large_cache_fixture("c-object-file-read");
        let path = root.join("provider.obj");
        let resources = Resources::default();
        let pressure = resources.reserve(Charge::io((8 << 20) - 1024)).unwrap();
        let pressured = resources.stats().current;
        let (rejected, peak) =
            measured_peak(|| ace_abi::cache::read_c_object_file(&path, &resources));
        assert!(
            matches!(rejected, Err(StoreError::ResourceExhausted(ref message)) if message.starts_with("C object read:"))
        );
        assert!(
            peak < 64 << 10,
            "object file buffer was created before I/O admission: {peak}"
        );
        assert_eq!(resources.stats().current, pressured);
        drop(pressure);
        let (result, hits, admitted) = with_charge_probe(
            bytes.len(),
            &resources,
            Charge::io(bytes.len() + 256),
            || ace_abi::cache::read_c_object_file(&path, &resources),
        );
        let object = result.unwrap();
        assert_eq!(hits, 1);
        assert!(
            admitted,
            "object file allocation preceded its I/O reservation"
        );
        assert_eq!(object.as_slice(), bytes.as_slice());
        assert_eq!(resources.stats().current.resident, object.capacity() + 40);
        assert_eq!(resources.stats().current.io, 0);
        let alias = object.clone();
        drop(object);
        assert!(resources.stats().current.resident > 0);
        drop(alias);
        let baseline = resources.stats().current;
        assert_eq!(baseline.resident, 0);
        let writer = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(3)
            .open(&path)
            .unwrap();
        assert!(ace_abi::cache::read_c_object_file(&path, &resources).is_err());
        drop(writer);
        assert_eq!(resources.stats().current, baseline);
        assert!(ace_abi::cache::read_c_object_file(&root, &resources).is_err());
        assert!(ace_abi::cache::read_c_object_file(&root.join("missing.obj"), &resources).is_err());
        let oversized = root.join("oversized.obj");
        std::fs::File::create(&oversized)
            .unwrap()
            .set_len((64 << 20) + 1)
            .unwrap();
        let (rejected, peak) =
            measured_peak(|| ace_abi::cache::read_c_object_file(&oversized, &resources));
        assert!(matches!(rejected, Err(StoreError::ResourceExhausted(_))));
        assert!(
            peak < 64 << 10,
            "oversized object was allocated before its artifact limit: {peak}"
        );
        assert_eq!(resources.stats().current, baseline);
    }

    #[test]
    fn c_cache_decoded_object_is_charged_at_its_real_allocation_and_pinned_by_aliases() {
        use ace_artifacts::codec::ChunkKind;
        use ace_source_query::resources::Resources;
        let (root, store, bytes, artifacts) = large_cache_fixture("c-cache-output-owner");
        let source = root.join("provider.c");
        let id: String = store
            .db()
            .query_row(
                "SELECT artifact_id FROM artifacts WHERE layer=?1",
                [ace_abi::cache::LAYER_OBJECT_V01.id],
                |row| row.get(0),
            )
            .unwrap();
        let resources = Resources::default();
        let raw = resources
            .load_payload(64 << 20, ChunkKind::Object, |bound| {
                store.get_artifact_by_id_bounded(&id, bound)
            })
            .unwrap()
            .unwrap();
        let raw_charge = raw.capacity() + 40;
        drop(raw);
        let before = resources.stats().current;
        // This is the pinned base64 engine's output-allocation upper bound;
        // the observation is made by the actual allocator before it returns
        // control to a caller that could otherwise reserve too late.
        let capacity = (bytes.len() + 2) / 3 * 3;
        let (result, hits, admitted) =
            with_resident_probe(capacity, &resources, raw_charge + capacity + 40, || {
                ace_abi::cache::produce_c_object_with_resources(
                    &store,
                    &source,
                    &root,
                    &artifacts,
                    &resources,
                    || panic!("decoded output test lost the cached object"),
                )
            });
        let (object, hit) = result.unwrap();
        assert!(hit);
        assert_eq!(object.as_slice(), bytes.as_slice());
        assert_eq!(hits, 1, "allocator did not observe the real decoded output");
        assert!(
            admitted,
            "decoded output was allocated before its shared resident reservation"
        );
        assert_eq!(
            resources.stats().current.resident - before.resident,
            object.capacity() + 40
        );
        resources
            .adopt(&object, Charge::resident(object.capacity() + 40))
            .unwrap();
        let alias = object.clone();
        drop(object);
        assert!(resources.stats().current.resident > before.resident);
        drop(alias);
        assert_eq!(resources.stats().current, before);
    }

    #[test]
    fn c_cache_invalid_base64_releases_its_output_reservation_and_remains_a_miss() {
        use ace_artifacts::{
            codec::ChunkKind,
            store::{ManagedStore, PutRequest, StoreError},
        };
        use ace_source_query::resources::Resources;
        let (root, bytes, _environment, _caps) = fixture("c-cache-invalid-base64");
        let source = root.join("provider.c");
        let header = root.join("source_foreign938.h");
        let artifacts = [root.join("provider.obj")];
        let store = ManagedStore::open(&root.with_extension("cache-store")).unwrap();
        ace_abi::cache::produce_c_object(&store, &source, &root, &artifacts, || {
            Ok((bytes.as_ref().clone(), vec![header.clone()]))
        })
        .unwrap();
        let id: String = store
            .db()
            .query_row(
                "SELECT artifact_id FROM artifacts WHERE layer=?1",
                [ace_abi::cache::LAYER_OBJECT_V01.id],
                |row| row.get(0),
            )
            .unwrap();
        let loaded = store.get_artifact_by_id(&id).unwrap().unwrap();
        let mut entry: serde_json::Value = serde_json::from_slice(&loaded.payload).unwrap();
        entry["obj_b64"] = "!!!".into();
        assert_eq!(
            store
                .db()
                .execute("DELETE FROM artifacts WHERE artifact_id=?1", [&id])
                .unwrap(),
            1
        );
        let identity = ace_abi::cache::c_source_identity(&source).unwrap();
        store
            .put_artifact(PutRequest {
                layer: ace_abi::cache::LAYER_OBJECT_V01.id,
                key: ace_abi::cache::c_object_key(&identity, "invalid-base64-fixture-v4"),
                kind: ChunkKind::Object,
                payload: serde_json::to_vec(&entry).unwrap(),
                edges: vec![],
                project: None,
            })
            .unwrap();
        let resources = Resources::default();
        let mut produced = 0;
        let (object, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || {
                produced += 1;
                let object = resources
                    .own_payload(bytes.as_ref().clone())
                    .map_err(|e| StoreError::ResourceExhausted(e.to_string()))?;
                Ok((object, vec![header]))
            },
        )
        .unwrap();
        assert!(!hit);
        assert_eq!(produced, 1);
        assert_eq!(object.as_slice(), bytes.as_slice());
        assert_eq!(
            resources.stats().current.resident,
            object.capacity() + 40,
            "failed base64 decode leaked a live output fee"
        );
        drop(object);
        assert_eq!(resources.stats().current.resident, 0);
        assert_eq!(resources.stats().current.io, 0);
    }

    #[test]
    fn c_cache_malformed_entry_remains_a_miss_in_the_shared_resource_path() {
        use ace_artifacts::{
            codec::ChunkKind,
            store::{ManagedStore, PutRequest},
        };
        use ace_source_query::resources::Resources;
        let (root, bytes, _environment, _caps) = fixture("c-cache-malformed-entry");
        let source = root.join("provider.c");
        let header = root.join("source_foreign938.h");
        let artifacts = [root.join("provider.obj")];
        let store = ManagedStore::open(&root.with_extension("cache-store")).unwrap();
        let identity = ace_abi::cache::c_source_identity(&source).unwrap();
        store
            .put_artifact(PutRequest {
                layer: ace_abi::cache::LAYER_OBJECT_V01.id,
                key: ace_abi::cache::c_object_key(&identity, "malformed-fixture-v4"),
                kind: ChunkKind::Object,
                payload: b"{".to_vec(),
                edges: vec![],
                project: None,
            })
            .unwrap();
        let resources = Resources::default();
        let mut produced = 0;
        let (actual, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || {
                produced += 1;
                let shared = resources.own_payload(bytes.as_ref().clone()).map_err(|e| {
                    ace_artifacts::store::StoreError::ResourceExhausted(e.to_string())
                })?;
                Ok((shared, vec![header]))
            },
        )
        .unwrap();
        assert!(!hit);
        assert_eq!(
            produced, 1,
            "malformed metadata was neither served nor a real miss"
        );
        assert_eq!(actual.as_slice(), bytes.as_slice());
        let (repeated, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || panic!("valid replacement was not reused"),
        )
        .unwrap();
        assert!(hit);
        assert_eq!(repeated.as_slice(), bytes.as_slice());
        assert_eq!(
            resources.stats().current.resident,
            actual.capacity() + repeated.capacity() + 80
        );
        drop((actual, repeated));
        assert_eq!(resources.stats().current.resident, 0);
        assert_eq!(resources.stats().current.io, 0);
    }

    #[test]
    fn c_cache_entry_decode_reserves_shared_index_before_owning_typed_metadata() {
        use ace_artifacts::{codec::ChunkKind, store::StoreError};
        use ace_source_query::resources::Resources;
        let (root, store, bytes, artifacts) = large_cache_fixture("c-cache-entry-decode");
        let source = root.join("provider.c");
        let id: String = store
            .db()
            .query_row(
                "SELECT artifact_id FROM artifacts WHERE layer=?1",
                [ace_abi::cache::LAYER_OBJECT_V01.id],
                |row| row.get(0),
            )
            .unwrap();
        let resources = Resources::default();
        // Observe the actual required raw-frame copies separately. A refused
        // typed decode may do these same reads, but no large owned metadata.
        let (raw, raw_copies) = measured_at(1 << 20, || {
            resources.load_payload(64 << 20, ChunkKind::Object, |bound| {
                store.get_artifact_by_id_bounded(&id, bound)
            })
        });
        drop(raw.unwrap().unwrap());
        assert!(raw_copies > 0);
        let before = resources.stats().current;
        let pressure = resources
            .reserve(Charge::index((8 << 20) - before.index - 4096))
            .unwrap();
        let pressured = resources.stats().current;
        let (rejected, copies) = measured_at(1 << 20, || {
            ace_abi::cache::produce_c_object_with_resources(
                &store,
                &source,
                &root,
                &artifacts,
                &resources,
                || panic!("typed entry budget refusal invoked the producer"),
            )
        });
        let error = rejected
            .err()
            .expect("C cache typed metadata bypassed shared index admission");
        assert!(
            matches!(error, StoreError::ResourceExhausted(ref message) if message.starts_with("C cache entry decode:")),
            "{error}"
        );
        assert_eq!(
            copies, raw_copies,
            "typed cache metadata allocated before shared admission"
        );
        assert_eq!(resources.stats().current, pressured);
        drop(pressure);
        let (repaired, hit) = ace_abi::cache::produce_c_object_with_resources(
            &store,
            &source,
            &root,
            &artifacts,
            &resources,
            || panic!("typed entry repair lost the original cached object"),
        )
        .unwrap();
        assert!(hit);
        assert_eq!(repaired.as_slice(), bytes.as_slice());
        assert_eq!(
            resources.stats().current.resident - before.resident,
            repaired.capacity() + 40
        );
        drop(repaired);
        assert_eq!(resources.stats().current, before);
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
            matches!(error, StoreError::ResourceExhausted(ref message) if message.starts_with("C cache metadata:")),
            "{error}"
        );
        assert_eq!(resources.stats().current.index, (8 << 20) - 64);
        // Earlier identity/query owners now also reserve their bytes. Admit
        // those owners here so the separate oversized-row control still
        // reaches the borrowed SQLite cell and its own pre-copy admission.
        drop(pressure);
        let pressure = resources
            .reserve(Charge::index((8 << 20) - baseline.index - (1 << 20)))
            .unwrap();
        // A corrupt metadata row must not turn the borrowed SQLite value
        // into a giant Rust String when less than its 2MiB size remains.
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
        assert_eq!(repaired.as_slice(), bytes.as_slice());
        assert_eq!(
            resources.stats().current.resident - baseline.resident,
            repaired.capacity() + 40
        );
        drop(repaired);
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
