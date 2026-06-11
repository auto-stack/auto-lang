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
| D. 管道中的外部 | `cargo build \| grep error` | pipe | pipe | 捕获→下游 | ✅ 已实现 |
| E. 交互式 | `vim`, `less`, `top` | inherit | inherit | 完全接管终端 | ✅ 已实现 |
| F. 流式+结构化 | `cat data.json \| from json` | pipe | pipe→Atom | 边读边解析 | ❌ 远期目标 |

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

#### 类型 D：管道中的外部命令（已实现）
- `ExternalStream` 封装 `std::process::Child`，后台线程等待 exit status
- `AtomPipeline::ExternalStream` 变体持有 `ExternalStream`，支持 `lines()` 逐行迭代和 `read_all()` 全量读取
- 上游管道输出通过 `ExternalStream::new_with_stdin()` 管道到子进程 stdin
- `spawn_external_stream()` / `spawn_external_stream_with_input()` 提供平台级 fallback（Windows PowerShell / Unix sh）

**实际架构**：
```
External Command (spawned with Stdio::piped)
    ↓ (stdout pipe reader)
ExternalStream (BufReader<ChildStdout> + background exit-status thread)
    ↓
AtomPipeline::ExternalStream (流式管道变体)
    ↓
下游命令 (可逐行 lines() 或全量 read_all())
```

**已实现改动**：
1. `ExternalStream` 结构体 — 封装 `BufReader<ChildStdout>`，后台线程收集 exit status
2. `ExternalStream::new_with_stdin()` — 支持管道上游输出通过 stdin 传入子进程
3. `AtomPipeline::ExternalStream` 变体 — 持有 `ExternalStream`，支持逐行消费
4. `spawn_external_stream_with_input()` — 带 stdin 管道的 spawn，含 Windows/Unix fallback

#### 类型 E：交互式命令（已实现）
- `interactive.rs` 提供 `is_interactive_command()` 检测
- 白名单涵盖：编辑器 (vim/nano/emacs/helix)、分页器 (less/more)、系统监控 (top/htop/btop)、远程 (ssh/telnet/mosh)、终端复用器 (tmux/screen)、调试器 (gdb/lldb)、REPL (python/node)、数据库 (psql/mysql/sqlite3)
- REPL 在命令执行前检测交互式命令，直接调用 `execute_external()` with inherit stdio
- Windows 上 `.status()` 自动处理 console mode；Unix 上 reedline raw mode 不阻塞子进程

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

## ASH 的目标架构（实际实现）

```
ASH 分层（对应 nushell）：
┌───────────────────────────────────┐
│ AtomPipeline                       │  统一管道抽象
│   ├ Atom(Atom)                     │  结构化数据
│   ├ Stream(AtomStream)             │  内存流
│   ├ ExternalStream(ExternalStream) │  外部进程字节流
│   ├ Text(String)                   │  纯文本
│   └ Empty                          │  无数据
├───────────────────────────────────┤
│ ExternalStream                     │  流式抽象
│   ├ reader: BufReader<ChildStdout> │  逐行/全量读取
│   ├ new_with_stdin(data)           │  管道上游→stdin
│   └ exit_status_thread             │  后台等待退出码
├───────────────────────────────────┤
│ ExternalCommand                    │  执行策略选择
│   ├ standalone → status()         │  B/C 类：inherit stdio
│   ├ piped → spawn() + pipe stdin  │  D 类：捕获输出+管道输入
│   └ interactive → inherit stdio   │  E 类：接管终端
├───────────────────────────────────┤
│ Shell.last_exit_code               │  $? 退出码追踪
│   ├ 0 = 成功                       │
│   ├ N = 外部命令实际退出码          │
│   └ 1 = 其他错误                    │
└───────────────────────────────────┘
```

## 实施计划

### Phase 1：流式外部命令（类型 D）— ✅ 已完成

**已实现文件**：
- `ash-core/src/pipeline/external_stream.rs` — `ExternalStream` 结构体（等同于 `ByteStream` + `ChildProcess`）
- `ash-core/src/cmd/external.rs` — `spawn_external_stream()` + `spawn_external_stream_with_input()`
- `ash-core/src/pipeline/atom_pipeline.rs` — `AtomPipeline::ExternalStream` 变体
- `auto-shell/src/shell.rs` — 管道执行使用流式 API，上游输出管道到外部命令 stdin

**已实现功能**：
1. `ExternalStream` — 封装 `BufReader<ChildStdout>` + 后台 exit status 线程
2. `ExternalStream::new_with_stdin()` — stdin 管道支持（后台线程写入数据）
3. `lines()` / `read_all()` — 逐行迭代和全量读取
4. `spawn_external_stream_with_input()` — 带 stdin 的 spawn，含 Windows/Unix fallback
5. 管道执行中，上游输出自动管道到外部命令 stdin

### Phase 2：交互式命令支持（类型 E）— ✅ 已完成

**已实现文件**：
- `ash-core/src/cmd/interactive.rs` — 交互式命令白名单检测
- `auto-shell/src/frontend/repl.rs` — REPL 中交互式命令执行

**已实现功能**：
1. `is_interactive_command()` — 白名单 + 路径解析 + Windows .exe 剥离
2. REPL 在执行前检测交互式命令，绕过管道系统直接 inherit stdio 执行
3. 覆盖编辑器、分页器、监控、远程、复用器、调试器、REPL、数据库客户端

### Phase 3：退出码追踪 — ✅ 已完成

**已实现文件**：
- `auto-shell/src/shell.rs` — `last_exit_code` 字段、`$?` 变量展开、`extract_exit_code()` 辅助函数

**已实现功能**：
1. `Shell.last_exit_code` — 每次命令执行后更新（0=成功，N=外部命令实际退出码，1=其他错误）
2. `$?` 变量展开 — `get_variable("?")` 返回上次退出码
3. 外部命令退出码提取 — 从错误消息解析 `"exit code: N"` 获取真实退出码
4. `Shell::execute()` 重置退出码，`execute_inner()` 可覆盖

### Phase 4：信号处理与 Job Control（远期）

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
