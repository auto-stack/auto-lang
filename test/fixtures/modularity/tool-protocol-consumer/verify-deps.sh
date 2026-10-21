#!/usr/bin/env bash
# PLAN-815 T-07/V-01 dependency-closure oracle: the pure protocol consumer's
# normal/build closure must contain only the data-domain crates
# (auto-tool-protocol + auto-project-model) and must NOT contain any UI,
# runtime, compiler, Man, or language implementation crate — the protocol
# is pure data by contract (data-stays-bottom).
#
# Usage: bash test/fixtures/modularity/tool-protocol-consumer/verify-deps.sh
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"

TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build 2>/dev/null)"

if grep -Eq "(auto-ui-tools|auto-ui-compiler|auto-ui-runtime|auto-lang |auto-man |auto-man-core|auto-api-gen|auto-vm |auto-gen |auto-media-core) v" <<<"$TREE"; then
  echo "FAIL: forbidden crates in the tool-protocol-consumer dependency closure:"
  grep -E "(auto-ui-tools|auto-ui-compiler|auto-ui-runtime|auto-lang |auto-man |auto-man-core|auto-api-gen|auto-vm |auto-gen |auto-media-core) v" <<<"$TREE"
  exit 1
fi

for required in auto-tool-protocol auto-project-model; do
  if ! grep -q "$required v" <<<"$TREE"; then
    echo "FAIL: $required missing from the closure (fixture broken?)"
    exit 1
  fi
done

echo "PASS: auto-tool-protocol closure is pure data (UI/runtime/compiler/Man-free)"
