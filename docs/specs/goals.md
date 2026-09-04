# 全局目标账本（Goals Ledger）

> **用途**：`/auto-plan:review` 填 `touched_goals` 时引用这里的 `GOAL-NNN`；
> `/auto-plan:merge` 时更新对应条目状态。新目标先入 roadmap/战略设计，再在此登记。
> 来源：[docs/roadmap.md](../roadmap.md)（v0.5）、[strategy/](../design/strategy/) 战略文档、
> Design 16/17/21/23/24。更新：2026-08-28（Plan 467 首建）。

| ID | 目标 | 状态 | 关联 |
|---|---|---|---|
| GOAL-001 | 语言核心成熟：生命周期检查强化、AutoFree、逃逸分析 + ARC fallback | 规划中 | roadmap 生命周期节 |
| GOAL-002 | 解释器/VM 快周转：1s 周转、热重载（含数据迁移）、实时反汇编 | 部分达成 | [vm-debugging](../design/vm-debugging.md)、plans 199/330 |
| GOAL-003 | **Auto 作 Rust 脚本层**：脚本开发→a2r 转译发布，三方行为一致（VM/a2r/原生 Rust） | 进行中 | [strategy/auto-as-rust-script-strategy](../design/strategy/auto-as-rust-script-strategy.md)、plan-359、442（musk 后端 .at VM 直跑合龙——axum/tokio 桥+serve/HTTP/SSE 全链，hw/ag/VM 三轨 parity 对照）、533（a2r 编译轨悬浮层通道——View::Popover 发射+on-only 枚举注入,overlay 家族三方一致性） |
| GOAL-004 | Rust 库复刻 parity：常见 Rust 库三后端行为对拍（30+ 库语料） | 进行中 | [strategy/rust-library-replication-roadmap](../design/strategy/rust-library-replication-roadmap.md)、plans 347/348、[parity](parity/project.md) |
| GOAL-005 | Python parity 第三维度：use.py 调 Python 库 + a2py 反向转译 | 部分达成 | [strategy/python-parity-roadmap](../design/strategy/python-parity-roadmap.md)、plan-369 |
| GOAL-006 | Consumer-mode parity：Auto 作为库消费者（消费而非复刻三方库） | 规划中 | [strategy/consumer-parity-strategy](../design/strategy/consumer-parity-strategy.md)（Draft） |
| GOAL-007 | AutoUI 跨端视觉一致：Vue 与 VM/iced 双端 base styles 与 parity 锁定 | 进行中 | Design [22](../design/autoui/base-styles-and-visual-parity.md)、plans 455/458/411/437（chart 组件双端同源，ADR-19）、498（chart 交互态 emphasis/legend 显隐双端同源）、504（静态分发函数双端同语义）、503（桌面视觉 stella 风格双端同源）、499（chart v2 指针交互——mousemove 流/axisPointer/扇区命中双端同源）、502（diagram 家族开篇——flow-diagram svg text 直通标签/Sugiyama-lite 布局/hover 交互双端同源）、512（w-[28rem] rem 任意值 iced 端不支持实证→Tailwind 刻度双端兼容口径）、515（vue 桌面壁纸层+scrim bg-background 10%/35% 双轨 token 对齐）、518（桌面视觉二期 stella 对齐——双主题 token 全组+backdrop 毛玻璃词汇三臂声明冻结）、448（语法六件双端同语义——裸 value 两向绑定/style 数组/grid 动态 cols/computed 块体四条均 Vue 与 VM/iced 同步落地） 、522（use 导入 helper fn 双端同源——Vue 侧按需转译进 SFC 对齐 VM import_aliases，437 §0.6.E-3 缺口关闭）、442（VM 渲染五能力双端——store facade/ext adapter 链/svg 节点/sched 原语/只读高亮，musk 前端 53 文件 VM link 全清）、527（VM 轨 Tailwind v3.4 清单驱动全量覆盖契约——清单锚定+静默丢弃关闭+三家族补全+变体/responsive/dark 管道,对拍审计台常驻）、528（widgets-gallery 检查跟踪——vue 展示层双修(事件单点/HTML 转义)+npm_deps 生效闭环+SCAFFOLD_DEPS 闭包+VM popover 默认 chrome/间隙/居中/防撞+toggle_group vue 全链+gallery 深浅主题）、530（VM 移动断点单份绘制修复+721GB 崩溃根除——Column 叠层 absolute-only 语义/Box::leak 家族去重/gutter ∞ 视口防御,toggle_group+alert-dialog 组件补缺）、537（029-photo-gallery——image fit cover/contain 双端实证+VM lucide 闭集/语义 grid cols 动态绑定二缺口登记 P537-D1/D2 三臂绕开双端对齐） 、533（VM 悬浮层运行时通道——overlay 三族（alert_dialog/dialog/dropdown_menu）双端+编译轨三轨同源自管开合/dismiss 语义锁定,MCP 合成键盘不经 overlay 等口径债 P533-D1..D8） |
| GOAL-008 | App 生成与 AI Authoring 战略：Rung 1–5（声明式 UI → blocks → AI 生成完整应用） | 进行中 | Design [16](../design/16-app-generation-and-ai-authoring.md)、[25](../design/autoui/a2ui-composer-analysis.md)、448（声明式 UI 表达力六件——msg 去名/事件内联 lambda/输入框两向绑定免三件套/style 组合/grid 动态列数/computed 块体） |
| GOAL-009 | 虚拟桌面与桌面 Shell：跨平台虚拟桌面（Web+桌面双端）、WM、自动排布、Launcher | 进行中（AutoShell 地基 ✅：472 投影协议 v1+workspace+dock；M2 switcher/pager 待立项；504 示例桌面化三件套——fit 窗口/设置上移 os-config；505 债务批一期——事件泵批化/shell 面五瑕疵/实机验收通道；509 Linux 合成宿主线启动 ✅——路线 B 裁定 Smithay+桌面协议宿主，host-smithay 骨架+WSLg/Xvfb 实跑+shell 首帧像素证据，I1 零分叉实证；515 债务批二期 ✅——queue 臂 scissor/typography 保真收口、HICON 真图标、真 launch e2e 通道、vue 壁纸层、工具链可见性三件） | Design [23](../design/autoui/virtual-desktop.md)/[24](../design/autoui/desktop-shell-and-launcher.md)/[25](../design/autoui/autoshell.md)、plans 452/462–465/472/504/505/512（fit 动态重测——内容尺寸变化窗口双向跟随）、509（Smithay Linux 宿主 Stage 1）、515（桌面债务批二期）、518（桌面视觉二期——双主题/壁纸资产/dock·chrome 精致化/透明度分级/Appearance 分区） |
| GOAL-010 | 示例应用轨道：examples/ui 应用矩阵（AutoOS 默认应用集） | 进行中 | Design [21](../design/autoui/examples-app-track.md)、plan-401、plans 402–441、504（011-calculator 桌面移植范式样板）、506/512（批一/批二 27 例桌面化迁移+fit 动态重测实证）、518（p518-glass-sample 毛玻璃样张）、448（示例语料语法迁移——005/010/013/016/017/018/024/038/043/459/p507 三件套·style-if·手工列表削减） 、522（016-calendar computed 化迁移——store ×4 重算链删除；024 donut dc/ds helper 回正）、537（029-photo-gallery 图库填洞——image widget 首个应用级双端示范,picsum 网络图源） |
| GOAL-011 | Blocks 一等公民生态：Skill 级 UI 区块包格式、agent 生成工作流、CLI | 部分达成 | Design [17](../design/blocks/blocks-first-class.md)、plans 342/343、[blocks](blocks/project.md) |
| GOAL-012 | Web 生态转译：a2ts/a2js 完善、shadcn 外第二组件库、React/Svelte 支持、响应式布局 | 规划中 | roadmap Web 节、[widgets](widgets/project.md) |
| GOAL-013 | C 生态与嵌入式：全语法 a2c、宏/预处理、CTE-C 无缝、Linker 接管、MCU 热重载 | 部分达成 | roadmap C 节、plans 027/044 |
| GOAL-014 | 开发者工具：LSP 现代化（TS 迁移/semantic tokens/CI）、Playground、MCP、调试器 | 进行中 | Design [14](../design/14-developer-tools.md)、plans 243/416 |
| GOAL-015 | Agent 生态：CodingAgent、嵌入式综合 Agent、Harness 架构（与 auto-os 协同） | 规划中 | roadmap Agent 节、Design [15](../design/15-ai-daemon-infrastructure.md) |
| GOAL-016 | 构建与测试基础设施：sccache、cargo t ≤30s、全量门禁收敛到 review、CI 闸门 | 部分达成 | plan-466、`.github/workflows/vm-files-ci.yml` |
| GOAL-017 | 自举：用 Auto 写 Auto 编译器（aavm，auto/ 目录 .at 实现，六道闸门） | 进行中 | [aavm](aavm/project.md)、plans 429–434、523（a2r 模式中阶覆盖+同步规约基建，四路 runner/三件套金样） |
| GOAL-018 | 开发范式与知识账本运转：auto-plan 四技能 + specs v2 账本，每计划完成即沉淀 | 进行中 | Design [26](../design/autoplan-spec-ledger.md)、[README v2](README.md)、plan-467 |

> 状态取值：`规划中 / 进行中 / 部分达成 / 已达成（可关闭）`。
> 一个 GOAL 关闭时保留行并标注达成日期与收尾 plan。
