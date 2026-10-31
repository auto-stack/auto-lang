//! Peak private-bytes sampling (PLAN-798 §3/§5.5 r2, review F798-R2-02).
//!
//! Endpoint deltas cannot prove process peaks: a transient spike that is
//! freed before the end is invisible to `mem_end - mem_start`. This sampler
//! polls `PrivateUsage` on a background thread for the LIFETIME of a scope
//! (bench worker, one-request child, GC child) and reports the observed
//! maximum. It is evidence, not a guard: callers gate on the sampled peak.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct PeakSampler {
    peak: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl PeakSampler {
    /// Start polling `private_bytes` every `interval_ms`.
    pub fn start(interval_ms: u64) -> PeakSampler {
        let peak = Arc::new(AtomicU64::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let peak2 = Arc::clone(&peak);
        let stop2 = Arc::clone(&stop);
        let handle = std::thread::Builder::new()
            .name("mem-peak-sampler".into())
            .spawn(move || {
                while !stop2.load(Ordering::Relaxed) {
                    #[cfg(windows)]
                    {
                        let m = ace_artifacts::sysinfo::current_process_memory();
                        peak2.fetch_max(m.private_bytes, Ordering::Relaxed);
                    }
                    #[cfg(not(windows))]
                    {
                        peak2.fetch_max(0, Ordering::Relaxed);
                    }
                    std::thread::sleep(Duration::from_millis(interval_ms));
                }
            })
            .expect("sampler thread");
        PeakSampler {
            peak,
            stop,
            handle: Some(handle),
        }
    }

    /// Current observed peak (poll-quantized).
    pub fn current_peak(&self) -> u64 {
        self.peak.load(Ordering::Relaxed)
    }

    /// Stop sampling and return the observed peak.
    pub fn stop(mut self) -> u64 {
        self.stop.store(true, Ordering::Relaxed);
        let peak = self.peak.load(Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        peak
    }
}

/// One synchronous sample (used by short-lived children and tests).
pub fn sample_private_bytes() -> u64 {
    #[cfg(windows)]
    {
        ace_artifacts::sysinfo::current_process_memory().private_bytes
    }
    #[cfg(not(windows))]
    {
        0
    }
}

/// PLAN-820 F820-R1-02: PARENT-side peak sampling of ANOTHER process (the
/// data worker). The parent polls the child's `PrivateUsage` by PID for
/// the child's whole lifetime — end-state and self-reported numbers are
/// attribution, never the verdict. Poll-quantized by `interval_ms`.
pub struct ChildPeakSampler {
    peak: Arc<AtomicU64>,
    samples: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl ChildPeakSampler {
    /// Start polling `pid`'s private bytes every `interval_ms`. Returns
    /// `None` when the child cannot be opened (already exited or access
    /// denied) — callers treat that as "no evidence", never as zero.
    pub fn start(pid: u32, interval_ms: u64) -> Option<ChildPeakSampler> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Threading::{
                OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
            };
            // The handle travels to the polling thread as an integer
            // (plainly Send); cast back at each Win32 call.
            let raw = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) } as usize;
            if raw == 0 {
                return None;
            }
            // Take the original first poll synchronously: start returning must
            // not race an unscheduled sampler thread against a short child.
            let Some(first) = poll_child_private_bytes(raw as *mut core::ffi::c_void) else {
                unsafe { windows_sys::Win32::Foundation::CloseHandle(raw as _) };
                return None;
            };
            let peak = Arc::new(AtomicU64::new(first));
            let samples = Arc::new(AtomicU64::new(1));
            let stop = Arc::new(AtomicBool::new(false));
            let (peak2, samples2, stop2) =
                (Arc::clone(&peak), Arc::clone(&samples), Arc::clone(&stop));
            let thread = std::thread::Builder::new()
                .name("child-mem-peak-sampler".into())
                .spawn(move || {
                    let handle = raw as *mut core::ffi::c_void;
                    loop {
                        if !stop2.load(Ordering::Relaxed) {
                            if let Some(v) = poll_child_private_bytes(handle) {
                                peak2.fetch_max(v, Ordering::Relaxed);
                                samples2.fetch_add(1, Ordering::Relaxed);
                                std::thread::sleep(Duration::from_millis(interval_ms));
                                continue;
                            }
                        }
                        // Stopped, or the child exited / access lost: close
                        // the owned handle and end the thread.
                        unsafe { windows_sys::Win32::Foundation::CloseHandle(handle) };
                        return;
                    }
                })
                .expect("child sampler thread");
            Some(ChildPeakSampler {
                peak,
                samples,
                stop,
                handle: Some(thread),
            })
        }
        #[cfg(not(windows))]
        {
            let _ = pid;
            let _ = interval_ms;
            None
        }
    }

    /// Current observed peak (poll-quantized).
    pub fn current_peak(&self) -> u64 {
        self.peak.load(Ordering::Relaxed)
    }

    /// How many successful samples the parent took.
    pub fn samples(&self) -> u64 {
        self.samples.load(Ordering::Relaxed)
    }

    /// Stop sampling and return `(observed peak, sample count)`.
    pub fn stop(mut self) -> (u64, u64) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        (
            self.peak.load(Ordering::Relaxed),
            self.samples.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod sampling_controls {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn immediate_stop_keeps_the_first_real_process_sample() {
        let sampler =
            ChildPeakSampler::start(std::process::id(), 5).expect("own process is queryable");
        let (peak, count) = sampler.stop();
        assert!(peak > 0, "actual private bytes, never a synthesized zero");
        assert!(
            count >= 1,
            "a successful start already observed the process"
        );
        assert!(ChildPeakSampler::start(u32::MAX, 5).is_none());
    }

    #[test]
    fn shutdown_includes_a_poll_finishing_after_stop_was_requested() {
        let peak = Arc::new(AtomicU64::new(1));
        let samples = Arc::new(AtomicU64::new(1));
        let stop = Arc::new(AtomicBool::new(false));
        let (release, wait) = std::sync::mpsc::channel();
        let (p, n) = (Arc::clone(&peak), Arc::clone(&samples));
        let poll = std::thread::spawn(move || {
            wait.recv().expect("release final in-flight poll");
            p.store(4096, Ordering::Relaxed);
            n.fetch_add(1, Ordering::Relaxed);
        });
        let sampler = ChildPeakSampler {
            peak,
            samples,
            stop: Arc::clone(&stop),
            handle: Some(poll),
        };
        let finish = std::thread::spawn(move || sampler.stop());
        let limit = std::time::Instant::now() + Duration::from_secs(5);
        while !stop.load(Ordering::Relaxed) && std::time::Instant::now() < limit {
            std::thread::yield_now();
        }
        let requested = stop.load(Ordering::Relaxed);
        release.send(()).unwrap();
        assert_eq!(
            finish.join().unwrap(),
            (4096, 2),
            "include the completed final poll"
        );
        assert!(requested, "stop reached its controlled interleaving");
    }
}

/// One private-bytes sample of a foreign process handle; `None` when the
/// query fails (child gone / access lost).
#[cfg(windows)]
fn poll_child_private_bytes(handle: *mut core::ffi::c_void) -> Option<u64> {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX,
    };
    let mut counters: PROCESS_MEMORY_COUNTERS_EX = unsafe { std::mem::zeroed() };
    counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    let ok =
        unsafe { GetProcessMemoryInfo(handle, &mut counters as *mut _ as *mut _, counters.cb) };
    if ok == 0 {
        None
    } else {
        Some(counters.PrivateUsage as u64)
    }
}
