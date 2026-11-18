//! PLAN-970 (T-03/T-08): the explicit-ACE session **daemon** — newline
//! JSONL over a named pipe (Windows) / unix socket, plus a stdio mode.
//!
//! The daemon is a TRANSPORT + PROTOCOL layer only: every session lives
//! behind the neutral contract (`crate::ace_session`), served by the
//! host the process installed (the `auto-lang-ace-host` bridge is the
//! reference). No legacy engine is ever dispatched from here — the
//! legacy `AutovmDaemon` remains a separate untouched surface
//! (retirement belongs to PLAN-987).
//!
//! Protocol (frozen by the PLAN-970 T-01 freeze record):
//! - `ac.session.daemon/1` — every response carries `version: 1`; a
//!   request without `version` is assumed to be 1 (the legacy daemon
//!   carried no version at all); a DIFFERENT version is the named error
//!   `version/mismatch` listing the supported versions.
//! - The legacy method names are kept (`new-session`, `eval`, `inspect`,
//!   `reset`, `delete`, `snapshot`, `list`, `ping`); the session
//!   capability adds `edit` (same-PID online update), `eval-async`,
//!   `result`, `cancel` and `close`.
//! - Idempotency: request `id`s are remembered with their response (a
//!   bounded FIFO of the last [`IDEMPOTENCY_CACHE`] replies); a
//!   DUPLICATE id carrying the SAME request line replays the stored
//!   response verbatim — a client retrying after a reconnect never
//!   re-executes a side-effecting commit (V-08). A reused id with a
//!   DIFFERENT body is a fresh dispatch: the legacy wire client numbers
//!   ids per connection from 1, so ids collide across connections by
//!   design (PLAN-982 round 187).
//! - A malformed/half line is a named in-band error and the loop
//!   CONTINUES (the legacy stdio discipline); stdio EOF is a graceful
//!   shutdown; a pipe connection's EOF drops that client only — the
//!   daemon stays up (the legacy pipe discipline).

use std::collections::HashMap;
use std::io::{BufRead, Write};

use serde_json::{json, Value};

use crate::ace_session::{
    ace_session_cleanup_expired, ace_session_create, ace_session_list, ace_session_op,
    AceSessionOp, AceSessionOutcome, AceSessionReport, AceSessionValue, SESSION_PROTOCOL_NAME,
    SESSION_PROTOCOL_VERSION,
};

/// How many recent request responses the idempotency cache keeps.
const IDEMPOTENCY_CACHE: usize = 64;

pub fn pipe_addr(name: &str) -> String {
    #[cfg(windows)]
    {
        format!(r"\\.\pipe\{}", name)
    }
    #[cfg(unix)]
    {
        format!("/tmp/{}.sock", name)
    }
}

/// The ACE session daemon (transport-agnostic core + transports).
pub struct AceDaemon {
    /// request id → (request line, serialized response). A replay fires
    /// only when the id AND the full request line match — a reused id
    /// carrying a DIFFERENT request is a fresh dispatch (the legacy wire
    /// client numbers ids per connection from 1, so the id alone cannot
    /// identify a retry; PLAN-982 round 187).
    cache: HashMap<u64, (String, String)>,
    cache_order: std::collections::VecDeque<u64>,
}

impl Default for AceDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl AceDaemon {
    pub fn new() -> Self {
        AceDaemon {
            cache: HashMap::new(),
            cache_order: std::collections::VecDeque::new(),
        }
    }

    fn remember(&mut self, id: u64, request: &str, response: String) {
        if self.cache_order.len() >= IDEMPOTENCY_CACHE {
            if let Some(evict) = self.cache_order.pop_front() {
                self.cache.remove(&evict);
            }
        }
        self.cache_order.push_back(id);
        self.cache.insert(id, (request.to_string(), response));
    }

    /// Handle one input LINE (without the trailing newline) and produce
    /// one output LINE. This is the whole protocol; the transports below
    /// only move bytes. Malformed input keeps the loop alive (in-band
    /// error, legacy discipline).
    pub fn handle_line(&mut self, line: &str) -> String {
        let req: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                return error_response(0, &format!("Invalid request: {e}"));
            }
        };
        // Version discipline: absent = 1 (the legacy protocol carried no
        // version); a different explicit version is a named refusal that
        // lists what this daemon speaks.
        match req.get("version").and_then(Value::as_u64) {
            None => {}
            Some(v) if v == SESSION_PROTOCOL_VERSION as u64 => {}
            Some(v) => {
                return error_response(
                    req.get("id").and_then(Value::as_u64).unwrap_or(0),
                    &format!(
                        "version/mismatch: request version {v} is not supported (this daemon speaks {SESSION_PROTOCOL_NAME})"
                    ),
                );
            }
        }
        // F-970-R-05: only requests that CARRY an id participate in the
        // idempotent replay cache (an id-less request must never replay a
        // different id-less request's stored response).
        let has_id = req.get("id").map(|v| !v.is_null()).unwrap_or(false);
        let id = req.get("id").and_then(Value::as_u64).unwrap_or(0);
        if has_id {
            // Idempotent replay: the SAME request line under the SAME id
            // answers with the stored response VERBATIM (no re-execution —
            // a reconnecting client's retry). A reused id with a DIFFERENT
            // body is a fresh dispatch: the legacy wire client numbers ids
            // per connection from 1, so ids collide across connections by
            // design (PLAN-982 round 187 — replaying the create response
            // to a same-id eval was a real interop bug).
            if let Some((cached_request, cached)) = self.cache.get(&id) {
                if cached_request == line {
                    return cached.clone();
                }
            }
        }
        let response = self.dispatch(id, &req);
        let serialized = serde_json::to_string(&response)
            .unwrap_or_else(|_| error_response(id, "response serialization failure"));
        if has_id {
            self.remember(id, line, serialized.clone());
        }
        serialized
    }

    fn dispatch(&mut self, id: u64, req: &Value) -> Value {
        let method = req.get("method").and_then(Value::as_str).unwrap_or("");
        let session = req.get("session").and_then(Value::as_str);
        let report = match method {
            "ping" => {
                return ok_response(id, None, None, None, Some("pong".to_string()));
            }
            "new-session" => {
                let report = ace_session_create(
                    req.get("code").and_then(Value::as_str),
                    req.get("wall_ms")
                        .and_then(Value::as_u64)
                        .map(std::time::Duration::from_millis),
                );
                return report_to_response(id, &report, true);
            }
            "list" => {
                // The legacy list triggers the expiry sweep.
                ace_session_cleanup_expired();
                // PLAN-982 (EXEC, round 187): the live ids ride `data` —
                // the legacy `AutovmClient::list` / `auto req --list` read
                // `data` only (parking them in `message` made the legacy
                // face print "No sessions" forever). An empty list keeps
                // `data: null` so the legacy print branch stays identical.
                let live = ace_session_list();
                let data = if live.is_empty() {
                    Value::Null
                } else {
                    json!(live.join(","))
                };
                return json!({
                    "id": id,
                    "version": SESSION_PROTOCOL_VERSION,
                    "status": "ok",
                    "session": Value::Null,
                    "value": Value::Null,
                    "type": Value::Null,
                    "message": Value::Null,
                    "data": data,
                });
            }
            _ => {
                let Some(session) = session else {
                    return error_value(id, "Invalid request: method needs a session");
                };
                match method {
                    "eval" => {
                        let Some(code) = req.get("code").and_then(Value::as_str) else {
                            return error_value(id, "Invalid request: eval needs code");
                        };
                        ace_session_op(
                            session,
                            AceSessionOp::Eval {
                                input: code,
                                wall: req
                                    .get("wall_ms")
                                    .and_then(Value::as_u64)
                                    .map(std::time::Duration::from_millis),
                                cancel: None,
                            },
                        )
                    }
                    "edit" => {
                        let (Some(start), Some(end), Some(insert)) = (
                            req.get("start").and_then(Value::as_u64),
                            req.get("end").and_then(Value::as_u64),
                            req.get("insert").and_then(Value::as_str),
                        ) else {
                            return error_value(id, "Invalid request: edit needs start/end/insert");
                        };
                        ace_session_op(
                            session,
                            AceSessionOp::Edit {
                                start,
                                end,
                                insert: insert.as_bytes(),
                            },
                        )
                    }
                    "eval-async" => {
                        let Some(code) = req.get("code").and_then(Value::as_str) else {
                            return error_value(id, "Invalid request: eval-async needs code");
                        };
                        ace_session_op(
                            session,
                            AceSessionOp::EvalAsync {
                                input: code,
                                wall: req
                                    .get("wall_ms")
                                    .and_then(Value::as_u64)
                                    .map(std::time::Duration::from_millis),
                            },
                        )
                    }
                    "result" => {
                        let Some(eval_id) = req.get("eval_id").and_then(Value::as_u64) else {
                            return error_value(id, "Invalid request: result needs eval_id");
                        };
                        ace_session_op(session, AceSessionOp::Result { eval_id })
                    }
                    "cancel" => {
                        let Some(eval_id) = req.get("eval_id").and_then(Value::as_u64) else {
                            return error_value(id, "Invalid request: cancel needs eval_id");
                        };
                        ace_session_op(session, AceSessionOp::CancelEval { eval_id })
                    }
                    "inspect" => ace_session_op(session, AceSessionOp::Inspect),
                    "vars" => ace_session_op(session, AceSessionOp::Vars),
                    "snapshot" => ace_session_op(session, AceSessionOp::Snapshot),
                    "reset" => ace_session_op(session, AceSessionOp::Reset),
                    "delete" | "close" => ace_session_op(session, AceSessionOp::Close),
                    other => {
                        return error_value(id, &format!("Unknown method: {other}"));
                    }
                }
            }
        };
        report_to_response(id, &report, false)
    }

    /// Serve stdin/stdout until EOF (graceful shutdown — the legacy
    /// stdio discipline).
    pub fn run_stdio(&mut self) {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        for line in stdin.lock().lines() {
            let Ok(line) = line else { break };
            if line.trim().is_empty() {
                continue;
            }
            let response = self.handle_line(&line);
            if writeln!(stdout, "{response}")
                .and_then(|_| stdout.flush())
                .is_err()
            {
                break;
            }
        }
    }

    /// Serve one named-pipe connection at a time; a client's EOF drops
    /// that client only — the daemon stays up for reconnects (the legacy
    /// pipe discipline). Two-thread shape (the legacy daemon's): a tokio
    /// thread owns accept/IO, the main thread handles requests
    /// synchronously (no runtime nesting).
    pub fn run_pipe(self, pipe_name: &str) {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime for ACE session daemon");
        let pipe_path = pipe_addr(pipe_name);
        eprintln!("ACE session daemon: listening on {}", pipe_path);

        let (conn_tx, conn_rx) =
            std::sync::mpsc::channel::<(String, std::sync::mpsc::Sender<String>)>();
        let path = pipe_path.clone();
        let io_thread = std::thread::spawn(move || {
            rt.block_on(async move {
                #[cfg(unix)]
                let _ = std::fs::remove_file(&path);

                loop {
                    let stream = accept_connection(&path).await;
                    eprintln!("ACE session daemon: client connected");
                    let (read, write) = tokio::io::split(stream);
                    let mut reader = tokio::io::BufReader::new(read);
                    let mut writer = tokio::io::BufWriter::new(write);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        let n = tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line).await;
                        match n {
                            Ok(0) => break, // client disconnected; daemon stays up
                            Ok(_) => {}
                            Err(_) => break,
                        }
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        let (resp_tx, resp_rx) = std::sync::mpsc::channel::<String>();
                        if conn_tx.send((trimmed.to_string(), resp_tx)).is_err() {
                            break;
                        }
                        if let Ok(json) = resp_rx.recv() {
                            let _ =
                                tokio::io::AsyncWriteExt::write_all(&mut writer, json.as_bytes())
                                    .await;
                            let _ = tokio::io::AsyncWriteExt::write_all(
                                &mut writer,
                                b"
",
                            )
                            .await;
                            let _ = tokio::io::AsyncWriteExt::flush(&mut writer).await;
                        } else {
                            break;
                        }
                    }
                    eprintln!("ACE session daemon: client disconnected");
                }
            })
        });

        let mut daemon = self;
        while let Ok((line, resp_tx)) = conn_rx.recv() {
            let response = daemon.handle_line(&line);
            let _ = resp_tx.send(response);
        }
        let _ = io_thread.join();
    }
}

#[cfg(windows)]
async fn accept_connection(pipe_path: &str) -> tokio::net::windows::named_pipe::NamedPipeServer {
    use tokio::net::windows::named_pipe::ServerOptions;
    let server = ServerOptions::new()
        .first_pipe_instance(false)
        .create(pipe_path)
        .expect("Failed to create named pipe");
    server.connect().await.expect("Pipe connect failed");
    server
}

#[cfg(unix)]
async fn accept_connection(socket_path: &str) -> tokio::net::UnixStream {
    use std::sync::OnceLock;
    use tokio::net::UnixListener;
    static LISTENER: OnceLock<UnixListener> = OnceLock::new();
    let listener = LISTENER
        .get_or_init(|| UnixListener::bind(socket_path).expect("Failed to bind Unix socket"));
    listener.accept().await.expect("Accept failed").0
}

fn ok_response(
    id: u64,
    session: Option<String>,
    value: Option<String>,
    type_: Option<&str>,
    message: Option<String>,
) -> Value {
    json!({
        "id": id,
        "version": SESSION_PROTOCOL_VERSION,
        "status": "ok",
        "session": session,
        "value": value,
        "type": type_,
        "message": message,
        "data": Value::Null,
    })
}

fn error_value(id: u64, message: &str) -> Value {
    json!({
        "id": id,
        "version": SESSION_PROTOCOL_VERSION,
        "status": "error",
        "session": Value::Null,
        "value": Value::Null,
        "type": Value::Null,
        "message": message,
        "data": Value::Null,
    })
}

fn error_response(id: u64, message: &str) -> String {
    serde_json::to_string(&error_value(id, message)).unwrap()
}

/// Map one neutral report to the wire response (the create path also
/// returns the session id in `data.session`).
fn report_to_response(id: u64, report: &AceSessionReport, is_create: bool) -> Value {
    let status = match &report.outcome {
        AceSessionOutcome::Ok => "ok",
        AceSessionOutcome::Cancelled => "cancelled",
        _ => "error",
    };
    let value = match report.value {
        AceSessionValue::None => None,
        AceSessionValue::Int(n) => Some(n.to_string()),
        AceSessionValue::Bool(b) => Some(b.to_string()),
    };
    let type_ = match report.value {
        AceSessionValue::None => None,
        AceSessionValue::Int(_) => Some("int"),
        AceSessionValue::Bool(_) => Some("bool"),
    };
    let message = match &report.outcome {
        AceSessionOutcome::Ok => None,
        AceSessionOutcome::Diagnostic { code, diagnostics } => Some(format!(
            "{code}: {}",
            diagnostics
                .first()
                .map(|d| d.message.clone())
                .unwrap_or_default()
        )),
        AceSessionOutcome::Denied { code, message } => Some(format!("{code}: {message}")),
        AceSessionOutcome::ResourceDenied { kind, message } => Some(format!("{kind}: {message}")),
        AceSessionOutcome::Cancelled => Some("cancelled".to_string()),
        AceSessionOutcome::HostFailure { code, message } => Some(format!("{code}: {message}")),
    };
    // F-970-R-02: the source text itself crosses the wire (the legacy
    // daemon returned it in `value`; wire clients need it to compute
    // `edit` offsets).
    let source_text = if matches!(report.outcome, AceSessionOutcome::Ok) {
        report.snapshot.clone()
    } else {
        None
    };
    let mut data = json!({});
    if is_create {
        data["session"] = json!(report.session.clone());
        data["revision"] = json!(report.revision);
    } else {
        data["revision"] = json!(report.revision);
        if report.eval_id.is_some() {
            data["eval_id"] = json!(report.eval_id);
        }
        if !report.vars.is_empty() {
            data["vars"] = json!(report
                .vars
                .iter()
                .map(|v| json!({"name": v.name, "value": match v.value {
                    AceSessionValue::Int(n) => Value::String(n.to_string()),
                    AceSessionValue::Bool(b) => Value::String(b.to_string()),
                    AceSessionValue::None => Value::Null,
                }}))
                .collect::<Vec<_>>());
        }
        if let Some(source) = &source_text {
            data["snapshot_len"] = json!(source.len());
        }
    }
    let value = match (&source_text, value) {
        (Some(source), None) => Some(source.clone()),
        (_, v) => v,
    };
    json!({
        "id": id,
        "version": SESSION_PROTOCOL_VERSION,
        "status": status,
        "session": report.session.clone(),
        "value": value,
        "type": type_,
        "message": message,
        "data": data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// p970: with no host installed the daemon answers the NAMED host
    /// failure in-band (never a panic, never a legacy route).
    use crate::ace_session::test_registry::REGISTRY_LOCK;

    #[test]
    fn p970_daemon_unregistered_host_is_in_band_error() {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        crate::ace_session::shutdown_ace_session_host();
        let mut d = AceDaemon::new();
        let out = d.handle_line(r#"{"id":1,"method":"new-session"}"#);
        assert!(out.contains("ace.session/host-unregistered"), "{out}");
        assert!(out.contains("\"version\":1"), "{out}");
    }

    /// p970: malformed / half lines are in-band errors and the loop
    /// continues (V-08 half-message discipline).
    #[test]
    fn p970_daemon_malformed_line_continues() {
        let mut d = AceDaemon::new();
        let out1 = d.handle_line(r#"{"id":1,"method":"#);
        assert!(out1.contains("Invalid request"), "{out1}");
        let out2 = d.handle_line(r#"{"id":2,"method":"ping"}"#);
        assert!(out2.contains("pong"), "{out2}");
    }

    /// p970: an explicit foreign version is the named version/mismatch
    /// error; an absent version is accepted as 1.
    #[test]
    fn p970_daemon_version_discipline() {
        let mut d = AceDaemon::new();
        let out = d.handle_line(r#"{"id":1,"version":99,"method":"ping"}"#);
        assert!(out.contains("version/mismatch"), "{out}");
        assert!(out.contains("ac.session.daemon/1"), "{out}");
        let out = d.handle_line(r#"{"id":2,"method":"ping"}"#);
        assert!(out.contains("pong"), "{out}");
    }

    /// p970: a duplicate request id replays the stored response verbatim
    /// and never re-dispatches (idempotent retry, V-08). The counter
    /// proof: with no host installed the first call fails with the host
    /// error; its replay is byte-identical.
    #[test]
    fn p970_daemon_idempotent_replay() {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        crate::ace_session::shutdown_ace_session_host();
        let mut d = AceDaemon::new();
        let first = d.handle_line(r#"{"id":7,"method":"new-session"}"#);
        let replay = d.handle_line(r#"{"id":7,"method":"new-session"}"#);
        assert_eq!(first, replay, "duplicate id replays verbatim");
        // A DIFFERENT id is a fresh dispatch (different content allowed).
        let other = d.handle_line(r#"{"id":8,"method":"ping"}"#);
        assert!(other.contains("pong"));
    }

    /// PLAN-982 (EXEC, round 187): a REUSED id carrying a different
    /// request body is a fresh dispatch, never a replay of the earlier
    /// id's response. The legacy wire client (`auto req`) numbers ids per
    /// connection from 1, so the create response for id=1 must NOT be
    /// replayed to the id=1 eval of the next connection — the real
    /// interop bug this round's CLI probe caught.
    #[test]
    fn p982_daemon_reused_id_with_new_body_dispatches_fresh() {
        let mut d = AceDaemon::new();
        // First request under id 1: ping (cached).
        let ping = d.handle_line(r#"{"id":1,"method":"ping"}"#);
        assert!(ping.contains("pong"), "{ping}");
        // The SAME ping line under id 1 replays verbatim (V-08 retry).
        let replay = d.handle_line(r#"{"id":1,"method":"ping"}"#);
        assert_eq!(ping, replay, "an identical retry still replays verbatim");
        // Same id 1, DIFFERENT body: must dispatch fresh (the stored pong
        // would otherwise mask this response).
        let other = d.handle_line(r#"{"id":1,"method":"version-missing","session":"ses_x"}"#);
        assert!(
            other.contains("Unknown method"),
            "a reused id with a new body is a fresh dispatch: {other}"
        );
    }

    /// p970: unknown methods and missing sessions are named errors.
    #[test]
    fn p970_daemon_unknown_method_and_session() {
        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        crate::ace_session::shutdown_ace_session_host();
        let mut d = AceDaemon::new();
        let out = d.handle_line(r#"{"id":1,"method":"nope","session":"ses_x"}"#);
        assert!(out.contains("Unknown method"), "{out}");
        let out = d.handle_line(r#"{"id":2,"method":"eval","session":"ses_x","code":"1"}"#);
        assert!(out.contains("ace.session/host-unregistered"), "{out}");
    }

    /// PLAN-982 (EXEC, round 187): `list` carries the live ids in `data`
    /// — the legacy wire client (`auto req --list`) reads `data` only.
    /// A stub host stands in for the registry slot; the REAL bridge ride
    /// is the man-side CLI case (`plan982_exec_req_cli`).
    #[test]
    fn p982_daemon_list_carries_sessions_in_data() {
        use crate::ace_session::{
            AceSessionHost, AceSessionOp, AceSessionOutcome, AceSessionReport, AceSessionValue,
        };
        use std::sync::Arc;

        struct ListHost;
        fn canned(session: &str) -> AceSessionReport {
            AceSessionReport {
                session: Some(session.to_string()),
                outcome: AceSessionOutcome::Ok,
                revision: 0,
                value: AceSessionValue::None,
                eval_id: None,
                vars: Vec::new(),
                snapshot: None,
                rebuilt: false,
                wall_ms: 0.0,
            }
        }
        impl AceSessionHost for ListHost {
            fn create(
                &self,
                _source: Option<&str>,
                _wall: Option<std::time::Duration>,
            ) -> AceSessionReport {
                canned("ses_a")
            }
            fn op(&self, _session: &str, _op: AceSessionOp<'_>) -> AceSessionReport {
                canned("ses_a")
            }
            fn list(&self) -> Vec<String> {
                vec!["ses_a".to_string(), "ses_b".to_string()]
            }
            fn cleanup_expired(&self) -> Vec<String> {
                Vec::new()
            }
        }

        let _guard = REGISTRY_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        crate::ace_session::install_ace_session_host(Arc::new(ListHost)).expect("install");
        let mut d = AceDaemon::new();
        let out = d.handle_line(r#"{"id":3,"method":"list"}"#);
        let resp: Value = serde_json::from_str(&out).expect("response json");
        assert_eq!(resp["status"], "ok", "{out}");
        assert_eq!(resp["data"], json!("ses_a,ses_b"), "{out}");
        assert!(resp["message"].is_null(), "{out}");
        crate::ace_session::shutdown_ace_session_host();
    }
}
