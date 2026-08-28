# 全局目标账本（Goals Ledger）

> **用途**：`/auto-plan:review` 填 `touched_goals` 时引用这里的 `GOAL-NNN`；
> `/auto-plan:merge` 时更新对应条目状态。新目标先入 roadmap/战略设计，再在此登记。
> 来源：[docs/roadmap.md](../roadmap.md)（v0.5）、[strategy/](../design/strategy/) 战略文档、
> Design 16/17/21/23/24。更新：2026-08-28（Plan 467 首建）。

| ID | 目标 | 状态 | 关联 |
|---|---|---|---|
| GOAL-001 | 语言核心成熟：生命周期检查强化、AutoFree、逃逸分析 + ARC fallback | 规划中 | roadmap 生命周期节 |
| GOAL-002 | 解释器/VM 快周转：1s 周转、热重载（含数据迁移）、实时反汇编 | 部分达成 | [vm-debugging](../design/vm-debugging.md)、plans 199/330 |
| GOAL-003 | **Auto 作 Rust 脚本层**：脚本开发→a2r 转译发布，三方行为一致（VM/a2r/原生 Rust） | 进行中 | [strategy/auto-as-rust-script-strategy](../design/strategy/auto-as-rust-script-strategy.md)、plan-359 |
| GOAL-004 | Rust 库复刻 parity：常见 Rust 库三后端行为对拍（30+ 库语料） | 进行中 | [strategy/rust-library-replication-roadmap](../design/strategy/rust-library-replication-roadmap.md)、plans 347/348、[parity](parity/project.md) |
| GOAL-005 | Python parity 第三维度：use.py 调 Python 库 + a2py 反向转译 | 部分达成 | [strategy/python-parity-roadmap](../design/strategy/python-parity-roadmap.md)、plan-369 |
| GOAL-006 | Consumer-mode parity：Auto 作为库消费者（消费而非复刻三方库） | 规划中 | [strategy/consumer-parity-strategy](../design/strategy/consumer-parity-strategy.md)（Draft） |
| GOAL-007 | AutoUI 跨端视觉一致：Vue 与 VM/iced 双端 base styles 与 parity 锁定 | 进行中 | Design [22](../design/22-base-styles-and-visual-parity.md)、plans 455/458/411 |
| GOAL-008 | App 生成与 AI Authoring 战略：Rung 1–5（声明式 UI → blocks → AI 生成完整应用） | 进行中 | Design [16](../design/16-app-generation-and-ai-authoring.md)、[25](../design/25-a2ui-composer-analysis.md) |
| GOAL-009 | 虚拟桌面与桌面 Shell：跨平台虚拟桌面（Web+桌面双端）、WM、自动排布、Launcher | 进行中 | Design [23](../design/23-autoui-virtual-desktop.md)/[24](../design/24-autoui-desktop-shell-and-launcher.md)、plans 452/462–465 |
| GOAL-010 | 示例应用轨道：examples/ui 应用矩阵（AutoOS 默认应用集） | 进行中 | Design [21](../design/21-examples-app-track.md)、plan-401、plans 402–441 |
| GOAL-011 | Blocks 一等公民生态：Skill 级 UI 区块包格式、agent 生成工作流、CLI | 部分达成 | Design [17](../design/17-blocks-first-class.md)、plans 342/343、[blocks](blocks/project.md) |
| GOAL-012 | Web 生态转译：a2ts/a2js 完善、shadcn 外第二组件库、React/Svelte 支持、响应式布局 | 规划中 | roadmap Web 节、[widgets](widgets/project.md) |
| GOAL-013 | C 生态与嵌入式：全语法 a2c、宏/预处理、CTE-C 无缝、Linker 接管、MCU 热重载 | 部分达成 | roadmap C 节、plans 027/044 |
| GOAL-014 | 开发者工具：LSP 现代化（TS 迁移/semantic tokens/CI）、Playground、MCP、调试器 | 进行中 | Design [14](../design/14-developer-tools.md)、plans 243/416 |
| GOAL-015 | Agent 生态：CodingAgent、嵌入式综合 Agent、Harness 架构（与 auto-os 协同） | 规划中 | roadmap Agent 节、Design [15](../design/15-ai-daemon-infrastructure.md) |
| GOAL-016 | 构建与测试基础设施：sccache、cargo t ≤30s、全量门禁收敛到 review、CI 闸门 | 部分达成 | plan-466、`.github/workflows/vm-files-ci.yml` |
| GOAL-017 | 自举：用 Auto 写 Auto 编译器（aavm，auto/ 目录 .at 实现，六道闸门） | 进行中 | [aavm](aavm/project.md)、plans 429–434 |

> 状态取值：`规划中 / 进行中 / 部分达成 / 已达成（可关闭）`。
> 一个 GOAL 关闭时保留行并标注达成日期与收尾 plan。
