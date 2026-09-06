//! Plan 565 P0: feature-gated counting global allocator (memory attribution).
//!
//! Compiled only with `--features mem-profile` (see `lib.rs` for the
//! `#[global_allocator]` swap). All statistics are lock-free `AtomicU64`
//! counters updated inline in the alloc hooks (Relaxed ordering — statistics
//! only, no synchronization semantics needed).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

/// Size-bucket upper bounds (inclusive); buckets per D1:
/// <=64B / <=256B / <=1K / <=4K / <=64K / >64K.
const BUCKET_BOUNDS: [u64; 5] = [64, 256, 1024, 4096, 65536];

/// Human-readable bucket labels matching the 6 buckets.
pub const BUCKET_LABELS: [&str; 6] = ["<=64B", "<=256B", "<=1K", "<=4K", "<=64K", ">64K"];

/// Cumulative allocation counters since process start.
pub struct MemStats {
    /// Total bytes ever allocated (realloc growth included).
    pub total_bytes: AtomicU64,
    pub alloc_count: AtomicU64,
    pub free_count: AtomicU64,
    /// Bytes currently live (allocated minus freed).
    pub current_live: AtomicU64,
    /// High-water mark of `current_live`, maintained via CAS loop.
    pub peak_live: AtomicU64,
    /// Cumulative allocated bytes per size bucket.
    pub bucket_bytes: [AtomicU64; 6],
    /// Cumulative allocation count per size bucket.
    pub bucket_count: [AtomicU64; 6],
}

pub static STATS: MemStats = MemStats {
    total_bytes: AtomicU64::new(0),
    alloc_count: AtomicU64::new(0),
    free_count: AtomicU64::new(0),
    current_live: AtomicU64::new(0),
    peak_live: AtomicU64::new(0),
    bucket_bytes: [
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
    ],
    bucket_count: [
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
        AtomicU64::new(0),
    ],
};

#[inline]
fn bucket_of(size: u64) -> usize {
    for (i, &bound) in BUCKET_BOUNDS.iter().enumerate() {
        if size <= bound {
            return i;
        }
    }
    5
}

#[inline]
fn record_alloc(size: u64) {
    STATS.total_bytes.fetch_add(size, Ordering::Relaxed);
    STATS.alloc_count.fetch_add(1, Ordering::Relaxed);
    let b = bucket_of(size);
    STATS.bucket_bytes[b].fetch_add(size, Ordering::Relaxed);
    STATS.bucket_count[b].fetch_add(1, Ordering::Relaxed);
    let live = STATS.current_live.fetch_add(size, Ordering::Relaxed) + size;
    bump_peak(live);
}

#[inline]
fn record_free(size: u64) {
    STATS.free_count.fetch_add(1, Ordering::Relaxed);
    STATS.current_live.fetch_sub(size, Ordering::Relaxed);
}

fn bump_peak(candidate: u64) {
    let mut cur = STATS.peak_live.load(Ordering::Relaxed);
    while candidate > cur {
        match STATS.peak_live.compare_exchange_weak(
            cur,
            candidate,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(actual) => cur = actual,
        }
    }
}

/// Counting wrapper around `std::alloc::System`; installed as
/// `#[global_allocator]` from `lib.rs` when the `mem-profile` feature is on.
pub struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            record_alloc(layout.size() as u64);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() {
            record_alloc(layout.size() as u64);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        record_free(layout.size() as u64);
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            // Account as free(old) + alloc(new) — identical invariants to a
            // dealloc+alloc pair.
            STATS.free_count.fetch_add(1, Ordering::Relaxed);
            STATS.current_live.fetch_sub(layout.size() as u64, Ordering::Relaxed);
            record_alloc(new_size as u64);
        }
        new_ptr
    }
}
