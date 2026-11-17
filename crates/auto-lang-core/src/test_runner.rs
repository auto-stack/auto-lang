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
        self.reports
            .iter()
            .filter(|r| matches!(r.outcome, TestOutcome::Passed))
            .count()
    }

    pub fn failed(&self) -> usize {
        self.reports
            .iter()
            .filter(|r| matches!(r.outcome, TestOutcome::Failed(_)))
            .count()
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
    let failures: Vec<_> = result
        .reports
        .iter()
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
    let addr = global_symbols
        .get(&test.name)
        .or_else(|| global_symbols.get(&qualified_name));

    let addr = match addr {
        Some(&a) => a as usize,
        None => {
            return TestReport {
                name: test.name.clone(),
                qualified_name,
                outcome: TestOutcome::Failed(format!(
                    "test function '{}' not found in compiled code",
                    test.name
                )),
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
    let task_arc = vm.tasks.get(&task_id).map(|r| r.value().clone());

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
    vm.remove_task(task_id);

    // Capture stdout
    let stdout = vm
        .output_buffer
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

/// PLAN-1066 T-03[B]: the migration document is the ONLY author source of a
/// corpus entry. Selection comes from `<stem>.case.ad` presence; the virtual
/// source path and the oracle channels come from the decoded document.
///
/// The frozen `.at`/`.as`/`.expected.*` files are history. They are consulted
/// only by the `legacy_*_selection` helpers below, and only to keep a REMOVED
/// (or unreadable) document visible as a loud per-case failure: an obligation
/// must never shrink into a silently smaller corpus, and a sidecar must never
/// stand in for the document.
fn legacy_vm_selection(
    case_path: &Path,
    stem: &str,
) -> Option<(PathBuf, Option<PathBuf>, Option<PathBuf>, Option<PathBuf>)> {
    let at = case_path.join(format!("{stem}.at"));
    let as_ = case_path.join(format!("{stem}.as"));
    let source = if at.is_file() {
        at
    } else if as_.is_file() {
        as_
    } else {
        return None;
    };
    let expected_out =
        Some(case_path.join(format!("{stem}.expected.out"))).filter(|p: &PathBuf| p.is_file());
    let expected_result =
        Some(case_path.join(format!("{stem}.expected.result"))).filter(|p: &PathBuf| p.is_file());
    let expected_error =
        Some(case_path.join(format!("{stem}.expected.error"))).filter(|p: &PathBuf| p.is_file());
    if expected_out.is_none() && expected_result.is_none() && expected_error.is_none() {
        return None;
    }
    Some((source, expected_out, expected_result, expected_error))
}

fn legacy_a2r_selection(case_path: &Path, stem: &str) -> Option<(PathBuf, PathBuf)> {
    let source = case_path.join(format!("{stem}.at"));
    let expected = case_path.join(format!("{stem}.expected.rs"));
    (source.is_file() && expected.is_file()).then_some((source, expected))
}

fn legacy_a2c_selection(
    case_path: &Path,
    stem: &str,
) -> Option<(PathBuf, PathBuf, PathBuf, Option<PathBuf>)> {
    let source = case_path.join(format!("{stem}.at"));
    if !source.is_file() {
        return None;
    }
    let expected_c = case_path.join(format!("{stem}.expected.c"));
    let expected_h = case_path.join(format!("{stem}.expected.h"));
    let expected_error = Some(case_path.join(format!("{stem}.expected.error.log")))
        .filter(|p: &PathBuf| p.is_file());
    (expected_c.is_file() || expected_error.is_some()).then_some((
        source,
        expected_c,
        expected_h,
        expected_error,
    ))
}

fn legacy_a2ts_selection(case_path: &Path, stem: &str) -> Option<(PathBuf, PathBuf)> {
    let source = case_path.join(format!("{stem}.at"));
    let expected = case_path.join(format!("{stem}.expected.ts"));
    (source.is_file() && expected.is_file()).then_some((source, expected))
}

/// Decode one entry's document facts; `Ok(None)` = no document at all.
fn entry_doc_facts(
    case_path: &Path,
    stem: &str,
    family: &str,
) -> Result<Option<crate::legacy_doc_for_trans::DocFacts>, String> {
    crate::legacy_doc_for_trans::doc_facts(case_path, stem)
        .map_err(|e| format!("{family} entry {}: {e}", case_path.display()))
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

                    // PLAN-1066 T-03[B]: the document decides selection and
                    // the oracle channels; a doc-only tree still discovers.
                    let facts = entry_doc_facts(&case_path, &stem, "vm");
                    let (source_file, expected_out, expected_result, expected_error) = match facts {
                        Ok(Some(facts)) => (
                            case_path.join(&facts.source_path),
                            facts
                                .has_check("vm-stdout")
                                .then(|| case_path.join(format!("{stem}.expected.out"))),
                            facts
                                .has_check("vm-result")
                                .then(|| case_path.join(format!("{stem}.expected.result"))),
                            facts
                                .has_check("vm-error")
                                .then(|| case_path.join(format!("{stem}.expected.error"))),
                        ),
                        // No document: keep a frozen-selected entry visible so
                        // the run fails on the missing document instead of the
                        // corpus silently losing an obligation.
                        Ok(None) => match legacy_vm_selection(&case_path, &stem) {
                            Some(entry) => entry,
                            // F-1066B-R7: a document-only entry has NO
                            // stem-named oracle to fall back on (the ten
                            // registered ones carry their oracle inside the
                            // document), so a deleted document would otherwise
                            // drop the obligation silently. A stem-named source
                            // with neither document nor frozen oracle is still
                            // an entry: surface it and let execution name the
                            // missing document. (Only the VM family has such
                            // registered entries; a2r/a2c doc-less directories
                            // were never selected and must not become selected.)
                            None => {
                                let at = case_path.join(format!("{stem}.at"));
                                let as_ = case_path.join(format!("{stem}.as"));
                                if at.is_file() || as_.is_file() {
                                    (
                                        if at.is_file() { at } else { as_ },
                                        Some(case_path.join(format!("{stem}.expected.out"))),
                                        None,
                                        None,
                                    )
                                } else {
                                    continue;
                                }
                            }
                        },
                        // Unreadable document: the entry stays selected; its
                        // execution reports the document error by path.
                        Err(_) => match legacy_vm_selection(&case_path, &stem) {
                            Some(entry) => entry,
                            None => (
                                case_path.join(format!("{stem}.at")),
                                Some(case_path.join(format!("{stem}.expected.out"))),
                                None,
                                None,
                            ),
                        },
                    };

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

// =============================================================================
// Plan 263: A2R transpiler file-based test support
// =============================================================================

/// A single a2r transpiler test case (directory with .at source + .expected.rs).
#[derive(Debug, Clone)]
pub struct A2rTestCase {
    pub name: String, // "a2r/01_basics/001_hello"
    pub dir: PathBuf,
    pub source_file: PathBuf,   // hello.at
    pub expected_file: PathBuf, // hello.expected.rs
}

/// Discover all a2r transpiler test cases under `test/a2r/` (or `test/cookbook/`).
/// Walks category/nnn_name/ directories, finds .at + .expected.rs pairs.
pub fn discover_a2r_tests(test_dir: &Path, suite_name: &str) -> Vec<A2rTestCase> {
    let mut cases = Vec::new();
    if !test_dir.is_dir() {
        return cases;
    }

    // Walk category directories
    if let Ok(entries) = std::fs::read_dir(test_dir) {
        let mut cat_entries: Vec<_> = entries.flatten().collect();
        cat_entries.sort_by_key(|e| e.file_name());

        for cat_entry in cat_entries {
            let cat_path = cat_entry.path();
            if !cat_path.is_dir() {
                continue;
            }

            // Walk test case directories: 001_hello, 002_sqrt, etc.
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

                    // PLAN-1066 T-03[B]: the document is the author source.
                    let (source_file, expected_file) =
                        match entry_doc_facts(&case_path, &stem, "a2r") {
                            Ok(Some(facts)) => {
                                let artifact =
                                    facts.artifacts.iter().find(|p| p.ends_with(".rs")).cloned();
                                match artifact {
                                    Some(path) => {
                                        (case_path.join(&facts.source_path), case_path.join(path))
                                    }
                                    None => match legacy_a2r_selection(&case_path, &stem) {
                                        Some(entry) => entry,
                                        None => continue,
                                    },
                                }
                            }
                            Ok(None) => match legacy_a2r_selection(&case_path, &stem) {
                                Some(entry) => entry,
                                None => continue,
                            },
                            Err(_) => match legacy_a2r_selection(&case_path, &stem) {
                                Some(entry) => entry,
                                None => (
                                    case_path.join(format!("{stem}.at")),
                                    case_path.join(format!("{stem}.expected.rs")),
                                ),
                            },
                        };

                    let cat_name = cat_entry.file_name().to_string_lossy().to_string();
                    let name = format!("{}/{}/{}", suite_name, cat_name, case_dir_name);

                    cases.push(A2rTestCase {
                        name,
                        dir: case_path,
                        source_file,
                        expected_file,
                    });
                }
            }
        }
    }

    cases
}

/// Format file-based test results in cargo test style.

// =============================================================================
// Plan 263 Phase 3: A2C transpiler file-based test support
// =============================================================================

/// A single a2c transpiler test case (directory with .at source + .expected.c/.h).
#[derive(Debug, Clone)]
pub struct A2cTestCase {
    pub name: String,
    pub dir: PathBuf,
    pub source_file: PathBuf,
    pub expected_c: PathBuf,
    pub expected_h: PathBuf,
    pub expected_error: Option<PathBuf>,
}

/// Discover all a2c transpiler test cases under `test/a2c/`.
/// Walks category/nnn_name/ directories, finds .at + .expected.c pairs.
pub fn discover_a2c_tests(test_dir: &Path) -> Vec<A2cTestCase> {
    let mut cases = Vec::new();
    if !test_dir.is_dir() {
        return cases;
    }

    if let Ok(entries) = std::fs::read_dir(test_dir) {
        let mut cat_entries: Vec<_> = entries.flatten().collect();
        cat_entries.sort_by_key(|e| e.file_name());

        for cat_entry in cat_entries {
            let cat_path = cat_entry.path();
            if !cat_path.is_dir() {
                continue;
            }
            let cat_name = cat_entry.file_name().to_string_lossy().to_string();

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

                    // PLAN-1066 T-03[B]: the document is the author source;
                    // a documented case without a C artifact is not an a2c
                    // emit obligation.
                    let (source_file, expected_c, expected_h, expected_error) =
                        match entry_doc_facts(&case_path, &stem, "a2c") {
                            Ok(Some(facts)) => {
                                let c = facts.artifacts.iter().find(|p| p.ends_with(".c")).cloned();
                                match c {
                                    Some(path) => (
                                        case_path.join(&facts.source_path),
                                        case_path.join(path),
                                        facts
                                            .artifacts
                                            .iter()
                                            .find(|p| p.ends_with(".h"))
                                            .map(|p| case_path.join(p))
                                            .unwrap_or_else(|| {
                                                case_path.join(format!("{stem}.expected.h"))
                                            }),
                                        facts.has_check("a2c-error").then(|| {
                                            case_path.join(format!("{stem}.expected.error.log"))
                                        }),
                                    ),
                                    None => match legacy_a2c_selection(&case_path, &stem) {
                                        Some(entry) => entry,
                                        None => continue,
                                    },
                                }
                            }
                            Ok(None) => match legacy_a2c_selection(&case_path, &stem) {
                                Some(entry) => entry,
                                None => continue,
                            },
                            Err(_) => match legacy_a2c_selection(&case_path, &stem) {
                                Some(entry) => entry,
                                None => (
                                    case_path.join(format!("{stem}.at")),
                                    case_path.join(format!("{stem}.expected.c")),
                                    case_path.join(format!("{stem}.expected.h")),
                                    None,
                                ),
                            },
                        };

                    let name = format!("{}/{}", cat_name, case_dir_name);

                    cases.push(A2cTestCase {
                        name,
                        dir: case_path,
                        source_file,
                        expected_c,
                        expected_h,
                        expected_error,
                    });
                }
            }
        }
    }

    cases
}

// =============================================================================
// Plan 263 Phase 3: A2TS transpiler file-based test support
// =============================================================================

/// A single a2ts transpiler test case (directory with .at source + .expected.ts).
#[derive(Debug, Clone)]
pub struct A2tsTestCase {
    pub name: String,
    pub dir: PathBuf,
    pub source_file: PathBuf,
    pub expected_file: PathBuf,
}

/// Discover all a2ts transpiler test cases under `test/a2ts/`.
/// Walks category/nnn_name/ directories, finds .at + .expected.ts pairs.
pub fn discover_a2ts_tests(test_dir: &Path) -> Vec<A2tsTestCase> {
    let mut cases = Vec::new();
    if !test_dir.is_dir() {
        return cases;
    }

    if let Ok(entries) = std::fs::read_dir(test_dir) {
        let mut cat_entries: Vec<_> = entries.flatten().collect();
        cat_entries.sort_by_key(|e| e.file_name());

        for cat_entry in cat_entries {
            let cat_path = cat_entry.path();
            if !cat_path.is_dir() {
                continue;
            }
            let cat_name = cat_entry.file_name().to_string_lossy().to_string();

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

                    // PLAN-1066 T-03[B]: the document is the author source.
                    let (source_file, expected_file) =
                        match entry_doc_facts(&case_path, &stem, "a2ts") {
                            Ok(Some(facts)) => {
                                let artifact =
                                    facts.artifacts.iter().find(|p| p.ends_with(".ts")).cloned();
                                match artifact {
                                    Some(path) => {
                                        (case_path.join(&facts.source_path), case_path.join(path))
                                    }
                                    None => match legacy_a2ts_selection(&case_path, &stem) {
                                        Some(entry) => entry,
                                        None => continue,
                                    },
                                }
                            }
                            Ok(None) => match legacy_a2ts_selection(&case_path, &stem) {
                                Some(entry) => entry,
                                None => continue,
                            },
                            Err(_) => match legacy_a2ts_selection(&case_path, &stem) {
                                Some(entry) => entry,
                                None => (
                                    case_path.join(format!("{stem}.at")),
                                    case_path.join(format!("{stem}.expected.ts")),
                                ),
                            },
                        };

                    let name = format!("{}/{}", cat_name, case_dir_name);

                    cases.push(A2tsTestCase {
                        name,
                        dir: case_path,
                        source_file,
                        expected_file,
                    });
                }
            }
        }
    }

    cases
}
