# 04 - Memory and Ownership

## Status

**Implemented**: `ParamMode` enum with `View`/`Mut`/`Move` trinity (`ast/fun.rs`), parser support for `view`/`mut`/`move` parameter modifiers, `Hold` expression for path binding (`ast/hold.rs`), storage modifiers (`let`/`var`/`const`/`mut`), `ParamMode::View` as default for function parameters, deprecated `Copy` and `Take` modes.

**Partial**: Path binding via `hold` is parsed but deep integration with the borrow checker is not complete. The `view` keyword doubles as both a memory modifier and a UI concept (AURA views).

**Planned**: Lifetime levels, full borrow checking, static stack analysis for tasks, `shared` type for cross-task ARC, ABI-level copy optimization for small types in a2r.

## Design

### Design Philosophy

AutoLang targets both embedded systems (STM32, ESP32) and high-performance PC applications. Its memory model aims for three properties simultaneously:

1. **Zero-overhead safety**: No garbage collector. Memory errors are caught at compile time.
2. **Transparent costs**: The syntax reveals the runtime cost of every operation.
3. **Implicit where safe**: The compiler handles 80% of ownership decisions automatically through default rules.

### The Trinity: view, mut, move

All memory access is reduced to three access modes. Each is O(1) overhead:

| Mode | Permission | Ownership | Call Site Syntax |
|------|-----------|-----------|-----------------|
| `view` | Read-only | Borrowed | `obj` (default) or `obj.view` |
| `mut` | Read-write | Borrowed | `obj.mut` |
| `move` | Full ownership | Transferred | `obj.move` |

The fourth operation, `clone`, is explicit and carries O(N) cost with parentheses as a visual warning: `obj.clone()`.

**Definition site** (function parameters):

```auto
fn print_user(u User) { ... }         // implicit view (default)
fn update_user(u mut User) { ... }    // explicit mut
fn consume_user(u move User) { ... }  // explicit move
```

**Call site** (postfix accessors):

```auto
let data = User{...}
process(data)           // view (default magic)
update(data.mut)        // mut (warning: data is being modified)
consume(data.move)      // move (data dies after this line)
backup(data.clone())    // clone (expensive, visually marked with ())
```

**Key asymmetry**: At the definition site, the default is `view` (safe, no annotation needed). At the call site, the default is also `view` (just pass the variable). When something more powerful is needed, both sides must agree -- `mut` or `move` at the definition requires `.mut` or `.move` at the call.

### Move Semantics

When a parameter is declared `move`, the function takes absolute ownership:

```auto
fn consume_user(u move User) {
    u.age += 1   // OK: move parameters are fully mutable inside the function
}
```

Inside the function, `move` parameters are implicitly mutable -- no additional `mut` declaration needed. After the call, the original variable is dead and cannot be used.

For resource types (lists, files, `!T` error values), assignment with `=` is prohibited. Transfer must be explicit:

```auto
let result = read_sensor()   // result is !int
let b = result.move          // explicit transfer; result is now dead
```

### Deprecated Keywords

- **`take`**: Replaced by `move`. The word "take" conflicts with collection methods (`take(n)` gets the first n elements). Retained as a deprecated alias for backward compatibility.
- **`copy`**: Removed entirely. Hidden deep copies violate the "transparent costs" principle. For heavy types, use `move` and let the caller decide whether to `.clone()`. For trivial types (int, bool), the transpiler automatically uses register-passing -- no keyword needed.

### Path Binding with `hold`

Deep data modification is a known pain point in ownership-based systems. AutoLang introduces `hold` for temporary path binding:

```auto
hold x.y.z as value {
    value.field = new_value
    // borrow released automatically at end of block
}
```

The `hold` expression (`ast/hold.rs`) records a path expression and a binding name. The compiler:
1. Records the access path (no immediate borrow).
2. On block entry, locks the intermediate structures to prevent conflicting access.
3. Materializes the path into a mutable reference.
4. On block exit, releases the borrow.

Generated code is equivalent to a scoped mutable borrow with compile-time offset calculation -- zero overhead compared to raw pointer access.

### Storage Modifiers

AutoLang uses four storage types, collectively called "storages" (存量):

| Keyword | Name | Mutability | Scope | Use |
|---------|------|-----------|-------|-----|
| `let` | Fixed quantity (定量) | Immutable | Block-scoped | Default for values |
| `mut` | Variable (变量) | Mutable | Block-scoped | When modification is needed |
| `const` | Constant (常量) | Immutable | Global | Compile-time constants |
| `var` | Phantom (幻量) | Mutable | Inferred | Type-inferred mutable binding |

The term "storage" (存量) replaces the contradictory "immutable variable" / "mutable variable" terminology. Each storage type has a distinct name reflecting its nature.

### Lifetime Levels

AutoLang defines a hierarchy of lifetime levels, from permanent to instantaneous:

| Level | Name | Chinese | Scope | Example |
|-------|------|---------|-------|---------|
| 0 | Immortal | 长生不老 | Survives program end | NVM-backed data |
| 1 | Process | 寿与天齐 | Program lifetime | Global variables |
| 2 | Auto (GC/RC) | 六道轮回 | Reference-counted | Heap objects |
| 3 | Task | 浮生若梦 | Task completion | Task-local state |
| 4 | Start/Stop | 缘起缘灭 | Manual stop | Managed resources |
| 5 | Period | 白驹过隙 | One frame | Game loop state |
| 6 | Scope | 朝生暮死 | Block exit | Local variables (default) |
| 7 | Instant | 瞬息蜉蝣 | Single statement | Temporary values |

Global variables default to Process level. Local variables default to Scope level. The `task` keyword assigns `@Task` lifetime to its members.

### Parameter Passing: Semantic View, Implementation Copy

The default parameter mode is semantically `view` (immutable reference), but the transpiler applies automatic optimization:

**For trivial types** (int, float, bool, char, byte): The transpiler generates direct value passing (register copy) instead of a pointer. This is faster on modern hardware while maintaining the same immutability guarantees. The frontend type checker still enforces immutability -- attempting to modify a `view` parameter is a compile error regardless of how it is passed at the machine level.

**For heavy types** (String, Vec, struct): The transpiler generates reference passing (`&T` in Rust, pointer in C).

The optimization is transparent to the programmer. From the user's perspective, all default parameters are `view` -- the implementation detail of copy-vs-reference is an ABI concern.

### Cross-Task Safety

Tasks communicate through ownership transfer (move semantics) by default, ensuring no two tasks can simultaneously write to the same memory. When shared access is required:

- The `shared` type provides atomic reference counting (ARC).
- On MCU, hardware atomic instructions handle preemptive task switching.
- Static analysis validates `shared` object access in interrupt service routines.

### Concurrency: Stack Analysis

For embedded targets with constrained memory, the compiler analyzes the call graph of each task to compute the maximum stack depth. Stack space is reserved at compile time, eliminating runtime stack overflow risk.

## Open Questions

- Whether lifetime annotations (e.g., `@Scope`, `@Task`) should be explicit syntax or purely inferred.
- How `hold` path binding interacts with the enum pattern matching system.
- Whether `shared` should be a keyword or a library type.
- The exact rules for when the compiler can prove "last use" and automatically insert a move.

## Source Documents

- [raw/memory.md](raw/memory.md)
- [raw/new_memory.md](raw/new_memory.md)
- [raw/value-access.md](raw/value-access.md)
- [raw/param-passing-default.md](raw/param-passing-default.md)
- [raw/storages.md](raw/storages.md)
