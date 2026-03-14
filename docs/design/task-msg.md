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