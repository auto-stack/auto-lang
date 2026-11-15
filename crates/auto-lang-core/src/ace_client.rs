//! PLAN-970 (T-03): the explicit-ACE session **client** — the daemon's
//! wire counterpart (`ac.session.daemon/1`). One persistent connection,
//! monotone request ids (the daemon's idempotency cache makes lost-reply
//! retries safe), bounded connect retries and a bounded receive timeout.
//! No legacy engine is reachable from here (the legacy
//! `AutovmPipeClient` remains a separate untouched surface).

use serde_json::{json, Value};

use crate::ace_daemon::pipe_addr;

#[cfg(windows)]
type PipeStream = tokio::net::windows::named_pipe::NamedPipeClient;
#[cfg(unix)]
type PipeStream = tokio::net::UnixStream;

/// A synchronous client for the ACE session daemon over its named pipe
/// (Windows) / unix socket. The connection persists across requests; a
/// reconnecting client RESUMES sessions by id (the daemon survives
/// client EOFs).
pub struct AceSessionClient {
    runtime: tokio::runtime::Runtime,
    writer: tokio::io::BufWriter<tokio::io::WriteHalf<PipeStream>>,
    reader: tokio::io::BufReader<tokio::io::ReadHalf<PipeStream>>,
    next_id: u64,
}

impl AceSessionClient {
    /// Connect with bounded retries (10 × 200ms — the legacy daemon
    /// client's startup discipline).
    pub fn connect(pipe_name: &str) -> Result<Self, String> {
        let runtime = tokio::runtime::Runtime::new().map_err(|e| format!("tokio runtime: {e}"))?;
        let addr = pipe_addr(pipe_name);
        let mut last = String::new();
        for _ in 0..10 {
            let connected = runtime.block_on(async {
                #[cfg(windows)]
                {
                    tokio::net::windows::named_pipe::ClientOptions::new()
                        .open(&addr)
                        .map_err(|e| format!("pipe open: {e}"))
                }
                #[cfg(unix)]
                {
                    tokio::net::UnixStream::connect(&addr)
                        .await
                        .map_err(|e| format!("socket connect: {e}"))
                }
            });
            match connected {
                Ok(stream) => {
                    let (read, write) = tokio::io::split(stream);
                    return Ok(AceSessionClient {
                        runtime,
                        writer: tokio::io::BufWriter::new(write),
                        reader: tokio::io::BufReader::new(read),
                        next_id: 1,
                    });
                }
                Err(e) => {
                    last = e;
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
        }
        Err(format!("ace.session/connect-failed: {last}"))
    }

    /// One request/response roundtrip on the persistent connection. The
    /// request carries the next monotone id and `version: 1`; the
    /// response must echo the id.
    pub fn request(&mut self, mut payload: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        payload["id"] = json!(id);
        payload
            .as_object_mut()
            .ok_or("request payload must be an object")?
            .entry("version")
            .or_insert(json!(1));
        let line = serde_json::to_string(&payload).map_err(|e| format!("serialize: {e}"))?;
        let reply = self.runtime.block_on(async {
            use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
            self.writer
                .write_all(line.as_bytes())
                .await
                .map_err(|e| format!("write: {e}"))?;
            self.writer
                .write_all(b"\n")
                .await
                .map_err(|e| format!("write: {e}"))?;
            self.writer
                .flush()
                .await
                .map_err(|e| format!("write: {e}"))?;
            let mut buf = String::new();
            // 30s bounded receive (long synchronous evaluations included).
            match tokio::time::timeout(
                std::time::Duration::from_secs(30),
                self.reader.read_line(&mut buf),
            )
            .await
            {
                Ok(Ok(0)) => Err("ace.session/daemon-closed".to_string()),
                Ok(Ok(_)) => Ok(buf),
                Ok(Err(e)) => Err(format!("read: {e}")),
                Err(_) => Err("ace.session/timeout".to_string()),
            }
        })?;
        let value: Value =
            serde_json::from_str(reply.trim_end()).map_err(|e| format!("parse: {e}"))?;
        if value.get("id").and_then(Value::as_u64) != Some(id) {
            return Err(format!("ace.session/id-mismatch in response: {value}"));
        }
        Ok(value)
    }

    // ── Method helpers (the daemon's frozen method set) ─────────────

    pub fn new_session(&mut self, code: Option<&str>) -> Result<Value, String> {
        let mut payload = json!({"method": "new-session"});
        if let Some(code) = code {
            payload["code"] = json!(code);
        }
        self.request(payload)
    }

    pub fn eval(&mut self, session: &str, code: &str) -> Result<Value, String> {
        self.request(json!({"method": "eval", "session": session, "code": code}))
    }

    pub fn edit(
        &mut self,
        session: &str,
        start: u64,
        end: u64,
        insert: &str,
    ) -> Result<Value, String> {
        self.request(json!({"method": "edit", "session": session, "start": start, "end": end, "insert": insert}))
    }

    pub fn ping(&mut self) -> Result<Value, String> {
        self.request(json!({"method": "ping"}))
    }
}
