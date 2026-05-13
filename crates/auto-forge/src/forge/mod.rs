//! AutoSmith — Spec-driven serial agent orchestration
//!
//! This module adds Forge (chat loop), Specs (knowledge management),
//! and Relay (agent pipeline) endpoints to the auto-playground server.
//! It reuses the existing NotebookActor for VM session sharing with AutoLab.

use axum::{
    extract::{Path, State},
    http::StatusCode,
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

use crate::ai::AIProviderState;

mod ai;
mod tools;

use axum::extract::FromRef;



// ─── Persistent Session Store ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeSession {
    pub id: String,
    pub notebook_sid: Option<String>,
    pub project_path: String,
    pub status: ForgeStatus,
    pub messages: Vec<ForgeMessage>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub pending_spec_changes: Vec<SpecChange>,
    #[serde(default)]
    pub focus_section: Option<String>,
}

/// Section type determines the lifecycle states and allowed transitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SectionType {
    Goals,
    Architecture,
    Designs,
    Plans,
    Tests,
    Reviews,
    Reports,
    Apis,
}

impl SectionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SectionType::Goals => "goals",
            SectionType::Architecture => "architecture",
            SectionType::Designs => "designs",
            SectionType::Plans => "plans",
            SectionType::Tests => "tests",
            SectionType::Reviews => "reviews",
            SectionType::Reports => "reports",
            SectionType::Apis => "apis",
        }
    }
}

/// Lifecycle status shared across all categories.
/// Not every category uses every variant — each SectionType configures its own subset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Empty,
    Proposed,
    Draft,
    UnderReview,
    Approved,
    InProgress,
    InImplementation,
    Implemented,
    Verified,
    Done,
    Archived,
    Rejected,
    Backlog,
    Ready,
    InReview,
    Blocked,
    Superseded,
    Outdated,
    Stable,
    Deprecated,
    Published,
    Analysed,
    Obsolete,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Empty => "empty",
            Status::Proposed => "proposed",
            Status::Draft => "draft",
            Status::UnderReview => "under_review",
            Status::Approved => "approved",
            Status::InProgress => "in_progress",
            Status::InImplementation => "in_implementation",
            Status::Implemented => "implemented",
            Status::Verified => "verified",
            Status::Done => "done",
            Status::Archived => "archived",
            Status::Rejected => "rejected",
            Status::Backlog => "backlog",
            Status::Ready => "ready",
            Status::InReview => "in_review",
            Status::Blocked => "blocked",
            Status::Superseded => "superseded",
            Status::Outdated => "outdated",
            Status::Stable => "stable",
            Status::Deprecated => "deprecated",
            Status::Published => "published",
            Status::Analysed => "analysed",
            Status::Obsolete => "obsolete",
        }
    }
}

/// A single item inside a SpecsSection.
/// Goals, Architecture, Designs, Plans, Tests, etc. are all represented as items with their own lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: Status,
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Auto-populated backlinks: IDs of items that reference this item.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milestone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    pub created_at: u64,
    pub modified_at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<u64>,
}

/// Per-category state-machine configuration.
pub struct SectionConfig {
    pub section_type: SectionType,
    pub allowed_statuses: Vec<Status>,
    pub allowed_transitions: Vec<(Status, Status)>,
}

impl SectionConfig {
    pub fn for_type(section_type: &SectionType) -> Self {
        match section_type {
            SectionType::Goals => Self {
                section_type: SectionType::Goals,
                allowed_statuses: vec![
                    Status::Empty, Status::Proposed, Status::Analysed, Status::Approved,
                    Status::InProgress, Status::Implemented, Status::Done, Status::Archived,
                ],
                allowed_transitions: vec![
                    (Status::Empty, Status::Proposed),
                    (Status::Proposed, Status::Analysed),
                    (Status::Analysed, Status::Approved),
                    (Status::Approved, Status::InProgress),
                    (Status::InProgress, Status::Implemented),
                    (Status::Implemented, Status::Done),
                    (Status::Done, Status::Archived),
                    (Status::InProgress, Status::Archived),
                ],
            },
            SectionType::Architecture | SectionType::Designs => Self {
                section_type: section_type.clone(),
                allowed_statuses: vec![
                    Status::Empty, Status::Draft, Status::UnderReview, Status::Approved,
                    Status::Superseded, Status::Outdated,
                ],
                allowed_transitions: vec![
                    (Status::Empty, Status::Draft),
                    (Status::Draft, Status::UnderReview),
                    (Status::UnderReview, Status::Approved),
                    (Status::UnderReview, Status::Rejected),
                    (Status::Approved, Status::Superseded),
                    (Status::Approved, Status::Outdated),
                ],
            },
            SectionType::Plans => Self {
                section_type: SectionType::Plans,
                allowed_statuses: vec![
                    Status::Empty, Status::Draft, Status::Approved, Status::InProgress,
                    Status::Done, Status::Obsolete,
                ],
                allowed_transitions: vec![
                    (Status::Empty, Status::Draft),
                    (Status::Draft, Status::Approved),
                    (Status::Approved, Status::InProgress),
                    (Status::InProgress, Status::Done),
                    (Status::Done, Status::Obsolete),
                ],
            },
            SectionType::Apis => Self {
                section_type: SectionType::Apis,
                allowed_statuses: vec![
                    Status::Empty, Status::Draft, Status::UnderReview, Status::Stable,
                    Status::Deprecated,
                ],
                allowed_transitions: vec![
                    (Status::Empty, Status::Draft),
                    (Status::Draft, Status::UnderReview),
                    (Status::UnderReview, Status::Stable),
                    (Status::Stable, Status::Deprecated),
                ],
            },
            SectionType::Tests => Self {
                section_type: SectionType::Tests,
                allowed_statuses: vec![
                    Status::Empty, Status::Draft, Status::Implemented,
                    Status::Done, Status::Verified, Status::Blocked,
                ],
                allowed_transitions: vec![
                    (Status::Empty, Status::Draft),
                    (Status::Draft, Status::Implemented),
                    (Status::Implemented, Status::Done),
                    (Status::Done, Status::Verified),
                    (Status::Implemented, Status::Blocked),
                    (Status::Blocked, Status::Implemented),
                ],
            },
            SectionType::Reviews | SectionType::Reports => Self {
                section_type: section_type.clone(),
                allowed_statuses: vec![
                    Status::Empty, Status::Draft, Status::Published,
                ],
                allowed_transitions: vec![
                    (Status::Empty, Status::Draft),
                    (Status::Draft, Status::Published),
                ],
            },
        }
    }

    pub fn can_transition(&self, from: &Status, to: &Status) -> bool {
        self.allowed_transitions.contains(&(from.clone(), to.clone()))
            || from == to
            || self.allowed_statuses.contains(to)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecChange {
    pub section_id: String,
    #[serde(default)]
    pub item_id: Option<String>,
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

    fn set_focus_section(&mut self, sid: &str, section_id: Option<String>) {
        let Some(session) = self.sessions.get_mut(sid) else { return };
        session.focus_section = section_id;
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

    fn rename(&mut self, sid: &str, name: String) -> bool {
        let Some(session) = self.sessions.get_mut(sid) else { return false };
        session.name = Some(name);
        let clone = session.clone();
        self.save(&clone);
        true
    }

    fn remove(&mut self, sid: &str) -> bool {
        let existed = self.sessions.remove(sid).is_some();
        if existed {
            let path = self.data_dir.join(format!("{}.json", sid));
            let _ = std::fs::remove_file(path);
            // Also remove any project lock held by this session
            self.project_locks.retain(|_, v| v != sid);
        }
        existed
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

// ─── Specs Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecsDocument {
    pub project: String,
    pub version: u64,
    pub sections: Vec<SpecsSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecsSection {
    pub id: String,
    #[serde(default = "default_section_type")]
    pub section_type: SectionType,
    pub title: String,
    #[serde(default)]
    pub items: Vec<SpecItem>,
    /// Representative status of the whole section (aggregated from items or set manually).
    #[serde(default = "default_status")]
    pub status: Status,
    /// Legacy content field kept for backward-compat during migration.
    /// If `items` is empty on load, content is auto-migrated into a single item.
    #[serde(default)]
    pub content: String,
    pub depends_on: Vec<String>,
    pub last_modified: u64,
    pub last_verified: Option<u64>,
}

fn default_section_type() -> SectionType {
    SectionType::Goals
}

fn default_status() -> Status {
    Status::Empty
}

// ─── Manifest Types (for .ad + manifest.at format) ──────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestAt {
    project: String,
    version: u32,
    #[serde(rename = "section", default)]
    sections: Vec<ManifestSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestSection {
    id: String,
    #[serde(rename = "section_type")]
    section_type: String,
    title: String,
    status: String,
    last_modified: u64,
    last_verified: Option<u64>,
}

// ─── Persistent Specs Store ─────────────────────────────────────────────────

struct SpecsStore {
    projects: std::collections::HashMap<String, SpecsDocument>,
    data_dir: PathBuf,
    templates_dir: PathBuf,
}

// ─── Embedded Default Templates ──────────────────────────────────────────────

const TMPL_GOALS: &str = include_str!("templates/goals.ad");
const TMPL_ARCHITECTURE: &str = include_str!("templates/architecture.ad");
const TMPL_DESIGNS: &str = include_str!("templates/designs.ad");
const TMPL_PLANS: &str = include_str!("templates/plans.ad");
const TMPL_TESTS: &str = include_str!("templates/tests.ad");
const TMPL_REVIEWS: &str = include_str!("templates/reviews.ad");
const TMPL_REPORTS: &str = include_str!("templates/reports.ad");
const TMPL_APIS: &str = include_str!("templates/apis.ad");

impl SpecsStore {
    fn new() -> Self {
        // Specs are stored in the project's own directory under docs/specs/
        // so they can be version-controlled alongside the code.
        // Override with AUTOFORGE_SPECS_DIR env var if needed.
        let data_dir = std::env::var("AUTOFORGE_SPECS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("docs")
                    .join("specs")
            });
        let _ = std::fs::create_dir_all(&data_dir);
        // Templates stay in a global cache dir so they don't pollute the repo
        let templates_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("autoforge")
            .join("templates");
        let _ = std::fs::create_dir_all(&templates_dir);

        let mut store = Self {
            projects: std::collections::HashMap::new(),
            data_dir,
            templates_dir,
        };
        store.extract_embedded_templates();
        store.load_all();
        store
    }

    fn extract_embedded_templates(&self) {
        let templates: [(&str, &str); 8] = [
            ("goals", TMPL_GOALS),
            ("architecture", TMPL_ARCHITECTURE),
            ("designs", TMPL_DESIGNS),
            ("plans", TMPL_PLANS),
            ("tests", TMPL_TESTS),
            ("reviews", TMPL_REVIEWS),
            ("reports", TMPL_REPORTS),
            ("apis", TMPL_APIS),
        ];
        for (name, content) in templates {
            let path = self.templates_dir.join(format!("{}.ad", name));
            if !path.exists() {
                let _ = std::fs::write(&path, content);
            }
        }
        tracing::info!("Templates directory: {:?}", self.templates_dir);
    }

    fn load_template(&self, name: &str) -> String {
        let path = self.templates_dir.join(format!("{}.ad", name));
        std::fs::read_to_string(&path).unwrap_or_else(|_| {
            tracing::warn!("Template file not found: {:?}, using embedded fallback", path);
            match name {
                "goals" => TMPL_GOALS.to_string(),
                "architecture" => TMPL_ARCHITECTURE.to_string(),
                "designs" => TMPL_DESIGNS.to_string(),
                "plans" => TMPL_PLANS.to_string(),
                "tests" => TMPL_TESTS.to_string(),
                "reviews" => TMPL_REVIEWS.to_string(),
                "reports" => TMPL_REPORTS.to_string(),
                "apis" => TMPL_APIS.to_string(),
                _ => String::new(),
            }
        })
    }

    fn load_all(&mut self) {
        let Ok(entries) = std::fs::read_dir(&self.data_dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // New format: directory with manifest.at + *.ad files
                let project_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if let Some(doc) = self.load_ad_format(&path, &project_name) {
                    self.projects.insert(project_name, doc);
                }
            } else if path.extension() == Some("json".as_ref()) {
                // Legacy format: single JSON file — auto-migrate
                let project_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                if let Some(doc) = self.load_json_and_migrate(&path, &project_name) {
                    self.projects.insert(project_name, doc);
                }
            }
        }
        tracing::info!("Loaded {} persistent specs documents", self.projects.len());
    }

    fn load_ad_format(&self, project_dir: &std::path::Path, project_name: &str) -> Option<SpecsDocument> {
        let manifest_path = project_dir.join("manifest.at");
        let manifest_content = std::fs::read_to_string(&manifest_path).ok()?;
        let manifest: ManifestAt = toml::from_str(&manifest_content).ok()?;

        let mut sections = Vec::new();
        for msec in &manifest.sections {
            let ad_path = project_dir.join(format!("{}.ad", msec.id));
            if let Ok(ad_content) = std::fs::read_to_string(&ad_path) {
                if let Some(section) = Self::parse_ad_file(&msec.id, &msec.section_type, &msec.title, &ad_content) {
                    sections.push(SpecsSection {
                        id: msec.id.clone(),
                        section_type: Self::parse_section_type(&msec.section_type),
                        title: msec.title.clone(),
                        items: section.items,
                        status: Self::parse_status(&msec.status),
                        content: section.content,
                        depends_on: section.depends_on,
                        last_modified: msec.last_modified,
                        last_verified: msec.last_verified,
                    });
                }
            }
        }
        Some(SpecsDocument {
            project: manifest.project,
            version: manifest.version as u64,
            sections,
        })
    }

    fn load_json_and_migrate(&self, path: &std::path::Path, project_name: &str) -> Option<SpecsDocument> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut doc: SpecsDocument = serde_json::from_str(&content).ok()?;
        // Migrate to .ad + manifest.at
        tracing::info!("Migrating legacy JSON specs for '{}' to .ad + manifest.at", project_name);
        self.save_ad_format(&doc, project_name);
        // Rename old JSON to .json.bak instead of deleting
        let bak_path = path.with_extension("json.bak");
        let _ = std::fs::rename(path, bak_path);
        Some(doc)
    }

    fn parse_section_type(s: &str) -> SectionType {
        match s {
            "goals" => SectionType::Goals,
            "architecture" => SectionType::Architecture,
            "designs" => SectionType::Designs,
            "plans" => SectionType::Plans,
            "tests" => SectionType::Tests,
            "reviews" => SectionType::Reviews,
            "reports" => SectionType::Reports,
            "apis" => SectionType::Apis,
            _ => SectionType::Goals,
        }
    }

    fn parse_status(s: &str) -> Status {
        match s {
            "empty" => Status::Empty,
            "proposed" => Status::Proposed,
            "draft" => Status::Draft,
            "under_review" => Status::UnderReview,
            "approved" => Status::Approved,
            "in_progress" => Status::InProgress,
            "in_implementation" => Status::InImplementation,
            "implemented" => Status::Implemented,
            "verified" => Status::Verified,
            "done" => Status::Done,
            "archived" => Status::Archived,
            "rejected" => Status::Rejected,
            "backlog" => Status::Backlog,
            "ready" => Status::Ready,
            "in_review" => Status::InReview,
            "blocked" => Status::Blocked,
            "superseded" => Status::Superseded,
            "outdated" => Status::Outdated,
            "stable" => Status::Stable,
            "deprecated" => Status::Deprecated,
            "published" => Status::Published,
            "analysed" => Status::Analysed,
            "obsolete" => Status::Obsolete,
            _ => Status::Draft,
        }
    }

    fn serialize_status(status: &Status) -> String {
        match status {
            Status::Empty => "empty",
            Status::Proposed => "proposed",
            Status::Draft => "draft",
            Status::UnderReview => "under_review",
            Status::Approved => "approved",
            Status::InProgress => "in_progress",
            Status::InImplementation => "in_implementation",
            Status::Implemented => "implemented",
            Status::Verified => "verified",
            Status::Done => "done",
            Status::Archived => "archived",
            Status::Rejected => "rejected",
            Status::Backlog => "backlog",
            Status::Ready => "ready",
            Status::InReview => "in_review",
            Status::Blocked => "blocked",
            Status::Superseded => "superseded",
            Status::Outdated => "outdated",
            Status::Stable => "stable",
            Status::Deprecated => "deprecated",
            Status::Published => "published",
            Status::Analysed => "analysed",
            Status::Obsolete => "obsolete",
        }.to_string()
    }

    fn parse_ad_file(section_id: &str, _section_type: &str, title: &str, content: &str) -> Option<SpecsSection> {
        use regex::Regex;
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() { return None; }

        // First line should be # Title
        let title_re = Regex::new(r"^#\s+(.+)$").unwrap();
        let first_line = lines[0];
        let parsed_title = title_re.captures(first_line).map(|c| c[1].to_string()).unwrap_or_else(|| title.to_string());

        let mut section_content_lines: Vec<&str> = Vec::new();
        let mut items: Vec<SpecItem> = Vec::new();
        let mut current_item: Option<SpecItem> = None;
        let mut item_content_lines: Vec<&str> = Vec::new();
        let mut in_section_content = true;
        let mut passed_separator = false;

        let item_heading_re = Regex::new(r"^##\s+([GADPSVXI]\d+(?:\.\d+)?)\s+(.+)$").unwrap();
        let meta_re = Regex::new(r"^\*\*(.+?):\*\*\s*(.*)$").unwrap();

        for line in lines.iter().skip(1) {
            let trimmed = line.trim();

            // Detect separator (--- or === or <!-- items -->)
            if in_section_content && (trimmed == "---" || trimmed == "===" || trimmed == "<!-- items -->") {
                passed_separator = true;
                continue;
            }

            // Detect item heading
            if let Some(caps) = item_heading_re.captures(line) {
                // Flush previous item
                if let Some(mut item) = current_item.take() {
                    item.content = item_content_lines.join("\n").trim().to_string();
                    items.push(item);
                    item_content_lines.clear();
                }
                in_section_content = false;
                let id = caps[1].to_string();
                let item_title = caps[2].to_string();
                current_item = Some(SpecItem {
                    id,
                    title: item_title,
                    content: String::new(),
                    status: Status::Draft,
                    depends_on: Vec::new(),
                    related: Vec::new(),
                    priority: None,
                    assignee: None,
                    test_file: None,
                    file: None,
                    milestone: None,
                    module: None,
                    created_at: now_secs(),
                    modified_at: now_secs(),
                    completed_at: None,
                });
                continue;
            }

            // If we're inside an item, try parsing metadata
            if let Some(ref mut item) = current_item {
                if let Some(meta_caps) = meta_re.captures(line) {
                    let key = meta_caps[1].trim().to_lowercase();
                    let value = meta_caps[2].trim();
                    match key.as_str() {
                        "status" => item.status = Self::parse_status(value),
                        "priority" => item.priority = Some(value.to_string()),
                        "assignee" => item.assignee = Some(value.to_string()),
                        "test file" => item.test_file = Some(value.to_string()),
                        "file" => item.file = Some(value.to_string()),
                        "milestone" => item.milestone = Some(value.to_string()),
                        "module" => item.module = Some(value.to_string()),
                        "depends on" => {
                            item.depends_on = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                        }
                        _ => {}
                    }
                    continue;
                }
            }

            if in_section_content {
                section_content_lines.push(line);
            } else if current_item.is_some() {
                item_content_lines.push(line);
            }
        }

        // Flush last item
        if let Some(mut item) = current_item.take() {
            item.content = item_content_lines.join("\n").trim().to_string();
            items.push(item);
        }

        // If no items were found and no separator, treat everything as section content
        let section_content = if items.is_empty() && !passed_separator {
            content.lines().skip(1).collect::<Vec<_>>().join("\n").trim().to_string()
        } else {
            section_content_lines.join("\n").trim().to_string()
        };

        Some(SpecsSection {
            id: section_id.to_string(),
            section_type: Self::parse_section_type(section_id),
            title: parsed_title,
            items,
            status: Status::Empty,
            content: section_content,
            depends_on: Vec::new(),
            last_modified: now_secs(),
            last_verified: None,
        })
    }

    fn serialize_section_to_ad(section: &SpecsSection) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("# {}", section.title));
        lines.push(String::new());
        if !section.content.trim().is_empty() {
            lines.push(section.content.trim().to_string());
            lines.push(String::new());
        }
        if !section.items.is_empty() {
            lines.push("---".to_string());
            lines.push(String::new());
            for item in &section.items {
                lines.push(format!("## {} {}", item.id, item.title));
                lines.push(format!("**Status:** {}", Self::serialize_status(&item.status)));
                if let Some(ref p) = item.priority { lines.push(format!("**Priority:** {}", p)); }
                if let Some(ref a) = item.assignee { lines.push(format!("**Assignee:** {}", a)); }
                if let Some(ref t) = item.test_file { lines.push(format!("**Test File:** {}", t)); }
                if let Some(ref f) = item.file { lines.push(format!("**File:** {}", f)); }
                if let Some(ref m) = item.milestone { lines.push(format!("**Milestone:** {}", m)); }
                if let Some(ref m) = item.module { lines.push(format!("**Module:** {}", m)); }
                if !item.depends_on.is_empty() { lines.push(format!("**Depends on:** {}", item.depends_on.join(", "))); }
                if !item.content.trim().is_empty() {
                    lines.push(String::new());
                    lines.push(item.content.trim().to_string());
                }
                lines.push(String::new());
            }
        }
        lines.join("\n")
    }

    fn save_ad_format(&self, doc: &SpecsDocument, project_name: &str) {
        let project_dir = self.data_dir.join(sanitize_filename(project_name));
        let _ = std::fs::create_dir_all(&project_dir);

        // Save manifest.at
        let manifest = ManifestAt {
            project: doc.project.clone(),
            version: doc.version as u32,
            sections: doc.sections.iter().map(|s| ManifestSection {
                id: s.id.clone(),
                section_type: match s.section_type {
                    SectionType::Goals => "goals",
                    SectionType::Architecture => "architecture",
                    SectionType::Designs => "designs",
                    SectionType::Plans => "plans",
                    SectionType::Tests => "tests",
                    SectionType::Reviews => "reviews",
                    SectionType::Reports => "reports",
                    SectionType::Apis => "apis",
                }.to_string(),
                title: s.title.clone(),
                status: Self::serialize_status(&s.status),
                last_modified: s.last_modified,
                last_verified: s.last_verified,
            }).collect(),
        };
        let manifest_path = project_dir.join("manifest.at");
        if let Ok(toml_str) = toml::to_string_pretty(&manifest) {
            let _ = std::fs::write(&manifest_path, toml_str);
        }

        // Save each section as .ad file
        for section in &doc.sections {
            let ad_path = project_dir.join(format!("{}.ad", section.id));
            let ad_content = Self::serialize_section_to_ad(section);
            let _ = std::fs::write(&ad_path, ad_content);
        }
    }

    fn get(&self, project: &str) -> Option<&SpecsDocument> {
        self.projects.get(project)
    }

    fn get_or_default(&mut self, project: &str) -> &mut SpecsDocument {
        if !self.projects.contains_key(project) {
            let doc = self.default_specs(project);
            self.save_ad_format(&doc, project);
            self.projects.insert(project.to_string(), doc);
        }
        // Ensure all default sections exist (backward compat: add missing sections)
        let default_doc = self.default_specs(project);
        let missing: Vec<SpecsSection> = {
            let doc = self.projects.get(project).unwrap();
            let existing_ids: std::collections::HashSet<String> =
                doc.sections.iter().map(|s| s.id.clone()).collect();
            default_doc
                .sections
                .into_iter()
                .filter(|s| !existing_ids.contains(&s.id))
                .collect()
        };
        let doc = self.projects.get_mut(project).unwrap();
        for section in missing {
            doc.sections.push(section);
        }
        doc
    }

    fn default_specs(&self, project: &str) -> SpecsDocument {
        let now = now_secs();
        SpecsDocument {
            project: project.to_string(),
            version: 1,
            sections: vec![
                SpecsSection { id: String::from("goals"), section_type: SectionType::Goals, title: String::from("🎯 Goals"), status: Status::Empty, items: vec![], content: self.load_template("goals"), depends_on: vec![], last_modified: now, last_verified: None },
                SpecsSection { id: String::from("architecture"), section_type: SectionType::Architecture, title: String::from("🏗️ Architecture"), status: Status::Empty, items: vec![], content: self.load_template("architecture"), depends_on: vec![], last_modified: now, last_verified: None },
                SpecsSection { id: String::from("designs"), section_type: SectionType::Designs, title: String::from("🎨 Designs"), status: Status::Empty, items: vec![], content: self.load_template("designs"), depends_on: vec![], last_modified: now, last_verified: None },
                SpecsSection { id: String::from("plans"), section_type: SectionType::Plans, title: String::from("📅 Plans"), status: Status::Empty, items: vec![], content: self.load_template("plans"), depends_on: vec![], last_modified: now, last_verified: None },
                SpecsSection { id: String::from("tests"), section_type: SectionType::Tests, title: String::from("🧪 Tests"), status: Status::Empty, items: vec![], content: self.load_template("tests"), depends_on: vec![], last_modified: now, last_verified: None },
                SpecsSection { id: String::from("reviews"), section_type: SectionType::Reviews, title: String::from("📝 Reviews"), status: Status::Empty, items: vec![], content: self.load_template("reviews"), depends_on: vec![], last_modified: now, last_verified: None },
                SpecsSection { id: String::from("reports"), section_type: SectionType::Reports, title: String::from("📊 Reports"), status: Status::Empty, items: vec![], content: self.load_template("reports"), depends_on: vec![], last_modified: now, last_verified: None },
                SpecsSection { id: String::from("apis"), section_type: SectionType::Apis, title: String::from("🔌 APIs"), status: Status::Empty, items: vec![], content: self.load_template("apis"), depends_on: vec![], last_modified: now, last_verified: None },
            ],
        }
    }

    fn update_section(&mut self, project: &str, section_id: &str, content: String, status: String) -> Result<(), String> {
        let doc = self.get_or_default(project);
        if let Some(section) = doc.sections.iter_mut().find(|s| s.id == section_id) {
            section.content = content;
            section.status = match status.as_str() {
                "empty" => Status::Empty,
                "proposed" => Status::Proposed,
                "draft" => Status::Draft,
                "under_review" => Status::UnderReview,
                "approved" => Status::Approved,
                "in_progress" => Status::InProgress,
                "in_implementation" => Status::InImplementation,
                "implemented" => Status::Implemented,
                "verified" => Status::Verified,
                "done" => Status::Done,
                "archived" => Status::Archived,
                "rejected" => Status::Rejected,
                "backlog" => Status::Backlog,
                "ready" => Status::Ready,
                "in_review" => Status::InReview,
                "blocked" => Status::Blocked,
                "superseded" => Status::Superseded,
                "outdated" => Status::Outdated,
                "stable" => Status::Stable,
                "deprecated" => Status::Deprecated,
                "published" => Status::Published,
                "analysed" => Status::Analysed,
                "obsolete" => Status::Obsolete,
                _ => Status::Draft,
            };
            section.last_modified = now_secs();
            doc.version += 1;
            Self::rebuild_relations(doc);
            let doc_clone = doc.clone();
            self.save(&doc_clone);
            Ok(())
        } else {
            Err(format!("Section '{}' not found", section_id))
        }
    }

    fn update_full(&mut self, incoming: SpecsDocument) -> Result<SpecsDocument, String> {
        let project = incoming.project.clone();
        let doc = self.get_or_default(&project);
        // Simple optimistic concurrency: just overwrite for now
        // (version check can be added later)
        *doc = incoming;
        doc.version += 1;
        Self::rebuild_relations(doc);
        let doc_clone = doc.clone();
        self.save(&doc_clone);
        Ok(doc_clone)
    }

    fn save(&self, doc: &SpecsDocument) {
        self.save_ad_format(doc, &doc.project);
    }

    /// Rebuild bidirectional `related` links across all items.
    /// Scans `depends_on` and content text for ID references.
    fn rebuild_relations(doc: &mut SpecsDocument) {
        use regex::Regex;
        let id_re = Regex::new(r"\b([GADPSVXI]\d+(?:\.\d+)?)\b").unwrap();

        // Collect all item IDs for validation
        let mut all_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for section in &doc.sections {
            for item in &section.items {
                all_ids.insert(item.id.clone());
            }
        }

        // Build forward links: ref_id -> [referrer_id]
        let mut links: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        for section in &doc.sections {
            for item in &section.items {
                // From depends_on
                for dep in &item.depends_on {
                    if all_ids.contains(dep) {
                        links.entry(dep.clone()).or_default().push(item.id.clone());
                    }
                }
                // From content text
                for cap in id_re.captures_iter(&item.content) {
                    let ref_id = cap[1].to_string();
                    if ref_id != item.id && all_ids.contains(&ref_id) {
                        links.entry(ref_id).or_default().push(item.id.clone());
                    }
                }
            }
        }

        // Write back
        for section in &mut doc.sections {
            for item in &mut section.items {
                item.related = links.get(&item.id).cloned().unwrap_or_default();
                item.related.sort();
                item.related.dedup();
            }
        }
    }
}



fn sanitize_filename(name: &str) -> String {
    name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}

fn specs() -> &'static Mutex<SpecsStore> {
    static STORE: OnceLock<Mutex<SpecsStore>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(SpecsStore::new()))
}

// ─── Handlers ────────────────────────────────────────────────────────────────

mod handlers {
    use super::*;
    use crate::ai::{AIProviderState, AiProvider};
    use crate::forge::ai::{ChatMessage, ContentBlock, ToolChatEvent, ToolChatRequest, ToolClaudeProvider};
    use crate::forge::tools::ToolRegistry;

    // ─── System Prompt & Tools ───────────────────────────────────────────

    fn build_system_prompt(_focus_section: &Option<String>) -> String {
        String::from(
            "You are AutoForge, an expert AI coding assistant. \
             You can read and write files, run shell commands, search code, \
             and manage project specifications (Jades). \
             Use the tools available to help the user build software."
        )
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
            name: None,
            pending_spec_changes: vec![],
            focus_section: None,
            messages: vec![ForgeMessage {
                id: format!("m-{}", uuid::Uuid::new_v4()),
                role: String::from("system"),
                content: String::from(
                    "You are AutoSmith Forge, a spec-driven AI coding assistant. \
                     Help the user build software by understanding goals, \
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
            let focus_section = {
                let store = forge_sessions().lock().unwrap();
                match store.get(&sid) {
                    Some(session) => {
                        crate::forge::tools::set_tool_context(&session.project_path, &sid);
                        session.focus_section.clone()
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

            // Build system prompt and tool set
            let system_prompt = build_system_prompt(&focus_section);
            let all_tools = registry.definitions();

            // ReAct loop: chat → tool_use → execute → tool_result → chat → ...
            let mut turn_count = 0;
            let max_turns = 5;

            while turn_count < max_turns {
                turn_count += 1;
                let mut turn_text = String::new();

                let request = ToolChatRequest {
                    messages: chat_messages.clone(),
                    tools: all_tools.clone(),
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

                // If no tool_use was requested, we're done
                if !got_tool_use {
                    break;
                }
            }

            // After turn completes, set session back to Idle
            {
                let mut store = forge_sessions().lock().unwrap();
                store.update_status(&sid, ForgeStatus::Idle);
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
        pub focus_section: Option<String>,
        pub name: Option<String>,
        pub preview: String,
        pub message_count: usize,
        pub last_activity: u64,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct RenameForgeSessionRequest {
        pub name: String,
    }

    pub async fn rename_forge_session(
        Path(sid): Path<String>,
        Json(req): Json<RenameForgeSessionRequest>,
    ) -> StatusCode {
        let mut store = forge_sessions().lock().unwrap();
        if store.rename(&sid, req.name) {
            StatusCode::NO_CONTENT
        } else {
            StatusCode::NOT_FOUND
        }
    }

    pub async fn delete_forge_session(Path(sid): Path<String>) -> StatusCode {
        let mut store = forge_sessions().lock().unwrap();
        if store.remove(&sid) {
            StatusCode::NO_CONTENT
        } else {
            StatusCode::NOT_FOUND
        }
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
                    focus_section: s.focus_section.clone(),
                    name: s.name.clone(),
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

    // ─── Specs Handlers ─────────────────────────────────────────────────

    pub async fn get_specs(Path(project): Path<String>) -> Json<SpecsDocument> {
        let mut store = specs().lock().unwrap();
        let doc = store.get_or_default(&project).clone();
        Json(doc)
    }

    pub async fn update_specs(
        Path(project): Path<String>,
        Json(doc): Json<SpecsDocument>,
    ) -> Result<Json<SpecsDocument>, String> {
        let mut store = specs().lock().unwrap();
        // Ensure the project matches the URL
        if doc.project != project {
            return Err("Project mismatch".to_string());
        }
        let updated = store.update_full(doc)?;
        Ok(Json(updated))
    }

    pub async fn get_specs_section(
        Path((project, section_id)): Path<(String, String)>,
    ) -> Json<Option<SpecsSection>> {
        let store = specs().lock().unwrap();
        let section = store
            .get(&project)
            .and_then(|d| d.sections.iter().find(|s| s.id == section_id).cloned());
        Json(section)
    }

    pub async fn update_specs_section(
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

        let mut store = specs().lock().unwrap();
        store.update_section(&project, &section_id, content, status)?;
        Ok(Json(serde_json::json!({"status": "ok"})))
    }

    pub async fn get_related_items(
        Path((project, item_id)): Path<(String, String)>,
    ) -> Json<serde_json::Value> {
        let store = specs().lock().unwrap();
        let doc = store.get(&project);

        let mut parents: Vec<serde_json::Value> = vec![];
        let mut children: Vec<serde_json::Value> = vec![];

        if let Some(doc) = doc {
            // Find the target item
            let mut target_item: Option<&SpecItem> = None;
            for section in &doc.sections {
                if let Some(item) = section.items.iter().find(|i| i.id == item_id) {
                    target_item = Some(item);
                    break;
                }
            }

            if let Some(target) = target_item {
                // Parents = items referenced by target's depends_on
                for dep_id in &target.depends_on {
                    for section in &doc.sections {
                        if let Some(item) = section.items.iter().find(|i| &i.id == dep_id) {
                            parents.push(serde_json::json!({
                                "id": item.id,
                                "title": item.title,
                                "section_type": section.section_type.as_str(),
                                "status": item.status.as_str(),
                            }));
                        }
                    }
                }
                // Children = items that have target in their related
                for section in &doc.sections {
                    for item in &section.items {
                        if item.related.contains(&item_id) {
                            children.push(serde_json::json!({
                                "id": item.id,
                                "title": item.title,
                                "section_type": section.section_type.as_str(),
                                "status": item.status.as_str(),
                            }));
                        }
                    }
                }
            }
        }

        Json(serde_json::json!({
            "id": item_id,
            "parents": parents,
            "children": children,
        }))
    }

    pub async fn rebuild_relations_endpoint(
        Path(project): Path<String>,
    ) -> Result<Json<SpecsDocument>, String> {
        let mut store = specs().lock().unwrap();
        let doc = store.get_or_default(&project);
        SpecsStore::rebuild_relations(doc);
        doc.version += 1;
        let doc_clone = doc.clone();
        store.save(&doc_clone);
        Ok(Json(doc_clone))
    }

    pub async fn trigger_drift_check(
        Path(project): Path<String>,
        State(ai): State<AIProviderState>,
    ) -> Json<serde_json::Value> {
        let specs_doc = {
            let store = specs().lock().unwrap();
            store.get(&project).cloned()
        };

        let Some(doc) = specs_doc else {
            return Json(serde_json::json!({
                "status": "ok",
                "drift_detected": false,
                "sections_checked": 0,
                "message": "No specs found",
            }));
        };

        // Find goals and plans sections
        let goals = doc.sections.iter().find(|s| s.id == "goals").map(|s| s.content.clone()).unwrap_or_default();
        let plans = doc.sections.iter().find(|s| s.id == "plans").map(|s| s.content.clone()).unwrap_or_default();

        // Extract file paths from plans (simple heuristic: lines mentioning file paths)
        let mut file_paths = Vec::new();
        for line in plans.lines() {
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

        if goals.is_empty() || code_content.is_empty() {
            return Json(serde_json::json!({
                "status": "ok",
                "drift_detected": false,
                "sections_checked": 0,
                "message": "No goals or code files to compare",
            }));
        }

        // Call AI to verify goals against code
        let prompt = format!(
            r#"You are a goals verifier. Compare the following goals against the implemented code.

Goals:
{}

Implemented code:
{}

For each goal, state whether it is:
- FULLY implemented
- PARTIALLY implemented
- NOT implemented
- UNKNOWN (cannot determine from code)

Format your response as:
G1: <status> — <brief explanation>
G2: <status> — <brief explanation>
...

If no goal IDs exist, number them sequentially."#,
            goals, code_content
        );

        let request = crate::ai::AIRequest {
            prompt,
            context: None,
        };

        let response = ai.chat(request).await;

        let drift_detected = response.content.to_lowercase().contains("not implemented")
            || response.content.to_lowercase().contains("partially implemented");

        // Update specs: mark goals section as drift if detected
        if drift_detected {
            let mut store = specs().lock().unwrap();
            let _ = store.update_section(&project, "goals", goals.clone(), "drift".to_string());
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
                name: None,
                messages: vec![],
                pending_spec_changes: vec![],
                focus_section: None,
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

        // 3. Apply pending (possibly edited) changes to Specs
        if !project.is_empty() && !changes.is_empty() {
            let mut specs = specs().lock().unwrap();
            for change in &changes {
                let _ = specs.update_section(
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
            store.update_status(&sid, ForgeStatus::Idle);
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
            store.update_status(&sid, ForgeStatus::Idle);
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
pub fn routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    crate::ai::AIProviderState: FromRef<S>,
{
    Router::new()
        // Forge
        .route("/api/forge/chats/session", post(handlers::create_forge_session))
        .route("/api/forge/chats/sessions", get(handlers::list_forge_sessions))
        .route("/api/forge/chats/session/{sid}", get(handlers::get_forge_session).patch(handlers::rename_forge_session).delete(handlers::delete_forge_session))
        .route("/api/forge/chats/{sid}/message", post(handlers::send_forge_message))
        .route("/api/forge/chats/{sid}/stream", get(handlers::forge_stream))
        .route("/api/forge/chats/{sid}/history", get(handlers::forge_history))
        .route("/api/forge/chats/{sid}/approve", post(handlers::approve_spec))
        .route("/api/forge/chats/{sid}/reject", post(handlers::reject_spec))
        // Specs (more specific routes FIRST)
        .route("/api/forge/specs/{project}/drift-check", post(handlers::trigger_drift_check))
        .route("/api/forge/specs/{project}/rebuild-relations", post(handlers::rebuild_relations_endpoint))
        .route("/api/forge/specs/{project}/related/{item_id}", get(handlers::get_related_items))
        .route("/api/forge/specs/{project}/{section_id}", get(handlers::get_specs_section).put(handlers::update_specs_section))
        .route("/api/forge/specs/{project}", get(handlers::get_specs).put(handlers::update_specs))
        // Relay
        .route("/api/forge/agents/run", post(handlers::start_run))
        .route("/api/forge/agents/runs", get(handlers::list_runs))
}
