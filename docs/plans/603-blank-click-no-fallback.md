---
plan_id: PLAN-603
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: 空白点击撤「最近块回落」（对齐网页轨槽外无效果语义）
author: [zhaopuming]
created_at: 2026-09-09
updated_at: 2026-09-09

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 1
total_steps: 1
---

# [PLAN-603] 空白点击撤「最近块回落」

## 变更摘要

PARITY #19 观察①（2026-09-09 用户裁定：对齐网页轨）：编辑壳 `hit_test` 在块矩形
未命中时回落「最近块中心 y」（core.rs:2052-2061 一带），致点内容外空白聚焦邻块、
光标被搬移——网页轨槽外点击无效果。本计划撤该回落：鼠标点击未命中任何块矩形
即无效果（焦点/caret 不动）。plan-061 复审已实证该回落可感知（OBS-1）。

## 目标

- 鼠标点击（MousePressed 路径）在所有块矩形之外 → 无效果（不建焦点/不动
  caret/不建拖选锚点）。
- 键盘路径（navigate_vertical 的 Action::Click y=±∞ 语义）与表格列宽拖拽
  命中不受扰。
- headless 单测钉死：块间 gap 与文末下方点击 → focus/caret 原状；块内点击
  语义不变（plan-061 三测回归）。

## 架构方案

`hit_test` 现两段：矩形包含优先，未命中回落最近块中心 y。新增严格口径
（仅矩形包含，未命中返回 None）供 `handle_mouse_press` 使用；原宽松口径若
无其他调用方则收口删除（以 grep 为准）。块矩形为 cosmic 布局 rect，fence
含 padding 全格命中域、cell 全格——矩形外即真空白。

## 需求分析与背景调查

- plan-061 复审 OBS-1 实证：文末下方 y=lastY+60 点击 → focus=末块且哨兵落
  其行内；首块上方/右侧同理。网页轨（click-caret 通道）槽外点击无效果。
- 现状代码：core.rs `hit_test`（:2047 一带）+ `handle_mouse_press`
  （:1162）调用点；`block_action` 的 Action::Click 走 cosmic 邻近字形
  （navigate_vertical 依赖 y=±∞ 语义，不触 hit_test）。

## 详细设计

1. `hit_test_strict(&layout, x, y) -> Option<usize>`：仅矩形包含。
2. `handle_mouse_press` 改用严格口径；`let Some(hit) = ... else { return
   DocOutput::default() }` 语义保留（命中不了即无效果）。
3. 宽松 `hit_test` 视调用方清点结果：无消费方则删除（YAGNI），有则保留。

## 测试设计

- headless：两块文档 → 文末下方/gap/首块上方点击 → focused_block 与各块
  cursor 均不变；块内点击仍建焦点（plan-061 三测 + 055 cell 测回归）。
- 实机（可选）：vm 探针复跑 OBS-1 口径（文末下方点击无效果）。

## 验收标准

1. 空白点击（块矩形外）无任何焦点/caret/选区效果。
2. 块内点击、表格 cell 命中、fence 全格命中、键盘路径零回归。
3. 单测钉死 + 全量 lib 基线差集口径过门。

## 执行步骤

- [✅ 已完成] T-1 实现：hit_test_strict + handle_mouse_press 换用 + 宽松口径清点
      处置 + headless 单测。验证：`cargo test -p auto-lang --features
      autodown --lib autodown_editor::core` scoped 绿。
      （2026-09-09 提交 a4be09367：hit_test_strict 仅矩形包含，press 路径换用；
      宽松 hit_test 留守拖选路径（handle_mouse_drag 拖穿 gap 续选邻块惯例，
      注释定档唯一消费方）；mouse_click_blank_outside_blocks_is_noop（文末下方
      +块间 gap 点击 focus/caret 双不变）。core::tests 82/82 绿。）

## 复审记录

- stage: work | PLAN-603 | r1 | outcome: **pass** | 2026-09-09。
  code_commit：plan-603-dev a4be09367（base 4d88854f6）。evidence：core::tests
  82/82（含 mouse_click_blank_outside_blocks_is_noop）；全量 lib wt 4635p/205f
  vs master 同轮 4643p/201f——差名 4 项裁定为零回归：ffi×1 并行抖动（串行过）、
  osconfig_daemon 家族 3 项为 master 基线期破损（master 串行同跑 11p/5f 同族
  红，并行会话新引入，非本计划回归）。blockers：无。next: review。

## 待澄清事项
