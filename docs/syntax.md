# AutoLang Syntax Reference

This document provides a comprehensive reference for AutoLang syntax, types, and language features.

---

## Table of Contents

- [Identifier Naming Rules](#identifier-naming-rules)
- [Storage Types](#storage-types)
  - [let (Immutable)](#let-immutable)
  - [var (Mutable)](#var-mutable)
  - [const (Constant)](#const-constant)
- [Basic Types](#basic-types)
  - [Numeric Types](#numeric-types)
  - [Arrays](#arrays)
  - [Objects](#objects)
  - [Grid](#grid)

---

## Identifier Naming Rules

Identifiers in AutoLang can contain letters, digits, underscores, and hyphens. However, there are special rules for hyphens to distinguish between identifiers and subtraction operations.

### Hyphenated Identifiers

A hyphen (`-`) is treated as part of an identifier when followed by a valid identifier character (letter or underscore, but not a digit).

| Syntax | Meaning |
|--------|---------|
| `preview-card` | Single identifier (hyphen is part of the name) |
| `a - b` | Subtraction (spaces required around `-`) |
| `a-b` | Single identifier |
| `a -b` | `a` followed by unary minus `-b` |
| `a-1` | `a` minus `1` (hyphen before digit is subtraction) |

**Rule:** Subtraction must have spaces on both sides. A hyphen is part of an identifier only if followed by a letter or underscore.

### Examples

```auto
// Valid identifiers with hyphens
let my-variable = 42
let preview-card = {title: "Hello"}
let button-active = true

// Subtraction (spaces required)
let result = a - b  // This is subtraction
let result = a -b   // This is 'a' then unary minus 'b'

// Invalid: hyphen before digit is subtraction
let x = value-10  // This parses as 'value' minus '10', not an identifier

// Valid: hyphen before letter
let x = value-x   // This is a single identifier 'value-x'
```

### Use Cases

Hyphenated identifiers are particularly useful for:

1. **CSS/UI naming**: `primary-button`, `active-tab`, `modal-overlay`
2. **Configuration keys**: `max-width`, `font-size`, `background-color`
3. **Descriptive names**: `user-profile`, `search-results`, `error-message`

---

## Storage Types

AutoLang provides four types of storage for variables, each with different mutability and type stability characteristics.

### let (Immutable)

**let** declares bindings that cannot be changed after assignment. Similar to Rust's `let`.

```auto
// let binding
let b = 1

// Error! let bindings cannot be modified
b = 2  // Compilation error

// Can be used to compute new values
let f = e + 4

// Shadowing: let can be redeclared, but type cannot change
let b = b * 2  // Shadowing, type is still int
```

**Use cases:**
- Values that don't need modification
- Function parameters (default)
- Intermediate computation results

### var (Mutable)

**var** declares bindings whose values can be changed, but once the type is determined it cannot change. Similar to C/C++ variables.

```auto
// var definition, type inferred
var a = 1

// var with explicit type
var b bool = false

// Multiple variable declarations
var c, d = 2, 3

// var can be modified (assignment)
a = 10

// Swap two variables
c, d = d, c

// Error! Type cannot change
a = "hello"  // Compilation error: a's type is int, cannot assign str
```

**Use cases:**
- Local state that needs modification
- Loop counters
- Accumulators

### const (Constant)

**const** declares bindings that cannot be changed and can only be declared in global scope. Similar to Rust's `const`.

```auto
// const definition: constants are global only
const PI = 3.14

// Error! const cannot be modified
PI = 3.15  // Compilation error

// const can be used anywhere
fn area(radius float) float {
    PI * radius * radius
}
```

**Use cases:**
- Global configuration
- Mathematical constants
- Magic number replacement

---

## Basic Types

### Numeric Types

AutoLang supports basic numeric types:

```auto
// Integer types
let a int = 42         // Signed integer (i32)
let b uint = 42        // Unsigned integer (u32)
let c byte = 42        // Byte (u8)

// Floating point types
let d float = 3.14     // Double precision (f64)

// Boolean type
let e bool = true
let f bool = false

// Nil type
let g nil = nil        // nil is a special zero-sized type
```

**Special behavior of nil type:**
```auto
// nil is a special type, it's zero-sized
var c = nil

// Operations involving nil always return nil
let d = nil + 1  // d is nil
```

### Arrays

Arrays are fixed-size sequences of homogeneous elements.

```auto
// Array literal
let arr = [1, 2, 3, 4, 5]

// Index access
println(arr[0])    // 1 (first element)
println(arr[-1])   // 5 (last element)

// Slicing
let slice = arr[1..3]   // [2, 3]
let slice1 = arr[..4]    // [1, 2, 3, 4]
let slice2 = arr[3..]    // [4, 5]
let slice3 = arr[..]     // [1, 2, 3, 4, 5] (full copy)

// Ranges
let r = 0..10    // 0 <= r < 10 (half-open)
let r1 = 0..=10  // 0 <= r <= 10 (closed)
```

**Array operations:**
```auto
var arr = [1, 2, 3, 4, 5]

// Modify element
arr[0] = 10

// Array length
let len = arr.len()

// Iterate array
for n in arr {
    println(n)
}

// Iterate with index
for i, n in arr {
    println(f"arr[{i}] = {n}")
}

// Modify array values (using ref)
var arr = [1, 2, 3, 4, 5]
for ref n in arr {
    n = n * n
}
println(arr)  // [1, 4, 9, 16, 25]
```

### Objects

Objects are collections of key-value pairs, similar to maps, dicts, or hashes in other languages.

```auto
// Object literal
var obj = {
    name: "John",
    age: 30,
    is_student: false
}

// Access object members
println(obj.name)   // "John"

// Member assignment
obj.name = "Tom"

// Method calls
println(obj.get_or("name", "Unknown"))     // "Tom"
println(obj.get_or("job", "Unknown"))      // "Unknown" (default value)

println(obj.get_or_insert("name", 10))     // "Tom" (exists)
println(obj.get_or_insert("job", "Dev"))   // "Dev" (insert and return)

// Get all members
println(obj.keys())    // ["name", "age", "is_student"]
println(obj.values())  // ["Tom", 30, false]
println(obj.items())   // [("name", "Tom"), ("age", 30), ...]

// Iterate object
for k, v in obj {
    println(f"obj[{k}] = {v}")
}

// Delete member
obj.remove("name")
```

### Grid

Grid is AutoLang's two-dimensional array structure, designed specifically for tabular data. Grid can be extended to DataFrame/Tensor-like multi-dimensional structures for AI development and Python interaction.

#### Basic Grid

```auto
// Define a Grid
let data = grid {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// Convert to JSON
var json = data.to_json()
```

Generated JSON:
```json
{
    "data": [
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9]
    ]
}
```

#### Grid with Column Names

```auto
// Define Grid with column names
let data = grid("a", "b", "c") {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// Convert to named JSON format
let json = data.to_json(named: true)
```

Generated JSON:
```json
{
    "grid": {
        "cols": ["a", "b", "c"],
        "rows": [
            {"a": 1, "b": 2, "c": 3},
            {"a": 4, "b": 5, "c": 6},
            {"a": 7, "b": 8, "c": 9}
        ]
    }
}
```

#### Grid Operations

```auto
let data = grid("a", "b", "c") {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// Get grid info
println(data.shape())   // (3, 3)
println(data.width())   // 3
println(data.height())  // 3

// Row access
println(data[0])        // [1, 2, 3]
println(data[1])        // [4, 5, 6]
println(data[2])        // [7, 8, 9]

// Or use row method
println(data.row(0))    // [1, 2, 3]

// Column access
println(data(0))        // [1, 4, 7]
println(data(1))        // [2, 5, 8]
println(data(2))        // [3, 6, 9]

// Or use col method
println(data.col(0))    // [1, 4, 7]

// Matrix operations
let transposed = data.transpose()
let sum = data.sum()
let mean = data.mean()
let std = data.std()
let min = data.min()
let max = data.max()
```

---

## Functions

### Function Definition

```auto
// Basic function definition
fn add(a int, b int) int {
    a + b
}

// No return value
fn greet(name str) {
    println(f"Hello, ${name}!")
}

// Multiple return values (using tuples)
fn divmod(a int, b int) (int, int) {
    (a / b, a % b)
}

// Usage
let (quot, rem) = divmod(10, 3)
```

### Lambda Expressions

Lambda expressions (anonymous functions) provide a concise way to define functions.

```auto
// Lambda expression
let mul = |a int, b int| a * b

// Call
println(mul(3, 4))  // 12

// Simplified lambda (type inference)
let add = |a, b| a + b

// Multi-statement lambda
let complex = |a, b| {
    let temp = a + b
    temp * temp
}
```

### Higher-Order Functions

Functions can be passed as parameters and returned as values.

```auto
// Function as parameter
fn calc(op |int, int| int, a int, b int) int {
    op(a, b)
}

// Function as return value
fn get_op(op_type str) |int, int| int {
    is op_type {
        "add" => |a, b| a + b
        "sub" => |a, b| a - b
        "mul" => |a, b| a * b
        "div" => |a, b| a / b
        else => |a, b| 0
    }
}

// Usage
let add = |a int, b int| a * b
calc(add, 2, 3)         // 6
calc(|a, b| a / b, 6, 3)  // 2

let op = get_op("add")
op(1, 2)  // 3
```

### Function Calls

```auto
// Normal call
let result = add(1, 2)

// Named parameters (if supported)
let result = create_user(name: "Alice", age: 30)

// Method call
obj.method()
obj.method(arg1, arg2)

// Chained calls
obj.method1().method2().method3()

// Optional parameters
fn greet(name str, greeting str = "Hello") {
    println(f"${greeting}, ${name}!")
}

greet("Bob")                  // Hello, Bob!
greet("Bob", "Hi")            // Hi, Bob!
```

---

## Value Passing

In AutoLang, values can be passed in several forms: copy, reference, move, and pointer. Each approach has different performance and semantic characteristics.

### Copy Passing

**Copy passing** directly duplicates the data.

```auto
// Numeric types: default copy passing
let a = 1
let b = a     // b is a copy of a
var c = a     // c is a copy of a, and c can be modified
c = 2
println(c)    // 2
println(a)    // 1 (a unchanged)
```

**Characteristics:**
- Simple and easy to understand
- Efficient for small data types (int, bool, etc.)
- Inefficient for large data types (arrays, objects)

### Reference Passing

**Reference passing** avoids data copying but doesn't allow modifying the original data.

```auto
// Reference types: default reference passing
let a = [1, 2, 3, 4, 5]  // Arrays are reference types by default
let b = a  // b is a reference to a, using b is like using a. Only one array in memory.

// Error! Since a is immutable, mutable c cannot reference it
var c = a  // Compilation error

// If you want to modify, explicitly copy it
var d = copy a
d[0] = 9  // d = [9, 2, 3, 4, 5]
println(a)  // a = [1, 2, 3, 4, 5], a array unchanged
```

**Value types vs Reference types:**

- **Value types**: int, float, bool, byte, simple structs (like `Point{x, y}`)
  - Default copy passing
  - Low copy cost

- **Reference types**: arrays, objects, strings, etc.
  - Default reference passing
  - High copy cost

### Move Passing

**Move passing** transfers ownership to the target binding. After transfer, the original binding cannot be used.

```auto
// Move passing
let a = [1, 2, 3, 4, 5]
let b = move a  // After transfer, a cannot be used

// Error! a can no longer be used
println(a)  // Compilation error

var c = move b  // b transferred to c, since it's a move, c can choose to modify
c[0] = 9  // c = [9, 2, 3, 4, 5]

// Error! b can no longer be used
println(b)  // Compilation error
```

**Move characteristics:**
- Zero copy (high performance)
- Original variable invalidated after transfer
- Compiler checks lifetimes
- `c` can have different properties from `b` (like `mut`)

> **Note**: Move and pointer are advanced features. Early versions of AutoLang may not fully implement them, they are included as design specifications.

### Pointers

**Pointers** create a new pointer to the same address for low-level operations. Pointers are only used in low-level system programming, so they must be in `sys` blocks.

See the next section [References and Pointers](#references-and-pointers).

---

## References and Pointers

Copy and move directly manipulate data, while references and pointers indirectly manipulate data.

The main differences between references and pointers:

1. **References** are primarily for avoiding copying (e.g., function parameters), convenient access. Although it's actually indirect access, the compiler optimizes the experience to make it feel like direct use.

2. **Pointers** have more low-level functionality: they can get addresses and even perform address arithmetic. These operations are needed in low-level system code, so they must be executed in `sys` blocks (similar to Rust's `unsafe` blocks).

### References

```auto
// Reference
let a = [0..99999]  // Large array
let b = a  // If b directly copies a's value, it would copy a

let c = ref a  // c is just a "reference view" of a, stores no data itself, no copy operation

// Error: References cannot modify original value
b = 2  // Compilation error (assuming b is ref)

// Reference in function parameters
// The `buf` parameter here is actually a reference
fn read_buffer(buf Buffer) {
    for n in buf.data {
        println(n)
    }
}

// Mutable reference: for modifying variables
var x = 1
fn inc(a mut ref int) {
    a += 1
}
inc(x)
println(x)  // 2
```

### Pointers

Unlike references, pointers can modify the original value because they point to the same address.

```auto
// Pointers can modify original value
var x = 1
sys {
    mut p = ptr x
    p.target += 1  // Indirectly modify x's value, note this is different from C, uses `.target`
}
println(x)  // 2

// Pointer in function parameters
// When calling a function, pointer-type parameters can modify the original value
var m = 10
fn inc(a ptr int) {
    a += 10
}
inc(m)
println(m)  // 20

// Pointer address arithmetic
sys {  // Note: address arithmetic must be in sys block
    var arr = [1, 2, 3, 4, 5]
    mut p = ptr arr  // p's type is Ptr<[5]int>
    println(p)  // [1, 2, 3, 4, 5]

    p[0] = 101  // Directly modify arr[0]'s value
    println(arr)  // [101, 2, 3, 4, 5]

    mut o = p  // Remember p's address

    p.inc(2)  // Address increment by 2, now p points to arr[2]
    println(p)  // [3, 4, 5]

    println(o[0])  // 101
    p.jump(o)  // Jump back to o
    println(p)  // [101, 2, 3, 4, 5]
}
```

---

## Control Flow

AutoLang provides rich control flow statements.

### Conditionals

```auto
// if-else
if a > 0 {
    println("a is positive")
} else if a == 0 {
    println("a is zero")
} else {
    println("a is negative")
}

// if expression
let abs = if x >= 0 { x } else { -x }

// Nested if
if condition1 {
    if condition2 {
        // ...
    }
}
```

### Loops

```auto
// for loop: iterate array
for n in [1, 2, 3] {
    println(n)
}

// for loop: modify array values
var arr = [1, 2, 3, 4, 5]
for ref n in arr {
    n = n * n
}
println(arr)  // [1, 4, 9, 16, 25]

// for loop: range
for n in 0..5 {
    println(n)  // 0, 1, 2, 3, 4
}

// for loop: indexed loop
for i, n in arr {
    println(f"arr[{i}] = {n}")
}

// Infinite loop
var i = 0
loop {
    println("loop")
    if i > 10 {
        break
    }
    i += 1
}

// while loop
while condition {
    // ...
}

// break and continue
for i in 0..10 {
    if i == 5 {
        break    // Exit loop
    }
    if i % 2 == 0 {
        continue // Skip this iteration
    }
    println(i)
}
```

### Pattern Matching (is)

The `is` statement is for pattern matching, similar to switch/match in other languages but more powerful.

```auto
// Exact match
is a {
    41 => println("a is 41")
    42 => println("a is 42")
}

// Multiple values
is a {
    42 or 43 or 44 => println("a is a little bigger")
}

// Range match
is a {
    in 0..9 => println("a is a single digit")
    in 10..99 => println("a is two digits")
}

// Conditional match
is a {
    if a > 10 => println("a is a big number")
    if a < 0 => println("a is negative")
}

// Type check
is a {
    as str => println("a is a string")
    as int => println("a is an integer")
    as float => println("a is a float")
}

// else branch
is a {
    1 => println("one")
    2 => println("two")
    else => println("other")
}

// Combined usage
is a {
    // is for exact match
    41 => print("a is 41")
    // Multiple different values
    42 or 43 or 44 => print("a is a little bigger")
    // in for range match
    in 0..9 => print("a is a single digit")
    // if for conditional match
    if a > 10 => print("a is a big number")
    // as for type check
    as str => print("a is a string")
    // Other cases
    else => print("a is a weird number")
}
```

---

## Type System

AutoLang provides a powerful type system supporting type aliases, type combinations, custom types, and more.

### Type Aliases

```auto
// Type alias
type MyInt = int

let a MyInt = 42
```

Type aliases are equivalent to `typedef` in C/C++, they don't create new types, just give existing types a new name.

### Type Combinations

```auto
// Type combination (union type)
type Num = int | float

fn add(a Num, b Num) Num {
    a + b
}

add(1, 2)      // OK
add(1, 2.0)    // OK
add(1.0, 2.0)  // OK
// add(1, "2") // Error!
```

### Custom Types

```auto
// Custom type
type Point {
    x int
    y int

    // Method
    fn distance(other Point) float {
        use std.math.sqrt
        // Here `x` is equivalent to `this.x` or `self.x` in other languages
        // If names conflict, use `.x` for member, `x` for parameter
        sqrt((.x - other.x) ** 2 + (.y - other.y) ** 2)
    }
}
```

In type methods, instance members (like `x` and `y` in the example above) can be accessed directly. This is because AutoLang adds the instance's scope to the method's scope during method calls.

If an instance member's name conflicts with a parameter or local variable name, use `.x` to distinguish:
- `.x` means member (equivalent to `this.x` or `self.x`)
- `x` means parameter or ordinary variable

#### Member Access and self

```auto
type Point {
    x int
    y int

    // Direct member access (no conflict)
    fn distance(other Point) float {
        sqrt((.x - other.x) ** 2 + (.y - other.y) ** 2)
    }

    // Name conflict, use .x to distinguish
    fn move(x int, y int) {
        .x = x  // Member x
        .y = y  // Member y
    }
}
```

If you want to use the instance itself directly, use `self`:

```auto
type Node {
    parent *Node
    kids []*Node

    pub fn mut add(mut kid *Node) {
        kid.parent = &self
        .kids.add(kid)
    }
}
```

#### Constructors

```auto
type Point {
    x int
    y int

    // Method
    fn distance(other Point) float {
        sqrt((.x - other.x) ** 2 + (.y - other.y) ** 2)
    }
}

// Default constructor
var myint = MyInt(10)
print(myint)

// Named constructor
var p = Point(x: 1, y: 2)
println(p.distance(Point(x: 4, y: 6)))

// Custom constructor
// Note: `static` indicates a static method, usually for constructors
// Static methods cannot use `.` to access instance members
Point {
    pub static fn new(x int, y int) Point {
        Point{x, y}
    }

    pub static fn stretch(p Point, scale float) Point {
        Point{x: p.x * scale, y: p.y * scale}
    }
}

// Use custom constructor
var p1 = Point.new(1, 2)
var p2 = Point.stretch(p1, 2.0)
```

### Extension Methods

Besides defining methods inside types, you can also "extend" types with new methods externally. The keyword for extension methods is `ext` (extends).

```auto
type Point {
    pub x int
    pub y int
}

ext Point {
    pub fn to_str() str {
        f"Point(${.x}, ${.y})"
    }
}
```

Note, unlike internal methods, extension methods can only access public members of `Point`, so we added `pub` modifier to `x` and `y` in the example above.

If you need to access private variables, just define the method inside the type.

The use case for extension methods is to add new functionality to types defined in third-party libraries or even system types.

For example, to add a new feature to the system string type `str`:

```auto
ext str {
    pub fn shape_shift() {
        for c in self {
            if c.is_upper() {
                c.lower()
            } else {
                c.upper()
            }
        }
    }
}

let s = "HellO"
let t = s.shape_shift()
assert_eq(t, "hELLo")
```

---

## Specs (Traits)

AutoLang extends Rust's trait concept to support more pattern matching. In AutoLang, these structures for determining type characteristics are called **Specs**.

AutoLang has three types of specs:

1. **Interface Spec**: Similar to Java's Interface and Rust's trait
2. **Expression Spec**: Similar to TypeScript's union types
3. **Predicate Spec/Function Spec**: Compile-time predicate functions

### Interface Spec

Similar to Java's Interface and Rust's trait, can be determined by the list of methods a type supports.

```auto
// Define interface spec
spec Printer {
    // Types implementing Printable must have print method
    fn print()
}

// Custom type
type MyInt {
    data int

    // Directly implement interface method
    pub fn print() {
        println(.data)
    }
}

// Can also implement by extending type methods
ext MyInt {
    pub fn print() {
        println(.data)
    }
}
```

**Multi-method interface:**

```auto
// Interface can contain multiple methods
spec Indexer<T> {
    fn size() usize
    fn get(n usize) T
    fn set(n usize, value T)
}

type IntArray {
    data []int

    pub static fn new(data int...) IntArray {
        IntArray{data: data.pack()}
    }

    // Implement Indexer interface
    pub fn size() int {
        data.len()
    }

    pub fn get(n int) int {
        data[n]
    }

    pub fn set(n int, value int) {
        data[n] = value
    }
}
```

### Expression Spec

Similar to TypeScript's union types.

```auto
// Expression spec
spec Number = int | uint | byte | float

// Use expression spec
fn add(a Number, b Number) Number {
    a + b
}

add(1, 2)      // OK
add(1, 2.0)    // OK
// add(1, "2") // Error!

// If name is too long, can also write:
fn <T = Number> add(a T, b T) T {
    a + b
}
```

**Type alias:**

Expression specs can also implement type aliases:

```auto
spec MyInt = int
```

At this point, MyInt is equivalent to int and can be used anywhere int is needed. For C/C++ programmers, this is equivalent to a macro.

### Predicate Spec

Calls a predicate function at compile time. If it returns true, the type check passes.

Predicate functions for type specs have `type` parameters and `bool` return values.

```auto
// Predicate function
fn predicate(t type) bool {
    // ...
    true
}
```

**Example: IsIterable Predicate**

```auto
// Predicate function
fn IsIterable(t type) bool {
    is t {
        // Is an array, element type can be anything
        as []any => true
        // Or has next() method
        if t.has_method("next") => true
        // Or implements Indexer interface
        as Indexer => true
        else => false
    }
}

// Here parameter arr's type must pass IsIterable(T) check to call, otherwise error
// Note: uses `if` expression, means calling predicate function at compile time
fn add_all(arr if IsIterable) {
    mut sum = 0
    for n in arr {
        sum += n
    }
    return sum
}

let arr = [1, 2, 3, 4, 5]
// OK, because arr is a `[]int` array
add_all(arr)

var my_arr = IntArray.new(1, 2, 3, 4, 5)
// OK, because my_arr implements Indexer interface
add_all(my_arr)

var d = "hello"
// Error! d is neither a []int array nor implements Indexer interface
// add_all(d)
```

---

## Enums

> **Note**: Enum functionality is not yet implemented, following is the design specification.

### Basic Enums

```auto
enum Axis {
    Vertical   // 0
    Horizontal // 1
}

// Usage
let axis = Axis.Vertical
println(axis)  // 0
```

### Enums with Members

```auto
// Enum with members
enum Scale {
    name str

    S("Small")
    M("Medium")
    L("Large")
}

// Enum variable
var a = Scale.M

// Access enum member
println(a.name)  // "Medium"

// Enum matching
is a as Scale {
    Scale.S => println("a is small")
    Scale.M => println("a is medium")
    Scale.L => println("a is large")
    else => println("a is not a Scale")
}
```

### Union Enums

```auto
// Union enum (similar to Rust's enum)
enum Shape union {
    Point(x int, y int)
    Rect(x int, y int, w int, h int)
    Circle(x int, y int, r int)
}

// Union enum matching
var s = get_shape(/*...*/)
is s as Shape {
    Shape.Point(x, y) => println(f"Point(${x}, ${y})")
    Shape.Rect(x, y, w, h) => println(f"Rect(${x}, ${y}, ${w}, ${h})")
    Shape.Circle(x, y, r) => println(f"Circle(${x}, ${y}, ${r})")
    else => println("not a shape")
}

// Get union enum data
var p = s as Shape.Point
println(p.x)
println(p.y)
```

---

## Generators

> **Note**: Generator functionality is not yet implemented, following is the design specification.

Generators allow lazy generation of value sequences, saving memory.

```auto
// Generator
fn fib() {
    mut a, b = 0, 1
    loop {
        yield b
        a, b = b, a + b
    }
}

// Use generator
for n in fib() {
    println(n)
    // 1, 1, 2, 3, 5, 8, 13, ...
}

// Or functional style
fib().take(10).foreach(|n| println(n))
```

**Generator methods:**

- `take(n)` - Take first n elements
- `skip(n)` - Skip first n elements
- `foreach(fn)` - Execute function on each element
- `map(fn)` - Map each element
- `filter(fn)` - Filter elements
- `collect()` - Collect into array

---

## Async

> **Note**: Async functionality is not yet implemented, following is the design specification.

AutoLang supports async programming model, similar to JavaScript's async/await or Rust's async/await.

```auto
// Any function
fn fetch(url str) str {
    // ...
}

// do keyword indicates async call
let r = do fetch("https://api.github.com")

// Returns a Future, need to wait for result
println(wait r)

// Multiple async calls
let tasks = for i in 1..10 {
    do fetch(f"https://api.github.com/${i}")
}
// Wait for all tasks to complete (or timeout)
let results = wait tasks
println(results)
```

**Async keywords:**

- `do` - Async function call, returns Future
- `wait` - Wait for Future to complete, get result
- `async` - Mark async function (optional)

**Concurrency patterns:**

```auto
// Execute multiple tasks concurrently
let tasks = [
    do fetch("url1"),
    do fetch("url2"),
    do fetch("url3")
]
let results = wait tasks

// Wait with timeout
let results = wait tasks timeout 5000  // 5 second timeout

// Select first completed
let result = wait race tasks
```

---

## Nodes

Nodes are special syntax in AutoLang for describing tree structures, particularly suitable for UI, XML, configuration, and other scenarios.

### Node Definition

```auto
// Node definition, can specify node attribute types in advance
node button {
    text str
    scale Scale
    onclick str
}
```

### Creating Nodes

```auto
// Create node with id btn1
button btn1 {
    text: "Click me"
    scale: Scale.M
    onclick: "click:btn1"
}
```

### Multi-level Nodes

```auto
// Multi-level node structure
ul {
    li {
        label {"Item 1"}
        button btn1 {
            text: "Click me"
            onclick: "click:btn1"
        }
        div { label {"div1"} }
    }
    li { label {"Item 2"} }
    li { label {"Item 3"} }
}
```

### Node and XML Correspondence

Node syntax corresponds one-to-one with XML syntax:

```auto
// Auto node syntax
parent(k1: "v1", k2: "v2") {
    kid(k1: "v1") {
        // more kids
    }
    kid(k2: "v2") {
        // more kids
    }
}
```

Equivalent to XML:

```xml
<parent k1="v1" k2="v2">
    <kid k1="v1" />
    <kid k2="v2" />
</parent>
```

### Nodes in UI

Node syntax is particularly suitable for describing UI interfaces:

```auto
// Define UI component
widget Counter {
    model {
        var count: i32 = 0
    }

    view {
        col {
            button("➕") {
                on_click: => count += 1
            }
            text(f"Count: ${count}")
            button("➖") {
                on_click: => count -= 1
            }
        }
    }
}
```

### Nodes in Configuration

Node syntax is also very suitable for describing XML-type configuration files:

```auto
// Represents a grade table
class(name: "Class 3-3", count: 55) {
    student(name: "Zihan", age: 18) {
        score(subject: "Chinese", score: 80) {}
        score(subject: "Math", score: 90) {}
        score(subject: "English", score: 85) {}
    }
    student(name: "Yichen", age: 19) {
        score(subject: "Chinese", score: 85) {}
        score(subject: "Math", score: 95) {}
        score(subject: "English", score: 80) {}
    }
}
```

Corresponding XML:

```xml
<class name="Class 3-3" count="55">
    <student name="Zihan" age="18">
        <score subject="Chinese" score="80" />
        <score subject="Math" score="90" />
        <score subject="English" score="85" />
    </student>
    <student name="Yichen" age="19">
        <score subject="Chinese" score="85" />
        <score subject="Math" score="95" />
        <score subject="English" score="80" />
    </student>
</class>
```

**Node + Programming = Powerful Configuration:**

```auto
// Call function to get student info
let info = fetch_class_scores("Class 3-3")

class(name: info.name, count: info.count) {
    for s in info.students {
        student(name: s.name, age: s.age) {
            for score in s.scores {
                score(subject: score.subject, score: score.score) {}
            }
        }
    }
}
```

This node-based format is very suitable for describing tree-like configurations like UI. Unlike XML and YAML, AutoConfig is programmable, making it more flexible and powerful.

---

## Summary

AutoLang provides rich syntax features:

- **Four storage types**: let (immutable), var (mutable), const (constant), var (dynamic)
- **Basic data types**: numeric, arrays, objects, Grid
- **Powerful function system**: supports Lambda, higher-order functions
- **Flexible value passing**: copy, reference, move, pointer
- **Rich control flow**: if, for, loop, is (pattern matching)
- **Type system**: type aliases, type combinations, custom types, extension methods
- **Spec system**: interface specs, expression specs, predicate specs
- **Advanced features** (planned): enums, generators, async, nodes

Through these features, Auto can adapt to different application scenarios, from embedded systems to scripting languages, from configuration files to UI frameworks.
