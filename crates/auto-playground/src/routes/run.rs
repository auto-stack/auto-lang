use axum::Json;
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use crate::vm_runner;

#[derive(Deserialize)]
pub struct RunRequest {
    pub source: String,
}

#[derive(Serialize)]
pub struct RunResponse {
    pub stdout: String,
    pub result: String,
    pub time_ms: u64,
}

pub async fn run_handler(
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, AppError> {
    let result = tokio::task::spawn_blocking(move || vm_runner::run_source(&req.source))
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(RunResponse {
        stdout: result.stdout,
        result: result.result,
        time_ms: result.time_ms,
    }))
}
