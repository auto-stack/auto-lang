#!/usr/bin/env bash
# PLAN-809 T-06 V-01 dependency-closure oracle: the external consumer's
# normal/build closure must NOT contain auto-lang / auto-ast / auto-man /
# auto-gen / auto-ui / auto-vm (the management rules core is
# auto-atom/auto-val + std only by construction; this pins it against
# regressions). auto-val IS a legal member of this closure (the request's
# data layer — Value/AutoStr are used as-is, no conversion copies).
#
# Usage: bash test/fixtures/modularity/man-core-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-809/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-lang|auto-ast|auto-man|auto-gen|auto-ui|auto-vm) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the man-core-consumer dependency closure:"
  grep -E "(auto-lang|auto-ast|auto-man|auto-gen|auto-ui|auto-vm) v" <<<"$TREE"
  exit 1
fi

if ! grep -q "auto-man-core v" <<<"$TREE"; then
  echo "FAIL: auto-man-core missing from the closure (fixture broken?)"
  exit 1
fi

echo "PASS: auto-man-core closure is language/AST/Man/VM/UI-free (auto-val allowed)"
echo "$TREE"
