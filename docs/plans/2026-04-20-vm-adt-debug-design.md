# Plan: VM Enum/Data, Generic Lists, Pattern Destructuring, and Debug Formatting

**Date:** 2026-04-20
**Status:** Design approved

## Problem

The Auto VM lacks four runtime features needed for realistic programs:

1. **No debug output for struct types** — `TO_STR` only handles i32 and tagged strings. Struct instances (heap objects) print as garbage integers.
2. **No enum variants with data** — Only C-style scalar enums (`enum Color { Red = 1 }`) work. Tuple variants like `Atom.Int(42)` have no runtime representation.
3. **No `List<UserType>`** — `GET_ELEM` only handles `List<int>`, `List<str>`, `List<bool>`. User-defined types in lists are unsupported.
4. **No pattern destructuring in `is`** — `is expr { Variant(x) -> body }` cannot bind variables from matched values.

## Phase 1: Default `to_str` for Struct Types

### Current State

`TO_STR` (engine.rs:1268) checks if value is a tagged string (negative i32). Otherwise converts the i32 to its decimal string. Heap object IDs (>= 4000000) print as the raw integer.

### Design

Extend `TO_STR` to detect heap objects and format them:

```
if value_bits >= 4000000:
    let instance = heap[value_bits].downcast::<GenericInstanceData>()
    let fields = instance.fields
    let name = instance.mono_name
    format!("{} {{ {} }}", name, fields.iter()
        .zip(field_names)
        .map(|(k, v)| format!("{}: {}", k, display_value(v)))
        .join(", "))
```

Field names come from `GenericInstanceData` — we add a `field_names: Vec<String>` field alongside the existing `fields: Vec<Value>`.

### Changes

- **`generic_registry.rs`**: Add `field_names` to `GenericInstanceData`. Populate during `CONSTRUCT_INSTANCE`.
- **`codegen.rs`**: When emitting `CONSTRUCT_INSTANCE`, also emit the field name strings into the constant pool and pass the string index.
- **`engine.rs` `TO_STR`**: Detect heap object IDs, downcast, format.
- **No new opcodes needed.**

### Format

```
Usage { input_tokens: 100, output_tokens: 50, cache_creation_input_tokens: 0, cache_read_input_tokens: 0 }
```

For nested objects, recurse up to depth 3 then print `...`.

## Phase 2: Enum Variants with Data

### Current State

- Parser accepts `enum Atom { Int int, Char char }` and `enum Foo { Bar { x int } }`.
- Codegen only emits `CONST_I32` with the variant discriminant for scalar enums.
- No runtime representation for data-carrying variants.

### Design

**Representation: Tagged heap objects.**

Each enum variant with data is stored as a heap object with:
- `__tag: String` — the variant name (e.g., `"Int"`, `"Text"`)
- `__enum: String` — the enum name (e.g., `"Atom"`, `"ContentBlock"`)
- Payload fields as `_0`, `_1`, ... for tuple variants or named fields for struct variants

This reuses the existing `GenericInstanceData` infrastructure — enum variants are just instances with extra metadata.

### New Opcodes

| Opcode | Code | Stack | Description |
|--------|------|-------|-------------|
| `CREATE_VARIANT` | `0x7C0` (chosen to avoid conflict) | ..., value1, ..., valueN → instance_id | Create enum variant. Code layout: `[opcode, tag_str_idx:u16, field_count:u8]` |

Actually, we can reuse `NEW_INSTANCE` + `CONSTRUCT_INSTANCE` since variants are just heap objects. The difference is the `mono_name` encodes the variant: `"Atom.Int"`, `"ContentBlock.Text"`.

### Construction

`Atom.Int(42)` compiles to:
1. `CONST_I32 42` — push payload
2. `CONST_I32 7` — mono_name length (`"Atom.Int"`)
3. `NEW_INSTANCE` — creates instance, reads `"Atom.Int"` from code
4. `CONST_I32 1` — field count
5. `CONSTRUCT_INSTANCE` — stores payload into instance

`ContentBlock.Text { text: "hello" }` compiles similarly but with named fields and `mono_name = "ContentBlock.Text"`.

### Changes

- **`codegen.rs`**: Add codegen for `Expr::EnumVariant(expr)` — emit NEW_INSTANCE + CONSTRUCT_INSTANCE with variant mono_name.
- **`engine.rs`**: No new opcode needed — existing NEW_INSTANCE + CONSTRUCT_INSTANCE handle it.
- **`generic_registry.rs`**: Register each enum variant as a separate "type" in the registry (e.g., `"Atom.Int"` has fields `[_0: int]`).

## Phase 3: `List<T>` with User-Defined Types

### Current State

`GET_ELEM` (engine.rs:1943) only handles `List<int>`, `List<str>`, `List<bool>` via `downcast_ref::<ListData<T>>()`. User-defined types in lists fail at runtime.

### Design

User-defined type instances are heap object IDs (i32 >= 4000000). On the stack they look like regular i32 values. A `List<UserType>` is just a list of i32 values where some happen to be heap refs.

### New Opcode

| Opcode | Code | Description |
|--------|------|-------------|
| `CREATE_LIST_REF` | `0xA6` (reuse adjacent range) | Create a list that stores i32 values (heap refs or raw ints) |
| `LIST_PUSH_REF` | `0xA7` | Push i32 value (no type-specific downcast) |
| `LIST_GET_REF` | `0xA8` | Get i32 value (returns raw i32, caller interprets) |

### Changes

- **`opcode.rs`**: Add three new opcodes.
- **`codegen.rs`**: When constructing a list with elements of user-defined type, emit `CREATE_LIST_REF` instead of `CREATE_LIST_INT`. When accessing elements, emit `LIST_GET_REF` instead of `LIST_GET_INT`.
- **`engine.rs`**: Implement the three opcodes. `CREATE_LIST_REF` creates a `Vec<i32>`. `LIST_PUSH_REF` pushes i32. `LIST_GET_REF` returns i32.
- **`LIST_GET_REF`**: The returned i32 is the heap object ID. The caller (e.g., field access) treats it as an instance reference.

### Simplification

We could use a single `Vec<Value>` instead of typed lists. But that's a bigger refactor. For now, `LIST_GET_REF` returns raw i32 and the caller handles interpretation.

## Phase 4: Pattern Destructuring in `is`-expressions

### Current State

- `is expr { Some(x) -> body }` works for Option/Result (hardcoded in codegen as opcode sequences).
- General `is expr { EnumVariant(x, y) -> body }` has no codegen or runtime support.

### Design

Compile `is` with destructuring to a sequence of tag-check + field-extraction:

```auto
is msg.content[0] {
    ContentBlock.Text { text } -> print(text),
    ContentBlock.ToolUse { id, name } -> print(name),
    else -> print("unknown")
}
```

Compiles to:

```
LOAD_LOCAL expr              ; push the value to match
GET_GENERIC_FIELD __tag_idx  ; get __tag field (string)
LOAD_STR "Text"              ; push expected tag
EQ                           ; compare
JMP_IF_Z else_branch         ; if not match, jump

; Matched: extract bindings
GET_GENERIC_FIELD _0_idx     ; get payload field
STORE_LOCAL text_local       ; bind to variable 'text'

; Body
CALL print(text_local)
JMP end

; else branch
else_branch:
CALL print("unknown")

end:
```

### Changes

- **`codegen.rs`**: Extend `compile_is()` to handle enum variant patterns. For each branch:
  1. Emit tag comparison (`GET_FIELD __tag` + string comparison)
  2. On match, emit `GET_GENERIC_FIELD` for each binding + `STORE_LOCAL`
  3. Compile branch body
  4. Jump to end
- **`engine.rs`**: No new opcodes needed — reuses GET_FIELD + EQ + conditional jumps.
- **`generic_registry.rs`**: Register enum variants with `__tag` as first field at index 0.

### Field Name Resolution

For tuple variants: fields are `_0`, `_1`, `_2`, etc.
For struct variants: fields use their declared names (e.g., `text`, `id`).

## Implementation Order

| Phase | Feature | Depends On | Estimated Complexity |
|-------|---------|------------|---------------------|
| 1 | Debug `to_str` for structs | None | Small — ~50 lines in engine.rs + generic_registry.rs |
| 2 | Enum variants with data | Phase 1 (for debugging) | Medium — codegen for variant construction, registry updates |
| 3 | `List<UserType>` | Phase 2 (lists of enum variants) | Small — 3 new opcodes, reuse heap ref pattern |
| 4 | Pattern destructuring in `is` | Phase 2 (variant objects) | Medium — codegen for tag-check + field extraction |

## Testing Strategy

Each phase adds test cases to `crates/auto-lang/test/vm/`:

- **Phase 1**: `type Point { x int, y int }` → `print(p)` outputs `Point { x: 1, y: 2 }`
- **Phase 2**: `enum Atom { Int int, Str str }` → construct + field access
- **Phase 3**: `let list = [Atom.Int(1), Atom.Int(2)]` → `list[0]` returns variant
- **Phase 4**: `is atom { Atom.Int(n) -> print(n) }` → binds and prints `n`

## Open Questions

- Should `CREATE_VARIANT` be a new opcode or reuse `NEW_INSTANCE`? (Leaning toward reuse.)
- Max recursion depth for nested debug formatting? (Defaulting to 3.)
- Should `List<UserType>` use `Vec<i32>` or `Vec<Value>` internally? (Leaning toward `Vec<i32>` for now.)
