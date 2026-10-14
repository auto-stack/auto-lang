use crate::repository;
use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<NoteDocument>>>;

#[derive(serde::Deserialize)]
pub struct UpdateNoteDocumentInput {
    pub title: String,
}

pub async fn list_notes(State(db): State<Db>) -> JsonResponse<Vec<NoteDocument>> {
    let mut docs: Vec<NoteDocument> = crate::repository::repo_list_active();
    let mut out: Vec<NoteDocument> = vec![];
    for d in &docs {
        out.push(d.clone().clone());
    }
    return JsonResponse(out);
}

pub async fn get_note(Path(id): Path<i64>, State(db): State<Db>) -> Result<JsonResponse<NoteView>, StatusCode> {
    let mut docs: Vec<NoteDocument> = crate::repository::repo_list_active();
    if (docs.len() as i64) > 0 {
        let mut d: NoteDocument = docs[0].clone();
        let mut v: NoteView = NoteView { schema_version: d.schema_version, note_id: d.note_id, title: d.title };
        return Ok(JsonResponse(v));
    }
    return Err(StatusCode::NOT_FOUND);
}

pub async fn update_note(Path(id): Path<i64>, State(db): State<Db>, Json(input): Json<UpdateNoteDocumentInput>) -> Result<JsonResponse<Vec<NoteDocument>>, StatusCode> {
    let title = input.title.clone();
    let mut docs: Vec<NoteDocument> = crate::repository::repo_list_active();
    let mut ok: bool = (docs.len() as i64) > 0;
    if ok {
        ok = docs[0].clone().title == title;
    }
    return Ok(JsonResponse(docs));
}
