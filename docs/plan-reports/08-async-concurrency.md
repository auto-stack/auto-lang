# 08 - Async and Concurrency

## Overview

AutoLang implements a complete Actor-based concurrency stack spanning eight implemented plans (121-128). The system introduces Task and Msg primitives as first-class language constructs, an async Future/Await system with the `~T` type syntax, polymorphic message routing with implicit union types and pattern matching, micro-concurrency via the `.go` suffix operator, and a scheduler-driven message dispatch loop built on Tokio's work-stealing runtime. The architecture spans the full compiler pipeline -- from lexer tokens through AST nodes, bytecode compilation, and runtime execution -- and supports both the AutoVM interpreter and the a2rs (Auto-to-Rust) transpiler backend.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 121 | AutoLang Task/Msg Foundation | Complete | Lexer/AST/Parser for task definitions, spawn/send FFI, TaskRegistry, lifecycle hooks |
| 122 | Value Access and Assignment Refactoring | Complete | `.move` accessor replacing deprecated `.take` with deprecation warnings |
| 123 | TypeStore Unification | Complete | Consolidate all type registries into TypeStore as single source of truth |
| 124 | Async Future/Await System | Complete | `~T` syntax, `.await`, CREATE_FUTURE/AWAIT_FUTURE opcodes, backpressure, ask/reply RPC |
| 125 | Phase 3 - Polymorphic Routing | Complete | Implicit union types, MessageContext runtime, pattern matcher integration |
| 126 | Phase 4 - Micro-Concurrency Engine | Complete | `.go` suffix operator, implicit worker pool, ownership-safe capture semantics |
| 127 | AutoVM TaskSystem Execution | Partial | Bytecode compilation for Task/Msg systems, on-block compilation, ctx.reply(); ask/reply sync deferred |
| 128 | Scheduler Message Dispatch Loop | Complete | Zero-shared-mutable-state scheduler with Tokio async, Arc + mpsc channels |
| 195 | HTTP Client + auto.http Unification | Planned | Upgrade to reqwest, unify http_stream, add async HTTP (Phase 3.2 blocked by Plan 196) |

## Status

**Implemented**: Plans 121-126, 128 (fully complete), Plan 127 (Phases 1-3, 5 complete).

**Partial**: Plan 127 Phase 4 (Ask/Reply synchronization) is deferred pending an async/sync bridge. The `.send()` fire-and-forget and `.go` patterns provide sufficient concurrency for current use cases, so blocking `ask().await` is postponed.

**Planned**: Plan 195 (HTTP client + auto.http unification); performance optimization of the scheduler under high task counts; pub visibility for task message types; wildcard re-exports; a2c (C transpiler) backend for `.go` using state machines and RTOS primitives.

## Design

### Actor Model: Tasks and Messages (Plans 121, 127)

AutoLang's concurrency model is built on the Actor pattern. A `task` definition declares an independent concurrent entity with private mutable state, lifecycle hooks (`start()` and `stop()`), and an `on` block for message handling. Tasks communicate exclusively through typed messages sent over channels -- there is no shared mutable state between tasks.

A task is defined with the `task` keyword and can be either multi-instance (spawned multiple times) or single-instance (annotated with `#[single]`). Multi-instance tasks are created via `TaskType.spawn()`, which returns a `Handle<TaskType>` value that can be copied, compared, and passed between tasks. Single-instance tasks are addressed directly by name (e.g., `Logger.send(Log("msg"))`), and the compiler rejects attempts to `spawn()` them.

```auto
task CounterTask {
    count mut = 0

    fn start() ! { self.count = 0 }
    fn stop() ! { print("final count: ${self.count}") }

    on {
        Add(val) => { self.count += val }
        Reset => { self.count = 0 }
        Print => { print("Count: ${self.count}") }
        else => { print("Unknown message") }
    }
}
```

The lifecycle model is orchestrated by `TaskSystem.start()`, which blocks the main thread, invokes all task `start()` hooks in spawn order, enters the message dispatch loop, waits for a shutdown signal (Ctrl+C or `TaskSystem.stop()`), and then calls `stop()` hooks in reverse spawn order (LIFO). Any code after `TaskSystem.start()` in `main()` is unreachable.

Internally, each task instance holds a private `HashMap<String, Value>` for state, a Tokio `mpsc::Receiver` for its mailbox, and a `mpsc::Sender` used to create handles. The default mailbox capacity is 64 messages, and the strict policy returns a `MailboxFull` error when the capacity is exceeded.

### Async Future/Await System (Plan 124)

Plan 124 introduces the async layer on top of the Task/Msg foundation. The `~T` syntax is syntactic sugar for `Future<T>`, representing a value that will be produced asynchronously. Async blocks are written as `~{ ... }`, and the `.await` suffix operator suspends the current execution context until a future resolves.

```auto
let future = ~{
    let x = 1 + 2
    return x
}
let result = future.await  // result: int
```

The `~` tilde token is parsed in the type position with higher precedence than union types (`|`), so `~int` becomes `Future<int>`, `~User` becomes `Future<User>`, and `~List<int>` becomes `Future<List<int>>`. The `.await` operator is only valid inside async contexts (`~{}` blocks or `on` blocks), and the compiler emits a semantic error if it appears in a synchronous function.

The runtime represents futures as `Value::Future(Arc<FutureData>)`, where `FutureData` holds the future state (Pending/Ready/Failed), the result value, and a list of wakers for suspension. The VM implements `CREATE_FUTURE`, `AWAIT_FUTURE`, and `POLL_FUTURE` opcodes to manage the future lifecycle.

A key integration point is `TaskSystem.run()`, which provides a synchronous bridge: it creates a single-threaded Tokio runtime and calls `block_on` to execute an async block from within a synchronous `main()` function. This allows programs that do not use the actor scheduler to still benefit from async I/O.

For backpressure, `send(msg).await` suspends the sender when the receiver's mailbox is full (in contrast to `send(msg).?` which immediately returns an error). This is implemented via `TaskHandle.send_await()` (FFI NATIVE_TASK_SEND_AWAIT = 2307), which maps to `tx.send(msg).await` in the Rust transpiler.

### Ask/Reply RPC (Plan 124, Phase 2.3)

The ask/reply pattern enables request-response communication between tasks. When a caller invokes `TaskHandle.ask(msg).await`, the compiler automatically injects a `FutureSender<T>` field into the message and returns a `Future<T>` to the caller. Inside the handler, the `reply expr` statement sends the result back through the implicit oneshot channel.

The compiler performs this injection during codegen. For example, `DBManager.ask(QueryUser(1001)).await` is expanded to create a oneshot channel, wrap the sender into the message, and await the receiver. Similarly, `reply user_info` inside a handler is expanded to `reply_tx.send(user_info)`. The reply type `T` is inferred by analyzing the `reply` expressions in the `on` block, enabling full type inference for `ask` return types without explicit annotations.

### Polymorphic Routing and Implicit Unions (Plan 125)

Plan 125 advances the message handling model beyond simple enum-based dispatch. It introduces three pattern types in `on` blocks: literal matching (`"ping" => { ... }`), type binding (`msg string => { ... }`), and enum variant matching with bindings (`Add(val) => { ... }`). Literal patterns match exact values (strings, integers, booleans), while type-binding patterns match any value of a given type and bind it to a local variable.

The compiler automatically extracts all pattern types from an `on` block and generates an implicit union type (the "envelope"). For instance, an `on` block matching `"ping"`, `msg string`, and `amount int` produces an implicit envelope equivalent to an enum with three variants. This eliminates the need for developers to manually define message enum types.

The `on(ctx)` syntax introduces an explicit message context parameter. The `ctx` object provides `ctx.reply(value)` for responding to `ask` calls, `ctx.can_reply()` to check whether a reply channel exists, and metadata fields like `sender_id` and `trace_id`. The context is scoped to a single handler invocation and is automatically invalidated when the handler completes.

The `PatternMatcher` module (`vm/pattern_matcher.rs`) handles runtime dispatch. It supports literal matching (string/int/bool/char), type-binding matching, simple variant matching, and variant-with-bindings matching. Guard expressions (`if amount > 10000`) can further refine pattern arms, and the matcher evaluates them in the binding environment established by the pattern.

### Micro-Concurrency with `.go` (Plan 126)

The `.go` suffix operator provides fire-and-forget concurrency. It applies to any expression of type `~T` (Future) and dispatches that future to a background worker pool without blocking the current task. The expression returns `void`.

```auto
~{
    let result = heavy_compute().await.?
    ctx.reply(result)
}.go  // Returns immediately, runs in background
```

The `.go` operator is symmetric with `.await`: `.await` suspends in time (blocking the current task), while `.go` transfers execution to space (the current task continues, work happens elsewhere). Capture semantics enforce ownership safety: Copy types (int, bool, float) are automatically copied into the background task, while non-Copy types require explicit `.move` (formerly `.take`) to transfer ownership. Using a non-Copy variable without `.move` in a `.go` block produces a compile-time error.

The implementation uses the `SPAWN_GO` opcode (0x89) in the VM. The a2rs transpiler maps `expr.go` to `tokio::spawn(async move { let _ = expr.await; })`. The default scheduling mode is M:N work-stealing (Tokio multi-thread runtime), with an optional 1:N single-thread mode (`#[single_thread]`) for embedded targets.

### Value Ownership and the `.move` Accessor (Plan 122)

Plan 122 refactors the ownership model to use the "Trinity of Resources": `view` (immutable borrow, O(1)), `mut` (mutable borrow, O(1)), and `move` (ownership transfer, O(1)). The `.take` accessor is deprecated in favor of `.move`, and the `.copy` mode is removed entirely -- deep copies now require an explicit `.clone()` method call.

The `Expr::Move` AST node and `Op::DotMove` opcode were added, the lexer recognizes `.move` as a keyword, and the parser handles it as a postfix operator. Deprecation warnings are emitted when `.take` is used. Both the C and Rust transpilers were updated to handle the new `Expr::Move` node. In the Rust backend, `.view` maps to `&T`, `.mut` to `&mut T`, and `.move` to pass-by-value (Rust's default move semantics). In the C backend, `.view` maps to `const T*`, `.mut` to `T* const`, and `.move` to pass by value.

### TypeStore Unification (Plan 123)

Prior to Plan 123, type information was scattered across multiple registries: `TypeStore` in `types.rs`, `TypeRegistry` in `type_registry.rs`, `infer/registry.rs`, and `Database.type_info_store`. Plan 123 consolidated all of these into `TypeStore` as the single source of truth.

The key change was adding `enum_decls: HashMap<AutoStr, Rc<EnumDecl>>` to `TypeStore`, enabling the codegen layer to look up enum variant values directly (e.g., `Color.Red` compiles to `PUSH_INT` with the variant's integer value). The parser was updated to register enum declarations in `TypeStore` during parsing. A unified `is_type()` method checks across type declarations, enum declarations, and spec declarations. The old `type_registry.rs` and `infer/registry.rs` modules received deprecation notices and will be removed in a future version.

### Bytecode Compilation and Task Execution (Plan 127)

Plan 127 bridges the gap between the AST-level task definitions and the VM's bytecode execution engine. The codegen module compiles each `on` block handler to executable bytecode, records handler metadata (pattern index, bytecode offset, context flag) in a `TaskHandlerTable`, and serializes patterns for runtime matching.

New opcodes were introduced: `TASK_LOOP` (0x8A) enters the message processing loop, `HANDLE_MSG` (0x8B) dispatches a message to the matched handler, and `REPLY` (0x8C) sends a reply via the message context. The `ctx.reply()` FFI shim (NATIVE_CTX_REPLY) was wired into the stdlib FFI layer.

The `TaskHandlerTable` stores per-task handler metadata: start/stop hook offsets, else-handler offset, and a vector of `(pattern_idx, body_offset, has_context)` entries. Patterns are serialized to `Vec<u8>` for compact storage in the bytecode. The compilation pipeline produces a `CompiledPackage` (the "ROM cartridge") containing bytecode, string pool, exports, and task definitions, which is then frozen into `GlobalMeta` for the scheduler.

### Scheduler and Message Dispatch Loop (Plan 128)

The scheduler implements a zero-shared-mutable-state architecture. `GlobalMeta`, wrapped in `Arc<GlobalMeta>`, holds read-only bytecode, the string pool, the native interface, and handler tables. Each `TaskContext` is fully owned by its task -- it holds the mailbox receiver, task RAM, executor state, and a reference to `GlobalMeta`. Tasks communicate exclusively through Tokio `mpsc` channels.

The system daemon runs a loop on a privileged `SystemCommand` channel, handling `Spawn` (dynamic task creation) and `Stop` (graceful shutdown) commands. Each task runs in its own `tokio::spawn` coroutine, leveraging Tokio's work-stealing scheduler for automatic load balancing across CPU cores.

Handler execution uses an async model with cooperative yielding. The `execute_handler_fully()` function runs in a loop, processing opcodes until it hits `RET` or `HALT`. Every 10,000 operations, it calls `tokio::task::yield_now()` to prevent CPU starvation from tight loops. Critically, `yield_now()` preserves `task.ip` state, so execution resumes correctly after the yield. The async design also enables `AWAIT_EXT` support for true suspension on async FFI operations.

The scheduler supports dynamic task spawning via `SystemCommand::Spawn`, with a global `DYNAMIC_TASK_ID` counter for generating unique instance IDs. Mailbox receivers are connected through a store/take pattern in `TaskRegistry`: `shim_task_spawn()` stores receivers, and `spawn_initial_tasks()` retrieves them when the scheduler starts.

## Open Questions

- **Ask/Reply synchronization** (Plan 127 Phase 4): The blocking `ask().await` pattern requires bridging the synchronous VM execution model with async message passing. A future implementation will need `SuspendedTask` state (saved IP, stack, wait target) and an `AWAIT_ASK` opcode that suspends the caller until the reply arrives. This is deferred until the async/sync bridge is designed.

- **Scheduler performance under high task counts**: No stress testing has been conducted yet for scenarios with tens of thousands of concurrent tasks. The cooperative yielding budget (10,000 ops) may need tuning.

- **a2c backend for async**: The C transpiler does not yet support `.go` or `~T`. Future work will map these to stackless coroutine state machines and RTOS task primitives (`xTaskCreate` on FreeRTOS).

- **Dead letter queue**: Currently, messages that fail pattern matching and have no `else` handler are silently dropped. A dead letter queue for debugging and monitoring is planned.

## Source Plans

- Plan 121: `docs/plans/121-task-msg-system.md`
- Plan 122: `docs/plans/122-value-access-refactor.md`
- Plan 123: `docs/plans/123-typestore-unification-impl.md`
- Plan 124: `docs/plans/124-async-future-await.md`
- Plan 125: `docs/plans/125-phase3-polymorphic-routing.md`
- Plan 126: `docs/plans/126-phase4-micro-concurrency.md`
- Plan 127: `docs/plans/127-autovm-task-system-execution.md`
- Plan 128: `docs/plans/128-scheduler-message-dispatch.md`
- [195-http-client-async-unification.md](../plans/195-http-client-async-unification.md)
