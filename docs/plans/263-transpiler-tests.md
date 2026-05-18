# Plan 263: A2R Transpiler Test Runner for `auto test`

## Context

`auto test` auto-discovers and runs VM file-based tests (Plan 262). The project also has ~270 a2r (Auto-to-Rust) transpiler tests in `test/a2r/` and ~150 cookbook tests in `test/cookbook/`. These are registered as ~420 individual `#[test]` boilerplate functions in `a2r_tests.rs`. Each test reads an `.at` file, transpiles it to Rust via `transpile_rust()`, and compares the output against `.expected.rs`.

This plan extends `auto test` to auto-discover and run a2r transpiler tests alongside VM tests, following the same file-based test pattern.

## Status: COMPLETE

All steps implemented. `auto test` now auto-discovers a2r and cookbook transpiler tests.

## A2R Test Conventions

```
test/a2r/01_basics/001_hello/
    hello.at               # Input Auto source
    hello.expected.rs      # Expected Rust transpiler output
    hello.wrong.rs         # Written on mismatch (actual output)
```

**Name extraction**: Same as VM tests — directory `001_hello` → source file `hello.at`.

**Comparison**: Read `.at`, call `transpile_rust()`, compare output bytes against `.expected.rs`. On mismatch, write `.wrong.rs`.

## Implementation

### Step 1 — A2R test types in `test_runner.rs` (DONE)

- `A2rTestCase` struct with name, dir, source_file, expected_file
- `discover_a2r_tests(test_a2r_dir, suite_name)` — parameterized by suite name ("a2r" or "cookbook")

### Step 2 — A2R test execution `run_a2r_file_test()` in `lib.rs` (DONE)

1. Read `.at` source file.
2. Call `transpile_rust(&name, &src)`, get output bytes.
3. Read `.expected.rs`.
4. Compare. On mismatch, write `.wrong.rs`.
5. Return `FileTestReport`.

### Step 3 — CLI integration in `main.rs` (DONE)

Discovers tests from `crates/auto-lang/test/a2r/` and `crates/auto-lang/test/cookbook/`. Supports `--dir` and `--filter` filtering.

## Files Modified

| File | Change |
|------|--------|
| `crates/auto-lang/src/test_runner.rs` | `A2rTestCase`, `discover_a2r_tests()` |
| `crates/auto-lang/src/lib.rs` | `run_a2r_file_test()` |
| `crates/auto/src/main.rs` | CLI extended with a2r/cookbook discovery |

## Verification

1. `auto test --filter 001_hello` — discovers and runs a2r hello test
2. `auto test` — runs VM tests + a2r tests + cookbook tests in one run
3. A failing a2r test generates `.wrong.rs` file
4. `cargo test -p auto-lang` — existing Rust tests still pass
