#!/bin/bash
# Remove build artifacts from example projects to free disk space.
#
# Note: examples are independent workspaces (see `[workspace]` in their
# Cargo.toml). Because their package paths differ from the main workspace,
# sharing a single target directory does NOT reduce disk usage — Cargo treats
# them as separate compilation graphs. The only reliable way to reclaim space
# is to remove their `target/` directories when they are not being actively
# developed.
#
# For examples/ui sub-projects that should share dependencies, use the
# external workspace at `D:\.auto\rust-workspace` instead of building them
# in-place under this repo.
#
# Usage: bash scripts/clean-example-targets.sh

set -e

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "Cleaning example target directories..."

for d in \
    "$REPO_ROOT"/examples/api-example/rust/target \
    "$REPO_ROOT"/examples/component-gallery/vue/src-tauri/target \
    "$REPO_ROOT"/examples/unified-demo/vue/src-tauri/target \
    "$REPO_ROOT"/examples/ui/*/gen/rust/target
do
    if [ -d "$d" ]; then
        echo "  removing $d"
        rm -rf "$d"
    fi
done

echo "Done."
