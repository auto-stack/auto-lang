# Spec Update: Batch 04 — Expressions

**Date**: 2026-04-16
**Plans Referenced**: Plan 026 (property keywords), Plan 060 (closures), Plan 095 (compile-time), Plan 120 (Option/Result), Plan 122 (ownership), Plan 124 (async), Plan 126 (micro-concurrency), Plan 161/162/165 (various expr enhancements)
**Source Files**: `ast.rs` (Expr enum, 55 variants)
**Sections Updated**: Expressions (Section 6) — major rewrite

## Old Content

Old Expressions section had 10 subsections covering basic expressions only:
- Literal, Identifier, Binary (Arithmetic/Comparison/Logical), Assignment, Range, F-String, Array, Object, Grouping

## New Content

Expanded to 22 subsections covering all 55 Expr variants:

- **Literal**: added `u`, `u8`, `i8`, `i64`, `d`, C-string (`c"..."`)
- **Arithmetic**: added modulo `%`
- **Logical**: added `and`/`or` keyword forms
- **Assignment**: added `%=`
- **Ownership**: `.view`, `.mut`, `.move` (Plan 122)
- **Type Conversion**: `.as(Type)`, `.to(Type)` (zero-cost reinterpret vs explicit conversion)
- **Option/Result**: `Some(v)`, `None`, `Ok(v)`, `Err(e)`, `??`, `.?`, `?.` (Plan 120)
- **Closures**: `(a, b) => expr`, `x => expr` (Plan 060)
- **If expressions**: as value-producing expressions
- **Block expressions**: last expression is the value
- **Null coalescing**: `??` operator
- **Error propagation**: `.?` operator
- **Smart pointers**: `Box(value)`, `Arc(value)`
- **Async**: `~{ }` blocks, `.await`, `.go` (Plans 124/126)
- **Compile-time**: `#{ expr }` (Plan 095)
- **Node**: widget construction
- **Grid**: tabular data
- **Hold**: lifetime extension
- **Precedence table**: 11 levels from highest to lowest

## Notes

- F-string examples updated to use `$name` (Auto syntax) not `{name}` (Rust syntax)
- `Take` expr variant is DEPRECATED — use `Move` instead
- `Cover`/`Uncover`/`OptionPattern`/`ResultPattern`/`StructPattern` variants are pattern-matching related; documented in Control Flow section
- `NavCall` variant is routing-specific; documented in UI/Routing section
- `GenName` is internal (generated names not in symbol table); not user-facing
