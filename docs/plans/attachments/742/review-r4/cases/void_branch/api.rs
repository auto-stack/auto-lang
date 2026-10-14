use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<Probe>>>;

pub async fn echo(State(db): State<Db>) -> StatusCode {
    if false {
        return StatusCode::OK;
    }
    let mut x: i64 = 1;
    return StatusCode::OK;
}
