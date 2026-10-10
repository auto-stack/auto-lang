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
    /// PLAN-746 (PG-MEM-1): execution time limit in seconds (1..=60,
    /// default 10). The VM cooperatively terminates the run once the
    /// deadline passes and the response carries a structured timeout error.
    pub timeout_secs: Option<u64>,
    /// PLAN-752: prepend the AAVM v1 bootstrap lib (auto/lib-legacy,
    /// AUTO_LIB_FILES manifest — same concat as the golden is_bootstrap
    /// branch) to the bare source before compiling. Hosts enable it for
    /// vm-bootstrap corpus notes; no effect on project/files runs.
    pub prepend_lib: Option<bool>,
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
    // PLAN-746 (PG-MEM-1): clamp 1..=60, default 10s.
    let timeout_secs = req.timeout_secs.unwrap_or(10).clamp(1, 60);
    let deadline = Some(std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs));
    let result = tokio::task::spawn_blocking(move || {
        match req.project_dir {
            Some(dir) => vm_runner::run_project_source(&req.source, &dir, req.files, deadline),
            // Files-only projects (manifest notes): materialize and run from
            // the temp root (entry main.at); module resolution via source_dirs.
            None => match req.files {
                Some(files) if !files.is_empty() => {
                    vm_runner::run_files_project(&req.source, files, deadline)
                }
                _ => vm_runner::run_source(&req.source, deadline, req.prepend_lib.unwrap_or(false)),
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
