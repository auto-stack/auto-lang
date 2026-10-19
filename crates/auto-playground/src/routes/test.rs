use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::project::{prepare_files_temp_dir_with_entry, ProjectFile};

#[derive(Deserialize)]
pub struct TestRequest {
    pub source: String,
    pub files: Option<Vec<ProjectFile>>,
    pub entry: Option<String>,
    /// Finite UTF-8 stdin for this test run. Missing input is immediate EOF.
    pub input: Option<String>,
    /// Time limit in seconds (1..=60, default 10), shared by the suite.
    pub timeout_secs: Option<u64>,
}

#[derive(Serialize)]
pub struct TestReport {
    pub name: String,
    pub qualified_name: String,
    pub passed: bool,
    pub error: Option<String>,
    pub duration_ms: u128,
    pub stdout: String,
}

#[derive(Serialize)]
pub struct TestResponse {
    pub executed: usize,
    pub passed: usize,
    pub failed: usize,
    pub reports: Vec<TestReport>,
}

pub async fn test_handler(Json(req): Json<TestRequest>) -> Result<Json<TestResponse>, AppError> {
    let timeout_secs = req.timeout_secs.unwrap_or(10).clamp(1, 60);
    let deadline = Some(std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs));
    let source = req.source;
    let files = req.files.unwrap_or_default();
    let entry = req.entry.unwrap_or_else(|| "main.at".to_string());
    let input = Some(req.input.unwrap_or_default());

    let result = tokio::task::spawn_blocking(move || {
        let (temp_dir, entry_path) =
            prepare_files_temp_dir_with_entry(&source, &files, Some(&entry))?;
        let runtime = auto_lang::get_global_runtime();
        let result = runtime.block_on(auto_lang::test_code_with_path_deadline_and_input(
            &source,
            &entry_path.to_string_lossy(),
            deadline,
            input,
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
        result.map_err(|error| AppError::CompileError(error.to_string()))
    })
    .await
    .map_err(|error| AppError::Internal(error.to_string()))??;

    let reports = result
        .reports
        .into_iter()
        .map(|report| {
            let (passed, error) = match report.outcome {
                auto_lang::test_runner::TestOutcome::Passed => (true, None),
                auto_lang::test_runner::TestOutcome::Failed(error) => (false, Some(error)),
            };
            TestReport {
                name: report.name,
                qualified_name: report.qualified_name,
                passed,
                error,
                duration_ms: report.duration_ms,
                stdout: report.stdout,
            }
        })
        .collect::<Vec<_>>();
    let passed = reports.iter().filter(|report| report.passed).count();
    let failed = reports.len() - passed;

    Ok(Json(TestResponse {
        executed: reports.len(),
        passed,
        failed,
        reports,
    }))
}
