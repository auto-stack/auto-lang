---
plan_id: PLAN-630
status: reviewed              # drafting → executing → execution_done → reviewed → archived
feature_name: menubar-component
author: [agent]
created_at: 2026-09-14
updated_at: 2026-09-14

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [auto-lang/ui (声明式 menubar 组件族: menubar-menu/trigger/content/item/checkbox-item/separator)]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, autoui-skill]
current_step: 4
total_steps: 4
---

# [PLAN-630] menubar-component

## 0. 变更摘要

把 VM 端 menubar 的"actions DSL 合成物"升级为**可复用的声明式组件族**（用户裁定：类比 contextmenu/popover 组件形态，语义对齐 Vue 端 shadcn Menubar）：

```text
menubar (class/style) {
    menubar-menu (value: "file") {
        menubar-trigger "文件"
        menubar-content {
            menubar-item (title: "新建", icon: "file-plus", shortcut: "Ctrl+N") { onclick: .ActNew }
            menubar-separator
            menubar-checkbox-item (title: "切换 Console", checked: .store.console_open) { onclick: .ActConsole }
        }
    }
}
```

- **VM 端**：`menubar` 标签分派消歧——**有子节点 = 声明式组件**（新转换），**空标签 = 原 actions DSL 合成**（完全向后兼容）；组件内部复用 PLAN-629 T-06 打磨的菜单项布局（前导槽 icon/勾选 + title 贴左 | shortcut 右贴），触发/开合/定位走公共 Popover 原语（BottomStart）。
- **Vue 端**：声明式族生成 shadcn `Menubar/MenubarMenu/MenubarTrigger/MenubarContent/MenubarItem/MenubarSeparator/MenubarCheckboxItem`（组件注册与 actions 合成共用同一套，`ui_gen/vue.rs` 的 actions menubar 生成器即模板）。
- **auto-edit 迁移**：示例改用声明式 menubar（证明可复用性）；actions DSL 保留（快捷键三源绑定继续生效），`menubar {}` 占位删除。

## 1. 目标

- **G1**：.at 视图代码可声明式使用 menubar 组件族（不依赖 actions DSL），VM/Vue 双端一致渲染。
- **G2**：既有 actions DSL `menubar {}` 合成路径零回归（占位语义不变）。
- **G3**：auto-edit 迁移到声明式组件，行为与迁移前一致（含 enabled/checked 表达式）。
- **非目标**：子菜单（menubar-sub）不在本计划；radio-item 不在本计划；actions DSL 合成路径不删除。

## 2. 架构方案

- **标签分派**：aura_view_builder `"menubar"` 臂——`children` 非空 → `convert_menubar_component(props, children)`；空 → 原 `convert_menubar`。vue.rs 同款消歧。
- **VM 转换**：每 `menubar-menu` 产出触发按钮 + `View::Popover{anchor: Widget(trigger), placement: BottomStart}`（MENUBAR_OPEN 注册表同款开合消息）；item 三形态——item（icon/shortcut 可选）、checkbox-item（checked 表达式经 eval_condition）、separator（横向通栏，复用 convert_sep horizontal）。菜单项布局函数从 convert_menubar 抽出共享（前导槽+title 左组 | shortcut 右组）。
- **Vue 生成**：`menubar` 有子节点 → 直出 shadcn 组件树；事件经标准 onclick 翻译；checked 表达式走 convert_condition（同 view if）。
- **props 约定**：item/checkbox-item：`title`（或文本子节点）、`icon`（lucide 名）、`shortcut`、`checked`（表达式）、`enabled`（表达式）、`onclick`。menu：`value`。

## 3. 技术栈

既有管线（aura_view_builder 标签分派 / view::Popover 原语 / vue 生成器 / shadcn 组件注册），无新依赖。

## 4. 需求分析与背景调查

**授权**：用户 2026-09-14 明确要求"菜单项、分隔线、勾选槽、图标槽做成可复用的 menubar 组件（类似于 contextmenu，但不同），和 Vue 端 shadcn 的 menubar 组件类似"。

**背景证据**（2026-09-14，HEAD=plan-629-dev@a5821a4b6）：
- 现状：`menubar` 标签单一分派 → convert_menubar（aura_view_builder.rs:1911；合成物见 6737-6914，T-06 菜单项布局为最新形态）。
- Vue 侧已有 shadcn Menubar 生成：ui_gen/vue.rs `generate_actions_menubar_html`（5936 起）——Menubar/MenubarMenu/MenubarTrigger/MenubarContent/MenubarItem/MenubarSeparator 注册与 HTML 生成现成，声明式族直接复用组件名。
- contextmenu 先例：无独立标签，即 popover 坐标锚形态（aura_view_builder 6859-6866）——组件族以"标签 + 原语 lowering"为既定模式。
- 开合机制：MENUBAR_OPEN 全局注册表 + `__menubar_toggle/__menubar_close` 内部消息（action_config.rs:369-384；renderer 13310）——组件版复用。

## 5. 详细设计

### 5.1 VM（aura_view_builder.rs）
- 分派：`"menubar"` 臂按 `children.is_empty()` 消歧。
- `convert_menubar_component`：遍历 children 收集 `menubar-menu`（value prop → menu id）；每 menu 内拆 `menubar-trigger`（文本 prop 或首子件）/ `menubar-content`（其余子件）；item 类子件 → 共享 `build_menu_item_view(title, icon, shortcut, checked, enabled, onclick)`（PLAN-629 T-06 布局）。
- 勾选/禁用表达式：`eval_condition_with`（与 actions enabled_if/checked_if 同引擎）。
- 事件：onclick 经既有事件转换（event_to_message_with）。

### 5.2 Vue（ui_gen/vue.rs）
- `menubar` 消歧：有子节点 → 声明式生成（Menubar 树 + @click 标准翻译 + convert_condition）；空 → `generate_actions_menubar_html`。

### 5.3 示例迁移（041-auto-edit）
- app.at：`menubar {}` 占位删除；视图加声明式 menubar（四菜单全量迁移，icon/shortcut 从 actions 声明复制）；actions 块保留（shortcut 回退层 + enabled_if/checked_if 数据源迁移为 item props）。

### 5.4 规范增量

| delta_id | add/modify/retire | docs/specs/... 目标 | before/after 规则 | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/architecture.md（组件族节） | add: 声明式 menubar 组件族（标签/props/双端 lowering；空标签保持 actions 合成语义） | 用户裁定 + 组件复用 | AC-01..03 |

## 6. 测试设计

- VM 单测：声明式结构（menu/item/separator/checkbox 数与 props）、消歧（空标签走合成）、checked 表达式翻转。
- vue 生成单测：声明式 menubar → shadcn 组件树断言。
- 矩阵：041 迁移后 desktop_mcp 全量（菜单流 T2/T3/T4/T5/T8/T10 关键路径）。
- 实机：四菜单渲染、勾选态、禁用态、分隔线、快捷键右贴。

## 7. 验收标准

- **AC-01**：.at 可用声明式 menubar 组件族渲染菜单，VM/Vue 双端一致（Vue 出 shadcn Menubar 树）。
- **AC-02**：item 支持 icon/shortcut/checked/enabled + onclick；separator 横向；checkbox-item 勾选表达式实时。
- **AC-03**：auto-edit 迁移后行为不回归（矩阵 48/2 口径），actions DSL 合成路径零回归。
- **AC-04**：`menubar {}` 空标签向后兼容（actions 合成）。

## 8. 执行步骤

worktree：`D:/autostack/.wt/lang-630/auto-lang`（branch `plan-630-dev`，基于 plan-629-dev 叠放；auto-down 只读兄弟同款）。

- **T-01** [VM] 分派消歧 + convert_menubar_component + 共享 item 布局抽取 + 单测。
- **T-02** [Vue] 声明式生成 + 单测。依赖 T-01（布局约定一致）。
- **T-03** [示例] 041 迁移声明式 + 矩阵回归。依赖 T-02。
- **T-04** [验证] 实机四菜单/勾选/禁用/快捷键 + README/规范注记。依赖 T-03。

### 执行证据（2026-09-14）

- **T-01** ✅ commit bb075889c：`menubar` 标签按 children 消歧（空=actions 合成，T10 锚不受影响）；`convert_menubar_component`（menubar-menu/trigger/content/item/checkbox-item/separator；checked/enabled 走 eval_condition_with；开合复用 MENUBAR_OPEN）；共享 `menu_item_button_view` 抽取（actions 合成与组件族同源布局）。测试 plan630_declarative_menubar_component 绿（结构/横向 sep/勾选表达式翻转）。
- **T-02** ✅ commit 1c024d2de：Vue 生成零新增代码——声明式标签族经既有 shadcn 元素管道直接出 Menubar 组件树（menubar_* 属性臂 Plan 451 时代已预埋），测试 plan630_declarative_menubar_generates_shadcn_tree 绿。
- **T-03/T-04** ✅ commit 98fb5d8c7：auto-edit 四菜单全量迁移声明式（icon/shortcut/checked/enabled；actions 块保留快捷键三源绑定）；矩阵 T10 热重载锚迁 toolbar（菜单不再随 actions 合成——语义演进记录）；T8 退出脏检查适配保持。矩阵 48/2（与 626/629 基线口径一致，仅 2 个存量快照项）；实机四菜单/勾选/分隔线人工验证通过。

## 9. 复审记录

- 2026-09-14 review：`stage: review | plan_id: PLAN-630 | plan_revision: 1 | outcome: pass | reviewed_commit: a91417643 + c3dbc80bf (plan-630-dev) | base_commit: plan-629-dev@ef2238aaf | spec_inputs: SD-01（声明式组件族） | acceptance_results: AC-01..04 全 PASS（双端声明式渲染=单测×2+实机；item 能力=icon/shortcut/checked/enabled 单测断言；迁移零回归=矩阵 48/2；空标签兼容=plan626 合成测试仍绿） | findings: F-1 同 626 复审（ffi-dual 串行组修复落本计划 c3dbc80bf） | evidence: 同复审门禁 + 实机四菜单 | next: merge。
- 2026-09-14 work handoff：`stage: work | plan_id: PLAN-630 | plan_revision: 1 | outcome: pass | code_commit: 98fb5d8c7 (branch plan-630-dev, base plan-629-dev@a5821a4b6 叠放) | task_ids: T-01..T-04 全完成 | evidence: 各任务行 + 矩阵 48/2 | blockers: 无 | next: review。
- 2026-09-14 draft handoff：`stage: new`，PLAN-630 rev1。用户裁定明确（引语见 §4）；Vue 组件注册/生成器现成，VM 侧抽取自已验证的 convert_menubar 内部。`outcome: pass`，`next: work`。

## 10. 待澄清事项

- 无阻塞。风险预登记：`menubar` 标签双语义消歧必须严格按 children 判定（空 vs 非空），actions 合成的既有测试与 T10 热重载锚（"menubar {" 行首锚）不受影响。