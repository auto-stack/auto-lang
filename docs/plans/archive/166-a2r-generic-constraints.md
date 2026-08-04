# Plan 166: a2r — Generic Constraints (`#[with(T as Trait)]` → `<T: Trait>`)

**Date**: 2026-04-14
**Status**: Design approved
**Scope**: a2r transpiler only
**Parent**: Plan 159 Phase 6B-3.1

## Goal

Emit generic type parameters and constraints from `#[with(T as Trait)]` annotations in a2r transpiler output.

## Background

The parser already handles `#[with(T as Spec)]` (Plan 061). The AST stores it in `Fn.type_params: Vec<TypeParam>` where `TypeParam` has `name` and `constraint: Option<Box<Type>>`. The a2r transpiler currently ignores `type_params` entirely — this plan adds emission.

## Syntax

```auto
#[with(T)]
fn identity(x T) T { x }

#[with(T as Clone)]
fn duplicate(x T) T { x }

#[with(A, B as Eq)]
fn compare(a A, b B) A { a }
```

Maps to:

```rust
fn identity<T>(x: T) -> T { x }
fn duplicate<T: Clone>(x: T) -> T { x }
fn compare<A, B: Eq>(a: A, b: B) -> A { a }
```

## What Changes

### Transpiler (`trans/rust.rs` — `fn_decl()`)

In `fn_decl()`, after writing `fn name`, emit `<T, U: Constraint>` from `fn_decl.type_params`.

### Test

New a2r test: `155_with_constraint/`

## Files to Modify

| File | Change |
|------|--------|
| `crates/auto-lang/src/trans/rust.rs` | Emit type_params in `fn_decl()` |
| `crates/auto-lang/test/a2r/155_with_constraint/` | New test case |
