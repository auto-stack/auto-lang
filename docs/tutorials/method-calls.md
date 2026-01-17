# Method Calls in AutoLang

This tutorial explains how method calls work in AutoLang, including both regular methods and VM methods.

## Overview

AutoLang supports two types of methods:

1. **Regular Methods**: Defined in `ext` blocks or type definitions using AutoLang syntax
2. **VM Methods**: Implemented in Rust (marked with `fn.vm`) for performance or system-level operations

Both types can be called using the same dot syntax: `obj.method(args)`

## Regular Methods

Regular methods are defined in AutoLang using the `ext` statement:

```auto
ext Point {
    fn distance(self, other Point) int {
        let dx = .x - other.x
        let dy = .y - other.y
        sqrt(dx*dx + dy*dy)
    }
}

// Usage:
fn main() {
    let p1 = Point {x: 0, y: 0}
    let p2 = Point {x: 3, y: 4}
    let d = p1.distance(p2)  // Returns 5
}
```

**Key Features:**
- Defined in AutoLang syntax
- Can access instance via `self` or `.property` shorthand
- Full control over implementation logic
- Type-checked at compile time

## VM Methods

VM methods are implemented in Rust and marked with `fn.vm`:

```auto
ext str {
    /// Split string by delimiter into array of strings
    fn.vm split(delimiter str) []str
    fn.vm lines() []str
    fn.vm words() []str

    /// Get the length of the string
    fn len() int {
        .size  // AutoLang implementation
    }
}
```

**Key Features:**
- Implemented in Rust for performance
- Can do system-level operations (file I/O, string processing, etc.)
- Called using same dot syntax as regular methods
- Naming convention: registered as `{type}_{method}` (e.g., `str_split`)

## Calling Methods

All methods are called using dot syntax:

```auto
// Basic method call
let len = "hello".len()  // Returns 5

// Method with arguments
let words = "hello world".split(" ")  // Returns ["hello", "world"]

// Method chaining
let first = "hello world".split(" ")[0]  // Returns "hello"

// Mixed VM and regular methods
let trimmed = "  hello  ".trim().len()  // Returns 5
```

## Method Call Resolution

When you call `obj.method(args)`, the evaluator:

1. **Checks for VM methods** (for `Value::Instance` types)
   - Looks in `VM_REGISTRY` for instances of user-defined types

2. **Checks for ext methods** (Plan 035)
   - Looks for `"TypeName::method_name"` in the symbol table
   - Supports both static and instance methods

3. **Checks for VM builtin functions** (Plan 038)
   - Constructs VM function name: `{type}_{method}`
   - Looks for `str_split`, `str_lines`, etc.
   - Calls VM function with `self` as first argument

4. **Returns error if not found**
   - "Invalid dot expression {type}.{method}"

## Examples

### String Methods

```auto
fn main() {
    // VM methods (Plan 038)
    let text = "hello world"
    let words = text.split(" ")      // ["hello", "world"]
    let first = words[0]              // "hello"
    let count = words.len()          // 2

    // AutoLang methods
    let trimmed = text.trim()         // "hello world"
    let upper = text.upper()          // "HELLO WORLD"
    let lower = text.lower()          // "hello world"

    // Method chaining
    let result = text.upper().split(" ")[0]  // "HELLO"
}
```

### File Methods

```auto
fn main() {
    use auto.io: File

    // VM methods
    let f = File.open("test.txt")
    let content = f.read_all()       // Read entire file
    f.close()

    // Method chaining
    let lines = File.open("test.txt").read_all().lines()
}
```

### Array Methods

```auto
fn main() {
    let arr = [1, 2, 3, 4, 5]

    // Regular methods
    let len = arr.len()              // 5
    let first = arr[0]               // 1
    let last = arr[arr.len() - 1]   // 5
}
```

## When to Use Each Type

### Use Regular Methods When:

- ✅ Logic can be expressed in AutoLang
- ✅ You need type-level operations
- ✅ You want full control over implementation
- ✅ Method is simple and doesn't require system access

**Examples:**
- Data structure methods (Point.distance(), Circle.area())
- Business logic methods
- Algorithm implementations

### Use VM Methods When:

- ✅ Performance is critical (string processing, file I/O)
- ✅ Need system-level operations (file access, system calls)
- ✅ Implementation is complex or error-prone in AutoLang
- ✅ Need to interface with C/Rust libraries

**Examples:**
- String operations (split, trim, replace)
- File I/O (read_all, write_lines)
- Math operations (sqrt, sin, cos)
- System operations (getpid, time)

## Best Practices

1. **Prefer AutoLang methods** for clarity and maintainability
2. **Use VM methods** for performance-critical code
3. **Document VM methods** with clear examples
4. **Keep method signatures simple** and intuitive
5. **Use method chaining** judiciously (avoid deeply nested chains)

## Migration Guide

### From Global Functions to Methods

**Before (Plan 025 style):**
```auto
let words = str_split("hello world", " ")
let lines = str_lines("line1\nline2")
let trimmed = str_trim("  hello  ")
```

**After (Plan 038 style):**
```auto
let words = "hello world".split(" ")
let lines = "line1\nline2".lines()
let trimmed = "  hello  ".trim()
```

Both syntaxes still work, but the method call syntax is more consistent and idiomatic.

## Related Documentation

- [Plan 035: ext Statement](../plans/035-ext-statement.md) - Method definition system
- [Plan 038: VM Method Call Expressions](../plans/038-vm-method-call-expressions.md) - Implementation details
- [String Methods Reference](../stdlib/str.md) - Complete string API documentation
- [File Methods Reference](../stdlib/file.md) - File I/O methods

## See Also

- [Functions](./functions.md) - Function definitions and calls
- [Types](./types.md) - Type system overview
- [ext Statement](./ext-statement.md) - Adding methods to types
