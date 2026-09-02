# Auto Language 设计文档总览与索引

> 本文档是 `docs/design/` 的总入口，索引树内**全部**文档并标注分类与状态。
> 整理历史：2026-06-15 首次体系化（14 章版）→ 2026-08-28 Plan 467 全量重整 →
> 2026-08-28 Plan 468 需求级文档去序号化、按模块归位。

---

## 三层文档资产（先读这个）

| 层 | 目录 | 职责 | 回答的问题 |
|---|---|---|---|
| **设计层** | `docs/design/`（本目录） | 意图、方案、取舍、历史素材 | "为什么这样设计 / 打算怎么做" |
| **账本层** | `docs/specs/` | 现状知识（project/module 树 + 6 段 spec ledger） | "现在是什么样、关键入口在哪" |
| **过程层** | `docs/plans/`（active + `archive/` 438 篇） | 一次开发任务的完整时间线叙事 | "某次改动是怎么做的" |

三层互相**引用**不复制：设计文档描述意图并链接 specs 现状；specs 描述现状并溯源 `(plan-NNN)`；
plans 记录过程并在收尾时经 `/auto-plan:merge` 把知识沉淀回 specs（范式详见
[autoplan-spec-ledger.md](autoplan-spec-ledger.md) 与 [specs/README.md](../specs/README.md)）。

---

## 新增设计文档的归位规则（Plan 468 确立）

写新设计文档前先定性：

| 定性 | 判据 | 去向 |
|---|---|---|
| **域级章** | 覆盖一个大的技术域、长期伞形视图（如类型系统、UI 架构） | 根级拿号（下一可用章号 **27**），在本索引登记 |
| **需求级/专题设计** | 服务于某条需求线或专题（特性、轨道、规范、研究输入） | 进对应模块子目录（`autoui/`、`blocks/`、`strategy/`…），**slug 命名不带号** |
| **流程体系类** | 开发范式、知识体系、流程设计 | 根级不拿号（与 `plan-spec-hybrid-model.md`、`autoplan-spec-ledger.md` 同列） |

**封存号说明**：17–19、21–26 已被历史文档用过并随 Plan 468 归位（对应关系见各子目录），
**不再复用**——历史文献中的"Design NN"靠各文档头部的归位注记溯源。
注：`autoui/desktop-shell.md`（Design 25，曾用名 AutoShell/25-autoshell-dsl-unified-shell）已归位转正式
`autoui/`，届时 25 号一并封存。

---

## 分类总览（115 篇）

| 分区 | 数量 | 内容 |
|---|---|---|
| 一、语言核心 | 10（01–10） | 从源码到执行的语言本体设计 |
| 二、应用框架与生态 | 5（11–15） | Shell / 并发 / 网络 / 开发工具 / AI 基础设施 |
| 三、AutoUI 与 App 生成域 | 2 章 + 2 子目录 | 域级章 16/20；需求级设计归 `autoui/`（10）与 `blocks/`（4） |
| 四、流程与体系（根级，无编号） | 2 | AutoPlan 账本（现行）、Plan+Spec v1（已取代） |
| 五、战略路线图 `strategy/` | 4 | Rust 脚本层 / 消费者 parity / Python parity / Rust 库复刻 |
| 六、专题诊断（根级） | 4 | 方言体系诊断、VM 调试、ASH 设计总览（外仓主题）、[管道算子 `|>`](pipe-operator.md)（讨论稿，Plan 514 衍生） |
| 七、附录 `forge/` | 5 | AutoForge（已迁 auto-os 生态） |
| 八、历史素材 `raw/` | 67 | 早期原始设计草稿（只读参考） |
| 0 | —（原在途 `25-autoshell-dsl-unified-shell.md` 已归位为 `autoui/desktop-shell.md`） |

对账公式：1(本索引) + 18(00–16、20 章号) + 1(25 在途) + 10(autoui 含 README) + 4(blocks)
+ 2(流程) + 4(strategy) + 3(诊断) + 5(forge) + 67(raw) = **114**。

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

| # | 文档 | 主题 |
|---|------|------|
| 01 | [Architecture](01-architecture.md) | 编译器管线、核心组件、AIE 增量编译 |
| 02 | [Type System](02-type-system.md) | 类型修饰符、推断、泛型、字符串、枚举、联合 |
| 03 | [Error Handling](03-error-handling.md) | Option/Result/Panic、May\<T\>、后缀操作符 |
| 04 | [Memory & Ownership](04-memory-ownership.md) | view/mut/move 三元组、hold、存储修饰符 |
| 05 | [VM & Runtime](05-vm-runtime.md) | ABC ISA、ART、AutoVM、MicroVM、并发模型 |
| 06 | [Code Generation](06-code-generation.md) | a2c、a2r、a2ark、a2jet、AutoGen、ASTL、FFI |
| 07 | [Data Structures](07-data-structures.md) | Node、Atom、Obj、ListData、存储式列表 |
| 08 | [UI Systems](08-ui-systems.md) | AURA、场景编程、设计令牌、前后端通信 |
| 09 | [Compiler](09-compiler.md) | AIE、AutoCache、DCE、comptime、CLI、AI 原生 |
| 10 | [Language Syntax](10-language-syntax.md) | 点号表示法、函数、位运算、OOP、模块系统 |

## 二、应用框架与生态（11–15）

| # | 文档 | 主题 | 备注 |
|---|------|------|------|
| 11 | [Shell Tools](11-shell-tools.md) | AutoShell/ASH、coreutils、SmartCmd、管道 | 主体已迁独立仓 `../auto-shell` |
| 12 | [Concurrency](12-concurrency.md) | Task/Msg Actor 模型、async/await、.go 运算符 | |
| 13 | [Networking](13-networking.md) | HTTP Server 标准库、async I/O、JSON/url | |
| 14 | [Developer Tools](14-developer-tools.md) | LSP、Web Playground、AutoLab、MCP Server | |
| 15 | [AI Daemon Infrastructure](15-ai-daemon-infrastructure.md) | AutoOS 共享 LLM Harness、并发仲裁、Key Vault | 跨仓主题，实现在 auto-os 生态 |

## 三、AutoUI 与 App 生成域

**域级章（仅此两篇拿号）**：

| # | 文档 | 主题 |
|---|------|------|
| 16 | [App Generation & AI Authoring](16-app-generation-and-ai-authoring.md) | Rung 1–5 分层战略、AI 生成工作流（域战略伞） |
| 20 | [AutoUI Separation Architecture](20-autoui-separation-architecture.md) | AutoUI 分离架构：a2ui/ui_gen/host 分层（域架构锚点） |

**需求级设计子目录**（各篇头部有归位注记，保留原 Design NN 对应）：

- [`autoui/`](autoui/README.md)（8 篇 + 索引）：shared-store(原18)、theming(19)、
  examples-app-track(21)、base-styles(22)、virtual-desktop(23)、desktop-shell-and-launcher(24)、
  a2ui-composer-analysis(原25)、025-gap-enumeration(16a)
- [`blocks/`](blocks/blocks-first-class.md)（4 篇）：blocks-first-class(原17) 为 Blocks 层主设计，
  配套 agent-generation-workflow / block-package-format / datasource-convention

**在途**：`25-autoshell-dsl-unified-shell.md`——桌面 Shell 的 AutoUI 统一层（细化 Design 23/24，
服务 463/464/465 shell-track），稳定后归位 `autoui/`。

## 四、流程与体系（根级，无编号）

| 文档 | 主题 | 状态 |
|------|------|------|
| [autoplan-spec-ledger](autoplan-spec-ledger.md)（原 26） | auto-plan 四技能范式 + 6 段账本 + 路径映射 | ✅ **现行**（开发范式权威描述） |
| [plan-spec-hybrid-model](plan-spec-hybrid-model.md) | Plan+Spec v1 五环流程（2026-07-23） | ⚠️ 已被取代（诊断与反模式清单仍有效） |

## 五、战略路线图（`strategy/`）

| 文档 | 主题 | 状态 |
|------|------|------|
| [auto-as-rust-script-strategy](strategy/auto-as-rust-script-strategy.md) | "Auto 作 Rust 脚本层"宣传与文档纲领 | ✅ Accepted（Plan 359） |
| [rust-library-replication-roadmap](strategy/rust-library-replication-roadmap.md) | Rust 库复刻验证路线（parity 语料） | ✅ 现行（Plan 347/348/369） |
| [python-parity-roadmap](strategy/python-parity-roadmap.md) | Python parity 第三维度（use.py + a2py） | ✅ 现行（Plan 369 首批落地） |
| [consumer-parity-strategy](strategy/consumer-parity-strategy.md) | Auto 作为库消费者（consumer-mode） | 📝 Draft |

## 六、专题诊断（根级）

| 文档 | 主题 |
|------|------|
| [dialect-extension-diagnosis](dialect-extension-diagnosis.md) | 方言/语法扩展体系结构性问题诊断与改进方案 |
| [vm-debugging](vm-debugging.md) | VM 调试方法论（ABC 汇编格式起步） |
| [ash-design-summary](ash-design-summary.md) | AutoShell(ASH) 设计总览（外仓主题，文首有新旧计划号对照） |

## 七、附录 `forge/`（AutoForge，工具本体已迁 auto-os 生态仓）

[spec-categories](forge/spec-categories.md) · [spec-driven-forge](forge/spec-driven-forge.md) ·
[spec-ui-and-relations](forge/spec-ui-and-relations.md) ·
[forge-specs-relay-frontend](forge/forge-specs-relay-frontend.md) ·
[agents-relay-orchestration](forge/agents-relay-orchestration.md)

> forge/ 的 Spec 分类思想（Goals/Architectures/Designs/Plans/Tests/Reviews）被
> [autoplan-spec-ledger](autoplan-spec-ledger.md) 的 6 段账本继承。

## 八、历史素材（`raw/`，67 篇）

早期未经整理的原始设计草稿，是 01–15 章的素材来源，**只读保留**（个别被外仓引用，
如 auto-down 仓引用 `raw/auto-down.md` 的方言三逃逸符）。按文件名主题检索即可。

---

## 设计演进时间线（要点）

```
2024 Q1–Q4  语言核心：AST/VM/类型系统 → a2c/a2r → 泛型/枚举/模式匹配 → 内存模型
2025 Q1–Q4  UI 系统(AURA/a2ark/a2jet) → Shell → 并发(Task/Actor) → AIE/AutoCache
2026 Q1–Q2  ASH 分层架构 → LSP/MCP → 设计文档 00–15 章首次体系化(2026-06-15)
2026 Q6–Q8  App 生成战略(16) → AutoUI 域需求设计族(17–25, 后归位 autoui/blocks)
            → 分离架构(20) → 虚拟桌面三部曲(23/24) → 范式收敛(Plan 467/468)
```

## 如何使用本目录

- **新开发者**：01（管线）→ 10（语法）→ 感兴趣领域的章节。
- **贡献 AutoUI/桌面**：16（战略）→ 20（架构）→ [autoui/](autoui/README.md) 各专题
  （虚拟桌面线：virtual-desktop → desktop-shell-and-launcher → desktop-shell[25]）。
- **查现状/关键代码入口**：去 [docs/specs/](../specs/overview.md)（本目录存意图，specs 存现状）。
- **查某次改动的来龙去脉**：去 [docs/plans/](../plans/archive/plans-status-audit-2026-08-20.md)（最新全量审计；2026-09-01 起由 [Plan 513](../plans/513-repo-integration-cleanup.md) 接任）。
- **新增设计文档**：按"归位规则"节先定性再落位，写完在本索引登记。
