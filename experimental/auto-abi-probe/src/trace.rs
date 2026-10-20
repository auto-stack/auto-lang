//! Per-request stage trace (plan §5.5): every probe request records its
//! stage spans — descriptor validation, layout/cache lookup, emit or C
//! compile, link, run, oracle compare — plus cache hit/miss and byte
//! counts. Traces serialize to JSONL under the case scratch dir; raw
//! samples are never deleted and failure requests keep their traces.

use std::time::Instant;

#[derive(Clone, Debug, serde::Serialize)]
pub struct StageEvent {
    pub stage: &'static str,
    pub dur_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct RequestTrace {
    pub scenario: &'static str,
    pub iteration: u32,
    pub ok: bool,
    pub total_ms: f64,
    pub stages: Vec<StageEvent>,
}

/// Collector for one probe request.
pub struct Trace {
    scenario: &'static str,
    iteration: u32,
    started: Instant,
    stages: Vec<StageEvent>,
}

impl Trace {
    pub fn new(scenario: &'static str, iteration: u32) -> Trace {
        Trace {
            scenario,
            iteration,
            started: Instant::now(),
            stages: Vec::new(),
        }
    }

    /// Record one stage span.
    pub fn stage(&mut self, name: &'static str, f: impl FnOnce() -> StageMeta) {
        let t0 = Instant::now();
        let meta = f();
        self.stages.push(StageEvent {
            stage: name,
            dur_ms: t0.elapsed().as_secs_f64() * 1000.0,
            hit: meta.hit,
            bytes: meta.bytes,
            note: meta.note,
        });
    }

    pub fn finish(mut self, ok: bool) -> RequestTrace {
        RequestTrace {
            scenario: self.scenario,
            iteration: self.iteration,
            ok,
            total_ms: self.started.elapsed().as_secs_f64() * 1000.0,
            stages: std::mem::take(&mut self.stages),
        }
    }
}

/// Optional metadata a stage reports.
#[derive(Default)]
pub struct StageMeta {
    pub hit: Option<bool>,
    pub bytes: Option<u64>,
    pub note: Option<String>,
}

impl StageMeta {
    pub fn hit(b: bool) -> StageMeta {
        StageMeta {
            hit: Some(b),
            ..Default::default()
        }
    }
    pub fn bytes(n: u64) -> StageMeta {
        StageMeta {
            bytes: Some(n),
            ..Default::default()
        }
    }
}

/// Append one JSONL line to the scenario's raw-sample file. Failures
/// keep their lines (nothing is deleted on red).
pub fn append_jsonl(dir: &std::path::Path, t: &RequestTrace) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(format!("turnaround-{}.jsonl", t.scenario)))?;
    serde_json::to_writer(&mut f, t)?;
    f.write_all(b"\n")
}

/// P50/P95/P99 of a sample set (plan §5.5 keeps all three percentiles).
pub fn percentiles(mut xs: Vec<f64>) -> (f64, f64, f64) {
    if xs.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let at = |q: f64| -> f64 {
        let idx = ((xs.len() as f64 - 1.0) * q).round() as usize;
        xs[idx.min(xs.len() - 1)]
    };
    (at(0.50), at(0.95), at(0.99))
}
