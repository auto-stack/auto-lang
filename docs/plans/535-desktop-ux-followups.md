---
plan_id: PLAN-535
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-ux-followups
author: [zhaopuming]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [ui/session, ui/iced, virtual_window.rs, popover.rs, code_editor]                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 0
---

# [PLAN-535] desktop-ux-followups

## 变更摘要

**滚动跟踪计划**（528 先例）：PLAN-526（桌面 Shell UX 十一波 39 任务，已归档
`docs/plans/archive/526-desktop-shell-ux-fixes.md`）复审后登记的未竟项与用户复核项，
统一在本计划继续跟踪。每个条目逐条转为执行步骤修复并验证；新的桌面问题亦可在此滚动登记。

承接自 526 的四组：
- **A 已知视觉债两项**（KNOWN-DEBT 🟢）：popover 首次打开横向锚点偏左；
  window_thumbnail 懒捕获前显示空。
- **B 公共基建一项**（KNOWN-DEBT 🟡，用户核准延后）：wrap_layout_onclick
  布局件级 hover/右键公共基建。
- **C 用户复核清单**（功能已实现+测试承载，真实手势/视觉待确认）：
  T20 跨窗首击、T24 最大化底缘、T28 三键 hover 盒、T31 icon 双击打开、
  T32 Ctrl+Tab 松手提交、T37 标题栏右键菜单、T38 launcher 滚动。
- **D 崩溃观察**：桌面进程常驻 RUST_BACKTRACE=1（8916 实例已带）——
  code_editor clamp（526 T21）与截图零尺寸（T39）两处已修；若再发同族
  panic 直接取全栈定位。

## 目标

1. 修复 popover 首开横向锚点偏左（首次打开时定位偏移，之后正常）。
2. 修复 window_thumbnail 懒捕获前空白显示。
3. wrap_layout_onclick 布局件级 hover/右键公共基建落地（或明确再延后并记录理由）。
4. 用户复核清单逐项确认（复核通过即勾销；发现问题转为新任务）。

## 架构方案

- A1（popover 锚点）：`popover.rs` Panel::layout 的 content 尺寸在首次
  layout 时可能为 0/未测量（首帧 bounds 未就绪）→ 首开定位偏左。方向：
  首帧后重定位（on_layout 二次定位）或缓存 anchor bounds 于 visible 翻转时
  刷新。
- A2（thumbnail 空显）：`snapshot.rs` request_capture 队列 + 渲染臂
  fallback icon 兜底已存在——空显发生在懒捕获回调前。方向：兜底态绘制
  skeleton（浅色占位块）替代空白，捕获完成后 ServiceTick 刷新。
- B（wrap_layout_onclick）：为 .at 布局件（row/col/grid/container）统一
  提供 hover 类消费与 oncontextmenu 挂点（当前仅 mouse-area/button 支持）。
  影响全示例回归面（526 待澄清③裁定独立立项）——本计划内先做设计草稿 +
  试点（launcher/桌面），全量铺开另立。
- C：无代码改动；用户反馈驱动。

## 需求分析与背景调查

- 前置：PLAN-526（11 波 39 任务，已归档）——本计划所有条目源自其复审记录
  与 KNOWN-DEBT-AND-RISKS.md 登记（🟡×1、🟢×2）。
- 用户复核项原文见 526 归档文件"复审记录"表与 T20/T24/T28/T31/T32/T37/T38
  各任务条目。

## 详细设计

（A1/A2/B 逐项实施时回填本节。）

## 测试设计

- A1：popover 首开定位 headless 单测（Simulator：visible 翻转后首帧 bounds
  断言不出锚定边）。
- A2：thumbnail 空显期 skeleton 断言（现有 snapshot 组测试扩展）。
- B：wrap_layout_onclick 试点组件单测 + 既有示例回归（全示例对拍另立）。
- C：实机截图/手势（用户路径）。

## 验收标准

- [ ] A1 popover 首开锚点居正（headless 断言 + 实机目检）。
- [ ] A2 thumbnail 空显消除（实机目检）。
- [ ] B wrap_layout_onclick 设计草稿 + 试点落地（或用户裁定再延后并更新
      KNOWN-DEBT 理由）。
- [ ] C 用户复核清单七项逐项确认并勾销。

## 执行步骤

（每项开工时从本清单转正；完成追加 [✅ 已完成] 证据。）
- [ ] A1 popover 首开锚点偏左修复（popover.rs Panel::layout）。
- [ ] A2 thumbnail 空显兜底（snapshot.rs/渲染臂）。
- [ ] B wrap_layout_onclick 设计草稿 + launcher/桌面试点。
- [ ] C 用户复核七项逐项销账（T20/T24/T28/T31/T32/T37/T38）。

## 复审记录

（/auto-plan:review 回填）

## 待澄清事项

1. A2 空显兜底的视觉形态（skeleton 块 vs 首帧降级 icon）需用户定夺。
2. B 的铺开范围（仅桌面 vs 全示例）影响回归面，试点后定。
