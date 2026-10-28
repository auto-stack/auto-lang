#!/usr/bin/env bash
# measure-build-cost-873.sh — PLAN-873 (THR-D1) 构建成本样本运行器（M873 协议 §8 骨架的受管实现）
#
# 用法:
#   bash scripts/measure-build-cost-873.sh <sample-id> <repo-dir> <target-dir> <cold|hot|keep> <incr0|incr1> -- <cargo 命令...>
#
#   cold  = 运行前清空 <target-dir>（仅测量拥有目录；绝不清共享/主检出路径）
#   hot   = 保留 target 原样（同输入热跑）
#   keep  = 保留 target，且不额外设置 CARGO_INCREMENTAL（显式传环境时用）
#   incr0 = CARGO_INCREMENTAL=0；incr1 = CARGO_INCREMENTAL=1
#
# 行为（M873 协议 §5/§8）:
#   - 显式 unset RUSTC_WRAPPER（禁用共享 sccache；协议 D-1：绝不读写 D:\autostack\.sccache）
#   - CARGO_TARGET_DIR 显式指向 .measure/target-*（target 不落树内）
#   - 日志 = .measure/logs/<sample-id>.log，头部含样本id/时间/pin/命令/env，尾部含墙钟秒与退出码
#   - 退出码透传 cargo 的退出码（污染/失败样本据实记录，不静默丢弃）
set -uo pipefail

SAMPLE_ID="$1"; REPO_DIR="$2"; TARGET_DIR="$3"; MODE="$4"; INCR="$5"; shift 5
[ "${1:-}" = "--" ] && shift

LOGS_DIR="$(dirname "$TARGET_DIR")/logs"
LOG="$LOGS_DIR/$SAMPLE_ID.log"
mkdir -p "$LOGS_DIR"

# 冷样本：只清测量拥有的 target 目录
if [ "$MODE" = "cold" ]; then
  rm -rf "${TARGET_DIR:?}"/*
fi

INCR_VAL=0; [ "$INCR" = "incr1" ] && INCR_VAL=1

{
  echo "sample_id=$SAMPLE_ID"
  echo "datetime=$(date '+%Y-%m-%d %H:%M:%S')"
  echo "repo=$REPO_DIR"
  echo "pin=$(git -C "$REPO_DIR" rev-parse HEAD 2>/dev/null)"
  echo "branch=$(git -C "$REPO_DIR" branch --show-current 2>/dev/null)"
  echo "mode=$MODE incremental=$INCR_VAL"
  echo "env: RUSTC_WRAPPER=<unset> CARGO_INCREMENTAL=$INCR_VAL CARGO_TARGET_DIR=$TARGET_DIR"
  echo "cmd: cargo $*"
  echo "start=$(date '+%H:%M:%S.%3N')"
} > "$LOG"

START=$(date +%s.%N)
(
  cd "$REPO_DIR" || exit 97
  exec env -u RUSTC_WRAPPER -u SCCACHE_DIR \
    CARGO_INCREMENTAL=$INCR_VAL \
    CARGO_TARGET_DIR=$TARGET_DIR \
    cargo "$@"
) >> "$LOG" 2>&1
RC=$?
END=$(date +%s.%N)
WALL=$(awk -v e="$END" -v s="$START" 'BEGIN{printf "%.1f", e-s}')

{
  echo "end=$(date '+%H:%M:%S.%3N')"
  echo "wall_seconds=$WALL"
  echo "exit=$RC"
} >> "$LOG"

echo "[$SAMPLE_ID] wall=${WALL}s exit=$RC"
exit $RC
