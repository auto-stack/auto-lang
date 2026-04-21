要让 AutoVM 从“黑盒字节码解释器”变成可高效调试的“白盒虚拟机”，需要围绕**可读性、可追踪性、可交互性、可回放性**四个维度增加基础设施。单纯加一个文本汇编格式只是解决“看见代码”的问题，但真正定位 bug 还需要更深层的支持。

以下是建议添加的基础设施清单，按优先级和依赖关系排列。

### 1. 核心基础：文本汇编格式（ABC Assembly）

这是**所有调试能力的基石**，必须首先实现。

#### 1.1 格式设计原则
- **双向转换**：必须支持 **字节码 ↔ 文本汇编** 的无损互转。
- **包含元数据**：除了指令，必须包含调试符号、行号映射、局部变量名。
- **人类友好 + 机器可解析**：采用 S-表达式或类似 WASM 的 S 式风格，易于手写也易于工具链处理。

**示例格式：**
```lisp
;; function: add (i32, i32) -> i32
(func $add (param $a i32) (param $b i32) (result i32)
  (local $tmp i32)
  ;; line 42: let tmp = a + b;
  i32.add
  local.get $a
  local.get $b
  i32.store local=$tmp
  ;; line 43: return tmp;
  local.get $tmp
  return
)
```

#### 1.2 配套工具链
- **汇编器** (`auto-as`)：将 `.abcasm` 编译为 `.abc` 字节码文件。
- **反汇编器** (`auto-dis`)：将 `.abc` 反编译为 `.abcasm`，供肉眼检查。
- **VM 内联打印**：启动 VM 时加 `--dump-bytecode` 选项，在控制台输出带 PC 地址的汇编。

### 2. 调试信息生成与嵌入（DWARF 风格轻量实现）

没有调试信息，汇编里看到的全是 `local.get $0` 这种无意义编号。

#### 2.1 需要嵌入到字节码的调试节
- **源映射表**：每条指令对应的源文件路径、行号、列号。
- **变量命名表**：将寄存器/局部变量索引映射到 Auto 源码中的变量名。
- **类型签名**：每个值在栈上时的具体类型（便于调试器显示带类型的值）。

#### 2.2 如何在 VM 崩溃时利用这些信息
当 VM 遇到 `StackUnderflow` 或 `TypeMismatch` 时，不要只打印：
```
Panic: Stack underflow at PC 0x023a
```
而要打印：
```
Panic: Stack underflow at PC 0x023a (function 'median', line 15:9)
Stack trace:
  #0 median (data: Vec<i32>) at src/main.auto:15:9
  #1 main () at src/main.auto:22:5
```

### 3. 交互式调试器（AutoDBG）

类似 GDB/LLDB 的命令行调试器，**直接在 VM 进程中集成**。

#### 3.1 核心命令集
- `break <func>` / `break <file:line>`
- `watch <local_name>` / `watch *<address>` （如果 VM 有内存抽象）
- `step` / `next` / `finish`
- `info locals` / `info stack` （显示当前栈帧的变量和操作数栈）
- `print <expr>` （需要集成一个微小的表达式求值器，利用调试信息）

#### 3.2 实现架构
由于 AutoVM 是 Rust 写的，可以利用 Rust 的 `nix::ptrace` 或直接**在同一进程内实现调试循环**（推荐后者，因为 VM 本身是解释器，可以轻松挂起）。

**伪代码架构：**
```rust
impl Vm {
    fn run(&mut self) -> Result<Value, VmError> {
        loop {
            // 检查断点
            if self.debugger.check_breakpoint(self.current_pc()) {
                self.debugger.prompt(self); // 进入 REPL，阻塞执行
            }
            // 执行当前指令...
        }
    }
}
```

### 4. 执行追踪与结构化日志（Trace Log）

这是目前“插入 debug 语句”的**自动化、结构化替代方案**。

#### 4.1 指令级 Trace
VM 增加 `--trace` 模式，输出 JSON 格式的逐指令日志（而非纯文本刷屏），方便用工具分析。

**示例单步 Trace 记录：**
```json
{
  "pc": 42,
  "func": "median",
  "inst": "i32.add",
  "line": 15,
  "stack_before": ["i32(10)", "i32(20)"],
  "stack_after": ["i32(30)"],
  "locals": { "$tmp": "i32(5)" }
}
```

#### 4.2 可视化工具
提供简单的 Web 界面或 TUI 工具（基于 `ratatui`），加载 JSON Trace 文件，可**单步回放**、查看栈变化曲线。这对定位“为什么栈少了一个值”极为有效。

### 5. 汇编测试框架（`.abcasm` 单元测试）

既然有了文本汇编格式，就可以直接在汇编层面写测试，隔离 Auto 编译器的前端问题。

#### 5.1 测试 DSL
```lisp
;; test/add.abcasm
(module
  (func $add (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
    return)

  (assert_eq (invoke $add (i32.const 2) (i32.const 3)) (i32.const 5))
  (assert_trap (invoke $add (i32.const 2147483647) (i32.const 1)) "overflow")
)
```

#### 5.2 集成进构建系统
`auto test --vm-only` 只运行 VM 的汇编测试，快速验证指令集实现的正确性，而不依赖上层编译器。

### 6. 栈操作静态验证器（Verifier）

在 VM 加载字节码时，增加一个**轻量级静态分析 Pass**，提前捕获栈平衡错误。

#### 6.1 验证内容
- **类型栈高度一致性**：确保每条指令执行前后的操作数栈高度变化符合指令签名。
- **局部变量作用域检查**：避免使用未初始化的局部变量。
- **控制流完整性**：确保跳转目标有效。

#### 6.2 效果
很多“运行时 push/pop 错乱”的 bug 其实在加载阶段就能被检测出来，并给出精确的错误位置（例如：“函数 `foo` 在 PC 5 处 `i32.add` 期望栈上有两个 i32，但实际栈高只有 1”）。这会大幅减少运行时调试的需要。

### 7. 执行确定性记录与回放（RR 风格）

参考 Mozilla 的 `rr` 调试器思想，让 VM 支持**记录非确定性事件**，以便单次执行后反复调试。

#### 7.1 需要记录的非确定性源
- 系统调用返回值（时间、随机数、外部输入）。
- 外部函数接口（FFI）调用结果。

#### 7.2 工作流
```bash
auto run --record my_program.auto  # 生成 trace.rec
auto debug --replay trace.rec      # 进入 AutoDBG，可反向执行、反向断点
```

### 8. 集成到 Auto 编译器工作流

所有上述能力不应仅面向 VM 开发者，也应面向**普通 Auto 语言用户**。

#### 8.1 源码级调试
用户执行 `auto debug my_program.auto` 时，编译器自动：
1. 生成带调试信息的字节码。
2. 启动 AutoDBG 并映射源码行。

用户看到的是 Auto 源码，但底层控制的是 VM 字节码执行。

#### 8.2 优化体验
在 AutoDBG 中执行 `print` 表达式时，调试器利用编译器前端的类型信息，将字节码栈上的值解释为 Auto 语言的值并打印。

### 总结：实施路线图

| 阶段 | 基础设施 | 解决的问题 |
| :--- | :--- | :--- |
| **Phase 1** | 文本汇编格式 + 反汇编器 | 能看到 VM 在跑什么 |
| **Phase 2** | 调试符号嵌入 + 崩溃堆栈 | 出错时知道对应源码位置 |
| **Phase 3** | 字节码静态验证器 | 提前发现栈/类型错误 |
| **Phase 4** | 结构化 Trace 日志 | 自动化分析栈变化 |
| **Phase 5** | 交互式调试器 (AutoDBG) | 实时检查、修改变量 |
| **Phase 6** | 记录与回放 | 处理偶发 bug |
| **Phase 7** | 汇编单元测试框架 | 确保指令集实现正确 |

按这个路线逐步完善，AutoVM 的调试体验将从“盲人摸象”进化到现代 VM 该有的水平，同时为将来可能出现的 JIT 或更复杂优化打下坚实基础。

是的，交互式调试的核心前提就是 **VM 能够“暂停”并“检视内部状态”**。要实现断点、单步执行和状态展示，AutoVM 需要在执行循环中植入**可控的挂起点**，并提供一套**状态查询接口**。下面具体说明这三项基础能力的实现思路。

### 1. 断点与单步执行的基础：执行循环的可中断设计

AutoVM 的核心是一个 `while` 循环不断读取 PC 并执行指令。要支持交互式调试，这个循环必须能从“一直跑”变为“随时停”。

```rust
// 概念性代码
pub enum ExecutionState {
    Running,
    Paused(PauseReason), // 断点、单步完成、异常
}

impl Vm {
    pub fn run(&mut self, debugger: &mut Debugger) -> Result<Value> {
        loop {
            // 1. 检查是否该暂停
            if debugger.should_pause(self) {
                debugger.prompt(self)?; // 进入 REPL，阻塞直到用户输入 continue/step
            }

            // 2. 取指、执行
            let inst = self.fetch();
            self.execute(inst)?;

            // 3. 如果是单步模式，执行一条后立即在下轮循环暂停
            if debugger.step_mode() {
                debugger.set_pause_reason(PauseReason::Step);
            }
        }
    }
}
```

#### 1.1 断点存储
- 维护一个 `HashSet<Breakpoint>`，支持按 **PC 地址**、**函数名**、**源码行号** 三种方式设置。
- 在执行循环头部调用 `check_breakpoint(current_pc, current_line)`。

#### 1.2 单步语义
- **Step Into**：执行一条字节码指令后立即暂停。
- **Step Over**：设置一个临时断点在下一行源码（或当前函数的返回地址）。
- **Step Out**：执行直到当前函数返回。

实现上，单步模式只需在 `execute()` 返回后设置一个 `pause_flag`，下一轮循环即挂起。

### 2. 暂停时的状态展示：暴露 VM 内部结构

暂停时，调试器需要读取 VM 的内部状态并以可读形式呈现给用户。AutoVM 需要提供**只读访问接口**给调试器模块。

#### 2.1 必须暴露的核心数据结构

| 数据 | 描述 | 访问方式 |
| :--- | :--- | :--- |
| **操作数栈** | 当前帧的求值栈，包含具体值及类型 | `vm.current_frame().operand_stack()` |
| **局部变量表** | 按索引存储的局部变量值 | `vm.current_frame().locals()` |
| **调用栈** | 所有活动栈帧，每个帧包含函数名、PC、返回地址 | `vm.frames()` |
| **全局变量/堆** | 如果 VM 有内存模型，需能读取特定地址 | `vm.heap.read(addr)` |
| **当前 PC 与源码映射** | 当前执行到哪条指令，对应哪一行 Auto 源码 | `vm.debug_info().line(pc)` |

#### 2.2 调试器 REPL 的典型交互
```text
(auto-dbg) break median
Breakpoint 1 set at function 'median'

(auto-dbg) run
[Breakpoint 1] median (data: Vec<i32>) at src/main.auto:15:9
(auto-dbg) info locals
  data: Vec<i32> @ 0x7f8a3c0010 (len: 5)
  sum: i32 = 0
  idx: i32 = 0
(auto-dbg) print idx + 1
  = 1 (i32)
(auto-dbg) step
[Step] median at src/main.auto:16:5
16        sum = sum + data[idx];
(auto-dbg) info stack
  Stack (height 2):
    0: i32 = 10   (data[idx])
    1: i32 = 0    (sum)
(auto-dbg) continue
```

#### 2.3 虚拟寄存器
如果 AutoVM 使用**寄存器式字节码**（例如 `r0 = add r1, r2`），则还需暴露虚拟寄存器文件。在暂停时，调试器应能按名（如 `$r0`）或索引显示其值。

### 3. 调试符号的必要性：让裸数据变可读

仅仅有 `local.get $2` 的值是 `i32(42)` 对用户意义有限。必须嵌入调试信息才能显示：

- 变量名：`sum` 而不是 `$2`
- 源码位置：`src/main.auto:15:9` 而不是 `PC 0x002a`

因此，在实现断点和状态展示**之前**，建议至少完成：
1. **字节码中嵌入源映射表**。
2. **变量名到寄存器/局部变量索引的映射**。
3. **简单的符号表查询函数**。

### 4. 实现路线的最小可行版本

如果你希望快速搭建一个可用的交互式调试器，建议按以下顺序实现：

1. **修改 VM 执行循环**：增加 `pause` 标志和 `Debugger` 钩子。
2. **实现内存中的断点列表**：只支持 PC 断点。
3. **暴露状态查询 API**：仅支持 `current_pc()` 和 `operand_stack()`。
4. **命令行 REPL**：简单的 `stdin` 读取，支持 `break <pc>`、`continue`、`step`、`print stack`。
5. **集成源映射**：让暂停时能打印源码行号。

完成这五步，就已经拥有了一个**基础但完整**的交互式调试能力，后续再逐步完善变量名显示、表达式求值、条件断点等高级功能。

**总结**：断点、单步、状态展示是交互式调试的“铁三角”，三者缺一不可。AutoVM 只要把执行循环设计为可挂起的，并为调试器开放内部状态的只读窗口，交互式调试就能水到渠成。