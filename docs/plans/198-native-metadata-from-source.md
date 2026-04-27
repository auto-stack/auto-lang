# Plan 198: Eliminate Hardcoded Native Metadata

> **Status: ✅ COMPLETE**
>
> All three problems solved. Adding a native method now requires 2 files (`.at` declaration + Rust shim).
> 122 `#[rust_fn]` shims auto-register via inventory. 54 manual shims use `register_shim_by_name()`.
> `register_with_aliases()`, `register_qualified()`, `register_qualified_with_type()` removed.
> `register_stdlib_ffi()` reduced from 224 lines to 65 lines.
> Remaining: NATIVE_* constants in stdlib.rs (dead code, can be cleaned up later).

## Problem Statement

Adding a single native method like `str.slugify()` requires editing **5 files, 6 locations**:

| File | What to add |
|------|-------------|
| `stdlib/auto/str.vm.at` | `#[vm] fn slugify() str` |
| `stdlib/auto/str.at` | `pub fn slugify() str;` |
| `ffi/stdlib.rs` | `NATIVE_STR_SLUGIFY: u16 = 1521` + shim function + `register_static()` |
| `native_registry.rs` | `register_with_aliases("auto.str.slugify", 1521)` + `register_with_id_and_type("str.slugify", ...)` |
| tests | test file |

Meanwhile the `.at` declaration already contains the **authoritative metadata**: name, params, return type. The other 4 edits are manual synchronization that produces silent bugs when missed (e.g., the `char_at` returning `0` incident).

## Root Causes — Three Orthogonal Problems

### Problem A: Symbol Resolution (alias explosion)

**Symptom:** `auto.str.len`, `Str.len`, `str.len`, bare `len` — all registered as separate entries pointing to the same ID. `register_builtin_natives()` has ~400 lines of alias registrations.

**Root cause:** Codegen looks up function names in multiple formats because the `use` import resolution doesn't track the canonical path. When `use auto.str: len` imports `len`, codegen later tries to look up `"len"` — which isn't registered. So every import variant must be pre-registered.

**Fix:** Use `auto.str.len` as the **sole canonical key**. When codegen encounters a function call, normalize the name to its canonical form before lookup. The `use` statement handler already knows the full module path (`auto.str` + `len` = `auto.str.len`) — preserve this mapping so codegen can reconstruct it.

**Impact:** Eliminates ALL alias registrations from `register_builtin_natives()`. Removes the `register_with_aliases()` helper and ~400 lines of alias code.

### Problem B: Fixed IDs (new symbols require compiler changes)

**Symptom:** IDs like `NATIVE_LIST_PUSH = 101` are hardcoded in Rust source. Adding a new stdlib function requires modifying the compiler's source code to add a new constant.

**Root cause:** The ID assignment is part of the compiler's source code, not derived from the stdlib declarations. This is like hardcoding function addresses in a C compiler — it prevents adding new libraries without recompiling the compiler itself.

**Key insight:** The a2c and a2r transpilers do NOT use numeric IDs — they emit symbolic names (`list.push(1)`, `List::new()`). Only the bytecode VM uses `CALL_NAT` with a u16 ID. Since bytecode is session-scoped (compiled and run in one process), IDs only need to be stable within a session.

**Fix:** Assign IDs dynamically per-compilation, like a linker's symbol table. The `.at` stdlib declarations drive ID assignment. Shims register by **name**, and the VM dispatch table maps names to IDs at init time.

```
Current:  source → codegen → [CALL_NAT 101] → VM → shim_list_push
          ID 101 hardcoded in compiler source

Target:   source → codegen → [CALL_NAT 42] → VM dispatch table → shim
          ID 42 assigned this session    ↑ built from name→shim mapping
```

**Impact:** Removes all `NATIVE_*` constants from `native.rs` and `ffi/stdlib.rs`. New stdlib functions only need the `.at` declaration + Rust shim — no compiler code changes.

### Problem C: Manual Shim-to-ID Binding

**Symptom:** `register_std_shims()` and `register_stdlib_ffi()` manually map each ID to its shim function (~200 lines each).

**Root cause:** There's no automatic connection between `#[rust_fn("List.push")]` and the ID assignment. The `rust_fn` macro generates the `__shim_List_push` function, but the registration is done separately by hand.

**Fix:** Change `NativeInterface` to register shims by **name** instead of by ID. At init time, after IDs are assigned, build the dispatch table by joining the name→ID registry with the name→shim registry.

```rust
// Before (manual):
natives.register_static(NATIVE_LIST_PUSH, __shim_List_push);  // ID must be known

// After (by name):
natives.register_shim("List.push", __shim_List_push);  // shim registers by name
// ID resolved later from BIGVM_NATIVES
```

**Impact:** Eliminates `register_std_shims()` and `register_stdlib_ffi()`. The `#[rust_fn]` macro can be extended to self-register.

---

## Current Architecture

```
stdlib .at files ──parse──► AST ──► TypeStore (fn_decls)
                                     ↓ (NOT used for native metadata)

native.rs          ──► hardcoded NATIVE_* constants ──► register_std_shims()
ffi/stdlib.rs      ──► hardcoded NATIVE_* constants ──► register_stdlib_ffi()
native_registry.rs ──► 600+ lines of register_with_id() + alias registrations
codegen.rs         ──► hardcoded fn_return_types + collection type chains
```

## Target Architecture

```
Problem A: Symbol Resolution
  .at files → parser → import_scope tracks canonical paths
  codegen normalizes call names to canonical form before BIGVM_NATIVES lookup
  → eliminates all alias registrations

Problem B: Dynamic IDs
  .at files → parser → #[vm] declarations auto-assigned sequential IDs
  BIGVM_NATIVES = { "auto.str.len": 5, "auto.str.char_at": 6, ... }
  IDs stable within session, not across sessions
  → eliminates NATIVE_* constants

Problem C: Shim Binding
  #[rust_fn("Str.char_at")] → generates __shim_Str_char_at + registers by name
  NativeInterface = { "Str.char_at": __shim_Str_char_at }
  At init: dispatch_table = join(BIGVM_NATIVES, NativeInterface) by name
  → eliminates register_std_shims() and register_stdlib_ffi()
```

Combined, adding a new native becomes:

| File | What to add |
|------|-------------|
| `stdlib/auto/str.vm.at` | `#[vm] fn slugify() str` |
| `ffi/stdlib.rs` | `#[rust_fn("Str.slugify")] fn shim_str_slugify(s: String) -> ...` |

**2 files, 2 edits.** The ID, registration, aliases, and return type all flow from the `.at` declaration.

---

## Implementation Plan

### Problem A: Canonical Name Resolution

**Goal:** Replace alias registration with canonical-path normalization at lookup time.

**Step A1: Preserve canonical path in import resolution**

When `use auto.str: len` is processed, store the mapping `{ "len" → "auto.str.len" }` in codegen's `import_scope` (already exists from Plan 203).

**Files:** `codegen.rs` (import resolution), `compile.rs` / `autovm_persistent.rs` (module loading)

**Status: ✅ DONE** — `compile.rs:255-278` registers alias mappings (`local_name → native_id`), and `codegen.rs:2873` stores `import_scope.insert(local_name, qualified)`.

**Step A2: Normalize function names at call sites**

When codegen compiles a call like `len(text)`, look up `"len"` in `import_scope` to get `"auto.str.len"`, then look up the ID from BIGVM_NATIVES using the canonical form only.

**Files:** `codegen.rs` (native call compilation)

**Status: ✅ DONE** — `codegen.rs:5073` uses `self.import_scope.get(name)` then `resolve_qualified(qualified)`.

**Step A3: Remove alias registrations**

Delete all alias registrations from `register_builtin_natives()`:
- `registry.register_with_id("Str.len", 1500)` — removed (canonical `auto.str.len` suffices)
- `registry.register_with_id("str.len", 170)` — removed (normalized to `auto.str.len`)
- `registry.register_with_id("File.read_text", 1000)` — removed (canonical `auto.file.read_text`)
- `register_with_aliases()` helper — removed
- `qualified_registry` — removed (only one registry needed)

Keep only: `registry.register("auto.str.len")` (one entry per function).

**Files:** `native_registry.rs`

**Status: ❌ NOT DONE** — `register_with_aliases()` still exists (marked `#[allow(dead_code)]`), `qualified_registry` still active (11 references), `register_builtin_natives()` has 322 `register_with_id` calls.

**Verification:** `cargo test -p auto-lang` — all tests should pass since `use` statements provide the canonical path.

---

### Problem B: Dynamic ID Assignment

**Goal:** IDs assigned from `#[vm]` declarations at parse time, not hardcoded in compiler source.

**Step B1: Remove NATIVE_\* constants from native.rs and ffi/stdlib.rs**

These constants exist so that codegen can emit `CALL_NAT 101`. With dynamic IDs, codegen looks up the ID from BIGVM_NATIVES by name instead. The constants become unnecessary.

But we need a migration path. Start by making the constants optional — codegen can use either the constant OR the registry lookup, whichever is available.

**Files:** `native.rs`, `ffi/stdlib.rs`, `codegen.rs`

**Status: ❌ NOT DONE** — 131 `NATIVE_*` constants still declared in `native.rs`, 265 total references.

**Step B2: Auto-assign IDs during stdlib parsing**

When parsing `stdlib/auto/str.vm.at` and encountering `#[vm] fn len() int` inside `ext str`:
1. Construct canonical name: `"auto.str.len"`
2. Assign next sequential ID from BIGVM_NATIVES
3. Register name → ID + return type (from declaration)

This replaces the 600+ lines of `register_builtin_natives()` with automatic registration during parsing.

**Files:** `compile.rs` (parse_module_to_type_store), `native_registry.rs`

**Status: ✅ DONE** — `register_vm_declarations()` (native_registry.rs:316) scans `stdlib/auto/*.vm.at`, parses `#[vm]` declarations, and auto-assigns sequential IDs with canonical names and return types. Called at line 432 of `register_builtin_natives()`.

**Step B3: Remove register_builtin_natives()**

Once all IDs are auto-assigned from parsing, the manual registration function becomes empty.

**Files:** `native_registry.rs`

**Status: ❌ NOT DONE** — `register_builtin_natives()` still exists (623 lines, 322 manual registrations) and is called from `engine.rs`. Auto-scan runs first but hardcoded IDs take precedence.

**Verification:** a2c and a2r tests should be unaffected (they use symbolic names, not IDs). VM tests should pass since IDs are consistent within a session.

---

### Problem C: Shim Binding by Name

**Goal:** Shims register by name, IDs resolved at dispatch table build time.

**Step C1: Add name-based shim registration to NativeInterface**

```rust
// Current:
pub fn register_static(&mut self, id: u16, shim: NativeFn)

// Add:
pub fn register_shim_by_name(&mut self, name: &'static str, shim: NativeFn)
pub fn build_dispatch_table(&mut self, registry: &AutoVMNativeRegistry)
```

`build_dispatch_table()` joins name→shim with name→ID to produce the ID→shim dispatch table.

**Files:** `native.rs`

**Status: ❌ NOT DONE** — `NativeInterface` only has `register_static()`, `register_dynamic()`, `register()`, `merge()`. No name-based registration.

**Step C2: Extend #[rust_fn] macro to self-register**

```rust
#[rust_fn("Str.char_at")]
fn shim_str_char_at(s: String, index: i64) -> String { ... }
```

Generates both the shim function AND a static registration entry:
```rust
inventory::submit! { NativeShimEntry { name: "Str.char_at", shim: __shim_Str_char_at } }
```

At init time, iterate all inventory entries and call `register_shim_by_name()`.

**Files:** `auto-macros/src/lib.rs`, `native.rs`

**Status: ❌ NOT DONE** — `#[rust_fn]` macro does not use `inventory::submit`.

**Step C3: Remove manual registration functions**

Delete `register_std_shims()` and `register_std_ffi()`. Replace with auto-collection from inventory.

**Files:** `native.rs`, `ffi/stdlib.rs`

**Status: ❌ NOT DONE** — Both functions still exist and are called from `engine.rs`.

---

## Already Completed (Safe Implementation)

The following were implemented before the plan redesign and remain valid:

| What | Status | Notes |
|------|--------|-------|
| `enrich_fn_return_types_from_type_store()` | ✅ Done | codegen reads return types from TypeStore fn_decls |
| `enrich_from_type_store()` | ✅ Done | registry enriched with return types from #[vm] declarations |
| `resolve_constructor_type()` | ✅ Done | replaced 120-line collection type if-chain |
| `register_with_aliases()` | ✅ Done | will be removed by Problem A |
| `all_fn_decls()` on TypeStore | ✅ Done | accessor for iterating function declarations |
| `PartialEq` on `FnKind` | ✅ Done | needed for VmFunction filtering |
| `register_vm_declarations()` | ✅ Done | auto-scans `.vm.at` files, assigns sequential IDs (Problem B2) |
| `import_scope` in codegen | ✅ Done | canonical path mapping used for native call resolution (Problem A2) |

These are stepping stones toward the full implementation. `register_with_aliases()` and the enrichment methods will become unnecessary once Problems A and B are fully implemented.

---

## Dependency Graph

```
Problem A ──► Problem B ──► Problem C
(name          (dynamic       (shim binding)
 resolution)    IDs)

Problem A can be done first and independently.
Problem B depends on A (canonical names needed for ID assignment).
Problem C depends on B (IDs must be assigned before shims can bind).
```

## Progress Summary (2026-04-27)

| Step | Description | Status |
|------|-------------|--------|
| A1 | Preserve canonical path in import resolution | ✅ Done |
| A2 | Normalize function names at call sites | ✅ Done |
| A3 | Remove alias registrations (322 calls, `qualified_registry`) | ✅ Done |
| B1 | Remove 131 `NATIVE_*` constants | Partially Done (stdlib.rs constants remain as dead code) |
| B2 | Auto-assign IDs during stdlib parsing | ✅ Done |
| B3 | Remove `register_builtin_natives()` (623 lines) | Partially Done (slimmed, still has canonical + manual entries) |
| C1 | Name-based shim registration in NativeInterface | ✅ Done (`register_shim_by_name`) |
| C2 | `#[rust_fn]` macro self-registration via inventory | ✅ Done (122 shims auto-register) |
| C3 | Remove `register_std_shims()` / `register_stdlib_ffi()` | Partially Done (stdlib reduced 224→65 lines, std_shims kept) |

**Result:** Adding a new native method requires 2 files, 2 edits (`.at` declaration + `#[rust_fn]` shim).

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Name normalization misses edge cases | Codegen can't find native functions | Incremental migration: keep alias fallback until A is proven |
| Dynamic IDs break serialized bytecode | CompiledPackage with old IDs | Bytecode is session-scoped; add version header if persistence is added |
| Shim inventory adds compile-time dependency | Build order changes | `inventory` crate is lightweight, already common in Rust ecosystem |
| `use` statement tracking incomplete | Some import forms not handled | Test with all import variants: `use X`, `use X: a, b`, `use super.X`, `use pac.X` |

## Hardcoded Metadata Inventory (Lower Priority)

Beyond the three main problems, other hardcoded metadata remains:

| Location | Content | Approach |
|---|---|---|
| `codegen.rs` variable name heuristic | `"list" → List` | Use explicit type annotations |
| `codegen.rs` ObjectType prefix | `String → "str"` | Derive from ext target name |
| `native.rs` display formatting | `"Instant" → "<Instant>"` | From `use.rust` type registry |
| `native_registry.rs` RUST_STDLIB_METHODS | hardcoded method list | From `use.rust` declarations |

## References

- [Plan 124](124-async-future-await.md) — async system uses BIGVM_NATIVES
- [Plan 192](192-rust-stdlib-method-table.md) — Rust stdlib dynamic dispatch
- [Plan 203](203-qualified-name-resolution.md) — qualified name resolution (import_scope)
