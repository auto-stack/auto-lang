//! Artifact helper compatibility over the existing std-only owned executor.
//! No second Job/reader implementation; caller deadlines and partial captures
//! stay observable, while containment/reader/cleanup failures remain errors.

use auto_tool_execution::{run_command_capture, ExecutionError, ExitKind, TreeScope};
use std::process::Command;
use std::time::Duration;

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub struct RunResult {
    pub code: Option<i32>,
    pub timed_out: bool,
    pub stdout: String,
    pub stderr: String,
    pub elapsed_ms: u64,
}

pub fn run_deadlined(command: Command, timeout: Duration) -> std::io::Result<RunResult> {
    let result =
        run_command_capture(command, TreeScope::ProcessTree, Some(timeout)).map_err(|error| {
            match error {
                ExecutionError::Spawn { source, .. } | ExecutionError::Io { source, .. } => source,
                other => std::io::Error::other(other),
            }
        })?;
    let (code, timed_out) = match result.kind {
        ExitKind::Success => (Some(0), false),
        ExitKind::Failure(code) => (Some(code), false),
        ExitKind::Timeout { .. } => (None, true),
        ExitKind::Terminated { .. } => (None, false),
        ExitKind::Cancelled => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "owned artifact helper cancelled",
            ))
        }
    };
    let stdout = result
        .stdout
        .ok_or_else(|| std::io::Error::other("missing owned stdout capture"))?;
    let stderr = result
        .stderr
        .ok_or_else(|| std::io::Error::other("missing owned stderr capture"))?;
    Ok(RunResult {
        code,
        timed_out,
        stdout: String::from_utf8(stdout)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?,
        stderr: String::from_utf8(stderr)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?,
        elapsed_ms: result.elapsed.as_millis() as u64,
    })
}

#[cfg(all(test, windows))]
mod adapter_controls {
    use super::*;
    fn fixture() -> std::path::PathBuf {
        std::env::var_os("AUTO_TOOL_EXECUTION_FIXTURE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../../../target/debug/te-child.exe")
            })
    }
    #[test]
    fn timeout_retains_the_original_partial_stdout_contract() {
        let mut command = Command::new(fixture());
        command
            .env("TE_CHILD_MODE", "stdout-kib")
            .env("TE_CHILD_ARG1", "1")
            .env("TE_CHILD_ARG2", "30");
        let result = run_deadlined(command, Duration::from_millis(500)).unwrap();
        assert!(result.timed_out);
        assert_eq!(result.code, None);
        assert_eq!(result.stdout, "x".repeat(1024));
        assert!(result.stderr.is_empty());
    }
    #[test]
    fn nonzero_exit_is_preserved_without_timeout() {
        let mut command = Command::new(fixture());
        command
            .env("TE_CHILD_MODE", "exit")
            .env("TE_CHILD_ARG1", "7");
        let result = run_deadlined(command, Duration::from_secs(5)).unwrap();
        assert_eq!(result.code, Some(7));
        assert!(!result.timed_out);
    }
    #[test]
    fn missing_program_remains_a_spawn_error() {
        let command = Command::new(fixture().with_file_name("missing-896-fixture.exe"));
        let error = run_deadlined(command, Duration::from_secs(5))
            .err()
            .expect("missing executable");
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn invalid_utf8_cannot_be_normalized_into_a_valid_json_response() {
        let mut command = Command::new(fixture());
        command.env("TE_CHILD_MODE", "stdout-invalid-utf8");
        let error = run_deadlined(command, Duration::from_secs(5))
            .err()
            .expect("invalid response bytes");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }
}
