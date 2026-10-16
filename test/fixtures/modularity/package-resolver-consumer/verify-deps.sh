#!/usr/bin/env bash
# PLAN-800 V01 dependency-closure oracle: the external consumer's normal/build
# closure must NOT contain auto-lang / auto-ast / auto-man / auto-gen /
# auto-ui / auto-vm / auto-val (the resolver core is std-only and language/
# AST/interpreter/renderer-free by construction; this pins it against
# regressions).
#
# Usage: bash test/fixtures/modularity/package-resolver-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-800/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-lang|auto-ast|auto-man|auto-gen|auto-ui|auto-vm|auto-val) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the package-resolver-consumer dependency closure:"
  grep -E "(auto-lang|auto-ast|auto-man|auto-gen|auto-ui|auto-vm|auto-val) v" <<<"$TREE"
  exit 1
fi

if ! grep -q "auto-package-resolver v" <<<"$TREE"; then
  echo "FAIL: auto-package-resolver missing from the closure (fixture broken?)"
  exit 1
fi

echo "PASS: auto-package-resolver closure is language/AST/Man/VM/UI-free"
echo "$TREE"
