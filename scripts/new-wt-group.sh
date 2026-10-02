#!/usr/bin/env bash
# new-wt-group.sh <group> [--branch <name>] — 一键建 worktree 分组（Plan 529 平铺布局）
#
# 背景（PLAN-726 T-04）：crates 的跨仓 path 依赖（autodown-core）按
# "组内 ../auto-down" 解析（AGENTS.md 解析序），单仓组 cargo 直接失败
# （2026-10-02 实录：手工补建兄弟仓后才通过）。本脚本把
# "mkdir 组目录 + 双仓 worktree add + 流程提示"收拢为一键，与 lang-719/724
# 既有组布局一致。
#
# 用法：
#   bash scripts/new-wt-group.sh <group> [--branch <name>]
#     <group>   组目录名（D:/autostack/.wt/<group>），如 lang-726 / fix-foo
#     --branch  auto-lang 侧开发分支名；缺省 = detached。
#               plan-<NNN>-dev 的 <NNN> 必须经 scripts/new-plan.sh 原子取号。
#   auto-down 兄弟恒 detached（依赖仓只读消费）。
#
# 建组基面恒为主检出 master（脚本可在任意 auto-lang 检出/worktree 运行；
# 主检出与 auto-down 主检出经 git worktree list porcelain 首行解析，与运行
# 位置无关）。
#
# 红线（AGENTS.md）：组内禁止任何 junction/symlink（2026-09-03 三仓 .git
# 事故）；移除组前必须 `bash D:/autostack/wt-guard.sh <worktree>` 输出 clean。
set -euo pipefail

# 脚本所在仓（任意 auto-lang 检出皆可）；主检出与其 auto-down 兄弟从
# porcelain 首行拿，杜绝 msys /d/... 与 D:/... 路径形态错配。
SELF_REPO="$(cd "$(dirname "$0")/.." && pwd)"
MAIN_LANG="$(git -C "$SELF_REPO" worktree list --porcelain | awk 'NR==1 && sub(/^worktree /, "")')"
AUTOSTACK="$(dirname "$MAIN_LANG")"           # D:/autostack
WT_ROOT="$AUTOSTACK/.wt"
DOWN_MAIN="$AUTOSTACK/auto-down"              # auto-down 主检出
GUARD="$AUTOSTACK/wt-guard.sh"

usage() {
  echo "usage: $0 <group> [--branch <name>]" >&2
  exit 1
}

[ $# -ge 1 ] || usage
GROUP="$1"; shift || true
BRANCH=""
while [ $# -gt 0 ]; do
  case "$1" in
    --branch) [ $# -ge 2 ] || usage; BRANCH="$2"; shift 2 ;;
    *) usage ;;
  esac
done

# ---- 校验 ----
if ! [[ "$GROUP" =~ ^[A-Za-z0-9._-]+$ ]]; then
  echo "error: 组名只允许 [A-Za-z0-9._-]（防路径穿越）: '$GROUP'" >&2
  exit 1
fi
GROUP_DIR="$WT_ROOT/$GROUP"
if [ -e "$GROUP_DIR" ]; then
  echo "error: 组目录已存在（可能是在途计划组，禁止复用）: $GROUP_DIR" >&2
  exit 1
fi
if [ ! -d "$DOWN_MAIN/.git" ]; then
  echo "error: auto-down 主检出缺失: $DOWN_MAIN （分组解析序依赖组内兄弟仓）" >&2
  exit 1
fi
if [ -n "$BRANCH" ] && git -C "$MAIN_LANG" show-ref --verify --quiet "refs/heads/$BRANCH"; then
  echo "error: 分支已存在: $BRANCH" >&2
  exit 1
fi
if [ -n "$BRANCH" ] && [[ "$BRANCH" =~ ^plan-[0-9]+-dev$ ]]; then
  echo "note: plan-<NNN>-dev 的编号须经 scripts/new-plan.sh 原子取号（勿自造撞号）"
fi

# ---- 建组（失败中途回滚，不留半组） ----
rollback() {
  echo "error: 建组失败，回滚已建部分" >&2
  git -C "$MAIN_LANG" worktree remove --force "$GROUP_DIR/auto-lang" 2>/dev/null || true
  git -C "$DOWN_MAIN" worktree remove --force "$GROUP_DIR/auto-down" 2>/dev/null || true
  rmdir "$GROUP_DIR" 2>/dev/null || true
  exit 1
}

mkdir -p "$WT_ROOT"
mkdir "$GROUP_DIR"
if [ -n "$BRANCH" ]; then
  git -C "$MAIN_LANG" worktree add "$GROUP_DIR/auto-lang" -b "$BRANCH" master || rollback
else
  git -C "$MAIN_LANG" worktree add --detach "$GROUP_DIR/auto-lang" master || rollback
fi
git -C "$DOWN_MAIN" worktree add --detach "$GROUP_DIR/auto-down" || rollback

LANG_REV="$(git -C "$GROUP_DIR/auto-lang" rev-parse --short HEAD)"
DOWN_REV="$(git -C "$GROUP_DIR/auto-down" rev-parse --short HEAD)"
echo "组已建: $GROUP_DIR （基面 master@$LANG_REV）"
echo "  auto-lang  $([ -n "$BRANCH" ] && echo "分支 $BRANCH" || echo detached) @$LANG_REV"
echo "  auto-down  detached@$DOWN_REV"
cat <<EOF

后续流程（AGENTS.md 范式）：
  1. 开发/验证   cd $GROUP_DIR/auto-lang   （cargo check / 分档 cargo t）
  2. 合入        master 上 merge 分支（Conventional Commit: feat(<scope>): <desc> (Plan <NNN>)）
  3. 移除前置    bash $GUARD $GROUP_DIR/auto-lang   # 必须 clean
                 bash $GUARD $GROUP_DIR/auto-down
  4. 移除        git -C "$MAIN_LANG" worktree remove $GROUP_DIR/auto-lang
                 git -C "$DOWN_MAIN" worktree remove $GROUP_DIR/auto-down
                 rmdir $GROUP_DIR
红线：组内禁止 junction/symlink；跨仓依赖走 env → 组内 ../auto-down → 主检出解析序。
EOF
