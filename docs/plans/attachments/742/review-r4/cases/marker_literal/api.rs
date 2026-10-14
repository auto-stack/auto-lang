use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<Probe>>>;

pub async fn echo(State(db): State<Db>) -> JsonResponse<Probe> {
    let mut msg: String = "__AUTO_A2R_STD_QUAL__ __AUTO_A2R_STD_SQLITE__ __AUTO_A2R_STD_SYS__".to_string();
    if false {
        return JsonResponse(Probe { text: msg.to_string() });
    }
    return JsonResponse(Probe { text: msg.to_string() });
}
