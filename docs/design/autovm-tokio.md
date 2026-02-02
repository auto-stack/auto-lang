这是一份针对 **AutoVM (BigVM Edition)** 的核心并发架构设计文档。

这份文档的确立意味着 AutoVM 将从一个简单的“循环解释器”升级为一个现代化的、基于 **Tokio** 的 **M:N 异步运行时**。这将是 Auto 语言在 PC 端实现高并发高性能的基石。

---

# AutoVM (BigVM) Asynchronous Concurrency Architecture

**Design Document v1.0**
**Target**: Rust (Tokio) Implementation
**Model**: Virtual Stackful Tasks on Green Threads (M:N)

## 1. Executive Summary (核心摘要)

本设计旨在为 AutoVM (BigVM) 引入原生的 **Task/Message** 并发模型。
通过利用 Rust 的 **Tokio** 异步运行时作为底层引擎，我们将 Auto 语言中的轻量级 `Task` 映射为 Rust 的 `Future`。这使得 Auto 语言能够以同步的直线代码风格（无 `async/await` 关键字），享受底层的异步非阻塞 I/O 能力。

**核心映射关系：**

* **Auto Task**  **Tokio Task (Future)**
* **Auto Channel**  **Tokio MPSC Channel**
* **Auto I/O (FFI)**  **Rust Async I/O**

---

## 2. System Architecture (系统架构)

### 2.1 关键组件

1. **Task Registry (任务注册表)**: 全局管理所有活跃任务的状态、元数据（用于调试/GC）。
2. **Executor (执行器)**: 基于 `async fn` 的字节码解释循环。
3. **Mailbox (邮箱)**: 基于 `tokio::sync::mpsc` 的消息通道。
4. **Scheduler (调度器)**: 直接复用 Tokio 的 Work-Stealing 调度器。

---

## 3. Data Structures (数据结构设计)

我们需要将传统的单体 VM 拆解为针对每个任务的独立上下文。

### 3.1 The Task Struct

每个 Auto 任务拥有自己的虚拟栈和状态。

```rust
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use std::sync::Arc;

// 任务 ID
type TaskId = u64;

// 任务状态
enum TaskStatus {
    Ready,
    Running,
    Waiting(String), // 等待原因 (如 "channel_recv")
    Terminated,
}

// 核心任务结构 (存放在 Heap 上，被 Arc 包裹)
struct AutoTask {
    id: TaskId,
    // 虚拟栈 (独立拥有)
    stack: Vec<Value>,
    // 调用栈帧
    frames: Vec<CallFrame>,
    // 指令指针
    ip: usize,
    // 任务状态
    status: TaskStatus,
}

// 虚拟机全局句柄
struct BigVM {
    // 任务注册表 (用于调试、监控、强制杀死任务)
    // Key: TaskId
    tasks: DashMap<TaskId, Arc<Mutex<AutoTask>>>,
    
    // 全局 ID 生成器
    id_gen: AtomicU64,
    
    // 字节码存储 (只读，所有任务共享)
    code_store: Arc<CodeStore>,
    
    // FFI 插件管理器 (线程安全)
    plugins: Arc<PluginManager>,
}

```

### 3.2 The Channel Struct

Auto 的 Channel 底层直接映射到 Tokio。

```rust
struct AutoChannel {
    tx: mpsc::Sender<Value>,
    rx: Arc<Mutex<mpsc::Receiver<Value>>>, // 需加锁因为可能有多个 Task 同时读
}

```

---

## 4. Execution Model: The Async Loop (执行模型)

这是架构的核心。解释器不再是一个阻塞的 `while(true)`，而是一个可以被 `await` 挂起的状态机。

### 4.1 Spawn Logic (启动任务)

当解析到 `OP_SPAWN <func_id>` 指令时：

```rust
impl BigVM {
    fn spawn_task(&self, func_id: FragId, args: Vec<Value>) {
        let task_id = self.id_gen.fetch_add(1, Ordering::SeqCst);
        
        // 1. 创建任务上下文
        let task = Arc::new(Mutex::new(AutoTask::new(task_id, func_id, args)));
        self.tasks.insert(task_id, task.clone());

        // 2. 克隆环境引用 (以便传入 Future)
        let vm_ref = self.clone(); 
        
        // 3. 提交给 Tokio 调度器
        tokio::spawn(async move {
            // 进入异步执行循环
            vm_ref.run_task_loop(task).await;
        });
    }
}

```

### 4.2 The Interpretation Loop (异步解释循环)

这是原本的 `vm.run()` 的异步改造版。

```rust
impl BigVM {
    async fn run_task_loop(&self, task_state: Arc<Mutex<AutoTask>>) {
        loop {
            // A. 获取锁，执行一段 CPU 密集逻辑
            // ----------------------------------------------------
            let mut task = task_state.lock().await;
            
            // "协作式调度"：每次只执行 N 条指令，防止死循环卡死线程
            const BUDGET: usize = 128; 
            let mut steps = 0;
            let mut yield_reason = None;

            while steps < BUDGET {
                let op = self.code_store.fetch(task.ip);
                task.ip += 1;

                match op {
                    // CPU 指令 (同步执行)
                    OP_ADD => self.do_add(&mut task),
                    
                    // 遇到 IO 指令，必须跳出锁的范围去 await
                    OP_RECV => {
                        yield_reason = Some(YieldReason::Recv);
                        break; 
                    },
                    
                    OP_CALL_ASYNC => {
                        yield_reason = Some(YieldReason::CallAsync);
                        break;
                    },

                    OP_RETURN => {
                        if task.frames.is_empty() {
                            // 任务结束
                            return; 
                        }
                    }
                    // ... 其他指令
                }
                steps += 1;
            }
            
            // 释放锁，归还 Task 状态的所有权
            drop(task); 
            // ----------------------------------------------------


            // B. 处理异步操作 (无锁状态，不阻塞 Tokio 线程)
            // ----------------------------------------------------
            match yield_reason {
                // 情况 1: 时间片用完，主动让出
                None => {
                    tokio::task::yield_now().await;
                },

                // 情况 2: 等待 Channel 消息
                Some(YieldReason::Recv) => {
                    // 从栈顶拿到 Channel Handle
                    let chan = self.get_channel_from_stack(&task_state).await;
                    
                    // 这里发生了真正的异步等待！
                    // Tokio 线程会去执行别的 Auto Task
                    let msg = chan.recv().await; 
                    
                    // 醒来后，把结果压栈
                    let mut task = task_state.lock().await;
                    task.stack.push(msg);
                },

                // 情况 3: 等待 FFI (如 JS Promise)
                Some(YieldReason::CallAsync) => {
                     let future = self.get_plugin_future(&task_state).await;
                     let res = future.await; // 等待 JS 完成
                     
                     let mut task = task_state.lock().await;
                     task.stack.push(res);
                }
            }
        }
    }
}

```

---

## 5. Key Instructions Design (关键指令)

### 5.1 `OP_RECV` (非阻塞接收)

* **Auto 语义**: 阻塞当前 Task，直到收到消息。
* **BigVM 实现**:
1. VM 暂停指令执行。
2. 调用 `tokio_channel.recv().await`。
3. Rust 编译器将其编译为状态机挂起。
4. 物理线程释放。



### 5.2 `OP_SEND` (非阻塞发送)

* **Auto 语义**: 发送消息，如果不满则立即返回，满则阻塞。
* **BigVM 实现**: `tokio_channel.send(msg).await`。

### 5.3 `OP_SLEEP` (睡眠)

* **Auto 语义**: `sleep(1000)`。
* **BigVM 实现**: `tokio::time::sleep(Duration::from_ms(1000)).await`。
* *注意*：这不会阻塞物理线程，只会让当前的 Auto Task 挂起 1 秒。



---

## 6. Compatibility Strategy (MicroVM 兼容性)

虽然 BigVM 用了 Tokio，MicroVM 用了 RTOS，但**指令集 (ISA) 是一致的**。

| Auto OpCode | BigVM (Rust/Tokio) Implementation | MicroVM (C/RTOS) Implementation |
| --- | --- | --- |
| `OP_SPAWN` | `tokio::spawn(async_loop)` | `xTaskCreate(c_loop)` |
| `OP_RECV` | `rx.recv().await` | `xQueueReceive(..., portMAX_DELAY)` |
| `OP_SEND` | `tx.send().await` | `xQueueSend(...)` |
| `OP_SLEEP` | `tokio::time::sleep().await` | `vTaskDelay(...)` |
| `OP_ADD` | `stack[sp-1] + stack[sp]` | `stack[sp-1] + stack[sp]` |

**结论**：只要保证 `OP_ADD` 等计算逻辑共享同一套代码（例如通过 `no_std` 的 Rust Core 或 C 代码逻辑），调度逻辑的差异被封装在 OpCode 的 handler 里，完全不影响兼容性。

---

## 7. Implementation Steps (实施步骤)

建议开发者（及 AI）按以下顺序重构代码：

1. **Step 1: Dependency Injection**
* 引入 `tokio = { version = "1", features = ["full"] }`。
* 将 `main` 函数改为 `#[tokio::main] async fn main()`.


2. **Step 2: Struct Refactoring**
* 将原有的单体 `struct VM` 拆分为 `struct BigVM` (Runtime) 和 `struct AutoTask` (State)。
* 实现 `Task` 的 `lock()` 机制。


3. **Step 3: The Async Loop**
* 实现 `run_task_loop` 骨架。
* 先只支持 `OP_PRINT` 和 `OP_SLEEP`。
* **验证点**: 启动两个 Task，一个每 1秒打印 "A"，另一个每 0.5秒打印 "B"。如果控制台能看到交替输出，说明 Tokio 调度成功。


4. **Step 4: Channel Implementation**
* 实现 `OP_MAKE_CHAN`, `OP_SEND`, `OP_RECV`。
* **验证点**: 生产者-消费者模型。


5. **Step 5: Integration**
* 将之前的数学计算、函数调用逻辑搬运到新的 Loop 中。



---

## 8. Summary

这个设计将 AutoVM 从一个“玩具解释器”提升为了一个“工业级运行时”。
它利用 Tokio 解决了并发和 IO 的难题，同时保留了 Auto 语言简洁的同步语法。这是实现 Auto "System Glue" 愿景的必经之路。

**Next Action for AI:** Start refactoring `struct VM` into `AutoTask` and `BigVM` based on Section 3.