# Plan 121: AutoLang Task/Msg 基础系统

## Status: ✅ COMPLETED

### Implementation Summary

**完成日期**: 2026-03-15

**已实现功能**:
1. ✅ Lexer: Task/Spawn 关键字
2. ✅ AST: TaskDef, TaskOnBlock, TaskMsgPattern 结构
3. ✅ Parser: task 定义解析，spawn/send 方法调用
4. ✅ AST: Handle<T> 类型
5. ✅ AutoVM: TaskRegistry, TaskHandle, TaskInstance
6. ✅ FFI: spawn(), send(), TaskSystem.start()
7. ✅ 生命周期钩子: start(), stop() (LIFO 顺序)
8. ✅ Codegen: TaskDef 语句编译
9. ✅ 集成测试

**测试覆盖**:
- 10 AST task tests
- 25 task_system tests
- 8 FFI task tests
- 3 codegen task tests

## Objective

实现 AutoLang 的 Actor 模型基础设施，建立 Task（任务实体）和 Msg（消息协议）的核心机制，用 Tokio 作为底层运行时。

## 核心概念

- **Task**：独立的并发实体，拥有私有状态和消息信箱
- **Msg**：Task 之间通信的强类型消息协议
- **Handle**：Task 实例的引用句柄，可复制、传递、用于发送消息

## Phase 1 范围

本阶段专注于**让多任务和消息发布机制能运行起来**，不涉及：
- `~T` (Future) 类型
- `.await` / `.go` 异步操作
- `ask(msg)` 请求-响应模式
- 可配置信箱策略

### 后续阶段预览

- **Phase 2**：`~T` 类型、`.await`、`ask()`、`#[mailbox]` 标注
- **Phase 3**：更复杂的调度策略、Task 监控

---

## 语法设计

### Task 定义

```auto
// 定义消息协议（强类型枚举）
enum CounterMsg {
    Add(int),
    Reset,
    Print
}

// 多实例 Task（默认）
task CounterTask {
    // 私有状态
    count mut = 0

    // 生命周期钩子
    fn start() ! {
        self.count = 0
    }

    fn stop() ! {
        print("CounterTask stopping, final count: ${self.count}")
    }

    // 消息处理
    on {
        Add(val) => { self.count += val }
        Reset => { self.count = 0 }
        Print => { print("Count: ${self.count}") }
        else => { print("Unknown message") }
    }
}

// 单例 Task
#[single]
task Logger {
    on {
        Log(msg) => { print("[LOG] ${msg}") }
        else => { }
    }
}
```

### Task 创建与通信

```auto
fn main() ! {
    // 多实例：spawn 创建，返回 Handle
    let h1 = CounterTask.spawn()
    let h2 = CounterTask.spawn()

    // 通过 Handle 发送
    h1.send(Add(10))
    h2.send(Add(20))

    // 单例：直接用名称发送
    Logger.send(Log("System started"))

    // 启动调度器
    TaskSystem.start()
}
```

---

## 类型系统

### Handle 类型

每个 Task 类型自动关联一个 `Handle<T>` 类型：

```auto
task CounterTask { ... }

// CounterTask.spawn() 返回 Handle<CounterTask>
let h: Handle<CounterTask> = CounterTask.spawn()

// Handle 可以复制
let h_copy = h

// Handle 可以作为参数传递
fn process(handle Handle<CounterTask>) {
    handle.send(Add(1))
}
```

### Handle 的语义

- **可复制**：`let h2 = h1` 两者指向同一实例
- **可比较**：`h1 == h2` 判断是否指向同一实例
- **可传递**：可作为函数参数、存储在结构体中、发送给其他 Task

### Handle 底层结构

在 AutoVM 中：
```rust
pub struct TaskHandle {
    task_type: String,   // "CounterTask"
    instance_id: u64,    // 唯一的取件码
    tx: Arc<mpsc::Sender<VmValue>>,
}
```

在 a2rs 中：直接映射到 `mpsc::Sender<T>`

### 单例 Task 的特殊性

```auto
#[single]
task Logger { ... }

// 单例没有 Handle 类型，直接用名称引用
Logger.send(Log("msg"))  // ✅ 正确

// 单例不能 spawn
let h = Logger.spawn()   // ❌ 编译错误
```

---

## 生命周期管理

### TaskSystem.start() 行为

```auto
fn main() ! {
    // 阶段 A：创建 Task 实例，预发消息
    let h1 = CounterTask.spawn()
    let h2 = CounterTask.spawn()

    h1.send(Add(10))
    h2.send(Add(20))

    // 阶段 B：启动调度器
    TaskSystem.start()

    // 此行之后的代码不可达
    // print("Never executed")
}
```

**`TaskSystem.start()` 语义**：
1. 阻塞主线程
2. 启动所有已创建的 Task 实例（调用其 `start()` 钩子）
3. 每个 Task 进入消息循环（从信箱读取消息，调用 `on` 处理）
4. 等待 `Ctrl+C` 信号
5. 收到信号后，按**反向启动顺序**（LIFO）调用各 Task 的 `stop()`
6. `stop()` 中的错误被记录，不影响其他 Task 的停止
7. 所有 Task 停止后，程序退出

### 编译器检查

- `main` 函数中必须调用且仅调用一次 `TaskSystem.start()`
- `TaskSystem.start()` 之后的代码标记为不可达（警告或错误）

---

## 信箱机制

### Phase 1 默认行为

- **容量**：固定 64 条消息
- **满员策略**：Strict 模式，直接报错

```auto
fn main() ! {
    let h = CounterTask.spawn()

    // 正常发送
    for i in 0..64 {
        h.send(Add(i))  // ✅ 成功入队
    }

    // 第 65 条消息触发错误
    h.send(Add(999))   // ❌ 运行时错误：邮箱已满
}
```

### 错误类型

```
RuntimeError: MailboxFull
  Task: CounterTask#1
  Message: Add(999)
  Capacity: 64
```

### Phase 2 预告

通过 `#[mailbox]` 标注配置：

```auto
#[mailbox(size = 128, policy = DropOldest)]
task CounterTask { ... }
```

| 策略 | 行为 |
|------|------|
| `Strict` (默认) | 满员报错 |
| `DropOldest` | 踢掉最旧消息 |
| `Block` | 阻塞等待空间 |

---

## 编译器实现 - AST 扩展

### 新增 AST 节点

```rust
// Task 定义
pub enum Stmt {
    // ... 现有变体
    Task {
        name: Name,
        is_single: bool,        // #[single] 标注
        fields: Vec<Field>,     // 私有状态
        start_hook: Option<FnDef>,
        stop_hook: Option<FnDef>,
        on_block: OnBlock,
    },
}

// 消息处理块
pub struct OnBlock {
    pub arms: Vec<OnArm>,
    pub else_arm: Option<Block>,
}

pub struct OnArm {
    pub pattern: Pattern,  // Add(val), Reset, Print 等
    pub body: Block,
}

// Handle 类型
pub enum Type {
    // ... 现有变体
    Handle { task_type: Name },  // Handle<CounterTask>
}

// spawn 表达式
pub enum Expr {
    // ... 现有变体
    Spawn { task: Name, args: Vec<Expr> },  // CounterTask.spawn()
}

// send 表达式
pub enum Expr {
    // ... 现有变体
    Send { target: SendTarget, msg: Expr },  // h.send(Add(1))
}

pub enum SendTarget {
    Handle(Expr),  // h.send(...)
    Single(Name),  // Logger.send(...)
}
```

---

## 编译器实现 - Lexer/Parser

### 新增关键字

| 关键字 | 用途 |
|--------|------|
| `task` | Task 定义 |
| `spawn` | 创建 Task 实例 |

### 新增标注

| 标注 | 用途 |
|------|------|
| `#[single]` | 单例 Task |
| `#[mailbox]` | 信箱配置（Phase 2） |

### Parser 规则（伪代码）

```
task_def ::= 'task' name '{' task_body '}'
task_body ::= (field | start_hook | stop_hook | on_block)*

start_hook ::= 'fn' 'start' '(' ')' '!' block
stop_hook ::= 'fn' 'stop' '(' ')' '!' block

on_block ::= 'on' '{' on_arm* (else_arm)? '}'
on_arm ::= pattern '=>' block
else_arm ::= 'else' '=>' block

spawn_expr ::= name '.' 'spawn' '(' args? ')'
send_expr ::= expr '.' 'send' '(' expr ')'
```

**注意**：`on` 关键字已有实现，具体实现时需分析现有代码复用。

### 类型解析

- `Handle<CounterTask>` 作为泛型类型解析
- `TaskSystem.start()` 作为静态方法调用解析

---

## AutoVM 运行时架构

### 核心数据结构

```rust
// VM 状态管理器
pub struct VmState {
    tasks: HashMap<u64, TaskInstance>,
    task_handles: HashMap<String, mpsc::Sender<VmValue>>,  // 单例 Task
    next_instance_id: AtomicU64,
}

// Task 实例
pub struct TaskInstance {
    task_type: String,
    instance_id: u64,
    state: HashMap<String, VmValue>,  // 私有状态
    mailbox_rx: mpsc::Receiver<VmValue>,
    mailbox_tx: mpsc::Sender<VmValue>,  // 用于创建 Handle
}

// Handle 结构
pub struct TaskHandle {
    task_type: String,
    instance_id: u64,
    tx: Arc<mpsc::Sender<VmValue>>,
}
```

### TaskSystem.start() 实现

```rust
impl VmState {
    pub fn start_scheduler(&mut self) {
        let rt = tokio::runtime::Runtime::new().unwrap();

        rt.block_on(async {
            // 1. 按 spawn 顺序调用 start() 钩子
            for (_, task) in &self.tasks {
                task.call_start_hook();
            }

            // 2. 启动消息循环
            let handles: Vec<_> = self.tasks.iter()
                .map(|(_, task)| tokio::spawn(task.eval_loop()))
                .collect();

            // 3. 等待 Ctrl+C
            tokio::signal::ctrl_c().await.ok();

            // 4. 按 LIFO 顺序调用 stop() 钩子
            for (_, task) in self.tasks.iter().rev() {
                if let Err(e) = task.call_stop_hook() {
                    eprintln!("Task stop error: {}", e);
                }
            }
        });
    }
}
```

---

## a2rs (Rust Transpiler) 映射

### Task 映射为 Rust 结构

**AutoLang 源码**:
```auto
enum CounterMsg {
    Add(int),
    Reset,
}

task CounterTask {
    count mut = 0

    fn start() ! { self.count = 0 }
    fn stop() ! { }

    on {
        Add(val) => { self.count += val }
        Reset => { self.count = 0 }
        else => { }
    }
}
```

**生成的 Rust 代码**:
```rust
pub enum CounterMsg {
    Add(i32),
    Reset,
}

pub struct CounterTask {
    count: i32,
    mailbox_rx: mpsc::Receiver<CounterMsg>,
}

impl CounterTask {
    pub async fn run(mut self) -> Result<(), String> {
        // start() 钩子
        self.count = 0;

        // 消息循环
        while let Some(msg) = self.mailbox_rx.recv().await {
            match msg {
                CounterMsg::Add(val) => { self.count += val; }
                CounterMsg::Reset => { self.count = 0; }
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        // stop() 钩子
        Ok(())
    }
}
```

### Handle 映射

```rust
// Handle<CounterTask> 映射为
pub type CounterTaskHandle = Arc<mpsc::Sender<CounterMsg>>;

// CounterTask.spawn() 映射为
fn spawn_counter_task() -> CounterTaskHandle {
    let (tx, rx) = mpsc::channel(64);
    let task = CounterTask { count: 0, mailbox_rx: rx };
    tokio::spawn(async move { task.run().await });
    Arc::new(tx)
}
```

---

## 实现计划

### Phase 1A: AST + Parser (2 天)

**修改文件**:
- `crates/auto-lang/src/ast.rs` - 新增 Task、OnBlock、Handle 等 AST 节点
- `crates/auto-lang/src/lexer.rs` - 新增 `task`、`spawn` 关键字
- `crates/auto-lang/src/parser.rs` - 解析 Task 定义、spawn/send 表达式

**任务**:
- [ ] `Stmt::Task` AST 节点
- [ ] `Type::Handle` AST 节点
- [ ] `Expr::Spawn` 和 `Expr::Send` AST 节点
- [ ] `OnBlock` 和 `OnArm` 结构
- [ ] 解析 `task Name { ... }` 语法
- [ ] 解析 `#[single]` 标注
- [ ] 解析 `Name.spawn()` 表达式
- [ ] 解析 `handle.send(msg)` 表达式
- [ ] 解析 `TaskSystem.start()` 调用

### Phase 1B: AutoVM 运行时 (3 天)

**修改文件**:
- `crates/auto-lang/src/vm/task.rs` - 新增 TaskInstance、TaskHandle
- `crates/auto-lang/src/vm/state.rs` - 扩展 VmState 支持 Task 管理
- `crates/auto-lang/src/vm/scheduler.rs` - 新增 TaskSystem.start() 实现

**任务**:
- [ ] TaskInstance 结构体
- [ ] TaskHandle 结构体（类型名 + 实例ID + Sender）
- [ ] VmState 扩展：tasks 注册表、单例 tasks
- [ ] `spawn` 执行：创建 Task 实例、分配信箱、返回 Handle
- [ ] `send` 执行：通过 Handle/单例名发送消息
- [ ] 消息循环：从信箱读取、匹配 `on` 分支、执行
- [ ] `TaskSystem.start()` 实现：启动、阻塞、Ctrl+C、LIFO 停止

### Phase 1C: a2rs Transpiler (3 天)

**修改文件**:
- `crates/auto-lang/src/trans/rust.rs` - Task/Msg 代码生成

**任务**:
- [ ] 生成 `enum MsgName { ... }` Rust 枚举
- [ ] 生成 `struct TaskName { ... }` 结构体
- [ ] 生成 `impl TaskName { async fn run() }` 消息循环
- [ ] 生成 `TaskNameHandle` 类型别名（`Arc<mpsc::Sender>`）
- [ ] 生成 `spawn_taskname()` 函数
- [ ] 生成 `main` 函数框架（Tokio runtime 初始化）
- [ ] 生成 `TaskSystem.start()` 对应的阻塞代码

### Phase 1D: 测试与集成 (2 天)

**测试文件**:
- `crates/auto-lang/test/vm/task_tests.rs` - VM 层 Task 测试
- `crates/auto-lang/test/a2r/task/` - a2rs Task 测试用例

**任务**:
- [ ] VM 层单元测试：Task 创建、send/send、消息循环
- [ ] VM 层集成测试：多 Task 协作、生命周期钩子
- [ ] a2rs 测试用例：基础 Task 定义、spawn、send
- [ ] a2rs 测试用例：单例 Task
- [ ] a2rs 测试用例：生命周期钩子
- [ ] 错误场景测试：邮箱满员报错
- [ ] 文档更新

### 总计：10 天

| 阶段 | 天数 | 内容 |
|------|------|------|
| 1A | 2 | AST + Parser |
| 1B | 3 | AutoVM 运行时 |
| 1C | 3 | a2rs Transpiler |
| 1D | 2 | 测试与集成 |

---

## 验收标准

### 功能验收

- [ ] `task TaskName { ... }` 语法定义正确解析
- [ ] `enum MsgName { ... }` 消息协议正确解析
- [ ] `on { ... }` 消息处理块正确执行
- [ ] `#[single]` 单例 Task 只允许一个实例
- [ ] `TaskName.spawn()` 返回 `Handle<TaskName>`
- [ ] `handle.send(msg)` 消息正确送达
- [ ] `TaskName.send(msg)` 单例发送正常工作
- [ ] `fn start()` 钩子在 Task 启动时调用
- [ ] `fn stop()` 钩子在系统关闭时按 LIFO 顺序调用
- [ ] `TaskSystem.start()` 阻塞直到 Ctrl+C
- [ ] 信箱满员时抛出运行时错误
- [ ] Handle 可复制、可比较、可传递

### 编译器检查

- [ ] 单例 Task 调用 `spawn()` 报编译错误
- [ ] `main` 函数必须调用 `TaskSystem.start()`
- [ ] `TaskSystem.start()` 后的代码标记为不可达

---

## 已知风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Tokio runtime 与 VM 集成复杂 | 中 | 先实现最小可用版本，逐步优化 |
| 消息循环与现有 VM 执行模型冲突 | 中 | 独立 Task 执行上下文 |
| LIFO 停止顺序的依赖跟踪 | 低 | 简单的 Vec 记录 spawn 顺序即可 |

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.x | Async runtime |
| tokio::sync::mpsc | 1.x | Message channels |

---

## References

- [设计文档](../design/task-msg.md) - Task/Msg 系统设计规范

---

# 详细实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 AutoLang Actor 模型的 Task/Msg 基础系统，支持 Task 定义、spawn 创建、send 通信、生命周期钩子和 TaskSystem.start() 调度器。

**Architecture:** 采用分层架构：Lexer/Parser 解析语法 → AST 表示 → AutoVM 运行时执行 / a2rs Transpiler 生成 Rust 代码。Task 使用 Tokio mpsc channel 作为信箱，Handle 作为任务引用。

**Tech Stack:** Rust, Tokio (mpsc channel, signal), DashMap, Arc<RwLock>

---

## Task 1: Lexer - 新增关键字

**Files:**
- Modify: `crates/auto-lang/src/token.rs:75-130`
- Modify: `crates/auto-lang/src/lexer.rs:keyword识别区域`

**Step 1: 在 TokenKind 中添加新关键字**

在 `crates/auto-lang/src/token.rs` 的 `TokenKind` enum 中添加：

```rust
// 在 TokenKind enum 中，在现有关键字后面添加
    // ... 现有关键字

    // Plan 121: Task/Msg system keywords
    Task,     // task 关键字
    Spawn,    // spawn 方法名（保留）

    // ... 其他
}
```

**Step 2: 在 lexer.rs 中添加关键字识别**

在 `crates/auto-lang/src/lexer.rs` 的关键字匹配函数中添加：

```rust
// 在关键字匹配表中添加
"task" => TokenKind::Task,
"spawn" => TokenKind::Spawn,
```

**Step 3: 运行测试验证**

```bash
cargo test -p auto-lang lexer
```

Expected: 所有 lexer 测试通过

**Step 4: Commit**

```bash
git add crates/auto-lang/src/token.rs crates/auto-lang/src/lexer.rs
git commit -m "feat(lexer): add Task and Spawn keywords for Plan 121"
```

---

## Task 2: AST - 新增 Task 节点

**Files:**
- Create: `crates/auto-lang/src/ast/task.rs`
- Modify: `crates/auto-lang/src/ast.rs` (添加 mod 和 re-export)

**Step 1: 创建 Task AST 结构**

创建文件 `crates/auto-lang/src/ast/task.rs`：

```rust
use super::{Body, Fn, Member, Name, ToAtom, ToAtomStr, ToNode, AtomWriter};
use auto_val::{AutoStr, Node as AutoNode, Value};
use std::{fmt, io as stdio};

/// Plan 121: Task 定义
/// 表示一个 Actor 实体，拥有私有状态和消息处理能力
#[derive(Debug, Clone)]
pub struct TaskDef {
    /// Task 类型名称
    pub name: Name,
    /// 是否为单例 Task（#[single] 标注）
    pub is_single: bool,
    /// 关联的消息类型名称（如 "CounterMsg"）
    pub msg_type: Option<Name>,
    /// 私有状态字段
    pub fields: Vec<Member>,
    /// start() 生命周期钩子
    pub start_hook: Option<Fn>,
    /// stop() 生命周期钩子
    pub stop_hook: Option<Fn>,
    /// 消息处理块
    pub on_block: TaskOnBlock,
}

/// Task 的消息处理块
#[derive(Debug, Clone)]
pub struct TaskOnBlock {
    /// 消息匹配分支
    pub arms: Vec<TaskOnArm>,
    /// else 兜底分支
    pub else_arm: Option<Body>,
}

/// 单个消息匹配分支
#[derive(Debug, Clone)]
pub struct TaskOnArm {
    /// 消息模式（如 "Add(val)" 或 "Reset"）
    pub pattern: TaskPattern,
    /// 处理体
    pub body: Body,
}

/// 消息模式
#[derive(Debug, Clone)]
pub enum TaskPattern {
    /// 带数据的消息变体，如 Add(val)
    Variant { name: Name, bindings: Vec<Name> },
    /// 无数据消息，如 Reset
    Unit(Name),
}

// Display implementations
impl fmt::Display for TaskDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(task {}", self.name)?;
        if self.is_single {
            write!(f, " #[single]")?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for TaskOnBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(on-block {} arms)", self.arms.len())
    }
}

impl fmt::Display for TaskOnArm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(on-arm {})", self.pattern)
    }
}

impl fmt::Display for TaskPattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TaskPattern::Variant { name, bindings } => {
                write!(f, "{}(", name)?;
                for (i, b) in bindings.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", b)?;
                }
                write!(f, ")")
            }
            TaskPattern::Unit(name) => write!(f, "{}", name),
        }
    }
}

// ToAtom implementations
impl AtomWriter for TaskDef {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        if self.is_single {
            write!(f, "#[single] ")?;
        }
        write!(f, "task {} {{", self.name)?;

        // Fields
        for field in &self.fields {
            write!(f, " {} mut = {}", field.name, field.ty.to_atom_str())?;
        }

        // Hooks
        if let Some(start) = &self.start_hook {
            write!(f, " fn start() ! {{ ... }}")?;
        }
        if let Some(stop) = &self.stop_hook {
            write!(f, " fn stop() ! {{ ... }}")?;
        }

        // On block
        write!(f, " on {{ ... }}")?;

        write!(f, " }}")?;
        Ok(())
    }
}

impl ToAtom for TaskDef {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}

impl ToNode for TaskDef {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("task");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("is_single", Value::Bool(self.is_single));
        if let Some(msg_type) = &self.msg_type {
            node.set_prop("msg_type", Value::str(msg_type.as_str()));
        }
        for field in &self.fields {
            node.add_kid(field.to_node());
        }
        node
    }
}

impl AtomWriter for TaskOnBlock {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        write!(f, "on {{")?;
        for arm in &self.arms {
            write!(f, " {} => {{ ... }}", arm.pattern)?;
        }
        if let Some(else_arm) = &self.else_arm {
            write!(f, " else => {{ ... }}")?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

impl ToAtom for TaskOnBlock {
    fn to_atom(&self) -> AutoStr {
        self.to_atom_str()
    }
}
```

**Step 2: 在 ast.rs 中添加 mod 和 re-export**

在 `crates/auto-lang/src/ast.rs` 中添加：

```rust
mod task;
pub use task::*;
```

同时修改 `Stmt` enum 添加 Task 变体：

```rust
pub enum Stmt {
    // ... 现有变体
    Task(TaskDef),  // Plan 121: Task 定义
    TaskOn(TaskOnBlock),  // Plan 121: Task 的 on 块（独立语句）
}
```

**Step 3: 运行编译验证**

```bash
cargo build -p auto-lang 2>&1 | head -50
```

Expected: 编译成功，可能有未使用的警告

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ast/task.rs crates/auto-lang/src/ast.rs
git commit -m "feat(ast): add Task AST nodes for Plan 121"
```

---

## Task 3: AST - 新增 Handle 类型

**Files:**
- Modify: `crates/auto-lang/src/ast/types.rs:8-50`

**Step 1: 在 Type enum 中添加 Handle 变体**

在 `crates/auto-lang/src/ast/types.rs` 的 `Type` enum 中添加：

```rust
#[derive(Debug, Clone)]
pub enum Type {
    // ... 现有变体

    // Plan 121: Task Handle 类型
    Handle { task_type: Name },  // Handle<CounterTask>
}
```

**Step 2: 更新 Type 的 trait 实现**

在 `unique_name()` 方法中添加：

```rust
pub fn unique_name(&self) -> AutoStr {
    match self {
        // ... 现有分支
        Type::Handle { task_type } => format!("Handle<{}>", task_type).into(),
    }
}
```

在 `Display` 实现中添加：

```rust
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // ... 现有分支
            Type::Handle { task_type } => write!(f, "Handle<{}>", task_type),
        }
    }
}
```

在 `write_atom()` 方法中添加：

```rust
impl AtomWriter for Type {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self {
            // ... 现有分支
            Type::Handle { task_type } => {
                write!(f, "Handle<{}>", task_type)?;
            }
        }
        Ok(())
    }
}
```

**Step 3: 运行测试验证**

```bash
cargo test -p auto-lang -- types
```

Expected: 所有类型相关测试通过

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ast/types.rs
git commit -m "feat(ast): add Handle<T> type for Plan 121"
```

---

## Task 4: AST - 新增 Spawn 和 Send 表达式

**Files:**
- 查找并修改 Expr enum 定义文件

**Step 1: 找到 Expr 定义位置并添加变体**

首先确定 Expr 的定义位置：

```bash
grep -n "pub enum Expr" crates/auto-lang/src/ast/*.rs
```

在 Expr enum 中添加：

```rust
pub enum Expr {
    // ... 现有变体

    // Plan 121: Task spawn 表达式
    Spawn {
        task: Name,           // Task 类型名
        args: Vec<Expr>,      // 初始化参数（字段赋值）
    },

    // Plan 121: 消息发送表达式
    Send {
        target: SendTarget,   // 发送目标
        msg: Expr,            // 消息值
    },
}

/// Plan 121: 消息发送目标
#[derive(Debug, Clone)]
pub enum SendTarget {
    /// Handle 变量（多实例 Task）
    Handle(Box<Expr>),
    /// 单例 Task 名称
    Single(Name),
}
```

**Step 2: 添加 Display 和 ToAtom 实现**

```rust
impl fmt::Display for SendTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SendTarget::Handle(expr) => write!(f, "{}", expr),
            SendTarget::Single(name) => write!(f, "{}", name),
        }
    }
}

impl AtomWriter for SendTarget {
    fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
        match self {
            SendTarget::Handle(expr) => write!(f, "{}", expr.to_atom_str()),
            SendTarget::Single(name) => write!(f, "{}", name),
        }
        Ok(())
    }
}
```

**Step 3: 运行编译验证**

```bash
cargo build -p auto-lang 2>&1 | head -50
```

**Step 4: Commit**

```bash
git add crates/auto-lang/src/ast/*.rs
git commit -m "feat(ast): add Spawn and Send expressions for Plan 121"
```

---

## Task 5: Parser - 解析 Task 定义

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`
- Modify: `crates/auto-lang/src/parser_helpers.rs` (如需要)

**Step 1: 添加 parse_task_def 方法**

在 parser.rs 中添加 Task 解析方法：

```rust
impl Parser {
    /// Plan 121: 解析 Task 定义
    /// task Name { fields? start_hook? stop_hook? on_block? }
    pub fn parse_task_def(&mut self, annotations: Vec<Annotation>) -> AutoResult<Stmt> {
        // 1. 检查 #[single] 标注
        let is_single = annotations.iter().any(|a| a.name == "single");

        // 2. 消费 'task' 关键字
        self.expect(TokenKind::Task)?;

        // 3. 解析 Task 名称
        let name = self.expect_ident()?;

        // 4. 消费 '{'
        self.expect(TokenKind::LBrace)?;

        // 5. 解析 Task body
        let mut fields = Vec::new();
        let mut start_hook = None;
        let mut stop_hook = None;
        let mut on_block = TaskOnBlock { arms: Vec::new(), else_arm: None };

        while !self.is_kind(TokenKind::RBrace) {
            match self.cur.kind {
                // 字段定义: name mut = value
                TokenKind::Ident if self.peek_kind(TokenKind::Mut) => {
                    fields.push(self.parse_task_field()?);
                }
                // start/stop 钩子: fn start() ! { ... }
                TokenKind::Fn => {
                    let fn_name = self.peek_name()?;
                    if fn_name == "start" {
                        start_hook = Some(self.parse_task_hook("start")?);
                    } else if fn_name == "stop" {
                        stop_hook = Some(self.parse_task_hook("stop")?);
                    } else {
                        return Err(SyntaxError::Generic {
                            message: format!("Task only allows 'start' or 'stop' hooks, got '{}'", fn_name),
                            span: pos_to_span(self.cur.pos),
                        }.into());
                    }
                }
                // on 块: on { ... }
                TokenKind::On => {
                    on_block = self.parse_task_on_block()?;
                }
                _ => {
                    return Err(SyntaxError::UnexpectedToken {
                        expected: "field, fn start/stop, or on".to_string(),
                        found: self.cur.text.to_string(),
                        span: pos_to_span(self.cur.pos),
                    }.into());
                }
            }
        }

        // 6. 消费 '}'
        self.expect(TokenKind::RBrace)?;

        Ok(Stmt::Task(TaskDef {
            name,
            is_single,
            msg_type: None, // 将在类型检查阶段推断
            fields,
            start_hook,
            stop_hook,
            on_block,
        }))
    }

    /// 解析 Task 字段: name mut = value
    fn parse_task_field(&mut self) -> AutoResult<Member> {
        let name = self.expect_ident()?;
        self.expect(TokenKind::Mut)?;
        self.expect(TokenKind::Asn)?;
        let value = self.parse_expr()?;
        // 类型从初始值推断
        Ok(Member::new(name, Type::Unknown, Some(value)))
    }

    /// 解析 Task 钩子: fn start() ! { ... }
    fn parse_task_hook(&mut self, hook_name: &str) -> AutoResult<Fn> {
        self.expect(TokenKind::Fn)?;
        let name = self.expect_ident()?;
        assert_eq!(&*name, hook_name, "Expected {} hook", hook_name);

        self.expect(TokenKind::LParen)?;
        self.expect(TokenKind::RParen)?;
        self.expect(TokenKind::Not)?;  // ! 表示错误传播

        let body = self.parse_block()?;

        Ok(Fn::new(
            FnKind::Function,
            name,
            None,  // no parent
            Vec::new(),  // no params
            body,
            Type::Void,
        ))
    }

    /// 解析 Task 的 on 块
    fn parse_task_on_block(&mut self) -> AutoResult<TaskOnBlock> {
        self.expect(TokenKind::On)?;
        self.expect(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        let mut else_arm = None;

        while !self.is_kind(TokenKind::RBrace) {
            if self.is_kind(TokenKind::Else) {
                // else => { ... }
                self.next();
                self.expect(TokenKind::DoubleArrow)?;
                else_arm = Some(self.parse_block()?);
            } else {
                // Pattern => { ... }
                let pattern = self.parse_task_pattern()?;
                self.expect(TokenKind::DoubleArrow)?;
                let body = self.parse_block()?;
                arms.push(TaskOnArm { pattern, body });
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(TaskOnBlock { arms, else_arm })
    }

    /// 解析消息模式
    fn parse_task_pattern(&mut self) -> AutoResult<TaskPattern> {
        let name = self.expect_ident()?;

        if self.is_kind(TokenKind::LParen) {
            // Variant(val1, val2)
            self.expect(TokenKind::LParen)?;
            let mut bindings = Vec::new();
            while !self.is_kind(TokenKind::RParen) {
                bindings.push(self.expect_ident()?);
                if !self.is_kind(TokenKind::RParen) {
                    self.expect(TokenKind::Comma)?;
                }
            }
            self.expect(TokenKind::RParen)?;
            Ok(TaskPattern::Variant { name, bindings })
        } else {
            // Unit variant
            Ok(TaskPattern::Unit(name))
        }
    }
}
```

**Step 2: 在 parse_stmt 中添加 Task 分支**

```rust
fn parse_stmt(&mut self) -> AutoResult<Stmt> {
    // 收集标注
    let annotations = self.collect_annotations()?;

    match self.cur.kind {
        TokenKind::Task => self.parse_task_def(annotations),
        // ... 其他分支
    }
}
```

**Step 3: 写单元测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_simple() {
        let code = r#"
            task Counter {
                count mut = 0
                on {
                    Add(val) => { self.count += val }
                    Reset => { self.count = 0 }
                    else => { }
                }
            }
        "#;
        let mut parser = Parser::new(code);
        let stmt = parser.parse_stmt().unwrap();
        assert!(matches!(stmt, Stmt::Task(_)));
    }

    #[test]
    fn test_parse_task_single() {
        let code = r#"
            #[single]
            task Logger {
                on {
                    Log(msg) => { print(msg) }
                }
            }
        "#;
        let mut parser = Parser::new(code);
        let stmt = parser.parse_stmt().unwrap();
        if let Stmt::Task(task) = stmt {
            assert!(task.is_single);
        } else {
            panic!("Expected Task");
        }
    }
}
```

**Step 4: 运行测试验证**

```bash
cargo test -p auto-lang -- parser
```

**Step 5: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat(parser): parse task definition for Plan 121"
```

---

## Task 6: Parser - 解析 spawn 和 send 表达式

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`

**Step 1: 在 parse_postfix 或 parse_call 中添加 spawn 解析**

```rust
/// 解析 TaskName.spawn(args) 表达式
fn parse_spawn_expr(&mut self, task_name: Name) -> AutoResult<Expr> {
    // 检查 .spawn
    if !self.is_kind(TokenKind::Dot) {
        return Ok(Expr::Ident(task_name));
    }
    self.next(); // consume '.'

    let method = self.expect_ident()?;
    if &*method != "spawn" {
        return Err(SyntaxError::Generic {
            message: format!("Expected 'spawn' method, got '{}'", method),
            span: pos_to_span(self.cur.pos),
        }.into());
    }

    // 解析参数
    self.expect(TokenKind::LParen)?;
    let args = self.parse_args()?;  // 复用现有方法
    self.expect(TokenKind::RParen)?;

    Ok(Expr::Spawn { task: task_name, args })
}

/// 解析 handle.send(msg) 或 TaskName.send(msg) 表达式
fn parse_send_expr(&mut self, target: Expr) -> AutoResult<Expr> {
    // 检查 .send
    if !self.is_kind(TokenKind::Dot) {
        return Ok(target);
    }
    self.next(); // consume '.'

    let method = self.expect_ident()?;
    if &*method != "send" {
        return Err(SyntaxError::Generic {
            message: format!("Expected 'send' method, got '{}'", method),
            span: pos_to_span(self.cur.pos),
        }.into());
    }

    // 解析消息
    self.expect(TokenKind::LParen)?;
    let msg = self.parse_expr()?;
    self.expect(TokenKind::RParen)?;

    // 确定 target 类型
    let send_target = match target {
        Expr::Ident(name) => {
            // 可能是单例 Task 或 Handle 变量
            // 暂时都作为 Handle 处理，语义分析阶段区分
            SendTarget::Handle(Box::new(Expr::Ident(name)))
        }
        _ => SendTarget::Handle(Box::new(target)),
    };

    Ok(Expr::Send { target: send_target, msg })
}
```

**Step 2: 写单元测试**

```rust
#[test]
fn test_parse_spawn() {
    let code = "CounterTask.spawn()";
    let mut parser = Parser::new(code);
    let expr = parser.parse_expr().unwrap();
    assert!(matches!(expr, Expr::Spawn { .. }));
}

#[test]
fn test_parse_send() {
    let code = "h.send(Add(10))";
    let mut parser = Parser::new(code);
    let expr = parser.parse_expr().unwrap();
    assert!(matches!(expr, Expr::Send { .. }));
}
```

**Step 3: 运行测试验证**

```bash
cargo test -p auto-lang -- parser
```

**Step 4: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat(parser): parse spawn and send expressions for Plan 121"
```

---

## Task 7: VM - 创建 TaskInstance 和 TaskHandle 结构

**Files:**
- Modify: `crates/auto-lang/src/vm/task.rs`

**Step 1: 添加 TaskInstance 和 TaskHandle**

在 `crates/auto-lang/src/vm/task.rs` 中添加：

```rust
use auto_val::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

// 现有的 AutoTask 和相关代码保留...

/// Plan 121: Task 消息（VM 层）
/// 在 VM 中，所有消息都序列化为 Value
pub type TaskMessage = Value;

/// Plan 121: 默认信箱容量
pub const DEFAULT_MAILBOX_CAPACITY: usize = 64;

/// Plan 121: Task 实例
/// 代表一个运行中的 Actor 实例
pub struct TaskInstance {
    /// Task 类型名（如 "CounterTask"）
    pub task_type: String,
    /// 唯一实例 ID
    pub instance_id: u64,
    /// 私有状态
    pub state: HashMap<String, Value>,
    /// 消息信箱接收端
    pub mailbox_rx: mpsc::Receiver<TaskMessage>,
    /// 消息信箱发送端（用于创建 Handle）
    pub mailbox_tx: mpsc::Sender<TaskMessage>,
    /// 是否已启动
    pub started: bool,
    /// 是否已停止
    pub stopped: bool,
}

impl TaskInstance {
    /// 创建新的 Task 实例
    pub fn new(task_type: String, instance_id: u64) -> Self {
        let (tx, rx) = mpsc::channel(DEFAULT_MAILBOX_CAPACITY);
        Self {
            task_type,
            instance_id,
            state: HashMap::new(),
            mailbox_rx: rx,
            mailbox_tx: tx,
            started: false,
            stopped: false,
        }
    }

    /// 创建指定容量的 Task 实例
    pub fn with_capacity(task_type: String, instance_id: u64, capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity);
        Self {
            task_type,
            instance_id,
            state: HashMap::new(),
            mailbox_rx: rx,
            mailbox_tx: tx,
            started: false,
            stopped: false,
        }
    }

    /// 创建 Handle
    pub fn create_handle(&self) -> TaskHandle {
        TaskHandle {
            task_type: self.task_type.clone(),
            instance_id: self.instance_id,
            tx: Arc::new(self.mailbox_tx.clone()),
        }
    }
}

/// Plan 121: Task Handle
/// 代表对 Task 实例的引用
#[derive(Clone)]
pub struct TaskHandle {
    /// Task 类型名
    pub task_type: String,
    /// 实例 ID
    pub instance_id: u64,
    /// 发送端（用于发送消息）
    pub tx: Arc<mpsc::Sender<TaskMessage>>,
}

impl TaskHandle {
    /// 发送消息到 Task
    /// 在 Phase 1 中，如果信箱满则返回错误
    pub async fn send(&self, msg: TaskMessage) -> Result<(), TaskError> {
        match self.tx.try_send(msg) {
            Ok(_) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                Err(TaskError::MailboxFull {
                    task_type: self.task_type.clone(),
                    instance_id: self.instance_id,
                })
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(TaskError::TaskClosed {
                    task_type: self.task_type.clone(),
                    instance_id: self.instance_id,
                })
            }
        }
    }

    /// 同步发送（用于非 async 上下文）
    pub fn send_blocking(&self, msg: TaskMessage) -> Result<(), TaskError> {
        match self.tx.try_send(msg) {
            Ok(_) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                Err(TaskError::MailboxFull {
                    task_type: self.task_type.clone(),
                    instance_id: self.instance_id,
                })
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Err(TaskError::TaskClosed {
                    task_type: self.task_type.clone(),
                    instance_id: self.instance_id,
                })
            }
        }
    }
}

impl std::fmt::Debug for TaskHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Handle<{}#{}>", self.task_type, self.instance_id)
    }
}

impl PartialEq for TaskHandle {
    fn eq(&self, other: &Self) -> bool {
        self.task_type == other.task_type && self.instance_id == other.instance_id
    }
}

/// Plan 121: Task 错误类型
#[derive(Debug, Clone)]
pub enum TaskError {
    /// 信箱已满
    MailboxFull { task_type: String, instance_id: u64 },
    /// Task 已关闭
    TaskClosed { task_type: String, instance_id: u64 },
    /// Task 不存在
    TaskNotFound { task_type: String, instance_id: u64 },
    /// 单例 Task 不能 spawn
    SingleTaskCannotSpawn { task_type: String },
    /// 运行时错误
    RuntimeError(String),
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::MailboxFull { task_type, instance_id } => {
                write!(f, "MailboxFull: {}#{} (capacity: {})", task_type, instance_id, DEFAULT_MAILBOX_CAPACITY)
            }
            TaskError::TaskClosed { task_type, instance_id } => {
                write!(f, "TaskClosed: {}#{}", task_type, instance_id)
            }
            TaskError::TaskNotFound { task_type, instance_id } => {
                write!(f, "TaskNotFound: {}#{}", task_type, instance_id)
            }
            TaskError::SingleTaskCannotSpawn { task_type } => {
                write!(f, "SingleTaskCannotSpawn: {} is marked #[single]", task_type)
            }
            TaskError::RuntimeError(msg) => write!(f, "RuntimeError: {}", msg),
        }
    }
}

impl std::error::Error for TaskError {}
```

**Step 2: 运行编译验证**

```bash
cargo build -p auto-lang 2>&1 | head -50
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/vm/task.rs
git commit -m "feat(vm): add TaskInstance and TaskHandle for Plan 121"
```

---

## Task 8: VM - 创建 TaskSystem 调度器

**Files:**
- Create: `crates/auto-lang/src/vm/task_system.rs`
- Modify: `crates/auto-lang/src/vm/mod.rs` (添加 mod)

**Step 1: 创建 TaskSystem 模块**

创建文件 `crates/auto-lang/src/vm/task_system.rs`：

```rust
use super::task::{TaskInstance, TaskHandle, TaskError, TaskMessage};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plan 121: TaskSystem 状态
/// 管理所有 Task 实例的注册和调度
pub struct TaskSystem {
    /// 多实例 Task 注册表 (instance_id -> TaskInstance)
    instances: Arc<RwLock<HashMap<u64, TaskInstance>>>,
    /// 单例 Task 注册表 (task_type -> Handle)
    singletons: Arc<RwLock<HashMap<String, TaskHandle>>>,
    /// spawn 顺序记录（用于 LIFO 停止）
    spawn_order: Arc<RwLock<Vec<u64>>>,
    /// 实例 ID 生成器
    next_instance_id: AtomicU64,
    /// 是否已启动
    started: Arc<RwLock<bool>>,
}

impl TaskSystem {
    /// 创建新的 TaskSystem
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
            singletons: Arc::new(RwLock::new(HashMap::new())),
            spawn_order: Arc::new(RwLock::new(Vec::new())),
            next_instance_id: AtomicU64::new(1),
            started: Arc::new(RwLock::new(false)),
        }
    }

    /// Spawn 一个新的 Task 实例
    /// 返回 Handle 用于通信
    pub async fn spawn(&self, task_type: &str) -> Result<TaskHandle, TaskError> {
        // 检查是否为单例
        {
            let singletons = self.singletons.read().await;
            if singletons.contains_key(task_type) {
                return Err(TaskError::SingleTaskCannotSpawn {
                    task_type: task_type.to_string(),
                });
            }
        }

        // 分配实例 ID
        let instance_id = self.next_instance_id.fetch_add(1, Ordering::SeqCst);

        // 创建实例
        let instance = TaskInstance::new(task_type.to_string(), instance_id);
        let handle = instance.create_handle();

        // 注册
        {
            let mut instances = self.instances.write().await;
            instances.insert(instance_id, instance);
        }

        // 记录 spawn 顺序
        {
            let mut order = self.spawn_order.write().await;
            order.push(instance_id);
        }

        Ok(handle)
    }

    /// 注册单例 Task
    pub async fn register_singleton(&self, task_type: &str) -> TaskHandle {
        let instance_id = self.next_instance_id.fetch_add(1, Ordering::SeqCst);
        let instance = TaskInstance::new(task_type.to_string(), instance_id);
        let handle = instance.create_handle();

        {
            let mut instances = self.instances.write().await;
            instances.insert(instance_id, instance);
        }

        {
            let mut singletons = self.singletons.write().await;
            singletons.insert(task_type.to_string(), handle.clone());
        }

        {
            let mut order = self.spawn_order.write().await;
            order.push(instance_id);
        }

        handle
    }

    /// 获取单例 Handle
    pub async fn get_singleton(&self, task_type: &str) -> Option<TaskHandle> {
        let singletons = self.singletons.read().await;
        singletons.get(task_type).cloned()
    }

    /// 启动调度器
    /// 阻塞直到 Ctrl+C
    pub async fn start(&self) {
        // 标记已启动
        {
            let mut started = self.started.write().await;
            *started = true;
        }

        // 创建 Tokio runtime
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            build()
            .expect("Failed to create Tokio runtime");

        rt.block_on(async {
            // 1. 调用所有 Task 的 start() 钩子
            {
                let instances = self.instances.read().await;
                for (_, instance) in instances.iter() {
                    // TODO: 调用 start() 钩子
                    // 需要从 AST 获取钩子代码并执行
                    println!("Task {}#{} starting", instance.task_type, instance.instance_id);
                }
            }

            // 2. 启动消息循环
            // TODO: 为每个 Task 启动 tokio::spawn 的消息循环

            // 3. 等待 Ctrl+C
            println!("TaskSystem started. Press Ctrl+C to stop...");
            tokio::signal::ctrl_c().await.ok();

            // 4. 按 LIFO 顺序调用 stop() 钩子
            let order = self.spawn_order.read().await;
            let instances = self.instances.read().await;

            for instance_id in order.iter().rev() {
                if let Some(instance) = instances.get(instance_id) {
                    // TODO: 调用 stop() 钩子
                    println!("Task {}#{} stopping", instance.task_type, instance.instance_id);
                }
            }

            println!("TaskSystem stopped");
        });
    }

    /// 检查是否已启动
    pub async fn is_started(&self) -> bool {
        *self.started.read().await
    }
}

impl Default for TaskSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_task() {
        let system = TaskSystem::new();
        let handle = system.spawn("CounterTask").await.unwrap();
        assert_eq!(handle.task_type, "CounterTask");
        assert_eq!(handle.instance_id, 1);
    }

    #[tokio::test]
    async fn test_singleton_cannot_spawn() {
        let system = TaskSystem::new();
        system.register_singleton("Logger").await;

        let result = system.spawn("Logger").await;
        assert!(matches!(result, Err(TaskError::SingleTaskCannotSpawn { .. })));
    }
}
```

**Step 2: 在 vm/mod.rs 中添加模块**

如果 `crates/auto-lang/src/vm/mod.rs` 存在，添加：

```rust
pub mod task_system;
pub use task_system::*;
```

如果不存在，可能需要在 `lib.rs` 或其他入口文件中添加。

**Step 3: 运行测试验证**

```bash
cargo test -p auto-lang -- task_system
```

**Step 4: Commit**

```bash
git add crates/auto-lang/src/vm/task_system.rs
git commit -m "feat(vm): add TaskSystem scheduler for Plan 121"
```

---

## Task 9: VM - 集成 TaskSystem 到 AutoVM

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs`

**Step 1: 在 AutoVM 中添加 TaskSystem**

在 `AutoVM` 结构体中添加 TaskSystem 字段：

```rust
use super::task_system::TaskSystem;

pub struct AutoVM {
    // ... 现有字段

    // Plan 121: Task 系统
    pub task_system: Arc<TaskSystem>,
}

impl AutoVM {
    pub fn new(flash: VirtualFlash, _ram_size: usize) -> Self {
        // ... 现有初始化

        Self {
            // ... 现有字段
            task_system: Arc::new(TaskSystem::new()),
        }
    }
}
```

**Step 2: 添加 spawn 和 send 的执行逻辑**

在 VM 执行循环中添加 Spawn 和 Send 的处理：

```rust
// 在 execute 或 eval 方法中
match expr {
    // ... 现有分支

    Expr::Spawn { task, args } => {
        // 1. 查找 Task 定义
        // 2. 调用 task_system.spawn()
        // 3. 返回 Handle 作为 Value
        todo!("Implement spawn execution")
    }

    Expr::Send { target, msg } => {
        // 1. 解析 target（Handle 或单例名）
        // 2. 求值 msg
        // 3. 调用 handle.send(msg)
        // 4. 返回 void 或错误
        todo!("Implement send execution")
    }
}
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/vm/engine.rs
git commit -m "feat(vm): integrate TaskSystem into AutoVM for Plan 121"
```

---

## Task 10: a2rs Transpiler - 生成 Task Rust 代码

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs`

**Step 1: 添加 Task 代码生成**

在 Rust transpiler 中添加 Task 处理：

```rust
impl RustTranspiler {
    /// 生成 Task 定义
    pub fn transpile_task(&self, task: &TaskDef) -> AutoResult<String> {
        let mut output = String::new();

        // 1. 生成 Handle 类型别名
        if !task.is_single {
            output.push_str(&format!(
                "pub type {}Handle = Arc<mpsc::Sender<{}>>;\n\n",
                task.name, task.msg_type.as_ref().map(|t| t.as_str()).unwrap_or("Value")
            ));
        }

        // 2. 生成 Task 结构体
        output.push_str(&format!("pub struct {} {{\n", task.name));
        output.push_str("    mailbox_rx: mpsc::Receiver<Msg>,\n");
        for field in &task.fields {
            output.push_str(&format!("    {}: {},\n", field.name, field.ty));
        }
        output.push_str("}\n\n");

        // 3. 生成 impl 块
        output.push_str(&format!("impl {} {{\n", task.name));
        output.push_str("    pub async fn run(mut self) -> Result<(), String> {\n");

        // start() 钩子
        if let Some(start) = &task.start_hook {
            output.push_str("        // start() hook\n");
            output.push_str(&self.transpile_block(&start.body)?);
            output.push_str("\n");
        }

        // 消息循环
        output.push_str("        while let Some(msg) = self.mailbox_rx.recv().await {\n");
        output.push_str("            match msg {\n");
        for arm in &task.on_block.arms {
            output.push_str(&format!("                {} => {{ /* {} */ }}\n",
                arm.pattern, arm.body.to_atom_str().chars().take(30).collect::<String>()));
        }
        output.push_str("            }\n");
        output.push_str("        }\n");

        output.push_str("        Ok(())\n");
        output.push_str("    }\n");

        // stop() 钩子
        if let Some(stop) = &task.stop_hook {
            output.push_str("    pub fn stop(&mut self) -> Result<(), String> {\n");
            output.push_str(&self.transpile_block(&stop.body)?);
            output.push_str("        Ok(())\n");
            output.push_str("    }\n");
        }

        output.push_str("}\n");

        Ok(output)
    }

    /// 生成 spawn 函数
    pub fn transpile_spawn_fn(&self, task: &TaskDef) -> AutoResult<String> {
        if task.is_single {
            return Ok(String::new()); // 单例不生成 spawn 函数
        }

        let mut output = String::new();
        output.push_str(&format!(
            "pub fn spawn_{}() -> {}Handle {{\n",
            task.name.to_string().to_lowercase(),
            task.name
        ));
        output.push_str("    let (tx, rx) = mpsc::channel(64);\n");
        output.push_str(&format!(
            "    let task = {} {{ mailbox_rx: rx }};\n",
            task.name
        ));
        output.push_str("    tokio::spawn(async move { task.run().await });\n");
        output.push_str("    Arc::new(tx)\n");
        output.push_str("}\n");

        Ok(output)
    }
}
```

**Step 2: 处理 spawn 和 send 表达式**

在表达式转译中添加：

```rust
match expr {
    Expr::Spawn { task, args } => {
        Ok(format!("spawn_{}()", task.to_string().to_lowercase()))
    }

    Expr::Send { target, msg } => {
        let target_code = match target {
            SendTarget::Handle(expr) => self.transpile_expr(expr)?,
            SendTarget::Single(name) => format!("{}_SINGLETON", name),
        };
        let msg_code = self.transpile_expr(msg)?;
        Ok(format!("{}.try_send({})?", target_code, msg_code))
    }
}
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/trans/rust.rs
git commit -m "feat(a2rs): generate Task Rust code for Plan 121"
```

---

## Task 11: 测试 - VM 层 Task 测试

**Files:**
- Create: `crates/auto-lang/src/vm/tests_task.rs`

**Step 1: 创建 VM Task 测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::task_system::TaskSystem;
    use crate::vm::task::{TaskHandle, TaskError};

    #[tokio::test]
    async fn test_task_system_spawn() {
        let system = TaskSystem::new();

        // Spawn 两个实例
        let h1 = system.spawn("CounterTask").await.unwrap();
        let h2 = system.spawn("CounterTask").await.unwrap();

        // ID 应该不同
        assert_ne!(h1.instance_id, h2.instance_id);
        assert_eq!(h1.task_type, h2.task_type);
    }

    #[tokio::test]
    async fn test_singleton_register() {
        let system = TaskSystem::new();

        // 注册单例
        let handle = system.register_singleton("Logger").await.unwrap();

        // 获取单例
        let retrieved = system.get_singleton("Logger").await;
        assert!(retrieved.is_some());

        // 单例不能 spawn
        let result = system.spawn("Logger").await;
        assert!(matches!(result, Err(TaskError::SingleTaskCannotSpawn { .. })));
    }

    #[tokio::test]
    async fn test_handle_send() {
        let system = TaskSystem::new();
        let handle = system.spawn("CounterTask").await.unwrap();

        // 发送消息
        let msg = Value::Int(42);
        let result = handle.send(msg).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mailbox_full() {
        let system = TaskSystem::new();
        let handle = system.spawn("CounterTask").await.unwrap();

        // 填满信箱（容量 64）
        for i in 0..64 {
            handle.send(Value::Int(i)).await.unwrap();
        }

        // 第 65 条应该失败
        let result = handle.send(Value::Int(999)).await;
        assert!(matches!(result, Err(TaskError::MailboxFull { .. })));
    }
}
```

**Step 2: 运行测试**

```bash
cargo test -p auto-lang -- tests_task
```

**Step 3: Commit**

```bash
git add crates/auto-lang/src/vm/tests_task.rs
git commit -m "test(vm): add Task system tests for Plan 121"
```

---

## Task 12: 测试 - a2r 集成测试

**Files:**
- Create: `crates/auto-lang/test/a2r/050_task/` 目录
- Create: `crates/auto-lang/test/a2r/050_task/task_basic.at`
- Create: `crates/auto-lang/test/a2r/050_task/task_basic.expected.rs`

**Step 1: 创建测试用例**

`task_basic.at`:
```auto
enum CounterMsg {
    Add(int),
    Reset,
    Print
}

task CounterTask {
    count mut = 0

    fn start() ! {
        self.count = 0
    }

    fn stop() ! {
        print("Stopping")
    }

    on {
        Add(val) => { self.count += val }
        Reset => { self.count = 0 }
        Print => { print(self.count) }
        else => { }
    }
}

fn main() ! {
    let h = CounterTask.spawn()
    h.send(Add(10))
    TaskSystem.start()
}
```

`task_basic.expected.rs`:
```rust
// Expected output - generated by a2rs transpiler
use tokio::sync::mpsc;
use std::sync::Arc;

pub enum CounterMsg {
    Add(i32),
    Reset,
    Print,
}

pub type CounterTaskHandle = Arc<mpsc::Sender<CounterMsg>>;

pub struct CounterTask {
    count: i32,
    mailbox_rx: mpsc::Receiver<CounterMsg>,
}

impl CounterTask {
    pub async fn run(mut self) -> Result<(), String> {
        self.count = 0;

        while let Some(msg) = self.mailbox_rx.recv().await {
            match msg {
                CounterMsg::Add(val) => { self.count += val; }
                CounterMsg::Reset => { self.count = 0; }
                CounterMsg::Print => { println!("{}", self.count); }
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        println!("Stopping");
        Ok(())
    }
}

pub fn spawn_counter_task() -> CounterTaskHandle {
    let (tx, rx) = mpsc::channel(64);
    let task = CounterTask { count: 0, mailbox_rx: rx };
    tokio::spawn(async move { let _ = task.run().await; });
    Arc::new(tx)
}

fn main() {
    let h = spawn_counter_task();
    h.try_send(CounterMsg::Add(10)).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        tokio::signal::ctrl_c().await.ok();
    });
}
```

**Step 2: 添加测试函数**

在 `crates/auto-lang/src/trans/rust.rs` 的测试模块中添加：

```rust
#[test]
fn test_050_task_basic() {
    test_a2r("050_task/task_basic").unwrap();
}
```

**Step 3: 运行测试**

```bash
cargo test -p auto-lang test_050_task_basic
```

**Step 4: Commit**

```bash
git add crates/auto-lang/test/a2r/050_task/
git commit -m "test(a2r): add Task transpilation test for Plan 121"
```

---

## Task 13: 最终集成与文档

**Step 1: 运行所有测试**

```bash
cargo test -p auto-lang
```

**Step 2: 更新 CLAUDE.md**

在 `CLAUDE.md` 中添加 Task/Msg 系统的说明：

```markdown
## Task/Msg System (Plan 121)

AutoLang 支持 Actor 模型的 Task/Msg 系统：

```auto
enum CounterMsg {
    Add(int),
    Reset,
}

task CounterTask {
    count mut = 0

    fn start() ! { self.count = 0 }
    fn stop() ! { }

    on {
        Add(val) => { self.count += val }
        Reset => { self.count = 0 }
        else => { }
    }
}

fn main() ! {
    let h = CounterTask.spawn()
    h.send(Add(10))
    TaskSystem.start()
}
```

### 关键概念

- `task` - 定义 Actor 实体
- `#[single]` - 单例 Task 标注
- `spawn()` - 创建 Task 实例
- `send(msg)` - 发送消息
- `TaskSystem.start()` - 启动调度器
```

**Step 3: Final Commit**

```bash
git add .
git commit -m "feat(plan-121): complete Task/Msg system implementation"
```

---

## 验收清单

完成以下检查确认实现正确：

- [ ] `cargo build -p auto-lang` 无错误
- [ ] `cargo test -p auto-lang` 全部通过
- [ ] 可以解析 `task Name { ... }` 语法
- [ ] 可以解析 `#[single]` 标注
- [ ] 可以解析 `Name.spawn()` 表达式
- [ ] 可以解析 `handle.send(msg)` 表达式
- [ ] VM 可以创建 Task 实例
- [ ] VM 可以发送消息到 Task
- [ ] 信箱满时返回错误
- [ ] a2rs 可以生成正确的 Rust 代码
