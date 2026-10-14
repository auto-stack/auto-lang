use axum::{
    extract::{Path, State, Json, Query},
    http::StatusCode,
    Json as JsonResponse,
};
use crate::types::*;
use std::sync::{Arc, Mutex};

pub type Db = Arc<Mutex<Vec<Probe>>>;

pub async fn echo(State(db): State<Db>) -> JsonResponse<Probe> {
    let mut msg: String = "auto_lang::a2r_std:: ::a2r_std::sqlite a2r_std::sys::".to_string();
    if false {
        return JsonResponse(Probe { text: msg.to_string() });
    }
    return JsonResponse(Probe { text: msg.to_string() });
}
