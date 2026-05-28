// AutoVM Client — connects to `auto serve` daemon via named pipe or stdio
//
// Plan 269: Used by `auto req` CLI subcommand to send requests to daemon.

use crate::autovm_daemon::{DaemonRequest, DaemonResponse};
use std::io::{BufRead, BufReader, Write};

// ── Pipe Client (connects to running daemon via named pipe) ─────────

/// Client connected to a running daemon via named pipe.
/// Multiple `auto req` invocations can share the same daemon and sessions.
pub struct AutovmPipeClient {
    runtime: tokio::runtime::Runtime,
    writer: tokio::io::BufWriter<tokio::io::WriteHalf<PipeStream>>,
    reader: tokio::io::BufReader<tokio::io::ReadHalf<PipeStream>>,
    next_id: u64,
}

#[cfg(windows)]
type PipeStream = tokio::net::windows::named_pipe::NamedPipeClient;
#[cfg(unix)]
type PipeStream = tokio::net::UnixStream;

impl AutovmPipeClient {
    /// Connect to a running daemon via named pipe.
    pub fn connect(pipe_name: &str) -> Result<Self, String> {
        let runtime = tokio::runtime::Runtime::new().map_err(|e| format!("Runtime error: {}", e))?;
        let stream = runtime.block_on(crate::autovm_daemon::connect_to_pipe(pipe_name))?;
        let (read, write) = tokio::io::split(stream);
        Ok(Self {
            runtime,
            writer: tokio::io::BufWriter::new(write),
            reader: tokio::io::BufReader::new(read),
            next_id: 1,
        })
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn send(&mut self, req: &DaemonRequest) -> Result<(), String> {
        let json = serde_json::to_string(req).map_err(|e| format!("Serialize error: {}", e))?;
        self.runtime.block_on(async {
            use tokio::io::AsyncWriteExt;
            self.writer.write_all(json.as_bytes()).await.map_err(|e| format!("Write error: {}", e))?;
            self.writer.write_all(b"\n").await.map_err(|e| format!("Write error: {}", e))?;
            self.writer.flush().await.map_err(|e| format!("Flush error: {}", e))
        })
    }

    fn recv(&mut self) -> Result<DaemonResponse, String> {
        let mut line = String::new();
        self.runtime.block_on(async {
            use tokio::io::AsyncBufReadExt;
            self.reader.read_line(&mut line).await.map_err(|e| format!("Read error: {}", e))
        })?;
        let line = line.trim();
        if line.is_empty() {
            return Err("Empty response from daemon".into());
        }
        serde_json::from_str(line).map_err(|e| format!("Parse error: {} (input: {})", e, line))
    }

    fn call(&mut self, method: &str, session: Option<&str>, code: Option<&str>) -> Result<DaemonResponse, String> {
        let id = self.next_id();
        let req = DaemonRequest {
            id,
            session: session.map(|s| s.to_string()),
            method: method.to_string(),
            code: code.map(|c| c.to_string()),
        };
        self.send(&req)?;
        self.recv()
    }

    pub fn new_session(&mut self) -> Result<String, String> {
        let resp = self.call("new-session", None, None)?;
        if resp.status != "ok" {
            return Err(resp.message.unwrap_or_else(|| "Unknown error".into()));
        }
        resp.session.ok_or_else(|| "No session_id in response".into())
    }

    pub fn eval(&mut self, session: &str, code: &str) -> Result<DaemonResponse, String> {
        self.call("eval", Some(session), Some(code))
    }

    pub fn inspect(&mut self, session: &str) -> Result<DaemonResponse, String> {
        self.call("inspect", Some(session), None)
    }

    pub fn reset(&mut self, session: &str) -> Result<DaemonResponse, String> {
        self.call("reset", Some(session), None)
    }

    pub fn delete(&mut self, session: &str) -> Result<DaemonResponse, String> {
        self.call("delete", Some(session), None)
    }

    pub fn snapshot(&mut self, session: &str) -> Result<DaemonResponse, String> {
        self.call("snapshot", Some(session), None)
    }

    pub fn list(&mut self) -> Result<DaemonResponse, String> {
        self.call("list", None, None)
    }
}

// ── Stdio Client (spawns daemon subprocess) ─────────────────────────

/// Client that spawns a private daemon subprocess (stdio mode).
/// Used as fallback when no named pipe daemon is running.
pub struct AutovmStdioClient {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl AutovmStdioClient {
    /// Spawn a private daemon subprocess via stdio.
    pub fn spawn() -> Result<Self, String> {
        let exe = std::env::current_exe().map_err(|e| format!("Cannot find auto executable: {}", e))?;
        let mut child = std::process::Command::new(exe)
            .args(["serve", "--stdio"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start daemon: {}", e))?;

        let stdin = child.stdin.take().ok_or("Failed to get daemon stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get daemon stdout")?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
        })
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn send(&mut self, req: &DaemonRequest) -> Result<(), String> {
        let json = serde_json::to_string(req).map_err(|e| format!("Serialize error: {}", e))?;
        writeln!(self.stdin, "{}", json).map_err(|e| format!("Write error: {}", e))?;
        self.stdin.flush().map_err(|e| format!("Flush error: {}", e))?;
        Ok(())
    }

    fn recv(&mut self) -> Result<DaemonResponse, String> {
        let mut line = String::new();
        self.stdout.read_line(&mut line).map_err(|e| format!("Read error: {}", e))?;
        let line = line.trim();
        if line.is_empty() {
            return Err("Empty response from daemon".into());
        }
        serde_json::from_str(line).map_err(|e| format!("Parse error: {} (input: {})", e, line))
    }

    fn call(&mut self, method: &str, session: Option<&str>, code: Option<&str>) -> Result<DaemonResponse, String> {
        let id = self.next_id();
        let req = DaemonRequest {
            id,
            session: session.map(|s| s.to_string()),
            method: method.to_string(),
            code: code.map(|c| c.to_string()),
        };
        self.send(&req)?;
        self.recv()
    }

    pub fn new_session(&mut self) -> Result<String, String> {
        let resp = self.call("new-session", None, None)?;
        if resp.status != "ok" {
            return Err(resp.message.unwrap_or_else(|| "Unknown error".into()));
        }
        resp.session.ok_or_else(|| "No session_id in response".into())
    }

    pub fn eval(&mut self, session: &str, code: &str) -> Result<DaemonResponse, String> {
        self.call("eval", Some(session), Some(code))
    }

    pub fn delete(&mut self, session: &str) -> Result<DaemonResponse, String> {
        self.call("delete", Some(session), None)
    }
}

impl Drop for AutovmStdioClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ── Unified Client (tries pipe first, falls back to stdio) ──────────

/// Auto-select transport: connect to named pipe daemon if running,
/// otherwise spawn a private stdio daemon.
pub enum AutovmClient {
    Pipe(AutovmPipeClient),
    Stdio(AutovmStdioClient),
}

impl AutovmClient {
    /// Try connecting to named pipe first, fall back to spawning stdio daemon.
    pub fn connect(pipe_name: &str) -> Result<Self, String> {
        // Try named pipe first
        if let Ok(client) = AutovmPipeClient::connect(pipe_name) {
            return Ok(AutovmClient::Pipe(client));
        }
        // Fallback: spawn stdio subprocess
        let client = AutovmStdioClient::spawn()?;
        Ok(AutovmClient::Stdio(client))
    }

    /// Connect pipe-only (error if no daemon running).
    pub fn connect_pipe_only(pipe_name: &str) -> Result<AutovmPipeClient, String> {
        AutovmPipeClient::connect(pipe_name)
    }

    fn next_id(&mut self) -> u64 {
        match self {
            AutovmClient::Pipe(c) => c.next_id(),
            AutovmClient::Stdio(c) => c.next_id(),
        }
    }

    pub fn new_session(&mut self) -> Result<String, String> {
        match self {
            AutovmClient::Pipe(c) => c.new_session(),
            AutovmClient::Stdio(c) => c.new_session(),
        }
    }

    pub fn eval(&mut self, session: &str, code: &str) -> Result<DaemonResponse, String> {
        match self {
            AutovmClient::Pipe(c) => c.eval(session, code),
            AutovmClient::Stdio(c) => c.eval(session, code),
        }
    }

    pub fn inspect(&mut self, session: &str) -> Result<DaemonResponse, String> {
        match self {
            AutovmClient::Pipe(c) => c.inspect(session),
            AutovmClient::Stdio(c) => c.call("inspect", Some(session), None),
        }
    }

    pub fn reset(&mut self, session: &str) -> Result<DaemonResponse, String> {
        match self {
            AutovmClient::Pipe(c) => c.reset(session),
            AutovmClient::Stdio(c) => c.call("reset", Some(session), None),
        }
    }

    pub fn delete(&mut self, session: &str) -> Result<DaemonResponse, String> {
        match self {
            AutovmClient::Pipe(c) => c.delete(session),
            AutovmClient::Stdio(c) => c.delete(session),
        }
    }

    pub fn snapshot(&mut self, session: &str) -> Result<DaemonResponse, String> {
        match self {
            AutovmClient::Pipe(c) => c.snapshot(session),
            AutovmClient::Stdio(c) => c.call("snapshot", Some(session), None),
        }
    }

    pub fn list(&mut self) -> Result<DaemonResponse, String> {
        match self {
            AutovmClient::Pipe(c) => c.list(),
            AutovmClient::Stdio(c) => c.call("list", None, None),
        }
    }
}

// ── One-shot mode (no daemon needed) ────────────────────────────────

/// Execute code in a temporary session. Creates session, runs code, deletes session.
/// Returns (value_string, is_ok).
pub fn eval_one_shot(code: &str) -> (String, bool) {
    // Try pipe first (reuse running daemon)
    if let Ok(mut client) = AutovmPipeClient::connect("autovm") {
        if let Ok(ses_id) = client.new_session() {
            match client.eval(&ses_id, code) {
                Ok(resp) => {
                    let _ = client.delete(&ses_id);
                    if resp.status == "ok" {
                        return (resp.value.unwrap_or_default(), true);
                    } else {
                        return (resp.message.unwrap_or_else(|| "Unknown error".into()), false);
                    }
                }
                Err(e) => {
                    let _ = client.delete(&ses_id);
                    return (format!("Error: {}", e), false);
                }
            }
        }
    }

    // Fallback: spawn stdio daemon
    let mut client = match AutovmStdioClient::spawn() {
        Ok(c) => c,
        Err(e) => return (format!("Error: {}", e), false),
    };

    let ses_id = match client.new_session() {
        Ok(s) => s,
        Err(e) => return (format!("Error creating session: {}", e), false),
    };

    match client.eval(&ses_id, code) {
        Ok(resp) => {
            let _ = client.delete(&ses_id);
            if resp.status == "ok" {
                (resp.value.unwrap_or_default(), true)
            } else {
                (resp.message.unwrap_or_else(|| "Unknown error".into()), false)
            }
        }
        Err(e) => {
            let _ = client.delete(&ses_id);
            (format!("Error: {}", e), false)
        }
    }
}
