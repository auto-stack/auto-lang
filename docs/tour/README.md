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

### [Chapter 7: Collections](ch07-collections/) — Data Structures
- [Arrays](ch07-collections/01_arrays.at) — Static arrays and slices
- [Lists](ch07-collections/02_lists.at) — Dynamic `List<T>`
- [Maps](ch07-collections/03_maps.at) — Key-value `Map<K,V>`
- [Iteration](ch07-collections/04_iteration.at) — Loops and indexes
- [Object Literals](ch07-collections/05_object_literal.at) — Inline `{key: value}`

### [Chapter 8: Methods & Extensions](ch08-methods/) — Type Behavior
- [Inline Methods](ch08-methods/01_inline_methods.at) — `fn`, `mut fn`, `static fn`
- [Ext](ch08-methods/02_ext.at) — Extending existing types
- [Builder Pattern](ch08-methods/03_builder.at) — Method chaining
- [Operator Overload](ch08-methods/04_operator_overload.at) — Custom operators
- [Spec Implementation](ch08-methods/05_spec_impl.at) — Interfaces and traits

### [Chapter 9: Generics](ch09-generics/) — Abstract Types
- [Generic Functions](ch09-generics/01_generic_fn.at) — `<T>` type parameters
- [Generic Types](ch09-generics/02_generic_type.at) — Parameterized structs
- [Spec with Generics](ch09-generics/03_spec_definition.at) — Constrained parameters
- [Generic Collection](ch09-generics/04_generic_collection.at) — Reusable containers
- [Multiple Specs](ch09-generics/05_multiple_constraints.at) — Combining constraints

### [Chapter 10: Modules](ch10-modules/) — Code Organization
- [Use Import](ch10-modules/01_use_import.at) — `use` statements
- [Pub Visibility](ch10-modules/02_pub_visibility.at) — `pub` keyword
- [Multi-File Module](ch10-modules/03_multi_file.at) — Project structure
- [C Import](ch10-modules/04_c_import.at) — C header imports
- [Annotations](ch10-modules/05_annotations.at) — `#[...]` attributes

### [Chapter 11: Async](ch11-async/) — Asynchronous Programming
- [Async Functions](ch11-async/01_async_fn.at) — `~T` return type + `.await`
- [Concurrent Tasks](ch11-async/02_concurrent.at) — Running tasks in parallel
- [Timeout](ch11-async/03_timeout.at) — Limiting async duration
- [Channel](ch11-async/04_channel.at) — Message passing
- [Actor Pattern](ch11-async/05_actor.at) — Encapsulated state
- [Async Main](ch11-async/06_async_main.at) — Async entry point

### [Chapter 12: Interop](ch12-interop/) — External Integration
- [Dependencies](ch12-interop/01_dep.at) — `dep` declarations
- [Rust Import](ch12-interop/02_rust_import.at) — `use.rust` for Rust crates
- [String Types](ch12-interop/03_string_types.at) — Raw, byte, f-strings
- [FFI Bridge](ch12-interop/04_ffi_bridge.at) — C/Rust FFI
- [Process](ch12-interop/05_process.at) — Running external commands
- [Byte Encoding](ch12-interop/06_byte_encoding.at) — Hex, Base64, hashing

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
