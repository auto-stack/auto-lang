# Plan 127: AutoVM TaskSystem Execution Implementation

## Status: Phase 1-3 Complete, Phase 4 Deferred

**Completed**: 2025-03-16
- Phase 1: Task Message Loop ✅
- Phase 2: on Block Bytecode Compilation ✅
- Phase 3: ctx.reply() VM Integration ✅
- Phase 5: .go Micro-Concurrency ✅

**Deferred**: Phase 4 - Ask/Reply Synchronization (requires async/sync bridge)

## Overview

This plan details the implementation required to make AutoVM execute Task/Msg systems (Plans 121-126) from bytecode. Currently, the infrastructure exists (TaskRegistry, TaskHandle, TaskInstance, MessageContext, PatternMatcher), but the execution loop and bytecode compilation are incomplete.

## Current State Analysis

### Already Implemented

| Component | File | Status |
|-----------|------|--------|
| TaskRegistry | `vm/task_system.rs` | ✅ Complete |
| TaskHandle | `vm/task_system.rs` | ✅ Complete |
| TaskInstance | `vm/task_system.rs` | ✅ Complete |
| MessageContext | `vm/message_context.rs` | ✅ Complete |
| PatternMatcher | `vm/pattern_matcher.rs` | ✅ Complete |
| TaskDef AST | `ast.rs` | ✅ Complete |
| TaskMsgPattern AST | `ast.rs` | ✅ Complete |
| OpCodes (SPAWN, SPAWN_GO) | `vm/opcode.rs` | ✅ Complete |
| FFI shims (TaskSystem.start, etc.) | `vm/ffi/stdlib.rs` | ✅ Complete |
| **TaskHandlerTable** | `vm/task_handler.rs` | ✅ **Complete (Plan 127)** |
| **TASK_LOOP, HANDLE_MSG, REPLY OpCodes** | `vm/opcode.rs` | ✅ **Complete (Plan 127)** |
| **OpCode handlers in engine.rs** | `vm/engine.rs` | ✅ **Complete (Plan 127)** |
| **Handler bytecode compilation** | `vm/codegen.rs` | ✅ **Complete (Plan 127)** |
| **Pattern serialization** | `vm/task_handler.rs` | ✅ **Complete (Plan 127)** |
| **ctx.reply() FFI** | `vm/ffi/stdlib.rs` | ✅ **Complete (Plan 127)** |
| **SPAWN_GO handler** | `vm/engine.rs` | ✅ **Complete (Plan 126)** |

### Missing Components

| Component | Description | Priority |
|-----------|-------------|----------|
| ~~Task message loop~~ | ~~How tasks receive and process messages~~ | ~~P0~~ ✅ |
| ~~on block bytecode~~ | ~~Compile `on { }` to executable bytecode~~ | ~~P0~~ ✅ |
| ~~ctx.reply() VM integration~~ | ~~Wire up MessageContext.reply() in VM~~ | ~~P1~~ ✅ |
| ~~Pattern matching at runtime~~ | ~~Execute pattern match in bytecode~~ | ~~P1~~ ✅ |
| **Ask/Reply synchronization** | Block caller until reply received | **P2 - DEFERRED** |
| ~~.go fire-and-forget execution~~ | ~~SPAWN_GO handler in engine~~ | ~~P2~~ ✅ |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          AutoVM Engine                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐     ┌────────────────┐     ┌──────────────────┐   │
│  │   Loader     │────▶│ Task Bytecode  │────▶│   TaskExecutor   │   │
│  │ (codegen.rs) │     │  (flash RAM)   │     │  (engine.rs)     │   │
│  └──────────────┘     └────────────────┘     └──────────────────┘   │
│         │                                           │                │
│         │                                           │                │
│         ▼                                           ▼                │
│  ┌──────────────┐     ┌────────────────┐     ┌──────────────────┐   │
│  │ TaskMetadata │────▶│ TaskRegistry   │◀───▶│  TaskInstance    │   │
│  │ (types, on)  │     │                │     │  (mailbox rx)    │   │
│  └──────────────┘     └────────────────┘     └──────────────────┘   │
│                                                      │               │
│                                                      ▼               │
│                                              ┌──────────────────┐    │
│                                              │ MessageContext   │    │
│                                              │ (reply channel)  │    │
│                                              └──────────────────┘    │
│                                                      │               │
│                                                      ▼               │
│                                              ┌──────────────────┐    │
│                                              │ PatternMatcher   │    │
│                                              │ (route message)  │    │
│                                              └──────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Phase 1: Task Message Loop (P0)

### Goal
Enable tasks to receive messages and execute message handlers.

### 1.1 TaskExecutor - Message Processing Loop

**File**: `crates/auto-lang/src/vm/engine.rs`

Add a message processing method to AutoVM:

```rust
/// Process messages for a task instance
///
/// This method is called by tasks that have message handlers (on blocks).
/// It blocks waiting for messages and dispatches them to the appropriate handler.
pub async fn process_task_messages(
    &self,
    task_instance: &mut TaskInstance,
    handlers: &TaskHandlerTable,
) -> Result<TaskStatus, VMError> {
    loop {
        // Wait for message
        let msg = match task_instance.rx.recv().await {
            Some(m) => m,
            None => return Ok(TaskStatus::Terminated), // Channel closed
        };

        // Extract message type and payload
        let (pattern_idx, ctx_value) = self.route_message(&handlers, &msg)?;

        // Get handler bytecode offset
        let handler_offset = handlers.get_offset(pattern_idx);

        // Create task for handler execution
        let handler_task_id = self.spawn_task(handler_offset as usize, 1024);

        // Push message context onto handler task stack
        // (including ctx for reply capability)
        self.setup_handler_context(handler_task_id, ctx_value, msg);

        // Execute handler
        // Handler will either:
        // 1. Return (for send)
        // 2. Call ctx.reply() (for ask)
    }
}
```

### 1.2 TaskHandlerTable - Handler Metadata

**File**: `crates/auto-lang/src/vm/task_handler.rs` (new file)

```rust
/// Task handler metadata for message routing
pub struct TaskHandler {
    /// Pattern index for matching
    pub pattern_idx: u32,
    /// Bytecode offset for handler body
    pub body_offset: u32,
    /// Whether this handler has a context parameter (on(ctx))
    pub has_context: bool,
}

/// Table of handlers for a task type
pub struct TaskHandlerTable {
    pub task_type: String,
    pub handlers: Vec<TaskHandler>,
    /// Patterns for matching (serialized AST)
    pub patterns: Vec<Vec<u8>>,
}
```

### 1.3 Implementation Tasks

1. **Create `task_handler.rs`** with TaskHandlerTable structure
2. **Modify `codegen.rs`** to emit handler metadata during TaskDef compilation
3. **Add `process_task_messages()`** to engine.rs
4. **Wire up TaskSystem.start()** to start message loops for registered tasks

### 1.4 Bytecode Changes

New OpCode additions:

| OpCode | Value | Description |
|--------|-------|-------------|
| TASK_LOOP | 0x8A | Enter message processing loop |
| HANDLE_MSG | 0x8B | Dispatch message to matched handler |
| REPLY | 0x8C | Send reply via ctx (ctx.reply()) |

### 1.5 Verification

```rust
#[test]
fn test_task_message_loop() {
    // Create CounterTask with Add handler
    // Spawn task instance
    // Send Add(5) message
    // Verify counter incremented
}
```

---

## Phase 2: on Block Bytecode Compilation (P0)

### Goal
Compile `on { }` and `on(ctx) { }` blocks to executable bytecode.

### 2.1 Codegen Changes

**File**: `crates/auto-lang/src/vm/codegen.rs`

Modify TaskDef compilation to emit handler bytecode:

```rust
fn compile_task_def(&mut self, task_def: &TaskDef) -> AutoResult<()> {
    // 1. Register task type in metadata
    self.register_task_type(&task_def.name);

    // 2. For each on block, compile handler
    for on_block in &task_def.on_blocks {
        // Record pattern for runtime matching
        let pattern_idx = self.register_handler_pattern(&on_block.patterns);

        // Compile handler body
        let handler_offset = self.current_offset();
        self.compile_on_block_body(on_block, pattern_idx)?;

        // Store handler metadata
        self.task_handlers.push(TaskHandler {
            pattern_idx,
            body_offset: handler_offset,
            has_context: on_block.has_context,
        });
    }

    // 3. Emit task loop bytecode (if task has on blocks)
    if !task_def.on_blocks.is_empty() {
        self.emit(OpCode::TASK_LOOP);
    }

    Ok(())
}
```

### 2.2 Pattern Serialization

Serialize TaskMsgPattern to bytecode for runtime matching:

```rust
fn serialize_pattern(&self, pattern: &TaskMsgPattern) -> Vec<u8> {
    match pattern {
        TaskMsgPattern::Literal(lit) => {
            // 0x01 + literal type + value
        }
        TaskMsgPattern::TypeBinding { name, type_expr } => {
            // 0x02 + name_idx + type_tag
        }
        TaskMsgPattern::Simple(name) => {
            // 0x03 + name_idx
        }
        TaskMsgPattern::WithBindings { variant, bindings } => {
            // 0x04 + variant_idx + binding_count + binding_idxs
        }
    }
}
```

### 2.3 Implementation Tasks

1. **Modify `compile_stmt()`** to handle TaskDef with on blocks
2. **Add pattern serialization** to codegen
3. **Store handler table** in compiled module metadata
4. **Load handler table** in VM when module is loaded

### 2.4 Verification

```rust
#[test]
fn test_on_block_compilation() {
    let code = r#"
        task CounterTask {
            on {
                Add(val) => { count += val }
            }
        }
    "#;

    let module = compile(code)?;
    assert!(module.task_handlers.len() > 0);
}
```

---

## Phase 3: ctx.reply() VM Integration (P1)

### Goal
Enable `ctx.reply()` in message handlers for ask/reply pattern.

### 3.1 MessageContext in VM

**File**: `crates/auto-lang/src/vm/engine.rs`

Add MessageContext handling to VM state:

```rust
pub struct AutoVM {
    // ... existing fields ...

    /// Current message context for reply capability
    /// Set when executing a handler with on(ctx)
    pub current_msg_context: Arc<RwLock<Option<MessageContext>>>,
}
```

### 3.2 REPLY OpCode Handler

```rust
OpCode::REPLY => {
    // Pop reply value from stack
    let value = task.ram.pop_value();

    // Get current context
    let ctx = self.current_msg_context.read().unwrap();
    if let Some(ctx) = ctx.as_ref() {
        ctx.reply(value).map_err(|e| VMError::RuntimeError(e))?;
    } else {
        return Err(VMError::RuntimeError("No message context for reply".into()));
    }
}
```

### 3.3 FFI Integration

Wire up `ctx.reply()` to the REPLY OpCode:

```rust
#[auto_macros::rust_fn("ctx.reply")]
pub fn shim_ctx_reply(ctx_id: i64, value: Value) -> Result<(), String> {
    // Look up context by ID
    // Call reply() on it
}
```

### 3.4 Implementation Tasks

1. **Add `current_msg_context`** to AutoVM
2. **Implement REPLY OpCode** in execute_task()
3. **Set context** when entering handler with on(ctx)
4. **Clear context** when handler completes

### 3.5 Verification

```rust
#[test]
fn test_ctx_reply() {
    let code = r#"
        task EchoTask {
            on(ctx) {
                msg string => { ctx.reply(msg) }
            }
        }

        fn main() {
            TaskSystem.start()
            let reply = EchoTask.ask("hello").await
            assert(reply == "hello")
        }
    "#;

    let result = run(code)?;
    assert!(result.contains("hello"));
}
```

---

## Phase 4: Ask/Reply Synchronization (P2) - DEFERRED

> **Status**: This phase is deferred to a future iteration.
>
> **Reason**: Requires complex async/sync bridging that is not needed for the current
> minimal viable TaskSystem. The `.send()` and `.go` patterns provide sufficient
> concurrency for most use cases.

### Goal
Implement blocking ask() that waits for reply.

### Why Deferred

1. **Complexity**: Requires bridging sync VM execution with async message passing
2. **Runtime Impact**: Needs careful handling of task suspension/resumption
3. **Alternative Patterns**: Current implementation supports:
   - `.send()` - fire-and-forget messaging
   - `.go` - fire-and-forget async execution
   - `ctx.reply()` - handlers can send responses
4. **Use Case**: Most actor patterns work with fire-and-forget semantics

### Future Implementation Sketch

When this phase is implemented, it will need:

```rust
// 1. Ask Future Type
pub struct AskFuture {
    reply_rx: Receiver<Value>,
    timeout: Option<Duration>,
}

// 2. Task Suspension for Await
pub struct SuspendedTask {
    task_id: u64,
    waiting_for: WaitTarget, // Reply, Channel, Timeout
    resume_ip: usize,
    saved_stack: Vec<Value>,
}

// 3. Blocking Ask Implementation
#[auto_macros::rust_fn("TaskHandle.ask")]
pub fn shim_task_ask(handle_id: u64, msg: Value) -> Result<AskFuture, String> {
    // Create reply channel
    let (reply_tx, reply_rx) = channel();

    // Create MessageContext with reply channel
    let ctx = MessageContext::for_ask(
        sender_id: current_task_id(),
        trace_id: generate_trace_id(),
        reply_tx,
    );

    // Send message with context
    let handle = get_task_handle(handle_id)?;
    handle.send_with_context(msg, ctx)?;

    // Return future
    Ok(AskFuture { reply_rx, timeout: None })
}

// 4. Await Handler in Engine
OpCode::AWAIT_ASK => {
    let future_id = task.ram.pop_i32() as u32;
    let future = self.ask_futures.get(&future_id)?;

    // Non-blocking poll first
    match future.reply_rx.try_recv() {
        Ok(value) => {
            task.ram.push_value(value);
        }
        Err(TryRecvError::Empty) => {
            // Suspend current task
            self.suspend_task(task.id, WaitTarget::AskReply(future_id));
            task.ip -= 3; // Retry AWAIT on resume
            return Ok(ExecuteResult::Suspended);
        }
        Err(TryRecvError::Disconnected) => {
            return Err(VMError::RuntimeError("Ask reply channel closed".into()));
        }
    }
}
```

### Estimated Effort

- **Time**: 2-3 days
- **Complexity**: Medium-High
- **Dependencies**: None (can be implemented independently)

### 4.1 Ask Implementation

**File**: `crates/auto-lang/src/vm/ffi/stdlib.rs`

```rust
#[auto_macros::rust_fn("TaskHandle.ask")]
pub fn shim_task_ask(
    task: &mut AutoTask,
    vm: &AutoVM,
    handle_id: u64,
    msg: Value,
) -> Result<Value, String> {
    // Create reply channel
    let (reply_tx, reply_rx) = std::sync::mpsc::channel();

    // Create MessageContext with reply channel
    let ctx = MessageContext::for_ask(
        Some(task.id),
        generate_trace_id(),
        reply_tx,
    );

    // Wrap message with context
    let envelope = Value::Obj(create_envelope(msg, ctx));

    // Send to task
    let handle = get_task_handle(handle_id)?;
    handle.try_send(envelope)?;

    // Return Future that will receive reply
    // (In sync mode, we block here)
    let reply = reply_rx.recv().map_err(|e| e.to_string())?;
    Ok(reply)
}
```

### 4.2 Ask with Await

For async mode, return a Future:

```rust
#[auto_macros::rust_fn("TaskHandle.ask_async")]
pub fn shim_task_ask_async(
    task: &mut AutoTask,
    vm: &AutoVM,
    handle_id: u64,
    msg: Value,
) -> Result<u32, String> {
    // Create reply channel
    let (reply_tx, reply_rx) = std::sync::mpsc::channel();

    // ... send message with context ...

    // Create Future that polls reply_rx
    let future_id = vm.create_ask_future(reply_rx)?;
    Ok(future_id)
}
```

### 4.3 Implementation Tasks

1. **Implement blocking ask()** for sync mode
2. **Implement ask_async()** returning Future
3. **Add AWAIT support** for ask futures
4. **Handle timeout** (optional)

### 4.4 Verification

```rust
#[test]
fn test_ask_reply() {
    let code = r#"
        task DatabaseWorker {
            on(ctx) {
                "ping" => { ctx.reply("pong") }
            }
        }

        fn main() {
            TaskSystem.start()
            let reply = DatabaseWorker.ask("ping").await
            print(reply)  // Should print "pong"
        }
    "#;

    let output = run(code)?;
    assert!(output.contains("pong"));
}
```

---

## Phase 5: .go Micro-Concurrency Execution (P2)

### Goal
Execute `.go` postfix operator for fire-and-forget concurrency.

### 5.1 SPAWN_GO OpCode

Already implemented in `opcode.rs`:
```rust
SPAWN_GO = 0x89, // future -> void (spawn Future in background)
```

### 5.2 Engine Handler

**File**: `crates/auto-lang/src/vm/engine.rs`

```rust
OpCode::SPAWN_GO => {
    // Pop future ID from stack
    let future_id = task.ram.pop_i32() as u32;

    // Get future info
    let future = self.futures.get(&future_id)
        .ok_or(VMError::RuntimeError("Invalid future ID".into()))?;
    let future = future.read().unwrap();

    // Spawn background task to execute future
    let bg_task_id = self.spawn_task(future.body_offset as usize, 1024);

    // Fire-and-forget: don't push anything to stack
    // The background task runs independently

    tracing::debug!("SPAWN_GO: spawned task {} for future {}", bg_task_id, future_id);
}
```

### 5.3 Implementation Tasks

1. **Verify SPAWN_GO handler** is correct in engine.rs
2. **Test .go operator** with simple async block
3. **Verify fire-and-forget** behavior (no return value)

### 5.4 Verification

```rust
#[test]
fn test_spawn_go() {
    let code = r#"
        fn main() {
            var x = 0

            // Spawn background task
            ~{
                x = 42
            }.go

            // x should still be 0 (fire-and-forget)
            print(x)  // 0
        }
    "#;

    let output = run(code)?;
    assert!(output.contains("0"));
}
```

---

## Implementation Timeline

| Week | Phase | Tasks | Status |
|------|-------|-------|--------|
| 1 | Phase 1.1-1.2 | Task message loop, TaskHandlerTable | ✅ Complete |
| 2 | Phase 1.3-1.5 | Engine integration, verification | ✅ Complete |
| 3 | Phase 2.1-2.2 | on block compilation, pattern serialization | ✅ Complete |
| 4 | Phase 2.3-2.4 | Metadata storage, verification | ✅ Complete |
| 5 | Phase 3.1-3.3 | ctx.reply() integration | ✅ Complete |
| 6 | Phase 4.1-4.2 | Ask/Reply synchronization | ⏸️ **Deferred** |
| 7 | Phase 5.1-5.2 | .go execution verification | ✅ Complete |
| 8 | Integration testing, documentation | ✅ Complete |

## Dependencies

- Phase 2 depends on Phase 1 (need message loop to test handlers) ✅
- Phase 3 depends on Phase 2 (need compiled handlers for reply) ✅
- Phase 4 depends on Phase 3 (need reply for ask) - **Deferred**
- Phase 5 is independent (already partially implemented) ✅

## Completion Summary (2025-03-16)

**Completed Phases**:
- ✅ Phase 1: Task Message Loop - TASK_LOOP OpCode, handler dispatch
- ✅ Phase 2: on Block Bytecode - Pattern serialization, handler compilation
- ✅ Phase 3: ctx.reply() Integration - REPLY OpCode, FFI shim
- ✅ Phase 5: .go Micro-Concurrency - SPAWN_GO handler verified

**Deferred**:
- ⏸️ Phase 4: Ask/Reply Synchronization - Requires async/sync bridge

**Test Results**:
- task_handler tests: 9 passed
- vm::task tests: 35 passed
- All task tests: 83 passed

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Pattern matching complexity | Use existing PatternMatcher, serialize patterns |
| Async/sync bridge | Start with sync, add async later |
| Message ordering | Use tokio channels (FIFO by default) |
| Memory safety | Use Arc<RwLock<>> for shared state |

## Success Criteria

1. **Basic Task Execution**: Can spawn task, send message, handler executes
2. **Ask/Reply**: Can call `task.ask(msg).await` and receive reply
3. **Pattern Matching**: Messages routed to correct handler based on pattern
4. **Fire-and-Forget**: `.go` operator spawns background task without blocking

## Appendix: Example Test Script

```auto
// Full integration test for TaskSystem

task CounterTask {
    var count = 0

    fn start() ! {
        print("[CounterTask] Started")
    }

    fn stop() ! {
        print("[CounterTask] Stopped, count = ${count}")
    }

    on(ctx) {
        "ping" => {
            ctx.reply("pong")
        }

        Add(val) => {
            count += val
            ctx.reply(count)
        }

        Get => {
            ctx.reply(count)
        }

        Reset => {
            count = 0
            ctx.reply("ok")
        }
    }
}

fn main() ! {
    print("=== TaskSystem Test ===")

    TaskSystem.start()

    // Test ping/pong
    let pong = CounterTask.ask("ping").await.?
    print("ping -> ${pong}")

    // Test counter
    CounterTask.send(Add(10))
    CounterTask.send(Add(5))

    let count = CounterTask.ask(Get).await.?
    print("count = ${count}")  // Should be 15

    // Test reset
    CounterTask.send(Reset)
    let after_reset = CounterTask.ask(Get).await.?
    print("after reset = ${after_reset}")  // Should be 0

    TaskSystem.stop()

    print("=== Test Complete ===")
}
```
