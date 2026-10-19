use crate::error::AppError;
use crate::project::ProjectFile;
use crate::vm_runner;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RunRequest {
    pub source: String,
    pub project_dir: Option<String>,
    pub files: Option<Vec<ProjectFile>>,
    /// Entry path within `files` for files-only projects (Plan 582); ignored
    /// when `project_dir` is set (those always run `main.at`).
    pub entry: Option<String>,
    /// Finite UTF-8 stdin for this run. Missing input is an immediate EOF.
    pub input: Option<String>,
    /// PLAN-746 (PG-MEM-1): execution time limit in seconds (1..=60,
    /// default 10). The VM cooperatively terminates the run once the
    /// deadline passes and the response carries a structured timeout error.
    pub timeout_secs: Option<u64>,
    /// PLAN-752: prepend the AAVM v1 bootstrap lib (packages/aavm/lib-legacy,
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

pub async fn run_handler(Json(req): Json<RunRequest>) -> Result<Json<RunResponse>, AppError> {
    // PLAN-746 (PG-MEM-1): clamp 1..=60, default 10s.
    let timeout_secs = req.timeout_secs.unwrap_or(10).clamp(1, 60);
    let deadline = Some(std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs));
    let input = Some(req.input.unwrap_or_default());
    let source = req.source;
    let project_dir = req.project_dir;
    let files = req.files;
    let entry = req.entry;
    let prepend_lib = req.prepend_lib.unwrap_or(false);
    let result = tokio::task::spawn_blocking(move || {
        match project_dir {
            Some(dir) => {
                vm_runner::run_project_source_with_input(&source, &dir, files, deadline, input)
            }
            // Files-only projects (manifest notes): materialize and run from
            // the temp root (entry main.at); module resolution via source_dirs.
            None => match files {
                Some(files) if !files.is_empty() => vm_runner::run_files_project_with_input(
                    &source,
                    files,
                    entry.as_deref(),
                    deadline,
                    input,
                ),
                _ if entry.is_some() => vm_runner::run_files_project_with_input(
                    &source,
                    Vec::new(),
                    entry.as_deref(),
                    deadline,
                    input,
                ),
                _ => vm_runner::run_source_with_input(&source, deadline, prepend_lib, input),
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
