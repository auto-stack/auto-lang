# Plan 262: File-Based Test Framework for `auto test`

## Context

`auto test` (Plan 260) currently only discovers `#[test]` functions in `.at` files. But the project has 427 VM file-based tests in `test/vm/` that use a different paradigm: an `.at` source file + `.expected.out` (or `.expected.result` / `.expected.error`). These are registered manually in Rust via ~790 lines of `#[test]` boilerplate in `vm_file_tests.rs`.

This plan extends `auto test` to auto-discover and run file-based tests alongside `#[test]` functions. Phase 1 covers VM execution tests only (~427 tests). Transpiler tests come in Plan 263.

## Status: COMPLETE

All steps implemented and verified. `auto test` now auto-discovers VM file-based tests.

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

## Implementation

### Step 1 — File-based test types in `test_runner.rs` (DONE)

- `FileTestCase` struct with name, dir, source_file, is_bootstrap, expected_out/result/error
- `FileTestReport` struct with name, outcome, duration_ms, stdout

### Step 2 — Discovery function `discover_vm_tests()` (DONE)

Walks `test/vm/` recursively, identifies test case directories (those containing `.at` files with matching `.expected.*` files), extracts names, detects bootstrap category.

### Step 3 — VM test execution `run_vm_file_test()` in `lib.rs` (DONE)

1. Read source file. If bootstrap, prepend `auto/lib/*.at`.
2. If `.expected.error` exists: run with `run()`, assert `is_err()`.
3. Otherwise: run with `run_with_capture()`, compare stdout against `.expected.out` and/or return value against `.expected.result`.
4. On mismatch: write `.wrong.out` / `.wrong.result` file.

### Step 4 — CLI integration in `main.rs` (DONE)

Discovers tests from `crates/auto-lang/test/vm/` and `test/vm/`. Supports `--dir` and `--filter` filtering.

## Files Modified

| File | Change |
|------|--------|
| `crates/auto-lang/src/test_runner.rs` | `FileTestCase`, `FileTestReport`, `discover_vm_tests()` |
| `crates/auto-lang/src/lib.rs` | `run_vm_file_test()` |
| `crates/auto/src/main.rs` | CLI `Commands::Test` handler extended |

## Verification

1. `auto test --dir crates/auto-lang/test/vm/01_basics` — discovers and runs VM tests
2. `auto test` — runs both `#[test]` function tests AND file-based VM tests
3. `auto test --filter 001_hello` — filters file-based tests by name
4. A failing test generates `.wrong.out` file next to `.expected.out`
5. Bootstrap tests (99_bootstrap/) correctly prepend auto/lib code
6. All 95 aavm bootstrap tests pass
