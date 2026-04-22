# Memory Safety

Auto combines Rust-inspired ownership with smart casts and flow typing for memory-safe code that remains ergonomic.

## Ownership System

Every value in Auto has a clear owner. When the owner goes out of scope, the value is automatically cleaned up. Ownership can be:
- **Transferred** — moving a value to a new owner
- **Borrowed** — lending a reference without transferring ownership
- **Shared** — multiple read-only references

## Smart Casts

Auto's type system tracks the state of values through control flow. After a type check or condition, the type is automatically narrowed:

```auto
if x is String {
    // x is automatically cast to String here
    x.len()
}
```

## Flow Typing

The compiler tracks which variables are initialized, moved, or modified through each code path, preventing use-after-move and uninitialized access bugs at compile time.
