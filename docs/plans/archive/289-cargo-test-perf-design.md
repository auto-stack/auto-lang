# Plan 289: Cargo Test Performance Optimization

**Date**: 2026-06-09
**Status**: ✅ Completed
**Scope**: auto-lang crate (4,097 tests, 87% of workspace total)

## Problem

`cargo test` takes several minutes. Both compilation and execution are slow.

### Root Causes

**Compilation**:
- Default `ui-iced` feature pulls in `iced` + `image` (intentional, not changing)
- 4,097 test functions compiled into a single test binary
- No `[profile.test]` optimization

**Execution**:
- 427 VM file tests each do individual file I/O (`read_to_string`)
- 509 transpiler tests (a2c/a2r/a2ts) each read + transpile + compare
- Standard `cargo test` has limited test-level parallelism

## Test Distribution

| Category | Count | Est. time each | Est. total | Location |
|---|---|---|---|---|
| VM file tests | 427 | ~20-50ms | ~15-20s | `tests/vm_file_tests.rs` |
| A2R transpiler | 290 | ~5-10ms | ~3s | `tests/a2r_tests.rs` |
| A2C transpiler | 134 | ~5-10ms | ~1.5s | `tests/a2c_tests.rs` |
| Cookbook VM | 108 | ~20-50ms | ~5s | `tests/cookbook_vm_tests.rs` |
| A2TS | 85 | ~5ms | ~0.5s | `tests/a2ts_tests.rs` |
| UI generation | ~300+ | ~2-10ms | ~3-10s | `ui_gen/*.rs` |
| Inline unit tests | ~2700+ | <1ms | ~5-10s | Various source files |

## Approach: Feature-Gated Test Groups + OnceLock I/O Cache

### Step 0: Fix Compilation Blocker

**Status**: ✅ Already fixed in prior commit. `autodown_tests.rs` compiles and all 69 tests pass.

### Step 1: Feature-Gated Test Groups

Split test modules with Cargo features so developers can run only relevant tests.

**Features**:

| Feature | Gated modules | Tests | When to use |
|---|---|---|---|
| `test-vm-files` | `vm_file_tests`, `cookbook_vm_tests`, `conformance_tests` | ~570 | Modifying VM code |
| `test-trans` | `a2c_tests`, `a2r_tests`, `a2ts_tests` | ~509 | Modifying transpilers |
| _(none)_ | All other unit tests | ~3000 | Daily development |

**Usage**:
```bash
cargo test -p auto-lang                          # Fast: ~3000 tests
cargo test -p auto-lang --features test-vm-files  # VM file tests
cargo test -p auto-lang --features test-trans     # Transpiler tests
cargo test -p auto-lang --features test-vm-files,test-trans  # CI: all ~4097 tests
```

> Note: Avoid `--all-features` as it pulls in `python` (PyO3) which may fail on Python 3.14+.

**Files changed**:
- `crates/auto-lang/Cargo.toml` — add `test-vm-files` and `test-trans` features
- `crates/auto-lang/src/tests.rs` — add `#[cfg(feature = "...")]` to 6 gated modules

### Step 2: Cargo Aliases + nextest Config

Convenient aliases for common test workflows.

**`.cargo/config.toml`**:
```toml
[alias]
t = "nextest run -p auto-lang --lib"
tv = "nextest run -p auto-lang --lib --features test-vm-files"
tt = "nextest run -p auto-lang --lib --features test-trans"
ta = "nextest run -p auto-lang --lib --features test-vm-files,test-trans"
```

Requires: `cargo install cargo-nextest`

### Step 3: VM Test I/O Batching (OnceLock)

Two `OnceLock` caches eliminate redundant file reads:

1. **`VM_TEST_CACHE`**: Two-level directory scan of `test/vm/{category}/{case}/` — preloads all 427 test sources + expected outputs into a `HashMap<String, VmTestData>`. First test call triggers the scan; subsequent tests hit the map.

2. **`AUTO_LIB_CACHE`**: AAVM tests need 12 `auto/lib/*.at` files concatenated. Was read per-test (~100+ calls × 12 reads). Now read once per process.

**Files changed**:
- `crates/auto-lang/src/tests/vm_file_tests.rs` — `VM_TEST_CACHE` + `AUTO_LIB_CACHE` + refactored `test_vm()`, `test_aavm()`, `test_rust_parser()`

## Results

### Compilation (incremental, after initial build)

| Scenario | Time |
|---|---|
| Default features (no test groups) | ~1.7s |
| + test-vm-files | ~0.6s |
| + test-trans | ~0.6s |
| All groups combined (full recompile) | ~31s |

### Execution

| Group | Feature | Active tests | Time |
|---|---|---|---|
| **Group 1** — Daily dev | _(none)_ | ~688 | ~2m10s |
| **Group 2a** — Cookbook | `test-vm-files` | 64 passed (30 ignored) | ~3.6s |
| **Group 2b** — VM files (sample) | `test-vm-files --ignored` | 37/427 tested | ~1.2s |
| **Group 3** — Transpilers | `test-trans` | 271 (238 ignored) | ~1m30s |

### Key Improvement

Daily development now runs **only Group 1** (~2 min) instead of waiting for the full 4-minute+ suite. VM and transpiler tests are opt-in.

## Files Modified

1. `crates/auto-lang/Cargo.toml` — add `test-vm-files`, `test-trans` features
2. `crates/auto-lang/src/tests.rs` — cfg gates on 6 modules
3. `crates/auto-lang/src/tests/vm_file_tests.rs` — OnceLock caches + refactored test runners
4. `.cargo/config.toml` — Cargo aliases (new file)
