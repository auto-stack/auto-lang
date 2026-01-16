# AutoLang For Loop Guide

## Overview

AutoLang's `for` statement is a versatile, multi-purpose looping construct that eliminates the need for separate `while` or `do-while` keywords found in other languages. A single `for` keyword can express four different looping patterns.

## The Four Forms of `for`

### 1. Infinite Loop

When no condition is provided, `for` creates an infinite loop:

```auto
fn main() {
    for {
        print("Running forever...")
        // Use 'break' to exit
        break
    }
}
```

**Transpiles to C:**
```c
while (1) {
    printf("Running forever...\n");
    break;
}
```

### 2. While Loop (Single Condition)

When given a single expression, `for` behaves like a traditional `while` loop:

```auto
fn main() {
    let mut i = 0
    for i < 10 {
        print(i)
        i = i + 1
    }
}
```

**Transpiles to C:**
```c
int i = 0;
while (i < 10) {
    printf("%d\n", i);
    i = i + 1;
}
```

**Note**: This is the equivalent of C's `while (condition) { ... }` statement.

### 3. For with Initializer and Condition (Go-style)

When you need to initialize variables before checking the condition:

```auto
fn main() {
    for let mut i = 0; i < 10 {
        print(i)
        i = i + 1
    }
}
```

**Transpiles to C:**
```c
for (int i = 0; i < 10; ) {
    printf("%d\n", i);
    i = i + 1;
}
```

This is similar to Go's `for init; condition { ... }` syntax.

### 4. Traditional C-Style For Loop

With initializer, condition, and step expression:

```auto
fn main() {
    for let mut i = 0; i < 10; i = i + 1 {
        print(i)
    }
}
```

**Transpiles to C:**
```c
for (int i = 0; i < 10; i = i + 1) {
    printf("%d\n", i);
}
```

### 5. Range-Based For Loop

Iterating over ranges or collections:

```auto
fn main() {
    let len = 10

    // Range with variable bounds
    for i in 0..len {
        print(i)
    }

    // Iterate over arrays (element only)
    let arr = [1, 2, 3, 4, 5]
    for n in arr {
        print(n)
    }
}
```

**Transpiles to C:**
```c
int len = 10;
for (int i = 0; i < len; i++) {
    printf("%d\n", i);
}

int arr[5] = {1, 2, 3, 4, 5};
for (int i = 0; i < 5; i++) {
    int n = arr[i];
    printf("%d\n", n);
}
```

### 6. Python-Style Iteration (Index + Element)

For loops can destructure tuples to get both index and element:

```auto
fn main() {
    let arr = [10, 20, 30, 40, 50]

    // Iterate with both index and value (Python-like)
    for i, v in arr {
        print(f"arr[$i] = $v")
    }
}
```

**Transpiles to C:**
```c
int arr[5] = {10, 20, 30, 40, 50};
for (int i = 0; i < 5; i++) {
    int v = arr[i];
    printf("arr[%d] = %d\n", i, v);
}
```

This is similar to Python's `enumerate()`:
```python
# Python equivalent
for i, v in enumerate(arr):
    print(f"arr[{i}] = {v}")
```

## Design Philosophy

AutoLang's unified `for` statement follows the principle of **orthogonality** - a single, well-designed construct that can express all looping patterns:

- **No need for `while` keyword** - `for condition { ... }` serves this purpose
- **No need for `do-while`** - Use infinite loop with conditional break
- **Consistent syntax** - All loops use the same `for` keyword
- **Clear intent** - The form used immediately indicates the loop pattern

## Comparison with Other Languages

### C/C++
```c
// C needs 4 different keywords
while (condition) { ... }      // → for condition { ... }
do { ... } while (condition);  // → for { ... if !condition { break } }
for (init; condition; step)    // → for init; condition; step { ... }
```

### Go
```go
// Go also uses 'for' for everything
for condition { ... }           // Same as Auto
for init; condition; step { }  // Same as Auto
for key, value := range map { } // → for key, value in map { ... }
```

### Rust
```rust
// Rust has 'loop' for infinite loops
loop { ... }              // → for { ... }
while condition { ... }   // → for condition { ... }
for i in 0..10 { ... }    // → for i in 0..10 { ... }
```

## Common Patterns

### Loop with Index and Element
```auto
fn process_array(arr []int) {
    for i, v in arr {
        print(f"index: $i, value: $v")
    }
}
```

### Conditional Break
```auto
fn search(arr []int, target int) int {
    for i, v in arr {
        if v == target {
            return i
        }
    }
    -1  // Not found
}
```

### Early Exit
```auto
fn find_first(arr []int) int {
    for v in arr {
        if v > 100 {
            return v
        }
    }
    0
}
```

## Summary

AutoLang's `for` statement is a powerful, unified looping construct that:

1. **Eliminates keyword clutter** - No need for `while`, `do-while`, or `loop`
2. **Covers all use cases** - From infinite loops to complex iteration patterns
3. **Maintains clarity** - Each form clearly expresses its intent
4. **Transpiles efficiently** - Maps directly to C's optimized loop constructs

The `for` statement demonstrates AutoLang's design philosophy: **fewer, more versatile language constructs that can express complex patterns elegantly**.
