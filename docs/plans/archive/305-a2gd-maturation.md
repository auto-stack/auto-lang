# Plan 305: a2gd Maturation — Align with a2py Feature Parity

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Date**: 2026-06-13
**Status**: Completed ✅
**Depends on**: Plan 290 (a2gd initial implementation — completed)
**Precedes**: Future Plan for GDScript/Godot-specific features (Phase B)

## Goal

Bring the GDScript transpiler (a2gd) to feature parity with the Python transpiler (a2py) by porting a2py's proven architecture patterns and feature set. This covers only **generic language features** — Godot engine-specific features (signals, @export, node lifecycle, etc.) are deferred to Phase B.

**Architecture:** Port a2py's method mapping, builtin function mapping, two-phase transpilation, type tracking, generic support, async handling, and spec generation into the existing `GDScriptTrans` struct. Adapt all mappings for GDScript syntax (tab indent, `func`/`var` keywords, `%`-format strings, `null`/`true`/`false` lowercase).

**Tech Stack:** Rust, AutoLang parser/AST (`crate::ast::*`), GDScript 2.0 (Godot 4.x)

**Key Files:**
- Modify: `crates/auto-lang/src/trans/gdscript.rs` (main transpiler — 1,297 lines)
- Reference: `crates/auto-lang/src/trans/python.rs` (a2py — 2,377 lines, source of patterns)
- Test dir: `crates/auto-lang/test/a2gd/`
- Build: `cargo build -p auto` before running tests
- Test: `cargo test -p auto-lang test_<test_name>`

## Current State

| Metric | a2py (Python) | a2gd (GDScript) | Gap |
|---|---|---|---|
| Code size | 2,377 lines | 1,297 lines | -1,080 lines |
| Test cases | 96 | 9 | -87 tests |
| Method mapping | ✅ Full (String/List/Dict) | ❌ None | Critical |
| Builtin mapping | ✅ 20+ functions | ❌ None | Critical |
| Import system | ✅ Two-phase | ❌ `use` skipped | High |
| Type tracking | ✅ HashMap | ❌ None | Medium |
| Generics | ✅ Type erasure | ❌ None | Medium |
| Async/Await | ✅ Full | ⚠️ `await` only | Medium |
| Spec generation | ✅ Protocol | ❌ Comment only | Low |

## Architecture: Reuse a2py Patterns

| Aspect | a2py (Python) | a2gd (GDScript) Adaptation |
|---|---|---|
| Struct fields | `@dataclass` + `x: int` | `var x: int` in class body |
| Indentation | 4 spaces | Tab characters |
| Boolean literals | `True`/`False` | `true`/`false` |
| Null literal | `None` | `null` |
| Function keyword | `def` | `func` |
| Main entry | `main()` + `if __name__` guard | `_ready()` in `extends Node` |
| F-strings | `f"Hello {name}"` | `"Hello %s" % name` |
| String methods | `.strip()`, `.upper()`, `.startswith()` | `.strip()`, `.to_upper()`, `.begins_with()` |
| List append | `.append()` | `.append()` (same) |
| Logical ops | `and`/`or`/`not` | `and`/`or`/`not` (same) |
| Division | `/` float division | `/` integer division (careful) |

## GDScript Method Reference (Godot 4.x)

| Python | GDScript | Notes |
|---|---|---|
| `.strip()` | `.strip()` | Same |
| `.upper()` | `.to_upper()` | Different name |
| `.lower()` | `.to_lower()` | Different name |
| `.startswith(s)` | `.begins_with(s)` | Different name |
| `.endswith(s)` | `.ends_with(s)` | Same |
| `.replace(a, b)` | `.replace(a, b)` | Same |
| `.split(s)` | `.split(s)` | Same |
| `.join(list)` | `s.join(list)` | Same pattern |
| `.append(x)` | `.append(x)` | Same |
| `.pop()` | `.pop()` | Same |
| `len(x)` | `len(x)` | Same |
| `x in list` | `x in list` | Same |
| `x in dict` | `x in dict` | Same |

## Task Dependency Graph

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

---

## Task 1: Method Call Mapping — String Methods

**Priority**: Critical — Most impactful single improvement
**Reference**: a2py `method_call()` (lines 978-1142)

### Step 1: Add `emit_args` and `extract_call_name` helpers

In `gdscript.rs`, add inside `impl GDScriptTrans` (after the `arg` method, around line 970):

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

Replace existing `dot()` (lines 992-997):

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

```rust
fn method_call(
    &mut self,
    receiver: &Expr,
    method: &AutoStr,
    args: &Args,
    out: &mut impl Write,
) -> AutoResult<()> {
    match method.as_ref() {
        // String methods
        "trim" => { self.expr(receiver, out)?; out.write(b".strip(")?; self.emit_args(args, out)?; out.write(b")")?; }
        "split" => { self.expr(receiver, out)?; out.write(b".split(")?; self.emit_args(args, out)?; out.write(b")")?; }
        "to_upper" | "upper" => { self.expr(receiver, out)?; out.write(b".to_upper()")?; }
        "to_lower" | "lower" => { self.expr(receiver, out)?; out.write(b".to_lower()")?; }
        "starts_with" | "startswith" => { self.expr(receiver, out)?; out.write(b".begins_with(")?; self.emit_args(args, out)?; out.write(b")")?; }
        "ends_with" | "endswith" => { self.expr(receiver, out)?; out.write(b".ends_with(")?; self.emit_args(args, out)?; out.write(b")")?; }
        "replace" => { self.expr(receiver, out)?; out.write(b".replace(")?; self.emit_args(args, out)?; out.write(b")")?; }
        "len" => { out.write(b"len(")?; self.expr(receiver, out)?; out.write(b")")?; }
        _ => { self.expr(receiver, out)?; out.write(b".")?; out.write_all(method.as_bytes())?; out.write(b"(")?; self.emit_args(args, out)?; out.write(b")")?; }
    }
    Ok(())
}
```

### Step 4: Intercept method calls from `Expr::Bina(Op::Dot)` in `call()`

In `call()`, add interception before plain emit:

```rust
fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
    // Intercept method calls via dot: obj.method(args)
    if let Expr::Dot(obj, method_name) = call.name.as_ref() {
        return self.method_call(obj, method_name, &call.args, out);
    }
    // ... rest unchanged
}
```

### Step 5: Create test `04_strings/001_string_methods`

### Step 6: Build, run test, review `.wrong.gd`, create `.expected.gd`

### Step 7: Run all tests, commit

---

## Task 2: Method Call Mapping — List/Dict Methods

**Priority**: Critical
**Reference**: a2py `method_call()` List/Dict section

Add to `method_call()` before default arm:

- List: `push`→`.append()`, `pop`→`.pop()`, `contains`→`x in arr`, `join`→`sep.join(arr)`
- Dict: `set`/`insert`→`dict[k]=v`, `get`→`.get()`, `has`/`contains_key`→`k in dict`, `keys`/`values`→pass through

Tests: `10_collections/001_array_methods`, `10_collections/002_dict_methods`

---

## Task 3: Builtin Function Mapping

**Priority**: Critical
**Reference**: a2py `call()` (lines 860-903)

Add builtin function interception in `call()` after dot interception:

| Auto | GDScript | Notes |
|---|---|---|
| `print`, `len`, `range`, `abs`, `min`, `max`, `str`, `int`, `float`, `clamp`, `lerp` | pass through | Same in GDScript |
| `type_name(x)` | `typeof(x)` | Different name |
| `sleep_ms(ms)` | `await get_tree().create_timer(ms / 1000.0).timeout` | Async context |
| `time_now()` | `Time.get_ticks_msec() / 1000.0` | Godot Time singleton |

Add `emit_plain_call()` helper. Test: `16_gdscript_std/001_builtin_map`

---

## Task 4: Two-Phase Transpilation + Import System

**Priority**: High
**Reference**: a2py `trans()` (lines 1643-1818), `handle_use()` (lines 1564-1599)

Restructure `GDScriptTrans`:

```rust
pub struct GDScriptTrans {
    indent: usize,
    name: AutoStr,
    gd_imports: Vec<(AutoStr, Option<Vec<AutoStr>>)>,
    local_var_types: HashMap<AutoStr, Type>,
}
```

- Add `handle_use()` and `emit_imports()` methods
- `use module` → `const Module = preload("res://module.gd")`
- `use module: sym` → preload + comment
- `use c <...>` / `use.py ...` → skip
- Restructure `trans()` into Phase 1 (collect), Phase 2 (codegen to buf), Phase 3 (assemble)

Test: `14_modules/001_import`

---

## Task 5: Local Variable Type Tracking

**Priority**: Medium
**Reference**: a2py `local_var_types` (line 31), `infer_type_from_expr()` (lines 1525-1557)

- Populate `local_var_types` from function params and `Store` statements
- Add `infer_type_from_expr()` for basic type inference
- Use tracked types for more precise GDScript type annotations

Test: `01_basics/031_typed_vars`

---

## Task 6: Generic Type Support

**Priority**: Medium
**Reference**: a2py `is_generic_param()` (lines 492-502)

- `is_generic_param()` / `is_type_decl_generic_param()` helpers
- Skip type annotations for generic params in `fn_decl()`
- Use `Variant` for generic struct fields in `type_decl()`

Tests: `08_generics/001_generic_func`, `08_generics/002_generic_struct`

---

## Task 7: Async/Await Enhancement

**Priority**: Medium
**Reference**: a2py `is_async_fn()` (lines 464-467), `has_await()` (lines 470-479)

GDScript does NOT need `async def` — all functions can `await`. Add detection helpers for:
- `is_async_fn()` — detect `~T` / `Future<T>` return types
- `has_await()` / `stmt_has_await()` / `expr_has_await()` — scan body for await expressions

Test: `03_control_flow/040_async_func`

---

## Task 8: Spec Declaration Generation

**Priority**: Low
**Reference**: a2py `spec_decl()` (lines 1275-1314)

Replace comment-only spec handler with GDScript class stubs:

```gdscript
# Protocol: SpecName
class SpecName:
	# Abstract: must override
	func method_name(params) -> ReturnType:
		pass
```

Test: `12_specs/001_basic_spec`

---

## Task 9: Comprehensive Test Suite Expansion

**Priority**: High — Runs alongside all other tasks
**Reference**: a2py test suite (96 tests across 20 categories)

Expand from 9 to ~40 tests:

| Category | Tests | Covers |
|---|---|---|
| `01_basics/` | ~6 | Comments, unary ops, const, boolean ops, arithmetic |
| `02_types/` | ~3 | Nested struct, type with methods, tag |
| `03_control_flow/` | ~1 | Async/await |
| `04_strings/` | ~1 | String methods |
| `05_expressions/` | ~5 | Lambda, tuple, object, null coalesce, chained methods |
| `08_generics/` | ~2 | Generic func, generic struct |
| `09_option_result/` | ~3 | Option, Result, propagate |
| `10_collections/` | ~2 | Array methods, dict methods |
| `12_specs/` | ~1 | Spec declaration |
| `14_modules/` | ~1 | Import statements |
| `16_gdscript_std/` | ~1 | Builtin mapping |

---

## Success Criteria (Phase A)

- [x] All existing 9 tests continue to pass
- [x] Method mapping works for all String/List/Dict methods
- [x] Builtin function mapping covers all functions listed
- [x] `use` statements generate proper GDScript preload code
- [x] Type tracking produces correct GDScript type annotations
- [x] Generic functions/structs transpile without errors
- [x] Async functions with `await` generate valid GDScript code
- [x] Spec declarations generate GDScript class stubs
- [x] Total test count reaches 30 (from initial 9)

---

## Phase B Roadmap: GDScript/Godot-Specific Features

*Derived from exhaustive analysis of the GDScript parser in the Godot engine source code.*

### B1: Godot Annotations System (Highest Priority)
`#[gd_export]`, `#[gd_onready]`, `#[gd_tool]`, `#[gd_rpc]` → `@export`, `@onready`, `@tool`, `@rpc`

### B2: Signal System
`signal health_changed(old: int, new_value: int)` with `.emit()` and `.connect()`

### B3: Property Setters/Getters
```gdscript
var health: int = 100:
    get: return health
    set(value): health = value
```

### B4: Class System Enhancements
Inner classes, `super()` inheritance, `class_name`/`extends` customization, static variables

### B5: Typed Collections
`List<int>` → `Array[int]`, `Map<str, int>` → `Dictionary[String, int]`, packed arrays

### B6: Godot Built-in Types
Vector2, Vector3, Color, Rect2, Transform2D/3D, Quaternion, NodePath, StringName, RID, Callable, Signal

### B7: Node Access Syntax ($ and %)
`$Sprite2D` / `%UniqueLabel` / `$Parent/Child`

### B8: Special Constants and Keywords
PI, TAU, INF, NAN, assert, breakpoint, preload, load

### B9: Enhanced Match Patterns
Array patterns, dictionary patterns, rest patterns, binding patterns, guard with `when`

### B10: Lambda `.call()` Requirement
GDScript lambdas are `Callable` objects — must invoke with `.call()` not direct `()`
