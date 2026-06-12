# a2gd Maturation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bring the GDScript transpiler (a2gd) to feature parity with the Python transpiler (a2py) by porting proven architecture patterns.

**Architecture:** Port a2py's method mapping, builtin function mapping, two-phase transpilation, type tracking, generic support, async handling, and spec generation into the existing `GDScriptTrans` struct. Adapt all mappings for GDScript syntax (tab indent, `func`/`var` keywords, `%`-format strings, `null`/`true`/`false` lowercase).

**Tech Stack:** Rust, AutoLang parser/AST (`crate::ast::*`), GDScript 2.0 (Godot 4.x)

**Design Doc:** `docs/plans/305-a2gd-maturation.md`

**Key Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs` (main transpiler — 1,297 lines)
- Reference: `crates/auto-lang/src/trans/python.rs` (a2py — 2,377 lines, source of patterns)
- Test dir: `crates/auto-lang/test/a2gd/`
- Build: `cargo build -p auto` before running tests
- Test: `cargo test -p auto-lang test_<test_name>`

---

## Task 1: Method Call Mapping — String Methods

**Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs:945-997` (call + dot methods)
- Create: `crates/auto-lang/test/a2gd/04_strings/001_string_methods/string_methods.at`
- Create: `crates/auto-lang/test/a2gd/04_strings/001_string_methods/string_methods.expected.gd`

**Why first:** String method mapping is the single most-used feature. Every program prints and manipulates strings. Without it, even simple AutoLang code generates broken GDScript.

### Step 1: Add `emit_args` helper and `extract_call_name` helper

In `gdscript.rs`, add these two helper methods inside `impl GDScriptTrans` (after the `arg` method, around line 970):

```rust
/// Emit call arguments as comma-separated list
fn emit_args(&mut self, args: &Args, out: &mut impl Write) -> AutoResult<()> {
    for (i, arg) in args.args.iter().enumerate() {
        if i > 0 {
            out.write(b", ")?;
        }
        self.arg(arg, out)?;
    }
    Ok(())
}

/// Extract a plain identifier name from a call expression
fn extract_call_name(&self, expr: &Expr) -> Option<AutoStr> {
    match expr {
        Expr::Ident(name) => Some(name.clone()),
        _ => None,
    }
}
```

### Step 2: Rewrite `dot()` to intercept method calls

Replace the existing `dot()` method (lines 992-997) with a version that intercepts method calls:

```rust
fn dot(&mut self, lhs: &Expr, rhs: &Expr, out: &mut impl Write) -> AutoResult<()> {
    // Intercept method calls: lhs.method(args) where rhs is Expr::Call
    if let Expr::Call(call) = rhs {
        if let Expr::Ident(method_name) = call.name.as_ref() {
            return self.method_call(lhs, method_name, &call.args, out);
        }
    }
    // Default: lhs.rhs
    self.expr(lhs, out)?;
    out.write(b".")?;
    self.expr(rhs, out)?;
    Ok(())
}
```

### Step 3: Add `method_call()` with string method mappings

Add this new method right after `dot()`:

```rust
/// Map AutoLang method calls to GDScript equivalents
fn method_call(
    &mut self,
    receiver: &Expr,
    method: &AutoStr,
    args: &Args,
    out: &mut impl Write,
) -> AutoResult<()> {
    match method.as_ref() {
        // ── String methods ──
        // .trim() → .strip()
        "trim" => {
            self.expr(receiver, out)?;
            out.write(b".strip(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
        // .split(sep) → .split(sep) (same in GDScript)
        "split" => {
            self.expr(receiver, out)?;
            out.write(b".split(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
        // .to_upper() → .to_upper() (same in GDScript)
        "to_upper" | "upper" => {
            self.expr(receiver, out)?;
            out.write(b".to_upper()")?;
        }
        // .to_lower() → .to_lower() (same in GDScript)
        "to_lower" | "lower" => {
            self.expr(receiver, out)?;
            out.write(b".to_lower()")?;
        }
        // .starts_with(s) → .begins_with(s) (GDScript uses begins_with)
        "starts_with" | "startswith" => {
            self.expr(receiver, out)?;
            out.write(b".begins_with(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
        // .ends_with(s) → .ends_with(s) (same in GDScript)
        "ends_with" | "endswith" => {
            self.expr(receiver, out)?;
            out.write(b".ends_with(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
        // .replace(old, new) → .replace(old, new) (same in GDScript)
        "replace" => {
            self.expr(receiver, out)?;
            out.write(b".replace(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
        // .len() → len(receiver)
        "len" => {
            out.write(b"len(")?;
            self.expr(receiver, out)?;
            out.write(b")")?;
        }

        // ── Default: pass through as receiver.method(args) ──
        _ => {
            self.expr(receiver, out)?;
            out.write(b".")?;
            out.write_all(method.as_bytes())?;
            out.write(b"(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
    }
    Ok(())
}
```

### Step 4: Also intercept method calls from `Expr::Bina(Op::Dot)` in `call()`

In the `call()` method (line 945), add interception for `Expr::Dot(obj, method_name)` pattern before the plain emit:

```rust
fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
    // Intercept method calls via dot: obj.method(args)
    if let Expr::Dot(obj, method_name) = call.name.as_ref() {
        return self.method_call(obj, method_name, &call.args, out);
    }

    self.expr(&call.name, out)?;
    out.write(b"(")?;

    for (i, arg) in call.args.args.iter().enumerate() {
        if i > 0 {
            out.write(b", ")?;
        }
        self.arg(arg, out)?;
    }

    out.write(b")")?;
    Ok(())
}
```

### Step 5: Create test `04_strings/001_string_methods`

Create directory and input file:
```
crates/auto-lang/test/a2gd/04_strings/001_string_methods/string_methods.at
```

```auto
fn main() {
    let greeting = "  Hello, World!  "
    let trimmed = greeting.trim()
    let upper = greeting.to_upper()
    let lower = greeting.to_lower()
    let hello = greeting.starts_with("  Hello")
    let world = greeting.ends_with("World!  ")
    let replaced = greeting.replace("World", "GDScript")
    let length = greeting.len()
    print(trimmed)
    print(upper)
    print(replaced)
    print(length)
}
```

### Step 6: Build and run test to generate `.wrong.gd`

Run:
```bash
cargo build -p auto
cargo test -p auto-lang test_string_methods
```

Expected: FAIL — generates `.wrong.gd` file to review.

### Step 7: Review output and create `.expected.gd`

Create the expected output file based on the generated `.wrong.gd` (adjust as needed):

```gdscript
# Auto-generated from string_methods.at — do not edit

extends Node

func _ready():
	var greeting: String = "  Hello, World!  "
	var trimmed = greeting.strip()
	var upper = greeting.to_upper()
	var lower = greeting.to_lower()
	var hello = greeting.begins_with("  Hello")
	var world = greeting.ends_with("World!  ")
	var replaced = greeting.replace("World", "GDScript")
	var length = len(greeting)
	print(trimmed)
	print(upper)
	print(replaced)
	print(length)
```

### Step 8: Run test to verify it passes

Run: `cargo test -p auto-lang test_string_methods`
Expected: PASS

### Step 9: Run all existing tests to verify no regressions

Run: `cargo test -p auto-lang -- tests` (run all a2gd tests)
Expected: All 10 tests pass (9 existing + 1 new)

### Step 10: Commit

```bash
git add crates/auto-lang/src/trans/gdscript.rs crates/auto-lang/test/a2gd/04_strings/
git commit -m "feat(a2gd): add method call mapping system with string methods"
```

---

## Task 2: Method Call Mapping — List/Dict Methods

**Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs` (add to `method_call()`)
- Create: `crates/auto-lang/test/a2gd/10_collections/001_array_methods/array_methods.at`
- Create: `crates/auto-lang/test/a2gd/10_collections/001_array_methods/array_methods.expected.gd`
- Create: `crates/auto-lang/test/a2gd/10_collections/002_dict_methods/dict_methods.at`
- Create: `crates/auto-lang/test/a2gd/10_collections/002_dict_methods/dict_methods.expected.gd`

### Step 1: Add List/Array method mappings to `method_call()`

Add these match arms inside `method_call()` before the default `_` arm:

```rust
        // ── List/Array methods ──
        // .push(item) → .append(item) (GDScript uses append)
        "push" => {
            self.expr(receiver, out)?;
            out.write(b".append(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
        // .pop() → .pop() (same in GDScript)
        "pop" => {
            self.expr(receiver, out)?;
            out.write(b".pop(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
        // .contains(item) → item in receiver
        "contains" => {
            if let Some(first_arg) = args.args.first() {
                self.arg(first_arg, out)?;
                out.write(b" in ")?;
                self.expr(receiver, out)?;
            } else {
                self.expr(receiver, out)?;
                out.write(b".contains(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
        }
        // .join(sep) → sep.join(receiver) (Python-style; GDScript also has String.join)
        "join" => {
            if let Some(first_arg) = args.args.first() {
                self.arg(first_arg, out)?;
                out.write(b".join(")?;
                self.expr(receiver, out)?;
                out.write(b")")?;
            } else {
                self.expr(receiver, out)?;
                out.write(b".join(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
        }

        // ── Dict/Map methods ──
        // .set(k, v) / .insert(k, v) → dict[k] = v
        "set" | "insert" => {
            self.expr(receiver, out)?;
            out.write(b"[")?;
            if let Some(first) = args.args.first() {
                self.arg(first, out)?;
            }
            out.write(b"] = ")?;
            if args.args.len() > 1 {
                self.arg(&args.args[1], out)?;
            } else {
                out.write(b"null")?;
            }
        }
        // .get(key) → receiver.get(key)
        "get" => {
            self.expr(receiver, out)?;
            out.write(b".get(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
        // .has(key) / .contains_key(key) → key in receiver
        "has" | "contains_key" => {
            if let Some(first_arg) = args.args.first() {
                self.arg(first_arg, out)?;
                out.write(b" in ")?;
                self.expr(receiver, out)?;
            } else {
                self.expr(receiver, out)?;
                out.write(b".has(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
        }
        // .keys() / .values() — pass through
        "keys" | "values" => {
            self.expr(receiver, out)?;
            out.write(b".")?;
            out.write_all(method.as_bytes())?;
            out.write(b"(")?;
            self.emit_args(args, out)?;
            out.write(b")")?;
        }
```

### Step 2: Create test `10_collections/001_array_methods`

```auto
fn main() {
    let fruits = ["apple", "banana", "cherry"]
    fruits.push("date")
    let last = fruits.pop()
    let count = fruits.len()
    let has_apple = fruits.contains("apple")
    print(count)
    print(has_apple)
}
```

### Step 3: Build, run test, review `.wrong.gd`, create `.expected.gd`

Run: `cargo build -p auto && cargo test -p auto-lang test_array_methods`
Create expected output based on generated `.wrong.gd`.

### Step 4: Create test `10_collections/002_dict_methods`

```auto
fn main() {
    let scores = {"alice": 90, "bob": 85}
    scores.set("charlie", 95)
    let score = scores.get("alice")
    let has_bob = scores.has("bob")
    let all_keys = scores.keys()
    print(score)
    print(has_bob)
    print(all_keys)
}
```

### Step 5: Build, run test, review `.wrong.gd`, create `.expected.gd`

Run: `cargo build -p auto && cargo test -p auto-lang test_dict_methods`

### Step 6: Run all tests

Run: `cargo test -p auto-lang -- tests`
Expected: All 12 tests pass

### Step 7: Commit

```bash
git add crates/auto-lang/src/trans/gdscript.rs crates/auto-lang/test/a2gd/10_collections/
git commit -m "feat(a2gd): add list and dict method mappings"
```

---

## Task 3: Builtin Function Mapping

**Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs` (rewrite `call()`)
- Create: `crates/auto-lang/test/a2gd/16_gdscript_std/001_builtin_map/builtin_map.at`
- Create: `crates/auto-lang/test/a2gd/16_gdscript_std/001_builtin_map/builtin_map.expected.gd`

### Step 1: Add builtin function mapping to `call()`

The `call()` method already intercepts `Expr::Dot` (from Task 1). Now add builtin function interception for plain identifiers. Replace `call()` with:

```rust
fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
    // Intercept method calls via dot: obj.method(args)
    if let Expr::Dot(obj, method_name) = call.name.as_ref() {
        return self.method_call(obj, method_name, &call.args, out);
    }

    // Builtin function mapping
    if let Some(ident) = self.extract_call_name(&call.name) {
        match ident.as_ref() {
            // Identical in GDScript — just pass through
            "print" | "len" | "range" | "abs" | "min" | "max" | "str" | "int" | "float"
            | "clamp" | "lerp" | "wrapi" | "wrapf" => {
                return self.emit_plain_call(call, out);
            }
            // type_name(x) → typeof(x)
            "type_name" => {
                out.write(b"typeof(")?;
                if let Some(arg) = call.args.args.first() {
                    self.arg(arg, out)?;
                }
                out.write(b")")?;
                return Ok(());
            }
            // sleep_ms(ms) → await get_tree().create_timer(ms / 1000.0).timeout
            "sleep_ms" => {
                out.write(b"await get_tree().create_timer(")?;
                if let Some(arg) = call.args.args.first() {
                    self.arg(arg, out)?;
                }
                out.write(b" / 1000.0).timeout")?;
                return Ok(());
            }
            // time_now() → Time.get_ticks_msec() / 1000.0
            "time_now" => {
                out.write(b"Time.get_ticks_msec() / 1000.0")?;
                return Ok(());
            }
            _ => {}
        }
    }

    self.emit_plain_call(call, out)
}

/// Emit a plain function call without any builtin mapping
fn emit_plain_call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
    self.expr(&call.name, out)?;
    out.write(b"(")?;

    for (i, arg) in call.args.args.iter().enumerate() {
        if i > 0 {
            out.write(b", ")?;
        }
        self.arg(arg, out)?;
    }

    out.write(b")")?;
    Ok(())
}
```

### Step 2: Create test `16_gdscript_std/001_builtin_map`

```auto
fn main() {
    let x = -5
    let absolute = abs(x)
    let bigger = max(3, 7)
    let smaller = min(3, 7)
    let t = type_name(x)
    let nums = range(0, 10)
    print(absolute)
    print(bigger)
    print(t)
}
```

### Step 3: Build, run test, review, create expected

Run: `cargo build -p auto && cargo test -p auto-lang test_builtin_map`

### Step 4: Run all tests

Run: `cargo test -p auto-lang -- tests`
Expected: All 13 tests pass

### Step 5: Commit

```bash
git add crates/auto-lang/src/trans/gdscript.rs crates/auto-lang/test/a2gd/16_gdscript_std/
git commit -m "feat(a2gd): add builtin function mapping (abs, max, min, typeof, etc.)"
```

---

## Task 4: Two-Phase Transpilation + Import System

**Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs` (restructure `GDScriptTrans` struct and `trans()`)
- Create: `crates/auto-lang/test/a2gd/14_modules/001_import/import.at`
- Create: `crates/auto-lang/test/a2gd/14_modules/001_import/import.expected.gd`

### Step 1: Update `GDScriptTrans` struct

Add imports tracking fields. Replace the struct definition (lines 8-12) with:

```rust
use std::collections::{HashMap, HashSet};

pub struct GDScriptTrans {
    indent: usize,
    #[allow(dead_code)]
    name: AutoStr,
    /// Collected preload paths from `use` statements
    gd_imports: Vec<(AutoStr, Option<Vec<AutoStr>>)>,
    /// Local variable type tracking (populated in Task 5)
    local_var_types: HashMap<AutoStr, Type>,
}
```

Update `new()`:
```rust
pub fn new(name: AutoStr) -> Self {
    Self {
        indent: 0,
        name,
        gd_imports: Vec::new(),
        local_var_types: HashMap::new(),
    }
}
```

### Step 2: Add `handle_use()` method

Add after the struct and `new()`:

```rust
/// Process a `use` statement for GDScript import emission
fn handle_use(&mut self, use_stmt: &Use) {
    // Only handle Auto and GDScript imports
    match &use_stmt.kind {
        UseKind::Auto => {
            let module_path = use_stmt.path.replace(".", "/");
            let symbols: Option<Vec<AutoStr>> = if use_stmt.symbols.is_empty() {
                None
            } else {
                Some(use_stmt.symbols.iter().cloned().collect())
            };
            self.gd_imports.push((module_path.into(), symbols));
        }
        UseKind::Py => {
            // Python imports not valid in GDScript — skip silently
        }
        UseKind::C | UseKind::Rust => {
            // C/Rust imports not relevant for GDScript — skip
        }
        _ => {}
    }
}
```

### Step 3: Add `emit_imports()` method

```rust
/// Emit collected GDScript imports (preload statements)
fn emit_imports(&self, out: &mut impl Write) -> AutoResult<()> {
    for (path, symbols) in &self.gd_imports {
        // use module → const Module = preload("res://module.gd")
        let module_name = path.rsplit('/').next().unwrap_or(path.as_ref());
        let class_name: String = module_name.chars().next().unwrap_or('m').to_uppercase()
            .to_string() + &module_name[1..];
        write!(out, "const {} = preload(\"res://{}.gd\")\n", class_name, path)?;
        if let Some(syms) = symbols {
            if !syms.is_empty() {
                write!(out, "# imported: {}\n", syms.join(", "))?;
            }
        }
    }
    if !self.gd_imports.is_empty() {
        out.write(b"\n")?;
    }
    Ok(())
}
```

### Step 4: Rewrite `trans()` to two-phase architecture

Replace the `trans()` implementation (lines 1137-1205) with:

```rust
fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
    // Find and save main function if it exists
    let main_func = ast.stmts.iter().find(|s| {
        if let Stmt::Fn(func) = s {
            func.name == "main"
        } else {
            false
        }
    }).cloned();

    // Split into declarations, main statements, and use statements
    let mut decls: Vec<(Stmt, usize)> = Vec::new();
    let mut main_stmts: Vec<(Stmt, usize)> = Vec::new();

    let source_lines = ast.source_lines;
    for (i, stmt) in ast.stmts.into_iter().enumerate() {
        let line = source_lines.get(i).copied().unwrap_or(0);
        // Skip main function — handled specially
        if let Stmt::Fn(func) = &stmt {
            if func.name == "main" {
                continue;
            }
        }
        // Collect use statements early for import emission
        if let Stmt::Use(use_stmt) = &stmt {
            self.handle_use(use_stmt);
            continue;
        }

        if stmt.is_decl() {
            decls.push((stmt, line));
        } else {
            main_stmts.push((stmt, line));
        }
    }

    // ── Phase 2: Generate code body into temporary buffer ──
    let mut code_buf: Vec<u8> = Vec::new();

    // Emit declarations (types, enums, non-main functions)
    for (i, (decl, line)) in decls.iter().enumerate() {
        sink.set_source_line(*line);
        self.stmt(decl, &mut code_buf)?;
        if i < decls.len() - 1 {
            code_buf.write(b"\n")?;
        }
    }

    // Generate main function or wrap statements
    if let Some(main_stmt) = main_func {
        if !decls.is_empty() {
            code_buf.write(b"\n")?;
        }
        self.stmt(&main_stmt, &mut code_buf)?;
    } else if !main_stmts.is_empty() {
        if !decls.is_empty() {
            code_buf.write(b"\n")?;
        }
        code_buf.write(b"func _ready():\n")?;
        self.indent();
        for (stmt, line) in &main_stmts {
            sink.set_source_line(*line);
            self.stmt(stmt, &mut code_buf)?;
        }
        self.dedent();
    }

    // ── Phase 3: Assemble final output ──
    // 1. File header
    write!(sink.body, "# Auto-generated from {}.at — do not edit\n\n", self.name)?;

    // 2. extends Node
    sink.body.write(b"extends Node\n\n")?;

    // 3. Emit collected imports
    self.emit_imports(&mut sink.body)?;

    // 4. Append code body
    sink.body.write(&code_buf)?;

    Ok(())
}
```

### Step 5: Create test `14_modules/001_import`

```auto
use utils
use math: add, multiply

fn main() {
    print("hello")
}
```

### Step 6: Build, run test, review, create expected

Run: `cargo build -p auto && cargo test -p auto-lang test_import`

Expected output should include:
```gdscript
# Auto-generated from import.at — do not edit

extends Node

const Utils = preload("res://utils.gd")
const Math = preload("res://math.gd")
# imported: add, multiply

func _ready():
	print("hello")
```

### Step 7: Run all tests

Run: `cargo test -p auto-lang -- tests`
Expected: All 14 tests pass

### Step 8: Commit

```bash
git add crates/auto-lang/src/trans/gdscript.rs crates/auto-lang/test/a2gd/14_modules/
git commit -m "feat(a2gd): add two-phase transpilation and import system"
```

---

## Task 5: Local Variable Type Tracking

**Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs` (populate `local_var_types` in store/fn_decl)
- Create: `crates/auto-lang/test/a2gd/01_basics/031_typed_vars/typed_vars.at`
- Create: `crates/auto-lang/test/a2gd/01_basics/031_typed_vars/typed_vars.expected.gd`

### Step 1: Add type tracking to `store()`

In the `store()` method, after writing the variable, add type tracking:

```rust
fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
    match store.kind {
        StoreKind::Let | StoreKind::Var => {
            out.write(b"var ")?;
            out.write_all(store.name.as_bytes())?;
            if !matches!(store.ty, Type::Unknown) {
                out.write(b": ")?;
                let type_name = self.gdscript_type_name(&store.ty);
                out.write_all(type_name.as_bytes())?;
            }
            out.write(b" = ")?;
            self.expr(&store.expr, out)?;
            // Track variable type
            self.local_var_types.insert(store.name.clone(), store.ty.clone());
        }
        // ... rest unchanged
    }
    Ok(())
}
```

### Step 2: Add type tracking to `fn_decl()` — populate from params

At the start of `fn_decl()`, after printing indent, clear and populate local types:

```rust
fn fn_decl(&mut self, func: &Fn, out: &mut impl Write) -> AutoResult<()> {
    // Track parameter types
    self.local_var_types.clear();
    for param in &func.params {
        if !matches!(param.ty, Type::Unknown) {
            self.local_var_types.insert(param.name.clone(), param.ty.clone());
        }
    }
    // ... rest of fn_decl unchanged
}
```

### Step 3: Add `infer_type_from_expr()` helper

```rust
/// Basic type inference from expression
fn infer_type_from_expr(&self, expr: &Expr) -> Type {
    match expr {
        Expr::Int(_) => Type::Int,
        Expr::Uint(_) => Type::Uint,
        Expr::Float(_, _) => Type::Float,
        Expr::Bool(_) => Type::Bool,
        Expr::Str(_) => Type::StrOwned,
        Expr::Array(_) => Type::List(Box::new(Type::Unknown)),
        Expr::Object(_) => Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown)),
        Expr::Ident(name) => {
            self.local_var_types.get(name).cloned().unwrap_or(Type::Unknown)
        }
        _ => Type::Unknown,
    }
}
```

### Step 4: Create test `01_basics/031_typed_vars`

```auto
fn main() {
    let x int = 42
    let name str = "Alice"
    let active bool = true
    let score float = 95.5
    print(x)
    print(name)
}
```

### Step 5: Build, run test, review, create expected

Run: `cargo build -p auto && cargo test -p auto-lang test_typed_vars`

### Step 6: Run all tests

Run: `cargo test -p auto-lang -- tests`
Expected: All 15 tests pass

### Step 7: Commit

```bash
git add crates/auto-lang/src/trans/gdscript.rs crates/auto-lang/test/a2gd/01_basics/
git commit -m "feat(a2gd): add local variable type tracking"
```

---

## Task 6: Generic Type Support

**Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs`
- Create: `crates/auto-lang/test/a2gd/08_generics/001_generic_func/generic_func.at`
- Create: `crates/auto-lang/test/a2gd/08_generics/001_generic_func/generic_func.expected.gd`
- Create: `crates/auto-lang/test/a2gd/08_generics/002_generic_struct/generic_struct.at`
- Create: `crates/auto-lang/test/a2gd/08_generics/002_generic_struct/generic_struct.expected.gd`

### Step 1: Add generic parameter detection helpers

```rust
/// Check if a type matches one of the function's generic type params
fn is_generic_param(&self, ty: &Type, generic_params: &[AutoStr]) -> bool {
    if let Type::User(td) = ty {
        generic_params.iter().any(|p| p == &td.name)
    } else {
        false
    }
}

/// Check if a type matches one of the TypeDecl's generic type params
fn is_type_decl_generic_param(&self, ty: &Type, type_params: &[TypeParam]) -> bool {
    if let Type::User(td) = ty {
        type_params.iter().any(|p| p.name == td.name)
    } else {
        false
    }
}
```

### Step 2: Skip type annotations for generic params in `fn_decl()`

In `fn_decl()`, when emitting parameter type annotations, skip generic params:

```rust
for (i, param) in func.params.iter().enumerate() {
    if i > 0 {
        out.write(b", ")?;
    }
    out.write_all(param.name.as_bytes())?;
    // Skip type annotation for generic params
    if !matches!(param.ty, Type::Unknown)
        && !self.is_generic_param(&param.ty, &func.type_params)
    {
        out.write(b": ")?;
        let type_name = self.gdscript_type_name(&param.ty);
        out.write_all(type_name.as_bytes())?;
    }
}
```

Similarly for return type — skip if it's a generic param.

### Step 3: Handle generic struct fields in `type_decl()`

In `type_decl()`, use `Variant` for generic type params:

```rust
for member in &type_decl.members {
    self.print_indent(out)?;
    out.write(b"var ")?;
    out.write_all(member.name.as_bytes())?;
    out.write(b": ")?;
    let type_name = if self.is_type_decl_generic_param(&member.ty, &type_decl.type_params) {
        AutoStr::from("Variant")
    } else {
        self.gdscript_type_name(&member.ty)
    };
    out.write_all(type_name.as_bytes())?;
    out.write(b"\n")?;
}
```

### Step 4: Create test `08_generics/001_generic_func`

```auto
fn identity[T](x T) T {
    x
}

fn main() {
    let result = identity(42)
    print(result)
}
```

Expected GDScript output should have no type annotation on `x` (generic param erased):
```gdscript
func identity(x):
	return x
```

### Step 5: Create test `08_generics/002_generic_struct`

```auto
type Container[T] {
    value T
}

fn main() {
    let c = Container(value: 10)
    print(c.value)
}
```

Expected: `var value: Variant` in the class.

### Step 6: Build, run tests, review, create expected files

Run: `cargo build -p auto && cargo test -p auto-lang test_generic_func && cargo test -p auto-lang test_generic_struct`

### Step 7: Run all tests

Run: `cargo test -p auto-lang -- tests`
Expected: All 17 tests pass

### Step 8: Commit

```bash
git add crates/auto-lang/src/trans/gdscript.rs crates/auto-lang/test/a2gd/08_generics/
git commit -m "feat(a2gd): add generic type support with type erasure"
```

---

## Task 7: Async/Await Enhancement

**Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs`
- Create: `crates/auto-lang/test/a2gd/03_control_flow/040_async_func/async_func.at`
- Create: `crates/auto-lang/test/a2gd/03_control_flow/040_async_func/async_func.expected.gd`

### Step 1: Add async detection helpers

```rust
/// Check if a function has an async return type (~T / Future<T>)
fn is_async_fn(&self, func: &Fn) -> bool {
    // Check for ~T (async) return type
    if let Type::GenericInstance(inst) = &func.ret {
        if inst.base_name == "Future" {
            return true;
        }
    }
    false
}

/// Scan function body for await expressions
fn has_await(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        if Self::stmt_has_await(stmt) {
            return true;
        }
    }
    false
}

fn stmt_has_await(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Expr(expr) => Self::expr_has_await(expr),
        Stmt::Return(expr) => Self::expr_has_await(expr),
        Stmt::If(if_stmt) => {
            if_stmt.branches.iter().any(|b| Self::body_has_await(&b.body))
                || if_stmt.else_.as_ref().map_or(false, |e| Self::body_has_await(e))
        }
        Stmt::For(for_loop) => Self::body_has_await(&for_loop.body),
        Stmt::Is(is_stmt) => is_stmt.branches.iter().any(|b| match b {
            IsBranch::EqBranch(_, body) | IsBranch::IfBranch(_, body) | IsBranch::ElseBranch(body) => Self::body_has_await(body),
        }),
        _ => false,
    }
}

fn expr_has_await(expr: &Expr) -> bool {
    match expr {
        Expr::Await { .. } => true,
        Expr::Call(call) => Self::expr_has_await(&call.name)
            || call.args.args.iter().any(|a| match a {
                Arg::Pos(e) => Self::expr_has_await(e),
                Arg::Pair(_, e) => Self::expr_has_await(e),
                Arg::Name(_) => false,
            }),
        Expr::Bina(l, _, r) => Self::expr_has_await(l) || Self::expr_has_await(r),
        _ => false,
    }
}

fn body_has_await(body: &Body) -> bool {
    Self::has_await(&body.stmts)
}
```

### Step 2: Note about GDScript async

GDScript does NOT need `async def` — all functions can use `await`. The detection helpers are useful for:
- Knowing if a function contains `await` (for documentation/comments)
- The `sleep_ms()` mapping already emits `await get_tree().create_timer(...).timeout`

No changes needed to `fn_decl()` — GDScript handles `await` natively without `async` keyword.

### Step 3: Create test `03_control_flow/040_async_func`

```auto
fn fetch_data() ~str {
    await get_data()
    let result = get_result()
    result
}

fn main() {
    let data = fetch_data()
    print(data)
}
```

Expected GDScript: no `async` keyword, just plain `func` with `await` inside:
```gdscript
func fetch_data() -> String:
	await get_data()
	var result = get_result()
	return result
```

### Step 4: Build, run test, review, create expected

Run: `cargo build -p auto && cargo test -p auto-lang test_async_func`

### Step 5: Run all tests

Run: `cargo test -p auto-lang -- tests`
Expected: All 18 tests pass

### Step 6: Commit

```bash
git add crates/auto-lang/src/trans/gdscript.rs crates/auto-lang/test/a2gd/03_control_flow/
git commit -m "feat(a2gd): add async/await detection helpers"
```

---

## Task 8: Spec Declaration Generation

**Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs` (replace spec handler at lines 383-390)
- Create: `crates/auto-lang/test/a2gd/12_specs/001_basic_spec/basic_spec.at`
- Create: `crates/auto-lang/test/a2gd/12_specs/001_basic_spec/basic_spec.expected.gd`

### Step 1: Rewrite spec declaration handler

Replace the current comment-only spec handler (lines 383-390 in `stmt()`) with:

```rust
Stmt::SpecDecl(spec_decl) => {
    self.spec_decl(spec_decl, out)?;
    Ok(true)
}
```

Add new `spec_decl()` method:

```rust
fn spec_decl(&mut self, spec_decl: &SpecDecl, out: &mut impl Write) -> AutoResult<()> {
    self.print_indent(out)?;
    out.write(b"# Protocol: ")?;
    out.write_all(spec_decl.name.as_bytes())?;
    out.write(b"\n")?;

    self.print_indent(out)?;
    out.write(b"class ")?;
    out.write_all(spec_decl.name.as_bytes())?;
    out.write(b":\n")?;

    self.indent();
    if spec_decl.methods.is_empty() {
        self.print_indent(out)?;
        out.write(b"pass\n")?;
    } else {
        for method in &spec_decl.methods {
            self.print_indent(out)?;
            out.write(b"# Abstract: must override\n")?;
            self.print_indent(out)?;
            out.write(b"func ")?;
            out.write_all(method.name.as_bytes())?;
            out.write(b"(")?;
            for (i, param) in method.params.iter().enumerate() {
                if i > 0 {
                    out.write(b", ")?;
                }
                out.write_all(param.name.as_bytes())?;
                if !matches!(param.ty, Type::Unknown) {
                    out.write(b": ")?;
                    let type_name = self.gdscript_type_name(&param.ty);
                    out.write_all(type_name.as_bytes())?;
                }
            }
            out.write(b")")?;
            if !matches!(method.ret, Type::Unknown | Type::Void) {
                out.write(b" -> ")?;
                let type_name = self.gdscript_type_name(&method.ret);
                out.write_all(type_name.as_bytes())?;
            }
            out.write(b":\n")?;
            self.indent();
            self.print_indent(out)?;
            out.write(b"pass\n")?;
            self.dedent();
        }
    }
    self.dedent();
    Ok(())
}
```

### Step 2: Create test `12_specs/001_basic_spec`

```auto
spec Drawable {
    draw() void
    area() float
}
```

### Step 3: Build, run test, review, create expected

Run: `cargo build -p auto && cargo test -p auto-lang test_basic_spec`

### Step 4: Run all tests

Run: `cargo test -p auto-lang -- tests`
Expected: All 19 tests pass

### Step 5: Commit

```bash
git add crates/auto-lang/src/trans/gdscript.rs crates/auto-lang/test/a2gd/12_specs/
git commit -m "feat(a2gd): generate class stubs for spec declarations"
```

---

## Task 9: Comprehensive Test Suite Expansion

**Files:**
- Create: Multiple test directories under `crates/auto-lang/test/a2gd/`
- Modify: `crates/auto-lang/src/trans/gdscript.rs` (add test functions)

This task adds ~20 more test cases to cover edge cases and bring total to ~40. Each test follows the same pattern: create `input.at` + `input.expected.gd`, add `#[test]` function.

### Step 1: Add `01_basics` test category (6 tests)

Create these tests in `crates/auto-lang/test/a2gd/01_basics/`:

**040_comments/comments.at:**
```auto
fn main() {
    // This is a comment
    let x = 10
    print(x)
}
```

**041_unary_neg/unary_neg.at:**
```auto
fn main() {
    let x = -5
    let y = -x
    print(y)
}
```

**042_unary_not/unary_not.at:**
```auto
fn main() {
    let flag = true
    let result = not flag
    print(result)
}
```

**044_const_decl/const_decl.at:**
```auto
const PI = 3.14

fn main() {
    print(PI)
}
```

**046_boolean_ops/boolean_ops.at:**
```auto
fn main() {
    let a = true
    let b = false
    let c = a and b
    let d = a or b
    let e = not a
    print(c)
    print(d)
    print(e)
}
```

**047_arithmetic/arithmetic.at:**
```auto
fn main() {
    let a = 10
    let b = 3
    let sum = a + b
    let diff = a - b
    let prod = a * b
    let quot = a / b
    let rem = a % b
    print(sum)
    print(diff)
    print(prod)
}
```

For each: build, run test, review `.wrong.gd`, create `.expected.gd`, add `#[test]` function.

### Step 2: Add `02_types` test category (3 tests)

**006_nested_struct/nested_struct.at:**
```auto
type Point {
    x int
    y int
}

type Rect {
    origin Point
    size int
}
```

**007_type_with_methods/type_with_methods.at:**
```auto
type Counter {
    count int

    fn increment() int {
        .count + 1
    }
}
```

**005_tag/tag.at:**
```auto
tag Shape {
    circle float
    square float
}
```

### Step 3: Add `05_expressions` test category (5 tests)

**010_lambda/lambda.at:**
```auto
fn main() {
    let double = (x) => x * 2
    print(double(5))
}
```

**020_tuple/tuple.at:**
```auto
fn main() {
    let point = (3, 4)
    print(point)
}
```

**021_object/object.at:**
```auto
fn main() {
    let config = {"host": "localhost", "port": 8080}
    print(config)
}
```

**030_null_coalesce/null_coalesce.at:**
```auto
fn main() {
    let x = None
    let result = x ?? "default"
    print(result)
}
```

**032_chained_method/chained_method.at:**
```auto
fn main() {
    let result = "  Hello  ".trim().to_upper()
    print(result)
}
```

### Step 4: Add `09_option_result` test category (3 tests)

**001_option/option.at:**
```auto
fn main() {
    let a = Some(42)
    let b = None
    print(a)
    print(b)
}
```

**003_result_ok/result_ok.at:**
```auto
fn main() {
    let ok = Ok(100)
    let err = Err("failed")
    print(ok)
    print(err)
}
```

**006_propagate/propagate.at:**
```auto
fn get_value() int {
    let x = fetch()?
    x
}

fn main() {
    let v = get_value()
    print(v)
}
```

### Step 5: Add test functions in `gdscript.rs`

Add a `#[test]` function for each new test case in the `mod tests` block at the bottom of `gdscript.rs`. Follow the existing pattern:

```rust
#[test]
fn test_comments() { test_a2gd("01_basics/040_comments").unwrap(); }

#[test]
fn test_unary_neg() { test_a2gd("01_basics/041_unary_neg").unwrap(); }
// ... etc for each test
```

### Step 6: Run all tests

Run: `cargo test -p auto-lang -- tests`
Expected: All ~40 tests pass

### Step 7: Commit

```bash
git add crates/auto-lang/src/trans/gdscript.rs crates/auto-lang/test/a2gd/
git commit -m "test(a2gd): expand test suite to 40 tests across 10 categories"
```

---

## Task Dependency Summary

```
Task 1 (String methods) ─────┐
Task 2 (List/Dict methods) ──┤  ← depends on Task 1 (method_call infrastructure)
Task 3 (Builtin mapping) ────┤  ← depends on Task 1 (call interception)
Task 4 (Import system) ──────┤  ← independent
Task 5 (Type tracking) ──────┤  ← depends on Task 4 (struct changes)
Task 6 (Generics) ───────────┤  ← depends on Task 5 (type tracking)
Task 7 (Async/Await) ────────┤  ← independent
Task 8 (Spec generation) ────┤  ← independent
Task 9 (Test expansion) ─────┘  ← depends on Tasks 1-8 all complete
```

**Implementation order**: 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9

Total estimated test count: ~40 (from initial 9)
