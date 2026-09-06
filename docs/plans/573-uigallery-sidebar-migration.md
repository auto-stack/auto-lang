---
plan_id: PLAN-573
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: uigallery-sidebar-migration
author: [kimi]
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 4
---

# [PLAN-573] uigallery-sidebar-migration

## 变更摘要

把 `examples/ui-gallery`（Plan 549 落地的示例画廊）左侧导航从手搓
`aside + button + style-if` 形态迁移到 sidebar 组件族（548 Vue 端 1:1 /
561 VM 契约子集 / 562 迁移惯用法），消除约 60 行手搓 active/hover 类串，
统一为 shadcn Sidebar 契约样式；左栏滚动顺带从原生 `overflow-y-auto`
换 AutoUI `scroll` 组件（Vue=ScrollArea，与 widgets-gallery 562 增补同款）。

用户裁定（2026-09-06）："它自己实现的那一套非常丑，我们需要统一成正常的
Sidebar 样式。"

## 目标

1. `examples/ui-gallery/src/front/app.at:122-178` 左栏迁移：
   - 分类筛选 Pills（全部/基础/组件/应用/系统）平移进 `sidebar_header`；
   - 示例列表（`for demo in .filteredDemos` 手搓 button）改为
     `scroll { sidebar_menu { sidebar_menu_item { sidebar_menu_button
     (active:, onclick:) { ... } } } }`；
   - 双行卡片形态（title + 可交互/独立徽标 + description）按 015-notes
     惯用法：menu_button 内单 col 孩子 + `h-auto` 覆盖（shadcn 默认 h-8）。
2. 左栏布局外壳（w-72/border-r/bg-card/40）保留在 aside 上（VM 端 provider
   独子直返忽略 class 的 562 已知行为，布局 chrome 不挂 provider）。
3. 行为零回归：点 demo 切换 selected_id + reload_key 重置、pills 筛选、
   搜索联动、右栏嵌入视口不受影响。
4. 左栏滚动走 `scroll` 组件（ScrollArea 主题化细滚动条）。

## 架构方案

- 迁移映射（562 惯用法直接复用）：
  - 手搓 button + style-if active → `sidebar_menu_button`
    `active: .selected_id == demo.id`（Vue :is-active / VM 契约三态）
  - onclick lambda（`.selected_id = demo.id; .reload_key = 0`）原样保留
    （menu_button 支持 onclick，448 内联 lambda 双端同语义）
  - 双行内容 → menu_button 内 `col (style: "h-auto ...")` 孩子
    （title 行 row + 徽标 text，描述行 text）
  - pills → `sidebar_header` 内原样保留（是筛选器不是导航项，不强转）
  - aside overflow-y-auto 摘除 → 列表区包 `scroll (style: "flex-1 min-h-0")`
- 不动：顶部 header（含搜索框）、右栏嵌入视口、model/msg/computed 全部。
- 预期纯示例资产改动（Category A）：不改 crates/ 代码；若实跑暴露
  sidebar 族缺口（如 for 循环内 active 表达式求值），再升级处置并记录。

## 技术栈

Auto 语言示例源码（.at）、sidebar 组件族（548/561）、AutoUI `scroll`
组件（Plan 105 ScrollArea / iced Scrollable）、autoui-verifier 探针
（scratch/p562/dual_probe.py + scroll_probe.mjs 复用）。

## 需求分析与背景调查

- specs 现状（docs/specs/auto-lang/ui/overview.md）：sidebar 族 548（Vue 端
  1:1）+ 561（VM 契约子集）+ 562（nav 族迁移退役，superseded_by 机制）
  三段在案；ui-gallery 为 549 落地（GOAL-010 示例轨道）。
- 现状勘察（2026-09-06 读源码实证）：ui-gallery 左栏**未用**已退役 nav 族，
  是手搓 aside+button（用户口中"Agent 又套一层"的典型）；pac.at
  `render: "vue"`（Vue 主端），port 3049，theme dark + accent indigo。
- 先例：015-notes 双行 desc 条目、widgets-gallery 外壳 67 项机械迁移、
  os-config Plain 模式处置——惯用法全部现成，无新技术风险。
- GOAL-010（示例应用轨道）、GOAL-007（双端一致）关联。

## 详细设计

迁移前后对照（app.at 左栏）：

| 现状（手搓） | 迁移目标 |
|---|---|
| `aside (style: "w-72 ... overflow-y-auto p-4 gap-4")` | `aside (style: "w-72 border-r border-border bg-card/40 flex-col shrink-0")`（摘 overflow/p4，padding 进 scroll 层） |
| Pills row（border-b 分隔） | `sidebar_provider (class: "w-full min-h-0 flex-1 flex-col") { sidebar_header { Pills row 原样 } ... }` |
| `col { for demo { button { style-if 两串 ~15 行; onclick lambda; row{title+徽标}; text desc } } }` | `scroll (style: "flex-1 min-h-0 px-3 py-4") { sidebar_menu { for demo { sidebar_menu_item { sidebar_menu_button (active: .selected_id == demo.id, onclick: 同 lambda, style: "h-auto py-1.5") { col { row{text title + 徽标}; text desc } } } } } }` |

注意点：
- menu_button 的 `style:` 透传 class（548 臂通用规则），`h-auto` 覆盖默认
  固定高；active 态样式由契约承载（bg-sidebar-accent），手搓 primary/10
  类串删除——这正是"统一成正常 Sidebar 样式"的目标。
- "可交互/独立"徽标保留为 title 行内 text（颜色类串保留，属内容不是导航
  chrome）。
- VM 端：scroll=Scrollable、menu_button=契约三态；`for` 循环内 active
  表达式引用循环变量 demo.id——561 契约子集对 for 内 expr 的支持在实跑
  验证，若红则记录并按 562 惯用法降级（预计算 active 字段）。

## 测试设计

1. 构建级：`auto gen` 重生成绿，生成物 App.vue 含 SidebarProvider/
   SidebarHeader/SidebarMenu/SidebarMenuButton/ScrollArea 且不含左栏旧
   style-if 类串（grep 锚）。
2. Vue 端实跑（主端，`render: "vue"`）：dual_probe 截图首屏（Sidebar 样式
   证据）+ press 一个 demo（selected_id 切换、右栏视口加载）+ pills 点击
   筛选 + scroll_probe.mjs 悬停/滚轮截图（ScrollArea 细滚动条证据）。
   证据存 scratch/p573/。
3. VM 端冒烟：`auto run -r vm` 起，快照含 sidebar 菜单树 + press 一个 demo
   切换（AppViewport 嵌入的 VM 支持度不在本计划范围，只验左栏与选择链）。
4. 本计划不动 crates/ → 不触发 cargo t/docs_gen 门禁（Category A）；若执行
   中被迫改 crates/ 代码，升级 Category B 并补对应 scoped 测试。

## 验收标准

- [ ] app.at 左栏全部 sidebar 族化，手搓 style-if active/hover 类串零残留
      （grep 锚：`bg-primary/10 border border-primary/40` 不再出现）
- [ ] Vue 端实跑：Sidebar 契约样式可见、pills 筛选正常、demo 点击切换 +
      active 高亮正常、ScrollArea 细滚动条证据（截图在 scratch/p573/）
- [ ] VM 端冒烟：侧栏菜单树渲染 + press 切换正常（快照证据）
- [ ] 右栏嵌入视口/搜索/主题切换零回归（Vue 截图对照）
- [ ] 若改了 crates/ 代码：对应 scoped 测试绿；未改则记录 Category A 裁定

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **T1 迁移 app.at 左栏**：按详细设计对照表改写
   `examples/ui-gallery/src/front/app.at:121-178`（aside 摘 overflow-y-auto、
   provider+header 包 pills、scroll+menu+menu_button 替换 for-button）。
   验证：`cargo build -p auto` 后
   `cd examples/ui-gallery && ../../target/debug/auto.exe gen` 退出码 0，
   且 `grep -c "SidebarMenuButton" gen/front/vue/src/App.vue` ≥ 1、
   `grep "bg-primary/10 border border-primary/40" src/front/app.at` 零命中。
2. **T2 Vue 端实跑**：`python scratch/p562/dual_probe.py --auto-bin
   ./target/debug/auto.exe --app-dir examples/ui-gallery --save-dir
   scratch/p573 --app uigallery --press-text "Counter" --skip-vm`
   （render 已是 vue，无需 --vue-render）；再用
   `node scratch/p562/scroll_probe.mjs http://localhost:3049/
   scratch/p573/p573_sidebar` 采 ScrollArea 悬停/滚动截图（侧栏选择器
   必要时从 aside 锚调整）。
   验证：探针退出码 0，scratch/p573/ 截图人工复核 Sidebar 样式。
3. **T3 VM 端冒烟**：同探针 `--skip-vue --press-text "Counter"`（或快照中
   实际存在的首个 demo 名）。
   验证：快照含 sidebar_menu 树 + press 后 selected_id 切换（快照 diff 非空）。
4. **T4 收尾**：grep 复核（仓内无遗漏手搓导航残留——只核 ui-gallery）、
   plan 簿记、证据清单清点。
   验证：`grep -nE 'style: if \.selected_id' examples/ui-gallery/src/front/app.at`
   零命中；scratch/p573/ 证据齐（vue 首屏/pressed/scroll 两 shot/vm 快照）。

## 复审记录

## 待澄清事项
