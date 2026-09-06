---
plan_id: PLAN-573
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: uigallery-sidebar-migration
author: [kimi]
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: [GOAL-010, GOAL-007]   # 010 示例轨道(ui-gallery 统一 sidebar 族);007 双端一致(迁移后双端冒烟+预存差异基线在案)

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 4
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

- [x] app.at 左栏全部 sidebar 族化，手搓 style-if active/hover 类串零残留
      （grep 锚：`bg-primary/10 border border-primary/40` 不再出现）
- [x] Vue 端实跑：Sidebar 契约样式可见、pills 筛选正常、demo 点击切换 +
      active 高亮正常、ScrollArea 细滚动条证据（截图在 scratch/p573/）
- [x] VM 端冒烟：侧栏菜单树渲染 + press 切换正常（快照证据）
- [x] 右栏嵌入视口/搜索/主题切换零回归（Vue 截图对照）
- [x] 若改了 crates/ 代码：对应 scoped 测试绿；未改则记录 Category A 裁定

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. [✅ 已完成] **T1 迁移 app.at 左栏**：按详细设计对照表改写
   `examples/ui-gallery/src/front/app.at:121-178`（aside 摘 overflow-y-auto、
   provider+header 包 pills、scroll+menu+menu_button 替换 for-button）。
   验证：`cargo build -p auto` 后
   `cd examples/ui-gallery && ../../target/debug/auto.exe gen` 退出码 0，
   且 `grep -c "SidebarMenuButton" gen/front/vue/src/App.vue` ≥ 1、
   `grep "bg-primary/10 border border-primary/40" src/front/app.at` 零命中。
   — 证据：gen 退出码 0；App.vue SidebarMenuButton=3/Sidebar 族+ScrollArea=16 命中；
   旧类串双文件零命中；worktree 提交 c4afa1845（注：需在组内补 auto-down 兄弟
   worktree 供 autodown-core path 依赖解析，与 lang-564 同型）。
2. [✅ 已完成] **T2 Vue 端实跑**：`python scratch/p562/dual_probe.py --auto-bin
   ./target/debug/auto.exe --app-dir examples/ui-gallery --save-dir
   scratch/p573 --app uigallery --press-text "Counter" --skip-vm`
   （render 已是 vue，无需 --vue-render）；再用
   `node scratch/p562/scroll_probe.mjs http://localhost:3049/
   scratch/p573/p573_sidebar` 采 ScrollArea 悬停/滚动截图（侧栏选择器
   必要时从 aside 锚调整）。
   验证：探针退出码 0，scratch/p573/ 截图人工复核 Sidebar 样式。
   — 证据：双探针退出码均 0；截图 4 张在 scratch/p573/（vue_home/vue_pressed/
   sidebar_hover/sidebar_scrolled）。**执行中发现并修复一处缺口**：首跑
   scrollTop 恒 0、viewport clientHeight=scrollHeight=2300——aside 只写
   `flex-col` 无 display:flex（Tailwind flex-col 不隐含 display），provider
   flex-1 塌缩，ScrollArea 未受约束、窗口级滚动顶破 header；按 widgets-gallery
   `md:flex` 先例给 aside 补 `flex` + `min-h-0`（worktree 提交 bd8a47ed8），
   复测 viewport 639/2300、wheel scrollTop 0→800、悬停可见细滚动条 thumb。
   另：press-text 改用 "Login"（Counter 是默认选中项，压它零视觉差，换
   Login 实证切换链——右栏视口加载 Login 应用、active 高亮迁移成功）。
3. [✅ 已完成] **T3 VM 端冒烟**：同探针 `--skip-vue --press-text "Counter"`（或快照中
   实际存在的首个 demo 名）。
   验证：快照含 sidebar_menu 树 + press 后 selected_id 切换（快照 diff 非空）。
   — 证据与偏差记录：快照含迁移后 sidebar 结构树（aside 子树内 scrollable+
   menu 容器链，见 scratch/p573/snapshot_uigallery_vm_home.txt）；
   **demo 列表项在 VM 端为空——实证为预存限制非本迁移回归**：数据源
   `filteredDemos => filterDemosBy(...)` 是 Vue-only TS extern fn
   （app.at:4 `from "src/front/utils/demos.ts"`），master 基线同探针同 2 项
   断言失败（scratch/p573/master-baseline/，按钮数 20=20 一致）。补做 pill
   press 冒烟：VM 端 `基础` pill press 返回 ok，`active_category:
   "all" -> "01-basic"`，快照 diff 非空（snapshot_uigallery_vm_pill_pressed.txt）
   ——左栏 onclick 链 VM 端正常。menu_button 的 for 内 active/onclick 因列表
   无数据无法在 VM 实证（562 降级惯用法无适用对象），已登记待澄清事项 ②。
4. [✅ 已完成] **T4 收尾**：grep 复核（仓内无遗漏手搓导航残留——只核 ui-gallery）、
   plan 簿记、证据清单清点。
   验证：`grep -nE 'style: if \.selected_id' examples/ui-gallery/src/front/app.at`
   零命中；scratch/p573/ 证据齐（vue 首屏/pressed/scroll 两 shot/vm 快照）。
   — 证据：两 grep 锚零命中（另核 `hover:bg-muted/60 border border-transparent`
   零命中）；scratch/p573/ 清单 = vue_home/vue_pressed/sidebar_hover/
   sidebar_scrolled 4 PNG + vm 快照 2 份 + master-baseline 对照 + 服务日志，
   探针落入 app 目录的 tests/screenshots 杂散已清除；证据提交 3 笔中的
   最后一笔（T1 c4afa1845 / T2 bd8a47ed8 / T4 证据）。

## 复审记录

**复审人**: kimi (/auto-plan:review) · **时间**: 2026-09-06
**范围**: worktree `D:/autostack/.wt/lang-573/auto-lang` 分支 plan-573-dev，
4 笔提交（c4afa1845 迁移 / bd8a47ed8 约束链修复 / 5a6a8ae9b+b8c7805ce 证据）；
diff 面 = `examples/ui-gallery/src/front/app.at` + `scratch/p573/` 证据，
**零 crates/ 改动 → Category A 成立，不触发 cargo t/tf/docs_gen 门禁**
（AGENTS.md Category A：纯示例资产改动严禁跑全档，复审同守）。
master 在分支点后前进 1 笔（b0d430349 plan571 文档），未触 ui-gallery，
无冲突面。

**验收标准逐项复核**（verify, don't trust）:

1. **左栏 sidebar 族化、手搓类串零残留** — ✅ PASS。复审重跑：
   `bg-primary/10 border border-primary/40` 0 命中、`style: if .selected_id`
   0 命中（另核 `hover:bg-muted/60 border border-transparent` 0 命中）；
   gen 产物 App.vue 含 SidebarProvider/SidebarHeader/SidebarMenu(Button)/
   ScrollArea 共 12 命中，`gen exit=0`。
2. **Vue 端实跑** — ✅ PASS。契约样式可见（双行条目 + active 高亮
   bg-sidebar-accent）；demo 点击切换实证（压 Login→右栏视口加载 Login
   应用、高亮迁移，p562_uigallery_vue_pressed.png）；ScrollArea 细滚动条
   + 内部滚动实证（hover 见 thumb，wheel scrollTop 0→800、窗口 chrome
   不动，p573_sidebar_hover/scrolled.png）。**复审补验执行期遗漏的两项**：
   pills 筛选（组件 pill→列表收敛组件类，p573_pill_filter.png）、搜索联动
   （login→单条，p573_search_filter.png）——均正常。
3. **VM 端冒烟** — ✅ PASS（带记录）。迁移后 sidebar 结构树在 VM 快照中
   渲染（aside 子树 scrollable+menu 容器链）；pill press 实证
   `active_category: all→01-basic` 快照 diff 非空。**demo 列表 VM 端为空
   为预存限制**：registry 数据源全系 Vue-only TS extern fn
   （app.at:4 `demos.ts`），master 基线同探针同 2 项断言失败
   （scratch/p573/master-baseline/，按钮数 20=20）——非本迁移回归，
   menu_button for 内 active/onclick 的 VM 实证因此无对象（待澄清事项①②）。
4. **右栏/搜索/主题零回归** — ✅ PASS。右栏嵌入视口正常（pressed 图 Login
   应用渲染）；搜索联动正常（复审补验）；亮色主题切换全页生效 + 设置
   popover 正常（p573_theme_light.png）。
5. **crates/ 改动门禁** — ✅ PASS。未改 crates/，Category A 裁记录在案。

**遗漏/延后/workaround 猎捕**:

- 遗漏（执行期）: Vue pills 筛选与搜索/主题实跑证据缺失 → 复审已补验
  补齐，非阻断。
- 延后： 无静默延后；VM menu_button 实证缺失已明示登记（待澄清事项①②），
  附 master 基线对照，非静默。
- Workaround: 无。aside `flex min-h-0` 是约束链根修（flex-col 不隐含
  display:flex 的 Tailwind 事实 + widgets-gallery `md:flex` 先例），
  非补丁；press-text Counter→Login 偏差已注记（Counter 为默认选中项）。
- **债项候选**（登记 KNOWN-DEBT 建议）: ui-gallery registry 数据源
  Vue-only TS extern（VM 端列表/标题空），跨端化需另立计划——见
  待澄清事项①。

**spec-impact**: 纯示例资产迁移，无 spec 组件增改（supersedes/new 均空）；
touched_goals = GOAL-010（示例轨道）+ GOAL-007（双端一致冒烟与基线对齐）。

**裁定**: 全部验收项 PASS，无阻断债项 → `status: reviewed`。

## 待澄清事项

1. **VM 端 demo 列表数据源（预存限制，非本计划回归）**：ui-gallery 的
   `filteredDemos`/`getDemoTitle` 等全部是 Vue-only TS extern fn
   （`src/front/utils/demos.ts`），VM 端无从执行 → 侧栏列表与右栏标题
   在 VM 上为空（master 基线同探针同败，证据 scratch/p573/master-baseline/）。
   本计划只迁移左栏导航形态，不解决数据源跨端化；若希望 ui-gallery 在
   VM 端可用，需另立计划把 registry 数据源迁回 .at（或 VM extern 桥）。
2. **T3 验收口径**：计划原文要求"VM 快照含 sidebar_menu 树 + press 后
   selected_id 切换"。因 ①，menu_button 条目在 VM 无数据可压；已用
   sidebar_header pill press（状态切换 + 快照 diff）替代证明左栏交互链，
   menu_button 的 for 内 active/onclick VM 实证留给 ① 解决后补。请复审
   裁定该替代是否满足 T3 意图。

## spec-sync 回写记录

- `.autoos/specs.json`：P573-1..6 六节存款（reports/goals/architecture/designs/tests/reviews 各 +1，id 稳定幂等）。
- `docs/specs/auto-lang/ui/overview.md`：现状段追加 573 一段（迁移要点 + aside `flex min-h-0` 约束链惯用法 + VM 预存限制注记）。
- `docs/specs/auto-lang/ui/plans.md`：573 行追加。
- `docs/specs/goals.md`：GOAL-007 / GOAL-010 行各补 573 关联。
- `docs/plans/KNOWN-DEBT-AND-RISKS.md`：P573-D1 登记（ui-gallery registry Vue-only TS extern 数据源，VM 端列表空，跨端化另立小计划）。
- worktree 清理：`D:/autostack/.wt/lang-573/auto-lang`（wt-guard 首跑 BLOCKED——`auto gen`/`pnpm` 工具链生成的 387 个 junction 逐一 link-only 摘除后复检 clean）+ 只读兄弟 `auto-down`（零改动）均已移除，组目录已删；分支 plan-573-dev / lang-573-dev 已删。
