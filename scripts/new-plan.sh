#!/usr/bin/env bash
# new-plan.sh <slug> — 中央取号并创建 plan 文件骨架（v2 frontmatter，Plan 467 对齐 auto-plan 范式）
#
# 重要：必须在 master 主检出上运行（不要在 plan-NNN worktree 里取号），
# 否则并发 worktree 会撞号（历史教训：336/337/338/342/351/355/359 重复）。
# 取号成功后应立即 commit .next-id 与新 plan 骨架，再开 .worktrees/plan-<NNN>-dev worktree。
# 范式：/auto-plan:new 起草（status: drafting）→ work 执行 → review 复审 → merge 沉淀归档。
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ID_FILE="$ROOT/docs/plans/.next-id"
PLANS_DIR="$ROOT/docs/plans"

if [ $# -lt 1 ]; then
  echo "usage: $0 <kebab-case-slug>" >&2
  exit 1
fi
SLUG="$1"

if [ ! -f "$ID_FILE" ]; then
  echo "error: $ID_FILE not found" >&2
  exit 1
fi

ID="$(tr -d '[:space:]' < "$ID_FILE")"
if ! [[ "$ID" =~ ^[0-9]+$ ]]; then
  echo "error: $ID_FILE does not contain a number: '$ID'" >&2
  exit 1
fi

# 防御：若该编号文件已存在（活跃区或归档区），直接报错，不覆盖
if ls "$PLANS_DIR/${ID}-"*.md "$PLANS_DIR/archive/${ID}-"*.md 2>/dev/null | grep -q .; then
  echo "error: plan $ID already exists on disk; bump .next-id manually after checking" >&2
  exit 1
fi

PLAN_FILE="$PLANS_DIR/${ID}-${SLUG}.md"
NOW="$(date +%F)"
cat > "$PLAN_FILE" <<EOF
---
plan_id: PLAN-${ID}
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: ${SLUG}
author: []
created_at: ${NOW}
updated_at: ${NOW}

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 0
---

# [PLAN-${ID}] ${SLUG}

## 变更摘要

## 目标

## 架构方案

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

## 详细设计

## 测试设计

## 验收标准

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

## 复审记录

## 待澄清事项
EOF

echo "$((ID + 1))" > "$ID_FILE"
echo "created: $PLAN_FILE"
echo "next id: $((ID + 1))"
echo "提醒：请先在 master 上 commit .next-id 与 plan 骨架，再创建 .worktrees/plan-${ID}-dev worktree（分支 plan-${ID}-dev）。"
