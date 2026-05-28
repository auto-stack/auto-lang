// AutoVM Daemon — persistent VM server over named pipe (Windows) / Unix socket
//
// Plan 269: `auto serve` starts this daemon; `auto req` connects as client.
// Reuses SessionManager from MCP module for session lifecycle.

use crate::mcp::session_manager::SessionManager;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

// ── Protocol Types ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonRequest {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonResponse {
    pub id: u64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl DaemonResponse {
    pub fn ok(id: u64) -> Self {
        Self { id, status: "ok".into(), session: None, value: None, type_: None, message: None, data: None }
    }
    pub fn error(id: u64, msg: impl Into<String>) -> Self {
        Self { id, status: "error".into(), session: None, value: None, type_: None, message: Some(msg.into()), data: None }
    }
}

// ── Daemon Core ─────────────────────────────────────────────────────

pub struct AutovmDaemon {
    sessions: SessionManager,
}

impl AutovmDaemon {
    pub fn new() -> Self {
        Self { sessions: SessionManager::new() }
    }

    /// Run daemon using stdin/stdout (simple mode, no named pipe).
    /// Used for initial implementation; named pipe support added later.
    pub fn run_stdio(&mut self) {
        eprintln!("AutoVM daemon: stdio mode started");
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let mut locked = stdin.lock();

        loop {
            let mut line = String::new();
            match locked.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {}
                Err(e) => {
                    eprintln!("AutoVM daemon: read error: {}", e);
                    break;
                }
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let req = match serde_json::from_str::<DaemonRequest>(line) {
                Ok(r) => r,
                Err(e) => {
                    let resp = DaemonResponse::error(0, format!("Invalid request: {}", e));
                    let _ = writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap());
                    let _ = stdout.flush();
                    continue;
                }
            };

            let resp = self.handle_request(req);
            let _ = writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap());
            let _ = stdout.flush();
        }

        eprintln!("AutoVM daemon: shutting down");
    }

    fn handle_request(&mut self, req: DaemonRequest) -> DaemonResponse {
        match req.method.as_str() {
            "new-session" => self.handle_new_session(&req),
            "eval" => self.handle_eval(&req),
            "inspect" => self.handle_inspect(&req),
            "reset" => self.handle_reset(&req),
            "delete" => self.handle_delete(&req),
            "snapshot" => self.handle_snapshot(&req),
            "list" => self.handle_list(&req),
            "ping" => {
                let mut r = DaemonResponse::ok(req.id);
                r.message = Some("pong".into());
                r
            }
            _ => DaemonResponse::error(req.id, format!("Unknown method: {}", req.method)),
        }
    }

    fn handle_new_session(&mut self, req: &DaemonRequest) -> DaemonResponse {
        let ses_id = self.sessions.create(false);
        let mut r = DaemonResponse::ok(req.id);
        r.session = Some(ses_id);
        r
    }

    fn handle_eval(&mut self, req: &DaemonRequest) -> DaemonResponse {
        let ses_id = match &req.session {
            Some(s) => s.clone(),
            None => return DaemonResponse::error(req.id, "Missing session_id"),
        };

        let code = match &req.code {
            Some(c) => c.clone(),
            None => return DaemonResponse::error(req.id, "Missing code"),
        };

        let session = match self.sessions.get(&ses_id) {
            Some(s) => s,
            None => return DaemonResponse::error(req.id, format!("Session not found: {}", ses_id)),
        };

        match session.run(&code) {
            Ok(_) => {
                let result_value = session.format_last_result()
                    .or_else(|| session.get_last_result().map(|v| v.to_string()));

                self.sessions.append_source(&ses_id, &code);

                let mut r = DaemonResponse::ok(req.id);
                r.session = Some(ses_id);
                r.value = result_value;
                r
            }
            Err(e) => {
                let mut r = DaemonResponse::error(req.id, format!("{}", e));
                r.session = Some(ses_id);
                r
            }
        }
    }

    fn handle_inspect(&mut self, req: &DaemonRequest) -> DaemonResponse {
        let ses_id = match &req.session {
            Some(s) => s.clone(),
            None => return DaemonResponse::error(req.id, "Missing session_id"),
        };

        let session = match self.sessions.get(&ses_id) {
            Some(s) => s,
            None => return DaemonResponse::error(req.id, format!("Session not found: {}", ses_id)),
        };

        let stats = session.stats();
        let functions: Vec<serde_json::Value> = session.functions()
            .into_iter()
            .map(|f| serde_json::json!({"name": f}))
            .collect();

        let mut r = DaemonResponse::ok(req.id);
        r.session = Some(ses_id);
        r.data = Some(serde_json::json!({
            "bytecode_size": stats.bytecode_size,
            "heap_objects": stats.heap_objects,
            "arrays": stats.arrays,
            "functions": functions,
        }));
        r
    }

    fn handle_reset(&mut self, req: &DaemonRequest) -> DaemonResponse {
        let ses_id = match &req.session {
            Some(s) => s.clone(),
            None => return DaemonResponse::error(req.id, "Missing session_id"),
        };

        if self.sessions.reset(&ses_id) {
            let mut r = DaemonResponse::ok(req.id);
            r.session = Some(ses_id);
            r.message = Some("Session reset".into());
            r
        } else {
            DaemonResponse::error(req.id, format!("Session not found: {}", ses_id))
        }
    }

    fn handle_delete(&mut self, req: &DaemonRequest) -> DaemonResponse {
        let ses_id = match &req.session {
            Some(s) => s.clone(),
            None => return DaemonResponse::error(req.id, "Missing session_id"),
        };

        if self.sessions.delete(&ses_id) {
            let mut r = DaemonResponse::ok(req.id);
            r.message = Some(format!("Session {} deleted", ses_id));
            r
        } else {
            DaemonResponse::error(req.id, format!("Session not found: {}", ses_id))
        }
    }

    fn handle_snapshot(&mut self, req: &DaemonRequest) -> DaemonResponse {
        let ses_id = match &req.session {
            Some(s) => s.clone(),
            None => return DaemonResponse::error(req.id, "Missing session_id"),
        };

        match self.sessions.get_source(&ses_id) {
            Some(source) => {
                let mut r = DaemonResponse::ok(req.id);
                r.session = Some(ses_id);
                r.value = Some(source);
                r
            }
            None => DaemonResponse::error(req.id, format!("Session not found: {}", ses_id)),
        }
    }

    fn handle_list(&mut self, _req: &DaemonRequest) -> DaemonResponse {
        let mut r = DaemonResponse::ok(_req.id);
        r.data = Some(serde_json::json!({
            "session_count": self.sessions.session_count(),
        }));
        r
    }
}
