//! Exact allocator requests for the dedicated collections worker. Every Rust
//! allocation carries its partition and full backing layout, including this
//! meter's header/alignment; deallocation uses the original partition on any
//! thread. Untagged frontend/store/CLIF/transport allocations remain visible in
//! staging. OS private commit is observed separately, including heap slack.
use crate::{
    resources::{Partition, MANAGED_TOTAL_CAP, PARTITION_CAPS},
    MnError,
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};
thread_local! {static PART:Cell<usize>=const{Cell::new(Partition::Staging as usize)};}
static ENABLED: AtomicBool = AtomicBool::new(false);
static CURRENT: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
static PEAK: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
static TOTAL: AtomicU64 = AtomicU64::new(0);
static TOTAL_PEAK: AtomicU64 = AtomicU64::new(0);
pub fn scoped<T>(partition: Partition, run: impl FnOnce() -> T) -> T {
    struct Restore(usize);
    impl Drop for Restore {
        fn drop(&mut self) {
            PART.with(|p| p.set(self.0));
        }
    }
    let old = PART.with(|p| p.replace(partition as usize));
    let _restore = Restore(old);
    run()
}
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Snapshot {
    pub current: [u64; 3],
    pub peak: [u64; 3],
    pub total_peak: u64,
    pub enabled: bool,
}
pub fn snapshot() -> Snapshot {
    Snapshot {
        current: std::array::from_fn(|i| CURRENT[i].load(Ordering::Relaxed)),
        peak: std::array::from_fn(|i| PEAK[i].load(Ordering::Relaxed)),
        total_peak: TOTAL_PEAK.load(Ordering::Relaxed),
        enabled: ENABLED.load(Ordering::Relaxed),
    }
}
/// Called by the protocol owner only between requests, after all request
/// readers/children have joined. Retained state remains in the new baseline.
pub fn reset_peaks() {
    for i in 0..3 {
        PEAK[i].store(CURRENT[i].load(Ordering::Relaxed), Ordering::Relaxed);
    }
    TOTAL_PEAK.store(TOTAL.load(Ordering::Relaxed), Ordering::Relaxed);
}
pub fn check() -> Result<(), MnError> {
    let s = snapshot();
    if s.enabled
        && (s
            .peak
            .iter()
            .zip(PARTITION_CAPS)
            .any(|(actual, cap)| *actual > cap)
            || s.total_peak > MANAGED_TOTAL_CAP)
    {
        Err(MnError::Resource(
            "compiler-allocations",
            format!(
                "actual allocation peaks {:?}, total {}",
                s.peak, s.total_peak
            ),
        ))
    } else {
        Ok(())
    }
}
#[repr(C)]
struct Header {
    offset: usize,
    total: usize,
    align: usize,
    partition: usize,
}
pub struct CollectionsAllocator;
unsafe impl GlobalAlloc for CollectionsAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align().max(std::mem::align_of::<Header>());
        let Some(offset) = std::mem::size_of::<Header>()
            .checked_add(layout.align() - 1)
            .map(|n| n & !(layout.align() - 1))
        else {
            return std::ptr::null_mut();
        };
        let Some(total) = offset.checked_add(layout.size().max(1)) else {
            return std::ptr::null_mut();
        };
        let Ok(backing) = Layout::from_size_align(total, align) else {
            return std::ptr::null_mut();
        };
        let base = System.alloc(backing);
        if base.is_null() {
            return base;
        }
        let partition = PART
            .try_with(Cell::get)
            .unwrap_or(Partition::Staging as usize);
        let user = base.add(offset);
        user.sub(std::mem::size_of::<Header>())
            .cast::<Header>()
            .write(Header {
                offset,
                total,
                align,
                partition,
            });
        ENABLED.store(true, Ordering::Relaxed);
        let current = CURRENT[partition].fetch_add(total as u64, Ordering::Relaxed) + total as u64;
        PEAK[partition].fetch_max(current, Ordering::Relaxed);
        let total = TOTAL.fetch_add(total as u64, Ordering::Relaxed) + total as u64;
        TOTAL_PEAK.fetch_max(total, Ordering::Relaxed);
        user
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let h = ptr
            .sub(std::mem::size_of::<Header>())
            .cast::<Header>()
            .read();
        CURRENT[h.partition].fetch_sub(h.total as u64, Ordering::Relaxed);
        TOTAL.fetch_sub(h.total as u64, Ordering::Relaxed);
        System.dealloc(
            ptr.sub(h.offset),
            Layout::from_size_align_unchecked(h.total, h.align),
        );
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = self.alloc(layout);
        if !ptr.is_null() {
            ptr.write_bytes(0, layout.size());
        }
        ptr
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) else {
            return std::ptr::null_mut();
        };
        let new = self.alloc(new_layout);
        if new.is_null() {
            return new;
        }
        std::ptr::copy_nonoverlapping(ptr, new, layout.size().min(new_size));
        self.dealloc(ptr, layout);
        new
    }
}
