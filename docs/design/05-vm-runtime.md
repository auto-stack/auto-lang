# 05 - VM Runtime

## Status

The VM runtime is substantially implemented in `crates/auto-lang/src/vm/`. The core components are active:

- **OpCode definitions** (`opcode.rs`, 311 lines): Full ABC instruction set with ~120 opcodes covering stack ops, constants, arithmetic, control flow, function calls, objects/arrays, closures, pattern matching, Option/Result types, and type conversions.
- **Execution engine** (`engine.rs`, 3515 lines): Complete dispatch loop with heap management, closures, iterators, generic dispatch, and FFI integration.
- **Bytecode compiler** (`codegen.rs`, 7079 lines): Full AST-to-bytecode compilation with type inference, monomorphization, and template codegen.
- **Virtual memory** (`virt_memory.rs`): VirtualFlash + VirtualRAM implementing the "digital twin" MCU memory model.
- **Task system** (`task.rs`, `scheduler.rs`, `task_system.rs`): Per-task stacks, Tokio-based M:N scheduling, actor-style message passing with mailboxes.

Not yet implemented: AutoLive hot-reload (GOT patching), MicroVM C implementation, Tier-2 JIT compilation, polyglot FFI plugin system.

## Design

### AutoByteCode (ABC) Instruction Set

ABC is the contract between the compiler backend and the VM. It targets 32-bit stack-based execution with variable-length encoding optimized for XIP (execute-in-place) on flash memory.

**Data model**: Each stack slot is 32 bits wide, holding i32, f32, pointer, or bool values. The instruction encoding uses 1-byte opcodes followed by variable-length operands, all little-endian.

**Instruction categories**:

| Range | Category | Key Instructions |
|-------|----------|-----------------|
| 0x00-0x0F | Stack | NOP, POP, POP_N, DUP, SWAP, DROP, RESERVE_STACK |
| 0x10-0x1F | Constants | CONST_I32, CONST_U8, CONST_0, CONST_1, CONST_F32, CONST_F64, CONST_I64, LOAD_STR |
| 0x20-0x2F | Variables + Objects | LOAD_LOCAL, STORE_LOCAL, SET_FIELD, GET_FIELD, CREATE_OBJ, CREATE_ARRAY |
| 0x30-0x4F | Arithmetic + Logic | ADD, SUB, MUL, DIV, MOD, NEG, AND, OR, XOR, NOT, SHL, SHR, MOD_F, MOD_D |
| 0x50-0x5F | Comparison | EQ, NE, LT, GT, LE, GE |
| 0x60-0x6F | Control Flow | JMP, JMP_IF_Z, JMP_IF_NZ, JMP_L (long jump) |
| 0x70-0x7F | Calls + Data | CALL, RET, CALL_NAT, CREATE_RANGE, BUILD_FSTR, NULL_COALESCE, ERROR_PROPAGATE |
| 0xE0-0xFF | Extended | Option/Result ops, type casts, conversions, RET_D (2-slot return) |

**Design decisions**: RET uses callee-cleanup convention (RET takes n_args to clean the stack). All jumps use signed 16-bit relative offsets for position-independent code. CALL_NAT indexes into a native function table (up to 65535 entries).

### Auto Runtime (ART) Cross-Platform Architecture

ART is a "smart shim layer" -- not a libc replacement -- that shields OS differences from Auto code through compile-time polymorphism.

**Four-layer stack**:

1. **Auto Native Core** (Layer 1): Platform-independent pure logic -- String, Slice, Vector, Option, Result, UTF-8, math. Written in Auto, transpiled to C99.
2. **Platform Abstraction Layer** (Layer 2): Unified `art_` prefixed C APIs. Different platforms provide different implementations of `art_fs_open`, `art_alloc`, etc.
3. **Capability Guard** (Layer 3): Static trimming via `caps` configuration. If the target lacks `fs`, the `std.fs` module is gated at compile time.
4. **Backend Implementations** (Layer 4): posix (pthread/socket/fopen), win32 (CreateThread/Winsock/CreateFile), rtos (FreeRTOS/ThreadX), bare (Newlib-stub/HAL).

**Memory management**: Routes allocation requests rather than implementing an allocator. Desktop maps to malloc/mimalloc; RTOS maps to pvPortMalloc; bare metal provides bump-pointer on static arrays.

### AutoVM (BigVM) -- PC Runtime

AutoVM is the reference implementation on Windows/Linux/macOS. It serves as both an interpreter and a "digital twin" of the embedded MicroVM.

**Architecture**: Hub-and-spoke with four core subsystems:

1. **Virtual Hardware**: VirtualFlash (read-only code space) + VirtualRAM (read-write data/stack), modeling MCU memory. Implemented as `Vec<u8>` and `Vec<Word>` with explicit BP/SP registers.
2. **Scheduler**: M:N task scheduling via Tokio. Each Auto task gets its own `AutoTask` struct (independent virtual stack, IP, call frames). Tasks are wrapped in `tokio::spawn` futures.
3. **Native Interface**: FFI gateway with shim registry for calling Rust implementations of standard library functions.
4. **Hot Linker**: Receives incremental patches from the compiler, writes new bytecode to VirtualRAM's Hot Zone, and updates the GOT (Global Offset Table) for function redirection.

**Task structure** (`AutoTask`): Owns a VirtualRAM instance, instruction pointer, base pointer, local count, task status (Ready/Running/Waiting/Terminated), wake time for sleep, closure context, function metadata from FN_PROLOG, result type tracking, error state, and message loop context for actor handlers.

**Scheduler** (`scheduler.rs`): Actor-style message dispatch with `GlobalMeta` (Arc-wrapped read-only metadata shared across tasks). Tasks communicate via Tokio MPSC channels. The daemon loop handles spawn/stop commands and manages task lifecycle.

### Concurrency Model: Task + Message

Auto uses a four-phase concurrency architecture:

**Phase 1 -- Actor Foundation**: Static `task` blocks with entity-private state (no shared locks). Messages via typed enums through implicit mailboxes. `TaskSystem.start()` hands the main thread to the scheduler; all code after it is unreachable.

**Phase 2 -- Async Suspension**: `~T` type (async blueprint), `.await` suspension, `TaskSystem.run(~{...})` for synchronous bridging, `ask(msg).await` / `reply` for bidirectional RPC via implicit oneshot channels. Back-pressure via `.send(msg).await`.

**Phase 3 -- Polymorphic Routing**: Implicit union types from `on` blocks, explicit `MessageContext` parameter (`on(ctx)`), literal matching, type capture, guard clauses. `ctx.reply(payload)` replaces the `reply` keyword.

**Phase 4 -- Micro-concurrency**: `.go` postfix operator dispatches `~T` blueprints to background worker pool. Maps to `tokio::spawn` on PC, `xTaskCreate` on FreeRTOS. Supports `#[single_thread]` for 1:N single-core event loop.

**ISA compatibility**: The same opcodes (OP_SPAWN, OP_RECV, OP_SEND, OP_SLEEP) map to different backends: Tokio on PC, RTOS primitives on MCU. Pure computation opcodes (OP_ADD, etc.) are shared identically.

### MicroVM -- Embedded Runtime

MicroVM targets resource-constrained MCUs (as low as 2KB RAM, 20KB Flash) with a C99 interpreter.

**Memory model**: Code and constants reside in Flash (zero-copy XIP). Stack is statically allocated in RAM. Heap is optional (arena or pool).

**Optimization strategy (three tiers)**:

| Tier | RAM | Strategy | Property Lookup | Fragmentation |
|------|-----|----------|----------------|---------------|
| Tiny | <64KB | Static linked lists | O(N) linear | 0% |
| Standard | 64-256KB | Lists + hash acceleration | Near O(1) | <5% |
| Performance | >256KB | Slab allocator + compact arrays | O(1) | 0% (internal only) |

**GC**: Reference counting as primary strategy (no STW pauses). Optional mark-sweep for cycle collection on Performance-tier systems.

**AutoLive hot-reload**: Compiles changed functions to PIC machine code, injects into RAM Hot Zone via SWD/JTAG, and patches the GOT. Targets sub-second iteration cycles on MCU. Falls back to full AOT for production.

### Streaming/REPL Mode

The VM supports incremental code execution for REPL and network-streamed code:

**Key principle**: Session state persists, code fragments are transient. The VM instance is long-lived; bytecode chunks are compiled, executed, and discarded while preserving the data stack, global variables, and symbol table.

**Statement-level stack balance**: After each statement executes, the stack pointer must equal `BP + locals_count`. Expression results are printed then popped. This guarantees that local variable offsets remain stable across fragments.

**Open frame technique**: For cross-fragment local variable access, the VM treats the session as one "infinitely long function." Chunks are appended to the current CodeObject. A `SUSPEND` instruction at chunk boundaries preserves the stack frame without destroying it.

### Storage Modifiers and Concurrency Primitives

Auto separates storage attributes from data types using a keyword chain: `[visibility] [storage] [atomicity] [volatility] [declaration] name type = value`

- **`const`**: Compile-time constant (symbol table only).
- **`let`**: Runtime immutable binding.
- **`var`**: Runtime mutable variable.
- **`shared`**: Static storage allocation (.data/.bss segments).
- **`atomic`**: Hardware-level atomic access, or `atomic { ... }` blocks with adaptive lock strategy (spinlock for interrupts, mutex for tasks, hardware semaphores for multi-core).
- **`volatile`**: Prevents compiler caching optimizations, used for hardware register mapping.

## Open Questions

- Tier-2 JIT compilation strategy: whether to use `a2r` + `rustc` + `dlopen` or a custom JIT backend.
- Polyglot FFI plugin interface: exact shape of the `Plugin` trait and handle table lifecycle.
- MicroVM/AutoVM shared core extraction into a `no_std` crate.
- Closure environment capture semantics across the `.go` boundary (ownership escape analysis).

## Source Documents

- [raw/abc.md](raw/abc.md) -- ABC instruction set reference
- [raw/art.md](raw/art.md) -- Auto Runtime cross-platform architecture
- [raw/autovm-autolive.md](raw/autovm-autolive.md) -- AutoVM + AutoLive embedded ecosystem
- [raw/autovm-generics.md](raw/autovm-generics.md) -- Generic implementation and optimization plan
- [raw/autovm-streaming.md](raw/autovm-streaming.md) -- Streaming/REPL execution model
- [raw/autovm-task-msg.md](raw/autovm-task-msg.md) -- Task/Msg concurrency rationale
- [raw/autovm-tokio.md](raw/autovm-tokio.md) -- Tokio-based async runtime design
- [raw/auto-vm-bigvm.md](raw/auto-vm-bigvm.md) -- BigVM high-level design
- [raw/auto-vm-mix.md](raw/auto-vm-mix.md) -- AutoVM architecture with AIE integration
- [raw/microvm-atom.md](raw/microvm-atom.md) -- MicroVM data structures and tiered optimization
- [raw/task-msg.md](raw/task-msg.md) -- Four-phase concurrency spec (Phase 1-4)
- [raw/shared.md](raw/shared.md) -- Storage modifiers and concurrency primitives
