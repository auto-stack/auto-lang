//! `semaworker/1` — the sema probe worker IPC mode (PLAN-796 T-06).
//!
//! Same discipline as 788's `acworker/1`: a bounded single-shot
//! JSON-lines session (begin → request → end) over stdin/stdout; the
//! parent owns the wall deadline and kills the process on expiry; the
//! response echoes `nonce` + `request_hash` for parent-side admission;
//! a `restart` step ENDS the session (the parent continues the
//! remaining steps in a FRESH process against the same store root —
//! the real cross-process persistence path).

use crate::open_store;
use ac_sema_core::query::QueryKind;
use ac_sema_core::SemaInput;
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
    /// MUTUALLY-EXCLUSIVE typed outcome (F-796-R4-02):
    /// ok | diags | resource | cancelled | infra. The cache/disposition
    /// detail rides `cache`; it can never masquerade as success.
    pub outcome: &'static str,
    pub cache: String,
    pub compute_calls: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<BindingOut>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<DiagOut>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<StageSpan>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageSpan {
    pub name: String,
    pub wall_ns: u128,
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
    pub worker_version: String,
    pub sema_core_version: String,
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
    // TEST-ONLY fault seams (F-796-R4-03/R4-04 process-level controls):
    // never set by production paths; the runner tests set them through
    // the child environment to prove the PARENT's lifecycle/admission
    // actually fires (deadline-after-response, bad exit, bad version).
    if let Ok(hang) = std::env::var("SEMA_TEST_HANG_AFTER_RESPONSE_SECS") {
        let secs: u64 = hang.parse().unwrap_or(61);
        std::thread::sleep(std::time::Duration::from_secs(secs));
    }
    if let Ok(code) = std::env::var("SEMA_TEST_EXIT_AFTER_RESPONSE") {
        let code: i32 = code.parse().unwrap_or(7);
        std::process::exit(code);
    }
    if let Ok(v) = std::env::var("SEMA_TEST_FAKE_VERSION") {
        let _ = v; // version is compiled in; the seam validates that the
                   // parent DOES admit the real one — see mirror expectations
    }
    0
}

fn fail_protocol(msg: &str) -> i32 {
    let resp = ResponseErr {
        schema: SEMAWORKER_SCHEMA,
        nonce: String::new(),
        request_hash: String::new(),
        error: WorkerError {
            code: "protocol".into(),
            message: msg.into(),
        },
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
                let query = QueryKind::InferBody {
                    fn_name: req.fn_name.clone(),
                };
                let report = match crate::run_query_flow(
                    &store,
                    &input,
                    &query,
                    Some("autodown-probe"),
                    None,
                ) {
                    Ok(r) => r,
                    Err(e) => {
                        // infra-class failures are their own typed
                        // outcome — never a language success
                        out_steps.push(StepOut {
                            op: "query",
                            outcome: "infra",
                            cache: format!("infra-error<{e}>"),
                            compute_calls: 0,
                            bindings: Vec::new(),
                            diagnostics: Vec::new(),
                            stages: Vec::new(),
                        });
                        continue;
                    }
                };
                let outcome = match report.disposition {
                    crate::Disposition::Diags => "diags",
                    crate::Disposition::ResourceExhausted => "resource",
                    crate::Disposition::Cancelled => "cancelled",
                    _ => "ok",
                };
                out_steps.push(StepOut {
                    op: "query",
                    outcome,
                    cache: report.disposition.as_str().to_string(),
                    compute_calls: report.compute_calls,
                    bindings: report
                        .bindings
                        .iter()
                        .map(|(n, t)| BindingOut {
                            name: n.clone(),
                            ty: t.clone(),
                        })
                        .collect(),
                    diagnostics: report
                        .diagnostics
                        .iter()
                        .map(|(c, m)| DiagOut {
                            code: c.clone(),
                            message: m.clone(),
                        })
                        .collect(),
                    stages: report
                        .stages
                        .iter()
                        .map(|(n, ns)| StageSpan {
                            name: n.to_string(),
                            wall_ns: *ns,
                        })
                        .collect(),
                });
            }
            "edit-source" => {
                let (from, to) = (
                    step.from.as_deref().unwrap_or(""),
                    step.to.as_deref().unwrap_or(""),
                );
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
            other => return err_req(req, "step", format!("unknown op {other:?}")),
        }
    }

    let worker_version: String =
        std::env::var("SEMA_TEST_FAKE_VERSION").unwrap_or_else(|_| WORKER_VERSION.to_string());
    let sema_core_version = std::env::var("SEMA_TEST_FAKE_CORE_VERSION")
        .unwrap_or_else(|_| "ac-sema-core/0.1.0".to_string());
    serde_json::to_value(ResponseOk {
        schema: SEMAWORKER_SCHEMA,
        nonce: req.nonce.clone(),
        request_hash: req.request_hash.clone(),
        ok: OkOut {
            recipe: RECIPE,
            steps: out_steps,
            restart_pending,
            worker_version,
            sema_core_version,
        },
    })
    .unwrap_or_default()
}

fn err_req(req: &WorkerRequest, code: &str, message: String) -> serde_json::Value {
    serde_json::to_value(ResponseErr {
        schema: SEMAWORKER_SCHEMA,
        nonce: req.nonce.clone(),
        request_hash: req.request_hash.clone(),
        error: WorkerError {
            code: code.into(),
            message,
        },
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
