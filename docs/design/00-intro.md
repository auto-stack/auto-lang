# Auto Language Design Documentation

> 本文档是 Auto 语言框架设计文档的总览索引。
> 日期：2026-06-15

---

## 文档结构

Auto 语言设计文档按主题分为 **14 个核心章节** + **1 个附录（AutoForge 工具链）**：

| # | 文档 | 主题 | 状态 |
|---|------|------|------|
| 00 | [本文档](00-intro.md) | 设计文档总览与导航 | ✅ |
| 01 | [Architecture](01-architecture.md) | 编译器管线、核心组件、AIE 增量编译 | ✅ 完整 |
| 02 | [Type System](02-type-system.md) | 类型修饰符、推断、泛型、字符串、枚举、联合 | ✅ 完整 |
| 03 | [Error Handling](03-error-handling.md) | Option/Result/Panic、May<T>、后缀操作符 | ✅ 完整 |
| 04 | [Memory & Ownership](04-memory-ownership.md) | view/mut/move 三元组、hold、存储修饰符 | ✅ 完整 |
| 05 | [VM & Runtime](05-vm-runtime.md) | ABC ISA、ART、AutoVM、MicroVM、并发模型 | ✅ 完整 |
| 06 | [Code Generation](06-code-generation.md) | a2c、a2r、a2ark、a2jet、AutoGen、ASTL、FFI | ✅ 完整 |
| 07 | [Data Structures](07-data-structures.md) | Node、Atom、Obj、ListData、存储式列表 | ✅ 完整 |
| 08 | [UI Systems](08-ui-systems.md) | AURA、场景编程、设计令牌、前后端通信 | ✅ 完整 |
| 09 | [Compiler](09-compiler.md) | AIE、AutoCache、DCE、comptime、CLI、AI 原生 | ✅ 完整 |
| 10 | [Language Syntax](10-language-syntax.md) | 点号表示法、函数、位运算、OOP、模块系统 | ✅ 完整 |
| 11 | [Shell Tools](11-shell-tools.md) | AutoShell/ASH、coreutils、SmartCmd、管道 | ⚠️ 需扩展 |
| 12 | [Concurrency](12-concurrency.md) | Task/Msg Actor 模型、async/await、.go 运算符 | 🆕 新建 |
| 13 | [Networking](13-networking.md) | HTTP Server 标准库、async I/O、JSON/url 模块 | 🆕 新建 |
| 14 | [Developer Tools](14-developer-tools.md) | LSP、Web Playground、AutoLab、MCP Server | 🆕 新建 |
| 15 | [AI Daemon](15-ai-daemon-infrastructure.md) | AutoOS 共享 LLM Harness、aillmd、并发仲裁、Key Vault | 🆕 新建 |

### 附录：AutoForge 工具链设计

| 文档 | 主题 |
|------|------|
| [forge/spec-categories.md](forge/spec-categories.md) | Spec 分类体系：Goals/Architectures/Designs/Plans/Tests/Reviews |
| [forge/spec-driven-forge.md](forge/spec-driven-forge.md) | Relay 阶段：Assistant→Advisor→Architect→Planner→Coder→Tester |
| [forge/spec-ui-and-relations.md](forge/spec-ui-and-relations.md) | Specs UI：双向可追溯性、分类渲染器、关系面板 |
| [forge/forge-specs-relay-frontend.md](forge/forge-specs-relay-frontend.md) | Forge 前端：Smart Secretary 模式、Gate Cards、Relay 节点图 |
| [forge/agents-relay-orchestration.md](forge/agents-relay-orchestration.md) | 多 Agent Relay：Soul+Profession+Model、交接协议、预算控制 |

---

## 主题域划分

### 🔤 语言核心（Language Core）

涵盖 Auto 语言本身的设计，从源码到执行：

```
源码 (.at)
  │
  ├─ 词法/语法 ──→ 10-language-syntax.md
  │
  ├─ 类型系统 ──→ 02-type-system.md
  │                03-error-handling.md
  │                04-memory-ownership.md
  │
  ├─ 编译管线 ──→ 01-architecture.md
  │                09-compiler.md
  │
  ├─ 数据结构 ──→ 07-data-structures.md
  │
  └─ 执行后端 ──→ 05-vm-runtime.md（VM 解释）
                   06-code-generation.md（a2c/a2r 转译）
```

### 🖥️ 应用框架（Application Framework）

涵盖基于 Auto 语言构建的应用层框架：

```
应用层
  │
  ├─ UI 系统 ──→ 08-ui-systems.md（AURA/a2ark/a2jet/a2vue）
  │
  ├─ Shell ────→ 11-shell-tools.md（AutoShell/ASH）
  │
  ├─ 并发 ────→ 12-concurrency.md（Task/Actor/async）
  │
  ├─ 网络 ────→ 13-networking.md（HTTP/JSON/async I/O）
  │
  └─ 开发工具 ─→ 14-developer-tools.md（LSP/Playground/MCP）
```

### 🤖 AI 集成（AI Integration）

Auto 语言从设计之初就面向 AI 原生场景：

- **AI 原生编译器**：`#[pre]`/`#[post]` 契约注解、`??` 类型化空洞、`#!` 元指令 → 09-compiler.md
- **MCP Server**：AutoVM 暴露为 AI Agent 可调用的工具服务 → 14-developer-tools.md
- **SmartCmd**：自然语言→Shell 命令的 AI 翻译层 → 11-shell-tools.md
- **AutoForge**：多 Agent Relay 编排系统 → forge/ 附录

---

## 设计演进时间线

```
2024 Q1  ─  语言核心：AST、VM、类型系统基础
     Q2  ─  转译器：a2c、a2r 实现
     Q3  ─  泛型、枚举、模式匹配
     Q4  ─  内存模型：view/mut/move、borrow checker

2025 Q1  ─  UI 系统：AURA、a2ark、a2jet
     Q2  ─  Shell：AutoShell 基础命令、管道
     Q3  ─  并发：Task/Actor 模型、async/await
     Q4  ─  编译器：AIE 增量编译、AutoCache

2026 Q1  ─  Shell 现代化：ASH 分层架构、ratatui
     Q2  ─  开发工具：LSP、MCP Server
     Q3  ─  网络：HTTP Server 标准库（规划中）
     Q4  ─  自举编译器（规划中）
```

---

## 如何使用本文档

- **新开发者**：从 01-architecture.md 开始，了解整体管线；然后阅读 10-language-syntax.md 了解语法
- **贡献编译器**：阅读 01、02、09、06（按需）
- **贡献 VM**：阅读 05-vm-runtime.md
- **贡献 UI**：阅读 08-ui-systems.md
- **贡献 Shell**：阅读 11-shell-tools.md
- **了解并发模型**：阅读 12-concurrency.md
- **了解 AI 集成**：阅读 09-compiler.md（AI 原生部分）+ 14-developer-tools.md（MCP）

---

## 原始设计文档

未经整理的原始设计文档保存在 `raw/` 目录下，共 60+ 份。这些文档是上述结构化文档的素材来源，保留作为历史参考。

AutoForge 相关的 5 份设计文档已移至 `forge/` 子目录，与 Auto 语言核心设计文档分离。
