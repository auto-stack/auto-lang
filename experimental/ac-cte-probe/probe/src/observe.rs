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
