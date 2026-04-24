# 191: Add assert builtins and precise linker error spans

## Status: ✅ COMPLETE

Verified 2026-04-23: All four components implemented end-to-end.
- assert/assert_eq/assert_ne intrinsics registered in `vm/native.rs` with IDs 4-6
- `Call.pos: Option<Pos>` field added in `ast/call.rs`, populated by parser
- `RelocEntry.source_pos` carries call-site position from codegen to linker
- `LinkError` struct with `source_pos` for "Undefined symbol" errors in `vm/loader.rs`

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `assert`/`assert_eq`/`assert_ne` as native intrinsics (like `print`), and propagate source positions from AST through codegen into the linker so "Undefined symbol" errors point to the exact call site instead of a heuristic guess.

**Architecture:** Two independent changes: (1) Register assert functions as native intrinsics with IDs 4-6, implement shim functions that print failure message and panic. (2) Add `pos: Option<Pos>` to `Call` struct, propagate into `RelocEntry`, change linker error from `String` to a structured `LinkError` type that carries source position.

**Tech Stack:** Rust (vm/codegen, vm/native, vm/loader, parser, ast)

---

## Background

### Problem 1: Missing assert functions

Auto has no `assert`/`assert_eq`/`assert_ne`. Users expect these as built-in testing primitives. Since they need access to call-site info (file, line) for useful error messages, the ideal solution uses comptime (`#{file}`, `#{line}`). However, comptime evaluation is not yet implemented, so we start with native intrinsics that print the condition/expression on failure.

Future work: When comptime is implemented, `assert` can become a stdlib macro that injects `#{file}:#{line}` automatically.

### Problem 2: Imprecise linker error spans

When a symbol is undefined, `find_use_symbol_span` does heuristic text search for `use` lines. If no `use` line exists (e.g., calling an undefined builtin like `assert`), the fallback span points to the start of the file — which is often a doc comment. The error should point to the actual call site.

Root cause: AST nodes (`Call`, `Use`) don't carry `Pos`. The parser has position info from tokens but discards it when constructing AST nodes. `RelocEntry` has no source position. The linker returns `String` errors with no position info.

---

## Tasks

### Task 1: Add assert/assert_eq/assert_ne as native intrinsics

**Files:**
- Modify: `crates/auto-lang/src/vm/native.rs` (add constants + shim functions)
- Modify: `crates/auto-lang/src/vm/codegen.rs` (register intrinsics + dispatch)

**Step 1: Add NATIVE_ASSERT constants**

In `native.rs`, after `NATIVE_PRINT_STR` (line 284), add:

```rust
pub const NATIVE_ASSERT: u16 = 4;
pub const NATIVE_ASSERT_EQ: u16 = 5;
pub const NATIVE_ASSERT_NE: u16 = 6;
```

**Step 2: Implement shim functions**

In `native.rs`, after `shim_print_str`, add three shim functions:

```rust
pub fn shim_assert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cond = task.ram.pop_i32();
    if cond == 0 {
        return Err(VMError::RuntimeError("Assertion failed".to_string()));
    }
    Ok(())
}

pub fn shim_assert_eq(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let right = task.ram.pop_i32();
    let left = task.ram.pop_i32();
    if left != right {
        return Err(VMError::RuntimeError(
            format!("Assertion failed: {} != {}", left, right)
        ));
    }
    Ok(())
}

pub fn shim_assert_ne(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let right = task.ram.pop_i32();
    let left = task.ram.pop_i32();
    if left == right {
        return Err(VMError::RuntimeError(
            format!("Assertion failed: {} == {}", left, right)
        ));
    }
    Ok(())
}
```

**Step 3: Register shims and intrinsics**

In `register_std_shims()` (line 135), add after print shims:

```rust
self.register(NATIVE_ASSERT, shim_assert);
self.register(NATIVE_ASSERT_EQ, shim_assert_eq);
self.register(NATIVE_ASSERT_NE, shim_assert_ne);
```

In `codegen.rs` `Codegen::new()` (line 184), add after print intrinsics:

```rust
intrinsics.insert("assert".to_string(), NATIVE_ASSERT);
intrinsics.insert("assert_eq".to_string(), NATIVE_ASSERT_EQ);
intrinsics.insert("assert_ne".to_string(), NATIVE_ASSERT_NE);
```

**Step 4: Mark assert functions as void-returning**

In `codegen.rs`, find the void-return check (around line 4052 where `print`/`say` are handled) and add:

```rust
if name.starts_with("assert") { ... }
```

Same pattern as print — set `self.last_expr_type = ObjectType::Void`.

**Step 5: Test**

```bash
echo 'fn main() { assert(1) print("ok") }' > tmp/test_assert.at
auto tmp/test_assert.at
# Expected: ok

echo 'fn main() { assert(0) print("bad") }' > tmp/test_assert_fail.at
auto tmp/test_assert_fail.at
# Expected: Assertion failed error
```

**Step 6: Commit**

### Task 2: Add `pos` field to `Call` struct

**Files:**
- Modify: `crates/auto-lang/src/ast/call.rs` (add pos field)
- Modify: all files constructing `Call { ... }` — add `pos: None`

**Step 1: Add `pos` field to `Call`**

In `call.rs`, add after `type_args`:

```rust
use crate::token::Pos;

#[derive(Debug, Clone)]
pub struct Call {
    pub name: Box<Expr>,
    pub args: Args,
    pub ret: Type,
    pub type_args: Vec<(Name, Type)>,
    /// Source position of the call site (the opening parenthesis)
    pub pos: Option<Pos>,
}
```

**Step 2: Update all `Call { ... }` construction sites**

Search for `Call {` across the codebase. Every occurrence needs `pos: None` added (or `pos: Some(pos)` for parser sites). Use `replace_all` or manual edits. Expected sites:

- `parser.rs` (~5 sites): Set `pos: Some(self.prev.pos)` using the `LParen` token position
- `ui_gen/vue.rs` (~14 sites): `pos: None`
- `ui_gen/rust.rs` (~3 sites): `pos: None`
- `ast/call.rs` tests (~3 sites): `pos: None`
- Other generator/transpiler files: `pos: None`

**Step 3: Run cargo build and fix any remaining sites**

```bash
cargo build -p auto-lang
```

The compiler will error on any missing `pos` field — fix each one with `pos: None`.

**Step 4: Commit**

### Task 3: Propagate source position into RelocEntry and linker errors

**Files:**
- Modify: `crates/auto-lang/src/vm/loader.rs` (add source_pos to RelocEntry, add LinkError type)
- Modify: `crates/auto-lang/src/vm/codegen.rs` (pass call.pos into RelocEntry)
- Modify: `crates/auto-lang/src/lib.rs` (use LinkError instead of heuristic text search)

**Step 1: Add `source_pos` to `RelocEntry`**

In `loader.rs`, update `RelocEntry`:

```rust
use crate::token::Pos;

#[derive(Debug, Clone)]
pub struct RelocEntry {
    pub offset: u32,
    pub symbol_name: String,
    pub reloc_type: RelocType,
    /// Source position of the call site that generated this relocation
    pub source_pos: Option<Pos>,
}
```

**Step 2: Create `LinkError` type**

In `loader.rs`, add before `Linker`:

```rust
#[derive(Debug, Clone)]
pub struct LinkError {
    pub message: String,
    pub symbol: String,
    pub module: String,
    pub source_pos: Option<Pos>,
}
```

Change `Linker::link()` return type from `Result<..., String>` to `Result<..., LinkError>`.

Update the error site (line ~253):

```rust
let target_addr = global_symbols.get(&reloc.symbol_name).ok_or_else(|| {
    LinkError {
        message: format!("Undefined symbol: {} in module {}", reloc.symbol_name, module.name),
        symbol: reloc.symbol_name.clone(),
        module: module.name.clone(),
        source_pos: reloc.source_pos,
    }
})?;
```

**Step 3: Pass call.pos into RelocEntry in codegen**

In `codegen.rs`, update the RelocEntry creation (~line 4214):

```rust
self.relocs.push(RelocEntry {
    offset: placeholder_idx as u32,
    symbol_name: reloc_name.clone(),
    reloc_type: RelocType::FuncCall,
    source_pos: call.pos,
});
```

Also update any other RelocEntry creation sites (search for `RelocEntry {`).

**Step 4: Update linker error consumer in lib.rs**

Replace the `find_use_symbol_span` heuristic with position from `LinkError`:

```rust
let (linked_code, global_symbols) = linker.link().map_err(|e| {
    let span = if let Some(pos) = e.source_pos {
        pos_to_span(pos)
    } else {
        // Fallback: try to find use statement, or point to file start
        find_use_symbol_span(code, &e.message)
    };
    let help = if e.symbol == e.message.strip_prefix("Undefined symbol: ").and_then(|s| s.split(" in ").next()).unwrap_or("") {
        Some(format!("Use a `use` statement to import '{}' from a module, or check for typos", e.symbol))
    } else {
        None
    };
    crate::error::AutoError::MsgWithSource(crate::error::MsgWithSource {
        source: miette::NamedSource::new("<script>", code.to_string()),
        message: e.message.clone(),
        span,
        help: help.or_else(|| extract_undefined_symbol(&e.message).map(|s| format!("Check if '{}' is defined and exported in the module", s))),
    })
})?;
```

Also update any other callers of `linker.link()` (search for `.link()`).

**Step 5: Run cargo build and fix compilation errors**

```bash
cargo build -p auto-lang
```

**Step 6: Commit**

### Task 4: Regression test

**Step 1:** Run `cargo test -p auto-lang --lib`

**Step 2:** Test the original failing case:

```bash
echo '/// Test doc comment
fn main() { assert(1) }' > tmp/test_linker_span.at
auto tmp/test_linker_span.at
# Expected: runs OK, no error

echo 'fn main() { undefined_func() }' > tmp/test_undef.at
auto tmp/test_undef.at
# Expected: error points to line 1, not to a comment
```

**Step 3:** Commit

---

## Priority Order

1. Task 1 (assert intrinsics) — standalone, no dependencies
2. Task 2 (Call pos field) — mechanical, many sites but simple
3. Task 3 (RelocEntry + LinkError) — depends on Task 2
4. Task 4 (regression) — quality gate

## Verification

After all tasks:
1. `assert(1)` runs without error
2. `assert(0)` produces runtime error
3. `assert_eq(1, 1)` passes, `assert_eq(1, 2)` fails with message
4. Undefined symbol error points to the actual call site line
5. `cargo test -p auto-lang --lib` passes with 0 failures
