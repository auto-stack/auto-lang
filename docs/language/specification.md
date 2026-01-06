# Auto Language Specification

**Version**: 0.1
**Status**: Draft
**Last Updated**: 2025

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
11. [Memory Management](#memory-management)
12. [Implementation Comparison](#implementation-comparison)

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
    println("Hello, World!")
}

// Expression-based (script mode)
println("Hello, World!")
```

### Key Features

- **Four Storage Modifiers**: `let`, `mut`, `const`, `var` for different mutability and lifetime semantics
- **Type Inference**: Automatic type deduction with optional explicit annotations
- **Pattern Matching**: Powerful `is` expression for pattern matching
- **F-Strings**: First-class string interpolation with embedded expressions
- **Ranges**: First-class range expressions `0..10` and `0..=10`
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

/*
   Multi-line comment
   Spans multiple lines
*/
```

### Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores.

```
identifier → (letter | "_") (letter | digit | "_")*
```

**Examples**: `foo`, `_bar`, `data123`, `my_variable`

### Keywords

The following keywords are reserved:

**Declarations**: `fn`, `let`, `mut`, `const`, `var`, `type`, `union`, `enum`, `tag`, `alias`
**Control Flow**: `if`, `else`, `for`, `while`, `when`, `break`, `is`, `in`, `on`, `as`
**Literals**: `true`, `false`, `nil`, `null`
**Other**: `use`, `has`, `fn`

### Operators and Punctuation

| Operator | Description |
|----------|-------------|
| `+` | Addition |
| `-` | Subtraction/negation |
| `*` | Multiplication |
| `/` | Division |
| `=` | Assignment |
| `==` | Equal comparison |
| `!=` | Not equal |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |
| `+=` | Addition assignment |
| `-=` | Subtraction assignment |
| `*=` | Multiplication assignment |
| `/=` | Division assignment |
| `..` | Range (exclusive) |
| `..=` | Range (inclusive) |
| `.` | Member access |
| `->` | Arrow (patterns/events) |
| `=>` | Double arrow (patterns) |
| `:` | Colon (type annotation, pairs) |
| `|` | Vertical bar |
| `?` | Question mark |
| `@` | At sign |
| `#` | Hash sign |

### Literals

#### Integer Literals

```
integer → digit+ ("u" | "u8" | "i8")?
```

```auto
42          // i32 (default)
42u         // u32
42u8        // u8
42i8        // i8
```

#### Floating-Point Literals

```
float → digit+ "." digit* (exponent)?
exponent → ("e" | "E") ("+" | "-")? digit+
```

```auto
3.14
3.14e-10
1.0e5
```

#### String Literals

```
string → '"' .* '"'
cstr → 'c' '"' '"'* '"'
```

```auto
"Hello, World!"           // Auto string (with length)
c"Hello, World!"          // C string (null-terminated)
```

#### Character Literals

```
char → "'" (letter | digit | symbol) "'"
```

```auto
'a'
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

### Basic Types

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
| `float` | `double` | `f64` | 64-bit floating-point |
| `double` | `double` | `f64` | Alias for float |
| `bool` | `bool` | `bool` | Boolean (true/false) |
| `str` | `struct { len; data; }` | `&str` | String slice with length |
| `char` | `char` | `char` | Single character |
| `void` | `void` | `()` | Unit/void type |
| `nil` | (no equivalent) | `!` | Zero-size type |

### Type Annotations

```auto
// Type inference
let x = 42

// Explicit type annotation
let y: int = 42

// Function with type annotations
fn add(a: int, b: int) int {
    a + b
}
```

### Type Coercion

Auto performs automatic type coercion for assignments:

```auto
let b: byte = 42    // OK: int coerced to byte
let i: int = b      // OK: byte promoted to int
```

---

## Expressions

### Literal Expressions

```auto
42          // integer
3.14        // float
"hello"     // string
'x'         // character
true        // boolean
false       // boolean
nil         // nil value
```

### Identifier Expressions

```auto
x
my_variable
_underscore
```

### Binary Operators

#### Arithmetic Operators

```auto
let sum = 10 + 5
let diff = 10 - 5
let product = 10 * 5
let quotient = 10 / 5
```

#### Comparison Operators

```auto
10 == 5    // false
10 != 5    // true
10 < 5     // false
10 > 5     // true
10 <= 10   // true
10 >= 10   // true
```

#### Logical Operators

```auto
true && false    // false
true || false    // true
!true            // false
```

### Assignment Operators

```auto
x = 10
x += 5    // x = x + 5
x -= 5    // x = x - 5
x *= 5    // x = x * 5
x /= 5    // x = x / 5
```

### Range Expressions

```auto
// Exclusive range: 0, 1, 2, 3, 4
let r1 = 0..5

// Inclusive range: 0, 1, 2, 3, 4, 5
let r2 = 0..=5

// Using ranges in for loops
for i in 0..10 {
    println(i)
}
```

### F-String Expressions

```auto
let name = "World"
let msg = f"Hello, {name}!"    // "Hello, World!"

let a = 5
let b = 10
let result = f"{a} + {b} = {a + b}"    // "5 + 10 = 15"

// Alternative tick string syntax
let msg = `Hello, ${name}!`
```

### Array Expressions

```auto
let arr = [1, 2, 3, 4, 5]
let empty = []
let strings = ["hello", "world"]
```

### Object Expressions

```auto
let obj = {
    name: "John",
    age: 30,
    active: true
}
```

### Grouping Expressions

```auto
let result = (2 + 3) * 5    // 25
```

---

## Statements

### Variable Declarations

Auto provides four storage modifiers with different semantics:

#### `let` - Immutable Binding

```auto
let x = 42
// Error: x = 10  // cannot reassign
```

#### `mut` - Mutable Binding

```auto
mut x = 42
x = 10    // OK
```

#### `const` - Compile-Time Constant

```auto
const MAX_SIZE = 100
// Error: MAX_SIZE = 200  // cannot modify
```

#### `var` - Dynamic Variable (Script Mode)

```auto
var x = 42
x = "hello"    // OK - type can change
x = nil        // OK - can be nil
```

### Expression Statements

Any expression can be a statement:

```auto
x + 1
func_call()
```

### Block Statements

```auto
{
    let x = 10
    let y = 20
    x + y
}
```

---

## Control Flow

### If Statements

```auto
// Basic if
if x > 0 {
    println("positive")
}

// If-else
if x > 0 {
    println("positive")
} else {
    println("non-positive")
}

// If-else if-else
if x > 0 {
    println("positive")
} else if x < 0 {
    println("negative")
} else {
    println("zero")
}

// If expression
let result = if x > 0 { 1 } else { 0 }
```

### For Loops

```auto
// Range iteration (exclusive)
for i in 0..5 {
    println(i)    // 0, 1, 2, 3, 4
}

// Range iteration (inclusive)
for i in 0..=5 {
    println(i)    // 0, 1, 2, 3, 4, 5
}

// Array iteration
for item in [1, 2, 3] {
    println(item)
}

// With index
for i, item in [1, 2, 3] {
    println(f"{i}: {item}")
}

// Mutable reference
mut arr = [1, 2, 3]
for ref item in arr {
    item = item * 2
}
// arr = [2, 4, 6]
```

### While Loops

```auto
mut i = 0
while i < 10 {
    println(i)
    i += 1
}
```

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
    42 -> println("exact match"),
    as str -> println("string type"),
    in 0..9 -> println("single digit"),
    if value > 10 -> println("big number"),
    else x -> println(f"other: {x}")
}
```

---

## Functions

### Function Definition

```auto
// Basic function
fn greet(name str) {
    println(f"Hello, {name}!")
}

// Function with return type
fn add(a int, b int) int {
    a + b    // Implicit return
}

// Function with explicit return
fn multiply(a int, b int) int {
    return a * b
}

// Function with no return value
fn print_message(msg str) void {
    println(msg)
}
```

### Function Calls

```auto
greet("World")

let result = add(1, 2)
```

### Lambda Functions

```auto
let multiply = |a int, b int| a * b
multiply(3, 4)    // 12
```

### Parameter Passing Modes

```auto
// copy - default for small types
fn process_copy(x int) {
    // x is a copy
}

// ref - immutable reference
fn process_ref(x ref int) {
    // can read but not modify
}

// mut - mutable reference
fn increment(x mut ref int) {
    x += 1    // modifies caller's value
}

// move - transfer ownership
fn consume(x move String) {
    // x is moved from caller
}
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
mut arr = [1, 2, 3]
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
println(obj.name)

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
    println(f"{k}: {v}")
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
| Lexer | ✅ Complete | ✅ Complete |
| Parser | ✅ Complete | ⚠️ Partial |
| Evaluator | ✅ Complete | ⚠️ Partial |
| Transpilation | ✅ Complete | ❌ Not implemented |
| Type System | ✅ Complete | ⚠️ Partial |
| Pattern Matching | ✅ Complete | ❌ Not implemented |
| F-Strings | ✅ Complete | ✅ Complete |

#### Performance

| Aspect | Rust | C |
|--------|------|---|
| Compilation Speed | Fast | Moderate |
| Execution Speed | Fast | Fast |
| Memory Usage | Moderate | Low |
| Binary Size | Moderate | Small |

#### Code Examples

##### F-String Processing

**Rust Implementation** ([lexer.rs:900-950](../../crates/auto-lang/src/lexer.rs#L900-L950)):
```rust
fn fstr(&mut self) -> AutoResult<Token> {
    // Rust uses match expressions and Result types
    // for robust error handling
    let note = self.lexer.fstr_note;
    let mut parts = vec![];
    // ... f-string parsing logic
    Ok(Token { kind: TokenKind::FStrStart, .. })
}
```

**C Implementation** ([lexer.c:564-620](autoc/lexer.c#L564-L620)):
```c
static Token lexer_fstr(Lexer* lexer) {
    // C uses manual memory management and return codes
    // Added in_fstr_expr flag to prevent infinite loops
    char note = lexer->fstr_note;
    // ... f-string parsing logic
    return token;
}
```

#### Key Differences

1. **Error Handling**:
   - Rust: Uses `Result<T, E>` for explicit error handling
   - C: Uses return codes and manual error checking

2. **Memory Management**:
   - Rust: Ownership system with borrow checker
   - C: Manual memory management with `malloc`/`free`

3. **Pattern Matching**:
   - Rust: Full pattern matching support in `is` expressions
   - C: Not yet implemented

4. **String Handling**:
   - Rust: Uses `AutoStr` with reference counting
   - C: Uses `AutoStr` with manual management via `astr_free()`

#### Bug Fixes

##### F-String Lexer Infinite Loop

The C implementation had a bug where the lexer would hang on `f"hello ${2 + 1} again"`. This was fixed by:

1. Adding `in_fstr_expr` flag to lexer state
2. Moving f-string detection before identifier check
3. Collecting tokens in temporary array to avoid buffer conflicts

```c
// lexer.h:18
typedef struct {
    // ... existing fields ...
    bool in_fstr_expr;  // Flag to prevent re-entering f-string mode
} Lexer;
```

This fix ensures that when processing `${...}` in f-strings, the lexer doesn't recursively enter f-string mode.

---

## Appendices

### Appendix A: Operator Precedence (Highest to Lowest)

1. Unary operators (`!`, `-`, `+`)
2. Multiplication (`*`, `/`)
3. Addition (`+`, `-`)
4. Comparison (`<`, `>`, `<=`, `>=`)
5. Equality (`==`, `!=`)
6. Logical AND (`&&`)
7. Logical OR (`||`)
8. Range (`..`, `..=`)
9. Assignment (`=`, `+=`, `-=`, `*=`, `/=`)

### Appendix B: Reserved Keywords

```
alias, as, break, const, else, enum, false, fn, for, has, if, in,
is, let, mut, nil, null, on, tag, true, type, union, use, var, when
```

### Appendix C: ASCII Art Grammar

```
Program
 └─ Stmt*
     ├─ VarStmt
     │   └─ ("let" | "mut" | "const" | "var") IDENTIFIER Type? "=" Expr
     ├─ IfStmt
     │   └─ "if" Expr Stmt ("else" Stmt)?
     ├─ ForStmt
     │   └─ "for" IDENTIFIER? "in" Expr Stmt
     ├─ WhileStmt
     │   └─ "while" Expr Stmt
     ├─ BlockStmt
     │   └─ "{" Stmt* "}"
     └─ ExprStmt
         └─ Expr
             └─ Assignment
                 └─ LogicalOr ("=" | "+=" | "-=" | "*=" | "/=" LogicalOr)?
```

---

**Document Version**: 0.1-draft
**Status**: Work in Progress
**Feedback**: Please report issues and suggest improvements via GitHub issues or pull requests.
