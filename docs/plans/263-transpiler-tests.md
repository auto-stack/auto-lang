# Plan 263: A2R Transpiler Test Runner for `auto test`

## Context

`auto test` auto-discovers and runs VM file-based tests (Plan 262). The project also has ~270 a2r (Auto-to-Rust) transpiler tests in `test/a2r/` and ~150 cookbook tests in `test/cookbook/`. These are registered as ~420 individual `#[test]` boilerplate functions in `a2r_tests.rs`. Each test reads an `.at` file, transpiles it to Rust via `transpile_rust()`, and compares the output against `.expected.rs`.

This plan extends `auto test` to auto-discover and run a2r transpiler tests alongside VM tests, following the same file-based test pattern.

## Status: IN PROGRESS — Phase 2: Declarative test discovery

Phase 1 (hardcoded paths) is complete. Now migrating to convention-based discovery via `tests/a2r_tests.at`.

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

### Phase 1: Hardcoded a2r discovery (DONE)

- `A2rTestCase` struct, `discover_a2r_tests()`, `run_a2r_file_test()`
- CLI integration with hardcoded paths in `main.rs`
- 5 transpiler bug fixes, lexer hash fix, 269 a2r tests passing

### Phase 2: Declarative test discovery via `tests/a2r_tests.at` (IN PROGRESS)

Replace hardcoded paths with a VM FFI approach:

1. **Register FFI function** `Test.run_a2r_dir(path: String) -> i32`
   - Discovers a2r test cases in the given directory
   - Runs each test (transpile → compare)
   - Prints per-test results to stdout
   - Returns failure count (0 = all pass)

2. **Create `tests/a2r_tests.at`** with `#[test]` functions:
   ```auto
   use auto.test: run_a2r_dir

   #[test]
   fn test_a2r_transpiler() {
       let failures = run_a2r_dir("crates/auto-lang/test/a2r")
       assert_eq(failures, 0)
   }

   #[test]
   fn test_cookbook_transpiler() {
       let failures = run_a2r_dir("crates/auto-lang/test/cookbook")
       assert_eq(failures, 0)
   }
   ```

3. **Remove hardcoded a2r paths** from `main.rs` (lines 716-758)

## Files Modified

| File | Change |
|------|--------|
| `crates/auto-lang/src/vm/ffi/stdlib.rs` | Add `shim_test_run_a2r_dir` FFI shim + register |
| `tests/a2r_tests.at` | New file — declares a2r + cookbook test runners |
| `crates/auto/src/main.rs` | Remove hardcoded a2r discovery block |
| `crates/auto-lang/src/test_runner.rs` | `A2rTestCase`, `discover_a2r_tests()` (Phase 1) |
| `crates/auto-lang/src/lib.rs` | `run_a2r_file_test()` (Phase 1) |

## Verification

1. `auto test --filter test_a2r` — discovers and runs a2r tests via `tests/a2r_tests.at`
2. `auto test` — runs VM tests + a2r tests + cookbook tests in one run
3. `cargo test -p auto-lang` — existing Rust tests still pass
