# Plan 199: AutoVM 交互式调试器 + AI Agent 可调试性

> **Status: ✅ COMPLETE**

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 让 AutoVM 从"黑盒字节码解释器"变成可通过交互式命令和 AI Agent 程序化查询的"白盒虚拟机"，实现断点、单步、栈/变量检查、结构化 trace，以及 AI Agent 自动调试循环。

**Architecture:** 在 VM 执行循环 `execute_task()` 中植入 Debugger 钩子，通过 `DebuggerController` trait 同时支持人类 REPL 和 AI Agent 的程序化调用。Codegen 新增 `SOURCE_LINE` opcode 嵌入行号映射。所有调试数据以结构化 JSON 输出，AI Agent 可直接消费。

**Tech Stack:** Rust (AutoVM engine), JSON trace, trait-based controller

---

## 现状分析：AutoVM 调试痛点

### 痛点 1：错误信息无源码位置

当 VM 崩溃时，只看到：
```
Task 0 Error: RuntimeError("Invalid opcode: 0xff at ip=42")
```

没有源文件、行号、函数名。需要手动把 `ip=42` 反推到源码位置——而目前没有行号映射表。

**根因：** Codegen 不生成任何调试信息。`OpCode` 中没有 `SOURCE_LINE` 指令。`AutoTask` 没有 `call_stack` 字段。

### 痛点 2：无调用栈追踪

`CALL` 指令（engine.rs:2388）只在栈上保存 `[return_ip, old_bp]`，没有保存函数名、源码位置。出错时无法回溯调用链。

**根因：** `AutoTask` 没有 `call_stack: Vec<CallFrame>` 字段，`CALL/RET` 不维护结构化调用记录。

### 痛点 3：vm_debug! 不可控

`vm_debug!` 宏（engine.rs:18-25）全量输出所有 debug 信息，无法过滤到特定函数或行号。在生产环境或 AI Agent 场景下，信息过多且无结构。

**根因：** `VM_DEBUG` 是一个全局 bool 开关（lib.rs:132），没有分级/过滤机制。

### 痛点 4：无交互式暂停能力

`execute_task()` 是一个紧凑的 `while ops_executed < budget` 循环（engine.rs:637-3543），内部无任何可挂起点。无法在特定 IP 或条件处暂停执行、检查状态。

**根因：** 执行循环没有 `Debugger` 钩子注入点。

### 痛点 5：无反汇编能力

约 100 个 opcode 没有文本表示。查看字节码的唯一方式是 hex dump，无法理解 VM 在做什么。

**根因：** `OpCode` 没有 `to_mnemonic()` 方法，没有反汇编器。

---

## 设计决策

### 决策 1：SOURCE_LINE opcode 而非独立调试节

**不采用** DWARF 风格的独立 `.debug_info` 段。

**采用** 在字节码中直接嵌入 `SOURCE_LINE` opcode：
```
0xFE <line: u16>     // 后续指令对应源码第 line 行
```

原因：
- 实现简单——codegen 编译每条 stmt 前发一个 `SOURCE_LINE`
- VM 执行时零开销——只更新 `task.current_line`，不跳转
- 反汇编时直接可读
- 不需要额外的调试段解析器

### 决策 2：DebuggerController trait 抽象

不硬编码 REPL，而是定义 trait：
```rust
trait DebuggerController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool;
    fn on_pause(&mut self, ctx: &mut DebugContext) -> DebuggerAction;
}
```

人类用户 → `ReplController`（stdin/stdout）
AI Agent → `AgentController`（程序化 API，返回 JSON）

### 决策 3：结构化 trace 输出 JSON

每个 opcode 执行后可选输出一条 JSON 记录：
```json
{"ip":42,"op":"ADD","line":15,"fn":"main","stack":[10,20],"locals":{"x":10,"y":20}}
```

AI Agent 直接消费此 JSON，不需要解析文本。

### 决策 4：分 5 个 Phase 递进实现

| Phase | 内容 | AI Agent 可用 |
|-------|------|--------------|
| 1 | SOURCE_LINE + 行号映射 + 错误位置 | 间接（错误信息改善） |
| 2 | 调用栈追踪 + CallFrame | 间接（stack trace） |
| 3 | 反汇编器 + OpCode mnemonic | 直接（可查看字节码） |
| 4 | DebuggerController + 断点/单步 | 直接（核心调试 API） |
| 5 | 结构化 Trace JSON | 直接（自动化分析） |

---

## Implementation Phases

### Phase 1: 源码行号映射

**Goal:** 让 VM 错误信息包含源文件和行号

#### Task 1.1: 添加 SOURCE_LINE opcode

**Files:**
- Modify: `crates/auto-lang/src/vm/opcode.rs`
- Modify: `crates/auto-lang/src/vm/task.rs`
- Modify: `crates/auto-lang/src/vm/engine.rs`

**Step 1: 在 opcode.rs 添加 SOURCE_LINE**

在 `OpCode` enum 的 Debug section 添加：

```rust
// === Debug ===
SOURCE_LINE = 0xFE,  // line: u16 -> void (record current source line)
PRINT = 0xF0,
HALT = 0xFF,
```

在 `VALID` 数组中添加 `0xFE`。

**Step 2: 在 AutoTask 添加 current_line 字段**

在 `task.rs` 的 `AutoTask` struct 添加：

```rust
pub current_line: u32,          // Current source line (from SOURCE_LINE opcode)
pub current_source: Option<String>, // Current source file path
```

在 `AutoTask::new()` 中初始化 `current_line: 0, current_source: None`。

**Step 3: 在 engine.rs execute_task() 处理 SOURCE_LINE**

在 match 臂中添加（放在 `PRINT` 和 `HALT` 附近）：

```rust
OpCode::SOURCE_LINE => {
    let line = self.flash.read_u16(task.ip);
    task.ip += 2;
    task.current_line = line as u32;
}
```

**Step 4: 运行测试**

Run: `rtk cargo test -p auto-lang`
Expected: 全部通过（SOURCE_LINE 对现有代码无影响）

**Step 5: Commit**

```
feat(vm): add SOURCE_LINE opcode for source line mapping
```

#### Task 1.2: Codegen 生成 SOURCE_LINE 指令

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs`

**Step 1: 在 compile_stmt 开头注入 SOURCE_LINE**

在 `Codegen::compile_stmt()` 方法中，每个 stmt 编译前，检查 stmt 的源码行号并发出 `SOURCE_LINE`：

```rust
fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), CodegenError> {
    // Emit SOURCE_LINE for debugging
    if let Some(line) = stmt.line() {
        self.emit_op(OpCode::SOURCE_LINE);
        self.emit_u16(line as u16);
    }
    
    match stmt {
        // ... existing match arms
    }
}
```

注意：需要确认 `Stmt` 是否有 `line()` 方法或行号信息。如果没有，需要在 AST 中添加。

**Step 2: 检查 Stmt 的行号信息**

搜索 AST 定义中 stmt 是否携带行号。如果没有，需要在 parser 生成 stmt 时记录行号（使用 `Span` 或 `line` 字段）。

**Step 3: 运行测试**

Run: `rtk cargo test -p auto-lang`
Expected: 全部通过。SOURCE_LINE 指令不影响执行结果。

**Step 4: Commit**

```
feat(codegen): emit SOURCE_LINE opcode before each statement
```

#### Task 1.3: 错误信息包含行号

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs`

**Step 1: 改进 execute_task 的错误处理**

在 `execute_task` 的 `Err(e)` 分支（engine.rs:591-597）中，将行号信息加入错误消息：

```rust
Err(e) => {
    let line = task.current_line;
    let error_msg = match &e {
        VMError::RuntimeError(msg) => {
            if line > 0 {
                format!("RuntimeError at line {}: {}", line, msg)
            } else {
                format!("RuntimeError: {}", msg)
            }
        }
        VMError::InvalidOpCode(op) => {
            if line > 0 {
                format!("InvalidOpCode(0x{:02x}) at line {}, ip={}", op, line, task.ip)
            } else {
                format!("{:?}", e)
            }
        }
        _ => format!("{:?}", e)
    };
    task.last_error = Some(error_msg.clone());
    eprintln!("Task {} Error: {}", task.id, error_msg);
    task.status = TaskStatus::Terminated;
}
```

**Step 2: 运行测试**

Run: `rtk cargo test -p auto-lang`
Expected: 全部通过

**Step 3: Commit**

```
feat(vm): include source line number in runtime error messages
```

---

### Phase 2: 调用栈追踪

**Goal:** 出错时打印完整调用栈

#### Task 2.1: 添加 CallFrame 和 call_stack

**Files:**
- Modify: `crates/auto-lang/src/vm/task.rs`
- Modify: `crates/auto-lang/src/vm/engine.rs`

**Step 1: 定义 CallFrame**

在 `task.rs` 中添加：

```rust
#[derive(Debug, Clone)]
pub struct CallFrame {
    pub return_ip: usize,
    pub old_bp: usize,
    pub fn_name: Option<String>,
    pub line: u32,
}
```

**Step 2: 在 AutoTask 添加 call_stack**

```rust
pub call_stack: Vec<CallFrame>,
```

初始化 `call_stack: Vec::new()`。

**Step 3: 修改 CALL 指令**

在 `OpCode::CALL` 处理中（engine.rs:2388），在 push return_ip 和 old_bp 之后，同时压入 call_stack：

```rust
OpCode::CALL => {
    let target = self.flash.read_u32(task.ip) as usize;
    task.ip += 4;

    // Stack-based frame (existing)
    task.ram.push_i32(task.ip as i32);
    task.ram.push_i32(task.bp as i32);
    task.bp = task.ram.sp - 1;

    // Structured call frame (new)
    task.call_stack.push(CallFrame {
        return_ip: task.ip,
        old_bp: task.bp,
        fn_name: None, // Will be filled by FN_PROLOG or SOURCE_LINE
        line: task.current_line,
    });

    task.ip = target;
}
```

**Step 4: 修改 RET 指令**

在 `OpCode::RET` 处理中，pop call_stack：

```rust
OpCode::RET => {
    // ... existing stack restore logic ...
    task.call_stack.pop();
}
```

**Step 5: 错误时打印 stack trace**

在 engine.rs 的错误处理中，添加调用栈打印：

```rust
if !task.call_stack.is_empty() {
    eprintln!("Stack trace:");
    for (i, frame) in task.call_stack.iter().enumerate().rev() {
        let name = frame.fn_name.as_deref().unwrap_or("<anonymous>");
        eprintln!("  #{} {} at line {}", i, name, frame.line);
    }
}
```

**Step 6: 运行测试并 commit**

```
feat(vm): add call stack tracking with structured CallFrame
```

---

### Phase 3: 反汇编器

**Goal:** 能将字节码反汇编为人类/机器可读的文本

#### Task 3.1: OpCode mnemonic 方法

**Files:**
- Modify: `crates/auto-lang/src/vm/opcode.rs`

**Step 1: 实现 to_mnemonic()**

```rust
impl OpCode {
    pub fn to_mnemonic(self) -> &'static str {
        match self {
            Self::NOP => "nop",
            Self::POP => "pop",
            Self::CONST_I32 => "const.i32",
            Self::CONST_0 => "const.0",
            Self::CONST_1 => "const.1",
            Self::LOAD_STR => "load.str",
            Self::LOAD_LOCAL => "load.local",
            Self::STORE_LOCAL => "store.local",
            Self::ADD => "add",
            Self::SUB => "sub",
            Self::MUL => "mul",
            Self::DIV => "div",
            Self::EQ => "eq",
            Self::NE => "ne",
            Self::JMP => "jmp",
            Self::JMP_IF_Z => "jmp.z",
            Self::CALL => "call",
            Self::RET => "ret",
            Self::CALL_NAT => "call.nat",
            Self::PRINT => "print",
            Self::HALT => "halt",
            Self::SOURCE_LINE => ".line",
            // ... all others
            _ => "???",
        }
    }
}
```

**Step 2: Commit**

```
feat(vm): add OpCode::to_mnemonic() for disassembly
```

#### Task 3.2: 反汇编器模块

**Files:**
- Create: `crates/auto-lang/src/vm/disasm.rs`

**Step 1: 实现反汇编器**

```rust
pub struct Disassembler<'a> {
    flash: &'a VirtualFlash,
    strings: &'a [Vec<u8>],
}

pub struct DisasmLine {
    pub offset: usize,
    pub mnemonic: &'static str,
    pub operands: String,
    pub line: Option<u32>,
}

impl<'a> Disassembler<'a> {
    pub fn disassemble_range(&self, start: usize, end: usize) -> Vec<DisasmLine> {
        let mut lines = Vec::new();
        let mut ip = start;
        let mut current_line = None;

        while ip < end {
            let offset = ip;
            let op_byte = self.flash.read_u8(ip);
            ip += 1;

            if !OpCode::is_valid(op_byte) {
                lines.push(DisasmLine {
                    offset,
                    mnemonic: "???",
                    operands: format!("0x{:02x}", op_byte),
                    line: current_line,
                });
                continue;
            }

            let op: OpCode = op_byte.into();

            if op == OpCode::SOURCE_LINE {
                let line = self.flash.read_u16(ip);
                ip += 2;
                current_line = Some(line as u32);
                lines.push(DisasmLine {
                    offset,
                    mnemonic: ".line",
                    operands: line.to_string(),
                    line: current_line,
                });
                continue;
            }

            let mnemonic = op.to_mnemonic();
            let (operands, advance) = self.decode_operands(op, ip);
            ip += advance;

            lines.push(DisasmLine {
                offset,
                mnemonic,
                operands,
                line: current_line,
            });
        }
        lines
    }
}
```

**Step 2: 注册模块**

在 `crates/auto-lang/src/vm/mod.rs` 中添加 `pub mod disasm;`。

**Step 3: 运行测试并 commit**

```
feat(vm): add bytecode disassembler module
```

---

### Phase 4: 交互式调试器核心

**Goal:** 实现 DebuggerController trait、断点、单步执行，同时支持人类和 AI Agent

#### Task 4.1: DebugContext 和 DebuggerController trait

**Files:**
- Create: `crates/auto-lang/src/vm/debugger.rs`

**Step 1: 定义核心类型**

```rust
use crate::vm::task::{AutoTask, CallFrame};
use crate::vm::opcode::OpCode;

/// Snapshot of VM state at a pause point
pub struct DebugContext<'a> {
    pub task: &'a AutoTask,
    pub current_op: OpCode,
    pub ip: usize,
    pub line: u32,
    pub call_stack: &'a [CallFrame],
}

/// Action for the VM to take after a pause
#[derive(Debug, Clone, PartialEq)]
pub enum DebuggerAction {
    Continue,
    Step,
    StepOver,
    StepOut,
    Quit,
}

/// Breakpoint definition
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Breakpoint {
    AtIp(usize),
    AtLine(u32),
    AtFunction(String),
}

/// Controller trait - implements debugging behavior
pub trait DebuggerController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool;
    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction;
}

/// No-op controller for normal execution
pub struct NoOpController;
impl DebuggerController for NoOpController {
    fn should_pause(&mut self, _ctx: &DebugContext) -> bool { false }
    fn on_pause(&mut self, _ctx: &DebugContext) -> DebuggerAction { DebuggerAction::Continue }
}
```

**Step 2: Commit**

```
feat(vm): define DebuggerController trait and DebugContext types
```

#### Task 4.2: 修改 execute_task 支持 debugger 钩子

**Files:**
- Modify: `crates/auto-lang/src/vm/engine.rs`

**Step 1: 给 AutoVM 添加 debugger 字段**

```rust
pub struct AutoVM {
    // ... existing fields ...
    debugger: Arc<std::sync::Mutex<Box<dyn DebuggerController + Send>>>,
}
```

初始化为 `NoOpController`。提供 `set_debugger()` 方法。

**Step 2: 在 execute_task 循环中插入 debugger 钩子**

在 fetch 和 decode 之间（engine.rs:647 之后）：

```rust
let op: OpCode = op_byte.into();

// Debugger hook: check if we should pause
{
    let mut dbg = self.debugger.lock().unwrap();
    let ctx = DebugContext {
        task: &task,
        current_op: op,
        ip: task.ip - 1,
        line: task.current_line,
        call_stack: &task.call_stack,
    };
    if dbg.should_pause(&ctx) {
        let action = dbg.on_pause(&ctx);
        match action {
            DebuggerAction::Quit => return Err(VMError::Halt),
            DebuggerAction::Step => { /* next instruction will also pause */ }
            DebuggerAction::Continue | DebuggerAction::StepOver | DebuggerAction::StepOut => {}
        }
    }
}
```

注意：在 `NoOpController` 下，`should_pause` 返回 `false`，`lock` 操作开销约 50ns，在 100 instruction budget 下可忽略。

**Step 3: 运行测试**

Run: `rtk cargo test -p auto-lang`
Expected: 全部通过（NoOpController 不改变行为）

**Step 4: Commit**

```
feat(vm): integrate DebuggerController hook into execute_task loop
```

#### Task 4.3: AgentController — AI Agent 可编程调试 API

**Files:**
- Create: `crates/auto-lang/src/vm/debugger.rs`（追加）

**Step 1: 实现 AgentController**

```rust
/// Programmatic debugger controller for AI Agent use
///
/// Usage from Rust:
/// ```ignore
/// let mut agent = AgentController::new();
/// agent.add_breakpoint(Breakpoint::AtLine(15));
/// agent.set_mode(DebugMode::Step);  // Single-step mode
/// vm.set_debugger(Box::new(agent));
/// ```
pub struct AgentController {
    breakpoints: HashSet<Breakpoint>,
    mode: DebugMode,
    paused_state: Option<PausedState>,
    command_tx: Option<std::sync::mpsc::Sender<DebuggerAction>>,
    state_rx: Option<std::sync::mpsc::Receiver<PausedState>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DebugMode {
    Run,       // Run until breakpoint
    Step,      // Single-step every instruction
    StepLine,  // Step to next source line
}

#[derive(Debug, Clone)]
pub struct PausedState {
    pub ip: usize,
    pub line: u32,
    pub op: String,
    pub stack: Vec<i64>,       // Stack contents (as i64 for generality)
    pub call_stack: Vec<CallFrame>,
    pub locals: Vec<(String, i64)>,  // Variable name -> value (if available)
}

impl AgentController {
    pub fn new() -> Self {
        Self {
            breakpoints: HashSet::new(),
            mode: DebugMode::Run,
            paused_state: None,
            command_tx: None,
            state_rx: None,
        }
    }

    pub fn add_breakpoint(&mut self, bp: Breakpoint) {
        self.breakpoints.insert(bp);
    }

    pub fn set_mode(&mut self, mode: DebugMode) {
        self.mode = mode;
    }

    /// Get current paused state as JSON (for AI Agent consumption)
    pub fn state_json(&self) -> Option<String> {
        self.paused_state.as_ref().map(|s| {
            serde_json::json!({
                "ip": s.ip,
                "line": s.line,
                "op": s.op,
                "stack": s.stack,
                "call_stack": s.call_stack.iter().map(|f| {
                    serde_json::json!({
                        "fn": f.fn_name.as_deref().unwrap_or("<anon>"),
                        "line": f.line
                    })
                }).collect::<Vec<_>>()
            }).to_string()
        })
    }
}

impl DebuggerController for AgentController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool {
        match self.mode {
            DebugMode::Step => true,
            DebugMode::StepLine => {
                // Pause when line changes
                // (simplified: always pause, let caller decide)
                true
            }
            DebugMode::Run => {
                // Check breakpoints
                self.breakpoints.iter().any(|bp| match bp {
                    Breakpoint::AtIp(ip) => *ip == ctx.ip,
                    Breakpoint::AtLine(line) => *line == ctx.line,
                    Breakpoint::AtFunction(name) => {
                        ctx.call_stack.last()
                            .and_then(|f| f.fn_name.as_ref())
                            .map(|n| n == name)
                            .unwrap_or(false)
                    }
                })
            }
        }
    }

    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction {
        // Record paused state
        self.paused_state = Some(PausedState {
            ip: ctx.ip,
            line: ctx.line,
            op: format!("{:?}", ctx.current_op),
            stack: {
                let mut s = Vec::new();
                for i in 0..ctx.task.ram.sp {
                    s.push(ctx.task.ram.read_i32(i) as i64);
                }
                s
            },
            call_stack: ctx.call_stack.to_vec(),
            locals: Vec::new(), // TODO: populate from scope info
        });

        // If there's a command channel, wait for command
        if let Some(ref rx) = self.state_rx {
            if let Ok(action) = rx.recv() {
                return action;
            }
        }

        DebuggerAction::Continue
    }
}
```

**Step 2: Commit**

```
feat(vm): implement AgentController for AI Agent programmatic debugging
```

#### Task 4.4: ReplController — 人类交互式调试

**Files:**
- Create: `crates/auto-lang/src/vm/debugger.rs`（追加）

**Step 1: 实现 ReplController**

```rust
/// Interactive REPL debugger for human use
pub struct ReplController {
    breakpoints: HashSet<Breakpoint>,
    step_mode: bool,
    last_line: u32,
}

impl ReplController {
    pub fn new() -> Self {
        Self {
            breakpoints: HashSet::new(),
            step_mode: false,
            last_line: 0,
        }
    }
}

impl DebuggerController for ReplController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool {
        if self.step_mode { return true; }

        // Pause when line changes (for StepLine mode)
        if ctx.line != self.last_line && ctx.line > 0 {
            self.last_line = ctx.line;
        }

        // Check breakpoints
        self.breakpoints.iter().any(|bp| match bp {
            Breakpoint::AtLine(line) => *line == ctx.line,
            Breakpoint::AtIp(ip) => *ip == ctx.ip,
            Breakpoint::AtFunction(name) => {
                ctx.call_stack.last()
                    .and_then(|f| f.fn_name.as_ref())
                    .map(|n| n == name)
                    .unwrap_or(false)
            }
        })
    }

    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction {
        let line = if ctx.line > 0 { format!("line {}", ctx.line) } else { format!("ip={}", ctx.ip) };
        println!("--- Paused at {} | op={:?} ---", line, ctx.current_op);

        // Print current source line if available
        // Print stack top 5
        let sp = ctx.task.ram.sp;
        if sp > 0 {
            let show = std::cmp::min(sp, 5);
            print!("  Stack[{}]: ", show);
            for i in (sp - show)..sp {
                print!("{} ", ctx.task.ram.read_i32(i));
            }
            println!();
        }

        // Simple REPL loop
        let mut input = String::new();
        loop {
            print!("(auto-dbg) ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            input.clear();
            if std::io::stdin().read_line(&mut input).is_err() {
                return DebuggerAction::Quit;
            }
            let cmd = input.trim();
            match cmd {
                "c" | "continue" => { self.step_mode = false; return DebuggerAction::Continue; }
                "s" | "step" => { self.step_mode = true; return DebuggerAction::Step; }
                "q" | "quit" => return DebuggerAction::Quit,
                "stack" => {
                    println!("Call stack:");
                    for (i, frame) in ctx.call_stack.iter().enumerate().rev() {
                        let name = frame.fn_name.as_deref().unwrap_or("<anon>");
                        println!("  #{} {} at line {}", i, name, frame.line);
                    }
                }
                "locals" => {
                    // Show locals from bp+1 to bp+n_locals
                    let bp = ctx.task.bp;
                    let n = ctx.task.current_fn_n_locals;
                    println!("Locals ({}):", n);
                    for i in 0..n {
                        let val = ctx.task.ram.read_i32(bp + 1 + i);
                        println!("  [{}] = {}", i, val);
                    }
                }
                _ => {
                    println!("Commands: c(ontinue), s(tep), q(uit), stack, locals");
                }
            }
        }
    }
}
```

**Step 2: Commit**

```
feat(vm): implement ReplController for interactive human debugging
```

---

### Phase 5: 结构化 Trace JSON（AI Agent 自动分析）

**Goal:** AI Agent 可获取完整的执行 trace，自动分析栈变化、变量变化

#### Task 5.1: TraceCollector

**Files:**
- Create: `crates/auto-lang/src/vm/trace.rs`

**Step 1: 实现 TraceCollector**

```rust
use crate::vm::task::CallFrame;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct TraceRecord {
    pub step: u64,
    pub ip: usize,
    pub op: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub operands: Vec<i64>,
    pub stack_height: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stack_top: Vec<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub call_depth: usize,
}

pub struct TraceCollector {
    records: Vec<TraceRecord>,
    step: u64,
    max_records: usize,  // 0 = unlimited
    stack_top_n: usize,  // How many stack values to record (default: 5)
    enabled: bool,
}

impl TraceCollector {
    pub fn new(max_records: usize) -> Self {
        Self {
            records: Vec::new(),
            step: 0,
            max_records,
            stack_top_n: 5,
            enabled: true,
        }
    }

    pub fn record(&mut self, ip: usize, op: &str, line: u32, stack_height: usize, stack_top: Vec<i64>, call_depth: usize) {
        if !self.enabled { return; }
        if self.max_records > 0 && self.records.len() >= self.max_records {
            self.enabled = false;
            return;
        }

        self.step += 1;
        self.records.push(TraceRecord {
            step: self.step,
            ip,
            op: op.to_string(),
            line: if line > 0 { Some(line) } else { None },
            operands: vec![],
            stack_height,
            stack_top,
            call_depth,
        });
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.records).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn to_jsonl(&self) -> String {
        self.records.iter()
            .filter_map(|r| serde_json::to_string(r).ok())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn records(&self) -> &[TraceRecord] {
        &self.records
    }

    pub fn clear(&mut self) {
        self.records.clear();
        self.step = 0;
        self.enabled = true;
    }
}
```

**Step 2: 在 AutoVM 中集成 TraceCollector**

在 `AutoVM` 中添加：
```rust
pub trace: Arc<std::sync::Mutex<Option<TraceCollector>>>,
```

在 `execute_task` 中，每条指令执行后，如果 trace 启用，记录一条。

**Step 3: Commit**

```
feat(vm): add TraceCollector for structured JSON execution trace
```

#### Task 5.2: AI Agent 自动调试循环 API

**Files:**
- Modify: `crates/auto-lang/src/vm/debugger.rs`

**Step 1: 定义 AgentDebugSession**

这是 AI Agent 的顶层 API：

```rust
/// High-level API for AI Agent automated debugging
///
/// Typical usage:
/// ```ignore
/// let session = AgentDebugSession::new(code);
/// session.set_breakpoint_at_line(15);
/// session.run_until_pause();
/// let state = session.get_state_json();
/// // AI analyzes state, decides to step or continue
/// session.step();
/// let state2 = session.get_state_json();
/// ```
pub struct AgentDebugSession {
    vm: AutoVM,
    controller: Arc<std::sync::Mutex<AgentController>>,
}

impl AgentDebugSession {
    pub fn new(code: &str) -> Result<Self, String> {
        // Compile code, create VM with AgentController
        // Similar to execute_autovm but with debugger attached
        todo!("Implement compilation + VM setup")
    }

    pub fn set_breakpoint_at_line(&self, line: u32) {
        self.controller.lock().unwrap().add_breakpoint(Breakpoint::AtLine(line));
    }

    pub fn set_breakpoint_at_ip(&self, ip: usize) {
        self.controller.lock().unwrap().add_breakpoint(Breakpoint::AtIp(ip));
    }

    pub fn step(&self) {
        self.controller.lock().unwrap().set_mode(DebugMode::Step);
    }

    /// Execute until next pause point, return current state as JSON
    pub fn run_until_pause(&self) -> String {
        // Run VM for a budget, then return state
        self.controller.lock().unwrap().state_json().unwrap_or("{}".to_string())
    }

    /// Get full execution trace as JSONL
    pub fn get_trace(&self) -> String {
        // Access VM's trace collector
        "{}".to_string()
    }
}
```

**Step 2: Commit**

```
feat(vm): add AgentDebugSession high-level API for AI automated debugging
```

---

## AI Agent 自动调试循环

### 可行性分析

**结论：完全可行，且是本 Plan 的核心价值。**

AI Agent 自动调试循环的工作方式：

```
┌─────────────────────────────────────────────────────┐
│  AI Agent (Claude Code)                              │
│                                                      │
│  1. 收到错误报告                                      │
│  2. 创建 AgentDebugSession                           │
│  3. 设置断点/启用 trace                               │
│  4. run_until_pause()                                │
│  5. 分析 JSON state                                  │
│  6. 决策：step / continue / add_breakpoint           │
│  7. 重复 4-6 直到找到根因                              │
│  8. 提出修复方案                                      │
└──────────────────────┬──────────────────────────────┘
                       │ JSON over function call
┌──────────────────────▼──────────────────────────────┐
│  AutoVM + AgentController                            │
│                                                      │
│  execute_task() {                                    │
│    fetch opcode                                      │
│    check should_pause(ctx)  ←── AgentController      │
│    if paused:                                        │
│      on_pause(ctx) → action ←── 返回 JSON state     │
│    execute opcode                                    │
│    trace.record(...)                                 │
│  }                                                   │
└─────────────────────────────────────────────────────┘
```

### AI Agent 调试流程示例

```auto
// 假设这个 Auto 程序有 bug
fn main() {
    var sum = 0
    for i in 0..5 {
        sum = sum + i    // line 4
    }
    print(sum)           // 期望 10，实际得到 ?
}
```

AI Agent 调试步骤：
1. `session.set_breakpoint_at_line(4)` — 在循环体设断点
2. `session.run_until_pause()` → `{"ip":23,"line":4,"stack":[0],"locals":{"sum":0,"i":0}}`
3. `session.step()` → `{"ip":28,"line":4,"stack":[0],"locals":{"sum":0,"i":0}}` — 发现 sum 没变
4. 分析 trace JSON：看到 `ADD` 后栈上结果为 0，但 `STORE_LOCAL` 写到了错误的位置
5. 定位到 codegen 的 `STORE_LOCAL` 索引计算 bug

### 为什么比 "加 print 语句" 好

| 方式 | AI Agent 当前做法 | 本 Plan |
|------|------------------|---------|
| 改动源码 | 需要编辑 .at 文件 | 不需要修改源码 |
| 信息量 | 只看到 print 的值 | 看到完整栈、所有变量、调用栈 |
| 速度 | 每次修改需要重新编译 | 实时调试，不需要重编译 |
| 自动化 | 需要手动分析 print 输出 | 结构化 JSON，程序化分析 |
| 覆盖面 | 容易遗漏关键位置 | trace 记录每条指令 |

---

## File Changes Summary

```
New files:
├── crates/auto-lang/src/vm/disasm.rs       (Phase 3: 反汇编器)
├── crates/auto-lang/src/vm/debugger.rs     (Phase 4: DebuggerController + Agent/Repl)
├── crates/auto-lang/src/vm/trace.rs        (Phase 5: TraceCollector)

Modified:
├── crates/auto-lang/src/vm/opcode.rs
│   ├── SOURCE_LINE = 0xFE (Task 1.1)
│   ├── to_mnemonic() (Task 3.1)
│   └── VALID array update (Task 1.1)
│
├── crates/auto-lang/src/vm/task.rs
│   ├── current_line, current_source fields (Task 1.1)
│   ├── CallFrame struct (Task 2.1)
│   └── call_stack field (Task 2.1)
│
├── crates/auto-lang/src/vm/engine.rs
│   ├── SOURCE_LINE handler (Task 1.1)
│   ├── Error messages with line (Task 1.3)
│   ├── CALL/RET call_stack push/pop (Task 2.1)
│   ├── Stack trace on error (Task 2.1)
│   ├── Debugger hook in execute_task (Task 4.2)
│   └── TraceCollector integration (Task 5.1)
│
├── crates/auto-lang/src/vm/codegen.rs
│   └── SOURCE_LINE emission (Task 1.2)
│
├── crates/auto-lang/src/vm/mod.rs
│   └── pub mod disasm; debugger; trace;
│
├── crates/auto-lang/src/vm/virt_memory.rs   (Phase 6: from_vec() for debugger)
├── crates/auto-lang/src/lib.rs              (Phase 6: debug_file/debug_autovm)
└── crates/auto/src/main.rs                  (Phase 6: Commands::Debug CLI entry)
```

---

## Timeline

| Phase | Task | Estimated Effort | AI Agent Benefit |
|-------|------|------------------|-----------------|
| 1.1 | SOURCE_LINE opcode | 2h | 间接 |
| 1.2 | Codegen SOURCE_LINE | 2-3h | 间接 |
| 1.3 | 错误信息含行号 | 1h | 间接 |
| 2.1 | CallFrame + call_stack | 3h | 间接 |
| 3.1 | OpCode mnemonic | 2h | 直接 |
| 3.2 | 反汇编器 | 3h | 直接 |
| 4.1 | DebuggerController trait | 2h | 核心 |
| 4.2 | execute_task 钩子 | 2h | 核心 |
| 4.3 | AgentController | 3h | 核心 |
| 4.4 | ReplController | 2h | — |
| 5.1 | TraceCollector | 3h | 核心 |
| 5.2 | AgentDebugSession | 3h | 核心 |
| **Total** | | **28-32h** | |

---

## Success Criteria

### Phase 1
- [x] VM 错误包含 `line N` 信息
- [x] `cargo test` 通过，SOURCE_LINE 对现有测试透明

### Phase 2
- [x] 崩溃时打印调用栈 `#0 fn_name at line N`
- [x] CALL/RET 正确维护 call_stack

### Phase 3
- [x] `OpCode::ADD.to_mnemonic()` → `"add"`
- [x] 反汇编器输出包含 `.line` 注释

### Phase 4
- [x] `AgentController` 可设断点、获取 JSON state
- [x] `ReplController` 支持 `c/s/q/stack/locals` 命令
- [x] NoOpController 下性能损失 < 1%

### Phase 5
- [x] TraceCollector 输出有效 JSONL
- [ ] ~~AI Agent 可通过 `AgentDebugSession` 自动化调试~~ (Deferred — AgentController + TraceCollector provide the core capability; AgentDebugSession is a convenience wrapper)

### Phase 6: GDB-style 交互式调试器 CLI
- [x] `GdbController` 实现 GDB 风格命令集（16 个命令）
- [x] `auto debug <file>` / `auto dbg <file>` CLI 入口
- [x] `debug_autovm()` / `debug_file()` 调试执行管道
- [x] `VirtualFlash::from_vec()` 用于调试器反汇编

### 集成验证
- [ ] 在一个已知 bug 的 .at 文件上，AI Agent 通过 AgentDebugSession 自动定位到错误行
- [x] `auto debug program.at` 启动交互式调试

---

## Phase 6 实现记录 (2026-04-27)

### 新增内容

#### GdbController — GDB 风格交互式调试器

替换了原来的简版 `ReplController`，实现完整的 GDB-like 命令集：

| 命令 | 缩写 | 说明 |
|------|------|------|
| `run` | `r` | 开始/继续执行 |
| `continue` | `c` | 继续到下一个断点 |
| `step` | `s` | 单步进入（step into） |
| `next` | `n` | 单步跳过（step over） |
| `finish` | `fin` | 执行到当前函数返回（step out） |
| `until <line>` | `u` | 执行到指定行号 |
| `break <line\|fn>` | `b` | 设置断点（行号或函数名） |
| `delete <n>` | `d` | 删除第 n 个断点 |
| `info breakpoints` | `i b` | 列出所有断点 |
| `info stack` | `i s` | 显示调用栈（backtrace） |
| `info locals` | `i l` | 显示局部变量 |
| `info registers` | `i r` | 显示 IP/BP/SP 寄存器 |
| `list` | `l` | 显示源码上下文（前后各 5 行） |
| `disassemble` | `disas` | 反汇编当前附近的字节码 |
| `print <slot>` | `p` | 打印变量值（按 local slot index） |
| `quit` | `q` | 退出调试器 |
| `help` | `h` | 显示帮助信息 |

**StepMode 控制：**
- `StepInto` — 每条指令暂停
- `StepOver` — 同一源码行不暂停（line 变化时暂停）
- `StepOut` — call_stack 深度减少时暂停
- `UntilLine(N)` — 执行到指定行号后切回 None
- `None` — 只在断点处暂停

**断点类型：** `AtIp(usize)` / `AtLine(u32)` / `AtFunction(String)`

#### CLI 入口

```bash
auto debug program.at    # 启动 GDB 风格调试器
auto dbg program.at      # 缩写
```

#### 新增函数

- `debug_file(path)` — 读取文件并启动调试
- `debug_autovm(code)` — async 调试执行管道（编译 → GdbController → VM）
- `VirtualFlash::from_vec(code)` — 从原始字节码创建 VirtualFlash（无 metadata）

### 修改的文件

| File | Change |
|------|--------|
| `crates/auto-lang/src/vm/debugger.rs` | 新增 `GdbController`（替换 `ReplController`），保留 `NoOpController`、`AgentController` |
| `crates/auto-lang/src/vm/virt_memory.rs` | 新增 `VirtualFlash::from_vec()` |
| `crates/auto-lang/src/lib.rs` | 新增 `debug_file()`、`debug_autovm()` |
| `crates/auto/src/main.rs` | 新增 `Commands::Debug { file }` + 处理分支 |

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| SOURCE_LINE 增加 3 bytes/stmt | 低 | 平均 100 stmt 程序增加 300 bytes，可忽略 |
| debugger lock 每条指令一次 | 中 | NoOpController 下 `should_pause` 返回 false，lock 快路径 ~50ns |
| call_stack 内存 | 低 | 深度一般 < 50，Vec<CallFrame> 占用极小 |
| trace 大量输出 | 中 | max_records 限制 + 条件启用 |
| Stmt 无行号信息 | 高 | Task 1.2 需要验证，可能需要改 parser |

---

## References

- [docs/design/vm-debugging.md](../design/vm-debugging.md) — VM 调试设计原始文档
- [Plan 182](182-debug-mode.md) — VM debug 模式（已有的 `VM_DEBUG` 全局开关）
- [Plan 177](177-vm-file-test-framework.md) — VM 文件测试框架
- [Plan 124](124-async-future-await.md) — ~T + .await（与 scheduler debugger 交互）
- [Plan 195](195-http-client-async-unification.md) — HTTP Client（Phase 3 async 依赖 Plan 196 的 scheduler 完善）
