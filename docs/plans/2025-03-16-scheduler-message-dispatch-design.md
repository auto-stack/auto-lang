# Scheduler Message Dispatch Loop - Design Document

## Overview

This document describes the design for the AutoVM scheduler message dispatch loop, which enables Actor-style message passing between tasks.

**Status**: Approved (2025-03-16)

## Architecture

### Core Principle: Zero-Shared-Mutable-State

The architecture follows a **Hybrid** model:
- **GlobalMeta** = `Arc<GlobalMeta>` (strictly read-only, zero locks)
- **TaskContext** = Per-task owned (mailbox rx + task.ram + executor)
- **Scheduling** = `tokio::spawn` × N, leveraging Tokio's work-stealing

```
┌─────────────────────────────────────────────────────────────────────┐
│                     TaskSystem.start() [Daemon Loop]                │
│                                                                     │
│   sys_rx.recv().await ──┬──► Spawn ──► spawn_dynamic_task()        │
│                         │                                           │
│                         └──► Stop ──► break (shutdown)              │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    tokio::spawn × N                           │   │
│  │                                                               │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐            │   │
│  │  │ TaskCtx 1   │  │ TaskCtx 2   │  │ TaskCtx N   │            │   │
│  │  │ (Logger)    │  │ (Monitor)   │  │ (Worker)    │            │   │
│  │  │             │  │             │  │             │            │   │
│  │  │ mailbox: rx │  │ mailbox: rx │  │ mailbox: rx │            │   │
│  │  │ task.ram    │  │ task.ram    │  │ task.ram    │            │   │
│  │  │ sys_tx      │  │ sys_tx      │  │ sys_tx      │            │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘            │   │
│  │         │                │                │                   │   │
│  │         ▼                ▼                ▼                   │   │
│  │  ┌────────────────────────────────────────────────────────┐   │   │
│  │  │              GlobalMeta (Arc<GlobalMeta>)              │   │   │
│  │  │              (零锁、零拷贝、零竞争)                      │   │   │
│  │  │                                                        │   │   │
│  │  │  bytecode: VirtualFlash                                │   │   │
│  │  │  string_pool: Vec<Vec<u8>>                             │   │   │
│  │  │  native_interface: NativeInterface                     │   │   │
│  │  │  handler_tables: HashMap<String, TaskHandlerTable>     │   │   │
│  │  └────────────────────────────────────────────────────────┘   │   │
│  │                                                               │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Components

### GlobalMeta (Shared Read-Only)

```rust
/// Global read-only metadata - wrapped in Arc<GlobalMeta>
/// No inner Arcs needed since outer Arc provides protection
pub struct GlobalMeta {
    /// Bytecode (Flash)
    pub bytecode: VirtualFlash,

    /// String pool (read-only)
    pub string_pool: Vec<Vec<u8>>,

    /// Native interface
    pub native_interface: NativeInterface,

    /// Handler tables per task type
    pub handler_tables: HashMap<String, TaskHandlerTable>,
}
```

### TaskContext (Per-Task Owned)

```rust
/// Per-task execution context - completely isolated, no shared mutable state
pub struct TaskContext {
    // ========== Shared Read-Only Metadata ==========
    pub meta: Arc<GlobalMeta>,

    // ========== Task-Isolated State ==========
    pub task_type: String,
    pub instance_id: u64,
    pub mailbox: mpsc::Receiver<Value>,
    pub sys_tx: mpsc::Sender<SystemCommand>,
    pub task: AutoTask,  // Contains ram, ip, bp

    // ========== Handler Table Reference ==========
    pub handlers: TaskHandlerTable,
}
```

### SystemCommand (Privileged Channel)

```rust
pub enum SystemCommand {
    /// Shutdown the system
    Stop,

    /// Dynamically spawn a child task
    Spawn {
        task_type: String,
        capacity: usize,
        parent_id: Option<u64>,
    },
}
```

## Task Lifecycle

### Phase 1: Deployment (Before TaskSystem.start())

- `main()` runs, compiles all task definitions
- `GlobalMeta` is frozen and constructed
- `TaskHandle.send()` fills mailboxes (messages wait quietly in queues)

### Phase 2: Ignition (TaskSystem.start())

```rust
fn start_scheduler(&self) {
    // Spawn all pre-registered tasks
    self.spawn_initial_tasks();

    // System Daemon Loop
    while let Some(cmd) = sys_rx.recv().await {
        match cmd {
            SystemCommand::Spawn { task_type, capacity, .. } => {
                spawn_dynamic_task(&meta, task_type, capacity);
            }
            SystemCommand::Stop => {
                break; // Shutdown
            }
        }
    }

    // Execute stop hooks
    execute_stop_hooks();
}
```

### Phase 3: Message Loop (Per-Task tokio::spawn)

```rust
async fn task_loop(mut ctx: TaskContext) {
    // 1. Start Hook
    if let Some(start_offset) = ctx.handlers.start_hook_offset {
        ctx.task.ip = start_offset as usize;
        execute_handler_fully(&ctx.meta, &mut ctx.task).await;
    }

    // 2. Main Message Loop
    loop {
        let msg = match ctx.mailbox.recv().await {
            Some(m) => m,
            None => break, // Mailbox closed
        };

        let mut matched = false;
        for handler in &ctx.handlers.handlers {
            let pattern = &ctx.meta.handler_patterns[handler.pattern_idx as usize];
            if let Some(bindings) = PatternMatcher::match_pattern(pattern, &msg) {
                inject_bindings(&mut ctx.task, &bindings);
                ctx.task.ip = handler.body_offset as usize;
                execute_handler_fully(&ctx.meta, &mut ctx.task).await;
                matched = true;
                break;
            }
        }

        // 3. Else Handler
        if !matched {
            if let Some(else_offset) = ctx.handlers.else_handler_offset {
                ctx.task.ip = else_offset as usize;
                execute_handler_fully(&ctx.meta, &mut ctx.task).await;
            }
        }
    }

    // 4. Stop Hook
    if let Some(stop_offset) = ctx.handlers.stop_hook_offset {
        ctx.task.ip = stop_offset as usize;
        execute_handler_fully(&ctx.meta, &mut ctx.task).await;
    }
}
```

### Phase 4: Shutdown

- Handler calls `TaskSystem.stop()`
- `sys_tx.send(SystemCommand::Stop)` is sent
- Daemon loop receives and breaks
- `TaskSystem.start()` returns
- `main()` continues with cleanup code

## Critical: Async Handler Execution

### The Problem

Two fatal flaws in a naive design:

1. **Budget exhaustion causes state loss**: If `ops_executed >= budget` triggers `break`, the handler is "腰斩" (cut in half) and the outer loop thinks the message is done.

2. **Sync function blocks Tokio**: A sync `execute_until_yield` cannot execute async FFI or `.await` operations.

### The Solution: Async Handler Execution with Cooperative Yielding

```rust
/// Execute handler fully - async to support await, yields instead of breaking
async fn execute_handler_fully(meta: &GlobalMeta, task: &mut AutoTask) {
    const BUDGET: u32 = 10_000;
    let mut ops_executed = 0;

    loop {
        if task.ip >= meta.bytecode.memory.len() { break; }

        let op_byte = meta.bytecode.read_u8(task.ip);
        let opcode = OpCode::from(op_byte);
        task.ip += 1;

        match opcode {
            OpCode::RET | OpCode::HALT => break, // Normal completion

            // Async FFI support
            OpCode::AWAIT_EXT => {
                handle_async_ffi(meta, task).await; // True suspension
            }

            _ => {
                execute_single_op(meta, task, opcode);
            }
        }

        // Budget defense: yield CPU, but preserve task.ip state
        ops_executed += 1;
        if ops_executed >= BUDGET {
            tokio::task::yield_now().await; // Cooperative yield
            ops_executed = 0; // Reset and continue
        }
    }
}
```

### Why This Works

| Feature | Mechanism |
|---------|-----------|
| **No state loss** | `yield_now()` preserves `task.ip`, loop continues after yield |
| **No CPU starvation** | Dead loops (`while true { 1+1 }`) still yield periodically |
| **Async native** | `async fn` enables `AWAIT_EXT` and `.await` support |
| **Fair scheduling** | Tokio work-stealing distributes tasks across CPU cores |

## Pattern Matching

Uses existing `PatternMatcher` from Plan 125 (`crates/auto-lang/src/vm/pattern_matcher.rs`):

- `Literal` matching (strings, ints, bools)
- `TypeBinding` patterns (`msg string`)
- `Simple` variants (`Hello`, `Quit`)
- `WithBindings` variants (`Add(val)`)

## Key Component Summary

| Component | Responsibility | Shared? |
|-----------|---------------|---------|
| **GlobalMeta** | Bytecode, strings, handlers | ✅ Arc (Read-Only) |
| **TaskContext** | Mailbox, ram, local vars | ❌ Per-task (Owned) |
| **TaskHandlerTable** | Pattern → body_offset | ✅ In GlobalMeta (Read-Only) |
| **AutoTask** | Execution state (ip, bp, ram) | ❌ Inside TaskContext |
| **SystemCommand** | Privileged control | Channel (Send/Receive) |

## Error Handling

- Pattern matching failure → else handler (if exists) → message dropped
- Handler execution error → task terminated, error logged
- Mailbox closed → task exits loop, runs stop hook

## Testing Strategy

1. **Unit tests**: Pattern matching, handler lookup
2. **Integration tests**: Single task message loop
3. **Concurrency tests**: Multiple tasks sending messages
4. **Stress tests**: High-volume message passing
5. **Shutdown tests**: TaskSystem.stop() from handler

## Future Enhancements

- Handler priority
- Message batching
- Dead letter queue
- Distributed task spawning
