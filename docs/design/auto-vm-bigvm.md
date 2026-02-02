这是一份关于 **AutoVM (BigVM Edition)** 的详细架构设计文档。

作为资深架构师，我将 BigVM 定义为 **MicroVM 的“数字孪生 (Digital Twin)”**。它不仅是一个解释器，更是 AutoLive 开发体验的核心引擎。它必须在 Rust 的安全包裹下，精确模拟 C/MCU 的底层行为。

---

# AutoVM (BigVM) Architecture Design Document

**Project:** Auto Language Runtime
**Target Platform:** Windows / Linux / macOS (Written in Rust)
**Version:** 1.0
**Status:** Blueprint for Implementation

## 1. Executive Summary (执行摘要)

**BigVM** 是 Auto 语言在 PC 端的参考实现运行时。
它的核心使命不是追求 PC 上的极致性能，而是**精确模拟** MicroVM 在嵌入式 MCU 上的行为。它是连接 AIE (增量编译引擎) 与 IDE 调试器的桥梁。

* **Role**: 开发环境仿真器、调试器后端、AIE 增量更新的接收端。
* **Key Characteristic**: **同构性 (Isomorphism)**。BigVM 的内存布局、栈操作、溢出行为必须与 MicroVM 保持逻辑一致，确保“在 PC 上跑通的代码，烧录到 MCU 必定能跑”。

---

## 2. Architecture Overview (架构概览)

BigVM 由四个核心子系统组成：

1. **Virtual Hardware (虚拟硬件层)**: 模拟 MCU 的 Flash (Code) 和 RAM (Stack/Heap)。
2. **Execution Engine (执行引擎)**: 一个基于 Rust 的 Dispatch Loop，执行 ABC (Auto Bytecode)。
3. **Hot Linker (热链接器)**: 负责接收 AIE 发来的 Patch，并模拟 RAM 覆盖和 GOT 更新。
4. **FFI Gateway (外部接口)**: 模拟 MCU 的 C 函数调用。

---

## 3. Core Design: Virtual Hardware (内存模型)

为了保证同构性，BigVM 不能直接使用 Rust 的 `Heap`，必须手动管理两块巨大的 `Vec<u8>`。

### 3.1 The "Flash" (Code Space)

模拟 MCU 的只读存储区。AIE 生成的字节码会被“烧录”到这里。

```rust
struct VirtualFlash {
    // 模拟 Flash 空间，例如 1MB
    // 所有的 ip (Instruction Pointer) 都是这个 slice 的索引
    memory: Vec<u8>, 
    
    // 映射表：FragID -> Flash Address
    // 用于 AIE 告诉 VM：“主函数在 Flash 的 0x1000 处”
    symbol_map: HashMap<FragId, usize>, 
}

```

### 3.2 The "RAM" (Data Space)

模拟 MCU 的 SRAM。这是 BigVM 与传统解释器最大的不同——**强类型的 Raw Memory**。

```rust
// 模拟 MCU 上的 32位 宽度的字
#[derive(Clone, Copy)]
union Word {
    i: i32,
    u: u32,
    f: f32,
    // BigVM 特供：为了调试，我们可以包含元数据，
    // 但在 release 模式下应强制对齐到 4 字节行为
    #[cfg(debug_assertions)]
    debug_ptr: usize, 
}

struct VirtualRAM {
    // 整个 RAM 是一块连续内存，例如 64KB
    raw: Vec<Word>, 
    
    // 寄存器模拟
    sp: usize, // Stack Pointer (Index)
    bp: usize, // Base Pointer (Index)
}

```

---

## 4. Execution Engine (执行引擎)

### 4.1 The Loop

BigVM 使用 Rust 编写核心循环。为了性能和模拟精确度，不使用递归函数，而是单循环。

```rust
impl BigVM {
    pub fn run(&mut self) -> Result<(), VMError> {
        loop {
            // 1. Fetch
            let op = self.flash.read_u8(self.ip);
            self.ip += 1;

            // 2. Decode & Execute
            match op {
                OP_CONST_I32 => {
                    let val = self.flash.read_i32(self.ip);
                    self.ip += 4;
                    self.stack_push(Word { i: val });
                },
                OP_ADD_I32 => {
                    let b = self.stack_pop().i;
                    let a = self.stack_pop().i;
                    // 模拟 C 的溢出行为 (Wrapping)
                    self.stack_push(Word { i: a.wrapping_add(b) });
                },
                OP_CALL => {
                    // 模拟函数调用的 Stack Frame 创建
                    self.push_frame(); 
                    // 跳转逻辑...
                },
                // ...
            }
        }
    }
}

```

### 4.2 Isomorphic Trap (同构陷阱)

BigVM 必须捕获那些在 Rust 里是 Panic 但在 C 里是 UB (Undefined Behavior) 的行为，并将其标准化。

* **栈溢出**: BigVM 必须在 `sp >= RAM_SIZE` 时抛出明确的 `StackOverflow` 错误，模拟 MCU 的 HardFault。
* **除以零**: 必须检查并抛出异常，而不是让 Rust Panic。

---

## 5. Integration with AIE: Hot Reload Simulation

这是 BigVM 最关键的特性：模拟 **AutoLive** 机制。

### 5.1 The "Hot Zone"

在 `VirtualRAM` 中，预留一块区域作为 **Hot Zone**（模拟 MCU 的 Heap 或特定 RAM 区）。

### 5.2 Patch Protocol

当 AIE (增量编译器) 完成编译后，它会发送一个 `Patch` 给 BigVM：

```rust
struct Patch {
    frag_id: u32,
    code: Vec<u8>,
    relocations: Vec<RelocEntry>,
}

```

### 5.3 The Loader Logic

BigVM 收到 Patch 后的行为：

1. **Alloc**: 在 `VirtualRAM` 的 Hot Zone 中分配一块空间 `addr`。
2. **Write**: 将 `patch.code` 写入这块 RAM。
* *注意*：此时代码在 RAM 里，而之前的代码在 Flash 里。VM 必须支持从 RAM 取指（Execute from RAM）。


3. **Link**: 遍历 `VirtualRAM` 中的 **GOT (Global Offset Table)**，将 `frag_id` 对应的函数指针更新为 `addr`。
4. **Effect**: 下次执行 `OP_CALL_INDIRECT [got_index]` 时，自动跳入 RAM 执行新代码。

---

## 6. FFI Simulation (ABI 垫片)

为了模拟 MicroVM 里的 C 函数调用，BigVM 使用 Rust 闭包来模拟 C 函数。

### 6.1 Shim Registry

```rust
type ShimFunc = Box<dyn Fn(&mut VirtualRAM, usize) -> Result<(), Trap>>;

struct NativeInterface {
    // 映射: "lcd_draw" -> Rust Closure
    registry: HashMap<String, ShimFunc>,
}

```

### 6.2 The Call

当执行 `OP_CALL_NATIVE id` 时：

1. VM 暂停字节码执行。
2. VM 查找 `registry` 拿到 Rust 闭包。
3. **关键点**: VM 将自己的 `RAM` 和当前的 `SP` 传给闭包。
4. **模拟 C**: 闭包通过 `ram.read(sp - 1)` 来获取参数，就像 C 代码访问栈一样。

---

## 7. Debugging & Observability (可观测性)

作为开发工具，BigVM 必须提供 MicroVM 无法提供的“上帝视角”。

### 7.1 Instruction Tracing

一个可选的 `trace` 开关。开启时，每执行一条指令，打印：
`[IP:0x0040] OP_ADD | Stack: [..., 10, 20] -> [..., 30]`

### 7.2 Reverse Debugging Support (Time Travel)

由于 BigVM 的状态完全封闭在 `VirtualFlash` 和 `VirtualRAM` 两个 Vec 中。
我们可以轻松实现 **快照 (Snapshotting)**：

* 每执行 N 条指令，clone 一份 `VirtualRAM`。
* 用户点击“后退”时，直接恢复旧的 `VirtualRAM`。
这是 MicroVM 绝对做不到的高级功能。

---

## 8. Implementation Roadmap (实施路径)

建议分为三个阶段实现 BigVM：

### Phase 1: The Core (纯计算)

* 实现 `VirtualStack` 和基础整数指令 (`ADD`, `SUB`, `JMP`)。
* 跑通斐波那契数列 (Fibonacci) 的字节码。
* 验证栈操作与 C 的逻辑一致性。

### Phase 2: The System (内存与调用)

* 引入 `VirtualFlash` 和 `VirtualRAM` 的区分。
* 实现 `CALL` / `RET` 和栈帧管理。
* 实现 `OP_LOAD_LOCAL` (基于 BP 的寻址)。

### Phase 3: The Live (热重载与 FFI)

* 实现 GOT 表机制。
* 对接 AIE，接收 Patch 并写入 RAM。
* 实现 Rust 版的 FFI Shim 系统，模拟 `print` 等标准库函数。

---

## 9. Conclusion

AutoVM (BigVM) 不是一个简单的模拟器，它是 Auto 语言**开发体验的基石**。
通过在 Rust 中严格模拟 MCU 的内存限制和底层行为，我们不仅能保证代码的跨平台一致性，还能利用 PC 的强大资源实现“时间旅行调试”和“亚秒级热更”。

**下一步行动建议**：
启动 Phase 1，定义 `enum OpCode` 和 `struct VM`，先让 `1 + 1 = 2` 在 Rust 里跑起来。