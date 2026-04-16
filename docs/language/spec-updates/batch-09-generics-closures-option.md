# Spec Update: Batch 09 — Generics, Closures, Option/Result

**Date**: 2026-04-16
**Plans Referenced**: Plan 048 (generics), Plan 049 (operators), Plan 052 (List/storage), Plan 057 (generic specs), Plan 059 (generic ext), Plan 060 (closures), Plan 120 (Option/Result), Plan 165 (struct patterns)
**Source Files**: `ast/types.rs` (GenericParam, TypeParam, ConstParam), `ast/fun.rs` (Closure, ClosureParam), `ast.rs` (Option/Result expr variants)
**Sections Updated**: Generics (Section 14 — NEW), Closures (Section 15 — NEW), Option and Result (Section 16 — NEW)

## Old Content

No dedicated sections for generics, closures, or Option/Result existed. These features were partially documented in Functions section or not at all.

## New Content

### Generics (Section 14)
- Type parameters: `<T>`
- Const parameters: `<N u32>` with default values
- Generic constraints: `<T has Comparable>`
- Monomorphization for C/Rust backends

### Closures (Section 15)
- Syntax forms: no-params `=> expr`, single `x => expr`, multi `(a, b) => expr`
- Typed closures: `(a int, b int) => int { ... }`
- Capture semantics: view (default) vs move
- Iterator usage with `.map()`, `.filter()`

### Option and Result (Section 16)
- `?T` (Option): Some(T) or None
- `!T` (Result): Ok(T) or Err(message)
- Constructors: Some, None, Ok, Err
- Null coalescing: `??`
- Error propagation: `.?`
- Pattern matching

## Notes

- Closures use `=>` syntax, NOT Rust's `|...|` syntax
- `?T` is syntactic sugar for `Option<T>`, `!T` for `Result<T>` (or `Result<T, E>`)
- GenericParam enum has two variants: Type (TypeParam) and Const (ConstParam)
- ConstParam supports default values: `N u32 = 64`
