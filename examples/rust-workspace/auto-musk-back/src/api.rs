use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<ToolCall>>>;

#[derive(serde::Deserialize)]
pub struct CreateToolCallInput {
    pub task: String,
    pub profession: String,
}

pub async fn run_agent(State(db): State<Db>, Json(input): Json<CreateToolCallInput>) -> JsonResponse<RunResult> {
    let mut items = db.lock().unwrap();
    let new_id = items.iter().map(|n| n.tool).max().unwrap_or(-1) + 1;
    let item = ToolCall {
        tool: new_id,
        task: input.task,
        profession: input.profession,
    };
    items.push(item.clone());
    JsonResponse(item)
}

pub async fn list_professions(State(db): State<Db>) -> JsonResponse<any> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

pub async fn get_config(State(db): State<Db>) -> JsonResponse<any> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

pub async fn list_skills(State(db): State<Db>) -> JsonResponse<any> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

pub async fn list_modes(State(db): State<Db>) -> JsonResponse<any> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

pub async fn list_workflows(State(db): State<Db>) -> JsonResponse<any> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

pub async fn run_workflow(State(db): State<Db>, Json(input): Json<CreateToolCallInput>) -> JsonResponse<WorkflowResult> {
    let mut items = db.lock().unwrap();
    let new_id = items.iter().map(|n| n.tool).max().unwrap_or(-1) + 1;
    let item = ToolCall {
        tool: new_id,
        task: input.task,
        workflow: input.workflow,
    };
    items.push(item.clone());
    JsonResponse(item)
}

pub async fn login(State(db): State<Db>, Json(input): Json<CreateToolCallInput>) -> JsonResponse<LoginResult> {
    let mut items = db.lock().unwrap();
    let new_id = items.iter().map(|n| n.tool).max().unwrap_or(-1) + 1;
    let item = ToolCall {
        tool: new_id,
        username: input.username,
        password: input.password,
    };
    items.push(item.clone());
    JsonResponse(item)
}

pub async fn me(State(db): State<Db>) -> JsonResponse<UserInfo> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

pub async fn get_specs(State(db): State<Db>) -> JsonResponse<any> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

pub async fn upsert_spec(State(db): State<Db>, Json(input): Json<CreateToolCallInput>) -> JsonResponse<any> {
    let mut items = db.lock().unwrap();
    let new_id = items.iter().map(|n| n.tool).max().unwrap_or(-1) + 1;
    let item = ToolCall {
        tool: new_id,
        section_id: input.section_id,
        item: input.item,
    };
    items.push(item.clone());
    JsonResponse(item)
}

pub async fn transition_spec(State(db): State<Db>, Json(input): Json<CreateToolCallInput>) -> JsonResponse<any> {
    let mut items = db.lock().unwrap();
    let new_id = items.iter().map(|n| n.tool).max().unwrap_or(-1) + 1;
    let item = ToolCall {
        tool: new_id,
        section_id: input.section_id,
        item_id: input.item_id,
        new_status: input.new_status,
    };
    items.push(item.clone());
    JsonResponse(item)
}
