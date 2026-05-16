/// Plan 260: Test framework for Auto — test discovery, execution, and reporting.
/// Plan 262: Extended with file-based test support for VM execution tests.
///
/// Mimics Rust's `cargo test`: discovers `#[test]` functions via AST walk,
/// executes each in an isolated VM task, and reports results.
/// Also discovers and runs file-based tests (source + expected output).
use crate::ast::Stmt;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Metadata about a discovered test function.
#[derive(Debug, Clone)]
pub struct TestInfo {
    pub name: String,
    pub module_path: String,
    pub qualified_name: String,
}

/// Collection of discovered tests.
#[derive(Debug, Clone, Default)]
pub struct TestRegistry {
    pub tests: Vec<TestInfo>,
}

/// Outcome of a single test execution.
#[derive(Debug, Clone)]
pub enum TestOutcome {
    Passed,
    Failed(String),
}

/// Report for a single test execution.
#[derive(Debug, Clone)]
pub struct TestReport {
    pub name: String,
    pub qualified_name: String,
    pub outcome: TestOutcome,
    pub duration_ms: u128,
    pub stdout: String,
}

/// Overall test results.
#[derive(Debug, Clone, Default)]
pub struct TestResult {
    pub reports: Vec<TestReport>,
}

impl TestResult {
    pub fn passed(&self) -> usize {
        self.reports.iter().filter(|r| matches!(r.outcome, TestOutcome::Passed)).count()
    }

    pub fn failed(&self) -> usize {
        self.reports.iter().filter(|r| matches!(r.outcome, TestOutcome::Failed(_))).count()
    }

    pub fn has_failures(&self) -> bool {
        self.failed() > 0
    }
}

/// Walk the AST and collect all `#[test]` functions.
pub fn collect_tests(stmts: &[Stmt], module_path: &str) -> TestRegistry {
    let mut registry = TestRegistry::default();
    for stmt in stmts {
        if let Stmt::Fn(f) = stmt {
            if f.is_test {
                let name = f.name.to_string();
                let qualified_name = if module_path.is_empty() {
                    name.clone()
                } else {
                    format!("{}::{}", module_path, name)
                };
                registry.tests.push(TestInfo {
                    name,
                    module_path: module_path.to_string(),
                    qualified_name,
                });
            }
        }
    }
    registry
}

/// Format test results in Rust `cargo test` style.
pub fn format_test_report(result: &TestResult, elapsed: u128) -> String {
    let mut output = String::new();

    // Print individual results
    for report in &result.reports {
        match &report.outcome {
            TestOutcome::Passed => {
                output.push_str(&format!("test {} ... ok\n", report.qualified_name));
            }
            TestOutcome::Failed(msg) => {
                output.push_str(&format!("test {} ... FAILED\n", report.qualified_name));
                output.push_str(&format!("    {}\n", msg));
            }
        }
    }

    // Print failure summary
    let failures: Vec<_> = result.reports.iter()
        .filter(|r| matches!(r.outcome, TestOutcome::Failed(_)))
        .collect();

    if !failures.is_empty() {
        output.push_str("\nfailures:\n");
        for report in failures {
            if let TestOutcome::Failed(msg) = &report.outcome {
                output.push_str(&format!("    {}: {}\n", report.qualified_name, msg));
            }
        }
        output.push('\n');
    }

    // Summary line
    let secs = elapsed as f64 / 1000.0;
    output.push_str(&format!(
        "test result: {} passed, {} failed, finished in {:.3}s\n",
        result.passed(),
        result.failed(),
        secs,
    ));

    output
}

/// Execute a single test function in the VM.
/// Returns a TestReport with the outcome.
pub async fn run_test_in_vm(
    vm: &crate::vm::engine::AutoVM,
    test: &TestInfo,
    global_symbols: &std::collections::HashMap<String, u32>,
) -> TestReport {
    let qualified_name = test.qualified_name.clone();

    // Look up the test function address
    // Try both plain name and qualified name
    let addr = global_symbols.get(&test.name)
        .or_else(|| global_symbols.get(&qualified_name));

    let addr = match addr {
        Some(&a) => a as usize,
        None => {
            return TestReport {
                name: test.name.clone(),
                qualified_name,
                outcome: TestOutcome::Failed(format!("test function '{}' not found in compiled code", test.name)),
                duration_ms: 0,
                stdout: String::new(),
            };
        }
    };

    // Clear output buffer before each test
    if let Some(buf) = &vm.output_buffer {
        buf.write().unwrap().clear();
    }

    let start = Instant::now();

    // Spawn a fresh task for this test
    let task_id = vm.spawn_task(addr, 65536);

    // Run the task to completion
    vm.run_task_loop().await;

    let duration_ms = start.elapsed().as_millis();

    // Check task result
    let task_arc = vm.tasks.get(&task_id)
        .map(|r| r.value().clone());

    let outcome = match task_arc {
        Some(task_mutex) => {
            let task = task_mutex.lock().await;
            if let Some(error) = &task.last_error {
                TestOutcome::Failed(error.clone())
            } else {
                TestOutcome::Passed
            }
        }
        None => TestOutcome::Failed("task disappeared during execution".to_string()),
    };

    // Clean up the task
    vm.tasks.remove(&task_id);

    // Capture stdout
    let stdout = vm.output_buffer
        .as_ref()
        .map(|buf| buf.read().unwrap().clone())
        .unwrap_or_default();

    TestReport {
        name: test.name.clone(),
        qualified_name,
        outcome,
        duration_ms,
        stdout,
    }
}

// =============================================================================
// Plan 262: File-based test support
// =============================================================================

/// A single file-based test case (directory with .at source + .expected.* files).
#[derive(Debug, Clone)]
pub struct FileTestCase {
    pub name: String,
    pub dir: PathBuf,
    pub source_file: PathBuf,
    pub is_bootstrap: bool,
    pub expected_out: Option<PathBuf>,
    pub expected_result: Option<PathBuf>,
    pub expected_error: Option<PathBuf>,
}

/// Report for a single file-based test execution.
#[derive(Debug, Clone)]
pub struct FileTestReport {
    pub name: String,
    pub outcome: TestOutcome,
    pub duration_ms: u128,
    pub stdout: String,
}

/// Extract test name from directory name: "001_hello" → "hello".
fn extract_test_stem(dir_name: &str) -> String {
    let parts: Vec<&str> = dir_name.splitn(2, '_').collect();
    parts.get(1).unwrap_or(&dir_name).to_string()
}

/// Discover all VM file-based test cases under `test/vm/`.
/// Walks category/nnn_name/ directories, finds .at + .expected.* pairs.
pub fn discover_vm_tests(test_vm_dir: &Path) -> Vec<FileTestCase> {
    let mut cases = Vec::new();
    if !test_vm_dir.is_dir() {
        return cases;
    }

    // Walk category directories: 01_basics, 09_functions, etc.
    if let Ok(entries) = std::fs::read_dir(test_vm_dir) {
        let mut cat_entries: Vec<_> = entries.flatten().collect();
        cat_entries.sort_by_key(|e| e.file_name());

        for cat_entry in cat_entries {
            let cat_path = cat_entry.path();
            if !cat_path.is_dir() {
                continue;
            }
            let cat_name = cat_entry.file_name().to_string_lossy().to_string();
            let is_bootstrap = cat_name == "99_bootstrap";

            // Walk test case directories: 001_hello, 002_arithmetic, etc.
            if let Ok(case_entries) = std::fs::read_dir(&cat_path) {
                let mut case_entries: Vec<_> = case_entries.flatten().collect();
                case_entries.sort_by_key(|e| e.file_name());

                for case_entry in case_entries {
                    let case_path = case_entry.path();
                    if !case_path.is_dir() {
                        continue;
                    }

                    let case_dir_name = case_entry.file_name().to_string_lossy().to_string();
                    let stem = extract_test_stem(&case_dir_name);

                    // Look for source file
                    let source_file = case_path.join(format!("{}.at", stem));
                    if !source_file.is_file() {
                        continue;
                    }

                    // Look for expected files
                    let expected_out = {
                        let p = case_path.join(format!("{}.expected.out", stem));
                        if p.is_file() { Some(p) } else { None }
                    };
                    let expected_result = {
                        let p = case_path.join(format!("{}.expected.result", stem));
                        if p.is_file() { Some(p) } else { None }
                    };
                    let expected_error = {
                        let p = case_path.join(format!("{}.expected.error", stem));
                        if p.is_file() { Some(p) } else { None }
                    };

                    // Must have at least one expected file
                    if expected_out.is_none() && expected_result.is_none() && expected_error.is_none() {
                        continue;
                    }

                    let name = format!("{}/{}", cat_name, case_dir_name);

                    cases.push(FileTestCase {
                        name,
                        dir: case_path,
                        source_file,
                        is_bootstrap,
                        expected_out,
                        expected_result,
                        expected_error,
                    });
                }
            }
        }
    }

    cases
}

/// Format file-based test results in cargo test style.
pub fn format_file_test_report(reports: &[FileTestReport], elapsed: u128) -> String {
    let mut output = String::new();

    for report in reports {
        match &report.outcome {
            TestOutcome::Passed => {
                output.push_str(&format!("test {} ... ok\n", report.name));
            }
            TestOutcome::Failed(msg) => {
                output.push_str(&format!("test {} ... FAILED\n", report.name));
                output.push_str(&format!("    {}\n", msg));
            }
        }
    }

    let passed = reports.iter().filter(|r| matches!(r.outcome, TestOutcome::Passed)).count();
    let failed = reports.iter().filter(|r| matches!(r.outcome, TestOutcome::Failed(_))).count();

    if failed > 0 {
        output.push_str("\nfailures:\n");
        for report in reports {
            if let TestOutcome::Failed(msg) = &report.outcome {
                output.push_str(&format!("    {}: {}\n", report.name, msg));
            }
        }
        output.push('\n');
    }

    let secs = elapsed as f64 / 1000.0;
    output.push_str(&format!(
        "file test result: {} passed, {} failed, finished in {:.3}s\n",
        passed, failed, secs,
    ));

    output
}
