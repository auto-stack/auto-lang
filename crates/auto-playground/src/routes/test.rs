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
        let runtime = auto_lang_core::get_global_runtime();
        let result = runtime.block_on(auto_lang_core::test_code_with_path_deadline_and_input(
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
                auto_lang_core::test_runner::TestOutcome::Passed => (true, None),
                auto_lang_core::test_runner::TestOutcome::Failed(error) => (false, Some(error)),
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

#[cfg(test)]
mod tests {
    use super::{test_handler, TestRequest, TestResponse};
    use axum::Json;
    use std::time::{Duration, Instant};

    async fn run_test_request(
        source: &str,
        input: Option<&str>,
        timeout_secs: Option<u64>,
    ) -> TestResponse {
        let result = test_handler(Json(TestRequest {
            source: source.to_string(),
            files: None,
            entry: None,
            input: input.map(str::to_string),
            timeout_secs,
        }))
        .await;
        match result {
            Ok(Json(response)) => response,
            Err(_) => panic!("test handler should return a test report"),
        }
    }

    #[tokio::test]
    async fn handler_reports_passing_and_failing_assertions() {
        let response = run_test_request(
            r#"
#[test]
fn correct_assertion() {
    assert_eq(2 + 2, 4)
}

#[test]
fn incorrect_assertion() {
    assert_eq(2 + 2, 5)
}
"#,
            None,
            Some(3),
        )
        .await;

        assert_eq!(response.executed, 2);
        assert_eq!(response.passed, 1);
        assert_eq!(response.failed, 1);
        let failure = response
            .reports
            .iter()
            .find(|report| !report.passed)
            .expect("the incorrect assertion should fail");
        assert!(
            failure
                .error
                .as_deref()
                .is_some_and(|error| error.contains("Assertion failed")),
            "failure should expose an assertion diagnostic: {:?}",
            failure.error
        );
    }

    #[tokio::test]
    async fn timed_out_handler_request_returns_and_the_next_request_runs() {
        let started = Instant::now();
        let timed_out = run_test_request(
            r#"
#[test]
fn never_finishes() {
    while true {}
}
"#,
            None,
            Some(1),
        )
        .await;

        assert!(started.elapsed() < Duration::from_secs(5));
        assert_eq!(timed_out.executed, 1);
        assert_eq!(timed_out.failed, 1);
        assert!(timed_out.reports[0]
            .error
            .as_deref()
            .is_some_and(|error| error.contains("ExecutionTimeout")));

        let next = run_test_request(
            "#[test]\nfn next_request_works() { assert_eq(2 + 2, 4) }",
            None,
            Some(3),
        )
        .await;
        assert_eq!(next.executed, 1);
        assert_eq!(next.passed, 1);
        assert_eq!(next.failed, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_handler_requests_keep_input_isolated() {
        let source = |expected: &str| {
            format!(
                "use auto.io\n#[test]\nfn input_matches_request() {{\n    let value = io.read_line_opt()\n    is value {{\n        Some(line) -> assert_eq(line, \"{expected}\"),\n        None -> assert_eq(\"EOF\", \"{expected}\")\n    }}\n}}"
            )
        };
        let left_source = source("left");
        let right_source = source("right");
        let (left, right) = tokio::join!(
            run_test_request(&left_source, Some("left\n"), Some(3)),
            run_test_request(&right_source, Some("right\n"), Some(3)),
        );

        for response in [left, right] {
            assert_eq!(response.executed, 1);
            assert_eq!(response.passed, 1);
            assert_eq!(response.failed, 0);
        }
    }
}
