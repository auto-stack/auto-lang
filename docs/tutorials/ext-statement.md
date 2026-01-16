# ext Statement User Guide

## Overview

The `ext` statement allows you to add methods to existing types **after** their initial definition. This is similar to Rust's `impl` blocks but adapted for Auto's syntax.

## Syntax

```auto
ext TypeName {
    // Instance methods
    fn method_name() ReturnType {
        // self is available implicitly
        // Use .prop to access self.prop
    }

    // Static methods
    static fn static_method() ReturnType {
        // No self available
        // Called on the type, not instances
    }
}
```

## Instance Methods

Instance methods have implicit access to `self` (the instance value):

```auto
ext int {
    fn double() int {
        self + self  // self is the int value
    }

    fn triple() int {
        self + self + self
    }
}

fn main() {
    let x = 5
    print(x.double())  // Output: 10
    print(x.triple())  // Output: 15
}
```

### Accessing Fields

For user-defined types with fields, use `.prop` shorthand:

```auto
type Point {
    x int
    y int
}

ext Point {
    fn sum() int {
        .x + .y  // .x is shorthand for self.x
    }
}

fn main() {
    let p = Point { x: 3, y: 4 }
    print(p.sum())  // Output: 7
}
```

## Static Methods

Static methods don't have access to `self` and are called on the type itself:

```auto
ext int {
    static fn default() int {
        42
    }

    static fn max() int {
        2147483647
    }
}

fn main() {
    let x = int.default()
    print(x)  // Output: 42

    let m = int.max()
    print(m)  // Output: 2147483647
}
```

## Built-in Type Extension

You can extend any built-in type:

```auto
ext int {
    fn is_even() bool {
        self % 2 == 0
    }
}

ext str {
    fn is_empty() bool {
        .size == 0
    }
}

fn main() {
    let x = 4
    print(x.is_even())  // Output: true

    let s = "hello"
    print(s.is_empty())  // Output: false
}
```

## Multiple ext Blocks

You can have multiple `ext` blocks for the same type:

```auto
ext int {
    fn double() int {
        self + self
    }
}

ext int {
    fn triple() int {
        self + self + self
    }
}

fn main() {
    let x = 5
    print(x.double())  // Output: 10
    print(x.triple())  // Output: 15
}
```

**Note**: If you define the same method twice, the later definition overwrites the earlier one with a warning.

## Method Names

Methods are registered using the format `TypeName::method_name`:

- `int.double()` → registered as `int::double`
- `Point.sum()` → registered as `Point::sum`

This avoids conflicts with methods of the same name on different types.

## C Transpilation

The Auto to C transpiler converts ext methods to regular C functions:

**Auto code**:
```auto
ext int {
    fn double() int {
        self + self
    }
}

fn main() {
    let x = 5
    print(x.double())
}
```

**Generated C code**:
```c
int int_double(int self) {
    return self + self;
}

int main(void) {
    int x = 5;
    printf("%d\n", int_double(x));
    return 0;
}
```

### Key Differences in C

1. **Function naming**: `TypeName_method_name`
2. **Instance methods**: Pass by value for built-in types
3. **Static methods**: No self parameter
4. **User-defined types**: Pass by pointer (e.g., `Point*`)

## Best Practices

1. **Group related methods**: Use one ext block per feature or concern
2. **Use static methods for constructors**: e.g., `int.from_string()`
3. **Keep methods focused**: Each method should do one thing well
4. **Document your methods**: Add comments explaining purpose and usage
5. **Avoid naming conflicts**: Be careful with common method names

## Examples

### Example 1: String Utilities

```auto
ext str {
    fn is_empty() bool {
        .size == 0
    }

    fn has_prefix(prefix str) bool {
        // Implementation depends on str functions
        false  // Placeholder
    }
}

fn main() {
    let s = "hello"
    print(s.is_empty())  // Output: false
}
```

### Example 2: Integer Operations

```auto
ext int {
    fn abs() int {
        if self < 0 {
            -self
        } else {
            self
        }
    }

    fn clamp(min int, max int) int {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

fn main() {
    let x = -5
    print(x.abs())  // Output: 5

    let y = 15
    print(y.clamp(0, 10))  // Output: 10
}
```

### Example 3: Type Conversion

```auto
ext int {
    static fn from_string(s str) int {
        // Implementation would parse string to int
        0  // Placeholder
    }
}

fn main() {
    let x = int.from_string("42")
    print(x)  // Output: 42 (when implemented)
}
```

## Limitations

1. **No generic methods**: Currently cannot define generic ext methods
2. **No method overloading**: Cannot have multiple methods with the same name and different parameters
3. **Runtime registration**: Methods are registered at runtime, not compile-time
4. **Type inference**: For complex expressions, you may need explicit type annotations

## See Also

- [Plan 035: ext Statement Implementation](../docs/plans/035-ext-statement.md) - Full implementation details
- [Type Declaration](./types.md) - How to define new types
- [Method Syntax](./methods.md) - Method syntax in type declarations
