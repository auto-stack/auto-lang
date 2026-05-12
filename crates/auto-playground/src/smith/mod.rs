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



// ─── Persistent Session Store ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseHistoryEntry {
    pub phase: String,
    pub entered_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeSession {
    pub id: String,
    pub notebook_sid: Option<String>,
    pub project_path: String,
    pub status: ForgeStatus,
    pub phase: ForgePhase,
    pub messages: Vec<ForgeMessage>,
    #[serde(default)]
    pub pending_spec_changes: Vec<SpecChange>,
    #[serde(default)]
    pub current_todo_index: Option<usize>,
    #[serde(default)]
    pub phase_history: Vec<PhaseHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ForgePhase {
    Intake,
    SpecDraft,
    SpecReview,
    Execution,
    Verification,
}

impl ForgePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            ForgePhase::Intake => "intake",
            ForgePhase::SpecDraft => "spec_draft",
            ForgePhase::SpecReview => "spec_review",
            ForgePhase::Execution => "execution",
            ForgePhase::Verification => "verification",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecChange {
    pub section_id: String,
    pub old_content: String,
    pub new_content: String,
    pub old_status: String,
    pub new_status: String,
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

    fn update_phase(&mut self, sid: &str, phase: ForgePhase) {
        let Some(session) = self.sessions.get_mut(sid) else { return };
        session.phase = phase;
        let session_clone = session.clone();
        self.save(&session_clone);
    }

    fn update_phase_and_status(&mut self, sid: &str, phase: ForgePhase, status: ForgeStatus) {
        let Some(session) = self.sessions.get_mut(sid) else { return };
        let phase_changed = session.phase != phase;
        let phase_str = phase.as_str().to_string();
        session.phase = phase;
        session.status = status;
        if phase_changed {
            session.phase_history.push(PhaseHistoryEntry {
                phase: phase_str,
                entered_at: now_secs(),
            });
        }
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
    #[serde(rename = "phase_change")]
    PhaseChange { phase: String },
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "error")]
    Error { message: String },
}

// ─── Ledger Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerDocument {
    pub project: String,
    pub version: u64,
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

// ─── Persistent Ledger Store ─────────────────────────────────────────────────

struct LedgerStore {
    projects: std::collections::HashMap<String, LedgerDocument>,
    data_dir: PathBuf,
}

impl LedgerStore {
    fn new() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("autoforge")
            .join("ledgers");
        let _ = std::fs::create_dir_all(&data_dir);

        let mut store = Self {
            projects: std::collections::HashMap::new(),
            data_dir,
        };
        store.load_all();
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
            let Ok(doc) = serde_json::from_str::<LedgerDocument>(&content) else { continue };
            self.projects.insert(doc.project.clone(), doc);
        }
        tracing::info!("Loaded {} persistent Ledger documents", self.projects.len());
    }

    fn get(&self, project: &str) -> Option<&LedgerDocument> {
        self.projects.get(project)
    }

    fn get_or_default(&mut self, project: &str) -> &mut LedgerDocument {
        if !self.projects.contains_key(project) {
            let doc = default_ledger(project);
            self.save(&doc);
            self.projects.insert(project.to_string(), doc);
        }
        self.projects.get_mut(project).unwrap()
    }

    fn update_section(&mut self, project: &str, section_id: &str, content: String, status: String) -> Result<(), String> {
        let doc = self.get_or_default(project);
        if let Some(section) = doc.sections.iter_mut().find(|s| s.id == section_id) {
            section.content = content;
            section.status = status;
            section.last_modified = now_secs();
            doc.version += 1;
            let doc_clone = doc.clone();
            self.save(&doc_clone);
            Ok(())
        } else {
            Err(format!("Section '{}' not found", section_id))
        }
    }

    fn update_full(&mut self, incoming: LedgerDocument) -> Result<LedgerDocument, String> {
        let project = incoming.project.clone();
        let doc = self.get_or_default(&project);
        // Simple optimistic concurrency: just overwrite for now
        // (version check can be added later)
        *doc = incoming;
        doc.version += 1;
        let doc_clone = doc.clone();
        self.save(&doc_clone);
        Ok(doc_clone)
    }

    fn save(&self, doc: &LedgerDocument) {
        let filename = sanitize_filename(&doc.project);
        let path = self.data_dir.join(format!("{}.json", filename));
        if let Ok(json) = serde_json::to_string_pretty(doc) {
            let _ = std::fs::write(path, json);
        }
    }
}

fn default_ledger(project: &str) -> LedgerDocument {
    let now = now_secs();
    LedgerDocument {
        project: project.to_string(),
        version: 1,
        sections: vec![
            LedgerSection { id: String::from("goals"), section_type: String::from("goals"), title: String::from("📋 Goals"), status: String::from("draft"), content: String::from("- Define project goals\n- Set success criteria"), depends_on: vec![], last_modified: now, last_verified: None },
            LedgerSection { id: String::from("requirements"), section_type: String::from("requirements"), title: String::from("📐 Requirements"), status: String::from("draft"), content: String::from("R1.1: Define functional requirements\nR1.2: Define non-functional requirements"), depends_on: vec![], last_modified: now, last_verified: None },
            LedgerSection { id: String::from("analysis"), section_type: String::from("analysis"), title: String::from("🔍 Analysis"), status: String::from("draft"), content: String::from("Technical approach and trade-offs."), depends_on: vec![], last_modified: now, last_verified: None },
            LedgerSection { id: String::from("plans"), section_type: String::from("plans"), title: String::from("📅 Plans"), status: String::from("draft"), content: String::from("Phase 1: Foundation\nPhase 2: Implementation\nPhase 3: Verification"), depends_on: vec![], last_modified: now, last_verified: None },
            LedgerSection { id: String::from("todos"), section_type: String::from("todos"), title: String::from("✅ Todos"), status: String::from("draft"), content: String::from("- [ ] Initial setup\n- [ ] Core implementation\n- [ ] Testing and review"), depends_on: vec![], last_modified: now, last_verified: None },
            LedgerSection { id: String::from("reports"), section_type: String::from("reports"), title: String::from("📊 Reports"), status: String::from("draft"), content: String::from("Coverage and quality reports."), depends_on: vec![], last_modified: now, last_verified: None },
            LedgerSection { id: String::from("reviews"), section_type: String::from("reviews"), title: String::from("📝 Reviews"), status: String::from("draft"), content: String::from("Code review notes and security audits."), depends_on: vec![], last_modified: now, last_verified: None },
        ],
    }
}

fn sanitize_filename(name: &str) -> String {
    name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}

fn ledgers() -> &'static Mutex<LedgerStore> {
    static STORE: OnceLock<Mutex<LedgerStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(LedgerStore::new()))
}

// ─── Handlers ────────────────────────────────────────────────────────────────

mod handlers {
    use super::*;
    use crate::notebook::ai::{AIProviderState, AiProvider};
    use crate::smith::ai::{ChatMessage, ContentBlock, ToolChatEvent, ToolChatRequest, ToolClaudeProvider};
    use crate::smith::tools::ToolRegistry;

    // ─── Phase Helpers ───────────────────────────────────────────────────

    fn get_phase_system_prompt(phase: &ForgePhase) -> String {
        let base = "You are AutoForge, an expert AI coding assistant.";
        match phase {
            ForgePhase::Intake => format!(
                r#"{base}

PHASE: INTAKE
Your job is to understand the user's request and classify the intent.

1. Ask clarifying questions if the request is ambiguous.
2. Classify the intent into one of:
   - NEW_GOAL: User wants to build something new → acknowledge and say you'll draft a spec
   - REQ_UPDATE: User wants to change existing requirements → acknowledge and say you'll update the spec
   - QUESTION: User is asking a question → answer directly, no spec needed
   - DIRECT: User wants immediate code changes → acknowledge and say you'll proceed to execution

Always explain your reasoning. If this is a new goal or requirement update, say so clearly."#
            ),
            ForgePhase::SpecDraft => format!(
                r#"{base}

PHASE: SPEC_DRAFT
Your job is to draft or update the project specification using the Jades (Ledger) tools.

Available tools: read_jade, write_jade, list_jades, read_file
You may NOT use write_file, edit_file, or shell in this phase.

1. Read existing Jades sections using list_jades and read_jade
2. Draft changes using write_jade to update relevant sections (goals, requirements, plans, todos)
3. Set status to "draft" for new/changed sections
4. Explain what you changed and why

Focus on correctness and completeness. Do not implement code yet."#
            ),
            ForgePhase::SpecReview => format!(
                r#"{base}

PHASE: SPEC_REVIEW
Review the proposed specification changes. This phase is read-only.

Available tools: read_jade, list_jades, read_file
You may NOT modify any files or Jades in this phase.

1. Read the current spec using list_jades and read_jade
2. Check for completeness, consistency, and feasibility
3. Report any issues or concerns
4. Confirm if the spec is ready for execution

After your review, the human will approve or reject the spec."#
            ),
            ForgePhase::Execution => format!(
                r#"{base}

PHASE: EXECUTION
Your job is to implement the approved specification.

Available tools: read_file, write_file, edit_file, shell, search
You may NOT use write_jade in this phase. The spec is locked.

1. Read the spec from Jades to understand requirements
2. Examine existing code using read_file and search
3. Implement changes using write_file and edit_file
4. Run tests or checks using shell when appropriate
5. Follow the spec precisely. Do not deviate without good reason.

Report progress as you work."#
            ),
            ForgePhase::Verification => format!(
                r#"{base}

PHASE: VERIFICATION
Your job is to verify that the implementation matches the specification.

Available tools: read_file, read_jade, list_jades, search
You may NOT modify any files in this phase.

1. Re-read the spec requirements
2. Examine the implemented code
3. Check for:
   - All requirements are met
   - No unintended changes
   - Code quality and correctness
4. Report findings. Flag any drift from the spec.

After verification, summarize the results."#
            ),
        }
    }

    fn get_phase_tools(
        phase: &ForgePhase,
        all_tools: Vec<crate::smith::tools::ToolDefinition>,
    ) -> Vec<crate::smith::tools::ToolDefinition> {
        let allowed: &[&str] = match phase {
            ForgePhase::Intake => &["read_file", "read_jade", "list_jades"],
            ForgePhase::SpecDraft => &["read_file", "read_jade", "write_jade", "list_jades"],
            ForgePhase::SpecReview => &["read_file", "read_jade", "list_jades"],
            ForgePhase::Execution => &["read_file", "write_file", "edit_file", "shell", "search"],
            ForgePhase::Verification => &["read_file", "read_jade", "list_jades", "search"],
        };
        all_tools
            .into_iter()
            .filter(|t| allowed.contains(&t.name.as_str()))
            .collect()
    }

    fn next_phase_after_turn(phase: &ForgePhase) -> (ForgePhase, ForgeStatus) {
        match phase {
            ForgePhase::Intake => (ForgePhase::SpecDraft, ForgeStatus::Idle),
            ForgePhase::SpecDraft => (ForgePhase::SpecReview, ForgeStatus::WaitingApproval),
            ForgePhase::SpecReview => (ForgePhase::SpecReview, ForgeStatus::WaitingApproval),
            ForgePhase::Execution => (ForgePhase::Verification, ForgeStatus::Idle),
            ForgePhase::Verification => (ForgePhase::Intake, ForgeStatus::Idle),
        }
    }

    pub async fn create_forge_session(
        Json(req): Json<CreateForgeSessionRequest>,
    ) -> Json<ForgeSession> {
        let sid = format!("forge-{}", uuid::Uuid::new_v4());
        let session = ForgeSession {
            id: sid.clone(),
            notebook_sid: req.notebook_sid,
            project_path: req.project_path.unwrap_or_else(|| String::from(".")),
            status: ForgeStatus::Idle,
            phase: ForgePhase::Intake,
            pending_spec_changes: vec![],
            current_todo_index: None,
            phase_history: vec![PhaseHistoryEntry {
                phase: ForgePhase::Intake.as_str().to_string(),
                entered_at: now_secs(),
            }],
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
            let _provider = ToolClaudeProvider::new(ai);

            // Inject project/session context for Jades tools
            let (_project_path, current_phase) = {
                let store = forge_sessions().lock().unwrap();
                match store.get(&sid) {
                    Some(session) => {
                        crate::smith::tools::set_tool_context(&session.project_path, &sid);
                        (session.project_path.clone(), session.phase.clone())
                    }
                    None => {
                        let _ = event_tx.send(Ok(Event::default().data(
                            serde_json::to_string(&ForgeStreamEvent::Error {
                                message: "Session not found".to_string(),
                            })
                            .unwrap(),
                        )));
                        return;
                    }
                }
            };

            // Phase already loaded above with project_path

            // Build conversation messages from session history
            let mut chat_messages = Vec::new();
            {
                let store = forge_sessions().lock().unwrap();
                if let Some(session) = store.get(&sid) {
                    for msg in &session.messages {
                        match msg.role.as_str() {
                            "system" => {
                                // System prompt is handled separately via phase prompt
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

            // Build phase-aware system prompt and tool set
            let system_prompt = get_phase_system_prompt(&current_phase);
            let phase_tools = get_phase_tools(&current_phase, registry.definitions());

            // ReAct loop: chat → tool_use → execute → tool_result → chat → ...
            let mut turn_count = 0;
            let max_turns = 5;
            let mut last_turn_text = String::new();

            while turn_count < max_turns {
                turn_count += 1;
                let mut turn_text = String::new();

                let request = ToolChatRequest {
                    messages: chat_messages.clone(),
                    tools: phase_tools.clone(),
                    system_prompt: Some(system_prompt.clone()),
                };

                let (turn_tx, mut turn_rx) = tokio::sync::mpsc::unbounded_channel::<ToolChatEvent>();
                let provider_clone = ToolClaudeProvider::new(ai_for_turns.clone());

                let turn_task = tokio::spawn(async move {
                    provider_clone.chat_turn(request, turn_tx).await
                });

                let mut got_tool_use = false;
                turn_text.clear();
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

                // Remember the last assistant text for intent classification
                last_turn_text = turn_text.clone();

                // If no tool_use was requested, we're done
                if !got_tool_use {
                    break;
                }
            }

            // Determine next phase based on current phase and intent classification
            let (next_phase, next_status) = if current_phase == ForgePhase::Intake {
                let lower = last_turn_text.to_lowercase();
                if lower.contains("classification: question") || lower.contains("**classification:** question") {
                    (ForgePhase::Intake, ForgeStatus::Idle)
                } else if lower.contains("classification: direct") || lower.contains("**classification:** direct") {
                    (ForgePhase::Execution, ForgeStatus::Idle)
                } else {
                    (ForgePhase::SpecDraft, ForgeStatus::Idle)
                }
            } else {
                next_phase_after_turn(&current_phase)
            };

            // Emit phase change event if transitioning
            if next_phase != current_phase {
                let phase_event = Event::default().data(
                    serde_json::to_string(&ForgeStreamEvent::PhaseChange {
                        phase: next_phase.as_str().to_string(),
                    })
                    .unwrap(),
                );
                let _ = event_tx.send(Ok(phase_event));
            }

            // Update session phase and status
            {
                let mut store = forge_sessions().lock().unwrap();
                store.update_phase_and_status(&sid, next_phase, next_status);
            }

            // Final done event
            let event = Event::default().data(
                serde_json::to_string(&ForgeStreamEvent::Done).unwrap(),
            );
            let _ = event_tx.send(Ok(event));
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
        pub phase: ForgePhase,
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
                    phase: s.phase.clone(),
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

    pub async fn get_ledger(Path(project): Path<String>) -> Json<LedgerDocument> {
        let mut store = ledgers().lock().unwrap();
        let doc = store.get_or_default(&project).clone();
        Json(doc)
    }

    pub async fn update_ledger(
        Path(project): Path<String>,
        Json(doc): Json<LedgerDocument>,
    ) -> Result<Json<LedgerDocument>, String> {
        let mut store = ledgers().lock().unwrap();
        // Ensure the project matches the URL
        if doc.project != project {
            return Err("Project mismatch".to_string());
        }
        let updated = store.update_full(doc)?;
        Ok(Json(updated))
    }

    pub async fn get_ledger_section(
        Path((project, section_id)): Path<(String, String)>,
    ) -> Json<Option<LedgerSection>> {
        let store = ledgers().lock().unwrap();
        let section = store
            .get(&project)
            .and_then(|d| d.sections.iter().find(|s| s.id == section_id).cloned());
        Json(section)
    }

    pub async fn update_ledger_section(
        Path((project, section_id)): Path<(String, String)>,
        Json(body): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, String> {
        let content = body
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let status = body
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("draft")
            .to_string();

        let mut store = ledgers().lock().unwrap();
        store.update_section(&project, &section_id, content, status)?;
        Ok(Json(serde_json::json!({"status": "ok"})))
    }

    pub async fn trigger_drift_check(
        Path(project): Path<String>,
        State(ai): State<AIProviderState>,
    ) -> Json<serde_json::Value> {
        let ledger = {
            let store = ledgers().lock().unwrap();
            store.get(&project).cloned()
        };

        let Some(doc) = ledger else {
            return Json(serde_json::json!({
                "status": "ok",
                "drift_detected": false,
                "sections_checked": 0,
                "message": "No ledger found",
            }));
        };

        // Find requirements and todos sections
        let requirements = doc.sections.iter().find(|s| s.id == "requirements").map(|s| s.content.clone()).unwrap_or_default();
        let todos = doc.sections.iter().find(|s| s.id == "todos").map(|s| s.content.clone()).unwrap_or_default();

        // Extract file paths from todos (simple heuristic: lines mentioning file paths)
        let mut file_paths = Vec::new();
        for line in todos.lines() {
            // Look for patterns like `src/...`, `crates/...`, `.rs`, `.ts`, `.vue`
            for word in line.split_whitespace() {
                if word.contains('/') && (word.ends_with(".rs") || word.ends_with(".ts") || word.ends_with(".vue") || word.ends_with(".js")) {
                    let clean = word.trim_matches(|c| c == '(' || c == ')' || c == '`' || c == '"' || c == ',' || c == '.');
                    if !clean.is_empty() && !file_paths.contains(&clean.to_string()) {
                        file_paths.push(clean.to_string());
                    }
                }
            }
        }

        // Read up to 5 files
        let mut code_content = String::new();
        for path in file_paths.iter().take(5) {
            if let Ok(content) = std::fs::read_to_string(path) {
                code_content.push_str(&format!("\n--- {} ---\n{}", path, content));
            }
        }

        if requirements.is_empty() || code_content.is_empty() {
            return Json(serde_json::json!({
                "status": "ok",
                "drift_detected": false,
                "sections_checked": 0,
                "message": "No requirements or code files to compare",
            }));
        }

        // Call AI to verify requirements against code
        let prompt = format!(
            r#"You are a requirements verifier. Compare the following requirements against the implemented code.

Requirements:
{}

Implemented code:
{}

For each requirement, state whether it is:
- FULLY implemented
- PARTIALLY implemented
- NOT implemented
- UNKNOWN (cannot determine from code)

Format your response as:
R1: <status> — <brief explanation>
R2: <status> — <brief explanation>
...

If no requirement IDs exist, number them sequentially."#,
            requirements, code_content
        );

        let request = crate::notebook::ai::AIRequest {
            prompt,
            context: None,
        };

        let response = ai.chat(request).await;

        let drift_detected = response.content.to_lowercase().contains("not implemented")
            || response.content.to_lowercase().contains("partially implemented");

        // Update ledger: mark requirements section as drift if detected
        if drift_detected {
            let mut store = ledgers().lock().unwrap();
            let _ = store.update_section(&project, "requirements", requirements.clone(), "drift".to_string());
        }

        Json(serde_json::json!({
            "status": "ok",
            "drift_detected": drift_detected,
            "sections_checked": 1,
            "report": response.content,
            "error": response.error,
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

    // ─── Approval Gate Handlers ──────────────────────────────────────────

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApproveSpecRequest {
        #[serde(default)]
        pub edited_specs: std::collections::HashMap<String, String>,
    }

    pub async fn approve_spec(
        Path(sid): Path<String>,
        Json(body): Json<ApproveSpecRequest>,
    ) -> Json<serde_json::Value> {
        // 1. Capture pending changes and project path
        let (project, mut changes) = {
            let store = forge_sessions().lock().unwrap();
            let session = store.get(&sid).cloned().unwrap_or_else(|| ForgeSession {
                id: sid.clone(),
                notebook_sid: None,
                project_path: String::new(),
                status: ForgeStatus::Idle,
                phase: ForgePhase::Intake,
                messages: vec![],
                pending_spec_changes: vec![],
                current_todo_index: None,
                phase_history: vec![],
            });
            (session.project_path.clone(), session.pending_spec_changes.clone())
        };

        // 2. Override with user-edited specs if provided
        if !body.edited_specs.is_empty() {
            for change in &mut changes {
                if let Some(edited) = body.edited_specs.get(&change.section_id) {
                    change.new_content = edited.clone();
                }
            }
        }

        // 3. Apply pending (possibly edited) changes to Ledger
        if !project.is_empty() && !changes.is_empty() {
            let mut ledger = ledgers().lock().unwrap();
            for change in &changes {
                let _ = ledger.update_section(
                    &project,
                    &change.section_id,
                    change.new_content.clone(),
                    change.new_status.clone(),
                );
            }
        }

        // 3. Clear pending changes and transition phase
        {
            let mut store = forge_sessions().lock().unwrap();
            if let Some(session) = store.get_mut(&sid) {
                session.pending_spec_changes.clear();
                let clone = session.clone();
                store.save(&clone);
            }
            store.update_phase_and_status(&sid, ForgePhase::Execution, ForgeStatus::Idle);
        }

        Json(serde_json::json!({"status": "ok", "phase": "execution"}))
    }

    pub async fn reject_spec(Path(sid): Path<String>) -> Json<serde_json::Value> {
        {
            let mut store = forge_sessions().lock().unwrap();
            if let Some(session) = store.get_mut(&sid) {
                session.pending_spec_changes.clear();
                let clone = session.clone();
                store.save(&clone);
            }
            store.update_phase_and_status(&sid, ForgePhase::SpecDraft, ForgeStatus::Idle);
        }
        Json(serde_json::json!({"status": "ok", "phase": "spec_draft"}))
    }

    // ─── Helpers ─────────────────────────────────────────────────────────

    fn now_secs() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
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
        .route("/api/smith/forge/{sid}/approve", post(handlers::approve_spec))
        .route("/api/smith/forge/{sid}/reject", post(handlers::reject_spec))
        // Ledger (more specific routes FIRST)
        .route("/api/smith/ledger/{project}/drift-check", post(handlers::trigger_drift_check))
        .route("/api/smith/ledger/{project}/{section_id}", get(handlers::get_ledger_section).put(handlers::update_ledger_section))
        .route("/api/smith/ledger/{project}", get(handlers::get_ledger).put(handlers::update_ledger))
        // Relay
        .route("/api/smith/relay/run", post(handlers::start_run))
        .route("/api/smith/relay/runs", get(handlers::list_runs))
}
