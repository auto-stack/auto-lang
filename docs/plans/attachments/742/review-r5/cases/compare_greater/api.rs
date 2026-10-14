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
    let mut s: String = "ok".to_string();
    let mut n: i64 = crate::repository::choose(2 > 1, &s);
    if n == 2 {
        return JsonResponse(Probe { text: "ok".to_string() });
    }
    return JsonResponse(Probe { text: "bad".to_string() });
}
