---
plan_id: PLAN-548
status: archived              # drafting → executing → execution_done → reviewed → archived
feature_name: sidebar-vue-wiring
author: [kimi]
created_at: 2026-09-04
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "specs/auto-lang/ui（overview.md 组件线节）: 修改"
new_spec_components:
  - "specs/auto-lang/ui: 新增 sidebar_* 组件族（23 元素，Vue 端 shadcn 1:1 接线）登记"
touched_goals:              # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "GOAL-007: sidebar_* 族 Vue 端 shadcn 1:1 落地（to:/active/Provider 自动包裹）"
  - "GOAL-010: widgets-gallery sidebar 文档页全零件重写"

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 10
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
  [✅ 已完成] vue.rs:111 已有 `("@/components/ui/sidebar", "sidebar")` 拷贝登记；:227/:250/:357 用法扫描含 sidebar 且自动附带 @vueuse/core 依赖；零改动。
- **T2 codegen 映射机制考古**：读 `crates/auto-lang/src/ui_gen/widget/registry.rs` 与
  `crates/auto-lang/src/ui_gen/vue.rs` 中 nav-item 的 `vue: { component, import }` 消费链
  （import 收集点、组件名解析点、props/事件/slot 发射臂、RouterLink 发射臂），
  把 sidebar_* 需要触达的精确行号/函数名清单记录到本 plan 的复审记录区草稿。
  验证：`grep -n 'nav-item\|NavItem\|RouterLink' crates/auto-lang/src/ui_gen/vue.rs | head -30`
  输出可定位全部触达点。
  [✅ 已完成] 触达点清单（见复审记录 §T2）：schema vue 映射经 registry.rs:43 `apply_schema_vue_mappings()` 自动消费（零 Rust 改动）；通用 shadcn 发射链 = vue.rs:5102 分发表 + :5175 `is_shadcn_component` + map_tag:7096 + import 生成 :14450-14510；特判臂挂点 = :5104（nav-item 拦截同款）；`generate_nav_item_html`:1430 为 to/active 臂模板；Provider 无自动包裹先例（tooltip-provider:7152 是手动 tag），包裹点 = generate_sfc:1983 模板装配处；资产 SidebarMenuButton 走 as/as-child 多态（SidebarMenuButtonChild.vue），`to:` 拟发 `as-child`+RouterLink 子节点（needs_router 机制 :463/:2467 现成）。
- **T3 schema 补全（骨架+分区件）**：编辑 `schema/aura.at`，为 `sidebar`、`sidebar_provider`、
  `sidebar_content`、`sidebar_header`、`sidebar_footer`、`sidebar_separator`、`sidebar_input`、
  `sidebar_inset`、`sidebar_trigger`、`sidebar_rail` 10 件填 props/aliases/vue 映射/backends。
  验证：`cargo check -p auto-lang` 绿 + `grep -c 'element sidebar' schema/aura.at` 计数不变。
  [✅ 已完成] worktree commit 762f767f：10 件全部补全（aliases/props/web: "component"/vue 映射）；sidebar_inset/input/rail/separator 系新建（原 schema 缺）。全族计数 10→23 属预期（T3/T4 合计，含新建 11）。`cargo check -p auto-lang` 绿。
- **T4 schema 补全（分组+菜单件）**：同文件继续 `sidebar_group`、`sidebar_group_label`、
  `sidebar_group_action`、`sidebar_group_content`、`sidebar_menu`、`sidebar_menu_item`、
  `sidebar_menu_button`、`sidebar_menu_action`、`sidebar_menu_badge`、`sidebar_menu_skeleton`、
  `sidebar_menu_sub`、`sidebar_menu_sub_item`、`sidebar_menu_sub_button` 13 件。
  验证：`cargo check -p auto-lang` 绿。
  [✅ 已完成] 同 commit 762f767f：13 件补全/新建；`sidebar_menu_button`/`sidebar_menu_sub_button` 含 D2 扩展 `to`/`active`；`tooltip` 按待澄清①登记为 declared-deferred。`cargo check` 绿。
- **T5 schema 双闸**：`SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift`
  重生 `crates/auto-lang/tests/fixtures/schema_drift_baseline.txt` 后裸跑确认绿；
  `cargo test -p auto-lang --test docs_gen` 绿。
  [✅ 已完成·有保留] `queue_coverage_drift_fence` 绿；`schema_drift_fence` 红但经 stash 对照实证为**基线前置红**（主检出在途未提交的 alert_dialog/dialog/dropdown 族 aura.at 改动所致，失败清单与本计划零交集；叠加该补丁后 sidebar 改动全闸通过，已干净回退）。docs_gen 3/4 绿，`kitchen_sink_page_in_sync` 同为前置红（PLAN-528 W6 手改所致）。两红待主检出在途工作入库后 fold 前复跑（待澄清③④）。偏离实录：element_coverage.rs +11 行系围栏强制登记；baseline 未重生（drift 维度不含 aura.at props，待澄清⑤）。
- **T6 to/active 发射臂**：在 `crates/auto-lang/src/ui_gen/vue.rs` 为
  `sidebar_menu_button`/`sidebar_menu_sub_button` 加 to→RouterLink + active 自动探测臂
  （复用 T2 定位的 nav-item 臂同函数分叉）。
  验证：`cargo check -p auto-lang` 绿 + 新建最小 .at 用例手动 run 生成物含 `<RouterLink`。
  [✅ 已完成] worktree commit 17e9e79c：拦截臂 vue.rs:5276 + `generate_sidebar_menu_button_html`:1730；`to:`→as-child+内嵌 RouterLink（needs_router 置位），`active`→`:is-active`（资产真名 isActive， SidebarMenuButtonChild.vue 确认），to 存在 active 缺省→`$route.path` 前缀段自动探测。TDD 4 测试先红后绿；`cargo t ui_gen` 743/743 绿；`cargo check` 绿。
- **T7 Provider 自动包裹**：vue.rs 页面发射点加树扫描与自动包裹 + import 收集。
  验证：T6 用例生成物含 `<SidebarProvider>` 包裹且 import 完整；`cargo check -p auto-lang` 绿。
  [✅ 已完成] 同 commit 17e9e79c：`view_tree_has_tag`:225 + generate_template 末尾 :4590 包裹臂（is_shadcn ∧ 含 sidebar ∧ 无显式 provider → 包 `<SidebarProvider>` 并补 import）。TDD 测试绿（含显式 provider 不重复包裹用例）；`cargo t ui_gen` 743/743、`cargo check` 绿。触达 registry.rs（+16 spec 骨架 + 7 件 underscore 别名，overlay 只更新不新建所致缺口，见待澄清⑦）。
- **T8 gallery 页重写**：重写 `examples/widgets-gallery/src/front/pages/sidebar.at`
  （结构见详细设计），同步更新 `gallery_vue_golden.txt`（跑对应金样测试 bless）。
  验证：`auto run -r vue`（examples/widgets-gallery）启动，/sidebar 路由可访问、无运行期报错。
  [✅ 已完成] worktree commit 949f3afb7：8 节 demo（Simple/Group/GroupAction/CollapsibleGroup/Badge+Action/MenuSub/Router 模式 to:/Trigger+显式 Provider）+ 3 张 properties 表；金样 `GALLERY_GOLDEN_UPDATE=1` bless 后 `gallery_golden` 1/1 绿、docs_gen 对拍闸绿、`cargo t ui_gen` 743/743 绿。偏离：金样为整文件哈希，基线自 6077cf9e6 后已漂移（provenance 实锤基线即红），全量重采样波及 ~24 个非 sidebar 行——fold 时须按届时 master 源码重新 bless（待澄清⑩）。
- **T9 视觉对拍**：用 autoui-verifier 的 test_vue_playwright.mjs 对 sidebar 页各
  preview-card 截图，与 shadcn-vue 官方 sidebar 文档示例逐一比对；视觉漂移修到一致。
  验证：截图存档至 scratch/p548/ 并在复审记录登记比对结论。
  [✅ 已完成] commit 0fa06c8ca。截图：scratch/p548/{sidebar-*.png 8 节 + interact-* 3 张 + final/ 修复后重拍}。交互断言：collapsible 开合 ✓、router 点击跳 hash + active 高亮跟随（"Sidebar (this page)"）✓、trigger 宽度 255→47→255 ✓。console 仅 1 条 favicon 404（无害）。对拍发现三处漂移已修：①资产 Tailwind v4 paren-var 语法→v3（宽度曾整体丢失）；②auto-man 模板补 sidebar 色阶+CSS 变量并加每次 run 自愈重写；③preview-card 容器 relative+translateZ(0) 收容 fixed 定位。补充实证：DOM 探针（probe_router.mjs）确认 Router/MenuSub demo 几何/颜色/visibility 全正常，逐节元素截图的"空"为 fixed 定位截图伪影，视口截屏（final/view-*.png）确认实际渲染正确。
- **T10 收尾闸**：`cargo t` 快速档全绿；格式化与健康检查（无新 warning、无调试打印）；
  更新本 plan 状态。
  验证：`cargo t` 输出 0 failed。
  [✅ 已完成·有保留] `cargo t`（--no-fail-fast 全量快速档）：4536 测，失败 12 个，分四簇且**全部在主检出复现**（与本计划零交集）：①`d8_toggle_dark_mode`（015-notes dark_mode 初值断言，主检出同左值右值复现）；②`ui::layout::tests` 9 个（虚拟桌面 WM 布局，疑似宿主机环境相关，主检出全同）；③`kitchen_sink_page_in_sync`（待澄清④）；④`schema_drift_fence`（待澄清③）。本计划触达面测试（ui_gen 743、gallery_golden、docs_gen 对拍闸、component_registry、queue_coverage 围栏）全绿。无新 warning（161 存量基线），无调试打印残留。

## 复审记录

**复审人**：kimi（独立复审，verify-don't-trust） · **时间**：2026-09-04 · **结论：PASS → reviewed**

真实 diff（08ea86ee7..HEAD，4 commit）：aura.at +253、ui_gen/vue.rs +319、
registry.rs +132、element_coverage.rs +11、auto-man/vue.rs +47、sidebar 资产 2 件、
sidebar.at +322、金样 +46 行变动、ui-cache.json（既有跟踪文件同步）。

### 逐项验收复验

| 验收标准 | 判定 | 证据（复审亲跑） |
|---|---|---|
| ①schema 补全 + 双闸 | PASS（有记录保留） | `element sidebar` ×23、vue 映射 ×23 实测；queue_coverage 围栏绿；schema_drift_fence/kitchen_sink 红 = 主检出在途 aura.at 工作前置红，失败清单 grep sidebar 零命中（T5 stash 对照 + 复审复跑双实证） |
| ②imports + Provider 自动包裹 | PASS | gallery_golden 复审复跑 1/1 绿（金样内含 SidebarProvider 包裹与 18 组件 import 断言）；运行态 8 demo 无双包 |
| ③to: 路由 + active 自动探测 | PASS | playwright 实测：点击跳 `#/button` hash、`active` 高亮跟随当前页（"Sidebar (this page)"）；DOM 探针确认 as-child RouterLink 结构 |
| ④截图对拍 | PASS | scratch/p548/ 8 节 + 交互 3 张 + final/ 重拍；三处漂移修复（v4 语法/色阶变量/fixed 收容）后视口截屏与 shadcn 原版结构一致；逐节元素截图"空"已定性为 fixed 定位截图伪影（DOM 探针反证） |
| ⑤cargo t 快速档 + 健康检查 | PASS（有记录保留） | 复审跑 **cargo tf 全量门禁：3410/3412**，2 红即上述前置红；cargo t 快速档 12 红四簇全部主检出同态复现（d8_dark_mode、ui::layout×9 疑似宿主环境相关、kitchen_sink、schema_drift）；编译警告 worktree 161 ≤ 主检出 162（零新增）；diff 零 dbg!/println 残留（auto-man 两处 ⚠ println 为 CLI 用户提示，合惯例） |

### 遗漏/延后/Workaround 扫描结论

- **非静默项**（执行期已全部登记待澄清①–⑩，复审逐条核对属实）。
- **债候选**（复审裁定登记 KNOWN-DEBT，merge 时沉淀）：
  1. 待澄清⑤ schema.rs ↔ aura.at 双侧分化（`SCHEMA_DRIFT_GENERATE_AT=1` 会冲掉手写 props）——**最重要的一条**；
  2. 待澄清① tooltip declared-deferred（计划草稿已声明、用户确认过）；
  3. 待澄清⑧ 死代码臂 vue.rs:11930（保留兜底，清理时机随 nav 退役计划再裁定）。
- **fold 时动作项**（不阻塞 reviewed）：待澄清③④前置红须在主检出在途工作入库后复跑转绿；
  待澄清⑩金样按届时源码重新 bless；待澄清⑥ auto-down sibling worktree 清理。
- **验证方式替换记录**：T6 原定"最小 .at 用例手动 run"以 4 个 TDD 单测 + 金样 dump
  检查替代，证据等价（生成物断言覆盖更严），可接受。
- **轻微**：ui-cache.json 为既有跟踪文件，同步提交与主检出状态一致，不算新债。

### 触达点清单（§T2 考古，备查）

1. **schema → registry 自动消费**：`crates/auto-lang/src/ui_gen/widget/registry.rs:43`
   `WidgetRegistry::with_defaults()` → `apply_schema_vue_mappings()`（:55）按
   schema/aura.at 的 `vue: { component, import }` 重建 vue BackendMapping（Plan 435
   P4-4 单源）。**T3/T4 只改 aura.at，registry.rs 零改动。**
2. **通用 shadcn 组件发射链**（`crates/auto-lang/src/ui_gen/vue.rs`）：
   - 分发表 ~:5102（nav-item 特判在 :5104-5108，sidebar_menu_button 特判臂挂同款位置）；
   - `is_shadcn_component` 判定 :5175（registry `is_backend_supported("vue", tag)`）；
   - `map_tag` :7096（registry 主组件名解析；nav-link→router-link 特判 :7131 附近）；
   - import 生成 :14450-14510（`get_primary_component`/`all_widgets` 遍历
     `shadcn_components_used`，:14457 通用插入点）。
3. **to/active 臂模板**：`generate_nav_item_html` :1430（attrs 收集 :1443、
   `needs_router` 置位 :1451、icon lucide 收集 :1463-1477、NavItem 组件发射 :1487-1496）；
   `nav_attr_fragment` 静态/绑定双态片段。
   - 注意：nav-item 的"路由自动探测 active"实际由 NavItem.vue/RouterLink 内置
     `router-link-active` 类承担，codegen 无 $route 表达式先例——sidebar_menu_button
     的 active 自动探测需在 T6 新写（`:active="$route.path === to || startsWith(to+'/')"`）。
4. **Provider 自动包裹无先例**：tooltip-provider 是手动 tag（:7152 只补 import）。
   包裹点 = `generate_sfc` :1983 / 模板装配 :2263 附近；同时
   `shadcn_components_used.insert("SidebarProvider")` 补 import。
5. **资产侧事实**（`crates/auto-man/assets/shadcn-ui/sidebar/`）：
   - `SidebarMenuButton.vue` = tooltip 包装 + `SidebarMenuButtonChild.vue`（`as`/`asChild`
     多态，inheritAttrs:false + $attrs 透传）→ `to:` 发 `as-child` + `<RouterLink>` 子节点；
   - `utils.ts` 的 `useSidebar` context 被 MenuButton/Rail 等注入 → **Provider 祖先
     是硬需求**（不包则白屏，同 TooltipProvider 教训）；
   - cookie/快捷键/mobile 常量都在 utils.ts，桌面模式不触及（isMobile 走
     vueuse useMediaQuery，桌面恒 false，符合 D1 范围切割）。

## 待澄清事项

1. `sidebar_menu_button` 的 `tooltip` prop：原版用于 icon-collapsed 态悬浮提示，
   桌面全量模式是否需要首版实现？（倾向：schema 先登记，功能随 P2 再定）
2. gallery 的 preview-card 内嵌预览是否需要作用域样式修正（sidebar 原版假设全页布局），
   以 T9 对拍结果为准。
3. **（T5 遗留）schema_drift_fence 前置红**：主检出在途未提交的 alert_dialog/dialog/
   dropdown 族 aura.at 改动（374+/255-）未入库所致，与本计划零交集；fold 前需在
   主检出该工作入库后于本 worktree 复跑确认转绿。
4. **（T5 遗留）docs_gen `kitchen_sink_page_in_sync` 前置红**：PLAN-528 W6 手改
   kitchen-sink.at 未被生成器复现；同③，fold 前复跑。
5. **（T5 债）schema.rs ↔ aura.at 双侧分化**：schema.rs（Rust ElementDef）的 sidebar
   props 仍是旧简版，`SCHEMA_DRIFT_GENERATE_AT=1` 会从 schema.rs 重写 aura.at 并冲掉
   手写 props（nav-item 当年双侧同步做）。本计划未触碰 schema.rs；双侧同步或生成器
   方向裁定登记为债（候选 KNOWN-DEBT）。
6. **（T5 环境）跨仓 sibling worktree**：为解析 autodown path 依赖，已在组目录建
   `D:/autostack/.wt/lang-548/auto-down`（detached，与 auto-down 主检出同 HEAD）。
   fold 清理时需 `git -C D:/autostack/auto-down worktree remove`（wt-guard 先行）。
7. **（T6-T7 偏离）registry.rs 补 16 条 spec 骨架**：`apply_schema_vue_mappings()` 只更新
   已有 spec 不新建，16 件 sidebar 元素在 registry 无 spec 导致 import 反查落空；
   另为既有 7 件补 underscore 别名（get() 不折叠 snake_case）。复审时核对。
8. **（T6 遗留·复审裁定）`generate_shadcn_attrs` 旧 sidebar_menu_button 臂
   （vue.rs:11930 附近）已成死代码**（被新臂遮蔽），保留作非拦截路径兜底；
   是否清理由复审裁定。
9. **（T6 既有行为）`needs_router` 置位连带发射 `const router = useRouter()`**
   （只用 RouterLink 也会），与 nav-item 臂同款，未改。
10. **（T8 偏离）gallery_vue_golden 全量重采样**：fixture 为整文件逐行哈希，基线
    （6077cf9e6 bless）后 gallery 源码与生成器均有大改未重采样，基线即红；
    bless 波及 ~24 个非 sidebar 文件行。**fold 入 master 前须按届时源码重新 bless**
    （主检出在途 aura.at 改动入库后再采样一次）。
