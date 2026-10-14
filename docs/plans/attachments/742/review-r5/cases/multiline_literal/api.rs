use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<Probe>>>;

pub async fn echo(State(db): State<Db>) -> JsonResponse<Probe> {
    let mut s: String = "first\na2r_std::sqlite repository.take(value) List<int>.new([])\nlast".to_string();
    if true {
        return JsonResponse(Probe { text: s.to_string() });
    }
    return JsonResponse(Probe { text: "bad".to_string() });
}
