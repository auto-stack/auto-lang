# Plan 269: AutoVM Daemon + Stateful CLI (`auto serve` / `auto req`)

**Status**: Done (Step 1-6 complete, named pipe IPC working)
**Created**: 2026-05-28
**Updated**: 2026-05-28
**Related**: [Plan 265 (MCP Server)](../old/265-autovm-mcp-server.md)

## Context

AutoVM 已有三种交互方式：
1. **Human REPL**（`auto` 无参数）— rustyline 交互，人类专用
2. **MCP Server**（`auto mcp`）— JSON-RPC over stdio，需 Claude Code 的 MCP 基础设施
3. **单次执行**（`auto eval "code"`）— 无状态，每次新建 VM

**问题**：AI agent 无法像人类那样与 REPL 交互。MCP 需要 JSON-RPC 协议栈和 Claude Code 支持，不够通用。需要一个**任何 AI agent 都能用**的 stateful CLI。

**方案**：`auto serve` 启动后台 daemon 常驻进程，`auto req` 作为轻量 client 通过 named pipe 与 daemon 通信。Session 状态常驻内存（和 REPL 一样），client 只是薄薄的 I/O 桥接层。

```
AI agent (任何框架)                    Human (终端)
    │                                      │
    │  auto req -s ses_abc "a + 1"         │  auto serve --foreground
    │         │                            │       │
    └─────────┼────────────────────────────┘       │
              ▼                                    │
     Named Pipe (\\.\pipe\autovm)                  │
              │                                    │
              ▼                                    │
     ┌─────────────────────────┐                   │
     │  auto serve (daemon)     │◄──────────────────┘
     │                         │
     │  SessionManager         │
     │  ┌─────────────────┐    │
     │  │ ses_abc          │    │
     │  │ AutovmReplSession│    │
     │  │ (VM+codegen)     │    │
     │  └─────────────────┘    │
     │  ┌─────────────────┐    │
     │  │ ses_def          │    │
     │  │ AutovmReplSession│    │
     │  └─────────────────┘    │
     └─────────────────────────┘
```

## 设计

### 通信协议

**JSON Lines**（每行一个 JSON 对象，`\n` 分隔），比 MCP 的 JSON-RPC 轻量得多：

**Client → Daemon**:
```json
{"id": 1, "method": "new-session"}
{"id": 2, "session": "ses_abc", "method": "eval", "code": "let a = 1"}
{"id": 3, "session": "ses_abc", "method": "eval", "code": "a + 1"}
{"id": 4, "session": "ses_abc", "method": "inspect"}
{"id": 5, "session": "ses_abc", "method": "reset"}
```

**Daemon → Client**:
```json
{"id": 1, "status": "ok", "session": "ses_abc"}
{"id": 2, "status": "ok", "value": null}
{"id": 3, "status": "ok", "value": "2", "type": "int"}
{"id": 4, "status": "ok", "sessions": ["ses_abc"], "functions": [...], "variables": [...]}
{"id": 5, "status": "ok"}
```

**错误响应**:
```json
{"id": 3, "status": "error", "message": "Undefined variable: x"}
```

### IPC 传输层

使用 `tokio::net::windows::named_pipe`（Windows）和 `tokio::net::UnixListener`（Linux/macOS）。

- **Windows**: `\\.\pipe\autovm`
- **Unix**: `/tmp/autovm.sock`

tokio `full` features 已包含 `net`，无需额外依赖。

### 子命令设计

#### `auto serve`

```bash
auto serve                          # 后台启动 daemon
auto serve --foreground             # 前台运行（调试用）
auto serve --pipe-name mypipe       # 自定义 pipe 名称
auto serve --max-sessions 20        # 最大 session 数
auto serve --timeout 1800           # session 超时（秒）
```

#### `auto req`

```bash
auto req "1 + 2"                    # 匿名 session（自动创建+执行+删除）
auto req -s ses_abc "let a = 1"     # 指定 session
auto req -s ses_abc "a + 1"         # 复用 session
auto req --new-session              # 创建新 session，返回 session_id
auto req -s ses_abc --inspect       # 查看 session 状态
auto req -s ses_abc --reset         # 重置 session
auto req -s ses_abc --delete        # 删除 session
auto req -s ses_abc --snapshot      # 导出 session 为 .at 文件
auto req --list                     # 列出所有活跃 session
auto req --json "1 + 2"             # JSON 输出模式（机器可读）
```

**匿名模式**：不指定 `-s` 时，自动创建临时 session，执行后删除。适合一次性表达式求值。

## 实施步骤

### Step 1: 创建 daemon 模块

**新建**: `crates/auto-lang/src/autovm_daemon.rs`

实现 daemon 核心逻辑：
- `AutovmDaemon` struct，内含 `SessionManager`（复用 MCP 的）
- `async fn run(pipe_name: &str)` — 监听 named pipe，accept 连接
- `fn handle_request(req: Request) -> Response` — 分发到 session 操作
- 复用 `AutovmReplSession::run()` 执行代码

关键：从 `mcp/session_manager.rs` 提取 `SessionManager`（已在 mcp 模块中，直接 `use`）。

### Step 2: 创建 client 模块

**新建**: `crates/auto-lang/src/autovm_client.rs`

实现 client 连接逻辑：
- `fn send_request(pipe_name: &str, req: Request) -> Result<Response>`
- 连接到 named pipe / Unix socket
- 发送 JSON line，读取响应 JSON line
- 超时处理（默认 10 秒）

### Step 3: 添加 CLI 子命令

**修改**: `crates/auto/src/main.rs`

在 `Commands` enum 中添加：
```rust
/// Start AutoVM daemon server (named pipe transport)
Serve {
    #[arg(short, long, default_value = "autovm")]
    pipe_name: String,
    #[arg(long)]
    foreground: bool,
    #[arg(long, default_value_t = 20)]
    max_sessions: usize,
    #[arg(long, default_value_t = 1800)]
    timeout: u64,
},

/// Send request to AutoVM daemon
Req {
    #[arg(short, long, default_value = "autovm")]
    pipe_name: String,
    #[arg(short, long)]
    session: Option<String>,
    #[arg(long)]
    new_session: bool,
    #[arg(long)]
    inspect: bool,
    #[arg(long)]
    reset: bool,
    #[arg(long)]
    delete: bool,
    #[arg(long)]
    snapshot: bool,
    #[arg(long)]
    list: bool,
    #[arg(long)]
    json: bool,
    /// Auto code to evaluate
    code: Option<String>,
},
```

在 `match cli.command` 中添加对应的 dispatch 逻辑。

### Step 4: 跨平台 IPC 抽象

在 `autovm_daemon.rs` 中：
```rust
#[cfg(target_family = "windows")]
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};

#[cfg(target_family = "unix")]
use tokio::net::UnixListener;
```

封装 `bind()` 和 `accept()` 为平台无关的 async 函数。

### Step 5: daemon 后台模式

`auto serve`（不带 `--foreground`）时：
1. fork/spawn 子进程
2. 子进程执行 daemon 循环
3. 父进程打印 "AutoVM daemon started" 并退出

Windows 上用 `Command::new(std::env::current_exe()).args(["serve", "--foreground"])` + `creation_flags(CREATE_NO_WINDOW)`。
Unix 上用 `daemon()` 或 double-fork。

### Step 6: 注册模块

**修改**: `crates/auto-lang/src/lib.rs` — 添加 `pub mod autovm_daemon;` 和 `pub mod autovm_client;`

## 关键文件

- **新建**: `crates/auto-lang/src/autovm_daemon.rs` — daemon 服务端（监听 pipe，管理 session）
- **新建**: `crates/auto-lang/src/autovm_client.rs` — client 端（连接 pipe，发送请求）
- **修改**: `crates/auto/src/main.rs` — 添加 `Serve` 和 `Req` 子命令
- **修改**: `crates/auto-lang/src/lib.rs` — 注册新模块
- **复用**: `crates/auto-lang/src/mcp/session_manager.rs` — Session 管理器
- **复用**: `crates/auto-lang/src/autovm_persistent.rs` — AutovmReplSession 核心

## 验证

```bash
# 1. 启动 daemon
auto serve --foreground &
# 或后台模式
auto serve

# 2. 创建 session
auto req --new-session
# 输出: ses_abc123

# 3. 在 session 中逐步执行
auto req -s ses_abc123 "let a = 1"
auto req -s ses_abc123 "a"
# 输出: 1
auto req -s ses_abc123 "let a = 3"
auto req -s ses_abc123 "a"
# 输出: 3

# 4. 匿名模式（一次性）
auto req "1 + 2"
# 输出: 3

# 5. JSON 模式（AI agent 友好）
auto req --json "1 + 2"
# 输出: {"status":"ok","value":"3","type":"int"}

# 6. Inspect session
auto req -s ses_abc123 --inspect

# 7. 运行测试
cargo test -p auto-lang -- autovm_daemon
```
