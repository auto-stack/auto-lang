# May<T> Type - Unified Optional/Result Type

**Status**: ✅ Phase 1b Complete (2025-01-16)
**Design**: [May Type Design Document](../../../docs/language/design/may-type.md)

## Overview

`May<T>` is a unified three-state enum that combines the concepts of `Option<T>` and `Result<T, E>` into a single, elegant type. It represents a computation that can have:

- **Empty** - No value (like `None` or null)
- **Value(T)** - Success with a value (like `Some` or `Ok`)
- **Error** - Failure with an error message (like `Err`)

**Syntactic Sugar**: In AutoLang, `May<T>` is written as `?T` (e.g., `?int`, `?str`)

```auto
fn divide(a int, b int) ?int {
    if b == 0 {
        return May_error("division by zero")
    }
    return May_value(a / b)
}
```

## Why May<T>?

### Problems with Separate Types

**Option<T> and Result<T, E> create complexity:**

1. **Nesting Hell**: `Result<Option<T>, E>` becomes difficult to work with
2. **Type Confusion**: When to use Option vs Result?
3. **Conversion Overhead**: Constantly converting between types
4. **Mental Load**: Two types instead of one

### May<T> Solution

**One type, three states, linear flow:**

```auto
// Clean, linear error propagation
fn get_user(id int) ?str {
    let user = May_value(find_user(id))  // ? str
    let name = May_unwrap(user)           // Early return on error
    return May_value(name.to_upper())
}
```

With the `.?` operator (planned):

```auto
fn get_user(id int) ?str {
    let user = find_user(id).?    // Auto early return
    return user.to_upper().?
}
```

## API Reference

### Creation Functions

#### `May_empty<T>() -> May<T>`

Creates an Empty May (no value, no error).

```auto
let may = May_empty<int>()
assert(May_is_empty(may))
```

**Use Case**: Search found nothing, optional missing data

#### `May_value<T>(value: T) -> May<T>`

Creates a May with a value.

```auto
let may = May_value(42)
assert(May_is_value(may))
assert(May_unwrap(may) == 42)
```

**Use Case**: Successful operation, valid data

#### `May_error<T>(error: str) -> May<T>`

Creates a May with an error.

```auto
let may = May_error<int>("something went wrong")
assert(May_is_error(may))
assert(May_unwrap_error(may) == "something went wrong")
```

**Use Case**: Operation failed, invalid input, system error

### Inspection Functions

#### `May_is_empty<T>(may: May<T>) -> bool`

Returns `true` if the May is in Empty state.

```auto
let may = May_empty<int>()
assert(May_is_empty(may))
```

#### `May_is_value<T>(may: May<T>) -> bool`

Returns `true` if the May has a value.

```auto
let may = May_value(42)
assert(May_is_value(may))
```

#### `May_is_error<T>(may: May<T>) -> bool`

Returns `true` if the May has an error.

```auto
let may = May_error<int>("error")
assert(May_is_error(may))
```

### Unwrapping Functions

#### `May_unwrap<T>(may: May<T>) -> T`

Gets the value. **Unsafe** if May is Empty or Error (will print error to stderr).

```auto
let may = May_value(42)
let value = May_unwrap(may)
assert(value == 42)

// Unsafe on Empty/Error:
let empty = May_empty<int>()
let value = May_unwrap(empty)  // Prints error, returns NULL
```

**Prefer**: `May_unwrap_or` or `May_unwrap_or_null` for safe access

#### `May_unwrap_or<T>(may: May<T>, default: T) -> T`

Gets the value or returns `default` if Empty or Error.

```auto
let may = May_value(42)
assert(May_unwrap_or(may, 100) == 42)

let empty = May_empty<int>()
assert(May_unwrap_or(empty, 100) == 100)

let error = May_error<int>("error")
assert(May_unwrap_or(error, 100) == 100)
```

#### `May_unwrap_or_null<T>(may: May<T>) -> T`

Gets the value or returns `null` if Empty or Error.

```auto
let may = May_value(42)
assert(May_unwrap_or_null(may) == 42)

let empty = May_empty<int>()
assert(May_unwrap_or_null(empty) == null)
```

#### `May_unwrap_error<T>(may: May<T>) -> str`

Gets the error message. **Unsafe** if May is not in Error state.

```auto
let may = May_error<int>("error")
let err = May_unwrap_error(may)
assert(err == "error")
```

#### `May_unwrap_error_or<T>(may: May<T>, default_error: str) -> str`

Gets the error or returns `default_error` if not in Error state.

```auto
let may = May_error<int>("actual error")
assert(May_unwrap_error_or(may, "default") == "actual error")

let may = May_value(42)
assert(May_unwrap_error_or(may, "default") == "default")
```

### Cleanup Function

#### `May_drop<T>(may: May<T>)`

Cleans up May resources (error payload if allocated).

```auto
let may = May_error<int>("error")
May_drop(may)  // Free error message
```

**Note**: In AutoLang, this is typically handled automatically by garbage collection.

## Usage Examples

### Example 1: Division with Error Handling

```auto
fn divide(a int, b int) ?int {
    if b == 0 {
        return May_error("division by zero")
    }
    return May_value(a / b)
}

fn main() {
    let result1 = divide(10, 2)
    if May_is_value(result1) {
        let value = May_unwrap(result1)
        print(value)  // 5
    }

    let result2 = divide(10, 0)
    if May_is_error(result2) {
        let error = May_unwrap_error(result2)
        print(error)  // "division by zero"
    }
}
```

### Example 2: User Lookup with Three States

```auto
fn find_user(id int) ?str {
    if id == 1 {
        return May_value("Alice")
    }
    if id == 2 {
        return May_error("User not found")
    }
    return May_empty()  // ID doesn't exist
}

fn main() {
    // Found
    let user1 = find_user(1)
    if May_is_value(user1) {
        print("Found: " + May_unwrap(user1))
    }

    // Error
    let user2 = find_user(2)
    if May_is_error(user2) {
        print("Error: " + May_unwrap_error(user2))
    }

    // Empty (no user, no error)
    let user3 = find_user(3)
    if May_is_empty(user3) {
        print("No user found")
    }
}
```

### Example 3: Configuration with Defaults

```auto
fn get_config(key str) ?int {
    if key == "timeout" {
        return May_value(30)
    }
    if key == "port" {
        return May_value(8080)
    }
    return May_empty()
}

fn main() {
    let timeout = get_config("timeout")
    let value = May_unwrap_or(timeout, 60)
    print("Timeout: " + value)  // 30

    let port = get_config("unknown")
    let value = May_unwrap_or(port, 8080)
    print("Port: " + value)  // 8080 (default)
}
```

### Example 4: Chained Operations

```auto
fn parse_int(s str) ?int {
    if s == "" {
        return May_empty()
    }

    // Simulate parsing
    let value = str_to_int(s)
    if value == null {
        return May_error("invalid integer")
    }

    return May_value(value)
}

fn double_string(s str) ?int {
    let num = parse_int(s)

    if May_is_error(num) {
        // Forward the error
        return May_error(May_unwrap_error(num))
    }

    if May_is_empty(num) {
        return May_empty()
    }

    let value = May_unwrap(num)
    return May_value(value * 2)
}
```

## State Diagram

```
                    May_empty()
                         ↓
                      ┌─────┴─────┐
                      │   Empty   │
                      └─────┬─────┘
                            │ May_value()
                            ↓
                      ┌─────┴─────┐
                      │   Value   │
                      └─────┬─────┘
                            │ May_error()
                            ↓
                      ┌─────┴─────┐
                      │   Error   │
                      └───────────┘
```

**Key**: States are **immutable**. Once created, a May cannot change state.

## Memory Layout

### C Implementation

```c
typedef enum {
    May_Empty = 0x00,
    May_Value = 0x01,
    May_Error = 0x02
} MayTag;

typedef struct {
    uint8_t tag;
    union {
        void* value;   // Valid when tag = May_Value
        void* error;   // Valid when tag = May_Error
    } data;
} May;
```

**Size**: 16 bytes (1 byte tag + 8 byte union + padding)

### AutoLang Implementation (Temporary)

Currently uses existing Value types:
- **Empty** → `Value::Nil`
- **Value** → The value itself
- **Error** → `Value::Error`

**Future**: Dedicated `Value::May` variant for better performance and type safety.

## Best Practices

### DO ✅

- **Use May_value** for successful operations
- **Use May_error** for failures with context
- **Use May_empty** for missing optional data
- **Check state before unwrapping**: `May_is_value`, `May_is_error`
- **Use safe unwrap**: `May_unwrap_or`, `May_unwrap_or_null`
- **Document error cases** in function docs

### DON'T ❌

- **Don't use May_unwrap** without checking state first
- **Don't ignore error states** in control flow
- **Don't use May_error** for expected missing data (use May_empty)
- **Don't nest May types** (use May<T> directly, not May<May<T>>)

## Migration from Option/Result

### From Option<T>

**Before:**
```auto
let opt = Option_some(42)
if Option_is_some(opt) {
    let value = Option_unwrap(opt)
}
```

**After:**
```auto
let may = May_value(42)
if May_is_value(may) {
    let value = May_unwrap(may)
}
```

### From Result<T, E>

**Before:**
```auto
let res = Result_ok(42)
if Result_is_ok(res) {
    let value = Result_unwrap(res)
} else {
    let err = Result_unwrap_err(res)
}
```

**After:**
```auto
let may = May_value(42)
if May_is_value(may) {
    let value = May_unwrap(may)
} else if May_is_error(may) {
    let err = May_unwrap_error(may)
}
```

## Planned Features

### `.?` Operator (Error Propagation)

**Planned for Phase 2**

```auto
fn get_user(id int) ?str {
    let user = find_user(id).?    // Auto early return on error/empty
    return user.name.to_upper().?
}
```

Desugars to:
```auto
fn get_user(id int) ?str {
    let __temp = find_user(id)
    if May_is_error(__temp) || May_is_empty(__temp) {
        return __temp
    }
    let user = May_unwrap(__temp)

    let __temp2 = user.name.to_upper()
    if May_is_error(__temp2) || May_is_empty(__temp2) {
        return __temp2
    }
    return __temp2
}
```

### `??` Operator (Null Coalescing)

**Planned for Phase 2**

```auto
let config = get_config("timeout") ?? 60
```

Desugars to:
```auto
let __temp = get_config("timeout")
let config = May_unwrap_or(__temp, 60)
```

## Performance

### Memory

- **C**: 16 bytes per May instance
- **AutoLang**: Temporary (uses existing Value types)

### Speed

- **State check**: O(1) (single tag comparison)
- **Unwrap**: O(1) (direct pointer access)
- **No heap allocation** for Empty/Value states
- **Error messages**: Heap-allocated (can be optimized with error codes on MCU)

### Benchmarks

*TODO: Phase 2*

## Testing

### AutoLang Tests

Run: `auto.exe run stdlib/may/test_may.at`

- **20 tests** covering all functionality
- Creation, inspection, unwrapping
- Usage examples (divide, find_user, get_config)
- State transitions

### C Tests

Run: `gcc -o test_may stdlib/may/test_may.c && ./test_may`

- **19 tests** for C implementation
- Memory management verification
- State machine validation

### Rust Tests

Run: `cargo test -p auto-lang libs::may`

- **17 tests** for Rust integration
- Pattern matching on Arg enum
- Three-state logic verification

**All tests passing**: ✅ 56/56 tests

## References

- [May Type Design Document](../../../docs/language/design/may-type.md) (Chinese)
- [Plan 027: Standard Library C Foundation](../../../docs/plans/027-stdlib-c-foundation.md)
- [Option/Result Implementation](../result/) (Deprecated - use May instead)

## License

MIT License - Part of AutoLang Project
