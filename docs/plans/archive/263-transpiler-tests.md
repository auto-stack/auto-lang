# Plan 263: A2R Transpiler Test Runner for `auto test`

## Context

`auto test` auto-discovers and runs VM file-based tests (Plan 262). The project also has ~270 a2r (Auto-to-Rust) transpiler tests in `test/a2r/` and ~150 cookbook tests in `test/cookbook/`. These are registered as ~420 individual `#[test]` boilerplate functions in `a2r_tests.rs`. Each test reads an `.at` file, transpiles it to Rust via `transpile_rust()`, and compares the output against `.expected.rs`.

This plan extends `auto test` to auto-discover and run a2r transpiler tests alongside VM tests, following the same file-based test pattern.

## Status: DONE — Phase 3: All test categories migrated

### Phase 1: Hardcoded a2r discovery (DONE)
### Phase 2: Declarative a2r test discovery via `tests/a2r_tests.at` (DONE)
### Phase 3: Migrate VM, A2C, A2TS tests to same pattern (DONE)

## Test Conventions

Each test category has a `tests/xxx_tests.at` file that declares `#[test]` functions calling `Test.run_xxx_dir(path)`:

| File | FFI Function | Native ID | Directory |
|------|-------------|-----------|-----------|
| `tests/a2r_tests.at` | `Test.run_a2r_dir(path)` | 2826 | `test/a2r/`, `test/cookbook/` |
| `tests/vm_tests.at` | `Test.run_vm_dir(path)` | 2827 | `test/vm/` |
| `tests/a2c_tests.at` | `Test.run_a2c_dir(path)` | 2828 | `test/a2c/` |
| `tests/a2ts_tests.at` | `Test.run_a2ts_dir(path)` | 2829 | `test/a2ts/` |

## Implementation

### Phase 1: Hardcoded a2r discovery (DONE)

- `A2rTestCase` struct, `discover_a2r_tests()`, `run_a2r_file_test()`
- CLI integration with hardcoded paths in `main.rs`
- 5 transpiler bug fixes, lexer hash fix, 269 a2r tests passing

### Phase 2: Declarative test discovery via `tests/a2r_tests.at` (DONE)

- FFI shim `shim_test_run_a2r_dir` in `stdlib.rs`
- Native ID 2826 in `native_catalog.rs`
- `tests/a2r_tests.at` with `test_a2r_transpiler` + `test_cookbook_transpiler`
- Removed hardcoded a2r paths from `main.rs`

### Phase 3: VM, A2C, A2TS migration (DONE)

1. **VM file tests** — Added FFI shim `shim_test_run_vm_dir` (ID 2827), `tests/vm_tests.at`, `#[ignore]` on 422 Rust tests
2. **A2C tests** — Added `A2cTestCase` + `discover_a2c_tests()` in `test_runner.rs`, `run_a2c_file_test()` in `lib.rs`, FFI shim (ID 2828), `tests/a2c_tests.at`, `#[ignore]` on 133 Rust tests
3. **A2TS tests** — Added `A2tsTestCase` + `discover_a2ts_tests()` in `test_runner.rs`, `run_a2ts_file_test()` in `lib.rs`, FFI shim (ID 2829), `tests/a2ts_tests.at`, `#[ignore]` on 85 Rust tests
4. **Cleanup** — Removed hardcoded VM test discovery from `main.rs`

## Files Modified

| File | Change |
|------|--------|
| `crates/auto-lang/src/test_runner.rs` | Added `A2cTestCase`, `A2tsTestCase`, `discover_a2c_tests()`, `discover_a2ts_tests()` |
| `crates/auto-lang/src/lib.rs` | Added `run_a2c_file_test()`, `run_a2ts_file_test()` |
| `crates/auto-lang/src/vm/ffi/stdlib.rs` | Added 3 FFI shims: `run_vm_dir`, `run_a2c_dir`, `run_a2ts_dir` |
| `crates/auto-lang/src/vm/native_catalog.rs` | Registered 3 native IDs (2827-2829) |
| `tests/vm_tests.at` | New file |
| `tests/a2c_tests.at` | New file |
| `tests/a2ts_tests.at` | New file |
| `crates/auto-lang/src/tests/vm_file_tests.rs` | `#[ignore]` on all 422 tests |
| `crates/auto-lang/src/tests/a2c_tests.rs` | `#[ignore]` on all 133 tests |
| `crates/auto-lang/src/tests/a2ts_tests.rs` | `#[ignore]` on all 85 tests |
| `crates/auto/src/main.rs` | Removed hardcoded VM + a2r test discovery |
| `docs/plans/263-transpiler-tests.md` | Updated status |

## Verification

```bash
cargo build --bin auto

# All test categories now discovered via tests/*.at files
./target/debug/auto test                          # runs all
./target/debug/auto test --filter test_a2r        # a2r + cookbook
./target/debug/auto test --filter test_vm         # VM file tests
./target/debug/auto test --filter test_a2c        # C transpiler
./target/debug/auto test --filter test_a2ts       # TypeScript transpiler

# Old Rust tests still pass individually
cargo test -p auto-lang -- test_01_basics_001_hello --ignored
```
