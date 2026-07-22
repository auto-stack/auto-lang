use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<Note>>>;

#[derive(serde::Deserialize)]
pub struct CreateNoteInput {
    pub title: String,
    pub body: String,
    pub folder: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateNoteInput {
    pub title: String,
    pub body: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateNoteUpdateTagsInput {
    pub tags: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct SearchNotesQuery {
    pub query: String,
}

pub async fn list_notes(State(db): State<Db>) -> JsonResponse<Vec<Note>> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

pub async fn get_note(Path(id): Path<i64>, State(db): State<Db>) -> Result<JsonResponse<Note>, StatusCode> {
    let items = db.lock().unwrap();
    items.iter()
        .find(|n| n.id == id)
        .cloned()
        .map(JsonResponse)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_note(State(db): State<Db>, Json(input): Json<CreateNoteInput>) -> JsonResponse<Note> {
    let mut items = db.lock().unwrap();
    let new_id = items.iter().map(|n| n.id).max().unwrap_or(-1) + 1;
    let item = Note {
        id: new_id,
        title: input.title,
        body: input.body,
        folder: input.folder,
        time: "Just now".to_string(),
        ..Default::default()
    };
    items.push(item.clone());
    JsonResponse(item)
}

pub async fn update_note(Path(id): Path<i64>, State(db): State<Db>, Json(input): Json<UpdateNoteInput>) -> Result<JsonResponse<Note>, StatusCode> {
    let mut items = db.lock().unwrap();
    if let Some(item) = items.iter_mut().find(|n| n.id == id) {
        item.title = input.title.clone();
        item.body = input.body.clone();
        item.time = "Just now".to_string();
        Ok(JsonResponse(item.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn delete_note(Path(id): Path<i64>, State(db): State<Db>) -> Result<JsonResponse<bool>, StatusCode> {
    let mut items = db.lock().unwrap();
    let len_before = items.len();
    items.retain(|n| n.id != id);
    if items.len() < len_before {
        Ok(JsonResponse(true))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn toggle_pin(Path(id): Path<i64>, State(db): State<Db>) -> Result<JsonResponse<Note>, StatusCode> {
    let mut items = db.lock().unwrap();
    if let Some(item) = items.iter_mut().find(|n| n.id == id) {
        item.pinned = !item.pinned;
        Ok(JsonResponse(item.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn update_tags(Path(id): Path<i64>, State(db): State<Db>, Json(input): Json<UpdateNoteUpdateTagsInput>) -> Result<JsonResponse<Note>, StatusCode> {
    let mut items = db.lock().unwrap();
    if let Some(item) = items.iter_mut().find(|n| n.id == id) {
        item.tags = input.tags.clone();
        item.time = "Just now".to_string();
        Ok(JsonResponse(item.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn search_notes(State(db): State<Db>, Query(query): Query<SearchNotesQuery>) -> JsonResponse<Vec<Note>> {
    let items = db.lock().unwrap();
    let filtered: Vec<_> = items.iter().filter(|n| {
        n.title.to_lowercase().contains(&query.query.to_lowercase()) || n.body.to_lowercase().contains(&query.query.to_lowercase())
    }).cloned().collect();
    JsonResponse(filtered)
}
