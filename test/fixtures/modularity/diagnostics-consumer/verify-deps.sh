#!/usr/bin/env bash
# PLAN-793 V01 dependency-closure oracle: the external consumer's normal/build
# closure must NOT contain auto-lang / auto-man / auto-vm / auto-ui (the
# diagnostics foundation is AST/parser/VM-free by construction; this pins it
# against regressions).
#
# Usage: bash test/fixtures/modularity/diagnostics-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-793/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-lang|auto-man|auto-vm|auto-ui) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the diagnostics-consumer dependency closure:"
  grep -E "(auto-lang|auto-man|auto-vm|auto-ui) v" <<<"$TREE"
  exit 1
fi

for required in auto-diagnostics auto-lex auto-val; do
  if ! grep -q "$required v" <<<"$TREE"; then
    echo "FAIL: $required missing from the closure (fixture broken?)"
    exit 1
  fi
done

echo "PASS: auto-diagnostics closure has no auto-lang/auto-man/auto-vm/auto-ui"
echo "$TREE"
