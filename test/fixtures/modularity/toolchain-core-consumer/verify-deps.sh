#!/usr/bin/env bash
# PLAN-805 V01 dependency-closure oracle: the external consumer's normal/build
# closure must NOT contain auto-lang / auto-ast / auto-man / auto-gen /
# auto-ui / auto-vm (the toolchain rules core is auto-val + std only and
# language/AST/interpreter/renderer/process-free by construction; this pins it
# against regressions). auto-val IS a legal member of this closure (the rules
# model's data layer — AutoStr/AutoPath are used as-is, no conversion copies).
#
# Usage: bash test/fixtures/modularity/toolchain-core-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-805/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-lang|auto-ast|auto-man|auto-gen|auto-ui|auto-vm) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the toolchain-core-consumer dependency closure:"
  grep -E "(auto-lang|auto-ast|auto-man|auto-gen|auto-ui|auto-vm) v" <<<"$TREE"
  exit 1
fi

if ! grep -q "auto-toolchain-core v" <<<"$TREE"; then
  echo "FAIL: auto-toolchain-core missing from the closure (fixture broken?)"
  exit 1
fi

echo "PASS: auto-toolchain-core closure is language/AST/Man/VM/UI-free (auto-val allowed)"
echo "$TREE"
