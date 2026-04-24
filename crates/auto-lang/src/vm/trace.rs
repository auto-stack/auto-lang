//! Plan 199 Phase 5: Structured execution trace for AI Agent analysis
//!
//! TraceCollector records each opcode execution as a structured record,
//! outputting JSON/JSONL for AI Agent consumption.

use serde::Serialize;

/// A single execution step record
#[derive(Debug, Clone, Serialize)]
pub struct TraceRecord {
    pub step: u64,
    pub ip: usize,
    pub op: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    pub stack_height: usize,
    pub call_depth: usize,
}

/// Collects execution trace records
pub struct TraceCollector {
    records: Vec<TraceRecord>,
    step: u64,
    max_records: usize,
    enabled: bool,
}

impl TraceCollector {
    pub fn new(max_records: usize) -> Self {
        Self {
            records: Vec::new(),
            step: 0,
            max_records,
            enabled: true,
        }
    }

    /// Record an execution step
    pub fn record(
        &mut self,
        ip: usize,
        op: &str,
        line: u32,
        stack_height: usize,
        call_depth: usize,
    ) {
        if !self.enabled {
            return;
        }
        if self.max_records > 0 && self.records.len() >= self.max_records {
            self.enabled = false;
            return;
        }

        self.step += 1;
        self.records.push(TraceRecord {
            step: self.step,
            ip,
            op: op.to_string(),
            line: if line > 0 { Some(line) } else { None },
            stack_height,
            call_depth,
        });
    }

    /// Output all records as JSON array
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.records).unwrap_or_else(|_| "[]".to_string())
    }

    /// Output records as JSONL (one JSON object per line)
    pub fn to_jsonl(&self) -> String {
        self.records
            .iter()
            .filter_map(|r| serde_json::to_string(r).ok())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn records(&self) -> &[TraceRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.step = 0;
        self.enabled = true;
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}
