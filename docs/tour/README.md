# Auto Language Tour

An interactive tour of the Auto programming language, inspired by [Go Tour](https://tour.golang.org) and [Rust by Example](https://doc.rust-lang.org/rust-by-example/).

## Chapters

### [Chapter 1: Hello Auto](ch01-hello/) — Getting Started
- [Hello World](ch01-hello/01_hello.at) — Your first Auto program
- [Variables](ch01-hello/02_variables.at) — `let`, `var`, type inference
- [Strings](ch01-hello/03_strings.at) — String literals, f-strings, concatenation
- [Comments](ch01-hello/04_comments.at) — Single-line and doc comments
- [Basic Types](ch01-hello/05_basic_types.at) — int, float, bool, str

### [Chapter 2: Types](ch02-types/) — Structs & Enums
- [Struct Definition](ch02-types/01_struct.at) — `type` with named fields
- [Enum Definition](ch02-types/02_enum.at) — `enum` with variants
- [Field Access](ch02-types/03_field_access.at) — Dot notation, nested types
- [Type Methods](ch02-types/04_type_methods.at) — Inline `fn` and `mut fn`
- [Constants](ch02-types/05_constants.at) — `const` declarations

### [Chapter 3: Functions](ch03-functions/) — Defining Behavior
- [Function Basics](ch03-functions/01_fn_basics.at) — Parameters, return types
- [Multiple Returns](ch03-functions/02_multi_return.at) — Returning tuples and values
- [Closures](ch03-functions/03_closures.at) — Lambda expressions
- [Function Values](ch03-functions/04_fn_values.at) — Functions as first-class values

### [Chapter 4: Control Flow](ch04-control/) — Making Decisions
- [If / Else](ch04-control/01_if_else.at) — Conditional branching
- [For Loop](ch04-control/02_for_loop.at) — Range and collection iteration
- [Conditional Loop](ch04-control/03_for_cond.at) — `for condition { }` (while-like)
- [Loop & Break](ch04-control/04_loop_break.at) — Infinite loops
- [If Expression](ch04-control/05_if_expr.at) — Conditional values

### [Chapter 5: Pattern Matching](ch05-patterns/) — Deconstructing Data
- [Basic Match](ch05-patterns/01_is_basics.at) — `is` with values and patterns
- [Enum Matching](ch05-patterns/02_enum_match.at) — Destructuring enum variants
- [Option Matching](ch05-patterns/03_option.at) — Some/None handling
- [Result Matching](ch05-patterns/04_result.at) — Ok/Err handling

### [Chapter 6: Error Handling](ch06-errors/) — Robust Code
- [Error Function](ch06-errors/01_error_fn.at) — `!` error-returning functions
- [Error Propagation](ch06-errors/02_propagate.at) — `.?` operator
- [Custom Errors](ch06-errors/03_custom_error.at) — Defining error types
- [Coalescing](ch06-errors/04_coalesce.at) — `??` default values

---

## Running Examples

```bash
# Run a single example
auto docs/tour/ch01-hello/01_hello.at

# Run all examples
for f in docs/tour/**/*.at; do
    echo "=== $f ==="
    auto "$f" || echo "FAIL: $f"
done
```
