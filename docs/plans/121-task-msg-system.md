# Plan 121: AutoLang Task/Msg 基础系统

## Status: 📋 PLANNING

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
