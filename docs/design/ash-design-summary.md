# AutoShell (Ash) 设计总览

> 本文汇总近期 ash 相关计划（281、291、295、297、301、302、303、304）的设计意图与落地状态，
> 并给出**未完成功能路线图**。区别于早期 [11-shell-tools.md](11-shell-tools.md)（AutoShell/SmartCmd 原始设计），
> 本文聚焦 291 之后的「现代化 ash」演进。
>
> 日期：2026-06-15 ｜ 状态核对基于 `crates/auto-shell/` 与 `crates/ash-core/` 实际代码。

---

## 1. 架构现状

### 1.1 Crate 分层（计划 291 / 295）

```
auto-shell   ← REPL + 命令实现 + reedline 前端（目前仍是主体，未瘦身为薄壳）
  ├── ash-core   ← 零终端依赖的纯逻辑层：pipeline / completions / shell 内核（已落地）
  ├── ash-tui   ← 计划中：迁出 TUI 代码（未创建）
  └── ash-gui   ← 计划中：占位空壳（未创建）
```

- **已落地**：`ash-core`（pipeline、completions 引擎、shell 状态）、`buffer_to_ansi()` 桥接 ratatui Buffer → ANSI。
- **未落地**：独立 `ash-tui` / `ash-gui` crate（见路线图 §7）。
- **依赖**：引入 `ratatui-core` + `ratatui-widgets`（按计划刻意不引主 crate），`reedline 0.44.0` 作为行编辑器。

### 1.2 子系统一览

| 子系统 | 位置 | 状态 |
|---|---|---|
| REPL / 行编辑 | `auto-shell/src/frontend/repl.rs` | ✅ |
| 命令注册（74+ 命令） | `auto-shell/src/cmd/commands/` | ✅ |
| 管道 / 链式 | `shell.rs::execute_pipeline_with_auto` + `external.rs` | ✅ 真 OS Pipe（external→external 走内核管道，仅 builtin 涉及时字符串缓冲） |
| 补全引擎 | `ash-core/src/completions/` + `definitions/` | ✅ 超额 |
| 历史 / Autosuggestion | reedline `CwdAwareHinter` | ✅ |
| 语法高亮 | `frontend/term/highlight.rs` | ✅ |
| 配置 | `config.rs` + `~/.ashrc` + `~/.config/ash.toml` | ✅ |
| 环境变量 / PATH | — | ❌ 整系统未启动 |

---

## 2. 计划逐篇摘要与落地状态

### 计划 281 — 历史自动建议（Fish 式） ✅ 已完成
复用 reedline `CwdAwareHinter`，灰色幽灵提示。Ctrl+F 接受整条、Ctrl+→ 接受一词。
> 落地后修复：hint 样式从默认 LightGray 改为 DarkGray+斜体；Ctrl+F 事件从错误的 `EditCommand::Complete` 改为 `HistoryHintComplete`。

### 计划 291 — Warp 式全栈升级 ⚠️ 部分完成（P0–P2 ✅，P3–P4 ⏸）
五个 Phase：Atom 管线（P0）→ Batom 二进制（P1）→ 命令扩展到 74+（P2）→ **AI 集成（P3 推迟）** → **Block UX（P4 推迟）**。放弃 `embed-nu`，零依赖自实现解析器。

### 计划 295 — 分层架构 + ratatui ⚠️ 部分完成（架构偏离）
`ash-core` + `buffer_to_ansi` 已落地；独立 `ash-tui` / `ash-gui` crate 未创建，`auto-shell` 未瘦身为薄壳。

### 计划 297 — 外部命令参数补全系统 ✅ 已完成（且超额）
Phase A（有状态 ShellCompleter + CompletionSignature）+ Phase B（CompletionSpec 声明式引擎 + TTL 缓存）。内置定义：git、cargo、docker、npm、ssh（计划只列 git/cargo）。

### 计划 301 — 环境变量系统 ❌ 未启动
`env` / `env.path` 命令族、ShellVars 作用域栈、`K=V` 内联语法、`~/.config/ash/env.at` 持久化、AutoLang FFI、`with env()` 块——**全部未实现**（代码核实：`vars.rs` 无任何 path_*/scope 方法、无 env 命令）。

### 计划 302 — 日常可用 Shell 补全路线图 ✅ 基本完成
重定向、`&&`/`||`、alias、glob、tilde、命令替换、高亮、Vi 模式、`source`、pushd/popd/dirs、`ash.toml` 全部落地。**仅缺**：多行续行（`\`）、统一 Shell Lexer。

### 计划 303 — 脚本执行 + `>` Shell 语法 ✅ 已完成
`ash hello.at` 直接执行脚本；脚本内 `>` 前缀行送 Shell、其余送 VM；`$var` 优先取 VM 变量。**仅缺可选项**：`let x = > cmd` 赋值捕获。

### 计划 304 — 生产级差距分析（文档）✅ 文档完成；差距项实施参差
对标 Fish/Nushell/Bash 列 20 项差距（P0–P3）。**已完成项**：`ash -c`/`-s`、Ctrl+R、`!!` 展开、process substitution `<(cmd)`、Ctrl+E 多行编辑、补全广度。**未完成项见路线图。**

---

## 3. 未完成功能路线图（按子系统归类）

> 经代码核实仍未做的项。优先级参考计划 304 §五结论。

### P0 — 核心阻塞项（建议优先）

| # | 功能 | 出处 | 代码证据 |
|---|---|---|---|
| 1 | **真正的 OS Pipe**：外部命令间用 `Stdio::piped()` 流式连接 | 304 P0-1 | `pipeline.rs:70-72` 仍 TODO，外部→外部串行字符串传递 |
| 2 | **环境变量 / PATH 系统**（计划 301 整篇） | 301 / 304 P1-9 | `vars.rs` 无 path_*/scope；无 env_cmd.rs |
| 3 | **错误上下文**：did-you-mean、统一 exit code、`$?` 一致性 | 304 P0-3 | 无模糊建议逻辑 |

### P1 — 脚本完整性

| # | 功能 | 出处 |
|---|---|---|
| 4 | Here Document `<<EOF` | 304 P1-5 |
| 5 | Shell 函数定义（REPL 内定义后可作命令） | 304 P1-7 |
| 6 | `let x = > cmd` 赋值捕获 | 303 Step 5（可选） |
| 7 | 特殊变量 `$@ $# $_` + brace expansion `{a,b,c}` + `$((1+2))` + `~user` | 304 P3-19 |
| 8 | 多行续行（`\` + 未闭合引号续行，非 Ctrl+E 路径） | 302 Step 2.3 |

### P2 — 数据流 / 框架

| # | 功能 | 出处 |
|---|---|---|
| 9 | 结构化数据管道激活（Atom 接入管道，现仍 `ShellValue::String`） | 304 P2-10；291 P0 |
| 10 | 统一命令参数解析框架（Command trait 签名、`--help` 自动生成） | 304 P1-8 |
| 11 | 命令品质加固：JSON `\uXXXX`、find `**`、HTTP 原生客户端 | 291 风险表 |

### P3 — 可定制性 / 生态

| # | 功能 | 出处 |
|---|---|---|
| 12 | REPL 内配置命令（`config set/get`、`theme`） | 304 P2-11 |
| 13 | `bind` 自定义键绑定命令 | 304 P3-16 |
| 14 | Abbreviation 系统（输入时展开，区别于 alias） | 304 P2-13 |
| 15 | 事件钩子（on_chdir / on_preexec / on_precmd） | 304 P2-14 |
| 16 | 插件系统（Fish 式函数文件 + 外部协议） | 304 P2-12 |

### P4 — 架构重构

| # | 功能 | 出处 |
|---|---|---|
| 17 | 创建独立 `ash-tui` crate（TUI 迁出 auto-shell） | 295 A4/A6 |
| 18 | `auto-shell` 瘦身为薄壳（移除冗余依赖） | 295 A5 |
| 19 | 创建 `ash-gui` 占位空壳 | 295 §三 |
| 20 | 统一 Shell Lexer（`ShellToken` 枚举） | 302 Step 4.1 |

### P5 — 大块（已明确推迟）

| # | 功能 | 出处 |
|---|---|---|
| 21 | AI 集成（LLMProvider、自然语言→命令、错误解释、Agent、`stdlib/auto/llm.at`） | 291 Phase 3 |
| 22 | Block UX（Block 模型、现代输入编辑器、ANSI Block 渲染） | 291 Phase 4 |

### P6 — 文档 / 性能

| # | 功能 | 出处 |
|---|---|---|
| 23 | 文档体系：`--help` 标准化、`man ash`、cookbook | 304 P3-20 |
| 24 | 性能基准（启动时间 / 命令延迟 / 内存） | 304 P3-18 |

---

## 4. 建议的下一步

按计划 304 路线图与代码核实，**最值得优先推进的 3 项**：

1. **OS Pipe**（P0-1）—— 外部命令管道目前是伪管道（串行字符串），影响所有 `a | b | c` 体验，是生产可用的硬阻塞。
2. **环境变量 / PATH 系统**（计划 301）—— 整系统未启动，是日常使用的高频缺失（`NODE_ENV=prod ...`、PATH 增删）。
3. **Here Document + Shell 函数**（304 P1）—— 脚本完整性阻塞项。

其余按 P1→P6 渐进。AI（P5-21）与 Block UX（P5-22）属战略级大块，建议在 P0–P2 稳定后再启动。
