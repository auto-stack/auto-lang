# Plan 069: AutoVM Task/Msg Async Concurrency Framework

**Status**: Not Started  
**Priority**: High (Architecture-Critical)  
**Owner**: AutoLang Team  
**Related**: `docs/design/autovm-task-msg.md`, `docs/design/autovm-tokio.md`, Plan 068

## 1. Objective

Integrate **Tokio-based Task/Msg async concurrency** into BigVM **before** expanding feature implementations. This ensures the VM architecture supports M:N green thread scheduling from the start, avoiding costly future rewrites.

**Core Principle**: "架构先行，特性填充" - Architecture first, features follow.

## 2. Rationale (Why Now?)

As documented in `autovm-task-msg.md`:

1. **Blocking Disaster**: Current synchronous `run()` loop cannot be extended with `RECV`, `SLEEP`, or async FFI without blocking the entire Tokio thread
2. **Refactor Hell**: Adding Task support later requires massive structural changes to stack/frame management
3. **FFI Incompatibility**: Sync FFI interfaces (`fn call() -> Value`) cannot migrate to async without breaking all plugins

**Cost Analysis**:
- Now: ~500-800 lines of framework refactoring
- Later: ~3000+ lines + subtle concurrency bugs

## 3. Architecture Overview

### 3.1 Core Mapping

| Auto Concept | Rust/Tokio Implementation |
|--------------|---------------------------|
| `Task` | `tokio::spawn(async move { ... })` |
| `Channel` | `tokio::sync::mpsc` |
| `sleep()` | `tokio::time::sleep()` |
| FFI async call | `Future<Output=Value>` |

### 3.2 Key Structs

```rust
// Per-task execution context (extracted from current BigVM)
struct AutoTask {
    id: TaskId,
    stack: Vec<i32>,      // Virtual stack (task-local)
    frames: Vec<Frame>,   // Call frames
    ip: usize,            // Instruction pointer
    bp: usize,            // Base pointer
    status: TaskStatus,
}

// VM Runtime (shared across tasks)
struct BigVM {
    tasks: DashMap<TaskId, Arc<Mutex<AutoTask>>>,
    id_gen: AtomicU64,
    flash: Arc<VirtualFlash>,     // Shared bytecode (read-only)
    strings: Arc<Vec<Vec<u8>>>,   // Shared string pool
    native: Arc<NativeInterface>, // Thread-safe FFI
}
```

## 4. Implementation Phases

### Phase 1: Tokio Integration & Struct Refactoring
**Goal**: Split monolithic `BigVM` into `BigVM` (runtime) + `AutoTask` (state)

- [ ] **1.1 Add Tokio Dependency**
    - Add `tokio = { version = "1", features = ["full"] }` to `crates/auto-lang/Cargo.toml`
    - Add `dashmap = "5"` for concurrent task registry
    
- [ ] **1.2 Create Task Module**
    - Create `crates/auto-lang/src/vm/task.rs`
    - Define `TaskId`, `TaskStatus`, `AutoTask` structs
    - Move per-task state (stack, frames, ip, bp) from `BigVM` to `AutoTask`
    
- [ ] **1.3 Refactor BigVM**
    - Modify `engine.rs`: `BigVM` holds shared resources only
    - Add `tasks: DashMap<TaskId, Arc<Mutex<AutoTask>>>`
    - Add `id_gen: AtomicU64` for task ID generation

---

### Phase 2: Async Execution Loop
**Goal**: Convert synchronous `run()` to async `run_task_loop()`

- [ ] **2.1 Implement Cooperative Scheduling**
    - Create `async fn run_task_loop(&self, task: Arc<Mutex<AutoTask>>)`
    - Budget-based execution: run N instructions, then yield
    - Use `tokio::task::yield_now().await` for fairness
    
- [ ] **2.2 Implement `spawn_task()`**
    - Create new `AutoTask` with initial function entry point
    - Register in task registry
    - Call `tokio::spawn()` with async execution loop
    
- [ ] **2.3 Add Task Opcodes**
    - `OP_SPAWN = 0x80`: Spawn new task from function
    - `OP_TASK_ID = 0x81`: Push current task ID to stack
    - `OP_YIELD = 0x82`: Explicit yield point

---

### Phase 3: Channel Implementation
**Goal**: Enable inter-task communication via message passing

- [ ] **3.1 Channel Data Structure**
    - Create `crates/auto-lang/src/vm/channel.rs`
    - Define `AutoChannel` wrapping `tokio::sync::mpsc`
    - Channel registry in `BigVM`
    
- [ ] **3.2 Channel Opcodes**
    - `OP_CHAN_NEW = 0x83`: Create new channel (capacity on stack)
    - `OP_SEND = 0x84`: Send value to channel (may await if full)
    - `OP_RECV = 0x85`: Receive value from channel (await until msg)
    - `OP_TRY_RECV = 0x86`: Non-blocking receive
    
- [ ] **3.3 Async Yield Points**
    - Modify execution loop to handle `YieldReason::Send/Recv`
    - Implement channel await outside task lock

---

### Phase 4: Timer & Sleep Support
**Goal**: Non-blocking sleep/timer operations

- [ ] **4.1 Sleep Opcode**
    - `OP_SLEEP = 0x87`: Sleep for N milliseconds
    - Implementation: `tokio::time::sleep(Duration::from_millis(n)).await`
    
- [ ] **4.2 Timeout Wrapper** (Optional)
    - `OP_TIMEOUT = 0x88`: Wrap channel recv with timeout

---

### Phase 5: Integration & Migration
**Goal**: Migrate existing functionality to async architecture

- [ ] **5.1 Update auto-vm Binary**
    - Change `main()` to `#[tokio::main] async fn main()`
    - Create initial task for `main()` function
    
- [ ] **5.2 Migrate Existing Instructions**
    - Move arithmetic/comparison handlers to work with `AutoTask`
    - Ensure all existing tests pass on new architecture
    
- [ ] **5.3 Update Test Infrastructure**
    - Modify `run_bigvm()` to use async runtime
    - Add concurrency-specific tests

---

### Phase 6: Verification & Validation
**Goal**: Prove M:N scheduling works correctly

- [ ] **6.1 Interleaved Execution Test**
    - Test: Two tasks, one prints "A" every 1s, one prints "B" every 0.5s
    - Verify: Console shows interleaved A/B output
    
- [ ] **6.2 Producer-Consumer Test**
    - Test: Producer task sends 1-10, consumer receives and prints
    - Verify: All messages received in order
    
- [ ] **6.3 Stress Test**
    - Spawn 1000 tasks, each does simple math
    - Verify: All complete without deadlock

## 5. New OpCodes Summary

| OpCode | Value | Description | Async? |
|--------|-------|-------------|--------|
| `SPAWN` | 0x80 | Spawn task from function addr | No |
| `TASK_ID` | 0x81 | Get current task ID | No |
| `YIELD` | 0x82 | Explicit yield | Yes |
| `CHAN_NEW` | 0x83 | Create channel | No |
| `SEND` | 0x84 | Send to channel | Yes |
| `RECV` | 0x85 | Receive from channel | Yes |
| `TRY_RECV` | 0x86 | Non-blocking recv | No |
| `SLEEP` | 0x87 | Sleep N ms | Yes |

## 6. File Changes

### New Files
- `crates/auto-lang/src/vm/task.rs` - Task struct and status
- `crates/auto-lang/src/vm/channel.rs` - Channel wrapper
- `crates/auto-lang/src/vm/scheduler.rs` - Scheduling utilities

### Modified Files
- `crates/auto-lang/Cargo.toml` - Add tokio, dashmap
- `crates/auto-lang/src/vm/engine.rs` - Major refactor
- `crates/auto-lang/src/vm/opcode.rs` - New task/channel opcodes
- `crates/auto-vm/src/main.rs` - Async main
- `crates/auto-lang/src/vm/tests_bigvm.rs` - Async test harness

## 7. MicroVM Compatibility Note

The Task abstraction is **ISA-compatible** with MicroVM (FreeRTOS):

| BigVM (Tokio) | MicroVM (FreeRTOS) |
|---------------|-------------------|
| `tokio::spawn()` | `xTaskCreate()` |
| `mpsc::recv().await` | `xQueueReceive()` |
| `tokio::time::sleep()` | `vTaskDelay()` |

The core interpretation logic (`run_steps()`) can be shared via `no_std` compatible code.

## 8. Success Criteria

1. ✅ Two tasks can run concurrently with interleaved output
2. ✅ Channel send/recv works correctly
3. ✅ All existing Category A tests still pass
4. ✅ No deadlocks under stress test
5. ✅ `sleep()` doesn't block other tasks

## 9. Timeline Estimate

| Phase | Effort | Dependency |
|-------|--------|------------|
| Phase 1 | 2-3 hours | None |
| Phase 2 | 3-4 hours | Phase 1 |
| Phase 3 | 2-3 hours | Phase 2 |
| Phase 4 | 1 hour | Phase 2 |
| Phase 5 | 2-3 hours | Phase 1-4 |
| Phase 6 | 1-2 hours | Phase 5 |

**Total**: ~12-16 hours of focused implementation

## 10. Risk Mitigation

1. **Deadlock Risk**: Use `tokio::sync::Mutex` (not `std::sync`), always drop lock before await
2. **Stack Safety**: Each task has isolated stack, no cross-task pointer issues
3. **Backward Compat**: Keep synchronous `run()` as legacy wrapper during transition
