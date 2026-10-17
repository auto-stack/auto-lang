#!/usr/bin/env bash
# PLAN-806 V01 dependency-closure oracle: the UI declarations grammar
# implementation's normal/build/dev closure must NOT contain auto-lang /
# auto-man / auto-vm / auto-ui (the grammar lives in a light crate over the
# dialect port; mirrors dialect-api-consumer verify-deps.sh).
#
# Usage: bash test/fixtures/modularity/ui-declarations-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

# cargo tree 只做依赖图解析，不需要 sccache；本机多会话并行时 sccache
# 端口冲突会让 rustc -vV 探测失败（801 复审在案），显式禁用 wrapper。
export RUSTC_WRAPPER=""

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build,dev 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-806/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-lang|auto-man|auto-vm|auto-ui) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the ui-declarations-consumer dependency closure:"
  grep -E "(auto-lang|auto-man|auto-vm|auto-ui) v" <<<"$TREE"
  exit 1
fi

if ! grep -q "auto-ui-dialect v" <<<"$TREE"; then
  echo "FAIL: auto-ui-dialect missing from the closure (fixture broken?)"
  exit 1
fi

echo "PASS: UI declarations grammar closure is language-implementation-free"
echo "$TREE"
