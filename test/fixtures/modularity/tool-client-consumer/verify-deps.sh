#!/usr/bin/env bash
# PLAN-816 V-05 dependency-closure oracle: the external I-TOOL client
# consumer's normal/build closure must contain the client stack
# (auto-tool-client + auto-tool-protocol + auto-tool-execution +
# auto-project-model) and must NOT contain auto-ui-tools (the tool host is
# consumed as a real process only — never statically linked) nor
# auto-man / auto-lang / auto-vm / auto-gen / auto-media-core (a Core
# install without the UI stack can consume the tool).
#
# Usage: bash test/fixtures/modularity/tool-client-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

# Match on "<crate> v" so checkout paths containing the repo directory name
# (e.g. .../.wt/lang-816/auto-lang/crates/...) are not false positives.
if grep -Eq "(auto-ui-tools|auto-lang|auto-man|auto-vm|auto-gen|auto-media-core) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the tool-client-consumer dependency closure:"
  grep -E "(auto-ui-tools|auto-lang|auto-man|auto-vm|auto-gen|auto-media-core) v" <<<"$TREE"
  exit 1
fi

for required in auto-tool-client auto-tool-protocol auto-tool-execution auto-project-model; do
  if ! grep -q "$required v" <<<"$TREE"; then
    echo "FAIL: $required missing from the closure (fixture broken?)"
    exit 1
  fi
done

echo "PASS: auto-tool-client closure is tool-host-static-free and Man/language/VM-free"
echo "$TREE"
