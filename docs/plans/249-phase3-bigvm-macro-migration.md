# Plan 249 Phase 3: BIGVM Registry Macro Migration

## Context

`register_builtin_natives()` in `native_registry.rs` has ~505 manual `register_with_id()` / `register_with_id_and_type()` / `register_return_type()` calls. Phase 3 goal: move all of them into a `for_each_bigvm_native!` macro so each function is defined once in the catalog.

Current state: 13 List entries already migrated as a prototype. Need to migrate the remaining ~490 entries.

## Approach: Three-tuple entry format

Extend `for_each_bigvm_native!` entry from `(name, id)` to `(name, id, ret_type)` where:
- `ret_type` is `Void` (default) for most entries
- `ret_type` is `List`, `Bool`, `Int`, `I64`, `String`, `Float`, `Map` for typed entries

This allows a single consumer macro to handle both `register_with_id` and `register_with_id_and_type`.

### Entry format
```rust
// (canonical_name, numeric_id, return_type)
("auto.list.new", 100, Void),
("auto.list.map", 2060, List),
("auto.chrono_opaque.local_now", 2700, Int),
```

### Consumer macro
```rust
macro_rules! __register_bigvm {
    (($name:expr, $id:expr, $ret:ident) $(, $rest:tt)*) => {
        // For typed entries, register_with_id_and_type; for Void, register_with_id
        __register_bigvm_typed!($name, $id, $ret);
        __register_bigvm!($($rest),*);
    };
    () => {};
}

macro_rules! __register_bigvm_typed {
    ($name:expr, $id:expr, Void) => { registry.register_with_id($name, $id); };
    ($name:expr, $id:expr, List) => { registry.register_with_id_and_type($name, $id, NativeRetType::List); };
    // ... etc for Bool, Int, I64, String, Float, Map
}
```

## Migration Strategy: Batch by category

Move entries group-by-group from `register_builtin_natives()` into the macro, keeping the same order. After each batch, build & test.

### Batches (in order)

1. **List HOF** (10 entries, IDs 2060-2069) — `register_with_id_and_type`
2. **Iterator** (8 entries, IDs 111-118)
3. **HashMap** (15 entries, IDs 119-1292) — includes aliases like `Map.new`, `HashMap.new`
4. **HashSet** (7 entries, IDs 129-135)
5. **VecDeque** (11 entries, IDs 136-146)
6. **BTreeMap** (11 entries, IDs 147-157)
7. **StringBuilder** (8 entries, IDs 160-167)
8. **Heap/Storage** (14 entries, IDs 190-202)
9. **String** (30+ entries, IDs 170-186, 1500-1520)
10. **Bit Operations** (16 entries, IDs 210-234)
11. **File/FS** (35+ entries, IDs 1000-1015 + fs aliases)
12. **Env/Time/Process/Path** (20 entries, IDs 1100-1404)
13. **Char/Log/Math/Rand** (40+ entries, IDs 1600-1854)
14. **JSON/TOML** (20+ entries, IDs 1900-2611)
15. **URL/Net/HTTP** (50+ entries, IDs 2000-2258)
16. **Task/TaskSystem** (15 entries, IDs 2300-2311)
17. **Regex/Sys/FS-extra** (10 entries, IDs 2400-2430)
18. **Opaque structs** (already in for_each_native! but need BIGVM entries too: re/url/semver/chrono/sha2, IDs 2450-2739)
19. **Bare names & aliases** (sleep, parse_sse, File.*, Str.*, Task.*, str.*, etc.)
20. **Return type annotations** (21 `register_return_type` calls — integrate into entries)

### Alias handling

Many entries are aliases (e.g., `auto.fs.read_text` → same ID as `auto.file.read_text`). Two options:
- **Option A**: Include aliases as separate entries in the macro (simple, some duplication)
- **Option B**: Add an `aliases` field to each entry

**Decision**: Option A — include aliases as separate entries. Reasons:
- Simpler macro format (no variadic aliases field)
- Clear 1:1 mapping from macro entry to registry call
- Aliases may have different IDs in edge cases

### Return type annotations

The 21 `register_return_type()` calls register return types for entries that were already registered via `register_with_id`. These should be converted to `register_with_id_and_type` and merged into the main entry list. This eliminates the separate `register_return_type` section entirely.

Specifically, entries like:
```rust
registry.register_with_id("auto.str.len", 1500);
// ...later...
registry.register_return_type("auto.str.len", NativeRetType::Int);
```
become:
```rust
("auto.str.len", 1500, Int),
```

## Files to modify

| File | Change |
|------|--------|
| `vm/native_catalog.rs` | Expand `for_each_bigvm_native!` with all ~477 entries |
| `vm/native_registry.rs` | Replace ~505 manual calls with single macro invocation |
| `vm/mod.rs` | No change needed (already has `pub mod native_catalog`) |

## Verification

After each batch:
1. `cargo build --bin auto` — compile
2. `cargo test -p auto-lang --lib -- vm::` — VM tests (baseline: 320 pass)

After full migration:
3. `cargo test -p auto-lang --lib` — all tests (baseline: 3276 pass)
4. `cargo test -p auto-lang -- trans` — transpiler tests
5. Run `list_comprehensive.at` E2E test
