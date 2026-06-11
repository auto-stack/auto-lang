# Plan 296: ASH Shell 外部命令执行架构升级

## Context

当前 ASH shell 的外部命令执行存在根本性缺陷：使用 `std::process::Command::output()` 阻塞等待进程结束，期间无任何输出。这导致 `cargo build` 等长任务看起来像"卡住了"。

当前已做临时修复：独立命令用 `.status()`（继承 stdio，实时输出），管道中用 `.output()`（捕获输出传下游）。但这只是权宜之计，缺少流式处理、交互式命令支持、信号处理等关键能力。

本文档参考 nushell（D:/github/nushell）的实现，设计一套完整的外部命令执行架构。

## 命令分类与处理方案

### 类型矩阵

| 类型 | 示例 | stdin | stdout/stderr | 输出方式 | 当前状态 |
|---|---|---|---|---|---|
| A. 内置结构化 | `ls`, `ps`, `grep` | N/A | AtomPipeline | 表格/结构化渲染 | ✅ 已实现 |
| B. 外部快速 | `git status`, `echo hi` | inherit | inherit | 直接到终端 | ✅ 已修复 |
| C. 外部长任务 | `cargo build`, `npm install` | inherit | inherit | 实时流到终端 | ✅ 已修复 |
| D. 管道中的外部 | `cargo build \| grep error` | pipe | pipe | 捕获→下游 | ⚠️ 临时方案 |
| E. 交互式 | `vim`, `less`, `top` | inherit | inherit | 完全接管终端 | ❌ 未处理 |
| F. 流式+结构化 | `cat data.json \| from json` | pipe | pipe→Atom | 边读边解析 | ❌ 未处理 |

### 各类型的处理策略

#### 类型 A：内置结构化命令（已实现）
- 不涉及外部进程
- 通过 `run_atom()` 产出 `AtomPipeline`
- `format_output()` 渲染为表格/文本
- **无需改动**

#### 类型 B/C：独立外部命令（已修复）
- `.status()` + `Stdio::inherit()`
- 子进程直接写终端，实时可见
- `Ok(None)` 表示输出已到终端，REPL 不再 println
- **无需改动**

#### 类型 D：管道中的外部命令
- 需要从 `.output()`（全量缓冲）升级为 **流式处理**
- 参考 nushell 的 `ByteStream::child()` 模式

**目标架构**（参考 nushell）：
```
External Command
    ↓ (Stdio::piped)
ChildProcess (封装 spawn 的子进程)
    ↓ (stdout pipe reader)
ByteStream (异步字节流，支持逐行读取)
    ↓
AtomPipeline::stream() (流式 Atom 管道)
    ↓
下游命令 (逐行消费，而非全量等待)
```

**核心改动**：
1. 新增 `ChildProcess` 结构体 —— 封装 `std::process::Child`，后台线程等待 exit status
2. 新增 `ByteStream` 类型 —— 包装 `BufReader<ChildStdout>`，实现 `Read` + `Iterator<Item=String>`
3. `AtomPipeline` 新增 `Stream` 变体 —— 持有 `ByteStream`，支持逐行消费
4. 下游命令从 `.into_text()` 全量读取改为 `.lines()` 逐行处理

#### 类型 E：交互式命令
- 需要将终端从 reedline 的 raw mode 切换回 normal mode
- 子进程完全接管 stdin/stdout/stderr
- 进程结束后恢复 raw mode，reedline 重新接管

**Unix 方案**（参考 nushell `ForegroundChild`）：
1. 为交互式进程创建独立进程组（`setpgid`）
2. 将终端前台控制权交给子进程（`tcsetpgrp`）
3. 子进程结束后收回控制权
4. 支持 Ctrl+Z 挂起 → `FrozenJob` → `fg` 恢复

**Windows 方案**：
- Windows 无进程组概念，`Command::new().status()` 本身就能正确处理
- 只需处理 reedline 的 raw mode 切换

**检测交互式命令的启发式**：
- 已知的交互式命令白名单：`vim`, `vi`, `nano`, `less`, `more`, `top`, `htop`, `man`, `ssh`, `telnet`
- 或者：检测 stdin 是否为终端 + 命令不在管道中

#### 类型 F：流式+结构化（远期目标）
- `ByteStream` → 边读边解析为 Atom
- 例如 `cat large.json | from json` 逐行解析
- 暂不实现，作为未来扩展点

## 参考架构：nushell 的分层设计

```
nushell 分层：
┌─────────────────────────────┐
│ PipelineData                 │  统一管道抽象
│   ├ Value (结构化数据)        │
│   ├ ByteStream (字节流)      │
│   └ Empty                    │
├─────────────────────────────┤
│ ByteStream                   │  流式抽象
│   └ ChildProcess             │  子进程封装
│       ├ ForegroundChild      │  进程组/前台控制
│       └ exit_status_thread   │  后台等待退出码
├─────────────────────────────┤
│ OutDest                      │  输出目标路由
│   ├ Inherit → Stdio::inherit │
│   ├ Pipe → Stdio::piped      │
│   ├ Value → 立即收集         │
│   └ Null → Stdio::null      │
└─────────────────────────────┘
```

## ASH 的目标架构

```
ASH 分层（对应 nushell）：
┌─────────────────────────────┐
│ AtomPipeline                 │  统一管道抽象
│   ├ Atoms (Vec<Atom>)        │  结构化数据（已有）
│   ├ Text (String)            │  纯文本（已有）
│   └ Stream (ByteStream)     │  字节流（新增）
├─────────────────────────────┤
│ ByteStream（新增）            │  流式抽象
│   └ ChildProcess（新增）     │  子进程封装
│       ├ exit_status_thread   │  后台等待退出码
│       └ signal_check         │  Ctrl+C 中断检查
├─────────────────────────────┤
│ ExternalCommand              │  执行策略选择
│   ├ standalone → status()   │  B/C 类：inherit stdio
│   ├ piped → spawn() + pipe  │  D 类：捕获输出
│   └ interactive → 前台进程   │  E 类：接管终端
└─────────────────────────────┘
```

## 实施计划

### Phase 1：流式外部命令（类型 D 完善）

**文件改动**：
- `ash-core/src/cmd/external.rs` — 新增 `ChildProcess` + `ByteStream`
- `ash-core/src/pipeline/mod.rs` — `AtomPipeline` 新增 `Stream` 变体
- `auto-shell/src/shell.rs` — 管道执行使用新的流式 API

**核心实现**：

```rust
// ash-core/src/cmd/byte_stream.rs（新文件）
pub struct ChildProcess {
    child: std::process::Child,
    exit_status: Arc<Mutex<Option<ExitStatus>>>,
}

pub struct ByteStream {
    reader: BufReader<ChildStdout>,
    source: Arc<ChildProcess>,
}

impl ByteStream {
    pub fn lines(self) -> impl Iterator<Item = Result<String>> { ... }
    pub fn read_all(self) -> Result<String> { ... }
}

// AtomPipeline 新增变体
pub enum AtomPipeline {
    Atoms(Vec<Atom>),
    Text(String),
    Stream(ByteStream),  // 新增
    Empty,
}
```

**改动要点**：
- `AtomPipeline::into_text()` 对 Stream 变体调用 `read_all()`
- `AtomPipeline::lines()` 新方法，对 Stream 返回逐行迭代器
- 下游命令可选择全量收集或逐行处理

### Phase 2：交互式命令支持（类型 E）

**文件改动**：
- `ash-core/src/cmd/external.rs` — 新增 `execute_interactive()`
- `auto-shell/src/frontend/repl.rs` — raw mode 切换逻辑
- `ash-core/src/cmd/interactive.rs`（新文件）— 交互式命令检测和进程管理

**核心实现**：

```rust
// ash-core/src/cmd/interactive.rs
const INTERACTIVE_COMMANDS: &[&str] = &[
    "vim", "vi", "nano", "emacs",
    "less", "more", "bat",
    "top", "htop", "btop",
    "man", "info",
    "ssh", "telnet", "mosh",
    "screen", "tmux",
];

pub fn is_interactive_command(cmd: &str) -> bool {
    let name = cmd.split_whitespace().next().unwrap_or("");
    let name = Path::new(name).file_name().unwrap_or_default().to_str().unwrap_or("");
    INTERACTIVE_COMMANDS.contains(&name)
}
```

**REPL 集成**：
```rust
// repl.rs 中执行命令前
if is_interactive_command(&line) {
    // 1. 暂停 reedline（释放 raw mode）
    // 2. 执行命令（inherit stdio）
    // 3. 等待完成
    // 4. 恢复 reedline（重新进入 raw mode）
}
```

### Phase 3：信号处理与 Job Control（远期）

- Ctrl+C 中断外部命令
- Ctrl+Z 挂起（Unix only）
- `fg`/`bg` 命令恢复挂起的任务
- 参考 nushell 的 `ForegroundChild` + `FrozenJob` 机制

## 验证方案

### Phase 1 验证
```bash
# 类型 B/C：独立命令，实时输出
cargo build -p auto          # 应实时看到编译输出

# 类型 D：管道中的外部命令
echo hello | grep hello       # 应正确传递输出
ls | sort                     # 内置→外部管道
```

### Phase 2 验证
```bash
# 类型 E：交互式命令
vim                          # 应正确接管终端，退出后恢复 shell
less README.md               # 应正确接管终端
git log                      # 非交互式，应正常分页（或直接流式输出）
```

## 关键参考文件（nushell）

| 概念 | nushell 文件 | 用途 |
|---|---|---|
| 外部命令执行 | `nu-command/src/system/run_external.rs` | spawn + stdio 配置 |
| 字节流 | `nu-protocol/src/pipeline/byte_stream.rs` | 流式抽象 |
| 子进程封装 | `nu-protocol/src/process/child.rs` | exit status 线程 |
| 前台进程控制 | `nu-system/src/foreground.rs` | 进程组 + tcsetpgrp |
| 输出目标 | `nu-protocol/src/pipeline/out_dest.rs` | OutDest 路由枚举 |
