# Plan 305: a2gd Maturation — Align with a2py Feature Parity

**Date**: 2026-06-13
**Status**: Design
**Depends on**: Plan 290 (a2gd initial implementation — completed)
**Precedes**: Future Plan for GDScript/Godot-specific features (Phase B)

## Goal

Bring the GDScript transpiler (a2gd) to feature parity with the Python transpiler (a2py) by porting a2py's proven architecture patterns and feature set. This covers only **generic language features** — Godot engine-specific features (signals, @export, node lifecycle, etc.) are deferred to Phase B.

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
| Struct destructuring | ✅ In match | ❌ None | Low |

## Architecture: Reuse a2py Patterns

The core strategy is to replicate a2py's proven architecture in a2gd, adapting only for GDScript syntax differences:

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

## Tasks

### Task 1: Method Call Mapping System
**Priority**: Critical — Most impactful single improvement
**Reference**: a2py `method_call()` (lines 978-1142)

Add `method_call()` to `GDScriptTrans` with GDScript-specific method translations:

**List/Array methods:**
| Auto | GDScript |
|---|---|
| `.push(x)` | `.append(x)` |
| `.pop()` | `.pop()` |
| `.len()` | `len(arr)` |
| `.contains(x)` | `x in arr` |
| `.join(sep)` | `"sep".join(arr)` |

**Dict/Map methods:**
| Auto | GDScript |
|---|---|
| `.set(k, v)` / `.insert(k, v)` | `dict[k] = v` |
| `.get(key)` | `dict.get(key)` |
| `.has(k)` / `.contains_key(k)` | `k in dict` |
| `.keys()` / `.values()` | `.keys()` / `.values()` |

**String methods (GDScript-specific naming):**
| Auto | GDScript |
|---|---|
| `.trim()` | `.strip()` |
| `.split(sep)` | `.split(sep)` |
| `.to_upper()` | `.to_upper()` |
| `.to_lower()` | `.to_lower()` |
| `.starts_with(s)` | `.begins_with(s)` |
| `.ends_with(s)` | `.ends_with(s)` |
| `.replace(old, new)` | `.replace(old, new)` |
| `.len()` | `len(s)` |

Also add `dot()` interception for method calls (like a2py lines 961-975), and `call()` interception for method-call-on-dot patterns (like a2py lines 860-863).

**Tests**: Add test directory `01_methods/` with cases for string methods, list methods, dict methods, chained methods.

---

### Task 2: Builtin Function Mapping
**Priority**: Critical
**Reference**: a2py `call()` (lines 866-901)

Add builtin function interception in `call()`:

| Auto | GDScript | Notes |
|---|---|---|
| `print(...)` | `print(...)` | Pass through |
| `len(x)` | `len(x)` | Pass through |
| `range(a, b)` | `range(a, b)` | Pass through |
| `abs(x)` | `abs(x)` | Pass through |
| `min(a,b)` / `max(a,b)` | `min(a,b)` / `max(a,b)` | Pass through |
| `type_name(x)` | `typeof(x)` | GDScript uses `typeof` |
| `sleep_ms(ms)` | `await get_tree().create_timer(ms / 1000.0).timeout` | Requires async context |
| `time_now()` | `Time.get_ticks_msec() / 1000.0` | Godot Time singleton |
| `str(x)` | `str(x)` | Pass through |
| `int(x)` | `int(x)` | Pass through |
| `float(x)` | `float(x)` | Pass through |

**Tests**: Add test directory `16_gdscript_std/` with builtin mapping tests.

---

### Task 3: Two-Phase Transpilation + Import System
**Priority**: High
**Reference**: a2py `trans()` (lines 1643-1818), `handle_use()` (lines 1564-1599)

Restructure `GDScriptTrans` to support two-phase transpilation:

**Phase 1** — Collect imports and metadata:
- Process `use` statements (currently skipped at line 394)
- Scan type declarations for type annotation imports
- Track which GDScript features need `class_name` or `preload`

**Phase 2** — Generate code body into temporary buffer:
- All codegen goes to `code_buf: Vec<u8>`
- Builtin function calls may add imports during this phase

**Phase 3** — Assemble final output:
- Write `extends Node` header
- Write collected imports/preloads
- Write code body from `code_buf`

**New struct fields:**
```rust
pub struct GDScriptTrans {
    indent: usize,
    name: AutoStr,
    /// Collected preload/dependency paths from `use` statements
    imports: HashSet<AutoStr>,
    /// GDScript module imports (preload paths)
    gd_imports: Vec<AutoStr>,
    /// Local variable type tracking
    local_var_types: HashMap<AutoStr, Type>,
}
```

**`use` statement handling:**
- `use module` → `const Module = preload("res://module.gd")` (or pass-through for Godot builtins)
- `use module: Symbol` → preload + comment noting imported symbols
- `use c <header>` → skip (C headers irrelevant in GDScript)
- `use.py module` → skip or warn (Python imports not valid in GDScript)

**Tests**: Add `14_modules/` test directory.

---

### Task 4: Local Variable Type Tracking
**Priority**: Medium
**Reference**: a2py `local_var_types` (line 31), `infer_type_from_expr()` (lines 1525-1557)

Add `local_var_types: HashMap<AutoStr, Type>` to `GDScriptTrans`:
- Populate from function params (param name → param type)
- Populate from `Store` statements with explicit type annotations
- Add `infer_type_from_expr()` for basic type inference
- Use tracked types for more precise GDScript type annotations

**Tests**: Add types in `02_types/` test directory.

---

### Task 5: Generic Type Support
**Priority**: Medium
**Reference**: a2py `is_generic_param()` (lines 492-502)

Add generic type awareness:
- `is_generic_param()` — check if a type matches a function's generic params
- Skip type annotations for generic parameters (use no annotation in GDScript)
- In struct fields, use `Variant` for generic type params
- `is_type_decl_generic_param()` — check TypeDecl-level generic params

**Tests**: Add `08_generics/` test directory.

---

### Task 6: Async/Await Enhancement
**Priority**: Medium
**Reference**: a2py `is_async_fn()` (lines 464-467), `has_await()` (lines 470-479)

GDScript does not use `async def` — all functions can `await`. Enhancements:
- `is_async_fn()` — detect `~T` / `Future<T>` return types
- `has_await()` — scan function body for `.await` / `Await` expressions
- For functions that contain `await`, no special keyword needed (GDScript handles this natively)
- `sleep_ms()` already handled in Task 2 with `await` pattern
- For `_ready()` with await, emit a note that `_ready()` may need `_process()` alternative

**Tests**: Add `03_control_flow/` test directory.

---

### Task 7: Spec Declaration Generation
**Priority**: Low
**Reference**: a2py `spec_decl()` (lines 1275-1314)

Replace the current comment-only output with actual GDScript code:
- Spec → abstract class pattern in GDScript
- Emit `class_name SpecName` with method stubs that contain `pass`
- GDScript doesn't have abstract classes natively, but we can emit:
  ```gdscript
  # Protocol: SpecName
  class_name SpecName
  # Abstract methods — override in implementing classes
  # func method_name(params) -> ReturnType:
  #     pass
  ```

**Tests**: Add `12_specs/` test directory.

---

### Task 8: Comprehensive Test Coverage
**Priority**: High — Runs alongside all other tasks
**Reference**: a2py test suite (96 tests across 20 categories)

Expand from 9 tests to ~50-60, organized into the same category structure as a2py:

| Category | Tests | Covers |
|---|---|---|
| `000_hello` | 1 | Basic output (existing) |
| `001_var` | 1 | Variables (existing) |
| `002_func` | 1 | Functions (existing) |
| `010_if` | 1 | Conditionals (existing) |
| `011_for` | 1 | Loops (existing) |
| `012_match` | 1 | Pattern matching (existing) |
| `013_struct` | 1 | Structs (existing) |
| `014_enum` | 1 | Enums (existing) |
| `015_string` | 1 | F-strings (existing) |
| `01_basics/` | ~10 | Comments, unary ops, const, boolean ops, range, mutable vars |
| `02_types/` | ~6 | Nested struct, type with methods, empty struct, union, tag |
| `03_control_flow/` | ~4 | While loops, nested loops, async/await, loop break |
| `04_strings/` | ~3 | String methods, f-string expressions, concatenation |
| `05_expressions/` | ~10 | Lambda, tuple, object literal, null coalesce, cast, chained methods |
| `06_pattern_matching/` | ~3 | Wildcard, multi-pattern, struct destructuring |
| `08_generics/` | ~3 | Generic function, generic struct, generic method |
| `09_option_result/` | ~8 | Option/Result constructors, propagate, pattern matching |
| `10_collections/` | ~3 | Array operations, object literals, indexing |
| `11_methods/` | ~3 | Static methods, method calls, method params |
| `12_specs/` | ~1 | Spec declaration |
| `14_modules/` | ~2 | Import statements |
| `16_gdscript_std/` | ~2 | Builtin mapping, method mapping |

Each test = `input.at` + `input.expected.gd` pair, with test function in `gdscript.rs`.

---

## Task Dependency Graph

```
Task 1 (Method mapping) ─────┐
Task 2 (Builtin mapping) ─────┤
Task 3 (Import system) ───────┤──→ Task 8 (Tests — runs alongside all)
Task 4 (Type tracking) ───────┤
Task 5 (Generics) ────────────┤
Task 6 (Async/Await) ─────────┤
Task 7 (Spec generation) ─────┘
```

Tasks 1-7 are largely independent and can be implemented in sequence. Task 8 adds tests incrementally as each feature is completed.

**Recommended implementation order**: 1 → 2 → 3 → 4 → 8 → 5 → 6 → 7

- Tasks 1-2 first (highest impact, most frequently needed)
- Task 3 next (structural change, enables import-aware codegen)
- Task 4 next (type tracking enables better codegen)
- Task 8 expanded testing at this point
- Tasks 5-7 last (lower priority, fewer use cases)

## GDScript Method Reference (Godot 4.x)

Key differences from Python that affect method mapping:

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
| `dict[key]` | `dict[key]` | Same |
| `dict.get(k)` | `dict.get(k)` | Same |

## Success Criteria (Phase A)

- [ ] All existing 9 tests continue to pass
- [ ] Method mapping works for all String/List/Dict methods listed above
- [ ] Builtin function mapping covers all functions listed above
- [ ] `use` statements generate proper GDScript preload code
- [ ] Type tracking produces correct GDScript type annotations
- [ ] Generic functions/structs transpile without errors
- [ ] Async functions with `await` generate valid GDScript code
- [ ] Spec declarations generate GDScript class stubs
- [ ] Total test count reaches 50+ (from current 9)

---

## Phase B Roadmap: GDScript/Godot-Specific Features

*Derived from exhaustive analysis of the GDScript parser in the Godot engine source code (`modules/gdscript/`) and official Godot 4.x documentation.*

Phase A covers generic language features (methods, builtins, imports, types). Phase B adds **GDScript- and Godot-specific features** that have no equivalent in a2py. These are organized by priority and grouped into sub-plans.

### B1: Godot Annotations System (Highest Priority)

GDScript has 30+ annotations that affect code generation. AutoLang needs a way to express these, likely via `#[...]` attributes.

**Annotations to support:**

| Category | Annotations | GDScript Output |
|---|---|---|
| **Script** | `@tool`, `@icon(path)`, `@static_unload` | Top-of-file annotations |
| **Variable** | `@onready` | `@onready var x = $Node` |
| **Export (basic)** | `@export` | `@export var x = 5` |
| **Export (typed)** | `@export_range`, `@export_enum`, `@export_file`, `@export_dir`, `@export_multiline`, `@export_placeholder`, `@export_color_no_alpha`, `@export_node_path`, `@export_flags*`, `@export_exp_easing`, `@export_storage`, `@export_custom`, `@export_tool_button` | Various `@export_*` forms |
| **Export grouping** | `@export_category`, `@export_group`, `@export_subgroup` | Inspector organization |
| **Networking** | `@rpc(mode, sync, transfer, channel)` | `@rpc` annotation |
| **Warnings** | `@warning_ignore`, `@warning_ignore_start`, `@warning_ignore_restore` | Warning suppression |
| **Abstract** | `@abstract` | Abstract class/method marking |

**AutoLang mapping strategy**: Use `#[gd_export]`, `#[gd_onready]`, `#[gd_tool]`, `#[gd_rpc("authority")]`, etc. as attributes, then transpile to GDScript `@` annotations.

### B2: Signal System

GDScript signals are a core inter-node communication mechanism.

```gdscript
# GDScript
signal health_changed(old_value: int, new_value: int)
signal died

# Emitting
health_changed.emit(old_hp, new_hp)

# Connecting
button.pressed.connect(_on_button_pressed)
```

**AutoLang mapping**: `signal` declarations via a new statement type or `#[gd_signal]` attribute. `.emit()` and `.connect()` method mapping.

### B3: Property Setters/Getters

GDScript 4.x uses inline property syntax:

```gdscript
var health: int = 100:
    get:
        return health
    set(value):
        health = value
```

Also supports referencing external functions:
```gdscript
var my_prop: get = get_my_prop, set = set_my_prop
```

**AutoLang mapping**: Map `#[gd_prop(get_fn, set_fn)]` or detect setter/getter patterns from AutoLang's `mut fn` syntax.

### B4: Class System Enhancements

**Inner classes:**
```gdscript
class InnerClass:
    var x = 10
```

**Inheritance with `super`:**
```gdscript
extends BaseClass

func _ready():
    super()  # Call parent _ready
    # ... custom code
```

**Class name and extends customization:**
- Currently hardcoded `extends Node` — should be configurable
- `class_name` should be optional (not all scripts need global registration)

**Static variables:**
```gdscript
static var count: int = 0
static func _static_init():
    count = 0
```

### B5: Typed Collections

GDScript 4.x supports typed arrays and dictionaries:

```gdscript
var scores: Array[int] = [10, 20, 30]
var names: Array[String] = ["Alice", "Bob"]
var mapping: Dictionary[String, int] = {"a": 1, "b": 2}
```

Also packed arrays: `PackedByteArray`, `PackedInt32Array`, `PackedStringArray`, etc.

**AutoLang mapping**: Map `List<int>` → `Array[int]`, `Map<str, int>` → `Dictionary[String, int]`. For packed arrays, could use `#[packed]` annotation or specific types.

### B6: Godot Built-in Types

GDScript has rich built-in types that Python/GDScript share natively:

| Type | Example | Notes |
|---|---|---|
| `Vector2`, `Vector2i` | `Vector2(1.0, 2.0)` | 2D coordinates |
| `Vector3`, `Vector3i` | `Vector3(1.0, 0.0, 0.0)` | 3D coordinates |
| `Color` | `Color(1, 0, 0, 1)` | RGBA color |
| `Rect2`, `Rect2i` | `Rect2(Vector2.ZERO, Vector2(100, 100))` | 2D rectangles |
| `Transform2D`, `Transform3D` | — | Spatial transforms |
| `Quaternion` | — | Rotation representation |
| `NodePath` | `^"Sprite2D/Label"` | Pre-parsed node path |
| `StringName` | `&"signal_name"` | Fast-comparison string |
| `RID` | — | Resource ID |
| `Callable` | `func(x): return x` | Function reference |
| `Signal` | — | Signal reference |

**AutoLang mapping**: These are pass-through — AutoLang code using these types should emit them directly. StringName/NodePath literals need special syntax handling.

### B7: Node Access Syntax ($ and %)

```gdscript
var sprite = $Sprite2D           # get_node("Sprite2D")
var label = %UniqueLabel         # get_node("%UniqueLabel")
var child = $Parent/Child        # get_node("Parent/Child")
```

**AutoLang mapping**: Map from AutoLang's `get_node()` calls or a new `#[gd_node]` syntax to `$`/`%` shorthand.

### B8: Special Constants and Keywords

| Constant | Value | Notes |
|---|---|---|
| `PI` | 3.14159... | Circle constant |
| `TAU` | 6.28318... | Full circle constant |
| `INF` | `inf` | Positive infinity |
| `NAN` | `nan` | Not a number |
| `assert(cond, msg)` | keyword | Debug assertion |
| `breakpoint` | keyword | Editor breakpoint |
| `preload(path)` | keyword | Compile-time load |
| `load(path)` | function | Runtime load |

### B9: Enhanced Match Patterns

GDScript supports rich pattern types beyond simple equality:

```gdscript
match value:
    [var x, _, "test"]:       # Array pattern with binding
        print(x)
    {"name": "Alice", ..}:    # Dictionary pattern (open-ended)
        print("Found Alice")
    [42, ..]:                 # Rest pattern (open-ended array)
        print("Starts with 42")
    var bound:                # Binding pattern
        print("Got: ", bound)
    1, 2, 3:                  # Multiple patterns
        print("1-3")
    [var x, var y] when y > x:  # Guard with when
        print("y > x")
```

**Pattern types from Godot parser**: PT_LITERAL, PT_EXPRESSION, PT_BIND, PT_ARRAY, PT_DICTIONARY, PT_REST, PT_WILDCARD

### B10: Lambda `.call()` Requirement

GDScript lambdas create `Callable` objects that must be invoked with `.call()`:

```gdscript
var add = func (a, b): return a + b
add.call(1, 2)  # NOT add(1, 2)
```

**Impact on a2gd**: When emitting lambda invocations, need to use `.call()` instead of direct `()`.

### Feature Coverage Matrix

| Feature | Phase A (current plan) | Phase B (future) |
|---|---|---|
| Method mapping (String/List/Dict) | ✅ Task 1-2 | — |
| Builtin function mapping | ✅ Task 3 | — |
| Import/preload system | ✅ Task 4 | Enhanced preload |
| Type tracking | ✅ Task 5 | Typed arrays/dicts |
| Generics | ✅ Task 6 | — |
| Async/await | ✅ Task 7 | — |
| Spec → class stubs | ✅ Task 8 | — |
| Comprehensive tests | ✅ Task 9 | — |
| Annotations (@export etc.) | — | B1 |
| Signal system | — | B2 |
| Property setters/getters | — | B3 |
| Inner classes + inheritance + super | — | B4 |
| Typed collections | — | B5 |
| Godot built-in types (Vector2, etc.) | — | B6 |
| $/% node access syntax | — | B7 |
| PI/TAU/INF/NAN/assert/breakpoint | — | B8 |
| Enhanced match patterns | — | B9 |
| Lambda `.call()` requirement | — | B10 |
