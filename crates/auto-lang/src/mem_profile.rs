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

// ---------------------------------------------------------------------------
// 采样与报告（Plan 565 T2）
// ---------------------------------------------------------------------------

/// 所有计数器的时点快照（用于阶段差值归因）。
#[derive(Clone, Copy, Debug)]
pub struct Snapshot {
    pub total_bytes: u64,
    pub alloc_count: u64,
    pub free_count: u64,
    pub current_live: u64,
    pub peak_live: u64,
    pub bucket_bytes: [u64; 6],
    pub bucket_count: [u64; 6],
}

impl Snapshot {
    pub fn take() -> Self {
        Snapshot {
            total_bytes: STATS.total_bytes.load(Ordering::Relaxed),
            alloc_count: STATS.alloc_count.load(Ordering::Relaxed),
            free_count: STATS.free_count.load(Ordering::Relaxed),
            current_live: STATS.current_live.load(Ordering::Relaxed),
            peak_live: STATS.peak_live.load(Ordering::Relaxed),
            bucket_bytes: core::array::from_fn(|i| STATS.bucket_bytes[i].load(Ordering::Relaxed)),
            bucket_count: core::array::from_fn(|i| STATS.bucket_count[i].load(Ordering::Relaxed)),
        }
    }

    /// 本快照相对更早快照的逐字段差值（单调计数器饱和减；
    /// `current_live` 可下降，保留符号）。
    pub fn delta_since(&self, earlier: &Snapshot) -> Delta {
        Delta {
            alloc_bytes: self.total_bytes.saturating_sub(earlier.total_bytes),
            allocs: self.alloc_count.saturating_sub(earlier.alloc_count),
            frees: self.free_count.saturating_sub(earlier.free_count),
            live_change: self.current_live as i64 - earlier.current_live as i64,
            bucket_bytes: core::array::from_fn(|i| {
                self.bucket_bytes[i].saturating_sub(earlier.bucket_bytes[i])
            }),
            bucket_allocs: core::array::from_fn(|i| {
                self.bucket_count[i].saturating_sub(earlier.bucket_count[i])
            }),
        }
    }
}

/// 两快照间的差值：阶段内分配量/次数/释放次数/留存变化/分桶。
#[derive(Clone, Copy, Debug)]
pub struct Delta {
    pub alloc_bytes: u64,
    pub allocs: u64,
    pub frees: u64,
    pub live_change: i64,
    pub bucket_bytes: [u64; 6],
    pub bucket_allocs: [u64; 6],
}

fn mib(bytes: u64) -> String {
    format!("{:.1}MiB", bytes as f64 / (1024.0 * 1024.0))
}

fn signed_mib(delta: i64) -> String {
    let sign = if delta < 0 { "-" } else { "+" };
    format!("{sign}{:.1}MiB", delta.unsigned_abs() as f64 / (1024.0 * 1024.0))
}

/// 一条阶段归因报告行（D1 格式：MEMPROFILE 前缀，可 grep）。
pub fn report_delta_line(tag: &str, d: &Delta) -> String {
    format!(
        "MEMPROFILE phase={tag} alloc_bytes={} ({}) allocs={} frees={} live_change={} ({}) bucket_bytes={:?} bucket_allocs={:?}",
        d.alloc_bytes,
        mib(d.alloc_bytes),
        d.allocs,
        d.frees,
        d.live_change,
        signed_mib(d.live_change),
        d.bucket_bytes,
        d.bucket_allocs,
    )
}

/// 进程全程总计行：total/peak_live/live/分配次数 + 分桶
/// （桶序 = BUCKET_LABELS：<=64B/<=256B/<=1K/<=4K/<=64K/>64K）。
pub fn report_total_line() -> String {
    let s = Snapshot::take();
    format!(
        "MEMPROFILE total total_bytes={} ({}) peak_live={} ({}) live={} ({}) allocs={} frees={} bucket_bytes={:?} bucket_allocs={:?} labels={:?}",
        s.total_bytes,
        mib(s.total_bytes),
        s.peak_live,
        mib(s.peak_live),
        s.current_live,
        mib(s.current_live),
        s.alloc_count,
        s.free_count,
        s.bucket_bytes,
        s.bucket_count,
        BUCKET_LABELS,
    )
}
