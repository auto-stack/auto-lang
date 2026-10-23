//! The `cteworker/1` protocol (contract-decision §9): one request over
//! stdin (all bytes to EOF), one JSON response line. The worker owns
//! `query`/`edit-source` semantics; `restart` is consumed PARENT-side
//! (a fresh worker process per restart chunk) and is refused here.

use crate::driver::{CteDriver, CteStepOutcome};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const CTE_WORKER_PROTOCOL: &str = "cteworker/1";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerRequest {
    pub recipe: String,
    pub nonce: String,
    #[serde(rename = "deck_b64")]
    pub deck_b64: String,
    #[serde(rename = "store_root")]
    pub store_root: Option<String>,
    #[serde(rename = "session")]
    pub session: Option<String>,
    pub steps: Vec<WorkerStep>,
    pub request_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum WorkerStep {
    Query,
    EditSource { from: String, to: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerStepOut {
    pub op: String,
    pub outcome: String,
    /// FNV-1a64 of the deck text this step executed (parent re-derives).
    pub source_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct_execute: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta_query: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_nodes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_types: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hir: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wall_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_private_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerOk {
    pub recipe: String,
    pub worker_version: String,
    pub engine: String,
    pub steps: Vec<WorkerStepOut>,
    /// The worker PROCESS's sampled private-bytes peak across the whole
    /// serve (the 384 MiB worker hard gate's input).
    #[serde(default)]
    pub worker_private_bytes_peak: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerResponse {
    pub schema: String,
    pub nonce: String,
    pub request_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ok: Option<WorkerOk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<WorkerError>,
}

/// Serve one full request (the case-serve entry used by the runner and
/// the workspace tests). `native_root` hosts per-request native build
/// dirs.
pub fn serve_request(req: &WorkerRequest, native_root: &Path) -> WorkerResponse {
    // Process-wide peak across the serve (1ms quantization).
    let sampler = auto_av_probe::mempeek::PeakSampler::start(1);
    let resp = |ok: Option<WorkerOk>, error: Option<WorkerError>| WorkerResponse {
        schema: CTE_WORKER_PROTOCOL.to_string(),
        nonce: req.nonce.clone(),
        request_hash: req.request_hash.clone(),
        ok,
        error,
    };
    use base64::Engine as _;
    let Ok(deck_bytes) = base64::engine::general_purpose::STANDARD.decode(&req.deck_b64) else {
        return resp(
            None,
            Some(WorkerError {
                code: "cte.worker.protocol".into(),
                message: "deck_b64 does not decode".into(),
            }),
        );
    };
    let Ok(mut deck) = String::from_utf8(deck_bytes) else {
        return resp(
            None,
            Some(WorkerError {
                code: "cte.worker.protocol".into(),
                message: "deck is not UTF-8".into(),
            }),
        );
    };
    let store_root = req.store_root.clone().unwrap_or_else(|| {
        std::env::temp_dir()
            .join("cte817-ephemeral")
            .display()
            .to_string()
    });
    let mut driver = match CteDriver::with_store(
        Path::new(&store_root),
        req.session.as_deref().unwrap_or("cte817"),
    ) {
        Ok(d) => d,
        Err(e) => {
            return resp(
                None,
                Some(WorkerError {
                    code: "cte.worker.store".into(),
                    message: e,
                }),
            )
        }
    };
    let mut steps_out: Vec<WorkerStepOut> = Vec::new();
    for (i, step) in req.steps.iter().enumerate() {
        match step {
            WorkerStep::Query => {
                let workdir = native_root.join(format!("step-{i}"));
                let out: CteStepOutcome = driver.query_step(&deck, Some(&workdir));
                steps_out.push(to_wire("query", &deck, out));
            }
            WorkerStep::EditSource { from, to } => {
                let hits = deck.matches(from.as_str()).count();
                if hits != 1 {
                    return resp(
                        None,
                        Some(WorkerError {
                            code: "cte.worker.edit".into(),
                            message: format!("step {i}: edit literal matched {hits} times"),
                        }),
                    );
                }
                deck = deck.replacen(from.as_str(), to.as_str(), 1);
                steps_out.push(WorkerStepOut {
                    op: "edit-source".into(),
                    outcome: "edited".into(),
                    source_hash: crate::fnv1a64_hex(deck.as_bytes()),
                    cache: None,
                    ct_execute: None,
                    meta_query: None,
                    query_nodes: None,
                    values: None,
                    consumer_types: None,
                    hir: None,
                    exit: None,
                    code: None,
                    message: None,
                    wall_ms: None,
                    peak_private_bytes: None,
                });
            }
        }
    }
    let peak = sampler.stop();
    resp(
        Some(WorkerOk {
            recipe: req.recipe.clone(),
            worker_version: crate::driver::probe_version().to_string(),
            engine: format!(
                "{}/{}/{}",
                auto_av_core::contract::AV_PROFILE,
                auto_av_core::contract::MIR_SCHEMA,
                auto_av_core::contract::ABC_SCHEMA
            ),
            steps: steps_out,
            worker_private_bytes_peak: peak,
        }),
        None,
    )
}

fn to_wire(op: &str, deck: &str, o: CteStepOutcome) -> WorkerStepOut {
    WorkerStepOut {
        op: op.to_string(),
        outcome: if o.status == crate::driver::StepStatus::Ok {
            "ok".into()
        } else {
            "diag".into()
        },
        source_hash: crate::fnv1a64_hex(deck.as_bytes()),
        cache: o.cache.map(|s| s.to_string()),
        ct_execute: Some(o.ct_execute),
        meta_query: Some(o.meta_query),
        query_nodes: Some(o.query_nodes),
        values: Some(
            o.values
                .iter()
                .map(|(k, v)| (k.clone(), v.render()))
                .collect(),
        ),
        consumer_types: Some(o.consumer_types.clone()),
        hir: Some(o.hir_proof.clone()),
        exit: o.native_exit,
        code: o.code,
        message: o.message,
        wall_ms: Some(o.wall_ms),
        peak_private_bytes: o.peak_private_bytes,
    }
}
