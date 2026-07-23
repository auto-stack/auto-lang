# App 生成与 AI 作者回路

## 范围

"从 demo/screen 到完整 app"的能力建设框架：AI 以中高复杂度可靠地用 Auto 生成完整 app（前端 Vue、后端 Rust）。战略层设计，指导 UI 相关 plan 的优先级。

## 核心论点

瓶颈是**目标语言的形态与反馈回路**，不是模型智力。Auto 的赌注（把复杂性吸收进语言）兑现的三条件：

1. widget 调色板天花板足够高（AI 可组合的原语覆盖目标复杂度）；
2. app 级关注点一等且可被编译器检查（数据生命周期、后端契约、布局、共享状态）；
3. 写 → 编译 → 预览 → 修回路紧凑精确（错误信息、dev server、preview）。

**每个能力台阶必须落四件套**：(a) 编译器/codegen 特性、(b) widget-gallery/block 示例、(c) creator skill 条目、(d) 基准 app + 评测。缺任一项，该能力只对人类可用，不对 AI 可用。

## 能力阶梯（Rung 0-5）

| 阶 | 内容 | 状态摘要 |
|---|---|---|
| 0 | 原语 + 单页 | ✅ widget-gallery、blocks、a2ui 复刻 |
| 1 | 多页 app shell（路由/outlet/nav、每页 model+handler） | ✅ 可表达；缺脚手架约定 |
| 2 | 服务端状态 + 后端契约 | ⚠️ `use back.api` 调用通；缺类型化契约、异步生命周期（loading/error/empty 槽）、SSE/流式（plan-329 底座） |
| 3 | 复杂数据 UI | ⚠️ 有原语无标准组合：data-table、校验表单、乐观更新、3-4 个组合模式 |
| 4 | app 架构 | ⚠️ 仅 widget 级 model；缺共享 store（plan-351）、auth/session + 路由守卫、响应式布局 + theming |
| 5 | AI 作者回路 | ⚠️ creator skill + preview；缺 `auto dev` 热重载（plan-362）、错误信息质量、分阶段生成器、评测套件 |

关键耦合：Rung 3 天花板 = plan-337 TODO-A（扩到 ~60 widget）——widget 库与 app 生成是同一攀登的两条腿。

## 基准阶梯（M1-M6）

每个基准覆盖一个**互不重叠**的能力簇，失败模式互不污染：

| 里程碑 | 基准 app | 能力簇 |
|---|---|---|
| M1 | 015-notes 扩展（+routing +tags +持久化） | 解耦架构、`#[api]`、app shell、数据生命周期（plan-338→354/357/360） |
| M2 | 022-kanban 重建 | 拖拽、乐观更新、派生状态、多列布局 |
| M3 | 017-chat 带流式后端 | SSE/流式（plan-329）、消息历史、长列表 |
| M4 | 016-calendar 扩展 | 时间逻辑、定时事件、外部 API 集成（plan-323） |
| M5 | 023-realworld（Conduit） | auth/profile/article/comment/follow/分页，完整中型社交 app |
| M6 | auto-musk | agent UI：流式 + config + explorer + specs（终点） |

**评测度量**：AI 从 spec 达到 green build + 功能对等所需的修复轮次 N；N 不降之处 = 下一波该吸收进语言的能力。

## 显式非目标

- 不做 Vue→Auto 反向转译（lossy，背离"AI 直写 Auto、编译到 Vue"的方向）。
- 不做"一键生成整个 app"的魔法；走分阶段（spec→骨架→页→widget）+ 编译/预览/修复回路。
- 开放：类型化后端契约的具体形态（derive / IDL / 复用 Rust `#[api]` 反射）留 Rung 2 实施 plan 定；基准难度起点（过早跳 M5 会让 N 失控）。

> 来源: docs/design/16-app-generation-and-ai-authoring.md
