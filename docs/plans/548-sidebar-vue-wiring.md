---
plan_id: PLAN-548
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: sidebar-vue-wiring
author: [kimi]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 10
---

# [PLAN-548] sidebar-vue-wiring

## 变更摘要

sidebar_* 组件族做实三阶段（设计文档
[docs/design/autoui/sidebar-family-and-nav-retirement.md](../design/autoui/sidebar-family-and-nav-retirement.md)）
的**第一阶段：Vue 端接线**。schema/aura.at 中已登记的 21 个 `sidebar_*` 元素
（当前 props TBD、`web: "none"` 空壳）补全 props 声明并映射到仓内现成的 shadcn 原版资产
（`crates/auto-man/assets/shadcn-ui/sidebar/`，25 个 .vue 已入库），codegen 加
`to:` prop（RouterLink）与 Provider 自动包裹；widgets-gallery 的 `sidebar.at` 页重写为
全零件示例，最终以 autoui-verifier 与 shadcn 原版视觉对拍验收。

**范围切割（设计 D1）**：只做桌面宽屏模式；mobile Sheet/isMobile/cookie 持久化/快捷键
整批不做。VM 端渲染属后续 P2 计划，本计划只保证 VM 端不回归（现状 fallback 行为不变）。

## 目标

1. schema/aura.at 的 21 个 sidebar_* 元素 props 补全 + `backends.web: "component"` +
   `vue: { component, import: "@/components/ui/sidebar" }` 映射，S001 schema drift 提示清零。
2. `sidebar_menu_button` / `sidebar_menu_sub_button` 支持 `to:` prop（Vue 渲染 RouterLink，
   hash 路由跳转）与 `active`（映射 isActive，to 模式支持自动探测）——设计 D2。
3. codegen 检测到 `sidebar` 根且无 `sidebar_provider` 祖先时自动包裹 Provider
   （isMobile 恒 false 的桌面路径）——设计 §5 决策。
4. widgets-gallery sidebar.at 重写为覆盖全零件的示例文档页，Vue 端与 shadcn 原版
   官方示例截图对拍通过（autoui-verifier）。

## 架构方案

沿用 nav 族（Plan 482）已验证的接线管线，不改管线本身，只扩展映射数据与两个特判臂：

```
schema/aura.at (element 声明 + vue 映射)
  → ui_gen/widget/registry.rs (schema 驱动的组件注册)
  → ui_gen/vue.rs (SFC 发射：import 收集 + props/事件/slot 转译)
  → auto-man 资产拷贝 (assets/shadcn-ui/sidebar/ → 工程 components/ui/sidebar/)
```

- `to:`/`active` 在 vue.rs 参照 nav-item 的既有发射臂实现（RouterLink 包裹 +
  路由自动探测），不引入新机制。
- Provider 自动包裹在 vue.rs 页面级发射点做：emit 前扫描页面树，含 `sidebar` 且
  无 `sidebar_provider` 祖先时外层包 `<SidebarProvider>`。
- 资产端预期零改动（482 已"参照 sidebar 资产接线"，T1 核实）；若 sidebar/ 不在
  拷贝清单则补一行。

## 需求分析与背景调查

（取材 docs/specs/overview.md 与 module spec）

- **schema/aura.at** 是 AURA 内置组件唯一声明源（Plan 435），sidebar_* 族登记于
  aura.at:4335–4500 区间，多为 "P1 extracted from production tables; props TBD"。
- **auto-lang/ui** 模块（spec: docs/specs/auto-lang/ui/）：ui_gen/vue.rs 为 Vue codegen
  主发射器；nav-item 的 to/active/exact 语义在此已有完整实现可复用。
- **auto-man**（spec: docs/specs/auto-man/project.md）：assets/shadcn-ui/ 为 shadcn
  快照资产源，sidebar/ 目录 25 件已在仓；拷贝接线点在其 vue 工程导出逻辑。
- **widgets-gallery**（examples/widgets-gallery，pac.at front_port 3024）：sidebar.at
  现页只用了 sidebar/header/content/footer/menu/menu-item/menu-button 7 个标签的
  极简示例，需扩为全零件展示；gallery_vue_golden 金样测试覆盖该工程
  （crates/auto-lang/tests/fixtures/gallery_vue_golden.txt），页面重写会改动金样。
- 后续阶段（不在本计划）：P2 VM 契约子集渲染；P3 迁移 015-notes/auto-musk/
  auto-os-config 并退役 nav-group/nav-item。

## 详细设计

### schema 补全（每元素）

props 以 shadcn-vue 官方文档（docs/components/sidebar）为准逐件誊写，要点：

- `sidebar`：side(left/right)、variant(sidebar/floating/inset)、collapsible(offcanvas/icon/none)、class
- `sidebar_provider`：default_open(bool)、open(expr 受控)、onopenchange(msg_ref)、class
- `sidebar_menu_button`：to(string 路由)、active(expr)、disabled(bool)、size(sm/md/lg)、
  tooltip(string，仅登记暂不实现 VM)、icon(string lucide)、class；allows_children
- `sidebar_menu_sub_button`：to/active/size、class
- `sidebar_menu_badge`/`sidebar_menu_action`/`sidebar_group_action`：icon、onclick、class
- 容器件（content/header/footer/group*/menu/menu_item/menu_sub/menu_sub_item/
  separator/input/trigger/rail/inset/skeleton）：label/class 等按原版 props 誊写
- 全部 `vue: { component: <PascalCase>, import: "@/components/ui/sidebar" }`，
  `backends: { web: "component", iced: "fallback"（维持现状，P2 再做）}`

### codegen 两臂

1. **to/active 臂**（vue.rs，参照 nav-item 臂）：`to:` 存在时根标签由 button 换
   RouterLink、`:to` 绑定；`active` 缺省且 `to` 存在时发射路由自动探测表达式
   （exact 前缀段匹配）；VM 端的 `__navigate` 分发不在本计划。
2. **Provider 自动包裹**：页面 emit 前树扫描；缺 Provider 时自动包裹并在 import
   收集里加 SidebarProvider。

### gallery 页重写

sidebar.at 按 shadcn 官方文档结构组织 preview-card 示例：Simple（header/content/footer）、
Group+Label+Action、Collapsible Group（包 collapsible）、Menu Badge/Action/Sub、
Trigger+Provider 受控示例、variant/collapsible 属性表演示；properties 表保留并补全。

## 测试设计

- **schema 闸**：schema 改动后 `SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift`
  重生基线，再裸跑确认绿；gallery 既有 S001 drift 提示（运行日志中 sidebar 相关条目）清零。
- **docs 闸**（Category C，schema 定义改动）：`cargo test -p auto-lang --test docs_gen`。
- **金样**：gallery_vue_golden 更新（页面重写所致），diff 人工过目。
- **类型闸**：每任务后 `cargo check -p auto-lang`。
- **视觉验收**：autoui-verifier（.agents/skills/autoui-verifier/scripts/test_vue_playwright.mjs）
  对 gallery sidebar 页各 preview-card 截图，与 shadcn-vue 官方文档对应示例人工比对。
- **回归**：widgets-gallery 全站 `auto run -r vue` 启动无新 WARNING/ERROR
  （既有 S001/R016 INFO 基线见运行日志）；`cargo t` 快速档收尾。

## 验收标准

1. aura.at 21 个 sidebar_* 元素 props 完整、`web: "component"`、vue 映射就位；
   schema_drift / docs_gen 两闸绿。
2. 生成的 App/页面 .vue 中 sidebar 结构 imports 自 `@/components/ui/sidebar`，
   页面含 sidebar 时自动包 `<SidebarProvider>`。
3. `to:` 在 gallery 示例页实际跳转 hash 路由；`active` 自动探测高亮正确。
4. autoui-verifier 截图对拍：sidebar 页各示例与 shadcn 原版视觉一致
   （桌面宽屏；mobile 行为不在验收范围）。
5. `cargo t` 快速档全绿，无新警告。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T1 资产拷贝核实**：Grep `crates/auto-man/src/vue.rs`（及同 crate 资产拷贝清单处）确认
  `sidebar` 目录在 shadcn 资产拷贝列表中；若缺失，参照 `nav` 的登记行补 `"sidebar"`。
  验证：`grep -n '"sidebar"\|sidebar' crates/auto-man/src/vue.rs` 命中登记行。
- **T2 codegen 映射机制考古**：读 `crates/auto-lang/src/ui_gen/widget/registry.rs` 与
  `crates/auto-lang/src/ui_gen/vue.rs` 中 nav-item 的 `vue: { component, import }` 消费链
  （import 收集点、组件名解析点、props/事件/slot 发射臂、RouterLink 发射臂），
  把 sidebar_* 需要触达的精确行号/函数名清单记录到本 plan 的复审记录区草稿。
  验证：`grep -n 'nav-item\|NavItem\|RouterLink' crates/auto-lang/src/ui_gen/vue.rs | head -30`
  输出可定位全部触达点。
- **T3 schema 补全（骨架+分区件）**：编辑 `schema/aura.at`，为 `sidebar`、`sidebar_provider`、
  `sidebar_content`、`sidebar_header`、`sidebar_footer`、`sidebar_separator`、`sidebar_input`、
  `sidebar_inset`、`sidebar_trigger`、`sidebar_rail` 10 件填 props/aliases/vue 映射/backends。
  验证：`cargo check -p auto-lang` 绿 + `grep -c 'element sidebar' schema/aura.at` 计数不变。
- **T4 schema 补全（分组+菜单件）**：同文件继续 `sidebar_group`、`sidebar_group_label`、
  `sidebar_group_action`、`sidebar_group_content`、`sidebar_menu`、`sidebar_menu_item`、
  `sidebar_menu_button`、`sidebar_menu_action`、`sidebar_menu_badge`、`sidebar_menu_skeleton`、
  `sidebar_menu_sub`、`sidebar_menu_sub_item`、`sidebar_menu_sub_button` 13 件。
  验证：`cargo check -p auto-lang` 绿。
- **T5 schema 双闸**：`SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift`
  重生 `crates/auto-lang/tests/fixtures/schema_drift_baseline.txt` 后裸跑确认绿；
  `cargo test -p auto-lang --test docs_gen` 绿。
- **T6 to/active 发射臂**：在 `crates/auto-lang/src/ui_gen/vue.rs` 为
  `sidebar_menu_button`/`sidebar_menu_sub_button` 加 to→RouterLink + active 自动探测臂
  （复用 T2 定位的 nav-item 臂同函数分叉）。
  验证：`cargo check -p auto-lang` 绿 + 新建最小 .at 用例手动 run 生成物含 `<RouterLink`。
- **T7 Provider 自动包裹**：vue.rs 页面发射点加树扫描与自动包裹 + import 收集。
  验证：T6 用例生成物含 `<SidebarProvider>` 包裹且 import 完整；`cargo check -p auto-lang` 绿。
- **T8 gallery 页重写**：重写 `examples/widgets-gallery/src/front/pages/sidebar.at`
  （结构见详细设计），同步更新 `gallery_vue_golden.txt`（跑对应金样测试 bless）。
  验证：`auto run -r vue`（examples/widgets-gallery）启动，/sidebar 路由可访问、无运行期报错。
- **T9 视觉对拍**：用 autoui-verifier 的 test_vue_playwright.mjs 对 sidebar 页各
  preview-card 截图，与 shadcn-vue 官方 sidebar 文档示例逐一比对；视觉漂移修到一致。
  验证：截图存档至 scratch/p548/ 并在复审记录登记比对结论。
- **T10 收尾闸**：`cargo t` 快速档全绿；格式化与健康检查（无新 warning、无调试打印）；
  更新本 plan 状态。
  验证：`cargo t` 输出 0 failed。

## 复审记录

（/auto-plan:review 时填写；T2 的触达点清单草稿先记此处）

## 待澄清事项

1. `sidebar_menu_button` 的 `tooltip` prop：原版用于 icon-collapsed 态悬浮提示，
   桌面全量模式是否需要首版实现？（倾向：schema 先登记，功能随 P2 再定）
2. gallery 的 preview-card 内嵌预览是否需要作用域样式修正（sidebar 原版假设全页布局），
   以 T9 对拍结果为准。
