# Plan 260: Auto Test Framework (`auto test`)

## Status: COMPLETE

## Context

Auto has no test runner. Users write `.at` files with assertion calls but must execute them manually as scripts. This plan adds a Rust `cargo test`-style test framework: `#[test]` attribute for marking test functions, compile-time discovery, and an `auto test` CLI command that runs all tests and reports results.

The VM already looks for a `test` entry point ([lib.rs:585](crates/auto-lang/src/lib.rs#L585)), and `#[test]` is already recognized as a pass-through annotation for the Rust transpiler ([parser.rs:6265](crates/auto-lang/src/parser.rs#L6265)). The assertion stdlib already exists in [stdlib/auto/test.at](stdlib/auto/test.at).

---

## Phase 1: `#[test]` Attribute & Test Discovery

### Step 1 — Refactor annotation parsing to a struct

**File**: [parser.rs:6231-6312](crates/auto-lang/src/parser.rs#L6231)

The current `parse_fn_annotations()` returns a 5-tuple `(bool, bool, bool, bool, Vec<TypeParam>)`. Refactor to a named struct:

```rust
struct FnAnnotations {
    has_c: bool,
    has_vm: bool,
    has_rs: bool,
    has_pub: bool,
    has_test: bool,       // NEW
    with_params: Vec<TypeParam>,
}
```

Add `"test"` as a first-class match arm (remove from the pass-through group at line 6265):
```rust
"test" => has_test = true,
```

Update all 6 call sites to destructure the struct instead of the tuple:
- Line ~3409 (top-level statements)
- Line ~3718 (type block methods)
- Line ~6429 (`fn_decl_stmt`)
- Line ~6622 (set `fn_expr.is_test` — new)
- Line ~7160 (`parse_type_decl` inner annotation check)
- Line ~7435 (`parse_type_body` methods)

### Step 2 — Add `is_test` flag to `Fn` struct

**File**: [fun.rs:27-32](crates/auto-lang/src/ast/fun.rs#L27)

Add `pub is_test: bool` to the `Fn` struct (after `is_mut`). Default to `false` in both constructors (`Fn::new` at line ~88, `Fn::with_ret_name` at line ~114).

### Step 3 — Wire `is_test` in parser

**File**: [parser.rs:6622](crates/auto-lang/src/parser.rs#L6622)

After `fn_expr.is_pub = has_pub;`, add:
```rust
fn_expr.is_test = has_test;
```

---

## Phase 2: Test Registry & Runner

### Step 4 — Create test registry module

**New file**: `crates/auto-lang/src/test_runner.rs`

```rust
pub struct TestInfo {
    pub name: String,           // "test_add"
    pub module_path: String,    // "math" (from module context)
    pub qualified_name: String, // "math::test_add"
}

pub struct TestRegistry {
    pub tests: Vec<TestInfo>,
}

pub enum TestOutcome { Passed, Failed(String) }

pub struct TestReport {
    pub name: String,
    pub outcome: TestOutcome,
    pub duration_ms: u128,
    pub stdout: String,
}

pub struct TestResult {
    pub reports: Vec<TestReport>,
}
```

Also add `fn collect_tests(stmts: &[Stmt]) -> TestRegistry` that walks the AST for `Stmt::Fn(f)` where `f.is_test == true`.

### Step 5 — Add `test_file()` API

**File**: [lib.rs](crates/auto-lang/src/lib.rs)

Add `pub fn test_file(code: &str, path: &str) -> AutoResult<TestResult>` following the same compilation pipeline as `execute_autovm()` (lines 465-634):

1. Parse → comptime → codegen → link (same as normal execution)
2. Call `collect_tests(&ast.stmts)` to get the test registry
3. For each test in registry, look up address in `global_symbols`
4. Create a fresh `AutoTask` per test (lightweight isolation — shared bytecode, independent stack/IP)
5. Run task to completion, capture stdout per-test
6. Record `TestOutcome::Passed` or `TestOutcome::Failed(error_msg)`

### Step 6 — Test output format

Rust-style output:
```
test math::test_add ... ok
test math::test_sub ... ok
test string::test_concat ... FAILED

failures:
    string::test_concat: assertion failed

test result: 2 passed, 1 failed, finished in 0.003s
```

Exit code: 0 if all pass, 1 if any fail.

---

## Phase 3: `auto test` CLI Command

### Step 7 — Add `Test` command to CLI

**File**: [main.rs:224](crates/auto/src/main.rs#L224) (`Commands` enum)

```rust
#[command(about = "Run all #[test] functions in the project", alias = "t")]
Test {
    #[arg(short, long)]
    dir: Option<String>,
    #[arg(short, long)]
    filter: Option<String>,
    #[arg(short = 'v', long)]
    verbose: bool,
},
```

### Step 8 — Implement test command handler

**File**: [main.rs](crates/auto/src/main.rs) (in the `match cli.command` block)

1. Find project root (look for `pac.at`)
2. Discover entry `.at` file (project source)
3. Compile with test mode, call `test_file()`
4. Apply filter (substring match on qualified name)
5. Print Rust-style test output
6. `std::process::exit(1)` if any failures

---

## Files Modified

| File | Change |
|------|--------|
| [crates/auto-lang/src/ast/fun.rs](crates/auto-lang/src/ast/fun.rs) | Add `is_test: bool` to `Fn` struct |
| [crates/auto-lang/src/parser.rs](crates/auto-lang/src/parser.rs) | Refactor `parse_fn_annotations()` to struct, add `has_test`, wire `fn_expr.is_test` |
| [crates/auto-lang/src/test_runner.rs](crates/auto-lang/src/test_runner.rs) | **New file** — `TestRegistry`, `TestResult`, `collect_tests()`, runner logic |
| [crates/auto-lang/src/lib.rs](crates/auto-lang/src/lib.rs) | Add `test_file()` public API |
| [crates/auto-lang/src/lib.rs](crates/auto-lang/src/lib.rs) | Register `mod test_runner` |
| [crates/auto/src/main.rs](crates/auto/src/main.rs) | Add `Test` command to CLI enum + handler |

---

## Future (not in this plan)

- `#[should_panic]` / `#[ignore]` attributes
- Parallel test execution
- `tests/` directory for integration tests
- Test coverage reporting

---

## Verification

1. **Build**: `cargo build` compiles without errors
2. **Existing tests**: `cargo test` — all existing tests still pass (no regressions from parser refactor)
3. **Manual test**: Create `tmp/test_basic.at` with:
   ```auto
   use auto.test: assert_eq

   #[test]
   fn test_addition() {
       assert_eq(2 + 2, 4)
   }

   #[test]
   fn test_failure() {
       assert_eq(1 + 1, 3)
   }
   ```
4. Run `auto test` and verify output shows `test_addition ... ok`, `test_failure ... FAILED`, exit code 1
5. Run `auto test --filter addition` and verify only `test_addition` runs
