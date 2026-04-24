# Plan 198: Eliminate Hardcoded Native Metadata — Derive from Source Declarations

> **Status: ❌ NOT IMPLEMENTED**

## Problem

The Auto compiler has a **three-way (up to six-way) duplication** problem for every native function:

1. **ID constant** in `native.rs` / `ffi/stdlib.rs` — `NATIVE_STR_CHAR_AT: u16 = 1502`
2. **Name-to-ID mapping** in `native_registry.rs` — `registry.register_with_id("str.char_at", 1502)`
3. **Return type** in `native_registry.rs` — `register_with_id_and_type("str.char_at", 1502, NativeRetType::String)`
4. **Type inference** in `codegen.rs` — `fn_return_types.insert("str.char_at", Type::String)`
5. **Collection type construction** in `codegen.rs` — hardcoded `TypeDecl` for `List.new()`, `HashMap.new()`, etc.
6. **Triple alias registration** — `auto.str.len`, `Str.len`, `str.len` all manually listed

Meanwhile, the `.at` stdlib files already contain the **authoritative declarations**:
```auto
ext str {
    #[vm]
    fn char_at(index int) str   // ← return type is RIGHT HERE
}
```

Adding a new native method requires changes in **3-6 files**, each with manual synchronization. Missing any entry produces silent bugs (e.g., the `char_at` returning `0` incident).

## Current Architecture

```
.at stdlib files ──parse──► AST ──► TypeStore (fn_decls)
                                     ↓ (NOT used for native metadata)

native.rs          ──► hardcoded NATIVE_* constants ──► register_std_shims()
ffi/stdlib.rs      ──► hardcoded NATIVE_* constants ──► register_stdlib_ffi()
native_registry.rs ──► hardcoded register_with_id() calls (500+ lines)
codegen.rs         ──► hardcoded fn_return_types, type construction, heuristics
```

## Target Architecture

```
.at stdlib files ──parse──► AST
                          │
                          ├─► TypeStore (fn_decls) ──► codegen reads return types
                          │
                          └─► NativeIdGenerator ──► auto-assigns IDs, registers shims
                                      │
                                      ▼
                              native_registry (auto-populated)
```

## Implementation Plan

### Phase 1: Return Types from TypeStore (Low Risk)

**Goal:** codegen reads return types from TypeStore instead of hardcoded lists.

**Changes:**
- `register_builtin_natives()` reverts to `register_with_id()` only (remove `register_with_id_and_type` / `NativeRetType`)
- `new_with_type_store()` already has `Arc<RwLock<TypeStore>>` — read `fn_decls` to build `fn_return_types`
- For `new()` (no TypeStore), fall back to the current registry import
- Delete `NativeRetType` enum and related methods

**Files:** `codegen.rs`, `native_registry.rs`

### Phase 2: Auto-Register Native IDs from `#[vm]` Declarations (Medium Risk)

**Goal:** Parsing stdlib `.at` files auto-populates the native registry.

**Changes:**
- Add `NativeIdGenerator` that auto-assigns sequential IDs per type category
- During `parse_module_to_type_store`, when encountering `#[vm] fn` in ext blocks:
  - Auto-assign a native ID (per-type range: List 100-199, str 200-299, etc.)
  - Register `Type.method` in `BIGVM_NATIVES`
  - Auto-generate `str.*`, `Str.*`, `auto.str.*` aliases from module path
  - Store return type from the declaration
- Delete 500+ lines of manual `register_with_id()` calls from `register_builtin_natives()`

**Files:** `native_registry.rs`, `compile.rs`, `types.rs`

### Phase 3: Auto-Register Shims from Rust FFI Annotations (Medium Risk)

**Goal:** `#[auto_macros::rust_fn("Str.char_at")]` macro auto-registers the shim.

**Changes:**
- Extend `rust_fn` proc macro to also call `BIGVM_NATIVES.register(name, id)` at compile time
- The macro already generates the shim name — extend it to register the name-to-shim mapping
- Delete `register_std_shims()` and `register_stdlib_ffi()` manual registration

**Files:** `auto-macros/src/lib.rs`, `native.rs`, `ffi/stdlib.rs`

### Phase 4: Collection Type Inference from TypeStore (Lower Risk)

**Goal:** `List.new()`, `HashMap.new()` etc. resolve types from TypeStore instead of hardcoded if-chains.

**Changes:**
- When codegen encounters `Type.new()` and TypeStore has the type decl, construct `Type` from the decl
- Delete the 120-line hardcoded chain: `if type_name == "List" && method == "new" { ... } else if ...`
- The `TypeDecl` in TypeStore already has generic params and field info

**Files:** `codegen.rs`

### Phase 5: Alias Auto-Generation (Low Risk)

**Goal:** No more manual triple registration (`auto.str.len`, `Str.len`, `str.len`).

**Changes:**
- When registering `auto.str.len` (from module path + method name), auto-generate:
  - `Str.len` (capitalized type name)
  - `str.len` (lowercase type name)
- Apply to all module-registered natives

**Files:** `native_registry.rs`

## Hardcoded Metadata Inventory

Beyond the native registry, other hardcoded metadata that should eventually be migrated:

| Location | Hardcoded Content | Migration Approach |
|---|---|---|
| `codegen.rs:997-1110` | Collection `Type.new()` type construction | Phase 4 |
| `codegen.rs:6370-6381` | Variable name → type heuristic (`"list" → List`) | Use explicit type annotations; remove heuristics |
| `codegen.rs:4213-4220` | `ObjectType` → native name prefix (`String → "str"`) | Derive from ext target name |
| `codegen.rs:6400-6419` | `needs_id_extraction()` Iterator method list | Derive from spec declaration |
| `native.rs:656-679` | Rust type display formatting (`"Instant" → "<Instant>"`) | From `use.rust` type registry |
| `native_registry.rs:637-667` | `RUST_STDLIB_METHODS` hardcoded list | From `use.rust` import declarations |
| `native.rs:976` | `ListData<T>` downcast chain | Derive from generic instantiation table |
| `compile.rs:429` | `"auto/"` stdlib prefix | From project package config |

## Migration Priority

1. **Phase 1** (this PR) — Return types from TypeStore, highest ROI, lowest risk
2. **Phase 2+3** — Auto-registration from `#[vm]` + `rust_fn`, biggest cleanup
3. **Phase 4+5** — Collection types + aliases, polish

## Risks

- **ID stability**: Auto-assigned IDs must not collide with manually assigned ones. Mitigation: reserve ID ranges per category.
- **Init order**: `register_builtin_natives()` is called in `Codegen::new()`. TypeStore may not be populated yet. Mitigation: two-phase init — register IDs early, fill return types after stdlib parse.
- **Shim discovery**: Phase 3 requires proc macro to call into the native registry at compile time. This works if the registry is a `lazy_static`.
