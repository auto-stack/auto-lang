---
plan_id: PLAN-600
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: callout 编辑臂盒式 chrome 对齐只读臂（撤 3px 左条，画 kind 色盒）
author: [zhaopuming]
created_at: 2026-09-09
updated_at: 2026-09-09

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 2
total_steps: 2
---

# [PLAN-600] callout 编辑臂盒式 chrome 对齐只读臂

## 变更摘要

用户 2026-09-09 实机复核报告（截图在案）：VM demo 双臂的 `$callout` 样式差很远
——编辑臂（左）只画 PLAN-054 T8 的 3px kind 色左条 + 标题行（引语式观感），
只读臂（右）是 `CALLOUT_CHROME` 完整盒（rounded border kind 色 50% + 底 kind
色 10% + px-4 py-3）。裁定：**统一用只读臂的盒式形态**，编辑臂撤左条、改画
kind 色盒（矩形近似圆角——fence 盒同款绘制原语）。

## 目标

- 编辑臂 callout 容器 chrome = 盒式：连续 callout 叶运行段一次画「底
  （kind -500 @ alpha 0.10，宽 = 视口宽）+ 四条 1px 边（@ alpha 0.50）」，
  首叶顶含标题行带。
- 撤 per-叶 3px 左条（引语式残段）。Details 摘要式呈现不在本计划范围
  （用户仅点名 callout）。
- headless 单测钉死盒填充（色/α/几何），实机双臂截图留证。

## 架构方案

- 绘制原语沿用 `DocDrawList.fills`（矩形填充，先于文本）；fence 盒
  （bg + header 带 + 4 条 1px 边）为同款先例。圆角以直角矩形近似——
  与编辑臂 fence 盒一致，不做逐角弧绘制。
- kind 色单源 `callout_kind_rgb`（-500/-400 档，未知 kind 落 accent），
  盒底/边均取 -500 档（bg α=0.10 / border α=0.50，语义对齐
  `callout_kind_classes` 的 `bg-*-500/10` 与 `border-*-500/50`）。
- 运行段分组：render_frame 逐叶循环内累积连续 callout 叶
  `(rgb, top, bottom)`；段界 = 该叶带 `cont_title`（新 callout 起）或
  strip 色/中断变化；循环末 flush 残段。top 取首叶标题带顶
  （y + extra_top − CONT_TITLE_H），bottom 逐叶以 y + total_h 刷新
  （末叶含 CONT_PAD_B_CALLOUT 底 pad）。

## 需求分析与背景调查

- 只读臂样式源：`autodown_blocks.rs` `CALLOUT_CHROME`（outer
  `rounded-lg border`、body `px-4 py-3`）+ `callout_kind_classes`
  （info = `border-blue-500/50 bg-blue-500/10`）。
- 编辑臂现状源：core.rs render_frame「PLAN-054 T8：callout 左条（3px…）」
  块 + `walk_skeleton_attribution` Seg::Callout 臂（cont_strip/cont_title/
  cont_extra_top/bottom，CONT_PAD_X=16 与只读 px-4 同值）。
- 双臂截图：编辑臂左条式 vs 只读臂盒式（用户截图 + 本计划 T-2 复现截图）。

## 详细设计

1. render_frame 逐叶循环：以 `callout_run: Option<(u8,u8,u8,f32,f32)>`
   累积运行段；新段判定 = `at.cont_title.is_some()` 或 strip 色变；非
   callout 叶即 flush。flush = 推底 + 四条边（宽 viewport_w，x=0——对齐
   fence 盒与只读 w-full）。
2. 撤原 3px 左条 push 块（引语式残段；quote 左条路径不动）。
3. 循环后 flush 残段（文档以 callout 结尾时不丢盒）。

## 测试设计

- headless：callout 文档 render_frame → fills 含 α≈0.10 的 blue-500 底
  （宽≈视口宽）与 4 条 α≈0.50 的 1px 边；双 callout 文档 → 两盒独立
  （中间隔叶断开）；fence/quote 回归不受扰。
- 实机：worktree 构建 exe，demo/auto 双臂截图（编辑臂盒式 vs 只读臂盒式
  同形态），留证 PNG。

## 验收标准

1. 编辑臂 callout 呈盒式（底+边框），不再出现 3px 左条；颜色取
   kind -500 档 α 0.10/0.50。
2. 只读臂不受扰；quote/Details/fence chrome 零回归。
3. headless 单测钉死 + 全量 lib 门禁按基线差集口径过门。

## 执行步骤

- [✅ 已完成] T-1 实现：render_frame callout 盒式 chrome（运行段累积 + flush +
      撤左条）+ headless 单测。验证：`cargo test -p auto-lang --features
      autodown --lib autodown_editor::core` scoped 绿。
      （2026-09-09 提交 4317d7885：运行段累积器 + push_callout_box 底@0.10/
      四边@0.50 + 撤 3px 左条；callout_edit_arm_paints_kind_box /
      callout_boxes_split_across_containers 两新测 + 054 旧测改写新契约，
      core::tests 78/78 绿。）
- [✅ 已完成] T-2 实机留证 + 门禁：worktree exe 双臂截图（PNG 入册）；全量 lib
      失败名集与 master 基线逐名差集为空。
      （2026-09-09 提交 825d120fd：vm-600-callout-box.png 双臂盒式同形态
      实证；wt 4617p/206f vs master e8f67b9b2 4616p/205f，逐名差集 3 项均为
      无关子系统并行竞争抖动（vm::ui_console×2 串行复跑过、
      ui::osconfig_daemon×1 master 侧），差集口径达标。）

## 复审记录

- stage: work | PLAN-600 | r1 | outcome: **pass** | 2026-09-09。
  code_commit：plan-600-dev 4317d7885（T-1 core.rs 盒式 chrome + 单测）→
  825d120fd（T-2 截图证据）；依赖 worktree `.wt/lang-600/auto-down`
  （detached master 只读，autodown-core path 依赖解析）。
  task_ids：T-1/T-2。evidence：core::tests 78/78；全量 lib wt 4617p/206f
  ⊆ master e8f67b9b2 4616p/205f（差集 3 项均无关子系统并行竞争抖动，
  串行复跑全过）；vm-600-callout-box.png 双臂盒式同形态。
  blockers：无。next: review（execution_done；worktree 留存待复审/合并）。

## 待澄清事项
