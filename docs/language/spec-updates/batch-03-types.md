# Spec Update: Batch 03 — Types

**Date**: 2026-04-16
**Plans Referenced**: Plan 048 (generics), Plan 052 (List/storage), Plan 120 (Option/Result), Plan 121 (Task/Handle), Plan 155 (String), Plan 160 (Map)
**Source Files**: `ast/types.rs`
**Sections Updated**: Types (Section 5) — major rewrite

## Old Content

Old "Basic Types" table listed 15 types in a single flat table:
- Primitives: int, uint, byte, i8, i16, i64, u16, u64, float, double, bool, str, char, void, nil
- No compound types documented (arrays, slices, pointers were in "Type Modifiers" section)
- No Option, Result, Handle, linear, String, Map types

## New Content

### Primitive Types (15 → 16)
- Added `usize` (pointer-sized unsigned integer)

### String Types (new section)
- `str` — borrowed string slice
- `String` — owned dynamic string (Plan 155)
- `cstr` — C string, null-terminated

### Compound Types (new section)
- Static arrays: `[N]T` (e.g., `[10]int`)
- Runtime-sized arrays: `[expr]T` (Plan 052)
- Slices: `[]T`
- Lists: `List<T>` (growable, heap-backed)
- Maps: `Map<K, V>` (Plan 160)
- Pointers: `*T`
- References: `&T`

### Special Types (new section)
- Option: `?T` — `Some(T)` or `None` (Plan 120)
- Result: `!T` — `Ok(T)` or `Err(...)` (Plan 120)
- Handle: `Handle<T>` — task reference (Plan 121)
- Linear: `linear<T>` — move-only semantics
- Function types: `fn(params) ret`
- Generic instances: `MyType<T>`, `MyType<T, N u32>`

### Type System Stats
- Total Type enum variants: 37 (from `ast/types.rs`)
- All 37 variants now documented in spec

## Notes

- The old "Type Modifiers" section in "Type Definitions" (postfix `[]`, `*`, `&`, `?`) is superseded by the new compound types documentation here
- `str` with a length parameter (`Str(usize)`) in types.rs represents the literal string length for compile-time tracking; in user code it's just `str`
- `Unknown` type variant exists in types.rs but is internal (type inference fallback); not documented as user-facing
