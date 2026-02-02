这是一个基于我们所有深度讨论（AIE 架构、Task+Msg 异步模型、统一 FFI 语法）汇总而成的 **AutoVM (BigVM Edition)** 最终设计蓝图。

这份文档将作为 BigVM 开发的权威指导。

---

# AutoVM (BigVM) High-Level Design Document

**Project:** Auto Language Runtime (PC Host Edition)
**Target Platform:** Windows / Linux / macOS (Rust Implementation)
**Architecture Style:** Async Runtime + Polyglot Interpreter
**Version:** 2.0 (Integrated)

## 1. Executive Summary (执行摘要)

**BigVM** 是 Auto 语言在 PC 端的参考实现。它不仅仅是一个解释器，更是一个 **"Polyglot Concurrency Runtime" (多语言并发运行时)**。

它肩负两个核心使命：

1. **MicroVM 的数字孪生**：精确模拟嵌入式环境（MCU）的内存模型、栈行为和实时性，配合 AIE 实现亚秒级热重载（AutoLive）。
2. **超级胶水层**：利用 Task+Msg 并发模型和统一 FFI 接口，无缝编排 Rust、C、Python、JS 等异构生态，实现“一次编写，到处连接”。

---

## 2. System Architecture (系统架构)

BigVM 采用 **"Hub-and-Spoke" (轮毂)** 架构，核心是异步的执行引擎，外围是动态插件系统。

### 2.1 Core Components

1. **Virtual Hardware**: 模拟 MCU 的 `VirtualFlash` (代码区) 和 `VirtualRAM` (数据区/栈)。
2. **Scheduler (M:N)**: 基于 Tokio 构建的用户态调度器，管理成千上万个 Auto Task。
3. **FFI Plugin Manager**: 负责动态加载外部语言运行时（C/Python/JS），并管理跨语言对象句柄。
4. **Hot Linker**: 接收 AIE 发送的增量补丁 (Patch)，动态修补虚拟内存。

---

## 3. Concurrency Model: Task + Message (并发模型)

BigVM 采用 **"Virtual Stackful Tasks on Green Threads"** 模型。
用户编写的是同步直线代码，VM 底层自动进行异步调度。

### 3.1 Task Mapping (M:N)

* **Auto Task**: 拥有独立的虚拟栈 (`Vec<Value>`)，存储在 Heap 上。
* **Rust Runtime**: 使用 **Tokio** 作为底座。
* **Mapping**: 一个 Auto Task 包装为一个 `tokio::task` (Future)。
* Auto 代码：`spawn(handler)`
* Rust 实现：`tokio::spawn(async move { vm.run_loop(handler_id).await })`



### 3.2 Async I/O & Preemption (异步与抢占)

Auto 语言本身没有 `async/await` 关键字。所有的 IO 操作（Channel, Sleep, FFI Call）都是 **Yield Points (让出点)**。

* **Mechanism**:
当解释器遇到 `OP_RECV` (接收消息) 或 `OP_CALL_ASYNC` (调用 JS Promise) 时：
1. VM 保存当前 Auto Task 的 `IP` 和 `Stack`。
2. Rust 代码执行 `.await`，让出 Tokio 线程控制权。
3. 当外部事件（消息到达 / JS Promise Resolve）发生时，Tokio 唤醒该 Task，VM 恢复上下文继续执行。



### 3.3 MicroVM Compatibility

* **BigVM**: 利用 Tokio 实现高并发（适合网络/脚本）。
* **MicroVM**: 利用 **RTOS (FreeRTOS/Zephyr)** 实现 1:1 映射（适合实时控制）。
* **兼容性**: 上层语法完全一致 (`spawn`, `send`, `recv`)。

---

## 4. The Unified FFI System (统一外部接口)

这是 BigVM 的核心竞争力。通过统一的语法和插件机制，打通异构语言壁垒。

### 4.1 FFI Syntax (大一统语法)

VM 支持三种引入模式，映射到不同的加载器：

```auto
// 1. C/Rust (Native Library) - 强类型
use.c "stdio" as io;
#[c] fn getchar() -> i32;

// 2. Python (Scripting) - 动态类型
use.py "numpy" as np; // 得到一个 handle

// 3. JavaScript (Web/Async) - 异步集成
use.js "axios" as axios;

```

### 4.2 Resource Handle Table (资源句柄表)

由于 AutoVM 栈槽只有 32 位，无法直接存 `PyObject*` 或 `JSContext*`。
BigVM 维护一张全局 **Handle Table**：

| HandleID (i32) | Real Pointer (void*) | Plugin Type | Destructor |
| --- | --- | --- | --- |
| 100 | 0x7F... (PyObject) | Python | `Py_DECREF` |
| 101 | 0xAA... (C FILE*) | C-Stdio | `fclose` |

* **Auto 视角**: 持有一个 `i32` 整数 (100)。
* **Plugin 视角**: 接收 `handle_id`，查询表拿到真实指针。

### 4.3 Plugin Architecture (插件架构)

AutoVM 核心不包含 Python/JS 引擎，而是通过 DLL/SO 动态加载。
定义标准 **FPI (Foreign Plugin Interface)**：

```rust
trait Plugin {
    // 1. 调用: Auto -> Plugin
    fn call(&self, func_name: &str, args: &[Value]) -> Result<Value>;
    
    // 2. 属性获取: Auto -> Plugin
    fn get_attr(&self, handle: i32, attr: &str) -> Result<Value>;
    
    // 3. 异步等待: Plugin -> Auto (Future)
    // 用于 JS Promise 适配
    fn await_promise(&self, handle: i32) -> JsFuture;
}

```

---

## 5. Execution Engine (执行引擎)

VM 的核心循环是一个 `async fn`，以支持与 Tokio 的协作。

### 5.1 The Async Loop

```rust
async fn run_loop(&mut self) {
    loop {
        let op = self.fetch();
        match op {
            OP_ADD => self.do_add(), // 纯计算，快速执行
            
            OP_CALL_NATIVE => {
                // 1. 查找插件
                let (plugin, func_id) = self.resolve_native(op.arg);
                // 2. 执行 (可能是耗时的)
                let res = plugin.invoke(func_id, &self.stack).await; // Point of Yield!
                self.push(res);
            },
            
            OP_RECV => {
                // 1. 异步等待通道消息
                let msg = self.channel.recv().await; // Point of Yield!
                self.push(msg);
            }
            
            // 协作式调度检查
            _ => if self.instructions++ > 1000 { tokio::task::yield_now().await; }
        }
    }
}

```

---

## 6. AIE Integration (增量编译与热重载)

BigVM 作为 AIE 的客户端，必须支持 **Hot Patching**。

### 6.1 Patch Application

AIE 编译出的不再是完整的 binary，而是 `Patch` 包：

1. **Code**: 新函数的字节码。
2. **Meta**: `FragID` -> `InterfaceHash`。

### 6.2 Hot Reload Process

当 BigVM 收到 Patch：

1. **Pause**: 暂停所有处于 **Safe Point**（循环边界或 IO 等待）的 Task。
2. **Load**: 将新字节码写入 `VirtualRAM` 的 **Hot Zone**（模拟 RAM 执行）。
3. **Link**: 更新 `Global Offset Table (GOT)`，将旧函数的 ID 指向新地址。
4. **Resume**: 恢复 Task 执行。下次调用该函数时，自动跳转到新代码。

---

## 7. Memory Model (数字孪生)

为了保证代码在 MCU 上能跑，BigVM 必须“假装”自己很穷。

### 7.1 Address Space

* **Code Space (Flash)**: 只读 `Vec<u8>`。AIE 输出烧录于此。
* **Data Space (RAM)**: 读写 `Vec<u32>`。
* **Stack**: 只有在 MicroVM 模式下开启严格的 Stack 限制检测。
* **Heap**: 简单的 Bump Pointer 分配器或 Buddy System 模拟。



---

## 8. Implementation Roadmap (执行路线图)

### Phase 1: The Core (Rust Async Base)

* **Goal**: 跑通基本的 Task + Msg 模型。
* **Task**: 实现 `VirtualStack`, `OpCode` 定义。
* **Runtime**: 集成 `Tokio`，实现 `spawn` 和 `channel` 指令。
* **Verify**: 写一个“生产者-消费者”的 Auto 字节码 Demo。

### Phase 2: The Isomorphism (Hardware Sim)

* **Goal**: 建立 Flash/RAM 内存模型。
* **Task**: 实现 `VirtualFlash`, `VirtualRAM`, `BP/SP` 寄存器模拟。
* **Task**: 实现 C 语言风格的 Stack Frame 管理。

### Phase 3: The Polyglot (FFI Plugins)

* **Goal**: 打通 C 和 Python。
* **Task**: 定义 `FPI` 接口。
* **Task**: 实现 `plugin_c` (基于 `libloading` + `libffi`)。
* **Task**: 实现 `plugin_python` (基于 `PyO3`) 和 Handle Table。

### Phase 4: The Tooling (AIE Interface)

* **Goal**: 支持热重载。
* **Task**: 实现 Patch Loader 和 GOT 表。
* **Task**: 实现简单的 Debugger Hook (Trace/Break)。

---

## 9. Conclusion (总结)

这个设计方案完美融合了 **Go 的并发体验**、**Rust 的安全性能**、**Python 的生态广度** 以及 **MCU 的实时约束**。

* **对开发者**：Auto 是一个“写起来像脚本，跑起来像系统语言”的神奇工具。
* **对架构**：BigVM 既是 MicroVM 的仿真器，又是强大的 PC 端胶水运行时。

**Ready to Build.** 建议从 Phase 1 开始，先用 Tokio 把基础的虚拟栈和调度跑起来。