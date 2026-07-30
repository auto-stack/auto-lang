#!/usr/bin/env bash
# Plan 376: re-transpile ALL .at → a2r → assemble into rust/src/
#
# Assembly rules (mirrors lib.rs hoisting):
#   src/X.at                 → rust/src/X.rs
#   src/orchestration/X.at   → rust/src/X.rs        (hoisted to root)
#       mod.at               → rust/src/orchestration.rs (aggregator)
#   src/builtin_roles/X.at   → rust/src/builtin_role_X.rs
#       mod.at               → rust/src/builtin_roles.rs (aggregator)
#   src/config/X.at          → rust/src/X.rs
#       mod.at               → rust/src/config.rs (aggregator)
#
# Hand-written glue files (NOT overwritten, have no .at source):
#   client_impl.rs, echo_tool.rs, main.rs, lib.rs
#
# Usage: ./retranspile.sh [check]
#   (no arg)  transpile + assemble, leave rust/src/ modified
#   check     after assembling, run cargo check and report error count
set -euo pipefail

AUTO="${AUTO:-../../target/debug/auto.exe}"
AGENT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$AGENT_DIR/src"
RUST="$AGENT_DIR/rust/src"

echo "[retranspile] transpiling all .at files..."
# Transpile every .at file (writes <name>.a2r.rs next to it)
while IFS= read -r f; do
    "$AUTO" trans --path "$f" rust >/dev/null 2>&1 || true
done < <(find "$SRC" -name "*.at")

echo "[retranspile] assembling into rust/src/ ..."

# lib.rs is hand-written (contains extern-crate shims + module declarations);
# the transpiled lib.a2r.rs only has `use` re-exports — do NOT overwrite it.
# echo_tool.rs and client_impl.rs are also hand-written (no .at source).

# Helper: copy a2r.rs → dest.rs only if the a2r.rs exists (transpile succeeded).
# If a file failed to transpile, keep the existing (hand-assembled) dest.rs.
copy_if_exists() {
    local src="$1" dst="$2"
    if [ -f "$src" ]; then
        cp "$src" "$dst"
    else
        echo "  [skip] $(basename "$src" .a2r.rs).at failed to transpile — keeping existing $(basename "$dst")"
    fi
}

# Flat src/*.at → rust/src/<name>.rs  (skip lib.at — keep hand-written lib.rs)
for f in "$SRC"/*.at; do
    bn=$(basename "$f" .at)
    [ "$bn" = "lib" ] && continue
    copy_if_exists "$SRC/${bn}.a2r.rs" "$RUST/${bn}.rs"
done

# orchestration/*.at → rust/src/<name>.rs (hoisted), mod.at → orchestration.rs
for f in "$SRC"/orchestration/*.at; do
    bn=$(basename "$f" .at)
    if [ "$bn" = "mod" ]; then
        copy_if_exists "$SRC/orchestration/mod.a2r.rs" "$RUST/orchestration.rs"
    else
        copy_if_exists "$SRC/orchestration/${bn}.a2r.rs" "$RUST/${bn}.rs"
    fi
done

# config/*.at → rust/src/<name>.rs, mod.at → config.rs
for f in "$SRC"/config/*.at; do
    bn=$(basename "$f" .at)
    if [ "$bn" = "mod" ]; then
        copy_if_exists "$SRC/config/mod.a2r.rs" "$RUST/config.rs"
    else
        copy_if_exists "$SRC/config/${bn}.a2r.rs" "$RUST/${bn}.rs"
    fi
done

# builtin_roles/*.at → rust/src/builtin_role_<name>.rs, mod.at → builtin_roles.rs
for f in "$SRC"/builtin_roles/*.at; do
    bn=$(basename "$f" .at)
    if [ "$bn" = "mod" ]; then
        copy_if_exists "$SRC/builtin_roles/mod.a2r.rs" "$RUST/builtin_roles.rs"
    else
        copy_if_exists "$SRC/builtin_roles/${bn}.a2r.rs" "$RUST/builtin_role_${bn}.rs"
    fi
done

# Clean up .a2r.rs intermediates
find "$SRC" -name "*.a2r.rs" -delete

echo "[retranspile] assembly complete."

if [ "${1:-}" = "check" ]; then
    echo "[retranspile] running cargo check..."
    cd "$RUST/.."
    n=$(cargo check --color never 2>&1 | grep -cE "^error" || true)
    echo "[retranspile] error count: $n"
fi
