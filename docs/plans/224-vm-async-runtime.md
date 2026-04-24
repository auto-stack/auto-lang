# Plan 224: VM Async Runtime — TaskSystem.run 桥接 + AWAIT_FUTURE 完善 + Async FFI

## Status: 🔧 PLANNED

**Goal:** 完善 AutoLang VM 的 async 执行基础设施，使 `TaskSystem.run(~{ ... })` 能真正执行异步代码块，`AWAIT_FUTURE` 能正确执行 async body 的字节码，FFI shim 能执行异步操作。

**Architecture:** 三层递进：先提取 VM 执行引擎为可重入函数，再让 `AWAIT_FUTURE` 真正执行 async body，最后实现 `TaskSystem.run` 的 tokio runtime 桥接。

**Tech Stack:** Rust (tokio), AutoLang VM engine, opcode system

---

## 现状分析

### 已有实现（Plan 124 Phase 2.1-2.3 完成）

| 组件 | 文件 | 状态 | 说明 |
|------|------|------|------|
| `CREATE_FUTURE` opcode | `engine.rs:4243-4268` | ✅ 可用 | 创建 FutureValue，压入编码 ID |
| `POLL_FUTURE` opcode | `engine.rs:4346-4398` | ✅ 可用 | 非阻塞 poll，返回 ready/value |
| `SPAWN_GO` opcode | `engine.rs:3740-3769` | ✅ 可用 | fire-and-forget task spawn |
| `AWAIT_FUTURE` (Ready/Failed) | `engine.rs:4285-4303` | ✅ 可用 | 已完成时返回结果 |
| `FutureValue` 结构 | `engine.rs:246-254` | ✅ 可用 | body_offset, state, result, owner_task_id |
| `AutoVM.futures` 注册表 | `engine.rs:234` | ✅ 可用 | `DashMap<u32, Arc<RwLock<FutureValue>>>` |
| `~T` 语法 + `.await` | lexer/parser | ✅ 可用 | 已有 AST + 编译到 opcode |

### 占位符实现（需要完善）

| 组件 | 文件 | 当前行为 | 缺失内容 |
|------|------|---------|---------|
| `AWAIT_FUTURE` (Pending) | `engine.rs:4305-4345` | 不执行 body，直接标记 Ready(Int(0)) | 需要递归执行 body 字节码 |
| `TaskSystem.run` FFI | `stdlib.rs:2984-2996` | `#[rust_fn]` 返回 `Ok(0)`，忽略 future_id | 需要访问 VM，创建 tokio runtime，执行 future |
| `execute_handler_fully` | `scheduler.rs:173-219` | 只支持 RET/HALT/NOP，跳过其他 | 需要支持 NATIVE_CALL/PUSH/POP/LOAD/STORE 等 |
| `AWAIT_EXT` opcode | `scheduler.rs:201-204` | 仅注释，未定义 | 需要定义 opcode + 实现挂起/恢复 |

### 关键架构限制

1. **VM 执行不可重入** — 主执行循环在 `AutoVM::impl` 中，未提取为独立函数，`AWAIT_FUTURE` 无法递归调用
2. **`#[rust_fn]` 无法访问 AutoVM** — 宏生成的签名是 `fn(args) -> Result<Ret, String>`，拿不到 VM 实例
3. **无 task 挂起/恢复机制** — 没有 waker，没有 scheduler 集成，task 不能暂停等待 future
4. **`execute_handler_fully` 断裂** — 只有 `GlobalMeta` 而非完整 `AutoVM`，无法执行 native call

### 数据结构现状

```rust
// engine.rs:246-262
pub struct FutureValue {
    pub body_offset: u32,        // async body 的字节码偏移
    pub state: FutureState,      // Pending / Ready / Failed
    pub result: Option<Value>,   // 执行结果
    pub owner_task_id: TaskId,   // 所属 task
}

pub enum FutureState {
    Pending,
    Ready,
    Failed,
}

// engine.rs:234-235
pub futures: DashMap<u32, Arc<RwLock<FutureValue>>>,
pub future_id_gen: AtomicU32,
```

---

## 设计决策

### 1. 提取 VM 执行为可重入函数

**选择提取独立函数而非重构为 iterator**

原因：
- 当前主循环在 `AutoVM::execute()` 中，约 4000+ 行
- 不需要完整的重构，只需提取核心 dispatch loop
- 保持现有 `execute()` 作为入口，内部调用提取出的函数

方案：提取 `execute_bytecode(vm: &AutoVM, task: &mut AutoTask, budget: u32) -> ExecutionResult`

### 2. AWAIT_FUTURE 执行策略：同步嵌套执行

**选择在当前 task 中同步执行 async body，而非挂起 task**

原因：
- 当前 scheduler 只有简单的 cooperative yielding，没有真正的 async 调度
- 挂起/恢复需要 waker + scheduler round-trip，复杂度高
- 嵌套执行可以立即获得结果，语义清晰
- 后续 Plan 可以升级为真正的挂起/恢复

方案：`AWAIT_FUTURE` 遇到 Pending 时，递归调用 `execute_bytecode()` 执行 body_offset 处的字节码，直到遇到 `RET` 或 `AWAIT_FUTURE`（递归 await）。

### 3. TaskSystem.run 策略：手动 shim + tokio::runtime::block_on

**选择手动 shim 替代 `#[rust_fn]`**

原因：
- `#[rust_fn]` 无法访问 `AutoVM`，拿不到 `futures` 注册表
- 手动 shim 可以接收 `task` 和 `vm` 引用
- `block_on` 在当前线程执行，语义简单

方案：将 `shim_task_system_run` 改为手动 shim（去掉 `#[rust_fn]`），创建 `tokio::runtime::Runtime` 并 `block_on` 执行 future body。

### 4. Async FFI 策略：暂不引入 AWAIT_EXT

**选择在 async FFI 中直接 `block_on`，不引入新 opcode**

原因：
- `AWAIT_EXT` 需要完整的 task 挂起/恢复机制（waker + scheduler）
- 当前阶段用 `block_on` 即可满足 HTTP async 等需求
- 后续需要真正非阻塞时再引入

方案：native function 内部创建 `tokio::runtime::Runtime` 并 `block_on`。简单但阻塞当前线程。

---

## Implementation Phases

### Phase 1: 提取 VM 执行引擎为可重入函数

**Goal:** 将主执行循环的核心 dispatch 提取为独立函数，供 `AWAIT_FUTURE` 和 `execute_handler_fully` 调用。

#### Task 1.1: 定义 ExecutionResult 和提取 execute_single_frame

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs`

**Step 1: 定义返回类型**

```rust
/// 执行一帧字节码的结果
pub enum FrameResult {
    /// 正常继续执行
    Continue,
    /// 遇到 RET/HALT，返回值为栈顶
    Return,
    /// 遇到 AWAIT_FUTURE (Pending)，需要递归处理
    AwaitFuture { future_id: u32, body_offset: u32 },
    /// 遇到错误
    Error(VMError),
    /// 执行预算耗尽（用于 cooperative yielding）
    BudgetExhausted,
}
```

**Step 2: 提取 execute_single_frame**

从 `AutoVM::execute()` 的主循环中提取出一个方法：

```rust
impl AutoVM {
    /// Execute bytecode for a single frame (until RET/HALT/error/budget)
    /// This can be called recursively for AWAIT_FUTURE execution.
    pub fn execute_single_frame(
        &self,
        task: &mut AutoTask,
        budget: u32,
    ) -> FrameResult {
        let mut ops = 0;
        loop {
            // ... 从主循环复制的 dispatch 逻辑 ...
            // 遇到 RET -> FrameResult::Return
            // 遇到 HALT -> FrameResult::Return
            // 遇到 AWAIT_FUTURE(Pending) -> FrameResult::AwaitFuture { ... }
            // 遇到错误 -> FrameResult::Error(...)
            ops += 1;
            if ops >= budget {
                return FrameResult::BudgetExhausted;
            }
        }
    }
}
```

**Step 3: 修改 AutoVM::execute() 调用 execute_single_frame**

```rust
pub fn execute(&mut self, task: &mut AutoTask) -> Result<(), VMError> {
    loop {
        match self.execute_single_frame(task, 10_000) {
            FrameResult::Continue => unreachable!(), // budget handles this
            FrameResult::Return => return Ok(()),
            FrameResult::AwaitFuture { future_id, body_offset } => {
                // Handle await (Phase 2)
                self.handle_await_future(task, future_id, body_offset)?;
            }
            FrameResult::Error(e) => return Err(e),
            FrameResult::BudgetExhausted => {
                std::thread::yield_now();
            }
        }
    }
}
```

**Step 4: 运行测试**

Run: `rtk cargo test -p auto-lang --lib`
Expected: 现有测试全部通过（行为不变，只是重构）

---

#### Task 1.2: execute_handler_fully 复用 VM 执行引擎

**Files:**
- Modify: `crates/auto-lang/src/vm/scheduler.rs`

**Step 1: 在 GlobalMeta 中添加 AutoVM 引用**

或者更好的方案：让 `execute_handler_fully` 接收 `&AutoVM` 参数。

**Step 2: 复用 execute_single_frame**

```rust
pub async fn execute_handler_fully(
    vm: &AutoVM,
    task: &mut AutoTask,
) -> Result<TaskStatus, VMError> {
    loop {
        match vm.execute_single_frame(task, 10_000) {
            FrameResult::Continue => unreachable!(),
            FrameResult::Return => return Ok(TaskStatus::Terminated),
            FrameResult::Error(e) => return Err(e),
            FrameResult::BudgetExhausted => {
                tokio::task::yield_now().await;
            }
            _ => return Err(VMError::RuntimeError("Unexpected frame result in handler".into())),
        }
    }
}
```

**Step 3: 更新所有调用点**

搜索 `execute_handler_fully` 的调用者，更新签名。

**Step 4: 运行测试**

Run: `rtk cargo test -p auto-lang --lib`
Expected: 通过

---

### Phase 2: AWAIT_FUTURE 完善

**Goal:** 让 `AWAIT_FUTURE` 遇到 Pending 状态时，真正执行 async body 的字节码。

#### Task 2.1: 实现 handle_await_future

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs`

**Step 1: 实现递归执行**

在 `execute_single_frame` 中，当遇到 `AWAIT_FUTURE` 且状态为 Pending 时：

```rust
FrameResult::AwaitFuture { future_id, body_offset } => {
    self.handle_await_future(task, future_id, body_offset)?;
    // 不 return Continue — handle_await_future 会将结果压入栈
    // 继续主循环的下一次迭代
}
```

实现 `handle_await_future`：

```rust
impl AutoVM {
    fn handle_await_future(
        &self,
        task: &mut AutoTask,
        future_id: u32,
        body_offset: u32,
    ) -> Result<(), VMError> {
        // 保存当前 IP
        let saved_ip = task.ip;

        // 创建临时 task 执行 async body
        let mut body_task = AutoTask::new(task.id);
        body_task.ip = body_offset as usize;
        // 复制栈帧（闭包捕获的变量）
        // ... 从当前 task 复制闭包环境 ...

        // 递归执行 async body
        loop {
            match self.execute_single_frame(&mut body_task, 10_000) {
                FrameResult::Return => {
                    // Body 执行完毕，取栈顶结果
                    let result = body_task.ram.pop_i64();
                    task.ram.push_i64(result);
                    break;
                }
                FrameResult::Error(e) => {
                    // 标记 future 为 Failed
                    if let Some(fv) = self.futures.get(&future_id) {
                        fv.write().unwrap().state = FutureState::Failed;
                    }
                    task.ram.push_i64(0);
                    break;
                }
                FrameResult::AwaitFuture { future_id: inner_id, body_offset: inner_offset } => {
                    // 递归 await
                    self.handle_await_future(&mut body_task, inner_id, inner_offset)?;
                }
                FrameResult::BudgetExhausted => {
                    std::thread::yield_now();
                }
                FrameResult::Continue => unreachable!(),
            }
        }

        // 恢复 IP
        task.ip = saved_ip;
        Ok(())
    }
}
```

**Step 2: 扩展 AWAIT_FUTURE 结果类型**

当前只支持 Int/Nil，需要支持 String、Array 等更多类型。

**Step 3: 运行测试**

Run: `rtk cargo test -p auto-lang --lib`
Expected: 通过

**Step 4: Commit**

```
feat(vm): implement AWAIT_FUTURE body execution via recursive execute_single_frame
```

---

### Phase 3: TaskSystem.run 桥接

**Goal:** 让 `TaskSystem.run(~{ ... })` 能创建 tokio runtime 并执行 future body。

#### Task 3.1: 将 TaskSystem.run 改为手动 shim

**Files:**
- Modify: `crates/auto-lang/src/vm/ffi/stdlib.rs`

**Step 1: 移除 #[rust_fn] 宏，改为手动 shim**

```rust
// 替换:
// #[auto_macros::rust_fn("TaskSystem.run")]
// pub fn shim_task_system_run(_future_id: i64) -> Result<i64, String> { Ok(0) }

// 改为手动 shim:
pub fn shim_task_system_run(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let future_id: i64 = task.ram.pop_i64();

    // 从 VM futures 注册表获取 future
    let future_val = _vm.futures.get(&(future_id as u32))
        .ok_or_else(|| VMError::RuntimeError(format!("Invalid future id: {}", future_id)))?;

    let mut fv = future_val.write().map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let body_offset = fv.body_offset;
    fv.state = FutureState::Pending;

    // 创建 tokio runtime 并执行
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| VMError::RuntimeError(format!("Failed to create runtime: {}", e)))?;

    // 注意：这里需要一种方式让 tokio runtime 执行 VM 字节码
    // 方案 A：在 block_on 内部递归调用 execute_single_frame
    // 方案 B：使用 oneshot channel 桥接

    // 方案 A（简单直接）:
    rt.block_on(async {
        // 在 async context 中执行 VM bytecode
        // 注意：这会阻塞 tokio runtime 的线程池
        let mut body_task = AutoTask::new(0);
        body_task.ip = body_offset as usize;
        // ... 复制闭包环境 ...

        // 同步执行（在 async block 内，但实际是阻塞的）
        // 这不是真正的 async，但功能上可以工作
        loop {
            match _vm.execute_single_frame(&mut body_task, 10_000) {
                FrameResult::Return => {
                    let result = body_task.ram.pop_i64();
                    fv.result = Some(auto_val::Value::Int(result));
                    fv.state = FutureState::Ready;
                    break;
                }
                FrameResult::Error(e) => {
                    fv.state = FutureState::Failed;
                    break;
                }
                FrameResult::BudgetExhausted => {
                    tokio::task::yield_now().await;
                }
                _ => {}
            }
        }
    });

    task.ram.push_i64(0); // void return
    Ok(())
}
```

**Step 2: 更新 register_stdlib_ffi 注册**

当前使用 `__shim_TaskSystem_run`（宏生成），需要改为手动注册：

```rust
// 替换:
// natives.register_static(NATIVE_TASK_SYSTEM_RUN, __shim_TaskSystem_run);
// 改为:
// natives.register_static(NATIVE_TASK_SYSTEM_RUN, shim_task_system_run);
```

**Step 3: 运行测试**

Run: `rtk cargo test -p auto-lang --lib`
Expected: 通过

**Step 4: Commit**

```
feat(vm): implement TaskSystem.run with tokio runtime bridge
```

---

#### Task 3.2: Async FFI shim 模式

**Goal:** 为 future Plan（如 async HTTP）提供在 FFI 内执行异步操作的模板。

**Step 1: 实现 async_block_on helper**

```rust
/// 在 FFI shim 中执行异步操作的 helper
/// 创建独立的 tokio runtime 并 block_on 执行
fn ffi_async_block_on<F, T>(f: F) -> Result<T, VMError>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| VMError::RuntimeError(format!("Failed to create runtime: {}", e)))?;
    rt.block_on(f).map_err(|e| VMError::RuntimeError(e))
}
```

**Step 2: 示例 — async HTTP shim**

```rust
/// 异步 HTTP GET（使用 reqwest async）
pub fn shim_http_async_get(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let url: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let result = ffi_async_block_on(async {
        let response = reqwest::get(&url).await
            .map_err(|e| format!("HTTP GET failed: {}", e))?;
        let status = response.status().as_u16();
        let body = response.text().await
            .map_err(|e| format!("Read body failed: {}", e))?;

        let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let resp_data = HttpResponseData {
            status,
            headers: vec![],
            body: body.into_bytes(),
        };
        HTTP_RESPONSES.with(|r| r.borrow_mut().insert(handle, resp_data));
        Ok(handle as i64)
    })?;

    task.ram.push_i64(result);
    Ok(())
}
```

**Step 3: Commit**

```
feat(vm): add async FFI helper pattern with tokio block_on
```

---

## File Changes Summary

```
Modified:
├── crates/auto-lang/src/vm/engine.rs
│   ├── 新增 FrameResult enum
│   ├── 提取 execute_single_frame() 方法
│   ├── 实现 handle_await_future() 方法
│   └── 修改 execute() 调用新方法
│
├── crates/auto-lang/src/vm/scheduler.rs
│   ├── 修改 execute_handler_fully() 复用 execute_single_frame
│   └── 更新签名接收 &AutoVM
│
└── crates/auto-lang/src/vm/ffi/stdlib.rs
    ├── shim_task_system_run: #[rust_fn] → 手动 shim
    ├── 新增 ffi_async_block_on() helper
    └── 更新 register_stdlib_ffi 注册
```

---

## Timeline

| Phase | Task | Estimated Effort | Status |
|-------|------|------------------|--------|
| 1.1 | 提取 execute_single_frame | 3-4h | Ready |
| 1.2 | scheduler 复用 VM 执行引擎 | 1-2h | Ready |
| 2.1 | 实现 handle_await_future | 2-3h | Ready |
| 3.1 | TaskSystem.run tokio 桥接 | 2-3h | Ready |
| 3.2 | Async FFI shim 模式 | 1h | Ready |
| **Total** | | **9-13h** | 可立即开始 |

---

## Success Criteria

### Phase 1 验证
- [ ] `execute_single_frame` 提取后，所有现有 VM 测试通过
- [ ] `execute_handler_fully` 能执行包含 NATIVE_CALL 的 handler

### Phase 2 验证
- [ ] `AWAIT_FUTURE` 遇到 Pending 时真正执行 body 字节码
- [ ] 嵌套 await（future 内部 await 另一个 future）正常工作
- [ ] Future 结果支持 Int、String、Nil 类型

### Phase 3 验证
- [ ] `TaskSystem.run(~{ print("hello") })` 实际执行 async body
- [ ] async FFI helper 能执行 reqwest 异步请求
- [ ] `ffi_async_block_on` 在 TaskSystem.run 内部不会死锁

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| execute_single_frame 提取引入 bug | High | Phase 1 只做重构，不改变行为；所有现有测试必须通过 |
| 递归执行 async body 栈溢出 | Medium | 设置递归深度限制（如 64 层），超出报错 |
| block_on 在已有 tokio runtime 中死锁 | Medium | TaskSystem.run 每次创建新 runtime；文档标注限制 |
| 闭包环境复制不完整 | Medium | Phase 2 先只支持无捕获的简单 async body |
| `#[rust_fn]` 宏迁移影响其他函数 | Low | 只改 TaskSystem.run，其他函数保持不变 |

---

## References

- [Plan 124](old/124-async-future-await.md) — ~T + .await + TaskSystem.run 设计
- [Plan 127](old/127-autovm-task-system-execution.md) — scheduler + message dispatch
- [Plan 195](195-http-client-async-unification.md) — HTTP async 需求（Phase 3.2 消费者）

## 关联 Plan

- **Plan 195** Phase 3.2 (Async HTTP) — 完成本 Plan 后解除阻塞
- **Plan 124** — 本 Plan 完成其未实现的 VM runtime 部分
