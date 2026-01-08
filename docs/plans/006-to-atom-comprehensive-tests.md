# Comprehensive Testing Plan for AST `to_atom()` Functions

## Objective

Write comprehensive markdown-based tests for all 33 `to_atom()` implementations to ensure they produce proper ATOM format with tree structure using `{}` brackets.

## Problem

Current `to_atom()` implementations return the same output as `fmt::Display`, which is NOT the proper ATOM node format. The ATOM format requires:
- Tree structure with `{}` brackets for child nodes
- Node name first, then optional primary property (usually id), then `()` for argument lists
- Following the syntax from `docs/language/design/atom.md`

## Current State

- **Existing Tests**: 4 files in `crates/auto-lang/test/ast/`
  - `01_literals.test.md` - int, float, str, fstr ✅
  - `02_exprs.test.md` - unary, binary, index, slice (incomplete) ✅ (now complete)
  - `03_functions.test.md` - function decl, C function, function call ✅
  - `04_controls.test.md` - if, for, while ✅

- **Coverage**: 33 `ToAtom` implementations exist

## ATOM Format Specification

From `docs/language/design/atom.md`:

```js
root(id:"123") {
    name("Puming")
    age(41)
}
```

**Key Rules**:
1. Node name first (e.g., `int`, `float`, `fn`, `if`, `call`)
2. Arguments in `()` - positional or named with `:`
3. Children in `{}` - nested nodes
4. Properties as named args: `prop(value)`
5. Arrays: `[1, 2, 3]` → `array(int(1), int(2), int(3))`
6. Objects: `{a: 1}` → `object { pair(name("a"), int(1)) }`

## Test Files Created

### 1. `05_types.test.md` ✅
**Structs**: Type, Key, Pair, Member, TypeDecl

**Test Cases**:
- Type primitives: int, float, bool, str, void, ptr
- Type arrays: `[int; 10]` → `array(int, 10)`
- Type user defined: `Point` → `Point`
- Key variants: named, integer, boolean, string
- Pair: `name: value` → `pair(name("name"), ident(value))`
- Member: `x: int = 42` → `member(name("x"), type(int), value(int(42)))`
- TypeDecl: structs with methods and generics

### 2. `07_advanced_control.test.md` ✅
**Structs**: If, Branch, For, Iter, Break, Range, Is, IsBranch

**Test Cases**:
- If: simple, if-else, multiple else-if
- Branch: condition with body
- For: range loops, inclusive, with index, infinite
- Iter: name, index, ever variants
- Break statement
- Range: exclusive `0..10` → `range(start(int(0)), end(int(10)), eq(false))`
- Is: pattern matching with eq/if/else branches

### 3. `08_statements.test.md` ✅
**Structs**: Store, Body, Node, Fn, Param

**Test Cases**:
- Store: let, mut, const with type annotations
- Body: single and multiple statements
- Node: HTML-like with props and children
- Fn: simple, no params, C function, method
- Param: with/without defaults

### 4. `09_events.test.md` ✅
**Structs**: Call, Args, Arg, OnEvents, Event, Arrow, CondArrow

**Test Cases**:
- Call: simple, named args, mixed, nested, method
- Args: empty, positional, named, mixed
- Arg: positional, named, pair expression
- OnEvents: simple and multiple events
- Event: simple and with properties
- Arrow: simple, with with clause, nested action
- CondArrow: conditional, multiple branches, with else

### 5. `06_declarations.test.md` ✅
**Structs**: Use, Union, UnionField, Tag, TagField, EnumDecl, EnumItem, Alias

**Test Cases**:
- Use: simple import, with items, C header, Rust import
- Union: simple and with complex fields
- Tag: simple and with data
- Enum: with and without explicit values
- Alias: simple and complex types

## Updated Existing Files

### `02_exprs.test.md` ✅ (Completed)
Added missing test cases:
- Slice expressions: `arr[1..10]` → `slice(ident(arr), int(1), int(10))`
- Array literals: `[1, 2, 3]` → `array(int(1), int(2), int(3))`
- Object literals: `{name: "John"}` → `object { pair(name("name"), str("John")) }`
- Lambda: `fn(x, y) { x + y }` → `lambda { param(name("x")) param(name("y")) body { binary("+", ident(x), ident(y)) } }`

## Test Coverage Summary

**Total Test Files**: 9
- `01_literals.test.md` - 4 tests
- `02_exprs.test.md` - 8 tests (was 3, added 5)
- `03_functions.test.md` - 3 tests
- `04_controls.test.md` - 3 tests
- `05_types.test.md` - 13 tests (NEW)
- `06_declarations.test.md` - 17 tests (NEW)
- `07_advanced_control.test.md` - 15 tests (NEW)
- `08_statements.test.md` - 14 tests (NEW)
- `09_events.test.md` - 17 tests (NEW)

**Total Test Cases**: ~94 tests

**Struct Coverage**: All 33 `ToAtom` implementations now have test coverage

## Next Steps

1. **Run Tests**: Verify all tests compile and can be parsed
2. **Fix `to_atom()` Implementations**: Update AST structs to output proper ATOM format
3. **Validate Output**: Ensure `to_atom()` returns strings matching test expectations
4. **CI Integration**: Add test runner to continuous integration

## Files Modified/Created

### Created:
- `D:\autostack\auto-lang\crates\auto-lang\test\ast\05_types.test.md`
- `D:\autostack\auto-lang\crates\auto-lang\test\ast\06_declarations.test.md`
- `D:\autostack\auto-lang\crates\auto-lang\test\ast\07_advanced_control.test.md`
- `D:\autostack\auto-lang\crates\auto-lang\test\ast\08_statements.test.md`
- `D:\autostack\auto-lang\crates\auto-lang\test\ast\09_events.test.md`

### Modified:
- `D:\autostack\auto-lang\crates\auto-lang\test\ast\02_exprs.test.md` (completed incomplete tests)

### Reference:
- `D:\autostack\auto-lang\docs\language\design\atom.md` - ATOM format specification

## Success Criteria

✅ All 33 `ToAtom` implementations have comprehensive test cases
✅ Tests use proper ATOM node format with `{}` tree structure
✅ Tests follow existing markdown format consistency
✅ All test files created and properly formatted
