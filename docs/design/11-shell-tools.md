# 11 - Shell Tools (AutoShell)

## Status

**Implemented:**
- AutoShell (`crates/auto-shell/`) has a working REPL with command parsing, pipeline support, and structured data output.
- Built-in commands: `ls`, `cd`, `pwd`, `echo`, `help`, `cp`, `mv`, `rm`, `mkdir`, `grep`, `wc`, `ps`, `sys`, `build`, `run`, `select`, `where`, `get`.
- Structured output for information commands: `ls` returns structured file entries; `ps` returns process info; `sys disks`/`sys cpu`/`sys mem` return structured data using the `sysinfo` crate.
- Pipeline architecture: commands communicate via `PipelineData` carrying typed `Value` objects, not raw text.
- Command registry: extensible `Command` trait with `name()`, `signature()`, and `run()` methods.
- Tab completion framework exists in `completions/`.

**Partial / Planned:**
- File operation commands (`cp`, `mv`, `rm`, `mkdir`) exist but do not yet integrate `uutils` crates. Current implementations use direct Rust stdlib calls.
- SmartCmd (natural language interface) is designed but not implemented. No `SmartCmd` trait or NLP parsing exists.
- AI-assisted command understanding is planned for a future phase.

## Design

### Architecture: Three-Layer Model

AutoShell separates concerns into three layers:

1. **Engine layer** (platform primitives): File I/O, process enumeration, disk info -- delegated to battle-tested Rust crates rather than reimplemented.
2. **Adaptation layer** (type conversion): Raw data from engine crates is converted into Auto's typed value system (`Value`, `Obj`). This is where `ls` output becomes a `List<FileEntry>` rather than a raw text stream.
3. **Intelligence layer** (AI augmentation): Natural language understanding, smart defaults, and context-aware suggestions. This layer interprets user intent and selects the appropriate command pipeline.

### Coreutils Strategy: Reuse, Don't Rewrite

The core design decision is to avoid reimplementing standard Unix commands from scratch. The rationale:

- **Edge cases**: Commands like `ls` and `cp` contain decades of edge-case handling (symlink cycles, permission masks, non-UTF-8 filenames, atomic writes on crash). Reimplementing these introduces risk.
- **Performance**: By using Rust crates directly (in-process), AutoShell avoids the `fork/exec` overhead of shelling out to external binaries. Commands run an order of magnitude faster than traditional bash.
- **Cross-platform**: Using crates like `sysinfo` and `nu-system` provides Windows/Linux/macOS support out of the box.

**Recommended source crates per command:**

| Command | Source | Output |
|---|---|---|
| `ls` | Custom (nushell-inspired logic) | `List<FileEntry>` with name, type, size, modified, permissions |
| `ps` | `sysinfo` crate | `List<ProcessEntry>` with pid, ppid, name, status, cpu/mem usage |
| `sys disks` | `sysinfo` crate | `List<DiskEntry>` with device, filesystem, mount, total, free |
| `sys cpu` | `sysinfo` crate | `CpuInfo` with usage percentages |
| `sys mem` | `sysinfo` crate | `MemInfo` with total, used, free |
| `cp` | `uutils uu_cp` (planned) | Success/failure |
| `mv` | `uutils uu_mv` (planned) | Success/failure |
| `rm` | `uutils uu_rm` (planned) | Success/failure |
| `mkdir` | `uutils uu_mkdir` (planned) | Success/failure |
| `find` | `nu-glob` crate (planned) | `List<Path>` |

### Command Classification

AutoShell commands fall into two categories based on their output:

**Structured-output commands** return typed data that can be filtered, mapped, and piped:

```auto
let files = ls("src/")
for f in files {
    if f.size > 1024 { print(f.name) }
}
```

This "object-level pipeline" approach combines PowerShell's expressiveness with Rust's performance.

**Status-only commands** return success or failure. These are straightforward wrappers around `uutils` crates:

```
cp source.txt dest.txt    // returns: ok or error
rm -rf temp/              // returns: ok or error
```

### Internal Type System

AutoShell defines internal Rust types for structured command output:

```rust
pub struct AshFileEntry {
    pub name: String,
    pub file_type: String,       // "file" | "dir" | "symlink"
    pub size: i64,
    pub modified: Option<DateTime<Utc>>,
    pub permissions: Option<String>,
    pub owner: Option<String>,
}

pub struct AshProcessEntry {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub status: String,
    pub cpu_usage: f64,
    pub mem_usage: i64,
}

pub struct AshDiskEntry {
    pub device: String,
    pub file_system: String,
    pub mount_point: String,
    pub total: i64,
    pub free: i64,
    pub removable: bool,
}
```

A conversion layer (`From` trait implementations) maps external crate types into these ASH types. This decouples the shell's public API from upstream crate internals.

### Pipeline Architecture

Commands communicate through `PipelineData`, which wraps Auto's `Value` type. This enables:

- **Structured piping**: `ls | grep ".at"` operates on typed records, not raw text.
- **Type-safe composition**: The compiler can verify pipeline compatibility.
- **Display abstraction**: The same structured data renders differently in interactive mode (table), script mode (text), or AI mode (JSON).

### SmartCmd Design

SmartCmd is a planned feature to make shell commands understand natural language input. The design envisions:

1. A `SmartCmd` trait that wraps standard commands with natural language parsing.
2. AI-assisted command interpretation: the user types intent in plain language, and the system selects and parameterizes the appropriate command.

This is deferred to a later phase, pending the natural language infrastructure.

## Open Questions

- Should `uutils` be integrated as crate dependencies or as git submodules?
- How should structured pipeline data be serialized for cross-process piping?
- What is the minimum viable set of commands before AutoShell can replace bash for daily development?
- Should SmartCmd use a local LLM, a cloud API, or rule-based NLP?

## ASH Modernization (Post-Plan 291)

> 汇总近期 ash 相关计划（281、291、295、297、301、302、303、304）的设计意图与落地状态。
> 本文聚焦 291 之后的「现代化 ash」演进，区别于上方的原始 AutoShell 设计。

### Crate 分层架构（Plan 291/295）

```
auto-shell   ← REPL + 命令实现 + reedline 前端（目前仍是主体，未瘦身为薄壳）
  ├── ash-core   ← 零终端依赖的纯逻辑层：pipeline / completions / shell 内核（已落地）
  ├── ash-tui   ← 计划中：迁出 TUI 代码（未创建）
  └── ash-gui   ← 计划中：占位空壳（未创建）
```

- **已落地**：`ash-core`（pipeline、completions 引擎、shell 状态）、`buffer_to_ansi()` 桥接 ratatui Buffer → ANSI
- **未落地**：独立 `ash-tui` / `ash-gui` crate
- **依赖**：引入 `ratatui-core` + `ratatui-widgets`（按计划刻意不引主 crate），`reedline 0.44.0` 作为行编辑器

### 子系统状态

| 子系统 | 位置 | 状态 |
|---|---|---|
| REPL / 行编辑 | `auto-shell/src/frontend/repl.rs` | ✅ |
| 命令注册（74+ 命令） | `auto-shell/src/cmd/commands/` | ✅ |
| 管道 / 链式 | `shell.rs::execute_pipeline_with_auto` + `external.rs` | ✅ 真 OS Pipe |
| 补全引擎 | `ash-core/src/completions/` + `definitions/` | ✅ 超额 |
| 历史 / Autosuggestion | reedline `CwdAwareHinter` | ✅ |
| 语法高亮 | `frontend/term/highlight.rs` | ✅ |
| 配置 | `config.rs` + `~/.ashrc` + `~/.config/ash.toml` | ✅ |
| 环境变量 / PATH | — | ❌ 整系统未启动 |

### 计划落地状态摘要

| 计划 | 主题 | 状态 |
|------|------|------|
| 281 | 历史自动建议（Fish 式） | ✅ 已完成 |
| 291 | Warp 式全栈升级（Atom 管线、Batom 二进制、74+ 命令） | ⚠️ P0-P2 ✅，P3-P4 ⏸ |
| 295 | 分层架构 + ratatui | ⚠️ 架构偏离 |
| 297 | 外部命令参数补全系统 | ✅ 已完成（且超额） |
| 301 | 环境变量系统 | ❌ 未启动 |
| 302 | 日常可用 Shell 补全路线图 | ✅ 基本完成 |
| 303 | 脚本执行 + `>` Shell 语法 | ✅ 已完成 |
| 304 | 生产级差距分析 | ✅ 文档完成 |

### 未完成功能路线图

**P0 — 核心阻塞项：**
1. **真正的 OS Pipe**：外部命令间用 `Stdio::piped()` 流式连接（目前是伪管道）
2. **环境变量 / PATH 系统**（Plan 301 整篇未实现）
3. **错误上下文**：did-you-mean、统一 exit code、`$?` 一致性

**P1 — 脚本完整性：**
4. Here Document `<<EOF`
5. Shell 函数定义（REPL 内定义后可作命令）
6. `let x = > cmd` 赋值捕获
7. 特殊变量 `$@ $# $_` + brace expansion `{a,b,c}` + `$((1+2))` + `~user`
8. 多行续行（`\` + 未闭合引号续行）

**P2 — 数据流 / 框架：**
9. 结构化数据管道激活（Atom 接入管道）
10. 统一命令参数解析框架（Command trait 签名、`--help` 自动生成）
11. 命令品质加固：JSON `\uXXXX`、find `**`、HTTP 原生客户端

**P3 — 可定制性 / 生态：**
12-16. REPL 配置命令、`bind` 自定义键绑定、Abbreviation 系统、事件钩子、插件系统

**P4 — 架构重构：**
17-20. 创建 `ash-tui` crate、`auto-shell` 瘦身、创建 `ash-gui` 占位、统一 Shell Lexer

**P5 — 大块（已明确推迟）：**
21. AI 集成（LLMProvider、自然语言→命令、Agent）
22. Block UX（Block 模型、现代输入编辑器、ANSI Block 渲染）

### Atom 管线设计（Plan 291 P0）

ASH 引入 Atom 管线，将 Shell 管道从字符串流升级为结构化数据流：

```
ls *.at | filter { .size > 1024 } | sort { .modified } | table
```

每个管道阶段传递的是 `Atom`（结构化值），不是文本。这使得 Shell 管道具备了 PowerShell 的表达力和 Rust 的类型安全。

### 环境变量系统设计（Plan 301，未实现）

```auto
// env 命令族
env                    // 列出所有环境变量
env.path               // 显示 PATH
env.path.add("/usr/local/bin")
env.path.remove("/old/path")

// ShellVars 作用域栈
K=V command args       // K=V 内联语法
with env({ NODE_ENV: "production" }) {
    npm.run("build")   // 子进程继承修改后的环境
}

// 持久化
// ~/.config/ash/env.at
```

## Source Documents

- [raw/ash-coreutils.md](raw/ash-coreutils.md)
- [raw/ash-smartcmd-design.md](raw/ash-smartcmd-design.md)
- [ash-design-summary.md](ash-design-summary.md) — ASH 现代化演进总览
- Plans: 281, 291, 295, 297, 301, 302, 303, 304
