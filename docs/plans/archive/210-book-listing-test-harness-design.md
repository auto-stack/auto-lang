# Plan 210: Book Listing Test Harness — Implementation Plan

## Status: ✅ COMPLETE (in ../book/ repo)

Verified 2026-04-23: Fully implemented in the `book` sibling repository.
- ✅ `book/build.rs` auto-discovers listings/ directories across all books
- ✅ Supports 5 test types: a2r, a2c, a2p, a2ts, vm (based on .expected.* files)
- ✅ 561 `main.at` listing files, 982 `.expected.*` output files
- ✅ Covers `rust/`, `little-c/`, `byte-of-python/` books
- ✅ Commit: 152c588 "feat: add listing test harness with auto-discovery (Plan 210)"

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create an auto-discovery test harness in the `book/` repo that transpiles and runs all 1136 code listings against their expected outputs.

**Architecture:** A minimal Rust crate in `book/` depends on `auto-lang` as a path dependency. A `build.rs` script discovers all listing directories at compile time and generates `#[test]` functions. The test runner functions in `harness.rs` handle transpilation (a2r/a2p/a2c/a2ts) and VM execution, comparing against `.expected.*` files.

**Tech Stack:** Rust, `auto-lang` crate, `build.rs` code generation, standard `#[test]` framework

---

## Task 1: Create `book/Cargo.toml`

**Files:**
- Create: `../book/Cargo.toml`

**Step 1: Write `Cargo.toml`**

```toml
[package]
name = "book-listing-tests"
version = "0.1.0"
edition = "2021"

[dependencies]
auto-lang = { path = "../auto-lang/crates/auto-lang" }

[[test]]
name = "listings"
harness = true
```

**Step 2: Verify it resolves**

Run: `cd ../book && cargo metadata --format-version 1 > /dev/null 2>&1 && echo OK || echo FAIL`

Expected: `OK` (dependency resolves)

**Step 3: Commit**

```bash
cd ../book
git add Cargo.toml
git commit -m "chore: add Cargo.toml for listing test harness"
```

---

## Task 2: Create `book/tests/harness.rs` — test runner functions

This file provides the runner functions that `build.rs`-generated tests will call.

**Files:**
- Create: `../book/tests/harness.rs`

**Step 1: Write the harness**

```rust
use auto_lang::{
    run, run_with_capture,
    trans::rust::transpile_rust,
    trans::c::transpile_c,
    trans::{Sink, Trans},
    trans::python::PythonTrans,
    trans::typescript::TypeScriptTrans,
    Parser,
};
use std::fs;
use std::path::Path;

/// Run a2r (Auto→Rust) test for a listing directory.
pub fn run_a2r(listing_dir: &Path) -> Result<(), String> {
    let at_path = listing_dir.join("main.at");
    let exp_path = listing_dir.join("main.expected.rs");

    let src = fs::read_to_string(&at_path)
        .map_err(|e| format!("read {}: {}", at_path.display(), e))?;
    let expected = fs::read_to_string(&exp_path)
        .map_err(|e| format!("read {}: {}", exp_path.display(), e))?;

    let mut sink = transpile_rust("main", &src)
        .map_err(|e| format!("transpile: {}", e))?;
    let actual = String::from_utf8_lossy(sink.done().map_err(|e| format!("done: {}", e))?).to_string();

    if actual != expected {
        let wrong_path = listing_dir.join("main.wrong.rs");
        fs::write(&wrong_path, &actual).ok();
        return Err(format!(
            "a2r mismatch in {}\nSee: {} vs {}",
            listing_dir.display(),
            exp_path.display(),
            wrong_path.display(),
        ));
    }

    // Clean up any stale .wrong.rs
    let wrong_path = listing_dir.join("main.wrong.rs");
    if wrong_path.exists() {
        fs::remove_file(&wrong_path).ok();
    }

    Ok(())
}

/// Run a2p (Auto→Python) test for a listing directory.
pub fn run_a2p(listing_dir: &Path) -> Result<(), String> {
    let at_path = listing_dir.join("main.at");
    let exp_path = listing_dir.join("main.expected.py");

    let src = fs::read_to_string(&at_path)
        .map_err(|e| format!("read {}: {}", at_path.display(), e))?;
    let expected = fs::read_to_string(&exp_path)
        .map_err(|e| format!("read {}: {}", exp_path.display(), e))?;

    let _scope = auto_lang::scope_manager::ScopeManager::new();
    // Note: ScopeManager is not thread-safe (Rc<RefCell<>>). Tests using a2p/a2ts
    // must run sequentially or we need a different approach.
    // For now, build.rs will mark a2p/a2ts tests with #[serial] or similar.

    let mut parser = Parser::from(src.as_str());
    let ast = parser.parse().map_err(|e| format!("parse: {}", e))?;
    let mut sink = Sink::new("main".into());
    let mut trans = PythonTrans::new("main".into());
    trans.trans(ast, &mut sink).map_err(|e| format!("trans: {}", e))?;
    let actual = String::from_utf8_lossy(sink.done().map_err(|e| format!("done: {}", e))?).to_string();

    if actual != expected {
        let wrong_path = listing_dir.join("main.wrong.py");
        fs::write(&wrong_path, &actual).ok();
        return Err(format!(
            "a2p mismatch in {}\nSee: {} vs {}",
            listing_dir.display(),
            exp_path.display(),
            wrong_path.display(),
        ));
    }

    let wrong_path = listing_dir.join("main.wrong.py");
    if wrong_path.exists() {
        fs::remove_file(&wrong_path).ok();
    }

    Ok(())
}

/// Run a2c (Auto→C) test for a listing directory.
pub fn run_a2c(listing_dir: &Path) -> Result<(), String> {
    let at_path = listing_dir.join("main.at");
    let exp_c_path = listing_dir.join("main.expected.c");
    let exp_h_path = listing_dir.join("main.expected.h");

    let src = fs::read_to_string(&at_path)
        .map_err(|e| format!("read {}: {}", at_path.display(), e))?;
    let expected_c = fs::read_to_string(&exp_c_path)
        .map_err(|e| format!("read {}: {}", exp_c_path.display(), e))?;

    let mut sink = transpile_c("main", &src)
        .map_err(|e| format!("transpile: {}", e))?;
    let actual_c = String::from_utf8_lossy(sink.done().map_err(|e| format!("done: {}", e))?).to_string();
    let header_bytes = sink.header.clone();
    let actual_h = String::from_utf8_lossy(&header_bytes).to_string();

    // Compare .c
    if actual_c != expected_c {
        let wrong_path = listing_dir.join("main.wrong.c");
        fs::write(&wrong_path, &actual_c).ok();
        return Err(format!(
            "a2c .c mismatch in {}\nSee: {} vs {}",
            listing_dir.display(),
            exp_c_path.display(),
            wrong_path.display(),
        ));
    }

    // Compare .h (if expected exists)
    if exp_h_path.exists() {
        let expected_h = fs::read_to_string(&exp_h_path)
            .map_err(|e| format!("read {}: {}", exp_h_path.display(), e))?;
        if actual_h != expected_h {
            let wrong_path = listing_dir.join("main.wrong.h");
            fs::write(&wrong_path, &actual_h).ok();
            return Err(format!(
                "a2c .h mismatch in {}\nSee: {} vs {}",
                listing_dir.display(),
                exp_h_path.display(),
                wrong_path.display(),
            ));
        }
    }

    // Clean up stale .wrong files
    for ext in &["main.wrong.c", "main.wrong.h"] {
        let p = listing_dir.join(ext);
        if p.exists() { fs::remove_file(&p).ok(); }
    }

    Ok(())
}

/// Run a2ts (Auto→TypeScript) test for a listing directory.
pub fn run_a2ts(listing_dir: &Path) -> Result<(), String> {
    let at_path = listing_dir.join("main.at");
    let exp_path = listing_dir.join("main.expected.ts");

    let src = fs::read_to_string(&at_path)
        .map_err(|e| format!("read {}: {}", at_path.display(), e))?;
    let expected = fs::read_to_string(&exp_path)
        .map_err(|e| format!("read {}: {}", exp_path.display(), e))?;

    let _scope = auto_lang::scope_manager::ScopeManager::new();

    let mut parser = Parser::from(src.as_str());
    let ast = parser.parse().map_err(|e| format!("parse: {}", e))?;
    let mut sink = Sink::new("main".into());
    let mut trans = TypeScriptTrans::new("main".into());
    trans.trans(ast, &mut sink).map_err(|e| format!("trans: {}", e))?;
    let actual = String::from_utf8_lossy(sink.done().map_err(|e| format!("done: {}", e))?).to_string();

    if actual != expected {
        let wrong_path = listing_dir.join("main.wrong.ts");
        fs::write(&wrong_path, &actual).ok();
        return Err(format!(
            "a2ts mismatch in {}\nSee: {} vs {}",
            listing_dir.display(),
            exp_path.display(),
            wrong_path.display(),
        ));
    }

    let wrong_path = listing_dir.join("main.wrong.ts");
    if wrong_path.exists() {
        fs::remove_file(&wrong_path).ok();
    }

    Ok(())
}

/// Run AutoVM test for a listing directory.
/// Checks for .expected.out (stdout) or .expected.error (runtime error).
pub fn run_vm(listing_dir: &Path) -> Result<(), String> {
    let at_path = listing_dir.join("main.at");
    let err_path = listing_dir.join("main.expected.error");
    let out_path = listing_dir.join("main.expected.out");

    let src = fs::read_to_string(&at_path)
        .map_err(|e| format!("read {}: {}", at_path.display(), e))?;

    // If .expected.error exists, expect runtime failure
    if err_path.exists() {
        let result = run(&src);
        if result.is_ok() {
            return Err(format!(
                "vm expected error but succeeded in {}",
                listing_dir.display(),
            ));
        }
        return Ok(());
    }

    // Otherwise, expect .expected.out to exist
    if !out_path.exists() {
        return Err(format!(
            "vm test has no .expected.out or .expected.error in {}",
            listing_dir.display(),
        ));
    }

    let expected_out = fs::read_to_string(&out_path)
        .map_err(|e| format!("read {}: {}", out_path.display(), e))?;

    let (result, stdout) = run_with_capture(&src)
        .map_err(|e| format!("vm execution failed: {}", e))?;

    // Drop result — we only check stdout for book listings
    let _ = result;

    if stdout != expected_out {
        let wrong_path = listing_dir.join("main.wrong.out");
        fs::write(&wrong_path, &stdout).ok();
        return Err(format!(
            "vm stdout mismatch in {}\nSee: {} vs {}",
            listing_dir.display(),
            out_path.display(),
            wrong_path.display(),
        ));
    }

    let wrong_path = listing_dir.join("main.wrong.out");
    if wrong_path.exists() {
        fs::remove_file(&wrong_path).ok();
    }

    Ok(())
}
```

**Step 2: Verify it compiles**

Run: `cd ../book && cargo test --no-run 2>&1 | tail -5`

Expected: Compilation succeeds (no tests found yet since we haven't written `listings.rs`, but the harness file itself should compile cleanly when included)

Note: This step may fail until Task 3 is done (the test file needs to `include!` the harness). Adjust as needed.

**Step 3: Commit**

```bash
cd ../book
git add tests/harness.rs
git commit -m "feat: add test runner functions for a2r/a2p/a2c/a2ts/vm"
```

---

## Task 3: Create `book/build.rs` — auto-discovery and test generation

**Files:**
- Create: `../book/build.rs`

**Step 1: Write `build.rs`**

The build script walks `listings/`, finds directories with `main.at`, checks for `.expected.*` files, and generates a Rust file with one `#[test]` function per listing-per-test-type.

```rust
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let book_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let listings_dir = Path::new(&book_dir).join("listings");

    if !listings_dir.is_dir() {
        eprintln!("No listings/ directory found, skipping test generation");
        return;
    }

    let mut tests_code = String::new();
    tests_code.push_str("use std::path::Path;\n\n");

    let mut test_count = 0;

    // Collect all listing directories
    let mut listing_dirs: Vec<_> = Vec::new();
    collect_listings(&listings_dir, &mut listing_dirs);

    // Sort for deterministic output
    listing_dirs.sort();

    for listing_dir in &listing_dirs {
        let rel = listing_dir.strip_prefix(&book_dir).unwrap_or(listing_dir);
        let rel_str = rel.to_str().unwrap_or("").replace('\\', "/");

        // Parse book/chapter/listing from path
        // e.g., "listings/rust/ch01/listing-01-01"
        let parts: Vec<&str> = rel_str.split('/').collect();
        if parts.len() < 4 {
            continue;
        }
        let _listings_prefix = parts[0]; // "listings"
        let book = parts[1];             // "rust", "byte-of-python", etc.
        let chapter = parts[2];          // "ch01"
        let listing = parts[3];          // "listing-01-01"

        // Sanitize names for Rust identifiers
        let book_id = book.replace('-', "_").replace(' ', "_");
        let chapter_id = chapter.replace('-', "_");
        let listing_id = listing.replace('-', "_");

        // Check which expected files exist
        let has_a2r = listing_dir.join("main.expected.rs").exists();
        let has_a2p = listing_dir.join("main.expected.py").exists();
        let has_a2c = listing_dir.join("main.expected.c").exists();
        let has_a2ts = listing_dir.join("main.expected.ts").exists();
        let has_vm_out = listing_dir.join("main.expected.out").exists();
        let has_vm_err = listing_dir.join("main.expected.error").exists();

        // Generate test functions
        if has_a2r {
            tests_code.push_str(&format!(
                "#[test] fn {book_id}_{chapter_id}_{listing_id}_a2r() {{ \
                 super::harness::run_a2r(Path::new(\"{rel_str}\")).unwrap(); \
                 }}\n",
                book_id = book_id,
                chapter_id = chapter_id,
                listing_id = listing_id,
                rel_str = rel_str,
            ));
            test_count += 1;
        }

        if has_a2p {
            tests_code.push_str(&format!(
                "#[test] fn {book_id}_{chapter_id}_{listing_id}_a2p() {{ \
                 super::harness::run_a2p(Path::new(\"{rel_str}\")).unwrap(); \
                 }}\n",
            ));
            test_count += 1;
        }

        if has_a2c {
            tests_code.push_str(&format!(
                "#[test] fn {book_id}_{chapter_id}_{listing_id}_a2c() {{ \
                 super::harness::run_a2c(Path::new(\"{rel_str}\")).unwrap(); \
                 }}\n",
            ));
            test_count += 1;
        }

        if has_a2ts {
            tests_code.push_str(&format!(
                "#[test] fn {book_id}_{chapter_id}_{listing_id}_a2ts() {{ \
                 super::harness::run_a2ts(Path::new(\"{rel_str}\")).unwrap(); \
                 }}\n",
            ));
            test_count += 1;
        }

        if has_vm_out || has_vm_err {
            tests_code.push_str(&format!(
                "#[test] fn {book_id}_{chapter_id}_{listing_id}_vm() {{ \
                 super::harness::run_vm(Path::new(\"{rel_str}\")).unwrap(); \
                 }}\n",
            ));
            test_count += 1;
        }
    }

    // Write generated tests
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");
    fs::write(&dest_path, &tests_code).unwrap();

    println!("cargo:rerun-if-changed=listings/");
    println!("Generated {} listing tests", test_count);
}

fn collect_listings(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if dir.join("main.at").exists() {
        out.push(dir.to_path_buf());
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_listings(&path, out);
        }
    }
}
```

**Step 2: Verify it generates tests**

Run: `cd ../book && cargo test --no-run 2>&1 | tail -10`

Expected: Build script runs and reports "Generated N listing tests" where N > 0

Note: This will likely fail to compile because `tests/listings.rs` doesn't exist yet. That's fine — we just want to see the build script output.

**Step 3: Commit**

```bash
cd ../book
git add build.rs
git commit -m "feat: add build.rs for auto-discovering listing tests"
```

---

## Task 4: Create `book/tests/listings.rs` — test entry point

**Files:**
- Create: `../book/tests/listings.rs`

**Step 1: Write the test entry point**

```rust
mod harness;

// Include auto-generated test functions from build.rs
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
```

**Step 2: Verify it compiles and discovers tests**

Run: `cd ../book && cargo test -- --list 2>&1 | head -30`

Expected: Lists generated test functions like `rust_ch01_listing_01_01_a2r`, etc.

**Step 3: Run a single test to verify end-to-end**

Run: `cd ../book && cargo test -- rust_ch01_listing_01_01_a2r --nocapture 2>&1`

Expected: Test passes (this listing transpiles correctly).

**Step 4: Commit**

```bash
cd ../book
git add tests/listings.rs
git commit -m "feat: add listing test entry point with auto-generated tests"
```

---

## Task 5: Fix compilation issues and run full test suite

**Files:**
- Modify: `../book/tests/harness.rs` (fix any import/path issues)
- Modify: `../book/build.rs` (fix any path generation issues)
- Modify: `../book/Cargo.toml` (add any missing deps)

**Step 1: Run full test suite**

Run: `cd ../book && cargo test 2>&1 | tail -30`

Expected: Most tests pass. Some may fail due to:
- Missing `.expected.*` files
- Transpiler regressions
- Path format issues on Windows

**Step 2: Fix any compilation errors**

Common issues to check:
- `auto_lang::scope_manager::ScopeManager` may not be `pub` — check and fix import
- Path separators on Windows (`\` vs `/`)
- `Parser::from()` may need a specific import path

**Step 3: Re-run and record results**

Run: `cd ../book && cargo test 2>&1 | grep -E "^test |passed|failed"`

Expected: Report of pass/fail counts

**Step 4: Commit fixes**

```bash
cd ../book
git add -u
git commit -m "fix: resolve compilation issues in listing test harness"
```

---

## Task 6: Add `book/.gitignore` entries for test artifacts

**Files:**
- Modify: `../book/.gitignore`

**Step 1: Add entries**

Add to `.gitignore`:
```
# Test artifacts
target/
**/*.wrong.rs
**/*.wrong.py
**/*.wrong.c
**/*.wrong.h
**/*.wrong.ts
**/*.wrong.out
```

**Step 2: Commit**

```bash
cd ../book
git add .gitignore
git commit -m "chore: update .gitignore for test artifacts"
```

---

## Task 7: Smoke test — run and verify test counts

**Step 1: Count generated tests**

Run: `cd ../book && cargo test -- --list 2>&1 | grep -c "test$"`

Expected: A count matching the number of `.expected.*` files found across all listings (should be 100+ at minimum for a2r tests).

**Step 2: Run tests by book**

```bash
cd ../book
cargo test -- rust_ 2>&1 | tail -5
cargo test -- byte_of_python_ 2>&1 | tail -5
cargo test -- vm 2>&1 | tail -5
```

Expected: Each filter runs the relevant subset of tests.

**Step 3: Verify .wrong.* file behavior**

Manually break an `.expected.rs` file, run the test, verify `.wrong.rs` is generated, then restore the expected file.

---

## Task 8: Clean up redundant `book_listing_tests.rs` in auto-lang

**Files:**
- Modify: `crates/auto-lang/src/tests.rs` — remove `book_listing_tests` module
- Delete: `crates/auto-lang/src/tests/book_listing_tests.rs`

**Step 1: Remove the old test module**

In `crates/auto-lang/src/tests.rs`, remove or comment out the `#[cfg(test)] mod book_listing_tests;` line.

**Step 2: Verify auto-lang tests still pass**

Run: `cargo test -p auto-lang 2>&1 | tail -10`

Expected: All existing auto-lang tests pass (excluding the removed book listing tests).

**Step 3: Commit**

```bash
git add -u
git commit -m "chore: remove redundant book_listing_tests (moved to book/ harness)"
```

---

## Important Notes for Implementer

### Import paths (verified against codebase)

```rust
// These are the correct import paths for the harness:
use auto_lang::run;                              // VM execute, returns result
use auto_lang::run_with_capture;                 // VM execute, returns (result, stdout)
use auto_lang::trans::rust::transpile_rust;      // a2r: (name, code) -> Result<Sink>
use auto_lang::trans::c::transpile_c;            // a2c: (name, code) -> Result<Sink>
use auto_lang::trans::python::PythonTrans;       // a2p: construct manually
use auto_lang::trans::typescript::TypeScriptTrans; // a2ts: construct manually
use auto_lang::trans::{Sink, Trans};             // Common types
use auto_lang::Parser;                           // For a2p/a2ts manual construction
```

### ScopeManager note

Python and TypeScript transpilers require a `ScopeManager` to be created before `Parser::from()`. The existing code uses `Rc<RefCell<ScopeManager>>`. This may cause issues with parallel test execution. If tests panic with borrow errors, add `-- --test-threads=1` to force sequential execution, or add `serial_test` crate as a dependency.

### Path handling on Windows

`build.rs` should normalize paths to use `/` separators for the generated code, since Rust string literals with `\` need escaping. The `rel_str` variable already does `.replace('\\', "/")`.

### Expected file counts (current state)

| Book | `.expected.rs` | `.expected.py` | `.expected.c` | `.expected.ts` |
|------|---------------|----------------|---------------|----------------|
| rust | ~175 | 0 | 0 | 0 |
| byte-of-python | 0 | ~57 | 0 | 0 |
| think-python | 0 | ~64 | 0 | 0 |
| little-c | 0 | 0 | ~54 | 0 |
| modern-c | 0 | 0 | ~42 | 0 |
| typescript | 0 | 0 | 0 | ~20 |
| typescript-deepdive | 0 | 0 | 0 | ~17 |
| tapl | varies | varies | varies | varies |

VM `.expected.out` files: **0** initially (will be added gradually).
