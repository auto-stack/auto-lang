# Cargo Test Performance Optimization — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Reduce `cargo test -p auto-lang` from "several minutes" to under 30 seconds for daily development, with full test suite under 2 minutes.

**Architecture:** Three progressive steps: (1) Feature-gated test groups so developers run only relevant tests, (2) nextest integration for parallel execution, (3) OnceLock-based I/O batching for VM file tests. A Step 0 fixes the current compilation blocker.

**Tech Stack:** Rust, Cargo features, cargo-nextest, std::sync::OnceLock

---

## Task 0: Fix Compilation Blocker — Missing `mcp/tests.rs`

**Context:** `crates/auto-lang/src/mcp/mod.rs:14` declares `#[cfg(test)] mod tests;` but the file `crates/auto-lang/src/mcp/tests.rs` doesn't exist. This prevents ALL tests from compiling.

**Files:**
- Create: `crates/auto-lang/src/mcp/tests.rs`

**Step 1: Create empty tests module**

Create `crates/auto-lang/src/mcp/tests.rs` with minimal content:

```rust
// Plan 265: MCP module tests (placeholder)
```

**Step 2: Verify compilation succeeds**

Run: `cargo test -p auto-lang --lib --no-run`
Expected: Compiles successfully (may have warnings, no errors)

**Step 3: Commit**

```bash
git add crates/auto-lang/src/mcp/tests.rs
git commit -m "fix(test): add missing mcp/tests.rs to fix compilation blocker"
```

---

## Task 1: Add Feature Flags for Test Grouping

**Context:** Add two Cargo features to gate heavy test modules. Daily dev runs ~3000 fast unit tests; CI runs all.

**Files:**
- Modify: `crates/auto-lang/Cargo.toml` (add features)
- Modify: `crates/auto-lang/src/tests.rs` (add cfg gates)

**Step 1: Add features to Cargo.toml**

In `crates/auto-lang/Cargo.toml`, add these lines after the existing `nanbox = []` line (around line 24):

```toml
# Plan 289: Test grouping features for faster daily development
test-vm-files = []   # Gate VM file tests, cookbook VM tests, conformance tests (~570 tests)
test-trans = []       # Gate a2c/a2r/a2ts transpiler tests (~509 tests)
```

**Step 2: Gate test modules in tests.rs**

In `crates/auto-lang/src/tests.rs`, wrap the heavy test modules with `#[cfg(feature = "...")]`:

Change these lines:

```rust
mod a2c_tests;
```
to:
```rust
#[cfg(feature = "test-trans")]
mod a2c_tests;
```

Change:
```rust
mod a2r_tests;
```
to:
```rust
#[cfg(feature = "test-trans")]
mod a2r_tests;
```

Change:
```rust
mod a2ts_tests;
```
to:
```rust
#[cfg(feature = "test-trans")]
mod a2ts_tests;
```

Change:
```rust
mod vm_file_tests; // Plan 177: VM file-based test framework
```
to:
```rust
#[cfg(feature = "test-vm-files")]
mod vm_file_tests; // Plan 177: VM file-based test framework
```

Change:
```rust
mod cookbook_vm_tests; // Plan 240: Cookbook VM output comparison tests
```
to:
```rust
#[cfg(feature = "test-vm-files")]
mod cookbook_vm_tests; // Plan 240: Cookbook VM output comparison tests
```

Change:
```rust
mod conformance_tests; // Plan 266: AutoVM ↔ a2r semantic conformance tests
```
to:
```rust
#[cfg(feature = "test-vm-files")]
mod conformance_tests; // Plan 266: AutoVM ↔ a2r semantic conformance tests
```

**Step 3: Verify default build compiles and runs fast tests**

Run: `cargo test -p auto-lang --lib --no-run`
Expected: Compiles successfully with fewer modules

Run: `cargo test -p auto-lang --lib 2>&1 | tail -5`
Expected: Runs ~3000 tests (no vm_file, cookbook, a2c, a2r, a2ts, conformance)

**Step 4: Verify all-features still includes everything**

Run: `cargo test -p auto-lang --lib --all-features --no-run`
Expected: Compiles with all modules included

**Step 5: Commit**

```bash
git add crates/auto-lang/Cargo.toml crates/auto-lang/src/tests.rs
git commit -m "feat(test): add test-vm-files and test-trans feature gates for faster daily testing"
```

---

## Task 2: Add Cargo Aliases and nextest Config

**Context:** Create convenient aliases so developers can type `cargo t` instead of the full command. Add nextest config.

**Files:**
- Create: `.cargo/config.toml`
- Create: `.nextest.toml` (nextest configuration)

**Step 1: Create .cargo/config.toml**

Create `.cargo/config.toml`:

```toml
# Plan 289: Test performance aliases
[alias]
# Quick tests (daily dev) — ~3000 fast unit tests
t = "test -p auto-lang --lib"
# VM file tests (when working on VM)
tv = "test -p auto-lang --lib --features test-vm-files"
# Transpiler tests (when working on a2c/a2r/a2ts)
tt = "test -p auto-lang --lib --features test-trans"
# All tests (CI equivalent)
ta = "test -p auto-lang --lib --all-features"

# nextest equivalents (install: cargo install cargo-nextest)
nt = "nextest run -p auto-lang --lib"
ntv = "nextest run -p auto-lang --lib --features test-vm-files"
ntt = "nextest run -p auto-lang --lib --features test-trans"
nta = "nextest run -p auto-lang --lib --all-features"
```

**Step 2: Create .nextest.toml**

Create `.nextest.toml`:

```toml
[profile.default]
# Slow tests get more time
slow-timeout = { period = "30s", terminate-after = 3 }
# Run ignored tests separately (VM file tests are #[ignore])
default-filter = "not test(test_ignored)"

[profile.ci]
# CI gets more retries
failure-output = "immediate-final"
fail-fast = false
```

**Step 3: Verify aliases work**

Run: `cargo t --no-run`
Expected: Compiles and builds test binary for quick test group

**Step 4: Commit**

```bash
git add .cargo/config.toml .nextest.toml
git commit -m "feat(test): add cargo aliases and nextest config for test perf"
```

---

## Task 3: Add OnceLock I/O Cache to vm_file_tests.rs

**Context:** The `test_vm()` function does 2-4 file reads per test (source, expected.out, expected.result, expected.error). With 427 tests, that's ~1000+ file reads. Cache all test data in a `OnceLock<HashMap>` so files are read only once per process.

**Files:**
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: Add cache struct and loader**

At the top of `vm_file_tests.rs`, add the cache infrastructure. The existing `test_vm` function already works correctly — we'll add a cache layer that pre-reads all expected files for a given case.

Add these imports and structs after the existing `use` statements (after line 11):

```rust
use std::collections::HashMap;
use std::sync::OnceLock;

struct VmTestCase {
    source: String,
    expected_out: Option<String>,
    expected_result: Option<String>,
    expected_error: Option<String>,
}

static VM_TEST_CACHE: OnceLock<HashMap<String, VmTestCase>> = OnceLock::new();

fn load_vm_tests() -> HashMap<String, VmTestCase> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cache = HashMap::new();

    // Scan test/vm/ for all test cases
    let vm_dir = d.join("test/vm");
    if let Ok(entries) = std::fs::read_dir(&vm_dir) {
        for category in entries.flatten() {
            if !category.file_type().map_or(false, |t| t.is_dir()) {
                continue;
            }
            let cat_name = category.file_name();
            let cat_str = cat_name.to_string_lossy();
            if let Ok(cases) = std::fs::read_dir(category.path()) {
                for case in cases.flatten() {
                    if !case.file_type().map_or(false, |t| t.is_dir()) {
                        continue;
                    }
                    let case_name = case.file_name();
                    let case_str = case_name.to_string_lossy();
                    let parts: Vec<&str> = case_str.splitn(2, '_').collect();
                    if parts.len() < 2 { continue; }
                    let name = parts[1..].join("_");

                    let src_path = case.path().join(format!("{}.at", name));
                    if !src_path.is_file() { continue; }

                    if let Ok(source) = read_to_string(&src_path) {
                        let expected_out = read_to_string(case.path().join(format!("{}.expected.out", name))).ok();
                        let expected_result = read_to_string(case.path().join(format!("{}.expected.result", name))).ok();
                        let expected_error = read_to_string(case.path().join(format!("{}.expected.error", name))).ok();
                        let key = format!("{}/{}", cat_str, case_str);
                        cache.insert(key, VmTestCase { source, expected_out, expected_result, expected_error });
                    }
                }
            }
        }
    }
    cache
}

fn get_vm_cache() -> &'static HashMap<String, VmTestCase> {
    VM_TEST_CACHE.get_or_init(load_vm_tests)
}
```

**Step 2: Refactor test_vm to use cache**

Replace the existing `test_vm` function with a cached version:

```rust
fn test_vm(case: &str) -> AutoResult<()> {
    let cache = get_vm_cache();
    let tc = cache.get(case)
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Test case '{}' not found in cache", case)
        ))?;

    // Check .expected.error — expect runtime error
    if tc.expected_error.is_some() {
        let result = run(&tc.source);
        assert!(
            result.is_err(),
            "Expected error but got: {:?}",
            result
        );
        return Ok(());
    }

    // Execute with stdout capture
    let (result, stdout) = run_with_capture(&tc.source)?;

    // Check .expected.out — stdout output
    if let Some(ref expected_out) = tc.expected_out {
        if stdout != *expected_out {
            let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let wrong_path = d.join(format!("test/vm/{}/{}.wrong.out", case, case.rsplit('/').next().unwrap_or(case).splitn(2, '_').nth(1).unwrap_or("")));
            std::fs::write(&wrong_path, &stdout)?;
        }
        assert_eq!(stdout, *expected_out);
    }

    // Check .expected.result — return value
    if let Some(ref expected_res) = tc.expected_result {
        if result != *expected_res {
            let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let wrong_path = d.join(format!("test/vm/{}/{}.wrong.result", case, case.rsplit('/').next().unwrap_or(case).splitn(2, '_').nth(1).unwrap_or("")));
            std::fs::write(&wrong_path, &result)?;
        }
        assert_eq!(result, *expected_res);
    }

    Ok(())
}
```

**Step 3: Verify vm_file_tests compile**

Run: `cargo test -p auto-lang --lib --features test-vm-files --no-run`
Expected: Compiles successfully

**Step 4: Run a single VM test to verify correctness**

Run: `cargo test -p auto-lang --lib --features test-vm-files test_01_basics_001_hello -- --ignored`
Expected: Test passes (same behavior as before, just cached)

**Step 5: Commit**

```bash
git add crates/auto-lang/src/tests/vm_file_tests.rs
git commit -m "perf(test): add OnceLock I/O cache for VM file tests"
```

---

## Task 4: Add OnceLock I/O Cache to cookbook_vm_tests.rs

**Context:** Same pattern as Task 3, but for cookbook tests (108 tests). These also do file I/O per test.

**Files:**
- Modify: `crates/auto-lang/src/tests/cookbook_vm_tests.rs`

**Step 1: Add cache to cookbook_vm_tests**

Add cache infrastructure after the existing `use` statements (after line 9):

```rust
use std::collections::HashMap;
use std::sync::OnceLock;

struct CookbookTestCase {
    source: String,
    expected_out: Option<String>,
    case_dir: PathBuf,
}

static COOKBOOK_CACHE: OnceLock<HashMap<String, CookbookTestCase>> = OnceLock::new();

fn load_cookbook_tests() -> HashMap<String, CookbookTestCase> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cache = HashMap::new();
    let cookbook_dir = d.join(COOKBOOK_DIR);

    if let Ok(categories) = std::fs::read_dir(&cookbook_dir) {
        for cat in categories.flatten() {
            if !cat.file_type().map_or(false, |t| t.is_dir()) { continue; }
            let cat_name = cat.file_name().to_string_lossy().to_string();
            if let Ok(cases) = std::fs::read_dir(cat.path()) {
                for case in cases.flatten() {
                    if !case.file_type().map_or(false, |t| t.is_dir()) { continue; }
                    let case_name = case.file_name().to_string_lossy().to_string();

                    // Find .at file
                    let at_file = std::fs::read_dir(case.path())
                        .ok()
                        .and_then(|entries| entries
                            .filter_map(|e| e.ok())
                            .find(|e| e.path().extension().map_or(false, |ext| ext == "at")));

                    if let Some(at) = at_file {
                        if let Ok(source) = std::fs::read_to_string(at.path()) {
                            let key = format!("{}/{}", cat_name, case_name);
                            let expected_out = std::fs::read_to_string(case.path().join("expected.out")).ok();
                            cache.insert(key, CookbookTestCase {
                                source,
                                expected_out,
                                case_dir: case.path(),
                            });
                        }
                    }
                }
            }
        }
    }
    cache
}

fn get_cookbook_cache() -> &'static HashMap<String, CookbookTestCase> {
    COOKBOOK_CACHE.get_or_init(load_cookbook_tests)
}
```

**Step 2: Refactor test_cookbook to use cache**

Replace the existing `test_cookbook` function:

```rust
fn test_cookbook(category: &str, name: &str) -> AutoResult<()> {
    let key = format!("{}/{}", category, name);
    let cache = get_cookbook_cache();
    let tc = cache.get(&key)
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Cookbook test '{}' not found in cache", key)
        ))?;

    let original_cwd = std::env::current_dir()?;
    std::env::set_current_dir(&tc.case_dir)
        .map_err(|e| std::io::Error::new(e.kind(), format!("Cannot cd to {:?}: {}", tc.case_dir, e)))?;

    let result = run_with_capture(&tc.source);
    let (_vm_result, stdout) = match result {
        Ok(v) => v,
        Err(e) => {
            std::env::set_current_dir(&original_cwd)?;
            return Err(e);
        }
    };

    std::env::set_current_dir(&original_cwd)?;

    if let Some(ref expected) = tc.expected_out {
        if stdout != *expected {
            let wrong_path = tc.case_dir.join("wrong.out");
            std::fs::write(&wrong_path, &stdout)?;
        }
        assert_eq!(stdout, *expected, "Output mismatch for {}", key);
    }

    Ok(())
}
```

**Step 3: Verify cookbook tests compile and run**

Run: `cargo test -p auto-lang --lib --features test-vm-files cb_algorithms_sort_int -- --nocapture`
Expected: Test passes

**Step 4: Commit**

```bash
git add crates/auto-lang/src/tests/cookbook_vm_tests.rs
git commit -m "perf(test): add OnceLock I/O cache for cookbook VM tests"
```

---

## Task 5: Measure and Document Results

**Context:** Time the tests before and after to quantify improvement. Update the design doc with results.

**Files:**
- Modify: `docs/plans/289-cargo-test-perf-design.md` (add results section)

**Step 1: Time default (fast) test group**

Run: `time cargo test -p auto-lang --lib 2>&1 | tail -10`
Record: Total time and test count

**Step 2: Time VM tests group**

Run: `time cargo test -p auto-lang --lib --features test-vm-files -- --ignored 2>&1 | tail -10`
Record: Total time

**Step 3: Time full suite**

Run: `time cargo test -p auto-lang --lib --all-features 2>&1 | tail -10`
Record: Total time

**Step 4: Update design doc with actual numbers**

Add a "Results" section to `docs/plans/289-cargo-test-perf-design.md`:

```markdown
## Results

| Scenario | Before | After | Improvement |
|---|---|---|---|
| Default (fast) | ? min | ? s | ?x |
| VM tests | ? min | ? s | ?x |
| Full suite | ? min | ? s | ?x |
```

**Step 5: Final commit**

```bash
git add docs/plans/289-cargo-test-perf-design.md
git commit -m "docs: add test perf optimization results to plan 289"
```

---

## Summary

| Task | What | Effort | Impact |
|---|---|---|---|
| 0 | Fix mcp/tests.rs missing file | 1 min | Unblocks all testing |
| 1 | Feature-gated test groups | 5 min | Daily dev < 30s |
| 2 | Cargo aliases + nextest config | 3 min | Developer convenience |
| 3 | OnceLock cache for vm_file_tests | 10 min | VM group 5-10s faster |
| 4 | OnceLock cache for cookbook_vm_tests | 5 min | Cookbook group faster |
| 5 | Measure and document | 5 min | Quantify results |
