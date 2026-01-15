# Option and Result Types

AutoLang standard library for optional values and error handling.

## Overview

This module provides two fundamental types for safe error handling:

- **Option<T>**: Represents an optional value (Some or None)
- **Result<T, E>**: Represents a success (Ok) or error (Err)

These types are implemented in C for performance and exposed to AutoLang through FFI.

## Option Type

### Creating Options

```auto
// Create Some value
let opt = Option_some(42)

// Create None value
let opt = Option_none()
```

### Checking Options

```auto
let opt = Option_some(42)

// Check if Some
if Option_is_some(opt) {
    print("Has value")
}

// Check if None
if Option_is_none(opt) {
    print("No value")
}
```

### Unwrapping Options

```auto
// Unsafe unwrap (panics if None)
let value = Option_unwrap(opt)

// Unwrap with default
let value = Option_unwrap_or(opt, 0)

// Unwrap or NULL
let value = Option_unwrap_or_null(opt)
```

### Option Example

```auto
fn find_user(id int) Option<str> {
    if id == 1 {
        return Option_some("Alice")
    }
    return Option_none()
}

fn main() {
    let user = find_user(1)
    if Option_is_some(user) {
        let name = Option_unwrap(user)
        print(f"Found user: $name")
    } else {
        print("User not found")
    }
}
```

## Result Type

### Creating Results

```auto
// Create Ok value
let res = Result_ok(42)

// Create Err value
let res = Result_err("something went wrong")
```

### Checking Results

```auto
let res = Result_ok(42)

// Check if Ok
if Result_is_ok(res) {
    print("Success")
}

// Check if Err
if Result_is_err(res) {
    print("Error")
}
```

### Unwrapping Results

```auto
// Unsafe unwrap (panics if Err)
let value = Result_unwrap(res)

// Get error (panics if Ok)
let error = Result_unwrap_err(res)

// Unwrap with default
let value = Result_unwrap_or(res, 0)

// Get error or default
let error = Result_unwrap_err_or(res, "default error")
```

### Result Example

```auto
fn divide(a int, b int) Result<int, str> {
    if b == 0 {
        return Result_err("division by zero")
    }
    return Result_ok(a / b)
}

fn main() {
    let res = divide(10, 2)
    if Result_is_ok(res) {
        let value = Result_unwrap(res)
        print(f"Result: $value")
    } else {
        let error = Result_unwrap_err(res)
        print(f"Error: $error")
    }
}
```

## C API Reference

### Option Functions

```c
// Create Option with value
Option Option_some(void* value);

// Create empty Option
Option Option_none(void);

// Check if Some
bool Option_is_some(Option* self);

// Check if None
bool Option_is_none(Option* self);

// Get value (undefined if None)
void* Option_unwrap(Option* self);

// Get value or default
void* Option_unwrap_or(Option* self, void* default_value);

// Get value or NULL
void* Option_unwrap_or_null(Option* self);
```

### Result Functions

```c
// Create Ok with value
Result Result_ok(void* value);

// Create Err with error message
Result Result_err(const char* error);

// Check if Ok
bool Result_is_ok(Result* self);

// Check if Err
bool Result_is_err(Result* self);

// Get value (undefined if Err)
void* Result_unwrap(Result* self);

// Get error message (undefined if Ok)
const char* Result_unwrap_err(Result* self);

// Get value or default
void* Result_unwrap_or(Result* self, void* default_value);

// Get error or default
const char* Result_unwrap_err_or(Result* self, const char* default_error);

// Clean up resources
void Result_drop(Result* self);
```

## Memory Management

### Option

- **No heap allocation**: Option values are stack-allocated
- **No cleanup needed**: Option doesn't own the contained value
- **Reference semantics**: The value pointer must remain valid

### Result

- **Error message allocation**: `Result_err()` allocates memory for error message
- **Must cleanup**: Call `Result_drop()` to free error message
- **Value ownership**: Result doesn't own the contained value

Example:

```c
Result res = Result_err("error message");
// Use res...
Result_drop(&res);  // Free error message
```

## Best Practices

### When to Use Option

- Use `Option` when a value might be absent
- Use `Option_unwrap_or()` to provide default values
- Always check `Option_is_some()` before `Option_unwrap()`

Example:

```auto
fn get_first(arr array<int>) Option<int> {
    if arr.len() > 0 {
        return Option_some(arr[0])
    }
    return Option_none()
}
```

### When to Use Result

- Use `Result` for operations that can fail
- Include descriptive error messages in `Result_err()`
- Always check `Result_is_ok()` before `Result_unwrap()`
- Call `Result_drop()` after use in C code

Example:

```auto
fn read_file(path str) Result<str, str> {
    if !file_exists(path) {
        return Result_err("file not found")
    }
    // ... read file ...
    return Result_ok(contents)
}
```

### Error Propagation

Chain Results to propagate errors:

```auto
fn parse_and_validate(s str) Result<int, str> {
    let parsed = parse_int(s)
    if Result_is_err(parsed) {
        return Result_err("parse failed")
    }

    let value = Result_unwrap(parsed)
    if value < 0 {
        return Result_err("negative value")
    }

    return Result_ok(value)
}
```

## Testing

Run the tests:

```bash
# C tests
cd stdlib/result
gcc -o test test_option_result.c option.c result.c
./test

# AutoLang tests
auto.exe test_option_result.at
```

## Implementation Notes

### Generic Types in C

Both Option and Result use `void*` to store generic values:

```c
typedef struct {
    OptionTag tag;
    void* value;  // Can point to any type
} Option;
```

This is type-unsafe but necessary for C generics. Type safety is enforced at the AutoLang level.

### Tag-based Discrimination

Both types use enum tags for variant discrimination:

```c
typedef enum {
    Option_None,
    Option_Some
} OptionTag;

typedef enum {
    Result_Ok,
    Result_Err
} ResultTag;
```

### Memory Safety

- All functions check for NULL pointers
- Unsafe operations print error messages to stderr
- Result errors are heap-allocated and must be freed

## Performance

- **Stack allocation**: No heap allocation for struct itself
- **Zero-cost**: Tag checking is compile-time optimized
- **Small footprint**: Option is 16 bytes, Result is 24 bytes (on 64-bit)

## Future Enhancements

Planned features:

- [ ] Option map and and_then methods
- [ ] Result map and and_then methods
- [ ] Custom error types (beyond string)
- [ ] Automatic Result_drop() with RAII wrappers
- [ ] Integration with AutoLang exception system

## See Also

- [AutoLang Language Guide](../../docs/README.md)
- [Standard Library](../stdlib.md)
- [C FFI Guide](../../docs/ffi.md)
