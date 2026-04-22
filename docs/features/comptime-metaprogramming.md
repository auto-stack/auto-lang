# Comptime Metaprogramming

Auto's compile-time execution engine allows you to run code during compilation, enabling powerful metaprogramming without traditional macros.

## What is Comptime?

The `comptime` keyword marks code that runs at compile time rather than runtime. This allows you to:
- Generate code programmatically
- Validate invariants before the program runs
- Compute constants from complex expressions
- Configure behavior based on compile-time parameters

## Examples

```auto
comptime fn generate_table(size: int) -> [int] {
    var table: [int; size]
    for i in 0..size {
        table[i] = i * i
    }
    return table
}

const squares = generate_table(16)
```

## Compared to Macros

Unlike C macros or Rust's `macro_rules!`, comptime code is:
- Type-safe
- Debuggable
- Written in the same language as runtime code
- Capable of accessing the full compiler API
