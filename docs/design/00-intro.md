# Auto Language 设计文档总览与索引

> 本文档是 `docs/design/` 的总入口，索引树内**全部**文档并标注分类与状态。
> 上次全面整理：2026-06-15（14 章版）；本次重整：2026-08-28（Plan 467）。

---

## 三层文档资产（先读这个）

| 层 | 目录 | 职责 | 回答的问题 |
|---|---|---|---|
| **设计层** | `docs/design/`（本目录） | 意图、方案、取舍、历史素材 | "为什么这样设计 / 打算怎么做" |
| **账本层** | `docs/specs/` | 现状知识（project/module 树 + 6 段 spec ledger） | "现在是什么样、关键入口在哪" |
| **过程层** | `docs/plans/`（active + `archive/` 438 篇） | 一次开发任务的完整时间线叙事 | "某次改动是怎么做的" |

三层互相**引用**不复制：设计文档描述意图并链接 specs 现状；specs 描述现状并溯源 `(plan-NNN)`；
plans 记录过程并在收尾时经 `/auto-plan:merge` 把知识沉淀回 specs（范式详见
[26-autoplan-spec-ledger.md](26-autoplan-spec-ledger.md) 与 [specs/README.md](../specs/README.md)）。

---

## 分类总览（111 篇）

| 分区 | 数量 | 内容 |
|---|---|---|
| 一、语言核心 | 10（01–10） | 从源码到执行的语言本体设计 |
| 二、应用框架与生态 | 5（11–15） | Shell / 并发 / 网络 / 开发工具 / AI 基础设施 |
| 三、AutoUI 与 App 生成 | 11（16–25，含 16a） | App 生成战略、Blocks、SharedStore、主题、分离架构、示例轨道、虚拟桌面、桌面 Shell |
| 四、流程与体系 | 2 | Plan+Spec 混合模型（v1，已取代）、AutoPlan 账本（v2，现行） |
| 五、战略路线图 `strategy/` | 4 | Rust 脚本层 / 消费者 parity / Python parity / Rust 库复刻 |
| 六、专题诊断（根级） | 3 | 方言体系诊断、VM 调试、ASH 设计总览（外仓主题） |
| 七、附录 `blocks/`、`forge/` | 3 + 5 | Design 17 配套；AutoForge（已迁 auto-os 生态） |
| 八、历史素材 `raw/` | 67 | 早期未经整理的原始设计草稿（只读参考） |

---

## 一、语言核心（01–10）

```
源码 (.at)
  ├─ 词法/语法 ──→ 10-language-syntax.md
  ├─ 类型系统 ──→ 02 / 03 / 04
  ├─ 编译管线 ──→ 01 / 09
  ├─ 数据结构 ──→ 07
  └─ 执行后端 ──→ 05（VM 解释）/ 06（a2c/a2r 转译）
```

| # | 文档 | 主题 | 状态 |
|---|------|------|------|
| 01 | [Architecture](01-architecture.md) | 编译器管线、核心组件、AIE 增量编译 | ✅ 现行 |
| 02 | [Type System](02-type-system.md) | 类型修饰符、推断、泛型、字符串、枚举、联合 | ✅ 现行 |
| 03 | [Error Handling](03-error-handling.md) | Option/Result/Panic、May\<T\>、后缀操作符 | ✅ 现行 |
| 04 | [Memory & Ownership](04-memory-ownership.md) | view/mut/move 三元组、hold、存储修饰符 | ✅ 现行 |
| 05 | [VM & Runtime](05-vm-runtime.md) | ABC ISA、ART、AutoVM、MicroVM、并发模型 | ✅ 现行 |
| 06 | [Code Generation](06-code-generation.md) | a2c、a2r、a2ark、a2jet、AutoGen、ASTL、FFI | ✅ 现行 |
| 07 | [Data Structures](07-data-structures.md) | Node、Atom、Obj、ListData、存储式列表 | ✅ 现行 |
| 08 | [UI Systems](08-ui-systems.md) | AURA、场景编程、设计令牌、前后端通信 | ✅ 现行（AutoUI 时代见三） |
| 09 | [Compiler](09-compiler.md) | AIE、AutoCache、DCE、comptime、CLI、AI 原生 | ✅ 现行 |
| 10 | [Language Syntax](10-language-syntax.md) | 点号表示法、函数、位运算、OOP、模块系统 | ✅ 现行 |

## 二、应用框架与生态（11–15）

| # | 文档 | 主题 | 状态 |
|---|------|------|------|
| 11 | [Shell Tools](11-shell-tools.md) | AutoShell/ASH、coreutils、SmartCmd、管道 | ⚠️ 主体已迁独立仓 `../auto-shell`（见 [ash-design-summary.md](ash-design-summary.md) 迁移注记） |
| 12 | [Concurrency](12-concurrency.md) | Task/Msg Actor 模型、async/await、.go 运算符 | ✅ 现行 |
| 13 | [Networking](13-networking.md) | HTTP Server 标准库、async I/O、JSON/url | ✅ 现行 |
| 14 | [Developer Tools](14-developer-tools.md) | LSP、Web Playground、AutoLab、MCP Server | ✅ 现行 |
| 15 | [AI Daemon Infrastructure](15-ai-daemon-infrastructure.md) | AutoOS 共享 LLM Harness、aillmd、并发仲裁、Key Vault | ✅ 现行（跨仓主题，实现在 auto-os 生态） |

## 三、AutoUI 与 App 生成（16–25）

2026-06 起的主开发线（App 生成战略 → AutoUI 双端渲染 → 虚拟桌面 → 桌面 Shell），
对应 plans 437–466 密集迭代。Design 20/23/24 构成桌面架构三部曲。

| # | 文档 | 主题 | 状态 |
|---|------|------|------|
| 16 | [App Generation & AI Authoring](16-app-generation-and-ai-authoring.md) | Rung 1–5 分层战略、AI 生成工作流 | ✅ 现行（战略纲领） |
| 16a | [025 Gap Enumeration](16-appendix-025-gap-enumeration.md) | 025 示例探针差距枚举 | 📜 历史记录（025 已删，留档） |
| 17 | [Blocks as a First-Class Tier](17-blocks-first-class.md) | Skill 级 UI 区块模型（配套 `blocks/` 附录） | ✅ 现行 |
| 18 | [Shared Store](18-shared-store.md) | Rung 4 跨 widget/跨路由状态 | ✅ 现行 |
| 19 | [Theming & Dark Mode](19-theming-and-dark-mode.md) | AutoUI 统一深浅色与主题色 | ✅ 现行（Plan 458 落地线） |
| 20 | [AutoUI Separation Architecture](20-autoui-separation-architecture.md) | 分离架构（a2ui/ui_gen/host 分层） | ✅ 现行（2026-08-26 按 Design 23 修订 §6.1） |
| 21 | [Examples App Track](21-examples-app-track.md) | `examples/ui/` 应用轨道与 AutoOS 默认应用矩阵 | ✅ 现行 |
| 22 | [Base Styles & Visual Parity](22-base-styles-and-visual-parity.md) | 跨后端视觉一致性规范 | ✅ 现行 |
| 23 | [AutoUI Virtual Desktop](23-autoui-virtual-desktop.md) | 虚拟桌面架构（修订 20、翻转 Plan 365 裁定） | ✅ 现行（Plan 452/462/465 落地线） |
| 24 | [Desktop Shell & Launcher](24-autoui-desktop-shell-and-launcher.md) | 虚拟桌面 M2–M4：桌面 Shell 与 Launcher | ✅ 现行（Plan 463/464 落地线） |
| 25 | [A2UI Composer Analysis](25-a2ui-composer-analysis.md) | Google A2UI 技术分析与 AutoUI 实现映射 | ✅ 现行（研究输入，Design 16 同族） |

## 四、流程与体系（2）

| 文档 | 主题 | 状态 |
|------|------|------|
| [Plan + Spec 混合开发模型 v1](plan-spec-hybrid-model.md) | 五环流程 + specs v1 树设计（2026-07-23） | ⚠️ **已被取代**（spec-sync 手工回写未跑通；诊断与反模式清单仍有效） |
| [26 — AutoPlan Spec Ledger](26-autoplan-spec-ledger.md) | v2：auto-plan 四技能范式 + 6 段账本 + 路径映射 | ✅ **现行**（本仓开发范式权威描述） |

## 五、战略路线图（`strategy/`）

| 文档 | 主题 | 状态 |
|------|------|------|
| [auto-as-rust-script-strategy](strategy/auto-as-rust-script-strategy.md) | "Auto 作 Rust 脚本层"宣传与文档纲领 | ✅ Accepted（实施计划 Plan 359） |
| [rust-library-replication-roadmap](strategy/rust-library-replication-roadmap.md) | Rust 库复刻验证路线（parity/libs 语料） | ✅ 现行（Plan 347/348/369 持续推进） |
| [python-parity-roadmap](strategy/python-parity-roadmap.md) | Python parity 第三维度（use.py + a2py） | ✅ 现行（Plan 369 已落地首批） |
| [consumer-parity-strategy](strategy/consumer-parity-strategy.md) | Auto 作为库消费者（consumer-mode） | 📝 Draft（待实施） |

## 六、专题诊断（根级）

| 文档 | 主题 | 状态 |
|------|------|------|
| [dialect-extension-diagnosis](dialect-extension-diagnosis.md) | 方言/语法扩展体系结构性问题诊断与改进方案 | ✅ 现行（specs frontend/ui 多处引用） |
| [vm-debugging](vm-debugging.md) | VM 调试方法论（ABC 汇编格式起步） | ✅ 现行（design/14 与 specs vm/runtime 引用） |
| [ash-design-summary](ash-design-summary.md) | AutoShell(ASH) 设计总览 | 🔗 外仓主题——计划已迁 `../auto-shell/plans/`（文首有新旧号对照表） |

## 七、附录

**`blocks/`（Design 17 配套，3 篇）**
[agent-generation-workflow](blocks/agent-generation-workflow.md) ·
[block-package-format](blocks/block-package-format.md) ·
[datasource-convention](blocks/datasource-convention.md)

**`forge/`（AutoForge 附录，5 篇——工具本体已迁 auto-os 生态仓，此处留档）**
[spec-categories](forge/spec-categories.md) ·
[spec-driven-forge](forge/spec-driven-forge.md) ·
[spec-ui-and-relations](forge/spec-ui-and-relations.md) ·
[forge-specs-relay-frontend](forge/forge-specs-relay-frontend.md) ·
[agents-relay-orchestration](forge/agents-relay-orchestration.md)

> forge/ 的 Spec 分类思想（Goals/Architectures/Designs/Plans/Tests/Reviews）被
> [26-autoplan-spec-ledger.md](26-autoplan-spec-ledger.md) 的 6 段账本继承。

## 八、历史素材（`raw/`，67 篇）

早期未经整理的原始设计草稿，是 01–15 章的素材来源，**只读保留**（个别被外仓引用，
如 auto-down 仓引用 `raw/auto-down.md` 的方言三逃逸符）。不逐一索引，按文件名主题检索即可。

---

## 设计演进时间线（要点）

```
2024 Q1–Q4  语言核心：AST/VM/类型系统 → a2c/a2r → 泛型/枚举/模式匹配 → 内存模型
2025 Q1–Q4  UI 系统(AURA/a2ark/a2jet) → Shell → 并发(Task/Actor) → AIE/AutoCache
2026 Q1–Q2  ASH 分层架构 → LSP/MCP → 设计文档 00–15 章首次体系化(2026-06-15)
2026 Q6–Q8  App 生成战略(16) → Blocks/Store/主题(17–19) → 分离架构(20)
            → 示例轨道(21)/视觉规范(22) → 虚拟桌面三部曲(23/24) → 范式收敛(26, Plan 467)
```

## 如何使用本目录

- **新开发者**：01（管线）→ 10（语法）→ 感兴趣领域的章节。
- **贡献 AutoUI/桌面**：08 → 20 → 22 → 23 → 24，配合 [21 示例轨道](21-examples-app-track.md)。
- **查现状/关键代码入口**：去 [docs/specs/](../specs/overview.md)（本目录存意图，specs 存现状）。
- **查某次改动的来龙去脉**：去 [docs/plans/](../plans/plans-status-audit-2026-08-20.md)。
- **新增设计文档**：续用编号系列（下一号 27），或按主题入 `strategy/` 等子目录；
  写完在本索引登记一行。
