# Plan 296: ASH Shell 外部命令执行架构升级

## Context

ASH shell 的外部命令执行架构已完成全面升级，实现了流式管道、交互式命令支持、退出码追踪、Ctrl+C 保护、流式迭代器和任务控制六大能力。本文档记录完整的架构设计和实现。

原始问题：外部命令使用 `std::process::Command::output()` 阻塞等待进程结束，期间无任何输出，导致 `cargo build` 等长任务看起来像"卡住了"。

本文档参考 nushell（D:/github/nushell）的实现，设计了一套完整的外部命令执行架构。

## 命令分类与处理方案

### 类型矩阵

| 类型 | 示例 | stdin | stdout/stderr | 输出方式 | 状态 |
|---|---|---|---|---|---|
| A. 内置结构化 | `ls`, `ps`, `grep` | N/A | AtomPipeline | 表格/结构化渲染 | ✅ |
| B. 外部快速 | `git status`, `echo hi` | inherit | inherit | 直接到终端 | ✅ |
| C. 外部长任务 | `cargo build`, `npm install` | inherit | inherit | 实时流到终端 | ✅ |
| D. 管道中的外部 | `cargo build \| grep error` | pipe | pipe | 捕获→下游 | ✅ |
| E. 交互式 | `vim`, `less`, `top` | inherit | inherit | 完全接管终端 | ✅ |
| F. 流式+结构化 | `cat data.json \| from json` | pipe | pipe→Atom | 边读边解析 | ✅ 部分（into_lines） |
| G. 后台任务 | `cargo build &` | null | inherit | 后台运行 | ✅ |

### 各类型的处理策略

#### 类型 A：内置结构化命令
- 不涉及外部进程
- 通过 `run_atom()` 产出 `AtomPipeline`
- `format_output()` 渲染为表格/文本

#### 类型 B/C：独立外部命令
- `.status()` + `Stdio::inherit()`
- 子进程直接写终端，实时可见
- `Ok(None)` 表示输出已到终端，REPL 不再 println

#### 类型 D：管道中的外部命令
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

#### 类型 E：交互式命令
- `interactive.rs` 提供 `is_interactive_command()` 检测
- 白名单涵盖：编辑器 (vim/nano/emacs/helix)、分页器 (less/more)、系统监控 (top/htop/btop)、远程 (ssh/telnet/mosh)、终端复用器 (tmux/screen)、调试器 (gdb/lldb)、REPL (python/node)、数据库 (psql/mysql/sqlite3)
- REPL 在命令执行前检测交互式命令，直接调用 `execute_external()` with inherit stdio
- Windows 上 `.status()` 自动处理 console mode；Unix 上 reedline raw mode 不阻塞子进程

#### 类型 F：流式+结构化
- `into_lines()` 提供逐行流式迭代器，避免全量缓冲
- `GrepCommand::run_atom_streaming()` 已实现 ExternalStream 逐行匹配
- 完整的 `ByteStream → Atom` 边读边解析（如 `cat large.json | from json`）为远期扩展点

#### 类型 G：后台任务
- `cmd &` 语法将命令放入后台执行
- `JobManager` 追踪所有后台和挂起任务
- `jobs`/`fg`/`bg` 内置命令管理任务生命周期

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

## ASH 架构（实际实现）

```
ASH 分层（对应 nushell）：
┌───────────────────────────────────┐
│ AtomPipeline                       │  统一管道抽象（对应 PipelineData）
│   ├ Atom(Atom)                     │  结构化数据（对应 Value）
│   ├ Stream(AtomStream)             │  内存流
│   ├ ExternalStream(ExternalStream) │  外部进程字节流（对应 ByteStream）
│   ├ Text(String)                   │  纯文本
│   └ Empty                          │  无数据
├───────────────────────────────────┤
│ ExternalStream                     │  流式抽象（对应 ByteStream + ChildProcess）
│   ├ reader: BufReader<ChildStdout> │  逐行/全量读取
│   ├ new_with_stdin(data)           │  管道上游→stdin
│   └ exit_status_thread             │  后台等待退出码
├───────────────────────────────────┤
│ ExternalCommand                    │  执行策略选择（对应 OutDest）
│   ├ standalone → status()         │  B/C 类：inherit stdio
│   ├ piped → spawn() + pipe stdin  │  D 类：捕获输出+管道输入
│   ├ interactive → inherit stdio   │  E 类：接管终端
│   └ background → spawn() + null  │  G 类：后台执行
├───────────────────────────────────┤
│ Shell.last_exit_code               │  $? 退出码追踪
│   ├ 0 = 成功                       │
│   ├ N = 外部命令实际退出码          │
│   └ 1 = 其他错误                    │
├───────────────────────────────────┤
│ CtrlCGuard                         │  Ctrl+C 信号保护
│   ├ Windows: SetConsoleCtrlHandler │
│   └ Unix: signal(SIGINT, handler)  │
├───────────────────────────────────┤
│ JobManager                         │  任务管理
│   ├ add() → 注册后台任务           │
│   ├ reap_finished() → 非阻塞轮询   │
│   ├ suspend_job() → 平台级挂起     │
│   ├ resume_job() → 平台级恢复      │
│   └ format_jobs() → 显示任务列表   │
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

### Phase 4：信号处理 — ✅ 已完成

**已实现文件**：
- `auto-shell/src/signal.rs` — 平台级 Ctrl+C 保护（零外部依赖，raw FFI）
- `ash-core/src/cmd/external.rs` — Unix `pre_exec` 恢复子进程 SIGINT
- `auto-shell/src/frontend/repl.rs` — 启动时初始化 Ctrl+C handler

**已实现功能**：
1. `CtrlCGuard` RAII 守卫 — 执行命令时 ASH 忽略 Ctrl+C，命令结束后恢复
2. Windows: `SetConsoleCtrlHandler` 自定义 handler（AtomicBool 控制行为）
3. Unix: `signal(SIGINT, handler)` + `pre_exec` 恢复子进程 SIG_DFL
4. 子进程正常接收 Ctrl+C 并退出，ASH 存活继续运行
5. 不干扰 reedline 的 Ctrl+C 处理（仅在命令执行期间保护）

### Phase 5：流式迭代器 — ✅ 已完成

**已实现文件**：
- `ash-core/src/pipeline/atom_pipeline.rs` — `into_lines()` 方法
- `auto-shell/src/cmd/commands/grep.rs` — ExternalStream 逐行流式处理

**已实现功能**：
1. `AtomPipeline::into_lines()` — 返回 `Box<dyn Iterator<Item = String>>`
   - ExternalStream: 从 pipe 逐行读取（零缓冲）
   - Text: 按 newline 分割
   - Atom/Stream: 转文本后分割
   - Empty: 空迭代器
2. `GrepCommand::run_atom_streaming()` — ExternalStream 输入时逐行匹配
   - 避免先全量读取到内存
   - 支持所有 grep 选项（-i, -v, -c, -n）

### Phase 6：Job Control — ✅ 已完成（核心功能）

**已实现文件**：
- `auto-shell/src/job.rs` — `Job` 结构体、`JobManager`、平台级 suspend/resume（367 行）
- `auto-shell/src/shell.rs` — `&` 后缀解析、`jobs`/`fg`/`bg`/`suspend` 内置命令、JobManager 集成
- `ash-core/src/cmd/external.rs` — `spawn_external_background()` + `try_spawn_background()`

**已实现功能**：
1. `cmd &` — 末尾 `&` 解析，spawn 子进程（stdout/stderr inherit，stdin null），不等待
2. `jobs` — 列出所有后台/挂起任务（自动 reap 已完成任务）
3. `fg [N]` — 将后台/挂起任务拉到前台等待（若 stopped 先 resume）
4. `bg [N]` — 恢复挂起的任务在后台运行
5. 任务状态追踪 — Running/Stopped/Done，`try_wait()` 非阻塞轮询
6. 完成通知 — 后台任务完成时自动打印 `[N] Done/Exit cmd`

**平台级 suspend/resume**：
- Windows: `CreateToolhelp32Snapshot` → 枚举线程 → `SuspendThread`/`ResumeThread`
- Unix: `kill(pid, SIGSTOP)` / `kill(pid, SIGCONT)`

**Ctrl+Z 前台挂起** — 待前端集成：
- 需修改 REPL wait-loop 使用 `waitpid(WUNTRACED)` 替代 `status()`
- `job.rs` 中 suspend/resume 基础设施已就绪
- 标记为后续前端增强任务

## 验证方案

### Phase 1：流式管道
```bash
# 类型 B/C：独立命令，实时输出
cargo build -p auto          # 应实时看到编译输出

# 类型 D：管道中的外部命令
echo hello | grep hello       # 应正确传递输出
ls | sort                     # 内置→外部管道
```

### Phase 2：交互式命令
```bash
vim                          # 应正确接管终端，退出后恢复 shell
less README.md               # 应正确接管终端
git log                      # 非交互式，应正常分式输出
```

### Phase 3：退出码追踪
```bash
1 + 2                        # $? → 0
nonexistent_command_xyz      # $? → 1
echo "exit: $?"              # 应显示上次退出码
```

### Phase 4：Ctrl+C 保护
```bash
# 在外部命令执行期间按 Ctrl+C
ping localhost               # Ctrl+C 应停止 ping 但不退出 ASH
echo "still alive"           # ASH 应继续运行
```

### Phase 5：流式迭代器
```bash
# grep 流式处理（不缓冲全部输入）
cat large_file.log | grep error   # 应逐行处理
```

### Phase 6：任务控制
```bash
sleep 10 &                   # 应打印 [1] Running in background
jobs                          # 应列出 [1] Running sleep 10
fg                            # 应拉到前台等待
sleep 20 &                    # 后台运行
jobs                          # 应显示任务列表
```

## 文件清单

| 文件 | Phase | 说明 |
|---|---|---|
| `ash-core/src/pipeline/external_stream.rs` | 1 | ExternalStream 流式抽象 |
| `ash-core/src/pipeline/atom_pipeline.rs` | 1,5 | AtomPipeline 统一管道 + into_lines() |
| `ash-core/src/cmd/external.rs` | 1,4,6 | spawn 流式/后台 + SIGINT 恢复 |
| `ash-core/src/cmd/interactive.rs` | 2 | 交互式命令检测 |
| `auto-shell/src/shell.rs` | 3,6 | 退出码追踪 + 任务控制 |
| `auto-shell/src/signal.rs` | 4 | Ctrl+C 信号保护 |
| `auto-shell/src/job.rs` | 6 | JobManager 任务管理 |
| `auto-shell/src/frontend/repl.rs` | 2,4 | REPL 交互式命令 + Ctrl+C 初始化 |
| `auto-shell/src/cmd/commands/grep.rs` | 5 | grep 流式处理 |

## 关键参考文件（nushell）

| 概念 | nushell 文件 | ASH 对应 |
|---|---|---|
| 统一管道抽象 | `nu-protocol/src/pipeline/byte_stream.rs` | `AtomPipeline` |
| 子进程封装 | `nu-protocol/src/process/child.rs` | `ExternalStream` |
| 外部命令执行 | `nu-command/src/system/run_external.rs` | `external.rs` |
| 前台进程控制 | `nu-system/src/foreground.rs` | `job.rs` (简化版) |
| 输出目标 | `nu-protocol/src/pipeline/out_dest.rs` | `external.rs` 内分支 |

## 后续扩展

1. **Ctrl+Z 前台挂起** — 修改 REPL wait-loop，`waitpid(WUNTRACED)` 检测 stopped 状态
2. **完整 Type F** — `ByteStream → Atom` 边读边解析（如 `from json` 逐行流式解析）
3. **进程组控制** — Unix `tcsetpgrp` 确保前台进程组正确设置
4. **管道信号传播** — 管道中某命令退出时 SIGPIPE 传播给上游
