#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
TREE="$(cd "$HERE" && cargo tree --workspace --edges normal,build --prefix none 2>/dev/null | sed 's/ (\*)$//' | sort -u)"

if grep -Eq "^(auto-lang|auto-man|auto-ui[^ ]*|auto-vm|auto-gen) v" <<<"$TREE"; then
  echo "FAIL: UI, compatibility, or duplicate implementation in three-repository consumer closure"
  grep -E "^(auto-lang|auto-man|auto-ui[^ ]*|auto-vm|auto-gen) v" <<<"$TREE"
  exit 1
fi
for required in auto-lang-core auto-man-core auto-tool-protocol auto-project-model; do
  count="$(grep -Ec "^${required} v" <<<"$TREE")"
  if [ "$count" -ne 1 ]; then
    echo "FAIL: ${required} source count=${count}, expected exactly one"
    exit 1
  fi
done
if grep -Eq "auto-ui|iced" <<<"$TREE"; then
  echo "FAIL: UI implementation found in Core consumer closure"
  exit 1
fi
echo "PASS: management + SDK + protocol consumer has one source and no UI implementation"
