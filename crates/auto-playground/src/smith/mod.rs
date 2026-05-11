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
use serde_json::Value;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::notebook::ai::AIProviderState;

mod ai;
mod tools;

pub use self::handlers::*;

// ─── Persistent Session Store ────────────────────────────────────────────────

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    pub result: Option<String>,
    pub status: String,
}

struct SessionStore {
    sessions: std::collections::HashMap<String, ForgeSession>,
    data_dir: PathBuf,
    /// Maps project_path → active_session_id.
    /// Only one session per project may hold the lock at a time.
    project_locks: std::collections::HashMap<String, String>,
}

impl SessionStore {
    fn new() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("autoforge")
            .join("sessions");
        let _ = std::fs::create_dir_all(&data_dir);

        let mut store = Self {
            sessions: std::collections::HashMap::new(),
            data_dir,
            project_locks: std::collections::HashMap::new(),
        };
        store.load_all();
        // Rebuild project locks from loaded sessions (any non-idle session claims its project)
        for (sid, session) in &store.sessions {
            if !matches!(session.status, ForgeStatus::Idle) {
                store.project_locks.insert(session.project_path.clone(), sid.clone());
            }
        }
        store
    }

    fn load_all(&mut self) {
        let Ok(entries) = std::fs::read_dir(&self.data_dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension() != Some("json".as_ref()) {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            let Ok(session) = serde_json::from_str::<ForgeSession>(&content) else { continue };
            self.sessions.insert(session.id.clone(), session);
        }
        tracing::info!("Loaded {} persistent Forge sessions", self.sessions.len());
    }

    fn get(&self, sid: &str) -> Option<&ForgeSession> {
        self.sessions.get(sid)
    }

    fn get_mut(&mut self, sid: &str) -> Option<&mut ForgeSession> {
        self.sessions.get_mut(sid)
    }

    fn insert(&mut self, session: ForgeSession) {
        self.save(&session);
        self.sessions.insert(session.id.clone(), session);
    }

    fn push_message(&mut self, sid: &str, msg: ForgeMessage) {
        let Some(session) = self.sessions.get_mut(sid) else { return };
        session.messages.push(msg);
        let session_clone = session.clone();
        self.save(&session_clone);
    }

    fn update_status(&mut self, sid: &str, status: ForgeStatus) {
        let Some(session) = self.sessions.get_mut(sid) else { return };
        session.status = status;
        let session_clone = session.clone();
        self.save(&session_clone);
    }

    fn save(&self, session: &ForgeSession) {
        let path = self.data_dir.join(format!("{}.json", session.id));
        if let Ok(json) = serde_json::to_string_pretty(session) {
            let _ = std::fs::write(path, json);
        }
    }

    fn list_all(&self) -> Vec<&ForgeSession> {
        self.sessions.values().collect()
    }

    /// Ensure only `sid` is active for its project.
    /// Any other session for the same project is demoted to Idle.
    fn acquire_project_lock(&mut self, sid: &str) {
        let Some(session) = self.sessions.get(sid) else { return };
        let project = session.project_path.clone();
        // Demote previous holder (if any and if different)
        if let Some(prev_sid) = self.project_locks.get(&project) {
            if prev_sid != sid {
                if let Some(prev) = self.sessions.get_mut(prev_sid) {
                    prev.status = ForgeStatus::Idle;
                    let clone = prev.clone();
                    self.save(&clone);
                }
            }
        }
        self.project_locks.insert(project, sid.to_string());
    }

    /// Get the currently active session for a project, if any.
    fn active_session_for(&self, project_path: &str) -> Option<&ForgeSession> {
        let sid = self.project_locks.get(project_path)?;
        self.sessions.get(sid)
    }
}

fn forge_sessions() -> &'static Mutex<SessionStore> {
    static STORE: OnceLock<Mutex<SessionStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(SessionStore::new()))
}

// ─── Request / Response Types ────────────────────────────────────────────────

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
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        name: String,
        arguments: Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        id: String,
        result: String,
    },
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
    use crate::smith::ai::{ChatMessage, ContentBlock, ToolChatEvent, ToolChatRequest, ToolClaudeProvider};
    use crate::smith::tools::ToolRegistry;

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
                tool_calls: None,
            }],
        };

        {
            let mut store = forge_sessions().lock().unwrap();
            store.insert(session.clone());
            store.acquire_project_lock(&sid);
        }
        Json(session)
    }

    pub async fn get_forge_session(Path(sid): Path<String>) -> Json<Option<ForgeSession>> {
        let store = forge_sessions().lock().unwrap();
        Json(store.get(&sid).cloned())
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
            tool_calls: None,
        };

        forge_sessions().lock().unwrap().push_message(&sid, user_msg.clone());

        {
            let mut store = forge_sessions().lock().unwrap();
            if let Some(session) = store.get_mut(&sid) {
                session.status = ForgeStatus::Thinking;
                let session_clone = session.clone();
                store.save(&session_clone);
            }
        }

        let assistant_msg = ForgeMessage {
            id: format!("m-{}", uuid::Uuid::new_v4()),
            role: String::from("assistant"),
            content: String::new(),
            timestamp: now_secs(),
            tool_calls: None,
        };

        Json(ForgeMessageResponse { message: assistant_msg })
    }

    pub async fn forge_stream(
        Path(sid): Path<String>,
        State(ai): State<AIProviderState>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let (event_tx, event_rx) =
            tokio::sync::mpsc::unbounded_channel::<Result<Event, Infallible>>();

        tokio::spawn(async move {
            let registry = ToolRegistry::new();
            let ai_for_turns = ai.clone();
            let provider = ToolClaudeProvider::new(ai);

            // Build conversation messages from session history
            let mut chat_messages = Vec::new();
            {
                let store = forge_sessions().lock().unwrap();
                if let Some(session) = store.get(&sid) {
                    for msg in &session.messages {
                        match msg.role.as_str() {
                            "system" => {
                                // System prompt is handled separately; skip here
                            }
                            "user" => {
                                chat_messages.push(ChatMessage::user(&msg.content));
                            }
                            "assistant" => {
                                if let Some(ref calls) = msg.tool_calls {
                                    let mut blocks = vec![ContentBlock::text(&msg.content)];
                                    for call in calls {
                                        blocks.push(ContentBlock::ToolUse {
                                            id: call.id.clone(),
                                            name: call.name.clone(),
                                            input: call.arguments.clone(),
                                        });
                                    }
                                    chat_messages.push(ChatMessage {
                                        role: "assistant".to_string(),
                                        content: blocks,
                                    });
                                } else {
                                    chat_messages.push(ChatMessage::assistant_text(&msg.content));
                                }
                            }
                            "tool" => {
                                if let Some(ref calls) = msg.tool_calls {
                                    for call in calls {
                                        if let Some(ref result) = call.result {
                                            chat_messages.push(ChatMessage::tool_result(
                                                &call.id, result,
                                            ));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // ReAct loop: chat → tool_use → execute → tool_result → chat → ...
            let mut turn_count = 0;
            let max_turns = 5;

            while turn_count < max_turns {
                turn_count += 1;

                let request = ToolChatRequest {
                    messages: chat_messages.clone(),
                    tools: registry.definitions(),
                    system_prompt: None,
                };

                let (turn_tx, mut turn_rx) = tokio::sync::mpsc::unbounded_channel::<ToolChatEvent>();
                let provider_clone = ToolClaudeProvider::new(ai_for_turns.clone());

                let turn_task = tokio::spawn(async move {
                    provider_clone.chat_turn(request, turn_tx).await
                });

                let mut got_tool_use = false;
                let mut turn_text = String::new();
                let mut turn_tool_calls: Vec<ToolCallInfo> = Vec::new();

                while let Some(event) = turn_rx.recv().await {
                    match event {
                        ToolChatEvent::TextDelta { text } => {
                            turn_text.push_str(&text);
                            let event = Event::default().data(
                                serde_json::to_string(&ForgeStreamEvent::Delta {
                                    text: text.clone(),
                                })
                                .unwrap(),
                            );
                            let _ = event_tx.send(Ok(event));
                        }
                        ToolChatEvent::ToolUse { id, name, input } => {
                            got_tool_use = true;
                            let input_clone = input.clone();
                            let call = ToolCallInfo {
                                id: id.clone(),
                                name: name.clone(),
                                arguments: input_clone.clone(),
                                result: None,
                                status: "running".to_string(),
                            };
                            turn_tool_calls.push(call.clone());

                            // Notify frontend about the tool call
                            let event = Event::default().data(
                                serde_json::to_string(&ForgeStreamEvent::ToolCall {
                                    id: id.clone(),
                                    name: name.clone(),
                                    arguments: input_clone.clone(),
                                })
                                .unwrap(),
                            );
                            let _ = event_tx.send(Ok(event));

                            // Execute the tool
                            if let Some(tool) = registry.get(&name) {
                                let result = tool.execute(input);
                                let result_str = match result {
                                    Ok(r) => r,
                                    Err(e) => format!("Error: {}", e),
                                };

                                // Update call with result
                                if let Some(c) = turn_tool_calls.iter_mut().find(|c| c.id == id) {
                                    c.result = Some(result_str.clone());
                                    c.status = "success".to_string();
                                }

                                // Notify frontend about the result
                                let event = Event::default().data(
                                    serde_json::to_string(&ForgeStreamEvent::ToolResult {
                                        id: id.clone(),
                                        result: result_str.clone(),
                                    })
                                    .unwrap(),
                                );
                                let _ = event_tx.send(Ok(event));

                                // Add tool result to conversation for next turn
                                chat_messages.push(ChatMessage::tool_result(&id, &result_str));

                                // Persist tool result message
                                let tool_msg = ForgeMessage {
                                    id: format!("m-{}", uuid::Uuid::new_v4()),
                                    role: "tool".to_string(),
                                    content: result_str,
                                    timestamp: now_secs(),
                                    tool_calls: Some(vec![ToolCallInfo {
                                        id: id.clone(),
                                        name: name.clone(),
                                        arguments: input_clone.clone(),
                                        result: turn_tool_calls.iter().find(|c| c.id == id).and_then(|c| c.result.clone()),
                                        status: "success".to_string(),
                                    }]),
                                };
                                forge_sessions().lock().unwrap().push_message(&sid, tool_msg);
                            }
                        }
                        ToolChatEvent::Done => break,
                        ToolChatEvent::Error { message } => {
                            let event = Event::default().data(
                                serde_json::to_string(&ForgeStreamEvent::Error { message })
                                    .unwrap(),
                            );
                            let _ = event_tx.send(Ok(event));
                            break;
                        }
                    }
                }

                // Check for turn errors
                if let Ok(Some(err)) = turn_task.await {
                    let event = Event::default().data(
                        serde_json::to_string(&ForgeStreamEvent::Error { message: err }).unwrap(),
                    );
                    let _ = event_tx.send(Ok(event));
                    break;
                }

                // Persist assistant message for this turn
                if !turn_text.is_empty() || !turn_tool_calls.is_empty() {
                    let assistant_msg = ForgeMessage {
                        id: format!("m-{}", uuid::Uuid::new_v4()),
                        role: "assistant".to_string(),
                        content: turn_text.clone(),
                        timestamp: now_secs(),
                        tool_calls: if turn_tool_calls.is_empty() {
                            None
                        } else {
                            Some(turn_tool_calls.clone())
                        },
                    };
                    forge_sessions().lock().unwrap().push_message(&sid, assistant_msg.clone());

                    // Also add to chat_messages for next turn continuity
                    if got_tool_use {
                        let mut blocks = vec![ContentBlock::text(&turn_text)];
                        for call in &turn_tool_calls {
                            blocks.push(ContentBlock::ToolUse {
                                id: call.id.clone(),
                                name: call.name.clone(),
                                input: call.arguments.clone(),
                            });
                        }
                        chat_messages.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: blocks,
                        });
                    }
                }

                // If no tool_use was requested, we're done
                if !got_tool_use {
                    break;
                }
            }

            // Final done event
            let event = Event::default().data(
                serde_json::to_string(&ForgeStreamEvent::Done).unwrap(),
            );
            let _ = event_tx.send(Ok(event));

            // Update session status back to idle
            forge_sessions().lock().unwrap().update_status(&sid, ForgeStatus::Idle);
        });

        let sse_stream = stream::unfold(event_rx, |mut rx| async move {
            rx.recv().await.map(|event| (event, rx))
        });

        Sse::new(sse_stream).keep_alive(KeepAlive::default())
    }

    pub async fn forge_history(Path(sid): Path<String>) -> Json<Vec<ForgeMessage>> {
        let store = forge_sessions().lock().unwrap();
        let messages = store
            .get(&sid)
            .map(|s| s.messages.clone())
            .unwrap_or_default();
        Json(messages)
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ForgeSessionSummary {
        pub id: String,
        pub status: ForgeStatus,
        pub preview: String,
        pub message_count: usize,
        pub last_activity: u64,
    }

    pub async fn list_forge_sessions() -> Json<Vec<ForgeSessionSummary>> {
        let store = forge_sessions().lock().unwrap();
        let mut summaries: Vec<ForgeSessionSummary> = store
            .list_all()
            .iter()
            .map(|s| {
                let preview = s
                    .messages
                    .iter()
                    .find(|m| m.role == "user")
                    .map(|m| {
                        let content = m.content.trim();
                        if content.len() > 60 {
                            format!("{}…", &content[..60])
                        } else {
                            content.to_string()
                        }
                    })
                    .unwrap_or_else(|| String::from("New session"));

                let last_activity = s
                    .messages
                    .last()
                    .map(|m| m.timestamp)
                    .unwrap_or(0);

                ForgeSessionSummary {
                    id: s.id.clone(),
                    status: s.status.clone(),
                    preview,
                    message_count: s.messages.len(),
                    last_activity,
                }
            })
            .collect();

        // Sort by most recent activity first
        summaries.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
        Json(summaries)
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
}

// Non-generic route builder — caller must provide state that can produce AIProviderState
pub fn routes() -> Router<crate::AppState> {
    Router::new()
        // Forge
        .route("/api/smith/forge/session", post(handlers::create_forge_session))
        .route("/api/smith/forge/session/{sid}", get(handlers::get_forge_session))
        .route("/api/smith/forge/sessions", get(handlers::list_forge_sessions))
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
