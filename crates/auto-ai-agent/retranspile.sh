#!/usr/bin/env bash
# Plan 376U: re-transpile ALL .at → a2r → assemble into rust/src/
#
# Crate-root files (lib.at, */mod.at) are transpiled with A2R_CRATE_ROOT=1,
# which makes a2r emit `pub use` (re-exports) + `#![allow(...)]` pragma.
# After transpilation, this script INJECTS the assembly-layer scaffolding
# a2r cannot know about:
#   - extern-crate shims (pub mod auto_ai_client { pub use ::auto_ai_client::*; })
#   - `pub mod X;` declarations for hoisted modules
#
# Assembly rules (hoisting):
#   src/X.at                 → rust/src/X.rs
#   src/lib.at               → rust/src/lib.rs       (crate root + injected shims/mods)
#   src/orchestration/X.at   → rust/src/X.rs         (hoisted to root)
#       mod.at               → rust/src/orchestration.rs (crate-root aggregator)
#   src/builtin_roles/X.at   → rust/src/builtin_role_X.rs
#       mod.at               → rust/src/builtin_roles.rs (crate-root aggregator)
#   src/config/X.at          → rust/src/X.rs
#       mod.at               → rust/src/config.rs    (crate-root aggregator)
#
# Hand-written glue files (NOT overwritten, have no .at source):
#   client_impl.rs, echo_tool.rs, main.rs
#
# Usage: ./retranspile.sh [check]
#   (no arg)  transpile + assemble, leave rust/src/ modified
#   check     after assembling, run cargo check and report error count
set -euo pipefail

AUTO="${AUTO:-../../target/debug/auto.exe}"
AGENT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC="$AGENT_DIR/src"
RUST="$AGENT_DIR/rust/src"

# ── extern-crate shims (a2r emits `use crate::<these>::...`, needs a shim) ──
# wire is special: it re-exports ai_config::wire + adds the JsonValue alias.
read_shims() {
    cat <<'SHIMS'
// ── extern-crate shims (a2r emits `use crate::<these>::...`) ────────────────
pub mod auto_ai_client {
    pub use ::auto_ai_client::*;
}
pub mod ai_config {
    pub use ::ai_config::*;
}
pub mod wire {
    pub use ::ai_config::wire::*;
    // a2r references `crate::wire::JsonValue` (generic JSON blob = serde_json::Value).
    pub type JsonValue = serde_json::Value;
}

// ── hand-written glue (no .at source) ──────────────────────────────────────
pub mod client_impl;

SHIMS
}

# ── pub mod declarations for every module file in rust/src/ (hoisted flat) ──
read_pub_mods() {
    # Every .rs file except lib.rs/main.rs/client_impl.rs/echo_tool.rs gets a
    # `pub mod <stem>;` declaration. lib.rs is this file; main/client_impl/echo
    # are hand-written glue declared above or are the binary entry.
    for f in "$RUST"/*.rs; do
        local stem
        stem=$(basename "$f" .rs)
        case "$stem" in
            lib|main|client_impl|echo_tool) continue ;;
        esac
        echo "pub mod ${stem};"
    done
}

echo "[retranspile] transpiling all .at files..."
# Transpile every .at file. Crate-root files (lib.at, */mod.at) get
# A2R_CRATE_ROOT=1 so a2r emits pub use + #![allow].
transpile_one() {
    local f="$1"
    local base
    base=$(basename "$f" .at)
    local crate_root=0
    if [ "$base" = "lib" ] || [ "$base" = "mod" ]; then
        crate_root=1
    fi
    A2R_CRATE_ROOT="$crate_root" "$AUTO" trans --path "$f" rust >/dev/null 2>&1 || true
}
while IFS= read -r f; do
    transpile_one "$f"
done < <(find "$SRC" -name "*.at")

echo "[retranspile] assembling into rust/src/ ..."

# Helper: copy a2r.rs → dest.rs only if the a2r.rs exists (transpile succeeded).
copy_if_exists() {
    local src="$1" dst="$2"
    if [ -f "$src" ]; then
        cp "$src" "$dst"
    else
        echo "  [skip] $(basename "$src" .a2r.rs).at failed to transpile — keeping existing $(basename "$dst")"
    fi
}

# Flat src/*.at → rust/src/<name>.rs  (lib.at handled separately below)
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

# ── Assemble lib.rs: transpiled crate-root + injected shims + pub mod decls ─
# The transpiled lib.a2r.rs has the #![allow] pragma + pub use re-exports.
# We inject (after #![allow], before pub use): extern shims + pub mod decls.
if [ -f "$SRC/lib.a2r.rs" ]; then
    awk -v shims="$(read_shims)" -v pubmods="$(read_pub_mods)" '
        /^#!\[allow/ { print; print ""; print shims; print pubmods; next }
        { print }
    ' "$SRC/lib.a2r.rs" > "$RUST/lib.rs"
    echo "  [lib] assembled lib.rs (crate-root transpile + shims + pub mod decls)"
else
    echo "  [skip] lib.at failed to transpile — keeping existing lib.rs"
fi

# Clean up .a2r.rs intermediates
find "$SRC" -name "*.a2r.rs" -delete

echo "[retranspile] assembly complete."

if [ "${1:-}" = "check" ]; then
    echo "[retranspile] running cargo check..."
    cd "$RUST/.."
    n=$(cargo check --color never 2>&1 | grep -cE "^error" || true)
    echo "[retranspile] error count: $n"
fi
