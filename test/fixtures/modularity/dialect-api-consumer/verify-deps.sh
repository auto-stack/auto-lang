#!/usr/bin/env bash
# PLAN-801 V01 dependency-closure oracle: the external grammar implementation's
# normal/build closure must NOT contain auto-lang / auto-man / auto-vm / auto-ui
# (the dialect port API inverts the dependency; this pins it against
# regressions — mirrors gen-core/package-lock/package-resolver verify-deps.sh).
#
# Usage: bash test/fixtures/modularity/dialect-api-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-801/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-lang|auto-man|auto-vm|auto-ui) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the dialect-api-consumer dependency closure:"
  grep -E "(auto-lang|auto-man|auto-vm|auto-ui) v" <<<"$TREE"
  exit 1
fi

if ! grep -q "auto-parser-api v" <<<"$TREE"; then
  echo "FAIL: auto-parser-api missing from the closure (fixture broken?)"
  exit 1
fi

echo "PASS: external grammar implementation closure is language-implementation-free"
echo "$TREE"
