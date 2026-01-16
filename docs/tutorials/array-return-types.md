# Array Return Types in AutoLang

**Status**: ✅ Fully Supported (as of 2025-01-17)

## Overview

AutoLang now supports functions that return array types. This enables implementation of advanced stdlib methods like `split()`, `lines()`, `words()`, etc.

## Syntax

```auto
fn function_name() []type {
    // array literal or variable
    [element1, element2, ...]
}
```

## Examples

### Example 1: Return Integer Array

```auto
fn get_numbers() []int {
    [1, 2, 3, 4, 5]
}

fn main() {
    let nums = get_numbers()
    print(nums[0])  // prints: 1
    print(nums[4])  // prints: 5
}
```

### Example 2: Return String Array

```auto
fn get_words() []str {
    ["hello", "world"]
}

fn main() {
    let words = get_words()
    print(words[0])  // prints: hello
    print(words[1])  // prints: world
}
```

### Example 3: Return Empty Array

```auto
fn empty_result() []int {
    []
}

fn main() {
    let result = empty_result()
    print(result.len())  // prints: 0
}
```

## C Transpilation

### Function Signature

AutoLang array return types are transpiled to C as pointer returns with an output size parameter:

```auto
// AutoLang
fn get_data() []int

// C
int* get_data(int* out_size)
```

### Function Body

Array literals are transpiled as static arrays with pointer returns:

```auto
// AutoLang
fn get_numbers() []int {
    [1, 2, 3, 4, 5]
}

// C
int* get_numbers(int* out_size) {
    static int _static_get_numbers[] = {1, 2, 3, 4, 5};
    *out_size = 5;
    return _static_get_numbers;
}
```

### Call Sites

Calling functions that return arrays generates size variable declarations:

```auto
// AutoLang
let nums = get_numbers()

// C
int _size_nums;
int* nums = get_numbers(&_size_nums);
```

## Type System

### Type Modifier Syntax

AutoLang uses **prefix** type modifiers (consistent design):

```auto
[]int    // Array of integers (not int[])
*int     // Pointer to integer (not int*)
[2]int   // Fixed-size array (not int[2])
?T       // Optional/Maybe type (not T?)
```

### Type Inference

The return type is inferred from the array literal or can be explicitly declared:

```auto
// Implicit type inference
fn get_numbers() []int {
    [1, 2, 3]  // Type inferred as []int
}

// Explicit return type (recommended for public APIs)
fn get_numbers() []int {
    [1, 2, 3]
}
```

## Implementation Details

### Evaluator (VM Interpreter)
- Arrays are returned by value (copied)
- Indexing works immediately: `nums[0]`
- No manual memory management

### C Transpiler
- Arrays returned as pointers to static data
- Size passed via output parameter
- Caller must not free the returned pointer

## Best Practices

### 1. Use Array Returns for Small, Fixed Data

```auto
// ✅ Good: Small, constant arrays
fn get_colors() []str {
    ["red", "green", "blue"]
}

// ⚠️  Use with caution: Large or dynamic data
// (Full implementation requires dynamic allocation support)
fn read_all_lines() []str {
    // TODO: needs more implementation
    []
}
```

### 2. Document Array Length

```auto
/// Returns array of 5 numbers
fn get_numbers() []int {
    [1, 2, 3, 4, 5]
}

/// Returns variable-length array
fn split(delimiter str) []str {
    // Actual length depends on input
    []
}
```

### 3. Handle Empty Arrays

```auto
fn safe_get() []int {
    if some_condition {
        [1, 2, 3]
    } else {
        []  // Empty array
    }
}

fn main() {
    let data = safe_get()
    if data.len() > 0 {
        print(data[0])
    }
}
```

## Limitations

Current implementation has these limitations:

1. **Dynamic Arrays**: Full implementation for variable-length arrays needs more work
2. **Memory Management**: C transpiler uses static arrays (not heap-allocated)
3. **Nested Arrays**: Multi-dimensional arrays `[][]int` not yet fully supported

## Future Work

- [ ] Dynamic array allocation (heap-based)
- [ ] Array append/push operations
- [ ] Full split() implementation with loop support
- [ ] Multi-dimensional array support

## Related Documentation

- [Plan 037](../plans/037-expression-and-array-support.md) - Implementation details
- [Plan 036](../plans/036-unified-auto-section.md) - Stdlib methods
- [For Loop Guide](for-loop-guide.md) - Array iteration patterns
