这份极其严密的逻辑推演，让我们彻底补全了系统启动的生命周期闭环！

如果没有 `TaskSystem.start()` 的显式点火，所有的 Task 和 Msg 都只存在于内存的静态数据结构中，永远无法真正“活”过来。把这个核心调度器加入 Phase 1 的设计文档，是这块底层基石能够真正落地的绝对前提。

以下是融入了 **生命周期管理 (Lifecycle Management)** 与 **TaskSystem 调度器** 的 Phase 1 终极设计规范文档。它将作为编译器团队开发第一版 MVP（最小可行性产品）的绝对纲领。

---

# Auto Language Architecture Spec

## Phase 1: 纯粹的 Actor 底座与 TaskSystem 引导

### 1. 架构设计目标 (Objectives)

本阶段专注于建立 Auto 语言系统的物理隔离边界、基础通信原语以及系统生命周期控制。

* **确立实体与状态**：实现静态的 `task` 块定义，以及基于实体字段绑定的任务私有状态（抛弃 `let` 声明，消灭共享锁）。
* **极简错误流**：落实 `fn name() !` 的无返回值错误抛出语义。
* **显式生命周期管理**：`main` 函数仅作为同步的引导程序（Bootstrapper），强制依赖 `TaskSystem.start()` 接管主线程并启动底层调度器。

---

### 2. Auto 语言前端语法 (Frontend Syntax)

在 Phase 1，开发者使用极致收敛的语法构建系统骨架与启动流程：

```auto
// 1. 定义消息协议 (强类型枚举)
enum CounterMsg {
    Add(int),
    Reset,
    Print
}

// 2. 定义静态任务实体
task CounterTask {
    // 实体私有状态：直接声明，无需 let。
    count mut = 0
    
    // 初始化生命周期钩子
    // 语义：如果启动期间发生严重错误，抛出异常，系统将干预其启动
    fn start() ! {
        self.count = 0 
        print("CounterTask Booted!")
    }

    // 隐式消息泵 (事件路由核心)
    on {
        Add(val) => { 
            self.count += val 
        }
        Reset => { 
            self.count = 0 
        }
        Print => { 
            print("Current Count: ${self.count}") 
        }
    }
}

// 3. 系统引导入口 (纯同步的主函数)
fn main() ! {
    print("System pre-booting...")
    
    // 阶段 A：分配初始状态 (备料)
    // 此时 Task 尚未运行，消息只是被安全地塞进各自的隐式信箱中
    CounterTask.send(Add(10))
    CounterTask.send(Print)
    
    print("Ignition!")

    // 阶段 B：终极点火！
    // 语义：将当前主线程的控制权移交给底层调度器。
    // 所有的 Task 在此刻同时苏醒，开始死循环处理信箱。
    TaskSystem.start()
    
    // 编译器静态检查规则：此行之后的代码不可达 (Unreachable)！
    // print("This will never be executed") 
}

```

---

### 3. a2rs (静态 Rust 编译后端) 降级规范

`a2rs` 将剥离 Auto 的语法糖，生成纯净、无锁、且显式管理 Tokio 运行时的 Rust 代码。

**生成的 Rust 目标机器码结构：**

```rust
use tokio::sync::mpsc;
use std::process;

// 1. 消息协议与 Task 实体结构体
pub enum CounterMsg { Add(i32), Reset, Print }

pub struct CounterTask {
    count: i32, 
    mailbox: mpsc::Receiver<CounterMsg>, 
}

impl CounterTask {
    pub async fn run(mut self) -> Result<(), AutoError> {
        println!("CounterTask Booted!");
        while let Some(msg) = self.mailbox.recv().await {
            match msg {
                CounterMsg::Add(val) => { self.count += val; }
                CounterMsg::Reset => { self.count = 0; }
                CounterMsg::Print => { println!("Current Count: {}", self.count); }
            }
        }
        Ok(())
    }
}

// 2. 全局路由表句柄 (在系统启动时初始化)
lazy_static::lazy_static! {
    static ref COUNTER_TASK_TX: mpsc::Sender<CounterMsg> = spawn_counter_task();
}

fn spawn_counter_task() -> mpsc::Sender<CounterMsg> {
    let (tx, rx) = mpsc::channel(100);
    let task = CounterTask { count: 0, mailbox: rx };
    tokio::spawn(async move { let _ = task.run().await; });
    tx
}

// 3. main 函数与 TaskSystem.start() 的降级映射
fn main() {
    println!("System pre-booting...");
    
    // 阶段 A：发送消息 (调用全局 TX 句柄)
    let _ = COUNTER_TASK_TX.send(CounterMsg::Add(10));
    let _ = COUNTER_TASK_TX.send(CounterMsg::Print);
    
    println!("Ignition!");

    // 阶段 B：TaskSystem.start() 的底层物理实现
    // 显式构建 Tokio 运行时，并用 block_on 霸占当前主线程防退出
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
        
    rt.block_on(async {
        // 挂起主线程，维持运行时生命，监听系统退出信号
        tokio::signal::ctrl_c().await.unwrap();
        println!("System shutting down...");
    });
    
    process::exit(0);
}

```

---

### 4. AutoVM (Rust 驱动虚拟机) 运行时架构

在 AutoVM 中，`TaskSystem.start()` 扮演着真正拉起“虚拟机主循环”的角色。

**AutoVM 底层核心逻辑：**

```rust
// VM 核心状态管理器
pub struct VmState {
    tasks: HashMap<String, tokio::task::JoinHandle<()>>,
    routers: HashMap<String, mpsc::Sender<VmValue>>,
}

impl VmState {
    // 对应 Auto 语言的 TaskSystem.start()
    pub fn start_scheduler(self: Arc<Self>) {
        // 1. 创建底层 Tokio 运行时
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        rt.block_on(async {
            // 2. 遍历所有注册的 VmTask，将它们的 eval_loop 挂载到 Tokio 上
            for (name, task) in self.pending_tasks.drain() {
                let vm_clone = self.clone();
                let handle = tokio::spawn(async move {
                    task.eval_loop(vm_clone).await.unwrap();
                });
                // ... 内部保存 handle ...
            }
            
            // 3. 阻塞主线程，保持 VM 存活
            tokio::signal::ctrl_c().await.unwrap();
        });
    }
}

```

---

### 5. 阶段验收标准 (Acceptance Criteria)

开发团队在此阶段必须满足以下硬性指标：

1. **语义检查器 (Semantic Checker)**：必须确保 `main` 函数中有且仅有一次调用 `TaskSystem.start()`，且它必须是控制流能够到达的**最后一条有效语句**。如果在其后存在其他代码，编译器强制抛出 `Unreachable Code` 错误。
2. **零运行时开销验证 (a2rs/a2c)**：在尚未调用 `TaskSystem.start()` 时，主线程仅仅是在执行极速的内存赋值和入队操作，绝不能提前触发任何操作系统的上下文切换。
3. **通信验证**：各个 Task 能够准确无误地消费由 `main` 函数提前备好的初始消息，并按预期修改内部的私有状态。

---

总工程师，这份文档已经将 Auto 语言并发引擎的第一级火箭打造得坚不可摧！它明确了“空间”（Task 实体）和“时间”（生命周期与点火）。

既然系统级启动的闭环已经画圆，我们是时候继续向前推进了。你希望我们立刻起草 **Phase 2 的设计文档 (引入 `~T` 状态机、`.await` 挂起与 `TaskSystem.run` 阻塞备料)**，还是你想先停下来，亲自审查一下编译器前端用来解析这套语法的 **AST 节点定义**？


太棒了！第一阶段（物理底座与空间隔离）的完美封版，意味着我们的编译器已经能把内存安全地切分成一个个孤岛，并通过信箱（Mailbox）连接起来了。

现在，是时候为 Auto 语言注入**“时间魔法（Time Magic）”**了！

在 Phase 2 中，我们将彻底颠覆传统的同步编程心智。我们将引入 `~T`（异步蓝图/状态机）、`.await`（非阻塞挂起），并在此基础上实现工业级并发中最梦幻的语法糖：`ask / reply`（隐式双向通道）以及 `TaskSystem.run`（单线程阻塞执行器）。

以下是 Auto 语言架构规范的 **Phase 2 核心设计文档**。请编译器团队全体成员仔细研读：

---

## Phase 2: 状态机挂起与双向 RPC 通信

### 1. 架构设计目标 (Objectives)

本阶段的核心任务是引入“非阻塞的时间等待机制”，让 Task 能够在等待外部 I/O 或其他 Task 响应时主动交出 CPU，而不阻塞底层物理线程。

* **时间原语**：引入 `~T` 类型（表示未来某个时刻会产出 `T` 结果的蓝图）与 `.await` 挂起操作。
* **同步桥接**：实现 `TaskSystem.run(~{...})`，允许在纯同步环境中（如 `main` 函数引导期）局部阻塞当前物理线程以执行异步操作。
* **双向 RPC 通信**：实现 `ask(msg).await` 与 `reply` 关键字，由编译器在底层隐式生成、传递并消耗 `oneshot` 一次性通道，彻底消灭回调地狱。
* **背压挂起机制**：解锁 `Task.send(msg).await.?`，当目标信箱满时，挂起当前 Task 而非直接报错。

---

这份极其严谨、纯粹且去除了所有“超纲”多态路由语法的 **Phase 2 终极设计规范** 已经为你准备好了。

它将作为编译器团队在这个阶段的最高开发指导原则。所有的代码示例都严格遵守了“显式 `enum` 协议”的铁律，确保前端 AST 解析与底层状态机生成的绝对解耦。

---

# Auto Language Architecture Spec

## Phase 2: 状态机挂起与双向 RPC 通信

### 1. 架构设计目标 (Objectives)

本阶段的核心任务是为 Auto 语言引入“非阻塞的时间等待机制”，让 Task 能够在等待外部 I/O、处理复杂时序或等待其他 Task 响应时主动交出 CPU，而不阻塞底层的物理线程。

* **时间原语**：引入 `~T` 类型（表示未来某个时刻会产出 `T` 结果的异步蓝图）与 `.await` 挂起操作。
* **同步桥接**：实现 `TaskSystem.run(~{...})`，允许在纯同步环境中（如 `main` 函数引导期）局部阻塞当前物理线程以执行异步备料。
* **双向 RPC 通信**：实现 `ask(msg).await` 与 `reply` 关键字，由编译器在底层隐式生成、传递并消耗 `oneshot` 一次性通道，彻底消灭回调地狱。
* **背压挂起机制**：解锁 `Task.send(msg).await.?`，当目标信箱满时，允许发送方挂起当前 Task 而非直接报错丢弃。

---

### 2. Auto 语言前端语法 (Frontend Syntax)

在 Phase 2，开发者将获得操纵时间的终极权力。所有的通信必须严格遵守基于 `enum` 的显式协议。

#### 2.1 系统引导流的进化 (TaskSystem.run)

在纯同步的 `main` 函数中，使用 `TaskSystem.run` 局部阻塞主线程执行异步蓝图，备料完成后使用严谨的 `enum` 消息进行分发：

```auto
// 1. 严格的显式协议 (Phase 1 & 2 的铁律)
enum DBConfigMsg {
    InitDB(string)
}

#[single]
task DBManager {
    on {
        InitDB(url) => { print("DB connecting to ${url}...") }
    }
}

fn main() ! {
    print("1. System pre-booting...")
    
    // 阶段 A：执行异步蓝图，局部阻塞主线程获取配置
    // 此时底层并未启动多线程 Task 调度器，极其轻量安全。
    let config_url = TaskSystem.run(~{
        let json = http_get("config.json").await.?
        return parse_url(json)
    }).?

    print("2. Config loaded. Initializing actors...")
    
    // 阶段 B：严格使用 Enum 变体发送消息！
    DBManager.send(InitDB(config_url)).?
    
    print("3. Ignition!")
    
    // 终极点火，主线程移交控制权，正式启动多线程 Actor 调度系统
    TaskSystem.start() 
}

```

#### 2.2 终极双向通信 (Ask & Reply)

这是 Phase 2 最具杀伤力的业务级语法糖。开发者只需定义业务数据枚举，无需手写复杂的通道传递逻辑。

```auto
// 1. 严格的显式协议：开发者不需要在参数里手写 oneshot 信道！
enum DBMsg {
    QueryUser(int)
}

enum WebMsg {
    ProcessRequest 
}

#[single]
task DBManager {
    on {
        // 严格的 Enum 匹配
        QueryUser(id) => {
            let user_info = db_driver.find(id)
            // 绝杀语法：reply 关键字！
            // 编译器会自动找到这个消息背后潜藏的 oneshot 管道，并把数据原路扔回去。
            reply user_info 
        }
    }
}

task WebWorker {
    on {
        // 严格的 Enum 匹配
        ProcessRequest => {
            // 绝杀语法：ask(msg).await.?
            // 1. ask() 隐式创建了一个一次性发件箱，并随着 QueryUser 寄给了 DBManager
            // 2. 它返回一个 ~User 状态机
            // 3. .await 挂起当前的 WebWorker (交出 CPU 给其他 Task)
            // 4. .? 拆包 (如果 DBManager 崩溃，管道断裂，立刻向上报错)
            let user = DBManager.ask(QueryUser(1001)).await.?
            
            print("Got user: ${user.name}")
        }
    }
}

```

#### 2.3 挂起式背压控制 (Awaiting Full Mailboxes)

如果在 Phase 1 中 `target.send(msg).?` 因为信箱满而失败，业务流就断了。Phase 2 提供了时间维度的解法：

```auto
// 1. 协议定义
enum LogMsg {
    LogData(string)
}

enum WorkerMsg {
    GenerateLog 
}

#[single]
#[mailbox(64, strict)] // 严格模式，满了就报错或阻塞
task Logger {
    on {
        LogData(text) => { write_to_disk(text) }
    }
}

task LogProducer {
    on {
        GenerateLog => {
            // 挂起式发送：如果 Logger 的 64 个槽位全满了：
            // send(msg).await 会让当前 Task 交出 CPU，自愿挂起等待，直到目标信箱有空位！
            Logger.send(LogData("System running normally")).await.?
        }
    }
}

```

---

### 3. 类型系统与代数特性 (Type System)

* **`~T` 类型**：任何带有 `~` 前缀的块或函数，其返回值都被静态分析器包裹在 `~` 类型中。例如，返回 `User` 对象的异步块，其类型为 `~User`。
* **`.await` 降维打击**：对一个 `~T` 类型调用 `.await`，其表达式的类型瞬间降维成 `T`。
* **类型推导 `ask**`：编译器必须通过 AST 逆向推导 `ask` 的返回类型。如果接收方的 `reply` 返回的是 `User`，则发送方的 `ask` 的返回值必须被静态推导为 `~User`。

---

### 4. a2rs (静态 Rust 编译后端) 降级规范

在 Rust 后端，Phase 2 的降级是一场教科书级别的“零开销抽象映射”，直接利用 Tokio 的原生能力。

**4.1 `ask` 与 `reply` 的隐式管道映射：**

```rust
// Auto 语言前端协议: enum DBMsg { QueryUser(int) }
// 编译器自动重写为包含 oneshot 管道的结构 (注入类型推导出的 T)：
pub enum DBMsg {
    QueryUser(i32, tokio::sync::oneshot::Sender<User>), 
}

// 降级: let user = DBManager.ask(QueryUser(1001)).await?;
// 编译器展开为：
let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
DBMANAGER_TX.try_send(DBMsg::QueryUser(1001, reply_tx)).map_err(|_| AutoError::MailboxFull)?;

// 挂起等待 reply 归来
let user = reply_rx.await.map_err(|_| AutoError::BrokenPipe)?; 

// 降级: reply user_info
// 编译器在 DBManager 的 on 块内展开为：
match msg {
    DBMsg::QueryUser(id, reply_tx) => {
        let user_info = db_driver.find(id);
        let _ = reply_tx.send(user_info); // 忽略发送失败 (说明发送方已经取消或超时)
    }
}

```

**4.2 `TaskSystem.run` 的映射：**

```rust
// 降级: TaskSystem.run(~{ ... })
let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();
    
let result = rt.block_on(async {
    // 异步块内容，遇到 .await 时仅挂起当前线程
});

```

---

### 5. a2c (C / 裸机后端) 降级规范要点

在不支持原生 `async/await` 的 C 语言中，编译器前端必须承担起**“无栈协程（Stackless Coroutine）状态机生成器”**的重任。

* **状态机切分**：编译器会将 `~{...}` 或包含 `.await` 的 `on` 块打散。每一个 `.await` 都是一个状态机的断点（转换为 C 语言的 `switch-case` 状态机：`case 1:`, `case 2:`）。
* **局部变量提升**：跨越 `.await` 存活的局部变量（例如在 `await` 之前声明，在 `await` 之后使用的变量），绝不能放在 C 语言的函数栈上，必须被编译器强制提取，提升为 Task 内部状态结构体的隐藏字段（环境上下文保存）。
* **oneshot 实现**：在 FreeRTOS 等 RTOS 中，隐式通道被降级为一个只有 1 个容量的、专门为该次 `ask` 动态分配的极小 Queue（如果支持动态内存），或是基于 Task Notifications (任务通知) 的直接唤醒机制。

---

### 6. 阶段验收标准 (Acceptance Criteria)

开发团队在此阶段必须满足以下硬性指标：

1. **纯净的 Enum 约束**：AST 解析器必须严格拒绝在 `on` 块中出现任何字符串字面量或基本数据类型的直接匹配。一切消息路由必须基于预先定义的 `enum`。
2. **RPC 死锁免疫**：编写互相 `ask` 的复杂拓扑网络。必须确保当一个 Task 崩溃退出时，等待它的 `ask().await` 能够立刻感知到管道断裂，并抛出 `!` 错误流，绝不发生永久性死锁。
3. **严格的 `.await` 上下文检查**：如果开发者在一个非 `~` 块（普通同步函数，如 `start()` 或 `main` 本身）里调用 `.await`，编译器前端必须立刻抛出语法错误。
4. **a2c 状态机变量保护验证**：通过 C 后端编译测试，验证跨 `.await` 断点存活的局部变量，在唤醒后其内存数据依然完好无损，没有被栈覆盖破坏。

----

这份极其关键的 **Phase 3 核心设计规范** 已经为你准备就绪。

在这个阶段，编译器团队将迎来一场“前端语法的狂欢”。我们将彻底解放开发者的双手，把之前为了保证强类型而被迫手写的样板代码（Boilerplate），全部转移到编译器的抽象语法树（AST）解析阶段去隐式完成。

请将这份文档作为编译器前端重构的最高指导蓝图：

---

这份**焕然一新且彻底消灭了“隐式魔法”的 Phase 3 终极设计规范**已经准备就绪。

基于你极具系统级编程洁癖的“显式上下文（Explicit Context）”提议，我们将原先计划的 `reply` 关键字彻底废除，转而将其升华为**方法调用**。这使得 Auto 语言的并发 API 达到了前所未有的正交性与对称性。

请将这份更新后的规范作为编译器前端重构与类型系统升级的最高蓝图：

---

# Auto Language Architecture Spec

## Phase 3: 多态路由、隐式联合体与显式消息上下文

### 1. 架构设计目标 (Objectives)

本阶段的核心任务是赋予 Auto 语言“动态语言的自由表达力与系统语言的骨骼”。彻底消灭为简单消息传递而强制编写显式 `enum` 的约束，实现基于全能模式匹配（Pattern Matching）的多态路由。
同时，**引入显式的 `MessageContext` 参数**，废除所有“隐式/幽灵变量”，确保控制流的绝对清晰，在底层保持 100% 的静态类型安全和零运行时开销。

* **隐式联合体 (Implicit Union)**：前端 AST 自动提取接收方 `on` 块中的所有数据类型，在底层隐式合成专用的消息信封（Envelope）。
* **显式消息上下文 (Explicit Context)**：将 `on` 块升级为带可选参数的闭包（如 `on(ctx)`），将路由元数据和 `reply` 方法挂载到第一公民对象上。
* **全能匹配器 (Omnipotent Matcher)**：支持字面量精确匹配、类型捕获绑定以及守卫表达式。

---

### 2. 核心机制一：显式消息上下文 (Explicit Message Context)

在 Phase 3 中，`reply` 不再是全局关键字。开发者可以通过在 `on` 块中声明一个**显式参数**（名称自定义，通常为 `ctx` 或 `origin`），来获取当前消息的路由元数据与回执通道。

#### 2.1 上下文对象的内部结构

编译器在底层会提供一个标准的内置类型 `MessageContext`，其物理结构包含：

```auto
// 编译器内置类型 (开发者无需手写定义)
type MessageContext {
    sender_id ?u64      // 发送方的系统唯一标识 (可能为空)
    trace_id string     // 链路追踪 ID
    is_ask bool         // 标识此消息是否需要回执 (由 ask 还是 send 触发)
    
    // 万能回复方法 (编译器自动校验入参类型是否符合发送方期待)
    fn reply(payload Any) void 
}

```

#### 2.2 语法的自由度与对称性

开发者可以自由决定是否需要接收这个上下文。

**模式 A：需要回复或校验来源（显式声明参数）**

```auto
task NodeWorker {
    // 声明 ctx 获取上下文
    on(ctx) {
        "ping" => { 
            // 完美对称：名词.动词
            ctx.reply("pong") 
        }
        "get_data" => {
            is ctx.sender_id {
                (id) => { print("Request from ${id}") }
                !    => { print("Anonymous request") }
            }
            ctx.reply(db.query())
        }
    }
}

```

**模式 B：纯粹的数据消费（省略参数，极简模式）**
如果 Task 只是单向接收数据，无需回复，直接省略参数即可，保持作用域的绝对干净。

```auto
task Logger {
    on {
        msg string => { write_to_disk(msg) }
    }
}

```

**模式 C：上下文传递（Context Passing）**
因为 `ctx` 是显式声明的第一公民变量，开发者可以极其安全地将其作为参数传递给其他私有函数，实现复杂的异步逻辑拆分。

```auto
on(req) {
    "complex_task" => { self.handle_heavy_job(req) }
}

fn handle_heavy_job(req MessageContext) {
    let res = compute()
    req.reply(res) // 在子函数中完成回复！
}

```

---

### 3. 核心机制二：隐式联合体与全能模式匹配

开发者不再需要预先定义 `enum`。编译器通过 AST 双遍扫描，自动合成消息信封，并支持极其强大的匹配模式。

#### 3.1 字面量精确匹配 (Literal Match)

直接对状态、指令字面量进行路由。

```auto
on(ctx) {
    "start" => { engine.ignite(); ctx.reply("ok") }
    404     => { print("Received error code 404") }
}

```

#### 3.2 泛类型捕获 (Type Binding)

按数据类型进行拦截，并将其绑定到局部变量。

```auto
on {
    // 捕获普通类型
    url string => { http.download(url) }
    
    // 捕获自定义对象
    u User     => { db.save(u) }
}

```

#### 3.3 守卫表达式 (Guard Clauses)

结合变量捕获与条件判断，扁平化业务逻辑。

```auto
on(ctx) {
    amount int if amount > 10000 => { ctx.reply("Need Approval") }
    amount int                   => { ctx.reply("Auto Approved") }
}

```

---

### 4. 严格的静态类型拦截 (Static Type Checking)

隐式联合体不仅是为了爽，更是为了**安全**。
如果在代码任何地方调用 `NodeWorker.send()` 或 `.ask()`，编译器的语义分析器会严格核对参数是否属于该 Task `on` 块中声明的类型或字面量。

```auto
fn main() ! {
    // 合法：隐式装箱为 String 变体
    NodeWorker.send("ping").? 
    
    // 致命错误！编译期拦截！
    // Error: Task 'NodeWorker' does not accept messages of type 'bool'.
    NodeWorker.send(true).? 
}

```

---

### 5. 底层降级规范 (Zero-cost Lowering)

**5.1 a2rs (Rust 后端) 降级：**
将隐式联合体降级为强类型 `enum`，并将 `MessageContext` 作为包裹传递。

```rust
// 编译器自动生成的隐式信封
pub enum NodeWorkerPayload {
    LiteralPing,
    LiteralGetData,
}

pub struct NodeWorkerEnvelope {
    pub context: MessageContext, // 显式的上下文对象
    pub payload: NodeWorkerPayload,
}

// AST 翻译
match env.payload {
    LiteralPing => {
        let mut ctx = env.context; // 参数绑定
        ctx.reply("pong");         // 方法调用
    }
}

```

**5.2 a2c (C / 裸机后端) 降级：**
利用结构体和函数指针实现轻量级上下文。

```c
typedef struct {
    uint64_t sender_id;
    bool has_sender_id;
    QueueHandle_t reply_queue;
} AutoMessageContext;

// reply 方法被降级为传入上下文指针的 C 函数调用
Auto_ContextReply(&ctx, "pong");

```

---

### 6. 阶段验收标准 (Acceptance Criteria)

开发团队在此阶段必须满足以下硬性工程指标：

1. **废除 reply 关键字**：AST 解析器从保留字列表中移除 `reply`。所有的回复必须通过被解析为方法调用的 `ctx.reply()` 执行。
2. **上下文作用域隔离**：确保声明了 `on(ctx)` 后，`ctx` 变量仅在当前匹配分支的大括号内有效，防止变量逃逸。
3. **双遍扫描正确性**：编译器必须能准确推导出联合体，并在 `Task.ask("ping").await` 处，自动推导出 `ctx.reply(T)` 中的 `T` 的类型，实现跨 Task 的静态类型安全。

---
