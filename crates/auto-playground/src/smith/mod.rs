//! AutoSmith — Spec-driven serial agent orchestration
//!
//! This module adds Forge (chat loop), Ledger (knowledge management),
//! and Relay (agent pipeline) endpoints to the auto-playground server.
//! It reuses the existing NotebookActor for VM session sharing with AutoLab.

use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::{Arc, Mutex, OnceLock};

use crate::notebook::ai::{AIProviderState, AIRequest, AIStreamDelta};

// Re-export handler functions so they can be mounted in main.rs
pub use self::handlers::*;

// ─── Forge Session Store (in-memory; replace with persistent store later) ─────

fn forge_sessions() -> &'static Mutex<std::collections::HashMap<String, ForgeSession>> {
    static SESSIONS: OnceLock<Mutex<std::collections::HashMap<String, ForgeSession>>> =
        OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeSession {
    pub id: String,
    pub notebook_sid: Option<String>,
    pub project_path: String,
    pub status: ForgeStatus,
    pub messages: Vec<ForgeMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForgeStatus {
    Idle,
    Thinking,
    ToolCall,
    WaitingApproval,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateForgeSessionRequest {
    pub notebook_sid: Option<String>,
    pub project_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeMessageResponse {
    pub message: ForgeMessage,
}

/// SSE event types sent to the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ForgeStreamEvent {
    #[serde(rename = "delta")]
    Delta { text: String },
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "error")]
    Error { message: String },
}

// ─── Ledger Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerDocument {
    pub project: String,
    pub sections: Vec<LedgerSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerSection {
    pub id: String,
    pub section_type: String,
    pub title: String,
    pub status: String,
    pub content: String,
    pub depends_on: Vec<String>,
    pub last_modified: u64,
    pub last_verified: Option<u64>,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

mod handlers {
    use super::*;

    pub async fn create_forge_session(
        Json(req): Json<CreateForgeSessionRequest>,
    ) -> Json<ForgeSession> {
        let sid = format!("forge-{}", uuid::Uuid::new_v4());
        let session = ForgeSession {
            id: sid.clone(),
            notebook_sid: req.notebook_sid,
            project_path: req.project_path.unwrap_or_else(|| String::from(".")),
            status: ForgeStatus::Idle,
            messages: vec![ForgeMessage {
                id: format!("m-{}", uuid::Uuid::new_v4()),
                role: String::from("system"),
                content: String::from(
                    "You are AutoSmith Forge, a spec-driven AI coding assistant. \
                     Help the user build software by understanding requirements, \
                     proposing specs, and generating code.",
                ),
                timestamp: now_secs(),
            }],
        };

        forge_sessions().lock().unwrap().insert(sid, session.clone());
        Json(session)
    }

    pub async fn send_forge_message(
        Path(sid): Path<String>,
        Json(req): Json<SendMessageRequest>,
    ) -> Json<ForgeMessageResponse> {
        let user_msg = ForgeMessage {
            id: format!("m-{}", uuid::Uuid::new_v4()),
            role: String::from("user"),
            content: req.content,
            timestamp: now_secs(),
        };

        // Store user message in session
        if let Ok(mut sessions) = forge_sessions().lock() {
            if let Some(session) = sessions.get_mut(&sid) {
                session.messages.push(user_msg.clone());
                session.status = ForgeStatus::Thinking;
            }
        }

        // Return placeholder; content streams via SSE
        let assistant_msg = ForgeMessage {
            id: format!("m-{}", uuid::Uuid::new_v4()),
            role: String::from("assistant"),
            content: String::new(),
            timestamp: now_secs(),
        };

        Json(ForgeMessageResponse { message: assistant_msg })
    }

    pub async fn forge_stream(
        Path(sid): Path<String>,
        State(ai): State<AIProviderState>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let (event_tx, mut event_rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();

        tokio::spawn(async move {
            // Build AI request from session history
            let request = build_ai_request(&sid);

            let (delta_tx, mut delta_rx) = tokio::sync::mpsc::unbounded_channel::<AIStreamDelta>();

            // Spawn the AI streaming call
            let ai_clone = ai.clone();
            let ai_task = tokio::spawn(async move {
                let error = ai_clone.chat_stream(request, delta_tx).await;
                error
            });

            // Bridge AI deltas → SSE events
            while let Some(delta) = delta_rx.recv().await {
                let event = Event::default().data(
                    serde_json::to_string(&ForgeStreamEvent::Delta { text: delta.text }).unwrap(),
                );
                let _ = event_tx.send(Ok(event));
            }

            // Wait for AI task to finish and check for errors
            match ai_task.await {
                Ok(Some(err)) => {
                    let event = Event::default().data(
                        serde_json::to_string(&ForgeStreamEvent::Error { message: err }).unwrap(),
                    );
                    let _ = event_tx.send(Ok(event));
                }
                _ => {
                    let event = Event::default().data(
                        serde_json::to_string(&ForgeStreamEvent::Done).unwrap(),
                    );
                    let _ = event_tx.send(Ok(event));
                }
            }

            // Mark session as idle
            if let Ok(mut sessions) = forge_sessions().lock() {
                if let Some(session) = sessions.get_mut(&sid) {
                    session.status = ForgeStatus::Idle;
                }
            }
        });

        let sse_stream = stream::unfold(event_rx, |mut rx| async move {
            rx.recv().await.map(|event| (event, rx))
        });

        Sse::new(sse_stream).keep_alive(KeepAlive::default())
    }

    pub async fn forge_history(Path(sid): Path<String>) -> Json<Vec<ForgeMessage>> {
        let sessions = forge_sessions().lock().unwrap();
        let messages = sessions
            .get(&sid)
            .map(|s| s.messages.clone())
            .unwrap_or_default();
        Json(messages)
    }

    // ─── Ledger Handlers ─────────────────────────────────────────────────

    pub async fn get_ledger() -> Json<LedgerDocument> {
        Json(LedgerDocument {
            project: String::from("auto-playground"),
            sections: vec![LedgerSection {
                id: String::from("goals"),
                section_type: String::from("goals"),
                title: String::from("Goals"),
                status: String::from("in_progress"),
                content: String::from("- Implement user authentication\n- Add JWT token flow"),
                depends_on: vec![],
                last_modified: 0,
                last_verified: None,
            }],
        })
    }

    pub async fn update_ledger(Json(_doc): Json<LedgerDocument>) -> Json<LedgerDocument> {
        get_ledger().await
    }

    pub async fn trigger_drift_check() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "status": "ok",
            "drift_detected": false,
            "sections_checked": 7,
        }))
    }

    // ─── Relay Handlers ──────────────────────────────────────────────────

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RunRequest {
        pub task: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RunInfo {
        pub id: String,
        pub task: String,
        pub status: String,
    }

    pub async fn start_run(Json(req): Json<RunRequest>) -> Json<RunInfo> {
        Json(RunInfo {
            id: format!("run-{}", uuid::Uuid::new_v4()),
            task: req.task,
            status: String::from("started"),
        })
    }

    pub async fn list_runs() -> Json<Vec<RunInfo>> {
        Json(vec![])
    }

    // ─── Helpers ─────────────────────────────────────────────────────────

    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn build_ai_request(sid: &str) -> AIRequest {
        let sessions = forge_sessions().lock().unwrap();
        let session = match sessions.get(sid) {
            Some(s) => s,
            None => {
                return AIRequest {
                    prompt: String::from("Hello."),
                    context: None,
                };
            }
        };

        // Build conversation context from message history
        let mut context = String::new();
        for msg in &session.messages {
            match msg.role.as_str() {
                "system" => context.push_str(&format!("System: {}\n", msg.content)),
                "user" => context.push_str(&format!("User: {}\n", msg.content)),
                "assistant" => context.push_str(&format!("Assistant: {}\n", msg.content)),
                _ => {}
            }
        }

        // The last message is the user's prompt; everything before is context
        let prompt = session
            .messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_else(|| String::from("Continue."));

        AIRequest {
            prompt,
            context: if context.is_empty() { None } else { Some(context) },
        }
    }
}

// Non-generic route builder — caller must provide state that can produce AIProviderState
pub fn routes() -> Router<crate::AppState> {
    Router::new()
        // Forge
        .route("/api/smith/forge/session", post(handlers::create_forge_session))
        .route("/api/smith/forge/{sid}/message", post(handlers::send_forge_message))
        .route("/api/smith/forge/{sid}/stream", get(handlers::forge_stream))
        .route("/api/smith/forge/{sid}/history", get(handlers::forge_history))
        // Ledger
        .route("/api/smith/ledger", get(handlers::get_ledger).put(handlers::update_ledger))
        .route("/api/smith/ledger/drift-check", post(handlers::trigger_drift_check))
        // Relay
        .route("/api/smith/relay/run", post(handlers::start_run))
        .route("/api/smith/relay/runs", get(handlers::list_runs))
}
