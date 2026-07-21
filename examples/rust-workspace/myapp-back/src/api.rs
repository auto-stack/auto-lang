use axum::{
    extract::{Path, State, Json},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<Status>>>;

pub async fn status(State(db): State<Db>) -> JsonResponse<Vec<Status>> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}
