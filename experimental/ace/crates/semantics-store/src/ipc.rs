//! `semaworker/1` — the sema probe worker IPC mode (PLAN-796 T-06).
//!
//! Same discipline as 788's `acworker/1`: a bounded single-shot
//! JSON-lines session (begin → request → end) over stdin/stdout; the
//! parent owns the wall deadline and kills the process on expiry; the
//! response echoes `nonce` + `request_hash` for parent-side admission;
//! a `restart` step ENDS the session (the parent continues the
//! remaining steps in a FRESH process against the same store root —
//! the real cross-process persistence path).
//!
//! F-796-R6 repairs wired here:
//! * R6-03 — lines are read with a PRE-ALLOCATION bound (never
//!   `lines()` materializing a hostile line first); the worker also
//!   self-contains at startup (parent kill reaps its tree).
//! * R6-05 — the SAME session meter covers decode/solve/publish and is
//!   reported per step (the previous `meter=None` bypass is gone).
//! * R6-07 — a language PARSE failure is a typed `diags` outcome judged
//!   parent-side, never a generic infrastructure error; an edit may fix
//!   the source (reparse) and clear the diagnostic.

use crate::open_store;
use ace_semantics::adapter::{parse_into_input_typed, LanguageDiagnostic};
use ace_semantics::query::QueryKind;
use ace_semantics::SemaInput;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

pub const SEMAWORKER_SCHEMA: &str = "semaworker/1";
/// Recipe identity this worker implements.
pub const RECIPE: &str = "sema/query-v1";

/// Bounded read of one stdin line (the request is source + steps). The
/// plan freezes request/response METADATA at 4 MiB each; the cap is
/// enforced while reading, before any allocation of the full line.
const MAX_LINE: usize = 4 << 20;

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

/// Realized session-meter readings for one step (F-796-R6-05): proof
/// the worker ran the SAME meter as the flow, not `None`.
#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MeterOut {
    pub scratch_peak_bytes: u64,
    pub hot_bytes: u64,
    pub rejections: u32,
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
    pub meter: MeterOut,
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

pub const WORKER_VERSION: &str = concat!("ac-sema-worker/", env!("CARGO_PKG_VERSION"));

/// Serve one session from stdin; exactly one response line on stdout;
/// exit 0. Any malformed input is a controlled error response (never a
/// panic, never a hang).
pub fn serve() -> i32 {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock();

    // begin
    let begin: SessionBegin = match next_json(&mut lines) {
        Ok(Some(v)) => v,
        _ => return fail_protocol("begin line missing or malformed"),
    };
    if begin.op != "begin" || begin.schema != SEMAWORKER_SCHEMA {
        return fail_protocol("bad begin (expected op=begin, schema=semaworker/1)");
    }
    // request — the envelope shape mirrors acworker/1: {"op":"request","request":{…}}
    let req_owned: String = match next_raw(&mut lines) {
        Ok(Some(l)) => l,
        Ok(None) => return fail_protocol("request line missing"),
        Err(observed) => {
            return fail_protocol(&format!(
                "request line exceeds the {}-byte bound ({} observed)",
                MAX_LINE, observed
            ))
        }
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
    let end = match next_raw(&mut lines) {
        Ok(v) => v,
        Err(observed) => {
            return fail_protocol(&format!(
                "end line exceeds the {}-byte bound ({} observed)",
                MAX_LINE, observed
            ))
        }
    };
    match end {
        Some(l) if l.contains(r#""op":"end""#) || l.contains(r#""op": "end""#) => {}
        _ => return fail_protocol("end line missing"),
    }

    let resp = execute(&req);
    let mut out = std::io::stdout().lock();
    let line = serde_json::to_string(&resp).unwrap_or_default();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
    // TEST-ONLY fault seams (F-796-R4-03/R4-04/R6-03 process-level
    // controls): never set by production paths; the runner tests set
    // them through the child environment to prove the PARENT's
    // lifecycle/admission actually fires (deadline-after-response, bad
    // exit, bad version, inherited-pipe descendant).
    if let Ok(hang) = std::env::var("SEMA_TEST_HANG_AFTER_RESPONSE_SECS") {
        let secs: u64 = hang.parse().unwrap_or(61);
        std::thread::sleep(std::time::Duration::from_secs(secs));
    }
    if let Ok(secs) = std::env::var("SEMA_TEST_SPAWN_LINGER_SECS") {
        // Spawn a descendant that INHERITS our stdout/stderr and outlives
        // this process. With self-containment the job kills it at exit;
        // with containment disabled the parent's own job must do so.
        let secs: u64 = secs.parse().unwrap_or(61);
        let _ = std::process::Command::new("cmd")
            .args(["/C", "ping", "-n", &(secs + 1).to_string(), "127.0.0.1"])
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn();
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

/// Read ONE line with a hard PRE-ALLOCATION cap (F-796-R6-03): the
/// buffer never exceeds `MAX_LINE`, no matter how hostile the input.
/// Ok(None) = EOF; Err(observed) = the line exceeded the cap.
fn next_raw<R: BufRead>(r: &mut R) -> Result<Option<String>, usize> {
    let mut line: Vec<u8> = Vec::new();
    loop {
        let available = match r.fill_buf() {
            Ok(b) => b,
            Err(_) => {
                return if line.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(String::from_utf8_lossy(&line).to_string()))
                }
            }
        };
        if available.is_empty() {
            return if line.is_empty() {
                Ok(None)
            } else {
                Ok(Some(String::from_utf8_lossy(&line).to_string()))
            };
        }
        match available.iter().position(|&b| b == b'\n') {
            Some(nl) => {
                if line.len() + nl > MAX_LINE {
                    return Err(line.len() + nl);
                }
                line.extend_from_slice(&available[..nl]);
                r.consume(nl + 1);
                return Ok(Some(String::from_utf8_lossy(&line).to_string()));
            }
            None => {
                if line.len() + available.len() > MAX_LINE {
                    return Err(line.len() + available.len());
                }
                let take = available.len();
                line.extend_from_slice(available);
                r.consume(take);
            }
        }
    }
}

fn next_json<T: serde::de::DeserializeOwned, R: BufRead>(r: &mut R) -> Result<Option<T>, ()> {
    match next_raw(r) {
        Ok(Some(raw)) => serde_json::from_str(&raw).map(Some).map_err(|_| ()),
        Ok(None) => Err(()),
        Err(_) => Err(()),
    }
}

fn meter_out(meter: &crate::budget::SessionMeter) -> MeterOut {
    MeterOut {
        scratch_peak_bytes: meter.scratch_peak() as u64,
        hot_bytes: meter.hot.bytes() as u64,
        rejections: meter.rejections() as u32,
    }
}

fn diag_step(
    op: &'static str,
    code: &str,
    message: &str,
    meter: &crate::budget::SessionMeter,
) -> StepOut {
    StepOut {
        op,
        outcome: "diags",
        cache: "parse-rejected".into(),
        compute_calls: 0,
        bindings: Vec::new(),
        diagnostics: vec![DiagOut {
            code: code.to_string(),
            message: message.to_string(),
        }],
        stages: Vec::new(),
        meter: meter_out(meter),
    }
}

/// Execute steps of THIS session (up to and excluding the next
/// `restart`), mirroring the probe's real query flow per step. A
/// language parse failure is a TYPED diagnostic outcome (F-796-R6-07);
/// an `edit-source` that makes the source parseable clears it.
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
    let store = match open_store(std::path::Path::new(&req.store_root)) {
        Ok(s) => s,
        Err(e) => return err_req(req, "store", format!("{e}")),
    };

    let mut source = source;
    let mut parse_diag: Option<(String, String)> = None;
    let mut input: Option<SemaInput> = match parse_into_input_typed(
        &source,
        &req.package,
        &req.module,
        ace_core::source::SourceProfile::Core,
    ) {
        Ok(i) => Some(i),
        Err(LanguageDiagnostic::Parse { code, message }) => {
            parse_diag = Some((code, message));
            None
        }
        Err(LanguageDiagnostic::Projection(e)) => return err_req(req, "parse", e),
    };

    let mut meter = crate::budget::SessionMeter::new();
    let mut out_steps: Vec<StepOut> = Vec::new();
    let mut restart_pending = false;
    for step in &req.steps {
        match step.op.as_str() {
            "query" => {
                if let Some((code, message)) = parse_diag.clone() {
                    out_steps.push(diag_step("query", &code, &message, &meter));
                    continue;
                }
                let Some(current) = input.as_ref() else {
                    return err_req(req, "query", "no parsed input for query".into());
                };
                let query = QueryKind::InferBody {
                    fn_name: req.fn_name.clone(),
                };
                let report = match crate::run_query_flow(
                    &store,
                    current,
                    &query,
                    Some("autodown-probe"),
                    Some(&mut meter),
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
                            meter: meter_out(&meter),
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
                    meter: meter_out(&meter),
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
                source = source.replacen(from, to, 1);
                match parse_into_input_typed(
                    &source,
                    &req.package,
                    &req.module,
                    ace_core::source::SourceProfile::Core,
                ) {
                    Ok(i) => {
                        input = Some(i);
                        parse_diag = None;
                    }
                    Err(LanguageDiagnostic::Parse { code, message }) => {
                        input = None;
                        parse_diag = Some((code, message));
                    }
                    Err(LanguageDiagnostic::Projection(e)) => return err_req(req, "reparse", e),
                }
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

fn b64_decode(s: &str) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| format!("source_b64: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("source is not utf-8: {e}"))
}
