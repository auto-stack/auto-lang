use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<Probe>>>;

pub async fn echo(State(db): State<Db>) -> JsonResponse<Probe> {
    let mut n: i64 = repository.ping();
    if false {
        let mut repository: String = "local".to_string();
        let mut size: i64 = (repository.len() as i64);
    }
    if n == 7 {
        return JsonResponse(Probe { text: "ok".to_string() });
    }
    return JsonResponse(Probe { text: "bad".to_string() });
}
