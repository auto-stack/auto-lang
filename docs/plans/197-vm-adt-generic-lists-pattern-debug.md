# Plan 197: VM Enum/Data, Generic Lists, Pattern Destructuring & Debug Formatting

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Date:** 2026-04-20
**Status:** Approved
**Goal:** Add ten runtime features to the Auto VM in dependency order: string equality via `==`, chained method call type resolution, struct-as-function-param passing, `str.slice()` method, struct equality, `Option<T>` + `Some()`/`None`, struct debug formatting, enum variants with data, List<UserType>, and pattern destructuring in `is`-expressions.
**Architecture:** Enum variants reuse the existing `GenericInstanceData` heap object system with a `mono_name` encoding the variant (`"Atom.Int"`). Pattern matching compiles to tag-check + field-extraction using existing opcodes. Debug formatting extends `TO_STR` to handle heap objects. String equality is fixed by interning literals in codegen + content-aware EQ in the engine. Method chaining is fixed by consulting `fn_return_types` in `infer_object_type()`. Struct param passing is fixed by correcting LOAD_LOCAL offset for heap ID arguments in CALL frames. `Option<T>` is represented as a built-in enum `Option` with variants `Some(T)` and `None`. Struct equality is a field-by-field comparison in EQ when both operands are heap objects.
**Tech Stack:** Rust, AutoLang crate (`auto-lang`), existing VM infrastructure (heap objects, generic registry, opcodes).

---

## Problem

The Auto VM lacks ten runtime features needed for realistic programs:

1. **String `==` compares tagged IDs, not content** — Identical string literals get different negative tag IDs because `Expr::Str` bypasses `add_string()` interning. `EQ` opcode compares raw i32 values, so `"http" == "http"` returns false when the two occurrences get different tags.
2. **Chained method calls fail type resolution** — `infer_object_type()` returns `NestedObject` for all `Expr::Call` (codegen.rs:5386), causing method lookup to generate `Unknown_display` instead of `ApiError.display`. The `fn_return_types` map has the correct return type but is never consulted.
3. **Struct instances can't be passed as function parameters** — Passing a struct (heap object ID ≥ 4000000) to a regular function fails. The CALL frame sets up BP correctly and the argument IS on the stack (debug shows `Stack[1] = 4000000`), but `LOAD_LOCAL param 0` reads 0 from the wrong offset. Methods (`self`) work because they use a different parameter resolution path.
4. **No `str.slice()` method** — `substr(start, end)` exists but `slice(start)` (one-arg, to end of string) and `slice(start, end)` (two-arg alias) don't. Examples need the one-arg form.
5. **No struct equality** — `==` on struct instances compares heap IDs (4000000 vs 4000001), not field contents. Two structurally equal instances are always `!=`.
6. **No `Option<T>` / `?T` / `Some()` / `None`** — The `?str` nullable type syntax and `Some(value)`/`None` constructors have no runtime representation. Pattern matching `is x { Some(v) -> ..., None -> ... }` is hardcoded for limited built-in cases only.
7. **No debug output for struct types** — `TO_STR` only handles i32 and tagged strings. Struct instances (heap objects) print as garbage integers.
8. **No enum variants with data** — Only C-style scalar enums (`enum Color { Red = 1 }`) work. Tuple variants like `Atom.Int(42)` have no runtime representation.
9. **No `List<UserType>`** — `GET_ELEM` only handles `List<int>`, `List<str>`, `List<bool>`. User-defined types in lists are unsupported.
10. **No pattern destructuring in `is`** — `is expr { Variant(x) -> body }` cannot bind variables from matched values.

## Current State

| Feature | Parser | Codegen | Engine |
|---------|--------|---------|--------|
| String `==` | OK | `Expr::Str` bypasses `add_string()` interning (codegen.rs:3000-3006) | `EQ` compares raw i32 (engine.rs:3150-3157) |
| Method chaining | OK | `infer_object_type()` returns `NestedObject` for all calls (codegen.rs:5386) | Runtime works if bytecode is correct |
| Struct-as-param | OK | `Arg 0: smart param passing` emits LOAD_LOCAL with wrong offset (codegen.rs) | `LOAD_LOCAL param 0` reads 0 from wrong BP offset (engine.rs) |
| `str.slice()` | OK | Not registered as native method | Only `substr(start, end)` exists in BIGVM_NATIVES |
| Struct equality | OK | OK | `EQ` compares heap IDs, not field contents (engine.rs:3150) |
| `Option<T>` / `?T` | Accepts `?T` syntax | No codegen for `Some()`/`None` construction | No runtime representation |
| Struct debug `to_str` | N/A | N/A | `TO_STR` only handles i32 + tagged strings (engine.rs:1268) |
| Enum data variants | Accepts `enum Foo { Bar(int) }` (enums.rs:53) | Emits `CONST_I32` with discriminant only (codegen.rs:1463) | No runtime representation for data payloads |
| `List<UserType>` | Accepts `List<T>` syntax | Emits typed CREATE_LIST opcodes | GET_ELEM only handles int/str/bool (engine.rs:1943) |
| Pattern destructuring | Accepts `is x { Foo(y) -> }` | Hardcoded for Option/Result only (codegen.rs:2138) | No variable binding from patterns |

---

## Design

### Phase 0a: String Equality via `==`

**Root cause (two bugs):**

1. **No interning in `Expr::Str`** — `codegen.rs:3000-3006` pushes string bytes directly to `self.strings` without checking for duplicates. So `"http"` in a constructor and `"http"` in an `if` condition get different indices (different negative tags). The `add_string()` helper at line 5914-5926 does check for duplicates, but `Expr::Str` bypasses it.

2. **EQ opcode is content-blind** — `engine.rs:3150-3157` does `if a == b` on raw i32 values. When both operands are negative (tagged strings), it should dereference them and compare the actual string bytes.

**Fix (two-part):**

- **Part 1 (codegen):** Change `Expr::Str` handler to call `self.add_string(s)` instead of pushing directly. This ensures identical string literals share the same tag. This alone fixes 90% of cases.

- **Part 2 (engine):** Enhance `EQ` and `NE` to detect when both operands are negative (tagged strings) and compare the string content from the string pool. This handles runtime-created strings and strings from different compilation units.

### Phase 0b: Chained Method Call Type Resolution

**Root cause:** `infer_object_type()` at `codegen.rs:5386` lumps `Expr::Call` with other complex expressions and returns `ObjectType::NestedObject`. When `ApiError.http("timeout").display()` is compiled, the outer call tries to resolve `.display()` on the inner call's return type, gets `NestedObject`, maps it to `"Unknown"`, and generates `Unknown_display`.

**Fix:** In `infer_object_type()`, add a special case for `Expr::Call` that:
1. Extracts the called function name from `call.name`
2. Looks up `fn_return_types` to get the actual return type
3. Maps the `Type` to the correct `ObjectType`

This is a pure codegen fix — the runtime already handles chained calls correctly when the bytecode is correct.

### Phase 0c: Struct-as-Function-Parameter Passing

**Root cause:** When a struct instance (heap ID ≥ 4000000) is passed as a function argument, the CALL frame setup puts the argument on the stack correctly, but `LOAD_LOCAL` reads the wrong offset.

Debug trace shows:
```
CALL: Stack depth before = 4
CALL: Stack[1] = 4000000        ← argument IS on stack
CALL: BP = 5
LOAD_LOCAL param 0: BP-2 = 3 = 0  ← reads 0, not 4000000
```

The `LOAD_LOCAL` offset calculation (`BP - n_args + param_index`) doesn't account for the CALL frame layout correctly. Methods work because they use a different code path for `self` parameter resolution.

**Fix:** Investigate the CALL frame layout and LOAD_LOCAL offset calculation in `engine.rs`. The parameter should be at `BP - 2` for a 1-arg function, but the value at that offset is 0 instead of the heap ID. Likely the CALL pushes extra words (return address, old BP) between the arguments and the new frame, shifting where parameters end up relative to the new BP.

### Phase 0d: `str.slice()` Method

**Root cause:** Only `substr(start, end)` is registered in BIGVM_NATIVES. The one-argument `slice(start)` form (return everything from `start` to end of string) is not registered. Many Rust-derived examples use `slice(1)` as the idiomatic "skip first char" pattern.

**Fix:** Register `str.slice` as a BIGVM_NATIVE that accepts 1 or 2 arguments. With 1 arg: equivalent to `substr(start, len)`. With 2 args: equivalent to `substr(start, end)`. Can be implemented as a thin wrapper around the existing `substr` logic in the engine.

### Phase 0e: Struct Equality

**Root cause:** `EQ` opcode (engine.rs:3150) compares two raw i32 values. When both are ≥ 4000000 (heap object IDs), it compares the IDs, not the struct contents. Two structurally identical instances created at different times have different heap IDs, so `==` returns false.

**Fix:** Enhance the `EQ` handler (already being modified in Task 2 for string content comparison) to also handle heap objects:
1. If both operands are ≥ 4000000 → look up both `GenericInstanceData` from the heap
2. Compare `mono_name` (same type?)
3. Compare each field value recursively (handling nested structs, strings, ints)
4. Return true if all fields match

This gives structural equality (like Rust's `PartialEq` derived impl) without requiring user annotations.

### Phase 0f: `Option<T>` / `?T` / `Some()` / `None`

**Root cause:** The parser accepts `?str` syntax and `Some(x)` / `None` in pattern position, but there is no runtime representation. The codegen doesn't emit construction opcodes for `Some(value)`, and the engine has no way to distinguish a "present" value from "absent".

**Design:** `Option<T>` is a built-in enum with two variants:
- `Some(T)` — stored as a heap object with `mono_name: "Option.Some"` and field `_0: T`
- `None` — stored as a special sentinel value (e.g., i32 = 0, or a dedicated `Nil` tag)

Construction:
- `Some(expr)` compiles to: evaluate expr → NEW_INSTANCE("Option.Some") → CONSTRUCT_INSTANCE(1 field)
- `None` compiles to: push the nil sentinel

Pattern matching:
- `is x { Some(v) -> body, None -> body2 }` checks if x is nil sentinel → if not, extract `_0` field

This reuses the enum variant infrastructure from Phase 2 (Tasks 8-10) plus pattern destructuring from Phase 4 (Task 12). It's placed after those phases.

### Phase 1: Default `to_str` for Struct Types

`TO_STR` (engine.rs:1268) checks if value is a tagged string (negative i32). Otherwise converts the i32 to its decimal string. Heap object IDs (>= 4000000) print as the raw integer.

Extend `TO_STR` to detect heap objects and format them as `TypeName { field: value, ... }`.

Field names come from `GenericInstanceData` — add a `field_names: Vec<String>` field alongside the existing `fields: Vec<Value>`. Populate during `CONSTRUCT_INSTANCE` from the class template.

No new opcodes needed.

### Phase 2: Enum Variants with Data

**Representation: Tagged heap objects.** Each enum variant with data is stored as a heap object using the existing `GenericInstanceData`, with:
- `mono_name` encoding the variant: `"Atom.Int"`, `"ContentBlock.Text"`
- Payload fields as `_0`, `_1`, ... for tuple variants or named fields for struct variants

Reuses `NEW_INSTANCE` + `CONSTRUCT_INSTANCE` — no new opcodes. Register each variant as a separate "type" in `GenericRegistry`.

`Atom.Int(42)` compiles to: push 42 → NEW_INSTANCE("Atom.Int") → CONSTRUCT_INSTANCE(1 field).

### Phase 3: `List<T>` with User-Defined Types

User-defined type instances are heap object IDs (i32 >= 4000000). A `List<UserType>` is just a list of i32 values.

New opcodes: `CREATE_LIST_REF` (0xA6), `LIST_PUSH_REF` (0xA7), `LIST_GET_REF` (0xA8). Storage is `Vec<i32>`. No per-type opcodes — all user types are just i32 heap refs.

### Phase 4: Pattern Destructuring in `is`-expressions

Compile `is expr { Variant(x) -> body }` to:
1. Load match expression
2. Get `__variant` tag field → compare string
3. On match: extract each binding with `GET_GENERIC_FIELD` → `STORE_LOCAL`
4. Compile branch body → jump to end

No new opcodes — reuses `GET_GENERIC_FIELD` + `EQ` + conditional jumps.

---

## Implementation Order

| Phase | Feature | Depends On | Complexity |
|-------|---------|------------|------------|
| 0a | String `==` (interning + content-aware EQ) | None | Small (~30 lines) |
| 0b | Method chaining type resolution | None | Small (~25 lines) |
| 0c | Struct-as-function-param passing | None | Medium (debug CALL frame layout) |
| 0d | `str.slice()` native method | None | Small (~20 lines) |
| 0e | Struct equality (field-by-field EQ) | Phase 0a (content-aware EQ) | Small (~30 lines) |
| 1 | Debug `to_str` for structs | None | Small (~50 lines) |
| 2 | Enum variants with data | Phase 1 (for debugging) | Medium |
| 3 | `List<UserType>` | Phase 2 (lists of enum variants) | Small (3 opcodes) |
| 4 | Pattern destructuring in `is` | Phase 2 (variant objects) | Medium |
| 5 | `Option<T>` / `Some()` / `None` | Phase 2 (enum variants) + Phase 4 (pattern destructuring) | Medium |

---

## Tasks

### Task 1: Fix string literal interning in codegen

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs:3000-3006` (Expr::Str handler)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/025_string_equality.at`:

```auto
fn main() {
    let a = "hello"
    let b = "hello"
    assert_eq(a, b)

    let c = "world"
    assert(a != c)

    // String in struct field vs literal
    let kind = "http"
    assert(kind == "http")

    print("string_equality: passed")
}
```

**Step 2: Run test to verify it fails**

Run: `cd d:/autostack/auto-lang && target/debug/auto test/vm/10_types/025_string_equality.at`
Expected: FAIL — `assert_eq(a, b)` fails because two `"hello"` literals get different tags.

**Step 3: Fix Expr::Str to use add_string()**

In `codegen.rs`, replace the `Expr::Str` handler (lines 3000-3006):

```rust
Expr::Str(s) => {
    let idx = self.add_string(s);
    self.emit(OpCode::LOAD_STR);
    self.code.extend_from_slice(&idx.to_le_bytes());
    self.last_expr_type = ObjectType::String;
}
```

This ensures identical string literals get the same tag via the existing `add_string()` interning logic at line 5914-5926.

**Step 4: Run test to verify it passes**

Expected: `string_equality: passed`

**Step 5: Rebuild auto binary and verify examples**

Run: `cargo build --bin auto && cd d:/autostack/auto-code-rs && auto crates/ac-examples/src/10_api_error_enum/main.at`
Expected: All assertions pass (this example uses string comparison in `display()` method).

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs crates/auto-lang/test/vm/10_types/025_string_equality.at
git commit -m "fix(codegen): intern string literals so == compares identical tags"
```

---

### Task 2: Content-aware string comparison in EQ/NE

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:3150-3163` (EQ and NE handlers)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/026_string_eq_runtime.at`:

```auto
fn greet() str {
    "hello"
}

fn main() {
    // Same literal from same function — already works after Task 1
    assert("hello" == "hello")

    // String from function vs literal — tests content-aware EQ
    let a = greet()
    assert(a == "hello")

    // NE on strings
    assert(a != "world")

    print("string_eq_runtime: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: Fails at `assert(a == "hello")` — `greet()` returns a string that may have a different tag than the literal `"hello"` in main.

**Step 3: Enhance EQ to handle tagged strings**

In `engine.rs`, replace the EQ handler (lines 3150-3157):

```rust
OpCode::EQ => {
    let b = task.ram.pop_i32();
    let a = task.ram.pop_i32();
    let result = if a == b {
        true
    } else if a < 0 && b < 0 {
        // Both are tagged strings — compare content
        let strings = self.strings.read().unwrap();
        let idx_a = (-a - 1) as usize;
        let idx_b = (-b - 1) as usize;
        if idx_a < strings.len() && idx_b < strings.len() {
            strings[idx_a] == strings[idx_b]
        } else {
            false
        }
    } else {
        false
    };
    task.ram.push_i32(if result { -2147483648 } else { -2147483647 });
}
```

Apply the same pattern to `NE` (lines 3158-3163).

**Step 4: Run test to verify it passes**

Expected: `string_eq_runtime: passed`

**Step 5: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs crates/auto-lang/test/vm/10_types/026_string_eq_runtime.at
git commit -m "fix(engine): content-aware string comparison in EQ/NE opcodes"
```

---

### Task 3: Fix method chaining — infer return type from fn_return_types

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs:5386` (infer_object_type Expr::Call case)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/027_method_chaining.at`:

```auto
type Point {
    x int
    y int
}

ext Point {
    static fn origin() Point {
        Point { x: 0, y: 0 }
    }

    fn to_str(self) str {
        f"(${self.x}, ${self.y})"
    }
}

fn main() {
    let p = Point.origin()
    let s = p.to_str()
    assert(s.contains("0"))

    // Chained: origin().to_str() without intermediate let
    let s2 = Point.origin().to_str()
    assert(s2.contains("0"))

    print("method_chaining: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: Fails at `Point.origin().to_str()` — generates `Unknown_to_str` instead of `Point.to_str`.

**Step 3: Enhance infer_object_type for Expr::Call**

In `codegen.rs`, replace line 5386:

```rust
Expr::Object(_) | Expr::Node(_) | Expr::Call(_) | Expr::Bina(_, _, _) | Expr::If(_) | Expr::Lambda(_) | Expr::Closure(_) | Expr::Pair(_) => ObjectType::NestedObject,
```

With:

```rust
Expr::Call(call) => {
    // Try to resolve return type from fn_return_types
    if let Expr::Dot(obj, method) = call.name.as_ref() {
        let fn_name = format!("{}.{}", self.expr_to_name(obj.as_ref()), method.as_ref());
        if let Some(ret_ty) = self.fn_return_types.get(&fn_name) {
            self.type_to_object_type(ret_ty)
        } else {
            ObjectType::NestedObject
        }
    } else if let Expr::Ident(name) = call.name.as_ref() {
        if let Some(ret_ty) = self.fn_return_types.get(name.as_ref()) {
            self.type_to_object_type(ret_ty)
        } else {
            ObjectType::NestedObject
        }
    } else {
        ObjectType::NestedObject
    }
}
Expr::Object(_) | Expr::Node(_) | Expr::Bina(_, _, _) | Expr::If(_) | Expr::Lambda(_) | Expr::Closure(_) | Expr::Pair(_) => ObjectType::NestedObject,
```

Add helper methods:

```rust
fn expr_to_name(&self, expr: &Expr) -> String {
    match expr {
        Expr::Ident(name) => name.to_string(),
        Expr::Dot(obj, method) => format!("{}.{}", self.expr_to_name(obj), method),
        _ => "Unknown".to_string(),
    }
}

fn type_to_object_type(&self, ty: &Type) -> ObjectType {
    match ty {
        Type::Str(_) | Type::String | Type::CStr | Type::StrSlice => ObjectType::String,
        Type::Char => ObjectType::Char,
        Type::Int | Type::I64 => ObjectType::Int,
        Type::Uint | Type::U64 | Type::USize => ObjectType::Uint,
        Type::Byte => ObjectType::Byte,
        Type::Float => ObjectType::Float,
        Type::Double => ObjectType::Double,
        Type::Bool => ObjectType::Bool,
        Type::Array(_) | Type::RuntimeArray(_) => ObjectType::Array,
        _ => ObjectType::NestedObject,
    }
}
```

**Step 4: Run test to verify it passes**

Expected: `method_chaining: passed`

**Step 5: Rebuild and verify examples**

Run: `cargo build --bin auto && cd d:/autostack/auto-code-rs && auto crates/ac-examples/src/10_api_error_enum/main.at`
Expected: Passes (examples can now use chained method calls).

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs crates/auto-lang/test/vm/10_types/027_method_chaining.at
git commit -m "fix(codegen): resolve method chaining via fn_return_types in infer_object_type"
```

---

### Task 4: Fix struct-as-function-parameter passing

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs` (CALL handler + LOAD_LOCAL parameter resolution)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/028_struct_param.at`:

```auto
type Foo { kind int, name str }

fn get_kind(event Foo) int {
    return event.kind
}

fn main() {
    let m = Foo { kind: 42, name: "test" }
    let k = get_kind(m)
    assert_eq(k, 42)
    print("struct_param: passed")
}
```

**Step 2: Run test to verify it fails**

Run: `cd d:/autostack/auto-lang && target/debug/auto test/vm/10_types/028_struct_param.at`
Expected: FAIL — `RuntimeError("Invalid instance ID: 0")`

**Step 3: Debug and fix LOAD_LOCAL offset**

Add debug logging to the CALL handler in `engine.rs` to trace:
1. Stack layout before and after frame setup (all positions, not just [0]-[2])
2. Where arguments end up relative to the new BP
3. What LOAD_LOCAL computes for parameter offsets

The fix will be in the LOAD_LOCAL offset calculation or in how CALL sets up the frame. The argument (heap ID 4000000) IS on the stack at the correct position, but LOAD_LOCAL reads from a different position.

Hypothesis: CALL may push return address and/or old BP on top of the arguments, shifting the effective parameter positions. If so, LOAD_LOCAL should use `BP - n_args - frame_overhead + param_index` instead of the current formula.

**Step 4: Run test to verify it passes**

Expected: `struct_param: passed`

**Step 5: Verify methods still work**

Run existing examples to ensure method calls (`self` parameter) are not affected:
```bash
auto crates/ac-examples/src/08_usage_struct/main.at
auto crates/ac-examples/src/10_api_error_enum/main.at
```

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs crates/auto-lang/test/vm/10_types/028_struct_param.at
git commit -m "fix(engine): correct LOAD_LOCAL offset for struct function parameters"
```

---

### Task 5: Add `str.slice()` native method

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs` (add slice handler in native dispatch)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/029_str_slice.at`:

```auto
fn main() {
    let s = "hello world"
    let suffix = s.slice(6)
    assert_eq(suffix, "world")

    let prefix = s.slice(0, 5)
    assert_eq(prefix, "hello")

    let skipped = s.slice(1)
    assert_eq(skipped, "ello world")

    print("str_slice: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL — `str.slice` is not a registered native.

**Step 3: Register `str.slice` as a BIGVM_NATIVE**

In the native method registration (wherever `str.substr`, `str.contains` etc. are registered), add `str.slice` that:
- With 1 arg: calls `substr(start, string.len())`
- With 2 args: calls `substr(start, end)`

Alternatively, add it as an alias in the codegen method resolution: when `method == "slice"`, emit the same bytecode as `substr`.

**Step 4: Run test to verify it passes**

Expected: `str_slice: passed`

**Step 5: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs crates/auto-lang/test/vm/10_types/029_str_slice.at
git commit -m "feat(engine): add str.slice() native method (1-arg and 2-arg forms)"
```

---

### Task 6: Struct equality — field-by-field comparison in EQ

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs` (extend EQ handler from Task 2)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/030_struct_equality.at`:

```auto
type Point {
    x int
    y int
}

fn main() {
    let p1 = Point { x: 3, y: 4 }
    let p2 = Point { x: 3, y: 4 }
    assert(p1 == p2)

    let p3 = Point { x: 1, y: 2 }
    assert(p1 != p3)

    // Same struct, same variable
    assert(p1 == p1)

    print("struct_equality: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL — `assert(p1 == p2)` fails because EQ compares heap IDs (4000000 != 4000001).

**Step 3: Extend EQ to handle heap objects**

In the EQ handler (already modified in Task 2 for strings), add a third case:

```rust
OpCode::EQ => {
    let b = task.ram.pop_i32();
    let a = task.ram.pop_i32();
    let result = if a == b {
        true
    } else if a < 0 && b < 0 {
        // Tagged strings — compare content (from Task 2)
        // ...
    } else if a >= 4000000 && b >= 4000000 {
        // Heap objects — structural equality
        struct_eq(&self.instances, a, b)
    } else {
        false
    };
    task.ram.push_i32(if result { -2147483648 } else { -2147483647 });
}
```

Add helper `struct_eq(instances, a, b)`:
1. Look up both `GenericInstanceData` from the heap
2. Compare `mono_name` — if different types, return false
3. Compare each field pairwise (recursing for nested structs, using content comparison for strings)

**Step 4: Run test to verify it passes**

Expected: `struct_equality: passed`

**Step 5: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs crates/auto-lang/test/vm/10_types/030_struct_equality.at
git commit -m "feat(engine): structural equality for struct instances in EQ opcode"
```

---

### Task 7: Add `field_names` to `GenericInstanceData`

**Files:**
- Modify: `crates/auto-lang/src/vm/generic_registry.rs:500-524`

**Step 1: Add `field_names` field to the struct**

In `generic_registry.rs`, update `GenericInstanceData`:

```rust
#[derive(Debug)]
pub struct GenericInstanceData {
    pub mono_name: String,
    pub fields: Vec<Value>,
    pub field_names: Vec<String>,
}
```

Update `new()` and add `new_with_names()`:

```rust
pub fn new(mono_name: String, fields: Vec<Value>) -> Self {
    let field_names = vec!["_unknown".to_string(); fields.len()];
    Self { mono_name, fields, field_names }
}

pub fn new_with_names(mono_name: String, fields: Vec<Value>, field_names: Vec<String>) -> Self {
    Self { mono_name, fields, field_names }
}
```

**Step 2: Build and verify compilation**

Run: `cargo build -p auto-lang 2>&1 | tail -5`
Expected: Possible warnings about unused `field_names`, no errors.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/vm/generic_registry.rs
git commit -m "refactor(vm): add field_names to GenericInstanceData"
```

---

### Task 8: Populate `field_names` during `CONSTRUCT_INSTANCE`

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:1517-1615` (CONSTRUCT_INSTANCE handler)
- Modify: `crates/auto-lang/src/vm/generic_registry.rs` (expose field names from template)

**Step 1: Expose field names from ClassType/template**

In `generic_registry.rs`, add a convenience method to `ClassType` or `GenericTemplate`:

```rust
pub fn field_names(&self) -> Vec<String> {
    self.template.fields.iter().map(|f| f.name.clone()).collect()
}
```

**Step 2: Store field names in CONSTRUCT_INSTANCE**

In `engine.rs`, after the line `instance.fields = field_values;` (~line 1584), add:

```rust
let field_names = self.generic_registry
    .get_type(&instance.mono_name)
    .map(|ct| ct.template.fields.iter().map(|f| f.name.clone()).collect())
    .unwrap_or_else(|| vec!["_unknown".to_string(); field_count]);
instance.field_names = field_names;
```

**Step 3: Build and verify**

Run: `cargo build -p auto-lang 2>&1 | tail -5`
Expected: Clean build.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs crates/auto-lang/src/vm/generic_registry.rs
git commit -m "feat(vm): populate field_names in CONSTRUCT_INSTANCE"
```

---

### Task 9: Extend `TO_STR` to format struct instances

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs:1268-1287` (TO_STR handler)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/020_struct_to_str.at`:

```auto
type Point {
    x int
    y int
}

fn main() {
    let p = Point { x: 3, y: 4 }
    let s = p.to(str)
    print(s)
    assert(s.contains("Point"))
    assert(s.contains("3"))
    print("struct_to_str: passed")
}
```

**Step 2: Run test to verify it fails**

Run: `cd d:/autostack/auto-lang && target/debug/auto test/vm/10_types/020_struct_to_str.at`
Expected: Fails — TO_STR outputs raw integer for heap objects.

**Step 3: Implement TO_STR for heap objects**

In `engine.rs`, replace the `TO_STR` handler (lines 1268-1287) with logic that:
1. If value < 0: already a tagged string, push back.
2. If value >= 4000000: heap object — downcast to `GenericInstanceData`, format as `TypeName { field: val, ... }`, push tagged string.
3. Otherwise: plain integer — convert to decimal string.

Add a helper `format_value(val: &Value, strings: &RwLock<Vec<Vec<u8>>>) -> String` to format individual field values (Int → decimal, Str → quoted string, VmRef → `<ref id>`, Nil → `"nil"`).

**Step 4: Run test to verify it passes**

Expected: `struct_to_str: passed`

**Step 5: Rebuild auto binary and verify 08_usage_struct**

Run: `cargo build --bin auto && cd d:/autostack/auto-code-rs && auto crates/ac-examples/src/08_usage_struct/main.at`
Expected: All assertions pass.

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs crates/auto-lang/test/vm/10_types/020_struct_to_str.at
git commit -m "feat(vm): TO_STR formats struct instances as Type { field: val }"
```

---

### Task 10: Register enum variants in `GenericRegistry`

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs:1463-1470` (EnumDecl handler)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/021_enum_variant_data.at`:

```auto
enum Atom {
    Int int
    Str str
}

fn main() {
    let a = Atom.Int(42)
    print(f"variant: ${a}")
    print("enum_variant_data: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: Fails — `Atom.Int(42)` has no codegen.

**Step 3: Register enum variants as generic types**

In `codegen.rs`, in the `EnumDecl` handler (~line 1463), after registering scalar variants, add:

```rust
for item in &enum_decl.items {
    if item.payload_type.is_some() {
        let variant_mono = format!("{}.{}", enum_decl.name, item.name);
        let payload = item.payload_type.as_ref().unwrap();
        let fields = vec![GenericField {
            name: "_0".to_string(),
            field_type: payload.clone(),
            default_value: None,
        }];
        self.generic_registry.register_template(
            &variant_mono,
            GenericTemplate::new(&variant_mono, fields),
        );
    }
}
```

**Step 4: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs
git commit -m "feat(codegen): register enum data variants in GenericRegistry"
```

---

### Task 11: Codegen for enum variant construction

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (Expr::Call handler, after is_generic_constructor check)

**Step 1: Add variant construction codegen**

In the `Expr::Call(call)` handler, after the `is_generic_constructor` check (~line 3920), add an `is_enum_variant` check. When `Type.Variant(args)` matches a registered template `"Type.Variant"`, emit NEW_INSTANCE + CONSTRUCT_INSTANCE with the variant mono_name.

```rust
let is_enum_variant = if let Expr::Dot(obj, method) = call.name.as_ref() {
    if let Expr::Ident(type_name) = obj.as_ref() {
        let variant_mono = format!("{}.{}", type_name.as_ref(), method.as_ref());
        self.generic_registry.has_template(&variant_mono)
    } else { false }
} else { false };

if is_enum_variant { /* emit NEW_INSTANCE + CONSTRUCT_INSTANCE with variant_mono */ }
```

**Step 2: Build and run test from Task 4**

Expected: Prints variant info, passes.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs
git commit -m "feat(codegen): compile enum variant construction (Atom.Int(42))"
```

---

### Task 12: Access payload fields from enum variants

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (field access on variant instances)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/022_enum_field_access.at`:

```auto
enum Atom {
    Int int
    Str str
}

fn main() {
    let a = Atom.Int(42)
    assert_eq(a._0, 42)

    let b = Atom.Str("hello")
    assert_eq(b._0, "hello")

    print("enum_field_access: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: Fails — field access on enum variants not yet supported.

**Step 3: Add field access support**

In the dot-access codegen (~line 3150), when resolving `a._0` on an enum variant variable: detect the variable's type name contains a `.` (indicating enum variant), look up field index from the variant's template in `generic_registry`, emit `GET_GENERIC_FIELD`.

If the variable type is not tracked as a variant, add a fallback: when `field_name.starts_with('_')`, try looking up `"{var_type}.{field_name}"` in generic_registry.

**Step 4: Run test**

Expected: Passes.

**Step 5: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs crates/auto-lang/test/vm/10_types/022_enum_field_access.at
git commit -m "feat(vm): field access on enum variant payloads"
```

---

### Task 13: `List<UserType>` support

**Files:**
- Modify: `crates/auto-lang/src/vm/opcode.rs` (add new opcodes + VALID entries)
- Modify: `crates/auto-lang/src/vm/engine.rs` (implement handlers)
- Modify: `crates/auto-lang/src/vm/codegen.rs` (emit LIST_REF opcodes for user-type arrays)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/023_list_user_type.at`:

```auto
enum Atom {
    Int int
}

fn main() {
    let a = Atom.Int(1)
    let b = Atom.Int(2)
    let list = [a, b]
    let first = list[0]
    assert_eq(first._0, 1)
    let second = list[1]
    assert_eq(second._0, 2)
    print("list_user_type: passed")
}
```

**Step 2: Add new opcodes**

In `opcode.rs`, add:

```rust
CREATE_LIST_REF = 0xA6,
LIST_PUSH_REF = 0xA7,
LIST_GET_REF = 0xA8,
```

Add `0xA6, 0xA7, 0xA8` to the VALID array.

**Step 3: Implement engine handlers**

In `engine.rs`, add handlers. Storage is `Vec<i32>`. `LIST_GET_REF` returns raw i32 — the caller interprets it as a heap object ID.

Extend `GET_ELEM` handler with a fallback for ref lists: when the list doesn't match int/str/bool, try downcasting to `Vec<i32>`.

**Step 4: Update codegen**

When creating an array literal with elements of user-defined type, emit `CREATE_LIST_REF` + `LIST_PUSH_REF`. When accessing elements of user-type arrays, emit `LIST_GET_REF`.

**Step 5: Build and run test**

Expected: Passes.

**Step 6: Commit**

```bash
git add crates/auto-lang/src/vm/opcode.rs crates/auto-lang/src/vm/engine.rs crates/auto-lang/src/vm/codegen.rs
git commit -m "feat(vm): List<UserType> support with heap ref opcodes"
```

---

### Task 14: Pattern destructuring in `is`-expressions

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (extend `is` compilation for enum patterns)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/024_is_enum_destructure.at`:

```auto
enum Atom {
    Int int
    Str str
}

fn main() {
    let a = Atom.Int(42)
    is a {
        Atom.Int(n) -> {
            assert_eq(n, 42)
            print(f"matched Int: $n")
        },
        Atom.Str(s) -> {
            print("wrong branch")
        },
        else -> print("no match")
    }
    print("is_enum_destructure: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: Fails — `is` with enum destructuring not implemented.

**Step 3: Implement destructuring codegen**

For each `Variant(binding1, binding2)` branch in an `is` expression:
1. Emit: load match expression
2. Emit: `GET_GENERIC_FIELD 0` to get the `__variant` tag string
3. Emit: compare with expected variant name string
4. On match: for each binding, emit `GET_GENERIC_FIELD idx` + `STORE_LOCAL`
5. Compile branch body
6. Jump to end

Add a `__variant` string field at index 0 of every enum variant instance during construction (Task 5).

**Step 4: Run test**

Expected: Passes.

**Step 5: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs crates/auto-lang/test/vm/10_types/024_is_enum_destructure.at
git commit -m "feat(codegen): pattern destructuring for enum variants in is-expressions"
```

---

### Task 15: `Option<T>` — built-in Some/None enum

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs` (codegen for `Some()` and `None`)
- Modify: `crates/auto-lang/src/vm/engine.rs` (runtime handling)

**Step 1: Write the test**

Create `crates/auto-lang/test/vm/10_types/031_option_some_none.at`:

```auto
fn maybe_greet(name str) ?str {
    if name == "" {
        return None
    }
    return Some(f"Hello, $name!")
}

fn main() {
    let a = maybe_greet("Auto")
    is a {
        Some(msg) -> assert(msg.contains("Auto")),
        None -> assert(false)
    }

    let b = maybe_greet("")
    is b {
        Some(_) -> assert(false),
        None -> print("got None as expected")
    }

    let c = Some(42)
    is c {
        Some(n) -> assert_eq(n, 42),
        None -> assert(false)
    }

    print("option_some_none: passed")
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL — `Some()` and `None` have no construction codegen.

**Step 3: Register Option as a built-in enum**

In codegen initialization, pre-register `Option.Some` and `Option.None` in `GenericRegistry`:
- `Option.Some` has 1 field: `_0` (the wrapped value)
- `Option.None` has 0 fields

**Step 4: Codegen for Some() and None**

- `Some(expr)` → compile expr → `NEW_INSTANCE("Option.Some")` → `CONSTRUCT_INSTANCE(1)`
- `None` → push a nil sentinel value (use 0, or create a dedicated `Option.None` heap object with 0 fields)

**Step 5: Pattern matching for Some/None**

Reuse the pattern destructuring from Task 12:
- `Some(v)` → check `mono_name == "Option.Some"`, extract `_0` field → bind `v`
- `None` → check for nil sentinel or `mono_name == "Option.None"`

**Step 6: Run test to verify it passes**

Expected: `option_some_none: passed`

**Step 7: Commit**

```bash
git add crates/auto-lang/src/vm/codegen.rs crates/auto-lang/src/vm/engine.rs crates/auto-lang/test/vm/10_types/031_option_some_none.at
git commit -m "feat(vm): Option<T> with Some()/None construction and pattern matching"
```

---

### Task 16: Integration — restore all simplified examples to their original form

All VM features are now implemented. This task restores each simplified example to its original idiomatic Auto code, using the features added in Tasks 1-13.

**Files:**
- Modify: `crates/ac-examples/src/07_glob_match/main.at`
- Modify: `crates/ac-examples/src/08_usage_struct/main.at`
- Modify: `crates/ac-examples/src/09_input_message_builders/main.at`
- Modify: `crates/ac-examples/src/10_api_error_enum/main.at`
- Modify: `crates/ac-examples/src/12_stream_event_types/main.at`

#### Example 07: `07_glob_match/main.at`

**Simplifications made → what to restore:**

| Simplification (current) | Original intent | Unblocked by |
|---|---|---|
| `glob.substr(1, glob.len())` | `glob.slice(1)` | Task 4b (`str.slice()`) |
| `is ext { ... }` pattern | `file_name == glob` (direct string ==) | Task 1 (`string ==` interning) |

**Restored code outline:**

```auto
fn matches_glob(file_name str, glob str) bool {
    if glob.starts_with("*.") {
        let suffix = glob.slice(1)
        file_name.ends_with(suffix)
    } else {
        file_name == glob
    }
}
```

#### Example 08: `08_usage_struct/main.at`

**Simplifications made → what to restore:**

| Simplification (current) | Original intent | Unblocked by |
|---|---|---|
| Field-by-field `==` in `assert_eq(u1, u2)` | Direct `u1 == u2` | Task 4c (struct equality) |
| Removed `to_str` assertion | `assert(s.contains("Usage"))` | Task 7 (struct `TO_STR`) |

**Restored code outline:**

```auto
fn main() {
    let u1 = Usage { input_tokens: 100, output_tokens: 50, ... }
    let u2 = Usage.new(100, 50, 0, 0)
    assert_eq(u1, u2)                    // struct equality now works
    let s = u1.to_str()                  // or however TO_STR is exposed
    assert(s.contains("Usage"))
}
```

#### Example 09: `09_input_message_builders/main.at`

**Simplifications made → what to restore:**

| Simplification (current) | Original intent | Unblocked by |
|---|---|---|
| Flat `ContentBlock` with `kind str` + factory methods | `enum ContentBlock { Text(str), ToolUse(str, str), ToolResult(str, str) }` | Tasks 8-10 (enum data variants) |
| No list of ContentBlock | `content List<ContentBlock>` with `[ContentBlock.Text { text }]` | Task 11 (`List<UserType>`) |
| Separate `let` bindings for field access | Direct chained `.role`, `.content_text` | Task 3 (method chaining) |
| `contains()` instead of `==` for string assert | `assert_eq(msg.role, "user")` | Task 1 (string ==) |

**Restored code outline:**

```auto
type ContentBlock {
    Text { text str }
    ToolUse { id str, name str }
    ToolResult { tool_use_id str, content str }
}

type InputMessage {
    role str
    content List<ContentBlock>
}

ext InputMessage {
    fn user_text(text str) InputMessage {
        InputMessage {
            role: "user",
            content: [ContentBlock.Text { text: text }],
        }
    }
}
```

#### Example 10: `10_api_error_enum/main.at`

**Simplifications made → what to restore:**

| Simplification (current) | Original intent | Unblocked by |
|---|---|---|
| `kind int` (numeric) | `kind str` or enum variants like `ApiError.Http { msg }` | Task 1 (string ==) + Tasks 8-10 (enum variants) |
| `if/else if` chain for `display()` | `is self { ApiError.Http { msg } -> ... }` | Task 12 (pattern destructuring) |
| Separate `let` for each chained call | `ApiError.http("x").display()` | Task 3 (method chaining) |
| `classify_kind(kind int)` passing int | `classify_event(err StreamEvent)` passing struct | Task 4 (struct-as-param) |
| `assert(msg.contains("HTTP"))` | `assert_eq(err.display(), "HTTP error: ...")` | Task 1 (string ==) + Task 4c (struct equality for inner fields) |

**Restored code outline:**

```auto
type ApiError {
    Http { msg str }
    Json { msg str }
    Api { status uint, message str, retryable bool }
    Auth { msg str }
    RetriesExhausted { attempts uint }
}

ext ApiError {
    fn display(self) str {
        is self {
            ApiError.Http { msg } -> f"HTTP error: ${msg}",
            ApiError.Api { status, message, .. } -> f"API error (status ${status}): ${message}",
            // ...
        }
    }
}

fn is_retryable_error(err ApiError) bool {
    is err {
        ApiError.Api { retryable, .. } -> retryable,
        ApiError.Http { .. } -> true,
        _ -> false
    }
}

fn main() {
    let err = ApiError.Http { msg: "connection refused" }
    assert_eq(err.display(), "HTTP error: connection refused")
    assert(is_retryable_error(ApiError.Http { msg: "timeout" }))
}
```

#### Example 12: `12_stream_event_types/main.at`

**Simplifications made → what to restore:**

| Simplification (current) | Original intent | Unblocked by |
|---|---|---|
| `kind int` + flat StreamEvent | `enum StreamEvent { MessageStart { message MessageStartData }, ... }` | Tasks 8-10 (enum data variants) |
| `classify_kind(kind int)` passing int | `classify_event(event StreamEvent)` passing struct | Task 4 (struct-as-param) |
| `stop_reason str` (empty = none) | `stop_reason ?str` with `Some("end_turn")` / `None` | Task 13 (Option) |
| `extract_text_from_delta` returning `str` | `extract_text(event) ?str` returning `Some(t)` or `None` | Task 13 (Option) |
| `if/else if` chain for classify | `is event { StreamEvent.MessageStart -> "begin", ... }` | Task 12 (pattern destructuring) |
| No nested type fields on StreamEvent | Full nested data: `message MessageStartData`, `delta ContentBlockDelta`, `usage Usage` | Task 4 (struct-as-param) |

**Restored code outline:**

```auto
enum ContentBlockDelta {
    TextDelta { text str }
    InputJsonDelta { partial_json str }
    ThinkingDelta { thinking str }
}

enum StreamEvent {
    MessageStart { message MessageStartData }
    ContentBlockDelta { index uint, delta ContentBlockDelta }
    ContentBlockStop { index uint }
    MessageDelta { delta MessageDeltaData, usage Usage }
    MessageStop
    Ping
}

fn classify_event(event StreamEvent) str {
    is event {
        StreamEvent.MessageStart -> "begin"
        StreamEvent.ContentBlockDelta -> "delta"
        StreamEvent.MessageStop -> "end"
        StreamEvent.Ping -> "heartbeat"
        else -> "other"
    }
}

fn extract_text(event StreamEvent) ?str {
    is event {
        StreamEvent.ContentBlockDelta(delta) ->
            is delta {
                ContentBlockDelta.TextDelta(text: t) -> Some(t)
                else -> None
            }
        else -> None
    }
}

fn main() {
    let delta = StreamEvent.ContentBlockDelta {
        index: 0,
        delta: ContentBlockDelta.TextDelta { text: "Hello" }
    }
    assert_eq(classify_event(delta), "delta")
    assert_eq(extract_text(delta), Some("Hello"))
}
```

#### Step 1: Restore examples one at a time

Restore each example following the outlines above. After each restoration, run it to verify:

```bash
cd d:/autostack/auto-code-rs
auto crates/ac-examples/src/07_glob_match/main.at
auto crates/ac-examples/src/08_usage_struct/main.at
auto crates/ac-examples/src/09_input_message_builders/main.at
auto crates/ac-examples/src/10_api_error_enum/main.at
auto crates/ac-examples/src/12_stream_event_types/main.at
```

#### Step 2: Run full example suite

```bash
auto crates/ac-examples/src/01_djb2_hash/main.at
auto crates/ac-examples/src/04_token_estimate/main.at
auto crates/ac-examples/src/07_glob_match/main.at
auto crates/ac-examples/src/08_usage_struct/main.at
auto crates/ac-examples/src/09_input_message_builders/main.at
auto crates/ac-examples/src/10_api_error_enum/main.at
auto crates/ac-examples/src/12_stream_event_types/main.at
```

Expected: All pass.

#### Step 3: Commit

```bash
git add crates/ac-examples/src/
git commit -m "feat(examples): restore all examples to idiomatic Auto with enum variants, Option, pattern matching, struct equality, and method chaining"
```
