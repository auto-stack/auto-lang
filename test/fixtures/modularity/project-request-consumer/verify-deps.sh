#!/usr/bin/env bash
# PLAN-809 T-02 V-02 dependency-closure oracle: the external consumer's
# normal/build closure must NOT contain auto-lang / auto-ast / auto-man /
# auto-gen / auto-ui / auto-vm (the generation request contract is
# auto-atom/auto-val + std only by construction; this pins it against
# regressions). auto-val IS a legal member of this closure (the request's
# data layer — Value/AutoStr are used as-is, no conversion copies).
#
# Usage: bash test/fixtures/modularity/project-request-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-809/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-lang|auto-ast|auto-man|auto-gen|auto-ui|auto-vm) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the project-request-consumer dependency closure:"
  grep -E "(auto-lang|auto-ast|auto-man|auto-gen|auto-ui|auto-vm) v" <<<"$TREE"
  exit 1
fi

if ! grep -q "auto-project-model v" <<<"$TREE"; then
  echo "FAIL: auto-project-model missing from the closure (fixture broken?)"
  exit 1
fi

echo "PASS: auto-project-model closure is language/AST/Man/VM/UI-free (auto-val allowed)"
echo "$TREE"
