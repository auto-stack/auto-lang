# Plan 289: Cargo Test Performance Optimization

**Date**: 2026-06-09
**Status**: Approved
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

**Blocking bug**: `autodown_tests.rs` has 5 inline submodules that fail to compile, preventing any tests from running.

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

## Approach: Three-Step Progressive Optimization

### Step 0: Fix Compilation Blocker

Fix `autodown_tests.rs` inline submodules so tests can compile and run.

### Step 1: Feature-Gated Test Groups

Split test modules with Cargo features so developers can run only relevant tests.

**Features**:

| Feature | Gated modules | Tests | When to use |
|---|---|---|---|
| `test-vm-files` | `vm_file_tests`, `cookbook_vm_tests`, `conformance_tests` | ~570 | Modifying VM code |
| `test-trans` | `a2c_tests`, `a2r_tests`, `a2ts_tests` | ~509 | Modifying transpilers |
| (none) | All other unit tests | ~3000 | Daily development |

**Usage**:
```bash
cargo test -p auto-lang                          # Fast: ~3000 tests, < 30s
cargo test -p auto-lang --features test-vm-files  # VM file tests
cargo test -p auto-lang --features test-trans     # Transpiler tests
cargo test -p auto-lang --all-features            # CI: all ~4097 tests
```

**Files changed**:
- `crates/auto-lang/Cargo.toml` — add `test-vm-files` and `test-trans` features
- `crates/auto-lang/src/tests.rs` — add `#[cfg(feature = "...")]` to gated modules

### Step 2: nextest Integration

Install `cargo-nextest` for better parallel test execution.

- Test-level parallelism (not just binary-level)
- Automatic retry of failed tests
- Better progress reporting

**Setup**:
```bash
cargo install cargo-nextest
```

**Cargo alias** (`.cargo/config.toml`):
```toml
[alias]
t = "nextest run -p auto-lang"
tv = "nextest run -p auto-lang --features test-vm-files"
ta = "nextest run -p auto-lang --all-features"
```

### Step 3: VM Test I/O Batching

Use `std::sync::OnceLock` to preload all test data once per process instead of per-test.

```rust
static TEST_CACHE: OnceLock<HashMap<String, TestCase>> = OnceLock::new();

struct TestCase {
    source: String,
    expected_out: Option<String>,
    expected_result: Option<String>,
    expected_error: Option<String>,
}
```

427 individual file reads → 1 directory scan + batch read. Expected savings: 5-10s.

Apply same pattern to `cookbook_vm_tests.rs` (108 tests).

## Expected Results

| Scenario | Before | After |
|---|---|---|
| Daily dev (no features) | Several minutes | < 30s |
| VM tests only | Several minutes | < 15s |
| Full test suite (CI) | Several minutes | < 2min |

## Files to Modify

1. `crates/auto-lang/Cargo.toml` — add features
2. `crates/auto-lang/src/tests.rs` — add cfg gates
3. `crates/auto-lang/src/tests/autodown_tests.rs` — fix compilation
4. `crates/auto-lang/src/tests/vm_file_tests.rs` — add OnceLock cache
5. `crates/auto-lang/src/tests/cookbook_vm_tests.rs` — add OnceLock cache
6. `.cargo/config.toml` — add aliases (new file)
