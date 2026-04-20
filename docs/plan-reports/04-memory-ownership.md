# 04 - Memory and Ownership

## Overview

AutoLang implements an ownership-based memory management system that provides zero-cost safety without garbage collection. The system is built on a three-phase hybrid approach: move semantics for linear types, owned string types, and a full borrow checker with `view`/`mut`/`take` keywords, lifetime tracking, and smart parameter passing. All four plans in this domain are completed, spanning from the foundational linear type system through VM-level reference instructions. Together they establish AutoLang's core differentiator: Rust-level memory safety with a simpler user-facing model where all parameters default to immutable borrows and the compiler automatically selects the optimal calling convention.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 024 | Ownership First Implementation | Completed | Three-phase ownership system: move semantics, owned str, borrow checker with view/mut/take |
| 034 | Borrow Checker Redesign | Completed | Three borrow types (view, mut, take) with AST, parser, evaluator, and C transpiler support |
| 038 | Fix VM Borrowing for OOP Methods | Completed | RefCell-based interior mutability for VM objects and VM method call expressions |
| 088 | Parameter Passing Modes | Completed | ABO-01 strategy: semantic view with automatic copy optimization for small types, reference passing for large types |

## Status

**Implemented**: All four plans are complete. The ownership module (`crates/auto-lang/src/ownership/`) contains the linear type system, borrow checker, and lifetime tracking. The borrow checker integrates with the evaluator and C transpiler. VM method calls use dot syntax (`"hello".split(" ")`) through automatic VM function lookup. Smart parameter compilation generates different bytecode depending on type size and passing mode.

**Partial**: The ParamChecker from Plan 088 Phase 6 is implemented as a standalone module (`typeck/param_check.rs`, ~130 lines) but is not yet wired into the compilation pipeline, so view-parameter immutability is not enforced at compile time. Non-Lexical Lifetimes (NLL) and automatic value cleanup on scope exit were deferred from Plan 024 Phase 1.

**Planned**: Integration of ParamChecker into the compilation pipeline, NLL for more precise borrow analysis, lifetime tracking at VM runtime for borrowed values, and `take` mode move semantics at the VM level.

## Design

### Move Semantics and Linear Types

The foundation of AutoLang's memory system is a linear type system where variables own their data and implicit cloning is eliminated. Before Plan 024, every assignment in the evaluator cloned values through `Rc<RefCell<ValueData>>`, meaning no ownership semantics existed at all.

The linear type infrastructure lives in `crates/auto-lang/src/ownership/linear.rs` and introduces two key types:

- `MoveState` -- an enum tracking whether a value is `Available` or `Moved`.
- `MoveTracker<T>` -- a wrapper that guards access with move-state assertions, panicking on use-after-move.

The `Linear` trait marks types requiring explicit ownership management. A `LastUseAnalyzer` skeleton exists in `ownership/cfa.rs` for control-flow analysis to detect the final use of a value and insert automatic cleanup.

At the evaluator level, `eval_store()` was enhanced to clear moved status on reassignment and track move state through `Universe` (methods `mark_moved`, `is_moved`, `clear_moved`). Use-after-move detection occurs in `lookup_val_recurse()`. Scope variable management uses `moved_vars: HashSet` to track which locals have been consumed.

```auto
let s = str_new("hello", 5)   // s owns the string
let t = s                      // move: s is no longer valid
use(t)                         // last use: automatic cleanup
```

The naming conventions established in Plan 024 follow a clear hierarchy: lowercase for built-in primitives (`int`, `bool`, `str`, `cstr`), lowercase with suffix for complex built-ins (`str_slice`), PascalCase for user-defined types (`Point`), and lowercase brackets for generics (`slice[T]`). Functions follow a `type_action()` pattern: `str_new()`, `str_len()`, `str_append()`.

### Borrow Checker: view, mut, take

The borrow checker (Plans 024 and 034) implements three borrowing modes, each mapping to a familiar Rust concept but with AutoLang's own keyword syntax:

| AutoLang | Rust Equivalent | Semantics |
|----------|----------------|-----------|
| `view x` | `&x` | Immutable borrow; multiple views can coexist |
| `mut x`  | `&mut x` | Mutable borrow; exclusive access required |
| `take x` | `x` (move) | Ownership transfer; original becomes invalid |

The core data structures live in `crates/auto-lang/src/ownership/borrow.rs` (~794 lines). The `BorrowKind` enum defines the three modes. The `Borrow` struct tracks each active borrow with its kind, lifetime, source expression, and a `Target` for precise conflict detection:

```rust
pub enum Target {
    Variable(String),           // x
    Path(Box<Target>, String),  // obj.field
    Index(Box<Target>),         // arr[index]
    Unknown,                    // temporary values
}
```

The `Target` system replaced an earlier discriminant-based comparison. `Target::from_expr()` resolves expressions to canonical targets, supporting simple variables, unwrapped borrow expressions, dot paths, nested paths, and index operations. Conflict detection uses `same_target()` on resolved targets rather than pointer equality.

The conflict rules are:

- `take` conflicts with all existing borrows (move semantics).
- Two `mut` borrows on the same target conflict.
- A `mut` borrow conflicts with any existing `view` borrow on the same target.
- Two `view` borrows on the same target coexist freely.

```auto
let s = "hello"
let v1 = view s              // OK
let v2 = view s              // OK: multiple views coexist
let m = mut s                // Error: mut conflicts with existing view
```

The AST was extended with three expression variants (`View(Box<Expr>)`, `Mut(Box<Expr>)`, `Take(Box<Expr>)`) plus a `Hold` expression for temporary path binding. The lexer added `view`, `take`, and `hold` keywords. The parser handles these as prefix expressions in the Pratt parser, and the C transpiler generates proper pointer references (`&x` for view/mut).

### Lifetime Tracking and Error Reporting

The lifetime system (`ownership/lifetime.rs`, ~200 lines) provides compile-time lifetime management through two structures:

- `Lifetime` -- a newtype over `u32` with a `STATIC` constant and operations like `outlives()` and `intersect()`.
- `LifetimeContext` -- manages a counter for fresh lifetime generation and a map from expression IDs to lifetimes.

Plan 024 added `LifetimeRegion` structs that track start and end points of lifetimes. The `overlaps()` method enables geometric overlap detection between lifetime regions. The `BorrowChecker` integrates `LifetimeContext` for region-aware conflict detection, using a conservative strategy when lifetimes might overlap.

Error reporting uses the miette framework with the diagnostic code `auto_borrow_E0001`. Error messages display the target name, borrow kind, and lifetime information. Span integration maps borrow errors back to source locations.

The `hold` expression enables temporary path binding with borrow checking:

```auto
hold path s.data as bytes {
    bytes[0] = 'H'           // modify through mut borrow
}
// s still valid here (borrows ended)
```

The parser handles `hold` as a special expression with path, `as` keyword, binding name, and body. The evaluator creates a scoped binding, runs borrow checking, and cleans up on scope exit.

### VM Interior Mutability for OOP

Plan 038 solved a practical problem: the VM's `Universe` stores objects in a `HashMap<usize, Box<dyn Any>>` field called `vmrefs`. When VM methods (e.g., `HashMap.insert()`) tried to mutate stored data, they could not obtain a mutable reference through the `RefCell<Universe>` chain. The `get_vmref()` method returned `Ref<'_, Box<dyn Any>>`, which created an immutable borrow blocking further mutation.

The chosen solution was interior mutability: wrapping `vmrefs` values in `RefCell` so that individual data structures can be mutably borrowed independently of the outer `Universe` borrow. This is safe because Rust's runtime borrow checking catches overlapping borrows, and the overhead is negligible for the VM's usage patterns. Three alternative approaches were considered and rejected: clone-modify-store (too expensive), raw pointers (unsafe), and a full architecture change (too invasive).

Plan 038 also delivered VM method call expressions, enabling dot-syntax calls on built-in types. When the evaluator encounters `"hello world".split(" ")`, it first tries the regular method table. If that fails, it constructs a VM function name using the `{type}_{method}` convention (e.g., `str_split`), looks it up in the builtin registry, and calls it with `self` prepended as the first argument. This was implemented in approximately 29 lines in `eval.rs` (lines 1681-1709) with no parser changes needed.

```auto
// Before: global function syntax
let words = str_split("hello world", " ")

// After: method call syntax
let words = "hello world".split(" ")

// Method chaining
let first = "hello world".split(" ")[0]
```

### Parameter Passing: ABO-01 Strategy

Plan 088 is the most architecturally complex plan in this domain. It implements the ABO-01 strategy ("Semantic View, Implementation Copy"), which gives users a simple mental model -- all parameters are immutable borrows by default -- while the compiler automatically selects the optimal machine-level calling convention.

The strategy classifies types into two categories:

- **Small types** (value-passed): `int`, `float`, `bool`, `char`, `byte`, C-style enums. These fit in a register and are cheaper to copy than to indirect through a pointer.
- **Large types** (reference-passed): `string`, `vector`, `map`, structs, closures. Copying these is expensive, so the compiler passes a reference instead.

The implementation spans seven phases across multiple files:

**Phase 1-2 (Type system and AST)**: Added `Type::is_optimized_by_value()` to `ast/types.rs` and the `ParamMode` enum (`Copy`, `View`, `Mut`, `Take`) to `ast/fun.rs`. The `Param` struct gained a `mode` field defaulting to `View`. All 23 existing `Param` construction sites were updated.

**Phase 3 (Parser)**: Modified `fn_params()` in `parser.rs` to accept optional mode keywords before parameter names. The `copy` token was added to `TokenKind`. The parser supports mixed modes in a single function signature:

```auto
fn process(mut self Point, copy x int, view y float) void
```

**Phase 4 (Codegen)**: This is the core of Plan 088. The codegen module in `vm/codegen.rs` gained approximately 250 lines implementing smart parameter compilation. Key additions:

- `ParamInfo` struct storing parameter type and mode.
- `fn_params: HashMap<String, Vec<ParamInfo>>` tracking metadata per function.
- `compile_call_arg()` method that selects `LOAD_LOC` (value) or `LOAD_REF`/`LOAD_MUT_REF` (reference) based on the type-size and mode matrix.
- A `FN_PROLOG` instruction (opcode `0xB8`) that emits `n_args` and `n_locals` at function entry, enabling the VM to distinguish parameters from locals in the stack frame.
- Fix for jump-over index corruption when `FN_PROLOG` is inserted between function bodies (tracked via `jump_placeholders: Vec<usize>`).

The stack frame layout uses a `bp`-relative encoding where parameters live at negative offsets from the base pointer and locals at positive offsets. The encoding uses `0x80 + index` for parameters and `0x00 + index` for locals, decoded at runtime using `current_fn_n_args` from `FN_PROLOG`.

**Phase 5 (VM execution)**: Added four reference opcodes to the VM engine in `vm/engine.rs`:

| Opcode | Code | Purpose |
|--------|------|---------|
| `LOAD_REF` | `0xB4` | Load immutable reference (var_index on stack) |
| `STORE_REF` | `0xB5` | Store through immutable reference |
| `LOAD_MUT_REF` | `0xB6` | Load mutable reference |
| `STORE_MUT_REF` | `0xB7` | Store through mutable reference |

Reference types `VmRef` and `VmMutRef` are defined in `vm/refs.rs`. They store a `var_index: u32` pointing back to the caller's stack frame. The design avoids extending the `Value` enum by representing references as `i32` values on the stack, maintaining compatibility with the existing stack-based VM architecture.

**Phase 6 (Type checker)**: Created `typeck/param_check.rs` (~130 lines) implementing `ParamChecker`, which walks the AST of each function and flags assignments to `view` parameters. The error type `CannotModifyViewParam` (code `auto_type_E0204`) uses miette diagnostics. The checker handles `Store`, `For`, `Block`, `Return`, and `Expr` statement types. It is not yet integrated into the compilation pipeline.

**Phase 7 (Integration testing)**: 15 test files in `test/param_passing/` cover default view mode, small-object optimization, large-object references, mut parameter modification, mixed modes, method parameters, nested calls, and stress scenarios.

A critical bug was found and fixed during Phase 4: when `RESERVE_STACK` (2 bytes) was inserted at a function entry point, all relocation offsets at or beyond that point needed adjustment. Without the fix, the linker would write call targets to the wrong byte positions, corrupting `LOAD_MUT_REF` operands and `CALL` opcodes. The fix adds a loop over `self.relocs` shifting any offset `>= entry_point` by the insertion size.

## Open Questions

- **ParamChecker integration**: The `ParamChecker` module exists but is not wired into the compilation pipeline. Deciding how aggressively to enforce view-parameter immutability (error vs. warning, which compilation stage) is unresolved.
- **Non-Lexical Lifetimes (NLL)**: The current borrow checker uses lexical scope boundaries for lifetime regions. NLL would allow more precise analysis where borrows end at their last use rather than at scope exit, reducing false-positive borrow conflicts.
- **Automatic value cleanup**: Deferred from Plan 024 Phase 1. The evaluator does not yet drop values when they go out of scope. Linear types rely on Rust's RAII guarantees, but AutoLang-level cleanup (calling `drop` methods, freeing VM resources) is not implemented.
- **Take mode at VM level**: The `take` keyword is parsed and type-checked but has the same runtime behavior as `view` in the VM. Full move semantics would require marking the source variable as invalid after the move, which needs runtime tracking.
- **VmRef vs VmMutRef distinction**: Currently both reference types store a `var_index` and the VM engine treats them identically for load/store operations. Enforcing the immutability of `VmRef` at the VM level (preventing stores through immutable references) is a future task.

## Source Plans

- docs/plans/024-ownership-first-implementation.md
- docs/plans/034-borrow-checker-redesign.md
- docs/plans/038-fix-vm-borrowing.md
- docs/plans/088-param-passing-modes.md
