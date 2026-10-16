//! Request-span observations for the ac-av probe (PLAN-798 §5.5).
//!
//! One recorder per request. Spans are inclusive (parent spans enclose
//! children); the report exposes each span's own duration PLUS the request
//! wall, so nested spans are never double-counted into a fake "total".
//! Counters are producer-side pipeline facts (mir_build/abc_emit/verify/
//! load/av_execute/cache-hit-abc/cache-hit-mir) — cache-hit claims are
//! derived from what the driver actually skipped, never from asking a
//! producer whether it was called.

use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct SpanRec {
    pub name: String,
    pub parent: Option<String>,
    pub start: Duration,
    pub dur: Duration,
}

#[derive(Default)]
pub struct Obs {
    t0: Option<Instant>,
    spans: Vec<SpanRec>,
    counters: std::collections::BTreeMap<String, u64>,
    notes: Vec<String>,
}

impl Obs {
    pub fn new() -> Self {
        Obs {
            t0: Some(Instant::now()),
            ..Default::default()
        }
    }

    pub fn counter(&mut self, name: &str, delta: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += delta;
    }

    pub fn set_counter(&mut self, name: &str, value: u64) {
        self.counters.insert(name.to_string(), value);
    }

    pub fn note(&mut self, s: String) {
        self.notes.push(s);
    }

    /// Record a span measured by the given closure.
    pub fn span<T>(
        &mut self,
        name: &str,
        parent: Option<&str>,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        let started = Instant::now();
        let start_off = started - self.t0.expect("obs initialized");
        let out = f(self);
        let dur = started.elapsed();
        self.spans.push(SpanRec {
            name: name.to_string(),
            parent: parent.map(|s| s.to_string()),
            start: start_off,
            dur,
        });
        out
    }

    pub fn request_wall(&self) -> Duration {
        self.t0.expect("obs initialized").elapsed()
    }

    pub fn spans(&self) -> &[SpanRec] {
        &self.spans
    }

    pub fn counters(&self) -> &std::collections::BTreeMap<String, u64> {
        &self.counters
    }

    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    /// A duration for a span recorded across a manual begin/end pair (for
    /// code paths that cannot hold the closure form).
    pub fn manual_span(&mut self, name: &str, parent: Option<&str>, started: Instant) {
        let start_off = started - self.t0.expect("obs initialized");
        self.spans.push(SpanRec {
            name: name.to_string(),
            parent: parent.map(|s| s.to_string()),
            start: start_off,
            dur: started.elapsed(),
        });
    }

    pub fn now(&self) -> Instant {
        Instant::now()
    }
}

/// Percentile helper over sorted sample milliseconds.
pub fn percentile_ms(samples: &[f64], p: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut v: Vec<f64> = samples.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((p / 100.0) * (v.len() as f64 - 1.0)).round() as usize;
    v[idx.min(v.len() - 1)]
}

/// Per-scenario sample summary for the turnaround report.
#[derive(Clone, Debug, serde::Serialize)]
pub struct ScenarioSummary {
    pub scenario: String,
    pub samples: usize,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
    pub failures: usize,
}

pub fn summarize(scenario: &str, samples: &[f64]) -> ScenarioSummary {
    let failures = samples.iter().filter(|x| x.is_nan()).count();
    let ok: Vec<f64> = samples.iter().copied().filter(|x| !x.is_nan()).collect();
    ScenarioSummary {
        scenario: scenario.to_string(),
        samples: samples.len(),
        failures,
        p50_ms: percentile_ms(&ok, 50.0),
        p95_ms: percentile_ms(&ok, 95.0),
        p99_ms: percentile_ms(&ok, 99.0),
        min_ms: ok
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min)
            .min(f64::INFINITY),
        max_ms: ok.iter().cloned().fold(0.0, f64::max),
        mean_ms: if ok.is_empty() {
            0.0
        } else {
            ok.iter().sum::<f64>() / ok.len() as f64
        },
    }
}
