# Spec Update: Batch 06 — Functions

**Date**: 2026-04-16
**Plans Referenced**: Plan 035 (static methods), Plan 048 (generics), Plan 060 (closures), Plan 088 (param modes), Plan 122 (ownership), Plan 163 (pub/mut)
**Source Files**: `ast/fun.rs` (Fn, Param, ParamMode, Closure), `ast.rs`
**Sections Updated**: Functions (Section 9) — major rewrite

## Old Content

Old Functions section had 4 subsections:
- Function Definition (basic syntax)
- Function Calls
- Lambda Functions (using old `|a int, b int| (int) expr` syntax)
- Parameter Passing Modes (using `ref`/`mut ref`/`move` prefix syntax)

Issues:
- Used `println()` instead of `print()`
- Lambda syntax was wrong (used `|...|` instead of `=>`)
- Parameter modes used prefix keywords (`ref`, `mut ref`, `move`)
- No generic functions, no static methods, no pub visibility, no annotations

## New Content

### Function Definition
- Fixed to use `print()` instead of `println()`
- Noted space-separated return type (no `->`)

### Generic Functions (Plan 048)
- `fn identity<T>(x T) T { x }`
- Type constraints via `has` keyword

### Parameter Modes (Plan 088)
- Updated to `ParamMode` enum: View (default), Mut, Move
- Deprecated: Copy, Take
- No more `ref`/`mut ref` prefix syntax

### Static Methods (Plan 035)
- `static fn new()` syntax
- `self` keyword for instance methods

### Public Visibility (Plan 163)
- `pub fn`, `pub type`, `pub enum`, `pub spec` keyword prefix

### Closures (Plan 060)
- Updated to `(a, b) => expr` syntax (was `|...|`)
- Single-param: `x => expr`
- Capture semantics documented

### Function Annotations
- `#[vm]`, `#[c]`, `#[c, vm]`

### Function Kinds
- 5 kinds: Function, Lambda, Method, CFunction, VmFunction

### Default Parameters
- `fn greet(name str, greeting str = "Hello") str`

### Mutable Methods (Plan 163)
- `mut fn increment()` syntax

## Notes

- Old lambda syntax `|a, b| expr` is NOT used in Auto. Use `(a, b) => expr`.
- `self` is implicit in methods (no need to declare as parameter)
- `FnKind::VmFunction` is for functions implemented via VM registry, not declared by users
