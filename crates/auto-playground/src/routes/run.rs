use axum::Json;
use serde::{Deserialize, Serialize};
use crate::error::AppError;
use crate::project::ProjectFile;
use crate::vm_runner;

#[derive(Deserialize)]
pub struct RunRequest {
    pub source: String,
    pub project_dir: Option<String>,
    pub files: Option<Vec<ProjectFile>>,
    /// Entry path within `files` for files-only projects (Plan 582); ignored
    /// when `project_dir` is set (those always run `main.at`).
    pub entry: Option<String>,
}

#[derive(Serialize)]
pub struct RunResponse {
    pub stdout: String,
    pub result: String,
    pub time_ms: u64,
    pub bytecode: Vec<serde_json::Value>,
    pub meta: Option<serde_json::Value>,
}

pub async fn run_handler(
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, AppError> {
    let result = tokio::task::spawn_blocking(move || {
        match req.project_dir {
            Some(dir) => vm_runner::run_project_source(&req.source, &dir, req.files),
            // Files-only projects (manifest notes): materialize and run from
            // the temp root (entry main.at); module resolution via source_dirs.
            None => match req.files {
                Some(files) if !files.is_empty() => {
                    vm_runner::run_files_project(&req.source, files)
                }
                _ => vm_runner::run_source(&req.source),
            },
        }
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(Json(RunResponse {
        stdout: result.stdout,
        result: result.result,
        time_ms: result.time_ms,
        bytecode: result.bytecode,
        meta: result.meta,
    }))
}
