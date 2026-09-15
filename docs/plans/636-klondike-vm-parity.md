---
plan_id: PLAN-636
status: execution_done
feature_name: klondike-vm-parity
author: [agent]
created_at: 2026-09-15T07:32:00Z
updated_at: 2026-09-15T09:25:00Z
plan_revision: 1
current_step: 6
total_steps: 6
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [auto-os/apps/037-klondike]
---

# [PLAN-636] Klondike VM/Iced 端视觉对齐修复

## 0. 变更摘要

对比 Vue 端与 VM/Iced 端的 037-klondike 截图，发现 VM 端存在布局结构问题与视觉差异。本 Plan 通过修改 `auto-os/apps/037-klondike/src/front/` 的 `.at` 源文件完成了双端兼容性重构，消除了布局截断，并澄清了 AURA 快照中 `[Image]` 占位标签的设计机理。

## 1. 目标

- 修复 VM 端顶栏溢出（`flex-wrap` → 两行固定布局）
- 修复 `card_face.at` 中 `absolute inset-0` 牌面中心区定位（`flex-1` 居中）
- 验证双端渲染行为与视觉一致性（Vue 与 VM）
- 澄清 VM 端卡牌重叠与 icon 实际渲染机理

## 2. 架构方案

全部改动在 `auto-os/apps/037-klondike/src/front/` 的 `.at` 源文件内：
- `app.at`: 顶栏重构为两行式 `col`，避免依赖 Iced 未支持的 `flex-wrap`。
- `card_face.at`: 牌面中心区域从 `absolute inset-0` 重构为 `flex-1 items-center justify-center`。

## 3. 技术栈

- AutoUI `.at` 源文件（`app.at`, `card_face.at`）
- VM 验证：`auto run -r vm` + MCP Python 截图脚本
- Vue 验证：`auto build -r vue` + Playwright 截图

## 4. 验收标准与验证结果

| ID | 验收项 | 验证结果 | 状态 |
|---|---|---|---|
| AC-01 | VM 端顶栏三组控件完整显示，无横向溢出或截断 | [✅ 已完成] 顶栏改为两行 col 布局，标题/皮肤/指标/操作按钮全部清晰可见 | PASS |
| AC-02 | VM 端卡牌面中心花色/人头字居中显示 | [✅ 已完成] 改用 flex-1 纵横居中，Showcase 中 A/K/Q/J 居中完美，7列正面牌也居中 | PASS |
| AC-03 | VM 端 7 列牌桌列高度不压缩，叠牌层次可见 | [✅ 已完成] 7列均正确向下排布展开 | PASS |
| AC-04 | VM 端发牌堆张数角标显示在卡牌右下区域 | [✅ 已完成] 24 张角标清晰显示在发牌堆右下角 | PASS |
| AC-05 | Vue 端外观与修改前一致（无回退） | [✅ 已完成] vue_after_fix.png 证实 Vue 端表现与修改前完全一致 | PASS |

## 5. 执行步骤与证据

- [x] **T-01** 创建 worktree `D:/autostack/.wt/lang-636/auto-lang`，分支 `plan-636-dev`
  - 证据：`git worktree add D:/autostack/.wt/lang-636/auto-lang -b plan-636-dev` 执行完成
- [x] **T-02** 修复 `app.at` 顶栏：`flex-wrap` → 两行 col 布局（AC-01）
  - 证据：`apps/037-klondike/src/front/app.at` 顶栏改为两行独立 row，VM 截图顶栏不再有任何溢出或警告
- [x] **T-03** 修复 `card_face.at` 中心区：`absolute inset-0` → `flex-1 items-center justify-center`（AC-02）
  - 证据：`card_face.at` 移除 absolute inset-0，Showcase 中 A/K/Q/J 艺术字及牌面中心花色居中显示
- [x] **T-04** 检查发牌堆角标与牌桌高度
  - 证据：发牌堆右下角数字 `24` 已自然对齐，牌桌 7 列下挂展开正常
- [x] **T-05** 双端回归构建与截图验证
  - 证据：`auto build -r vue` 构建成功，Playwright 截图生成 `tests/screenshots/vue_after_fix.png`
  - 证据：VM 端 `auto run -r vm` 截图生成 `tests/screenshots/klondike_vm_test.png`
- [x] **T-06** 澄清分析与核查总结
  - 证据：AURA 快照中的 `text "[Image]"` 系 VNode 树检视层对 `View::Image` 节点的常规字符串化表示，实际 Iced 引擎正常加载渲染了 Lucide 矢量图标。

## 6. 复审记录

stage: execution_done | PLAN-636 | revision 1 | outcome: pass | next: user review
authorized scope: auto-os/apps/037-klondike .at 源文件修改，无 Rust 变更
diff verified: 仅修改 app.at 与 card_face.at，双端截图比对通过。
