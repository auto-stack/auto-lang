use axum::Json;
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use crate::code_runner;

#[derive(Deserialize)]
pub struct RunCodeRequest {
    pub language: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct RunCodeResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub time_ms: u64,
}

pub async fn run_code_handler(
    Json(req): Json<RunCodeRequest>,
) -> Result<Json<RunCodeResponse>, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        match req.language.as_str() {
            "python" => code_runner::run_python(&req.code),
            "rust" => code_runner::run_rust(&req.code),
            "c" => code_runner::run_c(&req.code),
            "typescript" => code_runner::run_typescript(&req.code),
            other => code_runner::CodeRunResult {
                stdout: String::new(),
                stderr: format!("Unsupported language: {}", other),
                exit_code: -1,
                time_ms: 0,
            },
        }
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(RunCodeResponse {
        stdout: result.stdout,
        stderr: result.stderr,
        exit_code: result.exit_code,
        time_ms: result.time_ms,
    }))
}
