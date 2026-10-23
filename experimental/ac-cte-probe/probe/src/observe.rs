//! Per-step observation ledger: named spans (wall each), counters and
//! the sampled private-bytes peak. The turnaround tier (T-07) reads the
//! same spans; nothing here decides verdicts.

use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Default, Clone, Debug)]
pub struct Obs {
    spans: BTreeMap<String, f64>,
    counters: BTreeMap<String, u64>,
    started: Option<Instant>,
}

impl Obs {
    pub fn new() -> Self {
        Obs {
            started: Some(Instant::now()),
            ..Default::default()
        }
    }
    /// Record a manually-timed span (for paths that cannot lend the
    /// closure (borrow conflicts)).
    pub fn record_span(&mut self, name: &str, ms: f64) {
        *self.spans.entry(name.to_string()).or_insert(0.0) += ms;
    }

    pub fn span<T>(&mut self, name: &str, f: impl FnOnce(&mut Self) -> T) -> T {
        let t = Instant::now();
        let out = f(self);
        let ms = t.elapsed().as_secs_f64() * 1000.0;
        *self.spans.entry(name.to_string()).or_insert(0.0) += ms;
        out
    }
    pub fn counter(&mut self, name: &str, add: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += add;
    }
    pub fn set_counter(&mut self, name: &str, v: u64) {
        self.counters.insert(name.to_string(), v);
    }
    pub fn counters(&self) -> &BTreeMap<String, u64> {
        &self.counters
    }
    pub fn spans(&self) -> &BTreeMap<String, f64> {
        &self.spans
    }
    pub fn request_wall_ms(&self) -> f64 {
        self.started
            .map(|t| t.elapsed().as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }
}

// ---------------------------------------------------------------------------
// managed budget ledger (contract-decision §7)
// ---------------------------------------------------------------------------

/// The three managed-memory partitions, in bytes: hot (retained values /
/// loaded envelopes), staging (request/encode/decode/index temporaries)
/// and AV slots (the D-798 execution partition). The frozen total cap is
/// 32 MiB (8 + 16 + 8).
#[derive(Clone, Copy, Debug)]
pub struct BudgetCaps {
    pub hot: u64,
    pub staging: u64,
    pub av_slots: u64,
}

impl Default for BudgetCaps {
    fn default() -> Self {
        BudgetCaps {
            hot: 8 * 1024 * 1024,
            staging: 16 * 1024 * 1024,
            av_slots: 8 * 1024 * 1024,
        }
    }
}

impl BudgetCaps {
    pub fn total(self) -> u64 {
        self.hot + self.staging + self.av_slots
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Partition {
    Hot,
    Staging,
    AvSlots,
}

/// One observable ledger event (F-817-R6-02): every reservation (with
/// its verdict) and every gated large allocation, in ORDER — the
/// reservation-before-allocation invariant is testable from the log,
/// not just from reading the code.
#[derive(Clone, Debug, PartialEq)]
pub enum BudgetEvent {
    Reserve {
        partition: Partition,
        bytes: u64,
        what: String,
        ok: bool,
    },
    /// A budget-gated allocation that ACTUALLY happened (recorded right
    /// after the allocation, naming what it was).
    Alloc { tag: &'static str, bytes: u64 },
    /// Bytes returned when a temporary died (replaying Reserve+Release
    /// yields the LIVE level at any point).
    Release { partition: Partition, bytes: u64 },
}

/// The per-request managed ledger: reservations happen BEFORE the
/// corresponding clone/decode/encode; releases return bytes when a
/// temporary dies; the peak of the LIVE total and any rejection feed
/// `managed_target_met`.
#[derive(Clone, Debug)]
pub struct ManagedBudget {
    caps: BudgetCaps,
    now: [u64; 3],
    peak_live_total: u64,
    rejected: bool,
    /// Ordered reservation/allocation events (capped length — a
    /// diagnostic log, not an unbounded trace).
    events: Vec<BudgetEvent>,
}

const MAX_BUDGET_EVENTS: usize = 256;

impl Default for ManagedBudget {
    fn default() -> Self {
        ManagedBudget::new(BudgetCaps::default())
    }
}

impl ManagedBudget {
    pub fn new(caps: BudgetCaps) -> Self {
        ManagedBudget {
            caps,
            now: [0; 3],
            peak_live_total: 0,
            rejected: false,
            events: Vec::new(),
        }
    }

    fn idx(p: Partition) -> usize {
        match p {
            Partition::Hot => 0,
            Partition::Staging => 1,
            Partition::AvSlots => 2,
        }
    }

    fn push_event(&mut self, e: BudgetEvent) {
        if self.events.len() < MAX_BUDGET_EVENTS {
            self.events.push(e);
        }
    }

    fn touch_peak(&mut self) {
        let total: u64 = self.now.iter().sum();
        if total > self.peak_live_total {
            self.peak_live_total = total;
        }
    }

    /// Reserve BEFORE the allocation this covers. Over-cap is a resource
    /// rejection (recorded — the step can never report target_met).
    pub fn reserve(&mut self, p: Partition, bytes: u64, what: &str) -> Result<(), String> {
        let i = Self::idx(p);
        let cap = match p {
            Partition::Hot => self.caps.hot,
            Partition::Staging => self.caps.staging,
            Partition::AvSlots => self.caps.av_slots,
        };
        if self.now[i].saturating_add(bytes) > cap {
            self.rejected = true;
            self.push_event(BudgetEvent::Reserve {
                partition: p,
                bytes,
                what: what.to_string(),
                ok: false,
            });
            return Err(format!(
                "managed {what} would push the {:?} partition to {} bytes over the {} byte cap",
                p,
                self.now[i].saturating_add(bytes),
                cap
            ));
        }
        self.now[i] += bytes;
        self.touch_peak();
        self.push_event(BudgetEvent::Reserve {
            partition: p,
            bytes,
            what: what.to_string(),
            ok: true,
        });
        Ok(())
    }

    /// Record a budget-gated allocation that actually happened (right
    /// AFTER the allocation — the preceding Reserve event is the gate).
    pub fn record_alloc(&mut self, tag: &'static str, bytes: u64) {
        self.push_event(BudgetEvent::Alloc { tag, bytes });
    }

    /// The ordered event log (reservation/allocation order evidence).
    pub fn events(&self) -> &[BudgetEvent] {
        &self.events
    }

    /// Return bytes when a temporary dies (staging scratch, a decoded
    /// envelope that was consumed, ...).
    pub fn release(&mut self, p: Partition, bytes: u64) {
        let i = Self::idx(p);
        self.now[i] = self.now[i].saturating_sub(bytes);
        self.push_event(BudgetEvent::Release {
            partition: p,
            bytes,
        });
    }

    /// Record a FINAL, step-lifetime occupancy (e.g. the AV slot peak
    /// from the execution trace — already capped by D-798).
    pub fn settle(&mut self, p: Partition, bytes: u64) {
        let i = Self::idx(p);
        if bytes > self.now[i] {
            self.now[i] = bytes;
        }
        self.touch_peak();
    }

    pub fn peak_live_total(&self) -> u64 {
        self.peak_live_total
    }

    /// The hard gate: over-budget (or over-cap peak) steps NEVER report
    /// target met.
    pub fn target_met(&self) -> bool {
        !self.rejected && self.peak_live_total <= self.caps.total()
    }
}
