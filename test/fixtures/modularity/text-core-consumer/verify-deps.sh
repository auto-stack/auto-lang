#!/usr/bin/env bash
# PLAN-807 V-06 dependency-closure oracle: the external consumer's
# normal/build closure must NOT contain auto-lang / auto-man / auto-ui /
# auto-vm (the text foundation is interpreter/renderer-free by construction;
# this pins it against regressions).
#
# Usage: bash test/fixtures/modularity/text-core-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-807/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-lang|auto-man|auto-ui|auto-vm) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the text-core-consumer dependency closure:"
  grep -E "(auto-lang|auto-man|auto-ui|auto-vm) v" <<<"$TREE"
  exit 1
fi

if ! grep -q "auto-text-core v" <<<"$TREE"; then
  echo "FAIL: auto-text-core missing from the closure (fixture broken?)"
  exit 1
fi

echo "PASS: auto-text-core closure is language/UI/VM-free"
echo "$TREE"
