---
plan_id: PLAN-471
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: ui-module-spec-refresh
author: [zcode]
created_at: 2026-08-28
updated_at: 2026-08-28

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 6
---

# [PLAN-471] ui-module-spec-refresh

## 变更摘要

模块 spec 内容追现状：以 3 个并行蒸馏代理消化 437–465 UI/桌面线计划（活跃 14 + 归档 29 中筛 UI 相关），
刷新 auto-lang/ui 的 overview/architecture（ADR 追加）/plans.md；vm/trans/frontend/runtime 的 plans.md
补行 + 现状行微调；INDEX 重生成。纯 docs/specs 内容任务，不动代码。

## 目标

## 架构方案

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

## 详细设计

## 测试设计

## 验收标准

## 执行步骤

1. 3 个后台蒸馏代理：A=活跃 437-465（437/439/440/441/442/446/448/455/458/462/463/464/465/386）、
   B=归档 4xx UI（435/436/438/443/444/445/449/450×2/451×2/452×2/453-multi-app/454/457/459）、
   C=早期 UI 相关（365/374/411/413/414/418/420/421/422/423/425/426/428）。
   每计划输出：标题/终态/一句话沉淀（plans.md 行）/架构决策（ADR 候选）/代码落点/已知坑。
2. 读现有 auto-lang/ui/{overview,architecture,plans}.md，保 ADR 追加语义。
3. 合成刷新 ui 三件 + project.md ui 行 + INDEX 重生成。
4. vm/trans/frontend/runtime 的 plans.md 补行 + overview 现状行微调。
5. specs/overview.md 模块状态行同步。
6. 验证：lint 0 err、链接检查、plans.md 行与实际 plan 文件对账。

## 复审记录

## 待澄清事项
