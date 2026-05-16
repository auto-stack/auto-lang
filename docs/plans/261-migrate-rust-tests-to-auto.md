# Plan 261: Migrate Rust AutoVM Tests to Auto `#[test]`

## Context

The Auto language has ~150 Rust inline tests that run Auto code through the VM and assert on results. With Plan 260's `auto test` framework now working, these tests can be expressed directly in Auto as `#[test]` functions — eliminating the Rust wrapper boilerplate and letting tests live in the language they test.

The file-based VM tests (426 in `test/vm/`, 163 in `test/cookbook/`) use a different paradigm (print output → compare against expected file) and are NOT in scope for this migration. They remain as-is.

## What's In Scope

**Rust inline tests** — tests in `crates/auto-lang/src/tests/*.rs` that call `run("auto code")` and assert on the result string. These are the easiest to migrate because each test already contains the Auto code as a string literal.

### Test files to migrate (in order):

| File | Test Count | Status |
|------|-----------|--------|
| `tests/dstr_tests.rs` | 9 | All active, straightforward |
| `tests/infer_tests.rs` | 8 | All active, straightforward |
| `tests/list_tests.rs` | 4 active + 11 ignored | Migrate active only |
| `tests/field_access_tests.rs` | 5 | All active |
| `tests/memory_tests.rs` | 6 | All active |
| `tests/list_growth_tests.rs` | 2 | All active |
| `tests/mem_tests.rs` | 6 | All active |
| `tests/stdlib_tests.rs` | ~10 active + ~15 ignored | Migrate active only |
| `tests/may_tests.rs` | 6 | All active |

**Total: ~56 active tests to migrate**

### NOT migrated (stay as Rust tests):
- `tests/vm_tests.rs` — direct bytecode tests, AST inspection, config mode tests
- `tests/string_tests.rs` — tests Rust FFI shims directly, not Auto code
- `tests/pointer_tests.rs` — parser-level tests
- `tests/const_generic_tests.rs` — parser/AST tests
- `tests/storage_tests.rs` — parser/AST tests
- `tests/ownership_tests.rs` — parser tests (view keyword not implemented)
- All ignored tests (features not yet implemented)
- File-based tests (`test/vm/`, `test/cookbook/`)
- Transpiler tests (`test/a2r/`, etc.)

## Approach

### 1. Create Auto test files in `tests/` directory

New directory: `tests/auto/` at the project root (alongside existing test dirs). Each Rust test file maps to one Auto test file:

```
tests/auto/
├── dstr.at          ← tests/dstr_tests.rs
├── infer.at         ← tests/infer_tests.rs
├── list.at          ← tests/list_tests.rs (active only)
├── field_access.at  ← tests/field_access_tests.rs
├── memory.at        ← tests/memory_tests.rs
├── list_growth.at   ← tests/list_growth_tests.rs
├── mem.at           ← tests/mem_tests.rs
├── stdlib.at        ← tests/stdlib_tests.rs (active only)
├── may.at           ← tests/may_tests.rs
```

### 2. Migration pattern

Each Rust test:
```rust
#[test]
fn test_string_new() {
    let code = r#"
        let s = String.new()
        s.len()
    "#;
    let result = run(code).unwrap();
    assert_eq!(result, "0");
}
```

Becomes an Auto test:
```auto
#[test]
fn test_string_new() {
    let s = String.new()
    assert_eq(s.len(), 0)
}
```

### 3. Run with `auto test`

```bash
auto test --dir tests/auto/dstr.at
auto test --dir tests/auto/           # (future: run all files in dir)
```

## Implementation Steps

### Step 1 — Create test directory and first test file
Create `tests/auto/` directory and migrate `dstr_tests.rs` (9 tests, simplest pattern).

### Step 2 — Migrate remaining test files
Migrate in order: `infer`, `list` (active), `field_access`, `memory`, `list_growth`, `mem`, `stdlib` (active), `may`.

### Step 3 — Verify all migrated tests pass
Run each `.at` test file with `auto test` and confirm all pass.

### Step 4 — Add Rust test runner that delegates to `auto test`
Optionally add a Rust test in `tests/` that calls `auto_lang::test_file()` for each Auto test file, so `cargo test` still discovers and runs them.

## Verification

1. `auto test --dir tests/auto/dstr.at` — 9 tests pass
2. `auto test --dir tests/auto/infer.at` — 8 tests pass
3. `cargo test -p auto-lang --lib` — existing Rust tests still pass
4. No regressions from the migration
