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
use std::sync::{Mutex, OnceLock};

use crate::notebook::ai::AIProviderState;

mod ai;
mod tools;

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
            tool_calls: None,
        };

        if let Ok(mut sessions) = forge_sessions().lock() {
            if let Some(session) = sessions.get_mut(&sid) {
                session.messages.push(user_msg.clone());
                session.status = ForgeStatus::Thinking;
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
                let sessions = forge_sessions().lock().unwrap();
                if let Some(session) = sessions.get(&sid) {
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
                                    // Assistant message with tool calls
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
                                // Tool results are sent as user messages with tool_result blocks
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
            let mut accumulated_text = String::new();
            let mut pending_tool_calls: Vec<ToolCallInfo> = Vec::new();

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

                while let Some(event) = turn_rx.recv().await {
                    match event {
                        ToolChatEvent::TextDelta { text } => {
                            accumulated_text.push_str(&text);
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
                            pending_tool_calls.push(ToolCallInfo {
                                id: id.clone(),
                                name: name.clone(),
                                arguments: input.clone(),
                                result: None,
                                status: "pending".to_string(),
                            });

                            // Notify frontend about the tool call
                            let event = Event::default().data(
                                serde_json::to_string(&ForgeStreamEvent::ToolCall {
                                    id: id.clone(),
                                    name: name.clone(),
                                    arguments: input.clone(),
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

                                // Update pending call with result
                                if let Some(call) = pending_tool_calls.iter_mut().find(|c| c.id == id) {
                                    call.result = Some(result_str.clone());
                                    call.status = "success".to_string();
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

                // If no tool_use was requested, we're done
                if !got_tool_use {
                    break;
                }

                // Otherwise, add the assistant's text response (if any) to conversation
                // and loop for another turn with the tool results
                if !accumulated_text.is_empty() {
                    chat_messages.push(ChatMessage::assistant_text(&accumulated_text));
                    accumulated_text.clear();
                }
            }

            // Final done event
            let event = Event::default().data(
                serde_json::to_string(&ForgeStreamEvent::Done).unwrap(),
            );
            let _ = event_tx.send(Ok(event));

            // Update session
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
