#!/usr/bin/env bash
# new-plan.sh <slug> — 中央取号并创建 plan 文件骨架
#
# 重要：必须在 master 主检出上运行（不要在 plan-NNN worktree 里取号），
# 否则并发 worktree 会撞号（历史教训：336/337/338/342/351/355/359 重复）。
# 取号成功后应立即 commit .next-id 与新 plan 骨架，再开 worktree 实施。
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

ID="$(tr -d '[:space:]' < "$ID_FILE)"
if ! [[ "$ID" =~ ^[0-9]+$ ]]; then
  echo "error: $ID_FILE does not contain a number: '$ID'" >&2
  exit 1
fi

# 防御：若该编号文件已存在（任何目录），直接报错，不覆盖
if ls "$PLANS_DIR/${ID}-"*.md "$PLANS_DIR/archive/${ID}-"*.md "$PLANS_DIR/old/${ID}-"*.md 2>/dev/null | grep -q .; then
  echo "error: plan $ID already exists on disk; bump .next-id manually after checking" >&2
  exit 1
fi

PLAN_FILE="$PLANS_DIR/${ID}-${SLUG}.md"
cat > "$PLAN_FILE" <<EOF
---
plan: ${ID}
title: ${SLUG}
affects: []   # 必填：受影响的 specs 路径，如 [auto-lang/vm]
status: draft # draft | in-progress | complete
---

# Plan ${ID}: ${SLUG}

> **For Claude:** （执行上下文：worktree 名、构建/测试命令、前置 skill、回归要求）

## Goal / 目标

## 背景 / 已确认的决策

## 任务（按阶段）

## 风险与缓解

## Out of Scope

## Verification
EOF

echo "$((ID + 1))" > "$ID_FILE"
echo "created: $PLAN_FILE"
echo "next id: $((ID + 1))"
echo "提醒：请先在 master 上 commit .next-id 与 plan 骨架，再创建 plan-${ID}/${SLUG} worktree。"
