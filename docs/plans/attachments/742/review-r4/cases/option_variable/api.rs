use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<Probe>>>;

pub async fn echo(Path(id): Path<i64>, State(db): State<Db>) -> Result<JsonResponse<Probe>, StatusCode> {
    let mut p: Option<Probe> = Some(Probe { text: "ok".to_string() });
    if id > 0 {
        return match p { Some(__opt_v) => Ok(JsonResponse(__opt_v)), None => Err(StatusCode::NOT_FOUND) };
    }
    return Err(StatusCode::NOT_FOUND);
}
