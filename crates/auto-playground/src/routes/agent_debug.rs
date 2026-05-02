use crate::agent_debug::controller::{
    AgentDebugCommand, AgentDebugState, AgentDebugStatus,
};
use crate::agent_debug::session::AgentDebugSession;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Shared state holding all active agent debug sessions.
pub type AgentDebugSessions = Arc<Mutex<HashMap<String, AgentDebugSession>>>;

/// Maximum lifetime of an idle session before automatic cleanup.
const SESSION_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour

// -------------------------------------------------------------------------
// Request / Response DTOs
// -------------------------------------------------------------------------

#[derive(serde::Deserialize)]
pub struct StartRequest {
    pub source: String,
}

#[derive(serde::Serialize)]
pub struct StartResponse {
    pub session_id: String,
    pub bytecode: Vec<serde_json::Value>,
}

#[derive(serde::Deserialize)]
pub struct BreakpointsRequest {
    pub lines: Vec<u32>,
}

#[derive(serde::Deserialize)]
pub struct CommandRequest {
    pub cmd: String,
}

#[derive(serde::Serialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub status: String,
    pub line: u32,
    pub ip: usize,
    pub created_at_secs: u64,
    pub idle_secs: u64,
}

#[derive(serde::Serialize)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionSummary>,
}

// -------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------

/// Remove expired sessions to prevent memory leaks.
pub fn cleanup_expired_sessions(sessions: &AgentDebugSessions) {
    let now = Instant::now();
    let mut sessions = sessions.lock().unwrap();
    let expired: Vec<String> = sessions
        .iter()
        .filter(|(_, s)| now.duration_since(s.created_at) > SESSION_TIMEOUT)
        .map(|(id, _)| id.clone())
        .collect();
    for id in &expired {
        if let Some(session) = sessions.get(id) {
            let _ = session.cmd_tx.send(AgentDebugCommand::Stop);
        }
        sessions.remove(id);
        tracing::info!("Agent debug session {}: expired and removed", id);
    }
}

// -------------------------------------------------------------------------
// Handlers
// -------------------------------------------------------------------------

/// POST /api/agent-debug/start
/// Creates a new agent debug session, compiles the source, and starts the VM
/// in a dedicated thread.  The VM pauses at the first instruction waiting for
/// a command.
pub async fn start_handler(
    State(sessions): State<AgentDebugSessions>,
    Json(req): Json<StartRequest>,
) -> Result<Json<StartResponse>, (StatusCode, String)> {
    cleanup_expired_sessions(&sessions);

    let session_id = uuid::Uuid::new_v4().to_string();
    tracing::info!("Agent debug session {}: starting", session_id);

    let (session, bytecode) =
        AgentDebugSession::spawn(session_id.clone(), req.source).map_err(|e| {
            tracing::error!("Agent debug session {}: compile error: {}", session_id, e);
            (StatusCode::BAD_REQUEST, e)
        })?;

    sessions.lock().unwrap().insert(session_id.clone(), session);

    Ok(Json(StartResponse {
        session_id,
        bytecode,
    }))
}

/// GET /api/agent-debug/sessions
/// Lists all active sessions with a brief summary.
pub async fn sessions_handler(
    State(sessions): State<AgentDebugSessions>,
) -> Json<SessionsResponse> {
    cleanup_expired_sessions(&sessions);

    let sessions = sessions.lock().unwrap();
    let now = Instant::now();
    let summaries: Vec<SessionSummary> = sessions
        .iter()
        .map(|(id, session)| {
            let state = session.state_rx.borrow();
            SessionSummary {
                session_id: id.clone(),
                status: match state.status {
                    AgentDebugStatus::Paused => "paused".to_string(),
                    AgentDebugStatus::Running => "running".to_string(),
                    AgentDebugStatus::Finished => "finished".to_string(),
                    AgentDebugStatus::Error => "error".to_string(),
                },
                line: state.line,
                ip: state.ip,
                created_at_secs: session.created_at.elapsed().as_secs(),
                idle_secs: now.duration_since(session.created_at).as_secs(),
            }
        })
        .collect();

    Json(SessionsResponse { sessions: summaries })
}

/// POST /api/agent-debug/{id}/breakpoints
/// Sets (replaces) the breakpoints for the given session.
pub async fn breakpoints_handler(
    Path(id): Path<String>,
    State(sessions): State<AgentDebugSessions>,
    Json(req): Json<BreakpointsRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let sessions = sessions.lock().unwrap();
    let session = sessions
        .get(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Session {} not found", id)))?;

    let lines = req.lines;
    let mut bp = session.breakpoints.lock().unwrap();
    bp.clear();
    bp.extend(lines.iter().cloned());
    tracing::debug!("Agent debug session {}: breakpoints set to {:?}", id, lines);

    Ok(StatusCode::OK)
}

/// POST /api/agent-debug/{id}/command
/// Sends a command and **blocks** (up to a generous timeout) until the VM
/// pauses again or finishes.  Returns the new state.
pub async fn command_handler(
    Path(id): Path<String>,
    State(sessions): State<AgentDebugSessions>,
    Json(req): Json<CommandRequest>,
) -> Result<Json<AgentDebugState>, (StatusCode, String)> {
    let session = {
        let sessions = sessions.lock().unwrap();
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Session {} not found", id)))?
    };

    let cmd = match req.cmd.as_str() {
        "continue" => AgentDebugCommand::Continue,
        "step" => AgentDebugCommand::Step,
        "step_over" | "next" => AgentDebugCommand::StepOver,
        "step_out" | "finish" => AgentDebugCommand::StepOut,
        "stop" => AgentDebugCommand::Stop,
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Unknown command: {}", other),
            ))
        }
    };

    // Clone a fresh receiver and sync it to the latest state already sent by
    // the controller.  This is crucial because `watch::changed` returns
    // immediately if the value has already changed from the receiver's last
    // seen value — without this sync step we could get a stale Paused state
    // from the initial VM pause instead of the state after our command.
    let mut state_rx = session.state_rx.clone();
    if state_rx.changed().await.is_err() {
        return Ok(Json(state_rx.borrow().clone()));
    }

    // If the VM has already finished, there's nothing to command.
    let current = state_rx.borrow().clone();
    if matches!(current.status, AgentDebugStatus::Finished | AgentDebugStatus::Error) {
        return Ok(Json(current));
    }

    // Send command to the VM thread.
    session
        .cmd_tx
        .send(cmd)
        .map_err(|_| (StatusCode::GONE, "VM thread has exited".to_string()))?;

    // Wait for the state to change again (VM pauses or finishes).
    let timeout = tokio::time::Duration::from_secs(30);
    let result = tokio::time::timeout(timeout, async {
        if state_rx.changed().await.is_err() {
            return state_rx.borrow().clone();
        }
        state_rx.borrow().clone()
    })
    .await;

    match result {
        Ok(state) => Ok(Json(state)),
        Err(_) => Err((
            StatusCode::REQUEST_TIMEOUT,
            "VM did not respond within 30 seconds".to_string(),
        )),
    }
}

/// GET /api/agent-debug/{id}/state
/// Returns the **current** state without blocking.  Useful for polling or
/// checking whether a session is still alive.
pub async fn state_handler(
    Path(id): Path<String>,
    State(sessions): State<AgentDebugSessions>,
) -> Result<Json<AgentDebugState>, (StatusCode, String)> {
    let sessions = sessions.lock().unwrap();
    let session = sessions
        .get(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Session {} not found", id)))?;

    let state = session.state_rx.borrow().clone();
    Ok(Json(state))
}

/// DELETE /api/agent-debug/{id}
/// Stops the session (sends Stop command) and removes it from the registry.
pub async fn delete_handler(
    Path(id): Path<String>,
    State(sessions): State<AgentDebugSessions>,
) -> Result<StatusCode, (StatusCode, String)> {
    let session = {
        let sessions = sessions.lock().unwrap();
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Session {} not found", id)))?
    };

    let _ = session.cmd_tx.send(AgentDebugCommand::Stop);

    sessions.lock().unwrap().remove(&id);
    tracing::info!("Agent debug session {}: deleted", id);

    Ok(StatusCode::OK)
}
