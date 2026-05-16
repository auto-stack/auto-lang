# Plan 262: File-Based Test Framework for `auto test`

## Context

`auto test` (Plan 260) currently only discovers `#[test]` functions in `.at` files. But the project has 427 VM file-based tests in `test/vm/` that use a different paradigm: an `.at` source file + `.expected.out` (or `.expected.result` / `.expected.error`). These are registered manually in Rust via ~790 lines of `#[test]` boilerplate in `vm_file_tests.rs`.

This plan extends `auto test` to auto-discover and run file-based tests alongside `#[test]` functions. Phase 1 covers VM execution tests only (~427 tests). Cookbook, transpiler, and AST tests come in future phases.

## Design Decisions

- **Auto-discovery**: No `--mode` flag. `auto test` discovers both `#[test]` functions AND file-based tests in one run.
- **Test root**: Conventional `test/` directory under project root. Suite type is determined by subdirectory name (`test/vm/` = VM execution tests).
- **Phase 1 scope**: VM execution tests only (`test/vm/`), including AAVM bootstrap tests (`99_bootstrap/`).
- **Zero data changes**: Existing `.at` and `.expected.*` files are used as-is.

## File-Based Test Conventions

A test case is a directory containing an `.at` source file and one or more `.expected.*` files:

```
test/vm/01_basics/001_hello/
    hello.at               # Source file
    hello.expected.out     # Expected stdout (or)
    hello.expected.result  # Expected return value (or)
    hello.expected.error   # Expected runtime error
```

**Name extraction**: Directory `001_hello` → source file `hello.at` (drop numeric prefix).

**Suite detection**: If `--dir` path starts with `test/vm/` or contains `test/vm/`, it's a VM test suite.

**Bootstrap (AAVM)**: Categories under `99_bootstrap/` automatically prepend `auto/lib/*.at` compiler source.

## Implementation Steps

### Step 1 — Add file-based test types to `test_runner.rs`

New types alongside existing `TestInfo`/`TestReport`:

```rust
struct FileTestCase {
    name: String,            // "vm/01_basics/001_hello"
    dir: PathBuf,            // Absolute path to test case dir
    source_file: PathBuf,    // The .at input file
    is_bootstrap: bool,      // true for 99_bootstrap/*
    expected_out: Option<PathBuf>,
    expected_result: Option<PathBuf>,
    expected_error: Option<PathBuf>,
}

struct FileTestReport {
    name: String,
    outcome: TestOutcome,
    duration_ms: u128,
    stdout: String,
}
```

### Step 2 — Add discovery function to `test_runner.rs`

```rust
pub fn discover_vm_tests(test_root: &Path) -> Vec<FileTestCase>
```

Walks `test/vm/` recursively, identifies test case directories (those containing `.at` files with matching `.expected.*` files), extracts names, detects bootstrap category.

### Step 3 — Add VM test execution to `lib.rs`

```rust
pub async fn run_vm_file_test(case: &FileTestCase) -> FileTestReport
pub fn run_vm_file_test_sync(case: &FileTestCase) -> FileTestReport
```

Logic (adapted from existing `test_vm()` in `vm_file_tests.rs`):
1. Read source file. If bootstrap, prepend `auto/lib/*.at`.
2. If `.expected.error` exists: run with `run()`, assert `is_err()`.
3. Otherwise: run with `run_with_capture()`, compare stdout against `.expected.out` and/or return value against `.expected.result`.
4. On mismatch: write `.wrong.out` / `.wrong.result` file.

Reuse existing `run()` and `run_with_capture()` from `lib.rs`.

### Step 4 — Extend CLI in `main.rs`

In the `Commands::Test` handler, after running `#[test]` function tests:

1. Check if `test/vm/` directory exists relative to CWD.
2. If so, call `discover_vm_tests()`.
3. Apply `--filter` to test case names.
4. Run each test case via `run_vm_file_test_sync()`.
5. Print results in same `cargo test` style.
6. Include file-based test counts in the summary.

File-based tests are printed with `vm/` prefix:
```
test vm/01_basics/001_hello ... ok
test vm/01_basics/002_arithmetic ... ok
```

### Step 5 — Handle `.wrong.*` file generation

On assertion failure, write the actual output to a `.wrong.*` file alongside the expected file. This matches the existing Rust test behavior and aids debugging.

### Step 6 — Verify against existing Rust tests

1. Run `cargo test -p auto-lang -- test_01_basics` to get Rust baseline.
2. Run `auto test --dir test/vm/01_basics` to verify same results.
3. Run `auto test` to verify both `#[test]` functions and file-based tests coexist.

## Files to Modify

| File | Change |
|------|--------|
| `crates/auto-lang/src/test_runner.rs` | Add `FileTestCase`, `FileTestReport`, `discover_vm_tests()` |
| `crates/auto-lang/src/lib.rs` | Add `run_vm_file_test()`, export new types |
| `crates/auto/src/main.rs` | Extend `Commands::Test` handler with file-based test path |

## Verification

1. `auto test --dir test/vm/01_basics` — discovers and runs 11 VM tests
2. `auto test` — runs both `#[test]` function tests (tests/auto/) AND file-based VM tests (test/vm/)
3. `auto test --filter 001_hello` — filters file-based tests by name
4. A failing test generates `.wrong.out` file next to `.expected.out`
5. Bootstrap tests (99_bootstrap/) correctly prepend auto/lib code
6. `cargo test -p auto-lang` — existing Rust tests still pass
