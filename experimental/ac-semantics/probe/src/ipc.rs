//! `semaworker/1` — the sema probe worker IPC mode (PLAN-796 T-06).
//!
//! Same discipline as 788's `acworker/1`: a bounded single-shot
//! JSON-lines session (begin → request → end) over stdin/stdout; the
//! parent owns the wall deadline and kills the process on expiry; the
//! response echoes `nonce` + `request_hash` for parent-side admission;
//! a `restart` step ENDS the session (the parent continues the
//! remaining steps in a FRESH process against the same store root —
//! the real cross-process persistence path).

use ac_sema_core::query::QueryKind;
use ac_sema_core::{InferOutcome, SemaInput};
use crate::{lookup_only, open_store, publish_infer_result, run_inference, FetchOutcome};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

pub const SEMAWORKER_SCHEMA: &str = "semaworker/1";
/// Recipe identity this worker implements.
pub const RECIPE: &str = "sema/query-v1";

/// Bounded read of one stdin line (the request is source + steps; a
/// hostile caller cannot wedge the worker on an unbounded line).
const MAX_LINE: usize = 8 << 20;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionBegin {
    pub op: String,
    pub schema: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerRequest {
    pub recipe: String,
    pub nonce: String,
    pub request_hash: String,
    pub source_b64: String,
    pub package: String,
    pub module: String,
    pub fn_name: String,
    pub store_root: String,
    pub steps: Vec<StepIn>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StepIn {
    pub op: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StepOut {
    pub op: &'static str,
    pub cache: String,
    pub compute_calls: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<BindingOut>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<DiagOut>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BindingOut {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagOut {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OkOut {
    pub recipe: &'static str,
    pub steps: Vec<StepOut>,
    /// true when the step list contained a `restart` at the cut point —
    /// the parent must continue the remaining steps in a fresh process.
    pub restart_pending: bool,
    pub worker_version: &'static str,
    pub sema_core_version: &'static str,
}

/// The wire envelope (same shape discipline as acworker/1): identity
/// fields at the TOP level, payload under `ok`.
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseOk {
    pub schema: &'static str,
    pub nonce: String,
    pub request_hash: String,
    pub ok: OkOut,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseErr {
    pub schema: &'static str,
    pub nonce: String,
    pub request_hash: String,
    pub error: WorkerError,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerError {
    pub code: String,
    pub message: String,
}

pub const WORKER_VERSION: &str = concat!("ac-sema-probe/", env!("CARGO_PKG_VERSION"));

/// Serve one session from stdin; exactly one response line on stdout;
/// exit 0. Any malformed input is a controlled error response (never a
/// panic, never a hang).
pub fn serve() -> i32 {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    let mut line_buf = String::new();

    // begin
    let begin: SessionBegin = match next_json(&mut lines, &mut line_buf) {
        Ok(Some(v)) => v,
        _ => return fail_protocol("begin line missing or malformed"),
    };
    if begin.op != "begin" || begin.schema != SEMAWORKER_SCHEMA {
        return fail_protocol("bad begin (expected op=begin, schema=semaworker/1)");
    }
    // request — the envelope shape mirrors acworker/1: {"op":"request","request":{…}}
    let req_owned: String = match next_raw(&mut lines, &mut line_buf) {
        Some(l) => l.to_string(),
        None => return fail_protocol("request line missing"),
    };
    #[derive(Deserialize)]
    struct Envelope {
        op: String,
        request: WorkerRequest,
    }
    let env: Envelope = match serde_json::from_str(&req_owned) {
        Ok(e) => e,
        Err(e) => return fail_protocol(&format!("request malformed: {e}")),
    };
    if env.op != "request" {
        return fail_protocol("expected op=request");
    }
    let req = env.request;
    // end
    let mut end_buf = String::new();
    match next_raw(&mut lines, &mut end_buf) {
        Some(l) if l.contains(r#""op":"end""#) || l.contains(r#""op": "end""#) => {}
        _ => return fail_protocol("end line missing"),
    }

    let resp = execute(&req);
    let mut out = std::io::stdout().lock();
    let line = serde_json::to_string(&resp).unwrap_or_default();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
    0
}

fn fail_protocol(msg: &str) -> i32 {
    let resp = ResponseErr {
        schema: SEMAWORKER_SCHEMA,
        nonce: String::new(),
        request_hash: String::new(),
        error: WorkerError { code: "protocol".into(), message: msg.into() },
    };
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{}", serde_json::to_string(&resp).unwrap_or_default());
    let _ = out.flush();
    0
}

fn next_raw<'a>(
    lines: &mut impl Iterator<Item = Result<String, std::io::Error>>,
    buf: &'a mut String,
) -> Option<&'a str> {
    buf.clear();
    for _ in 0..64 {
        // bounded accumulate across physical lines (a JSON line is one
        // physical line in this protocol; the cap guards runaway input)
        match lines.next() {
            Some(Ok(l)) => {
                if buf.len() + l.len() > MAX_LINE {
                    return None;
                }
                buf.push_str(&l);
                if !buf.is_empty() {
                    return Some(buf);
                }
            }
            _ => return None,
        }
    }
    None
}

fn next_json<T: serde::de::DeserializeOwned>(
    lines: &mut impl Iterator<Item = Result<String, std::io::Error>>,
    buf: &mut String,
) -> Result<Option<T>, ()> {
    match next_raw(lines, buf) {
        Some(raw) => serde_json::from_str(raw).map(Some).map_err(|_| ()),
        None => Err(()),
    }
}

/// Execute steps of THIS session (up to and excluding the next
/// `restart`), mirroring the probe's real query flow per step.
fn execute(req: &WorkerRequest) -> serde_json::Value {
    if req.recipe != RECIPE {
        return serde_json::to_value(ResponseErr {
            schema: SEMAWORKER_SCHEMA,
            nonce: req.nonce.clone(),
            request_hash: req.request_hash.clone(),
            error: WorkerError {
                code: "recipe".into(),
                message: format!("worker implements {RECIPE}, got {}", req.recipe),
            },
        })
        .unwrap_or_default();
    }
    let source = match b64_decode(&req.source_b64) {
        Ok(s) => s,
        Err(e) => return err_req(req, "source", e),
    };
    let input: SemaInput = match ac_sema_core::adapter::parse_into_input(
        &source,
        &req.package,
        &req.module,
        auto_ac_prototype::source::SourceProfile::Core,
    ) {
        Ok(i) => i,
        Err(e) => return err_req(req, "parse", e),
    };
    // NOTE: source edits mutate a working COPY; each session starts from
    // the CURRENT declared source the parent sends (the parent chains
    // edits across sessions and re-sends the current text).
    let mut source = source;
    let mut input = input;
    let store = match open_store(std::path::Path::new(&req.store_root)) {
        Ok(s) => s,
        Err(e) => return err_req(req, "store", format!("{e}")),
    };

    let mut out_steps: Vec<StepOut> = Vec::new();
    let mut restart_pending = false;
    for step in &req.steps {
        match step.op.as_str() {
            "query" => {
                let query = QueryKind::InferBody { fn_name: req.fn_name.clone() };
                let mut compute_calls = 0u32;
                let cache: String;
                let mut bindings = Vec::new();
                let mut diagnostics = Vec::new();
                let mut stages = vec!["request".to_string(), "cache-read-validate".to_string()];
                match lookup_only(&store, &input, &query) {
                    Ok(FetchOutcome::Hit { page, .. }) => {
                        cache = "hit".into();
                        if let ac_sema_contract::PageBody::CachedResult(c) = &page.body {
                            if let ac_sema_contract::ResultFacts::Infer(f) = &c.facts {
                                bindings = f
                                    .bindings
                                    .iter()
                                    .map(|(n, t)| BindingOut { name: n.clone(), ty: desc_name(t) })
                                    .collect();
                                diagnostics = f
                                    .diagnostics
                                    .iter()
                                    .map(|d| DiagOut { code: d.code.to_string(), message: d.message.clone() })
                                    .collect();
                            }
                        }
                    }
                    Ok(FetchOutcome::Stale { reason }) => {
                        // fresh recompute through the real flow
                        cache = recompute(
                            &store, &input, &query, &req.fn_name, &mut compute_calls,
                            &mut bindings, &mut diagnostics, &mut stages, &format!("stale({reason})"),
                        );
                    }
                    Ok(FetchOutcome::Miss) => {
                        cache = recompute(
                            &store, &input, &query, &req.fn_name, &mut compute_calls,
                            &mut bindings, &mut diagnostics, &mut stages, "miss",
                        );
                    }
                    Err(e) => return err_req(req, "lookup", format!("{e}")),
                }
                out_steps.push(StepOut {
                    op: "query",
                    cache,
                    compute_calls,
                    bindings,
                    diagnostics,
                    stages,
                });
            }
            "edit-source" => {
                let (from, to) = (step.from.as_deref().unwrap_or(""), step.to.as_deref().unwrap_or(""));
                let hits = source.matches(from).count();
                if from.is_empty() || hits != 1 {
                    return err_req(
                        req,
                        "edit-source",
                        format!("edit literal must match exactly once, found {hits}"),
                    );
                }
                // apply the edit to the RUNNING source (edits chain:
                // later steps see earlier ones), then re-project
                let edited = source.replacen(from, to, 1);
                source = edited;
                input = match ac_sema_core::adapter::parse_into_input(
                    &source,
                    &req.package,
                    &req.module,
                    auto_ac_prototype::source::SourceProfile::Core,
                ) {
                    Ok(i) => i,
                    Err(e) => return err_req(req, "reparse", e),
                };
            }
            "restart" => {
                restart_pending = true;
                break;
            }
            other => {
                return err_req(req, "step", format!("unknown op {other:?}"))
            }
        }
    }

    serde_json::to_value(ResponseOk {
        schema: SEMAWORKER_SCHEMA,
        nonce: req.nonce.clone(),
        request_hash: req.request_hash.clone(),
        ok: OkOut {
            recipe: RECIPE,
            steps: out_steps,
            restart_pending,
            worker_version: WORKER_VERSION,
            sema_core_version: "ac-sema-core/0.1.0",
        },
    })
    .unwrap_or_default()
}

/// The miss/stale recompute path: real inference (Ok only is published —
/// the v1 recipe judges diagnostic bodies through the library suites,
/// not through the runner) + publish + structured report.
fn recompute(
    store: &ac_artifact_probe::store::ManagedStore,
    input: &SemaInput,
    query: &QueryKind,
    fn_name: &str,
    compute_calls: &mut u32,
    bindings: &mut Vec<BindingOut>,
    diagnostics: &mut Vec<DiagOut>,
    stages: &mut Vec<String>,
    origin: &str,
) -> String {
    stages.push("infer".into());
    *compute_calls += 1;
    match run_inference(input, fn_name) {
        InferOutcome::Ok(summary) => {
            *bindings = summary
                .facts
                .bindings
                .iter()
                .map(|(n, t)| BindingOut { name: n.clone(), ty: desc_name(t) })
                .collect();
            stages.push("publish".into());
            match publish_infer_result(store, input, query, &summary, Some("autodown-probe")) {
                Ok(_) => format!("published<{origin}>"),
                Err(e) => format!("publish-error<{origin}:{e}>"),
            }
        }
        InferOutcome::Diags(summary) => {
            *diagnostics = summary
                .facts
                .diagnostics
                .iter()
                .map(|d| DiagOut { code: d.code.to_string(), message: d.message.clone() })
                .collect();
            // diagnostics are definitive facts but the v1 recipe does not
            // publish them through the runner path
            format!("diags<{origin}>")
        }
        InferOutcome::ResourceExhausted(e) => format!("resource-exhausted<{e}>"),
        InferOutcome::Cancelled => "cancelled".into(),
    }
}

fn err_req(req: &WorkerRequest, code: &str, message: String) -> serde_json::Value {
    serde_json::to_value(ResponseErr {
        schema: SEMAWORKER_SCHEMA,
        nonce: req.nonce.clone(),
        request_hash: req.request_hash.clone(),
        error: WorkerError { code: code.into(), message },
    })
    .unwrap_or_default()
}

fn desc_name(t: &ac_sema_contract::TypeDesc) -> String {
    match t {
        ac_sema_contract::TypeDesc::I32 => "i32".into(),
        ac_sema_contract::TypeDesc::Bool => "bool".into(),
        ac_sema_contract::TypeDesc::Opaque { name } => format!("opaque:{name}"),
        other => format!("{other:?}"),
    }
}

fn b64_decode(s: &str) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| format!("source_b64: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("source is not utf-8: {e}"))
}
