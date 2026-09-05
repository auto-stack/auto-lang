---
plan_id: PLAN-561
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: sidebar-vm-contract
author: [kimi]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 10
---

# [PLAN-561] sidebar-vm-contract

## 变更摘要

Plan 548（P1）已完成 sidebar_* 23 元素的 schema 扩全与 Vue 端 shadcn 1:1 接线，
但 VM/iced 端仍是空壳——`crates/auto-lang/src/aura/element_coverage.rs:344-366`
中全部 23 个 sidebar_* 元素均为 `QueueStatus::NotConsumed`。本计划（P2）按设计文档
[sidebar-family-and-nav-retirement](../../design/autoui/sidebar-family-and-nav-retirement.md)
§3.3 落地 **VM 端契约子集**：sidebar 容器/分区/分组/菜单层级在 VM 端结构等价渲染，
active/hover/disabled/折叠状态正确，`to:` 走 `__navigate`；并以 nav_contract.rs 为
模板新建 `sidebar_contract.rs` class token 契约 + 单测，锁定双端不漂移。

## 目标

1. 新建 `ui_gen/sidebar_contract.rs`：sidebar 族 class token 单一来源，单测锁定
   VM 解析与 scaffold 资产镜像（镜像 nav_contract.rs 三测试模式）。
2. `aura_view_builder.rs` 新增 sidebar 族 VM 构建臂：容器（side/variant 布局语义）、
   header/content/footer 分区（content 可滚动）、group + label + content 层级、
   可折叠 group、menu/item/button 层级。
3. 状态语义：menu_button 的 active（`active` prop + 带 `to:` 时 exact/前缀自动探测，
   同 nav-item 语义）/hover/disabled；`to:` → dispatch `__navigate`（route/mod.rs
   现有机制）；badge；sub 菜单缩进。
4. `element_coverage.rs` 中子集元素从 NotConsumed 升格登记；不在子集的元素
   （rail/input/skeleton/trigger/icon-collapse/tooltip）保持 NotConsumed 并注明归属。
5. widgets-gallery `sidebar.at` 页 VM 实跑，与 Vue 端截图结构等价对拍通过。

## 架构方案

- **契约层**：`crates/auto-lang/src/ui_gen/sidebar_contract.rs`（新建）——class token
  常量按区域分组（CONTAINER/HEADER/CONTENT/FOOTER/GROUP*/MENU*/BUTTON 态/BADGE/SUB），
  与 `crates/auto-man/assets/shadcn-ui/sidebar/*.vue` scaffold 资产互为镜像；单测
  锁定：(a) token 可被 VM Tailwind v3.4 清单管道解析（P527 契约），(b) hover/active
  变体解析进对应 class 槽，(c) 与 scaffold 资产字符串互锚防漂移。
- **构建层**：`crates/auto-lang/src/ui/aura_view_builder.rs` 新增 sidebar 臂，结构
  参照 Plan 482 nav 臂（:4016 起）——契约类转换 + 语义锚（data-active、
  sidebar-name/sidebar-desc 等价锚）。
- **渲染层**：`crates/auto-lang/src/ui/iced/renderer.rs` 既有 `AbstractView::Sidebar`
  臂（:4348）对齐契约（宽度/分区/间距），menu button 视觉态等价 token。
- **路由层**：`to:` 复用 `crates/auto-lang/src/route/mod.rs` 的 `__navigate` +
  `__current_route` + `router.back()` 历史栈，与 nav-item 同语义。
- **coverage 层**：`element_coverage.rs:344-366` 子集元素升格，非子集元素注明
  "VM 子集外（设计 §3.3 不做清单）"。

## 技术栈

Rust（auto-lang crate：ui_gen / aura_view_builder / iced renderer / route）、
Tailwind v3.4 清单驱动 VM class 管道（P527）、widgets-gallery 双端对拍
（autoui-verifier 技能脚本）。

## 需求分析与背景调查

- specs 现状：`docs/specs/auto-lang/ui/overview.md` 已登记 sidebar 组件族段
  （Plan 548 沉淀：23 元素 schema 扩全 + Vue 端 1:1 接线 + `to:`/`active` D2 扩展 +
  Provider 自动包裹），GOAL-007（双端视觉一致）/GOAL-010（示例应用轨道）关联 548。
- 设计文档 §3.3 D3 契约子集表（做/不做清单）为本计划范围唯一权威；§2.2 非目标
  （mobile 整批、cookie、快捷键）本计划同样不做。
- P548 遗留债与本计划的接口：P548-D2（`sidebar_menu_button.tooltip` 登记未实现）
  按设计 §3.3 属于 VM 不做清单（collapsed 衍生能力），本计划不实现、不关闭该债；
  VM 端 tooltip 若将来要做需单独立项。
- 前车之鉴：nav 族 VM 端"标 full 实际不稳"（设计 §1.2），故本计划以**结构等价 +
  契约单测 + 实跑对拍**三重证据验收，不接受"分发点存在即完成"。

## 详细设计

VM 子集映射表（设计 §3.3 落地为元素级）：

| 元素 | VM 语义 |
|---|---|
| `sidebar` | 容器：side（左/右）与 variant（sidebar/floating/inset 布局差异）映射为 iced 布局参数；`collapsible="offcanvas"` 桌面=显隐切换；`"icon"` 模式不在子集（按无折叠处理 + 登记） |
| `sidebar_provider` | open 状态容器：defaultOpen/open 受控，VM 经 store/事件臂重构建切换 |
| `sidebar_header` / `sidebar_footer` | 分区容器（契约 padding/gap） |
| `sidebar_content` | 分区容器 + 可滚动（iced scrollable） |
| `sidebar_separator` | 分隔线 |
| `sidebar_group` / `sidebar_group_label` / `sidebar_group_content` | 分组层级；group 可折叠（点击 label 切换） |
| `sidebar_group_action` | 分组右上角动作槽 |
| `sidebar_menu` / `sidebar_menu_item` | 菜单列表层级（ul/li 语义 → column 结构） |
| `sidebar_menu_button` | 按钮：active/hover/disabled 三态 + `to:` 导航 + size 变体 |
| `sidebar_menu_action` / `sidebar_menu_badge` | 行内动作槽 / 徽标 |
| `sidebar_menu_sub` / `sidebar_menu_sub_item` / `sidebar_menu_sub_button` | 子菜单缩进层级，button 同 menu_button 态语义 |
| `sidebar_inset` | 主内容区容器 |

不在子集（保持 NotConsumed 或显式 no-op 登记）：`sidebar_rail`（拖拽）、
`sidebar_trigger`（VM 端触发器由 provider 状态直驱，后续可补）、`sidebar_input`、
`sidebar_menu_skeleton`、icon-collapse 模式、tooltip（P548-D2）、cookie/快捷键/mobile。

## 测试设计

1. **契约单测**（sidebar_contract.rs 内，TDD 先行）：
   - `sidebar_contract_tokens_parse_on_vm`：全部常量经 VM class 管道解析非空；
   - `sidebar_contract_active_hover_parse`：active/hover 变体进对应 class 槽；
   - `sidebar_contract_matches_scaffold_assets`：与
     `crates/auto-man/assets/shadcn-ui/sidebar/*.vue` 字符串互锚（镜像
     nav_contract.rs:129 测试）。
2. **构建臂单测**：aura_view_builder sidebar 臂产出结构（层级/锚点/状态 class）
   断言；`to:` 自动探测（exact/前缀）用例。
3. **coverage 测试**：`cargo t element_coverage` 升格登记一致。
4. **实跑对拍**：`auto run -r vm` widgets-gallery → sidebar 页，autoui-verifier
   `test_vm_mcp.py` 截图，与 Vue 端截图（P548 已有 `scratch/p548/final/`）结构等价
   比对；证据存 `scratch/p561/`。

## 验收标准

- [ ] `sidebar_contract.rs` 三测试全绿（`cargo t sidebar_contract`）
- [ ] 子集元素在 VM 端全部可构建渲染，`cargo t sidebar` / `cargo t iced` 绿
- [ ] `cargo t element_coverage` 绿（升格登记一致）
- [ ] widgets-gallery sidebar 页 VM 实跑截图与 Vue 端结构等价（分区/分组/菜单层级/
      active 态可辨），证据在 `scratch/p561/`
- [ ] `cargo check -p auto-lang` 零警告；合入前 `cargo tf` 全绿（pre-fold 门禁）
- [ ] 非子集元素保持 NotConsumed 且注明设计出处；P548-D2 债不关闭不扩大

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **T1 契约模块骨架**：新建 `crates/auto-lang/src/ui_gen/sidebar_contract.rs`，按
   区域分组定义 pub const class token（从 P1 Vue 发射臂
   `crates/auto-lang/src/ui_gen/vue.rs` sidebar 臂与 scaffold 资产
   `crates/auto-man/assets/shadcn-ui/sidebar/*.vue` 抄录）；在
   `crates/auto-lang/src/ui_gen/mod.rs` 注册 `pub mod sidebar_contract;`。
   验证：`cargo check -p auto-lang`。
2. **T2 TDD 契约测试**：在 sidebar_contract.rs 内写三测试（tokens_parse_on_vm /
   active_hover_parse / matches_scaffold_assets，镜像 nav_contract.rs:100-145），
   确认先红。验证：`cargo t sidebar_contract`（预期失败）。
3. **T3 构建臂：容器与分区**：`crates/auto-lang/src/ui/aura_view_builder.rs` 新增
   sidebar/sidebar_provider/sidebar_header/sidebar_content/sidebar_footer/
   sidebar_separator/sidebar_inset 构建臂（参照 :4016 nav 臂结构，契约类转换）；
   content 接 iced scrollable。验证：`cargo check -p auto-lang` + T2 测试转绿。
4. **T4 构建臂：分组与菜单层级**：同文件新增 sidebar_group 族（含可折叠 group
   状态切换）与 sidebar_menu/sidebar_menu_item/sidebar_menu_sub 族缩进层级臂。
   验证：新增结构断言单测 `cargo t sidebar`。
5. **T5 menu_button 状态语义**：active（prop + `to:` exact/前缀自动探测，复用
   nav-item 探测语义）/hover/disabled 三态 + data-active 锚；disabled 不响应事件。
   验证：`cargo t sidebar`（三态与探测用例）。
6. **T6 `to:` 导航接线**：menu_button/sub_button 的 `to:` dispatch `__navigate`
   （`crates/auto-lang/src/route/mod.rs` 现有机制，`__current_route` 更新 +
   历史栈）。验证：`cargo t sidebar`（导航 dispatch 用例）。
7. **T7 iced 渲染对齐**：`crates/auto-lang/src/ui/iced/renderer.rs`
   `AbstractView::Sidebar` 臂（:4348 起）对齐契约宽度/分区/间距，menu button
   视觉态等价 token。验证：`cargo t iced`。
8. **T8 coverage 升格**：`crates/auto-lang/src/aura/element_coverage.rs:344-366`
   子集元素改登记为已消费（注明本计划），非子集元素保持 NotConsumed 并改写注明
   "VM 子集外（sidebar-family-and-nav-retirement §3.3）"。
   验证：`cargo t element_coverage`。
9. **T9 双端对拍**：worktree 内 `auto run -r vm` 跑 examples/widgets-gallery
   sidebar 页，用 `.agents/skills/autoui-verifier/scripts/test_vm_mcp.py` 截图，
   与 Vue 端截图结构等价比对（分区/分组/菜单/active 态逐项），证据存
   `scratch/p561/`。验证：对拍清单全过。
10. **T10 收尾门禁**：`cargo check -p auto-lang` 零警告 + `cargo t sidebar` +
    `cargo t iced` + `cargo t element_coverage` 全绿；金样若因 VM 结构变动漂移则
    按既有重采样流程更新。验证：上述命令全绿。

## 复审记录

## 待澄清事项
