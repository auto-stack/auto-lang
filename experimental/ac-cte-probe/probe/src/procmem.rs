//! Real process-tree memory aggregation (PLAN-817 F-817-R5-01).
//!
//! The 384 MiB worker gate reads each child's SELF-REPORTED
//! `worker_private_bytes_peak` (a transient allocation inside the child
//! is caught by the child's own 1ms sampler). The 512 MiB TREE gate must
//! instead bound the memory of ALL simultaneously-alive processes:
//! parent + every registered worker child, polled together so the SUM,
//! not each process alone, is compared against the cap. A parent under
//! the cap plus a warm worker under the cap proves nothing about the
//! tree — this sampler closes exactly that hole.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// One private-bytes reading of ANOTHER process (`PrivateUsage` /
/// commit charge). `None` when the handle cannot be opened — the process
/// already exited, or it is not ours to query; callers treat that as "no
/// contribution" (dead processes hold no memory).
#[cfg(windows)]
pub fn per_pid_private_bytes(pid: u32) -> Option<u64> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle == INVALID_HANDLE_VALUE || handle.is_null() {
            return None;
        }
        let mut counters: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        let ok = GetProcessMemoryInfo(handle, &mut counters as *mut _ as *mut _, counters.cb);
        CloseHandle(handle);
        if ok == 0 {
            return None;
        }
        Some(counters.PrivateUsage as u64)
    }
}

#[cfg(not(windows))]
pub fn per_pid_private_bytes(_pid: u32) -> Option<u64> {
    // Non-Windows hosts are outside this probe's reference scope
    // (contract-decision §8); the tree gate then rides the parent-only
    // sampler and the Windows-gated controls document the boundary.
    None
}

/// The aggregate result: the maximum SIMULTANEOUS sum over the whole
/// sampling window, plus the parent's solo peak for the summary split.
#[derive(Clone, Copy, Debug, Default)]
pub struct TreeSample {
    /// max over ticks of (parent private bytes + Σ registered children).
    pub tree_peak: u64,
    /// max over ticks of the parent's own private bytes.
    pub parent_peak: u64,
}

/// Polls `parent + Σ registered child pids` every `interval_ms` for its
/// lifetime. Registration is dynamic: the bench registers each worker
/// child right after spawn (before the request is even written) and
/// unregisters it after reaping, so overlapping lives are summed.
pub struct TreeSampler {
    peak: Arc<AtomicU64>,
    parent_peak: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    pids: Arc<Mutex<BTreeSet<u32>>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl TreeSampler {
    pub fn start(interval_ms: u64) -> TreeSampler {
        let peak = Arc::new(AtomicU64::new(0));
        let parent_peak = Arc::new(AtomicU64::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let pids = Arc::new(Mutex::new(BTreeSet::new()));
        let peak2 = Arc::clone(&peak);
        let parent2 = Arc::clone(&parent_peak);
        let stop2 = Arc::clone(&stop);
        let pids2 = Arc::clone(&pids);
        let handle = std::thread::Builder::new()
            .name("tree-mem-sampler".into())
            .spawn(move || {
                while !stop2.load(Ordering::Relaxed) {
                    let parent = auto_av_probe::mempeek::sample_private_bytes();
                    let mut sum = parent;
                    if let Ok(set) = pids2.lock() {
                        for pid in set.iter() {
                            if let Some(b) = per_pid_private_bytes(*pid) {
                                sum = sum.saturating_add(b);
                            }
                        }
                    }
                    peak2.fetch_max(sum, Ordering::Relaxed);
                    parent2.fetch_max(parent, Ordering::Relaxed);
                    std::thread::sleep(Duration::from_millis(interval_ms));
                }
            })
            .expect("tree sampler thread");
        TreeSampler {
            peak,
            parent_peak,
            stop,
            pids,
            handle: Some(handle),
        }
    }

    /// Include `pid` in the tree sum until unregistered (a dead pid
    /// simply reads as absent).
    pub fn register(&self, pid: u32) {
        if let Ok(mut set) = self.pids.lock() {
            set.insert(pid);
        }
    }

    pub fn unregister(&self, pid: u32) {
        if let Ok(mut set) = self.pids.lock() {
            set.remove(&pid);
        }
    }

    /// Stop polling and return the aggregate peaks.
    pub fn stop(mut self) -> TreeSample {
        self.stop.store(true, Ordering::Relaxed);
        let tree_peak = self.peak.load(Ordering::Relaxed);
        let parent_peak = self.parent_peak.load(Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        TreeSample {
            tree_peak,
            parent_peak,
        }
    }
}
