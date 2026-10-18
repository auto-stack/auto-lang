#!/usr/bin/env bash
# PLAN-808 V-06 dependency-closure oracle: the external consumer's
# normal/build closure must NOT contain auto-lang / auto-man / auto-vm /
# auto-ui / auto-term (the terminal core is a non-UI runtime crate by
# construction; this pins it against regressions) and MUST contain
# auto-terminal-core itself.
#
# Usage: bash test/fixtures/modularity/terminal-core-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# are not false positives.
if grep -Eq "(auto-lang|auto-man|auto-vm|auto-ui|auto-term) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the terminal-core-consumer closure:"
  grep -E "(auto-lang|auto-man|auto-vm|auto-ui|auto-term) v" <<<"$TREE"
  exit 1
fi

if ! grep -q "auto-terminal-core v" <<<"$TREE"; then
  echo "FAIL: auto-terminal-core missing from the closure (fixture broken?)"
  exit 1
fi

echo "PASS: auto-terminal-core closure is UI/language/man/auto-term free"
echo "$TREE"
