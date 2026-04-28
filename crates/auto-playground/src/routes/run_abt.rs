use axum::Json;
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use crate::vm_runner;

#[derive(Deserialize)]
pub struct RunAbtRequest {
    pub abt: String,
}

#[derive(Serialize)]
pub struct RunAbtResponse {
    pub stdout: String,
    pub result: String,
    pub time_ms: u64,
}

pub async fn run_abt_handler(
    Json(req): Json<RunAbtRequest>,
) -> Result<Json<RunAbtResponse>, AppError> {
    let result = tokio::task::spawn_blocking(move || vm_runner::run_abt(&req.abt))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(RunAbtResponse {
        stdout: result.stdout,
        result: result.result,
        time_ms: result.time_ms,
    }))
}
