---
plan_id: PLAN-562
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: nav-to-sidebar-migration
author: [kimi]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 9
---

# [PLAN-562] nav-to-sidebar-migration

## 变更摘要

按设计文档
[sidebar-family-and-nav-retirement](../../design/autoui/sidebar-family-and-nav-retirement.md)
§3.4 D4 执行 nav 族迁移与退役：把全部 `nav-group`/`nav-item` 使用方迁移到
sidebar_* 族（Plan 548 Vue 端已 1:1、Plan 561 VM 契约子集），随后 schema 中
nav 族标记 superseded_by。实现移除（nav_contract.rs、scaffold 资产、构建/渲染臂）
按设计"观察一个周期后移除"**不属本计划**，拆后续小计划。

## 目标

1. 仓内三示例迁移：`examples/ui/015-notes/src/front/sidebar.at`、
   `examples/ui/018-book-reader/src/front/app.at`、
   `examples/ui/019-video-app/src/front/app.at` 的 nav-group/nav-item 用法
   改写为 sidebar_* 组合（group+label+content、menu+item+button、to:/active）。
2. widgets-gallery 文档页重写：`examples/widgets-gallery/src/front/pages/navitem.at`
   与 `navlink.at` 改写为 sidebar 族示例页（或裁定删除并由 sidebar.at 承接，
   执行时按 gallery 信息架构二选一并记录）。
3. 外仓迁移：`auto-musk`（052 应用）与 `auto-os-config`（012 应用），按 Plan 529
   分组平铺布局在 `D:/autostack/.wt/lang-562/{auto-musk,auto-os-config}` 开各自
   worktree（branch `lang-562-dev`），迁移完成后各 fold 回其 master。
4. `nav` 容器 `search: true` 内联搜索行：若有使用方依赖，平移为
   `sidebar_header` + `sidebar_input` 组合写法（VM 端 sidebar_input 不在 561 子集，
   该情形需先登记再裁定）。
5. schema 退役标注：`schema/aura.at:538-589` 的 `nav`/`nav-group`/`nav-item`
   标记 superseded_by sidebar_*（`nav-link` 已被 nav-item supersede，随族一并标注），
   用 `SCHEMA_DRIFT_UPDATE_BASELINE=1` 更新基线。

## 架构方案

- **迁移映射**（nav 族 → sidebar 族，设计 §3.4 + 548 D2 扩展）：
  - `nav` 容器 → `sidebar` + `sidebar_content`（Provider 由 codegen 自动包裹，548 已落地）
  - `nav-group` + label → `sidebar_group` + `sidebar_group_label` + `sidebar_group_content`
  - `nav-item`（icon/label+desc/badge、`to:`、active）→ `sidebar_menu` +
    `sidebar_menu_item` + `sidebar_menu_button`（`to:`/`active` 语义已对齐，
    badge → `sidebar_menu_badge`）
  - 嵌套分组 → `sidebar_menu_sub` / `sidebar_menu_sub_item` / `sidebar_menu_sub_button`
  - `nav(search:true)` 内联搜索行 → `sidebar_header` + `sidebar_input`（VM 端见目标 4 的裁定）
- **跨仓工作流**：组目录 `D:/autostack/.wt/lang-562/`，外仓 worktree 以本仓命名
  （branch `lang-562-dev`），消费验证后即 fold 回各仓 master，不留悬挂。
- **退役门控**：全部迁移方双端回归通过 → supersede 标注；实现移除留观察期，
  另立小计划执行（删 nav_contract.rs / scaffold Nav*.vue / aura_view_builder nav 臂 /
  iced 分发点 + P548-D3 旧臂死代码一并清理）。

## 技术栈

Auto 语言示例源码（.at）、auto-lang codegen 双端（Vue/VM）、跨仓 git worktree
（Plan 529 布局）、autoui-verifier 双端对拍、schema drift 基线流程。

## 需求分析与背景调查

- specs 现状：`docs/specs/auto-lang/ui/overview.md` 导航组件线（plan-482）与
  sidebar 组件族（plan-548）两段并存；本计划完成后导航组件线段整体改写为退役记录。
- 使用方清单（grep 实证 2026-09-05）：仓内 015-notes、018-book-reader、
  019-video-app 三例含 nav-group/nav-item；widgets-gallery 有 navitem.at/navlink.at
  两个 nav 族文档页（navigationmenu.at 是另一 shadcn 组件，不动）；外仓
  auto-musk/auto-os-config 按设计 §3.4 记载（执行时 grep 复核）。
- 前置依赖：Plan 561（VM 契约子集）完成后本计划的 VM 端回归才有意义；
  若 561 未完成，仓内迁移可先做 Vue 端验证，VM 回归留待 561 入库后补。
- GOAL-010（示例应用轨道）、GOAL-007（双端一致）关联。

## 详细设计

逐文件迁移要点：

| 文件 | 现状 | 迁移目标 |
|---|---|---|
| `examples/ui/015-notes/src/front/sidebar.at` | nav-group/nav-item 侧栏 | sidebar 族组合，笔记列表 active 态走 `to:`/`active` |
| `examples/ui/018-book-reader/src/front/app.at` | app.at 内 nav 用法 | 同上 |
| `examples/ui/019-video-app/src/front/app.at` | app.at 内 nav 用法 | 同上 |
| `examples/widgets-gallery/src/front/pages/navitem.at` | nav-item 文档页 | sidebar 族示例页或删除（sidebar.at 承接） |
| `examples/widgets-gallery/src/front/pages/navlink.at` | nav-link 文档页 | 同上 |
| auto-musk 052 / auto-os-config 012 | 外仓 nav 用法 | 同映射表，各自 worktree |

schema 标注：`schema/aura.at` nav 族三元素加 `superseded_by: "sidebar_*"` 元数据
（格式参照仓内既有 supersede 标注先例）；`docs/components/` 生成文档随
docs_gen 再生成同步。

## 测试设计

1. 每迁移一例：`auto run`（Vue）+ `auto run -r vm`（VM）实跑，导航点击/active
   态/折叠行为回归；截图证据存 `scratch/p562/<app>/`。
2. widgets-gallery：nav 页重写后 gallery_golden 重采样 + 双端对拍。
3. schema 标注后：`SCHEMA_DRIFT_UPDATE_BASELINE=1` 更新基线，
   `cargo t schema_drift` 绿 + `cargo test -p auto-lang --test docs_gen` 绿
   （Category C 门禁）。
4. 外仓：各仓自身验证命令 + auto-lang 侧消费验证（组内 `../auto-lang` 解析）。

## 验收标准

- [ ] 仓内三示例 + 两个 gallery 页全部迁移，无双端回归（截图证据在 scratch/p562/）
- [ ] auto-musk / auto-os-config 迁移完成并各 fold 回其 master；仓内 grep
      `nav-group|nav-group|nav_item` 在 examples 下零残留
- [ ] `nav` search:true 依赖方已平移或实证无依赖方
- [ ] schema nav 族 supersede 标注入库，schema_drift 基线更新、docs_gen 绿
- [ ] 实现移除后续小计划已立项（含 P548-D3 清理）或在本计划 review 时显式裁定延期
- [ ] 合入前 `cargo tf` 全绿（pre-fold 门禁）

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **T1 使用方复核**：仓内 `grep -rn "nav[-_]\(group\|item\|link\)" examples/ examples/widgets-gallery/src` +
   外仓 `grep -rn` auto-musk/auto-os-config，输出最终迁移清单附 plan。
   验证：grep 输出非空且与本文清单对账。
2. **T2 迁移 015-notes**：改写 `examples/ui/015-notes/src/front/sidebar.at` 为
   sidebar 族组合。验证：`auto run` 实跑 + 导航 active 态截图（scratch/p562/015/）。
3. **T3 迁移 018-book-reader**：改写 `examples/ui/018-book-reader/src/front/app.at`
   nav 用法。验证：同 T2（scratch/p562/018/）。
4. **T4 迁移 019-video-app**：改写 `examples/ui/019-video-app/src/front/app.at`
   nav 用法。验证：同 T2（scratch/p562/019/）。
5. **T5 gallery 页重写**：改写/删除 `examples/widgets-gallery/src/front/pages/navitem.at`
   与 `navlink.at`（sidebar.at 承接），重采样金样。验证：`cargo t gallery_golden` 绿
   + gallery 实跑截图。
6. **T6 外仓迁移**：`git -C D:/autostack/auto-musk worktree add
   D:/autostack/.wt/lang-562/auto-musk -b lang-562-dev`（auto-os-config 同），
   按映射表迁移 052/012 应用，各仓验证后 fold 回其 master 并 wt-guard 清理
   worktree。验证：外仓 grep 零残留 + 各仓自身构建/测试绿。
7. **T7 search:true 裁定**：grep 仓内外 `search:\s*true` 的 nav 用法；有则平移为
   sidebar_header + sidebar_input（VM 端缺口登记 KNOWN-DEBT），无则在 plan 记录
   "无依赖方"。验证：grep 证据附 plan。
8. **T8 schema supersede 标注**：`schema/aura.at:538-589` nav/nav-group/nav-item
   加 superseded_by 标注，`SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo t schema_drift`
   更新基线。验证：`cargo t schema_drift` +
   `cargo test -p auto-lang --test docs_gen` 全绿。
9. **T9 收尾门禁**：仓内 grep nav 族零残留复核 + `cargo check -p auto-lang` 零警告；
   在 KNOWN-DEBT 登记"nav 实现移除观察期"条目并起草后续移除小计划要点
   （含 P548-D3 清理）。验证：上述检查全绿。

## 复审记录

## 待澄清事项
