use crate::repository;
use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<Probe>>>;

pub async fn echo(State(db): State<Db>) -> JsonResponse<Probe> {
    let mut n: i64 = crate::repository::ping();
    if n == 7 {
        return JsonResponse(Probe { text: "ok".to_string() });
    }
    return JsonResponse(Probe { text: "bad".to_string() });
}
