# Plan 220: A2R Transpiler Improvement

> **Status: ✅ COMPLETE** — Rust transpiler at trans/rust.rs (4848 lines), 80 test directories, 17 categories, extensive stdlib mappings
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

Date: 2026-04-23

## Problem

The a2r (Auto-to-Rust) transpiler fails on 30 of 33 ac-examples. The transpiler
has working code for emitting structs, enums, traits, and impls, but the CLI's
incremental transpilation path (`trans_incremental`) only emits function
fragments, silently dropping all type declarations.

## Root Cause

`trans_incremental()` (rust.rs:4159) calls `fn_decl()` per dirty fragment but
never calls `type_decl()`, `enum_decl()`, `spec_decl()`, or `ext_decl()`. The
full `trans()` method (rust.rs:4205) has the correct pipeline but is only used
by the deprecated legacy path.

## Design

### Phase 1: Fix Incremental Pipeline

Modify `trans_incremental()` to replicate the `trans()` pipeline structure:

1. Emit file header + a2r stdlib imports (`emit_a2r_stdlib`)
2. Collect all declarations from the Database (TypeDecl, EnumDecl, SpecDecl,
   Ext, TypeAlias, Tag) — not just function fragments
3. Emit all declarations via `stmt()` dispatch
4. Emit dirty function fragments as standalone functions
5. Emit `fn main()` wrapper for top-level statements

Files: `trans/rust.rs` (trans_incremental), `lib.rs` (trans_rust_with_session)

Expected impact: ~25 additional examples compile.

### Phase 2: Fix Code Generation Bugs

Targeted fixes in `trans/rust.rs`:

**2a. Two-argument `.slice(start, end)`**
Current: emits `&s[start..]` (drops end). Fix: emit `&s[start..end]` when two
args present. Affects: 14, 15, 16.

**2b. Integer index `as usize` cast**
Insert `as usize` when i32/u32 expressions are used as slice/array indices.
Affects: 10+ examples.

**2c. Char vs string literal comparison**
`ch == "\n"` should emit `ch == '\n'` when LHS is char. Affects: 06, 15.

**2d. `&str` concatenation with `+`**
When both sides are `&str`, emit `format!("{}{}", a, b)` or convert to String.
Affects: 06.

Expected impact: ~3 more examples produce correct output.

### Phase 3: Add JSON Runtime Shim

Add a `Json` module to `a2r_std.rs` wrapping `serde_json`:

```rust
mod Json {
    pub fn is_valid(s: &str) -> bool
    pub fn get_at(val: &Value, idx: usize) -> Option<&Value>
    pub fn get(val: &Value, key: &str) -> Option<&Value>
    pub fn as_string(val: &Value) -> Option<&str>
}
```

The transpiler emits `use auto_lang::a2r_std::Json;` when Json is referenced.

Affects: 11, 16, 21, 22, 23, 27.

Expected impact: remaining JSON-heavy examples work.

## Scope Exclusions (YAGNI)

- No serde_json::json! macro transpilation
- No HashMap transpiler support
- No dyn Trait support
- No Auto language syntax or parser changes

## Expected Outcome

| Phase | Examples Fixed | Cumulative |
|-------|---------------|------------|
| Before | 3/33 | 3/33 |
| Phase 1 | ~25 | ~28/33 |
| Phase 2 | ~3 | ~31/33 |
| Phase 3 | ~2 | ~33/33 |

---

## Implementation Tasks

**Goal:** Fix the a2r transpiler to produce compilable Rust for all 33 ac-examples.

**Architecture:** The transpiler already has struct/enum/trait/impl emission code in its `trans()` method. The fix is to make `trans_rust_with_session()` use the full pipeline instead of only emitting function fragments via `trans_incremental()`. Then fix targeted code-generation bugs and add a JSON runtime shim.

**Tech Stack:** Rust, Auto language transpiler in `auto-lang` crate

---

### Task 1: Replace `trans_incremental()` call with full `trans()` in `trans_rust_with_session()`

**Files:**
- Modify: `d:/autostack/auto-lang/crates/auto-lang/src/lib.rs:1257-1303` (`trans_rust_with_session`)
- Modify: `d:/autostack/auto-lang/crates/auto-lang/src/trans/rust.rs:4159-4201` (`trans_incremental`)

**Context:** `trans_rust_with_session()` reads the source, compiles it into the Database via `compile_source()`, then calls `trans_incremental()` which only emits function fragments. We need it to use the full `trans()` pipeline instead.

**Step 1: Modify `trans_rust_with_session()` to use the full transpiler pipeline**

In `lib.rs:1257-1303`, replace the call to `trans_incremental()` with a call to `trans()` using the already-parsed AST. The function should:

1. Keep the `compile_source()` call (for Database population / incremental tracking)
2. Re-parse the source with `CompileDest::TransRust` set
3. Run CTEE transformation (as the legacy path does)
4. Call `RustTrans::trans()` on the AST
5. Write the full Sink output to the `.a2r.rs` file

The new body of `trans_rust_with_session` should be:

```rust
pub fn trans_rust_with_session(session: &mut CompileSession, path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)?;

    // Keep incremental tracking
    let _frag_ids = session.compile_source(&code, path)?;

    // Parse for transpilation (with TransRust dest)
    let _scope = shared(crate::scope_manager::ScopeManager::new());
    let mut parser = Parser::from(code.as_str());
    parser.set_dest(crate::parser::CompileDest::TransRust);
    let mut ast = parser.parse().map_err(|e| e.to_string())?;

    // Run CTEE
    let mut ctee = crate::comptime::CTEE::new();
    ctee.transform(&mut ast).map_err(|e| e.to_string())?;

    // Full transpilation
    let fname = AutoPath::new(path).filename();
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::rust::RustTrans::new(fname);
    trans.trans(ast, &mut sink)?;

    let output = String::from_utf8(sink.done()?.to_vec())
        .map_err(|e| format!("Invalid UTF-8: {}", e))?;

    // Write output
    let rsname = path.replace(".at", ".a2r.rs");
    if !output.is_empty() {
        std::fs::write(&rsname, &output)?;
        validate_rust_output(&rsname, &output);
    }

    // Report using dirty count from session
    let db = session.db();
    let file_id = db.read().unwrap().get_file_id_by_path(path);
    let dirty_count = if let Some(fid) = file_id {
        let db_read = db.read().unwrap();
        let all_frags = db_read.get_fragments_by_file(fid);
        all_frags.len()
    } else {
        0
    };

    Ok(format!(
        "[trans] {} -> {} ({} fragments)",
        path, rsname, dirty_count
    ))
}
```

Note: `shared()` is `crate::scope_manager::shared()` — check the legacy path at lib.rs:1136-1158 for the exact imports/patterns used.

**Step 2: Rebuild and test**

Run: `cd /d/autostack/auto-lang && cargo build --release -p auto 2>&1 | tail -20`
Expected: Compiles successfully

**Step 3: Re-transpile all 33 examples and verify type declarations now appear**

```bash
cd /d/autostack/auto-code-rs/crates/ac-examples/src
for dir in */; do
  if [ -f "$dir/main.at" ]; then
    cd "$dir"
    /d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust 2>&1
    cd ..
  fi
done

# Check that struct/enum definitions now appear
grep -l "^struct\|^enum\|^trait\|^impl" */main.a2r.rs | wc -l
```

Expected: 20+ files now contain type definitions (previously 0).

**Step 4: Spot-check example 08 (Usage struct)**

```bash
head -20 /d/autostack/auto-code-rs/crates/ac-examples/src/08_usage_struct/main.a2r.rs
```

Expected: Should now include `struct Usage { ... }` and `impl Usage { ... }` before `fn main()`.

**Step 5: Commit**

```bash
cd /d/autostack/auto-lang
git add crates/auto-lang/src/lib.rs
git commit -m "fix(a2r): use full trans() pipeline in trans_rust_with_session() to emit type declarations"
```

---

### Task 2: Handle non-function top-level statements in the main wrapper

**Files:**
- Modify: `d:/autostack/auto-lang/crates/auto-lang/src/trans/rust.rs:4213-4304` (the `trans()` method's Phase 2-4)

**Context:** The `trans()` method's Phase 2 splits statements into `decls` (is_decl() == true) and `main` (everything else). The `is_decl()` method (ast.rs:217-230) classifies `Stmt::Store` as a declaration, which means `let` bindings at the top level of the file go into `decls` and get emitted BEFORE `fn main()` — not inside it. But in the examples, top-level `let` bindings should be inside `main()`.

Check what `is_decl()` returns for `Stmt::Store`:

```rust
// ast.rs:217-230
pub fn is_decl(&self) -> bool {
    match self {
        Stmt::Fn(_) => true,
        Stmt::TypeDecl(_) => true,
        Stmt::EnumDecl(_) => true,
        Stmt::Store(store) => { /* check store kind */ }
        ...
    }
}
```

**Step 1: Verify how Store is handled in `trans()`**

Read `trans/rust.rs:4217-4239`. If `Stmt::Store` with `StoreKind::Var` goes to `decls`, it will be emitted as a global variable, not inside `fn main()`. Check if this is the case and whether the examples use top-level `let` or `var`.

**Step 2: Test transpilation of example 01 (simplest)**

```bash
cd /d/autostack/auto-code-rs/crates/ac-examples/src/01_djb2_hash
/d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust
cat main.a2r.rs
```

Verify the output is valid Rust with `fn main()` containing the assertions.

**Step 3: Run through all 33 examples and count how many produce compilable output**

```bash
cd /d/autostack/auto-code-rs/crates/ac-examples/src
for dir in */; do
  cd "$dir"
  /d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust 2>&1 | grep -o "transpiled.*"
  cd ..
done
```

Record which examples now work vs still fail.

**Step 4: Commit**

```bash
cd /d/autostack/auto-lang
git add -A
git commit -m "fix(a2r): ensure top-level let bindings go into fn main() wrapper"
```

---

### Task 3: Fix two-argument `.slice(start, end)` to emit `&s[start..end]`

**Files:**
- Modify: `d:/autostack/auto-lang/crates/auto-lang/src/trans/rust.rs:1669-1678`

**Context:** The `slice` handler at line 1669 only uses the first argument:
```rust
"slice" => {
    write!(out, "&")?;
    self.expr(lhs, out)?;
    write!(out, "[")?;
    if let Some(Arg::Pos(a)) = call.args.args.first() {
        self.expr(a, out)?;
    }
    write!(out, "..]")?;  // BUG: ignores second arg
    return Ok(());
}
```

**Step 1: Write the fix**

Replace lines 1669-1678 with:

```rust
"slice" => {
    // s.slice(n) -> &s[n..]
    // s.slice(start, end) -> &s[start..end]
    write!(out, "&")?;
    self.expr(lhs, out)?;
    write!(out, "[")?;
    let args = &call.args.args;
    if let Some(Arg::Pos(a)) = args.first() {
        self.expr(a, out)?;
    }
    if args.len() >= 2 {
        if let Some(Arg::Pos(b)) = args.get(1) {
            write!(out, "..")?;
            self.expr(b, out)?;
        }
        write!(out, "]")?;
    } else {
        write!(out, "..]")?;
    }
    return Ok(());
}
```

Also check if the same bug exists in the `Expr::Dot` path around line 1811 (search for `"slice"`). Fix it there too if present.

**Step 2: Test with example 14**

```bash
cd /d/autostack/auto-code-rs/crates/ac-examples/src/14_sse_frame_extract
/d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust
grep "slice\|\[0\.\." main.a2r.rs
```

Expected: Should show `&buffer[0..pos]` instead of `&buffer[0..]`.

**Step 3: Rebuild and re-test all examples**

```bash
cd /d/autostack/auto-lang && cargo build --release -p auto 2>&1 | tail -5
cd /d/autostack/auto-code-rs/crates/ac-examples/src
for dir in */; do
  [ -f "$dir/main.at" ] && cd "$dir" && /d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust 2>/dev/null && cd .. || cd ..
done
```

**Step 4: Commit**

```bash
cd /d/autostack/auto-lang
git add crates/auto-lang/src/trans/rust.rs
git commit -m "fix(a2r): emit correct &s[start..end] for two-argument .slice()"
```

---

### Task 4: Add `as usize` cast for integer slice/array indices

**Files:**
- Modify: `d:/autostack/auto-lang/crates/auto-lang/src/trans/rust.rs:703-708` (`Expr::Index` handling)

**Context:** `Expr::Index` emits the index expression as-is:
```rust
Expr::Index(arr, idx) => {
    self.expr(arr, out)?;
    write!(out, "[")?;
    self.expr(idx, out)?;
    write!(out, "]").map_err(Into::into)
}
```

When `idx` is `Expr::Int` (i32) or `Expr::Uint` (u32), Rust requires `usize`. This also affects the `.slice()` and `.sub()` methods which emit slice syntax.

**Step 1: Add a helper method to detect if an expression needs usize cast**

Add near the other helper methods:

```rust
/// Check if expression is an integer type that needs `as usize` for indexing
fn needs_usize_cast(expr: &Expr) -> bool {
    matches!(expr,
        Expr::Int(_) | Expr::Uint(_) | Expr::I8(_) | Expr::U8(_)
        | Expr::I64(_) | Expr::U64(_)
    )
}
```

**Step 2: Use it in `Expr::Index`**

```rust
Expr::Index(arr, idx) => {
    self.expr(arr, out)?;
    write!(out, "[")?;
    if Self::needs_usize_cast(idx) {
        write!(out, "(")?;
        self.expr(idx, out)?;
        write!(out, ") as usize")?;
    } else {
        self.expr(idx, out)?;
    }
    write!(out, "]").map_err(Into::into)
}
```

**Step 3: Also add casts in `.slice()` and `.sub()` methods**

In the `.slice()` handler (from Task 3), wrap index args:
```rust
if Self::needs_usize_cast(a) {
    write!(out, "(")?;
    self.expr(a, out)?;
    write!(out, ") as usize")?;
} else {
    self.expr(a, out)?;
}
```

Do the same for `.sub()` at line ~1652.

**Step 4: Rebuild and test**

```bash
cd /d/autostack/auto-lang && cargo build --release -p auto
cd /d/autostack/auto-code-rs/crates/ac-examples/src
for dir in */; do
  [ -f "$dir/main.at" ] && cd "$dir" && /d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust 2>/dev/null && cd .. || cd ..
done
# Check examples that use indexing
grep "as usize" 14_sse_frame_extract/main.a2r.rs 06_line_formatter/main.a2r.rs
```

**Step 5: Commit**

```bash
cd /d/autostack/auto-lang
git add crates/auto-lang/src/trans/rust.rs
git commit -m "fix(a2r): insert `as usize` casts for integer slice/array indices"
```

---

### Task 5: Fix char vs string literal comparison

**Files:**
- Modify: `d:/autostack/auto-lang/crates/auto-lang/src/trans/rust.rs` (binary comparison handling)

**Context:** When `ch == "\n"` appears in Auto code, `ch` is a char (from `.char_at()` or `.chars().nth()`) but `"\n"` is transpiled as a `&str` literal. In Rust, `char != &str`. Need to detect single-char string comparisons and emit char literals.

**Step 1: Find where binary equality comparison is emitted**

Search for `Op::Eq` or `"=="` in the `Bina` expression handler. The comparison is likely around lines 595-644.

**Step 2: Add logic to detect single-char string in comparisons**

When emitting `==` or `!=` with a string literal of length 1 (after escape processing), emit it as a char literal instead:

```rust
// In the Bina handler for Op::Eq / Op::Ne:
// If RHS is a string literal of length 1, emit as char
if let Expr::Str(s) = rhs.as_ref() {
    let decoded = unescape(s);
    if decoded.len() == 1 && is_char_type(lhs) {
        write!(out, "'{}'", escape_char(decoded.chars().next().unwrap()))?;
        return Ok(());
    }
}
```

This requires checking if LHS is char-typed. A simpler heuristic: if the LHS is a `.char_at()` or `.chars().nth()` call, treat the comparison as char.

**Step 3: Test with example 06**

```bash
cd /d/autostack/auto-lang && cargo build --release -p auto
cd /d/autostack/auto-code-rs/crates/ac-examples/src/06_line_formatter
/d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust
grep "== '\\\\n'" main.a2r.rs || grep "== \"\\\\n\"" main.a2r.rs
```

Expected: Should show `== '\n'` instead of `== "\n"`.

**Step 4: Commit**

```bash
cd /d/autostack/auto-lang
git add crates/auto-lang/src/trans/rust.rs
git commit -m "fix(a2r): emit char literal for single-char string comparisons against char expressions"
```

---

### Task 6: Fix `&str` concatenation with `+` operator

**Files:**
- Modify: `d:/autostack/auto-lang/crates/auto-lang/src/trans/rust.rs` (binary Add handler)

**Context:** Code like `out = out + " "` tries to concatenate `&str` values. Rust requires at least one side to be `String` for `+`. The simplest fix: detect string concatenation and emit `format!()`.

**Step 1: Find the binary Add handler**

The default binary operator handling is around lines 631-644. `Op::Add` falls through to the generic case which emits `lhs + rhs`.

**Step 2: Add special case for string concatenation**

When both sides are string-like (at least one is `Expr::Str`, or the LHS variable was assigned a string), emit `format!("{}{}", lhs, rhs)` instead.

A pragmatic approach: when `Op::Add` and either operand is `Expr::Str`, or when the LHS is `Expr::Str`, emit via format:

```rust
Op::Add => {
    // Check for string concatenation pattern
    if is_str_expr(&lhs) || is_str_expr(&rhs) {
        write!(out, "format!(\"{{}}{{}}\", ")?;
        self.expr(&lhs, out)?;
        write!(out, ", ")?;
        self.expr(&rhs, out)?;
        write!(out, ")")?;
        return Ok(());
    }
    // Default: numeric addition
    self.expr(&lhs, out)?;
    write!(out, " + ")?;
    self.expr(&rhs, out)?;
}
```

Where `is_str_expr` checks for `Expr::Str(_)` or known string-type variables.

**Step 3: Test with example 06**

```bash
cd /d/autostack/auto-lang && cargo build --release -p auto
cd /d/autostack/auto-code-rs/crates/ac-examples/src/06_line_formatter
/d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust
grep "format!" main.a2r.rs | head -5
```

**Step 4: Commit**

```bash
cd /d/autostack/auto-lang
git add crates/auto-lang/src/trans/rust.rs
git commit -m "fix(a2r): use format!() for string concatenation with + operator"
```

---

### Task 7: Add `Json` module to `a2r_std.rs`

**Files:**
- Modify: `d:/autostack/auto-lang/crates/auto-lang/src/a2r_std.rs`

**Context:** Examples 11, 16, 21, 22, 23, 27 use `Json.is_valid()`, `Json.get_at()`, `Json.get()`, `Json.as_string()`. These don't exist in Rust. We add thin wrappers around `serde_json`.

**Step 1: Add the Json module at the end of `a2r_std.rs`**

```rust
/// AutoLang's Json module - thin wrappers around serde_json for transpiled code
pub mod Json {
    use serde_json::Value;

    pub fn is_valid(s: &str) -> bool {
        serde_json::from_str::<Value>(s).is_ok()
    }

    pub fn parse(s: &str) -> Option<Value> {
        serde_json::from_str(s).ok()
    }

    pub fn get_at(val: &Value, idx: usize) -> Option<Value> {
        val.get(idx).cloned()
    }

    pub fn get<'a>(val: &'a Value, key: &str) -> Option<&'a Value> {
        val.get(key)
    }

    pub fn as_string(val: &Value) -> Option<String> {
        val.as_str().map(|s| s.to_string())
    }

    pub fn to_string(val: &Value) -> String {
        serde_json::to_string(val).unwrap_or_default()
    }
}
```

**Step 2: Ensure `serde_json` is a dependency**

```bash
cd /d/autostack/auto-lang
grep "serde_json" crates/auto-lang/Cargo.toml
```

If not present, add it:
```toml
serde_json = "1"
```

**Step 3: Update `emit_a2r_stdlib` to also import Json**

In `trans/rust.rs:393-399`:

```rust
fn emit_a2r_stdlib(&self, out: &mut impl Write) -> AutoResult<()> {
    writeln!(out, "// a2r Standard Library (from crate)")?;
    writeln!(out, "#[allow(unused_imports)]")?;
    writeln!(out, "use auto_lang::a2r_std::*;")?;
    writeln!(out, "#[allow(unused_imports)]")?;
    writeln!(out, "use auto_lang::a2r_std::Json;")?;
    writeln!(out)?;
    Ok(())
}
```

**Step 4: Rebuild and test**

```bash
cd /d/autostack/auto-lang && cargo build --release -p auto
cd /d/autostack/auto-code-rs/crates/ac-examples/src
for dir in */; do
  [ -f "$dir/main.at" ] && cd "$dir" && /d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust 2>/dev/null && cd .. || cd ..
done
# Verify Json appears in examples that need it
grep "Json::" 11_tool_result_serde/main.a2r.rs
```

**Step 5: Commit**

```bash
cd /d/autostack/auto-lang
git add crates/auto-lang/src/a2r_std.rs crates/auto-lang/src/trans/rust.rs crates/auto-lang/Cargo.toml
git commit -m "feat(a2r): add Json runtime shim wrapping serde_json"
```

---

### Task 8: Validate all 33 examples and produce final comparison

**Files:**
- All 33 `main.a2r.rs` files in `d:/autostack/auto-code-rs/crates/ac-examples/src/*/`

**Step 1: Re-transpile everything**

```bash
cd /d/autostack/auto-code-rs/crates/ac-examples/src
for dir in */; do
  [ -f "$dir/main.at" ] && cd "$dir" && /d/autostack/auto-lang/target/release/auto.exe trans --path main.at rust 2>/dev/null && cd .. || cd ..
done
```

**Step 2: For each example, compare the generated a2r.rs against the original main.rs**

For each of the 33 examples, read both files and assess:
- Does the a2r.rs compile? (check for missing types, syntax errors)
- Is it functionally equivalent to main.rs?

**Step 3: Categorize results**

| Category | Count | Examples |
|----------|-------|---------|
| Functionally equivalent | ? | ... |
| Compiles but differs | ? | ... |
| Still won't compile | ? | ... |

**Step 4: Document remaining gaps**

For any examples that still don't compile, document what's missing and whether it needs:
- A transpiler fix (specific code)
- An a2r_std addition (runtime support)
- An Auto language feature the transpiler can't express

**Step 5: Commit the final transpiled outputs**

```bash
cd /d/autostack/auto-code-rs
git add crates/ac-examples/src/*/main.a2r.rs
git commit -m "chore: update a2r transpiled outputs after transpiler improvements"
```

---

## Summary

| Task | Phase | What | Impact |
|------|-------|------|--------|
| 1 | 1 | Fix `trans_rust_with_session` to use full `trans()` | Unblocks ~25 examples |
| 2 | 1 | Ensure top-level statements go into main wrapper | Correctness |
| 3 | 2 | Fix `.slice(start, end)` two-arg form | Fixes 14, 15, 16 |
| 4 | 2 | Add `as usize` casts for integer indices | Fixes 10+ examples |
| 5 | 2 | Fix char vs string literal comparison | Fixes 06, 15 |
| 6 | 2 | Fix `&str` concatenation with `+` | Fixes 06 |
| 7 | 3 | Add Json runtime shim | Fixes 11, 16, 21, 22, 23, 27 |
| 8 | 4 | Validate all 33 examples and document | Final assessment |
