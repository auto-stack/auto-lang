# Auto Language Specification

**Version**: 0.2
**Status**: Draft
**Last Updated**: 2026-04

## Table of Contents

1. [Introduction](#introduction)
2. [Language Overview](#language-overview)
3. [Lexical Structure](#lexical-structure)
4. [Grammar (EBNF)](#grammar-ebnf)
5. [Types](#types)
6. [Expressions](#expressions)
7. [Statements](#statements)
8. [Control Flow](#control-flow)
9. [Functions](#functions)
10. [Data Structures](#data-structures)
11. [Type Definitions](#type-definitions)
12. [Enums](#enums)
13. [Specs (Traits)](#specs-traits)
14. [Generics](#generics)
15. [Closures](#closures)
16. [Option and Result](#option-and-result)
17. [Concurrency: Tasks and Async](#concurrency-tasks-and-async)
18. [Compile-Time Metaprogramming](#compile-time-metaprogramming)
19. [Ownership and Borrowing](#ownership-and-borrowing)
20. [Modules and Imports](#modules-and-imports)
21. [UI Widgets and Routing](#ui-widgets-and-routing)
22. [Nodes (Atom Format)](#nodes-atom-format)
23. [Memory Management](#memory-management)
24. [Implementation Comparison](#implementation-comparison)

---

## Introduction

Auto is a multi-paradigm programming language designed for automation and flexibility. It provides a unique blend of static and dynamic typing with multiple language variants:

- **AutoLang**: Static/dynamic hybrid that transpiles to C and Rust
- **AutoScript**: Dynamic scripting language for embedding
- **AutoConfig**: JSON superset for configuration
- **AutoShell**: Cross-platform shell scripting
- **AutoDSL**: Domain-specific language for UI applications

### Design Philosophy

Auto is designed around four core principles:

1. **Flexibility**: Support multiple programming paradigms and use cases
2. **Safety**: Memory safety with both manual and automatic management options
3. **Performance**: Zero-cost abstractions and efficient compilation
4. **Interoperability**: Seamless integration with C and Rust ecosystems

### Implementations

Auto has three implementations:
1. **Rust Implementation** (`crates/auto-lang/`): Reference implementation, hand-written
2. **C Implementation** (`autoc/`): AI-generated, portable, incomplete but functional
3. **Self-Hosted** (`auto/`): Compiler written in Auto itself (early stage)

---

## Language Overview

### Hello World

```auto
// Traditional approach
fn main() {
    print("Hello, World!")
}

// Expression-based (script mode)
print("Hello, World!")
```

### Key Features

- **Six Storage Modifiers**: `let`, `var`, `const`, `mut`, `shared`, `static` for different mutability and lifetime semantics
- **Type Inference**: Automatic type deduction with optional explicit annotations
- **Pattern Matching**: Powerful `is` expression for pattern matching with struct destructuring
- **F-Strings**: First-class string interpolation with embedded expressions
- **Ranges**: First-class range expressions `0..10` and `0..=10`
- **Option/Result**: `?T` for optional values, `!T` for error-propagating results (Plan 120)
- **Generics**: Parameterized types and functions with type and const parameters
- **Specs (Traits)**: Interface-like type constraints with polymorphic dispatch
- **Ownership Trinity**: `view`/`mut`/`move` resource access semantics (Plan 122)
- **Concurrency**: Task/Actor model with `spawn`/`send`, async/await (Plans 121/124/126)
- **Compile-Time**: `#if`/`#for`/`#is`/`#{}` for metaprogramming (Plan 095)
- **C/Rust Interop**: Seamless integration with C and Rust code

---

## Lexical Structure

### Source Code Representation

Auto source files use the `.at` extension and are encoded as UTF-8.

### Whitespace

- **Spaces**: Regular space character (U+0020)
- **Tabs**: Tab character (U+0009)
- **Newlines**: Line feed (U+000A) or carriage return + line feed (U+000D U+000A)

Newlines are significant in Auto - they act as statement terminators (like semicolons in other languages).

### Comments

```auto
// Single-line comment

/// Doc comment

/*
   Multi-line comment
   Spans multiple lines
*/
```

### Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, underscores, or hyphens.

```
identifier → (letter | "_") (letter | digit | "_" | "-")*
```

Hyphens are allowed within identifiers (no surrounding spaces): `preview-card` is a single identifier. With spaces, `a - b` is subtraction.

**Examples**: `foo`, `_bar`, `data123`, `my_variable`, `preview-card`

### Keywords

Auto reserves **56 keywords** (from `token.rs`), organized by category:

**Declarations**: `fn`, `let`, `mut`, `const`, `var`, `type`, `union`, `enum`, `tag`, `alias`, `spec`, `ext`, `static`, `shared`, `impl`, `node`
**Control Flow**: `if`, `else`, `for`, `when`, `break`, `is`, `in`, `on`, `as`, `to`
**Ownership**: `view`, `mut`, `move`, `copy`, `take`, `hold`
**Literals**: `true`, `false`, `nil`, `null`
**Option/Result**: `None`, `Some`, `Ok`, `Err`
**Concurrency**: `task`, `spawn`, `await`, `reply`, `go`
**Modules**: `use`, `pac`, `super`, `dep`, `has`
**Boolean Logic**: `and`, `or`
**UI/Routing**: `routes`, `outlet`, `link`, `route`, `nav`
**Other**: `grid`

### Operators and Punctuation

#### Arithmetic Operators

| Operator | Description |
|----------|-------------|
| `+` | Addition |
| `-` | Subtraction/negation |
| `*` | Multiplication |
| `/` | Division |
| `%` | Modulo |

#### Comparison Operators

| Operator | Description |
|----------|-------------|
| `==` | Equal comparison |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |

#### Assignment Operators

| Operator | Description |
|----------|-------------|
| `=` | Assignment |
| `+=` | Addition assignment |
| `-=` | Subtraction assignment |
| `*=` | Multiplication assignment |
| `/=` | Division assignment |
| `%=` | Modulo assignment |

#### Logical Operators

| Operator | Description |
|----------|-------------|
| `!` | Logical NOT |
| `&&` | Logical AND (also: `and` keyword) |
| `||` | Logical OR (also: `or` keyword) |

#### Range Operators

| Operator | Description |
|----------|-------------|
| `..` | Range (exclusive) |
| `..=` | Range (inclusive) |

#### Null-Safe / Error Propagation Operators (Plan 120)

| Operator | Description |
|----------|-------------|
| `?` | Question mark (Option/Result suffix) |
| `??` | Null coalescing |
| `?.` | Safe navigation / null-safe access |
| `.?` | Error propagation |

#### Ownership Dot-Operators (Plan 122)

| Operator | Description |
|----------|-------------|
| `.view` | Immutable borrow (`&T`) |
| `.mut` | Mutable borrow (`&mut T`) |
| `.move` | Ownership transfer |
| `.take` | DEPRECATED — use `.move` |

#### Arrow Operators

| Operator | Description |
|----------|-------------|
| `->` | Arrow: function return type annotation (`fn foo() -> int`), event routing (`src -> dest`) |
| `=>` | Double arrow: closures (`x => expr`) |
| `->` | Arrow: pattern match branches (`42 -> body`), event routing (`src -> dest`) |

#### Punctuation

| Symbol | Description |
|--------|-------------|
| `.` | Member access |
| `:` | Colon (field-value pairs, type annotations) |
| `|` | Vertical bar |
| `@` | At sign (annotations) |
| `#` | Hash sign (annotations, compile-time) |
| `~` | Tilde (async/future type marker) |
| `,` | Comma |
| `;` | Semicolon |
| `(` `)` | Parentheses |
| `[` `]` | Square brackets |
| `{` `}` | Curly braces |

#### Compile-Time Tokens (Plan 095)

| Token | Description |
|-------|-------------|
| `#if` | Compile-time conditional |
| `#for` | Compile-time loop |
| `#is` | Compile-time pattern match |
| `#{` | Compile-time expression block |

### Literals

#### Integer Literals

```
integer → digit+ ("u" | "u8" | "i8")?
       | "0x" hex_digit+          // hexadecimal
       | digit ("_" digit)* digit? // underscore separators
```

```auto
42          // i32 (default)
42u         // u32
42u8        // u8
42i8        // i8
0x1A        // hexadecimal (26)
1_000_000   // underscore separators
```

#### Floating-Point Literals

```
float → digit+ "." digit* ("f" | "d")? (exponent)?
     | digit+ ("f" | "d")              // suffix on integer
exponent → ("e" | "E") ("+" | "-")? digit+
```

```auto
3.14        // float (default)
3.14f       // float (explicit suffix)
3.14d       // double
3.14e-10    // scientific notation
1.0e5       // scientific notation
42f         // float from integer
42d         // double from integer
```

#### String Literals

```
string       → '"' .* '"'
multi_string → '"""' .* '"""'
cstr         → 'c' '"' .* '"'
```

```auto
"Hello, World!"           // Regular string (with length)
"""Hello
World"""                  // Multi-line string (Plan 169)
c"Hello, World!"          // C string (null-terminated, no escape processing)
```

Multi-line strings (triple-quoted, Plan 169):
- Preserve literal newlines
- Allow embedded 1-2 consecutive double quotes
- Process standard escape sequences (`\n`, `\t`, etc.)

#### Character Literals

```
char → "'" (letter | digit | escape_sequence) "'"
```

```auto
'a'         // Simple character
'\n'        // Escape sequence
'Z'
'0'
```

#### Boolean Literals

```auto
true
false
```

#### Nil/Null Literals

```auto
nil    // Auto's zero-size type
null   // Nullable pointer value
```

---

## Grammar (EBNF)

### Notation

The following EBNF notation is used:
- `*` - zero or more
- `+` - one or more
- `?` - optional (zero or one)
- `|` - alternation
- `()` - grouping
- `[]` - character class

### Complete Grammar

```
Program        → (Stmt)*

Stmt           → VarStmt
               | IfStmt
               | ForStmt
               | WhileStmt
               | BlockStmt
               | ExprStmt
               | UseStmt
               | FnStmt

VarStmt        → ("let" | "mut" | "const" | "var") IDENTIFIER Type? "=" Expr

IfStmt         → "if" Expr Stmt ("else" Stmt)?

ForStmt        → "for" IDENTIFIER? "in" Expr Stmt

WhileStmt      → "while" Expr Stmt

BlockStmt      → "{" (Stmt)* "}"

ExprStmt       → Expr

UseStmt        → "use" STRING

FnStmt         → "fn" IDENTIFIER "(" ParamList? ")" Type? BlockStmt

ParamList      → Param ("," Param)*

Param          → IDENTIFIER Type

Expr           → Assignment

Assignment     → LogicalOr (("=" | "+=" | "-=" | "*=" | "/=") LogicalOr)?

LogicalOr      → LogicalAnd ("||" LogicalAnd)*

LogicalAnd     → Equality ("&&" Equality)*

Equality       → Comparison (("==" | "!=") Comparison)*

Comparison     → Range (("<" | ">" | "<=" | ">=") Range)?

Range          → Additive (".." | "..=") Additive?

Additive       → Multiplicative (("+" | "-") Multiplicative)*

Multiplicative → Unary (("*" | "/") Unary)*

Unary          → ("!" | "-" | "+") Unary
               | Call

Call           → Primary ("(" ArgList? ")" | "[" Expr "]" | "." IDENTIFIER)*

ArgList        → Expr ("," Expr)*

Primary        → INTEGER
               | FLOAT
               | STRING
               | CHAR
               | "true"
               | "false"
               | "nil"
               | "null"
               | IDENTIFIER
               | ArrayExpr
               | ObjectExpr
               | FStrExpr
               | RangeExpr
               | "(" Expr ")"
               | "is" PatternMatch

ArrayExpr      → "[" (Expr ("," Expr)*)? "]"

ObjectExpr     → "{" (IdentValuePair ("," IdentValuePair)*)? "}"

IdentValuePair → IDENTIFIER ":" Expr

FStrExpr       → "f" '"' FStrPart* '"'

FStrPart       → STRING
               | "{" Expr "}"

RangeExpr      → Expr (".." | "..=") Expr?

PatternMatch   → Expr "{" (PatternBranch ("," PatternBranch)*)? "}"

PatternBranch  → Pattern "->" Expr

Pattern        → Literal
               | IDENTIFIER
               | "as" Type
               | "in" Range
               | Condition
```

---

## Types

### Type System Overview

Auto has a hybrid type system:
- **Static mode**: Type checking at compile time (default for AutoLang)
- **Dynamic mode**: Type checking at runtime (script mode with `var`)

The type system includes 37 type variants from `ast/types.rs`, covering primitives, compound types, generics, and special types.

### Primitive Types

| Auto Type | C Type | Rust Type | Description |
|-----------|---------|-----------|-------------|
| `int` | `int32_t` | `i32` | Signed 32-bit integer |
| `uint` | `uint32_t` | `u32` | Unsigned 32-bit integer |
| `byte` | `uint8_t` | `u8` | 8-bit unsigned integer |
| `i8` | `int8_t` | `i8` | 8-bit signed integer |
| `i16` | `int16_t` | `i16` | 16-bit signed integer |
| `i64` | `int64_t` | `i64` | 64-bit signed integer |
| `u16` | `uint16_t` | `u16` | 16-bit unsigned integer |
| `u64` | `uint64_t` | `u64` | 64-bit unsigned integer |
| `usize` | `size_t` | `usize` | Pointer-sized unsigned integer |
| `float` | `double` | `f64` | 64-bit floating-point |
| `double` | `double` | `f64` | Alias for float |
| `bool` | `bool` | `bool` | Boolean (true/false) |
| `char` | `char` | `char` | Single character |
| `void` | `void` | `()` | Unit/void type |
| `nil` | (no equivalent) | `!` | Zero-size type |

### String Types

| Auto Type | C Type | Rust Type | Description |
|-----------|---------|-----------|-------------|
| `str` | `struct { len; data; }` | `&str` | String slice with length |
| `String` | `char*` (dynamic) | `String` | Owned dynamic string (Plan 155) |
| `cstr` | `const char*` | `&CStr` | C string (null-terminated) |

### Compound Types

#### Array Types

```auto
[N]T        // Static array: fixed size N, type T (e.g., [10]int)
[expr]T     // Runtime-sized array: size determined at runtime (Plan 052)
[]T         // Slice: borrowed view into an array
```

```auto
let fixed [5]int = [1, 2, 3, 4, 5]
let slice []int = fixed[1..3]    // borrowed slice
```

#### List Type (Dynamic)

```auto
List<T>     // Growable list (heap-backed)
```

```auto
let list = List.new()
list.push(1)
list.push(2)
let len = list.len()    // 2
```

#### Map Type (Plan 160)

```auto
Map<K, V>   // Typed key-value dictionary
```

```auto
let scores Map<str, int> = Map.new()
scores.set("Alice", 95)
```

#### Pointer and Reference Types

```auto
*T          // Raw pointer
&T          // Reference (immutable borrow)
```

```auto
let val = 42
let ptr *int = &val
let ref &int = val.view
```

#### Option Type (Plan 120)

```auto
?T          // Optional value: Some(T) or None
```

```auto
let name ?str = Some("Alice")
let empty ?str = None
```

#### Result Type (Plan 120)

```auto
!T          // Error-propagating: Ok(T) or Err(...)
```

```auto
fn divide(a int, b int) !int {
    if b == 0 {
        return Err("division by zero")
    }
    Ok(a / b)
}
```

#### Handle Type (Plan 121)

```auto
Handle<T>   // Reference to a running task
```

```auto
let handle Handle<CounterTask> = spawn CounterTask()
handle.send(Increment(1))
```

#### Linear Type

```auto
linear<T>   // Move-only semantics (no implicit copy)
```

#### Function Types

```auto
fn(params) ret   // Function type signature
```

```auto
let callback fn(int, int) int = add
```

#### Generic Type Instances

```auto
MyType<T>           // Generic type with type parameter
MyType<T, N u32>    // Generic type with type + const parameter
```

### Type Annotations

```auto
// Type inference
let x = 42

// Explicit type annotation (space-separated)
let y int = 42

// Function with type annotations (space-separated)
fn add(a int, b int) int {
    a + b
}
```

### Type Coercion

Auto performs automatic type coercion for assignments:

```auto
let b byte = 42    // OK: int coerced to byte
let i int = b      // OK: byte promoted to int
```

---

## Expressions

Auto supports 55 expression types (from `ast.rs` Expr enum). This section documents all major expression categories.

### Literal Expressions

```auto
42          // int (i32)
42u         // uint (u32)
42u8        // byte (u8)
42i8        // i8
3.14        // float
3.14d       // double
'x'         // character
"hello"     // string
c"hello"    // C string (null-terminated)
true        // boolean
false       // boolean
nil         // nil value
```

### Identifier Expressions

```auto
x
my_variable
_underscore
preview-card    // hyphens allowed within identifiers
```

### Arithmetic Expressions

```auto
let sum = 10 + 5
let diff = 10 - 5
let product = 10 * 5
let quotient = 10 / 5
let remainder = 10 % 5
```

### Comparison Expressions

```auto
10 == 5    // false
10 != 5    // true
10 < 5     // false
10 > 5     // true
10 <= 10   // true
10 >= 10   // true
```

### Logical Expressions

```auto
true && false    // false
true || false    // true
!true            // false
true and false   // keyword form
true or false    // keyword form
```

### Assignment Expressions

```auto
x = 10
x += 5    // x = x + 5
x -= 5    // x = x - 5
x *= 5    // x = x * 5
x /= 5    // x = x / 5
x %= 5    // x = x % 5
```

### Range Expressions

```auto
// Exclusive range: 0, 1, 2, 3, 4
let r1 = 0..5

// Inclusive range: 0, 1, 2, 3, 4, 5
let r2 = 0..=5

// Using ranges in for loops
for i in 0..10 {
    print(i)
}
```

### F-String Expressions

```auto
let name = "World"
let msg = f"Hello, $name!"         // "Hello, World!"

let a = 5
let b = 10
let result = f"${a} + ${b} = ${a + b}"  // "5 + 10 = 15"

// Backtick syntax
let msg2 = `Hello, ${name}!`
```

### Array Expressions

```auto
let arr = [1, 2, 3, 4, 5]
let empty = []
let strings = ["hello", "world"]
let first = arr[0]           // 1
let slice = arr[1..3]        // [2, 3]
```

### Object Expressions

```auto
let obj = {
    name: "John",
    age: 30,
    active: true
}
let name = obj.name          // "John"
```

### Grouping Expressions

```auto
let result = (2 + 3) * 5    // 25
```

### Unary Expressions

```auto
let neg = -x        // negation
let not = !flag     // logical NOT
```

### Dot Expressions (Member Access / Method Call)

```auto
obj.field           // field access
obj.method()        // method call
list.push(1)        // method with argument
```

### Ownership Expressions (Plan 122)

```auto
let ref_val = x.view     // immutable borrow (&T)
let mut_ref = x.mut      // mutable borrow (&mut T)
let owned = x.move       // ownership transfer
```

### Type Conversion Expressions

```auto
let n = x.as(int)        // zero-cost reinterpret (.as)
let s = 42.to(str)       // explicit conversion (.to)
```

### Option/Result Expressions (Plan 120)

```auto
// Constructors
let val = Some(42)       // wrap in Option
let empty = None         // empty Option
let ok = Ok(42)          // success Result
let err = Err("failed")  // error Result

// Null coalescing
let x = maybe_val ?? 0   // default if None

// Error propagation
let result = expr.?      // propagate Err as return

// Safe navigation
let name = obj?.name     // access if not None
```

### Closure Expressions

```auto
// Simple closure
let add = (a, b) => a + b

// Typed closure
let multiply = (a int, b int) => int { a * b }

// Single-param closure (no parens needed)
let double = x => x * 2
```

### If Expressions

```auto
let result = if x > 0 { 1 } else { 0 }

// Multi-branch
let label = if x > 0 {
    "positive"
} else if x < 0 {
    "negative"
} else {
    "zero"
}
```

### Block Expressions

```auto
let result = {
    let x = 10
    let y = 20
    x + y       // last expression is the value
}
```

### Null Coalescing Expressions (Plan 120)

```auto
let value = maybe ?? default    // use default if None
```

### Error Propagation Expressions (Plan 120)

```auto
let result = expr.?    // if Err, return early; if Ok, unwrap
```

### Smart Pointer Expressions

```auto
let boxed = Box(value)    // heap allocation
let shared = Arc(value)   // reference-counted
```

### Async Expressions (Plan 124)

```auto
// Async block
let future = ~{
    let data = fetch().await
    process(data)
}

// Await a future
let result = future.await

// Spawn to background (Plan 126)
let handle = task.go
```

### Compile-Time Expressions (Plan 095)

```auto
let value = #{ 1 + 2 }    // evaluated at compile time
```

### Node Expressions

```auto
// Node construction
widget(attr: "value") {
    child()
}
```

### Grid Expressions

```auto
let grid = grid(a: "first", b: "second") {
    [1, 2, 3]
    [4, 5, 6]
}
```

### Hold Expressions

```auto
let val = expr.hold    // extend lifetime binding
```

### Precedence Table

| Precedence | Operators | Associativity |
|------------|-----------|---------------|
| 1 (highest) | `.view`, `.mut`, `.move`, `.await`, `.go`, `.hold` | Left |
| 2 | `!` (NOT), `-` (negate) | Right |
| 3 | `*`, `/`, `%` | Left |
| 4 | `+`, `-` | Left |
| 5 | `<`, `>`, `<=`, `>=` | Left |
| 6 | `==`, `!=` | Left |
| 7 | `&&`, `and` | Left |
| 8 | `||`, `or` | Left |
| 9 | `??` | Left |
| 10 | `..`, `..=` | Left |
| 11 (lowest) | `=`, `+=`, `-=`, `*=`, `/=`, `%=` | Right |

---

## Statements

Auto supports 34 statement types (from `ast.rs` Stmt enum).

### Variable Declarations

Auto provides six storage modifiers with different semantics:

#### `let` - Immutable Binding

```auto
let x = 42
let name str = "Alice"
// Error: x = 10  // cannot reassign
```

#### `var` - Mutable Binding

```auto
var x = 42
x = 10    // OK
```

#### `const` - Compile-Time Constant

```auto
const MAX_SIZE = 100
const GREETING str = "Hello"
// Error: MAX_SIZE = 200  // cannot modify
```

#### `mut` - Mutable Reference Keyword

```auto
mut x = 42
x = 10    // OK
```

Note: `var` and `mut` are both mutable. `var` is preferred for script mode; `mut` for typed mode.

#### `shared` - Static/Shared Storage (Plan 168)

```auto
shared counter int = 0
shared cache Map<str, str> = Map.new()
```

`shared` creates process-lifetime static storage. Transpiles to Rust `static` with `Lazy<Mutex<T>>`.

#### `static` - Static Member (in type context)

Used inside type definitions for static methods:

```auto
type Point {
    x int
    y int

    static fn new(x int, y int) Point {
        Point(x, y)
    }
}
```

### Expression Statements

Any expression can be a statement:

```auto
x + 1
func_call()
obj.method()
```

### Block Statements

```auto
{
    let x = 10
    let y = 20
    x + y
}
```

### Return Statements

```auto
fn greet() str {
    return "Hello"     // explicit return
}

fn add(a int, b int) int {
    a + b              // implicit return (last expression)
}
```

### Reply Statements (Plan 124)

Used in task message handlers for ask/reply RPC:

```auto
task CounterTask {
    on {
        GetCount() -> {
            reply self.count    // send reply to caller
        }
    }
}
```

### Break Statements

```auto
for i in 0..100 {
    if i == 42 {
        break
    }
}
```

### Import Statements

```auto
use math::add           // import from module
use pac.db              // import from package root
use super.utils         // import from parent directory
use db: load, save      // import specific symbols
dep database(path: "../database")  // declare dependency
```

### Type Declaration Statements

```auto
type Point { x int, y int }
enum Color { Red, Green, Blue }
tag Shape { Circle float, Rect int, int }
spec Printable { fn print() }
alias UserID = int
```

### Extension Statements

```auto
ext str {
    fn is_empty() bool { self.len() == 0 }
}
```

### Comment Statements

```auto
// Single-line comment
/// Doc comment
/* Block comment */
```

### Empty Lines

Empty lines are preserved as statement separators for code formatting.

---

## Control Flow

### If Statements

```auto
// Basic if
if x > 0 {
    print("positive")
}

// If-else
if x > 0 {
    print("positive")
} else {
    print("non-positive")
}

// If-else if-else
if x > 0 {
    print("positive")
} else if x < 0 {
    print("negative")
} else {
    print("zero")
}

// If expression (value-producing)
let result = if x > 0 { 1 } else { 0 }
```

### For Loops

```auto
// Range iteration (exclusive)
for i in 0..5 {
    print(i)    // 0, 1, 2, 3, 4
}

// Range iteration (inclusive)
for i in 0..=5 {
    print(i)    // 0, 1, 2, 3, 4, 5
}

// Array iteration
for item in [1, 2, 3] {
    print(item)
}

// With index
for i, item in [1, 2, 3] {
    print(f"${i}: ${item}")
}

// Condition loop (replaces while)
for i < 10 {
    print(i)
    i += 1
}
```

Note: Auto does not have a `while` keyword. Use `for condition { }` instead.

### Loop Control

```auto
loop {
    if condition {
        break
    }
}
```

### Pattern Matching (`is` expression)

```auto
is value {
    42 -> print("exact match"),
    as str -> print("string type"),
    in 0..9 -> print("single digit"),
    if value > 10 -> print("big number"),
    else -> print("other")
}
```

Note: Pattern branches use `->` (arrow). Closures use `=>` (double arrow).

#### Struct Destructuring (Plan 165)

```auto
let point = Point(10, 20)

is point {
    Point(x, y) -> print(f"x=${x}, y=${y}"),
    else -> print("not a point")
}
```

#### Option Pattern Matching (Plan 120)

```auto
is maybe_value {
    Some(x) -> print(f"got: ${x}"),
    None -> print("nothing")
}
```

#### Result Pattern Matching (Plan 120)

```auto
is result {
    Ok(value) -> print(f"success: ${value}"),
    Err(msg) -> print(f"error: ${msg}")
}
```

### When Blocks

```auto
when event {
    Click(x, y) -> handleClick(x, y),
    KeyPress(key) -> handleKey(key),
    else -> handleOther()
}
```

---

## Functions

### Function Definition

```auto
// Basic function (no return type annotation needed for void)
fn greet(name str) {
    print(f"Hello, $name!")
}

// Function with return type (space-separated, no ->)
fn add(a int, b int) int {
    a + b    // Implicit return (last expression)
}

// Function with explicit return
fn multiply(a int, b int) int {
    return a * b
}

// Function with no return value
fn print_message(msg str) void {
    print(msg)
}
```

### Function Calls

```auto
greet("World")

let result = add(1, 2)
```

### Generic Functions (Plan 048)

```auto
fn identity<T>(x T) T {
    x
}

fn first<T>(arr []T) ?T {
    if arr.len() == 0 {
        return None
    }
    Some(arr[0])
}

// With type constraints
fn compare<T has Comparable>(a T, b T) int {
    a.compare(b)
}
```

### Parameter Modes (Plan 088)

Auto uses three parameter passing modes. The default is `view` (immutable reference).

```auto
// view - immutable reference (DEFAULT, O(1))
fn process(x int) {
    // x is an immutable reference
    // cannot modify x
}

// mut - mutable reference (O(1))
fn increment(x mut int) {
    x += 1    // modifies caller's value
}

// move - ownership transfer (O(1))
fn consume(data String) {
    // data is moved from caller
    // caller can no longer use data
}
```

The `ParamMode` enum supports: `View` (default), `Mut`, `Move`, `Copy` (deprecated), `Take` (deprecated, use Move).

### Static Methods (Plan 035)

```auto
type Point {
    x int
    y int

    // Static method (called on type)
    static fn new(x int, y int) Point {
        Point(x, y)
    }

    // Instance method (called on instance)
    fn distance_to(other Point) float {
        let dx = self.x - other.x
        let dy = self.y - other.y
        (dx * dx + dy * dy).to(float).sqrt()
    }
}

let p = Point.new(3, 4)       // static call
let d = p.distance_to(p)      // instance call
```

### Public Visibility (Plan 163)

```auto
pub fn public_function() int { 42 }

pub type Point { x int, y int }

pub enum Color { Red, Green, Blue }

pub spec Printable { fn print() }
```

### Closures (Plan 060)

Closures use `=>` syntax:

```auto
// Simple closure
let add = (a, b) => a + b
add(3, 4)    // 7

// Single-parameter closure (no parens needed)
let double = x => x * 2

// Typed closure with block body
let transform = (x int, y int) => int {
    let sum = x + y
    sum * 2
}
```

Capture semantics:
- By default, closures capture by reference (view)
- Use `.move` on the closure to capture by value

### Function Annotations

```auto
#[vm]
fn vm_function(x int) void           // VM-implemented

#[c]
fn c_function(s str) int             // C-transpiled

#[c, vm]
fn hybrid_function(data []byte) void // both backends
```

### Function Kinds

The `FnKind` enum supports 5 function types:

| Kind | Description |
|------|-------------|
| `Function` | Regular function |
| `Lambda` | Lambda/anonymous function |
| `Method` | Instance method (associated with a type) |
| `CFunction` | C function declaration |
| `VmFunction` | VM-implemented function |

### Default Parameters

```auto
fn greet(name str, greeting str = "Hello") str {
    f"$greeting, $name!"
}

greet("World")               // "Hello, World!"
greet("World", "Hi")         // "Hi, World!"
```

### Mutable Methods (Plan 163)

```auto
type Counter {
    count int

    mut fn increment() {
        self.count += 1
    }
}

var c = Counter(0)
c.increment()    // c.count is now 1
```

---

## Data Structures

### Arrays

```auto
// Array literal
let arr = [1, 2, 3, 4, 5]

// Index access
let first = arr[0]       // 1
let last = arr[-1]       // 5

// Slicing
let slice = arr[1..3]    // [2, 3]
let slice2 = arr[..4]    // [1, 2, 3, 4]
let slice3 = arr[3..]    // [4, 5]
let slice4 = arr[..]     // [1, 2, 3, 4, 5]

// Modification
var arr = [1, 2, 3]
arr[0] = 10    // [10, 2, 3]
```

### Objects/Maps

```auto
// Object literal
let obj = {
    name: "John",
    age: 30,
    active: true
}

// Access members
print(obj.name)

// Modify members
obj.name = "Tom"

// Methods
obj.keys()           // ["name", "age", "active"]
obj.values()         // ["John", 30, true]
obj.items()          // [("name", "John"), ("age", 30), ("active", true)]

// Safe access
obj.get("name", "Unknown")
obj.get_or_insert("name", 10)

// Iteration
for k, v in obj {
    print(f"${k}: ${v}")
}

// Remove key
obj.remove("name")
```

### Grid (2D Arrays)

```auto
// Grid definition
let grid = grid(a:"first", b:"second", c:"third") {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// Convert to JSON
let json = grid.to_json()
```

---

## Type Definitions

### Type Definitions

Use the `type` keyword to define custom types:

```auto
type Point {
    x int
    y int
}

type Rectangle {
    top_left Point
    bottom_right Point
}

// Using custom types
let p Point = Point(10, 20)
let rect Rectangle = Rectangle(
    top_left: Point(0, 0),
    bottom_right: Point(100, 100)
)
```

### Single Inheritance

```auto
type Animal {
    name str
    fn speak() str
}

type Dog is Animal {
    breed str
    fn speak() str { "Woof!" }
}
```

### Generic Types (Plan 048)

```auto
type Container<T> {
    value T

    static fn new(value T) Container<T> {
        Container(value)
    }

    fn get() T {
        self.value
    }
}

// Const generics
type Inline<T, N u32> {
    data [N]T
}
```

### Spec Implementation

```auto
type Point has Printable {
    x int
    y int

    fn print() {
        print(f"(${self.x}, ${self.y})")
    }
}

// Multiple spec implementations
type File has Readable, Writable {
    path str
}
```

### Extension Blocks (`ext`)

```auto
// Add methods to existing types
ext str {
    fn is_empty() bool { self.len() == 0 }
    fn reversed() str { /* ... */ }
}

// With generic params (Plan 059)
ext<T> Vec<T> {
    fn first() ?T { /* ... */ }
}
```

### Type Aliases

Create type aliases with `alias`:

```auto
alias UserID = int
alias Name = str
alias Coordinate = float

let uid UserID = 12345
let name Name = "Alice"
let coord Coordinate = 45.5
```

### TypeDecl Fields

The `TypeDecl` struct in `ast/types.rs` includes:

| Field | Type | Description |
|-------|------|-------------|
| `name` | `Name` | Type name |
| `parent` | `Option<Box<Type>>` | Parent type (inheritance) |
| `has` | `Vec<Type>` | Spec implementations (`has`) |
| `spec_impls` | `Vec<SpecImpl>` | Spec implementations with type args |
| `generic_params` | `Vec<GenericParam>` | Generic type/const parameters |
| `members` | `Vec<Member>` | Data fields |
| `methods` | `Vec<Fn>` | Methods |
| `attrs` | `Vec<AutoStr>` | Annotations |
| `is_pub` | `bool` | Public visibility |

---

## Enums

Auto supports three enum kinds (from `ast/enums.rs` EnumKind).

### Scalar Enums (C-style)

Simple enumeration with optional explicit values and repr type:

```auto
enum Color { Red, Green, Blue }

// With explicit values
enum HttpStatus u16 {
    OK = 200,
    NotFound = 404,
    ServerError = 500
}
```

### Homogeneous Enums

All variants share the same payload type:

```auto
type Point { x int, y int }

enum Vertex Point {
    LeftTop
    RightTop
    LeftBottom
    RightBottom
}
```

### Heterogeneous Enums (ADT / Sum Types)

Each variant can have a different payload type:

```auto
enum Msg {
    Quit,
    Move Point,
    Write str,
    Color(int, int, int)
}
```

### Generic Enums

```auto
enum Option<T> {
    Some(T),
    None
}

enum Result<T, E> {
    Ok(T),
    Err(E)
}
```

### EnumDecl Fields

The `EnumDecl` struct includes:

| Field | Type | Description |
|-------|------|-------------|
| `name` | `AutoStr` | Enum name |
| `items` | `Vec<EnumItem>` | Variants |
| `kind` | `EnumKind` | Scalar, Homogeneous, or Heterogeneous |
| `is_pub` | `bool` | Public visibility |

### Tags (Tagged Unions)

Tags provide type-safe discriminated unions (similar to Rust enums):

```auto
tag MyTag {
    i int
    f float
    c char
}

let value MyTag = MyTag.i(42)

// Pattern matching with `is`
is value {
    i -> print("int"),
    f -> print("float"),
    c -> print("char")
}
```

---

## Specs (Traits)

Specs define interface-like contracts that types can implement (Plans 019, 057, 059).

### Spec Declaration

```auto
spec Printable {
    fn print() void
}

spec Readable {
    fn read(buf []byte) int
}
```

### Generic Specs

```auto
spec Iterable<T> {
    fn iterator() Iterator<T>
}

spec Comparable<T> {
    fn compare(other T) int
}
```

### Default Methods

```auto
spec Loggable {
    fn log_level() str { "info" }
    fn log(msg str) void {
        print(f"[${self.log_level()}] $msg")
    }
}
```

### Spec Implementation

```auto
type User has Printable {
    name str

    fn print() void {
        print(self.name)
    }
}
```

### SpecDecl Fields

The `SpecDecl` struct includes:

| Field | Type | Description |
|-------|------|-------------|
| `name` | `Name` | Spec name |
| `generic_params` | `Vec<GenericParam>` | Generic parameters |
| `methods` | `Vec<SpecMethod>` | Required/default methods |
| `is_pub` | `bool` | Public visibility |

### Transpilation Behavior

- **C backend**: Specs transpile to vtables (struct of function pointers)
- **Rust backend**: Specs transpile to native Rust traits
- **VM mode**: Specs use dynamic dispatch via method registry

---

## Generics

Auto supports generics with type parameters and const parameters (Plan 048).

### Type Parameters

```auto
type Container<T> {
    value T
}

fn identity<T>(x T) T { x }
```

### Const Parameters

```auto
type Inline<T, N u32> {
    data [N]T
}

let buffer Inline<int, 64> = Inline.new()
```

### Generic Constraints

```auto
fn sort<T has Comparable>(items []T) []T {
    // T must implement Comparable spec
}
```

### Monomorphization

Generic types and functions are monomorphized at compile time for C/Rust backends, generating specialized code for each concrete type instantiation.

---

## Closures

Auto closures use `=>` syntax (Plan 060).

### Syntax Forms

```auto
// No params
let thunk = => doSomething()

// Single param (no parens)
let double = x => x * 2

// Multiple params
let add = (a, b) => a + b

// Typed params
let multiply = (a int, b int) => int { a * b }
```

### Capture Semantics

```auto
let x = 10

// Captures x by reference (view)
let closure = () => x + 1

// Captures x by value (move)
let moved = () => x + 1 .move
```

### Iterator Usage

```auto
let nums = [1, 2, 3, 4, 5]
let doubled = nums.map(x => x * 2)
let evens = nums.filter(x => x % 2 == 0)
```

---

## Option and Result

Auto provides `?T` (Option) and `!T` (Result) for error handling (Plan 120).

### Option (`?T`)

```auto
let name ?str = Some("Alice")
let empty ?str = None

// Pattern matching
is name {
    Some(n) -> print(f"Hello, $n"),
    None -> print("no name")
}

// Null coalescing
let display = name ?? "unknown"
```

### Result (`!T`)

```auto
fn divide(a int, b int) !int {
    if b == 0 {
        return Err("division by zero")
    }
    Ok(a / b)
}

// Error propagation with .?
let result = divide(10, 0).?    // propagates Err

// Pattern matching
is result {
    Ok(value) -> print(value),
    Err(msg) -> print(f"Error: $msg")
}
```

### Constructors

| Constructor | Type | Description |
|-------------|------|-------------|
| `Some(value)` | Option | Wrap a value |
| `None` | Option | No value |
| `Ok(value)` | Result | Success |
| `Err(message)` | Result | Failure |

---

## Concurrency: Tasks and Async

Auto provides an Actor-based concurrency model (Plans 121, 124, 126).

### Task Definition

```auto
task CounterTask {
    count int = 0

    start() {
        print("Counter started")
    }

    on {
        Increment(n int) -> {
            self.count += n
        }
        GetCount() -> {
            reply self.count
        }
        Reset -> {
            self.count = 0
        }
        _ -> {
            print("unknown message")
        }
    }

    stop() {
        print(f"Counter stopped at ${self.count}")
    }
}
```

### Task Attributes

```auto
#[single]
task UniqueTask {
    on { /* ... */ }
}
```

### Spawning and Communication

```auto
let handle Handle<CounterTask> = spawn CounterTask()
handle.send(Increment(5))
handle.send(GetCount)    // for ask/reply
```

### Message Patterns

| Pattern | Syntax | Description |
|---------|--------|-------------|
| Simple | `Reset` | No data |
| With bindings | `Add(val)` | With named bindings |
| Literal match | `"start"` | Exact literal match |
| Type binding | `msg str` | Capture by type |

### Async / Await (Plan 124)

```auto
// Async function (~T return type)
fn fetch_data(url str) ~str {
    // ~T indicates async return
    let response = http_get(url).await
    response.body
}

// Async block
let result = ~{
    let a = fetch_data("/api/a").await
    let b = fetch_data("/api/b").await
    a + b
}
```

### Background Execution (Plan 126)

```auto
// .go spawns to background worker pool
let handle = compute_heavy(data).go
// ... do other work ...
let result = handle.await
```

---

## Compile-Time Metaprogramming

Auto supports compile-time code execution (Plan 095).

### `#if` — Compile-Time Conditional

```auto
#if DEBUG {
    print("debug mode")
} else {
    print("release mode")
}
```

### `#for` — Compile-Time Loop Unrolling

```auto
#for i in 0..4 {
    let bit_${i} = 1 << i
}
// Generates: let bit_0 = 1, bit_1 = 2, bit_2 = 4, bit_3 = 8
```

### `#is` — Compile-Time Pattern Match

```auto
#is target_os {
    "linux" -> { const PLATFORM = "linux" },
    "windows" -> { const PLATFORM = "windows" },
    else -> { const PLATFORM = "unknown" }
}
```

### `#{}` — Compile-Time Expression

```auto
const SIZE = #{ 1024 * 768 }
```

---

## Ownership and Borrowing

Auto implements the "Trinity of Resources": `view`, `mut`, `move` (Plan 122).

### Access Modes

| Mode | Syntax | Cost | Description |
|------|--------|------|-------------|
| `view` | `x.view` | O(1) | Immutable borrow (`&T`) |
| `mut` | `x.mut` | O(1) | Mutable borrow (`&mut T`) |
| `move` | `x.move` | O(1) | Ownership transfer |
| `clone` | `x.clone()` | O(N) | Deep copy |

### Default Parameter Mode

Function parameters default to `view` (immutable reference):

```auto
fn process(data str) {
    // data is borrowed immutably (view mode)
    print(data)
}
```

### Ownership Transfer

```auto
fn consume(s String) {
    // s is moved; caller no longer has access
    print(s)
}

let text = String.new("hello")
consume(text.move)
// text is no longer accessible here
```

### Hold

```auto
let val = expr.hold    // extend lifetime binding
```

---

## Modules and Imports

Auto supports a hierarchical module system (Plan 131).

### Import Syntax

```auto
use db              // Same directory: ./db.at or ./db/mod.at
use super.db         // Parent directory: ../db.at
use pac.db           // Package root: search source dirs
use pac.api.handlers // Deep path from root
use db: load, save   // Import specific symbols
```

### Dependency Declaration

```auto
// In pac.at
name: "myapp"
src: ["src"]
dep database(path: "../database")
```

### Public Exports

```auto
pub fn public_function() int { 42 }
pub type Point { x int, y int }
pub use math::add
```

### Resolution Rules

1. Try `name.at` (file module)
2. Try `name/mod.at` (directory module)
3. Error if both exist (ambiguous)
4. Error if neither exists

---

## UI Widgets and Routing

### Widget Declaration (Plan 096)

```auto
widget CounterView {
    model {
        count int = 0
    }

    msg {
        Increment
        Decrement
    }

    view {
        col {
            text(text: f"Count: ${model.count}")
            button(text: "+") {
                onclick: emit Increment
            }
            button(text: "-") {
                onclick: emit Decrement
            }
        }
    }
}
```

### Routing (Plan 105)

```auto
routes {
    route("/", HomeView)
    route("/about", AboutView)
    route("/users", UserListView)
}

// Navigation
nav("/about")
link(text: "Go to About", to: "/about")

// Outlet for nested routes
outlet()
```

---

## Nodes

Nodes are Auto's XML-like tree structure for data representation, combining JSON's simplicity with XML's hierarchical nature. Auto can be compiled down to Atom format.

### Basic Node Syntax

```auto
node_name(a: 1, b: "hello") {
    // children
    sub_node(c: 2) {
        // ...
    }

    // children
    sub_node2() {
        // ...
    }
}
```

这个结构和XML的树状结构基本一致，例如，上面的代码用XML可以表示为：

```xml
<node_name a="1" b="hello">
    <sub_node c="2">
        <!-- more content -->
    </sub_node>
    <sub_node2>
        <!-- more content -->
    </sub_node2>
</node_name>
```

可以看到，这里节点名称，属性参数（在XML中叫`attribute`）和子节点的定义，
信息量上是完全对等的。

相比于XML，Auto的节点定义格式有如下优点：

1. 更加紧凑，没有冗余的尖括号和结束标签。
2. 形式上更接近于C系列语言的风格，可以和其他Auto代码良好地融合在一起
3. 还有简化空间。

### Simplifications to node syntax

当某个节点没有属性时，可以省略括号：

```auto
root {
    // subnodes
}
```

当节点没有子节点时，可以省略掉`{..}`：

```auto
leaf(id: "my_leaf")
```

此时，节点的定义从语法上就和函数调用基本一致了，从语法上来看，产生了歧义。

Auto语言从语义上来解决这个歧义：

1. 节点的定义其实也是一种函数调用，相当于一个构造函数
2. 节点的名称如果定义为`fn`，则这个表达式是函数调用；如果是一个类型`type`，则为该类型的构造函数。

这样设计的话，节点表达式和类型的实例化就统一起来了。

当节点的定义很明确时，我们可以忽略掉属性的名字，直接调用参数值：

```auto
type Point {
    x int
    y int
}

let p = Point(10, 20)
```

如果节点的类型本身有一个主属性（一般是`id`或`name`），
那么可以用节点声明表达式来定义一个实体：

```auto
type User {
    @primary
    name str

    age int
}

User XiaoMing {
    age: 18
}

// 相当于：
let XiaoMing = User(name: "Xiaoming", age: 18)
```

这种方式可以直接定义一个变量，在结构化的配置文件中很好用。

### Examples

```auto
// Simple node
root(id: "123") {
    name("Puming")
    age(41)
}

// Nested nodes
root(id: "123") {
    name("Puming") {
        surname("Zhao") {
            // More nested content
        }
    }
    age(41)
}
```

### Node vs Object

In Auto, object is actually a subset of node.

You can view object as a special type of node that is:

1. Anonymous: the node type is not specified, or actally put as an property inside the `{}`
2. Single: no directy sub-nodes are specified, i.e. subnodes are specifed as properity values.

**Object** (data):
```auto
let obj = {
    name: "value",
    count: 42
}
```

**Node** (structure):
```auto
node(attr: "value") {
    child()
}
```

Key differences:
- Nodes use `()` for attributes, `{}` for children
- Nodes represent tree structures (like XML)
- Objects represent key-value mappings (like JSON)

### Use Cases

Nodes are commonly used in:

1. **UI Description** (AutoUI):
```auto
window(title: "My App") {
    button(label: "Click me") {
        onclick: => print("clicked")
    }
    textbox(placeholder: "Enter text")
}
```

2. **Configuration** (AutoConfig):
```auto
config(version: "1.0") {
    database(host: "localhost", port: 5432) {
        credentials(user: "admin", pass: "secret")
    }
}
```

3. **Code Generation** (AutoGen):
```auto
for student in students {
    student(name: student.name, age: student.age) {
        for course in student.courses {
            course(name: course.name, score: course.score)
        }
    }
}
```

### Node Compilation

Dynamic Auto code compiles to static Atom:

**Input** (Auto):
```auto
var name = "Puming"
root(id: "123") {
    name(name)
    age(41)
}
```

**Output** (Atom):
```auto
root(id: "123") {
    name("Puming")
    age(41)
}
```

---

## Memory Management

### Storage Lifetimes

Auto supports multiple memory lifetimes:

| Lifetime | Description |
|----------|-------------|
| **Immortal** | Persists beyond program end |
| **Process** | Program lifetime (globals) |
| **Auto** | GC/RC managed |
| **Task** | Task completion |
| **Scope** | Function/block scope |
| **Instant** | Statement-level |

### Memory Management Strategies

Auto provides three memory management strategies:

1. **Manual (C-like)**: Explicit allocation/deallocation
2. **Automatic (GC)**: Garbage collected
3. **Automatic (RC)**: Reference counted

---

## Implementation Comparison

### C Implementation vs Rust Implementation

#### Feature Completeness

| Feature | Rust | C |
|---------|------|---|
| Lexer | Complete | Complete |
| Parser | Complete | Partial |
| Evaluator | Complete | Partial |
| Transpilation | Complete | Not implemented |
| Type System | Complete | Partial |
| Pattern Matching | Complete | Not implemented |
| F-Strings | Complete | Complete |

---

## Appendices

### Appendix A: Operator Precedence (Highest to Lowest)

| Level | Operators | Associativity |
|-------|-----------|---------------|
| 1 (highest) | `.view`, `.mut`, `.move`, `.await`, `.go`, `.hold` | Left |
| 2 | `!` (NOT), `-` (negate) | Right |
| 3 | `*`, `/`, `%` | Left |
| 4 | `+`, `-` | Left |
| 5 | `<`, `>`, `<=`, `>=` | Left |
| 6 | `==`, `!=` | Left |
| 7 | `&&`, `and` | Left |
| 8 | `||`, `or` | Left |
| 9 | `??` | Left |
| 10 | `..`, `..=` | Left |
| 11 (lowest) | `=`, `+=`, `-=`, `*=`, `/=`, `%=` | Right |

### Appendix B: Reserved Keywords (56 keywords)

```
alias, and, as, await, break, const, copy, dep, else, enum, Err, ext,
fn, for, go, grid, has, hold, if, impl, in, is, let, link, route,
nav, move, mut, nil, node, None, null, Ok, on, or, outlet, pac,
pub, reply, routes, shared, Some, spec, spawn, static, super, tag,
task, to, true, false, type, union, use, var, view, when
```

### Appendix C: Expression Types (55 variants from ast.rs)

```
Int, Uint, I8, U8, I64, U64, Byte, Float, Double, Bool, Char, Str,
CStr, Ident, GenName, Ref, View, Mut, Move, Take, Hold, Unary, Bina,
Dot, Range, Array, Pair, Block, Object, Call, Node, Index, Lambda,
Closure, FStr, Grid, Cover, Uncover, OptionPattern, ResultPattern,
OptionUncover, ResultUncover, StructPattern, If, Nil, Null,
NullCoalesce, ErrorPropagate, Cast, To, Some, None, Ok, Err,
BoxExpr, ArcExpr, NavCall, AsyncBlock, Await, Go, Comptime
```

---

**Document Version**: 0.2-draft
**Status**: Work in Progress
**Feedback**: Please report issues and suggest improvements via GitHub issues or pull requests.
