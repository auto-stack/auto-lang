#!/usr/bin/env bash
# PLAN-815 V-01(A 范围)/V-05 dependency-closure oracle: the external
# consumer's normal/build closure must contain the UI tool + protocol +
# project model and must NOT contain auto-man / auto-lang / auto-vm /
# auto-gen / auto-media-core (the M815-A tool host is interpreter/Man-free
# by construction; this pins it against regressions).
#
# Usage: bash test/fixtures/modularity/ui-tool-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-815/auto-lang/crates/...) are not false positives.
# NOTE: auto-ui-tools is REQUIRED and would match a naive "auto-ui" prefix
# pattern — the forbidden list uses exact crate names via word boundaries.
if grep -Eq "(auto-lang|auto-man|auto-vm|auto-gen|auto-media-core) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the ui-tool-consumer dependency closure:"
  grep -E "(auto-lang|auto-man|auto-vm|auto-gen|auto-media-core) v" <<<"$TREE"
  exit 1
fi

for required in auto-ui-tools auto-tool-protocol auto-project-model; do
  if ! grep -q "$required v" <<<"$TREE"; then
    echo "FAIL: $required missing from the closure (fixture broken?)"
    exit 1
  fi
done

echo "PASS: auto-ui-tools closure is Man/language/VM-free"
echo "$TREE"
