#!/usr/bin/env bash
# PLAN-815 T-07/V-01 dependency-closure oracle: the pure protocol consumer's
# normal/build closure must contain ONLY the data-domain workspace crates
# (auto-tool-protocol + auto-project-model) and NO other auto-* crate —
# the protocol is pure data by contract (data-stays-bottom).
#
# PLAN-815 T-06..08 R1（F-815-T07-R1-03）重写：deny-by-default——旧脚本
# 硬编码少数包名，对实际高层实现包（auto-ui-dialect=UI 文法实现、
# auto-bindgen=777 标记 lang-infra）漏判，且 `auto-x `+` v` 双空格模式连
# 列表内的包都匹配不上。新 oracle 对 **workspace 内全部 auto-* 依赖**
# 默认拒绝、白名单精确两包；非 auto-* 的外部基础 crate（serde 等）不经
# 本门（fixture manifest 无法引入仓内实现包以外的越界）。
#
# Usage: bash test/fixtures/modularity/tool-protocol-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

# Dependency names (normal+build edges), one per line, deduplicated.
TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build --prefix none 2>/dev/null \
  | awk '{print $1}' | sort -u)"

# Every auto-* crate in the closure must be on the explicit allow list.
ALLOWED="auto-tool-protocol auto-project-model auto-val auto-atom"
VIOLATIONS="$(while read -r crate; do
  case "$crate" in
    auto-*) ;;
    *) continue ;;
  esac
  case " $ALLOWED " in
    *" $crate "*) ;;
    *) echo "$crate" ;;
  esac
done <<<"$TREE")"

if [ -n "$VIOLATIONS" ]; then
  echo "FAIL: forbidden workspace crates in the tool-protocol-consumer dependency closure:"
  echo "$VIOLATIONS"
  echo "(allowed auto-* crates: $ALLOWED)"
  exit 1
fi

for required in auto-tool-protocol auto-project-model; do
  if ! grep -qx "$required" <<<"$TREE"; then
    echo "FAIL: $required missing from the closure (fixture broken?)"
    exit 1
  fi
done

echo "PASS: auto-tool-protocol closure is pure data (deny-by-default; only $ALLOWED)"
