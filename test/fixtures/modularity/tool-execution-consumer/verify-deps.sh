#!/usr/bin/env bash
# PLAN-816 T-07：auto-tool-execution 闭包钉——仅 std（无 workspace crate、
# 无三方依赖）。usage: bash verify-deps.sh
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
tree="$(cargo tree --manifest-path "$here/Cargo.toml" --edges normal 2>/dev/null)"
bad="$(printf '%s\n' "$tree" | grep -E '^[├└┬│ a-z0-9_-]+ (auto-lang|auto-man|auto-ui|auto-vm|auto-gen|auto-val|auto-tool-protocol|auto-project|auto-package|auto-toolchain|shell-pack|iced|tokio|serde|clap|miette|colored|notify)' || true)"
if [ -n "$bad" ]; then
  echo "FAIL: auto-tool-execution closure is not std-only:"
  printf '%s\n' "$bad"
  exit 1
fi
echo "PASS: auto-tool-execution closure is std-only"
