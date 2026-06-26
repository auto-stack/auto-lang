# 16 — App Generation & AI Authoring

> **状态**:战略设计文档(Roadmap)
> **日期**:2026-06-26
> **关联**:Plan 331/336/337(@auto-ui/widgets 生态)、Plan 329(SSE/流式)、`examples/ui/*` 示例集、`../auto-musk` 迁移目标
> **目的**:把"从 demo/screen 到完整 APP"的跃迁拆成可执行的 **能力阶梯 × 基准阶梯**,指导后续多个实施 Plan 的优先级。

---

## 1. 背景与问题

AutoUI(a2vue)已能生成:**单 widget 页、block 组合、整站复刻(a2ui demo)**。`examples/ui/015-notes` 已是相对完整的例子(前后端解耦、多模块、`#[api]` REST、CRUD+搜索),`../auto-musk` 已用 Auto 表达了一个多页 Agent app(`routes{}` + nav rail + 6 页 + `use back.api`)。

**真正待解决的问题不是"Auto 能否表达 app"**(已能),而是:
> **如何让 AI 以中等乃至高复杂度、可靠地用 Auto 生成完整 app(前端渲染 vue,后端 rust)?**

## 2. 核心论点

> **瓶颈是目标语言的形态与反馈回路,不是模型智力。**

Auto 的赌注是"把复杂性吸收进语言"。对 AI 作者而言,这个赌注只在以下条件同时满足时兑现:
1. **widget 调色板的"天花板"足够高** —— AI 能组合的原语覆盖得到目标复杂度。
2. **app 级关注点是一等的、可被编译器检查的** —— 数据生命周期、后端契约、布局、共享状态,而不是让 AI 临时拼凑。
3. **写 → 编译 → 预览 → 修的回路紧凑且精确** —— 错误信息好、dev server 快、preview 即时。

因此:**每个能力台阶,都要落到 (a) 编译器/codegen 特性 + (b) widget-gallery/block 示例 + (c) `/auto-lang-creator` skill 条目 + (d) 基准 app + 评测**。缺任何一项,该能力只对人类可用,不对 AI 可用。

## 3. 能力阶梯(语言/工具要吸收什么)

| 阶 | 当前状态 | "真实 app"缺口 |
|---|---|---|
| **0. 原语 + 单页** | ✅ widget-gallery、blocks、a2ui 复刻 | — |
| **1. 多页 app shell**(路由/outlet/nav、每页 model+handler) | ✅ 可表达(auto-musk app.at) | 约定 + `auto new app --template` 脚手架(让 AI 改骨架而非从零写) |
| **2. 服务端状态 + 后端契约** | ⚠️ `use back.api` *调用*通;chats 页自称"非流式 MVP" | **类型化契约**(前端拿到 Rust 签名,编译器检查调用点)、**异步生命周期**(`on mount` 抓取、loading/error/empty 槽位)、**SSE/流式**(复用 Plan 329) |
| **3. 复杂数据 UI** | ⚠️ 有原语,无标准组合模式 | 真正的 **data-table**(排序/过滤/分页/服务端)、**带校验的表单**、乐观更新、3-4 个组合模式(master-detail、list+search、form+validation、stream-log) |
| **4. app 架构** | ⚠️ 仅 widget 级 model | **共享/全局 store**(跨页状态)、**auth/session + 路由守卫**(login → protected)、响应式布局系统 + theming/dark-mode |
| **5. AI 作者回路** | ⚠️ creator skill + vue-gallery preview | **`auto dev`** 热重载 dev server、a2vue 错误信息质量、**分阶段生成器**(spec→骨架→页→widget)、**评测套件**(基准 app × 修复轮次 N) |

**关键耦合**:第 3 阶的"调色板天花板"= Plan 337 TODO-A(扩到 ~60 widget)。**widget 库工作与 app 生成工作是同一攀登的两条腿**,不是两条独立赛道。

## 4. 基准阶梯(M1→M6,每个 app 覆盖一个独特能力簇)

基准不是"越多越好",而是每个覆盖一个**别的基准不覆盖**的能力簇,这样失败模式互不重叠,精准指出下一步该投资的语言特性。

| 里程碑 | 基准 app | 覆盖的能力簇 | 现状 |
|---|---|---|---|
| **M1** | **015-notes(扩展:+routing +tags/markdown +持久化)** | 解耦架构、`#[api]`、多模块、**app shell/路由**(Rung 1)、数据生命周期(Rung 2) | 唯一相对完整;需 +routing 与 CRUD 加深 → Plan **338** |
| **M2** | **022-kanban(重建)** | 拖拽、乐观更新、派生/客户端状态、多列布局 | 单 app.at 骨架;需整体重建 |
| **M3** | **017-chat(带流式后端)** | **SSE/流式**(复用 Plan 329)、消息历史、长列表 → **auto-musk 直系前身** | 单 app.at 骨架;需后端+流式 |
| **M4** | **016-calendar(扩展)** | 时间/日期逻辑、定时事件/闹钟、**外部 API 集成**(节假日)、定时后端任务 | 2 模块无后端;需事件模型+调度 |
| **M5** | **023-realworld(Conduit)** | auth、profile、article、comment、follow、分页 —— **完整中型社交 app** | 单 app.at 骨架;最高 CRUD+社交难度 |
| **M6** | **`../auto-musk`** | agent UI:流式 + config + explorer + specs | 已半 Auto 化;终点 |

**评测度量**:每个里程碑,"AI 从 spec 达到 *green build + 功能对等* 所需的修复轮次 N"。**N 不降的地方 = 下一步该吸收进语言的能力**。基准阶梯与能力阶梯由此闭环。

## 5. 建议节奏

1. **M1**(Plan 338)先做:把 015-notes 扩成中等 CRUD(+routing),既是像样的第一个基准,又顺手补 Rung 1。
2. **能力投资并行**:Rung 2(类型化后端契约 + 数据生命周期)与 Rung 5(`auto dev`)是第一波最高杠杆 —— 它们让"迭代 app"变得可行。
3. **逐级 M2→M6**:每级先"让 AI 从 spec 再生"做 gap 分析,失败模式驱动下一波语言投资。
4. **M6 auto-musk 为终点**:前端整体迁回 Auto,退役 AI 手写 Vue(`frontend/src`)。

## 6. 与现有工作的关系

- **Plan 337(薄同步)+ TODO-A(全量 widget)**:抬第 3 阶调色板天花板 —— 直接抬高 AI 可达复杂度。
- **Plan 329(IPC/SSE)**:M3(chat)/M6(auto-musk)流式的底座。
- **`/auto-lang-creator` skill + vue-gallery**:第 5 阶 AI 作者回路的一部分(Auto 的"文档/示例"输入)。
- **`examples/ui/016-023` 的完善愿景**:每个例子各展示 Auto 一面、且是完整可用 app —— 即本基准阶梯 M2-M5 的载体。

## 7. 非目标 / 开放问题

- **不做** Vue→Auto 反向转译(lossy,背离初衷);方向是 AI 直写 Auto、编译到 vue。
- **不做** "一键生成整个 app"的魔法;分阶段(spec→骨架→页→widget)+ 编译/预览/修复回路。
- **开放**:类型化后端契约的具体形态(derive? IDL? 还是复用 Rust `#[api]` 反射签名?)—— 留给 Rung 2 的实施 Plan 定。
- **开放**:第一个基准到底多难合适。015-notes 扩展后(M1)作为起点是保守稳妥的;若过早跳 M5(realworld)会让 N 失控、信号被噪声淹没。
