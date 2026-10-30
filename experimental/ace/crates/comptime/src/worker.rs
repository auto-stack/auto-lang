//! The `cteworker/1` protocol (contract-decision §9): one request over
//! stdin (all bytes to EOF), one JSON response line. The worker owns
//! `query`/`edit-source` semantics; `restart` is consumed PARENT-side
//! (a fresh worker process per restart chunk) and is refused here.

use crate::driver::{CteDriver, CteStepOutcome};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const CTE_WORKER_PROTOCOL: &str = "cteworker/1";

/// F-817-R6-02: the wire bound — one request carries at most a 4MiB
/// deck as base64 (4/3 expansion) plus the JSON envelope. Larger
/// inputs are refused BEFORE the read/parse allocates more than the
/// bound itself (the read is capped; nothing unbounded happens first).
pub const MAX_REQUEST_WIRE_BYTES: usize = 6 * 1024 * 1024;
/// Pre-decode bound on `deck_b64`: ceil(4MiB · 4/3) + slack. Checked
/// BEFORE the base64 decode allocates the deck bytes.
pub const MAX_DECK_B64_BYTES: usize =
    (ace_comptime_contract::MAX_REQUEST_INPUT_BYTES / 3 + 2) * 4 + 16;

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
    /// Language-class diag vs isolated resource/cancel/infra state —
    /// the runner's outcome admission reads this.
    #[serde(default)]
    pub language_class: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wall_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peak_private_bytes: Option<u64>,
    /// The managed-ledger gate verdict for this step (F-817-R4-01).
    #[serde(default)]
    pub managed: Option<bool>,
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
    /// The managed-budget gate verdict (all steps within the 32MiB
    /// ledger); false when any step over-reserved (F-817-R4-01).
    #[serde(default)]
    pub managed_target_met: bool,
    /// The worker PROCESS id (restart scenarios assert PIDs differ).
    #[serde(default)]
    pub worker_pid: u32,
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
pub fn serve_request(req: &WorkerRequest, _native_root: &Path) -> WorkerResponse {
    let mut driver = CteDriver::ephemeral();
    serve_request_with(req, &mut driver)
}

// ---- bench counter-control seams (F-817-R5-01/03) ----------------------
//
// All three are env-gated and read PER REQUEST (never cached): they are
// set only by the bench/turnaround controls that spawn this worker, so
// the production runner path (auto-case) never trips them. They exist
// so the FORMAL bench regression can prove the parent-side gates go red
// on a real child memory transient, a real managed overage, real
// sampling unavailability and real wire mutations — not on source-level
// stand-ins.

/// Hold a transient child allocation of `AUTO_CTE_BENCH_INJECT_ALLOC_MIB`
/// MiB for the request's lifetime (pages touched so the commit is real);
/// the child's own 1ms sampler folds it into
/// `worker_private_bytes_peak`.
fn bench_inject_hold() -> Option<Vec<u8>> {
    let mib: usize = std::env::var("AUTO_CTE_BENCH_INJECT_ALLOC_MIB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if mib == 0 {
        return None;
    }
    let mut buf = vec![0u8; mib * 1024 * 1024];
    // Touch every page: PrivateUsage counts commit, but touching keeps
    // the spike immune to lazy-zero accounting surprises.
    let last = buf.len() - 1;
    for off in (0..buf.len()).step_by(4096) {
        buf[off] = 1;
    }
    buf[last] = 1;
    Some(buf)
}

/// Shrink the managed staging partition to
/// `AUTO_CTE_BENCH_SHRINK_STAGING_KIB` KiB so the next request's
/// reservations spike the ledger (managed_target_met=false on the wire).
fn bench_shrink_managed(driver: &mut CteDriver) {
    if let Ok(v) = std::env::var("AUTO_CTE_BENCH_SHRINK_STAGING_KIB") {
        if let Ok(kib) = v.parse::<u64>() {
            driver.set_managed_caps(crate::observe::BudgetCaps {
                hot: 8 * 1024 * 1024,
                staging: kib * 1024,
                av_slots: 8 * 1024 * 1024,
            });
        }
    }
}

/// Mutate the outgoing response per `AUTO_CTE_BENCH_MUTATE` — a REAL
/// wire mutation (the bytes this process actually emits), so the
/// parent-side admission oracle is proven against actual bad responses,
/// not source-level comparisons.
fn apply_bench_mutation(resp: &mut WorkerResponse) {
    let Ok(mode) = std::env::var("AUTO_CTE_BENCH_MUTATE") else {
        return;
    };
    let Some(ok) = resp.ok.as_mut() else {
        return;
    };
    let last = ok.steps.len().saturating_sub(1);
    let first_query = ok
        .steps
        .iter()
        .position(|st| st.op == "query")
        .unwrap_or(last);
    match mode.as_str() {
        "wrong-value" => {
            if let Some(st) = ok.steps.get_mut(last) {
                if let Some(vals) = st.values.as_mut() {
                    for v in vals.values_mut() {
                        *v = "i32:0".into();
                    }
                }
            }
        }
        "wrong-hir" => {
            if let Some(st) = ok.steps.get_mut(last) {
                if let Some(hir) = st.hir.as_mut() {
                    for v in hir.values_mut() {
                        *v = "0".into();
                    }
                }
            }
        }
        "wrong-key" => {
            if let Some(st) = ok.steps.get_mut(last) {
                if let Some(vals) = st.values.as_mut() {
                    if let Some((k, v)) = vals.pop_first() {
                        vals.insert(format!("{k}x"), v);
                    }
                }
            }
        }
        "wrong-type" => {
            if let Some(st) = ok.steps.get_mut(last) {
                if let Some(ct) = st.consumer_types.as_mut() {
                    for v in ct.values_mut() {
                        *v = "bool".into();
                    }
                }
            }
        }
        "wrong-first-value" => {
            if let Some(st) = ok.steps.get_mut(first_query) {
                if let Some(vals) = st.values.as_mut() {
                    for v in vals.values_mut() {
                        *v = "i32:0".into();
                    }
                }
            }
        }
        "diag" => {
            if let Some(st) = ok.steps.get_mut(last) {
                st.outcome = "diag".into();
                st.language_class = true;
                st.code = Some("cte.bench.mutated".into());
                st.message = Some("bench mutation: diag inside a valid envelope".into());
            }
        }
        "no-hir" => {
            if let Some(st) = ok.steps.get_mut(last) {
                st.hir = None;
            }
        }
        "extra-step" => {
            if let Some(tail) = ok.steps.last().cloned() {
                ok.steps.push(tail);
            }
        }
        "missing-step" => {
            ok.steps.pop();
        }
        "bad-hash" => {
            if let Some(st) = ok.steps.get_mut(last) {
                st.source_hash = "00000000deadbeef".into();
            }
        }
        "omit-peak" => {
            ok.worker_private_bytes_peak = 0;
        }
        _ => {}
    }
}

/// The driver-sharing core: same protocol, caller-owned driver (the
/// stream mode keeps ONE driver per store root so the resident snapshot
/// lane and store handle stay warm across requests).
pub fn serve_request_with(req: &WorkerRequest, driver: &mut CteDriver) -> WorkerResponse {
    // Held across the whole serve (F-817-R5-01 child-spike control).
    let _hold = bench_inject_hold();
    // Process-wide peak across the serve (1ms quantization).
    let sampler = ace_av::mempeek::PeakSampler::start(1);
    let resp = |ok: Option<WorkerOk>, error: Option<WorkerError>| WorkerResponse {
        schema: CTE_WORKER_PROTOCOL.to_string(),
        nonce: req.nonce.clone(),
        request_hash: req.request_hash.clone(),
        ok,
        error,
    };
    use base64::Engine as _;
    // F-817-R6-02: size pre-check BEFORE the base64 decode allocates
    // the deck bytes (the read itself is wire-capped at the bin).
    if req.deck_b64.len() > MAX_DECK_B64_BYTES {
        return resp(
            None,
            Some(WorkerError {
                code: "cte.worker.protocol".into(),
                message: format!(
                    "deck_b64 {}B exceeds the {}B bound (the deck input is capped at {}B)",
                    req.deck_b64.len(),
                    MAX_DECK_B64_BYTES,
                    ace_comptime_contract::MAX_REQUEST_INPUT_BYTES
                ),
            }),
        );
    }
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
    if driver.store().is_none() {
        if let Err(e) = driver.attach_store(
            Path::new(&store_root),
            req.session.as_deref().unwrap_or("cte817"),
        ) {
            return resp(
                None,
                Some(WorkerError {
                    code: "cte.worker.store".into(),
                    message: e,
                }),
            );
        }
    }
    bench_shrink_managed(driver);
    let mut steps_out: Vec<WorkerStepOut> = Vec::new();
    for (i, step) in req.steps.iter().enumerate() {
        match step {
            WorkerStep::Query => {
                let workdir = std::env::temp_dir()
                    .join(format!("cte817-worker-{}", std::process::id()))
                    .join(format!("step-{i}"));
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
                    language_class: false,
                    message: None,
                    wall_ms: None,
                    peak_private_bytes: None,
                    managed: None,
                });
            }
        }
    }
    let peak = sampler.stop();
    let managed_ok = steps_out
        .iter()
        .filter(|st| st.managed.is_some())
        .all(|st| st.managed == Some(true));
    let mut out = resp(
        Some(WorkerOk {
            recipe: req.recipe.clone(),
            worker_version: crate::driver::probe_version().to_string(),
            engine: format!(
                "{}/{}/{}",
                ace_av_core::contract::AV_PROFILE,
                ace_av_core::contract::MIR_SCHEMA,
                ace_av_core::contract::ABC_SCHEMA
            ),
            steps: steps_out,
            worker_private_bytes_peak: peak,
            managed_target_met: managed_ok,
            worker_pid: std::process::id(),
        }),
        None,
    );
    apply_bench_mutation(&mut out);
    out
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
        language_class: o.language_class,
        message: o.message,
        wall_ms: Some(o.wall_ms),
        peak_private_bytes: o.peak_private_bytes,
        managed: Some(o.managed_target_met),
    }
}

/// `case-serve-stream`: the WARM-SESSION serve mode (F-817-R3-03). One
/// process, one resident driver per store root; JSON-line requests on
/// stdin until EOF, one JSON response line each. The first request warms
/// (cold start amortized); subsequent edit requests measure the warm
/// window through real IPC with the driver (and its resident snapshot
/// lane) alive across requests.
pub fn serve_stream(stdin: &mut dyn std::io::BufRead, stdout: &mut dyn std::io::Write) {
    use std::collections::BTreeMap;
    use std::io::BufRead as _;
    let mut drivers: BTreeMap<String, CteDriver> = BTreeMap::new();
    let mut line = String::new();
    loop {
        line.clear();
        // F-817-R6-02: the wire read is BOUNDED — a request line larger
        // than the wire bound is refused without allocating past it.
        let mut limited = std::io::Read::take(&mut *stdin, (MAX_REQUEST_WIRE_BYTES + 1) as u64);
        if limited.read_line(&mut line).unwrap_or(0) == 0 {
            break;
        }
        if line.len() > MAX_REQUEST_WIRE_BYTES {
            let resp = WorkerResponse {
                schema: CTE_WORKER_PROTOCOL.to_string(),
                nonce: String::new(),
                request_hash: String::new(),
                ok: None,
                error: Some(WorkerError {
                    code: "cte.worker.protocol".into(),
                    message: format!(
                        "stream request {}B exceeds the {}B wire bound",
                        line.len(),
                        MAX_REQUEST_WIRE_BYTES
                    ),
                }),
            };
            let _ = writeln!(
                stdout,
                "{}",
                serde_json::to_string(&resp).unwrap_or_default()
            );
            let _ = stdout.flush();
            continue;
        }
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let Ok(req) = serde_json::from_str::<WorkerRequest>(t) else {
            let resp = WorkerResponse {
                schema: CTE_WORKER_PROTOCOL.to_string(),
                nonce: String::new(),
                request_hash: String::new(),
                ok: None,
                error: Some(WorkerError {
                    code: "cte.worker.protocol".into(),
                    message: "stream request does not decode".into(),
                }),
            };
            let _ = writeln!(
                stdout,
                "{}",
                serde_json::to_string(&resp).unwrap_or_default()
            );
            let _ = stdout.flush();
            continue;
        };
        let root = req
            .store_root
            .clone()
            .unwrap_or_else(|| "ephemeral".to_string());
        let key = format!("{}|{}", root, req.session.clone().unwrap_or_default());
        let driver = drivers.entry(key).or_insert_with(|| CteDriver::ephemeral());
        let resp = serve_request_with(&req, driver);
        let _ = writeln!(
            stdout,
            "{}",
            serde_json::to_string(&resp).unwrap_or_default()
        );
        let _ = stdout.flush();
    }
}
