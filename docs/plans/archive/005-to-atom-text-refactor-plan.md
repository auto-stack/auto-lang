# Comprehensive Testing Plan for AST `to_atom()` Functions

## Objective

Write comprehensive markdown-based tests for all 33 `to_atom()` implementations to ensure they produce proper ATOM format (not just `fmt::Display` output).

## Current State

- **Existing Tests**: 4 files in `crates/auto-lang/test/ast/`
  - `01_literals.test.md` - int, float, str, fstr
  - `02_exprs.test.md` - unary, binary, index, slice (incomplete)
  - `03_functions.test.md` - function decl, C function, function call
  - `04_controls.test.md` - if, for, while

- **Problem**: Current `to_atom()` returns same output as `fmt::Display`, NOT proper ATOM node format

- **Coverage**: 33 `ToAtom` implementations exist, but only ~15 are indirectly tested

## Test Format

Each test follows this markdown format:
```markdown
## Test Name

input_code

---

expected_output
```

## Plan: Create 5 New Test Files

### 1. `05_types.test.md` (HIGH PRIORITY)

**Structs to test**: Type, Key, Pair, Member, TypeDecl

**Test Cases**:
- Type primitives: int, float, bool, str, void, ptr types
- Type arrays: `[int; 10]`
- Type generics: `List[T]`
- Key variants: named, integer, boolean, string
- Pair simple and nested
- Member with/without default values
- TypeDecl simple struct and generic

**Example**:
```markdown
## Type - Int

int

---

int

## Pair - Simple

name: value

---

pair(name("name"), ident(value))

## TypeDecl - Simple Struct

type Point {
    x: int
    y: int
}

---

type-decl(name="Point", kind="user") {
    member(name("x"), type(int))
    member(name("y"), type(int))
}
```

### 2. `07_advanced_control.test.md` (HIGH PRIORITY)

**Structs to test**: If, Branch, For, Iter, Break, Range, Is, IsBranch

**Test Cases**:
- If simple, if-else, multiple else-if
- Branch structure
- For range loops (exclusive and inclusive)
- Iter variants (name, index, ever)
- Break statement
- Range exclusive/inclusive
- Is pattern matching with eq/if/else branches

**Example**:
```markdown
## If - If-Else

if x > 0 {
    print("positive")
} else {
    print("non-positive")
}

---

if {
    branch(bina(">", x, int(0))) {
        call print(str("positive"))
    }
    else {
        call print(str("non-positive"))
    }
}

## Range - Exclusive

0..10

---

range(start=int(0), end=int(10), eq=false)
```

### 3. `08_statements.test.md` (HIGH PRIORITY)

**Structs to test**: Store, Body, Node, Fn, Param

**Test Cases**:
- Store: let, mut, const with type annotations
- Body: single and multiple statements
- Node: HTML-like nodes with props and children
- Fn: simple function, C function, method, lambda
- Param: with/without default values

**Example**:
```markdown
## Store - Let

let x = 42

---

let(name="x", type=int, expr=int(42))

## Fn - Simple Function

fn add(x int, y int) int {
    x + y
}

---

fn(name="add", kind="function", return=int) {
    param(name="x", type=int)
    param(name="y", type=int)
    body(expr=bina("+", ident(x), ident(y)))
}
```

### 4. `09_events.test.md` (MEDIUM PRIORITY)

**Structs to test**: Call, Args, Arg, OnEvents, Event, Arrow, CondArrow

**Test Cases**:
- Call: simple, named args, mixed args
- Args: empty, positional only, named only, mixed
- Arg: positional, named, pair
- OnEvents: simple on, multiple events
- Arrow: simple, with with clause
- CondArrow: conditional event routing

**Example**:
```markdown
## Call - With Named Args

printf(format="Hello %s", name="World")

---

call(name="printf") {
    pair(name="format", str("Hello %s"))
    pair(name="name", str("World"))
}

## Arrow - Simple

click => print("clicked")

---

arrow(from=event(click), to=call print(str("clicked")))
```

### 5. `06_declarations.test.md` (LOWER PRIORITY)

**Structs to test**: Use, Union, UnionField, Tag, TagField, EnumDecl, EnumItem, Alias

**Test Cases**:
- Use: simple import, with items, C header
- Union: simple and complex
- Tag: simple tag variants
- Enum: with and without explicit values
- Alias: type alias

**Example**:
```markdown
## Use - C Header

use c <stdio.h>

---

use(kind="c", path="stdio.h")

## Enum - Simple Enum

enum Option {
    Some = 1
    None = 0
}

---

enum(name="Option") {
    item(name="Some", value=1)
    item(name="None", value=0)
}
```

## ATOM Format Guidelines

Based on `docs/language/design/atom.md`:

1. **Basic Types**: `int`, `float`, `str`, `bool`, `null`
2. **Arrays**: `[1, 2, 3]`
3. **Objects**: `{a: 1, b: 2}`
4. **Nodes** (key feature):
   ```
   name(prop:"value") {
       child()
   }
   ```
5. **Pairs**: `key: value` (standalone or in objects)
6. **Nesting**: Use `{}` for child nodes

## Implementation Order

1. **Week 1**: `05_types.test.md` + `07_advanced_control.test.md` (HIGH priority)
2. **Week 2**: `08_statements.test.md` + `09_events.test.md` (MEDIUM priority)
3. **Week 3**: `06_declarations.test.md` (LOWER priority) + edge cases

## Edge Cases to Test

- Nested structures (functions in functions, nodes in nodes)
- Optional fields (Some vs None)
- Generic types with parameters
- Empty collections (empty arrays, empty bodies)
- Complex expressions deeply nested

## Critical Files

- **Test Location**: `D:\autostack\auto-lang\crates\auto-lang\test\ast\`
- **ATOM Spec**: `D:\autostack\auto-lang\docs\language\design\atom.md`
- **AST Implementations**: `D:\autostack\auto-lang\crates\auto-lang\src\ast\*.rs`

## Success Criteria

✅ All 33 `ToAtom` implementations have at least 1 test case
✅ Tests use proper ATOM node format (not Display format)
✅ 100% test pass rate
✅ Edge cases covered for complex structs
✅ Tests follow existing markdown format
