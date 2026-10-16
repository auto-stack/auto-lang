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
                        let m = ac_artifact_probe::sysinfo::current_process_memory();
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
        ac_artifact_probe::sysinfo::current_process_memory().private_bytes
    }
    #[cfg(not(windows))]
    {
        0
    }
}
