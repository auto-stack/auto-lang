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

pub async fn list_notes() -> JsonResponse<Vec<Note>> {
    JsonResponse::<Vec<Note>>(crate::db::all_notes())
}

pub async fn get_note(Path(id): Path<i64>) -> Result<JsonResponse<Note>, StatusCode> {
    crate::db::find_note(id).map(JsonResponse::<Note>).ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_note(Json(input): Json<CreateNoteInput>) -> JsonResponse<Note> {
    JsonResponse::<Note>(crate::db::create_note(&input.title, &input.body, &input.folder))
}

pub async fn update_note(Path(id): Path<i64>, Json(input): Json<UpdateNoteInput>) -> Result<JsonResponse<Note>, StatusCode> {
    crate::db::update_note(id, &input.title, &input.body).map(JsonResponse::<Note>).ok_or(StatusCode::NOT_FOUND)
}

pub async fn delete_note(Path(id): Path<i64>) -> Result<JsonResponse<bool>, StatusCode> {
    Ok(JsonResponse::<bool>(crate::db::delete_note(id)))
}

pub async fn toggle_pin(Path(id): Path<i64>) -> Result<JsonResponse<Note>, StatusCode> {
    crate::db::toggle_pin(id).map(JsonResponse::<Note>).ok_or(StatusCode::NOT_FOUND)
}

pub async fn update_tags(Path(id): Path<i64>, Json(input): Json<UpdateNoteUpdateTagsInput>) -> Result<JsonResponse<Note>, StatusCode> {
    crate::db::update_tags(id, &input.tags).map(JsonResponse::<Note>).ok_or(StatusCode::NOT_FOUND)
}

pub async fn search_notes(Query(query): Query<SearchNotesQuery>) -> JsonResponse<Vec<Note>> {
    JsonResponse::<Vec<Note>>(crate::db::search_notes(&query.query))
}
