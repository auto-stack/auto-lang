use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuthUser {
    pub username: String,
    pub role: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuthUserWire {
    pub username: String,
    pub role: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: AuthUserWire,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpecItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub depends_on: Vec<String>,
    pub related: Vec<String>,
    pub priority: String,
    pub assignee: String,
    pub test_file: String,
    pub file: String,
    pub milestone: String,
    pub module: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub modified_at: i64,
    pub completed_at: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpecsSection {
    pub id: String,
    pub section_type: String,
    pub title: String,
    pub items: Vec<SpecItem>,
    pub status: String,
    pub content: String,
    pub depends_on: Vec<String>,
    pub last_modified: i64,
    pub last_verified: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpecsDocument {
    pub project: String,
    pub version: i64,
    pub sections: Vec<SpecsSection>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpecsOverviewSection {
    pub id: String,
    pub title: String,
    pub item_count: i64,
    pub status: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpecsOverview {
    pub project: String,
    pub version: i64,
    pub total_items: i64,
    pub sections: Vec<SpecsOverviewSection>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlanFile {
    pub id: String,
    pub seq: i64,
    pub filename: String,
    pub status: String,
    pub feature_name: String,
    pub title: String,
    pub archived: bool,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub path: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlansListResponse {
    pub plans: Vec<PlanFile>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MergeResult {
    pub plan_id: String,
    pub sections_touched: Vec<String>,
    pub items_created: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WikiPage {
    pub slug: String,
    pub title: String,
    pub content: String,
    pub source_type: String,
    pub tags: Vec<String>,
    pub version: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WikiPageMeta {
    pub slug: String,
    pub title: String,
    pub source_type: String,
    pub tags: Vec<String>,
    pub version: i64,
    pub updated_at: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub type: String,
    pub children: Vec<TreeNode>,
    pub size: i64,
    pub modified: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ForgeMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    pub tool_calls: Vec<String>,
    pub profession_id: String,
    pub thinking: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ForgeSession {
    pub id: String,
    pub project_path: String,
    pub status: String,
    pub phase: String,
    pub messages: Vec<ForgeMessage>,
    pub active_profession: String,
    pub pending_spec_changes: Vec<SpecChange>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpecChange {
    pub section_id: String,
    pub old_content: String,
    pub new_content: String,
    pub old_status: String,
    pub new_status: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ForgeSessionSummary {
    pub id: String,
    pub name: String,
    pub preview: String,
    pub message_count: i64,
    pub last_activity: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionListResponse {
    pub sessions: Vec<ForgeSessionSummary>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionDetailResponse {
    pub session: ForgeSession,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StartRunRequest {
    pub flow_id: String,
    pub task: String,
    pub steps: Vec<any>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResolveGateBody {
    pub decision: String,
    pub feedback: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkspaceMeta {
    pub id: String,
    pub path: String,
    pub name: String,
    pub is_empty: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkspaceListResponse {
    pub workspaces: Vec<WorkspaceMeta>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkspaceStatusResponse {
    pub workspace: WorkspaceMeta,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BrowseEntry {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BrowseResponse {
    pub entries: Vec<BrowseEntry>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OpenWorkspaceBody {
    pub path: String,
}
