use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<AuthUser>>>;

#[derive(serde::Deserialize)]
pub struct CreateAuthUserInput {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserSpecsSaveItemInput {
    pub section_id: String,
    pub item: SpecItem,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserPlansCreateInput {
    pub feature_name: String,
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserPlansTransitionInput {
    pub seq: i64,
    pub status: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserPlansArchiveInput {
    pub seq: i64,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserWikiCreatePageInput {
    pub project: String,
    pub slug: String,
    pub title: String,
    pub content: String,
    pub source_type: String,
    pub tags: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserWikiSearchInput {
    pub project: String,
    pub q: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserWikiRawUploadInput {
    pub project: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserWikiRawMkdirInput {
    pub project: String,
    pub path: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserChatsCreateSessionInput {
    pub project_path: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserChatsSendMessageInput {
    pub id: String,
    pub content: String,
    pub profession_id: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserChatsApproveInput {
    pub id: String,
    pub index: i64,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserChatsRejectAllInput {
    pub id: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserRelayStartRunInput {
    pub req: StartRunRequest,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserRelayAdvanceRunInput {
    pub run_id: String,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserRelayResolveGateInput {
    pub run_id: String,
    pub req: ResolveGateBody,
}

#[derive(serde::Deserialize)]
pub struct CreateAuthUserWorkspaceOpenInput {
    pub req: OpenWorkspaceBody,
}

#[derive(serde::Deserialize)]
pub struct UpdateAuthUserInput {
    pub seq: i64,
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateAuthUserWikiUpdatePageInput {
    pub project: String,
    pub slug: String,
    pub content: String,
    pub title: String,
    pub tags: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateAuthUserForgeModeSetInput {
    pub mode: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateAuthUserChatsRenameSessionInput {
    pub id: String,
    pub name: String,
}

#[derive(serde::Deserialize)]
pub struct ChatStreamQuery {
    pub id: String,
}

#[derive(serde::Deserialize)]
pub struct SpecsDeleteItemQuery {
    pub section_id: String,
    pub id: String,
}

#[derive(serde::Deserialize)]
pub struct SpecsRelatedQuery {
    pub id: String,
}

#[derive(serde::Deserialize)]
pub struct SpecsGetFileQuery {
    pub path: String,
}

#[derive(serde::Deserialize)]
pub struct PlansGetQuery {
    pub seq: i64,
}

#[derive(serde::Deserialize)]
pub struct WikiListPagesQuery {
    pub project: String,
}

#[derive(serde::Deserialize)]
pub struct WikiGetPageQuery {
    pub project: String,
    pub slug: String,
}

#[derive(serde::Deserialize)]
pub struct WikiDeletePageQuery {
    pub project: String,
    pub slug: String,
}

#[derive(serde::Deserialize)]
pub struct WikiTreeQuery {
    pub project: String,
}

#[derive(serde::Deserialize)]
pub struct WikiRawTreeQuery {
    pub project: String,
}

#[derive(serde::Deserialize)]
pub struct WikiRawFileQuery {
    pub project: String,
    pub path: String,
}

#[derive(serde::Deserialize)]
pub struct WikiRawDeleteFileQuery {
    pub project: String,
    pub path: String,
}

#[derive(serde::Deserialize)]
pub struct ChatsGetSessionQuery {
    pub id: String,
}

#[derive(serde::Deserialize)]
pub struct ChatsDeleteSessionQuery {
    pub id: String,
}

#[derive(serde::Deserialize)]
pub struct RelayGetRunQuery {
    pub run_id: String,
}

#[derive(serde::Deserialize)]
pub struct WorkspaceStatusQuery {
    pub workspace: String,
}

#[derive(serde::Deserialize)]
pub struct WorkspaceBrowseQuery {
    pub path: String,
}

pub async fn chat_stream() -> axum::response::Sse<impl futures::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>> {
    let rx = crate::events::subscribe();
    let stream = async_stream::stream! {
        let mut rx = rx;
        while let Ok(json) = rx.recv().await {
            yield Ok(axum::response::sse::Event::default().data(json));
        }
    };
    axum::response::Sse::new(stream)
}

pub async fn auth_login(State(db): State<Db>, Json(input): Json<CreateAuthUserInput>) -> JsonResponse<AuthResponse> {
    JsonResponse::<AuthResponse>(Default::default())
}

pub async fn auth_register(State(db): State<Db>, Json(input): Json<CreateAuthUserInput>) -> JsonResponse<AuthResponse> {
    JsonResponse::<AuthResponse>(Default::default())
}

pub async fn auth_me(State(db): State<Db>) -> JsonResponse<AuthUser> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

pub async fn auth_logout(State(db): State<Db>) -> StatusCode {
    StatusCode::OK
}

pub async fn specs_list(State(db): State<Db>) -> JsonResponse<SpecsDocument> {
    JsonResponse::<SpecsDocument>(Default::default())
}

pub async fn specs_overview(State(db): State<Db>) -> JsonResponse<SpecsOverview> {
    JsonResponse::<SpecsOverview>(Default::default())
}

pub async fn specs_save_item(State(db): State<Db>, Json(input): Json<CreateAuthUserSpecsSaveItemInput>) -> StatusCode {
    StatusCode::OK
}

pub async fn specs_delete_item(State(db): State<Db>, Query(query): Query<SpecsDeleteItemQuery>) -> Result<StatusCode, StatusCode> {
    StatusCode::OK
}

pub async fn specs_rebuild_relations(State(db): State<Db>) -> JsonResponse<SpecsDocument> {
    JsonResponse::<SpecsDocument>(Default::default())
}

pub async fn specs_related(State(db): State<Db>, Query(query): Query<SpecsRelatedQuery>) -> StatusCode {
    StatusCode::OK
}

pub async fn specs_drift_check(State(db): State<Db>) -> StatusCode {
    StatusCode::OK
}

pub async fn specs_tree(State(db): State<Db>) -> JsonResponse<Vec<TreeNode>> {
    JsonResponse::<Vec<TreeNode>>(Default::default())
}

pub async fn specs_get_file(State(db): State<Db>, Query(query): Query<SpecsGetFileQuery>) -> JsonResponse<String> {
    JsonResponse::<String>(Default::default())
}

pub async fn plans_list(State(db): State<Db>) -> JsonResponse<PlansListResponse> {
    JsonResponse::<PlansListResponse>(Default::default())
}

pub async fn plans_get(State(db): State<Db>, Query(query): Query<PlansGetQuery>) -> JsonResponse<PlanFile> {
    JsonResponse::<PlanFile>(Default::default())
}

pub async fn plans_create(State(db): State<Db>, Json(input): Json<CreateAuthUserPlansCreateInput>) -> JsonResponse<PlanFile> {
    JsonResponse::<PlanFile>(Default::default())
}

pub async fn plans_update(State(db): State<Db>, Json(input): Json<UpdateAuthUserInput>) -> Result<JsonResponse<PlanFile>, StatusCode> {
    Ok(JsonResponse::<PlanFile>(Default::default()))
}

pub async fn plans_transition(State(db): State<Db>, Json(input): Json<CreateAuthUserPlansTransitionInput>) -> JsonResponse<PlanFile> {
    JsonResponse::<PlanFile>(Default::default())
}

pub async fn plans_archive(State(db): State<Db>, Json(input): Json<CreateAuthUserPlansArchiveInput>) -> JsonResponse<PlanFile> {
    JsonResponse::<PlanFile>(Default::default())
}

pub async fn plans_merge(State(db): State<Db>, Json(input): Json<CreateAuthUserPlansArchiveInput>) -> JsonResponse<MergeResult> {
    JsonResponse::<MergeResult>(Default::default())
}

pub async fn wiki_list_pages(State(db): State<Db>, Query(query): Query<WikiListPagesQuery>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn wiki_get_page(State(db): State<Db>, Query(query): Query<WikiGetPageQuery>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn wiki_create_page(State(db): State<Db>, Json(input): Json<CreateAuthUserWikiCreatePageInput>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn wiki_update_page(State(db): State<Db>, Json(input): Json<UpdateAuthUserWikiUpdatePageInput>) -> Result<JsonResponse<any>, StatusCode> {
    Ok(JsonResponse::<any>(Default::default()))
}

pub async fn wiki_delete_page(State(db): State<Db>, Query(query): Query<WikiDeletePageQuery>) -> Result<StatusCode, StatusCode> {
    StatusCode::OK
}

pub async fn wiki_search(State(db): State<Db>, Json(input): Json<CreateAuthUserWikiSearchInput>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn wiki_tree(State(db): State<Db>, Query(query): Query<WikiTreeQuery>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn wiki_raw_tree(State(db): State<Db>, Query(query): Query<WikiRawTreeQuery>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn wiki_raw_upload(State(db): State<Db>, Json(input): Json<CreateAuthUserWikiRawUploadInput>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn wiki_raw_mkdir(State(db): State<Db>, Json(input): Json<CreateAuthUserWikiRawMkdirInput>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn wiki_raw_file(State(db): State<Db>, Query(query): Query<WikiRawFileQuery>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn wiki_raw_delete_file(State(db): State<Db>, Query(query): Query<WikiRawDeleteFileQuery>) -> Result<JsonResponse<any>, StatusCode> {
    Ok(JsonResponse::<any>(Default::default()))
}

pub async fn forge_mode_get(State(db): State<Db>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn forge_mode_set(State(db): State<Db>, Json(input): Json<UpdateAuthUserForgeModeSetInput>) -> Result<JsonResponse<any>, StatusCode> {
    Ok(JsonResponse::<any>(Default::default()))
}

pub async fn chats_list_sessions(State(db): State<Db>) -> JsonResponse<SessionListResponse> {
    JsonResponse::<SessionListResponse>(Default::default())
}

pub async fn chats_create_session(State(db): State<Db>, Json(input): Json<CreateAuthUserChatsCreateSessionInput>) -> JsonResponse<SessionDetailResponse> {
    JsonResponse::<SessionDetailResponse>(Default::default())
}

pub async fn chats_get_session(State(db): State<Db>, Query(query): Query<ChatsGetSessionQuery>) -> JsonResponse<SessionDetailResponse> {
    JsonResponse::<SessionDetailResponse>(Default::default())
}

pub async fn chats_delete_session(State(db): State<Db>, Query(query): Query<ChatsDeleteSessionQuery>) -> Result<StatusCode, StatusCode> {
    StatusCode::OK
}

pub async fn chats_delete_all_sessions(State(db): State<Db>) -> Result<StatusCode, StatusCode> {
    StatusCode::OK
}

pub async fn chats_rename_session(State(db): State<Db>, Json(input): Json<UpdateAuthUserChatsRenameSessionInput>) -> StatusCode {
    StatusCode::OK
}

pub async fn chats_send_message(State(db): State<Db>, Json(input): Json<CreateAuthUserChatsSendMessageInput>) -> StatusCode {
    StatusCode::OK
}

pub async fn chats_approve(State(db): State<Db>, Json(input): Json<CreateAuthUserChatsApproveInput>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn chats_reject_all(State(db): State<Db>, Json(input): Json<CreateAuthUserChatsRejectAllInput>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn relay_professions(State(db): State<Db>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn relay_start_run(State(db): State<Db>, Json(input): Json<CreateAuthUserRelayStartRunInput>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn relay_get_run(State(db): State<Db>, Query(query): Query<RelayGetRunQuery>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn relay_advance_run(State(db): State<Db>, Json(input): Json<CreateAuthUserRelayAdvanceRunInput>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn relay_resolve_gate(State(db): State<Db>, Json(input): Json<CreateAuthUserRelayResolveGateInput>) -> JsonResponse<any> {
    JsonResponse::<any>(Default::default())
}

pub async fn workspace_list(State(db): State<Db>) -> JsonResponse<WorkspaceListResponse> {
    JsonResponse::<WorkspaceListResponse>(Default::default())
}

pub async fn workspace_status(State(db): State<Db>, Query(query): Query<WorkspaceStatusQuery>) -> JsonResponse<WorkspaceStatusResponse> {
    JsonResponse::<WorkspaceStatusResponse>(Default::default())
}

pub async fn workspace_open(State(db): State<Db>, Json(input): Json<CreateAuthUserWorkspaceOpenInput>) -> JsonResponse<WorkspaceMeta> {
    JsonResponse::<WorkspaceMeta>(Default::default())
}

pub async fn workspace_browse(State(db): State<Db>, Query(query): Query<WorkspaceBrowseQuery>) -> JsonResponse<BrowseResponse> {
    JsonResponse::<BrowseResponse>(Default::default())
}
