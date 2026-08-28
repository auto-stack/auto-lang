---
plan_id: PLAN-469
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: specs-tree-deoddity
author: [zcode]
created_at: 2026-08-28
updated_at: 2026-08-28

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 0
---

# [PLAN-469] specs-tree-deoddity

## 变更摘要

修复 docs/specs 树两处结构违例（467 结构收敛的漏网之鱼）+ lint 规则补齐：
① 根级游离文件 http-server-spec.md 归位 stdlib/design/http-server.md（v1 设计 §7 的原计划）；
② aavm/ 项目下 10 个平铺文档 + matrix html + data/ 归入 aavm/design/（对齐"project + design/"规约）；
③ spec-lint.py 对一层子目录的 module 检查豁免 design/ 目录（规约已定义其为合法非模块目录）。

## 目标

## 架构方案

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

## 详细设计

## 测试设计

## 验收标准

## 执行步骤

1. git mv docs/specs/http-server-spec.md → docs/specs/stdlib/design/http-server.md。
2. aavm 11 件（10 md + matrix-434.html）→ docs/specs/aavm/design/；data/ → design/data/。
3. spec-lint.py check_structure 豁免 design 目录名。
4. 引用批量修复：specs README v2（模板基准路径）、plan-spec-hybrid-model（4 处）、aavm/project.md 与内部全路径互引、归档 plans 431/432/434/447（aavm/<f> → aavm/design/<f>，aavm/data → aavm/design/data）、auto/lib/README.md、auto/lib-legacy/README.md、docs/guides/aavm-sync-guide.md。排除：在途 5 文件与 archive/467-468。
5. 验证：lint（含新豁免，aavm/data 警告应消失且无新告警）、全仓链接检查、残留 grep、对账（125 文件不变，仅位置变）。

## 复审记录

## 待澄清事项
