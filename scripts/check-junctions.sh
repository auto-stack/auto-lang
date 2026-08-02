#!/usr/bin/env bash
# check-junctions.sh — 扫描目标目录及其父目录链上的 junction/symlink
#
# 用途：在 D:/autostack/ 下执行任何删除/清空/镜像操作前，先运行本脚本
# 确认目标不含或未被 junction 指向。如果输出任何 junction/symlink，
# 停止你的操作，改用安全方案（见 SAFETY.md）。
#
# 用法：bash check-junctions.sh <目标目录>
#   退出码 0 = 未发现 junction（相对安全）
#   退出码 1 = 发现 junction（危险，停止破坏性操作）
#
# 详见 D:/autostack/auto-ai/SAFETY.md

set -euo pipefail

if [ $# -lt 1 ]; then
    echo "用法: $0 <目标目录>" >&2
    echo "  扫描该目录及其父目录链上的 junction/symlink" >&2
    exit 2
fi

TARGET="$(cd "$1" && pwd 2>/dev/null)" || TARGET="$1"
FOUND=0

echo "=== junction/symlink 预检 ==="
echo "目标: $TARGET"
echo

# 1. 检查从 D:/autostack 到目标路径的每一级父目录
#    （junction 可能在路径上的任何一级）
current="$TARGET"
while true; do
    # 检查 current 是否本身是 symlink
    if [ -L "$current" ]; then
        link_target=$(readlink "$current")
        echo "⚠️  [symlink] $current"
        echo "    → $link_target"
        FOUND=1
    fi
    # 父目录链终止条件
    parent="$(dirname "$current")"
    [ "$parent" = "$current" ] && break
    # 到达 autostack 根就停（不检查 autostack 之上）
    case "$parent" in
        */autostack) break ;;
    esac
    current="$parent"
done

# 2. 扫描目标目录内部（深度2）的 symlink（Git Bash 可见）
#    junction 在 Git Bash 里不显示为 -type l，需用 cmd 确认（见下）
echo "--- 目标内部 symlink（深度2）---"
internal_links=$(find "$TARGET" -maxdepth 2 -type l 2>/dev/null || true)
if [ -n "$internal_links" ]; then
    while IFS= read -r l; do
        echo "⚠️  [symlink] $l"
        echo "    → $(readlink "$l" 2>/dev/null || echo '?')"
        FOUND=1
    done <<< "$internal_links"
else
    echo "  (无 symlink)"
fi
echo

# 3. 用 Windows cmd 扫描 junction（Git Bash 看不到 junction，cmd 能看到）
#    这是关键——本次事故的循环 junction 只有 cmd 的 dir /AL 能发现
echo "--- 目标内部 junction（Windows dir /AL）---"
# 扫描目标自身 + 深度1子目录的 junction
junction_output=$(cmd //c "dir /AL /B \"$(cygpath -w "$TARGET" 2>/dev/null || echo "$TARGET")\"" 2>/dev/null || true)
if [ -n "$junction_output" ]; then
    while IFS= read -r j; do
        [ -z "$j" ] && continue
        full_path="$TARGET/$j"
        # 确认是 junction（不是普通目录）
        jtype=$(cmd //c "dir /AL \"$(cygpath -w "$full_path" 2>/dev/null || echo "$full_path")\"" 2>/dev/null | grep -i "JUNCTION" || true)
        if [ -n "$jtype" ]; then
            target_of=$(echo "$jtype" | sed 's/.*JUNCTION[ ]*//' | tr -d '\r')
            echo "⚠️  [JUNCTION] $full_path"
            echo "    → $target_of"
            FOUND=1
        fi
    done <<< "$junction_output"
fi

# 同时检查 .worktrees 下各 worktree 是否含同名仓库 junction
if [ -d "$TARGET/.worktrees" ]; then
    echo "--- .worktrees 下的 junction（高危区）---"
    for wt in "$TARGET"/.worktrees/*/; do
        [ -d "$wt" ] || continue
        wt_junctions=$(cmd //c "dir /AL /B \"$(cygpath -w "$wt" 2>/dev/null || echo "$wt")\"" 2>/dev/null || true)
        if [ -n "$wt_junctions" ]; then
            while IFS= read -r j; do
                [ -z "$j" ] && continue
                echo "⚠️  [JUNCTION in worktree] ${wt}${j}"
                FOUND=1
            done <<< "$wt_junctions"
        fi
    done
fi

echo
if [ "$FOUND" = "1" ]; then
    echo "❌ 发现 junction/symlink — 停止任何删除/镜像/清空操作！"
    echo "   参见 SAFETY.md §2（永久禁令）和 §4（worktree 安全清理）"
    exit 1
else
    echo "✅ 未发现 junction/symlink（相对安全）"
    echo "   注意：仍应避免 rm -rf 等操作，优先用 git 原生命令"
    exit 0
fi
