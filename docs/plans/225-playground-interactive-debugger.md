# Plan 225: Playground 交互式调试器 (Playground Interactive Debugger)

## Status: 📋 PLANNED

**Goal:** 在 auto-playground 中实现完整的 AutoVM 交互式调试体验。用户可以在浏览器中打开 Debug 模式，左侧编辑 Auto 源码、右侧查看对应字节码汇编，设置断点、单步执行（step into/over/out）、继续运行，并实时观察调用栈、局部变量和操作数栈的动态变化。

**Architecture:** 后端 Axum 新增 WebSocket 端点维持 Debug Session；新建 `PlaygroundController` 实现 `DebuggerController` trait，作为 VM 与前端之间的状态桥接；前端新增 Bytecode 面板、Debug 控制栏、Stack/Locals/Call Stack 调试面板，通过 WebSocket 与后端双向通信。源码到字节码的映射复用 VM 已有的 `SOURCE_LINE` opcode 机制。

**Tech Stack:** Rust (AutoVM, Axum, tokio), WebSocket, Vue 3, CodeMirror 6, TypeScript

---

## 现状分析

### 已有实现（Plan 199 完成）

| 组件 | 文件 | 状态 | 说明 |
|------|------|------|------|
| `DebuggerController` trait | `crates/auto-lang/src/vm/debugger.rs` | ✅ | `should_pause` / `on_pause` |
| `DebugContext` | `crates/auto-lang/src/vm/debugger.rs` | ✅ | 暂停时的 VM 状态快照 |
| `DebuggerAction` | `crates/auto-lang/src/vm/debugger.rs` | ✅ | `Continue/Step/StepOver/StepOut/Quit` |
| `Breakpoint` | `crates/auto-lang/src/vm/debugger.rs` | ✅ | `AtIp/AtLine/AtFunction` |
| `GdbController` | `crates/auto-lang/src/vm/debugger.rs` | ✅ | CLI REPL 调试器 |
| `AgentController` | `crates/auto-lang/src/vm/debugger.rs` | ✅ | AI Agent 程序化调试 |
| `SOURCE_LINE` opcode | `crates/auto-lang/src/vm/opcode.rs` | ✅ | `0xFE <line: u16>` |
| `Disassembler` | `crates/auto-lang/src/vm/disasm.rs` | ✅ | `DisasmLine { offset, mnemonic, operands, line }` |
| `TraceCollector` | `crates/auto-lang/src/vm/trace.rs` | ✅ | 执行轨迹 JSON 输出 |
| `AutoTask` 栈信息 | `crates/auto-lang/src/vm/task.rs` | ✅ | `call_stack`, `current_line`, `bp`, `ram.sp` |
| Playground HTTP API | `crates/auto-playground/src/routes/` | ✅ | `/api/run`, `/api/trans`, `/api/examples` |
| Playground 前端 | `crates/auto-playground/frontend/` | ✅ | Vue 3 + CodeMirror 6 + Vite |

### 缺失内容

| 组件 | 当前状态 | 缺失内容 |
|------|---------|---------|
| Playground Debug API | 无 | 没有 WebSocket / debug session / step / breakpoint 端点 |
| 网络化 Controller | 无 | 没有 `DebuggerController` 的 WebSocket 适配器 |
| Debug Session 管理 | 无 | 没有 session 生命周期管理和并发控制 |
| 前端字节码面板 | 无 | OutputPanel 只有 rust/c/python/ts 4 个 tab |
| 前端调试 UI | 无 | 没有断点 gutter、step 按钮、stack/locals 面板 |
| 源码-字节码双向映射 | 无 | 前端没有 `line <-> offset` 的映射表 |
| 前端 WebSocket 客户端 | 无 | 没有与后端 debug 通道的通信层 |

### 关键架构限制

1. **Playground 是纯 HTTP 请求/响应模型** — 没有长连接，无法支持调试器的"暂停-等待命令-继续"交互模型
2. **`run_handler` 直接调用 `run_with_capture()`** — 使用 `NoOpController`，调试器完全不介入，执行完才返回结果
3. **前端 CodeMirror 6 没有断点 decoration** — 当前 gutter 只支持点击高亮 transpile 输出行
4. **没有字节码到前端的传输管道** — `Disassembler` 的输出只在 CLI 使用，未暴露为 HTTP/WS API

---

## 设计决策

### 决策 1：WebSocket 而非 HTTP 轮询

**采用** WebSocket (`axum::extract::ws`) 作为调试通信通道。

原因：
- 调试器是事件驱动模型（VM 任意时刻命中断点后需立即通知前端）
- 双向通信：前端发 step/continue，后端 push state
- Axum 原生支持 WebSocket upgrade，实现简单
- 相比 SSE，WebSocket 支持前端向后端可靠发送命令

协议设计：
```json
// Client → Server
{ "type": "debug.start", "source": "fn main()..." }
{ "type": "command", "cmd": "step" }
{ "type": "command", "cmd": "continue" }
{ "type": "breakpoints.set", "lines": [3, 8, 15] }
{ "type": "debug.stop" }

// Server → Client
{ "type": "state", "status": "paused", "line": 5, "ip": 12, "stack": [...], "call_stack": [...] }
{ "type": "state", "status": "finished", "result": "42", "stdout": "..." }
{ "type": "state", "status": "error", "message": "..." }
{ "type": "bytecode", "lines": [{"offset": 0, "mnemonic": "const.i32", "operands": "42", "line": 1}, ...] }
```

### 决策 2：PlaygroundController — 异步条件变量模型

**采用** `tokio::sync::mpsc` + `tokio::sync::Notify` 实现 Controller 的暂停/继续逻辑。

原因：
- VM 执行在 `tokio::task::spawn_blocking` 中运行（已有 `run_handler` 模式）
- `DebuggerController` 要求是 `Send` trait，可以用跨线程/跨 await 的 channel
- `on_pause` 中阻塞等待前端命令，收到命令后解析为 `DebuggerAction`
- 复用 `AgentController` 中已有的 `DebugMode` / 步进逻辑

核心结构：
```rust
pub struct PlaygroundController {
    breakpoints: Arc<Mutex<HashSet<u32>>>,
    command_rx: Arc<Mutex<mpsc::Receiver<DebugCommand>>>,
    state_tx: mpsc::Sender<DebugState>,
    mode: Arc<Mutex<DebugMode>>,
    call_depth_at_pause: Arc<Mutex<usize>>,
}

enum DebugCommand {
    Continue,
    Step,
    StepOver,
    StepOut,
    SetBreakpoints(Vec<u32>),
    Stop,
}
```

### 决策 3：Debug Session 隔离

**采用** session-per-connection 模型：每个 WebSocket 连接对应一个独立的编译产物 + VM 实例。

原因：
- 避免多用户调试互相干扰
- VM 和 bytecode 不是 `Sync` 的，难以共享
- 实现简单：连接建立时编译源码、创建 VM；断开时清理资源

生命周期：
```
WebSocket connected
  → compile source (reuse auto_lang::compile_to_bytecode)
  → create AutoVM + load bytecode
  → spawn VM execution in blocking task
  → loop: receive WS message → send to Controller → VM resumes/pauses
  → on disconnect / stop: abort VM task, drop session
```

### 决策 4：字节码面板作为独立 tab（非替换 transpile）

**采用** OutputPanel 新增 **"bytecode"** tab，与 rust/c/python/ts 并列。

原因：
- 字节码是调试的辅助视图，不应替换 transpile 输出
- 用户可能同时想查看 transpile 结果和字节码
- 保留现有 Live Compile 功能不变

### 决策 5：行号映射复用 SOURCE_LINE

**不新增**独立的 source map 段，复用 VM 已有的 `SOURCE_LINE` opcode 信息。

原因：
- Plan 199 已在 codegen 中为每条 stmt 注入 `SOURCE_LINE`
- `DisasmLine.line` 已经包含源码行号
- 前端只需解析反汇编输出的 `line` 字段，即可建立 `offset <-> line` 映射

---

## Implementation Phases

---

### Phase 1: 后端 Debug 基础设施 — PlaygroundController + WebSocket API

**Goal:** 实现 VM 与前端之间的网络化调试桥接。新增 `PlaygroundController`、WebSocket handler、debug session 管理，使 VM 可以在浏览器控制下单步执行。

#### 1.1 新建 `PlaygroundController`

**文件:** `crates/auto-playground/src/debugger/controller.rs` (新建)

```rust
use auto_lang::vm::debugger::{DebuggerController, DebugContext, DebuggerAction, Breakpoint};
use tokio::sync::{mpsc, Mutex};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum DebugCommand {
    Continue,
    Step,
    StepOver,
    StepOut,
    SetBreakpoints(Vec<u32>),
    Stop,
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum DebugStatus {
    Paused,
    Running,
    Finished,
    Error,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DebugState {
    pub status: DebugStatus,
    pub line: u32,
    pub ip: usize,
    pub op: String,
    pub stack: Vec<String>,
    pub call_stack: Vec<CallFrameInfo>,
    pub locals: Vec<LocalInfo>,
    pub registers: RegisterInfo,
    pub stdout: String,
    pub stderr: String,
    pub result: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CallFrameInfo {
    pub fn_name: Option<String>,
    pub line: u32,
    pub return_ip: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LocalInfo {
    pub index: usize,
    pub value: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RegisterInfo {
    pub ip: usize,
    pub bp: usize,
    pub sp: usize,
}

enum DebugMode {
    Run,           // Continue until breakpoint
    Step,          // Step into (single instruction)
    StepOver,      // Step over (next source line, same depth)
    StepOut,       // Run until return from current function
}

pub struct PlaygroundController {
    breakpoints: Arc<Mutex<HashSet<u32>>>,
    command_rx: Arc<Mutex<mpsc::Receiver<DebugCommand>>>,
    state_tx: mpsc::Sender<DebugState>,
    mode: Arc<Mutex<DebugMode>>,
    depth_at_pause: Arc<Mutex<usize>>,
    stop_requested: Arc<Mutex<bool>>,
}

impl PlaygroundController {
    pub fn new(
        command_rx: mpsc::Receiver<DebugCommand>,
        state_tx: mpsc::Sender<DebugState>,
    ) -> Self {
        Self {
            breakpoints: Arc::new(Mutex::new(HashSet::new())),
            command_rx: Arc::new(Mutex::new(command_rx)),
            state_tx,
            mode: Arc::new(Mutex::new(DebugMode::Run)),
            depth_at_pause: Arc::new(Mutex::new(0)),
            stop_requested: Arc::new(Mutex::new(false)),
        }
    }
}

impl DebuggerController for PlaygroundController {
    fn should_pause(&mut self, ctx: &DebugContext) -> bool {
        let mode = self.mode.blocking_lock();
        let stop = *self.stop_requested.blocking_lock();
        if stop {
            return true;
        }
        match *mode {
            DebugMode::Run => {
                let bp = self.breakpoints.blocking_lock();
                bp.contains(&ctx.line)
            }
            DebugMode::Step => true,
            DebugMode::StepOver => {
                let depth = *self.depth_at_pause.blocking_lock();
                ctx.call_stack.len() > depth || ctx.line == 0
            }
            DebugMode::StepOut => {
                let depth = *self.depth_at_pause.blocking_lock();
                ctx.call_stack.len() < depth
            }
        }
    }

    fn on_pause(&mut self, ctx: &DebugContext) -> DebuggerAction {
        // 1. Build and send state to frontend
        let state = build_debug_state(ctx);
        let _ = self.state_tx.blocking_send(state);

        // 2. Wait for next command from frontend
        let mut rx = self.command_rx.blocking_lock();
        while let Some(cmd) = rx.blocking_recv() {
            match cmd {
                DebugCommand::Continue => {
                    *self.mode.blocking_lock() = DebugMode::Run;
                    return DebuggerAction::Continue;
                }
                DebugCommand::Step => {
                    *self.mode.blocking_lock() = DebugMode::Step;
                    *self.depth_at_pause.blocking_lock() = ctx.call_stack.len();
                    return DebuggerAction::Step;
                }
                DebugCommand::StepOver => {
                    *self.mode.blocking_lock() = DebugMode::StepOver;
                    *self.depth_at_pause.blocking_lock() = ctx.call_stack.len();
                    return DebuggerAction::StepOver;
                }
                DebugCommand::StepOut => {
                    *self.mode.blocking_lock() = DebugMode::StepOut;
                    *self.depth_at_pause.blocking_lock() = ctx.call_stack.len();
                    return DebuggerAction::StepOut;
                }
                DebugCommand::SetBreakpoints(lines) => {
                    let mut bp = self.breakpoints.blocking_lock();
                    bp.clear();
                    bp.extend(lines);
                    // Continue waiting for a real control command
                }
                DebugCommand::Stop => {
                    *self.stop_requested.blocking_lock() = true;
                    return DebuggerAction::Quit;
                }
            }
        }

        // Channel closed → stop
        DebuggerAction::Quit
    }
}

fn build_debug_state(ctx: &DebugContext) -> DebugState {
    let stack: Vec<String> = ctx.task.ram.raw[..ctx.task.ram.sp.min(256)]
        .iter()
        .map(|v| v.to_string())
        .collect();

    let call_stack: Vec<CallFrameInfo> = ctx.task.call_stack.iter().map(|f| CallFrameInfo {
        fn_name: f.fn_name.clone(),
        line: f.line,
        return_ip: f.return_ip,
    }).collect();

    let locals: Vec<LocalInfo> = (0..ctx.task.current_fn_n_locals)
        .map(|i| {
            let val = ctx.task.ram.read_i32(ctx.task.bp + 1 + i);
            LocalInfo { index: i, value: val }
        })
        .collect();

    DebugState {
        status: DebugStatus::Paused,
        line: ctx.line,
        ip: ctx.ip,
        op: ctx.current_op.to_mnemonic().to_string(),
        stack,
        call_stack,
        locals,
        registers: RegisterInfo {
            ip: ctx.task.ip,
            bp: ctx.task.bp,
            sp: ctx.task.ram.sp,
        },
        stdout: String::new(), // populated by session wrapper
        stderr: String::new(),
        result: None,
    }
}
```

**注意：** 由于 VM 执行在 `spawn_blocking` 中，`PlaygroundController` 使用 `blocking_lock()` / `blocking_recv()` 是安全的。如果未来 VM 改为 async 执行，需要改为 `async` 版本的 `DebuggerController` trait。

#### 1.2 新建 WebSocket Debug Session

**文件:** `crates/auto-playground/src/debugger/session.rs` (新建)

```rust
use auto_lang::vm::debugger::{DebuggerController, Breakpoint};
use auto_lang::vm::disasm::Disassembler;
use auto_lang::vm::virt_memory::VirtualFlash;
use axum::extract::ws::{WebSocket, Message};
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct DebugSession {
    pub ws: WebSocket,
    pub cmd_tx: mpsc::Sender<DebugCommand>,
    pub state_rx: mpsc::Receiver<DebugState>,
    pub vm_handle: tokio::task::JoinHandle<()>,
}

pub async fn run_debug_session(mut ws: WebSocket, source: String) {
    // 1. Compile source to bytecode
    let (flash, strings, entry_point, exports) = match compile_source(&source) {
        Ok(v) => v,
        Err(e) => {
            send_ws_msg(&mut ws, &json!({"type": "error", "message": e })).await;
            return;
        }
    };

    // 2. Disassemble and send bytecode to frontend immediately
    let disasm = Disassembler::new(&flash);
    let bytecode_lines = disasm.disassemble_range(0, flash.memory.len());
    let bytecode_json = serde_json::to_value(bytecode_lines).unwrap();
    send_ws_msg(&mut ws, &json!({"type": "bytecode", "lines": bytecode_json })).await;

    // 3. Setup controller channels
    let (cmd_tx, cmd_rx) = mpsc::channel::<DebugCommand>(16);
    let (state_tx, state_rx) = mpsc::channel::<DebugState>(16);

    let controller = PlaygroundController::new(cmd_rx, state_tx);

    // 4. Create VM and inject controller
    let mut vm = auto_lang::vm::engine::AutoVM::new(flash.clone(), 65536);
    vm.load_strings(strings);
    vm.set_debugger(Box::new(controller));
    vm.spawn_task(entry_point, 65536);

    // 5. Run VM in blocking task
    let vm_handle = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let _ = vm.run_task_loop().await;
        });
    });

    // 6. Forward state updates to WebSocket, commands from WebSocket to controller
    let mut session = DebugSession { ws, cmd_tx, state_rx, vm_handle };
    
    loop {
        tokio::select! {
            // Forward VM state → WebSocket
            Some(state) = session.state_rx.recv() => {
                let msg = json!({ "type": "state", "data": state });
                if send_ws_msg(&mut session.ws, &msg).await.is_err() {
                    break;
                }
            }
            // Forward WebSocket → Controller
            Some(Ok(msg)) = session.ws.recv() => {
                if let Ok(text) = msg.to_text() {
                    if let Ok(cmd) = parse_ws_command(text) {
                        let _ = session.cmd_tx.send(cmd).await;
                        if matches!(cmd, DebugCommand::Stop) {
                            break;
                        }
                    }
                }
            }
            else => break,
        }
    }

    session.vm_handle.abort();
}

fn compile_source(source: &str) -> Result<(VirtualFlash, Vec<String>, u32, HashMap<String, u32>), String> {
    // Reuse auto_lang's compile pipeline
    // Parse → codegen → link
    // Return (flash, strings, entry_point, exports)
    todo!()
}

fn parse_ws_command(text: &str) -> Result<DebugCommand, serde_json::Error> {
    let v: serde_json::Value = serde_json::from_str(text)?;
    match v["type"].as_str() {
        Some("debug.start") => Ok(DebugCommand::Continue),
        Some("command") => match v["cmd"].as_str() {
            Some("continue") => Ok(DebugCommand::Continue),
            Some("step") => Ok(DebugCommand::Step),
            Some("step_over") | Some("next") => Ok(DebugCommand::StepOver),
            Some("step_out") | Some("finish") => Ok(DebugCommand::StepOut),
            Some("stop") => Ok(DebugCommand::Stop),
            _ => Err(serde_json::Error::custom("unknown command")),
        },
        Some("breakpoints.set") => {
            let lines: Vec<u32> = serde_json::from_value(v["lines"].clone())?;
            Ok(DebugCommand::SetBreakpoints(lines))
        }
        _ => Err(serde_json::Error::custom("unknown message type")),
    }
}

async fn send_ws_msg(ws: &mut WebSocket, value: &serde_json::Value) -> Result<(), ()> {
    ws.send(Message::Text(value.to_string())).await.map_err(|_| ())
}
```

**关键决策：** VM 执行使用 `spawn_blocking` + `block_on`，因为 `AutoVM::run_task_loop()` 是异步的，但内部的 `DebuggerController` 调用是同步的（在 `run_one_instruction` 中同步调用）。`PlaygroundController` 内部使用 `blocking_recv` 阻塞等待前端命令，这是合理的。

#### 1.3 注册 WebSocket 路由

**文件:** `crates/auto-playground/src/main.rs`

在现有路由后新增：
```rust
use axum::routing::get;
use axum::extract::ws::WebSocketUpgrade;

async fn debug_ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| debugger::session::run_debug_session(socket))
}

// In app router:
let app = Router::new()
    .route("/api/run", post(routes::run::run_handler))
    .route("/api/trans", post(routes::trans::trans_handler))
    .route("/api/examples", get(routes::examples::examples_handler))
    .route("/api/debug/ws", get(debug_ws_handler))  // ← NEW
    .layer(cors);
```

**文件:** `crates/auto-playground/src/debugger/mod.rs` (新建)
```rust
pub mod controller;
pub mod session;
```

#### 1.4 `Cargo.toml` 依赖检查

**文件:** `crates/auto-playground/Cargo.toml`

确保已有：
```toml
[dependencies]
axum = { version = "0.7", features = ["ws"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
auto-lang = { path = "../auto-lang" }
```

**验收标准:**
- [ ] `PlaygroundController` 实现 `DebuggerController` trait，编译通过
- [ ] WebSocket `/api/debug/ws` 可连接，握手成功
- [ ] 发送 `{"type": "debug.start", "source": "fn main() { print(42) }"}` 后，VM 启动并命中初始断点或执行完成
- [ ] 发送 `{"type": "command", "cmd": "step"}` 后，VM 单步执行并返回新的 `state`
- [ ] 发送 `{"type": "debug.stop"}` 后，VM 任务被终止，连接关闭

---

### Phase 2: 前端 Debug 核心 — Bytecode 面板 + WebSocket 客户端

**Goal:** 前端建立与后端 debug 通道的连接，新增 Bytecode 显示面板，实现调试状态管理和字节码列表渲染。

#### 2.1 新建 `useDebugger` composable

**文件:** `crates/auto-playground/frontend/src/composables/useDebugger.ts` (新建)

```typescript
import { ref, computed } from 'vue';

export interface BytecodeLine {
  offset: number;
  mnemonic: string;
  operands: string;
  line?: number;
}

export interface CallFrameInfo {
  fn_name: string | null;
  line: number;
  return_ip: number;
}

export interface LocalInfo {
  index: number;
  value: number;
}

export interface RegisterInfo {
  ip: number;
  bp: number;
  sp: number;
}

export interface DebugState {
  status: 'paused' | 'running' | 'finished' | 'error';
  line: number;
  ip: number;
  op: string;
  stack: string[];
  call_stack: CallFrameInfo[];
  locals: LocalInfo[];
  registers: RegisterInfo;
  stdout: string;
  stderr: string;
  result: string | null;
}

export type DebugCommand = 'continue' | 'step' | 'step_over' | 'step_out' | 'stop';

export function useDebugger() {
  const ws = ref<WebSocket | null>(null);
  const isConnected = ref(false);
  const isDebugging = ref(false);
  const bytecode = ref<BytecodeLine[]>([]);
  const state = ref<DebugState | null>(null);
  const error = ref<string | null>(null);

  // Maps derived from bytecode
  const lineToOffsets = computed(() => {
    const map: Record<number, number[]> = {};
    for (const line of bytecode.value) {
      if (line.line !== undefined) {
        if (!map[line.line]) map[line.line] = [];
        map[line.line].push(line.offset);
      }
    }
    return map;
  });

  const offsetToLine = computed(() => {
    const map: Record<number, number> = {};
    for (const line of bytecode.value) {
      if (line.line !== undefined) {
        map[line.offset] = line.line;
      }
    }
    return map;
  });

  function connect(source: string) {
    if (ws.value) return;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const socket = new WebSocket(`${protocol}//${window.location.host}/api/debug/ws`);

    socket.onopen = () => {
      isConnected.value = true;
      isDebugging.value = true;
      // Send source to start debug session
      socket.send(JSON.stringify({ type: 'debug.start', source }));
    };

    socket.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      handleMessage(msg);
    };

    socket.onerror = (e) => {
      error.value = 'WebSocket error';
      console.error('Debug WS error:', e);
    };

    socket.onclose = () => {
      isConnected.value = false;
      isDebugging.value = false;
      ws.value = null;
    };

    ws.value = socket;
  }

  function handleMessage(msg: any) {
    switch (msg.type) {
      case 'bytecode':
        bytecode.value = msg.lines || [];
        break;
      case 'state':
        state.value = msg.data;
        if (msg.data.status === 'finished' || msg.data.status === 'error') {
          isDebugging.value = false;
        }
        break;
      case 'error':
        error.value = msg.message;
        isDebugging.value = false;
        break;
    }
  }

  function sendCommand(cmd: DebugCommand) {
    if (ws.value?.readyState === WebSocket.OPEN) {
      ws.value.send(JSON.stringify({ type: 'command', cmd }));
    }
  }

  function setBreakpoints(lines: number[]) {
    if (ws.value?.readyState === WebSocket.OPEN) {
      ws.value.send(JSON.stringify({ type: 'breakpoints.set', lines }));
    }
  }

  function stop() {
    sendCommand('stop');
    ws.value?.close();
    ws.value = null;
    isDebugging.value = false;
  }

  return {
    isConnected,
    isDebugging,
    bytecode,
    state,
    error,
    lineToOffsets,
    offsetToLine,
    connect,
    sendCommand,
    setBreakpoints,
    stop,
  };
}
```

#### 2.2 新增 Bytecode 面板组件

**文件:** `crates/auto-playground/frontend/src/components/BytecodePanel.vue` (新建)

```vue
<template>
  <div class="bytecode-panel">
    <div
      v-for="line in bytecode"
      :key="line.offset"
      :class="['bytecode-line', {
        'is-current': line.offset === currentIp,
        'is-highlighted': highlightedOffsets?.includes(line.offset),
        'has-source': line.line !== undefined,
      }]"
      @click="$emit('offsetClick', line.offset)"
    >
      <span class="offset">{{ formatOffset(line.offset) }}</span>
      <span class="mnemonic">{{ line.mnemonic }}</span>
      <span class="operands">{{ line.operands }}</span>
      <span v-if="line.line" class="line-info">; line {{ line.line }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { BytecodeLine } from '../composables/useDebugger';

defineProps<{
  bytecode: BytecodeLine[];
  currentIp?: number;
  highlightedOffsets?: number[];
}>();

defineEmits<{
  offsetClick: [offset: number];
}>();

function formatOffset(offset: number): string {
  return offset.toString(16).padStart(4, '0');
}
</script>

<style scoped>
.bytecode-panel {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  line-height: 1.6;
  overflow: auto;
  height: 100%;
  padding: 8px;
  background: #1e1e1e;
  color: #d4d4d4;
}
.bytecode-line {
  display: flex;
  gap: 12px;
  padding: 1px 4px;
  cursor: pointer;
  border-radius: 2px;
}
.bytecode-line:hover {
  background: #2a2d2e;
}
.bytecode-line.is-current {
  background: #0e639c;
  color: #fff;
}
.bytecode-line.is-highlighted {
  background: #3c3c3c;
}
.offset {
  color: #858585;
  min-width: 40px;
  user-select: none;
}
.mnemonic {
  color: #569cd6;
  min-width: 80px;
}
.operands {
  color: #9cdcfe;
  flex: 1;
}
.line-info {
  color: #6a9955;
  font-style: italic;
}
</style>
```

#### 2.3 改造 `OutputPanel` 添加 bytecode tab

**文件:** `crates/auto-playground/frontend/src/components/OutputPanel.vue`

在现有 tabs（rust/c/python/typescript）基础上新增 **"bytecode"** tab。当前 active tab 为 `OutputTab | 'bytecode'`。

当 `activeTab === 'bytecode'` 时，渲染 `<BytecodePanel>` 而非 transpile 输出。

#### 2.4 改造 `PlaygroundLayout` 和 `App.vue` 集成 debugger

**文件:** `crates/auto-playground/frontend/src/App.vue`

在 `usePlayground()` 旁引入 `useDebugger()`，将 debugger 状态和方法通过 props/events 传递到子组件。

新增 Debug 按钮在 Toolbar：
```vue
<button
  :class="['debug-btn', { active: debugger.isDebugging.value }]"
  @click="toggleDebug"
>
  {{ debugger.isDebugging.value ? 'Stop Debug' : 'Debug' }}
</button>
```

`toggleDebug` 逻辑：
- 若未在调试：调用 `debugger.connect(source.value)`
- 若在调试：调用 `debugger.stop()`

**验收标准:**
- [ ] 点击 Debug 按钮，WebSocket 连接建立，前端收到 `bytecode` 消息并显示字节码列表
- [ ] 字节码列表正确显示 `offset`, `mnemonic`, `operands`, `line`
- [ ] 运行简单代码 `fn main() { print(42) }`，能在 bytecode tab 中看到 `.line` / `const.i32` / `call` 等指令
- [ ] 调试结束后 WebSocket 正常关闭，无内存泄漏

---

### Phase 3: 源码 ↔ 字节码双向高亮 + 断点系统

**Goal:** 实现源码行与字节码指令的双向映射与高亮；在 CodeMirror gutter 中添加断点装饰，支持点击切换。

#### 3.1 CodeMirror 6 断点 Decoration

**文件:** `crates/auto-playground/frontend/src/lang/auto.ts` 或新建 `crates/auto-playground/frontend/src/components/debug/breakpointGutter.ts`

CodeMirror 6 的断点 gutter 实现：

```typescript
import { EditorState, StateField, StateEffect } from '@codemirror/state';
import { EditorView, Decoration, gutter, GutterMarker } from '@codemirror/view';

const breakpointEffect = StateEffect.define<number>({
  map: (val, mapping) => mapping.mapPos(val),
});

const breakpointState = StateField.define<Set<number>>({
  create() { return new Set(); },
  update(set, tr) {
    for (const e of tr.effects) {
      if (e.is(breakpointEffect)) {
        const line = e.value;
        const newSet = new Set(set);
        if (newSet.has(line)) newSet.delete(line);
        else newSet.add(line);
        return newSet;
      }
    }
    return set;
  },
});

class BreakpointMarker extends GutterMarker {
  toDOM() {
    const el = document.createElement('div');
    el.style.width = '10px';
    el.style.height = '10px';
    el.style.borderRadius = '50%';
    el.style.background = '#e51400';
    el.style.margin = '0 auto';
    el.style.cursor = 'pointer';
    return el;
  }
}

export const breakpointGutter = [
  breakpointState,
  gutter({
    class: 'cm-breakpoint-gutter',
    markers: (view) => {
      const markers = new RangeSetBuilder<GutterMarker>();
      const bps = view.state.field(breakpointState);
      for (const line of bps) {
        const pos = view.state.doc.line(line).from;
        markers.add(pos, pos, new BreakpointMarker());
      }
      return markers.finish();
    },
    initialSpacer: () => {
      const el = document.createElement('div');
      el.style.width = '16px';
      return el;
    },
    domEventHandlers: {
      mousedown(view, line) {
        const lineNo = view.state.doc.lineAt(line.from).number;
        view.dispatch({ effects: breakpointEffect.of(lineNo) });
        return true;
      }
    }
  }),
];

export function getBreakpoints(state: EditorState): number[] {
  return Array.from(state.field(breakpointState));
}

export function toggleBreakpoint(view: EditorView, line: number) {
  view.dispatch({ effects: breakpointEffect.of(line) });
}
```

**注意：** CodeMirror 6 的 `GutterMarker` 需要导入 `RangeSetBuilder`。如果项目版本较旧，可能需要使用 `RangeSet` 的静态方法。请根据实际安装的 `@codemirror/view` 版本调整。

#### 3.2 CodeMirror 当前执行行高亮

**文件:** 新建 `crates/auto-playground/frontend/src/components/debug/currentLine.ts`

```typescript
import { EditorState, StateField, StateEffect } from '@codemirror/state';
import { EditorView, Decoration } from '@codemirror/view';

const setCurrentLineEffect = StateEffect.define<number | null>();

const currentLineState = StateField.define<DecorationSet>({
  create() { return Decoration.none; },
  update(deco, tr) {
    for (const e of tr.effects) {
      if (e.is(setCurrentLineEffect)) {
        if (e.value === null) return Decoration.none;
        const line = tr.state.doc.line(e.value);
        return Decoration.set([
          Decoration.line({ class: 'cm-debug-current-line' }).range(line.from),
        ]);
      }
    }
    return deco.map(tr.mapping);
  },
});

export const currentLineHighlight = [
  currentLineState,
  EditorView.baseTheme({
    '.cm-debug-current-line': {
      backgroundColor: '#0e639c40',
      borderLeft: '2px solid #0e639c',
    },
  }),
];

export function setCurrentLine(view: EditorView, line: number | null) {
  view.dispatch({ effects: setCurrentLineEffect.of(line) });
}
```

#### 3.3 改造 `CodeEditor.vue` 集成断点和当前行高亮

**文件:** `crates/auto-playground/frontend/src/components/CodeEditor.vue`

新增 props：
```typescript
const props = defineProps<{
  modelValue: string;
  onRun?: () => void;
  breakpoints?: number[];
  currentLine?: number | null;
  isDebugging?: boolean;
}>();
```

在 `extensions` 数组中条件添加：
```typescript
if (props.isDebugging) {
  extensions.push(breakpointGutter);
  extensions.push(currentLineHighlight);
}
```

Watch `currentLine` 和 `breakpoints` 变化，调用 `setCurrentLine(view, line)` 或更新 breakpoint state。

监听断点变化并上报：
```typescript
// 在 updateListener 或定时检查中
const bps = getBreakpoints(editorView.state);
emit('breakpointsChange', bps);
```

#### 3.4 BytecodePanel 当前 IP 高亮 + 点击映射

**文件:** `crates/auto-playground/frontend/src/components/BytecodePanel.vue`

已在前面的组件中预留 `is-current` 和 `is-highlighted` class。

在 `useDebugger` 中新增：
```typescript
const currentBytecodeLine = computed(() => {
  return state.value?.ip ?? null;
});

const highlightedBytecodeOffsets = computed(() => {
  if (!highlightedSourceLine.value) return [];
  return lineToOffsets.value[highlightedSourceLine.value] ?? [];
});
```

点击 bytecode 行时：
```typescript
function onOffsetClick(offset: number) {
  const line = offsetToLine.value[offset];
  if (line) {
    emit('bytecodeLineClick', line);
  }
}
```

#### 3.5 `App.vue` 中连接双向高亮逻辑

```typescript
// 当源码行被点击（ transpile 的 line-click 或用户手动选择）
function onSourceLineClick(line: number) {
  debugger.highlightedSourceLine.value = line;
  // bytecode 高亮由 computed 自动推导
}

// 当字节码行被点击
function onBytecodeLineClick(line: number) {
  // 高亮源码对应行
  debugger.highlightedSourceLine.value = line;
}

// Watch debugger state，更新 CodeEditor 的 currentLine
watch(() => debugger.state.value?.line, (line) => {
  codeEditorCurrentLine.value = line ?? null;
});

// Watch breakpoints 变化，发送到后端
watch(breakpoints, (lines) => {
  debugger.setBreakpoints(lines);
}, { deep: true });
```

**验收标准:**
- [ ] Debug 模式下，点击源码 gutter 出现/消失红色断点圆点
- [ ] 断点变化实时同步到后端（通过 `breakpoints.set` WS 消息）
- [ ] VM 命中断点时，前端 CodeEditor 当前行高亮（蓝色背景）
- [ ] 点击源码行，右侧 bytecode 中对应的所有 offset 行高亮
- [ ] 点击 bytecode 行，左侧源码对应行高亮
- [ ] 非 Debug 模式下，不显示断点 gutter 和当前行高亮

---

### Phase 4: 完整调试 UI — Stack / Locals / Call Stack + Step 控制

**Goal:** 实现 Debug 控制栏（Step Into / Over / Out / Continue / Stop），以及底部 Debug 面板（Call Stack、Locals、Stack），形成完整的调试器界面。

#### 4.1 新建 `DebugToolbar` 组件

**文件:** `crates/auto-playground/frontend/src/components/DebugToolbar.vue` (新建)

```vue
<template>
  <div class="debug-toolbar">
    <button
      v-for="btn in buttons"
      :key="btn.cmd"
      :disabled="!isPaused"
      @click="$emit('command', btn.cmd)"
      :title="btn.title"
    >
      <span class="icon">{{ btn.icon }}</span>
      <span class="label">{{ btn.label }}</span>
    </button>
    <button class="stop-btn" @click="$emit('stop')" title="Stop Debugging">
      <span class="icon">■</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import type { DebugCommand } from '../composables/useDebugger';

defineProps<{ isPaused: boolean }>();
defineEmits<{
  command: [cmd: DebugCommand];
  stop: [];
}>();

const buttons = [
  { cmd: 'continue' as DebugCommand, icon: '▶', label: 'Continue', title: 'F5' },
  { cmd: 'step' as DebugCommand, icon: '↓', label: 'Step Into', title: 'F11' },
  { cmd: 'step_over' as DebugCommand, icon: '→', label: 'Step Over', title: 'F10' },
  { cmd: 'step_out' as DebugCommand, icon: '↑', label: 'Step Out', title: 'Shift+F11' },
];
</script>

<style scoped>
.debug-toolbar {
  display: flex;
  gap: 4px;
  padding: 4px 12px;
  background: #2d2d2d;
  border-bottom: 1px solid #444;
  align-items: center;
}
.debug-toolbar button {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: #3c3c3c;
  border: 1px solid #555;
  border-radius: 3px;
  color: #ccc;
  cursor: pointer;
  font-size: 12px;
}
.debug-toolbar button:hover:not(:disabled) {
  background: #4a4a4a;
  color: #fff;
}
.debug-toolbar button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.stop-btn {
  margin-left: auto;
  color: #e51400 !important;
}
</style>
```

#### 4.2 新建 `DebugPanel` 组件

**文件:** `crates/auto-playground/frontend/src/components/DebugPanel.vue` (新建)

```vue
<template>
  <div class="debug-panel">
    <div class="debug-tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id"
      >
        {{ tab.label }}
      </button>
    </div>
    <div class="debug-content">
      <!-- Stack -->
      <div v-if="activeTab === 'stack'" class="stack-view">
        <table>
          <thead>
            <tr><th>Index</th><th>Value</th></tr>
          </thead>
          <tbody>
            <tr
              v-for="(val, idx) in reversedStack"
              :key="idx"
              :class="{ 'is-top': idx === 0 }"
            >
              <td>{{ state.stack.length - 1 - idx }}</td>
              <td>{{ val }}</td>
            </tr>
          </tbody>
        </table>
        <div v-if="state.stack.length === 0" class="empty">Stack empty</div>
      </div>

      <!-- Call Stack -->
      <div v-if="activeTab === 'callstack'" class="callstack-view">
        <div
          v-for="(frame, idx) in reversedCallStack"
          :key="idx"
          class="frame-item"
        >
          <span class="frame-idx">#{{ reversedCallStack.length - 1 - idx }}</span>
          <span class="frame-name">{{ frame.fn_name ?? '<anonymous>' }}</span>
          <span class="frame-line">line {{ frame.line }}</span>
        </div>
        <div v-if="state.call_stack.length === 0" class="empty">No frames</div>
      </div>

      <!-- Locals -->
      <div v-if="activeTab === 'locals'" class="locals-view">
        <table>
          <thead>
            <tr><th>Slot</th><th>Value</th></tr>
          </thead>
          <tbody>
            <tr v-for="local in state.locals" :key="local.index">
              <td>[{{ local.index }}]</td>
              <td>{{ local.value }}</td>
            </tr>
          </tbody>
        </table>
        <div v-if="state.locals.length === 0" class="empty">No locals</div>
      </div>

      <!-- Registers -->
      <div v-if="activeTab === 'registers'" class="registers-view">
        <div class="reg-row"><span class="reg-name">IP</span><span class="reg-val">{{ formatHex(state.registers.ip) }}</span></div>
        <div class="reg-row"><span class="reg-name">BP</span><span class="reg-val">{{ formatHex(state.registers.bp) }}</span></div>
        <div class="reg-row"><span class="reg-name">SP</span><span class="reg-val">{{ formatHex(state.registers.sp) }}</span></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { DebugState } from '../composables/useDebugger';

const props = defineProps<{
  state: DebugState;
}>();

const activeTab = ref<'stack' | 'callstack' | 'locals' | 'registers'>('stack');

const tabs = [
  { id: 'stack' as const, label: 'Stack' },
  { id: 'callstack' as const, label: 'Call Stack' },
  { id: 'locals' as const, label: 'Locals' },
  { id: 'registers' as const, label: 'Registers' },
];

const reversedStack = computed(() => [...props.state.stack].reverse());
const reversedCallStack = computed(() => [...props.state.call_stack].reverse());

function formatHex(n: number): string {
  return `0x${n.toString(16).padStart(4, '0')} (${n})`;
}
</script>

<style scoped>
.debug-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1e1e1e;
  color: #d4d4d4;
  font-size: 13px;
}
.debug-tabs {
  display: flex;
  background: #2d2d2d;
  border-bottom: 1px solid #444;
}
.debug-tabs button {
  padding: 6px 14px;
  background: none;
  border: none;
  color: #ccc;
  cursor: pointer;
  font-size: 12px;
}
.debug-tabs button.active {
  background: #1e1e1e;
  color: #fff;
  border-bottom: 2px solid #0e639c;
}
.debug-content {
  flex: 1;
  overflow: auto;
  padding: 8px;
}
.stack-view table, .locals-view table {
  width: 100%;
  border-collapse: collapse;
}
.stack-view th, .locals-view th {
  text-align: left;
  padding: 4px;
  color: #858585;
  font-weight: 500;
  border-bottom: 1px solid #444;
}
.stack-view td, .locals-view td {
  padding: 3px 4px;
  font-family: monospace;
}
.stack-view .is-top td {
  background: #0e639c30;
  color: #fff;
}
.frame-item {
  padding: 4px;
  display: flex;
  gap: 8px;
}
.frame-idx { color: #858585; min-width: 28px; }
.frame-name { color: #9cdcfe; }
.frame-line { color: #6a9955; }
.reg-row {
  display: flex;
  gap: 12px;
  padding: 4px;
}
.reg-name { color: #569cd6; min-width: 40px; }
.reg-val { font-family: monospace; }
.empty { color: #858585; padding: 12px; text-align: center; }
</style>
```

#### 4.3 改造 `PlaygroundLayout` 添加 Debug 模式布局

**文件:** `crates/auto-playground/frontend/src/components/PlaygroundLayout.vue`

新增 props：
```typescript
defineProps<{
  // ... existing props
  isDebugging: boolean;
  isPaused: boolean;
  debugState: DebugState | null;
}>();
```

条件渲染：
- Debug 模式下：Toolbar 下方显示 `DebugToolbar`
- Console 区域可切换为 `DebugPanel`（新增 tab 切换或独立区域）

推荐布局（最小改动）：
```
┌─────────────────────────────────────────────┐
│ Toolbar + DebugToolbar (when debugging)      │
├──────────────────────┬──────────────────────┤
│   editor-pane        │   transpile-pane      │
│   (Auto 源码 +断点)   │   (Rust/C/Python/TS   │
│                      │    / Bytecode)        │
├──────────────────────┴──────────────────────┤
│  Console / DebugPanel (tab切换)              │
│  [Console] [Debug]                           │
└─────────────────────────────────────────────┘
```

底部 pane header 增加 tab 切换：
```vue
<div class="pane-header">
  <div class="tabs">
    <button :class="{ active: consoleTab === 'output' }" @click="consoleTab = 'output'">Console</button>
    <button v-if="isDebugging" :class="{ active: consoleTab === 'debug' }" @click="consoleTab = 'debug'">Debug</button>
  </div>
  <!-- run button -->
</div>
<div class="pane-body">
  <ConsoleOutput v-if="consoleTab === 'output'" ... />
  <DebugPanel v-else-if="consoleTab === 'debug' && debugState" :state="debugState" />
</div>
```

#### 4.4 App.vue 全局快捷键

**文件:** `crates/auto-playground/frontend/src/App.vue`

添加键盘监听（仅在 debug 模式）：
```typescript
onMounted(() => {
  window.addEventListener('keydown', onKeyDown);
});

function onKeyDown(e: KeyboardEvent) {
  if (!debugger.isDebugging.value) return;
  switch (e.key) {
    case 'F5': e.preventDefault(); debugger.sendCommand('continue'); break;
    case 'F10': e.preventDefault(); debugger.sendCommand('step_over'); break;
    case 'F11':
      e.preventDefault();
      debugger.sendCommand(e.shiftKey ? 'step_out' : 'step');
      break;
  }
}
```

#### 4.5 Stack 动态变化动画（可选增强）

在 `DebugPanel.vue` 的 Stack tab 中，对比前后两次 `stack` 数组：

```typescript
const prevStack = ref<string[]>([]);
const stackDiff = computed(() => {
  const curr = props.state.stack;
  const prev = prevStack.value;
  // items added (pushed)
  const added = curr.slice(prev.length);
  // items removed (popped)
  const removed = prev.slice(curr.length);
  return { added, removed };
});

watch(() => props.state.stack, (newVal) => {
  prevStack.value = newVal;
}, { flush: 'post' });
```

在模板中给新增的行添加 `.stack-pushed` class（绿色闪烁），给减少的位置做 `.stack-popped` 标记（红色）。

**验收标准:**
- [ ] Debug 模式下显示控制栏：Continue / Step Into / Step Over / Step Out / Stop
- [ ] 快捷键 F5(Continue)、F10(Step Over)、F11(Step Into)、Shift+F11(Step Out) 工作正常
- [ ] 底部 Debug 面板显示 Stack 内容，栈顶（SP 位置）高亮
- [ ] Call Stack 显示函数调用链，包含函数名和行号
- [ ] Locals 显示当前函数的局部变量 slot 和值
- [ ] Registers 显示 IP / BP / SP 的 hex 和十进制值
- [ ] VM 运行中（非 paused）时，Step 按钮禁用
- [ ] 单步执行时 Stack 变化有视觉反馈（push/pop 动画或颜色标记）

---

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| `DebuggerController` 是同步 trait，WebSocket 是 async | 中 | `PlaygroundController` 使用 `blocking_lock` + `blocking_recv`，VM 运行在 `spawn_blocking` 中 |
| CodeMirror 6 断点 gutter API 版本差异 | 低 | 先验证 `@codemirror/view` 版本，必要时用低层 `RangeSet` API |
| VM 执行 crash 导致 WebSocket 无响应 | 中 | `run_debug_session` 中用 `tokio::select!` + timeout，异常时发送 `error` 消息并关闭 |
| 多用户同时 debug 内存压力大 | 低 | 每个 session 独立 VM，设置内存上限；未来可加 session TTL |
| 反汇编大数据量传输慢 | 低 | `DisasmLine` 本身很小；必要时可分页传输 |

---

## 后续可扩展（非本 Plan 范围）

1. **表达式求值 / Watch**：Debug 面板增加输入框，输入 `a + b`，后端用 VM 的 `ram.read_i32` 或临时 AST 求值
2. **条件断点**：断点附带表达式，Controller 中 `should_pause` 时求值
3. **Hover 变量提示**：CodeMirror hover tooltip 显示当前 slot 值
4. **异常断点**：panic 时自动暂停，显示错误信息和 stack
5. **多 Task 调试**：AutoVM 支持 `spawn` 后，显示 task 列表并可切换
6. **源码 Step Over（语义级）**：当前 Step Over 基于 call_depth，未来可基于源码行号实现真正的"下一行"语义
7. **调试状态持久化 / 回放**：利用 `TraceCollector` 记录完整 trace，前端实现执行回放
