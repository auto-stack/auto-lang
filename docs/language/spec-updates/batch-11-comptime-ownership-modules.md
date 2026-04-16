# Spec Update: Batch 11 — Compile-Time, Ownership, Modules

**Date**: 2026-04-16
**Plans Referenced**: Plan 095 (compile-time), Plan 122 (ownership trinity), Plan 131 (module paths), Plan 163 (pub keyword), Plan 167/168 (shared, pub migration)
**Source Files**: `ast/comptime.rs` (HashIf, HashFor, HashIs, HashBrace), `ast/store.rs` (StoreKind), `ast/fun.rs` (ParamMode, AccessMode)
**Sections Updated**: Compile-Time Metaprogramming (Section 18 — NEW), Ownership and Borrowing (Section 19 — NEW), Modules and Imports (Section 20 — NEW)

## Old Content

No dedicated sections for compile-time, ownership, or modules existed.

## New Content

### Compile-Time Metaprogramming (Section 18)
- `#if cond { } else { }` — compile-time conditional
- `#for var in iter { }` — compile-time loop unrolling
- `#is target { pattern => body }` — compile-time pattern match
- `#{ expr }` — compile-time expression evaluation
- All constructs use `HashIf`/`HashFor`/`HashIs`/`HashBrace` AST nodes

### Ownership and Borrowing (Section 19)
- Trinity of Resources: view (O(1) immutable), mut (O(1) mutable), move (O(1) transfer)
- clone (O(N) deep copy) as explicit operation
- Default parameter mode: view
- `.hold` for lifetime extension
- `AccessMode` enum: View, Mut, Move, Clone

### Modules and Imports (Section 20)
- `use` statements: relative, `pac.` root, `super.` parent
- `dep` dependency declarations in `pac.at`
- `pub` exports for visibility
- Resolution rules: file module, directory module, ambiguity error

## Notes

- Compile-time constructs use `#` prefix (Hash token), not to be confused with annotations `#[...]`
- `#ifx` is lexed as `Hash` + `Ident("ifx")`, NOT `HashIf` — boundary check in lexer
- StoreKind has 6 variants: Let, Var, Const, Shared, CVar, Field
- `shared` creates process-lifetime static storage (Plan 168)
