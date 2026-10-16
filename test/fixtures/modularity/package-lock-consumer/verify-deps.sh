#!/usr/bin/env bash
# PLAN-795 V01 dependency-closure oracle: the external consumer's normal/build
# closure must NOT contain auto-lang / auto-man / auto-gen / auto-ui / auto-vm
# (the pac.lock core is Target-free and interpreter/renderer-free by
# construction; this pins it against regressions).
#
# Usage: bash test/fixtures/modularity/package-lock-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-795/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-lang|auto-man|auto-gen|auto-ui|auto-vm) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the package-lock-consumer dependency closure:"
  grep -E "(auto-lang|auto-man|auto-gen|auto-ui|auto-vm) v" <<<"$TREE"
  exit 1
fi

if ! grep -q "auto-package-lock v" <<<"$TREE"; then
  echo "FAIL: auto-package-lock missing from the closure (fixture broken?)"
  exit 1
fi

echo "PASS: auto-package-lock closure is Target/interpreter/renderer-free"
echo "$TREE"
