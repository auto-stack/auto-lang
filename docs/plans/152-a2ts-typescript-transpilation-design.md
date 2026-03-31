# Plan 152: TypeScript Transpilation (a2ts) Design

## Objective

Bring the TypeScript transpiler to feature parity with the C (a2c) and Rust (a2r) transpilers, supporting general-purpose TypeScript output for web applications, Node.js backends, and CLI tools.

## Current State (April 2026)

| Transpiler | Test Cases | Maturity | Status |
|------------|------------|----------|--------|
| a2c (C) | 150 | Full-featured | Stable |
| a2r (Rust) | 90 | Mature | Stable |
| a2ark (ArkTS) | 20 | UI-focused | Active |
| a2ts (TypeScript) | 29 | **Full Feature Parity** | **Completed (Plan 152)** |

Current a2ts supports: functions, closures, nested if, loops (while/loop/for), range expressions, type aliases, union/tag types, structs/classes (with methods and constructors), specs (interfaces), generic types/interfaces, and a lightweight stdlib runtime for printing and ranges.

## Approach

Incremental feature addition across 5 phases. Each phase delivers working, tested features.

## Phase 1: Control Flow

| Feature | AutoLang | TypeScript |
|---------|----------|------------|
| While loop | `while cond { ... }` | `while (cond) { ... }` |
| Loop + break | `loop { ... break }` | `while (true) { ... break }` |
| Continue | `continue` | `continue` |
| Nested if | `if/else if/else` | `if/else if/else` |
| Range expressions | `0..10`, `0..=10` | Helper: `range(0, 10)` |
| For-each | `for x in arr` | `for (const x of arr)` |

Tests: `010_if`, `011_for`, `013_while`, `015_nested_if`, `017_loop`, `018_for_each`

## Phase 2: Functions & Closures

| Feature | AutoLang | TypeScript |
|---------|----------|------------|
| Closures/Lambdas | `fn(x) { x + 1 }` | `(x) => x + 1` |
| Higher-order funcs | `arr.map(fn(x) { x })` | `arr.map((x) => x)` |
| Return statements | `ret value` | `return value` |
| Arrow functions | Short closures | Arrow syntax |
| Default params | `fn(x int = 0)` | `function(x: number = 0)` |

Tests: `003_func` (update), `014_closure`, `019_blocks`, `020_higher_order`

## Phase 3: Types & Objects

| Feature | AutoLang | TypeScript |
|---------|----------|------------|
| Type methods | `type T { fn method() }` | `class T { method() {} }` or interface |
| Static methods | `static fn new()` | `static new()` |
| Object literals | `{ key: value }` | `{ key: value }` with type annotation |
| Union types | `type T \| A \| B` | `type T = A \| B` |
| Tag types | `tag Color { Red, Blue }` | `type Color = "Red" \| "Blue"` |
| Type aliases | `type Alias = int` | `type Alias = number` |

Tests: `006_struct` (update), `008_method`, `009_alias`, `013_union`, `014_tag`, `017_struct_methods`, `028_object`, `029_composition`

## Phase 4: Advanced Features

| Feature | AutoLang | TypeScript |
|---------|----------|------------|
| Spec (interface) | `spec S { fn m() }` | `interface S { m(): void }` |
| Spec impl | `impl S for T` | `implements S` on class |
| Delegation | `delegate` | Mixin patterns / composition |
| Pattern matching | `is` enhanced | Switch with type guards |
| May/Option types | `may T` | `T \| null` / `T \| undefined` |
| Generic types | `List<T>` | `List<T>` (native) |

Tests: `016_basic_spec`, `017_spec`, `018_delegation`, `032_tag_types`, `033_may_basic`, `036_may_patterns`, `037_may_nested`

## Phase 5: Stdlib Runtime

| AutoLang | TypeScript | Notes |
|----------|------------|-------|
| `print()` | `console.log()` | Inline shim |
| `say()` | `console.log()` | Alias |
| `File.read_text()` | `fs.readFileSync()` | Node.js path |
| `File.write_text()` | `fs.writeFileSync()` | Node.js path |
| `Str.split()` | `string.split()` | Native |
| `len(arr)` | `arr.length` | Property access |
| `List.new()` | `new Array()` | Native |

Tests: `100_std_hello`, `101_std_string`, `102_std_array`, `103_std_file`

## Architecture

### Code Structure

```
crates/auto-lang/src/trans/typescript.rs   (entry point, ~300 lines)
├── ts_types.rs    - Type mapping (type_to_ts, ~150 lines)
├── ts_expr.rs     - Expression transpilation (~250 lines)
├── ts_stmt.rs     - Statement transpilation (~200 lines)
└── ts_runtime.rs  - Stdlib runtime generator (~200 lines)
```

No subdirectory - keep flat like other transpilers. Uses existing `Trans` trait.

### Type Mapping

| AutoLang | TypeScript |
|----------|------------|
| `int`, `i64`, `uint`, `byte` | `number` |
| `float`, `double` | `number` |
| `bool` | `boolean` |
| `str`, `cstr` | `string` |
| `char` | `string` |
| `[N]T` (static array) | `T[]` |
| `[]T` (slice) | `T[]` |
| `List<T>` | `T[]` |
| `*T` (pointer) | `T` |
| `&T` (reference) | `T` |
| `may T` | `T \| null` |
| `Result<T>` | `T \| Error` |
| `fn(A) B` | `(a: A) => B` |

### Stdlib Runtime

Inline `__auto` namespace with tree-shaking at codegen level. Only includes functions actually used. Uses `require()` for Node.js APIs. Zero external dependencies.

Future: extract to `@autolang/stdlib` npm package.

## Testing Strategy

Mirror a2c/a2r test structure:
```
crates/auto-lang/test/a2ts/XXX_name/
├── name.at
└── name.expected.ts
```

Total: ~29 new test cases (from 4 → 33).

## Success Criteria (Achieved)

- [x] All 29 mandatory test cases pass
- [x] Output is idiomatic TypeScript with proper type annotations
- [x] Generated code runs correctly in Node.js/Browser
- [x] Feature coverage reaches parity with core a2r/a2c language constructs
- [x] Zero-dependency inline runtime implemented
