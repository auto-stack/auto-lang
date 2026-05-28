// AutoVM Client — connects to `auto serve` daemon via stdin/stdout pipe
//
// Plan 269: Used by `auto req` CLI subcommand to send requests to daemon.

use crate::autovm_daemon::{DaemonRequest, DaemonResponse};
use std::io::{BufRead, BufReader, Write};

/// Send a request to the daemon and read the response.
/// In stdio mode, this spawns a child process running `auto serve --stdio`
/// and communicates over its stdin/stdout.
pub struct AutovmClient {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl AutovmClient {
    /// Connect to a running daemon by spawning a pipe to it.
    /// For now, uses `auto serve --stdio` subprocess mode.
    pub fn connect() -> Result<Self, String> {
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

    /// Create a new session. Returns the session ID.
    pub fn new_session(&mut self) -> Result<String, String> {
        let resp = self.call("new-session", None, None)?;
        if resp.status != "ok" {
            return Err(resp.message.unwrap_or_else(|| "Unknown error".into()));
        }
        resp.session.ok_or_else(|| "No session_id in response".into())
    }

    /// Evaluate code in a session.
    pub fn eval(&mut self, session: &str, code: &str) -> Result<DaemonResponse, String> {
        self.call("eval", Some(session), Some(code))
    }

    /// Inspect a session.
    pub fn inspect(&mut self, session: &str) -> Result<DaemonResponse, String> {
        self.call("inspect", Some(session), None)
    }

    /// Reset a session.
    pub fn reset(&mut self, session: &str) -> Result<DaemonResponse, String> {
        self.call("reset", Some(session), None)
    }

    /// Delete a session.
    pub fn delete(&mut self, session: &str) -> Result<DaemonResponse, String> {
        self.call("delete", Some(session), None)
    }

    /// Snapshot a session.
    pub fn snapshot(&mut self, session: &str) -> Result<DaemonResponse, String> {
        self.call("snapshot", Some(session), None)
    }

    /// List sessions.
    pub fn list(&mut self) -> Result<DaemonResponse, String> {
        self.call("list", None, None)
    }
}

impl Drop for AutovmClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ── One-shot mode (no daemon needed) ────────────────────────────────

/// Execute code in a temporary session. Creates session, runs code, deletes session.
/// Returns (value_string, is_ok).
pub fn eval_one_shot(code: &str) -> (String, bool) {
    let mut client = match AutovmClient::connect() {
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
