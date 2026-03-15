# Plan 124: 异步 Future/Await 系统

## Status: ✅ COMPLETED (Phase 2.1-2.3)

## 实现总结

### Phase 2.1: `~T` + `.await` + `TaskSystem.run` ✅
- Lexer: `Tilde`, `Await` tokens
- Parser: `~Type` → `Future<T>`, `~{ stmts }` async block, `.await` suffix
- AST: `Expr::AsyncBlock`, `Expr::Await`
- VM: `Value::Future`, `CREATE_FUTURE`, `AWAIT_FUTURE`, `POLL_FUTURE` opcodes
- FFI: `TaskSystem.run()` (NATIVE_TASK_SYSTEM_RUN = 2306)
- a2rs: `~{}` → `async {}`, `.await` → `.await`

### Phase 2.2: `send().await` 背压挂起 ✅
- FFI: `TaskHandle.send_await()` (NATIVE_TASK_SEND_AWAIT = 2307)
- a2rs: `send_await(msg)` → `send(msg).await`

### Phase 2.3: `ask/reply` 双向 RPC ✅
- Lexer: `Reply` token
- Parser: `reply expr` 语句
- AST: `Stmt::Reply(Box<Expr>)`
- FFI: `TaskHandle.ask()` (NATIVE_TASK_ASK = 2308)
- a2rs: `reply expr` → `reply_tx.send(expr)`, `ask(msg)` mapping
- Codegen: `Stmt::Reply` 处理

## Objective

为 AutoLang 实现 Phase 2 异步并发系统，在 Plan 121（Phase 1 Task/Msg 基础）之上引入"时间魔法"，实现非阻塞的时间等待机制。

## 核心特性

| 特性 | 描述 | 语法示例 |
|------|------|----------|
| `~T` 类型 | 异步蓝图，表示未来产出 `T` | `~User` → `Future<User>` |
| `.await` | 挂起等待 Future 完成 | `expr.await` |
| `TaskSystem.run` | 同步桥接，在 main 中执行异步代码 | `TaskSystem.run(~{ ... })` |
| `send().await` | 背压挂起，信箱满时挂起而非报错 | `Task.send(msg).await` |
| `ask/reply` | 双向 RPC，隐式 oneshot 通道 | `Task.ask(msg).await` / `reply expr` |

## 范围

- ✅ AutoVM 运行时
- ✅ a2rs 转译器（Rust async/await）
- ⏸️ a2c 转译器（状态机生成）→ 后续单独 Plan

## 依赖

- Plan 121: Task/Msg 基础系统（已完成）

---

## 设计文档参考

- [docs/design/task-msg.md](../design/task-msg.md) - Phase 2 完整设计规范

---

## Part 1: 类型系统设计

### 1.1 `Future<T>` 类型表示

**前端语法**：`~T` 作为 `Future<T>` 的语法糖

**内部表示**：复用现有 `GenericInstance` 泛型基础设施

```rust
// ast/types.rs - 无需新增 Type 变体，复用 GenericInstance
// ~int  → GenericInstance { name: "Future", args: [Type::Int] }
// ~User → GenericInstance { name: "Future", args: [Type::User(...)] }
```

### 1.2 类型推导规则

| 表达式 | 类型推导 |
|--------|----------|
| `~{ return 42 }` | `Future<int>` |
| `~{ return user }` | `Future<User>` |
| `expr.await` | 从 `Future<T>` 解包为 `T` |
| `Task.ask(Msg)` | `Future<T>`，T 由 `reply` 表达式推导 |

### 1.3 语义约束

- `.await` 只能在异步上下文中使用（`~{}` 块或 `on` 块）
- `reply` 只能在 `on` 块中使用
- `ask` 只能对 Task handle 调用

---

## Part 2: Lexer 与 Parser 扩展

### 2.1 Lexer 新增 Token

```rust
// token.rs
pub enum TokenKind {
    // ... 现有 tokens
    Tilde,      // ~
    Await,      // .await (后缀操作符)
    Reply,      // reply 关键字
}
```

### 2.2 Parser 类型解析

```
类型解析规则：
  ~Type → GenericInstance { name: "Future", args: [Type] }

优先级：~ 高于 |（联合类型）

示例：
  ~int           → Future<int>
  ~User          → Future<User>
  ~List<int>     → Future<List<int>>
  ~fn() int      → Future<fn() int>
```

### 2.3 Parser 表达式解析

```rust
// ast/expr.rs 新增

/// 异步块: ~{ stmts }
Expr::AsyncBlock {
    body: Vec<Stmt>,
    return_type: Option<Type>,  // 类型推导后填充
}

/// Await 表达式: expr.await
Expr::Await {
    expr: Box<Expr>,
}

/// 方法调用扩展（支持链式）
// Task.send(msg).await → MethodCall(MethodCall(...), "await")
// Task.ask(msg).await  → MethodCall(MethodCall(...), "await")
```

### 2.4 Parser 语句解析

```rust
// ast/stmt.rs 新增

/// Reply 语句: reply expr
Stmt::Reply {
    expr: Expr,
}
```

### 2.5 语法校验（语义检查阶段）

```
校验规则：
1. .await 上下文检查
   - 允许：~{} 块内、on {} 块内
   - 禁止：普通函数、main 函数（除非在 TaskSystem.run 内）

2. reply 上下文检查
   - 允许：on {} 块内
   - 禁止：其他任何位置

3. ask 调用检查
   - 只能对 TaskHandle 类型调用
```

---

## Part 3: AutoVM 运行时架构

### 3.1 Future 运行时表示

```rust
// vm/value.rs 扩展

pub enum Value {
    // ... 现有类型
    Future(Arc<FutureData>),
}

/// Future 运行时数据
pub struct FutureData {
    /// Future 状态
    pub state: Mutex<FutureState>,
    /// 完成后的结果
    pub result: Mutex<Option<Value>>,
    /// 等待者唤醒器（用于 .await 挂起）
    pub wakers: Mutex<Vec<Waker>>,
}

pub enum FutureState {
    Pending,   // 等待中
    Ready,     // 已完成
    Failed,    // 失败
}
```

### 3.2 内置 FutureChannel 类型

```rust
// vm/ffi/future.rs（新文件）

/// 编译器内置类型：FutureSender<T>
/// 对应 tokio::sync::oneshot::Sender<T>
pub struct FutureSender {
    pub inner: oneshot::Sender<Value>,
}

/// 编译器内置类型：FutureReceiver<T>
/// 对应 tokio::sync::oneshot::Receiver<T>
pub struct FutureReceiver {
    pub inner: oneshot::Receiver<Value>,
}

/// 编译器内置函数：FutureChannel.new<T>()
/// 创建 oneshot 通道
pub fn future_channel_new<T>() -> (FutureSender, FutureReceiver);
```

### 3.3 Task 消息循环扩展

```rust
// vm/task_system.rs 扩展

impl TaskInstance {
    /// 执行 on 块，支持 .await 挂起
    pub async fn run_on_block(&mut self, msg: Value, vm: &AutoVM) -> Result<(), VMError> {
        // 匹配消息，执行对应的处理逻辑
        // 如果遇到 .await，当前 Task 挂起（交出 CPU）
        // 挂起时保存执行状态（栈帧、局部变量）
    }
}
```

### 3.4 AsyncBlock 执行

```rust
// vm/eval.rs 扩展

impl Evaluator {
    /// 执行异步块，返回 Future
    pub fn eval_async_block(&mut self, block: &AsyncBlock) -> Result<Value, VMError> {
        // 立即返回 Value::Future
        // Future 内部包含执行状态和代码引用
    }

    /// 执行 .await，等待 Future 完成
    pub fn eval_await(&mut self, future: Value) -> Result<Value, VMError> {
        // 如果 Future 已 Ready，直接返回结果
        // 如果 Future Pending，挂起当前执行流
    }
}
```

### 3.5 TaskSystem.run 实现

```rust
// vm/ffi/stdlib.rs 扩展

#[auto_macros::rust_fn("TaskSystem.run")]
pub fn task_system_run(future: Value) -> Result<Value, VMError> {
    // 创建单线程 Tokio runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // block_on 执行 Future
    rt.block_on(async {
        // 执行传入的异步块
        // 返回结果
    })
}
```

---

## Part 4: ask/reply 编译期重写

### 4.1 消息协议重写

**原理**：编译器在 Codegen 阶段自动为包含 `ask` 调用的消息注入 `FutureSender` 字段。

**原始代码**：
```auto
enum DBMsg {
    QueryUser(int)
}
```

**编译器内部重写**：
```auto
enum DBMsg {
    QueryUser(int, FutureSender<User>)
}
```

### 4.2 ask 调用展开

**原始代码**：
```auto
let user = DBManager.ask(QueryUser(1001)).await.?
```

**编译器展开为**：
```auto
let (reply_tx, reply_rx) = FutureChannel.new<User>()
DBManager.send(QueryUser(1001, reply_tx)).?
let user = reply_rx.await.?
```

### 4.3 reply 语句展开

**原始代码**：
```auto
on {
    QueryUser(id) => {
        let user_info = db_driver.find(id)
        reply user_info
    }
}
```

**编译器展开为**：
```auto
on {
    QueryUser(id, reply_tx) => {
        let user_info = db_driver.find(id)
        reply_tx.send(user_info)
    }
}
```

### 4.4 类型推导

**流程**：
1. 分析 `on` 块中 `reply` 语句的表达式类型
2. 确定该消息变体的 reply 类型 `T`
3. 更新消息协议，注入 `FutureSender<T>`
4. 推导 `ask` 返回类型为 `Future<T>`

---

## Part 5: a2rs 转译器设计

### 5.1 类型映射

| Auto | Rust |
|------|------|
| `~T` / `Future<T>` | `impl std::future::Future<Output = T>` |
| `FutureSender<T>` | `tokio::sync::oneshot::Sender<T>` |
| `FutureReceiver<T>` | `tokio::sync::oneshot::Receiver<T>` |

### 5.2 异步块映射

```auto
// Auto
~{
    let x = foo().await.?
    return x
}
```

```rust
// Rust
async {
    let x = foo().await?;
    x
}
```

### 5.3 Task 映射

```rust
// Task run 方法变为 async
pub async fn run(mut self) -> Result<(), AutoError> {
    // 生命周期钩子
    self.execute_start_hook();

    while let Some(msg) = self.mailbox.recv().await {
        self.handle_message(msg).await?;
    }

    self.execute_stop_hook();
    Ok(())
}
```

### 5.4 ask/reply 映射

```rust
// ask 调用展开
let (tx, rx) = tokio::sync::oneshot::channel();
task_tx.send(Msg::Query(id, tx)).await.map_err(|_| AutoError::MailboxFull)?;
let result = rx.await.map_err(|_| AutoError::BrokenPipe)?;

// reply 语句展开
if let Msg::Query(id, reply_tx) = msg {
    let user_info = db_driver.find(id);
    let _ = reply_tx.send(user_info);  // 忽略发送失败
}
```

### 5.5 TaskSystem.run 映射

```auto
// Auto
let config = TaskSystem.run(~{
    let json = http_get("config.json").await.?
    return parse_url(json)
}).?
```

```rust
// Rust
let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();
let config = rt.block_on(async {
    let json = http_get("config.json").await?;
    parse_url(json)
})?;
```

---

## Part 6: 实现阶段划分

### Phase 2.1: `~T` + `.await` + `TaskSystem.run`

**目标**：基础异步能力，同步桥接

| 模块 | 任务 | 文件 |
|------|------|------|
| Lexer | 新增 `Tilde`、`Await` token | `lexer.rs` |
| Parser | 解析 `~Type` → `Future<T>` | `parser.rs` |
| Parser | 解析 `~{}` 异步块 | `parser.rs` |
| Parser | 解析 `.await` 后缀 | `parser.rs` |
| AST | 新增 `Expr::AsyncBlock` | `ast/expr.rs` |
| AST | 新增 `Expr::Await` | `ast/expr.rs` |
| VM | `Value::Future` 类型 | `vm/value.rs` |
| VM | 异步块执行 | `vm/eval.rs` |
| VM | `.await` 挂起机制 | `vm/eval.rs` |
| FFI | `TaskSystem.run()` 实现 | `vm/ffi/stdlib.rs` |
| a2rs | 异步块 → `async {}` | `trans/rust.rs` |
| a2rs | `.await` 映射 | `trans/rust.rs` |
| a2rs | `TaskSystem.run` 映射 | `trans/rust.rs` |
| 测试 | 异步块执行测试 | `test/` |
| 测试 | await 等待测试 | `test/` |
| 测试 | TaskSystem.run 测试 | `test/` |

**验收标准**：
- ✅ `~{ ... }` 异步块能正确执行并返回 `Future<T>`
- ✅ `.await` 能挂起并等待 Future 完成
- ✅ `TaskSystem.run(~{ ... })` 能在 `main` 中同步执行异步代码
- ✅ 类型推导：`~int` → `Future<int>`，`expr.await` → `int`
- ✅ 语义检查：`.await` 在非异步上下文中报错

---

### Phase 2.2: `send(msg).await` 背压挂起

**目标**：信箱满时挂起发送方，而非报错

| 模块 | 任务 | 文件 |
|------|------|------|
| Parser | 解析 `send().await` 链式调用 | `parser.rs` |
| VM | `TaskHandle.send_await()` 方法 | `vm/task_system.rs` |
| VM | 挂起/恢复机制 | `vm/eval.rs` |
| FFI | `send().await` FFI 支持 | `vm/ffi/stdlib.rs` |
| a2rs | `tx.send(msg).await` 映射 | `trans/rust.rs` |
| 测试 | 信箱满时挂起测试 | `test/` |
| 测试 | 恢复后继续测试 | `test/` |

**验收标准**：
- ✅ `send(msg).await` 在信箱满时挂起发送方
- ✅ 信箱有空位时，挂起的 Task 自动恢复
- ✅ 对比 `send(msg).?`（信箱满报错）的行为差异

---

### Phase 2.3: `ask/reply` 双向 RPC

**目标**：隐式 oneshot 通道，编译期重写

| 模块 | 任务 | 文件 |
|------|------|------|
| Lexer | 新增 `Reply` token | `lexer.rs` |
| Parser | 解析 `reply expr` 语句 | `parser.rs` |
| Parser | 解析 `Task.ask(msg)` 调用 | `parser.rs` |
| AST | 新增 `Stmt::Reply` | `ast/stmt.rs` |
| Codegen | 消息协议重写（注入 FutureSender） | `codegen/` |
| Codegen | `ask` 调用展开 | `codegen/` |
| Codegen | `reply` 语句展开 | `codegen/` |
| VM | `FutureChannel` 内置类型 | `vm/ffi/future.rs` |
| VM | `FutureSender`/`FutureReceiver` | `vm/ffi/future.rs` |
| a2rs | `tokio::sync::oneshot` 映射 | `trans/rust.rs` |
| 测试 | 双向通信测试 | `test/` |
| 测试 | 管道断裂检测测试 | `test/` |

**验收标准**：
- ✅ `ask(msg).await` 能正确接收 `reply` 返回值
- ✅ `reply` 语句能将数据原路返回
- ✅ Task 崩溃时，等待中的 `ask` 能感知管道断裂并抛出错误
- ✅ 编译期类型推导：`ask` 返回类型与 `reply` 表达式类型一致

---

## Part 7: 测试用例

### 7.1 基础异步测试

```auto
// test/async_basic.at
fn main() ! {
    // 异步块
    let future = ~{
        let x = 1 + 2
        return x
    }

    // await 等待
    let result = future.await
    assert(result == 3)
}
```

### 7.2 TaskSystem.run 测试

```auto
// test/task_system_run.at
fn main() ! {
    let config = TaskSystem.run(~{
        // 模拟异步 I/O
        let data = read_file("config.json").await.?
        return parse_config(data)
    }).?

    print("Config loaded: ${config}")
    TaskSystem.start()
}
```

### 7.3 背压挂起测试

```auto
// test/backpressure.at
#[single]
#[mailbox(2, strict)]
task SlowTask {
    on {
        Msg(data) => {
            sleep(100)  // 慢处理
        }
    }
}

task Producer {
    on {
        Start => {
            for i in 0..10 {
                // 信箱满时挂起
                SlowTask.send(Msg(i)).await.?
            }
        }
    }
}
```

### 7.4 ask/reply 测试

```auto
// test/ask_reply.at
enum DBMsg {
    QueryUser(int)
}

#[single]
task DBManager {
    on {
        QueryUser(id) => {
            let user = db_find(id)
            reply user
        }
    }
}

task WebClient {
    on {
        Request => {
            let user = DBManager.ask(QueryUser(1001)).await.?
            print("User: ${user.name}")
        }
    }
}
```

### 7.5 管道断裂测试

```auto
// test/broken_pipe.at
#[single]
task CrashTask {
    on {
        Msg => {
            panic("intentional crash")  // 模拟崩溃
        }
    }
}

task Caller {
    on {
        Start => {
            let result = CrashTask.ask(Msg).await
            if result.is_error() {
                print("Detected broken pipe!")
            }
        }
    }
}
```

---

## Part 8: 风险与缓解

### 8.1 死锁风险

**风险**：多个 Task 互相 `ask` 可能导致死锁

**缓解**：
- 编译期检测循环依赖（如果可能）
- 运行时超时机制（可选）
- 文档中明确最佳实践

### 8.2 内存泄漏

**风险**：挂起的 Future 如果永远不被唤醒，会泄漏

**缓解**：
- Task 崩溃时清理相关 Future
- 提供超时 API（后续）

### 8.3 类型推导复杂度

**风险**：`ask` 返回类型推导需要跨 Task 分析

**缓解**：
- 分阶段处理：先收集 reply 类型，再更新消息协议
- 限制 reply 语句必须在 on 块内

---

## Part 9: 后续工作

### a2c 状态机生成（后续 Plan）

- 无栈协程状态机
- 局部变量提升
- RTOS oneshot 实现

### Phase 3 功能（未来）

- Task 监控
- 更复杂的调度策略
- 超时机制

---

## 变更历史

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-03-15 | v1.0 | 初始设计 |
