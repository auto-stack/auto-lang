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

// ── Pipe Address ────────────────────────────────────────────────────

pub fn pipe_addr(name: &str) -> String {
    #[cfg(windows)]
    { format!(r"\\.\pipe\{}", name) }
    #[cfg(unix)]
    { format!("/tmp/{}.sock", name) }
}

/// Try to connect to a running daemon's pipe. Returns the named pipe stream.
/// Used by `autovm_client` to check if a daemon is already running.
#[cfg(windows)]
pub async fn connect_to_pipe(pipe_name: &str) -> Result<tokio::net::windows::named_pipe::NamedPipeClient, String> {
    use tokio::net::windows::named_pipe::ClientOptions;
    let addr = pipe_addr(pipe_name);
    ClientOptions::new().open(&addr).map_err(|e| format!("Pipe connect failed: {}", e))
}

#[cfg(unix)]
pub async fn connect_to_pipe(pipe_name: &str) -> Result<tokio::net::UnixStream, String> {
    let addr = pipe_addr(pipe_name);
    tokio::net::UnixStream::connect(&addr).await.map_err(|e| format!("Socket connect failed: {}", e))
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
    pub fn run_stdio(&mut self) {
        eprintln!("AutoVM daemon: stdio mode started");
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let mut locked = stdin.lock();

        loop {
            let mut line = String::new();
            match locked.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => {
                    eprintln!("AutoVM daemon: read error: {}", e);
                    break;
                }
            }

            let line = line.trim();
            if line.is_empty() { continue; }

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

    /// Run daemon listening on a named pipe (Windows) or Unix socket (Unix).
    /// Multiple `auto req` clients connect to the same daemon to share sessions.
    /// Connections are handled sequentially (one at a time) since VM execution
    /// is inherently single-threaded and VmSession contains !Send types.
    pub fn run_pipe(mut self, pipe_name: &str) {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime for daemon");
        let pipe_path = pipe_addr(pipe_name);

        eprintln!("AutoVM daemon: listening on {}", pipe_path);

        // We use a two-thread approach:
        // Thread 1 (tokio): accept connections, read/write JSON lines
        // Thread 2 (main): handle_request (synchronous VM execution, no runtime nesting)
        let (conn_tx, conn_rx) = std::sync::mpsc::channel::<(String, std::sync::mpsc::Sender<String>)>();

        // Thread: tokio accept + I/O
        let pipe_path_clone = pipe_path.clone();
        let io_thread = std::thread::spawn(move || {
            rt.block_on(async move {
                #[cfg(unix)]
                let _ = std::fs::remove_file(&pipe_path_clone);

                loop {
                    let stream = accept_connection(&pipe_path_clone).await;
                    eprintln!("AutoVM daemon: client connected");

                    let (read, write) = tokio::io::split(stream);
                    let mut reader = tokio::io::BufReader::new(read);
                    let mut writer = tokio::io::BufWriter::new(write);
                    let mut line = String::new();

                    loop {
                        line.clear();
                        let n = tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line).await;
                        match n {
                            Ok(0) => break,
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("AutoVM daemon: read error: {}", e);
                                break;
                            }
                        }

                        let trimmed = line.trim();
                        if trimmed.is_empty() { continue; }

                        // Send request to handler thread, wait for response
                        let (resp_tx, resp_rx) = std::sync::mpsc::channel::<String>();
                        if conn_tx.send((trimmed.to_string(), resp_tx)).is_err() {
                            break; // handler thread died
                        }
                        match resp_rx.recv() {
                            Ok(json) => {
                                let _ = tokio::io::AsyncWriteExt::write_all(&mut writer, json.as_bytes()).await;
                                let _ = tokio::io::AsyncWriteExt::write_all(&mut writer, b"\n").await;
                                let _ = tokio::io::AsyncWriteExt::flush(&mut writer).await;
                            }
                            Err(_) => break,
                        }
                    }
                    eprintln!("AutoVM daemon: client disconnected");
                }
            });
        });

        // Main thread: handle requests (no tokio runtime, safe for VM execution)
        while let Ok((line, resp_tx)) = conn_rx.recv() {
            let req = match serde_json::from_str::<DaemonRequest>(&line) {
                Ok(r) => r,
                Err(e) => {
                    let resp = DaemonResponse::error(0, format!("Invalid request: {}", e));
                    let _ = resp_tx.send(serde_json::to_string(&resp).unwrap());
                    continue;
                }
            };

            let resp = self.handle_request(req);
            let _ = resp_tx.send(serde_json::to_string(&resp).unwrap());
        }

        let _ = io_thread.join();
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

// ── Platform: Accept Connection ─────────────────────────────────────

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
    let listener = LISTENER.get_or_init(|| {
        UnixListener::bind(socket_path).expect("Failed to bind Unix socket")
    });
    listener.accept().await.expect("Accept failed").0
}
