

![icon](docs/icon.png)

AutoLang is a programming language designed for automation and flexibility.

- **Automation**: AutoLang is designed for automation of many development tasks.

- **Flexible**: AutoLang supports multiple syntaxes, each tailored to a particular scenario.
    - AutoLang: AutoLang itself is a static/dynamic mixed language, and can be transpiled to C and Rust.
    - AutoScript: AutoLang can be used as a dynamic scripting language, and be embedded into Rust/C projects as a scripting engine.
    - AutoConfig: AutoLang is a superset of JSON, and can be used as a dynamic configuration language.
    - AutoDSL: AutoLang can be used as a DSL for UI applications.
    - AutoShell: AutoLang can be used as a cross-platform shell script.
    - Auto2C: AutoLang can be transpiled to C, and work with C in a mixed project managed by AutoMan.

- **Simplicity & Efficiency**:
    - As a scripting language, AutoLang provides simplicity and ease of use on par with Python.
    - As a static language, AutoLang is transpiled to C and Rust, providing similar performance to C and Rust.

- **Fullstack**: AutoLang is part of AutoStack, a fullstack platform for development.
    - Standard Library: A customizable standard library that supports BareMetal, RTOS and Linux/Windows/MacOS/Web.
    - Builder & Package Manager: AutoMan is a builder that supports Auto/C/Rust mixed projects. It's configured with AutoConfig.
    - UI Framework: AutoUI is a cross-platform UI framework based on Rust/GPUI, similar to Jetpack Compose. It now supports Windows/Linux/Mac, and will be extended to Web, Bevy and HarmonyOS.
    - Code Gen: AutoGen is a powerful code generation tool that supports C/Rust/HTML and more. See [Tutorial](docs/tutorials/autogen-tutorial.md).
    - IDE: As AutoUI is based on Zed/GPUI, a plugin system will be built with AutoLang, and provide an IDE.

## Execution Modes

**AutoVM** is the default execution engine for AutoLang (Plan 081). AutoVM is a fast bytecode VM that provides consistent behavior across all platforms.

### Mode Selection

AutoLang supports multiple execution and transpilation modes:

- **Script Execution** (default) - `auto <file.at>` to run scripts directly via AutoVM
- **Project Management** - Use subcommands like `auto build`, `auto run`, `auto fetch`
- **REPL** - Run `auto` without arguments to enter the interactive shell

#### Script Execution

You can run an AutoLang script directly:

```bash
auto hello.at
```

#### Project Management (AutoMan Integration)

AutoMan functionalities are now integrated into the `auto` command:

```bash
auto new myapp    # Create a new project
auto build         # Build the current project
auto run           # Run the built project
auto fetch         # Download dependencies
```

You can specify the execution mode in your `pac.at` file:

```auto
// pac.at
name: "myapp"
version: "1.0.0"
mode: "autovm"  // Options: "autovm", "c", "rust", "evaluator"

app("myapp") {
    dependencies: [
        "std:core",
        ("hal", mode: "c"),      # HAL in C
        ("crypto", mode: "rust"), # Crypto in Rust
    ]
}
```

### Mixed-Mode Projects

Different parts of your project can use different execution modes:

```auto
mode: "autovm"  # Main app uses AutoVM

app("mixed_app") {
    dependencies: [
        ("hal", mode: "c"),       # Hardware layer in C
        ("crypto", mode: "rust"),  # Crypto library in Rust
        "utils",                   # Utilities in AutoVM (default)
    ]
}
```

AutoVM bytecode can call C and Rust functions via the FFI layer, enabling seamless integration between modes.

### Environment Variable Override

You can override the execution mode at runtime:

```bash
# Force Evaluator mode (for debugging)
export AUTO_EXECUTION_ENGINE=evaluator
auto run myapp.at

# Force AutoVM mode
export AUTO_EXECUTION_ENGINE=autovm
auto run myapp.at
```

**Note**: The `use-bigvm` feature flag is deprecated. AutoVM is now the default and no feature flags are required.

### Learn More

- [Mode Selection Guide](docs/guides/mode-selection-guide.md)
- [FFI Usage Guide](docs/guides/ffi-usage-guide.md)
- [Migration Guide](docs/guides/migration-guide.md)
- [Plan 081: AutoVM as Default](docs/plans/081-autovm-default-mode.md)

## Language Tour

#### Hello World

```rust
// Script mode
print("Hello, world!")

// Static mode
fn main {
    println("Hello, world!")
}
```

#### 基本类型和存量

AutoLang supports basic types: int(i32), uint(u32), byte(u8), float(f64), bool, nil.

```rust
// normal storage value, not mutable
let a int = 1
a = 2 // Error! a is not mutable

// variable storage value, with type inference
var b = 2.2
b = 3.3

// const storage value, usually used as global constants
const PI = 3.14
PI = 3.15 // Error! PI is not mutable

// variant storage value, used in script mode (dynamic typing)
var c = true
// vars can mutate its value
c = false
// and its type!
c = "hello"

// nil is a special type, it's a zero-size type
c = nil

// operations that includes nil will always return nil
let d = nil + 1 // d is nil
```

TODO: translate more syntax overview examples into Language Tour

## Scenarios and Usages

### 1. Auto2C

A function in AutoLang:

```rust
// math.a
pub fn add(a int, b int) int {
    a + b
}
```

```rust
// main.a
use math::add

fn main {
    println(add(1, 2))
}
```

Transpiles to three C files: math.h, math.c and main.c:

```c
// math.h
#pragma once
#include <stdint.h>

int32_t add(int32_t a, int32_t b);
```

```c
// math.c
#include <stdint.h>
#include "math.h"

int32_t add(int32_t a, int32_t b) {
    return a + b;
}
```

```c
#include <stdio.h>
#include <stdint.h>
#include "math.h"

int main(void) {
    printf("%d\n", add(1, 2));
    return 0;
}
```

### 2. AutoConfig

AutoConfig is a superset of JSON, and can use scripting abilities of AutoLang.

```rust
// use Standard library
use std::str::upper;

// Variable
var dir = "/home/user/data"

// {key : value} pairs
root: dir
// Function call
root_upper: root.upper()

// String interpolation
views: f"${dir}/views"
// Find key in config
styles: f"${views}/styles"

// Object
attrs: {
    prefix: "auto"
    // Array
    excludes: [".git", ".auto"]
}
```

This dynamic config is evaluated to a big JSON object.


### 3. AutoMan

AutoConfig is used to configure AutoMan, the builder for Auto and C projects.

```rust
project: "osal"
version: "v0.0.1"

// Dependencies, can specify parameters
dep(FreeRTOS, "v0.0.3") {
    heap: "heap_5"
    config_inc: "demo/inc"
}

// Libraries in this project
lib(osal) {
    pac(hsm) {
        skip: ["hsm_test.h", "hsm_test.c"]
    }
    pac(log)
    link: FreeRTOS
}

// Ports to different platforms with support for multiple toolchains/IDEs
port(windows, cmake, x64, win32, "v1.0.0")
port(stm32, iar, arm_cortex_m4, f103RE, "v1.0.0")

// Executables
exe(demo) {
    // Static link
    link: osal
    // Specify output file name
    outfile: "demo.bin"
}
```

### 4. AutoShell

```rust
#!auto
// Built-in common libraries in script mode
print "Hello, world!"

// The following command will be converted to function call: `mkdir("src/app", p=true)`
mkdir -p src/app

cd src/app
touch main.rs

// Define variables and functions as usual scripting language
let ext = ".c"
fn find_c_files(dir) {
    ls(dir).filter(|f| f.endswith(ext)).sort()
}

// Call commands in a loop
touch "merged.txt"
for f in find_c_files("src/app") {
    cat f >> "merged.txt"
}

// Call async commands in a loop
let downloads = for f in readlines("remote_files.txt").map(trim) {
    async curl f"http://database.com/download?file=${f}"
}

// Wait for all downloads to complete
await downloads.join()
```

AutoShell is implemented by adding a special rule to AutoLang:

- When in shell scenarios, all `first level` statements will support a shell like call syntax.


For example:

```bash
grep -Hirn TODO .
```

will be converted to this normal Auto function call:

```rust
grep(key:"TODO", dir:".", H, i, r, n)
```

And if `grep()` is defined in `std::shell`, it will be called directly.
If not found, a compile error will be reported.

These Auto shell functions are actually implemented by Rust code, e.g.: [coreutils](https://github.com/uutils/coreutils)

### 5. AutoTemplate

```html
<html>
<head>
    <title>${title}</title>
</head>
<body>
    <h1>${title}</h1>
    <ul>
    $ for n in 1..10 {
        <li>Item $n</li>
    }
    </ul>
</body>
</html>
```

An Auto Template is actually a normal code embedded with Auto snippets.

We do a translation from the above HTML code into normal Auto code:

```rust
`<html>`
`<head>`
`    <title>${title}</title>`
`</head>`
`<body>`
`    <h1>${title}</h1>`
`    <ul>`
for n in 1..10 {
`        <li>Item $n</li>`
}
`   </ul>`
`</body>`
`</html>`
```

These are lines of strings (potentially with `$` interpolation), some of which are wrapped by `for` blocks;

In Template scenario, these lines are treated as string expression statements, and will be concatenated into a big string.

As a comparison, statements in normal Auto code are executed one by one, but only the last statement is returned.

AutoTemplate can work with any type of text.

AutoTemplate is the basis of `AutoGen`, which can generate many types of code.

### 6. AutoUI

[`AutoUI`](https://github.com/auto-stack/auto-ui) is a UI framework based on `Zed/GPUI`, supporting Windows/Linux/MacOS/Web.

AutoLang works as a DSL to describe UI components.

The syntax is similar to Kotlin, and the code organization is similar to Vue.js.

```rust
// Define a component
widget counter {
    // Model that stores reactive data
    model {
        var count: i32 = 0

        fn reset() {
            count = 0
        }
    }

    // View that describes UI layout
    view {
        cols {
            button("➕") {
                // callback function that works with data in the model
                on_click: => count += 1
            }
            text(f"Count: {count}")
            button("➖") {
                on_click: => count -= 1
            }
            icon("🔄") {
                on_click: => reset()
            }
            style {gap-2 w-full}
        }
    }

    style {
        // Style currently supports Tailwind CSS syntax
        "w-24 h-24 border-1 border-color-gray-300"
    }
}
```

A widget described above will be parsed into a `DynamicWidget` object, which can be directly drawn in `AutoUI`.

In this dynamic mode, widgets support live reloading.

Later, a static mode will be provided that transpiles the Auto code into Rust code, and the output UI executable could be as performant as native GPUI applications (like the Zed Editor).

## Syntax Overview

TODO: translate into English

### 存量

在AutoLang语言中，有三种类型的“存量”用于存储和访问数据：

- - 定量（`let`）：定量声明后不可修改，类似于Rust中的`let`。
- - 变量（`var`）：值可以自由改变，但类型确定后不能再改变。类似于C/C++中的普通变量，或Rust中的`let mut`。
- - 常量（`const`）：声明后不可修改，用于全局常量。类似于Rust中的`const`。

```rust
// Let - immutable
let b = 1
// Error! let cannot be modified
b = 2
// Can be used to compute new values
let f = e + 4
// A let can be redeclared, but the type cannot change
let b = b * 2

// Var definition, type can be inferred by the compiler
var a = 1
// Var definition with explicit type
var b bool = false
// Declare multiple variables
var c, d = 2, 3

// Var can be modified, also called "assignment"
a = 10
// Swap two variables
c, d = d, c

// Const definition: const can only be global
const PI = 3.14
```

### 数组

```rust
// Array
let arr = [1, 2, 3, 4, 5]

// Indexing
println(arr[0])
println(arr[-1]) // Last element

// Slicing
let slice = arr[1..3] // [2, 3]
let slice1 = arr[..4] // [1, 2, 3, 4]
let slice2 = arr[3..] // [4, 5]
let slice3 = arr[..] // [1, 2, 3, 4, 5]

// Range
let r = 0..10  // 0 <= r < 10
let r1 = 0..=10 // 0 <= r <= 10
```

### 对象

```rust
// Object
var obj = {
    name: "John",
    age: 30,
    is_student: false
}

// Access object member
println(obj.name)
// Member assignment
obj.name = "Tom"

// get or else
println(obj.get_or("name", "Unknown"))
// get or insert
println(obj.get_or_insert("name", 10))

// All members
println(obj.keys())
println(obj.values())
println(obj.items())

// Iterate object
for k, v in obj {
    println(f"obj[{k}] = {v}")
}

// Delete
obj.remove("name")
```

### Grid（二维数组）

Grid是AutoLang的二维数组，可以用于表格数据。Grid可以扩展为类似DataFrame/Tensor的多维结构，用于和Python交互，进行AI相关的开发。

```rust
// Define a Grid
let grid = grid(a:"first", b:"second", c:"third") {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// Convert to JSON
var json = grid.to_json()

// Equivalent to
var grid = {
    "cols": [
        {id: "a", name: "first"},
        {id: "b", name: "second"},
        {id: "c", name: "third"},
    ],
    "data": [
        {"a": 1, "b": 2, "c": 3},
        {"a": 4, "b": 5, "c": 6},
        {"a": 7, "b": 8, "c": 9},
    ]
}
```


### 函数

```rust
// Function definition
fn add(a int, b int) int {
    a + b
}

// Lambda
let mul = |a int, b int| a * b

// Function as parameter
fn calc(op |int, int| int, a int, b int) int {
    op(a, b)
}

// Function call
calc(add, 2, 3)
calc(mul, 2, 3)
```

### 值的传递

在AutoLang中，值的传递有以下几种方式：

- 拷贝（copy）：直接拷贝数据。
- 引用（ref）：通过引用传递，不拷贝数据，但不能修改原始数据。
- 转移（move）：将所有权转移到目标存量，转移后原始存量不能再使用。
- 指针（ptr）：创建指向同一地址的指针。用于底层操作。指针仅用于底层系统编程，必须放在`sys`代码块中。

引用比拷贝节省了内存和复制时间，但因为引用通过地址间接访问数据，访问时间略慢于拷贝。

对于较小的数据（如`int`、`float`、`bool`，或简单类型如`Point{x, y}`），拷贝代价很小，拷贝往往更合适。
这类类型称为“数值类型”。

对于较大的数据（如`Vec<T>`、`HashMap<K, V>`、`String`），拷贝代价较大，引用往往更合适。
这类类型称为“引用类型”。

因此，AutoLang针对不同的数据采用不同的传递策略：

1. 较小的“数值类型”默认使用拷贝传递。
2. 较大的“引用类型”默认使用引用传递。

示例：

```rust
// Value type: default copy passing
let a = 1
let b = a // b is a copy of a
var c = a // c is a copy of a, and c is mutable
c = 2
println(c) // 2
println(a) // 1 - a is unchanged
```

```rust
// Reference type: default reference passing
let a = [1, 2, 3, 4, 5] // Arrays are reference types by default
let b = a // b is a reference to a; using b is the same as using a. Only one array exists in memory.
var c = a // Error! Since a is immutable, mutable c cannot reference it.
var d = copy a // To modify, explicitly copy it.
d[0] = 9 // d = [9, 2, 3, 4, 5]
println(a) // a = [1, 2, 3, 4, 5], the array is unchanged
```

上面的例子中使用了`copy`关键字显式地进行拷贝。
但这样效率不高，因此有更好的方式：**转移（move）**。

```rust
// Move passing
let a = [1, 2, 3, 4, 5]
let b = move a // After the move, a can no longer be used
println(a) // Error! a can no longer be used
var c = move b // b is moved to c; since it is a move, c can choose to be mutable
c[0] = 9 // c = [9, 2, 3, 4, 5]
println(b) // Error! b can no longer be used
```

当`a`的值转移到`b`后，`a`的生命周期就结束了。
存量`a`不再存在，但数据继续在`b`中存活。

同样，当`b`转移到`c`时，由于转移操作转移了所有权，
`c`可以拥有与`b`不同的属性，例如`var`。

转移结合了拷贝和引用的优点，但代价是什么？
编译器需要能够逐行分析每个存量的生命周期，
程序员也需要能够判断某个存量何时已经被消耍。

很多Rust程序员与编译器斗争，就是因为没有完全理解每个存量的生命周期。

由于转移和指针都是高级功能，AutoLang的早期版本暂不实现；
它们仅作为设计规范记录在此。

### 引用和指针

拷贝和转移直接操作数据，而引用和指针则间接操作数据。

引用和指针的主要区别：

1. 引用主要用于避免拷贝（例如函数传参时），方便访问。尽管引用实际上是间接访问，但编译器优化了体验，使其看起来和直接使用一样。
2. 指针提供更多底层功能：可以获取地址、甚至进行地址运算。这些操作仅用于系统级底层代码，必须在`sys`代码块中执行（类似于Rust的`unsafe`块）。


```rust
// Reference
let a = [0..99999] // A very large array
let b = a // If a new value for b is created directly, the value of a would be copied
let c = ref a // c is a "reference view" of a; it does not store data itself and no copy is performed.
b = 2  // Error: references cannot modify the original value

// The `buf` parameter here is actually a reference
fn read_buffer(buf Buffer) {
    for n in buf.data {
        println(n)
    }
}

// var ref can be used to modify a variable:

var x = 1
fn inc(a var ref int) {
    a += 1
}
inc(x)
println(x) // 2
```

```rust
// Pointer

// Unlike references, pointers point to the same address as the original value, so the original value can be modified.

var x = 1
sys {
    var p = ptr x
    p.target += 1 // Indirectly modify x's value; note that unlike C, `.target` is used.
}
println(x) // 2

// When calling functions, pointer-type parameters can modify the original value
var m = 10
fn inc(a ptr int) {
    a += 10
}
inc(m)
println(m) // 20

// Pointers can also perform address arithmetic directly
sys { // Note: address arithmetic must be in a sys block
    var arr = [1, 2, 3, 4, 5]
    var p = ptr arr // p's type is Ptr<[5]int>
    println(p) // [1, 2, 3, 4, 5]
    p[0] = 101 // Directly modify arr[0]'s value
    println(arr) // [101, 2, 3, 4, 5]

    var o = p // Remember p's address

    p.inc(2) // Increment address by 2; now p points to arr[2]
    println(p) // [3, 4, 5]

    println(o[0]) // 101
    p.jump(o) // Jump back to o
    println(p) // [101, 2, 3, 4, 5]
}
```

### 控制流

```rust
// Conditional
if a > 0 {
    println("a is positive")
} else if a == 0 {
    println("a is zero")
} else {
    println("a is negative")
}

// Iterate array
for n in [1, 2, 3] {
    println(n)
}

// Iterate and modify array values
var arr = [1, 2, 3, 4, 5]
for ref n in arr {
    n = n * n
}
println(arr) // [1, 4, 9, 16, 25]

// Iterate a range
for n in 0..5 {
    println(n)
}

// Iterate with index
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

// Pattern matching, similar to switch/match
is a {
    // Exact match
    41 -> println("a is 41"),
    // as is used for type checking
    as str -> println("a is a string"),
    // in is used for range matching
    in 0..9 -> println("a is a single digit"),
    // if is used for conditional matching
    if a > 10 -> println("a is a big number"),
    // Default case
    else x-> println("a is a weird number")
}
```

### 枚举（计划中）

```rust
enum Axis {
    Vertical   // 0
    Horizontal // 1
}

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
println(a.name)

// Enum matching
is a {
    Scale::S -> println("a is small")
    Scale::M -> println("a is medium")
    Scale::L -> println("a is large")
    else -> println("a is not a Scale")
}


// Union enum
enum Shape union {
    Point(x int, y int)
    Rect(x int, y int, w int, h int)
    Circle(x int, y int, r int)
}

// Union enum matching
var s = get_shape(/*...*/)
is s as Shape {
    Point(x, y) -> println(f"Point($x, $y)")
    Rect(x, y, w, h) -> println(f"Rect($x, $y, $w, $h)")
    Circle(x, y, r) -> println(f"Circle($x, $y, $r)")
    else -> println("not a shape")
}
// Access union enum data
var p = s as Shape::Point
println(p.x, p.y)
```

### Object-Oriented Programming

Auto provides complete object-oriented programming support, including type definitions, inheritance, composition, and the spec system.

#### Type Definitions

```rust
// Define a type
type Point {
    x int
    y int

    // Instance method
    fn distance(other Point) float {
        sqrt((.x - other.x) ** 2 + (.y - other.y) ** 2)
    }

    fn info() str {
        f"Point(.x, .y)"
    }
}

// Create instance
var p = Point()
p.x = 1
p.y = 2
println(p.info())        // "Point(1, 2)"
println(p.distance(p))   // 0.0
```

#### Single Inheritance

Use the `is` keyword for single inheritance. Child types automatically inherit all fields and methods from the parent:

```rust
// Parent class
type Animal {
    name str

    fn speak() {
        print("Animal sound")
    }

    fn info() str {
        f"{.name}"
    }
}

// Child class inherits from parent
type Dog is Animal {
    breed str

    // Can override parent methods
    fn speak() {
        print("Woof!")
    }

    // Can add new methods
    fn fetch() {
        print("Fetching...")
    }
}

fn main() {
    let dog = Dog()
    dog.name = "Buddy"
    dog.breed = "Labrador"

    // Access inherited fields
    print(dog.name)

    // Call inherited method (overridden)
    dog.speak()  // "Woof!"

    // Call own method
    dog.fetch()
}
```

**Inheritance Features**:
- ✅ Field inheritance: Child types automatically include all parent fields
- ✅ Method inheritance: Child types automatically get all parent methods
- ✅ Method overriding: Child types can override parent methods
- ✅ Type checking: Inheritance relationships are verified at compile time

#### Composition

Use the `has` keyword for composition to integrate functionality from other types:

```rust
type Engine {
    power int

    fn start() {
        print("Engine started")
    }
}

type Car {
    has engine Engine

    fn drive() {
        .engine.start()
        print("Driving...")
    }
}
```

#### Spec System

Specs define interface contracts. Types can implement multiple specs:

```rust
// Define spec
spec Reader {
    fn read() str
    fn is_eof() bool
}

spec Writer {
    fn write(s str)
    fn flush()
}

// Implement spec (using 'as' keyword)
type File as Reader, Writer {
    path str

    fn read() str {
        // Read file
    }

    fn is_eof() bool {
        // Check if end of file
    }

    fn write(s str) {
        // Write to file
    }

    fn flush() {
        // Flush buffer
    }
}

// Polymorphic function
fn copy(src Reader, dst Writer) {
    while !src.is_eof() {
        let line = src.read()
        dst.write(line)
    }
    dst.flush()
}
```

#### Transpiler Support

Auto's OOP features are supported by both C and Rust transpilers:

**C Transpilation** (flat struct + method prefix):
```c
struct Dog {
    char* name;      // inherited field
    char* breed;     // own field
};

void Dog_Speak(struct Dog *self) {
    printf("%s\n", "Woof!");
}
```

**Rust Transpilation** (flat struct + impl block):
```rust
struct Dog {
    name: String,      // inherited field
    breed: String,     // own field
}

impl Dog {
    fn speak(&self) {
        println!("Woof!");
    }
}
```

> 📖 **More OOP Features**? See [Single Inheritance Implementation](docs/plans/021-single-inheritance.md) and [Spec Polymorphism Documentation](docs/plans/020-stdlib-io-expansion.md)

### 生成器（计划中）

```rust
// Generator
fn fib() {
    var a, b = 0, 1
    loop {
        yield b
        a, b = b, a + b
    }
}

// Using a generator
for n in fib() {
    println(n)
}

// Or in functional style
fib().take(10).foreach(|n| println(n))
```

### 异步（计划中）

```rust
// Any function
fn fetch(url str) str {
    // ...
}

// The `do` keyword indicates an async call
let r = do fetch("https://api.github.com")

// Returns a Future; wait for the result
println(wait r)

// Multiple async calls
let tasks = for i in 1..10 {
    do fetch(f"https://api.github.com/$i")
}
// Wait for all tasks to complete (or timeout)
let results = wait tasks
println(results)
```

### 节点

```rust
// Node
node button(id) {
    text str
    scale Scale
    onclick fn()
}

// Create a node
button("btn1") {
    text: "Click me"
    scale: Scale.M
    onclick: => println("button clicked")
}

// Multi-level nodes
node div(id) {
    kids: []any
}

node li(id) {
    text str
    kids: []div
}

node ul(id=nil) {
    kids: []li
}

node label(content) {
}

ul {
    li {
        label("Item 1: ")
        button("btn1") {
            text: "Click me"
            onclick: => println("button clicked")
        }
        div { label("div1")}
    }
    li { label("Item 2") }
    li { label("Item 3") }
}
```

## 使用与安装

AutoLang编译器仅依赖于Rust和Cargo。

```bash
> git clone git@gitee.com:auto-stack/auto-lang.git
> cd auto-lang
> cargo build --release
> cargo run --release
```

## 架构说明

AutoLang有一个主要实现（Rust编译器），支持五种执行模式：

1. **解释执行**: 直接运行AutoLang代码（REPL、脚本执行）
2. **转译到 C (a2c)**: 将AutoLang转译为C代码，用于嵌入式系统
3. **转译到 Rust (a2r)**: 将AutoLang转译为Rust代码，用于原生应用
4. **转译到 Python (a2p)**: 将AutoLang转译为Python代码，用于快速原型和Python生态集成
5. **转译到 JavaScript (a2j)**: 将AutoLang转译为JavaScript (ES6+) 代码，用于Web开发和Node.js

测试目录：
- `crates/auto-lang/test/a2c/` - Auto到C转译器测试
- `crates/auto-lang/test/a2r/` - Auto到Rust转译器测试
- `crates/auto-lang/test/a2p/` - Auto到Python转译器测试
- `crates/auto-lang/test/a2j/` - Auto到JavaScript转译器测试

## Python Transpiler (a2p)

AutoLang支持转译到 Python 3.10+，具备以下特性：

### 核心特性

- ✅ **完美 F-string 映射**: AutoLang和Python的f-string语法几乎相同
- ✅ **模式匹配**: 完整支持`match/case`语句（需要Python 3.10+）
- ✅ **智能类生成**: 自动检测`@dataclass`和普通类
- ✅ **类型支持**: 结构体、枚举、方法和继承
- ✅ **零依赖**: 生成的Python代码仅需标准库

### 使用方法

```bash
# Transpile AutoLang to Python
auto python hello.at

# Run the generated Python
python hello.py
```

### 代码示例

**AutoLang代码:**
```auto
type Point {
    x int
    y int

    fn modulus() int {
        .x * .x + .y * .y
    }
}

fn main() {
    let p = Point{x: 0, y: 0}
    print(f"Modulus: ${p.modulus()}")
}
```

**生成的Python代码:**
```python
class Point:
    def __init__(self, x: int, y: int):
        self.x = x
        self.y = y

    def modulus(self):
        return self.x * self.x + self.y * self.y

def main():
    p = Point(x=0, y=0)
    print(f"Modulus: {p.modulus()}")

if __name__ == "__main__":
    main()
```

### 语言映射

| AutoLang | Python | 说明 |
|----------|--------|------|
| `type Point { x int }` | `@dataclass\nclass Point:` | 无方法时使用@dataclass |
| `type Point { fn m() {} }` | `class Point:\n def __init__...` | 有方法时使用普通类 |
| `enum Color { Red }` | `class Color(Enum)` | 使用enum.Enum |
| `is x { 0 => print() }` | `match x:\n case 0:` | Python 3.10+ |
| `for i in 0..10` | `for i in range(0, 10)` | 范围转换为range() |
| `f"hello $name"` | `f"hello {name}"` | 自动转换变量语法 |

### 测试覆盖

当前支持 10 个测试用例，全部通过 ✅：

1. `000_hello` - 基础打印
2. `002_array` - 数组和索引
3. `003_func` - 函数
4. `006_struct` - 结构体定义 (@dataclass)
5. `007_enum` - 枚举定义 (class Enum)
6. `008_method` - 类方法
7. `010_if` - if/else语句
8. `011_for` - for循环
9. `012_is` - 模式匹配 (match/case)
10. `015_str` - F-strings

### 文档

完整的Python转译器文档请参考：[Python Transpiler Documentation](docs/python-transpiler.md)

### 限制

以下特性尚未实现：

- Lambda函数
- 块表达式
- If表达式（三元运算符）
- 枚举变体访问（如`Color.Red`）
- 结构体构造语法（如`Point{x: 1, y: 2}`）
- for循环中的enumerate

### Python 版本要求

- **最低版本**: Python 3.10+
- **原因**: `match/case`语句需要Python 3.10或更高版本

## JavaScript Transpiler (a2j)

AutoLang支持转译到 JavaScript ES6+，具备以下特性：

### 核心特性

- ✅ **完美 Template Literal 映射**: AutoLang的f-string语法与JavaScript模板字符串几乎相同
- ✅ **ES6+ 类**: 使用现代ES6 class语法生成结构体
- ✅ **模式匹配**: 完整支持`switch/case`语句
- ✅ **方法支持**: 自动将`.x`转换为`this.x`
- ✅ **动态类型**: JavaScript的动态类型与AutoLang完美匹配
- ✅ **零依赖**: 生成的JavaScript代码无需任何polyfills

### 使用方法

```bash
# Transpile AutoLang to JavaScript
auto java-script hello.at

# Run the generated JavaScript (requires Node.js)
node hello.js
```

### 代码示例

**AutoLang代码:**
```auto
type Point {
    x int
    y int

    fn modulus() int {
        .x * .x + .y * .y
    }
}

fn main() {
    let p = Point{x: 3, y: 4}
    let m = p.modulus()
    print(f"Modulus: $m")
}
```

**Generated JavaScript code:**
```javascript
class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }

    modulus() {
        return this.x * this.x + this.y * this.y;
    }
}

function main() {
    const p = new Point(3, 4);
    const m = p.modulus();
    console.log(`Modulus: ${m}`);
}

main();
```

### 语言映射

| AutoLang | JavaScript | 说明 |
|----------|-----------|------|
| `let x = 1` | `const x = 1` | 不可变变量使用const |
| `var x = 1` | `let x = 1` | 可变变量使用let |
| `type Point { x int }` | `class Point { constructor... }` | ES6类语法 |
| `enum Color { Red }` | `const Color = Object.freeze({...})` | 冻结对象防止修改 |
| `is x { 0 => print() }` | `switch (x) { case 0: ... }` | switch/case语句 |
| `for i in 0..10` | `for (let i = 0; i < 10; i++)` | 传统for循环 |
| `f"hello $name"` | `` `hello ${name}` `` | 模板字符串（反引号） |
| `.x` (方法内) | `this.x` | 自动转换self为this |
| `print(...)` | `console.log(...)` | 自动转换函数名 |

### 测试覆盖

当前支持 9 个测试用例，全部通过 ✅：

1. `000_hello` - 基础打印
2. `002_array` - 数组和索引
3. `003_func` - 函数声明和调用
4. `006_struct` - 结构体定义 (ES6 class)
5. `007_enum` - 枚举定义 (Object.freeze)
6. `008_method` - 类方法 (this conversion)
7. `010_if` - if/else语句
8. `011_for` - for循环
9. `012_is` - Pattern matching (switch/case)

### 文档

完整的JavaScript转译器文档请参考：[JavaScript Transpiler Documentation](docs/javascript-transpiler.md)

### 限制

以下特性尚未实现：

- Lambda函数 (arrow functions)
- If表达式（三元运算符`? :`）
- ES6模块 (import/export)
- 异步支持 (async/await)
- 生成器函数

### 环境要求

- **Node.js**: v12.0.0或更高版本（支持ES6+）
- **浏览器**: 任意现代浏览器（Chrome 51+, Firefox 54+, Safari 10+, Edge 15+）
- **原因**: 需要支持ES6+特性（class、模板字符串、箭头函数等）

---

**[English](README.md)**
