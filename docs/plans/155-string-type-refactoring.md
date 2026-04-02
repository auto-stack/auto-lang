# Plan 155: AutoLang String Types Refactoring

This plan outlines the steps to introduce a three-tier string type system in AutoLang: string literals (`StrLit`), borrowed string slices (`StrSlice`), and owned dynamic strings (`String`).

## Goal

Refactor AutoLang's string type system to cleanly separate three string kinds:

| Auto Syntax | AST Type | Rust Equivalent | Description |
|---|---|---|---|
| `"hello"` | `Type::StrLit(usize)` | `&'static str` | Compile-time string literal with known length |
| `str` | `Type::StrSlice` | `&str` | Borrowed string slice (default string type) |
| `String` | `Type::String` *(new)* | `String` | Owned, growable dynamic string |

### Design Principles

- **No explicit reference syntax**: Auto does not have `&` syntax. The `str` keyword in Auto directly means a borrowed slice (Rust's `&str`). Ownership modifiers use `view` or `mut` keywords instead.
- **`str` is the default string type**: When users write `str`, they get a borrowed slice. This is the most common string type.
- **`String` is owned and dynamic**: Like Rust's `String`, it can grow at runtime. It has no compile-time length parameter.
- **Literals are `StrLit`**: String literals `"hello"` have compile-time known lengths, tracked via `StrLit(usize)` in the type system.
- **Compiler internals use descriptive names**: `StrSlice` (not just `Str`) in the compiler, even though users write `str`.

## Proposed Changes

### 1. AST Type System — [ast/types.rs](crates/auto-lang/src/ast/types.rs)

**Rename** `Type::Str(usize)` → `Type::StrLit(usize)`:
- String literals have compile-time known lengths
- The `usize` parameter tracks the literal's byte length

**Keep** `Type::StrSlice` unchanged:
- Already correctly represents borrowed slices
- Maps to Auto keyword `str` in the parser

**Add** `Type::String` *(new variant)*:
- Owned, growable dynamic string
- No `usize` parameter (size is runtime)
- Maps to Auto keyword `String`

**Add helper method** `is_any_string()`:
```rust
pub fn is_any_string(&self) -> bool {
    matches!(self, Type::StrLit(_) | Type::StrSlice | Type::String)
}
```

**Update** `unique_name`, `fmt::Display`, `default_value`, `to_value_type` for all three variants.

### 2. Expression AST — [ast.rs](crates/auto-lang/src/ast.rs)

**Rename** `Expr::Str(AutoStr)` → `Expr::StrLit(AutoStr)`:
- Represents a string literal expression `"hello"`
- Name aligns with `Type::StrLit`

### 3. Value System — [auto-val/src/value.rs](crates/auto-val/src/value.rs)

**Rename** `Value::Str(AutoStr)` → `Value::StrLit(AutoStr)`:
- Runtime value for string literals
- Uses reference-counted `AutoStr` (EcoString)

**Rename** `Value::OwnedStr(Str)` → `Value::String(Str)`:
- Owned dynamic string value
- Aligns with `Type::String`

**Keep** `Value::StrSlice(StrSlice)` unchanged.
**Keep** `Value::CStr(CStr)` unchanged.

### 4. Value Type Enum — [auto-val/src/types.rs](crates/auto-val/src/types.rs)

**Rename** `Type::Str` → `Type::StrLit`.
**Keep** `Type::StrSlice` unchanged.
**Add** `Type::String` *(new)*.

### 5. Parser — [parser.rs](crates/auto-lang/src/parser.rs)

**Update** `lookup_type` to map:
- `"str"` → `Type::StrSlice` (borrowed slice, default)
- `"String"` / `"string"` → `Type::String` (owned dynamic)

String literals `"hello"` in expressions continue to produce `Expr::StrLit`.

### 6. Type Inference — [infer/expr.rs](crates/auto-lang/src/infer/expr.rs)

**Update** expression inference:
```rust
Expr::StrLit(s) => Type::StrLit(s.len()),
```

**Update** binary operation inference:
- String concatenation (`+`) involving any string type → `Type::String` (owned result)
- `StrLit + StrLit` → `Type::String` (concatenation produces owned)

### 7. Type Unification — [infer/unification.rs](crates/auto-lang/src/infer/unification.rs)

**Update** `unify_with_coercion` to support:
- `Type::String` coerces to `Type::StrSlice` (owned → borrowed, implicit)
- `Type::StrLit(_)` coerces to `Type::StrSlice` (literal → slice, implicit)
- `Type::StrLit(_)` coerces to `Type::String` (literal → owned, implicit)

### 8. VM Runtime — [vm/codegen.rs](crates/auto-lang/src/vm/codegen.rs)

**Update** `is_string_expr` to check for all three: `Type::StrLit(_)`, `Type::StrSlice`, `Type::String`.
**Ensure** `STR_CAT` bytecode produces `Type::String` result.

### 9. VM Native Functions — [vm/native.rs](crates/auto-lang/src/vm/native.rs)

**Register** `to_string` for all string types:
- `StrLit.to_string()` → `String`
- `StrSlice.to_string()` → `String`

### 10. Transpilers

**C Transpiler** ([trans/c.rs](crates/auto-lang/src/trans/c.rs)):
- `Type::StrLit(_)` → `const char*` (literal, immutable)
- `Type::StrSlice` → `const char*` with length tracking
- `Type::String` → dynamic `char*` with management functions

**Rust Transpiler** ([trans/rust.rs](crates/auto-lang/src/trans/rust.rs)):
- `Type::StrLit(_)` → `&'static str`
- `Type::StrSlice` → `&str`
- `Type::String` → `String`

## Task Checklist

### Phase 1: Core Renames (breaking, do together)
- `[ ]` Rename `Type::Str(usize)` → `Type::StrLit(usize)` in `ast/types.rs`
- `[ ]` Rename `Expr::Str(AutoStr)` → `Expr::StrLit(AutoStr)` in `ast.rs`
- `[ ]` Rename `Value::Str(AutoStr)` → `Value::StrLit(AutoStr)` in `auto-val/src/value.rs`
- `[ ]` Rename `auto_val::Type::Str` → `auto_val::Type::StrLit` in `auto-val/src/types.rs`
- `[ ]` Fix all compile errors from renames (use `replace_all` across codebase)

### Phase 2: Add Type::String
- `[ ]` Add `Type::String` variant to `ast/types.rs`
- `[ ]` Add `Type::String` to `auto_val::Type` in `auto-val/src/types.rs`
- `[ ]` Rename `Value::OwnedStr(Str)` → `Value::String(Str)`
- `[ ]` Update parser: `"str"` → `Type::StrSlice`, `"String"` → `Type::String`

### Phase 3: Type Inference & Coercion
- `[ ]` Update `infer/expr.rs`: string concatenation → `Type::String`
- `[ ]` Update `infer/unification.rs`: coercion rules for StrLit/StrSlice/String
- `[ ]` Add `is_any_string()` helper method

### Phase 4: VM & Codegen
- `[ ]` Update `vm/codegen.rs`: `is_string_expr` for all three types
- `[ ]` Update `vm/native.rs`: `to_string` registration
- `[ ]` Update monomorphizer and pattern matcher for new type names

### Phase 5: Transpilers & Tests
- `[ ]` Update C transpiler for three string types
- `[ ]` Update Rust transpiler for three string types
- `[ ]` Update ArkTS, Jet, Python transpilers
- `[ ]` Run `cargo test -p auto-lang` — fix all failures
- `[ ]` Run `cargo test -p auto-val` — fix all failures
- `[ ]` Run full test suite

## Verification Plan

### Automated Tests
- `cargo test -p auto-lang`
- `cargo test -p auto-val`
- `cargo run -p auto -- crates/auto-lang/tests/string_test.at`

### Manual Verification
- String literals work as before
- `str` type annotations map to borrowed slices
- `String` type works as owned dynamic string
- String concatenation produces `String`
- Implicit coercion: `String` → `str`, `StrLit` → `str`

## Naming Summary

| What | Before (current) | After (this plan) |
|---|---|---|
| Literal type | `Type::Str(usize)` | `Type::StrLit(usize)` |
| Literal expr | `Expr::Str(AutoStr)` | `Expr::StrLit(AutoStr)` |
| Literal value | `Value::Str(AutoStr)` | `Value::StrLit(AutoStr)` |
| Literal val type | `auto_val::Type::Str` | `auto_val::Type::StrLit` |
| Borrowed slice type | `Type::StrSlice` | `Type::StrSlice` *(unchanged)* |
| Borrowed slice value | `Value::StrSlice` | `Value::StrSlice` *(unchanged)* |
| Owned dynamic type | *(none)* | `Type::String` *(new)* |
| Owned dynamic value | `Value::OwnedStr(Str)` | `Value::String(Str)` *(renamed)* |
| C string type | `Type::CStr` | `Type::CStr` *(unchanged)* |
| Auto keyword `str` | → `Type::Str(0)` | → `Type::StrSlice` |
| Auto keyword `String` | *(not supported)* | → `Type::String` |
