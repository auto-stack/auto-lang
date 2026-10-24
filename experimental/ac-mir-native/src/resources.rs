//! Resource evidence of the MIR-native backend (contract-decision §7.2/§11,
//! PLAN-818 T-05): the managed-data ledger (per-partition real-time charge
//! peaks), the request-tree peak sampler (own private bytes + direct child
//! processes, ≥5 ms polling — the 798 mempeek form extended to the tree),
//! and the raw-sample gate (zero/missing/NaN/failed samples can never pass
//! as 0ms/0bytes; a degenerate raw makes the aggregate gate fire).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Managed-data partitions (contract-decision §11). Index into the ledger
/// arrays; the caps mirror the frozen shares of the ≤48 MiB budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Partition {
    /// Restored MIR units, symbol tables, closure indexes — 16 MiB.
    MirSymbol = 0,
    /// Lowering/index/codec staging: emitted object bytes in flight,
    /// envelope assembly, manifest renders — 24 MiB.
    Staging = 1,
    /// Object/manifest read+write buffers, receipts — 8 MiB.
    Io = 2,
}

pub const PARTITION_NAMES: [&str; 3] = ["mir-symbol", "staging", "io"];
pub const PARTITION_CAPS: [u64; 3] = [16 << 20, 24 << 20, 8 << 20];
pub const MANAGED_TOTAL_CAP: u64 = 48 << 20;

/// Charge ledger: `charge`/`discharge` track the actual byte sizes of the
/// named in-flight values; peaks are the observed concurrent maxima. A
/// violation is `mn.resource.managed-memory` evidence, not a guard — the
/// callers (driver assertions, tier tests) gate on it.
#[derive(Default, Debug, Clone)]
pub struct ManagedLedger {
    current: [u64; 3],
    peak: [u64; 3],
    peak_total: u64,
    charges: u64,
    discharges: u64,
}

impl ManagedLedger {
    pub fn charge(&mut self, part: Partition, bytes: u64) {
        let i = part as usize;
        self.current[i] = self.current[i].saturating_add(bytes);
        self.peak[i] = self.peak[i].max(self.current[i]);
        self.peak_total = self
            .peak_total
            .max(self.current.iter().sum::<u64>());
        self.charges += 1;
    }

    pub fn discharge(&mut self, part: Partition, bytes: u64) {
        let i = part as usize;
        self.current[i] = self.current[i].saturating_sub(bytes);
        self.discharges += 1;
    }

    /// One transient staging window: charge, run, discharge.
    pub fn with_charge<R>(
        &mut self,
        part: Partition,
        bytes: u64,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.charge(part, bytes);
        let out = f(self);
        self.discharge(part, bytes);
        out
    }

    pub fn peaks(&self) -> [u64; 3] {
        self.peak
    }

    pub fn peak_total(&self) -> u64 {
        self.peak_total
    }

    /// Cap violations as `mn.resource.managed-memory` evidence lines
    /// (empty = within every frozen share and the ≤48 MiB total).
    pub fn violations(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (i, name) in PARTITION_NAMES.iter().enumerate() {
            if self.peak[i] > PARTITION_CAPS[i] {
                out.push(format!(
                    "mn.resource.managed-memory: partition `{name}` peak {} > cap {}",
                    self.peak[i],
                    PARTITION_CAPS[i]
                ));
            }
        }
        if self.peak_total > MANAGED_TOTAL_CAP {
            out.push(format!(
                "mn.resource.managed-memory: total peak {} > cap {}",
                self.peak_total, MANAGED_TOTAL_CAP
            ));
        }
        out
    }

    pub fn render(&self) -> String {
        let parts: Vec<String> = PARTITION_NAMES
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{n}={}", self.peak[i]))
            .collect();
        format!(
            "ledger[{}] total_peak={} charges={} discharges={}",
            parts.join(" "),
            self.peak_total,
            self.charges,
            self.discharges
        )
    }
}

/// Observed peaks of one sampling window: the own-process private-bytes
/// peak, the request-TREE peak (own + direct children summed per tick;
/// link/PE children are exactly that), and sampler health counters.
#[derive(Clone, Copy, Debug, Default)]
pub struct TreePeak {
    pub own_peak: u64,
    pub tree_peak: u64,
    pub ticks: u64,
    /// Ticks that observed ≥1 live direct child (0 everywhere means the
    /// child side of the sampler never actually exercised).
    pub child_ticks: u64,
}

/// Background peak sampler (≥5 ms polling, 798 mempeek form extended with
/// per-tick direct-child enumeration — endpoint deltas cannot see a spike
/// that was freed before the window closed).
pub struct TreePeakSampler {
    own_peak: Arc<AtomicU64>,
    tree_peak: Arc<AtomicU64>,
    ticks: Arc<AtomicU64>,
    child_ticks: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl TreePeakSampler {
    pub fn start(interval_ms: u64) -> TreePeakSampler {
        let own_peak = Arc::new(AtomicU64::new(0));
        let tree_peak = Arc::new(AtomicU64::new(0));
        let ticks = Arc::new(AtomicU64::new(0));
        let child_ticks = Arc::new(AtomicU64::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let (o, t, k, c, s) = (
            Arc::clone(&own_peak),
            Arc::clone(&tree_peak),
            Arc::clone(&ticks),
            Arc::clone(&child_ticks),
            Arc::clone(&stop),
        );
        let handle = std::thread::Builder::new()
            .name("mn-tree-peak-sampler".into())
            .spawn(move || {
                while !s.load(Ordering::Relaxed) {
                    let own = auto_av_probe::mempeek::sample_private_bytes();
                    let children = sum_direct_children_private_bytes();
                    o.fetch_max(own, Ordering::Relaxed);
                    t.fetch_max(own.saturating_add(children), Ordering::Relaxed);
                    k.fetch_add(1, Ordering::Relaxed);
                    if children > 0 {
                        c.fetch_add(1, Ordering::Relaxed);
                    }
                    std::thread::sleep(std::time::Duration::from_millis(interval_ms));
                }
            })
            .expect("sampler thread");
        TreePeakSampler {
            own_peak,
            tree_peak,
            ticks,
            child_ticks,
            stop,
            handle: Some(handle),
        }
    }

    pub fn stop(mut self) -> TreePeak {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        TreePeak {
            own_peak: self.own_peak.load(Ordering::Relaxed),
            tree_peak: self.tree_peak.load(Ordering::Relaxed),
            ticks: self.ticks.load(Ordering::Relaxed),
            child_ticks: self.child_ticks.load(Ordering::Relaxed),
        }
    }
}

/// Sum of private bytes over the CURRENT process's direct children (the
/// link/PE subtree of a request — the contained executor's children are
/// exactly that). 0 where the toolhelp snapshot is unavailable.
#[cfg(windows)]
pub fn sum_direct_children_private_bytes() -> u64 {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };

    let me = std::process::id();
    let snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snap == INVALID_HANDLE_VALUE {
        return 0;
    }
    let mut total: u64 = 0;
    unsafe {
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        if Process32FirstW(snap, &mut entry) != 0 {
            loop {
                if entry.th32ParentProcessID == me {
                    let child = OpenProcess(
                        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
                        0,
                        entry.th32ProcessID,
                    );
                    if !child.is_null() {
                        let mut counters: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
                        counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
                        if GetProcessMemoryInfo(child, &mut counters as *mut _ as *mut _, counters.cb) != 0 {
                            total += counters.PrivateUsage as u64;
                        }
                        CloseHandle(child);
                    }
                }
                if Process32NextW(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    total
}

#[cfg(not(windows))]
pub fn sum_direct_children_private_bytes() -> u64 {
    0
}

/// One measured raw sample of a gate dimension (time and a bytes
/// magnitude). `ok=false` marks a FAILED sample — a failure is raw, not a
/// zero-value pass.
#[derive(Clone, Debug)]
pub struct RawSample {
    pub label: String,
    pub ms: f64,
    pub bytes: u64,
    pub ok: bool,
}

impl RawSample {
    pub fn measured(label: &str, ms: f64, bytes: u64) -> RawSample {
        RawSample {
            label: label.to_string(),
            ms,
            bytes,
            ok: true,
        }
    }
}

/// Aggregate raw gate (contract §7.2 T-05/T-07 obligation): zero/missing/
/// NaN/failed samples must NOT default-pass as 0ms/0bytes. Returns the
/// violation list — EMPTY means the raw is admissible; a degenerate raw
/// (all zeros, NaNs, failures, or no samples at all) keeps it non-empty,
/// so a threshold of zero can never silently green the aggregate.
pub fn raw_gate(samples: &[RawSample]) -> Vec<String> {
    if samples.is_empty() {
        return vec!["raw gate: no samples (missing raw)".to_string()];
    }
    let mut out = Vec::new();
    for s in samples {
        if !s.ok {
            out.push(format!("`{}`: failed sample", s.label));
        } else if !s.ms.is_finite() {
            out.push(format!("`{}`: NaN/missing time", s.label));
        } else if s.ms <= 0.0 {
            out.push(format!("`{}`: zero time sample", s.label));
        } else if s.bytes == 0 {
            out.push(format!("`{}`: zero bytes sample", s.label));
        }
    }
    out
}
