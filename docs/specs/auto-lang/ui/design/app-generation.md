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

## vue 轨发射一致性契约（PLAN-677）

双轨（vm/vue）渲染与交互语义必须同源；vue 轨缺口修复一律落在生成器/模板层（消费方禁止补件生成物，2026-09-21 裁定）。以下四条为经复审的永久契约：

1. **视图声明式 menubar 族 lowering**：`menubar-menu/trigger/content/item/separator/checkbox-item` 必须 lower 到 shadcn Menubar 组件树（项渲染=icon(lucide 子组件)+title(span)+MenubarShortcut 右对齐 span；`enabled: expr`→`:disabled="!(expr)"`取反；`checked: expr`→MenubarCheckboxItem `:checked` 单向；`onclick`→契约事件），语义与 vm 侧 convert_menubar_component 同源；registry 查找键同时接受连字符与下划线两形态。验收 AC-04（消费方 auto-edit 浏览器实测）。
2. **for 循环透明包装**：fallback v-for 包装对单 Conditional 体循环发 `<template v-for :key>`（不产盒，内容直接参与父弹性布局）；多语句体维持 div 包装（多子迭代的块级布局语义不变）；SVG 子树维持 Plan 502 先例。验收 AC-01。
3. **natives 三层发射**：R 层真实现（`file_basename` rsplit 末段/双分隔符；`console_*` 内存 buffer，cap 500、lines(n=200) 最新在前——语义镜像 vm native.rs/ui_console.rs）；B 层 editorBridge 按 DSL editor key 寻址 code_editor_* 内建（光标 0-based、折叠行号 1-based、隐藏行计数；非活动编辑器静默 no-op = vm 单活动编辑器语义）；S 层维持 fail-fast 桩（671 §10-1）。验收 AC-05/06。
4. **脚手架修复可传播**：CodeEditor 壳与 editorBridge 同步发射于新建（generate）与再生成（regenerate_source_files）两管线；使用中的旧版自有脚手架（codemirror import 签名可识别）覆写为最新模板，手写文件零碰。验收 AC-07（稳态二跑零漂移）。
5. **Env 轨别原语与字面量折叠（PLAN-680）**：`Env.get("字面量键")` 在 vue 轨折叠为 gen 机 env 快照字符串字面量（gen 产物不入库不泄机器值；非字面量键与其余 `Env.*`/`Process.*` 维持 `__vmOnly` fail-fast——671 诚实抛错裁定不变）；新增 `Env.track()` 轨别原语（native 9907 高段，沿 PLAN-656 先例）——VM 运行时返 `"vm"`，vue 轨生成期折叠为 `"vue"` 字面量，为语料提供无 Env 依赖的轨别/FS 能力守卫信号（027 先例：Tick 引导/NavTo 汇聚点/GoUp/CommitNew 四点门控 + 轨别分派空态文案；sidebar-in-row 布局须 `w-auto`，018 同款）。验收 AC-01..04。

> 证据：消费方 auto-edit 冷再生成+浏览器五点实测（docs/plans/evidence/677/）；门禁 cargo tf 3694/3694 + tt 4063/4063。

## 显式非目标

- 不做 Vue→Auto 反向转译（lossy，背离"AI 直写 Auto、编译到 Vue"的方向）。
- 不做"一键生成整个 app"的魔法；走分阶段（spec→骨架→页→widget）+ 编译/预览/修复回路。
- 开放：类型化后端契约的具体形态（derive / IDL / 复用 Rust `#[api]` 反射）留 Rung 2 实施 plan 定；基准难度起点（过早跳 M5 会让 N 失控）。

> 来源: docs/design/16-app-generation-and-ai-authoring.md
