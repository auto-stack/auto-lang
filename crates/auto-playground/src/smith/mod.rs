//! AutoSmith — Spec-driven serial agent orchestration
//!
//! This module adds Forge (chat loop), Ledger (knowledge management),
//! and Relay (agent pipeline) endpoints to the auto-playground server.
//! It reuses the existing NotebookActor for VM session sharing with AutoLab.

use axum::{
    extract::Path,
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;

// ─── Forge Types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeSession {
    pub id: String,
    pub notebook_sid: String, // links to AutoLab session
    pub project_path: String,
    pub status: ForgeStatus,
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
    pub role: String, // "user" | "assistant" | "system" | "tool"
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
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
    pub section_type: String, // "goals" | "requirements" | ...
    pub title: String,
    pub status: String, // "draft" | "approved" | "in_progress" | "verified" | "drift" | "archived"
    pub content: String,
    pub depends_on: Vec<String>,
    pub last_modified: u64,
    pub last_verified: Option<u64>,
}

// ─── API Routes ──────────────────────────────────────────────────────────────

pub fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        // Forge
        .route("/api/smith/forge/session", post(create_forge_session))
        .route("/api/smith/forge/{sid}/message", post(send_forge_message))
        .route("/api/smith/forge/{sid}/stream", get(forge_stream))
        // Ledger
        .route("/api/smith/ledger", get(get_ledger).put(update_ledger))
        .route("/api/smith/ledger/drift-check", post(trigger_drift_check))
        // Relay
        .route("/api/smith/relay/run", post(start_run))
        .route("/api/smith/relay/runs", get(list_runs))
}

// ─── Handlers (scaffold) ─────────────────────────────────────────────────────

async fn create_forge_session() -> Json<ForgeSession> {
    Json(ForgeSession {
        id: format!("forge-{}", uuid::Uuid::new_v4()),
        notebook_sid: String::new(),
        project_path: String::from("."),
        status: ForgeStatus::Idle,
    })
}

async fn send_forge_message(
    Path(_sid): Path<String>,
    Json(_req): Json<SendMessageRequest>,
) -> Json<ForgeMessage> {
    Json(ForgeMessage {
        id: format!("m-{}", uuid::Uuid::new_v4()),
        role: String::from("assistant"),
        content: String::from(
            "AutoSmith Forge is scaffolded but not yet wired to the AI provider. \
             This endpoint will stream responses via SSE.",
        ),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    })
}

async fn forge_stream(
    Path(_sid): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    use futures::stream::{self, StreamExt};
    let events = stream::iter(vec![
        Ok(Event::default().data(r#"{"type":"delta","text":"AutoSmith "}"#)),
        Ok(Event::default().data(r#"{"type":"delta","text":"Forge "}"#)),
        Ok(Event::default().data(r#"{"type":"delta","text":"scaffolded."}"#)),
        Ok(Event::default().data(r#"{"type":"done"}"#)),
    ]);
    Sse::new(events)
}

async fn get_ledger() -> Json<LedgerDocument> {
    Json(LedgerDocument {
        project: String::from("auto-playground"),
        sections: vec![
            LedgerSection {
                id: String::from("goals"),
                section_type: String::from("goals"),
                title: String::from("Goals"),
                status: String::from("in_progress"),
                content: String::from("- Implement user authentication\n- Add JWT token flow"),
                depends_on: vec![],
                last_modified: 0,
                last_verified: None,
            },
        ],
    })
}

async fn update_ledger(Json(_doc): Json<LedgerDocument>) -> Json<LedgerDocument> {
    // TODO: validate diff, persist to .autosmith/ledger.ad
    get_ledger().await
}

async fn trigger_drift_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "drift_detected": false,
        "sections_checked": 7,
    }))
}

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

async fn start_run(Json(req): Json<RunRequest>) -> Json<RunInfo> {
    Json(RunInfo {
        id: format!("run-{}", uuid::Uuid::new_v4()),
        task: req.task,
        status: String::from("started"),
    })
}

async fn list_runs() -> Json<Vec<RunInfo>> {
    Json(vec![])
}
