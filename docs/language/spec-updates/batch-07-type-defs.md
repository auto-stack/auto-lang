# Spec Update: Batch 07 — Type Definitions, Enums, Tags

**Date**: 2026-04-16
**Plans Referenced**: Plan 019 (specs), Plan 021 (enums), Plan 035 (static methods), Plan 048 (generics), Plan 057 (generic specs), Plan 059 (ext blocks), Plan 163 (pub)
**Source Files**: `ast/types.rs` (TypeDecl, GenericParam), `ast/enums.rs` (EnumDecl, EnumKind), `ast/ext.rs` (Ext)
**Sections Updated**: Type Definitions (Section 11), Enums (Section 12 — NEW), Unions and Tags → merged into Enums

## Old Content

### Type Definitions
- "Type Modifiers" section with postfix syntax: `int[]`, `int*`, `int&`, `int?`
- Basic `type Point { x int, y int }` definitions
- Type aliases with `alias`

### Unions and Tags
- C-style unions and tagged unions in one section
- Tags pattern matching used `->` (incorrect)

## New Content

### Type Definitions (rewritten)
- Removed old "Type Modifiers" section (postfix syntax now documented in Types section)
- Added single inheritance: `type Dog is Animal { ... }`
- Added generic types: `type Container<T>`, const generics `type Inline<T, N u32>`
- Added spec implementation: `type X has SpecY`
- Added `ext` blocks: `ext str { fn method() { ... } }`
- Added TypeDecl fields table (internal reference)
- Fixed constructor syntax: `Point(10, 20)` not `{ x: 10, y: 20 }`

### Enums (NEW section)
- Three enum kinds: Scalar, Homogeneous, Heterogeneous (ADT)
- Scalar with repr type: `enum HttpStatus u16 { OK = 200 }`
- Generic enums: `enum Option<T>`, `enum Result<T, E>`
- EnumDecl fields table
- Tags moved here as a subsection
- Fixed pattern matching arrow: `->` → `=>`

## Notes

- Old "Unions and Tags" section replaced by new Enums section (Section 12)
- C-style unions (`union MyUnion { ... }`) still documented but de-emphasized
- Tags are a subset of the enum system (tagged unions)
