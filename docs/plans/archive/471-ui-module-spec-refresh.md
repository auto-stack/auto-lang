---
plan_id: PLAN-471
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: ui-module-spec-refresh
author: [zcode]
created_at: 2026-08-28
updated_at: 2026-08-28

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "auto-lang/ui/overview.md: 现状节整体重写（360 时代 → 2026-08-28 桌面线时代）"
  - "auto-lang/ui/architecture.md: 架构图补桌面运行时/样式/Action/编辑器节点"
new_spec_components:
  - "auto-lang/ui/architecture.md: ADR-09..17（契约单源/DSL 现代化/Action 分层/VM 组件边界/会话化/虚拟桌面路线 A/shell 即 App/vue 宿主/parity 规范）"
  - "auto-lang/ui/plans.md: 桌面线与 parity 时代 45 行补录 + 358-367 归档列校正"
  - "auto-lang/{vm,trans,frontend,runtime}/plans.md: 2026-08 增补节"
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉一致"
  - "GOAL-009: 虚拟桌面与桌面 Shell"
  - "GOAL-018: 开发范式与知识账本运转（首个内容型回填样板）"
             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 6
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
   [✅ 已完成] 3 个后台代理返回完整蒸馏（活跃 14 + 归档 17 + 早期 13）
   B=归档 4xx UI（435/436/438/443/444/445/449/450×2/451×2/452×2/453-multi-app/454/457/459）、
   C=早期 UI 相关（365/374/411/413/414/418/420/421/422/423/425/426/428）。
   每计划输出：标题/终态/一句话沉淀（plans.md 行）/架构决策（ADR 候选）/代码落点/已知坑。
2. 读现有 auto-lang/ui/{overview,architecture,plans}.md，保 ADR 追加语义。
   [✅ 已完成] 已读原三件；ADR-01..08 原文保留未改写
3. 合成刷新 ui 三件 + project.md ui 行 + INDEX 重生成。
   [✅ 已完成] 217ad8d98；overview 重写/architecture 追加 ADR-09..17/plans.md 45 行
4. vm/trans/frontend/runtime 的 plans.md 补行 + overview 现状行微调。
   [✅ 已完成] 217ad8d98；vm+3/trans+4/frontend+4/runtime+2 行
5. specs/overview.md 模块状态行同步。
   [✅ 已完成] 217ad8d98；project.md ui 行 active + overview.md 桌面线状态同步 + INDEX 重生成
6. 验证：lint 0 err、链接检查、plans.md 行与实际 plan 文件对账。
   [✅ 已完成] lint 0/0/0；模块 spec 断链零；代码落点抽查 OK（wm.rs/session.rs/style/theme.rs/action_config.rs 存在、schema/aura.at 含 virtual_window）

## 复审记录

## 待澄清事项

---

## spec-sync 回写记录（2026-08-28，/auto-plan:merge）

- 回写载体即本计划交付物（ui overview/architecture ADR-09..17/plans.md 45 行 + 四模块补行）。
- `.autoos/specs.json`：P471-1..2 入库（reports/designs，幂等）。
- 归档：`docs/plans/archive/`，`status: archived`（终态）。
