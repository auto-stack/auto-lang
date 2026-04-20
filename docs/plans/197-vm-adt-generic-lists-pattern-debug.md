# Plan 197: VM Enum/Data, Generic Lists, Pattern Destructuring & Debug Formatting

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Date:** 2026-04-20
**Status:** Approved
**Goal:** Add six runtime features to the Auto VM in dependency order: string equality via `==`, chained method call type resolution, struct debug formatting, enum variants with data, List<UserType>, and pattern destructuring in `is`-expressions.
**Architecture:** Enum variants reuse the existing `GenericInstanceData` heap object system with a `mono_name` encoding the variant (`"Atom.Int"`). Pattern matching compiles to tag-check + field-extraction using existing opcodes. Debug formatting extends `TO_STR` to handle heap objects. String equality is fixed by interning literals in codegen + content-aware EQ in the engine. Method chaining is fixed by consulting `fn_return_types` in `infer_object_type()`.
**Tech Stack:** Rust, AutoLang crate (`auto-lang`), existing VM infrastructure (heap objects, generic registry, opcodes).

---

## Problem

The Auto VM lacks four runtime features needed for realistic programs:

1. **String `==` compares tagged IDs, not content** — Identical string literals get different negative tag IDs because `Expr::Str` bypasses `add_string()` interning. `EQ` opcode compares raw i32 values, so `"http" == "http"` returns false when the two occurrences get different tags.
2. **Chained method calls fail type resolution** — `infer_object_type()` returns `NestedObject` for all `Expr::Call` (codegen.rs:5386), causing method lookup to generate `Unknown_display` instead of `ApiError.display`. The `fn_return_types` map has the correct return type but is never consulted.
3. **No debug output for struct types** — `TO_STR` only handles i32 and tagged strings. Struct instances (heap objects) print as garbage integers.
4. **No enum variants with data** — Only C-style scalar enums (`enum Color { Red = 1 }`) work. Tuple variants like `Atom.Int(42)` have no runtime representation.
5. **No `List<UserType>`** — `GET_ELEM` only handles `List<int>`, `List<str>`, `List<bool>`. User-defined types in lists are unsupported.
6. **No pattern destructuring in `is`** — `is expr { Variant(x) -> body }` cannot bind variables from matched values.

## Current State

| Feature | Parser | Codegen | Engine |
|---------|--------|---------|--------|
| String `==` | OK | `Expr::Str` bypasses `add_string()` interning (codegen.rs:3000-3006) | `EQ` compares raw i32 (engine.rs:3150-3157) |
| Method chaining | OK | `infer_object_type()` returns `NestedObject` for all calls (codegen.rs:5386) | Runtime works if bytecode is correct |
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
| 1 | Debug `to_str` for structs | None | Small (~50 lines) |
| 2 | Enum variants with data | Phase 1 (for debugging) | Medium |
| 3 | `List<UserType>` | Phase 2 (lists of enum variants) | Small (3 opcodes) |
| 4 | Pattern destructuring in `is` | Phase 2 (variant objects) | Medium |

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

### Task 4: Add `field_names` to `GenericInstanceData`

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

### Task 5: Populate `field_names` during `CONSTRUCT_INSTANCE`

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

### Task 6: Extend `TO_STR` to format struct instances

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

### Task 7: Register enum variants in `GenericRegistry`

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

### Task 8: Codegen for enum variant construction

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

### Task 9: Access payload fields from enum variants

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

### Task 10: `List<UserType>` support

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

### Task 11: Pattern destructuring in `is`-expressions

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

### Task 12: Integration test — full examples with all features

**Files:**
- Modify: `crates/ac-examples/src/09_input_message_builders/main.at`
- Modify: `crates/ac-examples/src/10_api_error_enum/main.at`

**Step 1: Restore 09_input_message_builders using enum variants + pattern matching**

Rewrite using `enum ContentBlock { Text(str), ToolUse(str, str), ToolResult(str, str) }`, pattern matching with destructuring, and `List<ContentBlock>`.

**Step 2: Restore 10_api_error_enum using string kind + method chaining**

Rewrite back to `kind str` with string `==` comparisons (now fixed) and chained calls like `ApiError.http("timeout").display()`.

**Step 3: Run all examples to verify no regressions**

```bash
cd d:/autostack/auto-code-rs
auto crates/ac-examples/src/01_djb2_hash/main.at
auto crates/ac-examples/src/07_glob_match/main.at
auto crates/ac-examples/src/08_usage_struct/main.at
auto crates/ac-examples/src/09_input_message_builders/main.at
auto crates/ac-examples/src/10_api_error_enum/main.at
```

Expected: All pass.

**Step 4: Commit**

```bash
git add crates/ac-examples/src/09_input_message_builders/main.at crates/ac-examples/src/10_api_error_enum/main.at
git commit -m "feat(examples): restore 09 and 10 with enum variants, pattern matching, and method chaining"
```
