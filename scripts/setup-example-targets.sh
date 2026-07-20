#!/bin/bash
# Configure example Rust projects to share the workspace target directory.
# This avoids each example compiling its own copy of auto-lang and other deps.
#
# Usage: bash scripts/setup-example-targets.sh

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_REL="target"

echo "Setting up shared target directory for examples..."
echo "Repo root: $REPO_ROOT"

# examples/ui/*/gen/rust
for d in "$REPO_ROOT"/examples/ui/*/gen/rust; do
    [ -d "$d" ] || continue
    mkdir -p "$d/.cargo"
    cat > "$d/.cargo/config.toml" <<'EOF'
[build]
target-dir = "../../../../../target"
EOF
    echo "  $d"
done

# Other example projects with their own target/
for d in \
    "$REPO_ROOT"/examples/api-example/rust \
    "$REPO_ROOT"/examples/component-gallery/vue/src-tauri \
    "$REPO_ROOT"/examples/unified-demo/vue/src-tauri
do
    [ -d "$d" ] || continue
    mkdir -p "$d/.cargo"
    cat > "$d/.cargo/config.toml" <<'EOF'
[build]
target-dir = "../../../../target"
EOF
    echo "  $d"
done

echo "Done. Examples now share the workspace target directory."
